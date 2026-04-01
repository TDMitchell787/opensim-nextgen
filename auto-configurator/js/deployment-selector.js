// OpenSim Next Auto-Configurator - Deployment Type Selector
// Intelligent deployment type selection with optimized defaults and recommendations

class DeploymentSelector {
    constructor() {
        this.deploymentTypes = new Map();
        this.defaultConfigurations = new Map();
        this.autoDetectionRules = new Map();
        this.migrationPaths = new Map();
        
        this.initializeDeploymentTypes();
        this.initializeDefaultConfigurations();
        this.initializeAutoDetectionRules();
        this.initializeMigrationPaths();
        
        console.log('Deployment selector initialized with smart defaults');
    }

    initializeDeploymentTypes() {
        // Development deployment type
        this.registerDeploymentType('development', {
            name: 'Development Environment',
            description: 'Optimized for rapid development, testing, and learning',
            icon: 'laptop',
            suitableFor: [
                'Solo developers and small teams',
                'Testing new features and configurations',
                'Learning OpenSim development',
                'Prototyping virtual world concepts',
                'Local network testing'
            ],
            characteristics: {
                complexity: 'low',
                maintenanceOverhead: 'minimal',
                scalability: 'limited',
                security: 'basic',
                performance: 'moderate',
                resourceUsage: 'low'
            },
            technicalSpecs: {
                expectedUsers: '1-10 concurrent',
                regionLimit: '1-4 regions',
                databaseType: 'SQLite',
                physicsEngine: 'ODE or Basic',
                networkScope: 'Local/LAN only',
                securityLevel: 'Development-friendly'
            },
            advantages: [
                'Quick setup and deployment',
                'Minimal resource requirements',
                'Easy configuration changes',
                'Built-in debugging features',
                'No external dependencies'
            ],
            limitations: [
                'Not suitable for production use',
                'Limited concurrent user capacity',
                'Basic security configuration',
                'Single-server architecture only'
            ],
            estimatedSetupTime: '15-30 minutes',
            recommendedHardware: {
                cpu: '4 cores @ 2.4GHz+',
                memory: '8GB RAM',
                storage: '50GB SSD',
                network: '100Mbps'
            }
        });

        // Production deployment type
        this.registerDeploymentType('production', {
            name: 'Production Environment',
            description: 'Battle-tested configuration for live virtual worlds',
            icon: 'server',
            suitableFor: [
                'Live virtual world communities',
                'Commercial virtual environments',
                'Educational institutions',
                'Corporate virtual meetings',
                'Public-facing virtual spaces'
            ],
            characteristics: {
                complexity: 'medium',
                maintenanceOverhead: 'moderate',
                scalability: 'good',
                security: 'high',
                performance: 'high',
                resourceUsage: 'medium-high'
            },
            technicalSpecs: {
                expectedUsers: '10-500 concurrent',
                regionLimit: '4-32 regions',
                databaseType: 'PostgreSQL',
                physicsEngine: 'Bullet or UBODE',
                networkScope: 'Internet-facing',
                securityLevel: 'Production-grade SSL/TLS'
            },
            advantages: [
                'High performance and reliability',
                'Comprehensive security features',
                'Professional monitoring and logging',
                'Backup and recovery systems',
                'Load balancing capabilities'
            ],
            limitations: [
                'Higher setup complexity',
                'Requires system administration knowledge',
                'Higher resource requirements',
                'Regular maintenance needed'
            ],
            estimatedSetupTime: '2-4 hours',
            recommendedHardware: {
                cpu: '16 cores @ 3.0GHz+',
                memory: '32GB RAM',
                storage: '500GB NVMe SSD',
                network: '1Gbps dedicated'
            }
        });

        // Grid deployment type
        this.registerDeploymentType('grid', {
            name: 'Grid Environment',
            description: 'Distributed multi-server architecture for massive scale',
            icon: 'network',
            suitableFor: [
                'Large-scale virtual world grids',
                'Multi-region distributed deployments',
                'Enterprise virtual environments',
                'Educational grid networks',
                'Commercial metaverse platforms'
            ],
            characteristics: {
                complexity: 'high',
                maintenanceOverhead: 'high',
                scalability: 'excellent',
                security: 'enterprise',
                performance: 'maximum',
                resourceUsage: 'high'
            },
            technicalSpecs: {
                expectedUsers: '100-10,000+ concurrent',
                regionLimit: '32-1000+ regions',
                databaseType: 'PostgreSQL cluster',
                physicsEngine: 'POS with GPU acceleration',
                networkScope: 'Global distribution',
                securityLevel: 'Zero trust with OpenZiti'
            },
            advantages: [
                'Massive scalability potential',
                'Geographic distribution',
                'Enterprise-grade security',
                'High availability and fault tolerance',
                'Advanced monitoring and analytics'
            ],
            limitations: [
                'Complex setup and configuration',
                'Requires expert system administration',
                'High infrastructure costs',
                'Advanced networking knowledge required'
            ],
            estimatedSetupTime: '1-2 days',
            recommendedHardware: {
                cpu: '64+ cores @ 3.5GHz+',
                memory: '128GB+ RAM',
                storage: '2TB+ NVMe SSD RAID',
                network: '10Gbps+ fiber'
            }
        });

        console.log('Deployment types configured');
    }

    initializeDefaultConfigurations() {
        // Development defaults
        this.setDefaultConfiguration('development', {
            general: {
                gridName: 'OpenSim Next Development Grid',
                gridNick: 'devgrid',
                welcomeMessage: 'Welcome to your development environment!',
                allowAnonymousUsers: true,
                publicGrid: false
            },
            network: {
                httpPort: 9000,
                httpsEnabled: false,
                externalHostname: 'localhost',
                internalIP: '127.0.0.1'
            },
            database: {
                type: 'SQLite',
                connectionString: 'Data Source=./OpenSim.db;Version=3;',
                pooling: false,
                connectionLifetime: 600
            },
            physics: {
                engine: 'ODE',
                meshSculptedPrim: true,
                meshSculptLOD: 32,
                avPIDD: 3200.0,
                avPIDP: 1400.0
            },
            security: {
                passwordComplexity: false,
                sessionTimeout: 3600,
                bruteForceProtection: false,
                allowGodFunctions: true,
                restrictInventoryTransfers: false
            },
            performance: {
                maxPrims: 15000,
                maxScripts: 1000,
                scriptTimeout: 30,
                cacheAssets: true,
                cacheTimeout: 48
            },
            logging: {
                logLevel: 'DEBUG',
                consoleLogging: true,
                fileLogging: true,
                logRotation: false
            }
        });

        // Production defaults
        this.setDefaultConfiguration('production', {
            general: {
                gridName: 'OpenSim Next Production Grid',
                gridNick: 'prodgrid',
                welcomeMessage: 'Welcome to our virtual world!',
                allowAnonymousUsers: false,
                publicGrid: true
            },
            network: {
                httpPort: 80,
                httpsPort: 443,
                httpsEnabled: true,
                externalHostname: 'yourgrid.com',
                internalIP: '0.0.0.0'
            },
            database: {
                type: 'PostgreSQL',
                connectionString: 'Host=localhost;Database=opensim;Username=opensim;Password=',
                pooling: true,
                connectionLifetime: 300,
                maxPoolSize: 50
            },
            physics: {
                engine: 'Bullet',
                meshSculptedPrim: true,
                meshSculptLOD: 64,
                avPIDD: 2200.0,
                avPIDP: 900.0,
                bulletSolver: 'Solver2',
                bulletFPS: 45
            },
            security: {
                passwordComplexity: true,
                sessionTimeout: 1800,
                bruteForceProtection: true,
                allowGodFunctions: false,
                restrictInventoryTransfers: true,
                sslCertificatePath: '/etc/ssl/certs/opensim.crt',
                sslPrivateKeyPath: '/etc/ssl/private/opensim.key'
            },
            performance: {
                maxPrims: 45000,
                maxScripts: 3000,
                scriptTimeout: 25,
                cacheAssets: true,
                cacheTimeout: 24,
                enableStatistics: true,
                statisticsUpdateInterval: 5000
            },
            logging: {
                logLevel: 'INFO',
                consoleLogging: false,
                fileLogging: true,
                logRotation: true,
                maxLogFiles: 10,
                maxLogSize: '10MB'
            },
            monitoring: {
                enabled: true,
                metricsEndpoint: '/metrics',
                healthCheckEndpoint: '/health',
                alerting: true
            }
        });

        // Grid defaults
        this.setDefaultConfiguration('grid', {
            general: {
                gridName: 'OpenSim Next Enterprise Grid',
                gridNick: 'enterprise',
                welcomeMessage: 'Welcome to our enterprise metaverse!',
                allowAnonymousUsers: false,
                publicGrid: true,
                hypergridEnabled: true
            },
            network: {
                httpPort: 80,
                httpsPort: 443,
                httpsEnabled: true,
                externalHostname: 'grid.enterprise.com',
                internalIP: '0.0.0.0',
                loadBalancer: true,
                cdnEnabled: true
            },
            database: {
                type: 'PostgreSQL',
                clustering: true,
                connectionString: 'Host=db-cluster.internal;Database=opensim_grid;Username=opensim;Password=',
                pooling: true,
                connectionLifetime: 180,
                maxPoolSize: 100,
                readReplicas: true
            },
            physics: {
                engine: 'POS',
                gpuAcceleration: true,
                meshSculptedPrim: true,
                meshSculptLOD: 128,
                particleSystem: true,
                fluidDynamics: true,
                avPIDD: 1800.0,
                avPIDP: 600.0
            },
            security: {
                passwordComplexity: true,
                twoFactorAuth: true,
                sessionTimeout: 900,
                bruteForceProtection: true,
                allowGodFunctions: false,
                restrictInventoryTransfers: true,
                zeroTrust: true,
                openZitiEnabled: true,
                encryptionAtRest: true,
                sslCertificatePath: '/etc/ssl/enterprise/opensim.crt',
                sslPrivateKeyPath: '/etc/ssl/enterprise/opensim.key'
            },
            performance: {
                maxPrims: 100000,
                maxScripts: 10000,
                scriptTimeout: 20,
                cacheAssets: true,
                distributedCache: true,
                cacheTimeout: 12,
                enableStatistics: true,
                statisticsUpdateInterval: 1000,
                loadBalancing: true,
                autoScaling: true
            },
            logging: {
                logLevel: 'WARN',
                consoleLogging: false,
                fileLogging: true,
                logRotation: true,
                maxLogFiles: 30,
                maxLogSize: '100MB',
                centralizedLogging: true,
                logAggregation: 'elasticsearch'
            },
            monitoring: {
                enabled: true,
                distributedTracing: true,
                metricsEndpoint: '/metrics',
                healthCheckEndpoint: '/health',
                alerting: true,
                slaMonitoring: true,
                performanceDashboard: true
            },
            clustering: {
                enabled: true,
                nodeDiscovery: 'consul',
                serviceRegistry: true,
                circuitBreaker: true,
                retryPolicy: 'exponential'
            }
        });

        console.log('Default configurations initialized');
    }

    initializeAutoDetectionRules() {
        // Hardware-based detection rules
        this.registerAutoDetectionRule('hardware_capacity', {
            name: 'Hardware Capacity Detection',
            priority: 'high',
            detector: (systemInfo) => {
                if (!systemInfo) return null;
                
                const ram = this.parseMemorySize(systemInfo.memory);
                const cpuCores = parseInt(systemInfo.cpuCores) || 4;
                
                // Grid deployment indicators
                if (ram >= 64 && cpuCores >= 32) {
                    return {
                        recommendation: 'grid',
                        confidence: 0.9,
                        reasoning: 'High-end hardware suitable for grid deployment'
                    };
                }
                
                // Production deployment indicators  
                if (ram >= 16 && cpuCores >= 8) {
                    return {
                        recommendation: 'production',
                        confidence: 0.8,
                        reasoning: 'Sufficient hardware for production deployment'
                    };
                }
                
                // Development deployment default
                return {
                    recommendation: 'development',
                    confidence: 0.7,
                    reasoning: 'Hardware suitable for development environment'
                };
            }
        });

        // Network environment detection
        this.registerAutoDetectionRule('network_environment', {
            name: 'Network Environment Detection',
            priority: 'medium',
            detector: (networkInfo) => {
                if (!networkInfo) return null;
                
                const hasPublicIP = networkInfo.hasPublicIP;
                const bandwidth = parseInt(networkInfo.bandwidth) || 100;
                const domainConfigured = networkInfo.domain && networkInfo.domain !== 'localhost';
                
                // Grid indicators
                if (hasPublicIP && bandwidth >= 1000 && domainConfigured) {
                    return {
                        recommendation: 'grid',
                        confidence: 0.8,
                        reasoning: 'Public IP, high bandwidth, and domain configuration suggest grid deployment'
                    };
                }
                
                // Production indicators
                if (hasPublicIP && domainConfigured) {
                    return {
                        recommendation: 'production',
                        confidence: 0.7,
                        reasoning: 'Public IP and domain configuration suggest production deployment'
                    };
                }
                
                // Development default
                return {
                    recommendation: 'development',
                    confidence: 0.6,
                    reasoning: 'Local network configuration suitable for development'
                };
            }
        });

        // Usage pattern detection
        this.registerAutoDetectionRule('usage_patterns', {
            name: 'Usage Pattern Detection',
            priority: 'medium',
            detector: (usageInfo) => {
                if (!usageInfo) return null;
                
                const expectedUsers = parseInt(usageInfo.expectedUsers) || 5;
                const expectedRegions = parseInt(usageInfo.expectedRegions) || 1;
                const isCommercial = usageInfo.isCommercial;
                
                // Grid deployment indicators
                if (expectedUsers > 100 || expectedRegions > 16 || isCommercial) {
                    return {
                        recommendation: 'grid',
                        confidence: 0.85,
                        reasoning: 'High user count or commercial use suggests grid deployment'
                    };
                }
                
                // Production deployment indicators
                if (expectedUsers > 10 || expectedRegions > 4) {
                    return {
                        recommendation: 'production',
                        confidence: 0.75,
                        reasoning: 'Medium scale usage suggests production deployment'
                    };
                }
                
                // Development default
                return {
                    recommendation: 'development',
                    confidence: 0.8,
                    reasoning: 'Small scale usage suitable for development environment'
                };
            }
        });

        console.log('Auto-detection rules initialized');
    }

    initializeMigrationPaths() {
        // Development to Production migration
        this.registerMigrationPath('development', 'production', {
            name: 'Development to Production',
            description: 'Upgrade from development to production-ready configuration',
            difficulty: 'medium',
            estimatedTime: '2-4 hours',
            prerequisites: [
                'Database migration from SQLite to PostgreSQL',
                'SSL certificate acquisition and installation',
                'Domain name configuration',
                'Security hardening review'
            ],
            steps: [
                {
                    step: 1,
                    title: 'Database Migration',
                    description: 'Migrate data from SQLite to PostgreSQL',
                    automated: true,
                    critical: true
                },
                {
                    step: 2,
                    title: 'Security Configuration',
                    description: 'Enable SSL/TLS and security features',
                    automated: false,
                    critical: true
                },
                {
                    step: 3,
                    title: 'Performance Optimization',
                    description: 'Optimize settings for production load',
                    automated: true,
                    critical: false
                },
                {
                    step: 4,
                    title: 'Monitoring Setup',
                    description: 'Configure monitoring and logging',
                    automated: true,
                    critical: false
                }
            ],
            warnings: [
                'Database migration may take significant time with large datasets',
                'SSL certificate setup requires domain validation',
                'Production settings may affect development workflows'
            ]
        });

        // Production to Grid migration
        this.registerMigrationPath('production', 'grid', {
            name: 'Production to Grid',
            description: 'Scale up to distributed grid architecture',
            difficulty: 'high',
            estimatedTime: '1-2 days',
            prerequisites: [
                'Database clustering setup',
                'Load balancer configuration',
                'Zero trust network architecture',
                'Advanced monitoring infrastructure'
            ],
            steps: [
                {
                    step: 1,
                    title: 'Database Clustering',
                    description: 'Set up PostgreSQL clustering for high availability',
                    automated: false,
                    critical: true
                },
                {
                    step: 2,
                    title: 'Zero Trust Network',
                    description: 'Implement OpenZiti zero trust architecture',
                    automated: false,
                    critical: true
                },
                {
                    step: 3,
                    title: 'Load Balancing',
                    description: 'Configure load balancing and auto-scaling',
                    automated: false,
                    critical: true
                },
                {
                    step: 4,
                    title: 'Advanced Monitoring',
                    description: 'Set up distributed monitoring and alerting',
                    automated: false,
                    critical: false
                }
            ],
            warnings: [
                'Grid deployment requires significant infrastructure expertise',
                'Zero trust network setup is complex and critical',
                'Load balancing configuration affects all grid services'
            ]
        });

        console.log('Migration paths initialized');
    }

    registerDeploymentType(name, config) {
        this.deploymentTypes.set(name, config);
    }

    setDefaultConfiguration(deploymentType, config) {
        this.defaultConfigurations.set(deploymentType, config);
    }

    registerAutoDetectionRule(name, rule) {
        this.autoDetectionRules.set(name, rule);
    }

    registerMigrationPath(from, to, path) {
        const key = `${from}->${to}`;
        this.migrationPaths.set(key, path);
    }

    // Main auto-detection method
    autoDetectDeploymentType(systemInfo) {
        const detectionResults = [];
        
        // Run all detection rules
        for (const [name, rule] of this.autoDetectionRules) {
            try {
                const result = rule.detector(systemInfo);
                if (result) {
                    detectionResults.push({
                        rule: name,
                        priority: rule.priority,
                        ...result
                    });
                }
            } catch (error) {
                console.warn(`Auto-detection rule ${name} failed:`, error);
            }
        }

        // Calculate weighted recommendation
        return this.calculateWeightedRecommendation(detectionResults);
    }

    calculateWeightedRecommendation(results) {
        const scores = {
            development: 0,
            production: 0,
            grid: 0
        };

        const priorityWeights = {
            high: 3,
            medium: 2,
            low: 1
        };

        // Calculate weighted scores
        for (const result of results) {
            const weight = priorityWeights[result.priority] || 1;
            const score = result.confidence * weight;
            scores[result.recommendation] += score;
        }

        // Find the highest scoring recommendation
        const maxScore = Math.max(...Object.values(scores));
        const recommendation = Object.keys(scores).find(key => scores[key] === maxScore);

        // Calculate overall confidence
        const totalWeight = results.reduce((sum, r) => sum + (priorityWeights[r.priority] || 1), 0);
        const confidence = totalWeight > 0 ? maxScore / totalWeight : 0;

        return {
            recommendation,
            confidence: Math.min(confidence, 1.0),
            scores,
            detectionResults: results,
            alternativeRecommendations: this.getAlternativeRecommendations(scores, recommendation)
        };
    }

    getAlternativeRecommendations(scores, primaryRecommendation) {
        const alternatives = [];
        const sortedScores = Object.entries(scores)
            .filter(([key]) => key !== primaryRecommendation)
            .sort(([,a], [,b]) => b - a);

        for (const [type, score] of sortedScores) {
            if (score > 0) {
                alternatives.push({
                    type,
                    score,
                    confidence: score / Math.max(...Object.values(scores))
                });
            }
        }

        return alternatives;
    }

    // Get deployment configuration with smart defaults
    getDeploymentConfiguration(deploymentType, customizations = {}) {
        const defaults = this.defaultConfigurations.get(deploymentType);
        if (!defaults) {
            throw new Error(`Unknown deployment type: ${deploymentType}`);
        }

        // Deep merge defaults with customizations
        return this.deepMerge(defaults, customizations);
    }

    // Get deployment type information
    getDeploymentTypeInfo(deploymentType) {
        return this.deploymentTypes.get(deploymentType);
    }

    // Get all available deployment types
    getAllDeploymentTypes() {
        return Array.from(this.deploymentTypes.entries()).map(([name, config]) => ({
            name,
            ...config
        }));
    }

    // Get migration path between deployment types
    getMigrationPath(fromType, toType) {
        const key = `${fromType}->${toType}`;
        return this.migrationPaths.get(key);
    }

    // Get all available migration paths from a specific deployment type
    getAvailableMigrationPaths(fromType) {
        const paths = [];
        for (const [key, path] of this.migrationPaths) {
            if (key.startsWith(`${fromType}->`)) {
                const toType = key.split('->')[1];
                paths.push({
                    toType,
                    ...path
                });
            }
        }
        return paths;
    }

    // Validate deployment configuration
    validateDeploymentConfiguration(deploymentType, configuration) {
        const deploymentInfo = this.getDeploymentTypeInfo(deploymentType);
        if (!deploymentInfo) {
            return {
                valid: false,
                errors: [`Unknown deployment type: ${deploymentType}`]
            };
        }

        const errors = [];
        const warnings = [];

        // Validate based on deployment type characteristics
        if (deploymentType === 'production') {
            if (!configuration.security?.httpsEnabled) {
                errors.push('Production deployments require HTTPS to be enabled');
            }
            if (!configuration.security?.sslCertificatePath) {
                errors.push('Production deployments require SSL certificate configuration');
            }
            if (configuration.database?.type === 'SQLite') {
                warnings.push('SQLite is not recommended for production deployments');
            }
        }

        if (deploymentType === 'grid') {
            if (!configuration.security?.zeroTrust) {
                errors.push('Grid deployments require zero trust networking');
            }
            if (!configuration.database?.clustering) {
                errors.push('Grid deployments require database clustering');
            }
            if (!configuration.monitoring?.distributedTracing) {
                warnings.push('Distributed tracing is recommended for grid deployments');
            }
        }

        return {
            valid: errors.length === 0,
            errors,
            warnings
        };
    }

    // Utility methods
    parseMemorySize(memoryString) {
        if (!memoryString) return 0;
        const match = memoryString.match(/(\d+(?:\.\d+)?)\s*(GB|MB|KB)?/i);
        if (!match) return 0;
        
        const value = parseFloat(match[1]);
        const unit = (match[2] || 'MB').toUpperCase();
        
        switch (unit) {
            case 'GB': return value;
            case 'MB': return value / 1024;
            case 'KB': return value / (1024 * 1024);
            default: return value;
        }
    }

    deepMerge(target, source) {
        const result = { ...target };
        
        for (const key in source) {
            if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
                result[key] = this.deepMerge(result[key] || {}, source[key]);
            } else {
                result[key] = source[key];
            }
        }
        
        return result;
    }

    // Generate deployment comparison matrix
    generateComparisonMatrix() {
        const types = this.getAllDeploymentTypes();
        const comparison = {
            deploymentTypes: types.map(t => t.name),
            characteristics: {},
            technicalSpecs: {},
            hardware: {}
        };

        // Build comparison data
        const characteristics = ['complexity', 'maintenanceOverhead', 'scalability', 'security', 'performance'];
        for (const char of characteristics) {
            comparison.characteristics[char] = types.map(t => t.characteristics[char]);
        }

        const specs = ['expectedUsers', 'regionLimit', 'databaseType', 'physicsEngine'];
        for (const spec of specs) {
            comparison.technicalSpecs[spec] = types.map(t => t.technicalSpecs[spec]);
        }

        const hardware = ['cpu', 'memory', 'storage', 'network'];
        for (const hw of hardware) {
            comparison.hardware[hw] = types.map(t => t.recommendedHardware[hw]);
        }

        return comparison;
    }
}

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.DeploymentSelector = DeploymentSelector;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { DeploymentSelector };
}