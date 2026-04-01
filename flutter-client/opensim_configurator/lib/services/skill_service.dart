import 'dart:convert';
import 'package:http/http.dart' as http;
import 'admin_service.dart';

class SkillService {
  static SkillService? _instance;
  static SkillService get instance => _instance ??= SkillService._();

  SkillService._();

  String get _baseUrl => AdminService.instance.adminUrl;

  Future<Map<String, dynamic>> getDashboard() async {
    await AdminService.instance.ensureDiscovered();
    try {
      final response = await http
          .get(Uri.parse('$_baseUrl/skills/dashboard'))
          .timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return _offlineDashboard();
    } catch (e) {
      return _offlineDashboard();
    }
  }

  Future<Map<String, dynamic>> getAllSkills() async {
    await AdminService.instance.ensureDiscovered();
    try {
      final response = await http
          .get(Uri.parse('$_baseUrl/skills'))
          .timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'total_skills': 0, 'total_domains': 0, 'domains': []};
    } catch (e) {
      return {'total_skills': 0, 'total_domains': 0, 'domains': []};
    }
  }

  Future<Map<String, dynamic>> getDomainSkills(String domain) async {
    await AdminService.instance.ensureDiscovered();
    try {
      final response = await http
          .get(Uri.parse('$_baseUrl/skills/$domain'))
          .timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'domain': domain, 'skills': [], 'total_skills': 0};
    } catch (e) {
      return {'domain': domain, 'skills': [], 'total_skills': 0};
    }
  }

  Future<Map<String, dynamic>> getSkillDetail(
      String domain, String skillId) async {
    await AdminService.instance.ensureDiscovered();
    try {
      final response = await http
          .get(Uri.parse('$_baseUrl/skills/$domain/$skillId'))
          .timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {};
    } catch (e) {
      return {};
    }
  }

  Future<Map<String, dynamic>> searchSkills(String query) async {
    await AdminService.instance.ensureDiscovered();
    try {
      final response = await http
          .get(Uri.parse('$_baseUrl/skills/search?q=$query'))
          .timeout(const Duration(seconds: 5));
      if (response.statusCode == 200) {
        return Map<String, dynamic>.from(json.decode(response.body));
      }
      return {'query': query, 'count': 0, 'results': []};
    } catch (e) {
      return {'query': query, 'count': 0, 'results': []};
    }
  }

  Map<String, dynamic> _offlineDashboard() {
    return {
      'total_skills': 0,
      'total_domains': 0,
      'domains': [],
      'overall_score': 0,
      'offline': true,
    };
  }
}
