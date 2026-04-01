// OpenSim Next Auto-Configurator - Comprehensive Help System
// Contextual documentation and intelligent assistance system

class HelpSystem {
    constructor() {
        this.helpDatabase = new Map();
        this.contextualHelp = new Map();
        this.searchIndex = new Map();
        this.helpHistory = [];
        this.currentContext = null;
        this.isInitialized = false;
        
        // Help system configuration
        this.settings = {
            enableTooltips: true,
            enableContextualHelp: true,
            enableSmartSuggestions: true,
            showHelpOnError: true,
            autoShowHelp: false,
            helpAnimations: true,
            searchDelay: 300
        };
        
        // Documentation sources
        this.documentationSources = {
            local: '/docs/',
            github: 'https://github.com/opensim/opensim-next/wiki/',
            community: 'https://opensim.org/wiki/',
            official: 'https://opensimulator.org/wiki/'
        };
        
        this.initializeHelpSystem();
    }

    async initializeHelpSystem() {
        try {
            // Load help database
            await this.loadHelpDatabase();
            
            // Build search index
            this.buildSearchIndex();
            
            // Setup contextual help
            this.setupContextualHelp();
            
            // Initialize UI components
            this.createHelpInterface();
            
            // Setup event listeners
            this.setupEventListeners();
            
            // Load user preferences
            this.loadUserPreferences();
            
            this.isInitialized = true;
            console.log('✅ Help system initialized successfully');
            
        } catch (error) {
            console.error('Failed to initialize help system:', error);
        }
    }

    async loadHelpDatabase() {
        // Comprehensive help database with topics, solutions, and links
        this.helpDatabase.set('general', {
            category: 'General Configuration',
            icon: 'icon-settings',
            color: '#3b82f6',
            topics: [
                {
                    id: 'grid-name',
                    title: 'Grid Name Configuration',
                    summary: 'Setting up your virtual world grid name and identity',
                    content: `
                        <h4>Grid Name Configuration</h4>
                        <p>The grid name is the public identifier for your virtual world. It appears in viewer login screens and region information.</p>
                        
                        <h5>Best Practices:</h5>
                        <ul>
                            <li><strong>Keep it descriptive:</strong> Choose a name that reflects your world's purpose</li>
                            <li><strong>Avoid special characters:</strong> Use letters, numbers, and spaces only</li>
                            <li><strong>Consider branding:</strong> Make it memorable and professional</li>
                            <li><strong>Length limits:</strong> Keep under 50 characters for compatibility</li>
                        </ul>
                        
                        <h5>Examples:</h5>
                        <div class="code-example">
                            <strong>Good:</strong> "My Virtual World", "Education Grid", "Corporate Training"<br>
                            <strong>Avoid:</strong> "Grid#1!", "Test@Grid", ""
                        </div>
                        
                        <h5>Technical Notes:</h5>
                        <p>The grid name is used in:</p>
                        <ul>
                            <li>Viewer login interface</li>
                            <li>Grid info services</li>
                            <li>Region welcome messages</li>
                            <li>Hypergrid communication</li>
                        </ul>
                    `,
                    keywords: ['grid', 'name', 'identity', 'branding', 'configuration'],
                    links: [
                        { title: 'Grid Configuration Guide', url: '/docs/grid-setup.html' },
                        { title: 'Viewer Integration', url: '/docs/viewer-integration.html' }
                    ],
                    relatedTopics: ['grid-nick', 'welcome-message', 'hypergrid']
                },
                {
                    id: 'grid-nick',
                    title: 'Grid Nickname Setup',
                    summary: 'Short identifier for your grid used in technical contexts',
                    content: `
                        <h4>Grid Nickname Configuration</h4>
                        <p>The grid nickname is a short, technical identifier used internally and in URLs.</p>
                        
                        <h5>Requirements:</h5>
                        <ul>
                            <li><strong>Length:</strong> 3-20 characters maximum</li>
                            <li><strong>Format:</strong> Lowercase letters and numbers only</li>
                            <li><strong>No spaces:</strong> Use hyphens for separation if needed</li>
                            <li><strong>Unique:</strong> Should be unique across the metaverse</li>
                        </ul>
                        
                        <h5>Usage:</h5>
                        <p>Grid nickname appears in:</p>
                        <ul>
                            <li>Hypergrid addresses (nickname.example.com:8002)</li>
                            <li>Asset server URLs</li>
                            <li>Database table prefixes</li>
                            <li>Log file identification</li>
                        </ul>
                        
                        <div class="warning-box">
                            <strong>Warning:</strong> Changing the grid nickname after initial setup may require database updates and configuration changes.
                        </div>
                    `,
                    keywords: ['grid', 'nickname', 'identifier', 'hypergrid', 'url'],
                    links: [
                        { title: 'Hypergrid Setup', url: '/docs/hypergrid.html' },
                        { title: 'Database Configuration', url: '/docs/database.html' }
                    ],
                    relatedTopics: ['grid-name', 'hypergrid-setup', 'database-config']
                },
                {
                    id: 'welcome-message',
                    title: 'Welcome Message Customization',
                    summary: 'Customizing the message users see when they log into your world',
                    content: `
                        <h4>Welcome Message Configuration</h4>
                        <p>The welcome message greets users when they first log into your virtual world.</p>
                        
                        <h5>Message Guidelines:</h5>
                        <ul>
                            <li><strong>Be welcoming:</strong> Create a positive first impression</li>
                            <li><strong>Provide guidance:</strong> Include basic navigation tips</li>
                            <li><strong>Keep it concise:</strong> Aim for 2-3 sentences maximum</li>
                            <li><strong>Include contact info:</strong> How to get help or support</li>
                        </ul>
                        
                        <h5>Template Examples:</h5>
                        <div class="code-example">
                            <strong>Basic:</strong> "Welcome to [Grid Name]! Explore, create, and connect with others in our virtual world."<br><br>
                            <strong>Detailed:</strong> "Welcome to [Grid Name]! Use arrow keys to move, click objects to interact. Need help? Contact admin@example.com or visit our help area."<br><br>
                            <strong>Community:</strong> "Welcome to [Grid Name]! Join our community events every Friday at 7 PM SLT. Visit the Community Center for upcoming activities."
                        </div>
                        
                        <h5>Variables Available:</h5>
                        <ul>
                            <li><code>{FIRSTNAME}</code> - User's first name</li>
                            <li><code>{LASTNAME}</code> - User's last name</li>
                            <li><code>{GRIDNAME}</code> - Your grid name</li>
                            <li><code>{REGION}</code> - Current region name</li>
                        </ul>
                    `,
                    keywords: ['welcome', 'message', 'greeting', 'users', 'first impression'],
                    links: [
                        { title: 'User Experience Guide', url: '/docs/user-experience.html' },
                        { title: 'Community Building', url: '/docs/community.html' }
                    ],
                    relatedTopics: ['user-management', 'community-features']
                }
            ]
        });

        this.helpDatabase.set('network', {
            category: 'Network Configuration',
            icon: 'icon-network',
            color: '#10b981',
            topics: [
                {
                    id: 'port-configuration',
                    title: 'Port Configuration Guide',
                    summary: 'Understanding and configuring network ports for optimal connectivity',
                    content: `
                        <h4>Network Port Configuration</h4>
                        <p>Proper port configuration is crucial for viewer connectivity and grid communication.</p>
                        
                        <h5>Standard Ports:</h5>
                        <table class="help-table">
                            <tr><th>Service</th><th>Default Port</th><th>Protocol</th><th>Purpose</th></tr>
                            <tr><td>HTTP</td><td>9000</td><td>TCP</td><td>Viewer login and web services</td></tr>
                            <tr><td>HTTPS</td><td>9001</td><td>TCP</td><td>Secure web services</td></tr>
                            <tr><td>Region</td><td>9000+</td><td>UDP</td><td>Viewer-region communication</td></tr>
                            <tr><td>WebSocket</td><td>9001</td><td>TCP</td><td>Web client real-time communication</td></tr>
                        </table>
                        
                        <h5>Firewall Configuration:</h5>
                        <p>Ensure these ports are open in your firewall:</p>
                        <div class="code-example">
                            # Ubuntu/Debian firewall rules<br>
                            sudo ufw allow 9000:9010/tcp<br>
                            sudo ufw allow 9000:9010/udp<br>
                            sudo ufw allow 8002/tcp  # Hypergrid (optional)
                        </div>
                        
                        <h5>Port Conflicts:</h5>
                        <p>Common port conflicts and solutions:</p>
                        <ul>
                            <li><strong>Port 9000 in use:</strong> Change to 9100-9200 range</li>
                            <li><strong>Corporate firewalls:</strong> Use ports 80/443 with reverse proxy</li>
                            <li><strong>Home routers:</strong> Configure port forwarding for external access</li>
                        </ul>
                        
                        <div class="tip-box">
                            <strong>Tip:</strong> Use <code>netstat -tulpn</code> to check which ports are currently in use on your system.
                        </div>
                    `,
                    keywords: ['ports', 'network', 'firewall', 'connectivity', 'tcp', 'udp'],
                    links: [
                        { title: 'Network Security Guide', url: '/docs/network-security.html' },
                        { title: 'Firewall Configuration', url: '/docs/firewall.html' }
                    ],
                    relatedTopics: ['https-setup', 'hypergrid-networking', 'load-balancing']
                },
                {
                    id: 'https-configuration',
                    title: 'HTTPS/SSL Setup',
                    summary: 'Securing your virtual world with SSL/TLS encryption',
                    content: `
                        <h4>HTTPS/SSL Configuration</h4>
                        <p>SSL encryption protects user data and provides secure authentication for your virtual world.</p>
                        
                        <h5>Certificate Options:</h5>
                        <ul>
                            <li><strong>Let's Encrypt:</strong> Free automated certificates (recommended)</li>
                            <li><strong>Commercial CA:</strong> Purchased certificates from trusted authorities</li>
                            <li><strong>Self-signed:</strong> For development/testing only</li>
                        </ul>
                        
                        <h5>Let's Encrypt Setup:</h5>
                        <div class="code-example">
                            # Install certbot<br>
                            sudo apt install certbot<br><br>
                            # Generate certificate<br>
                            sudo certbot certonly --standalone -d yourgrid.example.com<br><br>
                            # Certificate files will be in:<br>
                            /etc/letsencrypt/live/yourgrid.example.com/
                        </div>
                        
                        <h5>Configuration Files:</h5>
                        <p>Update your OpenSim configuration:</p>
                        <div class="code-example">
                            [Network]<br>
                            https_listener_port = 9001<br>
                            ssl_certificate = /path/to/fullchain.pem<br>
                            ssl_private_key = /path/to/privkey.pem
                        </div>
                        
                        <h5>Automatic Renewal:</h5>
                        <p>Set up automatic certificate renewal:</p>
                        <div class="code-example">
                            # Add to crontab (crontab -e)<br>
                            0 12 * * * /usr/bin/certbot renew --quiet
                        </div>
                        
                        <div class="warning-box">
                            <strong>Important:</strong> Always backup your certificates and test renewal process before going live.
                        </div>
                    `,
                    keywords: ['https', 'ssl', 'tls', 'certificates', 'security', 'encryption'],
                    links: [
                        { title: 'Let\'s Encrypt Guide', url: 'https://letsencrypt.org/getting-started/' },
                        { title: 'SSL Best Practices', url: '/docs/ssl-security.html' }
                    ],
                    relatedTopics: ['network-security', 'certificate-management', 'production-deployment']
                }
            ]
        });

        this.helpDatabase.set('database', {
            category: 'Database Configuration',
            icon: 'icon-database',
            color: '#8b5cf6',
            topics: [
                {
                    id: 'database-selection',
                    title: 'Database Engine Selection',
                    summary: 'Choosing the right database engine for your virtual world',
                    content: `
                        <h4>Database Engine Selection Guide</h4>
                        <p>The database engine affects performance, scalability, and maintenance requirements.</p>
                        
                        <h5>Database Options:</h5>
                        <table class="help-table">
                            <tr><th>Engine</th><th>Best For</th><th>Pros</th><th>Cons</th></tr>
                            <tr>
                                <td><strong>SQLite</strong></td>
                                <td>Development, Small grids</td>
                                <td>No setup, File-based, Fast</td>
                                <td>Single-user, Limited scalability</td>
                            </tr>
                            <tr>
                                <td><strong>PostgreSQL</strong></td>
                                <td>Production, Large grids</td>
                                <td>ACID compliant, Scalable, JSON support</td>
                                <td>Setup complexity, Resource usage</td>
                            </tr>
                            <tr>
                                <td><strong>MySQL</strong></td>
                                <td>Medium grids, Shared hosting</td>
                                <td>Widely supported, Good performance</td>
                                <td>Configuration sensitive, Replication complexity</td>
                            </tr>
                        </table>
                        
                        <h5>Selection Criteria:</h5>
                        <ul>
                            <li><strong>Concurrent Users:</strong>
                                <ul>
                                    <li>1-10 users: SQLite</li>
                                    <li>10-100 users: MySQL/PostgreSQL</li>
                                    <li>100+ users: PostgreSQL with optimization</li>
                                </ul>
                            </li>
                            <li><strong>Technical Expertise:</strong>
                                <ul>
                                    <li>Beginner: SQLite</li>
                                    <li>Intermediate: MySQL</li>
                                    <li>Advanced: PostgreSQL</li>
                                </ul>
                            </li>
                            <li><strong>Infrastructure:</strong>
                                <ul>
                                    <li>Single server: SQLite/MySQL</li>
                                    <li>Dedicated database server: PostgreSQL</li>
                                    <li>Cloud hosting: Managed database services</li>
                                </ul>
                            </li>
                        </ul>
                        
                        <h5>Migration Path:</h5>
                        <p>You can migrate between databases as your grid grows:</p>
                        <div class="migration-path">
                            SQLite → MySQL → PostgreSQL
                        </div>
                        
                        <div class="tip-box">
                            <strong>Tip:</strong> Start with SQLite for development, then migrate to PostgreSQL for production deployment.
                        </div>
                    `,
                    keywords: ['database', 'selection', 'sqlite', 'postgresql', 'mysql', 'scalability'],
                    links: [
                        { title: 'Database Setup Guide', url: '/docs/database-setup.html' },
                        { title: 'Migration Tools', url: '/docs/database-migration.html' }
                    ],
                    relatedTopics: ['database-optimization', 'backup-strategy', 'performance-tuning']
                },
                {
                    id: 'connection-optimization',
                    title: 'Database Connection Optimization',
                    summary: 'Optimizing database connections for maximum performance',
                    content: `
                        <h4>Database Connection Optimization</h4>
                        <p>Proper connection management is crucial for virtual world performance and stability.</p>
                        
                        <h5>Connection Pool Settings:</h5>
                        <table class="help-table">
                            <tr><th>Setting</th><th>Small Grid</th><th>Medium Grid</th><th>Large Grid</th></tr>
                            <tr><td>Min Pool Size</td><td>2</td><td>5</td><td>10</td></tr>
                            <tr><td>Max Pool Size</td><td>10</td><td>25</td><td>50</td></tr>
                            <tr><td>Connection Timeout</td><td>30s</td><td>30s</td><td>60s</td></tr>
                            <tr><td>Idle Timeout</td><td>300s</td><td>600s</td><td>900s</td></tr>
                        </table>
                        
                        <h5>PostgreSQL Optimization:</h5>
                        <div class="code-example">
                            # postgresql.conf settings<br>
                            max_connections = 100<br>
                            shared_buffers = 256MB<br>
                            effective_cache_size = 1GB<br>
                            maintenance_work_mem = 64MB<br>
                            checkpoint_completion_target = 0.9<br>
                            wal_buffers = 16MB
                        </div>
                        
                        <h5>MySQL Optimization:</h5>
                        <div class="code-example">
                            # my.cnf settings<br>
                            innodb_buffer_pool_size = 512MB<br>
                            innodb_log_file_size = 128MB<br>
                            max_connections = 100<br>
                            query_cache_size = 64MB<br>
                            tmp_table_size = 64MB
                        </div>
                        
                        <h5>Monitoring Tools:</h5>
                        <ul>
                            <li><strong>Connection monitoring:</strong> Watch for connection leaks</li>
                            <li><strong>Query performance:</strong> Identify slow queries</li>
                            <li><strong>Resource usage:</strong> Monitor CPU and memory</li>
                            <li><strong>Lock monitoring:</strong> Check for deadlocks and contention</li>
                        </ul>
                        
                        <div class="warning-box">
                            <strong>Warning:</strong> Too many connections can overwhelm the database. Monitor and adjust based on actual usage.
                        </div>
                    `,
                    keywords: ['database', 'connections', 'pool', 'optimization', 'performance'],
                    links: [
                        { title: 'PostgreSQL Performance', url: '/docs/postgresql-performance.html' },
                        { title: 'MySQL Tuning', url: '/docs/mysql-tuning.html' }
                    ],
                    relatedTopics: ['performance-monitoring', 'resource-management', 'troubleshooting']
                }
            ]
        });

        this.helpDatabase.set('security', {
            category: 'Security & Encryption',
            icon: 'icon-shield',
            color: '#ef4444',
            topics: [
                {
                    id: 'api-key-management',
                    title: 'API Key Security',
                    summary: 'Managing API keys and authentication tokens securely',
                    content: `
                        <h4>API Key Security Best Practices</h4>
                        <p>API keys are the primary security mechanism for OpenSim Next administration and inter-service communication.</p>
                        
                        <h5>Key Generation:</h5>
                        <p>Generate strong, cryptographically random API keys:</p>
                        <div class="code-example">
                            # Generate secure API key (32 bytes, base64 encoded)<br>
                            openssl rand -base64 32<br><br>
                            # Alternative with Python<br>
                            python3 -c "import secrets; print(secrets.token_urlsafe(32))"
                        </div>
                        
                        <h5>Key Storage:</h5>
                        <ul>
                            <li><strong>Environment variables:</strong> Store in OPENSIM_API_KEY</li>
                            <li><strong>Configuration files:</strong> Use restricted file permissions (600)</li>
                            <li><strong>Key management services:</strong> AWS KMS, Azure Key Vault, HashiCorp Vault</li>
                            <li><strong>USB key storage:</strong> Use the built-in encrypted USB key manager</li>
                        </ul>
                        
                        <h5>Key Rotation:</h5>
                        <p>Regular key rotation schedule:</p>
                        <ul>
                            <li><strong>Development:</strong> Monthly rotation</li>
                            <li><strong>Production:</strong> Quarterly rotation</li>
                            <li><strong>Compromised keys:</strong> Immediate rotation</li>
                            <li><strong>Staff changes:</strong> Rotate when admin staff leaves</li>
                        </ul>
                        
                        <h5>Access Control:</h5>
                        <table class="help-table">
                            <tr><th>Role</th><th>API Access</th><th>Key Type</th></tr>
                            <tr><td>Super Admin</td><td>Full system control</td><td>Master key</td></tr>
                            <tr><td>Grid Admin</td><td>Grid management</td><td>Admin key</td></tr>
                            <tr><td>Region Manager</td><td>Region operations</td><td>Regional key</td></tr>
                            <tr><td>Monitoring</td><td>Read-only metrics</td><td>Metrics key</td></tr>
                        </table>
                        
                        <div class="warning-box">
                            <strong>Critical:</strong> Never commit API keys to version control. Use environment variables or encrypted storage.
                        </div>
                    `,
                    keywords: ['api', 'keys', 'authentication', 'security', 'tokens', 'access control'],
                    links: [
                        { title: 'Security Hardening Guide', url: '/docs/security-hardening.html' },
                        { title: 'Key Management', url: '/docs/key-management.html' }
                    ],
                    relatedTopics: ['encryption-setup', 'usb-key-storage', 'access-control']
                }
            ]
        });

        this.helpDatabase.set('physics', {
            category: 'Physics Engines',
            icon: 'icon-activity',
            color: '#f59e0b',
            topics: [
                {
                    id: 'physics-engine-selection',
                    title: 'Physics Engine Selection Guide',
                    summary: 'Choosing the optimal physics engine for your virtual world content',
                    content: `
                        <h4>Multi-Physics Engine System</h4>
                        <p>OpenSim Next supports 5 physics engines, each optimized for different use cases. You can assign different engines to different regions.</p>
                        
                        <h5>Physics Engine Comparison:</h5>
                        <table class="help-table">
                            <tr><th>Engine</th><th>Max Bodies</th><th>Best For</th><th>Features</th></tr>
                            <tr>
                                <td><strong>ODE</strong></td>
                                <td>10,000</td>
                                <td>Avatars, Traditional content</td>
                                <td>Stable, Battle-tested, Avatar-optimized</td>
                            </tr>
                            <tr>
                                <td><strong>UBODE</strong></td>
                                <td>20,000</td>
                                <td>Large worlds, Many objects</td>
                                <td>Enhanced ODE, Better performance</td>
                            </tr>
                            <tr>
                                <td><strong>Bullet</strong></td>
                                <td>50,000</td>
                                <td>Vehicles, Advanced physics</td>
                                <td>Soft bodies, Advanced constraints</td>
                            </tr>
                            <tr>
                                <td><strong>POS</strong></td>
                                <td>100,000</td>
                                <td>Particles, Fluids, Cloth</td>
                                <td>Position-based dynamics, GPU acceleration</td>
                            </tr>
                            <tr>
                                <td><strong>Basic</strong></td>
                                <td>1,000</td>
                                <td>Testing, Lightweight scenarios</td>
                                <td>Minimal resource usage</td>
                            </tr>
                        </table>
                        
                        <h5>Use Case Recommendations:</h5>
                        <ul>
                            <li><strong>Social regions (avatars, chat):</strong> ODE - Most stable for avatar movement</li>
                            <li><strong>Building/sandbox regions:</strong> UBODE - Handles many objects well</li>
                            <li><strong>Vehicle testing areas:</strong> Bullet - Advanced vehicle physics</li>
                            <li><strong>Particle effects/art:</strong> POS - Fluid and particle simulation</li>
                            <li><strong>Development/testing:</strong> Basic - Minimal overhead</li>
                        </ul>
                        
                        <h5>Runtime Engine Switching:</h5>
                        <p>You can change physics engines without restarting:</p>
                        <div class="code-example">
                            # Via admin console<br>
                            change region physics Bullet<br><br>
                            # Via HTTP API<br>
                            curl -X POST -H "X-API-Key: your-key" \\<br>
                            &nbsp;&nbsp;http://localhost:8090/api/regions/myregion/physics \\<br>
                            &nbsp;&nbsp;-d '{"engine_type": "Bullet", "config": "for_vehicles"}'
                        </div>
                        
                        <h5>Performance Monitoring:</h5>
                        <p>Monitor physics performance:</p>
                        <ul>
                            <li><strong>Physics FPS:</strong> Should maintain 60+ FPS</li>
                            <li><strong>Body count:</strong> Monitor active rigid bodies</li>
                            <li><strong>Collision detection time:</strong> Watch for spikes</li>
                            <li><strong>Memory usage:</strong> Physics engine memory consumption</li>
                        </ul>
                    `,
                    keywords: ['physics', 'engines', 'ode', 'bullet', 'ubode', 'pos', 'performance'],
                    links: [
                        { title: 'Physics Configuration', url: '/docs/physics-config.html' },
                        { title: 'Performance Tuning', url: '/docs/physics-performance.html' }
                    ],
                    relatedTopics: ['performance-optimization', 'region-management', 'content-creation']
                }
            ]
        });

        this.helpDatabase.set('troubleshooting', {
            category: 'Troubleshooting',
            icon: 'icon-tool',
            color: '#6366f1',
            topics: [
                {
                    id: 'common-issues',
                    title: 'Common Issues & Solutions',
                    summary: 'Quick solutions for frequently encountered problems',
                    content: `
                        <h4>Common Configuration Issues</h4>
                        <p>Quick solutions for the most frequent problems encountered during setup.</p>
                        
                        <h5>Connection Issues:</h5>
                        <div class="issue-solution">
                            <div class="issue"><strong>Problem:</strong> Viewer can't connect to grid</div>
                            <div class="solution">
                                <strong>Solutions:</strong>
                                <ol>
                                    <li>Check firewall settings - ensure ports 9000-9010 are open</li>
                                    <li>Verify external hostname is correct and resolves</li>
                                    <li>Test with local IP address first (192.168.x.x)</li>
                                    <li>Check router port forwarding configuration</li>
                                </ol>
                            </div>
                        </div>
                        
                        <div class="issue-solution">
                            <div class="issue"><strong>Problem:</strong> Database connection failed</div>
                            <div class="solution">
                                <strong>Solutions:</strong>
                                <ol>
                                    <li>Verify database service is running</li>
                                    <li>Check username/password credentials</li>
                                    <li>Ensure database exists and is accessible</li>
                                    <li>Test connection with database client tools</li>
                                </ol>
                            </div>
                        </div>
                        
                        <h5>Performance Issues:</h5>
                        <div class="issue-solution">
                            <div class="issue"><strong>Problem:</strong> High server CPU usage</div>
                            <div class="solution">
                                <strong>Solutions:</strong>
                                <ol>
                                    <li>Check physics engine settings - reduce max bodies if needed</li>
                                    <li>Monitor database query performance</li>
                                    <li>Review script performance and disable problem scripts</li>
                                    <li>Consider using lighter physics engine (Basic or ODE)</li>
                                </ol>
                            </div>
                        </div>
                        
                        <h5>SSL/HTTPS Issues:</h5>
                        <div class="issue-solution">
                            <div class="issue"><strong>Problem:</strong> SSL certificate errors</div>
                            <div class="solution">
                                <strong>Solutions:</strong>
                                <ol>
                                    <li>Verify certificate file paths are correct</li>
                                    <li>Check certificate is not expired</li>
                                    <li>Ensure private key matches certificate</li>
                                    <li>Test certificate with: <code>openssl x509 -in cert.pem -text -noout</code></li>
                                </ol>
                            </div>
                        </div>
                        
                        <h5>Diagnostic Commands:</h5>
                        <div class="code-example">
                            # Check port availability<br>
                            netstat -tulpn | grep :9000<br><br>
                            # Test database connection<br>
                            psql -h localhost -U opensim -d opensim_db<br><br>
                            # Check SSL certificate<br>
                            openssl s509 -in /path/to/cert.pem -text -noout<br><br>
                            # Monitor system resources<br>
                            htop
                        </div>
                        
                        <div class="tip-box">
                            <strong>Tip:</strong> Enable debug logging to get more detailed error information: Set RUST_LOG=debug
                        </div>
                    `,
                    keywords: ['troubleshooting', 'issues', 'problems', 'solutions', 'debugging'],
                    links: [
                        { title: 'Debug Guide', url: '/docs/debugging.html' },
                        { title: 'Log Analysis', url: '/docs/log-analysis.html' }
                    ],
                    relatedTopics: ['performance-monitoring', 'error-handling', 'diagnostics']
                }
            ]
        });
    }

    buildSearchIndex() {
        console.log('Building help search index...');
        this.searchIndex.clear();
        
        for (const [category, categoryData] of this.helpDatabase.entries()) {
            for (const topic of categoryData.topics) {
                // Index by keywords
                for (const keyword of topic.keywords) {
                    if (!this.searchIndex.has(keyword)) {
                        this.searchIndex.set(keyword, []);
                    }
                    this.searchIndex.get(keyword).push({
                        category,
                        topicId: topic.id,
                        relevance: 1.0
                    });
                }
                
                // Index by title words
                const titleWords = topic.title.toLowerCase().split(/\s+/);
                for (const word of titleWords) {
                    if (word.length > 2) {
                        if (!this.searchIndex.has(word)) {
                            this.searchIndex.set(word, []);
                        }
                        this.searchIndex.get(word).push({
                            category,
                            topicId: topic.id,
                            relevance: 0.8
                        });
                    }
                }
                
                // Index by content words (limited to avoid noise)
                const contentWords = topic.content.replace(/<[^>]*>/g, ' ')
                    .toLowerCase().split(/\s+/).slice(0, 50);
                for (const word of contentWords) {
                    if (word.length > 3) {
                        if (!this.searchIndex.has(word)) {
                            this.searchIndex.set(word, []);
                        }
                        this.searchIndex.get(word).push({
                            category,
                            topicId: topic.id,
                            relevance: 0.3
                        });
                    }
                }
            }
        }
        
        console.log(`✅ Search index built with ${this.searchIndex.size} terms`);
    }

    setupContextualHelp() {
        // Map UI elements to help topics
        this.contextualHelp.set('#grid-name', {
            topicId: 'grid-name',
            category: 'general',
            trigger: 'focus'
        });
        
        this.contextualHelp.set('#grid-nick', {
            topicId: 'grid-nick',
            category: 'general',
            trigger: 'focus'
        });
        
        this.contextualHelp.set('#welcome-message', {
            topicId: 'welcome-message',
            category: 'general',
            trigger: 'focus'
        });
        
        this.contextualHelp.set('#http-port', {
            topicId: 'port-configuration',
            category: 'network',
            trigger: 'focus'
        });
        
        this.contextualHelp.set('#https-enabled', {
            topicId: 'https-configuration',
            category: 'network',
            trigger: 'change'
        });
        
        this.contextualHelp.set('#database-type', {
            topicId: 'database-selection',
            category: 'database',
            trigger: 'change'
        });
        
        this.contextualHelp.set('#api-key', {
            topicId: 'api-key-management',
            category: 'security',
            trigger: 'focus'
        });
        
        this.contextualHelp.set('#physics-engine', {
            topicId: 'physics-engine-selection',
            category: 'physics',
            trigger: 'change'
        });
    }

    createHelpInterface() {
        // Create help system UI
        const helpContainer = document.createElement('div');
        helpContainer.id = 'help-system';
        helpContainer.innerHTML = `
            <!-- Help Button -->
            <button class="help-toggle-btn" id="help-toggle" title="Help & Documentation">
                <i class="icon-help"></i>
                <span class="help-badge" id="help-badge" style="display: none;">!</span>
            </button>

            <!-- Help Panel -->
            <div class="help-panel" id="help-panel">
                <div class="help-header">
                    <div class="help-title">
                        <i class="icon-help"></i>
                        <h3>Help & Documentation</h3>
                    </div>
                    <div class="help-controls">
                        <button class="btn btn-sm btn-secondary" id="help-settings">
                            <i class="icon-settings"></i>
                        </button>
                        <button class="btn btn-sm btn-secondary" id="help-close">
                            <i class="icon-x"></i>
                        </button>
                    </div>
                </div>

                <div class="help-content">
                    <!-- Search -->
                    <div class="help-search">
                        <div class="search-input-wrapper">
                            <input type="text" id="help-search" placeholder="Search help topics...">
                            <i class="icon-search"></i>
                        </div>
                        <div class="search-suggestions" id="search-suggestions"></div>
                    </div>

                    <!-- Navigation Tabs -->
                    <div class="help-navigation">
                        <button class="help-nav-btn active" data-view="browse">
                            <i class="icon-book"></i>
                            Browse
                        </button>
                        <button class="help-nav-btn" data-view="context">
                            <i class="icon-target"></i>
                            Context
                        </button>
                        <button class="help-nav-btn" data-view="search">
                            <i class="icon-search"></i>
                            Search
                        </button>
                        <button class="help-nav-btn" data-view="history">
                            <i class="icon-clock"></i>
                            History
                        </button>
                    </div>

                    <!-- Content Views -->
                    <div class="help-views">
                        <!-- Browse View -->
                        <div class="help-view active" id="browse-view">
                            <div class="help-categories" id="help-categories">
                                <!-- Categories will be populated by JavaScript -->
                            </div>
                        </div>

                        <!-- Context View -->
                        <div class="help-view" id="context-view">
                            <div class="context-help-content">
                                <div class="no-context">
                                    <i class="icon-info"></i>
                                    <p>Click on any form field to see contextual help</p>
                                </div>
                            </div>
                        </div>

                        <!-- Search View -->
                        <div class="help-view" id="search-view">
                            <div class="search-results" id="search-results">
                                <div class="no-results">
                                    <i class="icon-search"></i>
                                    <p>Enter a search term to find help topics</p>
                                </div>
                            </div>
                        </div>

                        <!-- History View -->
                        <div class="help-view" id="history-view">
                            <div class="help-history" id="help-history">
                                <div class="no-history">
                                    <i class="icon-clock"></i>
                                    <p>No help topics viewed yet</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Help Modal -->
            <div class="help-modal-overlay" id="help-modal-overlay">
                <div class="help-modal">
                    <div class="help-modal-header">
                        <div class="help-modal-title">
                            <i class="icon-book"></i>
                            <h4 id="help-modal-title">Help Topic</h4>
                        </div>
                        <div class="help-modal-controls">
                            <button class="btn btn-sm btn-secondary" id="help-modal-back">
                                <i class="icon-arrow-left"></i>
                                Back
                            </button>
                            <button class="btn btn-sm btn-secondary" id="help-modal-print">
                                <i class="icon-printer"></i>
                                Print
                            </button>
                            <button class="btn btn-sm btn-secondary" id="help-modal-close">
                                <i class="icon-x"></i>
                            </button>
                        </div>
                    </div>
                    <div class="help-modal-content" id="help-modal-content">
                        <!-- Topic content will be populated here -->
                    </div>
                    <div class="help-modal-footer">
                        <div class="help-links" id="help-modal-links">
                            <!-- Related links will be populated here -->
                        </div>
                        <div class="help-actions">
                            <button class="btn btn-sm btn-secondary" id="help-feedback">
                                <i class="icon-message-circle"></i>
                                Feedback
                            </button>
                            <button class="btn btn-sm btn-primary" id="help-apply">
                                <i class="icon-check"></i>
                                Apply Solution
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Contextual Help Tooltip -->
            <div class="help-tooltip" id="help-tooltip">
                <div class="tooltip-content">
                    <div class="tooltip-title"></div>
                    <div class="tooltip-summary"></div>
                    <div class="tooltip-actions">
                        <button class="btn btn-xs btn-primary" id="tooltip-learn-more">
                            Learn More
                        </button>
                    </div>
                </div>
                <div class="tooltip-arrow"></div>
            </div>
        `;

        document.body.appendChild(helpContainer);
        this.populateCategories();
    }

    populateCategories() {
        const categoriesContainer = document.getElementById('help-categories');
        if (!categoriesContainer) return;

        let html = '';
        for (const [categoryId, categoryData] of this.helpDatabase.entries()) {
            html += `
                <div class="help-category" data-category="${categoryId}">
                    <div class="category-header">
                        <div class="category-icon" style="background-color: ${categoryData.color}">
                            <i class="${categoryData.icon}"></i>
                        </div>
                        <div class="category-info">
                            <h4>${categoryData.category}</h4>
                            <span class="topic-count">${categoryData.topics.length} topics</span>
                        </div>
                        <div class="category-toggle">
                            <i class="icon-chevron-down"></i>
                        </div>
                    </div>
                    <div class="category-topics">
                        ${categoryData.topics.map(topic => `
                            <div class="help-topic" data-topic="${topic.id}" data-category="${categoryId}">
                                <div class="topic-title">${topic.title}</div>
                                <div class="topic-summary">${topic.summary}</div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }

        categoriesContainer.innerHTML = html;
    }

    setupEventListeners() {
        // Help panel toggle
        document.getElementById('help-toggle')?.addEventListener('click', () => {
            this.toggleHelpPanel();
        });

        // Panel close
        document.getElementById('help-close')?.addEventListener('click', () => {
            this.hideHelpPanel();
        });

        // Navigation
        document.querySelectorAll('.help-nav-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const view = e.target.closest('button').dataset.view;
                this.switchView(view);
            });
        });

        // Search
        const searchInput = document.getElementById('help-search');
        if (searchInput) {
            let searchTimeout;
            searchInput.addEventListener('input', (e) => {
                clearTimeout(searchTimeout);
                searchTimeout = setTimeout(() => {
                    this.performSearch(e.target.value);
                }, this.settings.searchDelay);
            });
        }

        // Category expansion
        document.addEventListener('click', (e) => {
            if (e.target.closest('.category-header')) {
                const category = e.target.closest('.help-category');
                category.classList.toggle('expanded');
            }
        });

        // Topic selection
        document.addEventListener('click', (e) => {
            if (e.target.closest('.help-topic')) {
                const topicElement = e.target.closest('.help-topic');
                const topicId = topicElement.dataset.topic;
                const category = topicElement.dataset.category;
                this.showHelpTopic(category, topicId);
            }
        });

        // Modal controls
        document.getElementById('help-modal-close')?.addEventListener('click', () => {
            this.hideHelpModal();
        });

        document.getElementById('help-modal-overlay')?.addEventListener('click', (e) => {
            if (e.target.id === 'help-modal-overlay') {
                this.hideHelpModal();
            }
        });

        // Contextual help
        this.setupContextualEventListeners();

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'F1' || (e.ctrlKey && e.key === '/')) {
                e.preventDefault();
                this.toggleHelpPanel();
            }
            if (e.key === 'Escape') {
                this.hideHelpModal();
                this.hideHelpPanel();
            }
        });
    }

    setupContextualEventListeners() {
        // Add event listeners for contextual help
        for (const [selector, helpConfig] of this.contextualHelp.entries()) {
            const elements = document.querySelectorAll(selector);
            elements.forEach(element => {
                element.addEventListener(helpConfig.trigger, (e) => {
                    this.showContextualHelp(e.target, helpConfig);
                });

                // Hide contextual help on blur
                if (helpConfig.trigger === 'focus') {
                    element.addEventListener('blur', () => {
                        this.hideContextualHelp();
                    });
                }
            });
        }
    }

    toggleHelpPanel() {
        const panel = document.getElementById('help-panel');
        if (panel.classList.contains('active')) {
            this.hideHelpPanel();
        } else {
            this.showHelpPanel();
        }
    }

    showHelpPanel() {
        const panel = document.getElementById('help-panel');
        panel.classList.add('active');
        
        // Focus search input
        const searchInput = document.getElementById('help-search');
        if (searchInput) {
            setTimeout(() => searchInput.focus(), 100);
        }
    }

    hideHelpPanel() {
        const panel = document.getElementById('help-panel');
        panel.classList.remove('active');
    }

    switchView(viewName) {
        // Update navigation
        document.querySelectorAll('.help-nav-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.querySelector(`[data-view="${viewName}"]`).classList.add('active');

        // Update views
        document.querySelectorAll('.help-view').forEach(view => {
            view.classList.remove('active');
        });
        document.getElementById(`${viewName}-view`).classList.add('active');

        // Handle view-specific logic
        if (viewName === 'search') {
            document.getElementById('help-search').focus();
        } else if (viewName === 'history') {
            this.updateHistoryView();
        } else if (viewName === 'context') {
            this.updateContextView();
        }
    }

    performSearch(query) {
        if (!query || query.length < 2) {
            this.clearSearchResults();
            return;
        }

        const results = this.searchHelpTopics(query);
        this.displaySearchResults(results);
        
        // Auto-switch to search view if not already there
        if (!document.getElementById('search-view').classList.contains('active')) {
            this.switchView('search');
        }
    }

    searchHelpTopics(query) {
        const terms = query.toLowerCase().split(/\s+/);
        const results = new Map();

        for (const term of terms) {
            // Exact matches
            if (this.searchIndex.has(term)) {
                for (const result of this.searchIndex.get(term)) {
                    const key = `${result.category}:${result.topicId}`;
                    if (!results.has(key)) {
                        results.set(key, { ...result, score: 0 });
                    }
                    results.get(key).score += result.relevance;
                }
            }

            // Partial matches
            for (const [indexTerm, indexResults] of this.searchIndex.entries()) {
                if (indexTerm.includes(term) && indexTerm !== term) {
                    for (const result of indexResults) {
                        const key = `${result.category}:${result.topicId}`;
                        if (!results.has(key)) {
                            results.set(key, { ...result, score: 0 });
                        }
                        results.get(key).score += result.relevance * 0.5; // Partial match penalty
                    }
                }
            }
        }

        // Sort by score and return
        return Array.from(results.values())
            .sort((a, b) => b.score - a.score)
            .slice(0, 10); // Limit to top 10 results
    }

    displaySearchResults(results) {
        const container = document.getElementById('search-results');
        if (!container) return;

        if (results.length === 0) {
            container.innerHTML = `
                <div class="no-results">
                    <i class="icon-search"></i>
                    <p>No help topics found</p>
                </div>
            `;
            return;
        }

        let html = '';
        for (const result of results) {
            const topic = this.getHelpTopic(result.category, result.topicId);
            const category = this.helpDatabase.get(result.category);
            
            if (topic && category) {
                html += `
                    <div class="search-result" data-topic="${result.topicId}" data-category="${result.category}">
                        <div class="result-header">
                            <div class="result-category" style="color: ${category.color}">
                                <i class="${category.icon}"></i>
                                ${category.category}
                            </div>
                            <div class="result-score">${Math.round(result.score * 100)}%</div>
                        </div>
                        <div class="result-title">${topic.title}</div>
                        <div class="result-summary">${topic.summary}</div>
                        <div class="result-keywords">
                            ${topic.keywords.slice(0, 5).map(k => `<span class="keyword">${k}</span>`).join('')}
                        </div>
                    </div>
                `;
            }
        }

        container.innerHTML = html;
    }

    clearSearchResults() {
        const container = document.getElementById('search-results');
        if (container) {
            container.innerHTML = `
                <div class="no-results">
                    <i class="icon-search"></i>
                    <p>Enter a search term to find help topics</p>
                </div>
            `;
        }
    }

    showHelpTopic(category, topicId) {
        const topic = this.getHelpTopic(category, topicId);
        const categoryData = this.helpDatabase.get(category);
        
        if (!topic || !categoryData) {
            console.error('Help topic not found:', category, topicId);
            return;
        }

        // Add to history
        this.addToHistory(category, topicId);

        // Show modal
        this.showHelpModal(topic, categoryData);
    }

    showHelpModal(topic, categoryData) {
        const modal = document.getElementById('help-modal-overlay');
        const title = document.getElementById('help-modal-title');
        const content = document.getElementById('help-modal-content');
        const links = document.getElementById('help-modal-links');

        // Set title
        title.textContent = topic.title;

        // Set content
        content.innerHTML = topic.content;

        // Set links
        let linksHtml = '';
        if (topic.links && topic.links.length > 0) {
            linksHtml += '<h6>Related Links:</h6><div class="related-links">';
            for (const link of topic.links) {
                linksHtml += `<a href="${link.url}" target="_blank" class="related-link">
                    <i class="icon-external-link"></i>
                    ${link.title}
                </a>`;
            }
            linksHtml += '</div>';
        }

        if (topic.relatedTopics && topic.relatedTopics.length > 0) {
            linksHtml += '<h6>Related Topics:</h6><div class="related-topics">';
            for (const relatedId of topic.relatedTopics) {
                const relatedTopic = this.findTopicById(relatedId);
                if (relatedTopic) {
                    linksHtml += `<button class="related-topic-btn" 
                        data-category="${relatedTopic.category}" 
                        data-topic="${relatedTopic.topic.id}">
                        <i class="icon-arrow-right"></i>
                        ${relatedTopic.topic.title}
                    </button>`;
                }
            }
            linksHtml += '</div>';
        }

        links.innerHTML = linksHtml;

        // Show modal
        modal.classList.add('active');

        // Setup related topic navigation
        links.querySelectorAll('.related-topic-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const category = e.target.dataset.category;
                const topicId = e.target.dataset.topic;
                this.showHelpTopic(category, topicId);
            });
        });
    }

    hideHelpModal() {
        const modal = document.getElementById('help-modal-overlay');
        modal.classList.remove('active');
    }

    showContextualHelp(element, helpConfig) {
        if (!this.settings.enableContextualHelp) return;

        const topic = this.getHelpTopic(helpConfig.category, helpConfig.topicId);
        if (!topic) return;

        this.currentContext = { element, helpConfig, topic };
        this.updateContextView();
        
        // Show tooltip if enabled
        if (this.settings.enableTooltips) {
            this.showHelpTooltip(element, topic);
        }
    }

    showHelpTooltip(element, topic) {
        const tooltip = document.getElementById('help-tooltip');
        const title = tooltip.querySelector('.tooltip-title');
        const summary = tooltip.querySelector('.tooltip-summary');

        title.textContent = topic.title;
        summary.textContent = topic.summary;

        // Position tooltip
        const rect = element.getBoundingClientRect();
        tooltip.style.left = rect.left + (rect.width / 2) + 'px';
        tooltip.style.top = (rect.bottom + 10) + 'px';

        tooltip.classList.add('active');

        // Auto-hide after delay
        setTimeout(() => {
            tooltip.classList.remove('active');
        }, 5000);
    }

    hideContextualHelp() {
        const tooltip = document.getElementById('help-tooltip');
        tooltip.classList.remove('active');
    }

    updateContextView() {
        const container = document.querySelector('#context-view .context-help-content');
        if (!container) return;

        if (!this.currentContext) {
            container.innerHTML = `
                <div class="no-context">
                    <i class="icon-info"></i>
                    <p>Click on any form field to see contextual help</p>
                </div>
            `;
            return;
        }

        const { topic } = this.currentContext;
        container.innerHTML = `
            <div class="context-topic">
                <div class="context-header">
                    <h4>${topic.title}</h4>
                    <button class="btn btn-sm btn-primary" onclick="helpSystem.showHelpTopic('${this.currentContext.helpConfig.category}', '${topic.id}')">
                        <i class="icon-maximize"></i>
                        View Full Help
                    </button>
                </div>
                <div class="context-summary">${topic.summary}</div>
                <div class="context-content">${topic.content.substring(0, 500)}...</div>
            </div>
        `;
    }

    updateHistoryView() {
        const container = document.getElementById('help-history');
        if (!container) return;

        if (this.helpHistory.length === 0) {
            container.innerHTML = `
                <div class="no-history">
                    <i class="icon-clock"></i>
                    <p>No help topics viewed yet</p>
                </div>
            `;
            return;
        }

        let html = '';
        for (const historyItem of this.helpHistory.slice().reverse()) {
            const topic = this.getHelpTopic(historyItem.category, historyItem.topicId);
            const categoryData = this.helpDatabase.get(historyItem.category);
            
            if (topic && categoryData) {
                html += `
                    <div class="history-item" data-topic="${historyItem.topicId}" data-category="${historyItem.category}">
                        <div class="history-time">${new Date(historyItem.timestamp).toLocaleString()}</div>
                        <div class="history-topic">
                            <div class="topic-category" style="color: ${categoryData.color}">
                                <i class="${categoryData.icon}"></i>
                                ${categoryData.category}
                            </div>
                            <div class="topic-title">${topic.title}</div>
                        </div>
                    </div>
                `;
            }
        }

        container.innerHTML = html;
    }

    addToHistory(category, topicId) {
        // Remove existing entry if present
        this.helpHistory = this.helpHistory.filter(item => 
            !(item.category === category && item.topicId === topicId)
        );

        // Add to beginning
        this.helpHistory.unshift({
            category,
            topicId,
            timestamp: Date.now()
        });

        // Limit history size
        if (this.helpHistory.length > 50) {
            this.helpHistory = this.helpHistory.slice(0, 50);
        }

        // Save to localStorage
        this.saveUserPreferences();
    }

    getHelpTopic(category, topicId) {
        const categoryData = this.helpDatabase.get(category);
        if (!categoryData) return null;
        
        return categoryData.topics.find(topic => topic.id === topicId);
    }

    findTopicById(topicId) {
        for (const [categoryKey, categoryData] of this.helpDatabase.entries()) {
            const topic = categoryData.topics.find(t => t.id === topicId);
            if (topic) {
                return { category: categoryKey, topic };
            }
        }
        return null;
    }

    loadUserPreferences() {
        try {
            const saved = localStorage.getItem('opensim-help-preferences');
            if (saved) {
                const prefs = JSON.parse(saved);
                Object.assign(this.settings, prefs.settings || {});
                this.helpHistory = prefs.history || [];
            }
        } catch (error) {
            console.warn('Failed to load help preferences:', error);
        }
    }

    saveUserPreferences() {
        try {
            const prefs = {
                settings: this.settings,
                history: this.helpHistory
            };
            localStorage.setItem('opensim-help-preferences', JSON.stringify(prefs));
        } catch (error) {
            console.warn('Failed to save help preferences:', error);
        }
    }

    // Public API methods
    showHelp(category, topicId) {
        this.showHelpTopic(category, topicId);
    }

    search(query) {
        document.getElementById('help-search').value = query;
        this.performSearch(query);
        this.showHelpPanel();
        this.switchView('search');
    }

    showContextHelp() {
        this.showHelpPanel();
        this.switchView('context');
    }
}

// Initialize help system when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.helpSystem = new HelpSystem();
});

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = HelpSystem;
}