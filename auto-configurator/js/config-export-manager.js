// OpenSim Next Auto-Configurator - Configuration Export/Import Manager
// Comprehensive backup, sharing, and restoration system for configuration management

class ConfigExportManager {
    constructor() {
        this.fileGenerators = new Map();
        this.exportFormats = ['ini', 'xml', 'json', 'yaml'];
        this.templateCache = new Map();
        this.backupFormats = ['json', 'yaml', 'ini', 'xml'];
        this.compressionLevels = ['none', 'gzip', 'brotli'];
        this.encryptionMethods = ['none', 'aes256', 'chacha20'];
        this.templates = new Map();
        this.isInitialized = false;
        
        // Export/import configuration
        this.settings = {
            defaultFormat: 'json',
            includeSecrets: false,
            includeMetadata: true,
            compressOutput: true,
            encryptSensitive: true,
            validateBeforeExport: true,
            includeTimestamp: true,
            generateChecksum: true
        };
        
        // Import validation rules
        this.validationRules = {
            maxFileSize: 50 * 1024 * 1024, // 50MB
            allowedFormats: this.backupFormats,
            requireMetadata: true,
            validateSchema: true,
            sanitizeInput: true,
            checkCompatibility: true
        };
        
        this.initializeFileGenerators();
        this.initializeBackupSystem();
    }

    async initializeBackupSystem() {
        try {
            // Load built-in templates
            await this.loadBuiltInTemplates();
            
            // Setup backup interface
            this.createBackupInterface();
            
            // Setup event listeners
            this.setupBackupEventListeners();
            
            // Load user preferences
            this.loadUserPreferences();
            
            this.isInitialized = true;
            console.log('✅ Configuration backup/restore system initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize backup system:', error);
        }
    }

    initializeFileGenerators() {
        // OpenSim.ini generator
        this.fileGenerators.set('OpenSim.ini', {
            type: 'ini',
            generator: this.generateOpenSimIni.bind(this),
            description: 'Main OpenSim configuration file',
            critical: true
        });

        // Regions.ini generator
        this.fileGenerators.set('Regions.ini', {
            type: 'ini',
            generator: this.generateRegionsIni.bind(this),
            description: 'Region configuration file',
            critical: true
        });

        // GridCommon.ini generator
        this.fileGenerators.set('GridCommon.ini', {
            type: 'ini',
            generator: this.generateGridCommonIni.bind(this),
            description: 'Grid services configuration',
            critical: false
        });

        // StandaloneCommon.ini generator
        this.fileGenerators.set('StandaloneCommon.ini', {
            type: 'ini',
            generator: this.generateStandaloneCommonIni.bind(this),
            description: 'Standalone mode configuration',
            critical: false
        });

        // config-include files
        this.fileGenerators.set('config-include/FlotsamCache.ini', {
            type: 'ini',
            generator: this.generateFlotsamCacheIni.bind(this),
            description: 'Asset cache configuration',
            critical: false
        });

        this.fileGenerators.set('config-include/osslEnable.ini', {
            type: 'ini',
            generator: this.generateOsslEnableIni.bind(this),
            description: 'OSSL script functions configuration',
            critical: false
        });

        // Database configuration
        this.fileGenerators.set('config-include/DatabaseService.ini', {
            type: 'ini',
            generator: this.generateDatabaseServiceIni.bind(this),
            description: 'Database service configuration',
            critical: true
        });

        // Physics configuration
        this.fileGenerators.set('config-include/PhysicsService.ini', {
            type: 'ini',
            generator: this.generatePhysicsServiceIni.bind(this),
            description: 'Physics engine configuration',
            critical: false
        });

        // Security configuration
        this.fileGenerators.set('config-include/SecurityService.ini', {
            type: 'ini',
            generator: this.generateSecurityServiceIni.bind(this),
            description: 'Security settings configuration',
            critical: true
        });
    }

    getAffectedFiles(config, diff = null) {
        const files = [];
        
        for (const [path, generator] of this.fileGenerators.entries()) {
            const changeCount = diff ? this.countFileChanges(path, diff) : 0;
            const status = this.getFileStatus(path, config, diff);
            
            files.push({
                path,
                name: path.split('/').pop(),
                type: generator.type,
                description: generator.description,
                critical: generator.critical,
                status,
                changeCount,
                size: this.estimateFileSize(path, config)
            });
        }

        return files.sort((a, b) => {
            if (a.critical !== b.critical) return b.critical - a.critical;
            return a.changeCount - b.changeCount;
        });
    }

    generateFileContent(filePath, config) {
        const generator = this.fileGenerators.get(filePath);
        if (!generator) {
            throw new Error(`No generator found for file: ${filePath}`);
        }

        try {
            return generator.generator(config);
        } catch (error) {
            console.error(`Error generating ${filePath}:`, error);
            return `; Error generating file: ${error.message}\n; Please check your configuration and try again.`;
        }
    }

    generateAllFiles(config) {
        const files = [];
        
        for (const [path, generator] of this.fileGenerators.entries()) {
            if (this.shouldGenerateFile(path, config)) {
                const content = this.generateFileContent(path, config);
                files.push({
                    path,
                    content,
                    type: generator.type,
                    critical: generator.critical
                });
            }
        }

        return files;
    }

    shouldGenerateFile(filePath, config) {
        // Logic to determine if a file should be generated based on configuration
        if (filePath === 'GridCommon.ini') {
            return config.general.deploymentType === 'grid' || config.grid.mode === 'grid';
        }
        
        if (filePath === 'StandaloneCommon.ini') {
            return config.general.deploymentType === 'development' || config.grid.mode === 'standalone';
        }

        return true; // Generate most files by default
    }

    // File generators
    generateOpenSimIni(config) {
        const template = `; OpenSim Configuration File
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[Startup]
; Grid Name and Description
gridname = "${config.general.gridName}"
gridnick = "${config.general.gridNick}"
welcome = "${config.general.welcomeMessage}"

; Physics Engine Configuration
physics = ${config.physics.defaultEngine}
DefaultScriptEngine = "XEngine"

; Regional Settings
NonPhysicalPrimMax = ${config.performance.maxPrims}
PhysicalPrimMax = ${Math.floor(config.performance.maxPrims * 0.1)}
ClampPrimSize = true
AllowScriptCrossings = true
TrustBinaries = false

; Performance Settings
MaxPoolThreads = ${config.performance.threadPoolSize}
MaxAgentGroups = 100

; Script Engine Settings
AllowOSFunctions = true
OSFunctionThreatLevel = VeryLow
AllowedCompilers = "lsl"

[Network]
; Network Configuration
http_listener_port = ${config.network.httpPort}
${config.network.httpsEnabled ? `https_listener_port = ${config.network.httpsPort}` : '; HTTPS disabled'}
ExternalHostNameForLSL = "${config.network.externalHostname}"
shard = "OpenSim"

; Client Stack
clientstack_plugin = "OpenSim.Region.ClientStack.LindenUDP.dll"

[ClientStack.LindenUDP]
; UDP Client Stack Settings
${config.network.internalIp ? `bind_addr = "${config.network.internalIp}"` : ''}

[DatabaseService]
; Database Configuration
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[Architecture]
; Module Architecture
Include-Architecture = "config-include/Standalone.ini"
${config.grid.mode === 'grid' ? 'Include-Architecture = "config-include/GridHypergrid.ini"' : ''}

[Modules]
; Asset Cache
AssetCaching = "FlotsamAssetCache"
Include-FlotsamCache = "config-include/FlotsamCache.ini"

; Physics
Include-Physics = "config-include/PhysicsService.ini"

; Security
Include-Security = "config-include/SecurityService.ini"

; Database
Include-Database = "config-include/DatabaseService.ini"

; OSSL Functions
Include-OSSL = "config-include/osslEnable.ini"

[Security]
; Basic Security Settings
DefaultUserLevel = 0
allow_regionless = false
ThreatLevel = ${config.general.deploymentType === 'production' ? 'Low' : 'VeryLow'}

; Authentication
AuthenticationService = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
UserAccountService = "OpenSim.Services.UserAccountService.dll:UserAccountService"

[Economy]
; Economy Settings
EconomyModule = ""
SellEnabled = false
CurrencySymbol = "OS$"

[VivoxVoice]
; Voice Configuration (Vivox)
enabled = false

[FreeSwitchVoice]
; Voice Configuration (FreeSwitch)
enabled = false

[Groups]
; Groups Module
Enabled = false

[XEngine]
; Script Engine Configuration
Enabled = true
DefaultCompileLanguage = "lsl"
AllowedCompilers = "lsl"
CompileWithDebugInformation = true
MinTimerInterval = 0.5
ScriptDelayFactor = 1.0
ScriptDistanceLimitFactor = 1.0
ScriptEnginesPath = "ScriptEngines"
ScriptStoppedStrategy = abort
WriteScriptSourceToDebugFile = false
CompactMemOnLoad = false
DefaultScriptTimeout = ${config.performance.scriptTimeout}

; Performance
MaxScriptEventQueue = 300
IdleTimeout = 60
Priority = "BelowNormal"
MaxScriptQueue = 300
SensorMaxRange = 96.0
SensorMaxResults = 16

; Security
ScriptStoppedStrategy = abort
DeleteScriptsOnStartup = false
OSSLMethodPermissions = true

[OSSL]
; OSSL Function Security
Allow_osSetSpeed = true
Allow_osSetOwnerSpeed = true
Allow_osSetPrimitiveParams = true
OSFunctionThreatLevel = VeryLow
AllowOSFunctions = true

[Hypergrid]
; Hypergrid Configuration
${config.grid.hypergridEnabled ? `
HomeURI = "http://${config.network.externalHostname}:${config.network.httpPort}"
GatekeeperURI = "http://${config.network.externalHostname}:${config.network.httpPort}"
` : '; Hypergrid disabled'}

[Messaging]
; Inter-Region Communication
MessageTransferModule = "OpenSim.Region.CoreModules.dll:MessageTransferModule"
LureModule = "OpenSim.Region.CoreModules.dll:LureModule"

[EntityTransfer]
; Avatar Transfer Settings
max_distance = 65535
DisableInterRegionTeleportCancellation = False

[Estates]
; Estate Management
DefaultEstateName = "${config.general.gridName} Estate"
DefaultEstateOwnerName = "Estate Manager"

[GridInfoService]
; Grid Information
gridname = "${config.general.gridName}"
gridnick = "${config.general.gridNick}"
welcome = "${config.general.welcomeMessage}"
economy = "http://${config.network.externalHostname}:${config.network.httpPort}/"
about = "http://${config.network.externalHostname}:${config.network.httpPort}/"
register = "http://${config.network.externalHostname}:${config.network.httpPort}/"
password = "http://${config.network.externalHostname}:${config.network.httpPort}/"

[DataSnapshot]
; Data Snapshot Service
index_sims = false
data_exposure = minimum

[WebStats]
; Web Statistics
enabled = false

; End of OpenSim.ini
`;

        return this.cleanupIniContent(template);
    }

    generateRegionsIni(config) {
        if (!config.regions || config.regions.length === 0) {
            return '; No regions configured\n';
        }

        let content = `; Regions Configuration File
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

`;

        config.regions.forEach((region, index) => {
            content += `[${region.name}]
RegionUUID = ${region.uuid || this.generateUUID()}
Location = ${region.location.x},${region.location.y}
InternalAddress = ${config.network.internalIp || '0.0.0.0'}
InternalPort = ${9000 + index}
AllowAlternatePorts = false
ExternalHostName = ${config.network.externalHostname}

; Region Size (must be multiple of 256)
SizeX = ${region.size.x}
SizeY = ${region.size.y}

; Maximum Prims
MaxPrims = ${region.maxPrims}

; Physics Engine
PhysicsEngine = ${region.physicsEngine || config.physics.defaultEngine}

; Terrain
MasterAvatarFirstName = "OpenSim"
MasterAvatarLastName = "Admin"
MasterAvatarSandboxPassword = "password"

; Estate Settings
EstateOwnerFirstName = "Estate"
EstateOwnerLastName = "Manager"
EstateOwnerUUID = "00000000-0000-0000-0000-000000000000"
EstateOwnerPassword = "password"

; Region Status
Enabled = ${region.enabled ? 'true' : 'false'}

; Startup Position
DefaultLanding = <128, 128, 30>

; Loading Screen
LoadTerrain = true

`;
        });

        return content;
    }

    generateGridCommonIni(config) {
        return `; Grid Common Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[DatabaseService]
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[Hypergrid]
HomeURI = "http://${config.network.externalHostname}:${config.network.httpPort}"
GatekeeperURI = "http://${config.network.externalHostname}:${config.network.httpPort}"

[GridService]
LocalServiceModule = "OpenSim.Services.GridService.dll:GridService"
Realm = "regions"

[PresenceService]
LocalServiceModule = "OpenSim.Services.PresenceService.dll:PresenceService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[UserAccountService]
LocalServiceModule = "OpenSim.Services.UserAccountService.dll:UserAccountService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[GridUserService]
LocalServiceModule = "OpenSim.Services.UserAccountService.dll:GridUserService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[AuthenticationService]
LocalServiceModule = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[AvatarService]
LocalServiceModule = "OpenSim.Services.AvatarService.dll:AvatarService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[InventoryService]
LocalServiceModule = "OpenSim.Services.InventoryService.dll:XInventoryService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[AssetService]
LocalServiceModule = "OpenSim.Services.AssetService.dll:AssetService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"
DefaultAssetLoader = "OpenSim.Framework.AssetLoader.Filesystem.dll"
AssetLoaderArgs = "assets/AssetSets.xml"

[FriendsService]
LocalServiceModule = "OpenSim.Services.FriendsService.dll:FriendsService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[LibraryService]
LibraryName = "OpenSim Library"
DefaultLibrary = "./inventory/Libraries.xml"
`;
    }

    generateStandaloneCommonIni(config) {
        return `; Standalone Common Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[DatabaseService]
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[Modules]
AssetServices = "LocalAssetServicesConnector"
InventoryServices = "LocalInventoryServicesConnector"
NeighbourServices = "LocalNeighbourServicesConnector"
AuthenticationServices = "LocalAuthenticationServicesConnector"
AuthorizationServices = "LocalAuthorizationServicesConnector"
GridServices = "LocalGridServicesConnector"
PresenceServices = "LocalPresenceServicesConnector"
UserManagementModule = "BasicUserManagementModule"
SearchModule = "BasicSearchModule"

[AuthenticationService]
LocalServiceModule = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[UserAccountService]
LocalServiceModule = "OpenSim.Services.UserAccountService.dll:UserAccountService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"
AuthenticationService = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"

[InventoryService]
LocalServiceModule = "OpenSim.Services.InventoryService.dll:XInventoryService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[GridService]
LocalServiceModule = "OpenSim.Services.GridService.dll:GridService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"
Realm = "regions"

[PresenceService]
LocalServiceModule = "OpenSim.Services.PresenceService.dll:PresenceService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[AvatarService]
LocalServiceModule = "OpenSim.Services.AvatarService.dll:AvatarService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[FriendsService]
LocalServiceModule = "OpenSim.Services.FriendsService.dll:FriendsService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

[AssetService]
LocalServiceModule = "OpenSim.Services.AssetService.dll:AssetService"
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"
DefaultAssetLoader = "OpenSim.Framework.AssetLoader.Filesystem.dll"
AssetLoaderArgs = "assets/AssetSets.xml"

[LibraryService]
LibraryName = "OpenSim Library"
DefaultLibrary = "./inventory/Libraries.xml"

[LoginService]
LocalServiceModule = "OpenSim.Services.LLLoginService.dll:LLLoginService"
UserAccountService = "OpenSim.Services.UserAccountService.dll:UserAccountService"
GridUserService = "OpenSim.Services.UserAccountService.dll:GridUserService"
AuthenticationService = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
InventoryService = "OpenSim.Services.InventoryService.dll:XInventoryService"
AvatarService = "OpenSim.Services.AvatarService.dll:AvatarService"
PresenceService = "OpenSim.Services.PresenceService.dll:PresenceService"
GridService = "OpenSim.Services.GridService.dll:GridService"
FriendsService = "OpenSim.Services.FriendsService.dll:FriendsService"
UserProfilesService = "OpenSim.Services.UserProfilesService.dll:UserProfilesService"

WelcomeMessage = "${config.general.welcomeMessage}"
AllowRemoteSetLoginLevel = "false"
AllowLoginFallbackToAnyRegion = true

${config.grid.allowGuests ? `
GuestAccountsService = "OpenSim.Services.UserAccountService.dll:UserAccountService"
CreateDefaultAvatarEntries = true
` : ''}
`;
    }

    generateFlotsamCacheIni(config) {
        return `; Flotsam Asset Cache Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[AssetCache]
; Enable asset caching
AssetCaching = "FlotsamAssetCache"

; Cache Directory
CacheDirectory = ./cache
; Log Cache Activity
LogLevel = 0

; Cache timeout in hours
DefaultAssetExpiration = ${config.performance.cacheTimeout}
BucketCount = 32

; Memory cache settings
MemoryCacheEnabled = true
MemoryCacheTimeout = ${Math.floor(config.performance.cacheTimeout / 4)}

; File cache settings
FileCacheEnabled = true
FileCacheTimeout = ${config.performance.cacheTimeout}

; Cache size limits (in MB)
MemoryCacheSize = 128
FileCacheMaxFileCount = 100000

; Compression
${config.performance.enableGzip ? 'EnableCompression = true' : 'EnableCompression = false'}

; Maintenance
DeepScanBeforePurge = true
CacheHitRateDisplay = 100

; Asset request optimization
WaitOnInProgressTimeout = 3000
HitRateDisplay = 100

; Enable this to periodically display cache statistics
; LogLevel = 1 will display hit rates, 2 will additionally show cache activity
LogLevel = 0
`;
    }

    generateOsslEnableIni(config) {
        const threatLevel = config.general.deploymentType === 'production' ? 'Low' : 'VeryLow';
        
        return `; OSSL Functions Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[XEngine]
; Enable OSSL functions
AllowOSFunctions = true
OSFunctionThreatLevel = ${threatLevel}

; Individual function permissions
Allow_osSetSpeed = true
Allow_osSetOwnerSpeed = true
Allow_osSetPrimitiveParams = true
Allow_osGetPrimitiveParams = true
Allow_osSetPhysics = true
Allow_osGetPhysics = true
Allow_osSetParcelDetails = ${threatLevel === 'VeryLow' ? 'true' : 'false'}
Allow_osGetParcelDetails = true
Allow_osSetTerrainHeight = false
Allow_osGetTerrainHeight = true
Allow_osRegionRestart = false
Allow_osConsoleCommand = false
Allow_osKickAvatar = false
Allow_osTeleportAgent = false
Allow_osForceOtherSit = false

; Media and texture functions
Allow_osSetDynamicTextureURL = true
Allow_osSetDynamicTextureData = true
Allow_osGetMapTexture = true
Allow_osGetCurrentSunHour = true

; Script information functions
Allow_osGetScriptEngineName = true
Allow_osGetSimulatorVersion = true
Allow_osGetPhysicsEngineType = true

; Avatar functions
Allow_osGetAvatarList = true
Allow_osGetAgents = true
Allow_osGetAvatarHomeURI = true
Allow_osGetGridNick = true
Allow_osGetGridName = true
Allow_osGetGridLoginURI = true

; Estate functions
Allow_osGetEstateID = true
Allow_osGetSimulatorMemory = false
Allow_osGetSimulatorMemoryKB = false

; Notecard functions
Allow_osGetNotecard = true
Allow_osGetNotecardLine = true
Allow_osGetNumberOfNotecardLines = true

; Threat level overrides for specific functions
; Allows finer control over individual functions
OSFunctionThreatLevel = ${threatLevel}

; Force permissions - these override the general settings
; Use with caution in production environments
${config.general.deploymentType === 'development' ? `
; Development mode - more permissive settings
Allow_osConsoleCommand = ESTATE_MANAGER,ESTATE_OWNER
Allow_osKickAvatar = ESTATE_MANAGER,ESTATE_OWNER
Allow_osTeleportAgent = ESTATE_MANAGER,ESTATE_OWNER
` : `
; Production mode - restrictive settings
Allow_osConsoleCommand = false
Allow_osKickAvatar = false
Allow_osTeleportAgent = false
`}
`;
    }

    generateDatabaseServiceIni(config) {
        return `; Database Service Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[DatabaseService]
StorageProvider = "${this.getDatabaseProvider(config.database.type)}"
ConnectionString = "${this.generateConnectionString(config)}"

; Connection pooling settings
MaxPoolSize = ${config.database.poolSize}
MinPoolSize = 1
ConnectionLifetime = 600

; Transaction settings
CommandTimeout = 60
TransactionTimeout = 300

; Performance settings
${config.database.type !== 'sqlite' ? `
EnableConnectionPooling = true
ValidateConnections = true
` : `
EnableConnectionPooling = false
ValidateConnections = false
`}

; Migration settings
AutoMigrate = true
MigrationBackup = true

; Logging
LogLevel = ${config.general.deploymentType === 'development' ? 'Debug' : 'Info'}
LogQueries = ${config.general.deploymentType === 'development' ? 'true' : 'false'}

; Character encoding
CharacterSet = utf8mb4

; Regional settings for specific database providers
${config.database.type === 'mysql' ? `
[MySQLService]
DefaultEngine = InnoDB
StrictMode = true
SqlMode = "STRICT_TRANS_TABLES,NO_ZERO_DATE,NO_ZERO_IN_DATE,ERROR_FOR_DIVISION_BY_ZERO"
` : ''}

${config.database.type === 'postgresql' ? `
[PostgreSQLService]
SearchPath = "opensim"
EnableExtensions = true
` : ''}

${config.database.type === 'sqlite' ? `
[SQLiteService]
JournalMode = WAL
SynchronousMode = NORMAL
CacheSize = 10000
` : ''}
`;
    }

    generatePhysicsServiceIni(config) {
        return `; Physics Service Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[Physics]
; Default physics engine
DefaultPhysicsEngine = ${config.physics.defaultEngine}

; Physics engine specific settings
[${config.physics.defaultEngine}]
; Timestep configuration
physics_fps = ${Math.round(1 / config.physics.timestep)}
physics_timestep = ${config.physics.timestep}

; Body limits
MaxPrimSize = 64
MinPrimSize = 0.01
MaxPhysicalPrimSize = 10
MaxLinksetSize = 256

; Performance settings
MaxBodies = ${config.physics.maxBodies}
EnableCollisions = ${config.physics.enableCollisions ? 'true' : 'false'}

; Gravity settings
GravityX = ${config.physics.gravityX}
GravityY = ${config.physics.gravityY}
GravityZ = ${config.physics.gravityZ}

; Contact settings
ContactSurfaceLayer = 0.001
ContactBounce = 0.2
ContactFriction = 255

; Solver settings
SolverIterations = 10
ContactMaxCorrectingVel = 100
ContactErrorReductionParameter = 0.8

; Avatar physics
AvatarMass = 80
AvatarDensity = 3.5
AvatarCapsuleRadius = 0.37
AvatarCapsuleHeight = 1.5

; Vehicle physics
VehicleAngularMotorFriction = 0.2
VehicleLinearMotorFriction = 0.2

${config.physics.defaultEngine === 'ODE' ? `
; ODE specific settings
WorldStepMethod = WorldQuickStep
WorldContactMax = 8
WorldCFM = 0.0001
WorldERP = 0.2
` : ''}

${config.physics.defaultEngine === 'Bullet' ? `
; Bullet specific settings
ShouldMeshSculptedPrim = true
ShouldForceSimplePrimMeshing = false
ShouldUseHullsForPhysicalObjects = true
MaxSubSteps = 10
FixedTimeStep = ${config.physics.timestep}
MaxCollisionsPerFrame = 2048
MaxUpdatesPerFrame = 8192
` : ''}

${config.physics.defaultEngine === 'POS' ? `
; POS (Position-based dynamics) specific settings
ParticleCount = ${Math.min(config.physics.maxBodies, 50000)}
FluidDensity = 1000
Viscosity = 0.01
SurfaceTension = 0.0728
EnableGPUAcceleration = true
` : ''}

${config.physics.defaultEngine === 'UBODE' ? `
; UBODE specific settings
UseConvexHulls = true
MinFrameTime = 0.0089
MaxFrameTime = 0.1
AvatarsOnCollisionEvents = false
VehiclesOnCollisionEvents = true
` : ''}
`;
    }

    generateSecurityServiceIni(config) {
        return `; Security Service Configuration
; Generated by OpenSim Next Auto-Configurator
; Generated on: ${new Date().toISOString()}

[Security]
; API Security
APIKey = "${config.security.apiKey}"
RequireAPIKey = true

; Authentication settings
PasswordComplexity = ${config.security.passwordComplexity ? 'true' : 'false'}
MinPasswordLength = 8
RequireUppercase = ${config.security.passwordComplexity ? 'true' : 'false'}
RequireLowercase = ${config.security.passwordComplexity ? 'true' : 'false'}
RequireNumbers = ${config.security.passwordComplexity ? 'true' : 'false'}
RequireSpecialChars = ${config.security.passwordComplexity ? 'true' : 'false'}

; Session management
SessionTimeout = ${config.security.sessionTimeout}
MaxSessionsPerUser = 3
SessionRefreshInterval = 300

; Brute force protection
BruteForceProtection = ${config.security.bruteForceProtection ? 'true' : 'false'}
MaxLoginAttempts = 5
LoginAttemptWindow = 300
LockoutDuration = 900

; Rate limiting
RateLimitEnabled = ${config.security.rateLimitEnabled ? 'true' : 'false'}
MaxRequestsPerMinute = ${config.security.maxRequestsPerMinute}
RateLimitWindow = 60

; SSL/TLS Configuration
${config.network.httpsEnabled ? `
SSLEnabled = true
SSLCertificatePath = "${config.security.sslCertificatePath}"
SSLKeyPath = "${config.security.sslKeyPath}"
SSLProtocols = "TLSv1.2,TLSv1.3"
SSLCiphers = "ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20:!aNULL:!MD5:!DSS"
` : `
SSLEnabled = false
`}

; Content Security
AllowScriptSourceLoading = false
ValidateUploads = true
MaxUploadSize = 10485760
AllowedFileTypes = "jpg,jpeg,png,gif,tga,bmp,wav,ogg,mp3"

; Access Control
RequireInventoryPermissions = true
AllowRegionless = false
RestrictedToEstateManagers = false

; Encryption settings
EncryptionMethod = "AES256"
HashingMethod = "SHA256"
KeyDerivationRounds = 10000

; Audit and logging
LogSecurityEvents = true
LogFailedLogins = true
LogAPIAccess = ${config.general.deploymentType === 'production' ? 'true' : 'false'}
SecurityLogLevel = ${config.general.deploymentType === 'development' ? 'Debug' : 'Info'}

; CORS settings for web clients
CORSEnabled = true
CORSAllowedOrigins = "*"
CORSAllowedMethods = "GET,POST,PUT,DELETE,OPTIONS"
CORSAllowedHeaders = "Content-Type,Authorization,X-API-Key"

; CSP (Content Security Policy)
CSPEnabled = ${config.general.deploymentType === 'production' ? 'true' : 'false'}
CSPDirective = "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
`;
    }

    // Utility methods
    getDatabaseProvider(type) {
        const providers = {
            'sqlite': 'OpenSim.Data.SQLite.dll',
            'mysql': 'OpenSim.Data.MySQL.dll',
            'postgresql': 'OpenSim.Data.PGSQL.dll'
        };
        return providers[type] || providers.sqlite;
    }

    generateConnectionString(config) {
        const db = config.database;
        
        switch (db.type) {
            case 'sqlite':
                return `Data Source=opensim.db;Version=3;New=True;`;
                
            case 'mysql':
                return `Server=${db.host};Port=${db.port};Database=${db.name};User ID=${db.username};Password=${db.password};Pooling=true;Min Pool Size=0;Max Pool Size=${db.poolSize};`;
                
            case 'postgresql':
                return `Server=${db.host};Port=${db.port};Database=${db.name};User Id=${db.username};Password=${db.password};Pooling=true;MinPoolSize=0;MaxPoolSize=${db.poolSize};`;
                
            default:
                return 'Data Source=opensim.db;Version=3;New=True;';
        }
    }

    generateUUID() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c === 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }

    cleanupIniContent(content) {
        return content
            .replace(/\n\s*\n\s*\n/g, '\n\n') // Remove excessive blank lines
            .replace(/^\s+/gm, '') // Remove leading whitespace
            .trim();
    }

    countFileChanges(filePath, diff) {
        // This would analyze the diff to count changes affecting this specific file
        // For now, return a simple count based on related sections
        let changeCount = 0;
        
        if (filePath === 'OpenSim.ini') {
            changeCount += this.countSectionChanges(diff.general || {});
            changeCount += this.countSectionChanges(diff.network || {});
            changeCount += this.countSectionChanges(diff.physics || {});
        }
        
        if (filePath === 'Regions.ini') {
            changeCount += this.countSectionChanges(diff.regions || {});
        }
        
        // Add more file-specific change counting logic
        
        return changeCount;
    }

    countSectionChanges(section) {
        if (!section || typeof section !== 'object') return 0;
        
        let count = 0;
        for (const value of Object.values(section)) {
            if (value && value.__change_type && value.__change_type !== 'unchanged') {
                count++;
            } else if (typeof value === 'object') {
                count += this.countSectionChanges(value);
            }
        }
        return count;
    }

    getFileStatus(filePath, config, diff) {
        const changeCount = diff ? this.countFileChanges(filePath, diff) : 0;
        
        if (changeCount === 0) return 'unchanged';
        if (changeCount > 5) return 'major-changes';
        return 'modified';
    }

    estimateFileSize(filePath, config) {
        // Rough file size estimation in bytes
        const baseSizes = {
            'OpenSim.ini': 8000,
            'Regions.ini': 1000,
            'GridCommon.ini': 3000,
            'StandaloneCommon.ini': 4000
        };
        
        let size = baseSizes[filePath] || 2000;
        
        // Adjust based on configuration
        if (filePath === 'Regions.ini' && config.regions) {
            size += config.regions.length * 500;
        }
        
        return size;
    }

    showExportDialog(config) {
        // This would show a dialog for exporting configuration
        // For now, we'll create a simple download
        this.downloadAllConfigurations(config);
    }

    downloadAllConfigurations(config) {
        const files = this.generateAllFiles(config);
        
        if (files.length === 1) {
            this.downloadFile(files[0].path, files[0].content);
        } else {
            this.downloadAsZip(files, config.general.gridName || 'opensim-config');
        }
    }

    downloadFile(filename, content) {
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    async downloadAsZip(files, configName) {
        // This would require a ZIP library like JSZip
        // For now, download files individually
        files.forEach(file => {
            setTimeout(() => {
                this.downloadFile(file.path, file.content);
            }, files.indexOf(file) * 100);
        });
    }

    // ====== BACKUP AND RESTORE SYSTEM ======

    async loadBuiltInTemplates() {
        // Built-in configuration templates
        this.templates.set('development-basic', {
            name: 'Basic Development Setup',
            description: 'Minimal configuration for local development and testing',
            category: 'development',
            author: 'OpenSim Next Team',
            version: '1.0.0',
            tags: ['development', 'basic', 'local', 'testing'],
            config: {
                deploymentType: 'development',
                gridName: 'My Development Grid',
                gridNick: 'dev-grid',
                databaseType: 'sqlite',
                physicsEngine: 'ODE',
                httpPort: 9000,
                httpsEnabled: false,
                regionCount: 1,
                maxUsers: 10,
                welcomeMessage: 'Welcome to your development virtual world!'
            },
            features: ['Single region', 'SQLite database', 'ODE physics', 'Local only'],
            requirements: {
                minRAM: '2GB',
                minCPU: '2 cores',
                diskSpace: '5GB'
            }
        });

        this.templates.set('production-standard', {
            name: 'Standard Production Setup',
            description: 'Production-ready configuration for small to medium virtual worlds',
            category: 'production',
            author: 'OpenSim Next Team',
            version: '1.0.0',
            tags: ['production', 'standard', 'ssl', 'postgresql'],
            config: {
                deploymentType: 'production',
                gridName: 'My Virtual World',
                gridNick: 'my-world',
                databaseType: 'postgresql',
                physicsEngine: 'UBODE',
                httpPort: 9000,
                httpsPort: 9001,
                httpsEnabled: true,
                regionCount: 4,
                maxUsers: 100,
                welcomeMessage: 'Welcome to our virtual world! Explore, create, and connect.',
                security: {
                    apiKeyLength: 64,
                    sessionTimeout: 3600,
                    rateLimitEnabled: true
                }
            },
            features: ['Multiple regions', 'PostgreSQL database', 'HTTPS enabled', 'Enhanced security'],
            requirements: {
                minRAM: '8GB',
                minCPU: '4 cores',
                diskSpace: '50GB'
            }
        });

        this.templates.set('enterprise-grid', {
            name: 'Enterprise Grid Setup',
            description: 'Large-scale grid configuration for enterprise deployments',
            category: 'enterprise',
            author: 'OpenSim Next Team',
            version: '1.0.0',
            tags: ['enterprise', 'grid', 'scalable', 'high-availability'],
            config: {
                deploymentType: 'grid',
                gridName: 'Enterprise Virtual World',
                gridNick: 'enterprise-grid',
                databaseType: 'postgresql',
                physicsEngine: 'POS',
                httpPort: 9000,
                httpsPort: 9001,
                httpsEnabled: true,
                regionCount: 16,
                maxUsers: 1000,
                welcomeMessage: 'Welcome to our enterprise virtual environment.',
                security: {
                    apiKeyLength: 128,
                    sessionTimeout: 7200,
                    rateLimitEnabled: true,
                    encryptionEnabled: true
                },
                performance: {
                    loadBalancing: true,
                    caching: true,
                    monitoring: true
                }
            },
            features: ['Distributed grid', 'Load balancing', 'Advanced monitoring', 'Zero trust networking'],
            requirements: {
                minRAM: '32GB',
                minCPU: '16 cores',
                diskSpace: '500GB'
            }
        });

        this.templates.set('educational-setup', {
            name: 'Educational Institution Setup',
            description: 'Configuration optimized for educational environments',
            category: 'education',
            author: 'OpenSim Next Team',
            version: '1.0.0',
            tags: ['education', 'classroom', 'moderated', 'safe'],
            config: {
                deploymentType: 'production',
                gridName: 'Educational Virtual Campus',
                gridNick: 'edu-campus',
                databaseType: 'postgresql',
                physicsEngine: 'ODE',
                httpPort: 9000,
                httpsPort: 9001,
                httpsEnabled: true,
                regionCount: 8,
                maxUsers: 200,
                welcomeMessage: 'Welcome to our educational virtual campus. Learn, explore, and collaborate!',
                moderation: {
                    contentFiltering: true,
                    userModeration: true,
                    chatLogging: true
                },
                features: {
                    voiceEnabled: true,
                    recordingEnabled: true,
                    whiteboardSupport: true
                }
            },
            features: ['Content moderation', 'Voice support', 'Recording capabilities', 'Whiteboard tools'],
            requirements: {
                minRAM: '16GB',
                minCPU: '8 cores',
                diskSpace: '200GB'
            }
        });
    }

    createBackupInterface() {
        // Create backup/restore interface
        const container = document.getElementById('export-import-container');
        if (!container) return;

        container.innerHTML = `
            <div class="export-import-manager">
                <div class="manager-header">
                    <h3>Configuration Backup & Restore</h3>
                    <p>Save, share, and restore your configuration settings</p>
                </div>
                
                <div class="manager-tabs">
                    <button class="tab-button active" data-tab="export">Backup</button>
                    <button class="tab-button" data-tab="import">Restore</button>
                    <button class="tab-button" data-tab="templates">Templates</button>
                    <button class="tab-button" data-tab="history">History</button>
                </div>
                
                <!-- Backup Tab -->
                <div class="tab-content active" id="export-tab">
                    <div class="export-section">
                        <div class="section-header">
                            <h4>Create Configuration Backup</h4>
                            <div class="export-actions">
                                <button class="btn btn-secondary" id="preview-backup">Preview</button>
                                <button class="btn btn-primary" id="create-backup">Create Backup</button>
                            </div>
                        </div>
                        
                        <div class="export-options">
                            <div class="option-group">
                                <label>Backup Format:</label>
                                <select id="backup-format">
                                    <option value="json">JSON (Recommended)</option>
                                    <option value="yaml">YAML</option>
                                    <option value="ini">INI File</option>
                                    <option value="xml">XML</option>
                                </select>
                            </div>
                            
                            <div class="option-group">
                                <label>Include Options:</label>
                                <div class="checkbox-group">
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="include-secrets" checked>
                                        <span class="checkmark"></span>
                                        Include sensitive data (passwords, keys)
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="include-metadata" checked>
                                        <span class="checkmark"></span>
                                        Include metadata and timestamps
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="include-validation" checked>
                                        <span class="checkmark"></span>
                                        Include validation rules
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="compress-output" checked>
                                        <span class="checkmark"></span>
                                        Compress backup file
                                    </label>
                                </div>
                            </div>
                            
                            <div class="option-group">
                                <label>Security Options:</label>
                                <div class="security-options">
                                    <select id="encryption-method">
                                        <option value="none">No encryption</option>
                                        <option value="aes256">AES-256 encryption</option>
                                        <option value="chacha20">ChaCha20 encryption</option>
                                    </select>
                                    <input type="password" id="encryption-password" placeholder="Encryption password" style="display: none;">
                                </div>
                            </div>
                            
                            <div class="option-group">
                                <label>Backup Scope:</label>
                                <div class="scope-selector">
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="export-general" checked>
                                        <span class="checkmark"></span>
                                        General settings
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="export-database" checked>
                                        <span class="checkmark"></span>
                                        Database configuration
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="export-network" checked>
                                        <span class="checkmark"></span>
                                        Network settings
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="export-security" checked>
                                        <span class="checkmark"></span>
                                        Security configuration
                                    </label>
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="export-regions" checked>
                                        <span class="checkmark"></span>
                                        Region settings
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                
                <!-- Restore Tab -->
                <div class="tab-content" id="import-tab">
                    <div class="import-section">
                        <div class="section-header">
                            <h4>Restore Configuration</h4>
                            <div class="import-actions">
                                <button class="btn btn-secondary" id="validate-restore">Validate</button>
                                <button class="btn btn-primary" id="restore-config" disabled>Restore</button>
                            </div>
                        </div>
                        
                        <div class="import-zone" id="import-zone">
                            <div class="drop-zone">
                                <i class="icon-upload"></i>
                                <h5>Drop backup file here</h5>
                                <p>Or click to browse for files</p>
                                <input type="file" id="backup-file-input" accept=".json,.yaml,.yml,.ini,.xml,.gz,.br" hidden>
                                <div class="supported-formats">
                                    Supported: JSON, YAML, INI, XML (compressed or uncompressed)
                                </div>
                            </div>
                        </div>
                        
                        <div class="restore-options" id="restore-options" style="display: none;">
                            <div class="file-info" id="file-info">
                                <!-- File information will be displayed here -->
                            </div>
                            
                            <div class="restore-settings">
                                <div class="option-group">
                                    <label>Restore Strategy:</label>
                                    <select id="restore-strategy">
                                        <option value="merge">Merge with current configuration</option>
                                        <option value="replace">Replace current configuration</option>
                                        <option value="selective">Selective restore</option>
                                    </select>
                                </div>
                                
                                <div class="option-group">
                                    <label>Conflict Resolution:</label>
                                    <select id="conflict-resolution">
                                        <option value="prompt">Prompt for each conflict</option>
                                        <option value="keep-current">Keep current values</option>
                                        <option value="use-backup">Use backup values</option>
                                    </select>
                                </div>
                                
                                <div class="option-group">
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="backup-before-restore" checked>
                                        <span class="checkmark"></span>
                                        Create backup before restoring
                                    </label>
                                </div>
                                
                                <div class="option-group">
                                    <label class="checkbox-label">
                                        <input type="checkbox" id="validate-after-restore" checked>
                                        <span class="checkmark"></span>
                                        Validate configuration after restore
                                    </label>
                                </div>
                            </div>
                        </div>
                        
                        <div class="restore-preview" id="restore-preview" style="display: none;">
                            <h5>Restore Preview</h5>
                            <div class="preview-content">
                                <!-- Restore preview will be displayed here -->
                            </div>
                        </div>
                    </div>
                </div>
                
                <!-- Templates Tab -->
                <div class="tab-content" id="templates-tab">
                    <div class="templates-section">
                        <div class="section-header">
                            <h4>Configuration Templates</h4>
                            <div class="template-actions">
                                <button class="btn btn-secondary" id="create-template">Create Template</button>
                                <button class="btn btn-secondary" id="import-template">Import Template</button>
                            </div>
                        </div>
                        
                        <div class="template-filters">
                            <input type="text" id="template-search" placeholder="Search templates...">
                            <select id="template-category">
                                <option value="">All Categories</option>
                                <option value="development">Development</option>
                                <option value="production">Production</option>
                                <option value="enterprise">Enterprise</option>
                                <option value="education">Education</option>
                            </select>
                        </div>
                        
                        <div class="templates-grid" id="templates-grid">
                            <!-- Templates will be displayed here -->
                        </div>
                    </div>
                </div>
                
                <!-- History Tab -->
                <div class="tab-content" id="history-tab">
                    <div class="history-section">
                        <div class="section-header">
                            <h4>Backup/Restore History</h4>
                            <div class="history-actions">
                                <button class="btn btn-secondary" id="clear-history">Clear History</button>
                                <button class="btn btn-secondary" id="export-history">Export History</button>
                            </div>
                        </div>
                        
                        <div class="history-list" id="history-list">
                            <!-- History items will be displayed here -->
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    setupBackupEventListeners() {
        // Tab switching
        document.querySelectorAll('.tab-button').forEach(button => {
            button.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });

        // Backup functionality
        document.getElementById('preview-backup')?.addEventListener('click', () => {
            this.previewBackup();
        });

        document.getElementById('create-backup')?.addEventListener('click', () => {
            this.createConfigurationBackup();
        });

        // Restore functionality
        const fileInput = document.getElementById('backup-file-input');
        const dropZone = document.getElementById('import-zone');

        dropZone?.addEventListener('click', () => {
            fileInput?.click();
        });

        dropZone?.addEventListener('dragover', (e) => {
            e.preventDefault();
            dropZone.classList.add('dragover');
        });

        dropZone?.addEventListener('dragleave', () => {
            dropZone.classList.remove('dragover');
        });

        dropZone?.addEventListener('drop', (e) => {
            e.preventDefault();
            dropZone.classList.remove('dragover');
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleBackupFileSelect(files[0]);
            }
        });

        fileInput?.addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.handleBackupFileSelect(e.target.files[0]);
            }
        });

        document.getElementById('validate-restore')?.addEventListener('click', () => {
            this.validateRestore();
        });

        document.getElementById('restore-config')?.addEventListener('click', () => {
            this.restoreConfiguration();
        });

        // Template functionality
        document.getElementById('template-search')?.addEventListener('input', (e) => {
            this.filterTemplates(e.target.value);
        });

        document.getElementById('template-category')?.addEventListener('change', (e) => {
            this.filterTemplatesByCategory(e.target.value);
        });

        document.getElementById('create-template')?.addEventListener('click', () => {
            this.createCustomTemplate();
        });

        // Encryption password toggle
        document.getElementById('encryption-method')?.addEventListener('change', (e) => {
            const passwordField = document.getElementById('encryption-password');
            if (e.target.value !== 'none') {
                passwordField.style.display = 'block';
                passwordField.required = true;
            } else {
                passwordField.style.display = 'none';
                passwordField.required = false;
            }
        });

        // Initialize displays
        this.displayTemplates();
        this.loadBackupHistory();
    }

    // Backup and restore core functionality will be added in the next section
    // This includes methods for creating backups, validating restores, etc.

    // Utility methods for backup/restore system
    getFieldValue(fieldName) {
        const element = document.querySelector(`[name="${fieldName}"], #${fieldName}`);
        return element ? element.value : null;
    }

    setFieldValue(fieldName, value) {
        const element = document.querySelector(`[name="${fieldName}"], #${fieldName}`);
        if (element) {
            element.value = value;
            element.dispatchEvent(new Event('change', { bubbles: true }));
        }
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showError(message) {
        this.showNotification(message, 'error');
    }

    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <i class="icon-${type === 'success' ? 'check' : 'alert'}"></i>
                <span>${message}</span>
                <button class="notification-close" onclick="this.parentElement.parentElement.remove()">
                    <i class="icon-x"></i>
                </button>
            </div>
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    loadUserPreferences() {
        try {
            const prefs = JSON.parse(localStorage.getItem('opensim_backup_preferences') || '{}');
            Object.assign(this.settings, prefs);
        } catch {
            // Use defaults
        }
    }

    saveUserPreferences() {
        localStorage.setItem('opensim_backup_preferences', JSON.stringify(this.settings));
    }

    // Template and history management
    displayTemplates() {
        const grid = document.getElementById('templates-grid');
        if (!grid) return;
        
        grid.innerHTML = '';
        
        for (const [id, template] of this.templates.entries()) {
            const card = document.createElement('div');
            card.className = 'template-card';
            card.innerHTML = `
                <div class="template-header">
                    <h5>${template.name}</h5>
                    <div class="template-category">${template.category}</div>
                </div>
                <div class="template-content">
                    <p>${template.description}</p>
                    <div class="template-features">
                        ${template.features.map(feature => `<span class="feature-tag">${feature}</span>`).join('')}
                    </div>
                    <div class="template-requirements">
                        <small>Requirements: ${template.requirements.minRAM}, ${template.requirements.minCPU}</small>
                    </div>
                </div>
                <div class="template-actions">
                    <button class="btn btn-sm btn-secondary" onclick="configExportManager.previewTemplate('${id}')">Preview</button>
                    <button class="btn btn-sm btn-primary" onclick="configExportManager.applyTemplate('${id}')">Apply</button>
                </div>
            `;
            grid.appendChild(card);
        }
    }

    loadBackupHistory() {
        // Backup history management will be implemented
        console.log('Loading backup history...');
    }

    // Additional backup/restore methods will follow in next update
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ConfigExportManager;
}