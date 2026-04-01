// OpenSim Next Configurator Provider
// State management for the configurator app

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/deployment_models.dart';
import '../services/elegant_configurator_service.dart';
import '../widgets/language_selector.dart';
import '../widgets/deployment_selector.dart';
import '../widgets/configuration_dashboard.dart';

class ConfiguratorProvider extends ChangeNotifier {
  final ElegantConfiguratorService _service = ElegantConfiguratorService();
  
  // Connection settings
  String _serverUrl = 'http://localhost:9000';
  String _apiKey = 'default-key-change-me';
  bool _isConnected = false;

  // Current configuration state
  DeploymentType? _selectedDeploymentType;
  OpenSimConfig? _currentConfig;
  SystemInfo? _systemInfo;
  DeploymentRecommendation? _recommendation;
  ValidationResult? _validationResult;
  ServerStatus? _serverStatus;

  // UI state
  bool _isLoading = false;
  String? _errorMessage;
  bool _autoDetectionCompleted = false;

  // Getters
  String get serverUrl => _serverUrl;
  String get apiKey => _apiKey;
  bool get isConnected => _isConnected;
  DeploymentType? get selectedDeploymentType => _selectedDeploymentType;
  OpenSimConfig? get currentConfig => _currentConfig;
  SystemInfo? get systemInfo => _systemInfo;
  DeploymentRecommendation? get recommendation => _recommendation;
  ValidationResult? get validationResult => _validationResult;
  ServerStatus? get serverStatus => _serverStatus;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  bool get autoDetectionCompleted => _autoDetectionCompleted;

  ConfiguratorProvider() {
    _loadSettings();
    _initializeSystemInfo();
  }

  // Initialize system information
  Future<void> _initializeSystemInfo() async {
    try {
      _systemInfo = await _service.detectSystemCapabilities();
      notifyListeners();
    } catch (e) {
      _setError('Failed to detect system capabilities: $e');
    }
  }

  // Load saved settings
  Future<void> _loadSettings() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      _serverUrl = prefs.getString('server_url') ?? _serverUrl;
      _apiKey = prefs.getString('api_key') ?? _apiKey;
      
      // Try to restore last configuration
      final configJson = prefs.getString('last_config');
      if (configJson != null) {
        _currentConfig = await _service.importConfiguration(configJson);
        _selectedDeploymentType = _currentConfig?.deploymentType;
      }
      
      notifyListeners();
    } catch (e) {
      debugPrint('Failed to load settings: $e');
    }
  }

  // Save settings
  Future<void> _saveSettings() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('server_url', _serverUrl);
      await prefs.setString('api_key', _apiKey);
      
      // Save current configuration
      if (_currentConfig != null) {
        final configJson = await _service.exportConfiguration(_currentConfig!);
        await prefs.setString('last_config', configJson);
      }
    } catch (e) {
      debugPrint('Failed to save settings: $e');
    }
  }

  // Update connection settings
  Future<void> updateConnectionSettings(String serverUrl, String apiKey) async {
    _serverUrl = serverUrl;
    _apiKey = apiKey;
    _service.updateConnection(serverUrl, apiKey);
    await _saveSettings();
    notifyListeners();
    
    // Test connection
    await testConnection();
  }

  // Test server connection using elegant single-pass loading
  Future<bool> testConnection() async {
    _setLoading(true);
    try {
      final completeData = await _service.loadCompleteServerData();
      _serverStatus = completeData.serverStatus;
      _isConnected = completeData.isConnected;
      _clearError();
      return _isConnected;
    } catch (e) {
      _isConnected = false;
      _setError('Connection failed: $e');
      return false;
    } finally {
      _setLoading(false);
    }
  }

  // Run auto-detection
  Future<void> runAutoDetection() async {
    if (_systemInfo == null) {
      _setError('System information not available');
      return;
    }

    _setLoading(true);
    try {
      _recommendation = await _service.autoDetectDeployment(_systemInfo!);
      _autoDetectionCompleted = true;
      _clearError();
    } catch (e) {
      _setError('Auto-detection failed: $e');
    } finally {
      _setLoading(false);
    }
  }

  // Select deployment type
  Future<void> selectDeploymentType(DeploymentType deploymentType) async {
    _selectedDeploymentType = deploymentType;
    _setLoading(true);
    
    try {
      _currentConfig = await _service.getDefaultConfig(deploymentType);
      await _saveSettings();
      await validateConfiguration();
      _clearError();
    } catch (e) {
      _setError('Failed to load deployment configuration: $e');
    } finally {
      _setLoading(false);
    }
  }

  // Update configuration
  Future<void> updateConfiguration(OpenSimConfig config) async {
    _currentConfig = config;
    await _saveSettings();
    await validateConfiguration();
    notifyListeners();
  }

  // Validate current configuration
  Future<void> validateConfiguration() async {
    if (_currentConfig == null) return;

    try {
      _validationResult = await _service.validateConfiguration(_currentConfig!);
      notifyListeners();
    } catch (e) {
      _setError('Validation failed: $e');
    }
  }

  // Apply configuration to server
  Future<bool> applyConfiguration() async {
    if (_currentConfig == null) {
      _setError('No configuration to apply');
      return false;
    }

    _setLoading(true);
    try {
      await _service.applyConfiguration(_currentConfig!);
      _clearError();
      
      // Refresh server status after applying configuration
      await refreshServerStatus();
      return true;
    } catch (e) {
      _setError('Failed to apply configuration: $e');
      return false;
    } finally {
      _setLoading(false);
    }
  }

  // Refresh server status using elegant single-pass loading
  Future<void> refreshServerStatus() async {
    try {
      final completeData = await _service.loadCompleteServerData();
      _serverStatus = completeData.serverStatus;
      _isConnected = completeData.isConnected;
      notifyListeners();
    } catch (e) {
      _isConnected = false;
      debugPrint('Failed to refresh server status: $e');
    }
  }

  // Export configuration
  Future<String?> exportConfiguration() async {
    if (_currentConfig == null) {
      _setError('No configuration to export');
      return null;
    }

    try {
      final json = await _service.exportConfiguration(_currentConfig!);
      _clearError();
      return json;
    } catch (e) {
      _setError('Failed to export configuration: $e');
      return null;
    }
  }

  // Import configuration
  Future<void> importConfiguration(String jsonData) async {
    _setLoading(true);
    try {
      _currentConfig = await _service.importConfiguration(jsonData);
      _selectedDeploymentType = _currentConfig?.deploymentType;
      await _saveSettings();
      await validateConfiguration();
      _clearError();
    } catch (e) {
      _setError('Failed to import configuration: $e');
    } finally {
      _setLoading(false);
    }
  }

  // Update system information manually
  Future<void> updateSystemInfo(SystemInfo systemInfo) async {
    _systemInfo = systemInfo;
    _autoDetectionCompleted = false;
    notifyListeners();
  }

  // Reset configuration
  void resetConfiguration() {
    _currentConfig = null;
    _selectedDeploymentType = null;
    _validationResult = null;
    _recommendation = null;
    _autoDetectionCompleted = false;
    _clearError();
    _saveSettings();
  }

  // Get deployment type color
  Color getDeploymentTypeColor(DeploymentType type) {
    switch (type) {
      case DeploymentType.Development:
        return const Color(0xFF10B981); // Green
      case DeploymentType.Production:
        return const Color(0xFF3B82F6); // Blue
      case DeploymentType.Grid:
        return const Color(0xFF8B5CF6); // Purple
    }
  }

  // Get deployment type name
  String getDeploymentTypeName(DeploymentType type) {
    switch (type) {
      case DeploymentType.Development:
        return 'Development';
      case DeploymentType.Production:
        return 'Production';
      case DeploymentType.Grid:
        return 'Grid';
    }
  }

  // Get confidence level description
  String getConfidenceDescription(double confidence) {
    if (confidence >= 0.8) return 'High Confidence';
    if (confidence >= 0.6) return 'Medium Confidence';
    return 'Low Confidence';
  }

  // Get validation score color
  Color getValidationScoreColor(int score) {
    if (score >= 90) return const Color(0xFF10B981); // Green
    if (score >= 70) return const Color(0xFFFBBF24); // Yellow
    if (score >= 50) return const Color(0xFFF97316); // Orange
    return const Color(0xFFEF4444); // Red
  }

  // Get server status color
  Color getServerStatusColor() {
    if (_serverStatus?.isRunning == true) {
      return const Color(0xFF10B981); // Green
    }
    return const Color(0xFFEF4444); // Red
  }

  // Format uptime
  String formatUptime(int seconds) {
    final duration = Duration(seconds: seconds);
    final days = duration.inDays;
    final hours = duration.inHours % 24;
    final minutes = duration.inMinutes % 60;
    
    if (days > 0) {
      return '${days}d ${hours}h ${minutes}m';
    } else if (hours > 0) {
      return '${hours}h ${minutes}m';
    } else {
      return '${minutes}m';
    }
  }

  // Format bytes
  String formatBytes(int bytes) {
    if (bytes < 1024) return '${bytes}B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)}KB';
    if (bytes < 1024 * 1024 * 1024) return '${(bytes / (1024 * 1024)).toStringAsFixed(1)}MB';
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)}GB';
  }

  // Flutter Web Auto-Configurator Extensions
  bool _isExpertMode = false;
  bool _isAutoDetectLanguage = true;
  Language? _currentLanguage;
  AutoDetectionResults? _autoDetectionResults;
  List<ConfigurationRecommendation> _recommendations = [];

  // New getters for auto-configurator
  bool get isExpertMode => _isExpertMode;
  bool get isAutoDetectLanguage => _isAutoDetectLanguage;
  Language? get currentLanguage => _currentLanguage;
  AutoDetectionResults? get autoDetectionResults => _autoDetectionResults;
  double get overallProgress => _calculateOverallProgress();
  double get securityScore => _calculateSecurityScore();
  String get validationStatus => _getValidationStatus();

  // Configuration dashboard methods
  ConfigurationStatus getSectionStatus(String sectionId) {
    if (_currentConfig == null) return ConfigurationStatus.pending;
    
    switch (sectionId) {
      case 'deployment':
        return _selectedDeploymentType != null 
          ? ConfigurationStatus.completed 
          : ConfigurationStatus.pending;
      case 'security':
        return _hasSecurityWarnings() 
          ? ConfigurationStatus.warning 
          : ConfigurationStatus.pending;
      default:
        return ConfigurationStatus.pending;
    }
  }

  double getSectionProgress(String sectionId) {
    switch (sectionId) {
      case 'deployment':
        return _selectedDeploymentType != null ? 1.0 : 0.0;
      case 'environment':
        return 0.6; // Mock progress
      default:
        return 0.0;
    }
  }

  List<ConfigurationRecommendation> getRecommendations() {
    return _recommendations;
  }

  void dismissRecommendation(String id) {
    _recommendations.removeWhere((rec) => rec.id == id);
    notifyListeners();
  }

  void dismissAllRecommendations() {
    _recommendations.clear();
    notifyListeners();
  }

  void applyRecommendation(String id) {
    // Implementation for applying specific recommendations
    dismissRecommendation(id);
  }

  // Language management
  void changeLanguage(Language? language) {
    _currentLanguage = language;
    _isAutoDetectLanguage = language == null;
    notifyListeners();
  }

  void enableAutoDetectLanguage() {
    _isAutoDetectLanguage = true;
    _currentLanguage = null;
    notifyListeners();
  }

  // Expert mode toggle
  void toggleExpertMode() {
    _isExpertMode = !_isExpertMode;
    notifyListeners();
  }


  // Template loading
  void loadTemplate(String templateType) {
    switch (templateType) {
      case 'development':
        _selectedDeploymentType = DeploymentType.Development;
        break;
      case 'production':
        _selectedDeploymentType = DeploymentType.Production;
        break;
      case 'grid':
        _selectedDeploymentType = DeploymentType.Grid;
        break;
    }
    
    // Simulate loading template configuration
    _generateRecommendations();
    notifyListeners();
  }

  // Helper methods
  double _calculateOverallProgress() {
    double progress = 0.0;
    int sections = 6;
    
    if (_selectedDeploymentType != null) progress += 1.0;
    if (_currentConfig != null) progress += 2.0;
    
    return progress / sections;
  }

  double _calculateSecurityScore() {
    if (_currentConfig == null) return 0.3;
    
    double score = 0.5; // Base score
    if (_currentConfig!.securityConfig.passwordComplexity) score += 0.2;
    if (_currentConfig!.securityConfig.bruteForceProtection) score += 0.2;
    if (_currentConfig!.securityConfig.sslCertificatePath.isNotEmpty) score += 0.1;
    
    return score.clamp(0.0, 1.0);
  }

  String _getValidationStatus() {
    if (_validationResult == null) return 'Pending';
    if (_validationResult!.isValid) return 'Complete';
    if (_validationResult!.warnings.isNotEmpty) return 'Warning';
    return 'Error';
  }

  bool _hasSecurityWarnings() {
    if (_currentConfig == null) return true;
    return _currentConfig!.securityConfig.sslCertificatePath.isEmpty;
  }

  void _generateRecommendations() {
    _recommendations.clear();
    
    if (_selectedDeploymentType == DeploymentType.Development) {
      _recommendations.add(
        ConfigurationRecommendation(
          id: 'ssl-cert',
          title: 'SSL Certificate Configuration',
          description: 'Configure SSL certificate for secure connections, even in development',
          priority: RecommendationPriority.medium,
          actionText: 'Configure SSL',
        ),
      );
    }
    
    if (_currentConfig?.databaseType == 'SQLite') {
      _recommendations.add(
        ConfigurationRecommendation(
          id: 'backup-strategy',
          title: 'Database Backup Strategy',
          description: 'Set up automated backups to prevent data loss',
          priority: RecommendationPriority.high,
          actionText: 'Setup Backups',
        ),
      );
    }
  }

  // Private helper methods
  void _setLoading(bool loading) {
    _isLoading = loading;
    notifyListeners();
  }

  void _setError(String error) {
    _errorMessage = error;
    _isLoading = false;
    notifyListeners();
  }

  void _clearError() {
    _errorMessage = null;
    notifyListeners();
  }

  @override
  void dispose() {
    _service.dispose();
    super.dispose();
  }
}