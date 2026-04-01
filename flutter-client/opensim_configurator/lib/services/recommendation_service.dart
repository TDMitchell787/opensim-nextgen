import 'dart:convert';
import 'package:http/http.dart' as http;

class RecommendationService {
  static String _baseUrl = 'http://localhost:8080';
  static String _apiKey = 'default-key-change-me';

  static RecommendationService? _instance;
  static RecommendationService get instance => _instance ??= RecommendationService._();

  RecommendationService._();

  String get baseUrl => _baseUrl;

  void configure({String? baseUrl, String? apiKey}) {
    if (baseUrl != null) _baseUrl = baseUrl;
    if (apiKey != null) _apiKey = apiKey;
  }

  Map<String, String> get _headers => {
    'Content-Type': 'application/json',
    'X-API-Key': _apiKey,
  };

  Future<List<Map<String, dynamic>>> getTrendingContent({int limit = 10}) async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/trending?limit=$limit'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return List<Map<String, dynamic>>.from(body['data']);
        }
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<Map<String, dynamic>> getRecommenderStats() async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/stats'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return Map<String, dynamic>.from(body['data']);
        }
      }
      return _emptyStats();
    } catch (e) {
      return _emptyStats();
    }
  }

  Future<List<Map<String, dynamic>>> getContentRecommendations(String userId, {int limit = 10}) async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/$userId?limit=$limit'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return List<Map<String, dynamic>>.from(body['data']);
        }
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<List<Map<String, dynamic>>> getSocialRecommendations(String userId, {int limit = 10}) async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/$userId/social?limit=$limit'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return List<Map<String, dynamic>>.from(body['data']);
        }
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<List<Map<String, dynamic>>> getCreatorRecommendations(String userId, {int limit = 10}) async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/$userId/creators?limit=$limit'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return List<Map<String, dynamic>>.from(body['data']);
        }
      }
      return [];
    } catch (e) {
      return [];
    }
  }

  Future<Map<String, dynamic>> getEngagementMetrics(String userId) async {
    try {
      final response = await http.get(
        Uri.parse('$_baseUrl/api/ai/recommend/$userId/engagement'),
        headers: _headers,
      );
      if (response.statusCode == 200) {
        final body = json.decode(response.body);
        if (body['success'] == true && body['data'] != null) {
          return Map<String, dynamic>.from(body['data']);
        }
      }
      return _emptyEngagement();
    } catch (e) {
      return _emptyEngagement();
    }
  }

  Future<bool> recordActivity(String userId, String activityType, String targetId, String targetName, {int? durationSeconds}) async {
    try {
      final body = {
        'user_id': userId,
        'activity_type': activityType,
        'target_id': targetId,
        'target_name': targetName,
      };
      if (durationSeconds != null) {
        body['duration_seconds'] = durationSeconds.toString();
      }
      final response = await http.post(
        Uri.parse('$_baseUrl/api/ai/recommend/activity'),
        headers: _headers,
        body: json.encode(body),
      );
      return response.statusCode == 200;
    } catch (e) {
      return false;
    }
  }

  Future<bool> updateUserProfile(String userId, {
    List<String>? interests,
    List<String>? visitedRegions,
    List<String>? groups,
    List<String>? friends,
    List<String>? followedCreators,
  }) async {
    try {
      final body = <String, dynamic>{'user_id': userId};
      if (interests != null) body['interests'] = interests;
      if (visitedRegions != null) body['visited_regions'] = visitedRegions;
      if (groups != null) body['groups'] = groups;
      if (friends != null) body['friends'] = friends;
      if (followedCreators != null) body['followed_creators'] = followedCreators;

      final response = await http.post(
        Uri.parse('$_baseUrl/api/ai/recommend/profile'),
        headers: _headers,
        body: json.encode(body),
      );
      return response.statusCode == 200;
    } catch (e) {
      return false;
    }
  }

  Map<String, dynamic> _emptyStats() => {
    'total_users': 0, 'total_activities': 0, 'total_profiles': 0,
    'tracked_items': 0, 'cache_size': 0, '_offline': true,
  };

  Map<String, dynamic> _emptyEngagement() => {
    'sessions_last_30_days': 0, 'avg_session_duration_minutes': 0.0,
    'regions_visited_30_days': 0, 'social_interactions_30_days': 0,
    'content_created_30_days': 0, 'churn_risk_score': 0.0,
    'engagement_trend': 'Unknown', '_offline': true,
  };
}
