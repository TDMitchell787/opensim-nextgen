// OpenSim Next Configurator Models
// Data models matching Rust FFI structures

import 'package:json_annotation/json_annotation.dart';

part 'deployment_models.g.dart';

/// Deployment types supported by OpenSim Next
enum DeploymentType {
  @JsonValue('Development')
  Development,
  @JsonValue('Production')
  Production,
  @JsonValue('Grid')
  Grid,
}

/// System information for auto-detection
@JsonSerializable()
class SystemInfo {
  final double memoryGb;
  final int cpuCores;
  final bool hasPublicIp;
  final int bandwidthMbps;
  final String domain;
  final int expectedUsers;
  final int expectedRegions;
  final bool isCommercial;

  SystemInfo({
    required this.memoryGb,
    required this.cpuCores,
    required this.hasPublicIp,
    required this.bandwidthMbps,
    required this.domain,
    required this.expectedUsers,
    required this.expectedRegions,
    required this.isCommercial,
  });

  factory SystemInfo.fromJson(Map<String, dynamic> json) => _$SystemInfoFromJson(json);
  Map<String, dynamic> toJson() => _$SystemInfoToJson(this);
}

/// Alternative deployment option
@JsonSerializable()
class AlternativeOption {
  final DeploymentType deploymentType;
  final double confidence;
  final String reason;

  AlternativeOption({
    required this.deploymentType,
    required this.confidence,
    required this.reason,
  });

  factory AlternativeOption.fromJson(Map<String, dynamic> json) => _$AlternativeOptionFromJson(json);
  Map<String, dynamic> toJson() => _$AlternativeOptionToJson(this);
}

/// Deployment recommendation result
@JsonSerializable()
class DeploymentRecommendation {
  final DeploymentType recommendedType;
  final double confidence;
  final String reasoning;
  final List<AlternativeOption> alternativeOptions;

  DeploymentRecommendation({
    required this.recommendedType,
    required this.confidence,
    required this.reasoning,
    required this.alternativeOptions,
  });

  factory DeploymentRecommendation.fromJson(Map<String, dynamic> json) => _$DeploymentRecommendationFromJson(json);
  Map<String, dynamic> toJson() => _$DeploymentRecommendationToJson(this);
}

/// Configuration validation result
@JsonSerializable()
class ValidationResult {
  final bool isValid;
  final List<String> errors;
  final List<String> warnings;
  final List<String> recommendations;
  final int overallScore;

  ValidationResult({
    required this.isValid,
    required this.errors,
    required this.warnings,
    required this.recommendations,
    required this.overallScore,
  });

  factory ValidationResult.fromJson(Map<String, dynamic> json) => _$ValidationResultFromJson(json);
  Map<String, dynamic> toJson() => _$ValidationResultToJson(this);
}

/// Network configuration
@JsonSerializable()
class NetworkConfig {
  final int httpPort;
  final int httpsPort;
  final bool httpsEnabled;
  final String externalHostname;
  final String internalIp;

  NetworkConfig({
    required this.httpPort,
    required this.httpsPort,
    required this.httpsEnabled,
    required this.externalHostname,
    required this.internalIp,
  });

  NetworkConfig copyWith({
    int? httpPort,
    int? httpsPort,
    bool? httpsEnabled,
    String? externalHostname,
    String? internalIp,
  }) => NetworkConfig(
    httpPort: httpPort ?? this.httpPort,
    httpsPort: httpsPort ?? this.httpsPort,
    httpsEnabled: httpsEnabled ?? this.httpsEnabled,
    externalHostname: externalHostname ?? this.externalHostname,
    internalIp: internalIp ?? this.internalIp,
  );

  factory NetworkConfig.fromJson(Map<String, dynamic> json) => _$NetworkConfigFromJson(json);
  Map<String, dynamic> toJson() => _$NetworkConfigToJson(this);
}

/// Security configuration
@JsonSerializable()
class SecurityConfig {
  final bool passwordComplexity;
  final int sessionTimeout;
  final bool bruteForceProtection;
  final String sslCertificatePath;
  final String sslPrivateKeyPath;

  SecurityConfig({
    required this.passwordComplexity,
    required this.sessionTimeout,
    required this.bruteForceProtection,
    required this.sslCertificatePath,
    required this.sslPrivateKeyPath,
  });

  SecurityConfig copyWith({
    bool? passwordComplexity,
    int? sessionTimeout,
    bool? bruteForceProtection,
    String? sslCertificatePath,
    String? sslPrivateKeyPath,
  }) => SecurityConfig(
    passwordComplexity: passwordComplexity ?? this.passwordComplexity,
    sessionTimeout: sessionTimeout ?? this.sessionTimeout,
    bruteForceProtection: bruteForceProtection ?? this.bruteForceProtection,
    sslCertificatePath: sslCertificatePath ?? this.sslCertificatePath,
    sslPrivateKeyPath: sslPrivateKeyPath ?? this.sslPrivateKeyPath,
  );

  factory SecurityConfig.fromJson(Map<String, dynamic> json) => _$SecurityConfigFromJson(json);
  Map<String, dynamic> toJson() => _$SecurityConfigToJson(this);
}

/// Performance configuration
@JsonSerializable()
class PerformanceConfig {
  final int maxPrims;
  final int maxScripts;
  final int scriptTimeout;
  final bool cacheAssets;
  final int cacheTimeout;

  PerformanceConfig({
    required this.maxPrims,
    required this.maxScripts,
    required this.scriptTimeout,
    required this.cacheAssets,
    required this.cacheTimeout,
  });

  PerformanceConfig copyWith({
    int? maxPrims,
    int? maxScripts,
    int? scriptTimeout,
    bool? cacheAssets,
    int? cacheTimeout,
  }) => PerformanceConfig(
    maxPrims: maxPrims ?? this.maxPrims,
    maxScripts: maxScripts ?? this.maxScripts,
    scriptTimeout: scriptTimeout ?? this.scriptTimeout,
    cacheAssets: cacheAssets ?? this.cacheAssets,
    cacheTimeout: cacheTimeout ?? this.cacheTimeout,
  );

  factory PerformanceConfig.fromJson(Map<String, dynamic> json) => _$PerformanceConfigFromJson(json);
  Map<String, dynamic> toJson() => _$PerformanceConfigToJson(this);
}

/// OpenSim configuration structure
@JsonSerializable()
class OpenSimConfig {
  final DeploymentType deploymentType;
  final String gridName;
  final String gridNick;
  final String welcomeMessage;
  final String databaseType;
  final String databaseConnection;
  final String physicsEngine;
  final NetworkConfig networkConfig;
  final SecurityConfig securityConfig;
  final PerformanceConfig performanceConfig;

  OpenSimConfig({
    required this.deploymentType,
    required this.gridName,
    required this.gridNick,
    required this.welcomeMessage,
    required this.databaseType,
    required this.databaseConnection,
    required this.physicsEngine,
    required this.networkConfig,
    required this.securityConfig,
    required this.performanceConfig,
  });

  OpenSimConfig copyWith({
    DeploymentType? deploymentType,
    String? gridName,
    String? gridNick,
    String? welcomeMessage,
    String? databaseType,
    String? databaseConnection,
    String? physicsEngine,
    NetworkConfig? networkConfig,
    SecurityConfig? securityConfig,
    PerformanceConfig? performanceConfig,
  }) => OpenSimConfig(
    deploymentType: deploymentType ?? this.deploymentType,
    gridName: gridName ?? this.gridName,
    gridNick: gridNick ?? this.gridNick,
    welcomeMessage: welcomeMessage ?? this.welcomeMessage,
    databaseType: databaseType ?? this.databaseType,
    databaseConnection: databaseConnection ?? this.databaseConnection,
    physicsEngine: physicsEngine ?? this.physicsEngine,
    networkConfig: networkConfig ?? this.networkConfig,
    securityConfig: securityConfig ?? this.securityConfig,
    performanceConfig: performanceConfig ?? this.performanceConfig,
  );

  factory OpenSimConfig.fromJson(Map<String, dynamic> json) => _$OpenSimConfigFromJson(json);
  Map<String, dynamic> toJson() => _$OpenSimConfigToJson(this);
}

/// Network activity information
@JsonSerializable()
class NetworkActivity {
  final int bytesSent;
  final int bytesReceived;
  final int connections;

  NetworkActivity({
    required this.bytesSent,
    required this.bytesReceived,
    required this.connections,
  });

  factory NetworkActivity.fromJson(Map<String, dynamic> json) => _$NetworkActivityFromJson(json);
  Map<String, dynamic> toJson() => _$NetworkActivityToJson(this);
}

/// Server status information
@JsonSerializable()
class ServerStatus {
  final bool isRunning;
  final int uptimeSeconds;
  final int activeRegions;
  final int connectedUsers;
  final double cpuUsage;
  final double memoryUsage;
  final NetworkActivity networkActivity;

  ServerStatus({
    required this.isRunning,
    required this.uptimeSeconds,
    required this.activeRegions,
    required this.connectedUsers,
    required this.cpuUsage,
    required this.memoryUsage,
    required this.networkActivity,
  });

  factory ServerStatus.fromJson(Map<String, dynamic> json) => _$ServerStatusFromJson(json);
  Map<String, dynamic> toJson() => _$ServerStatusToJson(this);
}

/// Architecture mode - how services are distributed
enum ArchitectureMode {
  @JsonValue('standalone')
  standalone,
  @JsonValue('gridserver')
  gridServer,
  @JsonValue('regionserver')
  regionServer,
}

extension ArchitectureModeExtension on ArchitectureMode {
  String get displayName {
    switch (this) {
      case ArchitectureMode.standalone:
        return 'Standalone';
      case ArchitectureMode.gridServer:
        return 'Grid Server';
      case ArchitectureMode.regionServer:
        return 'Region Server';
    }
  }

  String get description {
    switch (this) {
      case ArchitectureMode.standalone:
        return 'All services on a single server - ideal for personal grids or testing';
      case ArchitectureMode.gridServer:
        return 'Central services (users, inventory, assets) - the hub of a multi-region grid';
      case ArchitectureMode.regionServer:
        return 'Region simulator connecting to a grid server - one per region';
    }
  }

  bool get needsUserService => this == ArchitectureMode.standalone || this == ArchitectureMode.gridServer;
  bool get needsInventoryService => this == ArchitectureMode.standalone || this == ArchitectureMode.gridServer;
  bool get needsAssetService => this == ArchitectureMode.standalone || this == ArchitectureMode.gridServer;
  bool get needsRegionTables => this == ArchitectureMode.standalone || this == ArchitectureMode.regionServer;
  bool get needsGridServerUrl => this == ArchitectureMode.regionServer;
  bool get needsGridName => this == ArchitectureMode.gridServer;
  bool get needsRegionName => this == ArchitectureMode.regionServer;

  String get icon {
    switch (this) {
      case ArchitectureMode.standalone:
        return '🖥️';
      case ArchitectureMode.gridServer:
        return '🌐';
      case ArchitectureMode.regionServer:
        return '🗺️';
    }
  }

  List<String> get features {
    switch (this) {
      case ArchitectureMode.standalone:
        return [
          'All services local (users, inventory, assets)',
          'Region simulator built-in',
          'Single database for everything',
          'Simplest to set up and maintain',
          'Ideal for personal or small grids',
        ];
      case ArchitectureMode.gridServer:
        return [
          'Central user authentication',
          'Shared inventory and assets',
          'Region registry for grid map',
          'Presence and friends tracking',
          'Scalable to many region servers',
        ];
      case ArchitectureMode.regionServer:
        return [
          'Connects to central grid server',
          'Local prim and terrain storage',
          'Region-specific settings',
          'Can run multiple regions',
          'Lightweight resource usage',
        ];
    }
  }
}

/// Service URLs for grid architecture
@JsonSerializable()
class ServiceUrls {
  final String userAccount;
  final String inventory;
  final String asset;
  final String avatar;
  final String gridUser;
  final String presence;
  final String friends;
  final String grid;

  ServiceUrls({
    required this.userAccount,
    required this.inventory,
    required this.asset,
    required this.avatar,
    required this.gridUser,
    required this.presence,
    required this.friends,
    required this.grid,
  });

  factory ServiceUrls.localhost({int port = 9000}) {
    final baseUrl = 'http://localhost:$port';
    return ServiceUrls(
      userAccount: baseUrl,
      inventory: baseUrl,
      asset: baseUrl,
      avatar: baseUrl,
      gridUser: baseUrl,
      presence: baseUrl,
      friends: baseUrl,
      grid: baseUrl,
    );
  }

  factory ServiceUrls.forGrid(String gridServerUrl) {
    return ServiceUrls(
      userAccount: '$gridServerUrl/accounts',
      inventory: '$gridServerUrl/inventory',
      asset: '$gridServerUrl/assets',
      avatar: '$gridServerUrl/avatar',
      gridUser: '$gridServerUrl/griduser',
      presence: '$gridServerUrl/presence',
      friends: '$gridServerUrl/friends',
      grid: '$gridServerUrl/grid',
    );
  }

  String toServiceUrlsString() {
    return 'HomeURI=$grid&GatekeeperURI=$grid&InventoryServerURI=$inventory&AssetServerURI=$asset';
  }

  factory ServiceUrls.fromJson(Map<String, dynamic> json) => _$ServiceUrlsFromJson(json);
  Map<String, dynamic> toJson() => _$ServiceUrlsToJson(this);
}

/// Grid information for viewers
@JsonSerializable()
class GridInfo {
  final String gridName;
  final String gridNick;
  final String loginUri;
  final String welcomeUri;
  final String economyUri;
  final String aboutUri;
  final String registerUri;
  final String passwordUri;
  final String gridStatus;

  GridInfo({
    required this.gridName,
    required this.gridNick,
    required this.loginUri,
    required this.welcomeUri,
    required this.economyUri,
    required this.aboutUri,
    required this.registerUri,
    required this.passwordUri,
    required this.gridStatus,
  });

  factory GridInfo.defaults({int port = 9000}) {
    final baseUrl = 'http://localhost:$port';
    return GridInfo(
      gridName: 'OpenSim NextGen',
      gridNick: 'nextgen',
      loginUri: baseUrl,
      welcomeUri: '$baseUrl/welcome',
      economyUri: '$baseUrl/economy',
      aboutUri: '$baseUrl/about',
      registerUri: '$baseUrl/register',
      passwordUri: '$baseUrl/password',
      gridStatus: '$baseUrl/status',
    );
  }

  factory GridInfo.withBaseUrl(String baseUrl, String name, String nick) {
    return GridInfo(
      gridName: name,
      gridNick: nick,
      loginUri: baseUrl,
      welcomeUri: '$baseUrl/welcome',
      economyUri: '$baseUrl/economy',
      aboutUri: '$baseUrl/about',
      registerUri: '$baseUrl/register',
      passwordUri: '$baseUrl/password',
      gridStatus: '$baseUrl/status',
    );
  }

  factory GridInfo.fromJson(Map<String, dynamic> json) => _$GridInfoFromJson(json);
  Map<String, dynamic> toJson() => _$GridInfoToJson(this);
}

/// Architecture-specific deployment configuration
@JsonSerializable()
class ArchitectureConfig {
  final ArchitectureMode mode;
  final String databaseUrl;
  final ServiceUrls serviceUrls;
  final GridInfo gridInfo;
  final String? gridServerUrl;
  final String? regionName;
  final String? regionUuid;

  ArchitectureConfig({
    required this.mode,
    required this.databaseUrl,
    required this.serviceUrls,
    required this.gridInfo,
    this.gridServerUrl,
    this.regionName,
    this.regionUuid,
  });

  factory ArchitectureConfig.standalone({
    required String databaseUrl,
    int port = 9000,
  }) {
    return ArchitectureConfig(
      mode: ArchitectureMode.standalone,
      databaseUrl: databaseUrl,
      serviceUrls: ServiceUrls.localhost(port: port),
      gridInfo: GridInfo.defaults(port: port),
    );
  }

  factory ArchitectureConfig.gridServer({
    required String databaseUrl,
    required String gridName,
    required String baseUrl,
  }) {
    final nick = gridName.toLowerCase().replaceAll(' ', '');
    return ArchitectureConfig(
      mode: ArchitectureMode.gridServer,
      databaseUrl: databaseUrl,
      serviceUrls: ServiceUrls.localhost(),
      gridInfo: GridInfo.withBaseUrl(baseUrl, gridName, nick),
    );
  }

  factory ArchitectureConfig.regionServer({
    required String databaseUrl,
    required String gridServerUrl,
    required String regionName,
  }) {
    return ArchitectureConfig(
      mode: ArchitectureMode.regionServer,
      databaseUrl: databaseUrl,
      serviceUrls: ServiceUrls.forGrid(gridServerUrl),
      gridInfo: GridInfo.defaults(),
      gridServerUrl: gridServerUrl,
      regionName: regionName,
    );
  }

  List<String> validate() {
    final errors = <String>[];

    if (databaseUrl.isEmpty) {
      errors.add('Database URL is required');
    }

    if (mode == ArchitectureMode.regionServer) {
      if (gridServerUrl == null || gridServerUrl!.isEmpty) {
        errors.add('Grid server URL is required for region server mode');
      }
      if (regionName == null || regionName!.isEmpty) {
        errors.add('Region name is required for region server mode');
      }
    }

    if (mode == ArchitectureMode.gridServer) {
      if (gridInfo.gridName.isEmpty) {
        errors.add('Grid name is required for grid server mode');
      }
    }

    return errors;
  }

  bool get isValid => validate().isEmpty;

  factory ArchitectureConfig.fromJson(Map<String, dynamic> json) => _$ArchitectureConfigFromJson(json);
  Map<String, dynamic> toJson() => _$ArchitectureConfigToJson(this);
}

/// Architecture mode information for UI display
class ArchitectureModeInfo {
  final ArchitectureMode mode;
  final String name;
  final String description;
  final String icon;
  final List<String> features;
  final Map<String, String> requirements;
  final String setupTime;

  ArchitectureModeInfo({
    required this.mode,
    required this.name,
    required this.description,
    required this.icon,
    required this.features,
    required this.requirements,
    required this.setupTime,
  });

  static List<ArchitectureModeInfo> getAllModes() {
    return [
      ArchitectureModeInfo(
        mode: ArchitectureMode.standalone,
        name: 'Standalone',
        description: 'All services on a single server',
        icon: '🖥️',
        features: [
          'All services local (users, inventory, assets)',
          'Region simulator built-in',
          'Single database for everything',
          'Simplest to set up and maintain',
        ],
        requirements: {
          'Database': 'PostgreSQL or SQLite',
          'Storage': '10GB+ recommended',
          'Memory': '4GB+ RAM',
        },
        setupTime: '15-30 minutes',
      ),
      ArchitectureModeInfo(
        mode: ArchitectureMode.gridServer,
        name: 'Grid Server',
        description: 'Central hub for multi-region grid',
        icon: '🌐',
        features: [
          'Central user authentication',
          'Shared inventory and assets',
          'Region registry for grid map',
          'Presence and friends tracking',
        ],
        requirements: {
          'Database': 'PostgreSQL recommended',
          'Storage': '100GB+ for assets',
          'Network': 'Public IP or domain',
        },
        setupTime: '30-60 minutes',
      ),
      ArchitectureModeInfo(
        mode: ArchitectureMode.regionServer,
        name: 'Region Server',
        description: 'Region simulator connecting to grid',
        icon: '🗺️',
        features: [
          'Connects to central grid server',
          'Local prim and terrain storage',
          'Region-specific settings',
          'Lightweight resource usage',
        ],
        requirements: {
          'Grid Server': 'URL to existing grid',
          'Database': 'SQLite or PostgreSQL',
          'Storage': '5GB+ for region data',
        },
        setupTime: '10-20 minutes',
      ),
    ];
  }

  static ArchitectureModeInfo? getInfo(ArchitectureMode mode) {
    return getAllModes().firstWhere((info) => info.mode == mode);
  }
}

/// Deployment type information for UI display
class DeploymentTypeInfo {
  final DeploymentType type;
  final String name;
  final String description;
  final String icon;
  final List<String> features;
  final Map<String, String> specs;
  final String setupTime;
  final String complexity;

  DeploymentTypeInfo({
    required this.type,
    required this.name,
    required this.description,
    required this.icon,
    required this.features,
    required this.specs,
    required this.setupTime,
    required this.complexity,
  });

  static List<DeploymentTypeInfo> getAllTypes() {
    return [
      DeploymentTypeInfo(
        type: DeploymentType.Development,
        name: 'Development Environment',
        description: 'Optimized for rapid development, testing, and learning',
        icon: '💻',
        features: [
          'SQLite database (no setup required)',
          'ODE physics engine (stable)',
          'Local network only',
          'Basic security for development',
          'Quick setup and deployment',
        ],
        specs: {
          'Users': '1-10 concurrent',
          'Regions': '1-4 regions',
          'Hardware': '4 cores, 8GB RAM',
          'Network': '100Mbps',
        },
        setupTime: '15-30 minutes',
        complexity: 'Low',
      ),
      DeploymentTypeInfo(
        type: DeploymentType.Production,
        name: 'Production Environment',
        description: 'Battle-tested configuration for live virtual worlds',
        icon: '🏢',
        features: [
          'PostgreSQL database (high performance)',
          'Bullet/UBODE physics engines',
          'SSL/TLS security',
          'Professional monitoring',
          'Load balancing capabilities',
        ],
        specs: {
          'Users': '10-500 concurrent',
          'Regions': '4-32 regions',
          'Hardware': '16 cores, 32GB RAM',
          'Network': '1Gbps dedicated',
        },
        setupTime: '2-4 hours',
        complexity: 'Medium',
      ),
      DeploymentTypeInfo(
        type: DeploymentType.Grid,
        name: 'Grid Environment',
        description: 'Distributed multi-server architecture for massive scale',
        icon: '🌐',
        features: [
          'PostgreSQL clustering',
          'POS physics with GPU acceleration',
          'Zero trust networking (OpenZiti)',
          'Enterprise security & monitoring',
          'Geographic distribution',
        ],
        specs: {
          'Users': '100-10,000+ concurrent',
          'Regions': '32-1000+ regions',
          'Hardware': '64+ cores, 128GB+ RAM',
          'Network': '10Gbps+ fiber',
        },
        setupTime: '1-2 days',
        complexity: 'High',
      ),
    ];
  }

  static DeploymentTypeInfo? getInfo(DeploymentType type) {
    return getAllTypes().firstWhere((info) => info.type == type);
  }
}