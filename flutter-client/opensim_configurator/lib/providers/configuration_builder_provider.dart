import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../models/configuration_builder_models.dart';

class ConfigurationBuilderProvider extends ChangeNotifier {
  final List<SimulatorTemplate> _builtInTemplates = [];
  final List<SavedConfiguration> _savedConfigurations = [];

  SimulatorTemplate? _selectedTemplate;
  SavedConfiguration? _currentConfiguration;
  String? _selectedConfigurationId;

  bool _isLoading = false;
  String? _errorMessage;
  WebSocketChannel? _wsChannel;
  StreamSubscription? _wsSubscription;

  final StreamController<DeploymentProgress> _deploymentProgressController =
      StreamController<DeploymentProgress>.broadcast();

  List<SimulatorTemplate> get builtInTemplates => List.unmodifiable(_builtInTemplates);
  List<SavedConfiguration> get savedConfigurations => List.unmodifiable(_savedConfigurations);
  SimulatorTemplate? get selectedTemplate => _selectedTemplate;
  SavedConfiguration? get currentConfiguration => _currentConfiguration;
  String? get selectedConfigurationId => _selectedConfigurationId;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  Stream<DeploymentProgress> get deploymentProgress => _deploymentProgressController.stream;

  List<SimulatorTemplate> get allTemplates => [
    ..._builtInTemplates,
    ..._savedConfigurations
        .where((c) => c.deploymentStatus == DeploymentStatus.ready)
        .map((c) => _configToTemplate(c)),
  ];

  ConfigurationBuilderProvider() {
    _initializeBuiltInTemplates();
  }

  void _initializeBuiltInTemplates() {
    _builtInTemplates.addAll([
      _createMainlandTemplate(),
      _createIslandTemplate(),
      _createMarinaTemplate(),
      _createSandboxTemplate(),
      _createWelcomeTemplate(),
      _createEventTemplate(),
      _createShoppingTemplate(),
      _createRoleplayTemplate(),
      _createResidentialTemplate(),
      _createVoidTemplate(),
      _createCustomTerrainTemplate(),
    ]);
  }

  SimulatorTemplate _createMainlandTemplate() => SimulatorTemplate(
    id: 'builtin-mainland',
    name: 'Mainland',
    description: 'Standard land region with ubODE physics, ideal for general purpose use',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.mainland,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'OpenSim Next Grid',
        welcomeMessage: 'Welcome to our mainland region!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Mainland Region',
      maxAgents: 100,
      maxPrims: 45000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.veryLow,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 1536,
      recommendedMemoryMB: 2048,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 100,
      diskSpaceGB: 10,
      notes: 'Standard configuration suitable for most use cases',
    ),
  );

  SimulatorTemplate _createIslandTemplate() => SimulatorTemplate(
    id: 'builtin-island',
    name: 'Island',
    description: 'Isolated standalone region surrounded by water border',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.island,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Island Paradise',
        welcomeMessage: 'Welcome to our island retreat!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Island Region',
      maxAgents: 50,
      maxPrims: 30000,
      sizeX: 256,
      sizeY: 256,
      waterHeight: 20.0,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.veryLow,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 1024,
      recommendedMemoryMB: 1536,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 50,
      diskSpaceGB: 8,
      notes: 'Lighter resource requirements for isolated regions',
    ),
  );

  SimulatorTemplate _createMarinaTemplate() => SimulatorTemplate(
    id: 'builtin-marina',
    name: 'Marina',
    description: 'Water-focused region with docks and enhanced boat physics',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.marina,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Marina Bay',
        welcomeMessage: 'Welcome to Marina Bay - boats welcome!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Marina Region',
      maxAgents: 75,
      maxPrims: 35000,
      sizeX: 256,
      sizeY: 256,
      waterHeight: 25.0,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.low,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 1536,
      recommendedMemoryMB: 2048,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 75,
      diskSpaceGB: 10,
      notes: 'Enhanced physics for vehicle/boat simulations',
    ),
  );

  SimulatorTemplate _createSandboxTemplate() => SimulatorTemplate(
    id: 'builtin-sandbox',
    name: 'Sandbox',
    description: 'Testing and building area with relaxed limits and debug logging',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.sandbox,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Sandbox Zone',
        welcomeMessage: 'Welcome to the sandbox - build freely!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Sandbox Region',
      maxAgents: 50,
      maxPrims: 100000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.high,
      allowMODFunctions: true,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 3072,
      recommendedMemoryMB: 4096,
      minCpuCores: 2,
      recommendedCpuCores: 4,
      networkBandwidthMbps: 100,
      diskSpaceGB: 20,
      notes: 'High prim limit requires more resources. Debug logging enabled.',
    ),
  );

  SimulatorTemplate _createWelcomeTemplate() => SimulatorTemplate(
    id: 'builtin-welcome',
    name: 'Welcome',
    description: 'New user landing zone optimized for low lag with high agent capacity',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.welcome,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Welcome Center',
        welcomeMessage: 'Welcome to our grid! Find help and tutorials here.',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Welcome Center',
      maxAgents: 100,
      maxPrims: 25000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.nuisance,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 2048,
      recommendedMemoryMB: 3072,
      minCpuCores: 2,
      recommendedCpuCores: 4,
      networkBandwidthMbps: 200,
      diskSpaceGB: 15,
      notes: 'Optimized for many simultaneous new users. Lower prim count reduces lag.',
    ),
  );

  SimulatorTemplate _createEventTemplate() => SimulatorTemplate(
    id: 'builtin-event',
    name: 'Event',
    description: 'High-capacity events with voice enabled and media streaming',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.event,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Event Center',
        welcomeMessage: 'Welcome to our event space!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Event Arena',
      maxAgents: 200,
      maxPrims: 20000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.low,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 6144,
      recommendedMemoryMB: 8192,
      minCpuCores: 4,
      recommendedCpuCores: 8,
      networkBandwidthMbps: 500,
      diskSpaceGB: 30,
      notes: 'High agent capacity requires substantial resources. Voice and streaming enabled.',
    ),
  );

  SimulatorTemplate _createShoppingTemplate() => SimulatorTemplate(
    id: 'builtin-shopping',
    name: 'Shopping',
    description: 'Commercial region with economy enabled and vendor script support',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.shopping,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Shopping District',
        welcomeMessage: 'Welcome to our shopping district!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Shopping Mall',
      maxAgents: 75,
      maxPrims: 40000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.low,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 1536,
      recommendedMemoryMB: 2048,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 100,
      diskSpaceGB: 12,
      notes: 'Economy module enabled. Vendor scripts allowed.',
    ),
  );

  SimulatorTemplate _createRoleplayTemplate() => SimulatorTemplate(
    id: 'builtin-roleplay',
    name: 'Roleplay',
    description: 'RP-focused region with combat scripts, NPC support, and custom time',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.roleplay,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Roleplay World',
        welcomeMessage: 'Enter a world of adventure!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Roleplay Zone',
      maxAgents: 50,
      maxPrims: 35000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.moderate,
      allowMODFunctions: true,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 2048,
      recommendedMemoryMB: 2560,
      minCpuCores: 2,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 75,
      diskSpaceGB: 15,
      notes: 'NPC support enabled. Combat scripts allowed. Custom sun position.',
    ),
  );

  SimulatorTemplate _createResidentialTemplate() => SimulatorTemplate(
    id: 'builtin-residential',
    name: 'Residential',
    description: 'Private living spaces with strict permissions and low agent limit',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.residential,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Residential Area',
        welcomeMessage: 'Welcome home!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Residential Zone',
      maxAgents: 20,
      maxPrims: 15000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.nuisance,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 768,
      recommendedMemoryMB: 1024,
      minCpuCores: 1,
      recommendedCpuCores: 1,
      networkBandwidthMbps: 25,
      diskSpaceGB: 5,
      notes: 'Low resource usage. Strict permission settings for privacy.',
    ),
  );

  SimulatorTemplate _createVoidTemplate() => SimulatorTemplate(
    id: 'builtin-void',
    name: 'Void',
    description: 'Empty starter region with minimal settings and flat terrain',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.voidRegion,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Void Region',
        welcomeMessage: 'Empty region - ready for your vision',
        physicsEngine: PhysicsEngine.basicPhysics,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Void',
      maxAgents: 10,
      maxPrims: 5000,
      sizeX: 256,
      sizeY: 256,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: false,
      osslThreatLevel: OsslThreatLevel.none,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 384,
      recommendedMemoryMB: 512,
      minCpuCores: 1,
      recommendedCpuCores: 1,
      networkBandwidthMbps: 10,
      diskSpaceGB: 2,
      notes: 'Minimal configuration. Basic physics only. No OSSL functions.',
    ),
  );

  SimulatorTemplate _createCustomTerrainTemplate() => SimulatorTemplate(
    id: 'builtin-custom-terrain',
    name: 'Custom Terrain',
    description: 'User-defined terrain with heightmap import and custom water level',
    category: TemplateCategory.builtin,
    templateType: SimulatorType.customTerrain,
    opensimIni: OpenSimIniConfig(
      startup: StartupSection(
        gridName: 'Custom Terrain',
        welcomeMessage: 'Welcome to our custom landscape!',
        physicsEngine: PhysicsEngine.ubODE,
      ),
      network: NetworkSection(httpPort: 9000),
      database: DatabaseSection(provider: DatabaseProvider.sqlite),
    ),
    regionIni: RegionIniConfig(
      regionName: 'Custom Terrain',
      maxAgents: 50,
      maxPrims: 30000,
      sizeX: 256,
      sizeY: 256,
      waterHeight: 20.0,
    ),
    osslConfig: OsslConfig(
      allowOSFunctions: true,
      osslThreatLevel: OsslThreatLevel.veryLow,
    ),
    systemRequirements: SystemRequirements(
      minMemoryMB: 1536,
      recommendedMemoryMB: 2048,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 50,
      diskSpaceGB: 10,
      notes: 'Supports RAW/R32 heightmap import. Water level adjustable.',
    ),
  );

  SimulatorTemplate _configToTemplate(SavedConfiguration config) => SimulatorTemplate(
    id: config.id,
    name: config.name,
    description: config.description,
    category: TemplateCategory.custom,
    templateType: SimulatorType.customTerrain,
    opensimIni: config.opensimIni,
    regionIni: config.regionIni,
    osslConfig: config.osslConfig,
    configIncludes: config.configIncludes,
    systemRequirements: config.systemRequirements ?? SystemRequirements(
      minMemoryMB: 1024,
      recommendedMemoryMB: 2048,
      minCpuCores: 1,
      recommendedCpuCores: 2,
      networkBandwidthMbps: 50,
      diskSpaceGB: 10,
    ),
    containerConfig: config.containerConfig,
  );

  void selectTemplate(SimulatorTemplate template) {
    _selectedTemplate = template;
    _currentConfiguration = SavedConfiguration.fromTemplate(
      template,
      '${template.name} Configuration',
    );
    notifyListeners();
  }

  void selectConfiguration(String? configId) {
    _selectedConfigurationId = configId;
    if (configId != null) {
      _currentConfiguration = _savedConfigurations.firstWhere(
        (c) => c.id == configId,
        orElse: () => _currentConfiguration!,
      );
    }
    notifyListeners();
  }

  void updateOpenSimIni(OpenSimIniConfig config) {
    if (_currentConfiguration != null) {
      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: config,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.draft,
        tags: _currentConfiguration!.tags,
      );
      notifyListeners();
    }
  }

  void updateRegionIni(RegionIniConfig config) {
    if (_currentConfiguration != null) {
      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: config,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.draft,
        tags: _currentConfiguration!.tags,
      );
      notifyListeners();
    }
  }

  void updateOsslConfig(OsslConfig config) {
    if (_currentConfiguration != null) {
      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: config,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.draft,
        tags: _currentConfiguration!.tags,
      );
      notifyListeners();
    }
  }

  void updateDeploymentSettings(DeploymentType type, ContainerConfig? containerConfig) {
    if (_currentConfiguration != null) {
      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: type,
        containerConfig: containerConfig,
        deploymentStatus: DeploymentStatus.draft,
        tags: _currentConfiguration!.tags,
      );
      notifyListeners();
    }
  }

  void updateConfigIncludes(Map<String, String> includes) {
    if (_currentConfiguration != null) {
      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: includes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.draft,
        tags: _currentConfiguration!.tags,
      );
      notifyListeners();
    }
  }

  Future<void> saveConfiguration({String? name, String? description}) async {
    if (_currentConfiguration == null) return;

    _setLoading(true);
    try {
      final config = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: name ?? _currentConfiguration!.name,
        description: description ?? _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.ready,
        tags: _currentConfiguration!.tags,
        updatedAt: DateTime.now(),
      );

      final existingIndex = _savedConfigurations.indexWhere((c) => c.id == config.id);
      if (existingIndex >= 0) {
        _savedConfigurations[existingIndex] = config;
      } else {
        _savedConfigurations.add(config);
      }

      _currentConfiguration = config;
      _clearError();
    } catch (e) {
      _setError('Failed to save configuration: $e');
    } finally {
      _setLoading(false);
    }
  }

  Future<void> deleteConfiguration(String configId) async {
    _setLoading(true);
    try {
      _savedConfigurations.removeWhere((c) => c.id == configId);
      if (_selectedConfigurationId == configId) {
        _selectedConfigurationId = null;
      }
      if (_currentConfiguration?.id == configId) {
        _currentConfiguration = null;
      }
      _clearError();
    } catch (e) {
      _setError('Failed to delete configuration: $e');
    } finally {
      _setLoading(false);
    }
  }

  Future<DeploymentResult> deployConfiguration({
    required String targetPath,
    bool autoStart = false,
  }) async {
    if (_currentConfiguration == null) {
      return DeploymentResult(
        configId: '',
        instanceId: '',
        deploymentType: DeploymentType.native,
        success: false,
        message: 'No configuration to deploy',
      );
    }

    _setLoading(true);
    try {
      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Validating',
        progress: 0.1,
        message: 'Validating configuration...',
      ));

      await Future.delayed(const Duration(milliseconds: 500));

      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Generating',
        progress: 0.3,
        message: 'Generating INI files...',
      ));

      await Future.delayed(const Duration(milliseconds: 500));

      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Writing',
        progress: 0.5,
        message: 'Writing files to $targetPath...',
      ));

      await Future.delayed(const Duration(milliseconds: 500));

      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Registering',
        progress: 0.7,
        message: 'Registering with Instance Manager...',
      ));

      await Future.delayed(const Duration(milliseconds: 500));

      if (autoStart) {
        _deploymentProgressController.add(DeploymentProgress(
          configId: _currentConfiguration!.id,
          step: 'Starting',
          progress: 0.9,
          message: 'Starting instance...',
        ));
        await Future.delayed(const Duration(milliseconds: 500));
      }

      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Complete',
        progress: 1.0,
        message: 'Deployment complete!',
      ));

      final instanceId = 'instance-${DateTime.now().millisecondsSinceEpoch}';

      _currentConfiguration = SavedConfiguration(
        id: _currentConfiguration!.id,
        name: _currentConfiguration!.name,
        description: _currentConfiguration!.description,
        basedOnTemplate: _currentConfiguration!.basedOnTemplate,
        opensimIni: _currentConfiguration!.opensimIni,
        regionIni: _currentConfiguration!.regionIni,
        osslConfig: _currentConfiguration!.osslConfig,
        configIncludes: _currentConfiguration!.configIncludes,
        systemRequirements: _currentConfiguration!.systemRequirements,
        deploymentType: _currentConfiguration!.deploymentType,
        containerConfig: _currentConfiguration!.containerConfig,
        deploymentStatus: DeploymentStatus.deployed,
        deployedInstanceId: instanceId,
        deployedPath: targetPath,
        tags: _currentConfiguration!.tags,
        lastDeployedAt: DateTime.now(),
      );

      _clearError();
      return DeploymentResult(
        configId: _currentConfiguration!.id,
        instanceId: instanceId,
        deploymentType: _currentConfiguration!.deploymentType,
        success: true,
        message: 'Configuration deployed successfully',
        deployedPath: targetPath,
      );
    } catch (e) {
      _deploymentProgressController.add(DeploymentProgress(
        configId: _currentConfiguration!.id,
        step: 'Error',
        progress: 0.0,
        message: 'Deployment failed: $e',
        isError: true,
      ));
      _setError('Deployment failed: $e');
      return DeploymentResult(
        configId: _currentConfiguration!.id,
        instanceId: '',
        deploymentType: _currentConfiguration!.deploymentType,
        success: false,
        message: 'Deployment failed: $e',
      );
    } finally {
      _setLoading(false);
    }
  }

  List<String> validateConfiguration() {
    final errors = <String>[];
    if (_currentConfiguration == null) {
      errors.add('No configuration loaded');
      return errors;
    }

    final config = _currentConfiguration!;

    if (config.regionIni.regionName.isEmpty) {
      errors.add('Region name is required');
    }

    if (config.opensimIni.network.httpPort < 1 ||
        config.opensimIni.network.httpPort > 65535) {
      errors.add('HTTP port must be between 1 and 65535');
    }

    if (config.regionIni.maxAgents < 1) {
      errors.add('Max agents must be at least 1');
    }

    if (config.regionIni.maxPrims < 100) {
      errors.add('Max prims must be at least 100');
    }

    if (config.deploymentType == DeploymentType.docker) {
      if (config.containerConfig == null) {
        errors.add('Docker configuration required for container deployment');
      }
    }

    if (config.deploymentType == DeploymentType.kubernetes) {
      if (config.containerConfig?.namespace == null) {
        errors.add('Kubernetes namespace is required');
      }
    }

    return errors;
  }

  void createNewConfiguration() {
    _currentConfiguration = SavedConfiguration(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      name: 'New Configuration',
      opensimIni: OpenSimIniConfig(),
      regionIni: RegionIniConfig(),
      osslConfig: OsslConfig(),
    );
    _selectedTemplate = null;
    notifyListeners();
  }

  void _setLoading(bool loading) {
    _isLoading = loading;
    notifyListeners();
  }

  void _setError(String error) {
    _errorMessage = error;
    notifyListeners();
  }

  void _clearError() {
    _errorMessage = null;
    notifyListeners();
  }

  Color getDeploymentTypeColor(DeploymentType type) {
    switch (type) {
      case DeploymentType.native:
        return const Color(0xFF10B981);
      case DeploymentType.docker:
        return const Color(0xFF3B82F6);
      case DeploymentType.kubernetes:
        return const Color(0xFF8B5CF6);
    }
  }

  IconData getDeploymentTypeIcon(DeploymentType type) {
    switch (type) {
      case DeploymentType.native:
        return Icons.computer;
      case DeploymentType.docker:
        return Icons.widgets;
      case DeploymentType.kubernetes:
        return Icons.cloud;
    }
  }

  Color getSimulatorTypeColor(SimulatorType type) {
    switch (type) {
      case SimulatorType.mainland:
        return const Color(0xFF10B981);
      case SimulatorType.island:
        return const Color(0xFF06B6D4);
      case SimulatorType.marina:
        return const Color(0xFF0EA5E9);
      case SimulatorType.sandbox:
        return const Color(0xFFF59E0B);
      case SimulatorType.welcome:
        return const Color(0xFF8B5CF6);
      case SimulatorType.event:
        return const Color(0xFFEC4899);
      case SimulatorType.shopping:
        return const Color(0xFFF97316);
      case SimulatorType.roleplay:
        return const Color(0xFFEF4444);
      case SimulatorType.residential:
        return const Color(0xFF84CC16);
      case SimulatorType.voidRegion:
        return const Color(0xFF6B7280);
      case SimulatorType.customTerrain:
        return const Color(0xFF78716C);
    }
  }

  @override
  void dispose() {
    _wsSubscription?.cancel();
    _wsChannel?.sink.close();
    _deploymentProgressController.close();
    super.dispose();
  }
}
