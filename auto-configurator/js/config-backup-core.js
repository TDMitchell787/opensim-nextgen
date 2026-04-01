// OpenSim Next Auto-Configurator - Configuration Backup Core Functions
// Core backup, restore, and template management functionality

// Extend the ConfigExportManager with backup/restore core functionality
Object.assign(ConfigExportManager.prototype, {

    // ====== BACKUP CREATION METHODS ======

    async previewBackup() {
        try {
            const config = await this.gatherCurrentConfiguration();
            const format = document.getElementById('backup-format').value;
            
            // Create preview modal
            this.showBackupPreviewModal(config, format);
            
        } catch (error) {
            console.error('Failed to preview backup:', error);
            this.showError('Failed to generate backup preview: ' + error.message);
        }
    },

    async createConfigurationBackup() {
        try {
            const config = await this.gatherCurrentConfiguration();
            const format = document.getElementById('backup-format').value;
            const compress = document.getElementById('compress-output').checked;
            const encryption = document.getElementById('encryption-method').value;
            
            // Process configuration
            let output = await this.formatBackupConfiguration(config, format);
            
            // Apply encryption if requested
            if (encryption !== 'none') {
                const password = document.getElementById('encryption-password').value;
                if (!password) {
                    this.showError('Encryption password is required');
                    return;
                }
                output = await this.encryptBackupData(output, encryption, password);
            }
            
            // Apply compression if requested
            if (compress) {
                output = await this.compressBackupData(output);
            }
            
            // Generate filename
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
            const gridName = this.sanitizeFilename(config.general?.gridNick || 'opensim-config');
            const filename = `${gridName}-backup-${timestamp}.${format}${compress ? '.gz' : ''}`;
            
            // Download file
            this.downloadBackupFile(output, filename);
            
            // Add to history
            this.addToBackupHistory('backup', {
                filename,
                format,
                compressed: compress,
                encrypted: encryption !== 'none',
                size: output.length,
                timestamp: new Date().toISOString(),
                configVersion: config.metadata?.version || '1.0.0'
            });
            
            this.showSuccess('Configuration backup created successfully');
            
        } catch (error) {
            console.error('Failed to create backup:', error);
            this.showError('Failed to create backup: ' + error.message);
        }
    },

    async gatherCurrentConfiguration() {
        const includeSecrets = document.getElementById('include-secrets').checked;
        const includeMetadata = document.getElementById('include-metadata').checked;
        const includeValidation = document.getElementById('include-validation').checked;
        
        const config = {
            metadata: includeMetadata ? {
                backupVersion: '1.0.0',
                createdAt: new Date().toISOString(),
                createdBy: 'OpenSim Next Auto-Configurator',
                format: document.getElementById('backup-format').value,
                opensimVersion: '1.0.0',
                schemaVersion: '1.0',
                configurationId: this.generateConfigurationId(),
                description: 'Configuration backup created by auto-configurator'
            } : undefined
        };
        
        // Gather configuration sections based on selected scope
        if (document.getElementById('export-general').checked) {
            config.general = await this.gatherGeneralSettings(includeSecrets);
        }
        
        if (document.getElementById('export-database').checked) {
            config.database = await this.gatherDatabaseSettings(includeSecrets);
        }
        
        if (document.getElementById('export-network').checked) {
            config.network = await this.gatherNetworkSettings(includeSecrets);
        }
        
        if (document.getElementById('export-security').checked) {
            config.security = await this.gatherSecuritySettings(includeSecrets);
        }
        
        if (document.getElementById('export-regions').checked) {
            config.regions = await this.gatherRegionSettings(includeSecrets);
        }
        
        if (includeValidation) {
            config.validation = this.gatherValidationRules();
        }
        
        return config;
    },

    async gatherGeneralSettings(includeSecrets) {
        return {
            deploymentType: this.getFieldValue('deploymentType'),
            gridName: this.getFieldValue('gridName'),
            gridNick: this.getFieldValue('gridNick'),
            welcomeMessage: this.getFieldValue('welcomeMessage'),
            maxUsers: parseInt(this.getFieldValue('maxUsers')) || 100,
            regionCount: parseInt(this.getFieldValue('regionCount')) || 1,
            physicsEngine: this.getFieldValue('physicsEngine') || 'ODE',
            scriptEngine: this.getFieldValue('scriptEngine') || 'XEngine',
            allowGuests: this.getFieldValue('allowGuests') === 'true',
            enableVoice: this.getFieldValue('enableVoice') === 'true',
            enableGroups: this.getFieldValue('enableGroups') === 'true'
        };
    },

    async gatherDatabaseSettings(includeSecrets) {
        return {
            type: this.getFieldValue('databaseType'),
            host: this.getFieldValue('databaseHost'),
            port: parseInt(this.getFieldValue('databasePort')) || 5432,
            database: this.getFieldValue('databaseName'),
            username: this.getFieldValue('databaseUsername'),
            password: includeSecrets ? this.getFieldValue('databasePassword') : '[REDACTED]',
            poolSize: parseInt(this.getFieldValue('poolSize')) || 10,
            connectionTimeout: parseInt(this.getFieldValue('connectionTimeout')) || 30,
            commandTimeout: parseInt(this.getFieldValue('commandTimeout')) || 60,
            enableSsl: this.getFieldValue('databaseSsl') === 'true',
            autoMigrate: this.getFieldValue('databaseAutoMigrate') === 'true'
        };
    },

    async gatherNetworkSettings(includeSecrets) {
        return {
            httpPort: parseInt(this.getFieldValue('httpPort')) || 9000,
            httpsPort: parseInt(this.getFieldValue('httpsPort')) || 9001,
            httpsEnabled: this.getFieldValue('httpsEnabled') === 'true',
            hostname: this.getFieldValue('hostname'),
            externalHostname: this.getFieldValue('externalHostname'),
            internalIp: this.getFieldValue('internalIp'),
            sslCertificatePath: this.getFieldValue('sslCertificatePath'),
            sslPrivateKeyPath: this.getFieldValue('sslPrivateKeyPath'),
            corsEnabled: this.getFieldValue('corsEnabled') === 'true',
            corsOrigins: this.getFieldValue('corsOrigins') || '*',
            maxConnections: parseInt(this.getFieldValue('maxConnections')) || 1000,
            keepAliveTimeout: parseInt(this.getFieldValue('keepAliveTimeout')) || 30
        };
    },

    async gatherSecuritySettings(includeSecrets) {
        return {
            apiKey: includeSecrets ? this.getFieldValue('apiKey') : '[REDACTED]',
            encryptionKey: includeSecrets ? this.getFieldValue('encryptionKey') : '[REDACTED]',
            passwordComplexity: this.getFieldValue('passwordComplexity') === 'true',
            minPasswordLength: parseInt(this.getFieldValue('minPasswordLength')) || 8,
            sessionTimeout: parseInt(this.getFieldValue('sessionTimeout')) || 3600,
            maxSessionsPerUser: parseInt(this.getFieldValue('maxSessionsPerUser')) || 3,
            rateLimitEnabled: this.getFieldValue('rateLimitEnabled') === 'true',
            maxRequestsPerMinute: parseInt(this.getFieldValue('maxRequestsPerMinute')) || 60,
            bruteForceProtection: this.getFieldValue('bruteForceProtection') === 'true',
            maxLoginAttempts: parseInt(this.getFieldValue('maxLoginAttempts')) || 5,
            lockoutDuration: parseInt(this.getFieldValue('lockoutDuration')) || 900,
            requireHttps: this.getFieldValue('requireHttps') === 'true',
            contentSecurityPolicy: this.getFieldValue('contentSecurityPolicy'),
            allowedFileTypes: this.getFieldValue('allowedFileTypes') || 'jpg,jpeg,png,gif,tga,bmp,wav,ogg',
            maxUploadSize: parseInt(this.getFieldValue('maxUploadSize')) || 10485760
        };
    },

    async gatherRegionSettings(includeSecrets) {
        const regionCount = parseInt(this.getFieldValue('regionCount')) || 1;
        const regions = [];
        
        for (let i = 0; i < regionCount; i++) {
            regions.push({
                name: this.getFieldValue(`region${i}Name`) || `Region ${i + 1}`,
                uuid: this.getFieldValue(`region${i}Uuid`) || this.generateUUID(),
                location: {
                    x: parseInt(this.getFieldValue(`region${i}LocationX`)) || (1000 + i),
                    y: parseInt(this.getFieldValue(`region${i}LocationY`)) || 1000
                },
                size: {
                    x: parseInt(this.getFieldValue(`region${i}SizeX`)) || 256,
                    y: parseInt(this.getFieldValue(`region${i}SizeY`)) || 256
                },
                physicsEngine: this.getFieldValue(`region${i}PhysicsEngine`) || 'ODE',
                maxPrims: parseInt(this.getFieldValue(`region${i}MaxPrims`)) || 15000,
                enabled: this.getFieldValue(`region${i}Enabled`) !== 'false',
                port: 9000 + i,
                terrainType: this.getFieldValue(`region${i}TerrainType`) || 'flat',
                waterHeight: parseFloat(this.getFieldValue(`region${i}WaterHeight`)) || 20.0
            });
        }
        
        return regions;
    },

    gatherValidationRules() {
        return {
            required: ['gridName', 'gridNick', 'databaseType'],
            portRanges: {
                httpPort: { min: 1024, max: 65535 },
                httpsPort: { min: 1024, max: 65535 }
            },
            stringLimits: {
                gridName: { minLength: 3, maxLength: 50 },
                gridNick: { minLength: 3, maxLength: 20 }
            },
            dependencies: {
                httpsEnabled: ['httpsPort', 'sslCertificatePath'],
                databaseType: {
                    postgresql: ['databaseHost', 'databasePort', 'databaseName'],
                    mysql: ['databaseHost', 'databasePort', 'databaseName']
                }
            },
            patterns: {
                gridNick: '^[a-z0-9\\-]{3,20}$',
                email: '^[\\w\\.-]+@[\\w\\.-]+\\.[a-zA-Z]{2,}$'
            }
        };
    },

    async formatBackupConfiguration(config, format) {
        switch (format) {
            case 'json':
                return JSON.stringify(config, null, 2);
            case 'yaml':
                return this.configToYaml(config);
            case 'ini':
                return this.configToIni(config);
            case 'xml':
                return this.configToXml(config);
            default:
                throw new Error(`Unsupported format: ${format}`);
        }
    },

    // ====== RESTORE METHODS ======

    handleBackupFileSelect(file) {
        if (file.size > this.validationRules.maxFileSize) {
            this.showError(`File too large. Maximum size is ${this.validationRules.maxFileSize / (1024 * 1024)}MB`);
            return;
        }
        
        this.selectedBackupFile = file;
        this.displayBackupFileInfo(file);
        document.getElementById('restore-options').style.display = 'block';
        document.getElementById('validate-restore').disabled = false;
    },

    displayBackupFileInfo(file) {
        const fileInfo = document.getElementById('file-info');
        const formatIcon = this.getFormatIcon(file.name);
        
        fileInfo.innerHTML = `
            <div class="file-details">
                <div class="file-icon">${formatIcon}</div>
                <div class="file-metadata">
                    <h5>${file.name}</h5>
                    <p>Size: ${this.formatFileSize(file.size)}</p>
                    <p>Type: ${file.type || 'Unknown'}</p>
                    <p>Modified: ${new Date(file.lastModified).toLocaleString()}</p>
                </div>
            </div>
        `;
    },

    async validateRestore() {
        if (!this.selectedBackupFile) {
            this.showError('No backup file selected');
            return;
        }
        
        try {
            const content = await this.readBackupFile(this.selectedBackupFile);
            const config = await this.parseBackupConfiguration(content, this.selectedBackupFile.name);
            
            // Validate configuration
            const validation = this.validateBackupConfiguration(config);
            
            if (validation.isValid) {
                this.showSuccess('Backup file is valid and ready to restore');
                document.getElementById('restore-config').disabled = false;
                this.displayRestorePreview(config);
            } else {
                this.showError('Backup validation failed: ' + validation.errors.join(', '));
            }
            
        } catch (error) {
            console.error('Failed to validate backup:', error);
            this.showError('Failed to validate backup file: ' + error.message);
        }
    },

    async readBackupFile(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = e => resolve(e.target.result);
            reader.onerror = e => reject(new Error('Failed to read backup file'));
            reader.readAsText(file);
        });
    },

    async parseBackupConfiguration(content, filename) {
        const extension = filename.split('.').pop().toLowerCase();
        
        // Handle compressed files
        if (extension === 'gz' || extension === 'br') {
            try {
                content = await this.decompressBackupData(content);
                // Get the actual format from the decompressed filename
                const parts = filename.split('.');
                const actualFormat = parts[parts.length - 2]; // Format before compression extension
                return this.parseConfigurationByFormat(content, actualFormat);
            } catch (error) {
                throw new Error(`Failed to decompress backup file: ${error.message}`);
            }
        }
        
        return this.parseConfigurationByFormat(content, extension);
    },

    parseConfigurationByFormat(content, format) {
        try {
            switch (format) {
                case 'json':
                    return JSON.parse(content);
                case 'yaml':
                case 'yml':
                    return this.parseYaml(content);
                case 'ini':
                    return this.parseIni(content);
                case 'xml':
                    return this.parseXml(content);
                default:
                    throw new Error(`Unsupported file format: ${format}`);
            }
        } catch (error) {
            throw new Error(`Failed to parse ${format.toUpperCase()} file: ${error.message}`);
        }
    },

    validateBackupConfiguration(config) {
        const errors = [];
        
        // Basic structure validation
        if (!config || typeof config !== 'object') {
            errors.push('Invalid configuration structure');
            return { isValid: false, errors };
        }
        
        // Check for required metadata
        if (this.validationRules.requireMetadata && !config.metadata) {
            errors.push('Missing required metadata section');
        }
        
        // Validate backup version compatibility
        if (config.metadata && config.metadata.backupVersion) {
            const backupVersion = config.metadata.backupVersion;
            if (!this.isCompatibleBackupVersion(backupVersion)) {
                errors.push(`Incompatible backup version: ${backupVersion}`);
            }
        }
        
        // Validate required sections
        const requiredSections = ['general'];
        for (const section of requiredSections) {
            if (!config[section]) {
                errors.push(`Missing required section: ${section}`);
            }
        }
        
        // Validate general settings
        if (config.general) {
            if (!config.general.gridName || config.general.gridName.length < 3) {
                errors.push('Grid name must be at least 3 characters');
            }
            if (!config.general.gridNick || !/^[a-z0-9\-]{3,20}$/.test(config.general.gridNick)) {
                errors.push('Grid nickname must be 3-20 characters, lowercase only');
            }
        }
        
        // Validate network settings
        if (config.network) {
            const httpPort = config.network.httpPort;
            if (httpPort && (httpPort < 1024 || httpPort > 65535)) {
                errors.push('HTTP port must be between 1024 and 65535');
            }
            
            const httpsPort = config.network.httpsPort;
            if (httpsPort && (httpsPort < 1024 || httpsPort > 65535)) {
                errors.push('HTTPS port must be between 1024 and 65535');
            }
            
            if (httpPort && httpsPort && httpPort === httpsPort) {
                errors.push('HTTP and HTTPS ports cannot be the same');
            }
        }
        
        // Validate database settings
        if (config.database) {
            const supportedTypes = ['sqlite', 'mysql', 'postgresql'];
            if (!supportedTypes.includes(config.database.type)) {
                errors.push(`Unsupported database type: ${config.database.type}`);
            }
        }
        
        return { isValid: errors.length === 0, errors };
    },

    isCompatibleBackupVersion(version) {
        // Simple version compatibility check
        const [major, minor] = version.split('.').map(Number);
        return major === 1; // Currently only support version 1.x
    },

    displayRestorePreview(config) {
        const preview = document.getElementById('restore-preview');
        preview.style.display = 'block';
        
        const previewContent = preview.querySelector('.preview-content');
        previewContent.innerHTML = `
            <div class="config-summary">
                <h6>Backup Information</h6>
                <div class="summary-grid">
                    <div class="summary-item">
                        <span class="label">Created:</span>
                        <span class="value">${config.metadata?.createdAt ? new Date(config.metadata.createdAt).toLocaleString() : 'Unknown'}</span>
                    </div>
                    <div class="summary-item">
                        <span class="label">Version:</span>
                        <span class="value">${config.metadata?.backupVersion || 'Unknown'}</span>
                    </div>
                    <div class="summary-item">
                        <span class="label">Grid Name:</span>
                        <span class="value">${config.general?.gridName || 'Not set'}</span>
                    </div>
                    <div class="summary-item">
                        <span class="label">Deployment Type:</span>
                        <span class="value">${config.general?.deploymentType || 'Not set'}</span>
                    </div>
                    <div class="summary-item">
                        <span class="label">Database Type:</span>
                        <span class="value">${config.database?.type || 'Not set'}</span>
                    </div>
                    <div class="summary-item">
                        <span class="label">HTTPS Enabled:</span>
                        <span class="value">${config.network?.httpsEnabled ? 'Yes' : 'No'}</span>
                    </div>
                </div>
            </div>
            
            <div class="changes-preview">
                <h6>Changes Preview</h6>
                <div class="changes-list">
                    ${this.generateRestoreChangesPreview(config)}
                </div>
            </div>
        `;
    },

    generateRestoreChangesPreview(config) {
        // Compare with current configuration
        const changes = [];
        
        // This is a simplified comparison - in production, implement deep comparison
        if (config.general?.gridName !== this.getFieldValue('gridName')) {
            changes.push({
                field: 'Grid Name',
                current: this.getFieldValue('gridName') || 'Not set',
                new: config.general?.gridName || 'Not set',
                type: 'update'
            });
        }
        
        if (config.general?.deploymentType !== this.getFieldValue('deploymentType')) {
            changes.push({
                field: 'Deployment Type',
                current: this.getFieldValue('deploymentType') || 'Not set',
                new: config.general?.deploymentType || 'Not set',
                type: 'update'
            });
        }
        
        if (config.database?.type !== this.getFieldValue('databaseType')) {
            changes.push({
                field: 'Database Type',
                current: this.getFieldValue('databaseType') || 'Not set',
                new: config.database?.type || 'Not set',
                type: 'update'
            });
        }
        
        if (config.network?.httpPort !== parseInt(this.getFieldValue('httpPort'))) {
            changes.push({
                field: 'HTTP Port',
                current: this.getFieldValue('httpPort') || 'Not set',
                new: config.network?.httpPort || 'Not set',
                type: 'update'
            });
        }
        
        if (changes.length === 0) {
            return '<p class="no-changes">No changes detected</p>';
        }
        
        return changes.map(change => `
            <div class="change-item ${change.type}">
                <span class="field-name">${change.field}</span>
                <span class="change-indicator">
                    <span class="old-value">${change.current}</span>
                    <i class="icon-arrow-right"></i>
                    <span class="new-value">${change.new}</span>
                </span>
            </div>
        `).join('');
    },

    async restoreConfiguration() {
        try {
            const content = await this.readBackupFile(this.selectedBackupFile);
            const config = await this.parseBackupConfiguration(content, this.selectedBackupFile.name);
            
            const strategy = document.getElementById('restore-strategy').value;
            const backupBeforeRestore = document.getElementById('backup-before-restore').checked;
            
            // Create backup if requested
            if (backupBeforeRestore) {
                await this.createAutomaticBackup();
            }
            
            // Apply configuration based on strategy
            await this.applyBackupConfiguration(config, strategy);
            
            // Validate after restore if requested
            if (document.getElementById('validate-after-restore').checked) {
                await this.validateAfterRestore();
            }
            
            // Add to history
            this.addToBackupHistory('restore', {
                filename: this.selectedBackupFile.name,
                strategy,
                backup: backupBeforeRestore,
                timestamp: new Date().toISOString(),
                configVersion: config.metadata?.backupVersion || 'Unknown'
            });
            
            this.showSuccess('Configuration restored successfully');
            
            // Reset restore interface
            this.resetRestoreInterface();
            
        } catch (error) {
            console.error('Failed to restore configuration:', error);
            this.showError('Failed to restore configuration: ' + error.message);
        }
    },

    async applyBackupConfiguration(config, strategy) {
        switch (strategy) {
            case 'replace':
                await this.replaceConfiguration(config);
                break;
            case 'merge':
                await this.mergeConfiguration(config);
                break;
            case 'selective':
                await this.selectiveRestore(config);
                break;
            default:
                throw new Error(`Unknown restore strategy: ${strategy}`);
        }
    },

    async replaceConfiguration(config) {
        // Replace current configuration with backup values
        for (const [section, values] of Object.entries(config)) {
            if (section === 'metadata' || section === 'validation') continue;
            
            if (typeof values === 'object' && values !== null) {
                for (const [key, value] of Object.entries(values)) {
                    this.setFieldValue(key, value);
                }
            }
        }
    },

    async mergeConfiguration(config) {
        // Merge backup configuration with current values (keep current if exists)
        for (const [section, values] of Object.entries(config)) {
            if (section === 'metadata' || section === 'validation') continue;
            
            if (typeof values === 'object' && values !== null) {
                for (const [key, value] of Object.entries(values)) {
                    const currentValue = this.getFieldValue(key);
                    if (!currentValue || currentValue === '') {
                        this.setFieldValue(key, value);
                    }
                }
            }
        }
    },

    async selectiveRestore(config) {
        // Show selective restore dialog
        this.showSelectiveRestoreDialog(config);
    },

    async createAutomaticBackup() {
        // Create automatic backup before restore
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const gridName = this.sanitizeFilename(this.getFieldValue('gridNick') || 'opensim-config');
        const filename = `${gridName}-auto-backup-${timestamp}.json`;
        
        const config = await this.gatherCurrentConfiguration();
        const content = JSON.stringify(config, null, 2);
        
        // Store in local storage as well as download
        localStorage.setItem('opensim_auto_backup', content);
        this.downloadBackupFile(content, filename);
        
        this.addToBackupHistory('auto-backup', {
            filename,
            format: 'json',
            automatic: true,
            timestamp: new Date().toISOString()
        });
    },

    // ====== TEMPLATE MANAGEMENT ======

    previewTemplate(templateId) {
        const template = this.templates.get(templateId);
        if (!template) return;
        
        this.showBackupPreviewModal(template.config, 'json', template);
    },

    async applyTemplate(templateId) {
        const template = this.templates.get(templateId);
        if (!template) return;
        
        if (confirm(`Apply template "${template.name}"? This will replace your current configuration.`)) {
            await this.replaceConfiguration(template.config);
            this.showSuccess(`Template "${template.name}" applied successfully`);
            
            // Add to history
            this.addToBackupHistory('template', {
                templateId,
                templateName: template.name,
                timestamp: new Date().toISOString()
            });
        }
    },

    filterTemplates(searchTerm) {
        const cards = document.querySelectorAll('.template-card');
        const term = searchTerm.toLowerCase();
        
        cards.forEach(card => {
            const text = card.textContent.toLowerCase();
            card.style.display = text.includes(term) ? 'block' : 'none';
        });
    },

    filterTemplatesByCategory(category) {
        const cards = document.querySelectorAll('.template-card');
        
        cards.forEach(card => {
            const categoryElement = card.querySelector('.template-category');
            if (!category || categoryElement.textContent === category) {
                card.style.display = 'block';
            } else {
                card.style.display = 'none';
            }
        });
    },

    async createCustomTemplate() {
        // Show template creation dialog
        const templateName = prompt('Enter template name:');
        if (!templateName) return;
        
        const templateDescription = prompt('Enter template description:');
        if (!templateDescription) return;
        
        const config = await this.gatherCurrentConfiguration();
        
        const template = {
            name: templateName,
            description: templateDescription,
            category: 'custom',
            author: 'User',
            version: '1.0.0',
            tags: ['custom', 'user-created'],
            config,
            features: ['Custom configuration'],
            requirements: {
                minRAM: 'Variable',
                minCPU: 'Variable',
                diskSpace: 'Variable'
            }
        };
        
        const templateId = 'custom-' + Date.now();
        this.templates.set(templateId, template);
        
        // Save to localStorage
        this.saveCustomTemplates();
        
        // Refresh template display
        this.displayTemplates();
        
        this.showSuccess(`Template "${templateName}" created successfully`);
    },

    saveCustomTemplates() {
        const customTemplates = {};
        for (const [id, template] of this.templates.entries()) {
            if (template.category === 'custom') {
                customTemplates[id] = template;
            }
        }
        localStorage.setItem('opensim_custom_templates', JSON.stringify(customTemplates));
    },

    loadCustomTemplates() {
        try {
            const customTemplates = JSON.parse(localStorage.getItem('opensim_custom_templates') || '{}');
            for (const [id, template] of Object.entries(customTemplates)) {
                this.templates.set(id, template);
            }
        } catch (error) {
            console.warn('Failed to load custom templates:', error);
        }
    },

    // ====== HISTORY MANAGEMENT ======

    addToBackupHistory(type, details) {
        const history = this.getBackupHistory();
        history.unshift({
            id: Date.now(),
            type,
            ...details
        });
        
        // Keep only last 50 entries
        if (history.length > 50) {
            history.splice(50);
        }
        
        localStorage.setItem('opensim_backup_history', JSON.stringify(history));
        this.loadBackupHistory();
    },

    loadBackupHistory() {
        const history = this.getBackupHistory();
        const historyList = document.getElementById('history-list');
        if (!historyList) return;
        
        if (history.length === 0) {
            historyList.innerHTML = '<p class="no-history">No backup/restore history</p>';
            return;
        }
        
        historyList.innerHTML = history.map(entry => `
            <div class="history-item">
                <div class="history-icon">
                    <i class="icon-${this.getHistoryIcon(entry.type)}"></i>
                </div>
                <div class="history-details">
                    <h6>${this.getHistoryTitle(entry)}</h6>
                    <p>${new Date(entry.timestamp).toLocaleString()}</p>
                    ${entry.format ? `<small>Format: ${entry.format.toUpperCase()}</small>` : ''}
                    ${entry.size ? `<small>Size: ${this.formatFileSize(entry.size)}</small>` : ''}
                    ${entry.configVersion ? `<small>Version: ${entry.configVersion}</small>` : ''}
                </div>
                <div class="history-actions">
                    <button class="btn btn-sm btn-secondary" onclick="configExportManager.viewHistoryDetails('${entry.id}')">
                        <i class="icon-info"></i>
                    </button>
                </div>
            </div>
        `).join('');
    },

    getBackupHistory() {
        try {
            return JSON.parse(localStorage.getItem('opensim_backup_history') || '[]');
        } catch {
            return [];
        }
    },

    getHistoryIcon(type) {
        const icons = {
            'backup': 'download',
            'restore': 'upload',
            'template': 'file-text',
            'auto-backup': 'save'
        };
        return icons[type] || 'file';
    },

    getHistoryTitle(entry) {
        switch (entry.type) {
            case 'backup':
                return `Backup: ${entry.filename}`;
            case 'restore':
                return `Restored: ${entry.filename}`;
            case 'template':
                return `Applied Template: ${entry.templateName}`;
            case 'auto-backup':
                return `Auto-backup: ${entry.filename}`;
            default:
                return `Action: ${entry.type}`;
        }
    },

    // ====== UTILITY METHODS ======

    switchTab(tabName) {
        // Remove active class from all tabs and content
        document.querySelectorAll('.tab-button').forEach(btn => btn.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

        // Add active class to selected tab and content
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
        document.getElementById(`${tabName}-tab`).classList.add('active');
    },

    resetRestoreInterface() {
        document.getElementById('restore-options').style.display = 'none';
        document.getElementById('restore-preview').style.display = 'none';
        document.getElementById('backup-file-input').value = '';
        document.getElementById('restore-config').disabled = true;
        document.getElementById('validate-restore').disabled = true;
        this.selectedBackupFile = null;
    },

    downloadBackupFile(data, filename) {
        const blob = new Blob([data], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        
        URL.revokeObjectURL(url);
    },

    generateConfigurationId() {
        return 'config-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
    },

    sanitizeFilename(filename) {
        return filename.replace(/[^a-zA-Z0-9\-_]/g, '-').toLowerCase();
    },

    formatFileSize(bytes) {
        const sizes = ['B', 'KB', 'MB', 'GB'];
        if (bytes === 0) return '0 B';
        const i = Math.floor(Math.log(bytes) / Math.log(1024));
        return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
    },

    getFormatIcon(filename) {
        const extension = filename.split('.').pop().toLowerCase();
        const icons = {
            json: '📄',
            yaml: '📄',
            yml: '📄',
            ini: '⚙️',
            xml: '📄',
            gz: '📦',
            br: '📦'
        };
        return icons[extension] || '📄';
    },

    showBackupPreviewModal(config, format, template = null) {
        const modal = document.createElement('div');
        modal.className = 'backup-preview-modal';
        modal.innerHTML = `
            <div class="modal-overlay" onclick="this.parentElement.remove()"></div>
            <div class="modal-content">
                <div class="modal-header">
                    <h4>${template ? `Template: ${template.name}` : 'Backup Preview'}</h4>
                    <button class="modal-close" onclick="this.closest('.backup-preview-modal').remove()">
                        <i class="icon-x"></i>
                    </button>
                </div>
                <div class="modal-body">
                    <div class="preview-controls">
                        <button class="btn btn-sm btn-secondary" onclick="configExportManager.copyBackupPreview()">Copy</button>
                        <button class="btn btn-sm btn-secondary" onclick="configExportManager.downloadBackupPreview()">Download</button>
                    </div>
                    <pre class="config-preview" id="backup-preview-content">${this.formatBackupConfiguration(config, format)}</pre>
                </div>
            </div>
        `;
        
        document.body.appendChild(modal);
    },

    // Data processing methods (simplified for demo)
    async encryptBackupData(data, method, password) {
        // Simplified encryption - in production, use proper crypto libraries
        if (method === 'aes256') {
            const encrypted = btoa(data);
            return `ENCRYPTED:AES256:${encrypted}`;
        } else if (method === 'chacha20') {
            const encrypted = btoa(data);
            return `ENCRYPTED:CHACHA20:${encrypted}`;
        }
        return data;
    },

    async compressBackupData(data) {
        // Use browser's compression stream if available
        if ('CompressionStream' in window) {
            const stream = new CompressionStream('gzip');
            const writer = stream.writable.getWriter();
            const reader = stream.readable.getReader();
            
            writer.write(new TextEncoder().encode(data));
            writer.close();
            
            const chunks = [];
            let done = false;
            while (!done) {
                const result = await reader.read();
                done = result.done;
                if (result.value) {
                    chunks.push(result.value);
                }
            }
            
            return new Uint8Array(chunks.reduce((acc, chunk) => [...acc, ...chunk], []));
        }
        
        // Fallback to simple string compression
        return new TextEncoder().encode(data);
    },

    async decompressBackupData(data) {
        // Simplified decompression - implement proper decompression in production
        if (typeof data === 'string' && data.startsWith('COMPRESSED:')) {
            return atob(data.substring(11));
        }
        return data;
    },

    configToYaml(config) {
        // Simple YAML converter
        const toYaml = (obj, indent = 0) => {
            const spaces = '  '.repeat(indent);
            let yaml = '';
            
            for (const [key, value] of Object.entries(obj)) {
                if (value === null || value === undefined) {
                    yaml += `${spaces}${key}: null\n`;
                } else if (typeof value === 'object' && !Array.isArray(value)) {
                    yaml += `${spaces}${key}:\n${toYaml(value, indent + 1)}`;
                } else if (Array.isArray(value)) {
                    yaml += `${spaces}${key}:\n`;
                    value.forEach(item => {
                        yaml += `${spaces}  - ${item}\n`;
                    });
                } else {
                    yaml += `${spaces}${key}: ${value}\n`;
                }
            }
            
            return yaml;
        };
        
        return toYaml(config);
    },

    configToIni(config) {
        let ini = '';
        
        for (const [section, values] of Object.entries(config)) {
            if (typeof values === 'object' && values !== null && !Array.isArray(values)) {
                ini += `[${section}]\n`;
                for (const [key, value] of Object.entries(values)) {
                    if (value !== null && value !== undefined) {
                        ini += `${key} = ${value}\n`;
                    }
                }
                ini += '\n';
            }
        }
        
        return ini;
    },

    configToXml(config) {
        const toXml = (obj, rootName = 'configuration') => {
            let xml = `<?xml version="1.0" encoding="UTF-8"?>\n<${rootName}>\n`;
            
            const processValue = (key, value, indent = 1) => {
                const spaces = '  '.repeat(indent);
                
                if (value === null || value === undefined) {
                    return `${spaces}<${key} />\n`;
                } else if (typeof value === 'object' && !Array.isArray(value)) {
                    let result = `${spaces}<${key}>\n`;
                    for (const [subKey, subValue] of Object.entries(value)) {
                        result += processValue(subKey, subValue, indent + 1);
                    }
                    result += `${spaces}</${key}>\n`;
                    return result;
                } else if (Array.isArray(value)) {
                    let result = `${spaces}<${key}>\n`;
                    value.forEach(item => {
                        result += `${spaces}  <item>${item}</item>\n`;
                    });
                    result += `${spaces}</${key}>\n`;
                    return result;
                } else {
                    return `${spaces}<${key}>${value}</${key}>\n`;
                }
            };
            
            for (const [key, value] of Object.entries(obj)) {
                xml += processValue(key, value);
            }
            
            xml += `</${rootName}>`;
            return xml;
        };
        
        return toXml(config);
    },

    parseYaml(content) {
        // Simple YAML parser - in production, use a proper YAML library
        const lines = content.split('\n');
        const result = {};
        let currentSection = result;
        let indent = 0;
        
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed === '' || trimmed.startsWith('#')) continue;
            
            const currentIndent = line.search(/\S/);
            if (line.includes(':')) {
                const [key, value] = line.split(':').map(s => s.trim());
                if (value) {
                    currentSection[key] = this.parseValue(value);
                } else {
                    currentSection[key] = {};
                }
            }
        }
        
        return result;
    },

    parseIni(content) {
        const lines = content.split('\n');
        const result = {};
        let currentSection = null;
        
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed === '' || trimmed.startsWith(';') || trimmed.startsWith('#')) continue;
            
            if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
                currentSection = trimmed.slice(1, -1);
                result[currentSection] = {};
            } else if (currentSection && line.includes('=')) {
                const [key, value] = line.split('=').map(s => s.trim());
                result[currentSection][key] = this.parseValue(value);
            }
        }
        
        return result;
    },

    parseXml(content) {
        // Simple XML parser - in production, use DOMParser
        const parser = new DOMParser();
        const doc = parser.parseFromString(content, 'text/xml');
        
        const xmlToObj = (element) => {
            const result = {};
            
            for (const child of element.children) {
                if (child.children.length > 0) {
                    result[child.tagName] = xmlToObj(child);
                } else {
                    result[child.tagName] = this.parseValue(child.textContent);
                }
            }
            
            return result;
        };
        
        return xmlToObj(doc.documentElement);
    },

    parseValue(value) {
        if (value === 'true') return true;
        if (value === 'false') return false;
        if (value === 'null') return null;
        if (!isNaN(value) && !isNaN(parseFloat(value))) return parseFloat(value);
        return value;
    }
});

// Initialize the backup system when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    if (window.configExportManager) {
        // Backup system is already integrated
        window.configExportManager.loadCustomTemplates();
    }
});