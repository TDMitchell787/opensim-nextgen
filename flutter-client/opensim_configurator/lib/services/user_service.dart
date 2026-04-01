import 'dart:convert';
import 'package:http/http.dart' as http;
import '../models/user_models.dart';

class UserService {
  final String baseUrl;
  final String? apiKey;

  UserService({
    required this.baseUrl,
    this.apiKey,
  });

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
        if (apiKey != null) 'X-API-Key': apiKey!,
      };

  Future<UserListResponse> getUsers({int page = 1, int pageSize = 50}) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/admin/users?page=$page&pageSize=$pageSize'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      return UserListResponse.fromJson(jsonDecode(response.body));
    }
    throw Exception('Failed to load users: ${response.statusCode}');
  }

  Future<UserAccount?> getUser(String userId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/admin/users/$userId'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      return UserAccount.fromJson(jsonDecode(response.body));
    } else if (response.statusCode == 404) {
      return null;
    }
    throw Exception('Failed to load user: ${response.statusCode}');
  }

  Future<UserAccount?> getUserByName(String firstName, String lastName) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/admin/users/by-name/$firstName/$lastName'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      return UserAccount.fromJson(jsonDecode(response.body));
    } else if (response.statusCode == 404) {
      return null;
    }
    throw Exception('Failed to load user: ${response.statusCode}');
  }

  Future<UserCreateResponse> createUser(UserCreateRequest request) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/admin/users'),
      headers: _headers,
      body: jsonEncode(request.toJson()),
    );

    if (response.statusCode == 200 || response.statusCode == 201) {
      return UserCreateResponse.fromJson(jsonDecode(response.body));
    }

    return UserCreateResponse(
      success: false,
      error: 'Failed to create user: ${response.statusCode} - ${response.body}',
    );
  }

  Future<bool> deleteUser(String userId) async {
    final response = await http.delete(
      Uri.parse('$baseUrl/api/admin/users/$userId'),
      headers: _headers,
    );

    return response.statusCode == 200 || response.statusCode == 204;
  }

  Future<bool> updateUserLevel(String userId, int level) async {
    final response = await http.patch(
      Uri.parse('$baseUrl/api/admin/users/$userId/level'),
      headers: _headers,
      body: jsonEncode({'level': level}),
    );

    return response.statusCode == 200;
  }

  Future<bool> setUserStatus(String userId, UserStatus status) async {
    final response = await http.patch(
      Uri.parse('$baseUrl/api/admin/users/$userId/status'),
      headers: _headers,
      body: jsonEncode({'status': status.name}),
    );

    return response.statusCode == 200;
  }

  Future<bool> resetPassword(String userId, String newPassword) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/admin/users/$userId/reset-password'),
      headers: _headers,
      body: jsonEncode({'password': newPassword}),
    );

    return response.statusCode == 200;
  }

  Future<AppearanceDiagnostics> getAppearanceDiagnostics(String userId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/admin/users/$userId/appearance/diagnostics'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      return AppearanceDiagnostics.fromJson(jsonDecode(response.body));
    }
    throw Exception('Failed to load appearance diagnostics: ${response.statusCode}');
  }

  Future<bool> repairAppearance(String userId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/admin/users/$userId/appearance/repair'),
      headers: _headers,
    );

    return response.statusCode == 200;
  }

  Future<bool> resetAppearance(String userId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/admin/users/$userId/appearance/reset'),
      headers: _headers,
    );

    return response.statusCode == 200;
  }

  Future<List<InventoryFolder>> getInventoryFolders(String userId) async {
    final response = await http.get(
      Uri.parse('$baseUrl/api/admin/users/$userId/inventory/folders'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      final List<dynamic> data = jsonDecode(response.body);
      return data.map((json) => InventoryFolder.fromJson(json)).toList();
    }
    throw Exception('Failed to load inventory folders: ${response.statusCode}');
  }

  Future<bool> rebuildInventory(String userId) async {
    final response = await http.post(
      Uri.parse('$baseUrl/api/admin/users/$userId/inventory/rebuild'),
      headers: _headers,
    );

    return response.statusCode == 200;
  }
}
