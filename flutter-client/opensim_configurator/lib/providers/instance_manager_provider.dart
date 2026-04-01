import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../models/instance_models.dart';
import '../services/unified_backend_service.dart';

class InstanceManagerProvider extends ChangeNotifier {
  final Map<String, ServerInstance> _instances = {};
  final Map<String, WebSocketChannel> _connections = {};
  final Map<String, StreamSubscription> _subscriptions = {};
  final Map<String, List<ConsoleEntry>> _consoleHistory = {};

  String? _selectedInstanceId;
  bool _isLoading = false;
  String? _errorMessage;
  ControllerConfig _controllerConfig = ControllerConfig();
  Timer? _healthCheckTimer;
  Timer? _reconnectTimer;
  Timer? _discoveryTimer;
  bool _discoveryComplete = false;

  List<ServerInstance> get instances => _instances.values.toList();
  List<ServerInstance> get connectedInstances =>
      _instances.values.where((i) => i.connected).toList();
  ServerInstance? get selectedInstance =>
      _selectedInstanceId != null ? _instances[_selectedInstanceId] : null;
  String? get selectedInstanceId => _selectedInstanceId;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  ControllerConfig get controllerConfig => _controllerConfig;

  int get totalInstances => _instances.length;
  int get connectedCount => _instances.values.where((i) => i.connected).length;
  int get healthyCount =>
      _instances.values.where((i) => i.isHealthy).length;
  int get needsAttentionCount =>
      _instances.values.where((i) => i.needsAttention).length;

  InstanceManagerProvider() {
    _startHealthCheckTimer();
    _startDiscoveryTimer();
    _runHttpDiscovery();
  }

  void _startHealthCheckTimer() {
    _healthCheckTimer?.cancel();
    _healthCheckTimer = Timer.periodic(
      Duration(milliseconds: _controllerConfig.healthCheckIntervalMs),
      (_) => _runHealthChecks(),
    );
  }

  Future<void> _runHealthChecks() async {
    for (final instance in _instances.values) {
      if (instance.connected) {
        _sendMessage(instance.id, {'type': 'Heartbeat'});
      }
    }
  }

  Future<void> loadInstancesFromConfig(InstancesConfig config) async {
    _setLoading(true);
    try {
      _controllerConfig = config.controller;

      for (final entry in config.instances) {
        final instance = entry.toServerInstance();
        _instances[instance.id] = instance;
        _consoleHistory[instance.id] = [];
      }

      _startHealthCheckTimer();

      final autoConnectIds = config.instances
          .where((i) => i.autoConnect)
          .map((i) => i.id)
          .toList();

      for (final id in autoConnectIds) {
        await connectToInstance(id);
      }

      _clearError();
    } catch (e) {
      _setError('Failed to load instances: $e');
    } finally {
      _setLoading(false);
    }
  }

  Future<void> addInstance(ServerInstance instance) async {
    _instances[instance.id] = instance;
    _consoleHistory[instance.id] = [];
    notifyListeners();

    if (instance.autoConnect) {
      await connectToInstance(instance.id);
    }
  }

  Future<void> removeInstance(String instanceId) async {
    await disconnectFromInstance(instanceId);
    _instances.remove(instanceId);
    _consoleHistory.remove(instanceId);

    if (_selectedInstanceId == instanceId) {
      _selectedInstanceId = _instances.isNotEmpty ? _instances.keys.first : null;
    }

    notifyListeners();
  }

  void selectInstance(String? instanceId) {
    _selectedInstanceId = instanceId;
    notifyListeners();
  }

  Future<bool> connectToInstance(String instanceId) async {
    final instance = _instances[instanceId];
    if (instance == null) {
      _setError('Instance not found: $instanceId');
      return false;
    }

    if (_connections.containsKey(instanceId)) {
      return true;
    }

    try {
      final channel = WebSocketChannel.connect(
        Uri.parse(instance.websocketUrl),
      );

      _connections[instanceId] = channel;

      _subscriptions[instanceId] = channel.stream.listen(
        (message) => _handleMessage(instanceId, message),
        onError: (error) => _handleConnectionError(instanceId, error),
        onDone: () => _handleDisconnect(instanceId),
      );

      _sendMessage(instanceId, {
        'type': 'Auth',
        'token': instance.apiKey,
      });

      instance.connected = true;
      instance.status = InstanceStatus.running;
      instance.lastSeen = DateTime.now();
      notifyListeners();

      _subscribe(instanceId, [
        'statusUpdates',
        'metrics',
        'console',
        'alerts',
      ]);

      return true;
    } catch (e) {
      _setError('Failed to connect to $instanceId: $e');
      _scheduleReconnect(instanceId);
      return false;
    }
  }

  Future<void> disconnectFromInstance(String instanceId) async {
    _subscriptions[instanceId]?.cancel();
    _subscriptions.remove(instanceId);

    await _connections[instanceId]?.sink.close();
    _connections.remove(instanceId);

    final instance = _instances[instanceId];
    if (instance != null) {
      instance.connected = false;
      instance.status = InstanceStatus.disconnected;
    }

    notifyListeners();
  }

  void _handleMessage(String instanceId, dynamic message) {
    try {
      final data = message is String ? jsonDecode(message) : message;
      final type = data['message']?['type'] ?? data['type'];

      final instance = _instances[instanceId];
      if (instance != null) {
        instance.lastSeen = DateTime.now();
      }

      switch (type) {
        case 'AuthResponse':
          _handleAuthResponse(instanceId, data);
          break;
        case 'InstanceStatusUpdate':
          _handleStatusUpdate(instanceId, data);
          break;
        case 'ConsoleOutput':
          _handleConsoleOutput(instanceId, data);
          break;
        case 'InstanceControlResponse':
          _handleControlResponse(instanceId, data);
          break;
        case 'InstanceList':
          _handleInstanceList(data);
          break;
        case 'Heartbeat':
          _sendMessage(instanceId, {'type': 'Pong'});
          break;
        case 'Pong':
          break;
        case 'Error':
          _handleError(instanceId, data);
          break;
      }

      notifyListeners();
    } catch (e) {
      debugPrint('Error handling message from $instanceId: $e');
    }
  }

  void _handleAuthResponse(String instanceId, Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final success = message['success'] ?? false;
    final instance = _instances[instanceId];

    if (instance != null) {
      if (success) {
        instance.connected = true;
        instance.status = InstanceStatus.running;
      } else {
        instance.connected = false;
        instance.status = InstanceStatus.error;
        _setError('Authentication failed for $instanceId');
      }
    }
  }

  void _handleStatusUpdate(String instanceId, Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final instance = _instances[instanceId];

    if (instance != null) {
      final statusStr = message['status'];
      if (statusStr != null) {
        instance.status = _parseStatus(statusStr);
      }

      final metricsData = message['metrics'];
      if (metricsData != null) {
        instance.metrics = InstanceMetrics.fromJson(metricsData);
      }

      final healthData = message['health'];
      if (healthData != null) {
        instance.health = HealthStatus.fromJson(healthData);
      }
    }
  }

  void _handleConsoleOutput(String instanceId, Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final entry = ConsoleEntry(
      instanceId: instanceId,
      content: message['content'] ?? '',
      outputType: _parseOutputType(message['output_type']),
      timestamp: DateTime.fromMillisecondsSinceEpoch(
        (message['timestamp'] ?? 0) * 1000,
      ),
    );

    _consoleHistory[instanceId]?.add(entry);

    if ((_consoleHistory[instanceId]?.length ?? 0) > 1000) {
      _consoleHistory[instanceId]?.removeAt(0);
    }
  }

  void _handleControlResponse(String instanceId, Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final success = message['success'] ?? false;
    final responseMessage = message['message'] ?? '';

    if (!success) {
      _setError('Command failed on $instanceId: $responseMessage');
    }
  }

  void _handleInstanceList(Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final instancesData = message['instances'] as List?;

    if (instancesData != null) {
      for (final instanceData in instancesData) {
        final id = instanceData['id'];
        final instance = _instances[id];
        if (instance != null) {
          instance.status = _parseStatus(instanceData['status']);
          instance.version = instanceData['version'];
        }
      }
    }
  }

  void _handleError(String instanceId, Map<String, dynamic> data) {
    final message = data['message'] ?? data;
    final errorMsg = message['message'] ?? 'Unknown error';
    _setError('Error from $instanceId: $errorMsg');
  }

  void _handleConnectionError(String instanceId, dynamic error) {
    debugPrint('Connection error for $instanceId: $error');
    final instance = _instances[instanceId];
    if (instance != null) {
      instance.connected = false;
      instance.status = InstanceStatus.error;
    }
    notifyListeners();
    _scheduleReconnect(instanceId);
  }

  void _handleDisconnect(String instanceId) {
    debugPrint('Disconnected from $instanceId');
    _connections.remove(instanceId);
    _subscriptions.remove(instanceId);

    final instance = _instances[instanceId];
    if (instance != null) {
      instance.connected = false;
      instance.status = InstanceStatus.disconnected;
    }
    notifyListeners();
    _scheduleReconnect(instanceId);
  }

  void _scheduleReconnect(String instanceId) {
    final instance = _instances[instanceId];
    if (instance?.autoConnect != true) return;

    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(
      Duration(milliseconds: _controllerConfig.reconnectDelayMs),
      () => connectToInstance(instanceId),
    );
  }

  void _sendMessage(String instanceId, Map<String, dynamic> message) {
    final channel = _connections[instanceId];
    if (channel != null) {
      final envelope = {
        'id': DateTime.now().millisecondsSinceEpoch.toString(),
        'timestamp': DateTime.now().millisecondsSinceEpoch ~/ 1000,
        'message': message,
      };
      channel.sink.add(jsonEncode(envelope));
    }
  }

  void _subscribe(String instanceId, List<String> channels) {
    _sendMessage(instanceId, {
      'type': 'Subscribe',
      'instance_id': instanceId,
      'channels': channels,
    });
  }

  Future<CommandResult> sendCommand(
    String instanceId,
    InstanceCommand command, {
    Map<String, dynamic>? parameters,
  }) async {
    final instance = _instances[instanceId];
    if (instance == null) {
      return CommandResult.failure('Instance not found');
    }

    if (!instance.connected) {
      return CommandResult.failure('Instance not connected');
    }

    try {
      _sendMessage(instanceId, {
        'type': 'InstanceControl',
        'instance_id': instanceId,
        'command': command.name,
        'parameters': parameters,
      });

      return CommandResult.success('Command sent');
    } catch (e) {
      return CommandResult.failure('Failed to send command: $e');
    }
  }

  Future<CommandResult> sendConsoleCommand(
    String instanceId,
    String command,
  ) async {
    final instance = _instances[instanceId];
    if (instance == null) {
      return CommandResult.failure('Instance not found');
    }

    if (!instance.connected) {
      return CommandResult.failure('Instance not connected');
    }

    try {
      _consoleHistory[instanceId]?.add(ConsoleEntry(
        instanceId: instanceId,
        content: '> $command',
        outputType: ConsoleOutputType.command,
        timestamp: DateTime.now(),
      ));

      _sendMessage(instanceId, {
        'type': 'ConsoleCommand',
        'instance_id': instanceId,
        'command': command,
      });

      notifyListeners();
      return CommandResult.success('Command sent');
    } catch (e) {
      return CommandResult.failure('Failed to send command: $e');
    }
  }

  Future<List<BatchResult>> broadcastCommand(
    InstanceCommand command, {
    List<String>? instanceIds,
    Map<String, dynamic>? parameters,
  }) async {
    final targetIds = instanceIds ?? connectedInstances.map((i) => i.id).toList();
    final results = <BatchResult>[];

    for (final id in targetIds) {
      final result = await sendCommand(id, command, parameters: parameters);
      results.add(BatchResult(
        instanceId: id,
        status: result.success ? 'success' : 'failed',
        message: result.message,
        data: result.data,
        durationMs: result.durationMs,
      ));
    }

    return results;
  }

  List<ConsoleEntry> getConsoleHistory(String instanceId) {
    return _consoleHistory[instanceId] ?? [];
  }

  void clearConsoleHistory(String instanceId) {
    _consoleHistory[instanceId]?.clear();
    notifyListeners();
  }

  Stream<ConsoleEntry> getConsoleStream(String instanceId) async* {
    var lastIndex = _consoleHistory[instanceId]?.length ?? 0;

    while (true) {
      await Future.delayed(const Duration(milliseconds: 100));

      final history = _consoleHistory[instanceId];
      if (history != null && history.length > lastIndex) {
        for (var i = lastIndex; i < history.length; i++) {
          yield history[i];
        }
        lastIndex = history.length;
      }
    }
  }

  List<ServerInstance> getInstancesByEnvironment(InstanceEnvironment env) {
    return _instances.values.where((i) => i.environment == env).toList();
  }

  List<ServerInstance> getInstancesByTag(String tag) {
    return _instances.values.where((i) => i.tags.contains(tag)).toList();
  }

  List<ServerInstance> getInstancesByStatus(InstanceStatus status) {
    return _instances.values.where((i) => i.status == status).toList();
  }

  InstanceStatus _parseStatus(String? status) {
    switch (status?.toLowerCase()) {
      case 'starting':
        return InstanceStatus.starting;
      case 'running':
        return InstanceStatus.running;
      case 'stopping':
        return InstanceStatus.stopping;
      case 'stopped':
        return InstanceStatus.stopped;
      case 'error':
        return InstanceStatus.error;
      case 'maintenance':
        return InstanceStatus.maintenance;
      case 'discovered':
        return InstanceStatus.discovered;
      case 'disconnected':
        return InstanceStatus.disconnected;
      default:
        return InstanceStatus.unknown;
    }
  }

  ConsoleOutputType _parseOutputType(String? type) {
    switch (type?.toLowerCase()) {
      case 'stdout':
        return ConsoleOutputType.stdout;
      case 'stderr':
        return ConsoleOutputType.stderr;
      case 'info':
        return ConsoleOutputType.info;
      case 'warning':
        return ConsoleOutputType.warning;
      case 'error':
        return ConsoleOutputType.error;
      case 'debug':
        return ConsoleOutputType.debug;
      case 'command':
        return ConsoleOutputType.command;
      default:
        return ConsoleOutputType.info;
    }
  }

  Color getStatusColor(InstanceStatus status) {
    switch (status) {
      case InstanceStatus.discovered:
        return const Color(0xFF8B5CF6);
      case InstanceStatus.running:
        return const Color(0xFF10B981);
      case InstanceStatus.starting:
      case InstanceStatus.stopping:
        return const Color(0xFFFBBF24);
      case InstanceStatus.stopped:
        return const Color(0xFF6B7280);
      case InstanceStatus.error:
        return const Color(0xFFEF4444);
      case InstanceStatus.maintenance:
        return const Color(0xFF3B82F6);
      case InstanceStatus.disconnected:
        return const Color(0xFFF97316);
      case InstanceStatus.unknown:
        return const Color(0xFF9CA3AF);
    }
  }

  Color getHealthColor(HealthState health) {
    switch (health) {
      case HealthState.healthy:
        return const Color(0xFF10B981);
      case HealthState.degraded:
        return const Color(0xFFFBBF24);
      case HealthState.unhealthy:
        return const Color(0xFFEF4444);
      case HealthState.unknown:
        return const Color(0xFF9CA3AF);
    }
  }

  void _setLoading(bool loading) {
    _isLoading = loading;
    notifyListeners();
  }

  void _setError(String error) {
    _errorMessage = error;
    notifyListeners();
  }

  void _clearError() {
    _errorMessage = null;
    notifyListeners();
  }

  // === Controller Mode Methods ===

  WebSocketChannel? _controllerConnection;
  StreamSubscription? _controllerSubscription;
  bool _controllerConnected = false;
  bool get controllerConnected => _controllerConnected;

  Future<bool> connectToController(String controllerWsUrl) async {
    try {
      _controllerConnection = WebSocketChannel.connect(
        Uri.parse(controllerWsUrl),
      );

      _controllerSubscription = _controllerConnection!.stream.listen(
        (message) => _handleControllerMessage(message),
        onError: (error) {
          debugPrint('Controller WS error: $error');
          _controllerConnected = false;
          notifyListeners();
        },
        onDone: () {
          debugPrint('Controller WS closed');
          _controllerConnected = false;
          notifyListeners();
        },
      );

      _controllerConnected = true;
      notifyListeners();
      return true;
    } catch (e) {
      _setError('Failed to connect to controller: $e');
      return false;
    }
  }

  void disconnectFromController() {
    _controllerSubscription?.cancel();
    _controllerConnection?.sink.close();
    _controllerConnection = null;
    _controllerSubscription = null;
    _controllerConnected = false;
    notifyListeners();
  }

  void _handleControllerMessage(dynamic message) {
    try {
      final data = message is String ? jsonDecode(message) : message;
      final type = data['type'] ?? '';

      switch (type) {
        case 'InstanceList':
          _handleControllerInstanceList(data);
          break;
        case 'InstanceAnnounced':
          _handleInstanceAnnounced(data);
          break;
        case 'InstanceDeparted':
          _handleInstanceDeparted(data);
          break;
        case 'ProcessOutput':
          _handleProcessOutput(data);
          break;
        case 'InstanceStatusUpdate':
          final instanceId = data['instance_id'] ?? '';
          _handleStatusUpdate(instanceId, data);
          break;
        case 'Pong':
          break;
        case 'SubscriptionConfirmed':
          break;
      }

      notifyListeners();
    } catch (e) {
      debugPrint('Error handling controller message: $e');
    }
  }

  void _handleControllerInstanceList(Map<String, dynamic> data) {
    final instancesData = data['instances'] as List? ?? [];
    for (final instanceData in instancesData) {
      final id = instanceData['id'] ?? '';
      if (id.isEmpty) continue;

      if (!_instances.containsKey(id)) {
        _instances[id] = ServerInstance(
          id: id,
          name: instanceData['name'] ?? id,
          host: instanceData['host'] ?? 'localhost',
          ports: InstancePorts(),
          apiKey: '',
          status: _parseStatus(instanceData['status']),
        );
        _consoleHistory[id] = [];
      } else {
        _instances[id]!.status = _parseStatus(instanceData['status']);
      }
    }
  }

  void _handleInstanceAnnounced(Map<String, dynamic> data) {
    final id = data['instance_id'] ?? '';
    if (id.isEmpty) return;

    if (!_instances.containsKey(id)) {
      _instances[id] = ServerInstance(
        id: id,
        name: data['name'] ?? id,
        host: data['host'] ?? 'localhost',
        ports: InstancePorts(),
        apiKey: '',
        status: InstanceStatus.running,
      );
      _consoleHistory[id] = [];
    } else {
      _instances[id]!.status = InstanceStatus.running;
      _instances[id]!.connected = true;
    }
  }

  void _handleInstanceDeparted(Map<String, dynamic> data) {
    final id = data['instance_id'] ?? '';
    if (id.isEmpty) return;

    final instance = _instances[id];
    if (instance != null) {
      instance.status = InstanceStatus.stopped;
      instance.connected = false;
    }
  }

  void _handleProcessOutput(Map<String, dynamic> data) {
    final id = data['instance_id'] ?? '';
    if (id.isEmpty) return;

    final entry = ConsoleEntry(
      instanceId: id,
      content: data['line'] ?? '',
      outputType: data['stream'] == 'stderr'
          ? ConsoleOutputType.stderr
          : ConsoleOutputType.stdout,
      timestamp: DateTime.now(),
    );

    _consoleHistory[id]?.add(entry);
    if ((_consoleHistory[id]?.length ?? 0) > 1000) {
      _consoleHistory[id]?.removeAt(0);
    }
  }

  Future<void> startInstance(String id) async {
    final service = UnifiedBackendService.instance;
    final result = await service.startControllerInstance(id);
    if (result['success'] == true) {
      _instances[id]?.status = InstanceStatus.starting;
      notifyListeners();
    } else {
      _setError(result['error'] ?? 'Failed to start instance');
    }
  }

  Future<void> stopInstance(String id) async {
    final service = UnifiedBackendService.instance;
    final result = await service.stopControllerInstance(id);
    if (result['success'] == true) {
      _instances[id]?.status = InstanceStatus.stopping;
      notifyListeners();
    } else {
      _setError(result['error'] ?? 'Failed to stop instance');
    }
  }

  Future<void> restartInstance(String id) async {
    final service = UnifiedBackendService.instance;
    final result = await service.restartControllerInstance(id);
    if (result['success'] == true) {
      _instances[id]?.status = InstanceStatus.starting;
      notifyListeners();
    } else {
      _setError(result['error'] ?? 'Failed to restart instance');
    }
  }

  void _startDiscoveryTimer() {
    _discoveryTimer?.cancel();
    _discoveryTimer = Timer.periodic(
      const Duration(seconds: 10),
      (_) => _runHttpDiscovery(),
    );
  }

  Future<void> _runHttpDiscovery() async {
    final service = UnifiedBackendService.instance;
    final running = await service.discoverRunningInstances();

    final discoveredIds = <String>{};
    for (final info in running) {
      final baseId = info['instance_id'] as String? ?? '';
      if (baseId.isEmpty) continue;

      final mode = info['service_mode'] as String? ?? '';
      final controllerPort = info['controller_port'] as int? ?? 0;
      final loginPort = info['login_port'] as int? ?? 0;
      final robustPort = info['robust_port'] as int? ?? 0;
      final gridName = info['grid_name'] as String? ?? '';

      final id = mode == 'robust' ? '$baseId-robust' : baseId;
      discoveredIds.add(id);

      if (!_instances.containsKey(id)) {
        _instances[id] = ServerInstance(
          id: id,
          name: gridName.isNotEmpty ? '$gridName ($mode)' : '$baseId ($mode)',
          host: 'localhost',
          ports: InstancePorts(
            http: loginPort > 0 ? loginPort : robustPort,
            udp: loginPort,
          ),
          apiKey: '',
          status: InstanceStatus.running,
        );
        _consoleHistory[id] = [];
      } else {
        _instances[id]!.status = InstanceStatus.running;
      }

      if (!_discoveryComplete && controllerPort > 0 && !_controllerConnected) {
        final url = 'http://localhost:$controllerPort';
        service.configure(controllerUrl: url);
        await connectToController(url.replaceFirst('http', 'ws') + '/ws');
        _discoveryComplete = true;
      }
    }

    for (final id in _instances.keys.toList()) {
      if (!discoveredIds.contains(id) && _instances[id]?.status == InstanceStatus.running) {
        final inst = _instances[id];
        if (inst != null && !inst.connected) {
          inst.status = InstanceStatus.stopped;
        }
      }
    }

    notifyListeners();
  }

  Future<void> fetchInstanceDirectories() async {
    await _runHttpDiscovery();

    final service = UnifiedBackendService.instance;
    final result = await service.getControllerInstanceDirs();
    final dirs = result['directories'] as List? ?? [];
    for (final dir in dirs) {
      final id = dir['id'] ?? '';
      if (id.isEmpty) continue;
      if (!_instances.containsKey(id)) {
        _instances[id] = ServerInstance(
          id: id,
          name: dir['name'] ?? id,
          host: 'localhost',
          ports: InstancePorts(),
          apiKey: '',
          status: InstanceStatus.discovered,
        );
        _consoleHistory[id] = [];
      }
    }

    if (_controllerConnected) {
      try {
        final runningResult = await service.getControllerHealth();
        if (runningResult['status'] == 'healthy') {
          final instancesResult = await service.getControllerInstances();
          final instances = instancesResult['instances'] as List? ?? [];
          for (final inst in instances) {
            final id = inst['id'] ?? '';
            if (id.isEmpty) continue;
            if (_instances.containsKey(id)) {
              final status = inst['status']?.toString().toLowerCase();
              if (status == 'running') {
                _instances[id]!.status = InstanceStatus.running;
              }
            }
          }
        }
      } catch (_) {}
    }

    notifyListeners();
  }

  @override
  void dispose() {
    _healthCheckTimer?.cancel();
    _reconnectTimer?.cancel();
    _discoveryTimer?.cancel();

    for (final subscription in _subscriptions.values) {
      subscription.cancel();
    }

    for (final connection in _connections.values) {
      connection.sink.close();
    }

    super.dispose();
  }
}
