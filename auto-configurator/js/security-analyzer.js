// OpenSim Next Auto-Configurator - Security Analyzer
// Advanced security analysis and threat assessment for OpenSim configurations

class SecurityAnalyzer {
    constructor() {
        this.securityRules = new Map();
        this.threatDatabase = new Map();
        this.complianceFrameworks = new Map();
        this.securityProfiles = new Map();
        
        this.initializeSecurityRules();
        this.initializeThreatDatabase();
        this.initializeComplianceFrameworks();
        this.initializeSecurityProfiles();
    }

    initializeSecurityRules() {
        // Encryption and Data Protection Rules
        this.registerSecurityRule('encryption_at_rest', {
            category: 'data_protection',
            severity: 'high',
            description: 'Validate encryption for data at rest',
            checker: this.checkEncryptionAtRest.bind(this),
            compliance: ['SOX', 'GDPR', 'HIPAA']
        });

        this.registerSecurityRule('encryption_in_transit', {
            category: 'data_protection',
            severity: 'critical',
            description: 'Validate encryption for data in transit',
            checker: this.checkEncryptionInTransit.bind(this),
            compliance: ['PCI-DSS', 'SOX', 'GDPR']
        });

        // Authentication and Access Control Rules
        this.registerSecurityRule('authentication_strength', {
            category: 'access_control',
            severity: 'high',
            description: 'Assess authentication mechanism strength',
            checker: this.checkAuthenticationStrength.bind(this),
            compliance: ['ISO27001', 'NIST']
        });

        this.registerSecurityRule('password_policy', {
            category: 'access_control',
            severity: 'medium',
            description: 'Validate password policy compliance',
            checker: this.checkPasswordPolicy.bind(this),
            compliance: ['NIST', 'ISO27001']
        });

        this.registerSecurityRule('session_management', {
            category: 'access_control',
            severity: 'high',
            description: 'Evaluate session management security',
            checker: this.checkSessionManagement.bind(this),
            compliance: ['OWASP', 'ISO27001']
        });

        // Network Security Rules
        this.registerSecurityRule('network_segmentation', {
            category: 'network_security',
            severity: 'medium',
            description: 'Assess network segmentation and isolation',
            checker: this.checkNetworkSegmentation.bind(this),
            compliance: ['PCI-DSS', 'ISO27001']
        });

        this.registerSecurityRule('firewall_configuration', {
            category: 'network_security',
            severity: 'high',
            description: 'Validate firewall and port security',
            checker: this.checkFirewallConfiguration.bind(this),
            compliance: ['PCI-DSS', 'NIST']
        });

        this.registerSecurityRule('ssl_tls_configuration', {
            category: 'network_security',
            severity: 'critical',
            description: 'Validate SSL/TLS configuration security',
            checker: this.checkSSLTLSConfiguration.bind(this),
            compliance: ['PCI-DSS', 'OWASP']
        });

        // Application Security Rules
        this.registerSecurityRule('input_validation', {
            category: 'application_security',
            severity: 'high',
            description: 'Assess input validation and sanitization',
            checker: this.checkInputValidation.bind(this),
            compliance: ['OWASP', 'SANS']
        });

        this.registerSecurityRule('error_handling', {
            category: 'application_security',
            severity: 'medium',
            description: 'Validate secure error handling',
            checker: this.checkErrorHandling.bind(this),
            compliance: ['OWASP']
        });

        this.registerSecurityRule('logging_and_monitoring', {
            category: 'monitoring',
            severity: 'high',
            description: 'Evaluate logging and monitoring capabilities',
            checker: this.checkLoggingAndMonitoring.bind(this),
            compliance: ['SOX', 'PCI-DSS', 'ISO27001']
        });

        // Data Privacy Rules
        this.registerSecurityRule('data_minimization', {
            category: 'privacy',
            severity: 'medium',
            description: 'Assess data collection and retention policies',
            checker: this.checkDataMinimization.bind(this),
            compliance: ['GDPR', 'CCPA']
        });

        this.registerSecurityRule('privacy_controls', {
            category: 'privacy',
            severity: 'high',
            description: 'Validate privacy protection mechanisms',
            checker: this.checkPrivacyControls.bind(this),
            compliance: ['GDPR', 'CCPA']
        });

        console.log('Security rules initialized');
    }

    initializeThreatDatabase() {
        // Common virtual world security threats
        this.threatDatabase.set('avatar_impersonation', {
            description: 'Unauthorized users impersonating legitimate avatars',
            risk_level: 'high',
            attack_vectors: ['Weak authentication', 'Session hijacking', 'Credential theft'],
            mitigation: ['Strong authentication', 'SSL/TLS encryption', 'Session management'],
            cve_references: ['CVE-2019-1234'] // Example CVE
        });

        this.threatDatabase.set('asset_theft', {
            description: 'Unauthorized access or copying of virtual world assets',
            risk_level: 'medium',
            attack_vectors: ['Insufficient access controls', 'Asset server vulnerabilities'],
            mitigation: ['Asset encryption', 'Access controls', 'Digital rights management'],
            cve_references: []
        });

        this.threatDatabase.set('griefing_attacks', {
            description: 'Malicious behavior designed to disrupt other users',
            risk_level: 'medium',
            attack_vectors: ['Script abuse', 'Physics exploitation', 'Chat spam'],
            mitigation: ['Script limitations', 'Physics sandboxing', 'User reporting'],
            cve_references: []
        });

        this.threatDatabase.set('data_exfiltration', {
            description: 'Unauthorized extraction of sensitive user data',
            risk_level: 'critical',
            attack_vectors: ['Database vulnerabilities', 'API abuse', 'Insider threats'],
            mitigation: ['Data encryption', 'Access monitoring', 'API rate limiting'],
            cve_references: ['CVE-2020-5678'] // Example CVE
        });

        this.threatDatabase.set('ddos_attacks', {
            description: 'Distributed denial of service attacks on virtual world infrastructure',
            risk_level: 'high',
            attack_vectors: ['Network flooding', 'Resource exhaustion', 'Amplification attacks'],
            mitigation: ['Rate limiting', 'Load balancing', 'DDoS protection services'],
            cve_references: []
        });

        this.threatDatabase.set('privilege_escalation', {
            description: 'Unauthorized elevation of user privileges',
            risk_level: 'critical',
            attack_vectors: ['Authentication bypasses', 'Authorization flaws', 'System vulnerabilities'],
            mitigation: ['Least privilege principle', 'Regular security audits', 'Access controls'],
            cve_references: ['CVE-2021-9012'] // Example CVE
        });

        console.log('Threat database initialized');
    }

    initializeComplianceFrameworks() {
        // GDPR (General Data Protection Regulation)
        this.complianceFrameworks.set('GDPR', {
            name: 'General Data Protection Regulation',
            description: 'EU privacy regulation for personal data protection',
            requirements: [
                'Data encryption at rest and in transit',
                'User consent mechanisms',
                'Data portability features',
                'Right to be forgotten implementation',
                'Privacy by design principles',
                'Data breach notification procedures'
            ],
            applicability: 'EU users or EU-based deployments',
            penalties: 'Up to 4% of annual revenue or €20 million'
        });

        // PCI-DSS (Payment Card Industry Data Security Standard)
        this.complianceFrameworks.set('PCI-DSS', {
            name: 'Payment Card Industry Data Security Standard',
            description: 'Security standard for payment card processing',
            requirements: [
                'Strong encryption for payment data',
                'Secure network architecture',
                'Regular security testing',
                'Access control implementation',
                'Network monitoring',
                'Information security policy'
            ],
            applicability: 'Systems processing payment cards',
            penalties: 'Fines and loss of payment processing privileges'
        });

        // SOX (Sarbanes-Oxley Act)
        this.complianceFrameworks.set('SOX', {
            name: 'Sarbanes-Oxley Act',
            description: 'US federal law for corporate financial reporting',
            requirements: [
                'Data integrity controls',
                'Access logging and monitoring',
                'Change management procedures',
                'Backup and recovery processes',
                'Audit trail maintenance',
                'Internal controls documentation'
            ],
            applicability: 'Public companies and subsidiaries',
            penalties: 'Criminal and civil penalties'
        });

        // OWASP Top 10
        this.complianceFrameworks.set('OWASP', {
            name: 'OWASP Top 10',
            description: 'Top 10 web application security risks',
            requirements: [
                'Injection prevention',
                'Broken authentication protection',
                'Sensitive data exposure prevention',
                'XML external entities (XXE) protection',
                'Broken access control prevention',
                'Security misconfiguration avoidance',
                'Cross-site scripting (XSS) prevention',
                'Insecure deserialization protection',
                'Known vulnerabilities management',
                'Insufficient logging and monitoring'
            ],
            applicability: 'Web applications',
            penalties: 'Security vulnerabilities and data breaches'
        });

        console.log('Compliance frameworks initialized');
    }

    initializeSecurityProfiles() {
        // Development Security Profile
        this.securityProfiles.set('development', {
            name: 'Development Environment',
            description: 'Relaxed security for development and testing',
            requirements: {
                encryption: 'optional',
                authentication: 'basic',
                logging: 'debug',
                monitoring: 'minimal',
                compliance: []
            },
            risks: ['Data exposure', 'Weak authentication'],
            mitigations: ['Network isolation', 'Test data usage', 'Regular updates']
        });

        // Production Security Profile
        this.securityProfiles.set('production', {
            name: 'Production Environment',
            description: 'Full security controls for production deployment',
            requirements: {
                encryption: 'required',
                authentication: 'strong',
                logging: 'comprehensive',
                monitoring: 'real-time',
                compliance: ['OWASP', 'ISO27001']
            },
            risks: ['Data breaches', 'Service disruption'],
            mitigations: ['Defense in depth', 'Regular audits', 'Incident response']
        });

        // Enterprise Security Profile
        this.securityProfiles.set('enterprise', {
            name: 'Enterprise Environment',
            description: 'Maximum security for enterprise deployments',
            requirements: {
                encryption: 'enterprise-grade',
                authentication: 'multi-factor',
                logging: 'immutable',
                monitoring: 'advanced',
                compliance: ['GDPR', 'SOX', 'PCI-DSS', 'ISO27001']
            },
            risks: ['Regulatory violations', 'Financial loss'],
            mitigations: ['Zero trust architecture', 'Continuous monitoring', 'Compliance automation']
        });

        console.log('Security profiles initialized');
    }

    registerSecurityRule(name, rule) {
        this.securityRules.set(name, { ...rule, name });
    }

    async analyzeSecurityPosture(configuration) {
        const analysis = {
            timestamp: new Date().toISOString(),
            configuration_id: configuration.id || 'unknown',
            deployment_type: configuration.deploymentType,
            security_profile: this.determineSecurityProfile(configuration),
            overall_score: 0,
            risk_level: 'unknown',
            categories: {
                data_protection: { score: 0, findings: [], recommendations: [] },
                access_control: { score: 0, findings: [], recommendations: [] },
                network_security: { score: 0, findings: [], recommendations: [] },
                application_security: { score: 0, findings: [], recommendations: [] },
                monitoring: { score: 0, findings: [], recommendations: [] },
                privacy: { score: 0, findings: [], recommendations: [] }
            },
            threats_identified: [],
            compliance_gaps: [],
            security_recommendations: [],
            remediation_plan: []
        };

        try {
            // Run security rule analysis
            await this.runSecurityAnalysis(configuration, analysis);
            
            // Identify threats
            analysis.threats_identified = this.identifyThreats(configuration, analysis);
            
            // Check compliance
            analysis.compliance_gaps = this.checkCompliance(configuration, analysis);
            
            // Generate recommendations
            analysis.security_recommendations = this.generateSecurityRecommendations(analysis);
            
            // Create remediation plan
            analysis.remediation_plan = this.createRemediationPlan(analysis);
            
            // Calculate overall security score
            this.calculateSecurityScore(analysis);

            console.log('Security analysis completed:', analysis);
            return analysis;

        } catch (error) {
            console.error('Security analysis failed:', error);
            analysis.error = error.message;
            analysis.risk_level = 'critical';
            return analysis;
        }
    }

    determineSecurityProfile(configuration) {
        const deploymentType = configuration.deploymentType;
        
        if (deploymentType === 'development') {
            return 'development';
        } else if (deploymentType === 'grid' || configuration.security?.enterprise_features) {
            return 'enterprise';
        } else {
            return 'production';
        }
    }

    async runSecurityAnalysis(configuration, analysis) {
        for (const [ruleName, rule] of this.securityRules) {
            try {
                const result = await rule.checker(configuration);
                const category = analysis.categories[rule.category];
                
                if (category) {
                    category.findings.push({
                        rule: ruleName,
                        severity: rule.severity,
                        result: result,
                        compliance: rule.compliance
                    });
                    
                    // Update category score
                    const ruleScore = result.passed ? 100 : result.score || 0;
                    category.score = (category.score + ruleScore) / 2; // Simple averaging
                    
                    // Add recommendations
                    if (result.recommendations) {
                        category.recommendations.push(...result.recommendations);
                    }
                }
                
            } catch (error) {
                console.error(`Security rule ${ruleName} failed:`, error);
                const category = analysis.categories[rule.category];
                if (category) {
                    category.findings.push({
                        rule: ruleName,
                        severity: rule.severity,
                        result: {
                            passed: false,
                            error: error.message,
                            message: `Rule execution failed: ${error.message}`
                        }
                    });
                }
            }
        }
    }

    // Security Rule Implementations
    checkEncryptionAtRest(configuration) {
        const findings = [];
        let score = 0;

        // Check database encryption
        if (configuration.database?.encryption_enabled) {
            findings.push('Database encryption is enabled');
            score += 30;
        } else {
            findings.push('Database encryption is not configured');
        }

        // Check asset storage encryption
        if (configuration.assets?.encryption_enabled) {
            findings.push('Asset storage encryption is enabled');
            score += 20;
        } else {
            findings.push('Asset storage encryption is not configured');
        }

        // Check backup encryption
        if (configuration.backup?.encryption_enabled) {
            findings.push('Backup encryption is enabled');
            score += 15;
        } else {
            findings.push('Backup encryption is not configured');
        }

        // Check log encryption
        if (configuration.logging?.encryption_enabled) {
            findings.push('Log encryption is enabled');
            score += 10;
        }

        const passed = score >= 50; // Require at least database encryption
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Encryption at rest is properly configured' : 'Encryption at rest needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Enable database encryption for sensitive data protection',
                'Configure asset storage encryption',
                'Implement backup encryption for data protection'
            ]
        };
    }

    checkEncryptionInTransit(configuration) {
        const findings = [];
        let score = 0;

        // Check SSL/TLS for main services
        if (configuration.security?.ssl_enabled) {
            findings.push('SSL/TLS is enabled for main services');
            score += 40;
            
            // Check SSL version
            const sslVersion = configuration.security.ssl_version || 'TLS1.2';
            if (sslVersion === 'TLS1.3') {
                findings.push('Using TLS 1.3 (excellent security)');
                score += 20;
            } else if (sslVersion === 'TLS1.2') {
                findings.push('Using TLS 1.2 (good security)');
                score += 10;
            } else {
                findings.push('Using outdated TLS version');
                score -= 10;
            }
        } else {
            findings.push('SSL/TLS is not enabled');
        }

        // Check database connection encryption
        if (configuration.database?.ssl_enabled) {
            findings.push('Database connections are encrypted');
            score += 20;
        } else {
            findings.push('Database connections are not encrypted');
        }

        // Check inter-service communication
        if (configuration.network?.service_mesh_enabled) {
            findings.push('Service mesh provides encrypted inter-service communication');
            score += 20;
        } else {
            findings.push('Inter-service communication encryption not configured');
        }

        const passed = score >= 60; // Require SSL and database encryption
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Encryption in transit is properly configured' : 'Encryption in transit needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Enable SSL/TLS for all client connections',
                'Use TLS 1.3 for maximum security',
                'Enable database connection encryption',
                'Implement service mesh for encrypted inter-service communication'
            ]
        };
    }

    checkAuthenticationStrength(configuration) {
        const findings = [];
        let score = 0;

        const authLevel = configuration.security?.authentication_level || 'basic';
        
        switch (authLevel) {
            case 'enterprise':
                findings.push('Enterprise-grade authentication configured');
                score += 50;
                break;
            case 'enhanced':
                findings.push('Enhanced authentication configured');
                score += 30;
                break;
            case 'basic':
                findings.push('Basic authentication configured');
                score += 10;
                break;
            default:
                findings.push('Authentication level not specified');
        }

        // Check multi-factor authentication
        if (configuration.security?.mfa_enabled) {
            findings.push('Multi-factor authentication is enabled');
            score += 30;
        } else {
            findings.push('Multi-factor authentication is not enabled');
        }

        // Check password complexity
        if (configuration.security?.password_policy) {
            findings.push('Password policy is configured');
            score += 15;
        } else {
            findings.push('No password policy configured');
        }

        // Check session management
        if (configuration.security?.session_timeout) {
            findings.push('Session timeout is configured');
            score += 5;
        }

        const passed = score >= 40;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Authentication strength is adequate' : 'Authentication needs strengthening',
            findings: findings,
            recommendations: passed ? [] : [
                'Implement multi-factor authentication',
                'Configure strong password policies',
                'Use enterprise-grade authentication for production',
                'Implement session management controls'
            ]
        };
    }

    checkPasswordPolicy(configuration) {
        const policy = configuration.security?.password_policy || {};
        const findings = [];
        let score = 0;

        // Check minimum length
        const minLength = policy.min_length || 0;
        if (minLength >= 12) {
            findings.push('Strong minimum password length (12+ chars)');
            score += 25;
        } else if (minLength >= 8) {
            findings.push('Adequate minimum password length (8+ chars)');
            score += 15;
        } else {
            findings.push('Weak minimum password length');
        }

        // Check complexity requirements
        if (policy.require_uppercase) {
            findings.push('Uppercase letters required');
            score += 10;
        }
        if (policy.require_lowercase) {
            findings.push('Lowercase letters required');
            score += 10;
        }
        if (policy.require_numbers) {
            findings.push('Numbers required');
            score += 10;
        }
        if (policy.require_special_chars) {
            findings.push('Special characters required');
            score += 15;
        }

        // Check password history
        if (policy.password_history) {
            findings.push('Password history tracking enabled');
            score += 10;
        }

        // Check expiration
        if (policy.expiration_days) {
            findings.push('Password expiration configured');
            score += 10;
        }

        const passed = score >= 50;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Password policy is strong' : 'Password policy needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Require minimum 12 character passwords',
                'Enforce character complexity requirements',
                'Implement password history tracking',
                'Configure appropriate password expiration'
            ]
        };
    }

    checkSessionManagement(configuration) {
        const session = configuration.security?.session || {};
        const findings = [];
        let score = 0;

        // Check session timeout
        if (session.timeout_minutes) {
            if (session.timeout_minutes <= 30) {
                findings.push('Short session timeout (30 min or less)');
                score += 20;
            } else if (session.timeout_minutes <= 120) {
                findings.push('Moderate session timeout (2 hours or less)');
                score += 10;
            } else {
                findings.push('Long session timeout (over 2 hours)');
            }
        } else {
            findings.push('No session timeout configured');
        }

        // Check secure session cookies
        if (session.secure_cookies) {
            findings.push('Secure session cookies enabled');
            score += 20;
        } else {
            findings.push('Secure session cookies not configured');
        }

        // Check session regeneration
        if (session.regenerate_on_login) {
            findings.push('Session regeneration on login enabled');
            score += 15;
        } else {
            findings.push('Session regeneration not configured');
        }

        // Check concurrent session limits
        if (session.max_concurrent_sessions) {
            findings.push('Concurrent session limits configured');
            score += 10;
        }

        // Check session storage security
        if (session.encrypted_storage) {
            findings.push('Encrypted session storage enabled');
            score += 15;
        } else {
            findings.push('Session storage encryption not configured');
        }

        const passed = score >= 40;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Session management is properly configured' : 'Session management needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Configure appropriate session timeouts',
                'Enable secure session cookies',
                'Implement session regeneration on login',
                'Use encrypted session storage'
            ]
        };
    }

    checkNetworkSegmentation(configuration) {
        const network = configuration.network || {};
        const findings = [];
        let score = 0;

        // Check if network segmentation is configured
        if (network.segmentation_enabled) {
            findings.push('Network segmentation is enabled');
            score += 30;
        } else {
            findings.push('Network segmentation is not configured');
        }

        // Check DMZ configuration
        if (network.dmz_enabled) {
            findings.push('DMZ is configured for external services');
            score += 20;
        } else {
            findings.push('No DMZ configuration detected');
        }

        // Check VLAN usage
        if (network.vlans_configured) {
            findings.push('VLANs are configured for traffic isolation');
            score += 15;
        }

        // Check internal service isolation
        if (network.service_isolation) {
            findings.push('Service isolation is implemented');
            score += 15;
        }

        // Check database network isolation
        if (network.database_isolation) {
            findings.push('Database is on isolated network segment');
            score += 20;
        } else {
            findings.push('Database network isolation not configured');
        }

        const passed = score >= 30;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Network segmentation is adequate' : 'Network segmentation needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Implement network segmentation for service isolation',
                'Configure DMZ for external-facing services',
                'Use VLANs for traffic separation',
                'Isolate database on separate network segment'
            ]
        };
    }

    checkFirewallConfiguration(configuration) {
        const firewall = configuration.firewall || {};
        const findings = [];
        let score = 0;

        // Check if firewall is enabled
        if (firewall.enabled) {
            findings.push('Firewall is enabled');
            score += 25;
        } else {
            findings.push('Firewall is not enabled');
        }

        // Check default deny policy
        if (firewall.default_policy === 'deny') {
            findings.push('Default deny policy configured');
            score += 20;
        } else {
            findings.push('Default deny policy not configured');
        }

        // Check port restrictions
        if (firewall.port_restrictions && firewall.port_restrictions.length > 0) {
            findings.push('Port restrictions are configured');
            score += 15;
        } else {
            findings.push('No port restrictions configured');
        }

        // Check IP whitelisting
        if (firewall.ip_whitelist && firewall.ip_whitelist.length > 0) {
            findings.push('IP whitelisting is configured');
            score += 10;
        }

        // Check rate limiting
        if (firewall.rate_limiting) {
            findings.push('Rate limiting is enabled');
            score += 15;
        } else {
            findings.push('Rate limiting not configured');
        }

        // Check logging
        if (firewall.logging_enabled) {
            findings.push('Firewall logging is enabled');
            score += 15;
        } else {
            findings.push('Firewall logging not enabled');
        }

        const passed = score >= 45;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Firewall configuration is adequate' : 'Firewall configuration needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Enable firewall with default deny policy',
                'Configure port restrictions based on service requirements',
                'Implement rate limiting for DDoS protection',
                'Enable comprehensive firewall logging'
            ]
        };
    }

    checkSSLTLSConfiguration(configuration) {
        const ssl = configuration.security?.ssl || {};
        const findings = [];
        let score = 0;

        // Check SSL/TLS version
        const version = ssl.version || 'TLS1.2';
        if (version === 'TLS1.3') {
            findings.push('Using TLS 1.3 (latest standard)');
            score += 30;
        } else if (version === 'TLS1.2') {
            findings.push('Using TLS 1.2 (acceptable)');
            score += 20;
        } else {
            findings.push('Using outdated TLS version');
        }

        // Check cipher suites
        if (ssl.strong_ciphers_only) {
            findings.push('Strong cipher suites enforced');
            score += 20;
        } else {
            findings.push('Weak cipher suites may be allowed');
        }

        // Check certificate configuration
        if (ssl.certificate_chain_complete) {
            findings.push('Complete certificate chain configured');
            score += 15;
        } else {
            findings.push('Certificate chain may be incomplete');
        }

        // Check HSTS (HTTP Strict Transport Security)
        if (ssl.hsts_enabled) {
            findings.push('HSTS is enabled');
            score += 10;
        } else {
            findings.push('HSTS not configured');
        }

        // Check certificate pinning
        if (ssl.certificate_pinning) {
            findings.push('Certificate pinning is enabled');
            score += 10;
        }

        // Check OCSP stapling
        if (ssl.ocsp_stapling) {
            findings.push('OCSP stapling is enabled');
            score += 5;
        }

        // Check forward secrecy
        if (ssl.perfect_forward_secrecy) {
            findings.push('Perfect forward secrecy is enabled');
            score += 10;
        } else {
            findings.push('Perfect forward secrecy not configured');
        }

        const passed = score >= 60;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'SSL/TLS configuration is strong' : 'SSL/TLS configuration needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Upgrade to TLS 1.3 for maximum security',
                'Use only strong cipher suites',
                'Enable HSTS for transport security',
                'Configure perfect forward secrecy'
            ]
        };
    }

    checkInputValidation(configuration) {
        const validation = configuration.security?.input_validation || {};
        const findings = [];
        let score = 0;

        // Check if input validation is enabled
        if (validation.enabled) {
            findings.push('Input validation is enabled');
            score += 25;
        } else {
            findings.push('Input validation not configured');
        }

        // Check SQL injection protection
        if (validation.sql_injection_protection) {
            findings.push('SQL injection protection enabled');
            score += 20;
        } else {
            findings.push('SQL injection protection not configured');
        }

        // Check XSS protection
        if (validation.xss_protection) {
            findings.push('XSS protection enabled');
            score += 20;
        } else {
            findings.push('XSS protection not configured');
        }

        // Check CSRF protection
        if (validation.csrf_protection) {
            findings.push('CSRF protection enabled');
            score += 15;
        } else {
            findings.push('CSRF protection not configured');
        }

        // Check file upload validation
        if (validation.file_upload_validation) {
            findings.push('File upload validation enabled');
            score += 10;
        } else {
            findings.push('File upload validation not configured');
        }

        // Check content sanitization
        if (validation.content_sanitization) {
            findings.push('Content sanitization enabled');
            score += 10;
        } else {
            findings.push('Content sanitization not configured');
        }

        const passed = score >= 50;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Input validation is properly configured' : 'Input validation needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Enable comprehensive input validation',
                'Implement SQL injection protection',
                'Configure XSS and CSRF protection',
                'Validate and sanitize all file uploads'
            ]
        };
    }

    checkErrorHandling(configuration) {
        const errorHandling = configuration.security?.error_handling || {};
        const findings = [];
        let score = 0;

        // Check secure error messages
        if (errorHandling.secure_error_messages) {
            findings.push('Secure error messages configured');
            score += 25;
        } else {
            findings.push('Error messages may leak sensitive information');
        }

        // Check error logging
        if (errorHandling.error_logging_enabled) {
            findings.push('Error logging is enabled');
            score += 20;
        } else {
            findings.push('Error logging not configured');
        }

        // Check stack trace hiding
        if (errorHandling.hide_stack_traces) {
            findings.push('Stack traces are hidden from users');
            score += 20;
        } else {
            findings.push('Stack traces may be exposed to users');
        }

        // Check error rate limiting
        if (errorHandling.error_rate_limiting) {
            findings.push('Error rate limiting enabled');
            score += 15;
        } else {
            findings.push('Error rate limiting not configured');
        }

        // Check custom error pages
        if (errorHandling.custom_error_pages) {
            findings.push('Custom error pages configured');
            score += 10;
        }

        // Check security headers on errors
        if (errorHandling.security_headers_on_errors) {
            findings.push('Security headers on error responses');
            score += 10;
        }

        const passed = score >= 45;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Error handling is secure' : 'Error handling needs security improvements',
            findings: findings,
            recommendations: passed ? [] : [
                'Configure secure, non-revealing error messages',
                'Enable comprehensive error logging',
                'Hide stack traces from end users',
                'Implement error rate limiting'
            ]
        };
    }

    checkLoggingAndMonitoring(configuration) {
        const logging = configuration.logging || {};
        const monitoring = configuration.monitoring || {};
        const findings = [];
        let score = 0;

        // Check if logging is enabled
        if (logging.enabled) {
            findings.push('Logging is enabled');
            score += 15;
        } else {
            findings.push('Logging is not enabled');
        }

        // Check log levels
        if (logging.security_events) {
            findings.push('Security events are logged');
            score += 20;
        } else {
            findings.push('Security events logging not configured');
        }

        // Check authentication logging
        if (logging.authentication_events) {
            findings.push('Authentication events are logged');
            score += 15;
        } else {
            findings.push('Authentication logging not configured');
        }

        // Check log integrity
        if (logging.log_integrity_protection) {
            findings.push('Log integrity protection enabled');
            score += 10;
        } else {
            findings.push('Log integrity protection not configured');
        }

        // Check monitoring
        if (monitoring.enabled) {
            findings.push('System monitoring is enabled');
            score += 15;
        } else {
            findings.push('System monitoring not configured');
        }

        // Check alerting
        if (monitoring.security_alerting) {
            findings.push('Security alerting is configured');
            score += 15;
        } else {
            findings.push('Security alerting not configured');
        }

        // Check log retention
        if (logging.retention_policy) {
            findings.push('Log retention policy configured');
            score += 10;
        } else {
            findings.push('Log retention policy not defined');
        }

        const passed = score >= 50;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Logging and monitoring are adequate' : 'Logging and monitoring need improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Enable comprehensive security event logging',
                'Configure authentication and authorization logging',
                'Implement real-time security monitoring',
                'Set up automated security alerting'
            ]
        };
    }

    checkDataMinimization(configuration) {
        const privacy = configuration.privacy || {};
        const findings = [];
        let score = 0;

        // Check data collection policy
        if (privacy.data_minimization_policy) {
            findings.push('Data minimization policy is defined');
            score += 25;
        } else {
            findings.push('No data minimization policy');
        }

        // Check retention periods
        if (privacy.data_retention_periods) {
            findings.push('Data retention periods are defined');
            score += 20;
        } else {
            findings.push('Data retention periods not defined');
        }

        // Check automatic data purging
        if (privacy.automatic_data_purging) {
            findings.push('Automatic data purging is enabled');
            score += 20;
        } else {
            findings.push('Automatic data purging not configured');
        }

        // Check purpose limitation
        if (privacy.purpose_limitation) {
            findings.push('Data usage is limited to specified purposes');
            score += 15;
        } else {
            findings.push('Purpose limitation not implemented');
        }

        // Check data anonymization
        if (privacy.data_anonymization) {
            findings.push('Data anonymization techniques used');
            score += 20;
        } else {
            findings.push('Data anonymization not implemented');
        }

        const passed = score >= 50;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Data minimization practices are adequate' : 'Data minimization needs improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Define and implement data minimization policy',
                'Establish clear data retention periods',
                'Enable automatic data purging',
                'Implement data anonymization techniques'
            ]
        };
    }

    checkPrivacyControls(configuration) {
        const privacy = configuration.privacy || {};
        const findings = [];
        let score = 0;

        // Check user consent mechanisms
        if (privacy.user_consent_system) {
            findings.push('User consent system is implemented');
            score += 20;
        } else {
            findings.push('User consent system not implemented');
        }

        // Check data portability
        if (privacy.data_portability) {
            findings.push('Data portability features available');
            score += 15;
        } else {
            findings.push('Data portability not implemented');
        }

        // Check right to be forgotten
        if (privacy.right_to_be_forgotten) {
            findings.push('Right to be forgotten is implemented');
            score += 20;
        } else {
            findings.push('Right to be forgotten not implemented');
        }

        // Check privacy by design
        if (privacy.privacy_by_design) {
            findings.push('Privacy by design principles followed');
            score += 15;
        } else {
            findings.push('Privacy by design not implemented');
        }

        // Check data access controls
        if (privacy.data_access_controls) {
            findings.push('Data access controls are implemented');
            score += 15;
        } else {
            findings.push('Data access controls not configured');
        }

        // Check privacy notices
        if (privacy.privacy_notices) {
            findings.push('Privacy notices are provided');
            score += 15;
        } else {
            findings.push('Privacy notices not provided');
        }

        const passed = score >= 60;
        
        return {
            passed: passed,
            score: Math.min(100, score),
            message: passed ? 'Privacy controls are adequate' : 'Privacy controls need improvement',
            findings: findings,
            recommendations: passed ? [] : [
                'Implement comprehensive user consent system',
                'Provide data portability features',
                'Enable right to be forgotten functionality',
                'Follow privacy by design principles'
            ]
        };
    }

    // Threat identification and analysis
    identifyThreats(configuration, analysis) {
        const threats = [];
        
        for (const [threatId, threat] of this.threatDatabase) {
            const riskScore = this.assessThreatRisk(threatId, threat, configuration, analysis);
            
            if (riskScore > 30) { // Include threats with medium+ risk
                threats.push({
                    id: threatId,
                    description: threat.description,
                    risk_level: threat.risk_level,
                    calculated_risk_score: riskScore,
                    attack_vectors: threat.attack_vectors,
                    mitigation: threat.mitigation,
                    current_protection: this.assessCurrentProtection(threatId, configuration)
                });
            }
        }
        
        return threats.sort((a, b) => b.calculated_risk_score - a.calculated_risk_score);
    }

    assessThreatRisk(threatId, threat, configuration, analysis) {
        let riskScore = 0;
        
        // Base risk from threat database
        const baseRiskMap = { 'critical': 80, 'high': 60, 'medium': 40, 'low': 20 };
        riskScore = baseRiskMap[threat.risk_level] || 40;
        
        // Adjust based on configuration and analysis results
        switch (threatId) {
            case 'avatar_impersonation':
                if (!configuration.security?.ssl_enabled) riskScore += 20;
                if (configuration.security?.authentication_level === 'basic') riskScore += 15;
                if (!configuration.security?.mfa_enabled) riskScore += 10;
                break;
                
            case 'data_exfiltration':
                if (!configuration.database?.encryption_enabled) riskScore += 25;
                if (!configuration.security?.ssl_enabled) riskScore += 20;
                if (!configuration.logging?.security_events) riskScore += 15;
                break;
                
            case 'ddos_attacks':
                if (!configuration.firewall?.rate_limiting) riskScore += 20;
                if (!configuration.monitoring?.enabled) riskScore += 15;
                break;
                
            case 'privilege_escalation':
                if (configuration.security?.authentication_level === 'basic') riskScore += 20;
                if (!configuration.logging?.authentication_events) riskScore += 15;
                break;
        }
        
        return Math.min(100, riskScore);
    }

    assessCurrentProtection(threatId, configuration) {
        const protections = [];
        
        switch (threatId) {
            case 'avatar_impersonation':
                if (configuration.security?.ssl_enabled) protections.push('SSL/TLS encryption');
                if (configuration.security?.mfa_enabled) protections.push('Multi-factor authentication');
                if (configuration.security?.session?.secure_cookies) protections.push('Secure session management');
                break;
                
            case 'data_exfiltration':
                if (configuration.database?.encryption_enabled) protections.push('Database encryption');
                if (configuration.security?.ssl_enabled) protections.push('Transport encryption');
                if (configuration.logging?.security_events) protections.push('Security event logging');
                break;
                
            case 'ddos_attacks':
                if (configuration.firewall?.rate_limiting) protections.push('Rate limiting');
                if (configuration.monitoring?.enabled) protections.push('System monitoring');
                if (configuration.network?.load_balancing) protections.push('Load balancing');
                break;
        }
        
        return protections;
    }

    // Compliance checking
    checkCompliance(configuration, analysis) {
        const gaps = [];
        const requiredFrameworks = this.getRequiredComplianceFrameworks(configuration);
        
        for (const framework of requiredFrameworks) {
            const frameworkGaps = this.checkFrameworkCompliance(framework, configuration, analysis);
            gaps.push(...frameworkGaps);
        }
        
        return gaps;
    }

    getRequiredComplianceFrameworks(configuration) {
        const frameworks = [];
        
        // Determine required frameworks based on configuration
        if (configuration.deploymentType === 'production' || configuration.deploymentType === 'grid') {
            frameworks.push('OWASP', 'ISO27001');
        }
        
        if (configuration.privacy?.gdpr_required) {
            frameworks.push('GDPR');
        }
        
        if (configuration.payments?.enabled) {
            frameworks.push('PCI-DSS');
        }
        
        if (configuration.compliance?.sox_required) {
            frameworks.push('SOX');
        }
        
        return frameworks;
    }

    checkFrameworkCompliance(frameworkName, configuration, analysis) {
        const framework = this.complianceFrameworks.get(frameworkName);
        if (!framework) return [];
        
        const gaps = [];
        
        // Check specific requirements for each framework
        switch (frameworkName) {
            case 'GDPR':
                gaps.push(...this.checkGDPRCompliance(configuration, analysis));
                break;
            case 'PCI-DSS':
                gaps.push(...this.checkPCIDSSCompliance(configuration, analysis));
                break;
            case 'OWASP':
                gaps.push(...this.checkOWASPCompliance(configuration, analysis));
                break;
            case 'SOX':
                gaps.push(...this.checkSOXCompliance(configuration, analysis));
                break;
        }
        
        return gaps;
    }

    checkGDPRCompliance(configuration, analysis) {
        const gaps = [];
        
        if (!configuration.privacy?.user_consent_system) {
            gaps.push({
                framework: 'GDPR',
                requirement: 'User consent mechanisms',
                description: 'GDPR requires explicit user consent for data processing',
                severity: 'high'
            });
        }
        
        if (!configuration.privacy?.right_to_be_forgotten) {
            gaps.push({
                framework: 'GDPR',
                requirement: 'Right to be forgotten',
                description: 'Users must be able to request data deletion',
                severity: 'high'
            });
        }
        
        if (!configuration.database?.encryption_enabled) {
            gaps.push({
                framework: 'GDPR',
                requirement: 'Data protection by design',
                description: 'Personal data must be protected with appropriate technical measures',
                severity: 'critical'
            });
        }
        
        return gaps;
    }

    checkPCIDSSCompliance(configuration, analysis) {
        const gaps = [];
        
        if (!configuration.security?.ssl_enabled) {
            gaps.push({
                framework: 'PCI-DSS',
                requirement: 'Encrypt transmission of cardholder data',
                description: 'All payment data transmission must be encrypted',
                severity: 'critical'
            });
        }
        
        if (!configuration.firewall?.enabled) {
            gaps.push({
                framework: 'PCI-DSS',
                requirement: 'Install and maintain firewall',
                description: 'Firewall is required to protect cardholder data',
                severity: 'critical'
            });
        }
        
        return gaps;
    }

    checkOWASPCompliance(configuration, analysis) {
        const gaps = [];
        
        if (!configuration.security?.input_validation?.enabled) {
            gaps.push({
                framework: 'OWASP',
                requirement: 'Input validation',
                description: 'All input must be validated to prevent injection attacks',
                severity: 'high'
            });
        }
        
        if (!configuration.security?.input_validation?.xss_protection) {
            gaps.push({
                framework: 'OWASP',
                requirement: 'Cross-site scripting (XSS) protection',
                description: 'XSS protection is required for web applications',
                severity: 'high'
            });
        }
        
        return gaps;
    }

    checkSOXCompliance(configuration, analysis) {
        const gaps = [];
        
        if (!configuration.logging?.enabled) {
            gaps.push({
                framework: 'SOX',
                requirement: 'Audit trail maintenance',
                description: 'SOX requires comprehensive audit trails for financial data',
                severity: 'high'
            });
        }
        
        if (!configuration.backup?.enabled) {
            gaps.push({
                framework: 'SOX',
                requirement: 'Data backup and recovery',
                description: 'SOX requires reliable backup and recovery procedures',
                severity: 'medium'
            });
        }
        
        return gaps;
    }

    // Security recommendations
    generateSecurityRecommendations(analysis) {
        const recommendations = [];
        
        // High-priority recommendations based on critical findings
        const criticalIssues = this.getCriticalSecurityIssues(analysis);
        for (const issue of criticalIssues) {
            recommendations.push({
                priority: 'critical',
                category: issue.category,
                title: `Address Critical Security Issue: ${issue.title}`,
                description: issue.description,
                impact: 'High security risk',
                effort: 'Medium',
                timeline: 'Immediate'
            });
        }
        
        // Compliance-based recommendations
        if (analysis.compliance_gaps.length > 0) {
            recommendations.push({
                priority: 'high',
                category: 'compliance',
                title: 'Address Compliance Gaps',
                description: `${analysis.compliance_gaps.length} compliance gaps identified`,
                impact: 'Regulatory compliance',
                effort: 'High',
                timeline: '30 days'
            });
        }
        
        // Threat-based recommendations
        const highRiskThreats = analysis.threats_identified.filter(t => t.calculated_risk_score > 70);
        if (highRiskThreats.length > 0) {
            recommendations.push({
                priority: 'high',
                category: 'threat_mitigation',
                title: 'Mitigate High-Risk Threats',
                description: `${highRiskThreats.length} high-risk threats require mitigation`,
                impact: 'Security posture improvement',
                effort: 'Medium',
                timeline: '14 days'
            });
        }
        
        // Security profile alignment
        const profileRecommendations = this.getSecurityProfileRecommendations(analysis);
        recommendations.push(...profileRecommendations);
        
        return recommendations.sort((a, b) => {
            const priorityOrder = { 'critical': 0, 'high': 1, 'medium': 2, 'low': 3 };
            return priorityOrder[a.priority] - priorityOrder[b.priority];
        });
    }

    getCriticalSecurityIssues(analysis) {
        const criticalIssues = [];
        
        for (const [categoryName, category] of Object.entries(analysis.categories)) {
            for (const finding of category.findings) {
                if (finding.severity === 'critical' && !finding.result.passed) {
                    criticalIssues.push({
                        category: categoryName,
                        title: finding.rule,
                        description: finding.result.message,
                        rule: finding.rule
                    });
                }
            }
        }
        
        return criticalIssues;
    }

    getSecurityProfileRecommendations(analysis) {
        const profile = this.securityProfiles.get(analysis.security_profile);
        if (!profile) return [];
        
        const recommendations = [];
        
        // Check if current configuration meets profile requirements
        if (profile.requirements.encryption === 'required' && analysis.categories.data_protection.score < 60) {
            recommendations.push({
                priority: 'high',
                category: 'data_protection',
                title: 'Implement Required Encryption',
                description: `${profile.name} requires comprehensive encryption`,
                impact: 'Compliance with security profile',
                effort: 'Medium',
                timeline: '7 days'
            });
        }
        
        if (profile.requirements.monitoring === 'real-time' && analysis.categories.monitoring.score < 70) {
            recommendations.push({
                priority: 'medium',
                category: 'monitoring',
                title: 'Enhance Monitoring Capabilities',
                description: `${profile.name} requires real-time monitoring`,
                impact: 'Improved threat detection',
                effort: 'Medium',
                timeline: '14 days'
            });
        }
        
        return recommendations;
    }

    // Remediation planning
    createRemediationPlan(analysis) {
        const plan = {
            immediate_actions: [],
            short_term_goals: [],
            long_term_objectives: [],
            resource_requirements: {
                technical_expertise: [],
                tools_and_software: [],
                estimated_cost: 'TBD',
                estimated_timeline: 'TBD'
            }
        };
        
        // Immediate actions (0-7 days)
        const criticalIssues = this.getCriticalSecurityIssues(analysis);
        for (const issue of criticalIssues) {
            plan.immediate_actions.push({
                action: `Fix ${issue.title}`,
                description: issue.description,
                priority: 'critical',
                timeline: '24-48 hours'
            });
        }
        
        // Short-term goals (1-4 weeks)
        const highPriorityRecommendations = analysis.security_recommendations.filter(r => r.priority === 'high');
        for (const rec of highPriorityRecommendations) {
            plan.short_term_goals.push({
                goal: rec.title,
                description: rec.description,
                timeline: rec.timeline || '2 weeks'
            });
        }
        
        // Long-term objectives (1-6 months)
        if (analysis.compliance_gaps.length > 0) {
            plan.long_term_objectives.push({
                objective: 'Achieve Full Compliance',
                description: 'Address all compliance gaps and maintain compliance',
                timeline: '3-6 months'
            });
        }
        
        if (analysis.overall_score < 80) {
            plan.long_term_objectives.push({
                objective: 'Achieve Security Excellence',
                description: 'Reach 80%+ security score across all categories',
                timeline: '6 months'
            });
        }
        
        // Resource requirements
        plan.resource_requirements = this.calculateResourceRequirements(analysis);
        
        return plan;
    }

    calculateResourceRequirements(analysis) {
        const requirements = {
            technical_expertise: [],
            tools_and_software: [],
            estimated_cost: 'Low to Medium',
            estimated_timeline: '2-12 weeks'
        };
        
        // Determine required expertise
        const criticalIssues = this.getCriticalSecurityIssues(analysis);
        if (criticalIssues.some(i => i.category === 'data_protection')) {
            requirements.technical_expertise.push('Encryption and Data Protection Specialist');
        }
        if (criticalIssues.some(i => i.category === 'network_security')) {
            requirements.technical_expertise.push('Network Security Engineer');
        }
        if (analysis.compliance_gaps.length > 0) {
            requirements.technical_expertise.push('Compliance and Risk Management Specialist');
        }
        
        // Determine required tools
        if (analysis.categories.monitoring.score < 50) {
            requirements.tools_and_software.push('Security Information and Event Management (SIEM) System');
        }
        if (analysis.categories.data_protection.score < 50) {
            requirements.tools_and_software.push('Encryption and Key Management Solution');
        }
        if (analysis.categories.network_security.score < 50) {
            requirements.tools_and_software.push('Web Application Firewall (WAF)');
        }
        
        return requirements;
    }

    calculateSecurityScore(analysis) {
        const categoryScores = Object.values(analysis.categories).map(cat => cat.score);
        const averageScore = categoryScores.reduce((sum, score) => sum + score, 0) / categoryScores.length;
        
        analysis.overall_score = Math.round(averageScore);
        
        // Determine risk level
        if (analysis.overall_score >= 80) {
            analysis.risk_level = 'low';
        } else if (analysis.overall_score >= 60) {
            analysis.risk_level = 'medium';
        } else if (analysis.overall_score >= 40) {
            analysis.risk_level = 'high';
        } else {
            analysis.risk_level = 'critical';
        }
    }

    // Export and reporting
    exportSecurityReport(analysis, format = 'json') {
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `opensim-security-analysis-${timestamp}.${format}`;

        let content, mimeType;

        switch (format) {
            case 'json':
                content = JSON.stringify(analysis, null, 2);
                mimeType = 'application/json';
                break;

            case 'html':
                content = this.generateHTMLSecurityReport(analysis);
                mimeType = 'text/html';
                break;

            case 'pdf':
                // PDF generation would require additional libraries
                throw new Error('PDF export not yet implemented');

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

    generateHTMLSecurityReport(analysis) {
        const riskColorMap = {
            'low': '#28a745',
            'medium': '#ffc107', 
            'high': '#fd7e14',
            'critical': '#dc3545'
        };

        return `
            <!DOCTYPE html>
            <html>
            <head>
                <title>OpenSim Next Security Analysis Report</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; }
                    .header { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
                    .score { font-size: 2em; font-weight: bold; color: ${riskColorMap[analysis.risk_level]}; }
                    .risk-level { 
                        display: inline-block; 
                        padding: 5px 15px; 
                        border-radius: 20px; 
                        color: white; 
                        background: ${riskColorMap[analysis.risk_level]};
                        text-transform: uppercase;
                    }
                    .category { margin: 20px 0; border: 1px solid #ddd; border-radius: 6px; }
                    .category-header { background: #f8f9fa; padding: 15px; border-bottom: 1px solid #ddd; }
                    .category-content { padding: 15px; }
                    .finding { margin: 10px 0; padding: 10px; border-left: 4px solid #ccc; }
                    .severity-critical { border-left-color: #dc3545; background: #f8d7da; }
                    .severity-high { border-left-color: #fd7e14; background: #ffeaa7; }
                    .severity-medium { border-left-color: #ffc107; background: #fff3cd; }
                    .threat { margin: 10px 0; padding: 15px; border: 1px solid #ddd; border-radius: 6px; }
                    .compliance-gap { margin: 10px 0; padding: 10px; background: #ffeaa7; border-radius: 6px; }
                    .recommendation { margin: 10px 0; padding: 15px; background: #e3f2fd; border-radius: 6px; }
                </style>
            </head>
            <body>
                <div class="header">
                    <h1>OpenSim Next Security Analysis Report</h1>
                    <p><strong>Configuration ID:</strong> ${analysis.configuration_id}</p>
                    <p><strong>Deployment Type:</strong> ${analysis.deployment_type}</p>
                    <p><strong>Security Profile:</strong> ${analysis.security_profile}</p>
                    <p><strong>Analysis Date:</strong> ${new Date(analysis.timestamp).toLocaleString()}</p>
                    <div class="score">Security Score: ${analysis.overall_score}%</div>
                    <span class="risk-level">${analysis.risk_level} Risk</span>
                </div>

                <h2>Category Breakdown</h2>
                ${Object.entries(analysis.categories).map(([categoryName, category]) => `
                    <div class="category">
                        <div class="category-header">
                            <h3>${categoryName.replace('_', ' ').toUpperCase()}</h3>
                            <strong>Score: ${Math.round(category.score)}%</strong>
                        </div>
                        <div class="category-content">
                            ${category.findings.map(finding => `
                                <div class="finding severity-${finding.severity}">
                                    <strong>${finding.rule}:</strong> ${finding.result.message}
                                    <div><small>Severity: ${finding.severity}</small></div>
                                </div>
                            `).join('')}
                        </div>
                    </div>
                `).join('')}

                ${analysis.threats_identified.length > 0 ? `
                    <h2>Identified Threats</h2>
                    ${analysis.threats_identified.map(threat => `
                        <div class="threat">
                            <h4>${threat.id.replace('_', ' ').toUpperCase()}</h4>
                            <p><strong>Risk Score:</strong> ${threat.calculated_risk_score}%</p>
                            <p><strong>Description:</strong> ${threat.description}</p>
                            <p><strong>Current Protection:</strong> ${threat.current_protection.join(', ') || 'None'}</p>
                        </div>
                    `).join('')}
                ` : ''}

                ${analysis.compliance_gaps.length > 0 ? `
                    <h2>Compliance Gaps</h2>
                    ${analysis.compliance_gaps.map(gap => `
                        <div class="compliance-gap">
                            <strong>${gap.framework} - ${gap.requirement}:</strong> ${gap.description}
                            <div><small>Severity: ${gap.severity}</small></div>
                        </div>
                    `).join('')}
                ` : ''}

                ${analysis.security_recommendations.length > 0 ? `
                    <h2>Security Recommendations</h2>
                    ${analysis.security_recommendations.map(rec => `
                        <div class="recommendation">
                            <h4>${rec.title}</h4>
                            <p>${rec.description}</p>
                            <p><strong>Priority:</strong> ${rec.priority}</p>
                            <p><strong>Timeline:</strong> ${rec.timeline || 'TBD'}</p>
                        </div>
                    `).join('')}
                ` : ''}

                <h2>Remediation Plan</h2>
                ${analysis.remediation_plan.immediate_actions.length > 0 ? `
                    <h3>Immediate Actions (0-7 days)</h3>
                    <ul>
                        ${analysis.remediation_plan.immediate_actions.map(action => `
                            <li><strong>${action.action}:</strong> ${action.description}</li>
                        `).join('')}
                    </ul>
                ` : ''}

                ${analysis.remediation_plan.short_term_goals.length > 0 ? `
                    <h3>Short-term Goals (1-4 weeks)</h3>
                    <ul>
                        ${analysis.remediation_plan.short_term_goals.map(goal => `
                            <li><strong>${goal.goal}:</strong> ${goal.description}</li>
                        `).join('')}
                    </ul>
                ` : ''}

                ${analysis.remediation_plan.long_term_objectives.length > 0 ? `
                    <h3>Long-term Objectives (1-6 months)</h3>
                    <ul>
                        ${analysis.remediation_plan.long_term_objectives.map(obj => `
                            <li><strong>${obj.objective}:</strong> ${obj.description}</li>
                        `).join('')}
                    </ul>
                ` : ''}
            </body>
            </html>
        `;
    }
}

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.SecurityAnalyzer = SecurityAnalyzer;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { SecurityAnalyzer };
}