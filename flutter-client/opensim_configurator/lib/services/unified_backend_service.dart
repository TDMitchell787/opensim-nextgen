import 'dart:async';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:web_socket_channel/io.dart';

class UnifiedBackendService {
  static String BASE_URL = 'http://localhost:8080';
  static String API_BASE_URL = 'http://localhost:9100';
  static String WS_URL = 'ws://localhost:9001';
  static String API_KEY = 'default-key-change-me';

  static String _controllerUrl = 'http://localhost:9300';
  static String _controllerWsUrl = 'ws://localhost:9300/ws';

  static UnifiedBackendService? _instance;
  static UnifiedBackendService get instance => _instance ??= UnifiedBackendService._();

  UnifiedBackendService._();

  String get controllerUrl => _controllerUrl;
  String get controllerWsUrl => _controllerWsUrl;

  void configure({
    String? controllerUrl,
    String? apiKey,
    String? baseUrl,
    String? apiBaseUrl,
    String? wsUrl,
  }) {
    if (controllerUrl != null) {
      _controllerUrl = controllerUrl;
      _controllerWsUrl = controllerUrl.replaceFirst('http', 'ws') + '/ws';
    }
    if (apiKey != null) API_KEY = apiKey;
    if (baseUrl != null) BASE_URL = baseUrl;
    if (apiBaseUrl != null) API_BASE_URL = apiBaseUrl;
    if (wsUrl != null) WS_URL = wsUrl;
  }

  Map<String, String> get _headers => {
    'Content-Type': 'application/json',
    'X-API-Key': API_KEY,
  };

  Future<Map<String, dynamic>> getControllerHealth() async {
    try {
      final response = await http.get(
        Uri.parse('$_controllerUrl/health'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return {'status': 'error', 'details': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'status': 'unavailable', 'details': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getControllerInstances() async {
    try {
      final response = await http.get(
        Uri.parse('$_controllerUrl/api/instances'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return {'instances': [], 'error': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'instances': [], 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getControllerInstanceDirs() async {
    try {
      final response = await http.get(
        Uri.parse('$_controllerUrl/api/instance-dirs'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return {'directories': [], 'error': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'directories': [], 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> startControllerInstance(String id) async {
    try {
      final response = await http.post(
        Uri.parse('$_controllerUrl/api/instance/$id/start'),
        headers: _headers,
      );
      return json.decode(response.body);
    } catch (e) {
      return {'success': false, 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> stopControllerInstance(String id) async {
    try {
      final response = await http.post(
        Uri.parse('$_controllerUrl/api/instance/$id/stop'),
        headers: _headers,
      );
      return json.decode(response.body);
    } catch (e) {
      return {'success': false, 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> restartControllerInstance(String id) async {
    try {
      final response = await http.post(
        Uri.parse('$_controllerUrl/api/instance/$id/restart'),
        headers: _headers,
      );
      return json.decode(response.body);
    } catch (e) {
      return {'success': false, 'error': e.toString()};
    }
  }

  WebSocketChannel? _wsChannel;
  StreamController<Map<String, dynamic>>? _wsController;

  Stream<Map<String, dynamic>> get realTimeData => _wsController?.stream ?? Stream.empty();

  Future<void> initializeWebSocket() async {
    try {
      _wsController = StreamController<Map<String, dynamic>>.broadcast();
      _wsChannel = IOWebSocketChannel.connect(WS_URL);

      _wsChannel?.stream.listen(
        (data) {
          try {
            final jsonData = json.decode(data);
            _wsController?.add(jsonData);
          } catch (e) {
            print('WebSocket data parse error: $e');
          }
        },
        onError: (error) {
          print('WebSocket error: $error');
        },
        onDone: () {
          print('WebSocket connection closed');
        },
      );
    } catch (e) {
      print('Failed to initialize WebSocket: $e');
    }
  }

  void closeWebSocket() {
    _wsChannel?.sink.close();
    _wsController?.close();
    _wsChannel = null;
    _wsController = null;
  }

  Future<Map<String, dynamic>> getSystemHealth() async {
    try {
      final response = await http.get(
        Uri.parse('$API_BASE_URL/health'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return {'status': 'healthy', 'details': response.body};
      }
      return {'status': 'error', 'details': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'status': 'unavailable', 'details': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getSystemMetrics() async {
    try {
      final response = await http.get(
        Uri.parse('$API_BASE_URL/metrics'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return {'success': true, 'data': response.body};
      }
      return {'success': false, 'error': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'success': false, 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getSystemInfo() async {
    try {
      final response = await http.get(
        Uri.parse('$API_BASE_URL/info'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return {'error': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getAnalyticsData(String timeRange) async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/analytics/$timeRange'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return _emptyAnalytics(timeRange);
    } catch (e) {
      return _emptyAnalytics(timeRange);
    }
  }

  Future<Map<String, dynamic>> getObservabilityData() async {
    try {
      final response = await http.get(
        Uri.parse('$API_BASE_URL/api/observability'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return _emptyObservability();
    } catch (e) {
      return _emptyObservability();
    }
  }

  Future<Map<String, dynamic>> getAdminData() async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/admin/overview'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return _emptyAdminData();
    } catch (e) {
      return _emptyAdminData();
    }
  }

  Future<Map<String, dynamic>> sendAdminCommand(String command) async {
    try {
      final response = await http.post(
        Uri.parse('$_controllerUrl/api/command'),
        headers: _headers,
        body: json.encode({'command': command}),
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return {'success': false, 'error': 'HTTP ${response.statusCode}'};
    } catch (e) {
      return {'success': false, 'error': e.toString()};
    }
  }

  Future<Map<String, dynamic>> getDashboardOverview() async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/dashboard/overview'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return _emptyDashboard();
    } catch (e) {
      return _emptyDashboard();
    }
  }

  Future<List<Map<String, dynamic>>> getServerInstances() async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/servers'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final List<dynamic> instances = json.decode(response.body);
        return instances.cast<Map<String, dynamic>>();
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<Map<String, dynamic>> getUserStatistics() async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/users'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        return json.decode(response.body);
      }
      return _emptyUserStats();
    } catch (e) {
      return _emptyUserStats();
    }
  }

  Future<List<Map<String, dynamic>>> getRegions() async {
    try {
      final response = await http.get(
        Uri.parse('$BASE_URL/api/regions'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final List<dynamic> regions = json.decode(response.body);
        return regions.cast<Map<String, dynamic>>();
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  // --- HTTP-based Instance Discovery (sandbox-safe) ---

  Future<List<Map<String, dynamic>>> discoverRunningInstances() async {
    final results = <Map<String, dynamic>>[];
    int? firstControllerPort;

    final futures = <Future<int?>>[];
    for (int port = 9300; port <= 9320; port++) {
      futures.add(_probeControllerPort(port));
    }
    final probeResults = await Future.wait(futures);
    for (final port in probeResults) {
      if (port != null) {
        firstControllerPort ??= port;
      }
    }

    if (firstControllerPort != null) {
      try {
        final response = await http.get(
          Uri.parse('http://localhost:$firstControllerPort/api/running'),
        ).timeout(const Duration(seconds: 3));
        if (response.statusCode == 200) {
          final data = json.decode(response.body);
          final running = data['running'] as List? ?? [];
          for (final item in running) {
            if (item is Map<String, dynamic>) {
              results.add(item);
            }
          }
        }
      } catch (_) {}
    }

    return results;
  }

  Future<int?> _probeControllerPort(int port) async {
    try {
      final response = await http.get(
        Uri.parse('http://localhost:$port/health'),
      ).timeout(const Duration(seconds: 2));
      if (response.statusCode == 200) {
        return port;
      }
    } catch (_) {}
    return null;
  }

  Future<Map<String, dynamic>?> autoDiscoverAndConnect() async {
    final futures = <Future<int?>>[];
    for (int port = 9300; port <= 9320; port++) {
      futures.add(_probeControllerPort(port));
    }
    final probeResults = await Future.wait(futures);
    for (final port in probeResults) {
      if (port != null) {
        final url = 'http://localhost:$port';
        configure(controllerUrl: url);
        return {'controller_port': port, 'host': 'localhost'};
      }
    }
    return null;
  }

  // --- Empty fallbacks (no fake data, just zeros) ---

  Map<String, dynamic> _emptyAnalytics(String timeRange) => {
    'timeRange': timeRange,
    'worldMetrics': {'usersOnline': 0, 'regionsActive': 0, 'objectsTotal': 0},
    'performance': {'cpuUsage': 0, 'memoryUsage': 0, 'responseTime': 0},
    'network': {'websocketConnections': 0, 'assetRequestsPerSec': 0, 'regionCrossingsPerMin': 0},
    'alerts': <Map<String, dynamic>>[],
  };

  Map<String, dynamic> _emptyObservability() => {
    'cpuUsage': 0, 'memoryUsage': 0, 'diskIo': 0,
    'activeUsers': 0, 'activeRegions': 0, 'physicsBodies': 0,
    'websocketConnections': 0, 'avgResponseTime': 0,
    'dbQueryTime': 0, 'physicsFrameTime': 0, 'throughput': 0,
    'traces': <Map<String, dynamic>>[], 'realtimeMetrics': <Map<String, dynamic>>[],
    'logs': <Map<String, dynamic>>[],
  };

  Map<String, dynamic> _emptyAdminData() => {
    'serverInfo': {'version': 'OpenSim Next', 'uptime': '0', 'buildHash': '', 'status': 'unavailable'},
    'regions': <Map<String, dynamic>>[],
    'userStats': {'totalUsers': 0, 'onlineUsers': 0, 'newRegistrationsToday': 0},
    'logs': <Map<String, dynamic>>[],
  };

  Map<String, dynamic> _emptyDashboard() => {
    'system': {'users_online': 0, 'regions_active': 0, 'uptime_seconds': 0, 'server_version': ''},
    'users': {'total_users': 0, 'online_users': 0, 'new_registrations_24h': 0},
    'database': {'active_connections': 0, 'idle_connections': 0, 'health': 'unavailable'},
    'performance': {'cpu_usage': 0.0, 'memory_usage_mb': 0, 'response_time_ms': 0},
  };

  Map<String, dynamic> _emptyUserStats() => {
    'total_users': 0, 'online_users': 0, 'new_registrations_24h': 0,
    'active_users_7d': 0, 'average_session_duration_minutes': 0,
  };
}
