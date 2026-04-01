// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'deployment_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SystemInfo _$SystemInfoFromJson(Map<String, dynamic> json) => SystemInfo(
  memoryGb: (json['memoryGb'] as num).toDouble(),
  cpuCores: (json['cpuCores'] as num).toInt(),
  hasPublicIp: json['hasPublicIp'] as bool,
  bandwidthMbps: (json['bandwidthMbps'] as num).toInt(),
  domain: json['domain'] as String,
  expectedUsers: (json['expectedUsers'] as num).toInt(),
  expectedRegions: (json['expectedRegions'] as num).toInt(),
  isCommercial: json['isCommercial'] as bool,
);

Map<String, dynamic> _$SystemInfoToJson(SystemInfo instance) =>
    <String, dynamic>{
      'memoryGb': instance.memoryGb,
      'cpuCores': instance.cpuCores,
      'hasPublicIp': instance.hasPublicIp,
      'bandwidthMbps': instance.bandwidthMbps,
      'domain': instance.domain,
      'expectedUsers': instance.expectedUsers,
      'expectedRegions': instance.expectedRegions,
      'isCommercial': instance.isCommercial,
    };

AlternativeOption _$AlternativeOptionFromJson(Map<String, dynamic> json) =>
    AlternativeOption(
      deploymentType: $enumDecode(
        _$DeploymentTypeEnumMap,
        json['deploymentType'],
      ),
      confidence: (json['confidence'] as num).toDouble(),
      reason: json['reason'] as String,
    );

Map<String, dynamic> _$AlternativeOptionToJson(AlternativeOption instance) =>
    <String, dynamic>{
      'deploymentType': _$DeploymentTypeEnumMap[instance.deploymentType]!,
      'confidence': instance.confidence,
      'reason': instance.reason,
    };

const _$DeploymentTypeEnumMap = {
  DeploymentType.Development: 'Development',
  DeploymentType.Production: 'Production',
  DeploymentType.Grid: 'Grid',
};

DeploymentRecommendation _$DeploymentRecommendationFromJson(
  Map<String, dynamic> json,
) => DeploymentRecommendation(
  recommendedType: $enumDecode(
    _$DeploymentTypeEnumMap,
    json['recommendedType'],
  ),
  confidence: (json['confidence'] as num).toDouble(),
  reasoning: json['reasoning'] as String,
  alternativeOptions:
      (json['alternativeOptions'] as List<dynamic>)
          .map((e) => AlternativeOption.fromJson(e as Map<String, dynamic>))
          .toList(),
);

Map<String, dynamic> _$DeploymentRecommendationToJson(
  DeploymentRecommendation instance,
) => <String, dynamic>{
  'recommendedType': _$DeploymentTypeEnumMap[instance.recommendedType]!,
  'confidence': instance.confidence,
  'reasoning': instance.reasoning,
  'alternativeOptions': instance.alternativeOptions,
};

ValidationResult _$ValidationResultFromJson(Map<String, dynamic> json) =>
    ValidationResult(
      isValid: json['isValid'] as bool,
      errors:
          (json['errors'] as List<dynamic>).map((e) => e as String).toList(),
      warnings:
          (json['warnings'] as List<dynamic>).map((e) => e as String).toList(),
      recommendations:
          (json['recommendations'] as List<dynamic>)
              .map((e) => e as String)
              .toList(),
      overallScore: (json['overallScore'] as num).toInt(),
    );

Map<String, dynamic> _$ValidationResultToJson(ValidationResult instance) =>
    <String, dynamic>{
      'isValid': instance.isValid,
      'errors': instance.errors,
      'warnings': instance.warnings,
      'recommendations': instance.recommendations,
      'overallScore': instance.overallScore,
    };

NetworkConfig _$NetworkConfigFromJson(Map<String, dynamic> json) =>
    NetworkConfig(
      httpPort: (json['httpPort'] as num).toInt(),
      httpsPort: (json['httpsPort'] as num).toInt(),
      httpsEnabled: json['httpsEnabled'] as bool,
      externalHostname: json['externalHostname'] as String,
      internalIp: json['internalIp'] as String,
    );

Map<String, dynamic> _$NetworkConfigToJson(NetworkConfig instance) =>
    <String, dynamic>{
      'httpPort': instance.httpPort,
      'httpsPort': instance.httpsPort,
      'httpsEnabled': instance.httpsEnabled,
      'externalHostname': instance.externalHostname,
      'internalIp': instance.internalIp,
    };

SecurityConfig _$SecurityConfigFromJson(Map<String, dynamic> json) =>
    SecurityConfig(
      passwordComplexity: json['passwordComplexity'] as bool,
      sessionTimeout: (json['sessionTimeout'] as num).toInt(),
      bruteForceProtection: json['bruteForceProtection'] as bool,
      sslCertificatePath: json['sslCertificatePath'] as String,
      sslPrivateKeyPath: json['sslPrivateKeyPath'] as String,
    );

Map<String, dynamic> _$SecurityConfigToJson(SecurityConfig instance) =>
    <String, dynamic>{
      'passwordComplexity': instance.passwordComplexity,
      'sessionTimeout': instance.sessionTimeout,
      'bruteForceProtection': instance.bruteForceProtection,
      'sslCertificatePath': instance.sslCertificatePath,
      'sslPrivateKeyPath': instance.sslPrivateKeyPath,
    };

PerformanceConfig _$PerformanceConfigFromJson(Map<String, dynamic> json) =>
    PerformanceConfig(
      maxPrims: (json['maxPrims'] as num).toInt(),
      maxScripts: (json['maxScripts'] as num).toInt(),
      scriptTimeout: (json['scriptTimeout'] as num).toInt(),
      cacheAssets: json['cacheAssets'] as bool,
      cacheTimeout: (json['cacheTimeout'] as num).toInt(),
    );

Map<String, dynamic> _$PerformanceConfigToJson(PerformanceConfig instance) =>
    <String, dynamic>{
      'maxPrims': instance.maxPrims,
      'maxScripts': instance.maxScripts,
      'scriptTimeout': instance.scriptTimeout,
      'cacheAssets': instance.cacheAssets,
      'cacheTimeout': instance.cacheTimeout,
    };

OpenSimConfig _$OpenSimConfigFromJson(Map<String, dynamic> json) =>
    OpenSimConfig(
      deploymentType: $enumDecode(
        _$DeploymentTypeEnumMap,
        json['deploymentType'],
      ),
      gridName: json['gridName'] as String,
      gridNick: json['gridNick'] as String,
      welcomeMessage: json['welcomeMessage'] as String,
      databaseType: json['databaseType'] as String,
      databaseConnection: json['databaseConnection'] as String,
      physicsEngine: json['physicsEngine'] as String,
      networkConfig: NetworkConfig.fromJson(
        json['networkConfig'] as Map<String, dynamic>,
      ),
      securityConfig: SecurityConfig.fromJson(
        json['securityConfig'] as Map<String, dynamic>,
      ),
      performanceConfig: PerformanceConfig.fromJson(
        json['performanceConfig'] as Map<String, dynamic>,
      ),
    );

Map<String, dynamic> _$OpenSimConfigToJson(OpenSimConfig instance) =>
    <String, dynamic>{
      'deploymentType': _$DeploymentTypeEnumMap[instance.deploymentType]!,
      'gridName': instance.gridName,
      'gridNick': instance.gridNick,
      'welcomeMessage': instance.welcomeMessage,
      'databaseType': instance.databaseType,
      'databaseConnection': instance.databaseConnection,
      'physicsEngine': instance.physicsEngine,
      'networkConfig': instance.networkConfig,
      'securityConfig': instance.securityConfig,
      'performanceConfig': instance.performanceConfig,
    };

NetworkActivity _$NetworkActivityFromJson(Map<String, dynamic> json) =>
    NetworkActivity(
      bytesSent: (json['bytesSent'] as num).toInt(),
      bytesReceived: (json['bytesReceived'] as num).toInt(),
      connections: (json['connections'] as num).toInt(),
    );

Map<String, dynamic> _$NetworkActivityToJson(NetworkActivity instance) =>
    <String, dynamic>{
      'bytesSent': instance.bytesSent,
      'bytesReceived': instance.bytesReceived,
      'connections': instance.connections,
    };

ServerStatus _$ServerStatusFromJson(Map<String, dynamic> json) => ServerStatus(
  isRunning: json['isRunning'] as bool,
  uptimeSeconds: (json['uptimeSeconds'] as num).toInt(),
  activeRegions: (json['activeRegions'] as num).toInt(),
  connectedUsers: (json['connectedUsers'] as num).toInt(),
  cpuUsage: (json['cpuUsage'] as num).toDouble(),
  memoryUsage: (json['memoryUsage'] as num).toDouble(),
  networkActivity: NetworkActivity.fromJson(
    json['networkActivity'] as Map<String, dynamic>,
  ),
);

Map<String, dynamic> _$ServerStatusToJson(ServerStatus instance) =>
    <String, dynamic>{
      'isRunning': instance.isRunning,
      'uptimeSeconds': instance.uptimeSeconds,
      'activeRegions': instance.activeRegions,
      'connectedUsers': instance.connectedUsers,
      'cpuUsage': instance.cpuUsage,
      'memoryUsage': instance.memoryUsage,
      'networkActivity': instance.networkActivity,
    };

ServiceUrls _$ServiceUrlsFromJson(Map<String, dynamic> json) => ServiceUrls(
  userAccount: json['userAccount'] as String,
  inventory: json['inventory'] as String,
  asset: json['asset'] as String,
  avatar: json['avatar'] as String,
  gridUser: json['gridUser'] as String,
  presence: json['presence'] as String,
  friends: json['friends'] as String,
  grid: json['grid'] as String,
);

Map<String, dynamic> _$ServiceUrlsToJson(ServiceUrls instance) =>
    <String, dynamic>{
      'userAccount': instance.userAccount,
      'inventory': instance.inventory,
      'asset': instance.asset,
      'avatar': instance.avatar,
      'gridUser': instance.gridUser,
      'presence': instance.presence,
      'friends': instance.friends,
      'grid': instance.grid,
    };

GridInfo _$GridInfoFromJson(Map<String, dynamic> json) => GridInfo(
  gridName: json['gridName'] as String,
  gridNick: json['gridNick'] as String,
  loginUri: json['loginUri'] as String,
  welcomeUri: json['welcomeUri'] as String,
  economyUri: json['economyUri'] as String,
  aboutUri: json['aboutUri'] as String,
  registerUri: json['registerUri'] as String,
  passwordUri: json['passwordUri'] as String,
  gridStatus: json['gridStatus'] as String,
);

Map<String, dynamic> _$GridInfoToJson(GridInfo instance) => <String, dynamic>{
  'gridName': instance.gridName,
  'gridNick': instance.gridNick,
  'loginUri': instance.loginUri,
  'welcomeUri': instance.welcomeUri,
  'economyUri': instance.economyUri,
  'aboutUri': instance.aboutUri,
  'registerUri': instance.registerUri,
  'passwordUri': instance.passwordUri,
  'gridStatus': instance.gridStatus,
};

ArchitectureConfig _$ArchitectureConfigFromJson(Map<String, dynamic> json) =>
    ArchitectureConfig(
      mode: $enumDecode(_$ArchitectureModeEnumMap, json['mode']),
      databaseUrl: json['databaseUrl'] as String,
      serviceUrls: ServiceUrls.fromJson(
        json['serviceUrls'] as Map<String, dynamic>,
      ),
      gridInfo: GridInfo.fromJson(json['gridInfo'] as Map<String, dynamic>),
      gridServerUrl: json['gridServerUrl'] as String?,
      regionName: json['regionName'] as String?,
      regionUuid: json['regionUuid'] as String?,
    );

Map<String, dynamic> _$ArchitectureConfigToJson(ArchitectureConfig instance) =>
    <String, dynamic>{
      'mode': _$ArchitectureModeEnumMap[instance.mode]!,
      'databaseUrl': instance.databaseUrl,
      'serviceUrls': instance.serviceUrls,
      'gridInfo': instance.gridInfo,
      'gridServerUrl': instance.gridServerUrl,
      'regionName': instance.regionName,
      'regionUuid': instance.regionUuid,
    };

const _$ArchitectureModeEnumMap = {
  ArchitectureMode.standalone: 'standalone',
  ArchitectureMode.gridServer: 'gridserver',
  ArchitectureMode.regionServer: 'regionserver',
};
