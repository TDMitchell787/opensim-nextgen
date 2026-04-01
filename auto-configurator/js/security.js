// OpenSim Next Auto-Configurator - Security Management
// Secure handling of cryptographic materials and sensitive configuration

class SecurityManager {
    constructor() {
        this.cryptoSupported = this.checkCryptoSupport();
        this.keyStorage = new SecureKeyStorage();
        this.certificateManager = new CertificateManager();
        this.usbKeyManager = new USBKeyManager();
        this.securityValidator = new SecurityValidator();
        
        this.init();
    }

    init() {
        this.setupSecurityEventListeners();
        this.initializeSecurityChecks();
        
        console.log('Security Manager initialized', {
            cryptoSupported: this.cryptoSupported,
            secureContext: this.isSecureContext()
        });
    }

    checkCryptoSupport() {
        return {
            webCrypto: typeof crypto !== 'undefined' && typeof crypto.subtle !== 'undefined',
            randomValues: typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function',
            textEncoder: typeof TextEncoder !== 'undefined',
            textDecoder: typeof TextDecoder !== 'undefined',
            fileApi: typeof File !== 'undefined' && typeof FileReader !== 'undefined'
        };
    }

    isSecureContext() {
        return window.isSecureContext;
    }

    setupSecurityEventListeners() {
        // SSL certificate upload
        document.addEventListener('change', (e) => {
            if (e.target.type === 'file' && e.target.id?.includes('ssl')) {
                this.handleCertificateUpload(e.target);
            }
        });

        // USB key detection
        if ('usb' in navigator) {
            navigator.usb.addEventListener('connect', (e) => {
                this.handleUSBConnect(e.device);
            });

            navigator.usb.addEventListener('disconnect', (e) => {
                this.handleUSBDisconnect(e.device);
            });
        }

        // Security validation on form changes
        document.addEventListener('input', (e) => {
            if (e.target.form?.id === 'security-form') {
                this.validateSecurityInput(e.target);
            }
        });
    }

    // Certificate Management
    async handleCertificateUpload(fileInput) {
        const files = Array.from(fileInput.files);
        
        for (const file of files) {
            try {
                await this.validateCertificateFile(file);
                await this.processCertificateFile(file, fileInput.id);
                
                this.showSecurityNotification('Certificate uploaded successfully', 'success');
            } catch (error) {
                this.showSecurityNotification(`Certificate upload failed: ${error.message}`, 'error');
            }
        }
    }

    async validateCertificateFile(file) {
        // Validate file type and basic structure
        const validExtensions = ['.pem', '.crt', '.cer', '.key', '.p12', '.pfx'];
        const extension = '.' + file.name.split('.').pop().toLowerCase();
        
        if (!validExtensions.includes(extension)) {
            throw new Error(`Unsupported certificate file type: ${extension}`);
        }

        // Check file size (reasonable limits)
        const maxSize = 1024 * 1024; // 1MB
        if (file.size > maxSize) {
            throw new Error('Certificate file too large (max 1MB)');
        }

        // Basic content validation for PEM files
        if (['.pem', '.crt', '.cer', '.key'].includes(extension)) {
            const content = await this.readFileAsText(file);
            
            if (extension === '.key' || file.name.toLowerCase().includes('key')) {
                if (!content.includes('BEGIN PRIVATE KEY') && 
                    !content.includes('BEGIN RSA PRIVATE KEY') &&
                    !content.includes('BEGIN EC PRIVATE KEY')) {
                    throw new Error('Invalid private key file format');
                }
            } else {
                if (!content.includes('BEGIN CERTIFICATE')) {
                    throw new Error('Invalid certificate file format');
                }
            }
        }
    }

    async processCertificateFile(file, inputId) {
        const content = await this.readFileAsText(file);
        
        // Store certificate metadata (not the actual content)
        const metadata = {
            filename: file.name,
            size: file.size,
            uploadDate: new Date().toISOString(),
            type: this.detectCertificateType(file.name, content),
            fingerprint: await this.calculateFingerprint(content)
        };

        // Store in secure storage
        await this.keyStorage.storeCertificateMetadata(inputId, metadata);
        
        // Update UI
        this.updateCertificateStatus(inputId, metadata);
        
        // Validate certificate chain if both cert and key are present
        this.validateCertificateChain();
    }

    detectCertificateType(filename, content) {
        const name = filename.toLowerCase();
        
        if (name.includes('key') || content.includes('PRIVATE KEY')) {
            return 'private_key';
        } else if (name.includes('ca') || content.includes('BEGIN CERTIFICATE')) {
            return 'certificate';
        } else if (name.includes('chain')) {
            return 'certificate_chain';
        } else {
            return 'unknown';
        }
    }

    async calculateFingerprint(content) {
        if (!this.cryptoSupported.webCrypto) {
            return 'unavailable';
        }

        try {
            const encoder = new TextEncoder();
            const data = encoder.encode(content);
            const hashBuffer = await crypto.subtle.digest('SHA-256', data);
            const hashArray = Array.from(new Uint8Array(hashBuffer));
            
            return hashArray.map(b => b.toString(16).padStart(2, '0')).join(':');
        } catch (error) {
            console.error('Failed to calculate fingerprint:', error);
            return 'error';
        }
    }

    updateCertificateStatus(inputId, metadata) {
        const statusElement = document.getElementById(`${inputId}-status`);
        if (statusElement) {
            statusElement.innerHTML = `
                <div class="certificate-status success">
                    <i class="icon-check"></i>
                    <span>${metadata.filename}</span>
                    <div class="certificate-details">
                        <small>Type: ${metadata.type}</small>
                        <small>Size: ${this.formatFileSize(metadata.size)}</small>
                        <small>Fingerprint: ${metadata.fingerprint.substring(0, 16)}...</small>
                    </div>
                </div>
            `;
        }
    }

    // USB Key Management
    async requestUSBDevice() {
        if (!('usb' in navigator)) {
            throw new Error('WebUSB not supported in this browser');
        }

        try {
            const device = await navigator.usb.requestDevice({
                filters: [
                    { vendorId: 0x1050 }, // Yubico
                    { vendorId: 0x20A0 }, // Nitrokey
                    { vendorId: 0x2581 }, // SoloKeys
                    { vendorId: 0x096E }  // Feitian
                ]
            });

            await this.setupUSBDevice(device);
            return device;
        } catch (error) {
            console.error('USB device request failed:', error);
            throw new Error('Failed to connect to USB security device');
        }
    }

    async setupUSBDevice(device) {
        try {
            await device.open();
            
            if (device.configuration === null) {
                await device.selectConfiguration(1);
            }

            await device.claimInterface(0);
            
            const deviceInfo = {
                productName: device.productName,
                manufacturerName: device.manufacturerName,
                serialNumber: device.serialNumber,
                vendorId: device.vendorId,
                productId: device.productId
            };

            await this.keyStorage.storeUSBDeviceInfo(deviceInfo);
            this.showSecurityNotification(`USB security device connected: ${deviceInfo.productName}`, 'success');
            
        } catch (error) {
            console.error('USB device setup failed:', error);
            throw new Error('Failed to setup USB security device');
        }
    }

    handleUSBConnect(device) {
        console.log('USB device connected:', device);
        this.updateUSBStatus('connected', device);
    }

    handleUSBDisconnect(device) {
        console.log('USB device disconnected:', device);
        this.updateUSBStatus('disconnected', device);
    }

    updateUSBStatus(status, device) {
        const statusElement = document.getElementById('usb-status');
        if (statusElement) {
            if (status === 'connected') {
                statusElement.innerHTML = `
                    <div class="usb-status connected">
                        <i class="icon-usb"></i>
                        <span>USB Security Device Connected</span>
                        <div class="device-info">
                            <small>${device.productName || 'Unknown Device'}</small>
                        </div>
                    </div>
                `;
            } else {
                statusElement.innerHTML = `
                    <div class="usb-status disconnected">
                        <i class="icon-usb-off"></i>
                        <span>No USB Security Device</span>
                        <button class="btn btn-sm btn-outline" onclick="securityManager.requestUSBDevice()">
                            Connect Device
                        </button>
                    </div>
                `;
            }
        }
    }

    // Security Validation
    validateSecurityConfiguration(config) {
        const validationResults = {
            valid: true,
            errors: [],
            warnings: [],
            recommendations: []
        };

        try {
            // SSL/TLS validation
            this.validateSSLConfiguration(config.security, validationResults);
            
            // Authentication validation
            this.validateAuthenticationConfiguration(config.security, validationResults);
            
            // Network security validation
            this.validateNetworkSecurity(config.network, validationResults);
            
            // Database security validation
            this.validateDatabaseSecurity(config.database, validationResults);
            
            // Deployment-specific security validation
            this.validateDeploymentSecurity(config.deploymentType, config, validationResults);
            
        } catch (error) {
            validationResults.valid = false;
            validationResults.errors.push(`Security validation error: ${error.message}`);
        }

        return validationResults;
    }

    validateSSLConfiguration(security, results) {
        if (security.sslEnabled) {
            if (!security.sslCertPath) {
                results.errors.push('SSL certificate path is required when SSL is enabled');
                results.valid = false;
            }
            
            if (!security.sslKeyPath) {
                results.errors.push('SSL private key path is required when SSL is enabled');
                results.valid = false;
            }
            
            // Check for self-signed certificates in production
            if (security.deploymentType === 'production' && security.selfSigned) {
                results.warnings.push('Self-signed certificates are not recommended for production');
            }
        } else {
            if (security.deploymentType === 'production') {
                results.warnings.push('SSL/TLS is strongly recommended for production deployments');
            }
        }
    }

    validateAuthenticationConfiguration(security, results) {
        const authLevels = ['basic', 'enhanced', 'enterprise'];
        
        if (!authLevels.includes(security.authenticationLevel)) {
            results.errors.push('Invalid authentication level');
            results.valid = false;
        }
        
        // Check password policies
        if (security.authenticationLevel === 'enhanced' || security.authenticationLevel === 'enterprise') {
            if (!security.passwordPolicy) {
                results.warnings.push('Password policy should be defined for enhanced security');
            }
        }
        
        // Two-factor authentication recommendations
        if (security.authenticationLevel === 'enterprise' && !security.twoFactorEnabled) {
            results.recommendations.push('Two-factor authentication is recommended for enterprise security');
        }
    }

    validateNetworkSecurity(network, results) {
        // Port security validation
        if (network.ports) {
            const insecurePorts = [80, 8080, 9000];
            const securityDeployments = ['production', 'grid'];
            
            if (securityDeployments.includes(network.deploymentType)) {
                Object.entries(network.ports).forEach(([service, port]) => {
                    if (insecurePorts.includes(port) && service !== 'http') {
                        results.warnings.push(`Port ${port} for ${service} may be insecure for production`);
                    }
                });
            }
        }
        
        // Firewall recommendations
        if (!network.firewallEnabled) {
            results.recommendations.push('Firewall configuration is recommended for network security');
        }
    }

    validateDatabaseSecurity(database, results) {
        if (database.type === 'postgresql' || database.type === 'mysql') {
            if (!database.sslEnabled) {
                results.warnings.push('Database SSL connections are recommended for production');
            }
            
            if (!database.username || database.username === 'root' || database.username === 'admin') {
                results.warnings.push('Use a dedicated database user with limited privileges');
            }
            
            if (database.host === 'localhost' && database.deploymentType === 'grid') {
                results.warnings.push('Consider using a dedicated database server for grid deployments');
            }
        }
    }

    validateDeploymentSecurity(deploymentType, config, results) {
        switch (deploymentType) {
            case 'production':
                this.validateProductionSecurity(config, results);
                break;
            case 'grid':
                this.validateGridSecurity(config, results);
                break;
            case 'development':
                this.validateDevelopmentSecurity(config, results);
                break;
        }
    }

    validateProductionSecurity(config, results) {
        if (!config.security.sslEnabled) {
            results.errors.push('SSL/TLS is required for production deployments');
            results.valid = false;
        }
        
        if (config.security.authenticationLevel === 'basic') {
            results.warnings.push('Enhanced authentication is recommended for production');
        }
        
        if (!config.monitoring?.enabled) {
            results.warnings.push('Security monitoring should be enabled for production');
        }
    }

    validateGridSecurity(config, results) {
        if (!config.security.zeroTrust) {
            results.recommendations.push('Zero trust networking is recommended for grid deployments');
        }
        
        if (!config.security.encryptionRequired) {
            results.warnings.push('End-to-end encryption should be enabled for grid communications');
        }
        
        if (config.security.authenticationLevel !== 'enterprise') {
            results.recommendations.push('Enterprise authentication is recommended for grid deployments');
        }
    }

    validateDevelopmentSecurity(config, results) {
        if (config.security.sslEnabled && config.security.selfSigned) {
            results.recommendations.push('Self-signed certificates are acceptable for development');
        }
        
        if (!config.security.sslEnabled) {
            results.recommendations.push('Consider enabling SSL even for development to test production configurations');
        }
    }

    // Key Generation and Management
    async generateSSLCertificate(options = {}) {
        if (!this.cryptoSupported.webCrypto) {
            throw new Error('Cryptographic operations not supported in this browser');
        }

        try {
            // Generate key pair
            const keyPair = await crypto.subtle.generateKey(
                {
                    name: 'RSA-PSS',
                    modulusLength: options.keySize || 2048,
                    publicExponent: new Uint8Array([1, 0, 1]),
                    hash: 'SHA-256'
                },
                true,
                ['sign', 'verify']
            );

            // Export keys
            const privateKey = await crypto.subtle.exportKey('pkcs8', keyPair.privateKey);
            const publicKey = await crypto.subtle.exportKey('spki', keyPair.publicKey);

            // Generate certificate request data
            const csrData = {
                subject: {
                    commonName: options.commonName || 'opensim-next.local',
                    organization: options.organization || 'OpenSim Next',
                    country: options.country || 'US'
                },
                keyPair: keyPair,
                extensions: options.extensions || []
            };

            return {
                privateKey: this.arrayBufferToPem(privateKey, 'PRIVATE KEY'),
                publicKey: this.arrayBufferToPem(publicKey, 'PUBLIC KEY'),
                csrData: csrData
            };

        } catch (error) {
            console.error('SSL certificate generation failed:', error);
            throw new Error('Failed to generate SSL certificate');
        }
    }

    arrayBufferToPem(buffer, type) {
        const base64 = btoa(String.fromCharCode(...new Uint8Array(buffer)));
        const formatted = base64.match(/.{1,64}/g).join('\n');
        return `-----BEGIN ${type}-----\n${formatted}\n-----END ${type}-----\n`;
    }

    // Utility methods
    async readFileAsText(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = (e) => resolve(e.target.result);
            reader.onerror = (e) => reject(new Error('Failed to read file'));
            reader.readAsText(file);
        });
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    showSecurityNotification(message, type = 'info') {
        const notification = document.createElement('div');
        notification.className = `security-notification ${type}`;
        notification.innerHTML = `
            <div class="notification-content">
                <i class="icon-${type}"></i>
                <span>${message}</span>
                <button class="notification-close" onclick="this.parentElement.parentElement.remove()">×</button>
            </div>
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            if (notification.parentElement) {
                notification.remove();
            }
        }, 5000);
    }

    initializeSecurityChecks() {
        // Check for secure context
        if (!this.isSecureContext()) {
            this.showSecurityNotification(
                'Secure context (HTTPS) required for full security features',
                'warning'
            );
        }

        // Check for crypto support
        if (!this.cryptoSupported.webCrypto) {
            this.showSecurityNotification(
                'Web Crypto API not available - some security features disabled',
                'warning'
            );
        }

        // Perform security audit
        this.performSecurityAudit();
    }

    performSecurityAudit() {
        const auditResults = {
            timestamp: new Date().toISOString(),
            secureContext: this.isSecureContext(),
            cryptoSupport: this.cryptoSupported,
            browserSecurity: this.auditBrowserSecurity(),
            recommendations: []
        };

        if (!auditResults.secureContext) {
            auditResults.recommendations.push('Use HTTPS for secure configuration management');
        }

        if (!auditResults.cryptoSupport.webCrypto) {
            auditResults.recommendations.push('Update browser for enhanced security features');
        }

        console.log('Security audit completed:', auditResults);
        return auditResults;
    }

    auditBrowserSecurity() {
        return {
            userAgent: navigator.userAgent,
            cookiesEnabled: navigator.cookieEnabled,
            doNotTrack: navigator.doNotTrack,
            webgl: this.checkWebGLSupport(),
            webasm: this.checkWebAssemblySupport()
        };
    }

    checkWebGLSupport() {
        try {
            const canvas = document.createElement('canvas');
            return !!(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'));
        } catch (e) {
            return false;
        }
    }

    checkWebAssemblySupport() {
        try {
            return typeof WebAssembly === 'object' && typeof WebAssembly.instantiate === 'function';
        } catch (e) {
            return false;
        }
    }
}

// Secure Key Storage Class
class SecureKeyStorage {
    constructor() {
        this.storagePrefix = 'opensim-secure-';
        this.encryptionKey = null;
        this.init();
    }

    async init() {
        // Initialize encryption key for local storage
        if (typeof crypto !== 'undefined' && crypto.subtle) {
            try {
                this.encryptionKey = await this.deriveEncryptionKey();
            } catch (error) {
                console.warn('Failed to initialize encryption key:', error);
            }
        }
    }

    async deriveEncryptionKey() {
        const keyMaterial = await crypto.subtle.importKey(
            'raw',
            new TextEncoder().encode('opensim-configurator-key'),
            { name: 'PBKDF2' },
            false,
            ['deriveKey']
        );

        return crypto.subtle.deriveKey(
            {
                name: 'PBKDF2',
                salt: new TextEncoder().encode('opensim-salt'),
                iterations: 100000,
                hash: 'SHA-256'
            },
            keyMaterial,
            { name: 'AES-GCM', length: 256 },
            false,
            ['encrypt', 'decrypt']
        );
    }

    async storeCertificateMetadata(id, metadata) {
        try {
            const key = `${this.storagePrefix}cert-${id}`;
            const data = JSON.stringify(metadata);
            
            if (this.encryptionKey) {
                const encrypted = await this.encrypt(data);
                localStorage.setItem(key, encrypted);
            } else {
                localStorage.setItem(key, data);
            }
        } catch (error) {
            console.error('Failed to store certificate metadata:', error);
        }
    }

    async getCertificateMetadata(id) {
        try {
            const key = `${this.storagePrefix}cert-${id}`;
            const data = localStorage.getItem(key);
            
            if (!data) return null;
            
            if (this.encryptionKey) {
                const decrypted = await this.decrypt(data);
                return JSON.parse(decrypted);
            } else {
                return JSON.parse(data);
            }
        } catch (error) {
            console.error('Failed to get certificate metadata:', error);
            return null;
        }
    }

    async storeUSBDeviceInfo(deviceInfo) {
        try {
            const key = `${this.storagePrefix}usb-device`;
            const data = JSON.stringify(deviceInfo);
            
            if (this.encryptionKey) {
                const encrypted = await this.encrypt(data);
                localStorage.setItem(key, encrypted);
            } else {
                localStorage.setItem(key, data);
            }
        } catch (error) {
            console.error('Failed to store USB device info:', error);
        }
    }

    async encrypt(data) {
        if (!this.encryptionKey) {
            throw new Error('Encryption key not available');
        }

        const iv = crypto.getRandomValues(new Uint8Array(12));
        const encodedData = new TextEncoder().encode(data);
        
        const encrypted = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv: iv },
            this.encryptionKey,
            encodedData
        );

        // Combine IV and encrypted data
        const combined = new Uint8Array(iv.length + encrypted.byteLength);
        combined.set(iv);
        combined.set(new Uint8Array(encrypted), iv.length);
        
        return btoa(String.fromCharCode(...combined));
    }

    async decrypt(encryptedData) {
        if (!this.encryptionKey) {
            throw new Error('Encryption key not available');
        }

        const combined = new Uint8Array(atob(encryptedData).split('').map(c => c.charCodeAt(0)));
        const iv = combined.slice(0, 12);
        const data = combined.slice(12);
        
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: iv },
            this.encryptionKey,
            data
        );

        return new TextDecoder().decode(decrypted);
    }

    clearAll() {
        const keys = Object.keys(localStorage).filter(key => 
            key.startsWith(this.storagePrefix)
        );
        
        keys.forEach(key => localStorage.removeItem(key));
    }
}

// Certificate Manager Class
class CertificateManager {
    constructor() {
        this.certificates = new Map();
    }

    async validateCertificateChain() {
        // Validate that certificate and private key match
        const certMetadata = await securityManager.keyStorage.getCertificateMetadata('ssl-cert');
        const keyMetadata = await securityManager.keyStorage.getCertificateMetadata('ssl-key');
        
        if (certMetadata && keyMetadata) {
            // Perform validation
            return this.validateKeyPair(certMetadata, keyMetadata);
        }
        
        return false;
    }

    validateKeyPair(certMetadata, keyMetadata) {
        // Basic validation - in a real implementation, this would
        // perform cryptographic verification
        return certMetadata.fingerprint && keyMetadata.fingerprint;
    }
}

// USB Key Manager Class
class USBKeyManager {
    constructor() {
        this.connectedDevices = new Map();
    }

    async detectSecurityDevices() {
        if (!('usb' in navigator)) {
            throw new Error('WebUSB not supported');
        }

        try {
            const devices = await navigator.usb.getDevices();
            return devices.filter(device => this.isSecurityDevice(device));
        } catch (error) {
            console.error('Failed to detect USB security devices:', error);
            return [];
        }
    }

    isSecurityDevice(device) {
        const securityVendors = [0x1050, 0x20A0, 0x2581, 0x096E]; // Common security key vendors
        return securityVendors.includes(device.vendorId);
    }
}

// Security Validator Class
class SecurityValidator {
    constructor() {
        this.validationRules = new Map();
        this.setupValidationRules();
    }

    setupValidationRules() {
        // Define security validation rules
        this.validationRules.set('ssl_certificate', {
            required: true,
            validator: this.validateSSLCertificate.bind(this)
        });
        
        this.validationRules.set('private_key', {
            required: true,
            validator: this.validatePrivateKey.bind(this)
        });
        
        this.validationRules.set('password_strength', {
            required: false,
            validator: this.validatePasswordStrength.bind(this)
        });
    }

    validateSSLCertificate(certificate) {
        // Validate SSL certificate format and content
        if (!certificate || typeof certificate !== 'string') {
            return { valid: false, message: 'Certificate content is required' };
        }
        
        if (!certificate.includes('BEGIN CERTIFICATE')) {
            return { valid: false, message: 'Invalid certificate format' };
        }
        
        return { valid: true };
    }

    validatePrivateKey(key) {
        // Validate private key format
        if (!key || typeof key !== 'string') {
            return { valid: false, message: 'Private key content is required' };
        }
        
        const validFormats = ['BEGIN PRIVATE KEY', 'BEGIN RSA PRIVATE KEY', 'BEGIN EC PRIVATE KEY'];
        const hasValidFormat = validFormats.some(format => key.includes(format));
        
        if (!hasValidFormat) {
            return { valid: false, message: 'Invalid private key format' };
        }
        
        return { valid: true };
    }

    validatePasswordStrength(password) {
        // Validate password strength
        if (!password || password.length < 8) {
            return { valid: false, message: 'Password must be at least 8 characters long' };
        }
        
        const hasUpper = /[A-Z]/.test(password);
        const hasLower = /[a-z]/.test(password);
        const hasNumber = /\d/.test(password);
        const hasSpecial = /[!@#$%^&*(),.?":{}|<>]/.test(password);
        
        const score = [hasUpper, hasLower, hasNumber, hasSpecial].filter(Boolean).length;
        
        if (score < 3) {
            return { 
                valid: false, 
                message: 'Password must contain uppercase, lowercase, numbers, and special characters' 
            };
        }
        
        return { valid: true, strength: score === 4 ? 'strong' : 'medium' };
    }
}

// Initialize security manager when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.securityManager = new SecurityManager();
});

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        SecurityManager,
        SecureKeyStorage,
        CertificateManager,
        USBKeyManager,
        SecurityValidator
    };
}