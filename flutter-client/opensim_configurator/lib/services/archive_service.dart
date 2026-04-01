import 'dart:convert';
import 'package:http/http.dart' as http;
import '../models/archive_models.dart';

class ArchiveService {
  final String baseUrl;
  final String? apiKey;

  ArchiveService({
    required this.baseUrl,
    this.apiKey,
  });

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
        if (apiKey != null) 'X-API-Key': apiKey!,
      };

  ArchiveJobResponse _parseJobResponse(http.Response response) {
    final data = jsonDecode(response.body) as Map<String, dynamic>;
    return ArchiveJobResponse(
      success: data['success'] as bool? ?? false,
      jobId: data['job_id'] as String?,
      message: data['message'] as String?,
      error: data['success'] == false ? data['message'] as String? : null,
    );
  }

  ArchiveType _parseArchiveType(String typeStr) {
    final lower = typeStr.toLowerCase();
    if (lower.contains('iar')) return ArchiveType.iar;
    return ArchiveType.oar;
  }

  JobOperation _parseJobOperation(String typeStr) {
    final lower = typeStr.toLowerCase();
    if (lower.contains('save')) return JobOperation.save;
    return JobOperation.load;
  }

  JobStatus _parseJobStatus(String statusStr) {
    final lower = statusStr.toLowerCase();
    if (lower.contains('completed') || lower.contains('complete')) return JobStatus.completed;
    if (lower.contains('running') || lower.contains('progress')) return JobStatus.running;
    if (lower.contains('failed') || lower.contains('fail')) return JobStatus.failed;
    if (lower.contains('cancelled') || lower.contains('cancel')) return JobStatus.cancelled;
    return JobStatus.queued;
  }

  Future<ArchiveJobListResponse> getJobs({int? limit}) async {
    final uri = limit != null
        ? Uri.parse('$baseUrl/admin/archives/jobs?limit=$limit')
        : Uri.parse('$baseUrl/admin/archives/jobs');

    final response = await http.get(uri, headers: _headers);

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      final jobsData = data['data']?['jobs'] as List<dynamic>? ?? [];
      final jobs = jobsData.map((j) {
        final jobMap = j as Map<String, dynamic>;
        return ArchiveJob(
          id: jobMap['id'] as String? ?? '',
          archiveType: _parseArchiveType(jobMap['type'] as String? ?? ''),
          operation: _parseJobOperation(jobMap['type'] as String? ?? ''),
          status: _parseJobStatus(jobMap['status'] as String? ?? ''),
          progress: (jobMap['progress'] as num?)?.toDouble() ?? 0.0,
          progressMessage: jobMap['message'] as String?,
          createdAt: DateTime.tryParse(jobMap['created_at'] as String? ?? '') ?? DateTime.now(),
          completedAt: jobMap['completed_at'] != null
              ? DateTime.tryParse(jobMap['completed_at'] as String)
              : null,
        );
      }).toList();

      final activeCount = jobs.where((j) => j.isActive).length;
      return ArchiveJobListResponse(
        jobs: jobs,
        totalCount: jobs.length,
        activeCount: activeCount,
      );
    }
    throw Exception('Failed to load archive jobs: ${response.statusCode}');
  }

  Future<ArchiveJob?> getJob(String jobId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/admin/archives/jobs/$jobId'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      final jobData = data['data'] as Map<String, dynamic>?;
      if (jobData == null) return null;
      return ArchiveJob(
        id: jobId,
        archiveType: _parseArchiveType(jobData['type'] as String? ?? ''),
        operation: _parseJobOperation(jobData['type'] as String? ?? ''),
        status: _parseJobStatus(jobData['status'] as String? ?? ''),
        progress: (jobData['progress'] as num?)?.toDouble() ?? 0.0,
        progressMessage: data['message'] as String?,
        createdAt: DateTime.tryParse(jobData['created_at'] as String? ?? '') ?? DateTime.now(),
        startedAt: jobData['started_at'] != null
            ? DateTime.tryParse(jobData['started_at'] as String)
            : null,
        completedAt: jobData['completed_at'] != null
            ? DateTime.tryParse(jobData['completed_at'] as String)
            : null,
        error: jobData['error'] as String?,
      );
    } else if (response.statusCode == 404) {
      return null;
    }
    throw Exception('Failed to load job: ${response.statusCode}');
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
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/iar/load'),
      headers: _headers,
      body: jsonEncode({
        'file_path': filePath,
        'user_firstname': userFirstname,
        'user_lastname': userLastname,
        if (targetFolder != null) 'target_folder': targetFolder,
        'merge': merge,
        if (createUserIfMissing) 'create_user_if_missing': true,
        if (userId != null) 'user_id': userId,
        if (userEmail != null) 'user_email': userEmail,
        if (userPassword != null) 'user_password': userPassword,
      }),
    );

    if (response.statusCode == 200 || response.statusCode == 202) {
      return _parseJobResponse(response);
    }

    return ArchiveJobResponse(
      success: false,
      error: 'Failed to start IAR load: ${response.statusCode} - ${response.body}',
    );
  }

  Future<ArchiveJobResponse> saveIar({
    required String userFirstname,
    required String userLastname,
    required String outputPath,
    String? folderPath,
    bool includeAssets = true,
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/iar/save'),
      headers: _headers,
      body: jsonEncode({
        'output_path': outputPath,
        'user_firstname': userFirstname,
        'user_lastname': userLastname,
        if (folderPath != null) 'folder_path': folderPath,
        'include_assets': includeAssets,
      }),
    );

    if (response.statusCode == 200 || response.statusCode == 202) {
      return _parseJobResponse(response);
    }

    return ArchiveJobResponse(
      success: false,
      error: 'Failed to start IAR save: ${response.statusCode} - ${response.body}',
    );
  }

  Future<ArchiveJobResponse> loadOar({
    required String regionName,
    required String filePath,
    bool merge = false,
    double displacementX = 0.0,
    double displacementY = 0.0,
    double displacementZ = 0.0,
    double rotationDegrees = 0.0,
    bool forceTerrain = false,
    bool forceParcels = false,
    String? defaultUserFirstname,
    String? defaultUserLastname,
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/oar/load'),
      headers: _headers,
      body: jsonEncode({
        'file_path': filePath,
        'region_name': regionName,
        'merge': merge,
        'displacement_x': displacementX,
        'displacement_y': displacementY,
        'displacement_z': displacementZ,
        'rotation_degrees': rotationDegrees,
        'force_terrain': forceTerrain,
        'force_parcels': forceParcels,
        if (defaultUserFirstname != null) 'default_user_firstname': defaultUserFirstname,
        if (defaultUserLastname != null) 'default_user_lastname': defaultUserLastname,
      }),
    );

    if (response.statusCode == 200 || response.statusCode == 202) {
      return _parseJobResponse(response);
    }

    return ArchiveJobResponse(
      success: false,
      error: 'Failed to start OAR load: ${response.statusCode} - ${response.body}',
    );
  }

  Future<ArchiveJobResponse> saveOar({
    required String regionName,
    required String outputPath,
    bool includeAssets = true,
    bool includeTerrain = true,
    bool includeObjects = true,
    bool includeParcels = true,
  }) async {
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/oar/save'),
      headers: _headers,
      body: jsonEncode({
        'output_path': outputPath,
        'region_name': regionName,
        'include_assets': includeAssets,
        'include_terrain': includeTerrain,
        'include_objects': includeObjects,
        'include_parcels': includeParcels,
      }),
    );

    if (response.statusCode == 200 || response.statusCode == 202) {
      return _parseJobResponse(response);
    }

    return ArchiveJobResponse(
      success: false,
      error: 'Failed to start OAR save: ${response.statusCode} - ${response.body}',
    );
  }

  Future<ArchiveJobResponse> clearRegion() async {
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/region/clear'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      return _parseJobResponse(response);
    }

    return ArchiveJobResponse(
      success: false,
      error: 'Failed to clear region: ${response.statusCode} - ${response.body}',
    );
  }

  Future<List<Map<String, dynamic>>> getOarFiles() async {
    final response = await http.get(
      Uri.parse('$baseUrl/admin/archives/oar/files'),
      headers: _headers,
    );
    if (response.statusCode == 200) {
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      final files = data['data']?['files'] as List<dynamic>? ?? [];
      return files.cast<Map<String, dynamic>>();
    }
    return [];
  }

  Future<bool> cancelJob(String jobId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/admin/archives/jobs/$jobId/cancel'),
      headers: _headers,
    );

    return response.statusCode == 200;
  }

  Future<String?> downloadArchive(String jobId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/admin/archives/jobs/$jobId/download'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['downloadUrl'] as String? ?? data['download_url'] as String?;
    }
    return null;
  }
}
