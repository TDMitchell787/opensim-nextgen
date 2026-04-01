// OpenSim Next Auto-Configurator - Deployment Integration
// Integrates deployment selector with validation and optimization systems

class DeploymentIntegration {
    constructor() {
        this.deploymentSelector = null;
        this.configurationValidator = null;
        this.deploymentOptimizer = null;
        this.currentConfiguration = {};
        this.selectedDeploymentType = null;
        
        this.init();
    }

    async init() {
        try {
            // Initialize core components
            this.deploymentSelector = new DeploymentSelector();
            this.configurationValidator = new ConfigurationValidator();
            this.deploymentOptimizer = new DeploymentOptimizer();
            
            // Set up event listeners
            this.setupEventListeners();
            
            // Load saved configuration if available
            this.loadSavedConfiguration();
            
            console.log('Deployment integration initialized successfully');
        } catch (error) {
            console.error('Failed to initialize deployment integration:', error);
        }
    }

    setupEventListeners() {
        // Auto-detection button
        const autoDetectionBtn = document.getElementById('run-auto-detection');
        if (autoDetectionBtn) {
            autoDetectionBtn.addEventListener('click', () => this.runAutoDetection());
        }

        // Deployment selection buttons
        const deploymentButtons = document.querySelectorAll('[data-deployment]');
        deploymentButtons.forEach(button => {
            button.addEventListener('click', (e) => {
                const deploymentType = e.target.getAttribute('data-deployment');
                this.selectDeploymentType(deploymentType);
            });
        });

        // Comparison matrix toggle
        const comparisonBtn = document.getElementById('show-comparison');
        if (comparisonBtn) {
            comparisonBtn.addEventListener('click', () => this.toggleComparisonMatrix());
        }

        // Listen for configuration changes from other modules
        document.addEventListener('configurationUpdated', (event) => {
            this.handleConfigurationUpdate(event.detail);
        });

        // Listen for validation results
        document.addEventListener('validationCompleted', (event) => {
            this.handleValidationResults(event.detail);
        });
    }

    async runAutoDetection() {
        const autoDetectionBtn = document.getElementById('run-auto-detection');
        const resultsDiv = document.getElementById('auto-detection-results');
        
        try {
            // Show loading state
            autoDetectionBtn.disabled = true;
            autoDetectionBtn.innerHTML = '<i class="icon-spinner"></i> Analyzing...';
            
            // Gather system information
            const systemInfo = await this.gatherSystemInformation();
            
            // Run auto-detection
            const recommendation = this.deploymentSelector.autoDetectDeploymentType(systemInfo);
            
            // Display results
            this.displayAutoDetectionResults(recommendation);
            
            // Show results section
            resultsDiv.style.display = 'block';
            
            // Update deployment options with recommendations
            this.updateDeploymentRecommendations(recommendation);
            
        } catch (error) {
            console.error('Auto-detection failed:', error);
            this.showError('Auto-detection failed. Please select deployment type manually.');
        } finally {
            // Reset button state
            autoDetectionBtn.disabled = false;
            autoDetectionBtn.innerHTML = '<i class="icon-settings"></i> Analyze My Environment';
        }
    }

    async gatherSystemInformation() {
        // Simulate gathering system information
        // In a real implementation, this would collect actual system data
        
        const systemInfo = {
            // Hardware information (simulated)
            memory: this.detectMemory(),
            cpuCores: this.detectCPUCores(),
            
            // Network information
            hasPublicIP: await this.detectPublicIP(),
            bandwidth: this.estimateBandwidth(),
            domain: window.location.hostname,
            
            // Usage information (from user inputs or defaults)
            expectedUsers: this.getExpectedUsers(),
            expectedRegions: this.getExpectedRegions(),
            isCommercial: this.getCommercialUse()
        };

        console.log('Gathered system information:', systemInfo);
        return systemInfo;
    }

    detectMemory() {
        // Simulate memory detection
        if (navigator.deviceMemory) {
            return `${navigator.deviceMemory}GB`;
        }
        // Default assumption for web environment
        return '8GB';
    }

    detectCPUCores() {
        if (navigator.hardwareConcurrency) {
            return navigator.hardwareConcurrency;
        }
        // Default assumption
        return 4;
    }

    async detectPublicIP() {
        // Simple check for localhost vs public domain
        const hostname = window.location.hostname;
        return hostname !== 'localhost' && hostname !== '127.0.0.1';
    }

    estimateBandwidth() {
        // Simulate bandwidth estimation
        // In practice, this could use Network Information API or connection tests
        if (navigator.connection) {
            const effectiveType = navigator.connection.effectiveType;
            switch (effectiveType) {
                case '4g': return 1000;
                case '3g': return 100;
                case '2g': return 10;
                default: return 100;
            }
        }
        return 100; // Default to 100 Mbps
    }

    getExpectedUsers() {
        // Get from form inputs or use default
        const userInput = document.querySelector('[name="expected-users"]');
        return userInput ? parseInt(userInput.value) || 5 : 5;
    }

    getExpectedRegions() {
        // Get from form inputs or use default
        const regionInput = document.querySelector('[name="expected-regions"]');
        return regionInput ? parseInt(regionInput.value) || 1 : 1;
    }

    getCommercialUse() {
        // Get from form inputs or use default
        const commercialInput = document.querySelector('[name="commercial-use"]');
        return commercialInput ? commercialInput.checked : false;
    }

    displayAutoDetectionResults(recommendation) {
        // Update recommendation display
        const recommendedElement = document.getElementById('recommended-deployment');
        const reasoningElement = document.getElementById('recommendation-reasoning');
        const confidenceFill = document.getElementById('confidence-fill');
        const confidencePercentage = document.getElementById('confidence-percentage');
        const detailsContent = document.getElementById('detection-details-content');

        if (recommendedElement) {
            recommendedElement.textContent = this.formatDeploymentName(recommendation.recommendation);
        }

        if (reasoningElement) {
            const mainReason = recommendation.detectionResults.find(r => 
                r.recommendation === recommendation.recommendation
            );
            reasoningElement.textContent = mainReason ? mainReason.reasoning : 
                'Based on comprehensive system analysis';
        }

        if (confidenceFill && confidencePercentage) {
            const confidencePercent = Math.round(recommendation.confidence * 100);
            confidenceFill.style.width = `${confidencePercent}%`;
            confidencePercentage.textContent = `${confidencePercent}%`;
        }

        if (detailsContent) {
            this.displayDetectionDetails(detailsContent, recommendation);
        }
    }

    displayDetectionDetails(container, recommendation) {
        container.innerHTML = '';
        
        recommendation.detectionResults.forEach(result => {
            const ruleDiv = document.createElement('div');
            ruleDiv.className = 'detection-rule';
            
            const confidenceLevel = this.getConfidenceLevel(result.confidence);
            
            ruleDiv.innerHTML = `
                <div class="rule-info">
                    <div class="rule-name">${result.rule.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}</div>
                    <div class="rule-reasoning">${result.reasoning}</div>
                </div>
                <div class="rule-confidence">
                    <span class="confidence-badge ${confidenceLevel}">${confidenceLevel}</span>
                    <span>${Math.round(result.confidence * 100)}%</span>
                </div>
            `;
            
            container.appendChild(ruleDiv);
        });
    }

    getConfidenceLevel(confidence) {
        if (confidence >= 0.8) return 'high';
        if (confidence >= 0.6) return 'medium';
        return 'low';
    }

    formatDeploymentName(deploymentType) {
        const names = {
            'development': 'Development Environment',
            'production': 'Production Environment',
            'grid': 'Grid Environment'
        };
        return names[deploymentType] || deploymentType;
    }

    updateDeploymentRecommendations(recommendation) {
        // Clear existing recommendations
        document.querySelectorAll('.deployment-option').forEach(option => {
            option.classList.remove('recommended');
        });

        // Mark recommended option
        const recommendedOption = document.querySelector(`[data-type="${recommendation.recommendation}"]`);
        if (recommendedOption) {
            recommendedOption.classList.add('recommended');
        }
    }

    selectDeploymentType(deploymentType) {
        this.selectedDeploymentType = deploymentType;
        
        // Update UI
        this.updateDeploymentSelection(deploymentType);
        
        // Get default configuration for selected deployment type
        const defaultConfig = this.deploymentSelector.getDeploymentConfiguration(deploymentType);
        this.currentConfiguration = { ...defaultConfig, deploymentType };
        
        // Show migration information if applicable
        this.showMigrationInformation(deploymentType);
        
        // Trigger configuration validation
        this.validateConfiguration();
        
        // Update optimization recommendations
        this.updateOptimizationRecommendations();
        
        // Emit event for other modules
        document.dispatchEvent(new CustomEvent('deploymentTypeSelected', {
            detail: {
                deploymentType,
                configuration: this.currentConfiguration
            }
        }));
        
        console.log(`Selected deployment type: ${deploymentType}`);
    }

    updateDeploymentSelection(deploymentType) {
        // Clear existing selections
        document.querySelectorAll('.deployment-option').forEach(option => {
            option.classList.remove('selected');
        });

        // Mark selected option
        const selectedOption = document.querySelector(`[data-type="${deploymentType}"]`);
        if (selectedOption) {
            selectedOption.classList.add('selected');
        }

        // Update wizard step status
        this.updateWizardProgress();
    }

    showMigrationInformation(deploymentType) {
        const migrationInfo = document.getElementById('migration-info');
        const migrationDetails = document.getElementById('migration-path-details');
        
        if (!migrationInfo || !migrationDetails) return;

        // Check if there are available migration paths from current selection
        const availablePaths = this.deploymentSelector.getAvailableMigrationPaths(deploymentType);
        
        if (availablePaths.length > 0) {
            migrationInfo.style.display = 'block';
            migrationDetails.innerHTML = '';
            
            availablePaths.forEach(path => {
                const pathDiv = document.createElement('div');
                pathDiv.className = 'migration-path';
                
                pathDiv.innerHTML = `
                    <div class="migration-header">
                        <span class="migration-title">${path.name}</span>
                        <span class="migration-difficulty ${path.difficulty}">${path.difficulty}</span>
                    </div>
                    <div class="migration-time">Estimated time: ${path.estimatedTime}</div>
                    <div class="migration-description">${path.description}</div>
                    <ul class="migration-steps">
                        ${path.steps.map(step => `<li>${step.title}: ${step.description}</li>`).join('')}
                    </ul>
                `;
                
                migrationDetails.appendChild(pathDiv);
            });
        } else {
            migrationInfo.style.display = 'none';
        }
    }

    toggleComparisonMatrix() {
        const matrix = document.getElementById('comparison-matrix');
        const button = document.getElementById('show-comparison');
        
        if (!matrix || !button) return;
        
        if (matrix.style.display === 'none' || !matrix.style.display) {
            // Show comparison matrix
            this.generateComparisonMatrix();
            matrix.style.display = 'block';
            button.innerHTML = '<i class="icon-help"></i> Hide Comparison';
        } else {
            // Hide comparison matrix
            matrix.style.display = 'none';
            button.innerHTML = '<i class="icon-help"></i> Show Detailed Comparison';
        }
    }

    generateComparisonMatrix() {
        const matrix = document.getElementById('comparison-matrix');
        if (!matrix) return;
        
        const comparison = this.deploymentSelector.generateComparisonMatrix();
        
        const table = document.createElement('table');
        table.className = 'comparison-table';
        
        // Create header
        const thead = document.createElement('thead');
        const headerRow = document.createElement('tr');
        headerRow.innerHTML = `
            <th>Characteristic</th>
            ${comparison.deploymentTypes.map(type => `<th>${this.formatDeploymentName(type)}</th>`).join('')}
        `;
        thead.appendChild(headerRow);
        table.appendChild(thead);
        
        // Create body
        const tbody = document.createElement('tbody');
        
        // Add characteristics
        Object.entries(comparison.characteristics).forEach(([char, values]) => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${this.formatCharacteristicName(char)}</td>
                ${values.map(value => `<td>${this.formatCharacteristicValue(char, value)}</td>`).join('')}
            `;
            tbody.appendChild(row);
        });
        
        // Add technical specs
        Object.entries(comparison.technicalSpecs).forEach(([spec, values]) => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${this.formatCharacteristicName(spec)}</td>
                ${values.map(value => `<td>${value}</td>`).join('')}
            `;
            tbody.appendChild(row);
        });
        
        table.appendChild(tbody);
        matrix.innerHTML = '';
        matrix.appendChild(table);
    }

    formatCharacteristicName(name) {
        return name.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase());
    }

    formatCharacteristicValue(characteristic, value) {
        const scoreClass = this.getScoreClass(value);
        if (scoreClass) {
            return `<span class="characteristic-score">
                <span class="score-indicator ${scoreClass}"></span>
                ${value}
            </span>`;
        }
        return value;
    }

    getScoreClass(value) {
        const lowerValue = value.toLowerCase();
        if (lowerValue === 'excellent' || lowerValue === 'high') return 'excellent';
        if (lowerValue === 'good' || lowerValue === 'medium') return 'high';
        if (lowerValue === 'basic' || lowerValue === 'low') return 'medium';
        if (lowerValue === 'minimal' || lowerValue === 'limited') return 'low';
        return null;
    }

    async validateConfiguration() {
        if (!this.currentConfiguration || !this.selectedDeploymentType) return;
        
        try {
            // Run validation
            const validationResult = await this.configurationValidator.validateConfiguration(
                this.currentConfiguration
            );
            
            // Update validation display
            this.updateValidationStatus(validationResult);
            
        } catch (error) {
            console.error('Configuration validation failed:', error);
        }
    }

    updateOptimizationRecommendations() {
        if (!this.currentConfiguration || !this.selectedDeploymentType) return;
        
        try {
            // Get optimization analysis
            const analysis = this.deploymentOptimizer.analyzeConfiguration(this.currentConfiguration);
            
            // Update deployment info with optimization recommendations
            this.displayOptimizationRecommendations(analysis);
            
        } catch (error) {
            console.error('Optimization analysis failed:', error);
        }
    }

    displayOptimizationRecommendations(analysis) {
        // This would integrate with the validation panel to show optimization recommendations
        const event = new CustomEvent('optimizationCompleted', {
            detail: analysis
        });
        document.dispatchEvent(event);
    }

    updateValidationStatus(result) {
        // Update validation panel with deployment-specific results
        const event = new CustomEvent('deploymentValidationCompleted', {
            detail: {
                deploymentType: this.selectedDeploymentType,
                result: result
            }
        });
        document.dispatchEvent(event);
    }

    updateWizardProgress() {
        // Update progress indicator
        const progressFill = document.getElementById('progress-fill');
        if (progressFill && this.selectedDeploymentType) {
            progressFill.style.width = '28%'; // Step 1 of 7 completed with deployment selection
        }
    }

    handleConfigurationUpdate(configuration) {
        this.currentConfiguration = { ...this.currentConfiguration, ...configuration };
        this.validateConfiguration();
        this.updateOptimizationRecommendations();
    }

    handleValidationResults(results) {
        // Handle validation results from other modules
        console.log('Received validation results:', results);
    }

    saveConfiguration() {
        // Save current configuration to localStorage
        const configData = {
            deploymentType: this.selectedDeploymentType,
            configuration: this.currentConfiguration,
            timestamp: new Date().toISOString()
        };
        
        localStorage.setItem('opensim-auto-configurator', JSON.stringify(configData));
    }

    loadSavedConfiguration() {
        try {
            const saved = localStorage.getItem('opensim-auto-configurator');
            if (saved) {
                const configData = JSON.parse(saved);
                if (configData.deploymentType) {
                    this.selectDeploymentType(configData.deploymentType);
                }
            }
        } catch (error) {
            console.warn('Failed to load saved configuration:', error);
        }
    }

    showError(message) {
        // Simple error display - could be enhanced with proper toast/notification system
        const errorDiv = document.createElement('div');
        errorDiv.className = 'alert alert-warning';
        errorDiv.innerHTML = `<i class="icon-warning"></i> ${message}`;
        
        const container = document.getElementById('deployment-auto-detection');
        if (container) {
            container.appendChild(errorDiv);
            setTimeout(() => errorDiv.remove(), 5000);
        }
    }

    // Public API methods
    getSelectedDeploymentType() {
        return this.selectedDeploymentType;
    }

    getCurrentConfiguration() {
        return this.currentConfiguration;
    }

    getDeploymentInfo(deploymentType) {
        return this.deploymentSelector.getDeploymentTypeInfo(deploymentType);
    }
}

// Initialize deployment integration when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.deploymentIntegration = new DeploymentIntegration();
});

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.DeploymentIntegration = DeploymentIntegration;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { DeploymentIntegration };
}