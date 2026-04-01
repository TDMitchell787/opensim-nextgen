// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'configuration_builder_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SystemRequirements _$SystemRequirementsFromJson(Map<String, dynamic> json) =>
    SystemRequirements(
      minMemoryMB: (json['minMemoryMB'] as num).toInt(),
      recommendedMemoryMB: (json['recommendedMemoryMB'] as num).toInt(),
      minCpuCores: (json['minCpuCores'] as num).toInt(),
      recommendedCpuCores: (json['recommendedCpuCores'] as num).toInt(),
      networkBandwidthMbps: (json['networkBandwidthMbps'] as num).toInt(),
      diskSpaceGB: (json['diskSpaceGB'] as num).toInt(),
      notes: json['notes'] as String?,
    );

Map<String, dynamic> _$SystemRequirementsToJson(SystemRequirements instance) =>
    <String, dynamic>{
      'minMemoryMB': instance.minMemoryMB,
      'recommendedMemoryMB': instance.recommendedMemoryMB,
      'minCpuCores': instance.minCpuCores,
      'recommendedCpuCores': instance.recommendedCpuCores,
      'networkBandwidthMbps': instance.networkBandwidthMbps,
      'diskSpaceGB': instance.diskSpaceGB,
      'notes': instance.notes,
    };

PortMapping _$PortMappingFromJson(Map<String, dynamic> json) => PortMapping(
  hostPort: (json['hostPort'] as num).toInt(),
  containerPort: (json['containerPort'] as num).toInt(),
  protocol: json['protocol'] as String? ?? 'tcp',
);

Map<String, dynamic> _$PortMappingToJson(PortMapping instance) =>
    <String, dynamic>{
      'hostPort': instance.hostPort,
      'containerPort': instance.containerPort,
      'protocol': instance.protocol,
    };

ContainerConfig _$ContainerConfigFromJson(Map<String, dynamic> json) =>
    ContainerConfig(
      type:
          $enumDecodeNullable(_$DeploymentTypeEnumMap, json['type']) ??
          DeploymentType.native,
      dockerImage: json['dockerImage'] as String?,
      memoryLimitMB: (json['memoryLimitMB'] as num?)?.toInt() ?? 2048,
      cpuLimit: (json['cpuLimit'] as num?)?.toDouble() ?? 2.0,
      ports:
          (json['ports'] as List<dynamic>?)
              ?.map((e) => PortMapping.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      envVars:
          (json['envVars'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as String),
          ) ??
          const {},
      volumes:
          (json['volumes'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
      restartPolicy: json['restartPolicy'] as String? ?? 'unless-stopped',
      networkName: json['networkName'] as String?,
      namespace: json['namespace'] as String?,
      replicas: (json['replicas'] as num?)?.toInt() ?? 1,
      enableHPA: json['enableHPA'] as bool? ?? false,
      minReplicas: (json['minReplicas'] as num?)?.toInt() ?? 1,
      maxReplicas: (json['maxReplicas'] as num?)?.toInt() ?? 3,
      ingressHost: json['ingressHost'] as String?,
      enableTLS: json['enableTLS'] as bool? ?? false,
    );

Map<String, dynamic> _$ContainerConfigToJson(ContainerConfig instance) =>
    <String, dynamic>{
      'type': _$DeploymentTypeEnumMap[instance.type]!,
      'dockerImage': instance.dockerImage,
      'memoryLimitMB': instance.memoryLimitMB,
      'cpuLimit': instance.cpuLimit,
      'ports': instance.ports,
      'envVars': instance.envVars,
      'volumes': instance.volumes,
      'restartPolicy': instance.restartPolicy,
      'networkName': instance.networkName,
      'namespace': instance.namespace,
      'replicas': instance.replicas,
      'enableHPA': instance.enableHPA,
      'minReplicas': instance.minReplicas,
      'maxReplicas': instance.maxReplicas,
      'ingressHost': instance.ingressHost,
      'enableTLS': instance.enableTLS,
    };

const _$DeploymentTypeEnumMap = {
  DeploymentType.native: 'native',
  DeploymentType.docker: 'docker',
  DeploymentType.kubernetes: 'kubernetes',
};

StartupSection _$StartupSectionFromJson(Map<String, dynamic> json) =>
    StartupSection(
      gridName: json['gridName'] as String? ?? 'OpenSim Next',
      welcomeMessage:
          json['welcomeMessage'] as String? ?? 'Welcome to OpenSim Next!',
      physicsEngine:
          $enumDecodeNullable(_$PhysicsEngineEnumMap, json['physicsEngine']) ??
          PhysicsEngine.ubODE,
      permissionsModule:
          json['permissionsModule'] as String? ?? 'DefaultPermissionsModule',
      saveOARs: json['saveOARs'] as bool? ?? true,
      allowScriptCrossing: json['allowScriptCrossing'] as bool? ?? true,
    );

Map<String, dynamic> _$StartupSectionToJson(StartupSection instance) =>
    <String, dynamic>{
      'gridName': instance.gridName,
      'welcomeMessage': instance.welcomeMessage,
      'physicsEngine': _$PhysicsEngineEnumMap[instance.physicsEngine]!,
      'permissionsModule': instance.permissionsModule,
      'saveOARs': instance.saveOARs,
      'allowScriptCrossing': instance.allowScriptCrossing,
    };

const _$PhysicsEngineEnumMap = {
  PhysicsEngine.ubODE: 'ubODE',
  PhysicsEngine.bulletSim: 'BulletSim',
  PhysicsEngine.openDynamicsEngine: 'OpenDynamicsEngine',
  PhysicsEngine.basicPhysics: 'basicphysics',
};

NetworkSection _$NetworkSectionFromJson(Map<String, dynamic> json) =>
    NetworkSection(
      httpPort: (json['httpPort'] as num?)?.toInt() ?? 9000,
      externalHostName: json['externalHostName'] as String? ?? 'SYSTEMIP',
      httpSSL: json['httpSSL'] as bool? ?? false,
      sslCertPath: json['sslCertPath'] as String?,
      sslKeyPath: json['sslKeyPath'] as String?,
      httpListenerPort: (json['httpListenerPort'] as num?)?.toInt() ?? 9000,
      allowRemoteAdmin: json['allowRemoteAdmin'] as bool? ?? true,
    );

Map<String, dynamic> _$NetworkSectionToJson(NetworkSection instance) =>
    <String, dynamic>{
      'httpPort': instance.httpPort,
      'externalHostName': instance.externalHostName,
      'httpSSL': instance.httpSSL,
      'sslCertPath': instance.sslCertPath,
      'sslKeyPath': instance.sslKeyPath,
      'httpListenerPort': instance.httpListenerPort,
      'allowRemoteAdmin': instance.allowRemoteAdmin,
    };

DatabaseSection _$DatabaseSectionFromJson(Map<String, dynamic> json) =>
    DatabaseSection(
      provider:
          $enumDecodeNullable(_$DatabaseProviderEnumMap, json['provider']) ??
          DatabaseProvider.sqlite,
      connectionString:
          json['connectionString'] as String? ??
          'URI=file:opensim.db,version=3',
      poolSize: (json['poolSize'] as num?)?.toInt() ?? 10,
      connectionTimeout: (json['connectionTimeout'] as num?)?.toInt() ?? 30,
    );

Map<String, dynamic> _$DatabaseSectionToJson(DatabaseSection instance) =>
    <String, dynamic>{
      'provider': _$DatabaseProviderEnumMap[instance.provider]!,
      'connectionString': instance.connectionString,
      'poolSize': instance.poolSize,
      'connectionTimeout': instance.connectionTimeout,
    };

const _$DatabaseProviderEnumMap = {
  DatabaseProvider.sqlite: 'SQLite',
  DatabaseProvider.postgresql: 'PostgreSQL',
  DatabaseProvider.mysql: 'MySQL',
  DatabaseProvider.mariadb: 'MariaDB',
};

OpenSimIniConfig _$OpenSimIniConfigFromJson(
  Map<String, dynamic> json,
) => OpenSimIniConfig(
  startup:
      json['startup'] == null
          ? null
          : StartupSection.fromJson(json['startup'] as Map<String, dynamic>),
  network:
      json['network'] == null
          ? null
          : NetworkSection.fromJson(json['network'] as Map<String, dynamic>),
  database:
      json['database'] == null
          ? null
          : DatabaseSection.fromJson(json['database'] as Map<String, dynamic>),
  additionalSections:
      json['additionalSections'] as Map<String, dynamic>? ?? const {},
);

Map<String, dynamic> _$OpenSimIniConfigToJson(OpenSimIniConfig instance) =>
    <String, dynamic>{
      'startup': instance.startup,
      'network': instance.network,
      'database': instance.database,
      'additionalSections': instance.additionalSections,
    };

EstateConfig _$EstateConfigFromJson(Map<String, dynamic> json) => EstateConfig(
  estateName: json['estateName'] as String? ?? 'My Estate',
  estateOwner: json['estateOwner'] as String? ?? 'Test User',
  estateOwnerUuid:
      json['estateOwnerUuid'] as String? ??
      '00000000-0000-0000-0000-000000000000',
  allowVoice: json['allowVoice'] as bool? ?? true,
  allowFly: json['allowFly'] as bool? ?? true,
  taxFree: json['taxFree'] as bool? ?? false,
  allowDirectTeleport: json['allowDirectTeleport'] as bool? ?? true,
);

Map<String, dynamic> _$EstateConfigToJson(EstateConfig instance) =>
    <String, dynamic>{
      'estateName': instance.estateName,
      'estateOwner': instance.estateOwner,
      'estateOwnerUuid': instance.estateOwnerUuid,
      'allowVoice': instance.allowVoice,
      'allowFly': instance.allowFly,
      'taxFree': instance.taxFree,
      'allowDirectTeleport': instance.allowDirectTeleport,
    };

RegionIniConfig _$RegionIniConfigFromJson(Map<String, dynamic> json) =>
    RegionIniConfig(
      regionName: json['regionName'] as String? ?? 'New Region',
      regionUuid: json['regionUuid'] as String?,
      locationX: (json['locationX'] as num?)?.toInt() ?? 1000,
      locationY: (json['locationY'] as num?)?.toInt() ?? 1000,
      sizeX: (json['sizeX'] as num?)?.toInt() ?? 256,
      sizeY: (json['sizeY'] as num?)?.toInt() ?? 256,
      sizeZ: (json['sizeZ'] as num?)?.toInt() ?? 4096,
      internalPort: (json['internalPort'] as num?)?.toInt() ?? 9000,
      maxAgents: (json['maxAgents'] as num?)?.toInt() ?? 100,
      maxPrims: (json['maxPrims'] as num?)?.toInt() ?? 45000,
      physicsEngine:
          $enumDecodeNullable(_$PhysicsEngineEnumMap, json['physicsEngine']) ??
          PhysicsEngine.ubODE,
      waterHeight: (json['waterHeight'] as num?)?.toDouble() ?? 20.0,
      estate:
          json['estate'] == null
              ? null
              : EstateConfig.fromJson(json['estate'] as Map<String, dynamic>),
      nonPhysicalPrimMax: json['nonPhysicalPrimMax'] as bool? ?? false,
      physicalPrimMax: json['physicalPrimMax'] as bool? ?? false,
      clampPrimSize: json['clampPrimSize'] as bool? ?? false,
    );

Map<String, dynamic> _$RegionIniConfigToJson(RegionIniConfig instance) =>
    <String, dynamic>{
      'regionName': instance.regionName,
      'regionUuid': instance.regionUuid,
      'locationX': instance.locationX,
      'locationY': instance.locationY,
      'sizeX': instance.sizeX,
      'sizeY': instance.sizeY,
      'sizeZ': instance.sizeZ,
      'internalPort': instance.internalPort,
      'maxAgents': instance.maxAgents,
      'maxPrims': instance.maxPrims,
      'physicsEngine': _$PhysicsEngineEnumMap[instance.physicsEngine]!,
      'waterHeight': instance.waterHeight,
      'estate': instance.estate,
      'nonPhysicalPrimMax': instance.nonPhysicalPrimMax,
      'physicalPrimMax': instance.physicalPrimMax,
      'clampPrimSize': instance.clampPrimSize,
    };

OsslConfig _$OsslConfigFromJson(Map<String, dynamic> json) => OsslConfig(
  allowOSFunctions: json['allowOSFunctions'] as bool? ?? true,
  osslThreatLevel:
      $enumDecodeNullable(_$OsslThreatLevelEnumMap, json['osslThreatLevel']) ??
      OsslThreatLevel.veryLow,
  allowMODFunctions: json['allowMODFunctions'] as bool? ?? false,
  allowLightShare: json['allowLightShare'] as bool? ?? true,
  allowWindLight: json['allowWindLight'] as bool? ?? true,
  functionOverrides:
      (json['functionOverrides'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, $enumDecode(_$OsslThreatLevelEnumMap, e)),
      ) ??
      const {},
);

Map<String, dynamic> _$OsslConfigToJson(OsslConfig instance) =>
    <String, dynamic>{
      'allowOSFunctions': instance.allowOSFunctions,
      'osslThreatLevel': _$OsslThreatLevelEnumMap[instance.osslThreatLevel]!,
      'allowMODFunctions': instance.allowMODFunctions,
      'allowLightShare': instance.allowLightShare,
      'allowWindLight': instance.allowWindLight,
      'functionOverrides': instance.functionOverrides.map(
        (k, e) => MapEntry(k, _$OsslThreatLevelEnumMap[e]!),
      ),
    };

const _$OsslThreatLevelEnumMap = {
  OsslThreatLevel.none: 'None',
  OsslThreatLevel.nuisance: 'Nuisance',
  OsslThreatLevel.veryLow: 'VeryLow',
  OsslThreatLevel.low: 'Low',
  OsslThreatLevel.moderate: 'Moderate',
  OsslThreatLevel.high: 'High',
  OsslThreatLevel.veryHigh: 'VeryHigh',
  OsslThreatLevel.severe: 'Severe',
};

SimulatorTemplate _$SimulatorTemplateFromJson(Map<String, dynamic> json) =>
    SimulatorTemplate(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String,
      category: $enumDecode(_$TemplateCategoryEnumMap, json['category']),
      templateType: $enumDecode(_$SimulatorTypeEnumMap, json['templateType']),
      opensimIni: OpenSimIniConfig.fromJson(
        json['opensimIni'] as Map<String, dynamic>,
      ),
      regionIni: RegionIniConfig.fromJson(
        json['regionIni'] as Map<String, dynamic>,
      ),
      osslConfig: OsslConfig.fromJson(
        json['osslConfig'] as Map<String, dynamic>,
      ),
      configIncludes:
          (json['configIncludes'] as Map<String, dynamic>?)?.map(
            (k, e) => MapEntry(k, e as String),
          ) ??
          const {},
      systemRequirements: SystemRequirements.fromJson(
        json['systemRequirements'] as Map<String, dynamic>,
      ),
      containerConfig:
          json['containerConfig'] == null
              ? null
              : ContainerConfig.fromJson(
                json['containerConfig'] as Map<String, dynamic>,
              ),
      thumbnailData: json['thumbnailData'] as String?,
      createdAt:
          json['createdAt'] == null
              ? null
              : DateTime.parse(json['createdAt'] as String),
      updatedAt:
          json['updatedAt'] == null
              ? null
              : DateTime.parse(json['updatedAt'] as String),
    );

Map<String, dynamic> _$SimulatorTemplateToJson(SimulatorTemplate instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'description': instance.description,
      'category': _$TemplateCategoryEnumMap[instance.category]!,
      'templateType': _$SimulatorTypeEnumMap[instance.templateType]!,
      'opensimIni': instance.opensimIni,
      'regionIni': instance.regionIni,
      'osslConfig': instance.osslConfig,
      'configIncludes': instance.configIncludes,
      'systemRequirements': instance.systemRequirements,
      'containerConfig': instance.containerConfig,
      'thumbnailData': instance.thumbnailData,
      'createdAt': instance.createdAt.toIso8601String(),
      'updatedAt': instance.updatedAt.toIso8601String(),
    };

const _$TemplateCategoryEnumMap = {
  TemplateCategory.builtin: 'builtin',
  TemplateCategory.custom: 'custom',
};

const _$SimulatorTypeEnumMap = {
  SimulatorType.mainland: 'mainland',
  SimulatorType.island: 'island',
  SimulatorType.marina: 'marina',
  SimulatorType.sandbox: 'sandbox',
  SimulatorType.welcome: 'welcome',
  SimulatorType.event: 'event',
  SimulatorType.shopping: 'shopping',
  SimulatorType.roleplay: 'roleplay',
  SimulatorType.residential: 'residential',
  SimulatorType.voidRegion: 'void',
  SimulatorType.customTerrain: 'custom_terrain',
};

SavedConfiguration _$SavedConfigurationFromJson(
  Map<String, dynamic> json,
) => SavedConfiguration(
  id: json['id'] as String,
  name: json['name'] as String,
  description: json['description'] as String? ?? '',
  basedOnTemplate: json['basedOnTemplate'] as String?,
  opensimIni: OpenSimIniConfig.fromJson(
    json['opensimIni'] as Map<String, dynamic>,
  ),
  regionIni: RegionIniConfig.fromJson(
    json['regionIni'] as Map<String, dynamic>,
  ),
  osslConfig: OsslConfig.fromJson(json['osslConfig'] as Map<String, dynamic>),
  configIncludes:
      (json['configIncludes'] as Map<String, dynamic>?)?.map(
        (k, e) => MapEntry(k, e as String),
      ) ??
      const {},
  systemRequirements:
      json['systemRequirements'] == null
          ? null
          : SystemRequirements.fromJson(
            json['systemRequirements'] as Map<String, dynamic>,
          ),
  deploymentType:
      $enumDecodeNullable(_$DeploymentTypeEnumMap, json['deploymentType']) ??
      DeploymentType.native,
  containerConfig:
      json['containerConfig'] == null
          ? null
          : ContainerConfig.fromJson(
            json['containerConfig'] as Map<String, dynamic>,
          ),
  deploymentStatus:
      $enumDecodeNullable(
        _$DeploymentStatusEnumMap,
        json['deploymentStatus'],
      ) ??
      DeploymentStatus.draft,
  deployedInstanceId: json['deployedInstanceId'] as String?,
  deployedPath: json['deployedPath'] as String?,
  tags:
      (json['tags'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const [],
  createdAt:
      json['createdAt'] == null
          ? null
          : DateTime.parse(json['createdAt'] as String),
  updatedAt:
      json['updatedAt'] == null
          ? null
          : DateTime.parse(json['updatedAt'] as String),
  lastDeployedAt:
      json['lastDeployedAt'] == null
          ? null
          : DateTime.parse(json['lastDeployedAt'] as String),
);

Map<String, dynamic> _$SavedConfigurationToJson(SavedConfiguration instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'description': instance.description,
      'basedOnTemplate': instance.basedOnTemplate,
      'opensimIni': instance.opensimIni,
      'regionIni': instance.regionIni,
      'osslConfig': instance.osslConfig,
      'configIncludes': instance.configIncludes,
      'systemRequirements': instance.systemRequirements,
      'deploymentType': _$DeploymentTypeEnumMap[instance.deploymentType]!,
      'containerConfig': instance.containerConfig,
      'deploymentStatus': _$DeploymentStatusEnumMap[instance.deploymentStatus]!,
      'deployedInstanceId': instance.deployedInstanceId,
      'deployedPath': instance.deployedPath,
      'tags': instance.tags,
      'createdAt': instance.createdAt.toIso8601String(),
      'updatedAt': instance.updatedAt.toIso8601String(),
      'lastDeployedAt': instance.lastDeployedAt?.toIso8601String(),
    };

const _$DeploymentStatusEnumMap = {
  DeploymentStatus.draft: 'draft',
  DeploymentStatus.ready: 'ready',
  DeploymentStatus.deployed: 'deployed',
  DeploymentStatus.failed: 'failed',
};

DeploymentProgress _$DeploymentProgressFromJson(Map<String, dynamic> json) =>
    DeploymentProgress(
      configId: json['configId'] as String,
      step: json['step'] as String,
      progress: (json['progress'] as num).toDouble(),
      message: json['message'] as String,
      isError: json['isError'] as bool? ?? false,
    );

Map<String, dynamic> _$DeploymentProgressToJson(DeploymentProgress instance) =>
    <String, dynamic>{
      'configId': instance.configId,
      'step': instance.step,
      'progress': instance.progress,
      'message': instance.message,
      'isError': instance.isError,
    };

DeploymentResult _$DeploymentResultFromJson(Map<String, dynamic> json) =>
    DeploymentResult(
      configId: json['configId'] as String,
      instanceId: json['instanceId'] as String,
      deploymentType: $enumDecode(
        _$DeploymentTypeEnumMap,
        json['deploymentType'],
      ),
      success: json['success'] as bool,
      message: json['message'] as String,
      deployedPath: json['deployedPath'] as String?,
      timestamp:
          json['timestamp'] == null
              ? null
              : DateTime.parse(json['timestamp'] as String),
    );

Map<String, dynamic> _$DeploymentResultToJson(DeploymentResult instance) =>
    <String, dynamic>{
      'configId': instance.configId,
      'instanceId': instance.instanceId,
      'deploymentType': _$DeploymentTypeEnumMap[instance.deploymentType]!,
      'success': instance.success,
      'message': instance.message,
      'deployedPath': instance.deployedPath,
      'timestamp': instance.timestamp.toIso8601String(),
    };
