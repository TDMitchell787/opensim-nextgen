// OpenSim Next Auto-Configurator - USB Key Integration
// Integration layer between USB key management and dashboard system

class UsbIntegration {
    constructor() {
        this.dashboard = null;
        this.usbManager = null;
        this.isInitialized = false;
        this.securityLevel = 'standard';
        
        // Auto-save settings
        this.autoSaveEnabled = false;
        this.autoSaveInterval = null;
        this.lastAutoSave = 0;
        
        // Security policies
        this.securityPolicies = {
            requireUsbForProduction: true,
            requireEncryptionForSensitive: true,
            autoLockTimeout: 300000, // 5 minutes
            maxPasswordAttempts: 3,
            passwordAttempts: 0
        };
        
        this.initializeIntegration();
    }

    async initializeIntegration() {
        try {
            // Wait for components to be available
            await this.waitForComponents();
            
            // Initialize dashboard integration
            this.setupDashboardIntegration();
            
            // Initialize USB manager integration
            this.setupUsbManagerIntegration();
            
            // Setup security monitoring
            this.setupSecurityMonitoring();
            
            // Setup auto-save functionality
            this.setupAutoSave();
            
            this.isInitialized = true;
            console.log('✅ USB integration initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize USB integration:', error);
        }
    }

    async waitForComponents() {
        // Wait for dashboard to be available
        while (!window.configDashboard) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        this.dashboard = window.configDashboard;
        
        // Wait for USB manager to be available
        while (!window.usbKeyManager) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        this.usbManager = window.usbKeyManager;
    }

    setupDashboardIntegration() {
        // Add USB key management to dashboard
        this.addUsbToDashboard();
        
        // Listen for configuration changes
        document.addEventListener('configurationChanged', (event) => {
            this.handleConfigurationChange(event.detail);
        });
        
        // Listen for security status changes
        document.addEventListener('securityStatusChanged', (event) => {
            this.handleSecurityStatusChange(event.detail);
        });
    }

    setupUsbManagerIntegration() {
        // Listen for USB device events
        if (this.usbManager.webUSBSupported) {
            navigator.usb.addEventListener('connect', (event) => {
                this.handleUsbDeviceConnected(event.device);
            });
            
            navigator.usb.addEventListener('disconnect', (event) => {
                this.handleUsbDeviceDisconnected(event.device);
            });
        }
        
        // Listen for credential events
        document.addEventListener('credentialsStored', (event) => {
            this.handleCredentialsStored(event.detail);
        });
        
        document.addEventListener('credentialsLoaded', (event) => {
            this.handleCredentialsLoaded(event.detail);
        });
    }

    setupSecurityMonitoring() {
        // Monitor for sensitive configuration changes
        this.monitorSensitiveData();
        
        // Setup auto-lock timer
        this.setupAutoLock();
        
        // Monitor failed authentication attempts
        this.monitorAuthenticationAttempts();
    }

    setupAutoSave() {
        // Setup auto-save interval if enabled
        if (this.autoSaveEnabled) {
            this.autoSaveInterval = setInterval(() => {
                this.performAutoSave();
            }, 30000); // Auto-save every 30 seconds
        }
    }

    addUsbToDashboard() {
        // Add USB security section to dashboard
        const dashboardContainer = document.querySelector('.dashboard-enhanced');
        if (!dashboardContainer) return;

        const usbSection = document.createElement('div');
        usbSection.className = 'dashboard-section usb-security-section';
        usbSection.innerHTML = `
            <div class="section-header">
                <div class="section-title">
                    <i class="icon-shield"></i>
                    <h4>USB Security</h4>
                </div>
                <div class="section-actions">
                    <button class="btn btn-sm btn-secondary" id="configure-usb-security">
                        <i class="icon-settings"></i>
                        Configure
                    </button>
                </div>
            </div>
            <div class="section-content">
                <div class="usb-status-grid">
                    <div class="status-card">
                        <div class="status-icon">
                            <i class="icon-usb"></i>
                        </div>
                        <div class="status-info">
                            <span class="status-label">USB Devices</span>
                            <span class="status-value" id="usb-device-count">0</span>
                        </div>
                    </div>
                    <div class="status-card">
                        <div class="status-icon">
                            <i class="icon-key"></i>
                        </div>
                        <div class="status-info">
                            <span class="status-label">Stored Credentials</span>
                            <span class="status-value" id="stored-credentials">None</span>
                        </div>
                    </div>
                    <div class="status-card">
                        <div class="status-icon">
                            <i class="icon-shield-check"></i>
                        </div>
                        <div class="status-info">
                            <span class="status-label">Security Level</span>
                            <span class="status-value" id="security-level">${this.securityLevel}</span>
                        </div>
                    </div>
                </div>
                <div class="usb-quick-actions">
                    <button class="btn btn-sm btn-primary" id="quick-store-credentials">
                        <i class="icon-save"></i>
                        Quick Store
                    </button>
                    <button class="btn btn-sm btn-info" id="quick-load-credentials">
                        <i class="icon-upload"></i>
                        Quick Load
                    </button>
                    <button class="btn btn-sm btn-warning" id="lock-credentials">
                        <i class="icon-lock"></i>
                        Lock
                    </button>
                </div>
            </div>
        `;

        // Insert before existing sections
        const existingSections = dashboardContainer.querySelector('.dashboard-sections');
        if (existingSections) {
            existingSections.parentNode.insertBefore(usbSection, existingSections);
        }

        this.setupUsbSectionEventListeners();
    }

    setupUsbSectionEventListeners() {
        document.getElementById('configure-usb-security')?.addEventListener('click', () => {
            this.showUsbSecurityConfiguration();
        });

        document.getElementById('quick-store-credentials')?.addEventListener('click', () => {
            this.quickStoreCredentials();
        });

        document.getElementById('quick-load-credentials')?.addEventListener('click', () => {
            this.quickLoadCredentials();
        });

        document.getElementById('lock-credentials')?.addEventListener('click', () => {
            this.lockCredentials();
        });
    }

    handleConfigurationChange(changeData) {
        // Check if sensitive data has changed
        if (this.isSensitiveData(changeData)) {
            this.updateSecurityLevel();
            this.suggestCredentialStorage();
        }
        
        // Update dashboard indicators
        this.updateDashboardIndicators();
    }

    handleSecurityStatusChange(statusData) {
        this.securityLevel = statusData.level || 'standard';
        this.updateSecurityIndicators();
        
        if (statusData.level === 'high' && this.securityPolicies.requireUsbForProduction) {
            this.suggestUsbSecurity();
        }
    }

    handleUsbDeviceConnected(device) {
        console.log('USB device connected:', device.productName);
        this.updateUsbDeviceCount();
        
        // Analyze device for security capabilities
        this.analyzeDeviceSecurity(device);
        
        // Show notification
        this.showSecurityNotification(
            `USB device connected: ${device.productName}`,
            'info'
        );
    }

    handleUsbDeviceDisconnected(device) {
        console.log('USB device disconnected:', device.productName);
        this.updateUsbDeviceCount();
        
        // Check if stored credentials were on this device
        this.checkCredentialAccess();
        
        // Show notification
        this.showSecurityNotification(
            `USB device disconnected: ${device.productName}`,
            'warning'
        );
    }

    handleCredentialsStored(credentialData) {
        console.log('Credentials stored:', credentialData.type);
        this.updateStoredCredentialsIndicator();
        
        // Update security level
        this.securityLevel = 'enhanced';
        this.updateSecurityIndicators();
        
        // Show success notification
        this.showSecurityNotification(
            'Credentials securely stored',
            'success'
        );
        
        // Log security event
        this.logSecurityEvent('Credentials stored', 'success', credentialData);
    }

    handleCredentialsLoaded(credentialData) {
        console.log('Credentials loaded:', credentialData.type);
        
        // Apply credentials to configuration
        this.applyLoadedCredentials(credentialData);
        
        // Update dashboard
        if (this.dashboard) {
            this.dashboard.updateProgressIndicators();
        }
        
        // Show success notification
        this.showSecurityNotification(
            'Credentials loaded successfully',
            'success'
        );
        
        // Log security event
        this.logSecurityEvent('Credentials loaded', 'success', credentialData);
    }

    async quickStoreCredentials() {
        try {
            // Gather current sensitive configuration
            const sensitiveData = this.gatherSensitiveConfiguration();
            
            if (Object.keys(sensitiveData).length === 0) {
                this.showSecurityNotification(
                    'No sensitive data to store',
                    'info'
                );
                return;
            }
            
            // Show quick password dialog
            const password = await this.showQuickPasswordDialog('store');
            if (!password) return;
            
            // Store credentials
            await this.usbManager.storeCredentials(
                this.usbManager.getSelectedDevice(),
                sensitiveData,
                password
            );
            
        } catch (error) {
            console.error('Quick store failed:', error);
            this.showSecurityNotification(
                'Failed to store credentials: ' + error.message,
                'error'
            );
        }
    }

    async quickLoadCredentials() {
        try {
            // Show quick password dialog
            const password = await this.showQuickPasswordDialog('load');
            if (!password) return;
            
            // Load credentials
            const credentials = await this.usbManager.loadCredentials(
                this.usbManager.getSelectedDevice(),
                password
            );
            
            // Apply credentials
            this.applyLoadedCredentials(credentials);
            
        } catch (error) {
            console.error('Quick load failed:', error);
            this.showSecurityNotification(
                'Failed to load credentials: ' + error.message,
                'error'
            );
            
            // Track failed attempts
            this.securityPolicies.passwordAttempts++;
            if (this.securityPolicies.passwordAttempts >= this.securityPolicies.maxPasswordAttempts) {
                this.lockCredentials();
            }
        }
    }

    lockCredentials() {
        // Clear sensitive data from memory
        this.clearSensitiveData();
        
        // Disable credential access
        this.setCredentialAccessLocked(true);
        
        // Update UI
        this.updateLockStatus(true);
        
        // Show notification
        this.showSecurityNotification(
            'Credentials locked for security',
            'warning'
        );
        
        // Log security event
        this.logSecurityEvent('Credentials locked', 'security', { manual: true });
    }

    isSensitiveData(changeData) {
        const sensitiveKeys = [
            'password', 'apiKey', 'secretKey', 'privateKey',
            'certificatePath', 'encryptionKey', 'token'
        ];
        
        return sensitiveKeys.some(key => 
            JSON.stringify(changeData).toLowerCase().includes(key.toLowerCase())
        );
    }

    gatherSensitiveConfiguration() {
        const config = window.configState || {};
        const sensitive = {};
        
        // Database credentials
        if (config.database) {
            sensitive.database = {
                username: config.database.username,
                password: config.database.password,
                connectionString: config.database.connectionString
            };
        }
        
        // Security keys
        if (config.security) {
            sensitive.security = {
                apiKey: config.security.apiKey,
                encryptionKey: config.security.encryptionKey,
                signingKey: config.security.signingKey,
                sslCertificatePath: config.security.sslCertificatePath,
                sslKeyPath: config.security.sslKeyPath
            };
        }
        
        return sensitive;
    }

    applyLoadedCredentials(credentialData) {
        if (!credentialData || !credentialData.data) return;
        
        const config = window.configState || {};
        
        if (credentialData.type === 'full') {
            // Apply full configuration
            Object.assign(config, credentialData.data);
        } else {
            // Apply specific credential type
            if (config[credentialData.type]) {
                Object.assign(config[credentialData.type], credentialData.data);
            }
        }
        
        // Trigger configuration update
        if (window.updateConfigurationUI) {
            window.updateConfigurationUI();
        }
        
        // Dispatch event
        document.dispatchEvent(new CustomEvent('credentialsApplied', {
            detail: credentialData
        }));
    }

    updateUsbDeviceCount() {
        const countElement = document.getElementById('usb-device-count');
        if (countElement && this.usbManager) {
            countElement.textContent = this.usbManager.keyStorage.size;
        }
    }

    updateStoredCredentialsIndicator() {
        const credentialsElement = document.getElementById('stored-credentials');
        if (credentialsElement) {
            // Check if any devices have credentials
            let hasCredentials = false;
            if (this.usbManager) {
                for (const [device, info] of this.usbManager.keyStorage.entries()) {
                    if (info.hasCredentials) {
                        hasCredentials = true;
                        break;
                    }
                }
            }
            credentialsElement.textContent = hasCredentials ? 'Available' : 'None';
        }
    }

    updateSecurityIndicators() {
        const securityLevelElement = document.getElementById('security-level');
        if (securityLevelElement) {
            securityLevelElement.textContent = this.securityLevel;
            securityLevelElement.className = `status-value security-${this.securityLevel}`;
        }
    }

    updateDashboardIndicators() {
        this.updateUsbDeviceCount();
        this.updateStoredCredentialsIndicator();
        this.updateSecurityIndicators();
    }

    showQuickPasswordDialog(action) {
        return new Promise((resolve) => {
            const dialog = document.createElement('div');
            dialog.className = 'password-dialog-overlay';
            dialog.innerHTML = `
                <div class="password-dialog">
                    <div class="dialog-header">
                        <h4>${action === 'store' ? 'Store' : 'Load'} Credentials</h4>
                    </div>
                    <div class="dialog-content">
                        <p>Enter encryption password:</p>
                        <input type="password" id="quick-password" placeholder="Password">
                        <div class="password-strength" id="quick-password-strength"></div>
                    </div>
                    <div class="dialog-actions">
                        <button class="btn btn-secondary" id="cancel-password">Cancel</button>
                        <button class="btn btn-primary" id="confirm-password">${action === 'store' ? 'Store' : 'Load'}</button>
                    </div>
                </div>
            `;
            
            document.body.appendChild(dialog);
            
            const passwordInput = dialog.querySelector('#quick-password');
            const confirmButton = dialog.querySelector('#confirm-password');
            const cancelButton = dialog.querySelector('#cancel-password');
            
            passwordInput.focus();
            
            const cleanup = () => {
                document.body.removeChild(dialog);
            };
            
            confirmButton.addEventListener('click', () => {
                const password = passwordInput.value;
                cleanup();
                resolve(password);
            });
            
            cancelButton.addEventListener('click', () => {
                cleanup();
                resolve(null);
            });
            
            passwordInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    const password = passwordInput.value;
                    cleanup();
                    resolve(password);
                }
                if (e.key === 'Escape') {
                    cleanup();
                    resolve(null);
                }
            });
        });
    }

    showSecurityNotification(message, type) {
        // Use the USB manager's notification system
        if (this.usbManager) {
            this.usbManager.showMessage(message, type);
        }
    }

    logSecurityEvent(event, status, data = {}) {
        // Add to USB manager's audit log
        if (this.usbManager) {
            this.usbManager.addAuditEntry(event, status);
        }
        
        // Also log to console for debugging
        console.log(`[USB Security] ${event}:`, { status, data });
    }

    // Additional utility methods
    clearSensitiveData() {
        // Clear sensitive configuration data from memory
        if (window.configState) {
            const config = window.configState;
            if (config.database) {
                delete config.database.password;
                delete config.database.username;
            }
            if (config.security) {
                delete config.security.apiKey;
                delete config.security.encryptionKey;
            }
        }
    }

    setCredentialAccessLocked(locked) {
        // Disable credential-related UI elements
        const credentialButtons = document.querySelectorAll(
            '#quick-store-credentials, #quick-load-credentials'
        );
        credentialButtons.forEach(button => {
            button.disabled = locked;
            if (locked) {
                button.classList.add('locked');
            } else {
                button.classList.remove('locked');
            }
        });
    }

    updateLockStatus(locked) {
        const lockButton = document.getElementById('lock-credentials');
        if (lockButton) {
            if (locked) {
                lockButton.innerHTML = '<i class="icon-unlock"></i> Unlock';
                lockButton.onclick = () => this.unlockCredentials();
            } else {
                lockButton.innerHTML = '<i class="icon-lock"></i> Lock';
                lockButton.onclick = () => this.lockCredentials();
            }
        }
    }

    async unlockCredentials() {
        try {
            const password = await this.showQuickPasswordDialog('unlock');
            if (!password) return;
            
            // Verify password by attempting to load credentials
            await this.usbManager.loadCredentials(
                this.usbManager.getSelectedDevice(),
                password
            );
            
            // Reset failed attempts
            this.securityPolicies.passwordAttempts = 0;
            
            // Unlock access
            this.setCredentialAccessLocked(false);
            this.updateLockStatus(false);
            
            this.showSecurityNotification('Credentials unlocked', 'success');
            this.logSecurityEvent('Credentials unlocked', 'success');
            
        } catch (error) {
            this.securityPolicies.passwordAttempts++;
            this.showSecurityNotification('Invalid password', 'error');
            
            if (this.securityPolicies.passwordAttempts >= this.securityPolicies.maxPasswordAttempts) {
                this.showSecurityNotification('Too many failed attempts - access locked', 'error');
            }
        }
    }
}

// Initialize integration when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.usbIntegration = new UsbIntegration();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = UsbIntegration;
}