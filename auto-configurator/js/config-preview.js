// OpenSim Next Auto-Configurator - Configuration Preview and Diff System
// Real-time configuration preview with intelligent diff visualization

class ConfigurationPreview {
    constructor() {
        this.currentConfig = null;
        this.pendingChanges = new Map();
        this.originalConfig = null;
        this.diffEngine = new ConfigDiffEngine();
        this.previewRenderer = new PreviewRenderer();
        this.validationEngine = new PreviewValidationEngine();
        this.exportManager = new ConfigExportManager();
        
        // Real-time update settings
        this.updateInterval = 500; // ms
        this.lastUpdateTime = 0;
        this.isPreviewActive = false;
        
        this.initializePreviewSystem();
    }

    initializePreviewSystem() {
        this.createPreviewInterface();
        this.setupEventListeners();
        this.loadCurrentConfiguration();
        this.startRealTimeUpdates();
    }

    createPreviewInterface() {
        const previewContainer = document.createElement('div');
        previewContainer.id = 'config-preview-system';
        previewContainer.className = 'preview-system';
        previewContainer.innerHTML = `
            <div class="preview-header">
                <div class="preview-title">
                    <h2>Configuration Preview</h2>
                    <div class="preview-status">
                        <span class="status-indicator" id="preview-status">Ready</span>
                        <span class="changes-count" id="changes-count">0 changes</span>
                    </div>
                </div>
                <div class="preview-controls">
                    <button class="btn-secondary" id="toggle-preview" title="Toggle preview panel">
                        <i class="fas fa-eye"></i> Preview
                    </button>
                    <button class="btn-secondary" id="toggle-diff-mode" title="Toggle diff view">
                        <i class="fas fa-code-branch"></i> Diff
                    </button>
                    <button class="btn-secondary" id="export-config" title="Export configuration">
                        <i class="fas fa-download"></i> Export
                    </button>
                    <button class="btn-primary" id="apply-changes" title="Apply all changes" disabled>
                        <i class="fas fa-check"></i> Apply Changes
                    </button>
                </div>
            </div>

            <div class="preview-content" id="preview-content">
                <div class="preview-tabs">
                    <button class="tab-button active" data-tab="overview">Overview</button>
                    <button class="tab-button" data-tab="diff">Detailed Diff</button>
                    <button class="tab-button" data-tab="validation">Validation</button>
                    <button class="tab-button" data-tab="files">File Preview</button>
                </div>

                <div class="tab-content">
                    <!-- Overview Tab -->
                    <div class="tab-panel active" id="overview-panel">
                        <div class="config-summary">
                            <div class="summary-grid">
                                <div class="summary-card">
                                    <h3>Configuration Status</h3>
                                    <div class="status-grid" id="config-status-grid">
                                        <!-- Dynamic content -->
                                    </div>
                                </div>
                                <div class="summary-card">
                                    <h3>Quick Stats</h3>
                                    <div class="stats-list" id="config-stats">
                                        <!-- Dynamic content -->
                                    </div>
                                </div>
                            </div>
                            <div class="changes-summary" id="changes-summary">
                                <!-- Dynamic content -->
                            </div>
                        </div>
                    </div>

                    <!-- Diff Tab -->
                    <div class="tab-panel" id="diff-panel">
                        <div class="diff-controls">
                            <div class="diff-options">
                                <label>
                                    <input type="checkbox" id="show-unchanged" checked>
                                    Show unchanged values
                                </label>
                                <label>
                                    <input type="checkbox" id="show-comments" checked>
                                    Show comments
                                </label>
                                <label>
                                    <input type="checkbox" id="side-by-side">
                                    Side-by-side view
                                </label>
                            </div>
                            <div class="diff-filters">
                                <select id="section-filter">
                                    <option value="all">All Sections</option>
                                    <!-- Dynamic options -->
                                </select>
                                <input type="text" id="search-filter" placeholder="Search in diff...">
                            </div>
                        </div>
                        <div class="diff-viewer" id="diff-viewer">
                            <!-- Dynamic diff content -->
                        </div>
                    </div>

                    <!-- Validation Tab -->
                    <div class="tab-panel" id="validation-panel">
                        <div class="validation-summary">
                            <div class="validation-status" id="validation-status">
                                <!-- Dynamic content -->
                            </div>
                            <div class="validation-details" id="validation-details">
                                <!-- Dynamic content -->
                            </div>
                        </div>
                    </div>

                    <!-- Files Tab -->
                    <div class="tab-panel" id="files-panel">
                        <div class="files-list" id="affected-files">
                            <!-- Dynamic content -->
                        </div>
                        <div class="file-preview" id="file-preview-content">
                            <!-- Dynamic content -->
                        </div>
                    </div>
                </div>
            </div>
        `;

        // Insert after the main configuration form
        const insertPoint = document.querySelector('.config-form') || document.body;
        insertPoint.parentNode.insertBefore(previewContainer, insertPoint.nextSibling);
    }

    setupEventListeners() {
        // Tab switching
        document.querySelectorAll('.tab-button').forEach(button => {
            button.addEventListener('click', (e) => {
                this.switchTab(e.target.dataset.tab);
            });
        });

        // Control buttons
        document.getElementById('toggle-preview').addEventListener('click', () => {
            this.togglePreviewPanel();
        });

        document.getElementById('toggle-diff-mode').addEventListener('click', () => {
            this.toggleDiffMode();
        });

        document.getElementById('export-config').addEventListener('click', () => {
            this.exportConfiguration();
        });

        document.getElementById('apply-changes').addEventListener('click', () => {
            this.applyChanges();
        });

        // Diff controls
        document.getElementById('show-unchanged').addEventListener('change', () => {
            this.updateDiffView();
        });

        document.getElementById('show-comments').addEventListener('change', () => {
            this.updateDiffView();
        });

        document.getElementById('side-by-side').addEventListener('change', () => {
            this.updateDiffView();
        });

        document.getElementById('section-filter').addEventListener('change', () => {
            this.updateDiffView();
        });

        document.getElementById('search-filter').addEventListener('input', (e) => {
            this.filterDiffContent(e.target.value);
        });

        // Listen for configuration changes
        document.addEventListener('configurationChanged', (e) => {
            this.handleConfigurationChange(e.detail);
        });

        // Listen for form field changes
        this.observeFormChanges();
    }

    observeFormChanges() {
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'attributes' || mutation.type === 'childList') {
                    this.scheduleUpdate();
                }
            });
        });

        // Observe all form elements
        document.querySelectorAll('form, .config-section').forEach(element => {
            observer.observe(element, {
                attributes: true,
                childList: true,
                subtree: true
            });
        });

        // Direct input listeners
        document.addEventListener('input', (e) => {
            if (e.target.matches('input, select, textarea')) {
                this.scheduleUpdate();
            }
        });

        document.addEventListener('change', (e) => {
            if (e.target.matches('input, select, textarea')) {
                this.scheduleUpdate();
            }
        });
    }

    scheduleUpdate() {
        const now = Date.now();
        if (now - this.lastUpdateTime > this.updateInterval) {
            this.updatePreview();
            this.lastUpdateTime = now;
        } else {
            // Debounce rapid changes
            clearTimeout(this.updateTimeout);
            this.updateTimeout = setTimeout(() => {
                this.updatePreview();
                this.lastUpdateTime = Date.now();
            }, this.updateInterval);
        }
    }

    loadCurrentConfiguration() {
        try {
            // Get current configuration from the form
            this.originalConfig = this.captureCurrentConfiguration();
            this.currentConfig = JSON.parse(JSON.stringify(this.originalConfig));
            this.updatePreview();
        } catch (error) {
            console.error('Failed to load current configuration:', error);
            this.showError('Failed to load configuration');
        }
    }

    captureCurrentConfiguration() {
        const config = {
            general: this.captureGeneralConfig(),
            network: this.captureNetworkConfig(),
            database: this.captureDatabaseConfig(),
            physics: this.capturePhysicsConfig(),
            security: this.captureSecurityConfig(),
            performance: this.capturePerformanceConfig(),
            regions: this.captureRegionsConfig(),
            grid: this.captureGridConfig()
        };

        return config;
    }

    captureGeneralConfig() {
        return {
            gridName: this.getFieldValue('grid-name', 'OpenSim Next Grid'),
            gridNick: this.getFieldValue('grid-nick', 'opensim'),
            welcomeMessage: this.getFieldValue('welcome-message', 'Welcome to OpenSim Next!'),
            deploymentType: this.getFieldValue('deployment-type', 'development'),
            instanceName: this.getFieldValue('instance-name', 'default'),
            publicAddress: this.getFieldValue('public-address', 'localhost'),
            adminEmail: this.getFieldValue('admin-email', 'admin@example.com')
        };
    }

    captureNetworkConfig() {
        return {
            httpPort: parseInt(this.getFieldValue('http-port', '9000')),
            httpsPort: parseInt(this.getFieldValue('https-port', '9001')),
            httpsEnabled: this.getFieldValue('https-enabled', false, 'checkbox'),
            externalHostname: this.getFieldValue('external-hostname', 'localhost'),
            internalIp: this.getFieldValue('internal-ip', '127.0.0.1'),
            websocketPort: parseInt(this.getFieldValue('websocket-port', '9002')),
            maxConnections: parseInt(this.getFieldValue('max-connections', '1000'))
        };
    }

    captureDatabaseConfig() {
        return {
            type: this.getFieldValue('database-type', 'sqlite'),
            host: this.getFieldValue('database-host', 'localhost'),
            port: parseInt(this.getFieldValue('database-port', '5432')),
            name: this.getFieldValue('database-name', 'opensim'),
            username: this.getFieldValue('database-username', 'opensim'),
            password: this.getFieldValue('database-password', ''),
            connectionString: this.getFieldValue('database-connection-string', ''),
            poolSize: parseInt(this.getFieldValue('database-pool-size', '10'))
        };
    }

    capturePhysicsConfig() {
        return {
            defaultEngine: this.getFieldValue('physics-engine', 'ODE'),
            timestep: parseFloat(this.getFieldValue('physics-timestep', '0.0167')),
            maxBodies: parseInt(this.getFieldValue('physics-max-bodies', '10000')),
            enableCollisions: this.getFieldValue('physics-collisions', true, 'checkbox'),
            gravityX: parseFloat(this.getFieldValue('gravity-x', '0')),
            gravityY: parseFloat(this.getFieldValue('gravity-y', '0')),
            gravityZ: parseFloat(this.getFieldValue('gravity-z', '-9.8'))
        };
    }

    captureSecurityConfig() {
        return {
            apiKey: this.getFieldValue('api-key', 'default-key-change-me'),
            passwordComplexity: this.getFieldValue('password-complexity', true, 'checkbox'),
            sessionTimeout: parseInt(this.getFieldValue('session-timeout', '3600')),
            bruteForceProtection: this.getFieldValue('brute-force-protection', true, 'checkbox'),
            sslCertificatePath: this.getFieldValue('ssl-certificate-path', ''),
            sslKeyPath: this.getFieldValue('ssl-key-path', ''),
            rateLimitEnabled: this.getFieldValue('rate-limit-enabled', true, 'checkbox'),
            maxRequestsPerMinute: parseInt(this.getFieldValue('max-requests-per-minute', '100'))
        };
    }

    capturePerformanceConfig() {
        return {
            maxPrims: parseInt(this.getFieldValue('max-prims', '15000')),
            maxScripts: parseInt(this.getFieldValue('max-scripts', '1000')),
            scriptTimeout: parseInt(this.getFieldValue('script-timeout', '30')),
            cacheAssets: this.getFieldValue('cache-assets', true, 'checkbox'),
            cacheTimeout: parseInt(this.getFieldValue('cache-timeout', '24')),
            enableGzip: this.getFieldValue('enable-gzip', true, 'checkbox'),
            threadPoolSize: parseInt(this.getFieldValue('thread-pool-size', '4'))
        };
    }

    captureRegionsConfig() {
        const regionElements = document.querySelectorAll('.region-config');
        const regions = [];

        regionElements.forEach(element => {
            const regionId = element.dataset.regionId;
            regions.push({
                name: this.getFieldValue(`region-${regionId}-name`, 'Default Region'),
                uuid: this.getFieldValue(`region-${regionId}-uuid`, ''),
                location: {
                    x: parseInt(this.getFieldValue(`region-${regionId}-x`, '1000')),
                    y: parseInt(this.getFieldValue(`region-${regionId}-y`, '1000'))
                },
                size: {
                    x: parseInt(this.getFieldValue(`region-${regionId}-size-x`, '256')),
                    y: parseInt(this.getFieldValue(`region-${regionId}-size-y`, '256'))
                },
                physicsEngine: this.getFieldValue(`region-${regionId}-physics`, 'ODE'),
                maxPrims: parseInt(this.getFieldValue(`region-${regionId}-max-prims`, '15000')),
                enabled: this.getFieldValue(`region-${regionId}-enabled`, true, 'checkbox')
            });
        });

        return regions;
    }

    captureGridConfig() {
        return {
            mode: this.getFieldValue('grid-mode', 'standalone'),
            assetServerUrl: this.getFieldValue('asset-server-url', ''),
            inventoryServerUrl: this.getFieldValue('inventory-server-url', ''),
            userServerUrl: this.getFieldValue('user-server-url', ''),
            hypergridEnabled: this.getFieldValue('hypergrid-enabled', false, 'checkbox'),
            hypergridAddress: this.getFieldValue('hypergrid-address', ''),
            allowGuests: this.getFieldValue('allow-guests', false, 'checkbox')
        };
    }

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
            default:
                return element.value || defaultValue;
        }
    }

    updatePreview() {
        if (!this.isPreviewActive) return;

        try {
            // Capture current form state
            const newConfig = this.captureCurrentConfiguration();
            
            // Generate diff
            const diff = this.diffEngine.generateDiff(this.originalConfig, newConfig);
            
            // Update pending changes
            this.pendingChanges = this.diffEngine.extractChanges(diff);
            
            // Update UI
            this.updateOverviewPanel(diff);
            this.updateDiffPanel(diff);
            this.updateValidationPanel(newConfig);
            this.updateFilesPanel(diff);
            this.updateStatusIndicators(diff);
            
            // Store current config
            this.currentConfig = newConfig;
            
            // Trigger event for other components
            this.dispatchPreviewUpdateEvent(diff);
            
        } catch (error) {
            console.error('Failed to update preview:', error);
            this.showError('Preview update failed');
        }
    }

    updateOverviewPanel(diff) {
        const statusGrid = document.getElementById('config-status-grid');
        const statsContainer = document.getElementById('config-stats');
        const changesSummary = document.getElementById('changes-summary');

        // Update status grid
        statusGrid.innerHTML = this.generateStatusGrid(diff);
        
        // Update stats
        statsContainer.innerHTML = this.generateConfigStats(diff);
        
        // Update changes summary
        changesSummary.innerHTML = this.generateChangesSummary(diff);
    }

    generateStatusGrid(diff) {
        const sections = Object.keys(diff);
        const statusItems = sections.map(section => {
            const sectionDiff = diff[section];
            const changeCount = this.countChanges(sectionDiff);
            const status = changeCount > 0 ? 'modified' : 'unchanged';
            
            return `
                <div class="status-item ${status}">
                    <div class="status-icon">
                        <i class="fas ${this.getStatusIcon(status)}"></i>
                    </div>
                    <div class="status-info">
                        <span class="status-name">${this.formatSectionName(section)}</span>
                        <span class="status-changes">${changeCount} changes</span>
                    </div>
                </div>
            `;
        }).join('');

        return statusItems;
    }

    generateConfigStats(diff) {
        const totalChanges = this.countTotalChanges(diff);
        const affectedSections = this.countAffectedSections(diff);
        const validationStatus = this.validationEngine.getOverallStatus(this.currentConfig);

        return `
            <div class="stat-item">
                <span class="stat-label">Total Changes:</span>
                <span class="stat-value ${totalChanges > 0 ? 'modified' : ''}">${totalChanges}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Affected Sections:</span>
                <span class="stat-value">${affectedSections}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Validation Status:</span>
                <span class="stat-value validation-${validationStatus.status}">${validationStatus.message}</span>
            </div>
            <div class="stat-item">
                <span class="stat-label">Deployment Type:</span>
                <span class="stat-value">${this.currentConfig.general.deploymentType}</span>
            </div>
        `;
    }

    generateChangesSummary(diff) {
        const changes = this.diffEngine.extractChanges(diff);
        if (changes.size === 0) {
            return '<div class="no-changes">No pending changes</div>';
        }

        const changesList = Array.from(changes.entries()).map(([path, change]) => {
            return `
                <div class="change-item ${change.type}">
                    <div class="change-type">
                        <i class="fas ${this.getChangeTypeIcon(change.type)}"></i>
                        ${change.type}
                    </div>
                    <div class="change-path">${path}</div>
                    <div class="change-details">
                        ${this.formatChangeDetails(change)}
                    </div>
                </div>
            `;
        }).join('');

        return `
            <div class="changes-header">
                <h4>Pending Changes (${changes.size})</h4>
            </div>
            <div class="changes-list">
                ${changesList}
            </div>
        `;
    }

    updateDiffPanel(diff) {
        const diffViewer = document.getElementById('diff-viewer');
        const showUnchanged = document.getElementById('show-unchanged').checked;
        const showComments = document.getElementById('show-comments').checked;
        const sideBySide = document.getElementById('side-by-side').checked;
        const sectionFilter = document.getElementById('section-filter').value;

        // Update section filter options
        this.updateSectionFilter(Object.keys(diff));

        // Generate diff view
        const diffHtml = this.diffEngine.renderDiff(diff, {
            showUnchanged,
            showComments,
            sideBySide,
            sectionFilter
        });

        diffViewer.innerHTML = diffHtml;
    }

    updateSectionFilter(sections) {
        const sectionFilter = document.getElementById('section-filter');
        const currentValue = sectionFilter.value;
        
        sectionFilter.innerHTML = '<option value="all">All Sections</option>';
        sections.forEach(section => {
            const option = document.createElement('option');
            option.value = section;
            option.textContent = this.formatSectionName(section);
            sectionFilter.appendChild(option);
        });
        
        // Restore previous selection if still valid
        if (sections.includes(currentValue) || currentValue === 'all') {
            sectionFilter.value = currentValue;
        }
    }

    updateValidationPanel(config) {
        const validationStatus = document.getElementById('validation-status');
        const validationDetails = document.getElementById('validation-details');

        const validation = this.validationEngine.validateConfiguration(config);

        validationStatus.innerHTML = this.generateValidationStatus(validation);
        validationDetails.innerHTML = this.generateValidationDetails(validation);
    }

    generateValidationStatus(validation) {
        const statusClass = validation.isValid ? 'valid' : 'invalid';
        const statusIcon = validation.isValid ? 'check-circle' : 'exclamation-triangle';
        
        return `
            <div class="validation-overview ${statusClass}">
                <div class="validation-icon">
                    <i class="fas fa-${statusIcon}"></i>
                </div>
                <div class="validation-summary">
                    <h3>${validation.isValid ? 'Configuration Valid' : 'Configuration Issues Found'}</h3>
                    <p>${validation.summary}</p>
                    <div class="validation-counts">
                        <span class="error-count">Errors: ${validation.errors.length}</span>
                        <span class="warning-count">Warnings: ${validation.warnings.length}</span>
                        <span class="info-count">Info: ${validation.info.length}</span>
                    </div>
                </div>
            </div>
        `;
    }

    generateValidationDetails(validation) {
        const allIssues = [
            ...validation.errors.map(e => ({...e, type: 'error'})),
            ...validation.warnings.map(w => ({...w, type: 'warning'})),
            ...validation.info.map(i => ({...i, type: 'info'}))
        ];

        if (allIssues.length === 0) {
            return '<div class="no-issues">No validation issues found</div>';
        }

        const issuesList = allIssues.map(issue => `
            <div class="validation-issue ${issue.type}">
                <div class="issue-icon">
                    <i class="fas ${this.getValidationIcon(issue.type)}"></i>
                </div>
                <div class="issue-content">
                    <div class="issue-title">${issue.message}</div>
                    <div class="issue-path">${issue.path}</div>
                    ${issue.suggestion ? `<div class="issue-suggestion">${issue.suggestion}</div>` : ''}
                </div>
                ${issue.autofix ? `<button class="autofix-btn" onclick="applyAutofix('${issue.id}')">Fix</button>` : ''}
            </div>
        `).join('');

        return `
            <div class="validation-issues">
                ${issuesList}
            </div>
        `;
    }

    updateFilesPanel(diff) {
        const affectedFiles = document.getElementById('affected-files');
        const filePreview = document.getElementById('file-preview-content');

        const files = this.exportManager.getAffectedFiles(this.currentConfig, diff);
        
        affectedFiles.innerHTML = this.generateFilesList(files);
        
        // Show preview of first file by default
        if (files.length > 0) {
            this.showFilePreview(files[0]);
        }
    }

    generateFilesList(files) {
        if (files.length === 0) {
            return '<div class="no-files">No configuration files affected</div>';
        }

        const filesList = files.map(file => `
            <div class="file-item" onclick="showFilePreview('${file.path}')">
                <div class="file-icon">
                    <i class="fas ${this.getFileIcon(file.type)}"></i>
                </div>
                <div class="file-info">
                    <div class="file-name">${file.name}</div>
                    <div class="file-path">${file.path}</div>
                    <div class="file-status ${file.status}">${file.status}</div>
                </div>
                <div class="file-changes">
                    <span class="changes-count">${file.changeCount} changes</span>
                </div>
            </div>
        `).join('');

        return filesList;
    }

    showFilePreview(filePath) {
        const filePreview = document.getElementById('file-preview-content');
        const content = this.exportManager.generateFileContent(filePath, this.currentConfig);
        
        filePreview.innerHTML = `
            <div class="file-preview-header">
                <h4>${filePath}</h4>
                <div class="preview-actions">
                    <button class="btn-secondary" onclick="downloadFilePreview('${filePath}')">
                        <i class="fas fa-download"></i> Download
                    </button>
                    <button class="btn-secondary" onclick="copyFileContent('${filePath}')">
                        <i class="fas fa-copy"></i> Copy
                    </button>
                </div>
            </div>
            <div class="file-content">
                <pre><code class="language-ini">${this.escapeHtml(content)}</code></pre>
            </div>
        `;

        // Highlight syntax if Prism.js is available
        if (window.Prism) {
            Prism.highlightAll();
        }
    }

    updateStatusIndicators(diff) {
        const totalChanges = this.countTotalChanges(diff);
        const previewStatus = document.getElementById('preview-status');
        const changesCount = document.getElementById('changes-count');
        const applyButton = document.getElementById('apply-changes');

        // Update status
        if (totalChanges === 0) {
            previewStatus.textContent = 'No Changes';
            previewStatus.className = 'status-indicator ready';
        } else {
            const validation = this.validationEngine.validateConfiguration(this.currentConfig);
            if (validation.isValid) {
                previewStatus.textContent = 'Ready to Apply';
                previewStatus.className = 'status-indicator ready';
            } else {
                previewStatus.textContent = 'Validation Issues';
                previewStatus.className = 'status-indicator error';
            }
        }

        // Update changes count
        changesCount.textContent = `${totalChanges} change${totalChanges !== 1 ? 's' : ''}`;

        // Update apply button
        const validation = this.validationEngine.validateConfiguration(this.currentConfig);
        applyButton.disabled = totalChanges === 0 || !validation.isValid;
    }

    // Tab management
    switchTab(tabName) {
        // Update tab buttons
        document.querySelectorAll('.tab-button').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

        // Update tab panels
        document.querySelectorAll('.tab-panel').forEach(panel => {
            panel.classList.remove('active');
        });
        document.getElementById(`${tabName}-panel`).classList.add('active');

        // Refresh content for active tab
        this.refreshActiveTabContent(tabName);
    }

    refreshActiveTabContent(tabName) {
        switch (tabName) {
            case 'diff':
                this.updateDiffView();
                break;
            case 'validation':
                this.updateValidationPanel(this.currentConfig);
                break;
            case 'files':
                const diff = this.diffEngine.generateDiff(this.originalConfig, this.currentConfig);
                this.updateFilesPanel(diff);
                break;
        }
    }

    // UI Controls
    togglePreviewPanel() {
        const previewContent = document.getElementById('preview-content');
        const toggleButton = document.getElementById('toggle-preview');
        
        if (previewContent.style.display === 'none') {
            previewContent.style.display = 'block';
            toggleButton.innerHTML = '<i class="fas fa-eye-slash"></i> Hide';
            this.isPreviewActive = true;
            this.updatePreview();
        } else {
            previewContent.style.display = 'none';
            toggleButton.innerHTML = '<i class="fas fa-eye"></i> Show';
            this.isPreviewActive = false;
        }
    }

    toggleDiffMode() {
        const sideBySideCheckbox = document.getElementById('side-by-side');
        sideBySideCheckbox.checked = !sideBySideCheckbox.checked;
        this.updateDiffView();
    }

    updateDiffView() {
        if (document.querySelector('.tab-panel#diff-panel.active')) {
            const diff = this.diffEngine.generateDiff(this.originalConfig, this.currentConfig);
            this.updateDiffPanel(diff);
        }
    }

    filterDiffContent(searchTerm) {
        const diffViewer = document.getElementById('diff-viewer');
        const items = diffViewer.querySelectorAll('.diff-item');
        
        items.forEach(item => {
            const text = item.textContent.toLowerCase();
            const matches = text.includes(searchTerm.toLowerCase());
            item.style.display = matches ? 'block' : 'none';
        });
    }

    // Configuration management
    exportConfiguration() {
        this.exportManager.showExportDialog(this.currentConfig);
    }

    async applyChanges() {
        try {
            const applyButton = document.getElementById('apply-changes');
            applyButton.disabled = true;
            applyButton.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Applying...';

            // Validate configuration one more time
            const validation = this.validationEngine.validateConfiguration(this.currentConfig);
            if (!validation.isValid) {
                throw new Error('Configuration validation failed');
            }

            // Apply changes
            await this.doApplyChanges();

            // Update original config to match current
            this.originalConfig = JSON.parse(JSON.stringify(this.currentConfig));
            this.pendingChanges.clear();

            // Update UI
            this.updatePreview();
            this.showSuccess('Configuration applied successfully');

            // Trigger event
            this.dispatchConfigurationAppliedEvent();

        } catch (error) {
            console.error('Failed to apply changes:', error);
            this.showError(`Failed to apply changes: ${error.message}`);
        } finally {
            const applyButton = document.getElementById('apply-changes');
            applyButton.disabled = false;
            applyButton.innerHTML = '<i class="fas fa-check"></i> Apply Changes';
        }
    }

    async doApplyChanges() {
        // Generate configuration files
        const files = this.exportManager.generateAllFiles(this.currentConfig);
        
        // Save configuration files
        for (const file of files) {
            await this.saveConfigurationFile(file);
        }

        // Update deployment configuration
        await this.updateDeploymentConfiguration();
        
        // Restart services if needed
        await this.restartServicesIfNeeded();
    }

    async saveConfigurationFile(file) {
        // This would typically make an API call to save the file
        console.log(`Saving configuration file: ${file.path}`);
        
        // Simulate API call
        return new Promise(resolve => setTimeout(resolve, 100));
    }

    async updateDeploymentConfiguration() {
        // Update deployment-specific settings
        console.log('Updating deployment configuration');
        return new Promise(resolve => setTimeout(resolve, 200));
    }

    async restartServicesIfNeeded() {
        // Check if services need restart and handle accordingly
        const needsRestart = this.checkIfRestartNeeded();
        if (needsRestart) {
            console.log('Restarting services...');
            // This would trigger service restart
            return new Promise(resolve => setTimeout(resolve, 1000));
        }
    }

    checkIfRestartNeeded() {
        // Check if changes require service restart
        const criticalChanges = Array.from(this.pendingChanges.keys()).some(path => 
            path.includes('network.') || 
            path.includes('database.') || 
            path.includes('security.apiKey')
        );
        return criticalChanges;
    }

    // Event handling
    handleConfigurationChange(detail) {
        this.scheduleUpdate();
    }

    dispatchPreviewUpdateEvent(diff) {
        const event = new CustomEvent('previewUpdated', {
            detail: {
                diff,
                config: this.currentConfig,
                changeCount: this.countTotalChanges(diff)
            }
        });
        document.dispatchEvent(event);
    }

    dispatchConfigurationAppliedEvent() {
        const event = new CustomEvent('configurationApplied', {
            detail: {
                config: this.currentConfig,
                timestamp: new Date().toISOString()
            }
        });
        document.dispatchEvent(event);
    }

    // Utility methods
    countChanges(obj) {
        let count = 0;
        for (const key in obj) {
            if (typeof obj[key] === 'object' && obj[key] !== null) {
                if (obj[key].__change_type) {
                    count++;
                } else {
                    count += this.countChanges(obj[key]);
                }
            }
        }
        return count;
    }

    countTotalChanges(diff) {
        return Object.values(diff).reduce((total, section) => 
            total + this.countChanges(section), 0);
    }

    countAffectedSections(diff) {
        return Object.values(diff).filter(section => 
            this.countChanges(section) > 0).length;
    }

    formatSectionName(section) {
        return section.charAt(0).toUpperCase() + section.slice(1).replace(/([A-Z])/g, ' $1');
    }

    getStatusIcon(status) {
        const icons = {
            modified: 'edit',
            unchanged: 'check',
            error: 'exclamation-triangle',
            warning: 'exclamation'
        };
        return icons[status] || 'question';
    }

    getChangeTypeIcon(type) {
        const icons = {
            added: 'plus',
            modified: 'edit',
            deleted: 'minus',
            moved: 'arrows-alt'
        };
        return icons[type] || 'edit';
    }

    getValidationIcon(type) {
        const icons = {
            error: 'exclamation-circle',
            warning: 'exclamation-triangle',
            info: 'info-circle'
        };
        return icons[type] || 'info';
    }

    getFileIcon(type) {
        const icons = {
            ini: 'file-code',
            xml: 'file-code',
            json: 'file-code',
            cfg: 'cog',
            conf: 'cog'
        };
        return icons[type] || 'file';
    }

    formatChangeDetails(change) {
        switch (change.type) {
            case 'added':
                return `<span class="new-value">${change.newValue}</span>`;
            case 'deleted':
                return `<span class="old-value">${change.oldValue}</span>`;
            case 'modified':
                return `
                    <span class="old-value">${change.oldValue}</span>
                    <span class="arrow">→</span>
                    <span class="new-value">${change.newValue}</span>
                `;
            default:
                return change.description || '';
        }
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    showError(message) {
        console.error(message);
        // This would typically show a toast notification
    }

    showSuccess(message) {
        console.log(message);
        // This would typically show a toast notification
    }

    startRealTimeUpdates() {
        this.isPreviewActive = true;
        this.updatePreview();
    }
}

// Initialize the configuration preview system when the DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.configPreview = new ConfigurationPreview();
});