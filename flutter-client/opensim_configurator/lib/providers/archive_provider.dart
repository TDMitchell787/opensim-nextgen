import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../models/archive_models.dart';
import '../services/archive_service.dart';

class ArchiveProvider extends ChangeNotifier {
  ArchiveService? _service;
  WebSocketChannel? _wsChannel;
  StreamSubscription? _wsSubscription;

  List<ArchiveJob> _jobs = [];
  ArchiveJob? _selectedJob;
  bool _isLoading = false;
  String? _errorMessage;
  int _activeJobCount = 0;

  final Map<String, ArchiveJob> _activeJobs = {};

  List<ArchiveJob> get jobs => _jobs;
  ArchiveJob? get selectedJob => _selectedJob;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  int get activeJobCount => _activeJobCount;
  Map<String, ArchiveJob> get activeJobs => _activeJobs;

  List<ArchiveJob> get pendingJobs =>
      _jobs.where((j) => j.status == JobStatus.queued).toList();
  List<ArchiveJob> get runningJobs =>
      _jobs.where((j) => j.status == JobStatus.running).toList();
  List<ArchiveJob> get completedJobs =>
      _jobs.where((j) => j.status == JobStatus.completed).toList();
  List<ArchiveJob> get failedJobs =>
      _jobs.where((j) => j.status == JobStatus.failed || j.status == JobStatus.cancelled).toList();

  void configure(String baseUrl, String? apiKey, {String? wsUrl}) {
    _service = ArchiveService(baseUrl: baseUrl, apiKey: apiKey);

    if (wsUrl != null) {
      _connectWebSocket(wsUrl, apiKey);
    }

    notifyListeners();
  }

  bool get isConfigured => _service != null;

  void _connectWebSocket(String wsUrl, String? apiKey) {
    _wsSubscription?.cancel();
    _wsChannel?.sink.close();

    try {
      final uri = apiKey != null
          ? Uri.parse('$wsUrl?apiKey=$apiKey')
          : Uri.parse(wsUrl);

      _wsChannel = WebSocketChannel.connect(uri);

      _wsSubscription = _wsChannel!.stream.listen(
        _handleWebSocketMessage,
        onError: (error) {
          debugPrint('WebSocket error: $error');
        },
        onDone: () {
          debugPrint('WebSocket closed');
        },
      );

      _wsChannel!.sink.add(jsonEncode({
        'type': 'ArchiveSubscribe',
      }));
    } catch (e) {
      debugPrint('Failed to connect WebSocket: $e');
    }
  }

  void _handleWebSocketMessage(dynamic message) {
    try {
      final data = jsonDecode(message as String) as Map<String, dynamic>;
      final type = data['type'] as String?;

      switch (type) {
        case 'ArchiveProgress':
          _handleProgressUpdate(ArchiveProgressUpdate.fromJson(data));
          break;
        case 'ArchiveCompleted':
          _handleJobCompleted(data);
          break;
        case 'ArchiveFailed':
          _handleJobFailed(data);
          break;
      }
    } catch (e) {
      debugPrint('Error handling WebSocket message: $e');
    }
  }

  void _handleProgressUpdate(ArchiveProgressUpdate update) {
    final jobIndex = _jobs.indexWhere((j) => j.id == update.jobId);
    if (jobIndex >= 0) {
      final job = _jobs[jobIndex];
      _jobs[jobIndex] = ArchiveJob(
        id: job.id,
        archiveType: job.archiveType,
        operation: job.operation,
        status: JobStatus.running,
        progress: update.progress,
        progressMessage: update.message,
        targetId: job.targetId,
        targetName: job.targetName,
        filePath: job.filePath,
        createdAt: job.createdAt,
        startedAt: job.startedAt ?? DateTime.now(),
        completedAt: null,
        error: null,
        result: null,
      );
      _activeJobs[update.jobId] = _jobs[jobIndex];
      notifyListeners();
    }
  }

  void _handleJobCompleted(Map<String, dynamic> data) {
    final jobId = data['jobId'] as String;
    final jobIndex = _jobs.indexWhere((j) => j.id == jobId);
    if (jobIndex >= 0) {
      loadJobs(refresh: true);
    }
    _activeJobs.remove(jobId);
    notifyListeners();
  }

  void _handleJobFailed(Map<String, dynamic> data) {
    final jobId = data['jobId'] as String;
    final error = data['error'] as String?;
    final jobIndex = _jobs.indexWhere((j) => j.id == jobId);
    if (jobIndex >= 0) {
      final job = _jobs[jobIndex];
      _jobs[jobIndex] = ArchiveJob(
        id: job.id,
        archiveType: job.archiveType,
        operation: job.operation,
        status: JobStatus.failed,
        progress: job.progress,
        progressMessage: null,
        targetId: job.targetId,
        targetName: job.targetName,
        filePath: job.filePath,
        createdAt: job.createdAt,
        startedAt: job.startedAt,
        completedAt: DateTime.now(),
        error: error,
        result: null,
      );
    }
    _activeJobs.remove(jobId);
    notifyListeners();
  }

  Future<void> loadJobs({bool refresh = false, int? limit}) async {
    if (_service == null) {
      _errorMessage = 'Service not configured';
      notifyListeners();
      return;
    }

    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    try {
      final response = await _service!.getJobs(limit: limit);
      _jobs = response.jobs;
      _activeJobCount = response.activeCount;

      _activeJobs.clear();
      for (final job in _jobs.where((j) => j.isActive)) {
        _activeJobs[job.id] = job;
      }

      _errorMessage = null;
    } catch (e) {
      _errorMessage = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> selectJob(String jobId) async {
    if (_service == null) return;

    _isLoading = true;
    notifyListeners();

    try {
      _selectedJob = await _service!.getJob(jobId);
    } catch (e) {
      _errorMessage = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  void clearSelection() {
    _selectedJob = null;
    notifyListeners();
  }

  Future<ArchiveJobResponse> loadIar({
    required String userFirstname,
    required String userLastname,
    required String filePath,
    String? targetFolder,
    bool merge = false,
    bool createUserIfMissing = false,
    String? userId,
    String? userEmail,
    String? userPassword,
  }) async {
    if (_service == null) {
      return ArchiveJobResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.loadIar(
        userFirstname: userFirstname,
        userLastname: userLastname,
        filePath: filePath,
        targetFolder: targetFolder,
        merge: merge,
        createUserIfMissing: createUserIfMissing,
        userId: userId,
        userEmail: userEmail,
        userPassword: userPassword,
      );

      if (response.success) {
        await loadJobs(refresh: true);
      }

      return response;
    } catch (e) {
      return ArchiveJobResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ArchiveJobResponse> saveIar({
    required String userFirstname,
    required String userLastname,
    required String outputPath,
    String? folderPath,
    bool includeAssets = true,
  }) async {
    if (_service == null) {
      return ArchiveJobResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.saveIar(
        userFirstname: userFirstname,
        userLastname: userLastname,
        outputPath: outputPath,
        folderPath: folderPath,
        includeAssets: includeAssets,
      );

      if (response.success) {
        await loadJobs(refresh: true);
      }

      return response;
    } catch (e) {
      return ArchiveJobResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ArchiveJobResponse> loadOar({
    required String regionName,
    required String filePath,
    bool merge = false,
    String? defaultUserFirstname,
    String? defaultUserLastname,
  }) async {
    if (_service == null) {
      return ArchiveJobResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.loadOar(
        regionName: regionName,
        filePath: filePath,
        merge: merge,
        defaultUserFirstname: defaultUserFirstname,
        defaultUserLastname: defaultUserLastname,
      );

      if (response.success) {
        await loadJobs(refresh: true);
      }

      return response;
    } catch (e) {
      return ArchiveJobResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ArchiveJobResponse> saveOar({
    required String regionName,
    required String outputPath,
    bool includeAssets = true,
    bool includeTerrain = true,
    bool includeObjects = true,
    bool includeParcels = true,
  }) async {
    if (_service == null) {
      return ArchiveJobResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.saveOar(
        regionName: regionName,
        outputPath: outputPath,
        includeAssets: includeAssets,
        includeTerrain: includeTerrain,
        includeObjects: includeObjects,
        includeParcels: includeParcels,
      );

      if (response.success) {
        await loadJobs(refresh: true);
      }

      return response;
    } catch (e) {
      return ArchiveJobResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ArchiveJobResponse> clearRegion() async {
    if (_service == null) {
      return ArchiveJobResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.clearRegion();
      return response;
    } catch (e) {
      return ArchiveJobResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> cancelJob(String jobId) async {
    if (_service == null) return false;

    try {
      final success = await _service!.cancelJob(jobId);
      if (success) {
        await loadJobs(refresh: true);
      }
      return success;
    } catch (e) {
      _errorMessage = e.toString();
      notifyListeners();
      return false;
    }
  }

  Future<String?> downloadArchive(String jobId) async {
    if (_service == null) return null;

    try {
      return await _service!.downloadArchive(jobId);
    } catch (e) {
      _errorMessage = e.toString();
      notifyListeners();
      return null;
    }
  }

  @override
  void dispose() {
    _wsSubscription?.cancel();
    _wsChannel?.sink.close();
    super.dispose();
  }
}
