// OpenSim Next Auto-Configurator - Intelligent Configuration Validation
// Pre-flight validation and optimization system for OpenSim configurations

class ConfigurationValidator {
    constructor() {
        this.validationRules = new Map();
        this.validationHistory = [];
        this.performanceBaseline = null;
        this.deploymentOptimizer = new DeploymentOptimizer();
        this.securityAnalyzer = new SecurityAnalyzer();
        this.preflightChecker = new PreflightChecker();
        
        this.initializeValidationRules();
        this.initializeRealTimeValidation();
    }

    initializeValidationRules() {
        // Core validation rules for different configuration types
        this.registerValidationRule('database_connectivity', {
            priority: 'critical',
            async: true,
            validator: this.validateDatabaseConnectivity.bind(this),
            description: 'Test database connection and validate schema compatibility'
        });

        this.registerValidationRule('port_availability', {
            priority: 'critical',
            async: false,
            validator: this.validatePortAvailability.bind(this),
            description: 'Check for port conflicts and availability'
        });

        this.registerValidationRule('physics_compatibility', {
            priority: 'high',
            async: false,
            validator: this.validatePhysicsConfiguration.bind(this),
            description: 'Validate physics engine settings and requirements'
        });

        this.registerValidationRule('security_requirements', {
            priority: 'high',
            async: false,
            validator: this.validateSecurityConfiguration.bind(this),
            description: 'Analyze security posture and requirements'
        });

        this.registerValidationRule('resource_requirements', {
            priority: 'medium',
            async: true,
            validator: this.validateResourceRequirements.bind(this),
            description: 'Check system resource requirements and capacity'
        });

        this.registerValidationRule('network_topology', {
            priority: 'medium',
            async: false,
            validator: this.validateNetworkTopology.bind(this),
            description: 'Validate network configuration and topology'
        });

        this.registerValidationRule('deployment_optimization', {
            priority: 'medium',
            async: false,
            validator: this.validateDeploymentOptimization.bind(this),
            description: 'Analyze and optimize deployment configuration'
        });

        this.registerValidationRule('compatibility_check', {
            priority: 'low',
            async: true,
            validator: this.validateCompatibility.bind(this),
            description: 'Check compatibility with existing OpenSim installations'
        });

        console.log('Configuration validation rules initialized');
    }

    registerValidationRule(name, rule) {
        this.validationRules.set(name, {
            ...rule,
            name: name,
            lastRun: null,
            results: []
        });
    }

    initializeRealTimeValidation() {
        // Real-time validation as user makes changes
        document.addEventListener('input', (e) => {
            if (e.target.form?.classList.contains('config-form')) {
                this.scheduleValidation('real-time', e.target);
            }
        });

        document.addEventListener('change', (e) => {
            if (e.target.form?.classList.contains('config-form')) {
                this.scheduleValidation('change', e.target);
            }
        });

        // Debounced validation scheduling
        this.validationQueue = new Map();
        this.validationTimer = null;
    }

    scheduleValidation(type, target) {
        const validationKey = `${type}-${target.name || target.id}`;
        this.validationQueue.set(validationKey, { type, target, timestamp: Date.now() });

        // Debounce validation execution
        if (this.validationTimer) {
            clearTimeout(this.validationTimer);
        }

        this.validationTimer = setTimeout(() => {
            this.executeQueuedValidations();
        }, 500); // 500ms debounce
    }

    async executeQueuedValidations() {
        const validations = Array.from(this.validationQueue.values());
        this.validationQueue.clear();

        for (const validation of validations) {
            await this.validateField(validation.target);
        }

        // Update overall validation status
        this.updateValidationStatus();
    }

    async validateField(field) {
        const fieldName = field.name || field.id;
        const fieldValue = field.value;
        const section = this.getFieldSection(field);

        try {
            // Field-specific validation
            const fieldValidation = await this.validateIndividualField(fieldName, fieldValue, section);
            
            // Update field UI
            this.updateFieldValidation(field, fieldValidation);

            // Check if this field affects other validations
            const dependentValidations = this.getDependentValidations(fieldName);
            for (const ruleName of dependentValidations) {
                await this.runValidationRule(ruleName);
            }

        } catch (error) {
            console.error('Field validation error:', error);
            this.updateFieldValidation(field, {
                valid: false,
                severity: 'error',
                message: `Validation error: ${error.message}`
            });
        }
    }

    getFieldSection(field) {
        const section = field.closest('[data-section]');
        return section ? section.dataset.section : 'unknown';
    }

    async validateIndividualField(fieldName, fieldValue, section) {
        const validationMap = {
            // Database fields
            'database_host': this.validateDatabaseHost.bind(this),
            'database_port': this.validateDatabasePort.bind(this),
            'database_name': this.validateDatabaseName.bind(this),
            'database_username': this.validateDatabaseUsername.bind(this),
            'database_connection_string': this.validateConnectionString.bind(this),

            // Network fields
            'http_port': this.validateHttpPort.bind(this),
            'https_port': this.validateHttpsPort.bind(this),
            'external_hostname': this.validateExternalHostname.bind(this),
            'internal_address': this.validateInternalAddress.bind(this),

            // Security fields
            'ssl_certificate': this.validateSSLCertificate.bind(this),
            'ssl_private_key': this.validateSSLPrivateKey.bind(this),
            'admin_password': this.validateAdminPassword.bind(this),

            // Region fields
            'region_name': this.validateRegionName.bind(this),
            'region_location': this.validateRegionLocation.bind(this),
            'region_size': this.validateRegionSize.bind(this),
            'physics_engine': this.validatePhysicsEngine.bind(this),

            // Grid fields
            'grid_name': this.validateGridName.bind(this),
            'grid_admin_email': this.validateGridAdminEmail.bind(this),
            'hypergrid_enabled': this.validateHypergridSettings.bind(this)
        };

        const validator = validationMap[fieldName];
        if (validator) {
            return await validator(fieldValue, section);
        }

        // Default validation
        return this.validateGenericField(fieldName, fieldValue, section);
    }

    // Database validation methods
    async validateDatabaseHost(host) {
        if (!host) {
            return { valid: false, severity: 'error', message: 'Database host is required' };
        }

        // Check if it's a valid hostname or IP
        const hostnameRegex = /^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$/;
        const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;

        if (!hostnameRegex.test(host) && !ipRegex.test(host) && host !== 'localhost') {
            return { valid: false, severity: 'error', message: 'Invalid hostname or IP address format' };
        }

        // Check reachability (if not localhost)
        if (host !== 'localhost' && !ipRegex.test(host)) {
            try {
                const reachable = await this.checkHostReachability(host);
                if (!reachable) {
                    return { valid: true, severity: 'warning', message: 'Host may not be reachable' };
                }
            } catch (error) {
                return { valid: true, severity: 'info', message: 'Could not verify host reachability' };
            }
        }

        return { valid: true, severity: 'success', message: 'Valid database host' };
    }

    validateDatabasePort(port) {
        const portNum = parseInt(port);
        
        if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
            return { valid: false, severity: 'error', message: 'Port must be between 1 and 65535' };
        }

        // Check for common database ports
        const commonPorts = {
            3306: 'MySQL',
            5432: 'PostgreSQL',
            1433: 'SQL Server',
            1521: 'Oracle'
        };

        if (commonPorts[portNum]) {
            return { valid: true, severity: 'success', message: `Standard ${commonPorts[portNum]} port` };
        }

        if (portNum < 1024) {
            return { valid: true, severity: 'warning', message: 'Using privileged port (requires admin rights)' };
        }

        return { valid: true, severity: 'success', message: 'Valid port number' };
    }

    validateDatabaseName(name) {
        if (!name) {
            return { valid: false, severity: 'error', message: 'Database name is required' };
        }

        // Check for valid database name format
        const nameRegex = /^[a-zA-Z][a-zA-Z0-9_]{0,63}$/;
        if (!nameRegex.test(name)) {
            return { valid: false, severity: 'error', message: 'Database name must start with letter and contain only letters, numbers, and underscores' };
        }

        // Check for reserved names
        const reservedNames = ['mysql', 'information_schema', 'performance_schema', 'sys', 'postgres', 'template0', 'template1'];
        if (reservedNames.includes(name.toLowerCase())) {
            return { valid: false, severity: 'error', message: 'Database name conflicts with reserved system name' };
        }

        return { valid: true, severity: 'success', message: 'Valid database name' };
    }

    // Network validation methods
    validateHttpPort(port) {
        return this.validatePortNumber(port, {
            standard: 80,
            alternates: [8080, 9000, 9100],
            reserved: [80, 443, 22, 21, 25]
        });
    }

    validateHttpsPort(port) {
        return this.validatePortNumber(port, {
            standard: 443,
            alternates: [8443, 9001, 9443],
            reserved: [80, 443, 22, 21, 25]
        });
    }

    validatePortNumber(port, options = {}) {
        const portNum = parseInt(port);
        
        if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
            return { valid: false, severity: 'error', message: 'Port must be between 1 and 65535' };
        }

        // Check for standard port
        if (options.standard && portNum === options.standard) {
            return { valid: true, severity: 'success', message: `Standard port for this service` };
        }

        // Check for alternate common ports
        if (options.alternates && options.alternates.includes(portNum)) {
            return { valid: true, severity: 'success', message: 'Common alternate port' };
        }

        // Check for reserved system ports
        if (portNum < 1024) {
            return { valid: true, severity: 'warning', message: 'Privileged port (requires admin rights)' };
        }

        // Check for well-known reserved ports
        if (options.reserved && options.reserved.includes(portNum)) {
            return { valid: false, severity: 'error', message: 'Port is reserved for system services' };
        }

        return { valid: true, severity: 'success', message: 'Valid port number' };
    }

    validateExternalHostname(hostname) {
        if (!hostname) {
            return { valid: false, severity: 'error', message: 'External hostname is required' };
        }

        if (hostname === 'SYSTEMIP') {
            return { valid: true, severity: 'info', message: 'Will use system IP address automatically' };
        }

        // Domain name validation
        const domainRegex = /^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$/;
        const ipRegex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;

        if (!domainRegex.test(hostname) && !ipRegex.test(hostname)) {
            return { valid: false, severity: 'error', message: 'Invalid hostname or IP address format' };
        }

        if (hostname === 'localhost' || hostname.startsWith('127.')) {
            return { valid: true, severity: 'warning', message: 'Localhost address - not accessible from external clients' };
        }

        if (hostname.startsWith('192.168.') || hostname.startsWith('10.') || hostname.startsWith('172.')) {
            return { valid: true, severity: 'warning', message: 'Private IP address - ensure proper network routing' };
        }

        return { valid: true, severity: 'success', message: 'Valid external hostname' };
    }

    // Security validation methods
    validateSSLCertificate(certContent) {
        if (!certContent) {
            return { valid: false, severity: 'error', message: 'SSL certificate is required for secure deployments' };
        }

        // Check certificate format
        if (!certContent.includes('-----BEGIN CERTIFICATE-----')) {
            return { valid: false, severity: 'error', message: 'Invalid certificate format - must be PEM encoded' };
        }

        if (!certContent.includes('-----END CERTIFICATE-----')) {
            return { valid: false, severity: 'error', message: 'Invalid certificate format - missing end marker' };
        }

        // Basic certificate validation
        try {
            const certInfo = this.parseCertificateInfo(certContent);
            const now = new Date();

            if (certInfo.notBefore > now) {
                return { valid: false, severity: 'error', message: 'Certificate is not yet valid' };
            }

            if (certInfo.notAfter < now) {
                return { valid: false, severity: 'error', message: 'Certificate has expired' };
            }

            const daysUntilExpiry = Math.floor((certInfo.notAfter - now) / (1000 * 60 * 60 * 24));
            if (daysUntilExpiry < 30) {
                return { valid: true, severity: 'warning', message: `Certificate expires in ${daysUntilExpiry} days` };
            }

            return { valid: true, severity: 'success', message: 'Valid SSL certificate' };

        } catch (error) {
            return { valid: false, severity: 'error', message: 'Could not parse certificate' };
        }
    }

    validateAdminPassword(password) {
        if (!password) {
            return { valid: false, severity: 'error', message: 'Admin password is required' };
        }

        const strength = this.calculatePasswordStrength(password);
        
        if (strength.score < 3) {
            return { valid: false, severity: 'error', message: `Weak password: ${strength.feedback}` };
        }

        if (strength.score < 4) {
            return { valid: true, severity: 'warning', message: `Moderate password: ${strength.feedback}` };
        }

        return { valid: true, severity: 'success', message: 'Strong password' };
    }

    calculatePasswordStrength(password) {
        let score = 0;
        const feedback = [];

        // Length check
        if (password.length >= 8) score++;
        else feedback.push('Use at least 8 characters');

        if (password.length >= 12) score++;

        // Character variety
        if (/[a-z]/.test(password)) score++;
        else feedback.push('Include lowercase letters');

        if (/[A-Z]/.test(password)) score++;
        else feedback.push('Include uppercase letters');

        if (/[0-9]/.test(password)) score++;
        else feedback.push('Include numbers');

        if (/[^a-zA-Z0-9]/.test(password)) score++;
        else feedback.push('Include special characters');

        // Common patterns
        if (/(.)\1{2,}/.test(password)) {
            score--;
            feedback.push('Avoid repeated characters');
        }

        const common = ['password', '123456', 'qwerty', 'admin', 'opensim'];
        if (common.some(word => password.toLowerCase().includes(word))) {
            score--;
            feedback.push('Avoid common words');
        }

        return {
            score: Math.max(0, Math.min(5, score)),
            feedback: feedback.length > 0 ? feedback.join(', ') : 'Good password'
        };
    }

    // Physics engine validation
    validatePhysicsEngine(engine) {
        const supportedEngines = {
            'ODE': {
                description: 'Open Dynamics Engine - Traditional OpenSim physics',
                maxBodies: 10000,
                features: ['Basic collision', 'Avatar physics', 'Stable dynamics'],
                requirements: ['ODE library']
            },
            'UBODE': {
                description: 'Enhanced ODE - Improved performance',
                maxBodies: 20000,
                features: ['Enhanced collision', 'Better performance', 'Large worlds'],
                requirements: ['UBODE library', 'Enhanced CPU']
            },
            'Bullet': {
                description: 'Bullet Physics - Advanced dynamics',
                maxBodies: 50000,
                features: ['Soft bodies', 'Vehicle physics', 'Advanced collision'],
                requirements: ['Bullet library', 'High-end CPU']
            },
            'POS': {
                description: 'Position-based Dynamics - Particle simulation',
                maxBodies: 100000,
                features: ['Particle systems', 'Fluid dynamics', 'GPU acceleration'],
                requirements: ['Modern GPU', 'CUDA/OpenCL support']
            },
            'Basic': {
                description: 'Basic Physics - Lightweight simulation',
                maxBodies: 1000,
                features: ['Simple collision', 'Low resource usage'],
                requirements: ['Minimal']
            }
        };

        if (!engine) {
            return { valid: false, severity: 'error', message: 'Physics engine selection is required' };
        }

        const engineInfo = supportedEngines[engine];
        if (!engineInfo) {
            return { valid: false, severity: 'error', message: 'Unsupported physics engine' };
        }

        return {
            valid: true,
            severity: 'success',
            message: `${engineInfo.description}`,
            details: {
                features: engineInfo.features,
                maxBodies: engineInfo.maxBodies,
                requirements: engineInfo.requirements
            }
        };
    }

    // Region validation methods
    validateRegionName(name) {
        if (!name) {
            return { valid: false, severity: 'error', message: 'Region name is required' };
        }

        if (name.length < 3) {
            return { valid: false, severity: 'error', message: 'Region name must be at least 3 characters' };
        }

        if (name.length > 32) {
            return { valid: false, severity: 'error', message: 'Region name must be 32 characters or less' };
        }

        // Check for valid characters
        const nameRegex = /^[a-zA-Z0-9\s\-_]+$/;
        if (!nameRegex.test(name)) {
            return { valid: false, severity: 'error', message: 'Region name contains invalid characters' };
        }

        // Check for reserved names
        const reserved = ['Welcome Area', 'Sandbox', 'Help Island', 'Orientation Island'];
        if (reserved.some(res => name.toLowerCase() === res.toLowerCase())) {
            return { valid: true, severity: 'warning', message: 'Region name may conflict with standard regions' };
        }

        return { valid: true, severity: 'success', message: 'Valid region name' };
    }

    validateRegionLocation(location) {
        if (!location) {
            return { valid: false, severity: 'error', message: 'Region location is required' };
        }

        // Parse coordinates (e.g., "1000,1000")
        const coords = location.split(',').map(c => parseInt(c.trim()));
        
        if (coords.length !== 2 || coords.some(isNaN)) {
            return { valid: false, severity: 'error', message: 'Location must be in format "X,Y" (e.g., "1000,1000")' };
        }

        const [x, y] = coords;

        // Validate coordinate ranges
        if (x < 0 || x > 100000 || y < 0 || y > 100000) {
            return { valid: false, severity: 'error', message: 'Coordinates must be between 0 and 100000' };
        }

        // Check for common starting locations
        if (x === 1000 && y === 1000) {
            return { valid: true, severity: 'info', message: 'Standard starting location (1000,1000)' };
        }

        return { valid: true, severity: 'success', message: 'Valid region coordinates' };
    }

    // Comprehensive validation orchestration
    async validateConfiguration(configuration) {
        const validationResult = {
            valid: true,
            timestamp: new Date().toISOString(),
            deploymentType: configuration.deploymentType,
            validationId: this.generateValidationId(),
            summary: {
                critical: { passed: 0, failed: 0 },
                high: { passed: 0, failed: 0 },
                medium: { passed: 0, failed: 0 },
                low: { passed: 0, failed: 0 }
            },
            results: [],
            recommendations: [],
            optimizations: [],
            errors: [],
            warnings: []
        };

        try {
            // Run all validation rules
            const validationPromises = Array.from(this.validationRules.entries()).map(
                async ([ruleName, rule]) => {
                    try {
                        const result = await this.runValidationRule(ruleName, configuration);
                        return { ruleName, result, rule };
                    } catch (error) {
                        return {
                            ruleName,
                            result: {
                                valid: false,
                                severity: 'error',
                                message: `Validation rule failed: ${error.message}`,
                                error: error
                            },
                            rule
                        };
                    }
                }
            );

            const validationResults = await Promise.all(validationPromises);

            // Process results
            for (const { ruleName, result, rule } of validationResults) {
                validationResult.results.push({
                    rule: ruleName,
                    priority: rule.priority,
                    ...result
                });

                // Update summary
                const priority = rule.priority;
                if (result.valid) {
                    validationResult.summary[priority].passed++;
                } else {
                    validationResult.summary[priority].failed++;
                    validationResult.valid = false;

                    if (priority === 'critical') {
                        validationResult.errors.push({
                            rule: ruleName,
                            message: result.message
                        });
                    } else if (priority === 'high') {
                        validationResult.warnings.push({
                            rule: ruleName,
                            message: result.message
                        });
                    }
                }

                // Collect recommendations
                if (result.recommendations) {
                    validationResult.recommendations.push(...result.recommendations);
                }

                // Collect optimizations
                if (result.optimizations) {
                    validationResult.optimizations.push(...result.optimizations);
                }
            }

            // Generate deployment-specific recommendations
            const deploymentRecommendations = await this.generateDeploymentRecommendations(
                configuration, validationResult
            );
            validationResult.recommendations.push(...deploymentRecommendations);

            // Store validation history
            this.validationHistory.push(validationResult);

            // Update UI
            this.updateValidationUI(validationResult);

            return validationResult;

        } catch (error) {
            console.error('Configuration validation failed:', error);
            
            validationResult.valid = false;
            validationResult.errors.push({
                rule: 'system',
                message: `Validation system error: ${error.message}`
            });

            return validationResult;
        }
    }

    async runValidationRule(ruleName, configuration = null) {
        const rule = this.validationRules.get(ruleName);
        if (!rule) {
            throw new Error(`Unknown validation rule: ${ruleName}`);
        }

        const startTime = performance.now();
        
        try {
            let result;
            if (rule.async) {
                result = await rule.validator(configuration);
            } else {
                result = rule.validator(configuration);
            }

            // Ensure result has required structure
            result = {
                valid: true,
                severity: 'success',
                message: 'Validation passed',
                ...result,
                executionTime: performance.now() - startTime
            };

            // Update rule history
            rule.lastRun = new Date().toISOString();
            rule.results.push(result);

            // Keep only last 10 results
            if (rule.results.length > 10) {
                rule.results = rule.results.slice(-10);
            }

            return result;

        } catch (error) {
            console.error(`Validation rule ${ruleName} failed:`, error);
            
            const result = {
                valid: false,
                severity: 'error',
                message: `Rule execution failed: ${error.message}`,
                error: error.message,
                executionTime: performance.now() - startTime
            };

            rule.lastRun = new Date().toISOString();
            rule.results.push(result);

            return result;
        }
    }

    generateValidationId() {
        return `validation-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    }

    updateFieldValidation(field, validation) {
        // Remove existing validation indicators
        const existingIndicator = field.parentElement.querySelector('.field-validation');
        if (existingIndicator) {
            existingIndicator.remove();
        }

        // Create validation indicator
        const indicator = document.createElement('div');
        indicator.className = `field-validation ${validation.severity}`;
        indicator.innerHTML = `
            <div class="validation-content">
                <i class="icon-${this.getSeverityIcon(validation.severity)}"></i>
                <span class="validation-message">${validation.message}</span>
                ${validation.details ? `<div class="validation-details">${this.formatValidationDetails(validation.details)}</div>` : ''}
            </div>
        `;

        // Insert after field
        field.parentElement.appendChild(indicator);

        // Update field styling
        field.classList.remove('valid', 'invalid', 'warning');
        if (validation.valid) {
            field.classList.add(validation.severity === 'warning' ? 'warning' : 'valid');
        } else {
            field.classList.add('invalid');
        }
    }

    getSeverityIcon(severity) {
        const icons = {
            'success': 'check-circle',
            'info': 'info-circle',
            'warning': 'warning-triangle',
            'error': 'x-circle'
        };
        return icons[severity] || 'help-circle';
    }

    formatValidationDetails(details) {
        if (typeof details === 'string') {
            return details;
        }

        if (Array.isArray(details)) {
            return details.map(item => `<div>• ${item}</div>`).join('');
        }

        if (typeof details === 'object') {
            return Object.entries(details)
                .map(([key, value]) => `<div><strong>${key}:</strong> ${value}</div>`)
                .join('');
        }

        return JSON.stringify(details);
    }

    updateValidationStatus() {
        const statusElement = document.getElementById('validation-status');
        if (!statusElement) return;

        // Collect all field validations
        const validationElements = document.querySelectorAll('.field-validation');
        const validations = Array.from(validationElements).map(el => ({
            severity: el.classList.contains('error') ? 'error' :
                     el.classList.contains('warning') ? 'warning' :
                     el.classList.contains('info') ? 'info' : 'success'
        }));

        const summary = validations.reduce((acc, v) => {
            acc[v.severity] = (acc[v.severity] || 0) + 1;
            return acc;
        }, {});

        const hasErrors = summary.error > 0;
        const hasWarnings = summary.warning > 0;

        let status, icon, message;
        if (hasErrors) {
            status = 'invalid';
            icon = 'x-circle';
            message = `${summary.error} error${summary.error > 1 ? 's' : ''} found`;
        } else if (hasWarnings) {
            status = 'warning';
            icon = 'warning-triangle';
            message = `${summary.warning} warning${summary.warning > 1 ? 's' : ''} found`;
        } else if (validations.length > 0) {
            status = 'valid';
            icon = 'check-circle';
            message = 'Configuration is valid';
        } else {
            status = 'pending';
            icon = 'clock';
            message = 'Validation pending';
        }

        statusElement.innerHTML = `
            <div class="validation-status ${status}">
                <i class="icon-${icon}"></i>
                <span>${message}</span>
                <div class="validation-summary">
                    ${summary.error ? `<span class="error">${summary.error} errors</span>` : ''}
                    ${summary.warning ? `<span class="warning">${summary.warning} warnings</span>` : ''}
                    ${summary.success ? `<span class="success">${summary.success} valid</span>` : ''}
                </div>
            </div>
        `;
    }

    // Utility methods for async validations
    async checkHostReachability(host) {
        // This would typically use a backend service
        // For demo purposes, we'll simulate the check
        return new Promise((resolve) => {
            setTimeout(() => {
                // Simulate 90% reachability success rate
                resolve(Math.random() > 0.1);
            }, 1000);
        });
    }

    parseCertificateInfo(certContent) {
        // Basic certificate parsing simulation
        // In production, this would use proper certificate parsing libraries
        const lines = certContent.split('\n');
        const certData = lines.slice(1, -2).join('');
        
        try {
            // Simulate certificate info extraction
            const now = new Date();
            return {
                notBefore: new Date(now.getTime() - (30 * 24 * 60 * 60 * 1000)), // 30 days ago
                notAfter: new Date(now.getTime() + (365 * 24 * 60 * 60 * 1000)), // 1 year from now
                subject: 'CN=opensim-next.local',
                issuer: 'CN=Self-Signed'
            };
        } catch (error) {
            throw new Error('Invalid certificate format');
        }
    }

    getDependentValidations(fieldName) {
        // Define field dependencies
        const dependencies = {
            'database_type': ['database_connectivity', 'resource_requirements'],
            'deployment_type': ['security_requirements', 'deployment_optimization'],
            'physics_engine': ['physics_compatibility', 'resource_requirements'],
            'ssl_enabled': ['security_requirements'],
            'region_count': ['resource_requirements', 'deployment_optimization']
        };

        return dependencies[fieldName] || [];
    }

    async generateDeploymentRecommendations(configuration, validationResult) {
        const recommendations = [];

        // Deployment-specific recommendations
        switch (configuration.deploymentType) {
            case 'development':
                recommendations.push({
                    type: 'optimization',
                    priority: 'medium',
                    title: 'Development Optimization',
                    message: 'Consider using SQLite database and Basic physics engine for faster iteration'
                });
                break;

            case 'production':
                recommendations.push({
                    type: 'security',
                    priority: 'high',
                    title: 'Production Security',
                    message: 'Ensure SSL/TLS is enabled and use PostgreSQL database for better performance'
                });
                break;

            case 'grid':
                recommendations.push({
                    type: 'scaling',
                    priority: 'high',
                    title: 'Grid Scaling',
                    message: 'Consider load balancing and distributed database configuration'
                });
                break;
        }

        return recommendations;
    }

    updateValidationUI(validationResult) {
        // Update main validation display
        const validationContainer = document.getElementById('validation-results');
        if (validationContainer) {
            validationContainer.innerHTML = this.renderValidationResults(validationResult);
        }

        // Update progress indicators
        this.updateValidationProgress(validationResult);
    }

    renderValidationResults(result) {
        const { summary, results, recommendations, errors, warnings } = result;
        
        return `
            <div class="validation-results">
                <div class="validation-header ${result.valid ? 'valid' : 'invalid'}">
                    <i class="icon-${result.valid ? 'check-circle' : 'x-circle'}"></i>
                    <h3>Configuration Validation ${result.valid ? 'Passed' : 'Failed'}</h3>
                    <div class="validation-timestamp">${new Date(result.timestamp).toLocaleString()}</div>
                </div>

                <div class="validation-summary">
                    <div class="summary-grid">
                        <div class="summary-item critical">
                            <span class="label">Critical</span>
                            <span class="count">${summary.critical.passed}/${summary.critical.passed + summary.critical.failed}</span>
                        </div>
                        <div class="summary-item high">
                            <span class="label">High</span>
                            <span class="count">${summary.high.passed}/${summary.high.passed + summary.high.failed}</span>
                        </div>
                        <div class="summary-item medium">
                            <span class="label">Medium</span>
                            <span class="count">${summary.medium.passed}/${summary.medium.passed + summary.medium.failed}</span>
                        </div>
                        <div class="summary-item low">
                            <span class="label">Low</span>
                            <span class="count">${summary.low.passed}/${summary.low.passed + summary.low.failed}</span>
                        </div>
                    </div>
                </div>

                ${errors.length > 0 ? this.renderValidationErrors(errors) : ''}
                ${warnings.length > 0 ? this.renderValidationWarnings(warnings) : ''}
                ${recommendations.length > 0 ? this.renderValidationRecommendations(recommendations) : ''}
                
                <div class="validation-details">
                    <h4>Detailed Results</h4>
                    ${results.map(r => this.renderValidationDetail(r)).join('')}
                </div>
            </div>
        `;
    }

    renderValidationErrors(errors) {
        return `
            <div class="validation-errors">
                <h4><i class="icon-x-circle"></i> Critical Issues</h4>
                ${errors.map(error => `
                    <div class="validation-error">
                        <strong>${error.rule}:</strong> ${error.message}
                    </div>
                `).join('')}
            </div>
        `;
    }

    renderValidationWarnings(warnings) {
        return `
            <div class="validation-warnings">
                <h4><i class="icon-warning-triangle"></i> Warnings</h4>
                ${warnings.map(warning => `
                    <div class="validation-warning">
                        <strong>${warning.rule}:</strong> ${warning.message}
                    </div>
                `).join('')}
            </div>
        `;
    }

    renderValidationRecommendations(recommendations) {
        return `
            <div class="validation-recommendations">
                <h4><i class="icon-lightbulb"></i> Recommendations</h4>
                ${recommendations.map(rec => `
                    <div class="validation-recommendation ${rec.priority}">
                        <div class="recommendation-header">
                            <strong>${rec.title}</strong>
                            <span class="priority">${rec.priority}</span>
                        </div>
                        <div class="recommendation-message">${rec.message}</div>
                    </div>
                `).join('')}
            </div>
        `;
    }

    renderValidationDetail(result) {
        return `
            <div class="validation-detail ${result.valid ? 'valid' : 'invalid'} ${result.priority}">
                <div class="detail-header">
                    <i class="icon-${this.getSeverityIcon(result.severity)}"></i>
                    <span class="rule-name">${result.rule}</span>
                    <span class="priority-badge ${result.priority}">${result.priority}</span>
                    ${result.executionTime ? `<span class="execution-time">${Math.round(result.executionTime)}ms</span>` : ''}
                </div>
                <div class="detail-message">${result.message}</div>
                ${result.details ? `<div class="detail-extra">${this.formatValidationDetails(result.details)}</div>` : ''}
            </div>
        `;
    }

    updateValidationProgress(result) {
        const progressElement = document.getElementById('validation-progress');
        if (progressElement) {
            const total = Object.values(result.summary).reduce((sum, s) => sum + s.passed + s.failed, 0);
            const passed = Object.values(result.summary).reduce((sum, s) => sum + s.passed, 0);
            const percentage = total > 0 ? Math.round((passed / total) * 100) : 0;

            progressElement.innerHTML = `
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${percentage}%"></div>
                </div>
                <div class="progress-text">${passed}/${total} validation rules passed (${percentage}%)</div>
            `;
        }
    }

    // Export validation results
    exportValidationReport(validationResult, format = 'json') {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `opensim-validation-${timestamp}.${format}`;

        let content, mimeType;

        switch (format) {
            case 'json':
                content = JSON.stringify(validationResult, null, 2);
                mimeType = 'application/json';
                break;

            case 'html':
                content = this.generateHTMLReport(validationResult);
                mimeType = 'text/html';
                break;

            case 'csv':
                content = this.generateCSVReport(validationResult);
                mimeType = 'text/csv';
                break;

            default:
                throw new Error(`Unsupported export format: ${format}`);
        }

        // Create download
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    generateHTMLReport(result) {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <title>OpenSim Next Configuration Validation Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; }
                    .header { background: #f5f5f5; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
                    .summary { display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; margin: 20px 0; }
                    .summary-item { background: white; border: 1px solid #ddd; padding: 15px; border-radius: 6px; text-align: center; }
                    .valid { color: #28a745; }
                    .invalid { color: #dc3545; }
                    .warning { color: #ffc107; }
                    .detail { margin: 10px 0; padding: 10px; border-left: 4px solid #ccc; }
                    .critical { border-left-color: #dc3545; }
                    .high { border-left-color: #fd7e14; }
                    .medium { border-left-color: #ffc107; }
                    .low { border-left-color: #28a745; }
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>OpenSim Next Configuration Validation Report</h1>
                    <p><strong>Deployment Type:</strong> ${result.deploymentType}</p>
                    <p><strong>Validation Time:</strong> ${new Date(result.timestamp).toLocaleString()}</p>
                    <p><strong>Overall Status:</strong> <span class="${result.valid ? 'valid' : 'invalid'}">${result.valid ? 'PASSED' : 'FAILED'}</span></p>
                </div>

                <div class="summary">
                    <div class="summary-item">
                        <h3>Critical</h3>
                        <p>${result.summary.critical.passed}/${result.summary.critical.passed + result.summary.critical.failed}</p>
                    </div>
                    <div class="summary-item">
                        <h3>High</h3>
                        <p>${result.summary.high.passed}/${result.summary.high.passed + result.summary.high.failed}</p>
                    </div>
                    <div class="summary-item">
                        <h3>Medium</h3>
                        <p>${result.summary.medium.passed}/${result.summary.medium.passed + result.summary.medium.failed}</p>
                    </div>
                    <div class="summary-item">
                        <h3>Low</h3>
                        <p>${result.summary.low.passed}/${result.summary.low.passed + result.summary.low.failed}</p>
                    </div>
                </div>

                <h2>Detailed Results</h2>
                ${result.results.map(r => `
                    <div class="detail ${r.priority} ${r.valid ? 'valid' : 'invalid'}">
                        <h4>${r.rule} (${r.priority})</h4>
                        <p>${r.message}</p>
                        ${r.executionTime ? `<small>Execution time: ${Math.round(r.executionTime)}ms</small>` : ''}
                    </div>
                `).join('')}

                ${result.recommendations.length > 0 ? `
                    <h2>Recommendations</h2>
                    ${result.recommendations.map(rec => `
                        <div class="detail ${rec.priority}">
                            <h4>${rec.title}</h4>
                            <p>${rec.message}</p>
                        </div>
                    `).join('')}
                ` : ''}
            </body>
            </html>
        `;
    }

    generateCSVReport(result) {
        const headers = ['Rule', 'Priority', 'Status', 'Message', 'Execution Time'];
        const rows = result.results.map(r => [
            r.rule,
            r.priority,
            r.valid ? 'PASSED' : 'FAILED',
            r.message.replace(/"/g, '""'),
            r.executionTime ? Math.round(r.executionTime) : ''
        ]);

        const csvContent = [
            headers.join(','),
            ...rows.map(row => row.map(cell => `"${cell}"`).join(','))
        ].join('\n');

        return csvContent;
    }

    // Cleanup method
    destroy() {
        if (this.validationTimer) {
            clearTimeout(this.validationTimer);
        }
        
        // Remove event listeners
        document.removeEventListener('input', this.scheduleValidation);
        document.removeEventListener('change', this.scheduleValidation);
        
        console.log('Configuration validator destroyed');
    }
}

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.ConfigurationValidator = ConfigurationValidator;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { ConfigurationValidator };
}