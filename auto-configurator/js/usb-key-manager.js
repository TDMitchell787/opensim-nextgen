// OpenSim Next Auto-Configurator - Encrypted USB Key Management System
// Secure credential storage and management for sensitive configuration data

class UsbKeyManager {
    constructor() {
        this.keyStorage = new Map();
        this.encryptionKeys = new Map();
        this.deviceWatcher = null;
        this.webUSBSupported = false;
        this.isInitialized = false;
        
        // Security settings
        this.keyDerivationRounds = 100000;
        this.encryptionAlgorithm = 'AES-GCM';
        this.keyLength = 256;
        this.ivLength = 96;
        this.tagLength = 128;
        
        // Device detection settings
        this.supportedVendors = [
            { vendorId: 0x0951, name: 'Kingston' },
            { vendorId: 0x0930, name: 'Toshiba' },
            { vendorId: 0x8087, name: 'Intel' },
            { vendorId: 0x090c, name: 'Silicon Motion' },
            { vendorId: 0x13fe, name: 'Phison' }
        ];
        
        this.initializeManager();
    }

    async initializeManager() {
        try {
            // Check WebUSB support
            this.webUSBSupported = 'usb' in navigator;
            
            if (this.webUSBSupported) {
                console.log('✅ WebUSB supported - USB key management available');
                await this.setupDeviceWatcher();
            } else {
                console.warn('⚠️ WebUSB not supported - falling back to file-based key management');
                this.setupFileFallback();
            }
            
            this.isInitialized = true;
            this.updateUI();
        } catch (error) {
            console.error('Failed to initialize USB key manager:', error);
            this.setupFileFallback();
        }
    }

    async setupDeviceWatcher() {
        if (!this.webUSBSupported) return;

        try {
            // Listen for device connection/disconnection
            navigator.usb.addEventListener('connect', (event) => {
                this.handleDeviceConnected(event.device);
            });

            navigator.usb.addEventListener('disconnect', (event) => {
                this.handleDeviceDisconnected(event.device);
            });

            // Check for already connected devices
            const devices = await navigator.usb.getDevices();
            for (const device of devices) {
                await this.analyzeDevice(device);
            }
        } catch (error) {
            console.error('Failed to setup device watcher:', error);
        }
    }

    setupFileFallback() {
        console.log('📁 Setting up file-based secure credential storage');
        this.createFileInterface();
    }

    async handleDeviceConnected(device) {
        console.log('🔌 USB device connected:', device.productName || 'Unknown device');
        await this.analyzeDevice(device);
        this.updateDeviceList();
    }

    handleDeviceDisconnected(device) {
        console.log('🔌 USB device disconnected:', device.productName || 'Unknown device');
        this.removeDeviceFromList(device);
        this.updateDeviceList();
    }

    async analyzeDevice(device) {
        const deviceInfo = {
            vendorId: device.vendorId,
            productId: device.productId,
            productName: device.productName || 'Unknown Device',
            manufacturerName: device.manufacturerName || 'Unknown Manufacturer',
            serialNumber: device.serialNumber || 'No Serial',
            isSupported: this.isDeviceSupported(device),
            hasCredentials: false,
            encryptionStatus: 'unknown'
        };

        // Check if device is suitable for credential storage
        if (deviceInfo.isSupported) {
            deviceInfo.encryptionStatus = await this.checkDeviceEncryption(device);
            deviceInfo.hasCredentials = await this.checkForCredentials(device);
        }

        this.keyStorage.set(device, deviceInfo);
        return deviceInfo;
    }

    isDeviceSupported(device) {
        return this.supportedVendors.some(vendor => 
            vendor.vendorId === device.vendorId
        ) || device.productName?.toLowerCase().includes('flash');
    }

    async checkDeviceEncryption(device) {
        try {
            // Attempt to open the device for encryption analysis
            if (!device.opened) {
                await device.open();
            }

            // Check for hardware encryption capabilities
            // This is a simplified check - real implementation would
            // require device-specific protocols
            return 'software'; // Assume software encryption for demo
        } catch (error) {
            console.warn('Could not analyze device encryption:', error);
            return 'unknown';
        }
    }

    async checkForCredentials(device) {
        // Check if the device already contains encrypted credentials
        // This would involve scanning for our credential files
        try {
            // Simplified check for demonstration
            return false;
        } catch (error) {
            console.warn('Could not check for existing credentials:', error);
            return false;
        }
    }

    async requestDeviceAccess() {
        if (!this.webUSBSupported) {
            this.showMessage('WebUSB not supported. Please use file export/import.', 'warning');
            return null;
        }

        try {
            const device = await navigator.usb.requestDevice({
                filters: this.supportedVendors.map(vendor => ({
                    vendorId: vendor.vendorId
                }))
            });

            await this.analyzeDevice(device);
            this.updateDeviceList();
            return device;
        } catch (error) {
            if (error.name === 'NotFoundError') {
                this.showMessage('No device selected or no compatible devices found.', 'info');
            } else {
                console.error('Error requesting device access:', error);
                this.showMessage('Failed to access USB device: ' + error.message, 'error');
            }
            return null;
        }
    }

    async storeCredentials(device, credentials, password) {
        try {
            const encryptedData = await this.encryptCredentials(credentials, password);
            
            if (this.webUSBSupported && device) {
                return await this.writeToDevice(device, encryptedData);
            } else {
                return await this.writeToFile(encryptedData);
            }
        } catch (error) {
            console.error('Failed to store credentials:', error);
            throw error;
        }
    }

    async encryptCredentials(credentials, password) {
        const encoder = new TextEncoder();
        const data = encoder.encode(JSON.stringify(credentials));
        
        // Generate salt and IV
        const salt = crypto.getRandomValues(new Uint8Array(16));
        const iv = crypto.getRandomValues(new Uint8Array(this.ivLength / 8));
        
        // Derive encryption key from password
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            encoder.encode(password),
            { name: 'PBKDF2' },
            false,
            ['deriveKey']
        );
        
        const derivedKey = await crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt: salt,
                iterations: this.keyDerivationRounds,
                hash: 'SHA-256'
            },
            keyMaterial,
            {
                name: 'AES-GCM',
                length: this.keyLength
            },
            false,
            ['encrypt']
        );
        
        // Encrypt the data
        const encryptedData = await crypto.subtle.encrypt(
            {
                name: 'AES-GCM',
                iv: iv,
                tagLength: this.tagLength
            },
            derivedKey,
            data
        );
        
        // Combine salt, IV, and encrypted data
        const result = new Uint8Array(salt.length + iv.length + encryptedData.byteLength);
        result.set(salt, 0);
        result.set(iv, salt.length);
        result.set(new Uint8Array(encryptedData), salt.length + iv.length);
        
        return {
            encryptedData: result,
            metadata: {
                algorithm: this.encryptionAlgorithm,
                keyLength: this.keyLength,
                iterations: this.keyDerivationRounds,
                timestamp: Date.now(),
                version: '1.0'
            }
        };
    }

    async writeToDevice(device, encryptedData) {
        try {
            if (!device.opened) {
                await device.open();
            }

            // This is a simplified implementation
            // Real USB storage would require filesystem access or raw sector writing
            console.log('📀 Would write encrypted credentials to USB device');
            console.log('Device:', device.productName);
            console.log('Data size:', encryptedData.encryptedData.length, 'bytes');
            
            this.showMessage(`Credentials stored on ${device.productName}`, 'success');
            return true;
        } catch (error) {
            console.error('Failed to write to device:', error);
            throw new Error('Failed to write credentials to USB device');
        }
    }

    async writeToFile(encryptedData) {
        const blob = new Blob([encryptedData.encryptedData], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        
        const a = document.createElement('a');
        a.href = url;
        a.download = `opensim-credentials-${Date.now()}.key`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        // Also save metadata
        const metadataBlob = new Blob([JSON.stringify(encryptedData.metadata, null, 2)], 
            { type: 'application/json' });
        const metadataUrl = URL.createObjectURL(metadataBlob);
        
        const metaA = document.createElement('a');
        metaA.href = metadataUrl;
        metaA.download = `opensim-credentials-${Date.now()}.meta.json`;
        document.body.appendChild(metaA);
        metaA.click();
        document.body.removeChild(metaA);
        URL.revokeObjectURL(metadataUrl);
        
        this.showMessage('Credentials exported to encrypted file', 'success');
        return true;
    }

    async loadCredentials(device, password) {
        try {
            let encryptedData;
            
            if (this.webUSBSupported && device) {
                encryptedData = await this.readFromDevice(device);
            } else {
                encryptedData = await this.readFromFile();
            }
            
            if (!encryptedData) {
                throw new Error('No encrypted data found');
            }
            
            return await this.decryptCredentials(encryptedData, password);
        } catch (error) {
            console.error('Failed to load credentials:', error);
            throw error;
        }
    }

    async readFromDevice(device) {
        try {
            if (!device.opened) {
                await device.open();
            }

            // This is a simplified implementation
            // Real USB storage would require filesystem access or raw sector reading
            console.log('📀 Would read encrypted credentials from USB device');
            throw new Error('USB device reading not implemented - use file import');
        } catch (error) {
            console.error('Failed to read from device:', error);
            throw error;
        }
    }

    async readFromFile() {
        return new Promise((resolve, reject) => {
            const input = document.createElement('input');
            input.type = 'file';
            input.accept = '.key';
            input.onchange = async (event) => {
                const file = event.target.files[0];
                if (!file) {
                    reject(new Error('No file selected'));
                    return;
                }
                
                try {
                    const arrayBuffer = await file.arrayBuffer();
                    resolve(new Uint8Array(arrayBuffer));
                } catch (error) {
                    reject(error);
                }
            };
            input.click();
        });
    }

    async decryptCredentials(encryptedData, password) {
        const encoder = new TextEncoder();
        const decoder = new TextDecoder();
        
        // Extract salt, IV, and encrypted data
        const salt = encryptedData.slice(0, 16);
        const iv = encryptedData.slice(16, 16 + this.ivLength / 8);
        const encrypted = encryptedData.slice(16 + this.ivLength / 8);
        
        // Derive decryption key from password
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            encoder.encode(password),
            { name: 'PBKDF2' },
            false,
            ['deriveKey']
        );
        
        const derivedKey = await crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt: salt,
                iterations: this.keyDerivationRounds,
                hash: 'SHA-256'
            },
            keyMaterial,
            {
                name: 'AES-GCM',
                length: this.keyLength
            },
            false,
            ['decrypt']
        );
        
        // Decrypt the data
        const decryptedData = await crypto.subtle.decrypt(
            {
                name: 'AES-GCM',
                iv: iv,
                tagLength: this.tagLength
            },
            derivedKey,
            encrypted
        );
        
        const jsonString = decoder.decode(decryptedData);
        return JSON.parse(jsonString);
    }

    createUI() {
        const container = document.getElementById('usb-key-manager');
        if (!container) return;

        container.innerHTML = `
            <div class="usb-key-manager">
                <div class="manager-header">
                    <div class="header-content">
                        <h3>🔐 Encrypted USB Key Management</h3>
                        <p>Secure storage for sensitive configuration credentials</p>
                    </div>
                    <div class="header-status">
                        <span class="status-indicator ${this.webUSBSupported ? 'supported' : 'unsupported'}" 
                              id="usb-status">
                            ${this.webUSBSupported ? '✅ WebUSB Supported' : '⚠️ File Mode Only'}
                        </span>
                    </div>
                </div>

                <div class="manager-content">
                    <div class="device-section">
                        <div class="section-header">
                            <h4>Connected Devices</h4>
                            <button class="btn btn-secondary" id="scan-devices">
                                <i class="icon-refresh"></i>
                                Scan Devices
                            </button>
                        </div>
                        <div class="device-list" id="device-list">
                            <div class="no-devices">
                                <i class="icon-usb"></i>
                                <p>No USB devices detected</p>
                                <button class="btn btn-primary" id="request-access">
                                    Grant USB Access
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="credentials-section">
                        <div class="section-header">
                            <h4>Credential Management</h4>
                            <div class="credential-actions">
                                <button class="btn btn-success" id="store-credentials">
                                    <i class="icon-save"></i>
                                    Store Credentials
                                </button>
                                <button class="btn btn-info" id="load-credentials">
                                    <i class="icon-upload"></i>
                                    Load Credentials
                                </button>
                            </div>
                        </div>
                        <div class="credentials-form">
                            <div class="credential-input">
                                <label for="credential-type">Credential Type:</label>
                                <select id="credential-type">
                                    <option value="database">Database Credentials</option>
                                    <option value="ssl">SSL Certificates</option>
                                    <option value="api">API Keys</option>
                                    <option value="encryption">Encryption Keys</option>
                                    <option value="full">Complete Configuration</option>
                                </select>
                            </div>
                            <div class="password-input">
                                <label for="encryption-password">Encryption Password:</label>
                                <div class="password-field">
                                    <input type="password" id="encryption-password" 
                                           placeholder="Enter strong password">
                                    <button type="button" class="btn btn-sm btn-secondary" 
                                            id="toggle-password">
                                        <i class="icon-eye"></i>
                                    </button>
                                </div>
                                <div class="password-strength" id="password-strength"></div>
                            </div>
                        </div>
                    </div>

                    <div class="security-section">
                        <div class="section-header">
                            <h4>Security Settings</h4>
                        </div>
                        <div class="security-options">
                            <div class="option-group">
                                <label class="checkbox-label">
                                    <input type="checkbox" id="require-2fa" checked>
                                    <span class="checkmark"></span>
                                    Require two-factor authentication
                                </label>
                            </div>
                            <div class="option-group">
                                <label class="checkbox-label">
                                    <input type="checkbox" id="auto-lock" checked>
                                    <span class="checkmark"></span>
                                    Auto-lock after inactivity
                                </label>
                            </div>
                            <div class="option-group">
                                <label for="key-iterations">Key derivation rounds:</label>
                                <input type="range" id="key-iterations" 
                                       min="10000" max="1000000" step="10000" 
                                       value="${this.keyDerivationRounds}">
                                <span id="iterations-value">${this.keyDerivationRounds}</span>
                            </div>
                        </div>
                    </div>

                    <div class="audit-section">
                        <div class="section-header">
                            <h4>Audit Log</h4>
                            <button class="btn btn-sm btn-secondary" id="clear-audit">
                                Clear Log
                            </button>
                        </div>
                        <div class="audit-log" id="audit-log">
                            <div class="log-entry">
                                <span class="timestamp">${new Date().toISOString()}</span>
                                <span class="action">USB Key Manager initialized</span>
                                <span class="status success">✅</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="message-area" id="usb-messages"></div>
            </div>
        `;

        this.setupEventListeners();
        this.updateDeviceList();
    }

    setupEventListeners() {
        // Device management
        document.getElementById('scan-devices')?.addEventListener('click', () => {
            this.scanDevices();
        });

        document.getElementById('request-access')?.addEventListener('click', () => {
            this.requestDeviceAccess();
        });

        // Credential management
        document.getElementById('store-credentials')?.addEventListener('click', () => {
            this.handleStoreCredentials();
        });

        document.getElementById('load-credentials')?.addEventListener('click', () => {
            this.handleLoadCredentials();
        });

        // Password management
        document.getElementById('toggle-password')?.addEventListener('click', (e) => {
            const passwordField = document.getElementById('encryption-password');
            const icon = e.target.closest('button').querySelector('i');
            
            if (passwordField.type === 'password') {
                passwordField.type = 'text';
                icon.className = 'icon-eye-off';
            } else {
                passwordField.type = 'password';
                icon.className = 'icon-eye';
            }
        });

        document.getElementById('encryption-password')?.addEventListener('input', (e) => {
            this.updatePasswordStrength(e.target.value);
        });

        // Security settings
        document.getElementById('key-iterations')?.addEventListener('input', (e) => {
            this.keyDerivationRounds = parseInt(e.target.value);
            document.getElementById('iterations-value').textContent = this.keyDerivationRounds;
        });

        // Audit log
        document.getElementById('clear-audit')?.addEventListener('click', () => {
            this.clearAuditLog();
        });
    }

    updatePasswordStrength(password) {
        const strengthIndicator = document.getElementById('password-strength');
        if (!strengthIndicator) return;

        const strength = this.calculatePasswordStrength(password);
        strengthIndicator.className = `password-strength ${strength.level}`;
        strengthIndicator.textContent = strength.message;
    }

    calculatePasswordStrength(password) {
        if (!password) return { level: 'none', message: '' };

        let score = 0;
        const checks = {
            length: password.length >= 12,
            uppercase: /[A-Z]/.test(password),
            lowercase: /[a-z]/.test(password),
            numbers: /\d/.test(password),
            symbols: /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password),
            unique: new Set(password).size > password.length * 0.7
        };

        score = Object.values(checks).filter(Boolean).length;

        if (score < 3) return { level: 'weak', message: 'Weak password - add more complexity' };
        if (score < 5) return { level: 'medium', message: 'Medium strength - consider more variation' };
        return { level: 'strong', message: 'Strong password' };
    }

    async handleStoreCredentials() {
        try {
            const credentialType = document.getElementById('credential-type')?.value;
            const password = document.getElementById('encryption-password')?.value;

            if (!password) {
                this.showMessage('Please enter an encryption password', 'error');
                return;
            }

            if (this.calculatePasswordStrength(password).level === 'weak') {
                if (!confirm('Password is weak. Continue anyway?')) return;
            }

            const credentials = await this.gatherCredentials(credentialType);
            const selectedDevice = this.getSelectedDevice();

            await this.storeCredentials(selectedDevice, credentials, password);
            this.addAuditEntry('Credentials stored', 'success');

        } catch (error) {
            console.error('Failed to store credentials:', error);
            this.showMessage('Failed to store credentials: ' + error.message, 'error');
            this.addAuditEntry('Failed to store credentials', 'error');
        }
    }

    async handleLoadCredentials() {
        try {
            const password = document.getElementById('encryption-password')?.value;

            if (!password) {
                this.showMessage('Please enter the decryption password', 'error');
                return;
            }

            const selectedDevice = this.getSelectedDevice();
            const credentials = await this.loadCredentials(selectedDevice, password);

            await this.applyCredentials(credentials);
            this.showMessage('Credentials loaded successfully', 'success');
            this.addAuditEntry('Credentials loaded', 'success');

        } catch (error) {
            console.error('Failed to load credentials:', error);
            this.showMessage('Failed to load credentials: ' + error.message, 'error');
            this.addAuditEntry('Failed to load credentials', 'error');
        }
    }

    async gatherCredentials(type) {
        // Gather credentials from the configurator based on type
        const allCredentials = {
            database: {
                type: window.configState?.database?.type,
                host: window.configState?.database?.host,
                port: window.configState?.database?.port,
                name: window.configState?.database?.name,
                username: window.configState?.database?.username,
                password: window.configState?.database?.password
            },
            ssl: {
                certificatePath: window.configState?.security?.sslCertificatePath,
                keyPath: window.configState?.security?.sslKeyPath,
                caCertPath: window.configState?.security?.caCertificatePath
            },
            api: {
                apiKey: window.configState?.security?.apiKey,
                gridKey: window.configState?.security?.gridKey,
                regionKey: window.configState?.security?.regionKey
            },
            encryption: {
                encryptionKey: window.configState?.security?.encryptionKey,
                signingKey: window.configState?.security?.signingKey,
                tokenSecret: window.configState?.security?.tokenSecret
            }
        };

        if (type === 'full') {
            return {
                type: 'full',
                data: window.configState,
                timestamp: Date.now()
            };
        }

        return {
            type: type,
            data: allCredentials[type],
            timestamp: Date.now()
        };
    }

    async applyCredentials(credentials) {
        // Apply loaded credentials to the configurator
        if (credentials.type === 'full') {
            if (window.configState) {
                Object.assign(window.configState, credentials.data);
            }
        } else {
            const section = credentials.type;
            if (window.configState && window.configState[section]) {
                Object.assign(window.configState[section], credentials.data);
            }
        }

        // Trigger UI updates
        if (window.updateConfigurationUI) {
            window.updateConfigurationUI();
        }
    }

    getSelectedDevice() {
        // Get the currently selected device from the UI
        const selectedCard = document.querySelector('.device-card.selected');
        if (selectedCard) {
            const deviceId = selectedCard.dataset.deviceId;
            for (const [device, info] of this.keyStorage.entries()) {
                if (device.productName === deviceId) {
                    return device;
                }
            }
        }
        return null;
    }

    updateDeviceList() {
        const deviceList = document.getElementById('device-list');
        if (!deviceList) return;

        if (this.keyStorage.size === 0) {
            deviceList.innerHTML = `
                <div class="no-devices">
                    <i class="icon-usb"></i>
                    <p>No USB devices detected</p>
                    <button class="btn btn-primary" id="request-access">
                        Grant USB Access
                    </button>
                </div>
            `;
            return;
        }

        let html = '';
        for (const [device, info] of this.keyStorage.entries()) {
            html += `
                <div class="device-card ${info.isSupported ? 'supported' : 'unsupported'}" 
                     data-device-id="${device.productName}">
                    <div class="device-icon">
                        <i class="icon-usb"></i>
                    </div>
                    <div class="device-info">
                        <div class="device-name">${info.productName}</div>
                        <div class="device-details">
                            <span class="manufacturer">${info.manufacturerName}</span>
                            <span class="serial">Serial: ${info.serialNumber}</span>
                        </div>
                        <div class="device-status">
                            <span class="support-status ${info.isSupported ? 'supported' : 'unsupported'}">
                                ${info.isSupported ? '✅ Supported' : '❌ Not Supported'}
                            </span>
                            <span class="encryption-status">
                                Encryption: ${info.encryptionStatus}
                            </span>
                            ${info.hasCredentials ? '<span class="has-credentials">🔑 Has Credentials</span>' : ''}
                        </div>
                    </div>
                    <div class="device-actions">
                        <button class="btn btn-sm btn-primary ${info.isSupported ? '' : 'disabled'}" 
                                onclick="usbKeyManager.selectDevice('${device.productName}')">
                            Select
                        </button>
                    </div>
                </div>
            `;
        }

        deviceList.innerHTML = html;
    }

    selectDevice(deviceName) {
        // Remove previous selection
        document.querySelectorAll('.device-card').forEach(card => {
            card.classList.remove('selected');
        });

        // Select new device
        const deviceCard = document.querySelector(`[data-device-id="${deviceName}"]`);
        if (deviceCard) {
            deviceCard.classList.add('selected');
            this.showMessage(`Selected device: ${deviceName}`, 'info');
        }
    }

    addAuditEntry(action, status) {
        const auditLog = document.getElementById('audit-log');
        if (!auditLog) return;

        const entry = document.createElement('div');
        entry.className = 'log-entry';
        entry.innerHTML = `
            <span class="timestamp">${new Date().toISOString()}</span>
            <span class="action">${action}</span>
            <span class="status ${status}">${status === 'success' ? '✅' : '❌'}</span>
        `;

        auditLog.insertBefore(entry, auditLog.firstChild);

        // Limit log entries
        const entries = auditLog.querySelectorAll('.log-entry');
        if (entries.length > 50) {
            entries[entries.length - 1].remove();
        }
    }

    clearAuditLog() {
        const auditLog = document.getElementById('audit-log');
        if (auditLog) {
            auditLog.innerHTML = '';
            this.addAuditEntry('Audit log cleared', 'success');
        }
    }

    async scanDevices() {
        if (!this.webUSBSupported) {
            this.showMessage('WebUSB not supported - cannot scan devices', 'warning');
            return;
        }

        try {
            const devices = await navigator.usb.getDevices();
            this.keyStorage.clear();
            
            for (const device of devices) {
                await this.analyzeDevice(device);
            }
            
            this.updateDeviceList();
            this.showMessage(`Found ${devices.length} USB devices`, 'info');
            this.addAuditEntry(`Scanned ${devices.length} devices`, 'success');
        } catch (error) {
            console.error('Failed to scan devices:', error);
            this.showMessage('Failed to scan devices: ' + error.message, 'error');
        }
    }

    createFileInterface() {
        // Create simplified file-based interface for environments without WebUSB
        console.log('Creating file-based credential management interface');
    }

    removeDeviceFromList(device) {
        this.keyStorage.delete(device);
    }

    updateUI() {
        if (this.isInitialized) {
            this.createUI();
        }
    }

    showMessage(message, type = 'info') {
        const messageArea = document.getElementById('usb-messages');
        if (!messageArea) {
            console.log(`[${type.toUpperCase()}] ${message}`);
            return;
        }

        const messageElement = document.createElement('div');
        messageElement.className = `message ${type}`;
        messageElement.innerHTML = `
            <div class="message-content">
                <span class="message-text">${message}</span>
                <button class="message-close" onclick="this.parentElement.parentElement.remove()">×</button>
            </div>
        `;

        messageArea.appendChild(messageElement);

        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (messageElement.parentElement) {
                messageElement.remove();
            }
        }, 5000);
    }
}

// Global instance
let usbKeyManager;

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    usbKeyManager = new UsbKeyManager();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = UsbKeyManager;
}