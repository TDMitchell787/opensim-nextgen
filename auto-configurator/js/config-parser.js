// OpenSim Next Auto-Configurator - Configuration Template Parser
// Intelligent parsing and generation of OpenSim configuration files

class ConfigurationParser {
    constructor() {
        this.templates = new Map();
        this.configSchemas = new Map();
        this.validationRules = new Map();
        this.defaultValues = new Map();
        
        this.initializeTemplates();
    }

    initializeTemplates() {
        // Define configuration file templates and their schemas
        this.registerTemplate('opensim', {
            filename: 'OpenSim.ini',
            sections: [
                'Startup', 'Network', 'Database', 'Physics', 'Permissions',
                'Economy', 'FreeSwitchVoice', 'Groups', 'Messaging', 'Architecture'
            ],
            required: ['Startup', 'Network', 'Database'],
            description: 'Main OpenSim configuration file'
        });

        this.registerTemplate('regions', {
            filename: 'Regions/Regions.ini', 
            sections: ['RegionInfo'],
            required: ['RegionInfo'],
            description: 'Region configuration file'
        });

        this.registerTemplate('gridcommon', {
            filename: 'config-include/GridCommon.ini',
            sections: ['DatabaseService', 'Hypergrid', 'HGAssetService'],
            required: ['DatabaseService'],
            description: 'Grid mode common configuration'
        });

        this.registerTemplate('standalone', {
            filename: 'config-include/StandaloneCommon.ini',
            sections: ['DatabaseService', 'AssetService', 'InventoryService'],
            required: ['DatabaseService'],
            description: 'Standalone mode configuration'
        });

        this.registerTemplate('flotsam', {
            filename: 'config-include/FlotsamCache.ini',
            sections: ['AssetCache'],
            required: ['AssetCache'],
            description: 'Flotsam asset cache configuration'
        });

        this.registerTemplate('ossl', {
            filename: 'config-include/osslEnable.ini',
            sections: ['AllowedClients', 'Agents'],
            required: [],
            description: 'OSSL script functions configuration'
        });

        console.log('Configuration templates initialized');
    }

    registerTemplate(name, config) {
        this.templates.set(name, config);
        this.loadTemplateSchema(name);
    }

    loadTemplateSchema(templateName) {
        // Define schemas for each configuration template
        const schemas = {
            opensim: {
                Startup: {
                    physics: {
                        type: 'string',
                        options: ['OpenDynamicsEngine', 'BulletSim', 'ubODE', 'POS', 'BasicPhysics'],
                        default: 'OpenDynamicsEngine',
                        description: 'Physics engine to use'
                    },
                    meshing: {
                        type: 'string', 
                        options: ['Meshmerizer', 'ZeroMesher'],
                        default: 'Meshmerizer',
                        description: 'Mesh generation engine'
                    },
                    storage_provider: {
                        type: 'string',
                        options: ['OpenSim.Data.SQLite.dll', 'OpenSim.Data.MySQL.dll', 'OpenSim.Data.PGSQL.dll'],
                        default: 'OpenSim.Data.SQLite.dll',
                        description: 'Database storage provider'
                    }
                },
                Network: {
                    http_listener_port: {
                        type: 'integer',
                        min: 1024,
                        max: 65535,
                        default: 9000,
                        description: 'Port for HTTP listener'
                    },
                    console_port: {
                        type: 'integer',
                        min: 1024,
                        max: 65535,
                        default: 0,
                        description: 'Console port (0 = disabled)'
                    }
                },
                Database: {
                    ConnectionString: {
                        type: 'string',
                        template: 'Data Source=file:{database};version=3;DateTimeFormat=UniversalSortableDateTime;',
                        description: 'Database connection string'
                    },
                    StorageProvider: {
                        type: 'string',
                        options: ['OpenSim.Data.SQLite.dll', 'OpenSim.Data.MySQL.dll', 'OpenSim.Data.PGSQL.dll'],
                        default: 'OpenSim.Data.SQLite.dll',
                        description: 'Database provider assembly'
                    }
                }
            },
            regions: {
                RegionInfo: {
                    RegionName: {
                        type: 'string',
                        required: true,
                        description: 'Name of the region'
                    },
                    RegionUUID: {
                        type: 'uuid',
                        required: true,
                        description: 'Unique identifier for the region'
                    },
                    Location: {
                        type: 'coordinates',
                        required: true,
                        description: 'Grid coordinates (X,Y)'
                    },
                    SizeX: {
                        type: 'integer',
                        default: 256,
                        options: [256, 512, 1024],
                        description: 'Region size in X direction'
                    },
                    SizeY: {
                        type: 'integer', 
                        default: 256,
                        options: [256, 512, 1024],
                        description: 'Region size in Y direction'
                    },
                    InternalPort: {
                        type: 'integer',
                        min: 9000,
                        max: 9999,
                        description: 'Internal port for region'
                    },
                    AllowAlternatePorts: {
                        type: 'boolean',
                        default: false,
                        description: 'Allow alternative ports if specified port is unavailable'
                    },
                    ExternalHostName: {
                        type: 'string',
                        default: 'SYSTEMIP',
                        description: 'External hostname or IP address'
                    }
                }
            }
        };

        if (schemas[templateName]) {
            this.configSchemas.set(templateName, schemas[templateName]);
        }
    }

    // Configuration Generation
    generateConfiguration(deploymentType, userConfig) {
        const configFiles = new Map();
        
        try {
            // Generate main OpenSim.ini
            configFiles.set('OpenSim.ini', this.generateOpenSimConfig(deploymentType, userConfig));
            
            // Generate region configurations
            if (userConfig.regions && userConfig.regions.length > 0) {
                userConfig.regions.forEach((region, index) => {
                    const regionFile = `Regions/${region.name || `Region${index + 1}`}.ini`;
                    configFiles.set(regionFile, this.generateRegionConfig(region));
                });
            }
            
            // Generate deployment-specific configurations
            this.generateDeploymentSpecificConfigs(deploymentType, userConfig, configFiles);
            
            // Generate advanced configurations
            this.generateAdvancedConfigs(userConfig, configFiles);
            
            console.log(`Generated ${configFiles.size} configuration files`);
            return configFiles;
            
        } catch (error) {
            console.error('Configuration generation failed:', error);
            throw new Error(`Failed to generate configuration: ${error.message}`);
        }
    }

    generateOpenSimConfig(deploymentType, userConfig) {
        const config = new ConfigurationFile('OpenSim.ini');
        
        // Add header comment
        config.addComment('OpenSim Next Configuration File');
        config.addComment(`Generated on: ${new Date().toISOString()}`);
        config.addComment(`Deployment Type: ${deploymentType}`);
        config.addComment('');
        
        // Startup Section
        config.addSection('Startup');
        
        // Physics configuration based on deployment type and user preferences
        const physicsEngine = this.getPhysicsEngine(deploymentType, userConfig);
        config.addSetting('physics', physicsEngine.name);
        
        if (physicsEngine.settings) {
            Object.entries(physicsEngine.settings).forEach(([key, value]) => {
                config.addSetting(key, value);
            });
        }
        
        // Storage provider based on database configuration
        const storageProvider = this.getStorageProvider(userConfig.database?.type || 'sqlite');
        config.addSetting('storage_provider', storageProvider);
        
        // Meshing configuration
        config.addSetting('meshing', 'Meshmerizer');
        
        // Asset service configuration
        config.addSetting('AssetCaching', 'FlotsamAssetCache');
        
        // Network Section
        config.addSection('Network');
        
        const networkConfig = userConfig.network || {};
        config.addSetting('http_listener_port', networkConfig.ports?.http || 9000);
        
        if (userConfig.security?.sslEnabled) {
            config.addSetting('https_listener', 'true');
            config.addSetting('https_port', networkConfig.ports?.https || 9001);
            config.addSetting('cert_path', userConfig.security.sslCertPath || '');
            config.addSetting('cert_pass', ''); // Password should be provided at runtime
        }
        
        // Console configuration
        if (deploymentType === 'development') {
            config.addSetting('console_port', networkConfig.consolePort || 0);
        }
        
        // Database Section
        config.addSection('Database');
        
        const connectionString = this.generateConnectionString(userConfig.database);
        config.addSetting('ConnectionString', connectionString);
        config.addSetting('StorageProvider', storageProvider);
        
        // Advanced database settings for production
        if (deploymentType === 'production' || deploymentType === 'grid') {
            config.addSetting('ConnectionPooling', 'true');
            config.addSetting('ConnectionLifetime', '600');
            config.addSetting('ConnectionPoolSize', userConfig.database?.connectionPoolSize || 20);
        }
        
        // Architecture Section
        config.addSection('Architecture');
        
        if (deploymentType === 'grid') {
            config.addSetting('Include-Architecture', 'config-include/Grid.ini');
        } else {
            config.addSetting('Include-Architecture', 'config-include/Standalone.ini');
        }
        
        // Security Section
        if (userConfig.security) {
            config.addSection('Security');
            
            if (userConfig.security.authenticationLevel) {
                config.addSetting('AuthenticationLevel', userConfig.security.authenticationLevel);
            }
            
            if (userConfig.security.encryptionRequired) {
                config.addSetting('RequireEncryption', 'true');
            }
        }
        
        // Permissions Section
        config.addSection('Permissions');
        config.addSetting('serverside_object_permissions', 'true');
        config.addSetting('allow_grid_gods', 'false');
        config.addSetting('region_owner_is_god', 'true');
        
        // Groups Section (for grid deployments)
        if (deploymentType === 'grid') {
            config.addSection('Groups');
            config.addSetting('Enabled', 'true');
            config.addSetting('Module', 'GroupsModule');
            config.addSetting('DebugEnabled', 'false');
        }
        
        return config.toString();
    }

    generateRegionConfig(regionConfig) {
        const config = new ConfigurationFile(`${regionConfig.name}.ini`);
        
        config.addComment(`Region Configuration: ${regionConfig.name}`);
        config.addComment(`Generated on: ${new Date().toISOString()}`);
        config.addComment('');
        
        config.addSection(regionConfig.name);
        
        // Required region settings
        config.addSetting('RegionUUID', regionConfig.uuid || this.generateUUID());
        config.addSetting('Location', `${regionConfig.location?.x || 1000},${regionConfig.location?.y || 1000}`);
        config.addSetting('SizeX', regionConfig.size?.x || 256);
        config.addSetting('SizeY', regionConfig.size?.y || 256);
        config.addSetting('SizeZ', regionConfig.size?.z || 4096);
        
        // Network settings
        config.addSetting('InternalAddress', regionConfig.internalAddress || '0.0.0.0');
        config.addSetting('InternalPort', regionConfig.internalPort || 9000);
        config.addSetting('AllowAlternatePorts', regionConfig.allowAlternatePorts || false);
        config.addSetting('ExternalHostName', regionConfig.externalHostName || 'SYSTEMIP');
        
        // Physics settings (if specified)
        if (regionConfig.physicsEngine) {
            config.addSetting('PhysicsEngine', regionConfig.physicsEngine);
        }
        
        // Terrain settings
        if (regionConfig.terrainFile) {
            config.addSetting('TerrainFile', regionConfig.terrainFile);
        }
        
        // Estate settings
        config.addSetting('MasterAvatarFirstName', regionConfig.masterAvatar?.firstName || 'Test');
        config.addSetting('MasterAvatarLastName', regionConfig.masterAvatar?.lastName || 'User');
        config.addSetting('MasterAvatarSandboxPassword', regionConfig.masterAvatar?.password || 'password');
        
        return config.toString();
    }

    generateDeploymentSpecificConfigs(deploymentType, userConfig, configFiles) {
        switch (deploymentType) {
            case 'development':
                this.generateDevelopmentConfigs(userConfig, configFiles);
                break;
            case 'production':
                this.generateProductionConfigs(userConfig, configFiles);
                break;
            case 'grid':
                this.generateGridConfigs(userConfig, configFiles);
                break;
        }
    }

    generateDevelopmentConfigs(userConfig, configFiles) {
        // Standalone configuration for development
        const standaloneConfig = new ConfigurationFile('config-include/StandaloneCommon.ini');
        
        standaloneConfig.addComment('Standalone Development Configuration');
        standaloneConfig.addComment('Optimized for local development and testing');
        standaloneConfig.addComment('');
        
        standaloneConfig.addSection('DatabaseService');
        standaloneConfig.addSetting('StorageProvider', 'OpenSim.Data.SQLite.dll');
        standaloneConfig.addSetting('ConnectionString', 'Data Source=file:opensim.db;version=3;DateTimeFormat=UniversalSortableDateTime;');
        
        standaloneConfig.addSection('AssetService');
        standaloneConfig.addSetting('DefaultAssetLoader', 'OpenSim.Framework.AssetLoader.Filesystem.dll');
        standaloneConfig.addSetting('AssetLoaderArgs', 'assets/AssetSets.xml');
        
        standaloneConfig.addSection('InventoryService');
        standaloneConfig.addSetting('DefaultInventoryLoader', 'OpenSim.Framework.InventoryLoader.dll');
        standaloneConfig.addSetting('InventoryLoaderArgs', 'inventory/inventory.xml');
        
        configFiles.set('config-include/StandaloneCommon.ini', standaloneConfig.toString());
    }

    generateProductionConfigs(userConfig, configFiles) {
        // Production-optimized configurations
        const gridCommonConfig = new ConfigurationFile('config-include/GridCommon.ini');
        
        gridCommonConfig.addComment('Production Grid Configuration');
        gridCommonConfig.addComment('Optimized for production deployment');
        gridCommonConfig.addComment('');
        
        gridCommonConfig.addSection('DatabaseService');
        const dbConfig = userConfig.database || {};
        
        if (dbConfig.type === 'postgresql') {
            gridCommonConfig.addSetting('StorageProvider', 'OpenSim.Data.PGSQL.dll');
            gridCommonConfig.addSetting('ConnectionString', this.generateConnectionString(dbConfig));
        } else {
            gridCommonConfig.addSetting('StorageProvider', 'OpenSim.Data.SQLite.dll');
            gridCommonConfig.addSetting('ConnectionString', this.generateConnectionString(dbConfig));
        }
        
        // Asset service configuration
        gridCommonConfig.addSection('AssetService');
        gridCommonConfig.addSetting('AssetLoaderArgs', 'assets/AssetSets.xml');
        
        // Hypergrid configuration
        if (userConfig.hypergrid?.enabled) {
            gridCommonConfig.addSection('Hypergrid');
            gridCommonConfig.addSetting('Enabled', 'true');
            gridCommonConfig.addSetting('AllowTeleportsToAnyRegion', 'true');
            gridCommonConfig.addSetting('RestrictAppearanceAbroad', 'false');
        }
        
        configFiles.set('config-include/GridCommon.ini', gridCommonConfig.toString());
    }

    generateGridConfigs(userConfig, configFiles) {
        // Grid-specific configurations
        this.generateProductionConfigs(userConfig, configFiles);
        
        // Additional grid configurations
        const gridConfig = new ConfigurationFile('config-include/Grid.ini');
        
        gridConfig.addComment('Grid Architecture Configuration');
        gridConfig.addComment('Multi-server grid deployment settings');
        gridConfig.addComment('');
        
        gridConfig.addSection('Includes');
        gridConfig.addSetting('Include-Common', 'config-include/GridCommon.ini');
        gridConfig.addSetting('Include-Storage', 'config-include/storage/GridStorage.ini');
        
        gridConfig.addSection('GridServices');
        gridConfig.addSetting('GridService', 'OpenSim.Services.GridService.dll:GridService');
        gridConfig.addSetting('UserAccountService', 'OpenSim.Services.UserAccountService.dll:UserAccountService');
        gridConfig.addSetting('AuthenticationService', 'OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService');
        
        configFiles.set('config-include/Grid.ini', gridConfig.toString());
        
        // Zero trust networking configuration if enabled
        if (userConfig.security?.zeroTrust) {
            this.generateZeroTrustConfig(userConfig, configFiles);
        }
    }

    generateAdvancedConfigs(userConfig, configFiles) {
        // Flotsam cache configuration
        this.generateFlotsamCacheConfig(userConfig, configFiles);
        
        // OSSL configuration
        this.generateOSSLConfig(userConfig, configFiles);
        
        // Custom physics configurations
        this.generatePhysicsConfigs(userConfig, configFiles);
    }

    generateFlotsamCacheConfig(userConfig, configFiles) {
        const cacheConfig = new ConfigurationFile('config-include/FlotsamCache.ini');
        
        cacheConfig.addComment('Flotsam Asset Cache Configuration');
        cacheConfig.addComment('High-performance asset caching system');
        cacheConfig.addComment('');
        
        cacheConfig.addSection('AssetCache');
        cacheConfig.addSetting('CacheDirectory', './assetcache');
        cacheConfig.addSetting('LogLevel', '0');
        cacheConfig.addSetting('HitRateDisplay', '100');
        
        // Cache sizing based on deployment type
        if (userConfig.deploymentType === 'grid') {
            cacheConfig.addSetting('MemoryCacheSize', '512MB');
            cacheConfig.addSetting('FileCacheSize', '10GB');
        } else if (userConfig.deploymentType === 'production') {
            cacheConfig.addSetting('MemoryCacheSize', '256MB');
            cacheConfig.addSetting('FileCacheSize', '5GB');
        } else {
            cacheConfig.addSetting('MemoryCacheSize', '128MB');
            cacheConfig.addSetting('FileCacheSize', '1GB');
        }
        
        cacheConfig.addSetting('FileCleanupTimer', '0.166667'); // 10 minutes
        cacheConfig.addSetting('CacheCleanupTimer', '0.166667');
        cacheConfig.addSetting('DeepScanBeforePurge', 'true');
        
        configFiles.set('config-include/FlotsamCache.ini', cacheConfig.toString());
    }

    generateOSSLConfig(userConfig, configFiles) {
        const osslConfig = new ConfigurationFile('config-include/osslEnable.ini');
        
        osslConfig.addComment('OSSL Script Functions Configuration');
        osslConfig.addComment('Controls which OSSL functions are available to scripts');
        osslConfig.addComment('');
        
        osslConfig.addSection('AllowedClients');
        osslConfig.addSetting('Allow_All', 'false');
        osslConfig.addSetting('Allow_osConsoleCommand', 'false');
        osslConfig.addSetting('Allow_osKickAvatar', 'ESTATE_OWNER,ESTATE_MANAGER');
        osslConfig.addSetting('Allow_osSetDynamicTextureData', 'true');
        osslConfig.addSetting('Allow_osSetDynamicTextureDataBlend', 'true');
        
        // Development vs production OSSL permissions
        if (userConfig.deploymentType === 'development') {
            osslConfig.addSetting('Allow_osGetRegionStats', 'true');
            osslConfig.addSetting('Allow_osGetSimulatorMemory', 'true');
        } else {
            osslConfig.addSetting('Allow_osGetRegionStats', 'ESTATE_OWNER,ESTATE_MANAGER');
            osslConfig.addSetting('Allow_osGetSimulatorMemory', 'ESTATE_OWNER');
        }
        
        configFiles.set('config-include/osslEnable.ini', osslConfig.toString());
    }

    generatePhysicsConfigs(userConfig, configFiles) {
        // Generate physics-specific configurations for each engine
        const physicsEngines = ['ODE', 'BulletSim', 'ubODE', 'POS'];
        
        physicsEngines.forEach(engine => {
            if (this.isPhysicsEngineUsed(engine, userConfig)) {
                const physicsConfig = this.generatePhysicsEngineConfig(engine, userConfig);
                configFiles.set(`config-include/physics/${engine}.ini`, physicsConfig);
            }
        });
    }

    generateZeroTrustConfig(userConfig, configFiles) {
        const zeroTrustConfig = new ConfigurationFile('config-include/ZeroTrust.ini');
        
        zeroTrustConfig.addComment('Zero Trust Networking Configuration');
        zeroTrustConfig.addComment('OpenZiti integration for secure grid communication');
        zeroTrustConfig.addComment('');
        
        zeroTrustConfig.addSection('ZeroTrustNetwork');
        zeroTrustConfig.addSetting('Enabled', 'true');
        zeroTrustConfig.addSetting('ZitiConfigFile', 'ziti/ziti-config.json');
        zeroTrustConfig.addSetting('ServiceName', userConfig.network?.serviceName || 'opensim-grid');
        
        zeroTrustConfig.addSection('EncryptedOverlay');
        zeroTrustConfig.addSetting('Enabled', 'true');
        zeroTrustConfig.addSetting('EncryptionAlgorithm', 'AES-256-GCM');
        zeroTrustConfig.addSetting('KeyRotationInterval', '86400'); // 24 hours
        
        configFiles.set('config-include/ZeroTrust.ini', zeroTrustConfig.toString());
    }

    // Utility Methods
    getPhysicsEngine(deploymentType, userConfig) {
        const engines = {
            development: {
                name: 'OpenDynamicsEngine',
                settings: {
                    physics_logging: 'false',
                    physics_logging_interval: '0',
                    physics_logging_append_existing_logfile: 'false'
                }
            },
            production: {
                name: 'BulletSim',
                settings: {
                    physics_logging: 'false',
                    BulletEngine: 'BulletUnmanaged',
                    BulletTimeStep: '0.0167'
                }
            },
            grid: {
                name: 'POS',
                settings: {
                    physics_logging: 'false',
                    EnableGPUPhysics: 'true',
                    MaxParticles: '100000'
                }
            }
        };
        
        return engines[deploymentType] || engines.development;
    }

    getStorageProvider(databaseType) {
        const providers = {
            'sqlite': 'OpenSim.Data.SQLite.dll',
            'postgresql': 'OpenSim.Data.PGSQL.dll',
            'mysql': 'OpenSim.Data.MySQL.dll'
        };
        
        return providers[databaseType] || providers.sqlite;
    }

    generateConnectionString(databaseConfig) {
        if (!databaseConfig || !databaseConfig.type) {
            return 'Data Source=file:opensim.db;version=3;DateTimeFormat=UniversalSortableDateTime;';
        }
        
        switch (databaseConfig.type) {
            case 'sqlite':
                return `Data Source=file:${databaseConfig.location || 'opensim.db'};version=3;DateTimeFormat=UniversalSortableDateTime;`;
                
            case 'postgresql':
                return `Host=${databaseConfig.host || 'localhost'};Port=${databaseConfig.port || 5432};Database=${databaseConfig.database || 'opensim'};Username=${databaseConfig.username || 'opensim'};SSL Mode=Prefer;`;
                
            case 'mysql':
                return `Data Source=${databaseConfig.host || 'localhost'};Port=${databaseConfig.port || 3306};Database=${databaseConfig.database || 'opensim'};User ID=${databaseConfig.username || 'opensim'};`;
                
            default:
                return 'Data Source=file:opensim.db;version=3;DateTimeFormat=UniversalSortableDateTime;';
        }
    }

    generateUUID() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c == 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }

    isPhysicsEngineUsed(engine, userConfig) {
        // Check if a specific physics engine is used in any region
        if (userConfig.regions) {
            return userConfig.regions.some(region => region.physicsEngine === engine);
        }
        return false;
    }

    generatePhysicsEngineConfig(engine, userConfig) {
        // Generate engine-specific configuration
        const configs = {
            'ODE': this.generateODEConfig(userConfig),
            'BulletSim': this.generateBulletConfig(userConfig),
            'ubODE': this.generateUbODEConfig(userConfig),
            'POS': this.generatePOSConfig(userConfig)
        };
        
        return configs[engine] || '';
    }

    generateODEConfig(userConfig) {
        const config = new ConfigurationFile('ODE.ini');
        
        config.addComment('ODE Physics Engine Configuration');
        config.addSection('ODEPhysicsSettings');
        config.addSetting('world_stepsize', '0.020');
        config.addSetting('world_internal_steps_without_collisions', '10');
        config.addSetting('world_hashspace_size_low', '-4');
        config.addSetting('world_hashspace_size_high', '128');
        config.addSetting('meters_in_small_space', '29.9');
        config.addSetting('small_hashspace_size_low', '-4');
        config.addSetting('small_hashspace_size_high', '66');
        
        return config.toString();
    }

    generateBulletConfig(userConfig) {
        const config = new ConfigurationFile('BulletSim.ini');
        
        config.addComment('BulletSim Physics Engine Configuration');
        config.addSection('BulletSim');
        config.addSetting('BulletEngine', 'BulletUnmanaged');
        config.addSetting('BulletTimeStep', '0.0167');
        config.addSetting('MaxSubSteps', '10');
        config.addSetting('FixedTimeStep', '0.0167');
        config.addSetting('MaxCollisionsPerFrame', '2048');
        config.addSetting('MaxUpdatesPerFrame', '8192');
        
        return config.toString();
    }

    generateUbODEConfig(userConfig) {
        const config = new ConfigurationFile('ubODE.ini');
        
        config.addComment('ubODE Physics Engine Configuration');
        config.addSection('ubODEPhysicsSettings');
        config.addSetting('world_stepsize', '0.020');
        config.addSetting('world_internal_steps_without_collisions', '10');
        config.addSetting('world_contact_surface_layer', '0.001');
        config.addSetting('world_hashspace_level_low', '-5');
        config.addSetting('world_hashspace_level_high', '12');
        
        return config.toString();
    }

    generatePOSConfig(userConfig) {
        const config = new ConfigurationFile('POS.ini');
        
        config.addComment('POS Physics Engine Configuration');
        config.addSection('POSPhysicsSettings');
        config.addSetting('timestep', '0.0167');
        config.addSetting('max_particles', '100000');
        config.addSetting('enable_gpu', 'true');
        config.addSetting('particle_radius', '0.1');
        config.addSetting('fluid_density', '1000.0');
        config.addSetting('viscosity', '0.01');
        
        return config.toString();
    }

    // Validation Methods
    validateConfiguration(configFiles) {
        const validationResults = {
            valid: true,
            errors: [],
            warnings: []
        };
        
        try {
            // Validate each configuration file
            configFiles.forEach((content, filename) => {
                const fileValidation = this.validateConfigurationFile(filename, content);
                
                if (!fileValidation.valid) {
                    validationResults.valid = false;
                    validationResults.errors.push(...fileValidation.errors);
                }
                
                validationResults.warnings.push(...fileValidation.warnings);
            });
            
            // Cross-file validation
            this.performCrossFileValidation(configFiles, validationResults);
            
        } catch (error) {
            validationResults.valid = false;
            validationResults.errors.push(`Validation error: ${error.message}`);
        }
        
        return validationResults;
    }

    validateConfigurationFile(filename, content) {
        const results = {
            valid: true,
            errors: [],
            warnings: []
        };
        
        // Parse INI content and validate
        const parsedConfig = this.parseINIContent(content);
        
        // Validate based on file type
        if (filename.includes('OpenSim.ini')) {
            this.validateOpenSimConfig(parsedConfig, results);
        } else if (filename.includes('Regions/')) {
            this.validateRegionConfig(parsedConfig, results);
        }
        
        return results;
    }

    parseINIContent(content) {
        const config = {};
        let currentSection = null;
        
        content.split('\n').forEach(line => {
            line = line.trim();
            
            if (line.startsWith('[') && line.endsWith(']')) {
                currentSection = line.slice(1, -1);
                config[currentSection] = {};
            } else if (line.includes('=') && currentSection) {
                const [key, value] = line.split('=').map(s => s.trim());
                config[currentSection][key] = value;
            }
        });
        
        return config;
    }

    performCrossFileValidation(configFiles, validationResults) {
        // Validate consistency across configuration files
        // Check for port conflicts, database consistency, etc.
        
        const ports = new Set();
        
        configFiles.forEach((content, filename) => {
            if (filename.includes('.ini')) {
                const config = this.parseINIContent(content);
                
                // Check for port conflicts
                Object.values(config).forEach(section => {
                    Object.entries(section).forEach(([key, value]) => {
                        if (key.toLowerCase().includes('port') && !isNaN(value)) {
                            const port = parseInt(value);
                            if (ports.has(port)) {
                                validationResults.errors.push(`Port conflict: ${port} is used in multiple configurations`);
                                validationResults.valid = false;
                            }
                            ports.add(port);
                        }
                    });
                });
            }
        });
    }
}

// Configuration File Builder Class
class ConfigurationFile {
    constructor(filename) {
        this.filename = filename;
        this.content = [];
        this.currentSection = null;
    }

    addComment(comment) {
        if (comment === '') {
            this.content.push('');
        } else {
            this.content.push(`; ${comment}`);
        }
        return this;
    }

    addSection(sectionName) {
        this.currentSection = sectionName;
        if (this.content.length > 0) {
            this.content.push('');
        }
        this.content.push(`[${sectionName}]`);
        return this;
    }

    addSetting(key, value) {
        if (this.currentSection) {
            // Handle different value types
            let formattedValue = value;
            
            if (typeof value === 'boolean') {
                formattedValue = value ? 'true' : 'false';
            } else if (typeof value === 'string' && value.includes(' ')) {
                // Quote strings with spaces if needed
                formattedValue = value;
            }
            
            this.content.push(`${key} = ${formattedValue}`);
        }
        return this;
    }

    addRawLine(line) {
        this.content.push(line);
        return this;
    }

    toString() {
        return this.content.join('\n');
    }
}

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.ConfigurationParser = ConfigurationParser;
    window.ConfigurationFile = ConfigurationFile;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ConfigurationParser, ConfigurationFile };
}