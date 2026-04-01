// OpenSim Next Auto-Configurator - Internationalization (i18n) Manager
// Comprehensive multi-language support for global OpenSim Next adoption

class I18nManager {
    constructor() {
        this.currentLanguage = 'en';
        this.fallbackLanguage = 'en';
        this.translations = new Map();
        this.supportedLanguages = new Map();
        this.isInitialized = false;
        this.rtlLanguages = new Set(['ar', 'he', 'fa', 'ur']);
        this.observers = new Set();
        
        // Language detection preferences
        this.detectionOrder = [
            'localStorage',
            'navigator',
            'accept-language',
            'default'
        ];
        
        // Translation cache for performance
        this.translationCache = new Map();
        
        // Pluralization rules cache
        this.pluralRules = new Map();
        
        this.initializeI18n();
    }

    async initializeI18n() {
        try {
            // Initialize supported languages
            this.initializeSupportedLanguages();
            
            // Load default language (English)
            await this.loadTranslations('en');
            
            // Detect and set user's preferred language
            const detectedLanguage = this.detectUserLanguage();
            await this.setLanguage(detectedLanguage);
            
            // Setup language switching UI
            this.createLanguageSelector();
            
            // Setup mutation observer for dynamic content
            this.setupDynamicContentObserver();
            
            this.isInitialized = true;
            console.log(`✅ I18n Manager initialized with language: ${this.currentLanguage}`);
            
            // Notify observers
            this.notifyLanguageChange();
            
        } catch (error) {
            console.error('Failed to initialize i18n manager:', error);
            // Fallback to English
            this.currentLanguage = 'en';
        }
    }

    initializeSupportedLanguages() {
        // Major languages with significant OpenSim user bases
        this.supportedLanguages.set('en', {
            name: 'English',
            nativeName: 'English',
            flag: '🇺🇸',
            region: 'US',
            rtl: false,
            pluralRule: 'en'
        });
        
        this.supportedLanguages.set('es', {
            name: 'Spanish',
            nativeName: 'Español',
            flag: '🇪🇸',
            region: 'ES',
            rtl: false,
            pluralRule: 'es'
        });
        
        this.supportedLanguages.set('fr', {
            name: 'French',
            nativeName: 'Français',
            flag: '🇫🇷',
            region: 'FR',
            rtl: false,
            pluralRule: 'fr'
        });
        
        this.supportedLanguages.set('de', {
            name: 'German',
            nativeName: 'Deutsch',
            flag: '🇩🇪',
            region: 'DE',
            rtl: false,
            pluralRule: 'de'
        });
        
        this.supportedLanguages.set('it', {
            name: 'Italian',
            nativeName: 'Italiano',
            flag: '🇮🇹',
            region: 'IT',
            rtl: false,
            pluralRule: 'it'
        });
        
        this.supportedLanguages.set('pt', {
            name: 'Portuguese',
            nativeName: 'Português',
            flag: '🇵🇹',
            region: 'PT',
            rtl: false,
            pluralRule: 'pt'
        });
        
        this.supportedLanguages.set('ru', {
            name: 'Russian',
            nativeName: 'Русский',
            flag: '🇷🇺',
            region: 'RU',
            rtl: false,
            pluralRule: 'ru'
        });
        
        this.supportedLanguages.set('zh', {
            name: 'Chinese (Simplified)',
            nativeName: '简体中文',
            flag: '🇨🇳',
            region: 'CN',
            rtl: false,
            pluralRule: 'zh'
        });
        
        this.supportedLanguages.set('ja', {
            name: 'Japanese',
            nativeName: '日本語',
            flag: '🇯🇵',
            region: 'JP',
            rtl: false,
            pluralRule: 'ja'
        });
        
        this.supportedLanguages.set('ko', {
            name: 'Korean',
            nativeName: '한국어',
            flag: '🇰🇷',
            region: 'KR',
            rtl: false,
            pluralRule: 'ko'
        });
        
        this.supportedLanguages.set('ar', {
            name: 'Arabic',
            nativeName: 'العربية',
            flag: '🇸🇦',
            region: 'SA',
            rtl: true,
            pluralRule: 'ar'
        });
        
        this.supportedLanguages.set('tr', {
            name: 'Turkish',
            nativeName: 'Türkçe',
            flag: '🇹🇷',
            region: 'TR',
            rtl: false,
            pluralRule: 'tr'
        });
        
        this.supportedLanguages.set('nl', {
            name: 'Dutch',
            nativeName: 'Nederlands',
            flag: '🇳🇱',
            region: 'NL',
            rtl: false,
            pluralRule: 'nl'
        });
        
        this.supportedLanguages.set('sv', {
            name: 'Swedish',
            nativeName: 'Svenska',
            flag: '🇸🇪',
            region: 'SE',
            rtl: false,
            pluralRule: 'sv'
        });
        
        this.supportedLanguages.set('no', {
            name: 'Norwegian',
            nativeName: 'Norsk',
            flag: '🇳🇴',
            region: 'NO',
            rtl: false,
            pluralRule: 'no'
        });
        
        this.supportedLanguages.set('da', {
            name: 'Danish',
            nativeName: 'Dansk',
            flag: '🇩🇰',
            region: 'DK',
            rtl: false,
            pluralRule: 'da'
        });
        
        this.supportedLanguages.set('pl', {
            name: 'Polish',
            nativeName: 'Polski',
            flag: '🇵🇱',
            region: 'PL',
            rtl: false,
            pluralRule: 'pl'
        });
        
        this.supportedLanguages.set('cs', {
            name: 'Czech',
            nativeName: 'Čeština',
            flag: '🇨🇿',
            region: 'CZ',
            rtl: false,
            pluralRule: 'cs'
        });
        
        this.supportedLanguages.set('hu', {
            name: 'Hungarian',
            nativeName: 'Magyar',
            flag: '🇭🇺',
            region: 'HU',
            rtl: false,
            pluralRule: 'hu'
        });
        
        this.supportedLanguages.set('fi', {
            name: 'Finnish',
            nativeName: 'Suomi',
            flag: '🇫🇮',
            region: 'FI',
            rtl: false,
            pluralRule: 'fi'
        });
    }

    async loadTranslations(languageCode) {
        try {
            // Try to load from server first, then fall back to built-in translations
            let translations;
            
            try {
                const response = await fetch(`/i18n/${languageCode}.json`);
                if (response.ok) {
                    translations = await response.json();
                }
            } catch (error) {
                console.log(`Loading built-in translations for ${languageCode}`);
            }
            
            // Fall back to built-in translations
            if (!translations) {
                translations = this.getBuiltInTranslations(languageCode);
            }
            
            this.translations.set(languageCode, translations);
            
            // Load plural rules for this language
            this.loadPluralRules(languageCode);
            
            return translations;
            
        } catch (error) {
            console.error(`Failed to load translations for ${languageCode}:`, error);
            return null;
        }
    }

    getBuiltInTranslations(languageCode) {
        // Built-in translations for core functionality
        const translations = {
            en: {
                // Navigation and General
                'nav.dashboard': 'Dashboard',
                'nav.wizard': 'Configuration Wizard',
                'nav.testing': 'Testing Framework',
                'nav.backup': 'Backup & Restore',
                'nav.help': 'Help',
                'nav.settings': 'Settings',
                
                // Common Actions
                'action.save': 'Save',
                'action.cancel': 'Cancel',
                'action.continue': 'Continue',
                'action.back': 'Back',
                'action.next': 'Next',
                'action.finish': 'Finish',
                'action.export': 'Export',
                'action.import': 'Import',
                'action.delete': 'Delete',
                'action.edit': 'Edit',
                'action.add': 'Add',
                'action.remove': 'Remove',
                'action.upload': 'Upload',
                'action.download': 'Download',
                'action.refresh': 'Refresh',
                'action.reset': 'Reset',
                'action.apply': 'Apply',
                'action.test': 'Test',
                'action.validate': 'Validate',
                
                // Status Messages
                'status.success': 'Success',
                'status.error': 'Error',
                'status.warning': 'Warning',
                'status.info': 'Information',
                'status.loading': 'Loading...',
                'status.saving': 'Saving...',
                'status.complete': 'Complete',
                'status.pending': 'Pending',
                'status.failed': 'Failed',
                'status.in_progress': 'In Progress',
                
                // Configuration
                'config.title': 'OpenSim Next Auto-Configurator',
                'config.subtitle': 'Intelligent configuration wizard for OpenSim Next virtual world server',
                'config.deployment_type': 'Deployment Type',
                'config.environment': 'Environment Configuration',
                'config.database': 'Database Setup',
                'config.regions': 'Region Configuration',
                'config.security': 'Security Configuration',
                'config.network': 'Network Configuration',
                'config.review': 'Review & Deploy',
                
                // Deployment Types
                'deploy.development': 'Development Environment',
                'deploy.development.desc': 'Optimized for rapid development, testing, and learning',
                'deploy.production': 'Production Environment',
                'deploy.production.desc': 'Battle-tested configuration for live virtual worlds',
                'deploy.grid': 'Grid Environment',
                'deploy.grid.desc': 'Distributed multi-server architecture for massive scale',
                
                // Testing Framework
                'test.title': 'Configuration Testing Framework',
                'test.subtitle': 'Automated testing and validation for your OpenSim Next configuration',
                'test.quick': 'Quick Test',
                'test.full': 'Full Test Suite',
                'test.custom': 'Custom Tests',
                'test.results': 'Test Results',
                'test.run': 'Run Tests',
                'test.passed': 'Passed',
                'test.failed': 'Failed',
                'test.skipped': 'Skipped',
                'test.duration': 'Duration',
                
                // Validation Messages
                'validation.required': 'This field is required',
                'validation.invalid_email': 'Please enter a valid email address',
                'validation.invalid_url': 'Please enter a valid URL',
                'validation.min_length': 'Minimum length is {min} characters',
                'validation.max_length': 'Maximum length is {max} characters',
                'validation.invalid_port': 'Please enter a valid port number (1-65535)',
                'validation.invalid_ip': 'Please enter a valid IP address',
                'validation.password_weak': 'Password is too weak',
                'validation.passwords_match': 'Passwords do not match',
                
                // Error Messages
                'error.network': 'Network error occurred',
                'error.server': 'Server error occurred',
                'error.permission': 'Permission denied',
                'error.not_found': 'Resource not found',
                'error.invalid_input': 'Invalid input provided',
                'error.file_too_large': 'File size exceeds maximum limit',
                'error.unsupported_format': 'Unsupported file format',
                
                // Help & Documentation
                'help.title': 'Help & Documentation',
                'help.getting_started': 'Getting Started',
                'help.configuration': 'Configuration Guide',
                'help.troubleshooting': 'Troubleshooting',
                'help.faq': 'Frequently Asked Questions',
                'help.contact': 'Contact Support',
                
                // Language Selector
                'lang.select': 'Select Language',
                'lang.current': 'Current Language',
                'lang.change': 'Change Language',
                'lang.auto_detect': 'Auto-detect language from browser',
                
                // Pluralization Examples
                'item.count': '{count} item',
                'item.count_plural': '{count} items',
                'test.count': '{count} test',
                'test.count_plural': '{count} tests',
                'error.count': '{count} error',
                'error.count_plural': '{count} errors'
            },
            
            es: {
                // Navigation and General
                'nav.dashboard': 'Panel de Control',
                'nav.wizard': 'Asistente de Configuración',
                'nav.testing': 'Marco de Pruebas',
                'nav.backup': 'Copia de Seguridad y Restauración',
                'nav.help': 'Ayuda',
                'nav.settings': 'Configuración',
                
                // Common Actions
                'action.save': 'Guardar',
                'action.cancel': 'Cancelar',
                'action.continue': 'Continuar',
                'action.back': 'Atrás',
                'action.next': 'Siguiente',
                'action.finish': 'Terminar',
                'action.export': 'Exportar',
                'action.import': 'Importar',
                'action.delete': 'Eliminar',
                'action.edit': 'Editar',
                'action.add': 'Agregar',
                'action.remove': 'Quitar',
                'action.upload': 'Subir',
                'action.download': 'Descargar',
                'action.refresh': 'Actualizar',
                'action.reset': 'Restablecer',
                'action.apply': 'Aplicar',
                'action.test': 'Probar',
                'action.validate': 'Validar',
                
                // Status Messages
                'status.success': 'Éxito',
                'status.error': 'Error',
                'status.warning': 'Advertencia',
                'status.info': 'Información',
                'status.loading': 'Cargando...',
                'status.saving': 'Guardando...',
                'status.complete': 'Completo',
                'status.pending': 'Pendiente',
                'status.failed': 'Fallido',
                'status.in_progress': 'En Progreso',
                
                // Configuration
                'config.title': 'Autoconfigurador OpenSim Next',
                'config.subtitle': 'Asistente de configuración inteligente para servidor de mundo virtual OpenSim Next',
                'config.deployment_type': 'Tipo de Implementación',
                'config.environment': 'Configuración del Entorno',
                'config.database': 'Configuración de Base de Datos',
                'config.regions': 'Configuración de Regiones',
                'config.security': 'Configuración de Seguridad',
                'config.network': 'Configuración de Red',
                'config.review': 'Revisar e Implementar',
                
                // Deployment Types
                'deploy.development': 'Entorno de Desarrollo',
                'deploy.development.desc': 'Optimizado para desarrollo rápido, pruebas y aprendizaje',
                'deploy.production': 'Entorno de Producción',
                'deploy.production.desc': 'Configuración probada en batalla para mundos virtuales en vivo',
                'deploy.grid': 'Entorno de Grid',
                'deploy.grid.desc': 'Arquitectura distribuida multi-servidor para escala masiva',
                
                // Testing Framework
                'test.title': 'Marco de Pruebas de Configuración',
                'test.subtitle': 'Pruebas y validación automatizadas para tu configuración OpenSim Next',
                'test.quick': 'Prueba Rápida',
                'test.full': 'Suite de Pruebas Completa',
                'test.custom': 'Pruebas Personalizadas',
                'test.results': 'Resultados de Pruebas',
                'test.run': 'Ejecutar Pruebas',
                'test.passed': 'Pasado',
                'test.failed': 'Fallido',
                'test.skipped': 'Omitido',
                'test.duration': 'Duración',
                
                // Validation Messages
                'validation.required': 'Este campo es obligatorio',
                'validation.invalid_email': 'Por favor ingrese una dirección de email válida',
                'validation.invalid_url': 'Por favor ingrese una URL válida',
                'validation.min_length': 'La longitud mínima es {min} caracteres',
                'validation.max_length': 'La longitud máxima es {max} caracteres',
                'validation.invalid_port': 'Por favor ingrese un número de puerto válido (1-65535)',
                'validation.invalid_ip': 'Por favor ingrese una dirección IP válida',
                'validation.password_weak': 'La contraseña es muy débil',
                'validation.passwords_match': 'Las contraseñas no coinciden',
                
                // Error Messages
                'error.network': 'Ocurrió un error de red',
                'error.server': 'Ocurrió un error del servidor',
                'error.permission': 'Permiso denegado',
                'error.not_found': 'Recurso no encontrado',
                'error.invalid_input': 'Entrada inválida proporcionada',
                'error.file_too_large': 'El tamaño del archivo excede el límite máximo',
                'error.unsupported_format': 'Formato de archivo no soportado',
                
                // Help & Documentation
                'help.title': 'Ayuda y Documentación',
                'help.getting_started': 'Comenzando',
                'help.configuration': 'Guía de Configuración',
                'help.troubleshooting': 'Solución de Problemas',
                'help.faq': 'Preguntas Frecuentes',
                'help.contact': 'Contactar Soporte',
                
                // Language Selector
                'lang.select': 'Seleccionar Idioma',
                'lang.current': 'Idioma Actual',
                'lang.change': 'Cambiar Idioma',
                'lang.auto_detect': 'Detectar automáticamente el idioma del navegador',
                
                // Pluralization Examples
                'item.count': '{count} elemento',
                'item.count_plural': '{count} elementos',
                'test.count': '{count} prueba',
                'test.count_plural': '{count} pruebas',
                'error.count': '{count} error',
                'error.count_plural': '{count} errores'
            },
            
            fr: {
                // Navigation and General
                'nav.dashboard': 'Tableau de Bord',
                'nav.wizard': 'Assistant de Configuration',
                'nav.testing': 'Framework de Test',
                'nav.backup': 'Sauvegarde et Restauration',
                'nav.help': 'Aide',
                'nav.settings': 'Paramètres',
                
                // Common Actions
                'action.save': 'Enregistrer',
                'action.cancel': 'Annuler',
                'action.continue': 'Continuer',
                'action.back': 'Retour',
                'action.next': 'Suivant',
                'action.finish': 'Terminer',
                'action.export': 'Exporter',
                'action.import': 'Importer',
                'action.delete': 'Supprimer',
                'action.edit': 'Éditer',
                'action.add': 'Ajouter',
                'action.remove': 'Retirer',
                'action.upload': 'Télécharger',
                'action.download': 'Télécharger',
                'action.refresh': 'Actualiser',
                'action.reset': 'Réinitialiser',
                'action.apply': 'Appliquer',
                'action.test': 'Tester',
                'action.validate': 'Valider',
                
                // Status Messages
                'status.success': 'Succès',
                'status.error': 'Erreur',
                'status.warning': 'Avertissement',
                'status.info': 'Information',
                'status.loading': 'Chargement...',
                'status.saving': 'Enregistrement...',
                'status.complete': 'Terminé',
                'status.pending': 'En attente',
                'status.failed': 'Échec',
                'status.in_progress': 'En cours',
                
                // Configuration
                'config.title': 'Auto-configurateur OpenSim Next',
                'config.subtitle': 'Assistant de configuration intelligent pour serveur de monde virtuel OpenSim Next',
                'config.deployment_type': 'Type de Déploiement',
                'config.environment': 'Configuration de l\'Environnement',
                'config.database': 'Configuration de Base de Données',
                'config.regions': 'Configuration des Régions',
                'config.security': 'Configuration de Sécurité',
                'config.network': 'Configuration Réseau',
                'config.review': 'Révision et Déploiement',
                
                // Language Selector
                'lang.select': 'Sélectionner la Langue',
                'lang.current': 'Langue Actuelle',
                'lang.change': 'Changer de Langue',
                'lang.auto_detect': 'Détection automatique de la langue du navigateur',
                
                // Pluralization Examples
                'item.count': '{count} élément',
                'item.count_plural': '{count} éléments',
                'test.count': '{count} test',
                'test.count_plural': '{count} tests',
                'error.count': '{count} erreur',
                'error.count_plural': '{count} erreurs'
            },
            
            de: {
                // Navigation and General
                'nav.dashboard': 'Dashboard',
                'nav.wizard': 'Konfigurationsassistent',
                'nav.testing': 'Test-Framework',
                'nav.backup': 'Sicherung & Wiederherstellung',
                'nav.help': 'Hilfe',
                'nav.settings': 'Einstellungen',
                
                // Common Actions
                'action.save': 'Speichern',
                'action.cancel': 'Abbrechen',
                'action.continue': 'Fortfahren',
                'action.back': 'Zurück',
                'action.next': 'Weiter',
                'action.finish': 'Beenden',
                'action.export': 'Exportieren',
                'action.import': 'Importieren',
                'action.delete': 'Löschen',
                'action.edit': 'Bearbeiten',
                'action.add': 'Hinzufügen',
                'action.remove': 'Entfernen',
                'action.upload': 'Hochladen',
                'action.download': 'Herunterladen',
                'action.refresh': 'Aktualisieren',
                'action.reset': 'Zurücksetzen',
                'action.apply': 'Anwenden',
                'action.test': 'Testen',
                'action.validate': 'Validieren',
                
                // Status Messages
                'status.success': 'Erfolg',
                'status.error': 'Fehler',
                'status.warning': 'Warnung',
                'status.info': 'Information',
                'status.loading': 'Lädt...',
                'status.saving': 'Speichert...',
                'status.complete': 'Abgeschlossen',
                'status.pending': 'Ausstehend',
                'status.failed': 'Fehlgeschlagen',
                'status.in_progress': 'In Bearbeitung',
                
                // Configuration
                'config.title': 'OpenSim Next Auto-Konfigurator',
                'config.subtitle': 'Intelligenter Konfigurationsassistent für OpenSim Next Virtual World Server',
                
                // Language Selector
                'lang.select': 'Sprache Auswählen',
                'lang.current': 'Aktuelle Sprache',
                'lang.change': 'Sprache Ändern',
                'lang.auto_detect': 'Sprache automatisch vom Browser erkennen',
                
                // Pluralization Examples
                'item.count': '{count} Element',
                'item.count_plural': '{count} Elemente',
                'test.count': '{count} Test',
                'test.count_plural': '{count} Tests',
                'error.count': '{count} Fehler',
                'error.count_plural': '{count} Fehler'
            }
        };
        
        return translations[languageCode] || translations['en'];
    }

    loadPluralRules(languageCode) {
        // Initialize Intl.PluralRules for the language
        try {
            this.pluralRules.set(languageCode, new Intl.PluralRules(languageCode));
        } catch (error) {
            console.warn(`Plural rules not available for ${languageCode}, using English rules`);
            this.pluralRules.set(languageCode, new Intl.PluralRules('en'));
        }
    }

    detectUserLanguage() {
        for (const method of this.detectionOrder) {
            const detectedLang = this.detectLanguageByMethod(method);
            if (detectedLang && this.supportedLanguages.has(detectedLang)) {
                return detectedLang;
            }
        }
        return this.fallbackLanguage;
    }

    detectLanguageByMethod(method) {
        switch (method) {
            case 'localStorage':
                return localStorage.getItem('opensim-language');
                
            case 'navigator':
                if (navigator.language) {
                    return navigator.language.split('-')[0];
                }
                break;
                
            case 'accept-language':
                if (navigator.languages) {
                    for (const lang of navigator.languages) {
                        const code = lang.split('-')[0];
                        if (this.supportedLanguages.has(code)) {
                            return code;
                        }
                    }
                }
                break;
                
            case 'default':
                return this.fallbackLanguage;
        }
        return null;
    }

    async setLanguage(languageCode) {
        if (!this.supportedLanguages.has(languageCode)) {
            console.warn(`Language ${languageCode} not supported, falling back to ${this.fallbackLanguage}`);
            languageCode = this.fallbackLanguage;
        }
        
        // Load translations if not already loaded
        if (!this.translations.has(languageCode)) {
            await this.loadTranslations(languageCode);
        }
        
        const previousLanguage = this.currentLanguage;
        this.currentLanguage = languageCode;
        
        // Store in localStorage
        localStorage.setItem('opensim-language', languageCode);
        
        // Update document language and direction
        this.updateDocumentLanguage();
        
        // Clear translation cache
        this.translationCache.clear();
        
        // Translate all existing content
        this.translatePage();
        
        // Update language selector
        this.updateLanguageSelector();
        
        // Notify observers of language change
        this.notifyLanguageChange(previousLanguage);
        
        console.log(`Language changed to: ${languageCode}`);
    }

    updateDocumentLanguage() {
        const langInfo = this.supportedLanguages.get(this.currentLanguage);
        
        // Update document language
        document.documentElement.lang = this.currentLanguage;
        
        // Update text direction
        document.documentElement.dir = langInfo.rtl ? 'rtl' : 'ltr';
        
        // Add language class to body
        document.body.className = document.body.className.replace(/lang-\w+/g, '');
        document.body.classList.add(`lang-${this.currentLanguage}`);
        
        // Add RTL class if needed
        document.body.classList.toggle('rtl', langInfo.rtl);
    }

    translate(key, params = {}) {
        // Check cache first
        const cacheKey = `${this.currentLanguage}:${key}:${JSON.stringify(params)}`;
        if (this.translationCache.has(cacheKey)) {
            return this.translationCache.get(cacheKey);
        }
        
        let translation = this.getTranslation(key);
        
        // Handle pluralization
        if (params.count !== undefined) {
            translation = this.handlePluralization(key, params.count, translation);
        }
        
        // Replace parameters
        translation = this.replaceParameters(translation, params);
        
        // Cache the result
        this.translationCache.set(cacheKey, translation);
        
        return translation;
    }

    getTranslation(key) {
        const currentTranslations = this.translations.get(this.currentLanguage);
        const fallbackTranslations = this.translations.get(this.fallbackLanguage);
        
        // Try current language first
        if (currentTranslations && currentTranslations[key]) {
            return currentTranslations[key];
        }
        
        // Fall back to English
        if (fallbackTranslations && fallbackTranslations[key]) {
            return fallbackTranslations[key];
        }
        
        // Return key if no translation found
        console.warn(`Translation not found for key: ${key}`);
        return key;
    }

    handlePluralization(key, count, translation) {
        const pluralRule = this.pluralRules.get(this.currentLanguage);
        const rule = pluralRule.select(count);
        
        // Look for plural form
        const pluralKey = `${key}_${rule}`;
        const pluralTranslation = this.getTranslation(pluralKey);
        
        if (pluralTranslation !== pluralKey) {
            return pluralTranslation;
        }
        
        // Fall back to simple plural rule
        if (count !== 1) {
            const simplePluralKey = `${key}_plural`;
            const simplePluralTranslation = this.getTranslation(simplePluralKey);
            if (simplePluralTranslation !== simplePluralKey) {
                return simplePluralTranslation;
            }
        }
        
        return translation;
    }

    replaceParameters(text, params) {
        return text.replace(/\{(\w+)\}/g, (match, key) => {
            return params[key] !== undefined ? params[key] : match;
        });
    }

    translatePage() {
        // Translate elements with data-i18n attribute
        document.querySelectorAll('[data-i18n]').forEach(element => {
            const key = element.getAttribute('data-i18n');
            const params = this.parseDataParams(element);
            element.textContent = this.translate(key, params);
        });
        
        // Translate elements with data-i18n-placeholder attribute
        document.querySelectorAll('[data-i18n-placeholder]').forEach(element => {
            const key = element.getAttribute('data-i18n-placeholder');
            const params = this.parseDataParams(element);
            element.placeholder = this.translate(key, params);
        });
        
        // Translate elements with data-i18n-title attribute
        document.querySelectorAll('[data-i18n-title]').forEach(element => {
            const key = element.getAttribute('data-i18n-title');
            const params = this.parseDataParams(element);
            element.title = this.translate(key, params);
        });
        
        // Translate elements with data-i18n-html attribute (be careful with XSS)
        document.querySelectorAll('[data-i18n-html]').forEach(element => {
            const key = element.getAttribute('data-i18n-html');
            const params = this.parseDataParams(element);
            element.innerHTML = this.translate(key, params);
        });
    }

    parseDataParams(element) {
        const paramsAttr = element.getAttribute('data-i18n-params');
        if (!paramsAttr) return {};
        
        try {
            return JSON.parse(paramsAttr);
        } catch (error) {
            console.warn('Invalid data-i18n-params:', paramsAttr);
            return {};
        }
    }

    createLanguageSelector() {
        const header = document.querySelector('.header-actions');
        if (!header) return;
        
        const languageSelector = document.createElement('div');
        languageSelector.className = 'language-selector';
        languageSelector.innerHTML = `
            <button class="btn btn-secondary language-toggle" id="language-toggle">
                <i class="icon-globe"></i>
                <span class="current-language">${this.supportedLanguages.get(this.currentLanguage).flag}</span>
            </button>
            <div class="language-dropdown" id="language-dropdown">
                <div class="language-dropdown-header">
                    <h4 data-i18n="lang.select">Select Language</h4>
                </div>
                <div class="language-options" id="language-options">
                    ${Array.from(this.supportedLanguages.entries()).map(([code, info]) => `
                        <div class="language-option ${code === this.currentLanguage ? 'active' : ''}" 
                             data-language="${code}">
                            <span class="language-flag">${info.flag}</span>
                            <div class="language-info">
                                <div class="language-name">${info.name}</div>
                                <div class="language-native">${info.nativeName}</div>
                            </div>
                        </div>
                    `).join('')}
                </div>
                <div class="language-dropdown-footer">
                    <label class="auto-detect-option">
                        <input type="checkbox" id="auto-detect-language">
                        <span data-i18n="lang.auto_detect">Auto-detect language from browser</span>
                    </label>
                </div>
            </div>
        `;
        
        header.insertBefore(languageSelector, header.firstChild);
        
        // Setup event listeners
        this.setupLanguageSelectorEvents();
    }

    setupLanguageSelectorEvents() {
        const toggle = document.getElementById('language-toggle');
        const dropdown = document.getElementById('language-dropdown');
        
        if (toggle && dropdown) {
            toggle.addEventListener('click', (e) => {
                e.stopPropagation();
                dropdown.classList.toggle('show');
            });
            
            // Close dropdown when clicking outside
            document.addEventListener('click', () => {
                dropdown.classList.remove('show');
            });
            
            dropdown.addEventListener('click', (e) => {
                e.stopPropagation();
            });
        }
        
        // Language option clicks
        document.querySelectorAll('.language-option').forEach(option => {
            option.addEventListener('click', async () => {
                const languageCode = option.getAttribute('data-language');
                await this.setLanguage(languageCode);
                dropdown.classList.remove('show');
            });
        });
        
        // Auto-detect checkbox
        const autoDetect = document.getElementById('auto-detect-language');
        if (autoDetect) {
            autoDetect.checked = localStorage.getItem('opensim-auto-detect-language') === 'true';
            autoDetect.addEventListener('change', () => {
                localStorage.setItem('opensim-auto-detect-language', autoDetect.checked);
                if (autoDetect.checked) {
                    const detectedLang = this.detectUserLanguage();
                    if (detectedLang !== this.currentLanguage) {
                        this.setLanguage(detectedLang);
                    }
                }
            });
        }
    }

    updateLanguageSelector() {
        const currentFlag = document.querySelector('.current-language');
        const options = document.querySelectorAll('.language-option');
        
        if (currentFlag) {
            currentFlag.textContent = this.supportedLanguages.get(this.currentLanguage).flag;
        }
        
        options.forEach(option => {
            const code = option.getAttribute('data-language');
            option.classList.toggle('active', code === this.currentLanguage);
        });
    }

    setupDynamicContentObserver() {
        // Observer for dynamically added content
        const observer = new MutationObserver((mutations) => {
            mutations.forEach((mutation) => {
                if (mutation.type === 'childList') {
                    mutation.addedNodes.forEach((node) => {
                        if (node.nodeType === Node.ELEMENT_NODE) {
                            this.translateElement(node);
                        }
                    });
                }
            });
        });
        
        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    }

    translateElement(element) {
        // Translate the element itself
        if (element.hasAttribute('data-i18n')) {
            const key = element.getAttribute('data-i18n');
            const params = this.parseDataParams(element);
            element.textContent = this.translate(key, params);
        }
        
        // Translate child elements
        element.querySelectorAll('[data-i18n]').forEach(child => {
            const key = child.getAttribute('data-i18n');
            const params = this.parseDataParams(child);
            child.textContent = this.translate(key, params);
        });
        
        // Handle other i18n attributes
        element.querySelectorAll('[data-i18n-placeholder]').forEach(child => {
            const key = child.getAttribute('data-i18n-placeholder');
            const params = this.parseDataParams(child);
            child.placeholder = this.translate(key, params);
        });
        
        element.querySelectorAll('[data-i18n-title]').forEach(child => {
            const key = child.getAttribute('data-i18n-title');
            const params = this.parseDataParams(child);
            child.title = this.translate(key, params);
        });
    }

    // Observer management
    addLanguageChangeObserver(callback) {
        this.observers.add(callback);
    }

    removeLanguageChangeObserver(callback) {
        this.observers.delete(callback);
    }

    notifyLanguageChange(previousLanguage = null) {
        this.observers.forEach(callback => {
            try {
                callback(this.currentLanguage, previousLanguage);
            } catch (error) {
                console.error('Error in language change observer:', error);
            }
        });
    }

    // Utility methods
    getCurrentLanguage() {
        return this.currentLanguage;
    }

    getSupportedLanguages() {
        return Array.from(this.supportedLanguages.entries()).map(([code, info]) => ({
            code,
            ...info
        }));
    }

    isRTL() {
        return this.supportedLanguages.get(this.currentLanguage)?.rtl || false;
    }

    formatNumber(number, options = {}) {
        try {
            return new Intl.NumberFormat(this.currentLanguage, options).format(number);
        } catch (error) {
            return number.toString();
        }
    }

    formatDate(date, options = {}) {
        try {
            return new Intl.DateTimeFormat(this.currentLanguage, options).format(date);
        } catch (error) {
            return date.toString();
        }
    }

    formatRelativeTime(value, unit) {
        try {
            const rtf = new Intl.RelativeTimeFormat(this.currentLanguage, { numeric: 'auto' });
            return rtf.format(value, unit);
        } catch (error) {
            return `${value} ${unit}${Math.abs(value) !== 1 ? 's' : ''} ago`;
        }
    }

    // Export translations for external use
    exportTranslations(languageCode = this.currentLanguage) {
        return this.translations.get(languageCode) || {};
    }

    // Import custom translations
    importTranslations(languageCode, translations) {
        const existing = this.translations.get(languageCode) || {};
        this.translations.set(languageCode, { ...existing, ...translations });
        
        if (languageCode === this.currentLanguage) {
            this.translationCache.clear();
            this.translatePage();
        }
    }
}

// Global translation function
window.t = function(key, params = {}) {
    if (window.i18nManager) {
        return window.i18nManager.translate(key, params);
    }
    return key;
};

// Initialize i18n manager when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.i18nManager = new I18nManager();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = I18nManager;
}