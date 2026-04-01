import 'package:json_annotation/json_annotation.dart';

part 'configuration_builder_models.g.dart';

enum DeploymentType {
  @JsonValue('native')
  native,
  @JsonValue('docker')
  docker,
  @JsonValue('kubernetes')
  kubernetes,
}

enum DeploymentStatus {
  @JsonValue('draft')
  draft,
  @JsonValue('ready')
  ready,
  @JsonValue('deployed')
  deployed,
  @JsonValue('failed')
  failed,
}

enum TemplateCategory {
  @JsonValue('builtin')
  builtin,
  @JsonValue('custom')
  custom,
}

enum SimulatorType {
  @JsonValue('mainland')
  mainland,
  @JsonValue('island')
  island,
  @JsonValue('marina')
  marina,
  @JsonValue('sandbox')
  sandbox,
  @JsonValue('welcome')
  welcome,
  @JsonValue('event')
  event,
  @JsonValue('shopping')
  shopping,
  @JsonValue('roleplay')
  roleplay,
  @JsonValue('residential')
  residential,
  @JsonValue('void')
  voidRegion,
  @JsonValue('custom_terrain')
  customTerrain,
}

enum OsslThreatLevel {
  @JsonValue('None')
  none,
  @JsonValue('Nuisance')
  nuisance,
  @JsonValue('VeryLow')
  veryLow,
  @JsonValue('Low')
  low,
  @JsonValue('Moderate')
  moderate,
  @JsonValue('High')
  high,
  @JsonValue('VeryHigh')
  veryHigh,
  @JsonValue('Severe')
  severe,
}

enum PhysicsEngine {
  @JsonValue('ubODE')
  ubODE,
  @JsonValue('BulletSim')
  bulletSim,
  @JsonValue('OpenDynamicsEngine')
  openDynamicsEngine,
  @JsonValue('basicphysics')
  basicPhysics,
}

enum DatabaseProvider {
  @JsonValue('SQLite')
  sqlite,
  @JsonValue('PostgreSQL')
  postgresql,
  @JsonValue('MySQL')
  mysql,
  @JsonValue('MariaDB')
  mariadb,
}

@JsonSerializable()
class SystemRequirements {
  final int minMemoryMB;
  final int recommendedMemoryMB;
  final int minCpuCores;
  final int recommendedCpuCores;
  final int networkBandwidthMbps;
  final int diskSpaceGB;
  final String? notes;

  SystemRequirements({
    required this.minMemoryMB,
    required this.recommendedMemoryMB,
    required this.minCpuCores,
    required this.recommendedCpuCores,
    required this.networkBandwidthMbps,
    required this.diskSpaceGB,
    this.notes,
  });

  String get memoryDisplay => '${(recommendedMemoryMB / 1024).toStringAsFixed(1)}GB';
  String get cpuDisplay => '$recommendedCpuCores cores';
  String get networkDisplay => '${networkBandwidthMbps}Mbps';
  String get diskDisplay => '${diskSpaceGB}GB';

  factory SystemRequirements.fromJson(Map<String, dynamic> json) =>
      _$SystemRequirementsFromJson(json);
  Map<String, dynamic> toJson() => _$SystemRequirementsToJson(this);
}

@JsonSerializable()
class PortMapping {
  final int hostPort;
  final int containerPort;
  final String protocol;

  PortMapping({
    required this.hostPort,
    required this.containerPort,
    this.protocol = 'tcp',
  });

  factory PortMapping.fromJson(Map<String, dynamic> json) =>
      _$PortMappingFromJson(json);
  Map<String, dynamic> toJson() => _$PortMappingToJson(this);
}

@JsonSerializable()
class ContainerConfig {
  final DeploymentType type;
  final String? dockerImage;
  final int memoryLimitMB;
  final double cpuLimit;
  final List<PortMapping> ports;
  final Map<String, String> envVars;
  final List<String> volumes;
  final String restartPolicy;
  final String? networkName;
  final String? namespace;
  final int replicas;
  final bool enableHPA;
  final int minReplicas;
  final int maxReplicas;
  final String? ingressHost;
  final bool enableTLS;

  ContainerConfig({
    this.type = DeploymentType.native,
    this.dockerImage,
    this.memoryLimitMB = 2048,
    this.cpuLimit = 2.0,
    this.ports = const [],
    this.envVars = const {},
    this.volumes = const [],
    this.restartPolicy = 'unless-stopped',
    this.networkName,
    this.namespace,
    this.replicas = 1,
    this.enableHPA = false,
    this.minReplicas = 1,
    this.maxReplicas = 3,
    this.ingressHost,
    this.enableTLS = false,
  });

  factory ContainerConfig.docker({
    String? image,
    int memoryMB = 2048,
    double cpu = 2.0,
  }) =>
      ContainerConfig(
        type: DeploymentType.docker,
        dockerImage: image ?? 'opensim-next:latest',
        memoryLimitMB: memoryMB,
        cpuLimit: cpu,
      );

  factory ContainerConfig.kubernetes({
    String? namespace,
    int replicas = 1,
    bool hpa = false,
  }) =>
      ContainerConfig(
        type: DeploymentType.kubernetes,
        namespace: namespace ?? 'opensim',
        replicas: replicas,
        enableHPA: hpa,
      );

  factory ContainerConfig.fromJson(Map<String, dynamic> json) =>
      _$ContainerConfigFromJson(json);
  Map<String, dynamic> toJson() => _$ContainerConfigToJson(this);
}

@JsonSerializable()
class StartupSection {
  final String gridName;
  final String welcomeMessage;
  final PhysicsEngine physicsEngine;
  final String permissionsModule;
  final bool saveOARs;
  final bool allowScriptCrossing;

  StartupSection({
    this.gridName = 'OpenSim Next',
    this.welcomeMessage = 'Welcome to OpenSim Next!',
    this.physicsEngine = PhysicsEngine.ubODE,
    this.permissionsModule = 'DefaultPermissionsModule',
    this.saveOARs = true,
    this.allowScriptCrossing = true,
  });

  factory StartupSection.fromJson(Map<String, dynamic> json) =>
      _$StartupSectionFromJson(json);
  Map<String, dynamic> toJson() => _$StartupSectionToJson(this);
}

@JsonSerializable()
class NetworkSection {
  final int httpPort;
  final String externalHostName;
  final bool httpSSL;
  final String? sslCertPath;
  final String? sslKeyPath;
  final int httpListenerPort;
  final bool allowRemoteAdmin;

  NetworkSection({
    this.httpPort = 9000,
    this.externalHostName = 'SYSTEMIP',
    this.httpSSL = false,
    this.sslCertPath,
    this.sslKeyPath,
    this.httpListenerPort = 9000,
    this.allowRemoteAdmin = true,
  });

  factory NetworkSection.fromJson(Map<String, dynamic> json) =>
      _$NetworkSectionFromJson(json);
  Map<String, dynamic> toJson() => _$NetworkSectionToJson(this);
}

@JsonSerializable()
class DatabaseSection {
  final DatabaseProvider provider;
  final String connectionString;
  final int poolSize;
  final int connectionTimeout;

  DatabaseSection({
    this.provider = DatabaseProvider.sqlite,
    this.connectionString = 'URI=file:opensim.db,version=3',
    this.poolSize = 10,
    this.connectionTimeout = 30,
  });

  factory DatabaseSection.fromJson(Map<String, dynamic> json) =>
      _$DatabaseSectionFromJson(json);
  Map<String, dynamic> toJson() => _$DatabaseSectionToJson(this);
}

@JsonSerializable()
class OpenSimIniConfig {
  final StartupSection startup;
  final NetworkSection network;
  final DatabaseSection database;
  final Map<String, dynamic> additionalSections;

  OpenSimIniConfig({
    StartupSection? startup,
    NetworkSection? network,
    DatabaseSection? database,
    this.additionalSections = const {},
  })  : startup = startup ?? StartupSection(),
        network = network ?? NetworkSection(),
        database = database ?? DatabaseSection();

  factory OpenSimIniConfig.fromJson(Map<String, dynamic> json) =>
      _$OpenSimIniConfigFromJson(json);
  Map<String, dynamic> toJson() => _$OpenSimIniConfigToJson(this);
}

@JsonSerializable()
class EstateConfig {
  final String estateName;
  final String estateOwner;
  final String estateOwnerUuid;
  final bool allowVoice;
  final bool allowFly;
  final bool taxFree;
  final bool allowDirectTeleport;

  EstateConfig({
    this.estateName = 'My Estate',
    this.estateOwner = 'Test User',
    this.estateOwnerUuid = '00000000-0000-0000-0000-000000000000',
    this.allowVoice = true,
    this.allowFly = true,
    this.taxFree = false,
    this.allowDirectTeleport = true,
  });

  factory EstateConfig.fromJson(Map<String, dynamic> json) =>
      _$EstateConfigFromJson(json);
  Map<String, dynamic> toJson() => _$EstateConfigToJson(this);
}

@JsonSerializable()
class RegionIniConfig {
  final String regionName;
  final String regionUuid;
  final int locationX;
  final int locationY;
  final int sizeX;
  final int sizeY;
  final int sizeZ;
  final int internalPort;
  final int maxAgents;
  final int maxPrims;
  final PhysicsEngine physicsEngine;
  final double waterHeight;
  final EstateConfig estate;
  final bool nonPhysicalPrimMax;
  final bool physicalPrimMax;
  final bool clampPrimSize;

  RegionIniConfig({
    this.regionName = 'New Region',
    String? regionUuid,
    this.locationX = 1000,
    this.locationY = 1000,
    this.sizeX = 256,
    this.sizeY = 256,
    this.sizeZ = 4096,
    this.internalPort = 9000,
    this.maxAgents = 100,
    this.maxPrims = 45000,
    this.physicsEngine = PhysicsEngine.ubODE,
    this.waterHeight = 20.0,
    EstateConfig? estate,
    this.nonPhysicalPrimMax = false,
    this.physicalPrimMax = false,
    this.clampPrimSize = false,
  })  : regionUuid = regionUuid ?? _generateUuid(),
        estate = estate ?? EstateConfig();

  static String _generateUuid() {
    final now = DateTime.now().millisecondsSinceEpoch;
    return '${now.toRadixString(16).padLeft(8, '0')}-0000-0000-0000-000000000000';
  }

  factory RegionIniConfig.fromJson(Map<String, dynamic> json) =>
      _$RegionIniConfigFromJson(json);
  Map<String, dynamic> toJson() => _$RegionIniConfigToJson(this);
}

@JsonSerializable()
class OsslConfig {
  final bool allowOSFunctions;
  final OsslThreatLevel osslThreatLevel;
  final bool allowMODFunctions;
  final bool allowLightShare;
  final bool allowWindLight;
  final Map<String, OsslThreatLevel> functionOverrides;

  OsslConfig({
    this.allowOSFunctions = true,
    this.osslThreatLevel = OsslThreatLevel.veryLow,
    this.allowMODFunctions = false,
    this.allowLightShare = true,
    this.allowWindLight = true,
    this.functionOverrides = const {},
  });

  factory OsslConfig.fromJson(Map<String, dynamic> json) =>
      _$OsslConfigFromJson(json);
  Map<String, dynamic> toJson() => _$OsslConfigToJson(this);
}

@JsonSerializable()
class SimulatorTemplate {
  final String id;
  final String name;
  final String description;
  final TemplateCategory category;
  final SimulatorType templateType;
  final OpenSimIniConfig opensimIni;
  final RegionIniConfig regionIni;
  final OsslConfig osslConfig;
  final Map<String, String> configIncludes;
  final SystemRequirements systemRequirements;
  final ContainerConfig? containerConfig;
  final String? thumbnailData;
  final DateTime createdAt;
  final DateTime updatedAt;

  SimulatorTemplate({
    required this.id,
    required this.name,
    required this.description,
    required this.category,
    required this.templateType,
    required this.opensimIni,
    required this.regionIni,
    required this.osslConfig,
    this.configIncludes = const {},
    required this.systemRequirements,
    this.containerConfig,
    this.thumbnailData,
    DateTime? createdAt,
    DateTime? updatedAt,
  })  : createdAt = createdAt ?? DateTime.now(),
        updatedAt = updatedAt ?? DateTime.now();

  factory SimulatorTemplate.fromJson(Map<String, dynamic> json) =>
      _$SimulatorTemplateFromJson(json);
  Map<String, dynamic> toJson() => _$SimulatorTemplateToJson(this);
}

@JsonSerializable()
class SavedConfiguration {
  final String id;
  final String name;
  final String description;
  final String? basedOnTemplate;
  final OpenSimIniConfig opensimIni;
  final RegionIniConfig regionIni;
  final OsslConfig osslConfig;
  final Map<String, String> configIncludes;
  final SystemRequirements? systemRequirements;
  final DeploymentType deploymentType;
  final ContainerConfig? containerConfig;
  final DeploymentStatus deploymentStatus;
  final String? deployedInstanceId;
  final String? deployedPath;
  final List<String> tags;
  final DateTime createdAt;
  final DateTime updatedAt;
  final DateTime? lastDeployedAt;

  SavedConfiguration({
    required this.id,
    required this.name,
    this.description = '',
    this.basedOnTemplate,
    required this.opensimIni,
    required this.regionIni,
    required this.osslConfig,
    this.configIncludes = const {},
    this.systemRequirements,
    this.deploymentType = DeploymentType.native,
    this.containerConfig,
    this.deploymentStatus = DeploymentStatus.draft,
    this.deployedInstanceId,
    this.deployedPath,
    this.tags = const [],
    DateTime? createdAt,
    DateTime? updatedAt,
    this.lastDeployedAt,
  })  : createdAt = createdAt ?? DateTime.now(),
        updatedAt = updatedAt ?? DateTime.now();

  bool get isDraft => deploymentStatus == DeploymentStatus.draft;
  bool get isReady => deploymentStatus == DeploymentStatus.ready;
  bool get isDeployed => deploymentStatus == DeploymentStatus.deployed;

  String get deploymentTypeLabel {
    switch (deploymentType) {
      case DeploymentType.native:
        return 'Native';
      case DeploymentType.docker:
        return 'Docker';
      case DeploymentType.kubernetes:
        return 'Kubernetes';
    }
  }

  factory SavedConfiguration.fromTemplate(SimulatorTemplate template, String name) =>
      SavedConfiguration(
        id: DateTime.now().millisecondsSinceEpoch.toString(),
        name: name,
        description: 'Based on ${template.name}',
        basedOnTemplate: template.id,
        opensimIni: template.opensimIni,
        regionIni: template.regionIni,
        osslConfig: template.osslConfig,
        configIncludes: Map.from(template.configIncludes),
        systemRequirements: template.systemRequirements,
        containerConfig: template.containerConfig,
      );

  factory SavedConfiguration.fromJson(Map<String, dynamic> json) =>
      _$SavedConfigurationFromJson(json);
  Map<String, dynamic> toJson() => _$SavedConfigurationToJson(this);
}

@JsonSerializable()
class DeploymentProgress {
  final String configId;
  final String step;
  final double progress;
  final String message;
  final bool isError;

  DeploymentProgress({
    required this.configId,
    required this.step,
    required this.progress,
    required this.message,
    this.isError = false,
  });

  factory DeploymentProgress.fromJson(Map<String, dynamic> json) =>
      _$DeploymentProgressFromJson(json);
  Map<String, dynamic> toJson() => _$DeploymentProgressToJson(this);
}

@JsonSerializable()
class DeploymentResult {
  final String configId;
  final String instanceId;
  final DeploymentType deploymentType;
  final bool success;
  final String message;
  final String? deployedPath;
  final DateTime timestamp;

  DeploymentResult({
    required this.configId,
    required this.instanceId,
    required this.deploymentType,
    required this.success,
    required this.message,
    this.deployedPath,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  factory DeploymentResult.fromJson(Map<String, dynamic> json) =>
      _$DeploymentResultFromJson(json);
  Map<String, dynamic> toJson() => _$DeploymentResultToJson(this);
}

class SimulatorTypeInfo {
  final SimulatorType type;
  final String label;
  final String description;
  final String icon;
  final SystemRequirements defaultRequirements;

  const SimulatorTypeInfo({
    required this.type,
    required this.label,
    required this.description,
    required this.icon,
    required this.defaultRequirements,
  });

  static SimulatorTypeInfo getInfo(SimulatorType type) {
    switch (type) {
      case SimulatorType.mainland:
        return SimulatorTypeInfo(
          type: type,
          label: 'Mainland',
          description: 'Standard land region with ubODE physics',
          icon: 'terrain',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 1536,
            recommendedMemoryMB: 2048,
            minCpuCores: 1,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 100,
            diskSpaceGB: 10,
          ),
        );
      case SimulatorType.island:
        return SimulatorTypeInfo(
          type: type,
          label: 'Island',
          description: 'Isolated standalone region with water border',
          icon: 'beach_access',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 1024,
            recommendedMemoryMB: 1536,
            minCpuCores: 1,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 50,
            diskSpaceGB: 8,
          ),
        );
      case SimulatorType.marina:
        return SimulatorTypeInfo(
          type: type,
          label: 'Marina',
          description: 'Water-focused region with docks and boat physics',
          icon: 'directions_boat',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 1536,
            recommendedMemoryMB: 2048,
            minCpuCores: 1,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 75,
            diskSpaceGB: 10,
          ),
        );
      case SimulatorType.sandbox:
        return SimulatorTypeInfo(
          type: type,
          label: 'Sandbox',
          description: 'Testing and building area with relaxed limits',
          icon: 'construction',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 3072,
            recommendedMemoryMB: 4096,
            minCpuCores: 2,
            recommendedCpuCores: 4,
            networkBandwidthMbps: 100,
            diskSpaceGB: 20,
          ),
        );
      case SimulatorType.welcome:
        return SimulatorTypeInfo(
          type: type,
          label: 'Welcome',
          description: 'New user landing zone with low lag settings',
          icon: 'waving_hand',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 2048,
            recommendedMemoryMB: 3072,
            minCpuCores: 2,
            recommendedCpuCores: 4,
            networkBandwidthMbps: 200,
            diskSpaceGB: 15,
          ),
        );
      case SimulatorType.event:
        return SimulatorTypeInfo(
          type: type,
          label: 'Event',
          description: 'High-capacity events with voice and streaming',
          icon: 'celebration',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 6144,
            recommendedMemoryMB: 8192,
            minCpuCores: 4,
            recommendedCpuCores: 8,
            networkBandwidthMbps: 500,
            diskSpaceGB: 30,
          ),
        );
      case SimulatorType.shopping:
        return SimulatorTypeInfo(
          type: type,
          label: 'Shopping',
          description: 'Commercial region with economy enabled',
          icon: 'shopping_cart',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 1536,
            recommendedMemoryMB: 2048,
            minCpuCores: 1,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 100,
            diskSpaceGB: 12,
          ),
        );
      case SimulatorType.roleplay:
        return SimulatorTypeInfo(
          type: type,
          label: 'Roleplay',
          description: 'RP-focused region with combat scripts and NPCs',
          icon: 'theater_comedy',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 2048,
            recommendedMemoryMB: 2560,
            minCpuCores: 2,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 75,
            diskSpaceGB: 15,
          ),
        );
      case SimulatorType.residential:
        return SimulatorTypeInfo(
          type: type,
          label: 'Residential',
          description: 'Private living spaces with strict permissions',
          icon: 'home',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 768,
            recommendedMemoryMB: 1024,
            minCpuCores: 1,
            recommendedCpuCores: 1,
            networkBandwidthMbps: 25,
            diskSpaceGB: 5,
          ),
        );
      case SimulatorType.voidRegion:
        return SimulatorTypeInfo(
          type: type,
          label: 'Void',
          description: 'Empty starter region with minimal settings',
          icon: 'grid_off',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 384,
            recommendedMemoryMB: 512,
            minCpuCores: 1,
            recommendedCpuCores: 1,
            networkBandwidthMbps: 10,
            diskSpaceGB: 2,
          ),
        );
      case SimulatorType.customTerrain:
        return SimulatorTypeInfo(
          type: type,
          label: 'Custom Terrain',
          description: 'User-defined terrain with heightmap import',
          icon: 'landscape',
          defaultRequirements: SystemRequirements(
            minMemoryMB: 1536,
            recommendedMemoryMB: 2048,
            minCpuCores: 1,
            recommendedCpuCores: 2,
            networkBandwidthMbps: 50,
            diskSpaceGB: 10,
          ),
        );
    }
  }
}
