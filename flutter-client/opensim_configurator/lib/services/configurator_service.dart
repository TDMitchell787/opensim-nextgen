// OpenSim Next Configurator Service
// Service layer for communicating with Rust FFI bridge

import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;
import '../models/deployment_models.dart';
import 'admin_service.dart';

// Mock implementation for development
// This will be replaced with actual FFI calls once the Rust bridge is generated
class ConfiguratorService {
  String _serverUrl = 'http://localhost:9000';
  String _apiKey = 'default-key-change-me';

  void updateConnection(String serverUrl, String apiKey) {
    _serverUrl = serverUrl;
    _apiKey = apiKey;
  }

  // Auto-detect optimal deployment type
  Future<DeploymentRecommendation> autoDetectDeployment(SystemInfo systemInfo) async {
    await _simulateDelay();

    // Simulate the auto-detection logic
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

    // Find the highest scoring deployment type
    var sortedEntries = scores.entries.toList()
      ..sort((a, b) => b.value.compareTo(a.value));

    DeploymentType recommendedType = sortedEntries.isNotEmpty 
        ? sortedEntries.first.key 
        : DeploymentType.Development;
    double confidence = sortedEntries.isNotEmpty 
        ? sortedEntries.first.value.clamp(0.0, 1.0) 
        : 0.5;

    // Create alternative options
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

  // Get default configuration for deployment type
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

  // Validate configuration
  Future<ValidationResult> validateConfiguration(OpenSimConfig config) async {
    await _simulateDelay();

    List<String> errors = [];
    List<String> warnings = [];
    List<String> recommendations = [];
    int score = 100;

    // Validate based on deployment type
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

  // Get server status from actual OpenSim Next API
  Future<ServerStatus> getServerStatus() async {
    try {
      await AdminService.instance.ensureDiscovered();
      final adminUrl = AdminService.instance.adminUrl;

      final healthResponse = await http.get(
        Uri.parse('$adminUrl/admin/health'),
        headers: {
          'X-API-Key': _apiKey,
          'Content-Type': 'application/json',
        },
      );

      final infoResponse = await http.get(
        Uri.parse('$adminUrl/info'),
        headers: {
          'X-API-Key': _apiKey,
          'Content-Type': 'application/json',
        },
      );
      
      bool isRunning = healthResponse.statusCode == 200;
      
      if (!isRunning) {
        // Return offline status if health check fails
        return ServerStatus(
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
        );
      }

      Map<String, dynamic> infoData = {};
      if (infoResponse.statusCode == 200) {
        infoData = json.decode(infoResponse.body);
      }

      // Extract real server data
      int uptimeSeconds = 0;
      int activeRegions = 1;
      int connectedUsers = 0;
      double cpuUsage = 0.0;
      double memoryUsage = 0.0;
      int totalConnections = 0;

      if (infoData.isNotEmpty) {
        // Parse uptime (convert from seconds to match our format)
        if (infoData['uptime'] != null) {
          uptimeSeconds = (infoData['uptime'] as num).toInt();
        }
        
        // Parse active regions
        if (infoData['active_regions'] != null) {
          activeRegions = (infoData['active_regions'] as num).toInt();
        }
        
        // Parse active connections (treat as connected users)
        if (infoData['active_connections'] != null) {
          totalConnections = (infoData['active_connections'] as num).toInt();
          connectedUsers = totalConnections; // For now, assume all connections are users
        }
        
        // Parse system metrics
        if (infoData['cpu_usage'] != null) {
          cpuUsage = (infoData['cpu_usage'] as num).toDouble();
        }
        
        if (infoData['memory_usage'] != null) {
          memoryUsage = (infoData['memory_usage'] as num).toDouble();
        }
      }

      return ServerStatus(
        isRunning: true,
        uptimeSeconds: uptimeSeconds,
        activeRegions: activeRegions,
        connectedUsers: connectedUsers,
        cpuUsage: cpuUsage,
        memoryUsage: memoryUsage,
        networkActivity: NetworkActivity(
          bytesSent: 0, // Not available in current API, could add later
          bytesReceived: 0, // Not available in current API, could add later
          connections: totalConnections,
        ),
      );
    } catch (e) {
      debugPrint('Error getting server status: $e');
      
      // Return fallback data if API call fails
      return ServerStatus(
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
      );
    }
  }

  // Apply configuration
  Future<String> applyConfiguration(OpenSimConfig config) async {
    await _simulateDelay(2000); // Longer delay for configuration application

    // Simulate success or failure
    if (config.gridName.isEmpty) {
      throw Exception('Grid name cannot be empty');
    }

    return 'Configuration applied successfully';
  }

  // Export configuration
  Future<String> exportConfiguration(OpenSimConfig config) async {
    await _simulateDelay(500);
    return jsonEncode(config.toJson());
  }

  // Import configuration
  Future<OpenSimConfig> importConfiguration(String jsonData) async {
    await _simulateDelay(500);
    
    try {
      final Map<String, dynamic> json = jsonDecode(jsonData);
      return OpenSimConfig.fromJson(json);
    } catch (e) {
      throw Exception('Invalid configuration format: $e');
    }
  }

  // Detect system capabilities
  Future<SystemInfo> detectSystemCapabilities() async {
    await _simulateDelay();

    // Gather some basic system information
    double memoryGb = 8.0; // Default assumption
    int cpuCores = 4; // Default assumption
    
    // Try to get actual values if possible
    try {
      if (!kIsWeb) {
        // On mobile/desktop, we could potentially get more accurate info
        // For now, use reasonable defaults
      }
    } catch (e) {
      debugPrint('Could not detect system capabilities: $e');
    }

    return SystemInfo(
      memoryGb: memoryGb,
      cpuCores: cpuCores,
      hasPublicIp: false, // Default to false for mobile
      bandwidthMbps: 100, // Default assumption
      domain: 'localhost',
      expectedUsers: 5,
      expectedRegions: 1,
      isCommercial: false,
    );
  }

  // Get available physics engines
  Future<List<String>> getAvailablePhysicsEngines() async {
    await _simulateDelay();
    return ['ODE', 'UBODE', 'Bullet', 'POS', 'Basic'];
  }

  // Get available database types
  Future<List<String>> getAvailableDatabaseTypes() async {
    await _simulateDelay();
    return ['SQLite', 'PostgreSQL', 'MySQL'];
  }

  // Helper methods for creating default configurations
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

  // Simulate network delay for realistic UX
  Future<void> _simulateDelay([int milliseconds = 800]) async {
    if (kDebugMode) {
      await Future.delayed(Duration(milliseconds: milliseconds));
    }
  }

  void dispose() {
    // Clean up resources if needed
  }
}