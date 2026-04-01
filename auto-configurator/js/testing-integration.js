// OpenSim Next Auto-Configurator - Testing Framework Integration
// Integration component for connecting testing framework with dashboard and validation system

class TestingFrameworkIntegration {
    constructor() {
        this.testingFramework = null;
        this.dashboard = null;
        this.validationSystem = null;
        this.isInitialized = false;
        
        // Integration settings
        this.settings = {
            autoRunOnConfigChange: true,
            showTestingInDashboard: true,
            integrateWithValidation: true,
            showQuickTests: true,
            highlightFailures: true
        };
        
        // Test categories for dashboard integration
        this.dashboardTestCategories = [
            'syntax-validation',
            'config-validation', 
            'security-testing'
        ];
        
        this.initializeIntegration();
    }

    async initializeIntegration() {
        try {
            // Wait for dependencies
            await this.waitForDependencies();
            
            // Initialize framework integration
            this.setupFrameworkIntegration();
            
            // Setup dashboard integration
            this.setupDashboardIntegration();
            
            // Setup validation integration
            this.setupValidationIntegration();
            
            // Add testing interface to dashboard
            this.addTestingInterfaceToDashboard();
            
            this.isInitialized = true;
            console.log('✅ Testing framework integration initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize testing framework integration:', error);
        }
    }

    async waitForDependencies() {
        // Wait for testing framework
        let attempts = 0;
        while (!window.configTestingFramework && attempts < 50) {
            await this.delay(100);
            attempts++;
        }
        
        if (window.configTestingFramework) {
            this.testingFramework = window.configTestingFramework;
        } else {
            throw new Error('Testing framework not available');
        }
        
        // Wait for dashboard
        attempts = 0;
        while (!window.configDashboard && attempts < 50) {
            await this.delay(100);
            attempts++;
        }
        
        if (window.configDashboard) {
            this.dashboard = window.configDashboard;
        }
        
        // Validation system is optional
        if (window.validationSystem) {
            this.validationSystem = window.validationSystem;
        }
    }

    setupFrameworkIntegration() {
        if (!this.testingFramework) return;
        
        // Create testing framework interface
        this.createTestingFrameworkInterface();
        
        // Setup event listeners
        this.setupTestingEventListeners();
        
        // Auto-initialize testing interface
        this.showTestingInterface();
    }

    createTestingFrameworkInterface() {
        const container = document.getElementById('testing-framework-container');
        if (!container) return;
        
        container.innerHTML = `
            <div class="testing-framework" id="testing-framework">
                <div class="testing-header">
                    <h3>Configuration Testing Framework</h3>
                    <p>Automated testing and validation for your OpenSim Next configuration</p>
                </div>
                
                <div class="testing-tabs">
                    <button class="testing-tab-button active" data-tab="quick-test">
                        <i class="icon-zap"></i>
                        Quick Test
                    </button>
                    <button class="testing-tab-button" data-tab="full-test">
                        <i class="icon-check-circle"></i>
                        Full Test Suite
                    </button>
                    <button class="testing-tab-button" data-tab="custom-test">
                        <i class="icon-settings"></i>
                        Custom Tests
                    </button>
                    <button class="testing-tab-button" data-tab="test-results">
                        <i class="icon-file-text"></i>
                        Results
                    </button>
                    <button class="testing-tab-button" data-tab="test-settings">
                        <i class="icon-sliders"></i>
                        Settings
                    </button>
                </div>

                <!-- Quick Test Tab -->
                <div class="testing-tab-content active" id="quick-test">
                    <div class="test-config-section">
                        <h4>Quick Configuration Test</h4>
                        <p>Run essential tests to verify your configuration is valid and secure.</p>
                        
                        <div class="config-source-selector">
                            <div class="source-option selected" data-source="current">
                                <input type="radio" name="config-source" value="current" checked>
                                <div class="source-radio"></div>
                                <div class="source-label">Current Configuration</div>
                            </div>
                            <div class="source-option" data-source="file">
                                <input type="radio" name="config-source" value="file">
                                <div class="source-radio"></div>
                                <div class="source-label">Upload File</div>
                            </div>
                            <div class="source-option" data-source="template">
                                <input type="radio" name="config-source" value="template">
                                <div class="source-radio"></div>
                                <div class="source-label">Test Template</div>
                            </div>
                        </div>
                        
                        <div id="test-file-upload" class="file-upload-zone" style="display: none;">
                            <i class="icon-upload-cloud"></i>
                            <h5>Upload Configuration File</h5>
                            <p>Drop your .ini, .xml, or .json configuration file here or click to browse</p>
                            <input type="file" id="test-config-file" accept=".ini,.xml,.json,.yaml" style="display: none;">
                        </div>
                        
                        <div id="test-template-selector" class="template-selector" style="display: none;">
                            <select id="template-select">
                                <option value="">Select a test template...</option>
                                <option value="development">Development Environment</option>
                                <option value="production">Production Environment</option>
                                <option value="grid">Grid Environment</option>
                            </select>
                        </div>
                        
                        <div class="test-controls">
                            <button class="test-action-button primary" id="run-quick-test">
                                <i class="icon-play"></i>
                                Run Quick Test
                            </button>
                            <button class="test-action-button secondary" id="preview-quick-test">
                                <i class="icon-eye"></i>
                                Preview Tests
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Full Test Tab -->
                <div class="testing-tab-content" id="full-test">
                    <div class="test-config-section">
                        <h4>Complete Test Suite</h4>
                        <p>Run comprehensive tests covering all aspects of your configuration.</p>
                        
                        <div class="test-controls">
                            <button class="test-action-button primary" id="run-full-test">
                                <i class="icon-check-circle"></i>
                                Run Full Test Suite
                            </button>
                            <button class="test-action-button secondary" id="schedule-test">
                                <i class="icon-clock"></i>
                                Schedule Tests
                            </button>
                            <button class="test-action-button secondary" id="export-test-plan">
                                <i class="icon-download"></i>
                                Export Test Plan
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Custom Test Tab -->
                <div class="testing-tab-content" id="custom-test">
                    <div class="test-config-section">
                        <h4>Custom Test Selection</h4>
                        <p>Choose specific test categories to run based on your needs.</p>
                        
                        <div class="test-categories">
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-syntax" checked>
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-code"></i>
                                    </div>
                                </div>
                                <div class="category-title">Syntax Validation</div>
                                <div class="category-description">
                                    Validates INI, XML, and JSON syntax correctness
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">INI Syntax</span>
                                    <span class="test-tag">XML Syntax</span>
                                    <span class="test-tag">JSON Syntax</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-validation" checked>
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-check"></i>
                                    </div>
                                </div>
                                <div class="category-title">Configuration Validation</div>
                                <div class="category-description">
                                    Validates configuration values and dependencies
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">Required Fields</span>
                                    <span class="test-tag">Dependencies</span>
                                    <span class="test-tag">Value Ranges</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-security" checked>
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-shield"></i>
                                    </div>
                                </div>
                                <div class="category-title">Security Testing</div>
                                <div class="category-description">
                                    Tests security configuration and best practices
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">Weak Passwords</span>
                                    <span class="test-tag">SSL Config</span>
                                    <span class="test-tag">Permissions</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-performance">
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-activity"></i>
                                    </div>
                                </div>
                                <div class="category-title">Performance Testing</div>
                                <div class="category-description">
                                    Tests performance settings and optimizations
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">Resource Limits</span>
                                    <span class="test-tag">Caching</span>
                                    <span class="test-tag">Database</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-compatibility">
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-layers"></i>
                                    </div>
                                </div>
                                <div class="category-title">Compatibility Testing</div>
                                <div class="category-description">
                                    Tests compatibility with viewers and platforms
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">OpenSim Version</span>
                                    <span class="test-tag">Viewers</span>
                                    <span class="test-tag">Database</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-integration">
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-link"></i>
                                    </div>
                                </div>
                                <div class="category-title">Integration Testing</div>
                                <div class="category-description">
                                    Tests service integration and connectivity
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">Services</span>
                                    <span class="test-tag">Database</span>
                                    <span class="test-tag">Network</span>
                                </div>
                            </div>
                            
                            <div class="category-card">
                                <div class="category-header">
                                    <div class="category-checkbox">
                                        <input type="checkbox" id="test-functional">
                                        <div class="category-checkmark"></div>
                                    </div>
                                    <div class="category-icon">
                                        <i class="icon-play-circle"></i>
                                    </div>
                                </div>
                                <div class="category-title">Functional Testing</div>
                                <div class="category-description">
                                    Tests basic functionality and operations
                                </div>
                                <div class="category-tests">
                                    <span class="test-tag">Startup</span>
                                    <span class="test-tag">Region Loading</span>
                                    <span class="test-tag">User Login</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="test-controls">
                            <button class="test-action-button primary" id="run-custom-test">
                                <i class="icon-play"></i>
                                Run Selected Tests
                            </button>
                            <button class="test-action-button secondary" id="save-test-profile">
                                <i class="icon-save"></i>
                                Save Test Profile
                            </button>
                        </div>
                    </div>
                </div>

                <!-- Test Results Tab -->
                <div class="testing-tab-content" id="test-results">
                    <div id="test-progress" class="test-progress">
                        <div class="progress-header">
                            <div class="progress-title">Running Tests...</div>
                            <div class="progress-stats">
                                <span><span id="test-current">0</span> / <span id="test-total">0</span> tests</span>
                            </div>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill" id="test-progress-fill"></div>
                        </div>
                        <div class="current-test" id="current-test">Initializing...</div>
                    </div>
                    
                    <div id="results-summary" class="results-summary">
                        <div class="summary-card">
                            <h5>No test results available</h5>
                            <p>Run tests to see results here</p>
                        </div>
                    </div>
                    
                    <div id="results-details" class="results-details"></div>
                    
                    <div class="results-actions">
                        <button class="test-action-button secondary" id="export-results">
                            <i class="icon-download"></i>
                            Export Results
                        </button>
                        <button class="test-action-button secondary" id="clear-results">
                            <i class="icon-trash-2"></i>
                            Clear Results
                        </button>
                        <button class="test-action-button secondary" id="share-results">
                            <i class="icon-share"></i>
                            Share Results
                        </button>
                    </div>
                </div>

                <!-- Test Settings Tab -->
                <div class="testing-tab-content" id="test-settings">
                    <div class="test-settings">
                        <div class="settings-group">
                            <h5>Testing Options</h5>
                            <div class="setting-item">
                                <div class="setting-label">Enable Real-time Validation</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="enable-realtime" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Enable Performance Testing</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="enable-performance" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Enable Security Testing</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="enable-security" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Generate Test Reports</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="generate-reports" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                        </div>
                        
                        <div class="settings-group">
                            <h5>Test Execution</h5>
                            <div class="setting-item">
                                <div class="setting-label">Max Test Duration (seconds)</div>
                                <div class="setting-control">
                                    <input type="number" id="max-duration" value="30" min="5" max="300">
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Parallel Test Execution</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="parallel-execution" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Auto-fix Issues</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="auto-fix">
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                        </div>
                        
                        <div class="settings-group">
                            <h5>Notification Settings</h5>
                            <div class="setting-item">
                                <div class="setting-label">Show Success Notifications</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="show-success" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Show Error Notifications</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="show-errors" checked>
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                            <div class="setting-item">
                                <div class="setting-label">Email Test Reports</div>
                                <div class="setting-control">
                                    <input type="checkbox" id="email-reports">
                                    <div class="setting-toggle"></div>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="test-controls">
                        <button class="test-action-button primary" id="save-settings">
                            <i class="icon-save"></i>
                            Save Settings
                        </button>
                        <button class="test-action-button secondary" id="reset-settings">
                            <i class="icon-refresh-cw"></i>
                            Reset to Defaults
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    setupTestingEventListeners() {
        // Tab switching
        document.querySelectorAll('.testing-tab-button').forEach(button => {
            button.addEventListener('click', (e) => {
                this.switchTestingTab(e.target.closest('.testing-tab-button').dataset.tab);
            });
        });
        
        // Source selection
        document.querySelectorAll('.source-option').forEach(option => {
            option.addEventListener('click', (e) => {
                this.selectConfigSource(e.target.closest('.source-option').dataset.source);
            });
        });
        
        // Category selection
        document.querySelectorAll('.category-checkbox').forEach(checkbox => {
            checkbox.addEventListener('click', (e) => {
                const input = checkbox.querySelector('input[type="checkbox"]');
                input.checked = !input.checked;
                checkbox.closest('.category-card').classList.toggle('selected', input.checked);
            });
        });
        
        // File upload
        const fileUploadZone = document.getElementById('test-file-upload');
        const fileInput = document.getElementById('test-config-file');
        
        if (fileUploadZone && fileInput) {
            fileUploadZone.addEventListener('click', () => fileInput.click());
            fileUploadZone.addEventListener('dragover', (e) => {
                e.preventDefault();
                fileUploadZone.classList.add('dragover');
            });
            fileUploadZone.addEventListener('dragleave', () => {
                fileUploadZone.classList.remove('dragover');
            });
            fileUploadZone.addEventListener('drop', (e) => {
                e.preventDefault();
                fileUploadZone.classList.remove('dragover');
                if (e.dataTransfer.files.length > 0) {
                    this.handleFileUpload(e.dataTransfer.files[0]);
                }
            });
            fileInput.addEventListener('change', (e) => {
                if (e.target.files.length > 0) {
                    this.handleFileUpload(e.target.files[0]);
                }
            });
        }
        
        // Settings toggles
        document.querySelectorAll('.setting-control input[type="checkbox"]').forEach(checkbox => {
            checkbox.addEventListener('change', (e) => {
                this.updateSettingToggle(e.target);
            });
        });
    }

    switchTestingTab(tabId) {
        // Update buttons
        document.querySelectorAll('.testing-tab-button').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabId}"]`).classList.add('active');
        
        // Update content
        document.querySelectorAll('.testing-tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(tabId).classList.add('active');
    }

    selectConfigSource(source) {
        // Update radio selection
        document.querySelectorAll('.source-option').forEach(option => {
            option.classList.remove('selected');
        });
        document.querySelector(`[data-source="${source}"]`).classList.add('selected');
        
        // Update radio input
        document.querySelector(`input[value="${source}"]`).checked = true;
        
        // Show/hide relevant sections
        document.getElementById('test-file-upload').style.display = source === 'file' ? 'block' : 'none';
        document.getElementById('test-template-selector').style.display = source === 'template' ? 'block' : 'none';
    }

    handleFileUpload(file) {
        console.log('File uploaded:', file.name);
        // Handle file upload logic here
    }

    updateSettingToggle(checkbox) {
        const toggle = checkbox.nextElementSibling;
        toggle.classList.toggle('checked', checkbox.checked);
    }

    setupDashboardIntegration() {
        if (!this.dashboard) return;
        
        // Add testing status to dashboard
        this.addTestingStatusToDashboard();
        
        // Add quick test button to dashboard
        this.addQuickTestButtonToDashboard();
    }

    addTestingStatusToDashboard() {
        const dashboardStatus = document.querySelector('.dashboard-status');
        if (!dashboardStatus) return;
        
        const testingStatus = document.createElement('div');
        testingStatus.className = 'status-item testing-status-item';
        testingStatus.innerHTML = `
            <span class="status-label">Testing Status:</span>
            <span class="status-value pending" id="testing-dashboard-status">
                <i class="icon-clock"></i>
                Not Run
            </span>
        `;
        
        dashboardStatus.appendChild(testingStatus);
    }

    addQuickTestButtonToDashboard() {
        const quickActions = document.querySelector('.quick-actions .actions');
        if (!quickActions) return;
        
        const testButton = document.createElement('button');
        testButton.className = 'action-btn';
        testButton.id = 'dashboard-quick-test';
        testButton.innerHTML = `
            <i class="icon-check-circle"></i>
            Quick Test
        `;
        
        testButton.addEventListener('click', () => {
            this.runQuickTestFromDashboard();
        });
        
        quickActions.appendChild(testButton);
    }

    async runQuickTestFromDashboard() {
        if (!this.testingFramework) return;
        
        // Update dashboard status
        const status = document.getElementById('testing-dashboard-status');
        if (status) {
            status.innerHTML = `
                <i class="icon-refresh-cw"></i>
                Running...
            `;
            status.className = 'status-value in-progress';
        }
        
        try {
            // Run quick test
            const results = await this.testingFramework.runQuickTest();
            
            // Update status based on results
            if (status) {
                if (results.failedTests === 0) {
                    status.innerHTML = `
                        <i class="icon-check-circle"></i>
                        All Tests Passed
                    `;
                    status.className = 'status-value success';
                } else {
                    status.innerHTML = `
                        <i class="icon-alert-circle"></i>
                        ${results.failedTests} Tests Failed
                    `;
                    status.className = 'status-value warning';
                }
            }
            
            // Show notification
            this.showTestingNotification('Quick test completed', 'success');
            
        } catch (error) {
            console.error('Quick test failed:', error);
            
            if (status) {
                status.innerHTML = `
                    <i class="icon-x-circle"></i>
                    Test Failed
                `;
                status.className = 'status-value error';
            }
            
            this.showTestingNotification('Quick test failed', 'error');
        }
    }

    setupValidationIntegration() {
        if (!this.validationSystem) return;
        
        // Integrate with existing validation system
        // This allows validation to trigger testing automatically
    }

    showTestingInterface() {
        const framework = document.getElementById('testing-framework');
        if (framework) {
            framework.style.display = 'block';
        }
    }

    hideTestingInterface() {
        const framework = document.getElementById('testing-framework');
        if (framework) {
            framework.style.display = 'none';
        }
    }

    showTestingNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <i class="icon-${type === 'success' ? 'check' : type === 'error' ? 'alert' : 'info'}"></i>
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

    async delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    // Public API
    isInitialized() {
        return this.isInitialized;
    }

    getTestingFramework() {
        return this.testingFramework;
    }

    async runQuickTest() {
        if (this.testingFramework) {
            return await this.testingFramework.runQuickTest();
        }
        return null;
    }

    async runFullTest() {
        if (this.testingFramework) {
            return await this.testingFramework.runFullTestSuite();
        }
        return null;
    }
}

// Initialize testing integration when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.testingIntegration = new TestingFrameworkIntegration();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TestingFrameworkIntegration;
}