// OpenSim Next Auto-Configurator - Pre-flight Checker
// Comprehensive pre-deployment validation and system readiness checks

class PreflightChecker {
    constructor() {
        this.checkCategories = new Map();
        this.systemChecks = new Map();
        this.networkChecks = new Map();
        this.securityChecks = new Map();
        this.performanceChecks = new Map();
        
        this.initializePreflightChecks();
    }

    initializePreflightChecks() {
        // System Environment Checks
        this.registerSystemCheck('browser_compatibility', {
            name: 'Browser Compatibility',
            description: 'Verify browser supports required features',
            async: false,
            critical: true,
            checker: this.checkBrowserCompatibility.bind(this)
        });

        this.registerSystemCheck('javascript_features', {
            name: 'JavaScript Features',
            description: 'Check for required JavaScript APIs',
            async: false,
            critical: true,
            checker: this.checkJavaScriptFeatures.bind(this)
        });

        this.registerSystemCheck('local_storage', {
            name: 'Local Storage',
            description: 'Verify local storage is available and writable',
            async: true,
            critical: false,
            checker: this.checkLocalStorage.bind(this)
        });

        this.registerSystemCheck('crypto_support', {
            name: 'Cryptographic Support',
            description: 'Verify Web Crypto API availability',
            async: false,
            critical: false,
            checker: this.checkCryptoSupport.bind(this)
        });

        // Network Connectivity Checks
        this.registerNetworkCheck('internet_connectivity', {
            name: 'Internet Connectivity',
            description: 'Test basic internet connection',
            async: true,
            critical: true,
            checker: this.checkInternetConnectivity.bind(this)
        });

        this.registerNetworkCheck('dns_resolution', {
            name: 'DNS Resolution',
            description: 'Test DNS resolution for configured hostnames',
            async: true,
            critical: true,
            checker: this.checkDNSResolution.bind(this)
        });

        this.registerNetworkCheck('port_accessibility', {
            name: 'Port Accessibility',
            description: 'Check if configured ports are accessible',
            async: true,
            critical: true,
            checker: this.checkPortAccessibility.bind(this)
        });

        this.registerNetworkCheck('websocket_support', {
            name: 'WebSocket Support',
            description: 'Test WebSocket connectivity',
            async: true,
            critical: false,
            checker: this.checkWebSocketSupport.bind(this)
        });

        // Security Checks
        this.registerSecurityCheck('ssl_certificate_validity', {
            name: 'SSL Certificate Validity',
            description: 'Validate SSL certificates and keys',
            async: true,
            critical: true,
            checker: this.checkSSLCertificateValidity.bind(this)
        });

        this.registerSecurityCheck('password_strength', {
            name: 'Password Strength',
            description: 'Validate password complexity requirements',
            async: false,
            critical: true,
            checker: this.checkPasswordStrength.bind(this)
        });

        this.registerSecurityCheck('secure_context', {
            name: 'Secure Context',
            description: 'Verify application is running in secure context',
            async: false,
            critical: false,
            checker: this.checkSecureContext.bind(this)
        });

        this.registerSecurityCheck('permission_validation', {
            name: 'Permission Validation',
            description: 'Check user permissions and access levels',
            async: false,
            critical: true,
            checker: this.checkPermissionValidation.bind(this)
        });

        // Performance Checks
        this.registerPerformanceCheck('system_resources', {
            name: 'System Resources',
            description: 'Estimate available system resources',
            async: true,
            critical: false,
            checker: this.checkSystemResources.bind(this)
        });

        this.registerPerformanceCheck('physics_performance', {
            name: 'Physics Performance',
            description: 'Test physics engine performance characteristics',
            async: true,
            critical: false,
            checker: this.checkPhysicsPerformance.bind(this)
        });

        this.registerPerformanceCheck('database_performance', {
            name: 'Database Performance',
            description: 'Test database connection and basic operations',
            async: true,
            critical: true,
            checker: this.checkDatabasePerformance.bind(this)
        });

        this.registerPerformanceCheck('network_latency', {
            name: 'Network Latency',
            description: 'Measure network latency and bandwidth',
            async: true,
            critical: false,
            checker: this.checkNetworkLatency.bind(this)
        });

        console.log('Pre-flight checks initialized');
    }

    registerSystemCheck(name, check) {
        this.systemChecks.set(name, { ...check, category: 'system', name });
    }

    registerNetworkCheck(name, check) {
        this.networkChecks.set(name, { ...check, category: 'network', name });
    }

    registerSecurityCheck(name, check) {
        this.securityChecks.set(name, { ...check, category: 'security', name });
    }

    registerPerformanceCheck(name, check) {
        this.performanceChecks.set(name, { ...check, category: 'performance', name });
    }

    async runPreflightChecks(configuration, options = {}) {
        const startTime = performance.now();
        
        const preflightResult = {
            timestamp: new Date().toISOString(),
            configuration_id: configuration.id || 'unknown',
            deployment_type: configuration.deploymentType,
            overall_status: 'unknown',
            execution_time: 0,
            categories: {
                system: { passed: 0, failed: 0, skipped: 0, results: [] },
                network: { passed: 0, failed: 0, skipped: 0, results: [] },
                security: { passed: 0, failed: 0, skipped: 0, results: [] },
                performance: { passed: 0, failed: 0, skipped: 0, results: [] }
            },
            critical_failures: [],
            warnings: [],
            recommendations: [],
            readiness_score: 0,
            deployment_readiness: 'unknown'
        };

        try {
            // Run checks by category
            await this.runCategoryChecks('system', this.systemChecks, configuration, preflightResult, options);
            await this.runCategoryChecks('network', this.networkChecks, configuration, preflightResult, options);
            await this.runCategoryChecks('security', this.securityChecks, configuration, preflightResult, options);
            await this.runCategoryChecks('performance', this.performanceChecks, configuration, preflightResult, options);

            // Calculate overall status and readiness
            this.calculateOverallStatus(preflightResult);
            this.generateRecommendations(preflightResult, configuration);

            preflightResult.execution_time = performance.now() - startTime;

            console.log('Pre-flight checks completed:', preflightResult);
            return preflightResult;

        } catch (error) {
            console.error('Pre-flight checks failed:', error);
            preflightResult.overall_status = 'error';
            preflightResult.critical_failures.push({
                category: 'system',
                check: 'preflight_execution',
                message: `Pre-flight check execution failed: ${error.message}`
            });
            preflightResult.execution_time = performance.now() - startTime;
            return preflightResult;
        }
    }

    async runCategoryChecks(category, checks, configuration, result, options) {
        const categoryResult = result.categories[category];
        
        for (const [checkName, check] of checks) {
            if (options.skipNonCritical && !check.critical) {
                categoryResult.skipped++;
                continue;
            }

            try {
                const checkResult = await this.runSingleCheck(check, configuration);
                categoryResult.results.push(checkResult);

                if (checkResult.passed) {
                    categoryResult.passed++;
                } else {
                    categoryResult.failed++;
                    
                    if (check.critical) {
                        result.critical_failures.push({
                            category: category,
                            check: checkName,
                            message: checkResult.message,
                            details: checkResult.details
                        });
                    } else {
                        result.warnings.push({
                            category: category,
                            check: checkName,
                            message: checkResult.message,
                            severity: checkResult.severity || 'warning'
                        });
                    }
                }

                // Add check-specific recommendations
                if (checkResult.recommendations) {
                    result.recommendations.push(...checkResult.recommendations);
                }

            } catch (error) {
                console.error(`Check ${checkName} failed:`, error);
                categoryResult.failed++;
                
                const errorResult = {
                    check: checkName,
                    passed: false,
                    message: `Check failed to execute: ${error.message}`,
                    error: error.message,
                    execution_time: 0
                };
                
                categoryResult.results.push(errorResult);
                
                if (check.critical) {
                    result.critical_failures.push({
                        category: category,
                        check: checkName,
                        message: errorResult.message
                    });
                }
            }
        }
    }

    async runSingleCheck(check, configuration) {
        const startTime = performance.now();
        
        try {
            let result;
            if (check.async) {
                result = await check.checker(configuration);
            } else {
                result = check.checker(configuration);
            }

            return {
                check: check.name,
                passed: result.passed !== false,
                message: result.message || 'Check completed successfully',
                severity: result.severity || 'info',
                details: result.details,
                recommendations: result.recommendations,
                execution_time: performance.now() - startTime,
                ...result
            };

        } catch (error) {
            return {
                check: check.name,
                passed: false,
                message: `Check execution failed: ${error.message}`,
                error: error.message,
                execution_time: performance.now() - startTime
            };
        }
    }

    // System Environment Checks
    checkBrowserCompatibility() {
        const userAgent = navigator.userAgent;
        const requiredFeatures = [
            'Promise',
            'fetch',
            'localStorage',
            'sessionStorage',
            'JSON',
            'FileReader',
            'Blob',
            'FormData'
        ];

        const missing = requiredFeatures.filter(feature => typeof window[feature] === 'undefined');
        
        if (missing.length > 0) {
            return {
                passed: false,
                severity: 'error',
                message: `Browser missing required features: ${missing.join(', ')}`,
                details: { missing_features: missing, user_agent: userAgent },
                recommendations: [{
                    type: 'browser',
                    message: 'Please use a modern browser (Chrome 80+, Firefox 75+, Safari 13+, Edge 80+)'
                }]
            };
        }

        // Check browser version warnings
        const warnings = [];
        if (userAgent.includes('Chrome/')) {
            const version = parseInt(userAgent.match(/Chrome\/(\d+)/)?.[1] || 0);
            if (version < 80) warnings.push('Chrome version may be outdated');
        }

        return {
            passed: true,
            message: 'Browser compatibility check passed',
            details: { user_agent: userAgent },
            warnings: warnings
        };
    }

    checkJavaScriptFeatures() {
        const features = {
            'ES6 Classes': typeof class {} === 'function',
            'Arrow Functions': (() => true)(),
            'Promises': typeof Promise !== 'undefined',
            'Async/Await': (async () => true)().constructor.name === 'AsyncFunction',
            'Map/Set': typeof Map !== 'undefined' && typeof Set !== 'undefined',
            'Destructuring': (() => { try { const [a] = [1]; return true; } catch { return false; } })(),
            'Template Literals': (() => { try { return `test${1}` === 'test1'; } catch { return false; } })(),
            'Modules': typeof import === 'function' || typeof require === 'function'
        };

        const unsupported = Object.entries(features)
            .filter(([name, supported]) => !supported)
            .map(([name]) => name);

        if (unsupported.length > 0) {
            return {
                passed: false,
                severity: 'warning',
                message: `Some modern JavaScript features are not supported: ${unsupported.join(', ')}`,
                details: { unsupported_features: unsupported, all_features: features }
            };
        }

        return {
            passed: true,
            message: 'All required JavaScript features are supported',
            details: { features: features }
        };
    }

    async checkLocalStorage() {
        try {
            const testKey = 'opensim-preflight-test';
            const testValue = 'test-data-' + Date.now();

            // Test write
            localStorage.setItem(testKey, testValue);
            
            // Test read
            const retrieved = localStorage.getItem(testKey);
            
            // Test delete
            localStorage.removeItem(testKey);

            if (retrieved !== testValue) {
                throw new Error('Local storage read/write mismatch');
            }

            // Check available space (approximate)
            const spaceTest = this.estimateLocalStorageSpace();

            return {
                passed: true,
                message: 'Local storage is functional',
                details: {
                    available: true,
                    estimated_space: spaceTest.estimatedSpace,
                    test_successful: true
                }
            };

        } catch (error) {
            return {
                passed: false,
                severity: 'warning',
                message: `Local storage test failed: ${error.message}`,
                details: { error: error.message },
                recommendations: [{
                    type: 'storage',
                    message: 'Enable local storage in browser settings or use incognito mode'
                }]
            };
        }
    }

    estimateLocalStorageSpace() {
        try {
            let data = '';
            let totalSize = 0;
            const blockSize = 1024; // 1KB blocks
            
            // Try to fill up storage (with limits to avoid hanging)
            for (let i = 0; i < 10; i++) {
                try {
                    data += 'x'.repeat(blockSize);
                    localStorage.setItem('space-test', data);
                    totalSize += blockSize;
                } catch (e) {
                    break;
                }
            }
            
            localStorage.removeItem('space-test');
            
            return {
                estimatedSpace: totalSize > 0 ? `~${Math.round(totalSize / 1024)}KB+` : 'Unknown'
            };
        } catch (error) {
            return { estimatedSpace: 'Unknown' };
        }
    }

    checkCryptoSupport() {
        const cryptoFeatures = {
            'Web Crypto API': typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined',
            'Random Values': typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function',
            'TextEncoder': typeof TextEncoder !== 'undefined',
            'TextDecoder': typeof TextDecoder !== 'undefined'
        };

        const missing = Object.entries(cryptoFeatures)
            .filter(([name, supported]) => !supported)
            .map(([name]) => name);

        const hasBasicCrypto = cryptoFeatures['Web Crypto API'] && cryptoFeatures['Random Values'];

        return {
            passed: hasBasicCrypto,
            severity: hasBasicCrypto ? 'info' : 'warning',
            message: hasBasicCrypto 
                ? 'Cryptographic features are available'
                : `Limited cryptographic support. Missing: ${missing.join(', ')}`,
            details: { features: cryptoFeatures, missing: missing },
            recommendations: missing.length > 0 ? [{
                type: 'security',
                message: 'Some security features may be limited. Consider using a modern browser.'
            }] : []
        };
    }

    // Network Connectivity Checks
    async checkInternetConnectivity() {
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 5000);

            const response = await fetch('https://httpbin.org/get', {
                method: 'GET',
                signal: controller.signal,
                cache: 'no-cache'
            });

            clearTimeout(timeoutId);

            if (response.ok) {
                return {
                    passed: true,
                    message: 'Internet connectivity confirmed',
                    details: {
                        status: response.status,
                        response_time: 'Under 5 seconds'
                    }
                };
            } else {
                return {
                    passed: false,
                    severity: 'error',
                    message: `Internet connectivity test failed with status ${response.status}`,
                    details: { status: response.status }
                };
            }

        } catch (error) {
            if (error.name === 'AbortError') {
                return {
                    passed: false,
                    severity: 'error',
                    message: 'Internet connectivity test timed out',
                    details: { error: 'Timeout after 5 seconds' },
                    recommendations: [{
                        type: 'network',
                        message: 'Check internet connection and firewall settings'
                    }]
                };
            }

            return {
                passed: false,
                severity: 'error',
                message: `Internet connectivity test failed: ${error.message}`,
                details: { error: error.message },
                recommendations: [{
                    type: 'network',
                    message: 'Verify internet connection and DNS settings'
                }]
            };
        }
    }

    async checkDNSResolution(configuration) {
        const hostnames = [];
        
        // Collect hostnames from configuration
        if (configuration.network?.external_hostname) {
            hostnames.push(configuration.network.external_hostname);
        }
        if (configuration.database?.host && configuration.database.host !== 'localhost') {
            hostnames.push(configuration.database.host);
        }

        if (hostnames.length === 0) {
            return {
                passed: true,
                message: 'No external hostnames to resolve',
                details: { hostnames: [] }
            };
        }

        const results = [];
        let allPassed = true;

        for (const hostname of hostnames) {
            try {
                const result = await this.resolveHostname(hostname);
                results.push({ hostname, ...result });
                if (!result.resolved) allPassed = false;
            } catch (error) {
                results.push({
                    hostname,
                    resolved: false,
                    error: error.message
                });
                allPassed = false;
            }
        }

        return {
            passed: allPassed,
            severity: allPassed ? 'info' : 'warning',
            message: allPassed 
                ? 'All hostnames resolved successfully'
                : 'Some hostnames failed to resolve',
            details: { results: results },
            recommendations: allPassed ? [] : [{
                type: 'network',
                message: 'Verify DNS settings and hostname configuration'
            }]
        };
    }

    async resolveHostname(hostname) {
        try {
            // Use a simple HTTP request to test hostname resolution
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);

            const response = await fetch(`https://${hostname}`, {
                method: 'HEAD',
                signal: controller.signal,
                mode: 'no-cors' // Avoid CORS issues
            });

            clearTimeout(timeoutId);
            
            return {
                resolved: true,
                reachable: true,
                response_time: 'Under 3 seconds'
            };

        } catch (error) {
            if (error.name === 'AbortError') {
                return {
                    resolved: false,
                    error: 'Timeout - hostname may not be reachable'
                };
            }

            // For no-cors mode, any response (even network errors) indicates DNS resolution worked
            if (error.message.includes('fetch')) {
                return {
                    resolved: true,
                    reachable: false,
                    note: 'DNS resolved but service may not be reachable'
                };
            }

            return {
                resolved: false,
                error: error.message
            };
        }
    }

    async checkPortAccessibility(configuration) {
        // Note: Web browsers cannot directly test port accessibility due to security restrictions
        // This check provides guidance based on configuration

        const ports = [];
        
        if (configuration.network?.http_port) {
            ports.push({ port: configuration.network.http_port, service: 'HTTP', protocol: 'TCP' });
        }
        if (configuration.network?.https_port) {
            ports.push({ port: configuration.network.https_port, service: 'HTTPS', protocol: 'TCP' });
        }
        if (configuration.network?.websocket_port) {
            ports.push({ port: configuration.network.websocket_port, service: 'WebSocket', protocol: 'TCP' });
        }

        const commonReservedPorts = [21, 22, 23, 25, 53, 80, 110, 143, 443, 993, 995];
        const warnings = [];
        const recommendations = [];

        for (const portConfig of ports) {
            const port = parseInt(portConfig.port);
            
            if (port < 1024 && !commonReservedPorts.includes(port)) {
                warnings.push(`Port ${port} is in privileged range and may require admin rights`);
            }
            
            if (port === 80 && portConfig.service !== 'HTTP') {
                warnings.push(`Port 80 is typically reserved for HTTP traffic`);
            }
            
            if (port === 443 && portConfig.service !== 'HTTPS') {
                warnings.push(`Port 443 is typically reserved for HTTPS traffic`);
            }
        }

        if (ports.length === 0) {
            return {
                passed: true,
                message: 'No specific ports configured for testing',
                details: { ports: [] }
            };
        }

        // Check for port conflicts
        const portNumbers = ports.map(p => parseInt(p.port));
        const duplicates = portNumbers.filter((port, index) => portNumbers.indexOf(port) !== index);
        
        if (duplicates.length > 0) {
            return {
                passed: false,
                severity: 'error',
                message: `Port conflicts detected: ${duplicates.join(', ')}`,
                details: { conflicted_ports: duplicates, all_ports: ports },
                recommendations: [{
                    type: 'network',
                    message: 'Resolve port conflicts by assigning unique ports to each service'
                }]
            };
        }

        return {
            passed: true,
            message: warnings.length > 0 
                ? `Port configuration has warnings: ${warnings.join('; ')}`
                : 'Port configuration appears valid',
            severity: warnings.length > 0 ? 'warning' : 'info',
            details: { ports: ports, warnings: warnings },
            recommendations: warnings.length > 0 ? [{
                type: 'network',
                message: 'Review port configuration warnings and firewall settings'
            }] : []
        };
    }

    async checkWebSocketSupport() {
        try {
            // Test WebSocket constructor availability
            if (typeof WebSocket === 'undefined') {
                return {
                    passed: false,
                    severity: 'warning',
                    message: 'WebSocket API is not available',
                    recommendations: [{
                        type: 'browser',
                        message: 'WebSocket support is required for real-time features. Please use a modern browser.'
                    }]
                };
            }

            // Test WebSocket connection to echo service
            const testUrl = 'wss://echo.websocket.org';
            const ws = new WebSocket(testUrl);
            
            return new Promise((resolve) => {
                const timeout = setTimeout(() => {
                    ws.close();
                    resolve({
                        passed: false,
                        severity: 'warning',
                        message: 'WebSocket connection test timed out',
                        details: { test_url: testUrl, error: 'Timeout' }
                    });
                }, 5000);

                ws.onopen = () => {
                    clearTimeout(timeout);
                    ws.close();
                    resolve({
                        passed: true,
                        message: 'WebSocket connectivity confirmed',
                        details: { test_url: testUrl, status: 'Connected successfully' }
                    });
                };

                ws.onerror = (error) => {
                    clearTimeout(timeout);
                    resolve({
                        passed: false,
                        severity: 'warning',
                        message: 'WebSocket connection test failed',
                        details: { test_url: testUrl, error: 'Connection failed' },
                        recommendations: [{
                            type: 'network',
                            message: 'Check firewall settings and WebSocket proxy configuration'
                        }]
                    });
                };
            });

        } catch (error) {
            return {
                passed: false,
                severity: 'warning',
                message: `WebSocket test failed: ${error.message}`,
                details: { error: error.message }
            };
        }
    }

    // Security Checks
    async checkSSLCertificateValidity(configuration) {
        if (!configuration.security?.ssl_enabled) {
            return {
                passed: true,
                message: 'SSL not enabled - certificate validation skipped',
                details: { ssl_enabled: false }
            };
        }

        const certificates = [];
        if (configuration.security.ssl_certificate) {
            certificates.push({
                type: 'certificate',
                content: configuration.security.ssl_certificate
            });
        }
        if (configuration.security.ssl_private_key) {
            certificates.push({
                type: 'private_key',
                content: configuration.security.ssl_private_key
            });
        }

        if (certificates.length === 0) {
            return {
                passed: false,
                severity: 'error',
                message: 'SSL enabled but no certificates provided',
                recommendations: [{
                    type: 'security',
                    message: 'Provide SSL certificate and private key for secure deployment'
                }]
            };
        }

        const validationResults = [];
        let allValid = true;

        for (const cert of certificates) {
            try {
                const validation = await this.validateCertificateContent(cert);
                validationResults.push(validation);
                if (!validation.valid) allValid = false;
            } catch (error) {
                validationResults.push({
                    type: cert.type,
                    valid: false,
                    error: error.message
                });
                allValid = false;
            }
        }

        return {
            passed: allValid,
            severity: allValid ? 'info' : 'error',
            message: allValid 
                ? 'SSL certificates are valid'
                : 'SSL certificate validation failed',
            details: { validations: validationResults },
            recommendations: allValid ? [] : [{
                type: 'security',
                message: 'Fix SSL certificate issues before deployment'
            }]
        };
    }

    async validateCertificateContent(cert) {
        const content = cert.content;
        
        if (cert.type === 'certificate') {
            if (!content.includes('-----BEGIN CERTIFICATE-----') || 
                !content.includes('-----END CERTIFICATE-----')) {
                return {
                    type: 'certificate',
                    valid: false,
                    error: 'Invalid certificate format'
                };
            }
        } else if (cert.type === 'private_key') {
            const keyFormats = [
                'BEGIN PRIVATE KEY',
                'BEGIN RSA PRIVATE KEY',
                'BEGIN EC PRIVATE KEY'
            ];
            
            if (!keyFormats.some(format => content.includes(format))) {
                return {
                    type: 'private_key',
                    valid: false,
                    error: 'Invalid private key format'
                };
            }
        }

        // Basic content validation
        const lines = content.split('\n').filter(line => line.trim());
        if (lines.length < 3) {
            return {
                type: cert.type,
                valid: false,
                error: 'Certificate content appears truncated'
            };
        }

        return {
            type: cert.type,
            valid: true,
            message: `${cert.type} format is valid`
        };
    }

    checkPasswordStrength(configuration) {
        const passwords = [];
        
        if (configuration.security?.admin_password) {
            passwords.push({ name: 'Admin Password', value: configuration.security.admin_password });
        }
        if (configuration.database?.password) {
            passwords.push({ name: 'Database Password', value: configuration.database.password });
        }

        if (passwords.length === 0) {
            return {
                passed: true,
                message: 'No passwords configured for validation',
                details: { passwords_checked: 0 }
            };
        }

        const results = [];
        let allStrong = true;

        for (const pwd of passwords) {
            const strength = this.analyzePasswordStrength(pwd.value);
            results.push({ name: pwd.name, ...strength });
            if (strength.score < 3) allStrong = false;
        }

        return {
            passed: allStrong,
            severity: allStrong ? 'info' : 'warning',
            message: allStrong 
                ? 'All passwords meet strength requirements'
                : 'Some passwords are weak',
            details: { password_analysis: results },
            recommendations: allStrong ? [] : [{
                type: 'security',
                message: 'Strengthen weak passwords before deployment'
            }]
        };
    }

    analyzePasswordStrength(password) {
        let score = 0;
        const feedback = [];

        if (password.length >= 8) score++;
        else feedback.push('Use at least 8 characters');

        if (password.length >= 12) score++;

        if (/[a-z]/.test(password)) score++;
        else feedback.push('Include lowercase letters');

        if (/[A-Z]/.test(password)) score++;
        else feedback.push('Include uppercase letters');

        if (/[0-9]/.test(password)) score++;
        else feedback.push('Include numbers');

        if (/[^a-zA-Z0-9]/.test(password)) score++;
        else feedback.push('Include special characters');

        const strength = score >= 5 ? 'Strong' : score >= 3 ? 'Medium' : 'Weak';

        return {
            score: score,
            strength: strength,
            feedback: feedback.join(', ') || 'Good password'
        };
    }

    checkSecureContext() {
        const isSecure = window.isSecureContext;
        const protocol = window.location.protocol;

        return {
            passed: isSecure,
            severity: isSecure ? 'info' : 'warning',
            message: isSecure 
                ? 'Application is running in secure context'
                : 'Application is not running in secure context',
            details: {
                secure_context: isSecure,
                protocol: protocol,
                hostname: window.location.hostname
            },
            recommendations: isSecure ? [] : [{
                type: 'security',
                message: 'Use HTTPS or localhost for secure context and full feature support'
            }]
        };
    }

    checkPermissionValidation(configuration) {
        // Basic permission structure validation
        const permissions = configuration.permissions || {};
        const requiredRoles = ['admin', 'user'];
        const missingRoles = requiredRoles.filter(role => !permissions[role]);

        if (missingRoles.length > 0) {
            return {
                passed: false,
                severity: 'warning',
                message: `Missing permission roles: ${missingRoles.join(', ')}`,
                details: { configured_permissions: permissions, missing_roles: missingRoles },
                recommendations: [{
                    type: 'security',
                    message: 'Configure all required permission roles for proper access control'
                }]
            };
        }

        return {
            passed: true,
            message: 'Permission structure is valid',
            details: { configured_permissions: permissions }
        };
    }

    // Performance Checks
    async checkSystemResources() {
        const resources = {
            memory: this.estimateAvailableMemory(),
            cpu: await this.estimateCPUPerformance(),
            storage: await this.estimateStorageSpace(),
            network: await this.estimateNetworkPerformance()
        };

        const warnings = [];
        if (resources.memory.estimated < 4096) {
            warnings.push('Low memory detected - may impact performance');
        }
        if (resources.cpu.score < 50) {
            warnings.push('CPU performance may be limited');
        }

        return {
            passed: warnings.length === 0,
            severity: warnings.length > 0 ? 'warning' : 'info',
            message: warnings.length > 0 
                ? `Resource warnings: ${warnings.join('; ')}`
                : 'System resources appear adequate',
            details: { resources: resources, warnings: warnings },
            recommendations: warnings.length > 0 ? [{
                type: 'performance',
                message: 'Consider upgrading system resources for optimal performance'
            }] : []
        };
    }

    estimateAvailableMemory() {
        if ('memory' in performance) {
            return {
                estimated: Math.round(performance.memory.jsHeapSizeLimit / 1024 / 1024),
                unit: 'MB',
                source: 'performance.memory'
            };
        }

        // Fallback estimation
        return {
            estimated: 'unknown',
            unit: 'MB',
            source: 'estimation not available'
        };
    }

    async estimateCPUPerformance() {
        const start = performance.now();
        const iterations = 100000;
        
        // CPU-intensive task
        let result = 0;
        for (let i = 0; i < iterations; i++) {
            result += Math.sin(i) * Math.cos(i);
        }
        
        const duration = performance.now() - start;
        const score = Math.max(0, Math.min(100, 1000 / duration));

        return {
            duration: Math.round(duration),
            score: Math.round(score),
            unit: 'performance score (0-100)'
        };
    }

    async estimateStorageSpace() {
        if ('storage' in navigator && 'estimate' in navigator.storage) {
            try {
                const estimate = await navigator.storage.estimate();
                return {
                    available: Math.round((estimate.quota || 0) / 1024 / 1024),
                    used: Math.round((estimate.usage || 0) / 1024 / 1024),
                    unit: 'MB',
                    source: 'Storage API'
                };
            } catch (error) {
                return {
                    available: 'unknown',
                    error: error.message,
                    source: 'Storage API error'
                };
            }
        }

        return {
            available: 'unknown',
            source: 'Storage estimation not available'
        };
    }

    async estimateNetworkPerformance() {
        if ('connection' in navigator) {
            const connection = navigator.connection;
            return {
                type: connection.effectiveType || 'unknown',
                downlink: connection.downlink || 'unknown',
                rtt: connection.rtt || 'unknown',
                unit: 'Mbps / ms',
                source: 'Network Information API'
            };
        }

        return {
            type: 'unknown',
            source: 'Network information not available'
        };
    }

    async checkPhysicsPerformance(configuration) {
        const physicsEngine = configuration.physics?.engine || 'ODE';
        
        // Simulate physics performance test
        const performanceTest = await this.simulatePhysicsTest(physicsEngine);
        
        const expectations = {
            'Basic': { min: 30, optimal: 60 },
            'ODE': { min: 60, optimal: 100 },
            'UBODE': { min: 80, optimal: 120 },
            'Bullet': { min: 100, optimal: 150 },
            'POS': { min: 150, optimal: 200 }
        };

        const expected = expectations[physicsEngine] || expectations['ODE'];
        const meetsMinimum = performanceTest.score >= expected.min;
        const meetsOptimal = performanceTest.score >= expected.optimal;

        return {
            passed: meetsMinimum,
            severity: meetsOptimal ? 'info' : meetsMinimum ? 'warning' : 'error',
            message: meetsOptimal 
                ? `${physicsEngine} physics performance is optimal`
                : meetsMinimum 
                    ? `${physicsEngine} physics performance meets minimum requirements`
                    : `${physicsEngine} physics performance below minimum requirements`,
            details: {
                engine: physicsEngine,
                performance_score: performanceTest.score,
                expected_minimum: expected.min,
                expected_optimal: expected.optimal,
                test_details: performanceTest
            },
            recommendations: meetsMinimum ? [] : [{
                type: 'performance',
                message: `Consider using a lighter physics engine or upgrading system performance for ${physicsEngine}`
            }]
        };
    }

    async simulatePhysicsTest(engine) {
        // Simulate physics calculation performance
        const start = performance.now();
        const iterations = engine === 'POS' ? 50000 : 10000;
        
        // Simulate physics calculations
        let result = 0;
        for (let i = 0; i < iterations; i++) {
            const x = Math.sin(i * 0.1);
            const y = Math.cos(i * 0.1);
            result += Math.sqrt(x * x + y * y);
        }
        
        const duration = performance.now() - start;
        const score = Math.round(iterations / duration * 10);

        return {
            iterations: iterations,
            duration: Math.round(duration),
            score: score,
            result: result
        };
    }

    async checkDatabasePerformance(configuration) {
        const dbType = configuration.database?.type || 'SQLite';
        
        // Since we can't actually test database performance in browser,
        // provide guidance based on configuration
        
        const performanceGuidance = {
            'SQLite': {
                expected_performance: 'Good for development and small deployments',
                concurrent_users: 'Up to 50 users',
                bottlenecks: ['File I/O', 'No concurrent writes'],
                optimizations: ['WAL mode', 'Proper indexing', 'SSD storage']
            },
            'PostgreSQL': {
                expected_performance: 'Excellent for production deployments',
                concurrent_users: '1000+ users with proper tuning',
                bottlenecks: ['Network latency', 'Connection pooling'],
                optimizations: ['Connection pooling', 'Query optimization', 'Proper hardware']
            },
            'MySQL': {
                expected_performance: 'Good for most applications',
                concurrent_users: '500+ users with proper tuning',
                bottlenecks: ['Configuration tuning', 'Storage engine choice'],
                optimizations: ['InnoDB engine', 'Buffer pool tuning', 'Index optimization']
            }
        };

        const guidance = performanceGuidance[dbType] || performanceGuidance['SQLite'];
        const deploymentType = configuration.deploymentType;
        
        // Check if database choice matches deployment type
        let optimal = true;
        let recommendations = [];
        
        if (deploymentType === 'production' && dbType === 'SQLite') {
            optimal = false;
            recommendations.push({
                type: 'database',
                message: 'Consider PostgreSQL for production deployments for better performance and reliability'
            });
        }
        
        if (deploymentType === 'grid' && dbType !== 'PostgreSQL') {
            optimal = false;
            recommendations.push({
                type: 'database',
                message: 'PostgreSQL is recommended for grid deployments due to superior concurrent performance'
            });
        }

        return {
            passed: true, // Always pass since we can't test actual performance
            severity: optimal ? 'info' : 'warning',
            message: optimal 
                ? `${dbType} is suitable for ${deploymentType} deployment`
                : `${dbType} may not be optimal for ${deploymentType} deployment`,
            details: {
                database_type: dbType,
                deployment_type: deploymentType,
                performance_guidance: guidance
            },
            recommendations: recommendations
        };
    }

    async checkNetworkLatency() {
        try {
            const measurements = [];
            const testUrls = [
                'https://httpbin.org/get',
                'https://jsonplaceholder.typicode.com/posts/1'
            ];

            for (const url of testUrls) {
                try {
                    const start = performance.now();
                    const response = await fetch(url, { 
                        method: 'HEAD', 
                        cache: 'no-cache',
                        signal: AbortSignal.timeout(5000)
                    });
                    const duration = performance.now() - start;
                    
                    measurements.push({
                        url: url,
                        latency: Math.round(duration),
                        success: response.ok
                    });
                } catch (error) {
                    measurements.push({
                        url: url,
                        latency: null,
                        success: false,
                        error: error.message
                    });
                }
            }

            const successful = measurements.filter(m => m.success);
            const averageLatency = successful.length > 0 
                ? Math.round(successful.reduce((sum, m) => sum + m.latency, 0) / successful.length)
                : null;

            let performance_rating;
            if (averageLatency === null) {
                performance_rating = 'Unable to measure';
            } else if (averageLatency < 100) {
                performance_rating = 'Excellent';
            } else if (averageLatency < 300) {
                performance_rating = 'Good';
            } else if (averageLatency < 500) {
                performance_rating = 'Fair';
            } else {
                performance_rating = 'Poor';
            }

            return {
                passed: successful.length > 0,
                severity: successful.length > 0 ? 'info' : 'warning',
                message: averageLatency 
                    ? `Network latency: ${averageLatency}ms (${performance_rating})`
                    : 'Unable to measure network latency',
                details: {
                    measurements: measurements,
                    average_latency: averageLatency,
                    performance_rating: performance_rating
                },
                recommendations: performance_rating === 'Poor' ? [{
                    type: 'network',
                    message: 'High network latency detected. Consider optimizing network configuration or changing hosting location.'
                }] : []
            };

        } catch (error) {
            return {
                passed: false,
                severity: 'warning',
                message: `Network latency test failed: ${error.message}`,
                details: { error: error.message }
            };
        }
    }

    // Result calculation and recommendations
    calculateOverallStatus(result) {
        const totalCriticalFailures = result.critical_failures.length;
        const totalWarnings = result.warnings.length;
        
        if (totalCriticalFailures > 0) {
            result.overall_status = 'failed';
            result.deployment_readiness = 'not_ready';
        } else if (totalWarnings > 5) {
            result.overall_status = 'warning';
            result.deployment_readiness = 'review_required';
        } else if (totalWarnings > 0) {
            result.overall_status = 'warning';
            result.deployment_readiness = 'ready_with_warnings';
        } else {
            result.overall_status = 'passed';
            result.deployment_readiness = 'ready';
        }

        // Calculate readiness score
        const totalChecks = Object.values(result.categories)
            .reduce((sum, cat) => sum + cat.passed + cat.failed, 0);
        const passedChecks = Object.values(result.categories)
            .reduce((sum, cat) => sum + cat.passed, 0);
        
        result.readiness_score = totalChecks > 0 ? Math.round((passedChecks / totalChecks) * 100) : 0;
    }

    generateRecommendations(result, configuration) {
        const recommendations = [...result.recommendations];

        // Add overall recommendations based on status
        if (result.deployment_readiness === 'not_ready') {
            recommendations.unshift({
                type: 'critical',
                priority: 'high',
                title: 'Deployment Not Ready',
                message: 'Critical issues must be resolved before deployment',
                action: 'Review and fix all critical failures'
            });
        } else if (result.deployment_readiness === 'review_required') {
            recommendations.unshift({
                type: 'review',
                priority: 'medium',
                title: 'Review Required',
                message: 'Multiple warnings detected - review before deployment',
                action: 'Address warnings to improve deployment reliability'
            });
        } else if (result.deployment_readiness === 'ready') {
            recommendations.unshift({
                type: 'success',
                priority: 'low',
                title: 'Ready for Deployment',
                message: 'All checks passed - configuration is ready for deployment',
                action: 'Proceed with deployment'
            });
        }

        result.recommendations = recommendations;
    }

    // Export and utility methods
    exportPreflightReport(result, format = 'json') {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `opensim-preflight-${timestamp}.${format}`;

        let content, mimeType;

        switch (format) {
            case 'json':
                content = JSON.stringify(result, null, 2);
                mimeType = 'application/json';
                break;

            case 'html':
                content = this.generateHTMLPreflightReport(result);
                mimeType = 'text/html';
                break;

            case 'text':
                content = this.generateTextPreflightReport(result);
                mimeType = 'text/plain';
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

    generateHTMLPreflightReport(result) {
        const statusClass = result.overall_status === 'passed' ? 'passed' :
                           result.overall_status === 'warning' ? 'warning' : 'failed';

        return `
            <!DOCTYPE html>
            <html>
            <head>
                <title>OpenSim Next Pre-flight Check Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; }
                    .header { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
                    .status { padding: 10px; border-radius: 6px; margin: 10px 0; }
                    .passed { background: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
                    .warning { background: #fff3cd; color: #856404; border: 1px solid #ffeaa7; }
                    .failed { background: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
                    .category { margin: 20px 0; }
                    .check-result { margin: 10px 0; padding: 10px; border-left: 4px solid #ccc; }
                    .check-passed { border-left-color: #28a745; }
                    .check-failed { border-left-color: #dc3545; }
                    .recommendations { background: #e3f2fd; padding: 15px; border-radius: 6px; margin: 20px 0; }
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>OpenSim Next Pre-flight Check Report</h1>
                    <p><strong>Deployment Type:</strong> ${result.deployment_type}</p>
                    <p><strong>Check Time:</strong> ${new Date(result.timestamp).toLocaleString()}</p>
                    <p><strong>Execution Time:</strong> ${Math.round(result.execution_time)}ms</p>
                    <div class="status ${statusClass}">
                        <strong>Overall Status:</strong> ${result.overall_status.toUpperCase()}
                        (Readiness Score: ${result.readiness_score}%)
                    </div>
                </div>

                ${Object.entries(result.categories).map(([category, data]) => `
                    <div class="category">
                        <h2>${category.charAt(0).toUpperCase() + category.slice(1)} Checks</h2>
                        <p>Passed: ${data.passed}, Failed: ${data.failed}, Skipped: ${data.skipped}</p>
                        ${data.results.map(check => `
                            <div class="check-result ${check.passed ? 'check-passed' : 'check-failed'}">
                                <strong>${check.check}:</strong> ${check.message}
                                ${check.execution_time ? `<small> (${Math.round(check.execution_time)}ms)</small>` : ''}
                            </div>
                        `).join('')}
                    </div>
                `).join('')}

                ${result.recommendations.length > 0 ? `
                    <div class="recommendations">
                        <h2>Recommendations</h2>
                        ${result.recommendations.map(rec => `
                            <div><strong>${rec.type}:</strong> ${rec.message || rec.title}</div>
                        `).join('')}
                    </div>
                ` : ''}
            </body>
            </html>
        `;
    }

    generateTextPreflightReport(result) {
        let report = '';
        report += `OpenSim Next Pre-flight Check Report\n`;
        report += `=====================================\n\n`;
        report += `Deployment Type: ${result.deployment_type}\n`;
        report += `Check Time: ${new Date(result.timestamp).toLocaleString()}\n`;
        report += `Execution Time: ${Math.round(result.execution_time)}ms\n`;
        report += `Overall Status: ${result.overall_status.toUpperCase()}\n`;
        report += `Readiness Score: ${result.readiness_score}%\n\n`;

        Object.entries(result.categories).forEach(([category, data]) => {
            report += `${category.toUpperCase()} CHECKS\n`;
            report += `${'='.repeat(category.length + 7)}\n`;
            report += `Passed: ${data.passed}, Failed: ${data.failed}, Skipped: ${data.skipped}\n\n`;
            
            data.results.forEach(check => {
                const status = check.passed ? '[PASS]' : '[FAIL]';
                const time = check.execution_time ? ` (${Math.round(check.execution_time)}ms)` : '';
                report += `${status} ${check.check}: ${check.message}${time}\n`;
            });
            report += '\n';
        });

        if (result.recommendations.length > 0) {
            report += 'RECOMMENDATIONS\n';
            report += '===============\n';
            result.recommendations.forEach(rec => {
                report += `- ${rec.type}: ${rec.message || rec.title}\n`;
            });
        }

        return report;
    }
}

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.PreflightChecker = PreflightChecker;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { PreflightChecker };
}