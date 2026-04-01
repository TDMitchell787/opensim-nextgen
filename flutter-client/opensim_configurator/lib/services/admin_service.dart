import 'dart:convert';
import 'package:http/http.dart' as http;

class AdminService {
  static String _adminUrl = 'http://localhost:9200';
  static bool _discovered = false;

  static AdminService? _instance;
  static AdminService get instance => _instance ??= AdminService._();

  AdminService._();

  String get adminUrl => _adminUrl;

  Future<void> ensureDiscovered() async {
    if (_discovered) return;
    for (final port in [9200, 9700, 9300, 9301, 9302, 9600, 9800]) {
      try {
        final response = await http.get(
          Uri.parse('http://localhost:$port/admin/health'),
        ).timeout(const Duration(seconds: 2));
        if (response.statusCode == 200) {
          _adminUrl = 'http://localhost:$port';
          _discovered = true;
          return;
        }
      } catch (_) {}
    }
    _discovered = true;
  }

  Future<Map<String, dynamic>> getHealth() async {
    await ensureDiscovered();
    try {
      final response = await http.get(Uri.parse('$_adminUrl/admin/health'));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return _offlineHealth();
    } catch (e) {
      return _offlineHealth();
    }
  }

  Future<Map<String, dynamic>> getDatabaseStats() async {
    try {
      final response = await http.get(Uri.parse('$_adminUrl/admin/database/stats'));
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return Map<String, dynamic>.from(body['data']);
        }
      }
      return _offlineStats();
    } catch (e) {
      return _offlineStats();
    }
  }

  Future<List<Map<String, dynamic>>> listUsers({int limit = 50, int offset = 0}) async {
    try {
      final response = await http.get(
        Uri.parse('$_adminUrl/admin/users?limit=$limit&offset=$offset'),
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          final data = body['data'];
          if (data['users'] != null) {
            return List<Map<String, dynamic>>.from(data['users']);
          }
        }
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<Map<String, dynamic>?> showUserAccount(String firstname, String lastname) async {
    try {
      final response = await http.get(
        Uri.parse('$_adminUrl/admin/users/account?firstname=$firstname&lastname=$lastname'),
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return Map<String, dynamic>.from(body['data']);
        }
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  Future<Map<String, dynamic>> createUser(String firstname, String lastname, String email, String password, {int? userLevel}) async {
    try {
      final body = <String, dynamic>{
        'firstname': firstname,
        'lastname': lastname,
        'email': email,
        'password': password,
      };
      if (userLevel != null) body['user_level'] = userLevel;

      final response = await http.post(
        Uri.parse('$_adminUrl/admin/users'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode(body),
      );
      final result = json.decode(response.body);
      return Map<String, dynamic>.from(result);
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> deleteUser(String firstname, String lastname) async {
    try {
      final request = http.Request('DELETE', Uri.parse('$_adminUrl/admin/users/delete'));
      request.headers['Content-Type'] = 'application/json';
      request.body = json.encode({'firstname': firstname, 'lastname': lastname});
      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> resetPassword(String firstname, String lastname, String newPassword) async {
    try {
      final response = await http.put(
        Uri.parse('$_adminUrl/admin/users/password'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({
          'firstname': firstname,
          'lastname': lastname,
          'new_password': newPassword,
        }),
      );
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> resetEmail(String firstname, String lastname, String newEmail) async {
    try {
      final response = await http.put(
        Uri.parse('$_adminUrl/admin/users/email'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({
          'firstname': firstname,
          'lastname': lastname,
          'new_email': newEmail,
        }),
      );
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> setUserLevel(String firstname, String lastname, int level) async {
    try {
      final response = await http.put(
        Uri.parse('$_adminUrl/admin/users/level'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({
          'firstname': firstname,
          'lastname': lastname,
          'user_level': level,
        }),
      );
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> getSecurityStats() async {
    try {
      final response = await http.get(Uri.parse('$_adminUrl/api/security/stats'));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'udp': {'total_packets': 0, 'total_dropped': 0, 'tracked_ips': 0, 'blocked_ips': 0}, 'blocked_ip_count': 0, '_offline': true};
    } catch (e) {
      return {'udp': {'total_packets': 0, 'total_dropped': 0, 'tracked_ips': 0, 'blocked_ips': 0}, 'blocked_ip_count': 0, '_offline': true};
    }
  }

  Future<Map<String, dynamic>> getSecurityThreats() async {
    try {
      final response = await http.get(Uri.parse('$_adminUrl/api/security/threats'));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'threat_count': 0, 'total_dropped_packets': 0, 'threats': [], '_offline': true};
    } catch (e) {
      return {'threat_count': 0, 'total_dropped_packets': 0, 'threats': [], '_offline': true};
    }
  }

  Future<Map<String, dynamic>> getSecurityLockouts() async {
    try {
      final response = await http.get(Uri.parse('$_adminUrl/api/security/lockouts'));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'lockout_count': 0, 'lockouts': [], '_offline': true};
    } catch (e) {
      return {'lockout_count': 0, 'lockouts': [], '_offline': true};
    }
  }

  Future<Map<String, dynamic>> blacklistIp(String ip) async {
    try {
      final response = await http.post(Uri.parse('$_adminUrl/api/security/blacklist/$ip'));
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> unblockIp(String ip) async {
    try {
      final request = http.Request('DELETE', Uri.parse('$_adminUrl/api/security/blacklist/$ip'));
      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);
      return Map<String, dynamic>.from(json.decode(response.body));
    } catch (e) {
      return {'success': false, 'message': 'Connection failed: $e'};
    }
  }

  Future<Map<String, dynamic>> getZitiStatus() async {
    try {
      final response = await http.get(Uri.parse('$_adminUrl/api/security/ziti/status'));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'enabled': false, 'running': false, '_offline': true};
    } catch (e) {
      return {'enabled': false, 'running': false, '_offline': true};
    }
  }

  Map<String, dynamic> _offlineHealth() {
    return {'status': 'offline', '_offline': true};
  }

  Map<String, dynamic> _offlineStats() {
    return {'total_users': 0, 'total_assets': 0, 'total_regions': 0, '_offline': true};
  }
}
