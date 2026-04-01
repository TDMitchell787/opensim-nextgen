// OpenSim Next Auto-Configurator - Configuration Dashboard
// Comprehensive dashboard with configuration completeness tracking and progress indicators

class ConfigurationDashboard {
    constructor() {
        this.configState = new Map();
        this.completionRequirements = new Map();
        this.progressMetrics = new Map();
        this.milestones = new Map();
        this.notifications = [];
        
        // Dashboard update settings
        this.updateInterval = 1000; // 1 second
        this.lastUpdateTime = 0;
        this.isActive = false;
        
        this.initializeDashboard();
    }

    initializeDashboard() {
        this.defineCompletionRequirements();
        this.createDashboardInterface();
        this.setupEventListeners();
        this.startProgressTracking();
        this.loadSavedState();
    }

    defineCompletionRequirements() {
        // General Configuration Requirements
        this.completionRequirements.set('general', {
            name: 'General Configuration',
            weight: 15,
            critical: true,
            requirements: [
                { id: 'gridName', name: 'Grid Name', required: true, weight: 3 },
                { id: 'gridNick', name: 'Grid Nickname', required: true, weight: 2 },
                { id: 'welcomeMessage', name: 'Welcome Message', required: false, weight: 1 },
                { id: 'deploymentType', name: 'Deployment Type', required: true, weight: 5 },
                { id: 'instanceName', name: 'Instance Name', required: true, weight: 2 },
                { id: 'adminEmail', name: 'Administrator Email', required: true, weight: 2 }
            ]
        });

        // Network Configuration Requirements
        this.completionRequirements.set('network', {
            name: 'Network Configuration',
            weight: 20,
            critical: true,
            requirements: [
                { id: 'httpPort', name: 'HTTP Port', required: true, weight: 3 },
                { id: 'httpsPort', name: 'HTTPS Port', required: false, weight: 2 },
                { id: 'httpsEnabled', name: 'HTTPS Configuration', required: false, weight: 3 },
                { id: 'externalHostname', name: 'External Hostname', required: true, weight: 5 },
                { id: 'internalIp', name: 'Internal IP Address', required: true, weight: 3 },
                { id: 'websocketPort', name: 'WebSocket Port', required: false, weight: 2 },
                { id: 'maxConnections', name: 'Connection Limits', required: true, weight: 2 }
            ]
        });

        // Database Configuration Requirements
        this.completionRequirements.set('database', {
            name: 'Database Configuration',
            weight: 25,
            critical: true,
            requirements: [
                { id: 'type', name: 'Database Type', required: true, weight: 5 },
                { id: 'host', name: 'Database Host', required: 'conditional', weight: 4 },
                { id: 'port', name: 'Database Port', required: 'conditional', weight: 2 },
                { id: 'name', name: 'Database Name', required: true, weight: 4 },
                { id: 'username', name: 'Database Username', required: 'conditional', weight: 3 },
                { id: 'password', name: 'Database Password', required: 'conditional', weight: 3 },
                { id: 'poolSize', name: 'Connection Pool Size', required: true, weight: 2 },
                { id: 'connectionTest', name: 'Connection Test', required: false, weight: 2 }
            ]
        });

        // Security Configuration Requirements
        this.completionRequirements.set('security', {
            name: 'Security Configuration',
            weight: 20,
            critical: true,
            requirements: [
                { id: 'apiKey', name: 'API Key', required: true, weight: 5 },
                { id: 'passwordComplexity', name: 'Password Policy', required: true, weight: 2 },
                { id: 'sessionTimeout', name: 'Session Timeout', required: true, weight: 2 },
                { id: 'bruteForceProtection', name: 'Brute Force Protection', required: true, weight: 2 },
                { id: 'sslCertificate', name: 'SSL Certificate', required: 'conditional', weight: 3 },
                { id: 'rateLimiting', name: 'Rate Limiting', required: true, weight: 2 },
                { id: 'encryptionKeys', name: 'Encryption Keys', required: false, weight: 2 },
                { id: 'securityAudit', name: 'Security Audit', required: false, weight: 2 }
            ]
        });

        // Physics Configuration Requirements
        this.completionRequirements.set('physics', {
            name: 'Physics Configuration',
            weight: 10,
            critical: false,
            requirements: [
                { id: 'defaultEngine', name: 'Default Physics Engine', required: true, weight: 4 },
                { id: 'timestep', name: 'Physics Timestep', required: true, weight: 2 },
                { id: 'maxBodies', name: 'Maximum Bodies', required: true, weight: 2 },
                { id: 'gravitySettings', name: 'Gravity Configuration', required: true, weight: 1 },
                { id: 'collisionSettings', name: 'Collision Settings', required: false, weight: 1 }
            ]
        });

        // Regions Configuration Requirements
        this.completionRequirements.set('regions', {
            name: 'Regions Configuration',
            weight: 15,
            critical: true,
            requirements: [
                { id: 'regionCount', name: 'Number of Regions', required: true, weight: 3 },
                { id: 'regionNames', name: 'Region Names', required: true, weight: 3 },
                { id: 'regionCoordinates', name: 'Region Coordinates', required: true, weight: 3 },
                { id: 'regionSizes', name: 'Region Sizes', required: true, weight: 2 },
                { id: 'regionPhysics', name: 'Per-Region Physics', required: false, weight: 2 },
                { id: 'regionLimits', name: 'Region Limits', required: true, weight: 2 }
            ]
        });

        // Performance Configuration Requirements
        this.completionRequirements.set('performance', {
            name: 'Performance Configuration',
            weight: 10,
            critical: false,
            requirements: [
                { id: 'maxPrims', name: 'Maximum Prims', required: true, weight: 2 },
                { id: 'maxScripts', name: 'Maximum Scripts', required: true, weight: 2 },
                { id: 'scriptTimeout', name: 'Script Timeout', required: true, weight: 1 },
                { id: 'cacheSettings', name: 'Cache Configuration', required: true, weight: 2 },
                { id: 'threadPoolSize', name: 'Thread Pool Size', required: true, weight: 1 },
                { id: 'memoryLimits', name: 'Memory Limits', required: false, weight: 1 },
                { id: 'compressionSettings', name: 'Compression Settings', required: false, weight: 1 }
            ]
        });

        // Define configuration milestones
        this.defineMilestones();
    }

    defineMilestones() {
        this.milestones.set('basic_setup', {
            name: 'Basic Setup Complete',
            description: 'Essential configuration sections completed',
            threshold: 40,
            requirements: ['general', 'network'],
            icon: 'check-circle',
            color: '#4caf50'
        });

        this.milestones.set('core_services', {
            name: 'Core Services Configured',
            description: 'Database and security properly configured',
            threshold: 65,
            requirements: ['general', 'network', 'database', 'security'],
            icon: 'shield-check',
            color: '#2196f3'
        });

        this.milestones.set('world_ready', {
            name: 'Virtual World Ready',
            description: 'Regions and physics configured for deployment',
            threshold: 85,
            requirements: ['general', 'network', 'database', 'security', 'regions', 'physics'],
            icon: 'world',
            color: '#ff9800'
        });

        this.milestones.set('production_ready', {
            name: 'Production Ready',
            description: 'All configuration sections completed and optimized',
            threshold: 100,
            requirements: ['general', 'network', 'database', 'security', 'regions', 'physics', 'performance'],
            icon: 'rocket',
            color: '#9c27b0'
        });
    }

    createDashboardInterface() {
        // Check if dashboard container already exists
        let dashboardContainer = document.getElementById('dashboard-enhanced');
        if (!dashboardContainer) {
            dashboardContainer = document.createElement('div');
            dashboardContainer.id = 'dashboard-enhanced';
            dashboardContainer.className = 'dashboard-enhanced';
            
            // Insert after existing dashboard or validation panel
            const insertPoint = document.getElementById('dashboard') || 
                               document.getElementById('validation-panel') || 
                               document.querySelector('.main .container');
            insertPoint.parentNode.insertBefore(dashboardContainer, insertPoint.nextSibling);
        }

        dashboardContainer.innerHTML = `
            <div class="dashboard-header">
                <div class="dashboard-title">
                    <h2>Configuration Progress Dashboard</h2>
                    <p>Track your OpenSim Next configuration completion and get intelligent guidance</p>
                </div>
                <div class="dashboard-controls">
                    <button class="btn btn-secondary" id="refresh-dashboard" title="Refresh dashboard">
                        <i class="fas fa-sync-alt"></i> Refresh
                    </button>
                    <button class="btn btn-secondary" id="export-progress" title="Export progress report">
                        <i class="fas fa-download"></i> Export Report
                    </button>
                    <button class="btn btn-primary" id="next-action" disabled title="Continue to next step">
                        <i class="fas fa-arrow-right"></i> <span id="next-action-text">Next Step</span>
                    </button>
                </div>
            </div>

            <div class="dashboard-summary">
                <div class="progress-overview">
                    <div class="overall-progress">
                        <div class="progress-circle" id="overall-progress-circle">
                            <svg class="progress-svg" viewBox="0 0 120 120">
                                <circle class="progress-bg" cx="60" cy="60" r="50"></circle>
                                <circle class="progress-fill" id="progress-circle-fill" cx="60" cy="60" r="50"></circle>
                            </svg>
                            <div class="progress-text">
                                <span class="progress-percentage" id="overall-percentage">0%</span>
                                <span class="progress-label">Complete</span>
                            </div>
                        </div>
                        <div class="progress-details">
                            <h3>Overall Progress</h3>
                            <div class="progress-stats">
                                <div class="stat-item">
                                    <span class="stat-value" id="completed-sections">0</span>
                                    <span class="stat-label">Sections Complete</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-value" id="critical-remaining">0</span>
                                    <span class="stat-label">Critical Items Remaining</span>
                                </div>
                                <div class="stat-item">
                                    <span class="stat-value" id="estimated-time">--</span>
                                    <span class="stat-label">Est. Time Remaining</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="milestone-tracker">
                        <h4>Configuration Milestones</h4>
                        <div class="milestones-list" id="milestones-list">
                            <!-- Milestones will be populated here -->
                        </div>
                    </div>
                </div>

                <div class="quick-insights">
                    <h4>Quick Insights</h4>
                    <div class="insights-grid" id="insights-grid">
                        <!-- Insights will be populated here -->
                    </div>
                </div>
            </div>

            <div class="dashboard-sections">
                <div class="sections-header">
                    <h3>Configuration Sections</h3>
                    <div class="view-controls">
                        <button class="view-btn active" data-view="grid" title="Grid view">
                            <i class="fas fa-th"></i>
                        </button>
                        <button class="view-btn" data-view="list" title="List view">
                            <i class="fas fa-list"></i>
                        </button>
                        <button class="view-btn" data-view="timeline" title="Timeline view">
                            <i class="fas fa-timeline"></i>
                        </button>
                    </div>
                </div>
                <div class="sections-content" id="sections-content">
                    <!-- Section cards will be populated here -->
                </div>
            </div>

            <div class="dashboard-recommendations">
                <h3>Intelligent Recommendations</h3>
                <div class="recommendations-list" id="recommendations-list">
                    <!-- Recommendations will be populated here -->
                </div>
            </div>

            <div class="dashboard-actions">
                <div class="action-buttons">
                    <button class="btn btn-outline" id="save-progress">
                        <i class="fas fa-save"></i> Save Progress
                    </button>
                    <button class="btn btn-outline" id="load-template">
                        <i class="fas fa-template"></i> Load Template
                    </button>
                    <button class="btn btn-warning" id="reset-progress">
                        <i class="fas fa-undo"></i> Reset Progress
                    </button>
                </div>
                <div class="deployment-readiness" id="deployment-readiness">
                    <!-- Deployment readiness will be shown here -->
                </div>
            </div>
        `;
    }

    setupEventListeners() {
        // Dashboard controls
        document.getElementById('refresh-dashboard').addEventListener('click', () => {
            this.refreshDashboard();
        });

        document.getElementById('export-progress').addEventListener('click', () => {
            this.exportProgressReport();
        });

        document.getElementById('next-action').addEventListener('click', () => {
            this.performNextAction();
        });

        // View controls
        document.querySelectorAll('.view-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                this.switchView(e.target.dataset.view);
            });
        });

        // Action buttons
        document.getElementById('save-progress').addEventListener('click', () => {
            this.saveProgress();
        });

        document.getElementById('load-template').addEventListener('click', () => {
            this.showTemplateSelector();
        });

        document.getElementById('reset-progress').addEventListener('click', () => {
            this.showResetConfirmation();
        });

        // Listen for configuration changes
        document.addEventListener('configurationChanged', (e) => {
            this.handleConfigurationChange(e.detail);
        });

        document.addEventListener('previewUpdated', (e) => {
            this.handlePreviewUpdate(e.detail);
        });

        document.addEventListener('validationCompleted', (e) => {
            this.handleValidationUpdate(e.detail);
        });
    }

    startProgressTracking() {
        this.isActive = true;
        this.updateDashboard();
        
        // Set up periodic updates
        setInterval(() => {
            if (this.isActive) {
                this.updateDashboard();
            }
        }, this.updateInterval);
    }

    updateDashboard() {
        try {
            // Capture current configuration state
            const currentConfig = this.captureConfigurationState();
            
            // Calculate completion metrics
            const metrics = this.calculateCompletionMetrics(currentConfig);
            
            // Update progress indicators
            this.updateProgressIndicators(metrics);
            
            // Update section cards
            this.updateSectionCards(metrics);
            
            // Update milestones
            this.updateMilestones(metrics);
            
            // Generate recommendations
            this.updateRecommendations(metrics);
            
            // Update deployment readiness
            this.updateDeploymentReadiness(metrics);
            
            // Update quick insights
            this.updateQuickInsights(metrics);
            
            // Update next action
            this.updateNextAction(metrics);
            
            this.lastUpdateTime = Date.now();
            
        } catch (error) {
            console.error('Dashboard update error:', error);
        }
    }

    captureConfigurationState() {
        const config = {
            general: this.captureGeneralState(),
            network: this.captureNetworkState(),
            database: this.captureDatabaseState(),
            security: this.captureSecurityState(),
            physics: this.capturePhysicsState(),
            regions: this.captureRegionsState(),
            performance: this.capturePerformanceState()
        };

        // Store in configState for comparison
        this.configState.set('current', config);
        return config;
    }

    captureGeneralState() {
        return {
            gridName: this.getFieldValue('grid-name'),
            gridNick: this.getFieldValue('grid-nick'),
            welcomeMessage: this.getFieldValue('welcome-message'),
            deploymentType: this.getFieldValue('deployment-type'),
            instanceName: this.getFieldValue('instance-name'),
            adminEmail: this.getFieldValue('admin-email')
        };
    }

    captureNetworkState() {
        return {
            httpPort: this.getFieldValue('http-port'),
            httpsPort: this.getFieldValue('https-port'),
            httpsEnabled: this.getFieldValue('https-enabled', false, 'checkbox'),
            externalHostname: this.getFieldValue('external-hostname'),
            internalIp: this.getFieldValue('internal-ip'),
            websocketPort: this.getFieldValue('websocket-port'),
            maxConnections: this.getFieldValue('max-connections')
        };
    }

    captureDatabaseState() {
        return {
            type: this.getFieldValue('database-type'),
            host: this.getFieldValue('database-host'),
            port: this.getFieldValue('database-port'),
            name: this.getFieldValue('database-name'),
            username: this.getFieldValue('database-username'),
            password: this.getFieldValue('database-password'),
            poolSize: this.getFieldValue('database-pool-size'),
            connectionTest: this.getFieldValue('database-connection-tested', false, 'data')
        };
    }

    captureSecurityState() {
        return {
            apiKey: this.getFieldValue('api-key'),
            passwordComplexity: this.getFieldValue('password-complexity', false, 'checkbox'),
            sessionTimeout: this.getFieldValue('session-timeout'),
            bruteForceProtection: this.getFieldValue('brute-force-protection', false, 'checkbox'),
            sslCertificate: this.getFieldValue('ssl-certificate-path'),
            rateLimiting: this.getFieldValue('rate-limit-enabled', false, 'checkbox'),
            encryptionKeys: this.getFieldValue('encryption-keys-generated', false, 'data'),
            securityAudit: this.getFieldValue('security-audit-completed', false, 'data')
        };
    }

    capturePhysicsState() {
        return {
            defaultEngine: this.getFieldValue('physics-engine'),
            timestep: this.getFieldValue('physics-timestep'),
            maxBodies: this.getFieldValue('physics-max-bodies'),
            gravitySettings: this.getFieldValue('gravity-configured', false, 'data'),
            collisionSettings: this.getFieldValue('collision-configured', false, 'data')
        };
    }

    captureRegionsState() {
        const regionElements = document.querySelectorAll('.region-config');
        return {
            regionCount: regionElements.length,
            regionNames: Array.from(regionElements).map(el => this.getFieldValue(`region-${el.dataset.regionId}-name`)),
            regionCoordinates: Array.from(regionElements).map(el => ({
                x: this.getFieldValue(`region-${el.dataset.regionId}-x`),
                y: this.getFieldValue(`region-${el.dataset.regionId}-y`)
            })),
            regionSizes: Array.from(regionElements).map(el => ({
                x: this.getFieldValue(`region-${el.dataset.regionId}-size-x`),
                y: this.getFieldValue(`region-${el.dataset.regionId}-size-y`)
            })),
            regionPhysics: Array.from(regionElements).map(el => this.getFieldValue(`region-${el.dataset.regionId}-physics`)),
            regionLimits: Array.from(regionElements).map(el => this.getFieldValue(`region-${el.dataset.regionId}-max-prims`))
        };
    }

    capturePerformanceState() {
        return {
            maxPrims: this.getFieldValue('max-prims'),
            maxScripts: this.getFieldValue('max-scripts'),
            scriptTimeout: this.getFieldValue('script-timeout'),
            cacheSettings: this.getFieldValue('cache-assets', false, 'checkbox'),
            threadPoolSize: this.getFieldValue('thread-pool-size'),
            memoryLimits: this.getFieldValue('memory-limits-configured', false, 'data'),
            compressionSettings: this.getFieldValue('enable-gzip', false, 'checkbox')
        };
    }

    calculateCompletionMetrics(config) {
        const metrics = {
            overall: 0,
            sections: new Map(),
            critical: 0,
            optional: 0,
            errors: 0,
            warnings: 0,
            readiness: 'not-ready'
        };

        let totalWeight = 0;
        let completedWeight = 0;
        let criticalCompleted = 0;
        let criticalTotal = 0;

        for (const [sectionId, requirements] of this.completionRequirements.entries()) {
            const sectionConfig = config[sectionId];
            const sectionMetrics = this.calculateSectionMetrics(sectionConfig, requirements);
            
            metrics.sections.set(sectionId, sectionMetrics);
            
            totalWeight += requirements.weight;
            completedWeight += (sectionMetrics.completion / 100) * requirements.weight;
            
            if (requirements.critical) {
                criticalTotal += requirements.weight;
                if (sectionMetrics.completion >= 80) {
                    criticalCompleted += requirements.weight;
                }
            }
            
            metrics.errors += sectionMetrics.errors;
            metrics.warnings += sectionMetrics.warnings;
        }

        metrics.overall = totalWeight > 0 ? Math.round((completedWeight / totalWeight) * 100) : 0;
        metrics.critical = criticalTotal > 0 ? Math.round((criticalCompleted / criticalTotal) * 100) : 0;

        // Determine readiness level
        if (metrics.overall >= 100 && metrics.errors === 0) {
            metrics.readiness = 'production-ready';
        } else if (metrics.critical >= 100 && metrics.errors === 0) {
            metrics.readiness = 'deployment-ready';
        } else if (metrics.overall >= 60) {
            metrics.readiness = 'testing-ready';
        } else if (metrics.overall >= 30) {
            metrics.readiness = 'development-ready';
        }

        return metrics;
    }

    calculateSectionMetrics(sectionConfig, requirements) {
        const metrics = {
            completion: 0,
            completedItems: 0,
            totalItems: requirements.requirements.length,
            errors: 0,
            warnings: 0,
            issues: []
        };

        let completedWeight = 0;
        let totalWeight = 0;

        for (const requirement of requirements.requirements) {
            totalWeight += requirement.weight;
            
            const value = sectionConfig?.[requirement.id];
            const isCompleted = this.isRequirementCompleted(value, requirement, sectionConfig);
            
            if (isCompleted) {
                completedWeight += requirement.weight;
                metrics.completedItems++;
            } else if (requirement.required === true || 
                      (requirement.required === 'conditional' && this.isConditionallyRequired(requirement, sectionConfig))) {
                metrics.errors++;
                metrics.issues.push({
                    type: 'error',
                    requirement: requirement.id,
                    message: `${requirement.name} is required but not configured`
                });
            } else if (requirement.required === false) {
                metrics.warnings++;
                metrics.issues.push({
                    type: 'warning',
                    requirement: requirement.id,
                    message: `${requirement.name} is recommended but not configured`
                });
            }
        }

        metrics.completion = totalWeight > 0 ? Math.round((completedWeight / totalWeight) * 100) : 0;
        return metrics;
    }

    isRequirementCompleted(value, requirement, sectionConfig) {
        if (value === null || value === undefined || value === '') {
            return false;
        }

        // Special validation for specific requirements
        switch (requirement.id) {
            case 'apiKey':
                return value !== 'default-key-change-me' && value.length >= 16;
            case 'connectionTest':
                return value === true;
            case 'regionCount':
                return value > 0;
            case 'sslCertificate':
                return sectionConfig?.httpsEnabled ? value !== '' : true;
            default:
                return true;
        }
    }

    isConditionallyRequired(requirement, sectionConfig) {
        switch (requirement.id) {
            case 'host':
            case 'port':
            case 'username':
            case 'password':
                return sectionConfig?.type !== 'sqlite';
            case 'sslCertificate':
                return sectionConfig?.httpsEnabled === true;
            default:
                return false;
        }
    }

    updateProgressIndicators(metrics) {
        // Update overall progress circle
        const progressFill = document.getElementById('progress-circle-fill');
        const percentage = document.getElementById('overall-percentage');
        
        if (progressFill && percentage) {
            const circumference = 2 * Math.PI * 50;
            const offset = circumference - (metrics.overall / 100) * circumference;
            progressFill.style.strokeDasharray = circumference;
            progressFill.style.strokeDashoffset = offset;
            percentage.textContent = `${metrics.overall}%`;
        }

        // Update stats
        const completedSections = document.getElementById('completed-sections');
        const criticalRemaining = document.getElementById('critical-remaining');
        const estimatedTime = document.getElementById('estimated-time');

        if (completedSections) {
            const completed = Array.from(metrics.sections.values()).filter(s => s.completion >= 80).length;
            completedSections.textContent = `${completed}/${metrics.sections.size}`;
        }

        if (criticalRemaining) {
            const criticalSections = Array.from(this.completionRequirements.values()).filter(r => r.critical).length;
            const criticalCompleted = Array.from(metrics.sections.entries())
                .filter(([id, metrics]) => this.completionRequirements.get(id).critical && metrics.completion >= 80).length;
            criticalRemaining.textContent = criticalSections - criticalCompleted;
        }

        if (estimatedTime) {
            const remainingItems = Array.from(metrics.sections.values())
                .reduce((sum, section) => sum + (section.totalItems - section.completedItems), 0);
            const estimatedMinutes = remainingItems * 2; // 2 minutes per item estimate
            estimatedTime.textContent = this.formatEstimatedTime(estimatedMinutes);
        }
    }

    updateSectionCards(metrics) {
        const sectionsContent = document.getElementById('sections-content');
        if (!sectionsContent) return;

        const currentView = document.querySelector('.view-btn.active')?.dataset.view || 'grid';
        sectionsContent.className = `sections-content view-${currentView}`;

        let html = '';
        
        for (const [sectionId, sectionMetrics] of metrics.sections.entries()) {
            const requirements = this.completionRequirements.get(sectionId);
            const statusClass = this.getSectionStatusClass(sectionMetrics);
            const statusIcon = this.getSectionStatusIcon(sectionMetrics);
            
            html += `
                <div class="section-card ${statusClass}" data-section="${sectionId}">
                    <div class="section-header">
                        <div class="section-icon">
                            <i class="fas ${statusIcon}"></i>
                        </div>
                        <div class="section-info">
                            <h4>${requirements.name}</h4>
                            <div class="section-progress">
                                <div class="progress-bar">
                                    <div class="progress-fill" style="width: ${sectionMetrics.completion}%"></div>
                                </div>
                                <span class="progress-text">${sectionMetrics.completion}%</span>
                            </div>
                        </div>
                        <div class="section-status">
                            ${requirements.critical ? '<span class="critical-badge">Critical</span>' : ''}
                            <span class="weight-badge">Weight: ${requirements.weight}</span>
                        </div>
                    </div>
                    
                    <div class="section-details">
                        <div class="completion-stats">
                            <div class="stat">
                                <span class="stat-value">${sectionMetrics.completedItems}</span>
                                <span class="stat-label">of ${sectionMetrics.totalItems} items</span>
                            </div>
                            <div class="stat">
                                <span class="stat-value ${sectionMetrics.errors > 0 ? 'error' : ''}">${sectionMetrics.errors}</span>
                                <span class="stat-label">errors</span>
                            </div>
                            <div class="stat">
                                <span class="stat-value ${sectionMetrics.warnings > 0 ? 'warning' : ''}">${sectionMetrics.warnings}</span>
                                <span class="stat-label">warnings</span>
                            </div>
                        </div>
                        
                        ${sectionMetrics.issues.length > 0 ? `
                            <div class="section-issues">
                                <h5>Issues:</h5>
                                <ul>
                                    ${sectionMetrics.issues.slice(0, 3).map(issue => `
                                        <li class="issue-${issue.type}">${issue.message}</li>
                                    `).join('')}
                                    ${sectionMetrics.issues.length > 3 ? `
                                        <li class="more-issues">...and ${sectionMetrics.issues.length - 3} more</li>
                                    ` : ''}
                                </ul>
                            </div>
                        ` : ''}
                    </div>
                    
                    <div class="section-actions">
                        <button class="btn btn-sm btn-primary" onclick="configDashboard.openSectionConfig('${sectionId}')">
                            ${sectionMetrics.completion === 0 ? 'Configure' : 'Edit'}
                        </button>
                        ${sectionMetrics.completion > 0 ? `
                            <button class="btn btn-sm btn-secondary" onclick="configDashboard.validateSection('${sectionId}')">
                                Validate
                            </button>
                        ` : ''}
                    </div>
                </div>
            `;
        }

        sectionsContent.innerHTML = html;
    }

    updateMilestones(metrics) {
        const milestonesList = document.getElementById('milestones-list');
        if (!milestonesList) return;

        let html = '';
        
        for (const [milestoneId, milestone] of this.milestones.entries()) {
            const isCompleted = metrics.overall >= milestone.threshold;
            const isActive = !isCompleted && this.isMilestoneInProgress(milestone, metrics);
            
            html += `
                <div class="milestone-item ${isCompleted ? 'completed' : ''} ${isActive ? 'active' : ''}">
                    <div class="milestone-icon" style="color: ${milestone.color}">
                        <i class="fas fa-${milestone.icon}"></i>
                    </div>
                    <div class="milestone-content">
                        <h5>${milestone.name}</h5>
                        <p>${milestone.description}</p>
                        <div class="milestone-progress">
                            <span>${Math.min(metrics.overall, milestone.threshold)}% of ${milestone.threshold}% required</span>
                        </div>
                    </div>
                    <div class="milestone-status">
                        ${isCompleted ? '<i class="fas fa-check-circle"></i>' : 
                          isActive ? '<i class="fas fa-clock"></i>' : 
                          '<i class="fas fa-circle"></i>'}
                    </div>
                </div>
            `;
        }

        milestonesList.innerHTML = html;
    }

    updateRecommendations(metrics) {
        const recommendationsList = document.getElementById('recommendations-list');
        if (!recommendationsList) return;

        const recommendations = this.generateRecommendations(metrics);
        
        let html = '';
        
        recommendations.forEach(rec => {
            html += `
                <div class="recommendation-item ${rec.priority}">
                    <div class="recommendation-icon">
                        <i class="fas fa-${rec.icon}"></i>
                    </div>
                    <div class="recommendation-content">
                        <h5>${rec.title}</h5>
                        <p>${rec.description}</p>
                        ${rec.actions ? `
                            <div class="recommendation-actions">
                                ${rec.actions.map(action => `
                                    <button class="btn btn-sm btn-outline" onclick="${action.handler}">
                                        ${action.label}
                                    </button>
                                `).join('')}
                            </div>
                        ` : ''}
                    </div>
                    <div class="recommendation-priority">
                        <span class="priority-badge ${rec.priority}">${rec.priority}</span>
                    </div>
                </div>
            `;
        });

        recommendationsList.innerHTML = html || '<p class="no-recommendations">No recommendations at this time. Great job!</p>';
    }

    generateRecommendations(metrics) {
        const recommendations = [];

        // Critical section recommendations
        for (const [sectionId, sectionMetrics] of metrics.sections.entries()) {
            const requirements = this.completionRequirements.get(sectionId);
            
            if (requirements.critical && sectionMetrics.completion < 50) {
                recommendations.push({
                    priority: 'high',
                    icon: 'exclamation-triangle',
                    title: `Complete ${requirements.name}`,
                    description: `This critical section is only ${sectionMetrics.completion}% complete. ${sectionMetrics.errors} errors need attention.`,
                    actions: [{
                        label: 'Configure Now',
                        handler: `configDashboard.openSectionConfig('${sectionId}')`
                    }]
                });
            }
        }

        // Security recommendations
        const securityMetrics = metrics.sections.get('security');
        if (securityMetrics && securityMetrics.completion < 80) {
            recommendations.push({
                priority: 'high',
                icon: 'shield-alt',
                title: 'Strengthen Security Configuration',
                description: 'Your security settings need attention for a production deployment.',
                actions: [{
                    label: 'Security Wizard',
                    handler: `configDashboard.openSecurityWizard()`
                }]
            });
        }

        // Performance recommendations
        if (metrics.overall > 60 && metrics.sections.get('performance')?.completion < 50) {
            recommendations.push({
                priority: 'medium',
                icon: 'tachometer-alt',
                title: 'Optimize Performance Settings',
                description: 'Configure performance settings for better virtual world experience.',
                actions: [{
                    label: 'Performance Tuner',
                    handler: `configDashboard.openPerformanceTuner()`
                }]
            });
        }

        // Deployment recommendations
        if (metrics.overall >= 80 && metrics.errors === 0) {
            recommendations.push({
                priority: 'low',
                icon: 'rocket',
                title: 'Ready for Deployment',
                description: 'Your configuration looks good! Consider running a final validation before deployment.',
                actions: [{
                    label: 'Final Validation',
                    handler: `configDashboard.runFinalValidation()`
                }]
            });
        }

        return recommendations.sort((a, b) => {
            const priorityOrder = { high: 3, medium: 2, low: 1 };
            return priorityOrder[b.priority] - priorityOrder[a.priority];
        });
    }

    updateDeploymentReadiness(metrics) {
        const deploymentReadiness = document.getElementById('deployment-readiness');
        if (!deploymentReadiness) return;

        const readinessInfo = this.getReadinessInfo(metrics.readiness, metrics);
        
        deploymentReadiness.innerHTML = `
            <div class="readiness-indicator ${metrics.readiness}">
                <div class="readiness-icon">
                    <i class="fas fa-${readinessInfo.icon}"></i>
                </div>
                <div class="readiness-content">
                    <h4>${readinessInfo.title}</h4>
                    <p>${readinessInfo.description}</p>
                    ${readinessInfo.actions ? `
                        <div class="readiness-actions">
                            ${readinessInfo.actions.map(action => `
                                <button class="btn btn-${action.type}" onclick="${action.handler}">
                                    ${action.label}
                                </button>
                            `).join('')}
                        </div>
                    ` : ''}
                </div>
                <div class="readiness-progress">
                    <div class="readiness-bar">
                        <div class="readiness-fill" style="width: ${metrics.overall}%"></div>
                    </div>
                    <span>${metrics.overall}% Complete</span>
                </div>
            </div>
        `;
    }

    getReadinessInfo(readiness, metrics) {
        const info = {
            'not-ready': {
                icon: 'times-circle',
                title: 'Not Ready for Deployment',
                description: 'Complete more configuration sections before deployment.',
                actions: [{
                    type: 'primary',
                    label: 'Continue Configuration',
                    handler: 'configDashboard.continueConfiguration()'
                }]
            },
            'development-ready': {
                icon: 'code',
                title: 'Development Ready',
                description: 'Ready for development and testing environments.',
                actions: [{
                    type: 'primary',
                    label: 'Deploy for Development',
                    handler: 'configDashboard.deployDevelopment()'
                }]
            },
            'testing-ready': {
                icon: 'flask',
                title: 'Testing Ready',
                description: 'Ready for testing environments with some limitations.',
                actions: [{
                    type: 'primary',
                    label: 'Deploy for Testing',
                    handler: 'configDashboard.deployTesting()'
                }]
            },
            'deployment-ready': {
                icon: 'check-circle',
                title: 'Deployment Ready',
                description: 'Critical sections complete. Ready for production deployment.',
                actions: [{
                    type: 'success',
                    label: 'Deploy to Production',
                    handler: 'configDashboard.deployProduction()'
                }]
            },
            'production-ready': {
                icon: 'rocket',
                title: 'Production Ready',
                description: 'All sections complete and optimized for production use.',
                actions: [{
                    type: 'success',
                    label: 'Deploy Now',
                    handler: 'configDashboard.deployProduction()'
                }]
            }
        };

        return info[readiness] || info['not-ready'];
    }

    updateQuickInsights(metrics) {
        const insightsGrid = document.getElementById('insights-grid');
        if (!insightsGrid) return;

        const insights = [
            {
                label: 'Configuration Health',
                value: this.getHealthScore(metrics),
                trend: this.getHealthTrend(),
                icon: 'heartbeat',
                color: this.getHealthColor(metrics)
            },
            {
                label: 'Security Score',
                value: `${metrics.sections.get('security')?.completion || 0}%`,
                trend: 'stable',
                icon: 'shield-alt',
                color: '#2196f3'
            },
            {
                label: 'Deployment Risk',
                value: this.getDeploymentRisk(metrics),
                trend: 'improving',
                icon: 'exclamation-triangle',
                color: this.getRiskColor(metrics)
            },
            {
                label: 'Estimated Setup Time',
                value: this.getEstimatedSetupTime(metrics),
                trend: 'decreasing',
                icon: 'clock',
                color: '#ff9800'
            }
        ];

        insightsGrid.innerHTML = insights.map(insight => `
            <div class="insight-card">
                <div class="insight-icon" style="color: ${insight.color}">
                    <i class="fas fa-${insight.icon}"></i>
                </div>
                <div class="insight-content">
                    <span class="insight-label">${insight.label}</span>
                    <span class="insight-value">${insight.value}</span>
                    <span class="insight-trend ${insight.trend}">
                        <i class="fas fa-${this.getTrendIcon(insight.trend)}"></i>
                    </span>
                </div>
            </div>
        `).join('');
    }

    updateNextAction(metrics) {
        const nextActionBtn = document.getElementById('next-action');
        const nextActionText = document.getElementById('next-action-text');
        
        if (!nextActionBtn || !nextActionText) return;

        const nextAction = this.determineNextAction(metrics);
        
        nextActionText.textContent = nextAction.text;
        nextActionBtn.disabled = !nextAction.enabled;
        nextActionBtn.onclick = nextAction.handler;
        
        if (nextAction.enabled) {
            nextActionBtn.className = `btn btn-${nextAction.type}`;
        }
    }

    determineNextAction(metrics) {
        // Find the section with lowest completion that's critical
        const criticalSections = Array.from(metrics.sections.entries())
            .filter(([id, _]) => this.completionRequirements.get(id).critical)
            .sort(([,a], [,b]) => a.completion - b.completion);

        if (criticalSections.length > 0 && criticalSections[0][1].completion < 80) {
            const [sectionId, sectionMetrics] = criticalSections[0];
            const sectionName = this.completionRequirements.get(sectionId).name;
            
            return {
                text: `Configure ${sectionName}`,
                enabled: true,
                type: 'primary',
                handler: () => this.openSectionConfig(sectionId)
            };
        }

        // If all critical sections are done, suggest next logical step
        if (metrics.overall >= 80) {
            return {
                text: 'Deploy Configuration',
                enabled: true,
                type: 'success',
                handler: () => this.deployConfiguration()
            };
        }

        // Find any incomplete section
        const incompleteSections = Array.from(metrics.sections.entries())
            .filter(([_, sectionMetrics]) => sectionMetrics.completion < 100)
            .sort(([,a], [,b]) => b.completion - a.completion);

        if (incompleteSections.length > 0) {
            const [sectionId, sectionMetrics] = incompleteSections[0];
            const sectionName = this.completionRequirements.get(sectionId).name;
            
            return {
                text: `Complete ${sectionName}`,
                enabled: true,
                type: 'primary',
                handler: () => this.openSectionConfig(sectionId)
            };
        }

        return {
            text: 'Configuration Complete',
            enabled: false,
            type: 'success',
            handler: null
        };
    }

    // Utility methods
    getFieldValue(fieldId, defaultValue = '', type = 'text') {
        const element = document.getElementById(fieldId) || 
                      document.querySelector(`[name="${fieldId}"]`) ||
                      document.querySelector(`[data-field="${fieldId}"]`);
        
        if (!element) return defaultValue;

        switch (type) {
            case 'checkbox':
                return element.checked;
            case 'number':
                return parseFloat(element.value) || defaultValue;
            case 'data':
                return element.dataset.value === 'true' || element.dataset[fieldId] === 'true';
            default:
                return element.value || defaultValue;
        }
    }

    getSectionStatusClass(metrics) {
        if (metrics.errors > 0) return 'error';
        if (metrics.completion >= 100) return 'complete';
        if (metrics.completion >= 80) return 'good';
        if (metrics.completion >= 50) return 'partial';
        return 'incomplete';
    }

    getSectionStatusIcon(metrics) {
        if (metrics.errors > 0) return 'fa-exclamation-circle';
        if (metrics.completion >= 100) return 'fa-check-circle';
        if (metrics.completion >= 80) return 'fa-check';
        if (metrics.completion >= 50) return 'fa-clock';
        return 'fa-circle';
    }

    isMilestoneInProgress(milestone, metrics) {
        const requiredSections = milestone.requirements;
        const completedRequirements = requiredSections.filter(sectionId => 
            metrics.sections.get(sectionId)?.completion >= 80
        ).length;
        
        return completedRequirements >= requiredSections.length - 1 && metrics.overall < milestone.threshold;
    }

    getHealthScore(metrics) {
        const errorPenalty = metrics.errors * 5;
        const warningPenalty = metrics.warnings * 2;
        const baseScore = metrics.overall;
        const healthScore = Math.max(0, Math.min(100, baseScore - errorPenalty - warningPenalty));
        
        if (healthScore >= 90) return 'Excellent';
        if (healthScore >= 70) return 'Good';
        if (healthScore >= 50) return 'Fair';
        if (healthScore >= 30) return 'Poor';
        return 'Critical';
    }

    getHealthColor(metrics) {
        const score = this.getHealthScore(metrics);
        const colors = {
            'Excellent': '#4caf50',
            'Good': '#8bc34a',
            'Fair': '#ff9800',
            'Poor': '#ff5722',
            'Critical': '#f44336'
        };
        return colors[score] || '#9e9e9e';
    }

    getDeploymentRisk(metrics) {
        if (metrics.errors > 5) return 'High';
        if (metrics.errors > 2) return 'Medium';
        if (metrics.errors > 0) return 'Low';
        if (metrics.warnings > 3) return 'Low';
        return 'Minimal';
    }

    getRiskColor(metrics) {
        const risk = this.getDeploymentRisk(metrics);
        const colors = {
            'Minimal': '#4caf50',
            'Low': '#ff9800',
            'Medium': '#ff5722',
            'High': '#f44336'
        };
        return colors[risk] || '#9e9e9e';
    }

    getEstimatedSetupTime(metrics) {
        const remainingItems = Array.from(metrics.sections.values())
            .reduce((sum, section) => sum + (section.totalItems - section.completedItems), 0);
        const minutes = remainingItems * 2;
        return this.formatEstimatedTime(minutes);
    }

    formatEstimatedTime(minutes) {
        if (minutes <= 0) return 'Complete';
        if (minutes < 60) return `${minutes}m`;
        const hours = Math.floor(minutes / 60);
        const remainingMinutes = minutes % 60;
        if (remainingMinutes === 0) return `${hours}h`;
        return `${hours}h ${remainingMinutes}m`;
    }

    getTrendIcon(trend) {
        const icons = {
            'improving': 'arrow-up',
            'stable': 'minus',
            'declining': 'arrow-down',
            'decreasing': 'arrow-down'
        };
        return icons[trend] || 'minus';
    }

    // Action handlers
    refreshDashboard() {
        this.updateDashboard();
        this.showNotification('Dashboard refreshed', 'success');
    }

    openSectionConfig(sectionId) {
        // Navigate to the appropriate wizard step or configuration section
        console.log(`Opening configuration for section: ${sectionId}`);
        // This would integrate with the existing wizard navigation
        const event = new CustomEvent('navigateToSection', { 
            detail: { section: sectionId } 
        });
        document.dispatchEvent(event);
    }

    validateSection(sectionId) {
        console.log(`Validating section: ${sectionId}`);
        // This would trigger section-specific validation
        const event = new CustomEvent('validateSection', { 
            detail: { section: sectionId } 
        });
        document.dispatchEvent(event);
    }

    switchView(viewType) {
        document.querySelectorAll('.view-btn').forEach(btn => btn.classList.remove('active'));
        document.querySelector(`[data-view="${viewType}"]`).classList.add('active');
        
        const sectionsContent = document.getElementById('sections-content');
        if (sectionsContent) {
            sectionsContent.className = `sections-content view-${viewType}`;
        }
    }

    exportProgressReport() {
        const currentConfig = this.captureConfigurationState();
        const metrics = this.calculateCompletionMetrics(currentConfig);
        
        const report = {
            timestamp: new Date().toISOString(),
            overall: metrics.overall,
            sections: Object.fromEntries(metrics.sections),
            recommendations: this.generateRecommendations(metrics),
            readiness: metrics.readiness
        };

        const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `opensim-next-progress-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        this.showNotification('Progress report exported', 'success');
    }

    saveProgress() {
        const currentConfig = this.captureConfigurationState();
        localStorage.setItem('opensim-next-config-state', JSON.stringify(currentConfig));
        localStorage.setItem('opensim-next-config-timestamp', Date.now().toString());
        this.showNotification('Progress saved', 'success');
    }

    loadSavedState() {
        const savedState = localStorage.getItem('opensim-next-config-state');
        if (savedState) {
            try {
                const state = JSON.parse(savedState);
                this.configState.set('saved', state);
                console.log('Loaded saved configuration state');
            } catch (error) {
                console.error('Failed to load saved state:', error);
            }
        }
    }

    showNotification(message, type = 'info') {
        // This would integrate with a notification system
        console.log(`${type.toUpperCase()}: ${message}`);
    }

    // Event handlers
    handleConfigurationChange(detail) {
        // Debounce dashboard updates
        clearTimeout(this.updateTimeout);
        this.updateTimeout = setTimeout(() => {
            this.updateDashboard();
        }, 500);
    }

    handlePreviewUpdate(detail) {
        this.updateDashboard();
    }

    handleValidationUpdate(detail) {
        this.updateDashboard();
    }
}

// Initialize dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.configDashboard = new ConfigurationDashboard();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = ConfigurationDashboard;
}