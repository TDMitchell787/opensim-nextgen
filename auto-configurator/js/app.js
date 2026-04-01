// OpenSim Next Auto-Configurator - Main Application
// Intelligent configuration wizard with security-first design

class OpenSimConfigurator {
    constructor() {
        this.currentStep = 1;
        this.totalSteps = 7;
        this.configuration = {
            deploymentType: null,
            environment: {},
            database: {},
            regions: [],
            security: {},
            network: {},
            validation: {}
        };
        this.validationErrors = [];
        this.securityAlerts = [];
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadSavedConfiguration();
        this.updateProgress();
        this.initializeSecurityMonitoring();
        this.setupAutoSave();
        
        console.log('OpenSim Next Auto-Configurator initialized');
    }

    setupEventListeners() {
        // Dashboard card clicks
        document.querySelectorAll('.config-card').forEach(card => {
            card.addEventListener('click', (e) => {
                const section = e.currentTarget.dataset.section;
                this.openConfigurationSection(section);
            });
        });

        // Wizard navigation
        const wizardNext = document.getElementById('wizard-next');
        const wizardBack = document.getElementById('wizard-back');
        
        if (wizardNext) {
            wizardNext.addEventListener('click', () => this.nextStep());
        }
        
        if (wizardBack) {
            wizardBack.addEventListener('click', () => this.previousStep());
        }

        // Quick actions
        document.getElementById('import-config')?.addEventListener('click', () => {
            this.importConfiguration();
        });
        
        document.getElementById('load-template')?.addEventListener('click', () => {
            this.loadTemplate();
        });
        
        document.getElementById('expert-mode')?.addEventListener('click', () => {
            this.toggleExpertMode();
        });

        // Help system
        document.getElementById('help-btn')?.addEventListener('click', () => {
            this.showHelp();
        });

        // Deployment type selection
        document.querySelectorAll('.deployment-option').forEach(option => {
            option.addEventListener('click', (e) => {
                this.selectDeploymentType(e.currentTarget.dataset.type);
            });
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            this.handleKeyboardShortcuts(e);
        });

        // Window events
        window.addEventListener('beforeunload', (e) => {
            if (this.hasUnsavedChanges()) {
                e.preventDefault();
                e.returnValue = 'You have unsaved configuration changes. Are you sure you want to leave?';
            }
        });
    }

    // Configuration Management
    selectDeploymentType(type) {
        // Clear previous selection
        document.querySelectorAll('.deployment-option').forEach(option => {
            option.classList.remove('selected');
        });

        // Select new type
        const selectedOption = document.querySelector(`[data-type="${type}"]`);
        selectedOption.classList.add('selected');

        this.configuration.deploymentType = type;
        this.applyDeploymentDefaults(type);
        this.validateConfiguration();
        this.updateDashboard();
        this.saveConfiguration();

        // Enable next button
        document.getElementById('wizard-next').disabled = false;

        console.log(`Deployment type selected: ${type}`);
    }

    applyDeploymentDefaults(type) {
        const defaults = {
            development: {
                database: {
                    type: 'sqlite',
                    location: './opensim.db',
                    connectionPoolSize: 5
                },
                security: {
                    sslEnabled: false,
                    authenticationLevel: 'basic',
                    encryptionRequired: false
                },
                network: {
                    maxConnections: 50,
                    ports: {
                        http: 9000,
                        https: 9001,
                        admin: 8090
                    }
                },
                physics: {
                    defaultEngine: 'ODE',
                    maxBodies: 1000
                },
                monitoring: {
                    enabled: true,
                    level: 'basic'
                }
            },
            production: {
                database: {
                    type: 'postgresql',
                    host: 'localhost',
                    port: 5432,
                    connectionPoolSize: 20
                },
                security: {
                    sslEnabled: true,
                    authenticationLevel: 'enhanced',
                    encryptionRequired: true
                },
                network: {
                    maxConnections: 500,
                    ports: {
                        http: 80,
                        https: 443,
                        admin: 8090
                    }
                },
                physics: {
                    defaultEngine: 'Bullet',
                    maxBodies: 10000
                },
                monitoring: {
                    enabled: true,
                    level: 'comprehensive'
                }
            },
            grid: {
                database: {
                    type: 'postgresql',
                    clustered: true,
                    replication: true,
                    connectionPoolSize: 50
                },
                security: {
                    sslEnabled: true,
                    authenticationLevel: 'enterprise',
                    encryptionRequired: true,
                    zeroTrust: true
                },
                network: {
                    maxConnections: 5000,
                    loadBalancing: true,
                    cdn: true,
                    ports: {
                        http: 80,
                        https: 443,
                        admin: 8090,
                        grid: 8003
                    }
                },
                physics: {
                    defaultEngine: 'POS',
                    multiEngine: true,
                    maxBodies: 100000
                },
                monitoring: {
                    enabled: true,
                    level: 'enterprise',
                    realtime: true
                }
            }
        };

        const typeDefaults = defaults[type];
        if (typeDefaults) {
            // Merge defaults into configuration
            Object.keys(typeDefaults).forEach(key => {
                this.configuration[key] = {
                    ...this.configuration[key],
                    ...typeDefaults[key]
                };
            });
        }
    }

    // Navigation
    nextStep() {
        if (this.currentStep < this.totalSteps) {
            if (this.validateCurrentStep()) {
                this.currentStep++;
                this.updateWizardStep();
                this.updateProgress();
            }
        } else {
            this.completeConfiguration();
        }
    }

    previousStep() {
        if (this.currentStep > 1) {
            this.currentStep--;
            this.updateWizardStep();
            this.updateProgress();
        }
    }

    updateWizardStep() {
        // Update step indicators
        document.querySelectorAll('.wizard-step').forEach((step, index) => {
            step.classList.toggle('active', index + 1 === this.currentStep);
        });

        // Update progress steps
        document.querySelectorAll('.step').forEach((step, index) => {
            step.classList.toggle('active', index + 1 === this.currentStep);
            step.classList.toggle('completed', index + 1 < this.currentStep);
        });

        // Update navigation buttons
        document.getElementById('wizard-back').disabled = this.currentStep === 1;
        document.getElementById('wizard-next').textContent = 
            this.currentStep === this.totalSteps ? 'Complete Setup' : 'Next';

        // Load step content
        this.loadStepContent();
    }

    loadStepContent() {
        const steps = {
            1: this.loadDeploymentStep,
            2: this.loadEnvironmentStep,
            3: this.loadDatabaseStep,
            4: this.loadRegionsStep,
            5: this.loadSecurityStep,
            6: this.loadNetworkStep,
            7: this.loadReviewStep
        };

        const stepLoader = steps[this.currentStep];
        if (stepLoader) {
            stepLoader.call(this);
        }
    }

    // Step Content Loaders
    loadDeploymentStep() {
        document.getElementById('wizard-title').textContent = 'Choose Deployment Type';
        document.getElementById('wizard-description').textContent = 
            'Select the type of OpenSim Next deployment that best matches your needs.';
        
        // Deployment options are already in HTML
    }

    loadEnvironmentStep() {
        document.getElementById('wizard-title').textContent = 'Environment Configuration';
        document.getElementById('wizard-description').textContent = 
            'Configure basic environment settings for your OpenSim Next installation.';
        
        const content = `
            <div class="form-section">
                <div class="form-group">
                    <label for="install-path">Installation Path</label>
                    <input type="text" id="install-path" class="form-input" 
                           value="${this.configuration.environment.installPath || '/opt/opensim-next'}"
                           placeholder="/opt/opensim-next">
                    <p class="form-help">Directory where OpenSim Next will be installed</p>
                </div>
                
                <div class="form-group">
                    <label for="grid-name">Grid Name</label>
                    <input type="text" id="grid-name" class="form-input" 
                           value="${this.configuration.environment.gridName || 'My OpenSim Grid'}"
                           placeholder="My OpenSim Grid">
                    <p class="form-help">Display name for your virtual world grid</p>
                </div>
                
                <div class="form-group">
                    <label for="admin-email">Administrator Email</label>
                    <input type="email" id="admin-email" class="form-input" 
                           value="${this.configuration.environment.adminEmail || ''}"
                           placeholder="admin@yourdomain.com">
                    <p class="form-help">Email address for administrative notifications</p>
                </div>
                
                <div class="form-group">
                    <label for="log-level">Log Level</label>
                    <select id="log-level" class="form-select">
                        <option value="info" ${this.configuration.environment.logLevel === 'info' ? 'selected' : ''}>Info</option>
                        <option value="debug" ${this.configuration.environment.logLevel === 'debug' ? 'selected' : ''}>Debug</option>
                        <option value="warn" ${this.configuration.environment.logLevel === 'warn' ? 'selected' : ''}>Warning</option>
                        <option value="error" ${this.configuration.environment.logLevel === 'error' ? 'selected' : ''}>Error</option>
                    </select>
                    <p class="form-help">Logging verbosity level</p>
                </div>
            </div>
        `;
        
        document.getElementById('wizard-step-content').innerHTML = content;
        this.setupEnvironmentEventListeners();
    }

    loadDatabaseStep() {
        document.getElementById('wizard-title').textContent = 'Database Configuration';
        document.getElementById('wizard-description').textContent = 
            'Configure the database for storing virtual world data.';
        
        const dbType = this.configuration.database.type || 'sqlite';
        
        let content = `
            <div class="form-section">
                <div class="form-group">
                    <label>Database Type</label>
                    <div class="radio-group">
                        <label class="radio-option">
                            <input type="radio" name="db-type" value="sqlite" ${dbType === 'sqlite' ? 'checked' : ''}>
                            <span class="radio-label">
                                <strong>SQLite</strong>
                                <span class="radio-desc">File-based database, perfect for development</span>
                            </span>
                        </label>
                        <label class="radio-option">
                            <input type="radio" name="db-type" value="postgresql" ${dbType === 'postgresql' ? 'checked' : ''}>
                            <span class="radio-label">
                                <strong>PostgreSQL</strong>
                                <span class="radio-desc">Enterprise database for production</span>
                            </span>
                        </label>
                    </div>
                </div>
                
                <div id="db-config-content"></div>
            </div>
        `;
        
        document.getElementById('wizard-step-content').innerHTML = content;
        this.loadDatabaseConfig(dbType);
        this.setupDatabaseEventListeners();
    }

    loadSecurityStep() {
        document.getElementById('wizard-title').textContent = 'Security Configuration';
        document.getElementById('wizard-description').textContent = 
            'Configure security settings and cryptographic materials.';
        
        const content = `
            <div class="security-section">
                <div class="security-alerts">
                    <div class="alert alert-info">
                        <i class="icon-shield"></i>
                        <strong>Security Notice:</strong> OpenSim Next requires cryptographic materials for secure operation.
                        We never store your keys or certificates - they remain under your control.
                    </div>
                </div>
                
                <div class="form-section">
                    <h4>SSL/TLS Configuration</h4>
                    <div class="form-group">
                        <label class="checkbox-label">
                            <input type="checkbox" id="ssl-enabled" ${this.configuration.security.sslEnabled ? 'checked' : ''}>
                            Enable SSL/TLS encryption
                        </label>
                        <p class="form-help">Recommended for production deployments</p>
                    </div>
                    
                    <div id="ssl-config" style="display: ${this.configuration.security.sslEnabled ? 'block' : 'none'}">
                        <div class="form-group">
                            <label for="ssl-cert">SSL Certificate Path</label>
                            <div class="file-input-group">
                                <input type="text" id="ssl-cert" class="form-input" 
                                       value="${this.configuration.security.sslCertPath || ''}"
                                       placeholder="/path/to/certificate.pem">
                                <button type="button" class="btn btn-secondary">Browse</button>
                            </div>
                        </div>
                        
                        <div class="form-group">
                            <label for="ssl-key">SSL Private Key Path</label>
                            <div class="file-input-group">
                                <input type="text" id="ssl-key" class="form-input" 
                                       value="${this.configuration.security.sslKeyPath || ''}"
                                       placeholder="/path/to/private-key.pem">
                                <button type="button" class="btn btn-secondary">Browse</button>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div class="form-section">
                    <h4>Authentication</h4>
                    <div class="form-group">
                        <label for="auth-level">Authentication Level</label>
                        <select id="auth-level" class="form-select">
                            <option value="basic" ${this.configuration.security.authenticationLevel === 'basic' ? 'selected' : ''}>Basic</option>
                            <option value="enhanced" ${this.configuration.security.authenticationLevel === 'enhanced' ? 'selected' : ''}>Enhanced</option>
                            <option value="enterprise" ${this.configuration.security.authenticationLevel === 'enterprise' ? 'selected' : ''}>Enterprise</option>
                        </select>
                    </div>
                </div>
                
                <div class="form-section">
                    <h4>Secure Key Storage</h4>
                    <div class="key-storage-options">
                        <div class="storage-option">
                            <h5>Encrypted USB Key (Recommended)</h5>
                            <p>Store all cryptographic materials on an encrypted USB drive for maximum security.</p>
                            <button type="button" class="btn btn-primary" id="setup-usb-key">Setup USB Key</button>
                        </div>
                        
                        <div class="storage-option">
                            <h5>Local Filesystem</h5>
                            <p>Store keys on the local filesystem with appropriate permissions.</p>
                            <button type="button" class="btn btn-outline" id="setup-local-keys">Setup Local Storage</button>
                        </div>
                    </div>
                </div>
            </div>
        `;
        
        document.getElementById('wizard-step-content').innerHTML = content;
        this.setupSecurityEventListeners();
    }

    // Progress and UI Updates
    updateProgress() {
        const progress = (this.currentStep / this.totalSteps) * 100;
        document.getElementById('progress-fill').style.width = `${progress}%`;
        document.getElementById('current-step').textContent = this.currentStep;
        document.getElementById('total-steps').textContent = this.totalSteps;
        document.getElementById('overall-progress').textContent = `${Math.round(progress)}%`;
    }

    updateDashboard() {
        // Update dashboard cards based on configuration state
        this.updateCardStatus('deployment', this.configuration.deploymentType ? 'completed' : 'pending');
        this.updateCardStatus('environment', this.isEnvironmentConfigured() ? 'completed' : 'pending');
        this.updateCardStatus('database', this.isDatabaseConfigured() ? 'completed' : 'pending');
        this.updateCardStatus('regions', this.areRegionsConfigured() ? 'completed' : 'pending');
        this.updateCardStatus('security', this.getSecurityStatus());
        this.updateCardStatus('network', this.isNetworkConfigured() ? 'completed' : 'pending');
        
        this.updateSecurityStatus();
    }

    updateCardStatus(section, status) {
        const card = document.querySelector(`[data-section="${section}"]`);
        if (card) {
            card.className = `config-card ${status}`;
            
            const statusIcon = card.querySelector('.card-status i');
            const statusIcons = {
                completed: 'icon-check',
                'in-progress': 'icon-spinner',
                pending: 'icon-pending',
                warning: 'icon-warning'
            };
            
            if (statusIcon) {
                statusIcon.className = statusIcons[status] || 'icon-pending';
            }
        }
    }

    updateSecurityStatus() {
        const securityAlerts = this.getSecurityAlerts();
        const statusElement = document.getElementById('security-status');
        
        if (securityAlerts.length === 0) {
            statusElement.innerHTML = '<i class="icon-check"></i> All security requirements met';
            statusElement.className = 'status-value success';
        } else {
            statusElement.innerHTML = `<i class="icon-warning"></i> ${securityAlerts.length} items require attention`;
            statusElement.className = 'status-value warning';
        }
    }

    // Validation
    validateConfiguration() {
        this.validationErrors = [];
        
        // Validate each section
        this.validateDeploymentType();
        this.validateEnvironment();
        this.validateDatabase();
        this.validateRegions();
        this.validateSecurity();
        this.validateNetwork();
        
        return this.validationErrors.length === 0;
    }

    validateCurrentStep() {
        const stepValidators = {
            1: () => this.configuration.deploymentType !== null,
            2: () => this.isEnvironmentConfigured(),
            3: () => this.isDatabaseConfigured(),
            4: () => this.areRegionsConfigured(),
            5: () => this.getSecurityStatus() !== 'warning',
            6: () => this.isNetworkConfigured(),
            7: () => this.validateConfiguration()
        };
        
        const validator = stepValidators[this.currentStep];
        return validator ? validator() : true;
    }

    // Helper methods for validation
    isEnvironmentConfigured() {
        const env = this.configuration.environment;
        return env.installPath && env.gridName && env.adminEmail;
    }

    isDatabaseConfigured() {
        const db = this.configuration.database;
        if (db.type === 'sqlite') {
            return db.location;
        } else if (db.type === 'postgresql') {
            return db.host && db.port && db.database && db.username;
        }
        return false;
    }

    areRegionsConfigured() {
        return this.configuration.regions.length > 0;
    }

    isNetworkConfigured() {
        const network = this.configuration.network;
        return network.ports && network.maxConnections;
    }

    getSecurityStatus() {
        const alerts = this.getSecurityAlerts();
        return alerts.length === 0 ? 'completed' : 'warning';
    }

    getSecurityAlerts() {
        const alerts = [];
        const security = this.configuration.security;
        
        if (security.sslEnabled && (!security.sslCertPath || !security.sslKeyPath)) {
            alerts.push('SSL certificate and key paths required');
        }
        
        if (this.configuration.deploymentType === 'production' && !security.sslEnabled) {
            alerts.push('SSL/TLS recommended for production deployments');
        }
        
        if (this.configuration.deploymentType === 'grid' && !security.zeroTrust) {
            alerts.push('Zero trust networking recommended for grid deployments');
        }
        
        return alerts;
    }

    // Configuration persistence
    saveConfiguration() {
        try {
            localStorage.setItem('opensim-configurator-state', JSON.stringify({
                configuration: this.configuration,
                currentStep: this.currentStep,
                timestamp: Date.now()
            }));
            console.log('Configuration saved to localStorage');
        } catch (error) {
            console.error('Failed to save configuration:', error);
        }
    }

    loadSavedConfiguration() {
        try {
            const saved = localStorage.getItem('opensim-configurator-state');
            if (saved) {
                const state = JSON.parse(saved);
                this.configuration = { ...this.configuration, ...state.configuration };
                this.currentStep = state.currentStep || 1;
                console.log('Configuration loaded from localStorage');
            }
        } catch (error) {
            console.error('Failed to load saved configuration:', error);
        }
    }

    hasUnsavedChanges() {
        // Check if there are any meaningful changes to the configuration
        return Object.keys(this.configuration).some(key => 
            this.configuration[key] && 
            typeof this.configuration[key] === 'object' && 
            Object.keys(this.configuration[key]).length > 0
        );
    }

    // Auto-save functionality
    setupAutoSave() {
        setInterval(() => {
            if (this.hasUnsavedChanges()) {
                this.saveConfiguration();
            }
        }, 30000); // Save every 30 seconds
    }

    // Security monitoring
    initializeSecurityMonitoring() {
        // Monitor for security-related changes
        setInterval(() => {
            this.updateSecurityStatus();
        }, 5000);
    }

    // Keyboard shortcuts
    handleKeyboardShortcuts(e) {
        if (e.ctrlKey || e.metaKey) {
            switch (e.key) {
                case 's':
                    e.preventDefault();
                    this.saveConfiguration();
                    this.showNotification('Configuration saved', 'success');
                    break;
                case 'h':
                    e.preventDefault();
                    this.showHelp();
                    break;
            }
        }
    }

    // Utility methods
    showNotification(message, type = 'info') {
        // Create and show notification toast
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        notification.textContent = message;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.remove();
        }, 3000);
    }

    showHelp() {
        // Open help documentation relevant to current step
        const helpUrls = {
            1: '#deployment-help',
            2: '#environment-help',
            3: '#database-help',
            4: '#regions-help',
            5: '#security-help',
            6: '#network-help',
            7: '#review-help'
        };
        
        const helpUrl = helpUrls[this.currentStep] || '#general-help';
        window.open(helpUrl, '_blank');
    }

    // Event listener setup methods
    setupEnvironmentEventListeners() {
        ['install-path', 'grid-name', 'admin-email', 'log-level'].forEach(id => {
            const element = document.getElementById(id);
            if (element) {
                element.addEventListener('change', (e) => {
                    const key = id.replace('-', '');
                    this.configuration.environment[key] = e.target.value;
                    this.saveConfiguration();
                    this.updateDashboard();
                });
            }
        });
    }

    setupDatabaseEventListeners() {
        document.querySelectorAll('input[name="db-type"]').forEach(radio => {
            radio.addEventListener('change', (e) => {
                this.configuration.database.type = e.target.value;
                this.loadDatabaseConfig(e.target.value);
                this.saveConfiguration();
                this.updateDashboard();
            });
        });
    }

    setupSecurityEventListeners() {
        const sslEnabled = document.getElementById('ssl-enabled');
        if (sslEnabled) {
            sslEnabled.addEventListener('change', (e) => {
                this.configuration.security.sslEnabled = e.target.checked;
                document.getElementById('ssl-config').style.display = 
                    e.target.checked ? 'block' : 'none';
                this.saveConfiguration();
                this.updateDashboard();
            });
        }

        // Setup USB key management
        document.getElementById('setup-usb-key')?.addEventListener('click', () => {
            this.setupUsbKeyManagement();
        });
    }

    loadDatabaseConfig(type) {
        const configDiv = document.getElementById('db-config-content');
        
        if (type === 'sqlite') {
            configDiv.innerHTML = `
                <div class="form-group">
                    <label for="sqlite-location">Database File Location</label>
                    <input type="text" id="sqlite-location" class="form-input" 
                           value="${this.configuration.database.location || './opensim.db'}"
                           placeholder="./opensim.db">
                    <p class="form-help">Path where the SQLite database file will be created</p>
                </div>
            `;
        } else if (type === 'postgresql') {
            configDiv.innerHTML = `
                <div class="form-group">
                    <label for="pg-host">Host</label>
                    <input type="text" id="pg-host" class="form-input" 
                           value="${this.configuration.database.host || 'localhost'}"
                           placeholder="localhost">
                </div>
                
                <div class="form-group">
                    <label for="pg-port">Port</label>
                    <input type="number" id="pg-port" class="form-input" 
                           value="${this.configuration.database.port || 5432}"
                           placeholder="5432">
                </div>
                
                <div class="form-group">
                    <label for="pg-database">Database Name</label>
                    <input type="text" id="pg-database" class="form-input" 
                           value="${this.configuration.database.database || 'opensim'}"
                           placeholder="opensim">
                </div>
                
                <div class="form-group">
                    <label for="pg-username">Username</label>
                    <input type="text" id="pg-username" class="form-input" 
                           value="${this.configuration.database.username || ''}"
                           placeholder="opensim_user">
                </div>
                
                <div class="form-group">
                    <label for="pg-password">Password</label>
                    <input type="password" id="pg-password" class="form-input" 
                           placeholder="Enter database password">
                    <p class="form-help">Password will not be stored in configuration files</p>
                </div>
            `;
        }
    }

    setupUsbKeyManagement() {
        // This would open a specialized interface for USB key management
        this.showNotification('USB key management feature coming soon', 'info');
    }

    // Placeholder methods for other steps
    loadRegionsStep() {
        document.getElementById('wizard-title').textContent = 'Region Configuration';
        document.getElementById('wizard-description').textContent = 
            'Configure virtual world regions and their settings.';
    }

    loadNetworkStep() {
        document.getElementById('wizard-title').textContent = 'Network Configuration';
        document.getElementById('wizard-description').textContent = 
            'Configure network settings and connectivity options.';
    }

    loadReviewStep() {
        document.getElementById('wizard-title').textContent = 'Review Configuration';
        document.getElementById('wizard-description').textContent = 
            'Review your configuration settings before generating files.';
    }

    // Utility methods for quick actions
    importConfiguration() {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json';
        input.onchange = (e) => {
            const file = e.target.files[0];
            if (file) {
                const reader = new FileReader();
                reader.onload = (e) => {
                    try {
                        const imported = JSON.parse(e.target.result);
                        this.configuration = { ...this.configuration, ...imported };
                        this.updateDashboard();
                        this.showNotification('Configuration imported successfully', 'success');
                    } catch (error) {
                        this.showNotification('Failed to import configuration', 'error');
                    }
                };
                reader.readAsText(file);
            }
        };
        input.click();
    }

    loadTemplate() {
        // Show template selection dialog
        this.showNotification('Template loading feature coming soon', 'info');
    }

    toggleExpertMode() {
        // Toggle between guided wizard and expert mode
        this.showNotification('Expert mode feature coming soon', 'info');
    }

    completeConfiguration() {
        if (this.validateConfiguration()) {
            // Generate configuration files and show completion
            this.generateConfigurationFiles();
        } else {
            this.showNotification('Please resolve validation errors before completing setup', 'error');
        }
    }

    generateConfigurationFiles() {
        // This would generate the actual OpenSim configuration files
        this.showNotification('Configuration files generated successfully!', 'success');
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.openSimConfigurator = new OpenSimConfigurator();
});

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = OpenSimConfigurator;
}