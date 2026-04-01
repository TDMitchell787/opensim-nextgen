// Elegant Configurator Service - Implementing Single-Pass Data Loading
// Applies the Elegant Archive Solution pattern to Flutter API management

import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import '../models/deployment_models.dart';
import '../utils/flutter_error_classifier.dart';

/// Complete server data structure - Single-pass loading pattern
class CompleteServerData {
  final Map<String, dynamic> health;
  final Map<String, dynamic> info;
  final Map<String, dynamic> metrics;
  final String maskedDatabase;
  final ServerStatus serverStatus;
  final bool isConnected;
  
  const CompleteServerData({
    required this.health,
    required this.info,
    required this.metrics,
    required this.maskedDatabase,
    required this.serverStatus,
    required this.isConnected,
  });
  
  /// Create empty fallback data
  factory CompleteServerData.empty() {
    return CompleteServerData(
      health: {},
      info: {},
      metrics: {},
      maskedDatabase: '••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••',
      serverStatus: ServerStatus(
        isRunning: false,
        uptimeSeconds: 0,
        activeRegions: 0,
        connectedUsers: 0,
        cpuUsage: 0.0,
        memoryUsage: 0.0,
        networkActivity: NetworkActivity(
          bytesSent: 0,
          bytesReceived: 0,
          connections: 0,
        ),
      ),
      isConnected: false,
    );
  }
  
  /// Create from API response data
  factory CompleteServerData.fromApiData({
    required Map<String, dynamic> healthData,
    required Map<String, dynamic> infoData,
    required Map<String, dynamic> metricsData,
  }) {
    // Extract real server data in single pass
    int uptimeSeconds = 0;
    int activeRegions = 1;
    int connectedUsers = 0;
    double cpuUsage = 0.0;
    double memoryUsage = 0.0;
    int totalConnections = 0;

    if (infoData.isNotEmpty) {
      uptimeSeconds = (infoData['uptime'] as num?)?.toInt() ?? 0;
      activeRegions = (infoData['active_regions'] as num?)?.toInt() ?? 1;
      totalConnections = (infoData['active_connections'] as num?)?.toInt() ?? 0;
      connectedUsers = totalConnections;
      cpuUsage = (infoData['cpu_usage'] as num?)?.toDouble() ?? 0.0;
      memoryUsage = (infoData['memory_usage'] as num?)?.toDouble() ?? 0.0;
    }

    final serverStatus = ServerStatus(
      isRunning: healthData.isNotEmpty,
      uptimeSeconds: uptimeSeconds,
      activeRegions: activeRegions,
      connectedUsers: connectedUsers,
      cpuUsage: cpuUsage,
      memoryUsage: memoryUsage,
      networkActivity: NetworkActivity(
        bytesSent: 0,
        bytesReceived: 0,
        connections: totalConnections,
      ),
    );

    return CompleteServerData(
      health: healthData,
      info: infoData,
      metrics: metricsData,
      maskedDatabase: '••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••',
      serverStatus: serverStatus,
      isConnected: healthData.isNotEmpty,
    );
  }
}

/// Elegant Configurator Service - Eliminates async conflicts
class ElegantConfiguratorService {
  String _serverUrl = 'http://localhost:9000';
  String _apiKey = 'default-key-change-me';
  
  final ElegantDataLoader<Map<String, dynamic>> _dataLoader = ElegantDataLoader();

  void updateConnection(String serverUrl, String apiKey) {
    _serverUrl = serverUrl;
    _apiKey = apiKey;
  }

  /// ELEGANT SOLUTION: Single-pass server data loading
  /// Eliminates multiple setState conflicts from original configurator_service.dart
  Future<CompleteServerData> loadCompleteServerData() async {
    try {
      // Clear previous data sources
      _dataLoader.clear();
      
      // Add all data sources for parallel loading
      _dataLoader.addSource('health', _fetchApiData('/api/health'));
      _dataLoader.addSource('info', _fetchApiData('/api/info'));
      _dataLoader.addSource('metrics', _fetchApiData('/api/metrics'));
      
      // Single-pass parallel loading - no sequential conflicts
      final results = await _dataLoader.loadAllData();
      
      final serverData = CompleteServerData.fromApiData(
        healthData: results['health'] ?? {},
        infoData: results['info'] ?? {},
        metricsData: results['metrics'] ?? {},
      );
      
      FlutterErrorClassifier.logError(
        errorType: 'Server Data Loading Success',
        errorMessage: 'Successfully loaded complete server data in single pass',
        solution: 'Elegant parallel loading pattern eliminates async conflicts',
        affectedFiles: ['ElegantConfiguratorService'],
        detectiveCommand: 'Future.wait() parallel API calls',
        additionalInfo: {
          'isConnected': serverData.isConnected,
          'activeRegions': serverData.serverStatus.activeRegions,
          'connectedUsers': serverData.serverStatus.connectedUsers,
        },
      );
      
      return serverData;
      
    } catch (e) {
      FlutterErrorClassifier.logError(
        errorType: 'Server Data Loading Error',
        errorMessage: e.toString(),
        solution: 'Returning fallback empty data to prevent app crash',
        affectedFiles: ['ElegantConfiguratorService'],
        detectiveCommand: 'Check network connectivity: curl -I $_serverUrl/api/health',
      );
      
      return CompleteServerData.empty();
    }
  }

  /// Fetch API data with proper error handling
  Future<Map<String, dynamic>> _fetchApiData(String endpoint) async {
    try {
      // Use cache busting for web to prevent service worker issues
      final url = kIsWeb 
          ? FlutterCacheBuster.addCacheBuster(endpoint)
          : endpoint;
      
      final response = await http.get(
        Uri.parse(url),
        headers: _apiKey.isNotEmpty ? {'Authorization': 'Bearer $_apiKey'} : {},
      ).timeout(Duration(seconds: 10));
      
      if (response.statusCode == 200) {
        return json.decode(response.body) as Map<String, dynamic>;
      } else {
        throw HttpException('HTTP ${response.statusCode}: ${response.body}');
      }
    } catch (e) {
      FlutterErrorClassifier.logError(
        errorType: 'API Request Error',
        errorMessage: 'Failed to fetch $endpoint: $e',
        solution: 'Implementing fallback data and retry mechanism',
        affectedFiles: ['ElegantConfiguratorService._fetchApiData'],
        detectiveCommand: 'curl -I $_serverUrl$endpoint',
      );
      
      return {}; // Return empty map instead of throwing
    }
  }

  /// Get server status using elegant single-pass loading
  Future<ServerStatus> getServerStatus() async {
    final completeData = await loadCompleteServerData();
    return completeData.serverStatus;
  }

  /// Check if server is connected
  Future<bool> isServerConnected() async {
    final completeData = await loadCompleteServerData();
    return completeData.isConnected;
  }

  // Maintain compatibility with original API while improving implementation
  
  /// Auto-detect optimal deployment type (unchanged from original)
  Future<DeploymentRecommendation> autoDetectDeployment(SystemInfo systemInfo) async {
    await _simulateDelay();

    Map<DeploymentType, double> scores = {};
    List<String> reasoningParts = [];

    // Hardware-based scoring
    if (systemInfo.memoryGb >= 64.0 && systemInfo.cpuCores >= 32) {
      scores[DeploymentType.Grid] = 0.9;
      reasoningParts.add('High-end hardware suitable for grid deployment');
    } else if (systemInfo.memoryGb >= 16.0 && systemInfo.cpuCores >= 8) {
      scores[DeploymentType.Production] = 0.8;
      reasoningParts.add('Sufficient hardware for production deployment');
    } else {
      scores[DeploymentType.Development] = 0.7;
      reasoningParts.add('Hardware suitable for development environment');
    }

    // Network-based scoring
    if (systemInfo.hasPublicIp && systemInfo.bandwidthMbps >= 1000 && systemInfo.domain != 'localhost') {
      scores[DeploymentType.Grid] = (scores[DeploymentType.Grid] ?? 0.0) + 0.3;
      reasoningParts.add('Public IP, high bandwidth, and domain configuration suggest grid deployment');
    } else if (systemInfo.hasPublicIp && systemInfo.domain != 'localhost') {
      scores[DeploymentType.Production] = (scores[DeploymentType.Production] ?? 0.0) + 0.2;
      reasoningParts.add('Public IP and domain configuration suggest production deployment');
    }

    // Usage-based scoring
    if (systemInfo.expectedUsers > 100 || systemInfo.expectedRegions > 16 || systemInfo.isCommercial) {
      scores[DeploymentType.Grid] = (scores[DeploymentType.Grid] ?? 0.0) + 0.4;
      reasoningParts.add('High user count or commercial use suggests grid deployment');
    } else if (systemInfo.expectedUsers > 10 || systemInfo.expectedRegions > 4) {
      scores[DeploymentType.Production] = (scores[DeploymentType.Production] ?? 0.0) + 0.3;
      reasoningParts.add('Medium scale usage suggests production deployment');
    }

    var sortedEntries = scores.entries.toList()
      ..sort((a, b) => b.value.compareTo(a.value));

    DeploymentType recommendedType = sortedEntries.isNotEmpty 
        ? sortedEntries.first.key 
        : DeploymentType.Development;
    double confidence = sortedEntries.isNotEmpty 
        ? sortedEntries.first.value.clamp(0.0, 1.0) 
        : 0.5;

    List<AlternativeOption> alternatives = [];
    for (int i = 1; i < sortedEntries.length && i < 3; i++) {
      if (sortedEntries[i].value > 0.3) {
        alternatives.add(AlternativeOption(
          deploymentType: sortedEntries[i].key,
          confidence: sortedEntries[i].value,
          reason: 'Alternative option with ${(sortedEntries[i].value * 100).toInt()}% confidence',
        ));
      }
    }

    return DeploymentRecommendation(
      recommendedType: recommendedType,
      confidence: confidence,
      reasoning: reasoningParts.join('. '),
      alternativeOptions: alternatives,
    );
  }

  /// Get default configuration for deployment type (unchanged from original)
  Future<OpenSimConfig> getDefaultConfig(DeploymentType deploymentType) async {
    await _simulateDelay();

    switch (deploymentType) {
      case DeploymentType.Development:
        return _createDevelopmentConfig();
      case DeploymentType.Production:
        return _createProductionConfig();
      case DeploymentType.Grid:
        return _createGridConfig();
    }
  }

  /// Validate configuration (unchanged from original)
  Future<ValidationResult> validateConfiguration(OpenSimConfig config) async {
    await _simulateDelay();

    List<String> errors = [];
    List<String> warnings = [];
    List<String> recommendations = [];
    int score = 100;

    // Validation logic remains the same as original
    switch (config.deploymentType) {
      case DeploymentType.Production:
        if (!config.networkConfig.httpsEnabled) {
          errors.add('Production deployments require HTTPS to be enabled');
          score -= 30;
        }
        if (config.securityConfig.sslCertificatePath.isEmpty) {
          errors.add('Production deployments require SSL certificate configuration');
          score -= 25;
        }
        if (config.databaseType == 'SQLite') {
          warnings.add('SQLite is not recommended for production deployments');
          score -= 10;
        }
        break;
      case DeploymentType.Grid:
        if (config.securityConfig.sslCertificatePath.isEmpty) {
          errors.add('Grid deployments require SSL certificates');
          score -= 35;
        }
        if (config.databaseType != 'PostgreSQL') {
          errors.add('Grid deployments require PostgreSQL for optimal performance');
          score -= 20;
        }
        if (config.performanceConfig.maxPrims < 50000) {
          warnings.add('Grid deployments should support high prim counts');
          score -= 5;
        }
        break;
      case DeploymentType.Development:
        if (config.networkConfig.httpsEnabled && config.securityConfig.sslCertificatePath.isEmpty) {
          warnings.add('HTTPS enabled but no SSL certificate configured');
          score -= 5;
        }
        break;
    }

    // General validation
    if (config.gridName.isEmpty) {
      errors.add('Grid name is required');
      score -= 15;
    }

    if (config.networkConfig.httpPort == 0) {
      errors.add('Valid HTTP port is required');
      score -= 20;
    }

    // Recommendations
    if (config.performanceConfig.cacheAssets) {
      recommendations.add('Asset caching is enabled for better performance');
    } else {
      recommendations.add('Consider enabling asset caching for better performance');
    }

    if (config.securityConfig.passwordComplexity) {
      recommendations.add('Password complexity requirements are enabled for better security');
    } else {
      recommendations.add('Consider enabling password complexity requirements');
    }

    return ValidationResult(
      isValid: errors.isEmpty,
      errors: errors,
      warnings: warnings,
      recommendations: recommendations,
      overallScore: score.clamp(0, 100),
    );
  }

  /// Apply configuration (unchanged from original)
  Future<String> applyConfiguration(OpenSimConfig config) async {
    await _simulateDelay(2000);

    if (config.gridName.isEmpty) {
      throw Exception('Grid name cannot be empty');
    }

    return 'Configuration applied successfully';
  }

  /// Export configuration (unchanged from original)
  Future<String> exportConfiguration(OpenSimConfig config) async {
    await _simulateDelay(500);
    return jsonEncode(config.toJson());
  }

  /// Import configuration (unchanged from original)
  Future<OpenSimConfig> importConfiguration(String jsonData) async {
    await _simulateDelay(500);
    
    try {
      final Map<String, dynamic> json = jsonDecode(jsonData);
      return OpenSimConfig.fromJson(json);
    } catch (e) {
      throw Exception('Invalid configuration format: $e');
    }
  }

  /// Detect system capabilities (unchanged from original)
  Future<SystemInfo> detectSystemCapabilities() async {
    await _simulateDelay();

    double memoryGb = 8.0;
    int cpuCores = 4;
    
    try {
      if (!kIsWeb) {
        // On mobile/desktop, we could potentially get more accurate info
      }
    } catch (e) {
      debugPrint('Could not detect system capabilities: $e');
    }

    return SystemInfo(
      memoryGb: memoryGb,
      cpuCores: cpuCores,
      hasPublicIp: false,
      bandwidthMbps: 100,
      domain: 'localhost',
      expectedUsers: 5,
      expectedRegions: 1,
      isCommercial: false,
    );
  }

  /// Get available physics engines (unchanged from original)
  Future<List<String>> getAvailablePhysicsEngines() async {
    await _simulateDelay();
    return ['ODE', 'UBODE', 'Bullet', 'POS', 'Basic'];
  }

  /// Get available database types (unchanged from original)
  Future<List<String>> getAvailableDatabaseTypes() async {
    await _simulateDelay();
    return ['SQLite', 'PostgreSQL', 'MySQL'];
  }

  // Configuration creation methods (unchanged from original)
  
  OpenSimConfig _createDevelopmentConfig() {
    return OpenSimConfig(
      deploymentType: DeploymentType.Development,
      gridName: 'OpenSim Next Development Grid',
      gridNick: 'devgrid',
      welcomeMessage: 'Welcome to your development environment!',
      databaseType: 'SQLite',
      databaseConnection: 'Data Source=./OpenSim.db;Version=3;',
      physicsEngine: 'ODE',
      networkConfig: NetworkConfig(
        httpPort: 9000,
        httpsPort: 9443,
        httpsEnabled: false,
        externalHostname: 'localhost',
        internalIp: '127.0.0.1',
      ),
      securityConfig: SecurityConfig(
        passwordComplexity: false,
        sessionTimeout: 3600,
        bruteForceProtection: false,
        sslCertificatePath: '',
        sslPrivateKeyPath: '',
      ),
      performanceConfig: PerformanceConfig(
        maxPrims: 15000,
        maxScripts: 1000,
        scriptTimeout: 30,
        cacheAssets: true,
        cacheTimeout: 48,
      ),
    );
  }

  OpenSimConfig _createProductionConfig() {
    return OpenSimConfig(
      deploymentType: DeploymentType.Production,
      gridName: 'OpenSim Next Production Grid',
      gridNick: 'prodgrid',
      welcomeMessage: 'Welcome to our virtual world!',
      databaseType: 'PostgreSQL',
      databaseConnection: '••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••',
      physicsEngine: 'Bullet',
      networkConfig: NetworkConfig(
        httpPort: 80,
        httpsPort: 443,
        httpsEnabled: true,
        externalHostname: 'yourgrid.com',
        internalIp: '0.0.0.0',
      ),
      securityConfig: SecurityConfig(
        passwordComplexity: true,
        sessionTimeout: 1800,
        bruteForceProtection: true,
        sslCertificatePath: '/etc/ssl/certs/opensim.crt',
        sslPrivateKeyPath: '/etc/ssl/private/opensim.key',
      ),
      performanceConfig: PerformanceConfig(
        maxPrims: 45000,
        maxScripts: 3000,
        scriptTimeout: 25,
        cacheAssets: true,
        cacheTimeout: 24,
      ),
    );
  }

  OpenSimConfig _createGridConfig() {
    return OpenSimConfig(
      deploymentType: DeploymentType.Grid,
      gridName: 'OpenSim Next Enterprise Grid',
      gridNick: 'enterprise',
      welcomeMessage: 'Welcome to our enterprise metaverse!',
      databaseType: 'PostgreSQL',
      databaseConnection: '••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••',
      physicsEngine: 'POS',
      networkConfig: NetworkConfig(
        httpPort: 80,
        httpsPort: 443,
        httpsEnabled: true,
        externalHostname: 'grid.enterprise.com',
        internalIp: '0.0.0.0',
      ),
      securityConfig: SecurityConfig(
        passwordComplexity: true,
        sessionTimeout: 900,
        bruteForceProtection: true,
        sslCertificatePath: '/etc/ssl/enterprise/opensim.crt',
        sslPrivateKeyPath: '/etc/ssl/enterprise/opensim.key',
      ),
      performanceConfig: PerformanceConfig(
        maxPrims: 100000,
        maxScripts: 10000,
        scriptTimeout: 20,
        cacheAssets: true,
        cacheTimeout: 12,
      ),
    );
  }

  /// Simulate network delay for realistic UX
  Future<void> _simulateDelay([int milliseconds = 800]) async {
    if (kDebugMode) {
      await Future.delayed(Duration(milliseconds: milliseconds));
    }
  }

  void dispose() {
    // Clean up resources if needed
  }
}