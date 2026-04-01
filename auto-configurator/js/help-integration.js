// OpenSim Next Auto-Configurator - Help System Integration
// Enhanced integration between help system and configuration interface

class HelpIntegration {
    constructor() {
        this.helpSystem = null;
        this.formFieldMappings = new Map();
        this.errorHelp = new Map();
        this.contextualTips = new Map();
        this.isInitialized = false;
        
        // Smart help settings
        this.settings = {
            autoShowOnError: true,
            showTooltipsOnFocus: true,
            enableSmartSuggestions: true,
            trackHelpUsage: true,
            preloadCriticalHelp: true
        };
        
        this.initializeIntegration();
    }

    async initializeIntegration() {
        try {
            // Wait for help system to be available
            await this.waitForHelpSystem();
            
            // Setup form field mappings
            this.setupFormFieldMappings();
            
            // Setup error help mappings
            this.setupErrorHelpMappings();
            
            // Setup smart suggestions
            this.setupSmartSuggestions();
            
            // Enhance form fields with help
            this.enhanceFormFields();
            
            // Setup error monitoring
            this.setupErrorMonitoring();
            
            this.isInitialized = true;
            console.log('✅ Help integration initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize help integration:', error);
        }
    }

    async waitForHelpSystem() {
        while (!window.helpSystem || !window.helpSystem.isInitialized) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        this.helpSystem = window.helpSystem;
    }

    setupFormFieldMappings() {
        // Enhanced form field to help topic mappings
        this.formFieldMappings.set('input[name="gridName"]', {
            category: 'general',
            topicId: 'grid-name',
            priority: 'high',
            tip: 'Choose a descriptive name for your virtual world'
        });

        this.formFieldMappings.set('input[name="gridNick"]', {
            category: 'general',
            topicId: 'grid-nick',
            priority: 'high',
            tip: 'Short technical identifier (lowercase, no spaces)'
        });

        this.formFieldMappings.set('textarea[name="welcomeMessage"]', {
            category: 'general',
            topicId: 'welcome-message',
            priority: 'medium',
            tip: 'Greeting message shown to new users'
        });

        this.formFieldMappings.set('input[name="httpPort"]', {
            category: 'network',
            topicId: 'port-configuration',
            priority: 'high',
            tip: 'Port for HTTP connections (default: 9000)'
        });

        this.formFieldMappings.set('input[name="httpsPort"]', {
            category: 'network',
            topicId: 'https-configuration',
            priority: 'high',
            tip: 'Port for HTTPS connections (requires SSL certificate)'
        });

        this.formFieldMappings.set('select[name="databaseType"]', {
            category: 'database',
            topicId: 'database-selection',
            priority: 'critical',
            tip: 'Choose database engine based on your needs'
        });

        this.formFieldMappings.set('input[name="apiKey"]', {
            category: 'security',
            topicId: 'api-key-management',
            priority: 'critical',
            tip: 'Secure API key for administration access'
        });

        this.formFieldMappings.set('select[name="physicsEngine"]', {
            category: 'physics',
            topicId: 'physics-engine-selection',
            priority: 'medium',
            tip: 'Physics engine affects performance and features'
        });

        // Deployment type specific mappings
        this.formFieldMappings.set('input[name="deploymentType"][value="development"]', {
            category: 'general',
            topicId: 'development-setup',
            priority: 'high',
            tip: 'Optimized for learning and testing'
        });

        this.formFieldMappings.set('input[name="deploymentType"][value="production"]', {
            category: 'general',
            topicId: 'production-setup',
            priority: 'high',
            tip: 'Battle-tested for live virtual worlds'
        });

        this.formFieldMappings.set('input[name="deploymentType"][value="grid"]', {
            category: 'general',
            topicId: 'grid-setup',
            priority: 'high',
            tip: 'Distributed architecture for massive scale'
        });

        // Security fields
        this.formFieldMappings.set('input[name="sslCertificatePath"]', {
            category: 'security',
            topicId: 'ssl-certificate-setup',
            priority: 'critical',
            tip: 'Path to SSL certificate file for HTTPS'
        });

        this.formFieldMappings.set('input[name="passwordComplexity"]', {
            category: 'security',
            topicId: 'password-policy',
            priority: 'high',
            tip: 'Enforce strong password requirements'
        });

        // Database connection fields
        this.formFieldMappings.set('input[name="databaseHost"]', {
            category: 'database',
            topicId: 'database-connection',
            priority: 'high',
            tip: 'Database server hostname or IP address'
        });

        this.formFieldMappings.set('input[name="databasePort"]', {
            category: 'database',
            topicId: 'database-connection',
            priority: 'medium',
            tip: 'Database server port (PostgreSQL: 5432, MySQL: 3306)'
        });

        this.formFieldMappings.set('input[name="poolSize"]', {
            category: 'database',
            topicId: 'connection-optimization',
            priority: 'medium',
            tip: 'Maximum database connections (10-50 recommended)'
        });
    }

    setupErrorHelpMappings() {
        // Common error messages and their help solutions
        this.errorHelp.set('port already in use', {
            category: 'network',
            topicId: 'port-configuration',
            solution: 'Change to a different port number or stop conflicting service',
            priority: 'high'
        });

        this.errorHelp.set('database connection failed', {
            category: 'database',
            topicId: 'database-selection',
            solution: 'Check database credentials and ensure service is running',
            priority: 'critical'
        });

        this.errorHelp.set('ssl certificate not found', {
            category: 'security',
            topicId: 'https-configuration',
            solution: 'Verify certificate file path and file permissions',
            priority: 'high'
        });

        this.errorHelp.set('invalid api key', {
            category: 'security',
            topicId: 'api-key-management',
            solution: 'Generate a new API key or check key format',
            priority: 'critical'
        });

        this.errorHelp.set('grid name already exists', {
            category: 'general',
            topicId: 'grid-name',
            solution: 'Choose a unique grid name or check existing installations',
            priority: 'medium'
        });

        this.errorHelp.set('invalid hostname', {
            category: 'network',
            topicId: 'hostname-configuration',
            solution: 'Use valid domain name or IP address format',
            priority: 'high'
        });

        this.errorHelp.set('physics engine failed to load', {
            category: 'physics',
            topicId: 'physics-engine-selection',
            solution: 'Try a different physics engine or check system requirements',
            priority: 'medium'
        });
    }

    setupSmartSuggestions() {
        // Context-aware suggestions based on user selections
        this.contextualTips.set('development-mode', [
            {
                condition: () => this.getFieldValue('deploymentType') === 'development',
                suggestions: [
                    { text: 'Consider SQLite for easy setup', action: () => this.setFieldValue('databaseType', 'sqlite') },
                    { text: 'ODE physics is most stable for development', action: () => this.setFieldValue('physicsEngine', 'ODE') },
                    { text: 'Use default ports for local testing', action: () => this.setFieldValue('httpPort', '9000') }
                ]
            }
        ]);

        this.contextualTips.set('production-mode', [
            {
                condition: () => this.getFieldValue('deploymentType') === 'production',
                suggestions: [
                    { text: 'PostgreSQL recommended for production', action: () => this.setFieldValue('databaseType', 'postgresql') },
                    { text: 'Enable HTTPS for security', action: () => this.setFieldValue('httpsEnabled', true) },
                    { text: 'Generate strong API key', action: () => this.generateSecureApiKey() }
                ]
            }
        ]);

        this.contextualTips.set('security-recommendations', [
            {
                condition: () => this.getFieldValue('httpsEnabled') === false,
                suggestions: [
                    { text: 'Enable HTTPS for production security', priority: 'high', helpTopic: 'https-configuration' },
                    { text: 'Generate SSL certificate with Let\'s Encrypt', action: () => this.showSSLWizard() }
                ]
            }
        ]);
    }

    enhanceFormFields() {
        // Add help indicators and tooltips to form fields
        for (const [selector, helpConfig] of this.formFieldMappings.entries()) {
            const elements = document.querySelectorAll(selector);
            elements.forEach(element => {
                this.enhanceFormField(element, helpConfig);
            });
        }
    }

    enhanceFormField(element, helpConfig) {
        // Add help icon next to field
        const helpIcon = this.createHelpIcon(helpConfig);
        this.insertHelpIcon(element, helpIcon);

        // Add event listeners
        element.addEventListener('focus', () => {
            this.handleFieldFocus(element, helpConfig);
        });

        element.addEventListener('blur', () => {
            this.handleFieldBlur(element, helpConfig);
        });

        element.addEventListener('input', () => {
            this.handleFieldInput(element, helpConfig);
        });

        element.addEventListener('change', () => {
            this.handleFieldChange(element, helpConfig);
        });

        // Add validation enhancement
        this.enhanceFieldValidation(element, helpConfig);
    }

    createHelpIcon(helpConfig) {
        const icon = document.createElement('button');
        icon.type = 'button';
        icon.className = `help-field-icon priority-${helpConfig.priority}`;
        icon.innerHTML = '<i class="icon-help-circle"></i>';
        icon.title = helpConfig.tip || 'Click for help';
        
        icon.addEventListener('click', (e) => {
            e.preventDefault();
            this.helpSystem.showHelp(helpConfig.category, helpConfig.topicId);
        });

        return icon;
    }

    insertHelpIcon(element, helpIcon) {
        const wrapper = element.closest('.form-group') || element.parentElement;
        if (wrapper && !wrapper.querySelector('.help-field-icon')) {
            // Find the best position for the help icon
            const label = wrapper.querySelector('label');
            if (label) {
                label.appendChild(helpIcon);
            } else {
                wrapper.appendChild(helpIcon);
            }
        }
    }

    handleFieldFocus(element, helpConfig) {
        if (this.settings.showTooltipsOnFocus) {
            this.showFieldTooltip(element, helpConfig);
        }

        // Show contextual help in help panel if open
        if (this.helpSystem && document.getElementById('help-panel').classList.contains('active')) {
            this.helpSystem.showContextualHelp(element, helpConfig);
        }

        // Check for smart suggestions
        this.checkSmartSuggestions(element, helpConfig);
    }

    handleFieldBlur(element, helpConfig) {
        this.hideFieldTooltip(element);
        this.validateFieldWithHelp(element, helpConfig);
    }

    handleFieldInput(element, helpConfig) {
        // Real-time validation and suggestions
        this.validateFieldWithHelp(element, helpConfig);
        this.updateSmartSuggestions(element, helpConfig);
    }

    handleFieldChange(element, helpConfig) {
        // Handle significant changes
        this.updateDependentFields(element, helpConfig);
        this.checkConfigurationConsistency(element, helpConfig);
    }

    showFieldTooltip(element, helpConfig) {
        const tooltip = document.getElementById('help-tooltip');
        if (!tooltip) return;

        const title = tooltip.querySelector('.tooltip-title');
        const summary = tooltip.querySelector('.tooltip-summary');

        if (title && summary) {
            title.textContent = this.getFieldDisplayName(element);
            summary.textContent = helpConfig.tip;

            // Position tooltip
            const rect = element.getBoundingClientRect();
            tooltip.style.left = rect.right + 10 + 'px';
            tooltip.style.top = rect.top + 'px';

            tooltip.classList.add('active');
        }
    }

    hideFieldTooltip(element) {
        const tooltip = document.getElementById('help-tooltip');
        if (tooltip) {
            tooltip.classList.remove('active');
        }
    }

    validateFieldWithHelp(element, helpConfig) {
        // Enhanced validation with contextual help
        const value = element.value;
        const validationResult = this.validateFieldValue(element, value, helpConfig);

        if (!validationResult.isValid) {
            this.showFieldError(element, validationResult.error, helpConfig);
        } else {
            this.clearFieldError(element);
        }

        return validationResult.isValid;
    }

    validateFieldValue(element, value, helpConfig) {
        const fieldName = element.name || element.id;
        
        // Field-specific validation rules
        switch (fieldName) {
            case 'gridName':
                if (!value || value.length < 3) {
                    return { isValid: false, error: 'Grid name must be at least 3 characters' };
                }
                if (value.length > 50) {
                    return { isValid: false, error: 'Grid name must be under 50 characters' };
                }
                break;

            case 'gridNick':
                if (!value || !/^[a-z0-9\-]{3,20}$/.test(value)) {
                    return { isValid: false, error: 'Grid nickname must be 3-20 characters, lowercase letters, numbers, and hyphens only' };
                }
                break;

            case 'httpPort':
                const port = parseInt(value);
                if (!port || port < 1024 || port > 65535) {
                    return { isValid: false, error: 'Port must be between 1024 and 65535' };
                }
                break;

            case 'apiKey':
                if (!value || value.length < 32) {
                    return { isValid: false, error: 'API key must be at least 32 characters for security' };
                }
                break;

            case 'databaseHost':
                if (!value || (!this.isValidIP(value) && !this.isValidHostname(value))) {
                    return { isValid: false, error: 'Must be a valid IP address or hostname' };
                }
                break;
        }

        return { isValid: true };
    }

    showFieldError(element, errorMessage, helpConfig) {
        // Clear existing error
        this.clearFieldError(element);

        // Create error element
        const errorElement = document.createElement('div');
        errorElement.className = 'field-error-with-help';
        errorElement.innerHTML = `
            <div class="error-message">
                <i class="icon-alert-circle"></i>
                <span>${errorMessage}</span>
            </div>
            <button class="error-help-btn" type="button">
                <i class="icon-help-circle"></i>
                Get Help
            </button>
        `;

        // Add help button functionality
        const helpButton = errorElement.querySelector('.error-help-btn');
        helpButton.addEventListener('click', () => {
            this.helpSystem.showHelp(helpConfig.category, helpConfig.topicId);
        });

        // Insert error after field
        element.parentNode.insertBefore(errorElement, element.nextSibling);

        // Add error styling to field
        element.classList.add('field-error');

        // Auto-show help if enabled
        if (this.settings.autoShowOnError) {
            this.showErrorHelp(errorMessage, helpConfig);
        }
    }

    clearFieldError(element) {
        // Remove error styling
        element.classList.remove('field-error');

        // Remove error message
        const errorElement = element.parentNode.querySelector('.field-error-with-help');
        if (errorElement) {
            errorElement.remove();
        }
    }

    showErrorHelp(errorMessage, helpConfig) {
        // Check if we have specific help for this error
        for (const [errorPattern, errorHelpConfig] of this.errorHelp.entries()) {
            if (errorMessage.toLowerCase().includes(errorPattern)) {
                // Show specific error help
                this.helpSystem.showHelp(errorHelpConfig.category, errorHelpConfig.topicId);
                return;
            }
        }

        // Fallback to general field help
        this.helpSystem.showHelp(helpConfig.category, helpConfig.topicId);
    }

    checkSmartSuggestions(element, helpConfig) {
        if (!this.settings.enableSmartSuggestions) return;

        // Check context-aware suggestions
        for (const [contextKey, suggestions] of this.contextualTips.entries()) {
            for (const suggestionGroup of suggestions) {
                if (suggestionGroup.condition && suggestionGroup.condition()) {
                    this.showSmartSuggestions(element, suggestionGroup.suggestions);
                }
            }
        }
    }

    showSmartSuggestions(element, suggestions) {
        // Create suggestion popup
        const existingSuggestions = document.querySelector('.smart-suggestions');
        if (existingSuggestions) {
            existingSuggestions.remove();
        }

        const suggestionsElement = document.createElement('div');
        suggestionsElement.className = 'smart-suggestions';
        suggestionsElement.innerHTML = `
            <div class="suggestions-header">
                <i class="icon-lightbulb"></i>
                <span>Smart Suggestions</span>
                <button class="suggestions-close" type="button">
                    <i class="icon-x"></i>
                </button>
            </div>
            <div class="suggestions-list">
                ${suggestions.map(suggestion => `
                    <div class="suggestion-item ${suggestion.priority || ''}">
                        <span class="suggestion-text">${suggestion.text}</span>
                        <button class="suggestion-action" type="button">
                            ${suggestion.action ? 'Apply' : 'Learn More'}
                        </button>
                    </div>
                `).join('')}
            </div>
        `;

        // Position near the field
        const rect = element.getBoundingClientRect();
        suggestionsElement.style.position = 'fixed';
        suggestionsElement.style.left = rect.left + 'px';
        suggestionsElement.style.top = (rect.bottom + 10) + 'px';
        suggestionsElement.style.zIndex = '1000';

        document.body.appendChild(suggestionsElement);

        // Setup event listeners
        suggestionsElement.querySelector('.suggestions-close').addEventListener('click', () => {
            suggestionsElement.remove();
        });

        suggestionsElement.querySelectorAll('.suggestion-action').forEach((button, index) => {
            button.addEventListener('click', () => {
                const suggestion = suggestions[index];
                if (suggestion.action) {
                    suggestion.action();
                } else if (suggestion.helpTopic) {
                    this.helpSystem.showHelp(suggestion.category || 'general', suggestion.helpTopic);
                }
                suggestionsElement.remove();
            });
        });

        // Auto-hide after 10 seconds
        setTimeout(() => {
            if (suggestionsElement.parentNode) {
                suggestionsElement.remove();
            }
        }, 10000);
    }

    setupErrorMonitoring() {
        // Monitor for configuration errors and show relevant help
        document.addEventListener('configurationError', (event) => {
            this.handleConfigurationError(event.detail);
        });

        // Monitor validation events
        document.addEventListener('validationFailed', (event) => {
            this.handleValidationFailure(event.detail);
        });

        // Monitor for network errors
        window.addEventListener('error', (event) => {
            this.handleGlobalError(event);
        });
    }

    handleConfigurationError(errorDetail) {
        const { field, error, context } = errorDetail;
        
        // Find relevant help
        const helpConfig = this.findHelpForField(field);
        if (helpConfig) {
            this.showErrorHelp(error, helpConfig);
        }

        // Show help badge
        this.showHelpBadge();
    }

    handleValidationFailure(validationDetail) {
        const { section, errors } = validationDetail;
        
        // Show help for the most critical error
        const criticalError = errors.find(error => error.severity === 'critical');
        if (criticalError) {
            const helpConfig = this.findHelpForError(criticalError.message);
            if (helpConfig) {
                this.showErrorHelp(criticalError.message, helpConfig);
            }
        }
    }

    // Utility methods
    getFieldValue(fieldName) {
        const element = document.querySelector(`[name="${fieldName}"], #${fieldName}`);
        return element ? element.value : null;
    }

    setFieldValue(fieldName, value) {
        const element = document.querySelector(`[name="${fieldName}"], #${fieldName}`);
        if (element) {
            element.value = value;
            element.dispatchEvent(new Event('change', { bubbles: true }));
        }
    }

    getFieldDisplayName(element) {
        const label = element.closest('.form-group')?.querySelector('label');
        return label?.textContent?.replace(':', '') || element.name || element.id || 'Field';
    }

    isValidIP(ip) {
        const ipRegex = /^(\d{1,3}\.){3}\d{1,3}$/;
        return ipRegex.test(ip) && ip.split('.').every(octet => parseInt(octet) <= 255);
    }

    isValidHostname(hostname) {
        const hostnameRegex = /^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$/;
        return hostnameRegex.test(hostname);
    }

    findHelpForField(fieldName) {
        for (const [selector, helpConfig] of this.formFieldMappings.entries()) {
            if (selector.includes(fieldName)) {
                return helpConfig;
            }
        }
        return null;
    }

    findHelpForError(errorMessage) {
        for (const [errorPattern, helpConfig] of this.errorHelp.entries()) {
            if (errorMessage.toLowerCase().includes(errorPattern)) {
                return helpConfig;
            }
        }
        return null;
    }

    showHelpBadge() {
        const badge = document.getElementById('help-badge');
        if (badge) {
            badge.style.display = 'flex';
            setTimeout(() => {
                badge.style.display = 'none';
            }, 5000);
        }
    }

    generateSecureApiKey() {
        // Generate a secure API key
        const key = Array.from(crypto.getRandomValues(new Uint8Array(32)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
        
        this.setFieldValue('apiKey', key);
        
        // Show success message
        this.showSmartSuggestions(document.querySelector('[name="apiKey"]'), [
            { text: 'Secure API key generated successfully', action: null }
        ]);
    }

    showSSLWizard() {
        // Show SSL certificate setup wizard
        this.helpSystem.showHelp('security', 'https-configuration');
    }

    updateDependentFields(element, helpConfig) {
        // Update dependent fields based on changes
        const fieldName = element.name || element.id;
        
        if (fieldName === 'deploymentType') {
            this.updateDeploymentDependentFields(element.value);
        } else if (fieldName === 'databaseType') {
            this.updateDatabaseDependentFields(element.value);
        } else if (fieldName === 'httpsEnabled') {
            this.updateHttpsDependentFields(element.checked);
        }
    }

    updateDeploymentDependentFields(deploymentType) {
        switch (deploymentType) {
            case 'development':
                this.setFieldValue('databaseType', 'sqlite');
                this.setFieldValue('physicsEngine', 'ODE');
                this.setFieldValue('httpsEnabled', false);
                break;
            case 'production':
                this.setFieldValue('databaseType', 'postgresql');
                this.setFieldValue('httpsEnabled', true);
                break;
            case 'grid':
                this.setFieldValue('databaseType', 'postgresql');
                this.setFieldValue('physicsEngine', 'POS');
                this.setFieldValue('httpsEnabled', true);
                break;
        }
    }

    updateDatabaseDependentFields(databaseType) {
        const hostField = document.querySelector('[name="databaseHost"]');
        const portField = document.querySelector('[name="databasePort"]');
        
        if (databaseType === 'sqlite') {
            if (hostField) hostField.disabled = true;
            if (portField) portField.disabled = true;
        } else {
            if (hostField) hostField.disabled = false;
            if (portField) portField.disabled = false;
            
            if (databaseType === 'postgresql') {
                this.setFieldValue('databasePort', '5432');
            } else if (databaseType === 'mysql') {
                this.setFieldValue('databasePort', '3306');
            }
        }
    }

    updateHttpsDependentFields(httpsEnabled) {
        const httpsPortField = document.querySelector('[name="httpsPort"]');
        const certPathField = document.querySelector('[name="sslCertificatePath"]');
        
        if (httpsPortField) httpsPortField.disabled = !httpsEnabled;
        if (certPathField) certPathField.disabled = !httpsEnabled;
        
        if (httpsEnabled && httpsPortField && !httpsPortField.value) {
            this.setFieldValue('httpsPort', '9001');
        }
    }

    checkConfigurationConsistency(element, helpConfig) {
        // Check for configuration inconsistencies and suggest fixes
        const issues = this.findConfigurationIssues();
        
        if (issues.length > 0) {
            this.showConfigurationIssues(issues);
        }
    }

    findConfigurationIssues() {
        const issues = [];
        
        // Check port conflicts
        const httpPort = this.getFieldValue('httpPort');
        const httpsPort = this.getFieldValue('httpsPort');
        
        if (httpPort && httpsPort && httpPort === httpsPort) {
            issues.push({
                type: 'conflict',
                message: 'HTTP and HTTPS ports cannot be the same',
                severity: 'high',
                helpTopic: 'port-configuration'
            });
        }
        
        // Check database consistency
        const deploymentType = this.getFieldValue('deploymentType');
        const databaseType = this.getFieldValue('databaseType');
        
        if (deploymentType === 'production' && databaseType === 'sqlite') {
            issues.push({
                type: 'recommendation',
                message: 'SQLite not recommended for production deployment',
                severity: 'medium',
                helpTopic: 'database-selection'
            });
        }
        
        // Check security settings
        const httpsEnabled = this.getFieldValue('httpsEnabled');
        
        if (deploymentType === 'production' && !httpsEnabled) {
            issues.push({
                type: 'security',
                message: 'HTTPS recommended for production security',
                severity: 'high',
                helpTopic: 'https-configuration'
            });
        }
        
        return issues;
    }

    showConfigurationIssues(issues) {
        // Show configuration issues with help links
        const issuesContainer = document.querySelector('.configuration-issues') || this.createIssuesContainer();
        
        let html = '<h4>Configuration Issues</h4>';
        for (const issue of issues) {
            html += `
                <div class="config-issue ${issue.severity}">
                    <div class="issue-content">
                        <i class="icon-alert-triangle"></i>
                        <span>${issue.message}</span>
                    </div>
                    <button class="issue-help-btn" data-help-topic="${issue.helpTopic}">
                        <i class="icon-help-circle"></i>
                        Help
                    </button>
                </div>
            `;
        }
        
        issuesContainer.innerHTML = html;
        
        // Setup help button listeners
        issuesContainer.querySelectorAll('.issue-help-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const helpTopic = btn.dataset.helpTopic;
                // Find the category for this topic
                for (const [category, data] of this.helpSystem.helpDatabase.entries()) {
                    const topic = data.topics.find(t => t.id === helpTopic);
                    if (topic) {
                        this.helpSystem.showHelp(category, helpTopic);
                        break;
                    }
                }
            });
        });
    }

    createIssuesContainer() {
        const container = document.createElement('div');
        container.className = 'configuration-issues';
        
        // Insert into dashboard or validation panel
        const dashboard = document.getElementById('dashboard');
        if (dashboard) {
            dashboard.appendChild(container);
        }
        
        return container;
    }

    // Public API
    showFieldHelp(fieldName) {
        const helpConfig = this.findHelpForField(fieldName);
        if (helpConfig) {
            this.helpSystem.showHelp(helpConfig.category, helpConfig.topicId);
        }
    }

    validateAllFields() {
        let allValid = true;
        
        for (const [selector, helpConfig] of this.formFieldMappings.entries()) {
            const elements = document.querySelectorAll(selector);
            elements.forEach(element => {
                if (!this.validateFieldWithHelp(element, helpConfig)) {
                    allValid = false;
                }
            });
        }
        
        return allValid;
    }

    getHelpUsageStats() {
        // Return help usage statistics for analytics
        return {
            fieldsWithHelp: this.formFieldMappings.size,
            errorsWithHelp: this.errorHelp.size,
            suggestionGroups: this.contextualTips.size,
            helpEnabled: this.isInitialized
        };
    }
}

// Initialize help integration when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.helpIntegration = new HelpIntegration();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HelpIntegration;
}