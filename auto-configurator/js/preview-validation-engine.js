// OpenSim Next Auto-Configurator - Preview Validation Engine
// Real-time configuration validation with intelligent error detection

class PreviewValidationEngine {
    constructor() {
        this.validationRules = new Map();
        this.validationCache = new Map();
        this.cacheTimeout = 5000; // 5 seconds
        
        this.initializeValidationRules();
    }

    initializeValidationRules() {
        // General configuration validation rules
        this.addValidationRule('general.gridName', {
            required: true,
            type: 'string',
            minLength: 3,
            maxLength: 50,
            pattern: /^[a-zA-Z0-9\s\-_]+$/,
            message: 'Grid name must be 3-50 characters, alphanumeric with spaces, hyphens, or underscores'
        });

        this.addValidationRule('general.gridNick', {
            required: true,
            type: 'string',
            minLength: 3,
            maxLength: 20,
            pattern: /^[a-zA-Z0-9\-_]+$/,
            message: 'Grid nick must be 3-20 characters, alphanumeric with hyphens or underscores only'
        });

        this.addValidationRule('general.welcomeMessage', {
            required: false,
            type: 'string',
            maxLength: 500,
            message: 'Welcome message cannot exceed 500 characters'
        });

        this.addValidationRule('general.deploymentType', {
            required: true,
            type: 'string',
            enum: ['development', 'staging', 'production', 'grid'],
            message: 'Deployment type must be one of: development, staging, production, grid'
        });

        this.addValidationRule('general.adminEmail', {
            required: true,
            type: 'string',
            pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
            message: 'Admin email must be a valid email address'
        });

        // Network configuration validation rules
        this.addValidationRule('network.httpPort', {
            required: true,
            type: 'number',
            min: 1024,
            max: 65535,
            message: 'HTTP port must be between 1024 and 65535'
        });

        this.addValidationRule('network.httpsPort', {
            required: true,
            type: 'number',
            min: 1024,
            max: 65535,
            message: 'HTTPS port must be between 1024 and 65535',
            customValidator: (value, config) => {
                if (config.network.httpsEnabled && value === config.network.httpPort) {
                    return 'HTTPS port cannot be the same as HTTP port';
                }
                return null;
            }
        });

        this.addValidationRule('network.externalHostname', {
            required: true,
            type: 'string',
            pattern: /^[a-zA-Z0-9\-\.]+$/,
            message: 'External hostname must be a valid hostname or IP address'
        });

        this.addValidationRule('network.internalIp', {
            required: true,
            type: 'string',
            pattern: /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/,
            message: 'Internal IP must be a valid IPv4 address'
        });

        this.addValidationRule('network.maxConnections', {
            required: true,
            type: 'number',
            min: 1,
            max: 10000,
            message: 'Max connections must be between 1 and 10000'
        });

        // Database configuration validation rules
        this.addValidationRule('database.type', {
            required: true,
            type: 'string',
            enum: ['sqlite', 'postgresql', 'mysql'],
            message: 'Database type must be one of: sqlite, postgresql, mysql'
        });

        this.addValidationRule('database.host', {
            required: (config) => config.database.type !== 'sqlite',
            type: 'string',
            minLength: 1,
            message: 'Database host is required for non-SQLite databases'
        });

        this.addValidationRule('database.port', {
            required: (config) => config.database.type !== 'sqlite',
            type: 'number',
            min: 1,
            max: 65535,
            message: 'Database port must be between 1 and 65535'
        });

        this.addValidationRule('database.name', {
            required: true,
            type: 'string',
            minLength: 1,
            pattern: /^[a-zA-Z0-9_\-]+$/,
            message: 'Database name must contain only alphanumeric characters, underscores, or hyphens'
        });

        this.addValidationRule('database.username', {
            required: (config) => config.database.type !== 'sqlite',
            type: 'string',
            minLength: 1,
            message: 'Database username is required for non-SQLite databases'
        });

        this.addValidationRule('database.poolSize', {
            required: true,
            type: 'number',
            min: 1,
            max: 100,
            message: 'Database pool size must be between 1 and 100'
        });

        // Physics configuration validation rules
        this.addValidationRule('physics.defaultEngine', {
            required: true,
            type: 'string',
            enum: ['ODE', 'UBODE', 'Bullet', 'POS', 'Basic'],
            message: 'Physics engine must be one of: ODE, UBODE, Bullet, POS, Basic'
        });

        this.addValidationRule('physics.timestep', {
            required: true,
            type: 'number',
            min: 0.001,
            max: 0.1,
            message: 'Physics timestep must be between 0.001 and 0.1 seconds'
        });

        this.addValidationRule('physics.maxBodies', {
            required: true,
            type: 'number',
            min: 100,
            max: 100000,
            message: 'Max physics bodies must be between 100 and 100000'
        });

        this.addValidationRule('physics.gravityZ', {
            required: true,
            type: 'number',
            min: -50,
            max: 50,
            message: 'Gravity Z must be between -50 and 50'
        });

        // Security configuration validation rules
        this.addValidationRule('security.apiKey', {
            required: true,
            type: 'string',
            minLength: 16,
            pattern: /^[a-zA-Z0-9\-_!@#$%^&*()]+$/,
            message: 'API key must be at least 16 characters with alphanumeric and special characters',
            customValidator: (value) => {
                if (value === 'default-key-change-me') {
                    return 'API key must be changed from default value for security';
                }
                return null;
            }
        });

        this.addValidationRule('security.sessionTimeout', {
            required: true,
            type: 'number',
            min: 300,
            max: 86400,
            message: 'Session timeout must be between 300 seconds (5 minutes) and 86400 seconds (24 hours)'
        });

        this.addValidationRule('security.maxRequestsPerMinute', {
            required: true,
            type: 'number',
            min: 10,
            max: 1000,
            message: 'Max requests per minute must be between 10 and 1000'
        });

        // Performance configuration validation rules
        this.addValidationRule('performance.maxPrims', {
            required: true,
            type: 'number',
            min: 1000,
            max: 100000,
            message: 'Max prims must be between 1000 and 100000'
        });

        this.addValidationRule('performance.maxScripts', {
            required: true,
            type: 'number',
            min: 100,
            max: 10000,
            message: 'Max scripts must be between 100 and 10000'
        });

        this.addValidationRule('performance.scriptTimeout', {
            required: true,
            type: 'number',
            min: 1,
            max: 300,
            message: 'Script timeout must be between 1 and 300 seconds'
        });

        this.addValidationRule('performance.cacheTimeout', {
            required: true,
            type: 'number',
            min: 1,
            max: 168,
            message: 'Cache timeout must be between 1 and 168 hours (1 week)'
        });

        this.addValidationRule('performance.threadPoolSize', {
            required: true,
            type: 'number',
            min: 1,
            max: 32,
            message: 'Thread pool size must be between 1 and 32'
        });

        // Regions configuration validation rules
        this.addValidationRule('regions', {
            required: true,
            type: 'array',
            minLength: 1,
            message: 'At least one region must be configured',
            customValidator: (regions) => {
                const names = regions.map(r => r.name);
                const uniqueNames = new Set(names);
                if (names.length !== uniqueNames.size) {
                    return 'Region names must be unique';
                }

                const coordinates = regions.map(r => `${r.location.x},${r.location.y}`);
                const uniqueCoords = new Set(coordinates);
                if (coordinates.length !== uniqueCoords.size) {
                    return 'Region coordinates must be unique';
                }

                return null;
            }
        });

        // Grid configuration validation rules
        this.addValidationRule('grid.mode', {
            required: true,
            type: 'string',
            enum: ['standalone', 'grid', 'hypergrid'],
            message: 'Grid mode must be one of: standalone, grid, hypergrid'
        });
    }

    addValidationRule(path, rule) {
        this.validationRules.set(path, rule);
    }

    validateConfiguration(config) {
        const cacheKey = this.generateCacheKey(config);
        
        // Check cache first
        if (this.validationCache.has(cacheKey)) {
            const cached = this.validationCache.get(cacheKey);
            if (Date.now() - cached.timestamp < this.cacheTimeout) {
                return cached.result;
            }
        }

        const result = this.performValidation(config);
        
        // Cache the result
        this.validationCache.set(cacheKey, {
            result,
            timestamp: Date.now()
        });

        // Clean old cache entries
        this.cleanCache();

        return result;
    }

    performValidation(config) {
        const errors = [];
        const warnings = [];
        const info = [];

        // Validate individual fields
        for (const [path, rule] of this.validationRules.entries()) {
            const value = this.getValueByPath(config, path);
            const fieldValidation = this.validateField(path, value, rule, config);
            
            if (fieldValidation.error) {
                errors.push(fieldValidation.error);
            }
            if (fieldValidation.warning) {
                warnings.push(fieldValidation.warning);
            }
            if (fieldValidation.info) {
                info.push(fieldValidation.info);
            }
        }

        // Perform cross-field validation
        const crossValidation = this.performCrossFieldValidation(config);
        errors.push(...crossValidation.errors);
        warnings.push(...crossValidation.warnings);
        info.push(...crossValidation.info);

        // Perform deployment-specific validation
        const deploymentValidation = this.performDeploymentValidation(config);
        errors.push(...deploymentValidation.errors);
        warnings.push(...deploymentValidation.warnings);
        info.push(...deploymentValidation.info);

        const isValid = errors.length === 0;
        const summary = this.generateValidationSummary(errors, warnings, info);

        return {
            isValid,
            summary,
            errors,
            warnings,
            info,
            timestamp: new Date().toISOString()
        };
    }

    validateField(path, value, rule, config) {
        const result = { error: null, warning: null, info: null };

        // Check if field is required
        if (this.isFieldRequired(rule, config) && this.isEmpty(value)) {
            result.error = {
                id: `${path}-required`,
                path,
                message: `${this.formatFieldName(path)} is required`,
                type: 'required',
                autofix: false
            };
            return result;
        }

        // Skip validation if field is empty and not required
        if (this.isEmpty(value) && !this.isFieldRequired(rule, config)) {
            return result;
        }

        // Type validation
        if (rule.type && !this.validateType(value, rule.type)) {
            result.error = {
                id: `${path}-type`,
                path,
                message: `${this.formatFieldName(path)} must be of type ${rule.type}`,
                type: 'type',
                autofix: false
            };
            return result;
        }

        // Enum validation
        if (rule.enum && !rule.enum.includes(value)) {
            result.error = {
                id: `${path}-enum`,
                path,
                message: `${this.formatFieldName(path)} must be one of: ${rule.enum.join(', ')}`,
                type: 'enum',
                autofix: true,
                suggestion: `Try: ${rule.enum[0]}`
            };
            return result;
        }

        // Range validation
        if (rule.min !== undefined && value < rule.min) {
            result.error = {
                id: `${path}-min`,
                path,
                message: `${this.formatFieldName(path)} must be at least ${rule.min}`,
                type: 'range',
                autofix: true,
                suggestion: `Set to ${rule.min}`
            };
            return result;
        }

        if (rule.max !== undefined && value > rule.max) {
            result.error = {
                id: `${path}-max`,
                path,
                message: `${this.formatFieldName(path)} must be at most ${rule.max}`,
                type: 'range',
                autofix: true,
                suggestion: `Set to ${rule.max}`
            };
            return result;
        }

        // Length validation
        if (rule.minLength !== undefined && value.length < rule.minLength) {
            result.error = {
                id: `${path}-minlength`,
                path,
                message: `${this.formatFieldName(path)} must be at least ${rule.minLength} characters`,
                type: 'length',
                autofix: false
            };
            return result;
        }

        if (rule.maxLength !== undefined && value.length > rule.maxLength) {
            result.error = {
                id: `${path}-maxlength`,
                path,
                message: `${this.formatFieldName(path)} must be at most ${rule.maxLength} characters`,
                type: 'length',
                autofix: true,
                suggestion: `Truncate to ${rule.maxLength} characters`
            };
            return result;
        }

        // Pattern validation
        if (rule.pattern && !rule.pattern.test(value)) {
            result.error = {
                id: `${path}-pattern`,
                path,
                message: rule.message || `${this.formatFieldName(path)} format is invalid`,
                type: 'pattern',
                autofix: false
            };
            return result;
        }

        // Custom validation
        if (rule.customValidator) {
            const customError = rule.customValidator(value, config);
            if (customError) {
                result.error = {
                    id: `${path}-custom`,
                    path,
                    message: customError,
                    type: 'custom',
                    autofix: false
                };
                return result;
            }
        }

        // Generate warnings for potentially problematic values
        const warning = this.generateFieldWarning(path, value, rule, config);
        if (warning) {
            result.warning = warning;
        }

        // Generate info for optimization suggestions
        const infoMsg = this.generateFieldInfo(path, value, rule, config);
        if (infoMsg) {
            result.info = infoMsg;
        }

        return result;
    }

    performCrossFieldValidation(config) {
        const errors = [];
        const warnings = [];
        const info = [];

        // Port conflicts
        if (config.network.httpPort === config.network.httpsPort) {
            errors.push({
                id: 'port-conflict',
                path: 'network.ports',
                message: 'HTTP and HTTPS ports cannot be the same',
                type: 'conflict',
                autofix: true,
                suggestion: 'Use different ports (e.g., 9000 and 9001)'
            });
        }

        if (config.network.httpPort === config.network.websocketPort) {
            errors.push({
                id: 'websocket-port-conflict',
                path: 'network.websocketPort',
                message: 'WebSocket port cannot be the same as HTTP port',
                type: 'conflict',
                autofix: true,
                suggestion: 'Use a different port for WebSocket'
            });
        }

        // Database connection validation
        if (config.database.type !== 'sqlite' && !config.database.host) {
            errors.push({
                id: 'database-host-required',
                path: 'database.host',
                message: 'Database host is required for non-SQLite databases',
                type: 'dependency',
                autofix: false
            });
        }

        // SSL configuration validation
        if (config.network.httpsEnabled && !config.security.sslCertificatePath) {
            warnings.push({
                id: 'ssl-cert-missing',
                path: 'security.sslCertificatePath',
                message: 'SSL certificate path should be configured when HTTPS is enabled',
                type: 'security',
                autofix: false
            });
        }

        // Performance consistency checks
        if (config.physics.maxBodies > config.performance.maxPrims) {
            warnings.push({
                id: 'physics-prim-mismatch',
                path: 'physics.maxBodies',
                message: 'Max physics bodies is higher than max prims - this may cause performance issues',
                type: 'performance',
                autofix: true,
                suggestion: 'Consider aligning physics bodies with prim limits'
            });
        }

        // Region validation
        if (config.regions) {
            for (let i = 0; i < config.regions.length; i++) {
                const region = config.regions[i];
                
                if (region.maxPrims > config.performance.maxPrims) {
                    warnings.push({
                        id: `region-${i}-prim-limit`,
                        path: `regions[${i}].maxPrims`,
                        message: `Region "${region.name}" prim limit exceeds global maximum`,
                        type: 'consistency',
                        autofix: true,
                        suggestion: `Set to ${config.performance.maxPrims} or lower`
                    });
                }
            }
        }

        return { errors, warnings, info };
    }

    performDeploymentValidation(config) {
        const errors = [];
        const warnings = [];
        const info = [];

        const deploymentType = config.general.deploymentType;

        switch (deploymentType) {
            case 'development':
                // Development-specific warnings
                if (config.security.apiKey === 'default-key-change-me') {
                    info.push({
                        id: 'dev-default-api-key',
                        path: 'security.apiKey',
                        message: 'Using default API key is acceptable for development',
                        type: 'info'
                    });
                }

                if (config.database.type === 'sqlite') {
                    info.push({
                        id: 'dev-sqlite-ok',
                        path: 'database.type',
                        message: 'SQLite is suitable for development environments',
                        type: 'info'
                    });
                }
                break;

            case 'production':
                // Production-specific requirements
                if (config.security.apiKey === 'default-key-change-me') {
                    errors.push({
                        id: 'prod-default-api-key',
                        path: 'security.apiKey',
                        message: 'Default API key must be changed for production deployment',
                        type: 'security',
                        autofix: false
                    });
                }

                if (config.database.type === 'sqlite') {
                    warnings.push({
                        id: 'prod-sqlite-warning',
                        path: 'database.type',
                        message: 'SQLite is not recommended for production environments',
                        type: 'performance',
                        autofix: false,
                        suggestion: 'Consider using PostgreSQL or MySQL for better performance'
                    });
                }

                if (!config.network.httpsEnabled) {
                    warnings.push({
                        id: 'prod-https-required',
                        path: 'network.httpsEnabled',
                        message: 'HTTPS should be enabled for production environments',
                        type: 'security',
                        autofix: true,
                        suggestion: 'Enable HTTPS and configure SSL certificates'
                    });
                }

                if (config.performance.maxConnections < 100) {
                    info.push({
                        id: 'prod-low-connections',
                        path: 'performance.maxConnections',
                        message: 'Consider increasing max connections for production load',
                        type: 'performance',
                        suggestion: 'Increase to at least 100 for production'
                    });
                }
                break;

            case 'grid':
                // Grid-specific requirements
                if (config.grid.mode === 'standalone') {
                    warnings.push({
                        id: 'grid-mode-mismatch',
                        path: 'grid.mode',
                        message: 'Grid deployment type should use grid or hypergrid mode',
                        type: 'consistency',
                        autofix: true,
                        suggestion: 'Set grid mode to "grid" or "hypergrid"'
                    });
                }

                if (!config.grid.assetServerUrl) {
                    warnings.push({
                        id: 'grid-asset-server-missing',
                        path: 'grid.assetServerUrl',
                        message: 'Asset server URL should be configured for grid deployments',
                        type: 'configuration',
                        autofix: false
                    });
                }
                break;
        }

        return { errors, warnings, info };
    }

    generateFieldWarning(path, value, rule, config) {
        // Generate warnings for potentially problematic but valid values
        
        if (path === 'performance.maxPrims' && value > 50000) {
            return {
                id: `${path}-high-prims`,
                path,
                message: 'Very high prim limit may impact performance',
                type: 'performance',
                suggestion: 'Monitor server performance with high prim counts'
            };
        }

        if (path === 'physics.timestep' && value < 0.01) {
            return {
                id: `${path}-high-frequency`,
                path,
                message: 'Very low timestep may impact server performance',
                type: 'performance',
                suggestion: 'Consider using 0.0167 (60 FPS) for most applications'
            };
        }

        if (path === 'security.sessionTimeout' && value > 7200) {
            return {
                id: `${path}-long-timeout`,
                path,
                message: 'Long session timeout may pose security risks',
                type: 'security',
                suggestion: 'Consider shorter timeout for better security'
            };
        }

        return null;
    }

    generateFieldInfo(path, value, rule, config) {
        // Generate informational messages for optimization

        if (path === 'physics.defaultEngine') {
            const engineInfo = {
                'ODE': 'Traditional engine, stable for avatars and basic physics',
                'UBODE': 'Enhanced ODE with better performance for large worlds',
                'Bullet': 'Modern engine with advanced features like soft bodies',
                'POS': 'Position-based dynamics, excellent for particles and fluids',
                'Basic': 'Lightweight engine for testing and simple scenarios'
            };

            return {
                id: `${path}-info`,
                path,
                message: engineInfo[value] || 'Physics engine information',
                type: 'info'
            };
        }

        if (path === 'database.type' && value === 'postgresql') {
            return {
                id: `${path}-postgresql-info`,
                path,
                message: 'PostgreSQL offers excellent performance and scalability for OpenSim',
                type: 'info'
            };
        }

        return null;
    }

    getOverallStatus(config) {
        const validation = this.validateConfiguration(config);
        
        if (!validation.isValid) {
            return {
                status: 'error',
                message: `${validation.errors.length} error${validation.errors.length !== 1 ? 's' : ''} found`
            };
        }

        if (validation.warnings.length > 0) {
            return {
                status: 'warning',
                message: `${validation.warnings.length} warning${validation.warnings.length !== 1 ? 's' : ''} found`
            };
        }

        return {
            status: 'valid',
            message: 'Configuration is valid'
        };
    }

    generateValidationSummary(errors, warnings, info) {
        if (errors.length > 0) {
            return `Configuration has ${errors.length} error${errors.length !== 1 ? 's' : ''} that must be fixed before deployment.`;
        }

        if (warnings.length > 0) {
            return `Configuration is valid but has ${warnings.length} warning${warnings.length !== 1 ? 's' : ''} that should be addressed.`;
        }

        if (info.length > 0) {
            return `Configuration is valid. ${info.length} optimization suggestion${info.length !== 1 ? 's' : ''} available.`;
        }

        return 'Configuration is valid and ready for deployment.';
    }

    // Utility methods
    getValueByPath(obj, path) {
        return path.split('.').reduce((current, key) => {
            if (current && typeof current === 'object') {
                return current[key];
            }
            return undefined;
        }, obj);
    }

    isFieldRequired(rule, config) {
        if (typeof rule.required === 'function') {
            return rule.required(config);
        }
        return rule.required === true;
    }

    isEmpty(value) {
        return value === null || 
               value === undefined || 
               value === '' ||
               (Array.isArray(value) && value.length === 0);
    }

    validateType(value, expectedType) {
        switch (expectedType) {
            case 'string':
                return typeof value === 'string';
            case 'number':
                return typeof value === 'number' && !isNaN(value);
            case 'boolean':
                return typeof value === 'boolean';
            case 'array':
                return Array.isArray(value);
            case 'object':
                return typeof value === 'object' && value !== null && !Array.isArray(value);
            default:
                return true;
        }
    }

    formatFieldName(path) {
        return path.split('.').map(part => 
            part.charAt(0).toUpperCase() + part.slice(1).replace(/([A-Z])/g, ' $1')
        ).join(' › ');
    }

    generateCacheKey(config) {
        // Generate a cache key based on configuration content
        return btoa(JSON.stringify(config)).substring(0, 32);
    }

    cleanCache() {
        const now = Date.now();
        for (const [key, cached] of this.validationCache.entries()) {
            if (now - cached.timestamp > this.cacheTimeout) {
                this.validationCache.delete(key);
            }
        }
    }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = PreviewValidationEngine;
}