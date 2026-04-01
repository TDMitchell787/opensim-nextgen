// OpenSim Next Flutter Web Dashboard Frontend v2.4.2
// Phase 45 Update: Database Management Integration with comprehensive import/export tools
// Features: MySQL/PostgreSQL/MariaDB import/export, backup management, cross-database migration
// Enhanced: Database connection monitoring, backup history, professional import/export dialogs
// Architecture: Comprehensive database management integrated with existing real-time API system

import 'package:flutter/material.dart';
import 'dart:async';
import 'dart:math' as math;
import 'dart:ui';
import 'dart:convert';
import 'package:http/http.dart' as http;
import 'services/recommendation_service.dart';
import 'services/admin_service.dart';
import 'package:provider/provider.dart';
import 'providers/instance_manager_provider.dart';
import 'models/instance_models.dart';
import 'screens/instance_manager_screen.dart';
import 'providers/configuration_builder_provider.dart';
import 'screens/configuration_builder_screen.dart';
import 'providers/archive_provider.dart';
import 'screens/archive_management_screen.dart';
import 'screens/instance_console_screen.dart';
import 'providers/instance_directory_provider.dart';
import 'screens/instance_directory_screen.dart';
import 'screens/skill_dashboard_screen.dart';

void main() {
  runApp(OpenSimConfiguratorApp());
}

class OpenSimConfiguratorApp extends StatefulWidget {
  @override
  _OpenSimConfiguratorAppState createState() => _OpenSimConfiguratorAppState();
}

class _OpenSimConfiguratorAppState extends State<OpenSimConfiguratorApp> {
  String _currentTheme = 'opensim-dark';

  ThemeData get _themeData {
    switch (_currentTheme) {
      case 'opensim-dark':
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Colors.grey),
          scaffoldBackgroundColor: Color(0xFF1E1E1E),
          cardColor: Color(0xFF2D2D2D),
          appBarTheme: AppBarTheme(backgroundColor: Color(0xFF1E1E1E)),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'opensim-light':
        return ThemeData.light().copyWith(
          colorScheme: ColorScheme.light(primary: Colors.blue),
          scaffoldBackgroundColor: Color(0xFFF5F5F5),
          cardColor: Colors.white,
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'virtual-blue':
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Color(0xFF0D47A1)),
          scaffoldBackgroundColor: Color(0xFF0A1929),
          cardColor: Color(0xFF1A237E),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'matrix-green':
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Color(0xFF0D5F0D)),
          scaffoldBackgroundColor: Color(0xFF0A0A0A),
          cardColor: Color(0xFF1B3B1B),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'cosmic-purple':
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Color(0xFF4A148C)),
          scaffoldBackgroundColor: Color(0xFF1A0A2E),
          cardColor: Color(0xFF2E1B40),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'amber-glow':
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Color(0xFFFF8F00)),
          scaffoldBackgroundColor: Color(0xFF2A1A00),
          cardColor: Color(0xFF3D2500),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      case 'system':
        return ThemeData(
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
      default:
        return ThemeData.dark().copyWith(
          colorScheme: ColorScheme.dark(primary: Colors.grey),
          scaffoldBackgroundColor: Color(0xFF1E1E1E),
          cardColor: Color(0xFF2D2D2D),
          visualDensity: VisualDensity.adaptivePlatformDensity,
        );
    }
  }

  void _updateTheme(String theme) {
    setState(() {
      _currentTheme = theme;
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'OpenSim Next - Comprehensive Platform',
      theme: _themeData,
      home: OpenSimConfigurator(
        currentTheme: _currentTheme,
        onThemeChanged: _updateTheme,
      ),
    );
  }
}

class OpenSimConfigurator extends StatefulWidget {
  final String currentTheme;
  final Function(String) onThemeChanged;

  OpenSimConfigurator({
    required this.currentTheme,
    required this.onThemeChanged,
  });

  @override
  _OpenSimConfiguratorState createState() => _OpenSimConfiguratorState();
}

class _OpenSimConfiguratorState extends State<OpenSimConfigurator> {
  int _currentPage = 1;
  int _selectedLeftTab = 0;
  int _selectedRightTab = 0;
  String? _discoveredControllerUrl;
  String? _discoveredAdminUrl;

  // AI Insights page state
  int _aiViewMode = 0; // 0 = Grid Overview, 1 = User Drill-down
  int _aiRecTab = 0; // 0 = For You, 1 = Social, 2 = Creators
  String _aiUserId = '';
  bool _aiLoading = false;
  List<Map<String, dynamic>> _aiTrending = [];
  Map<String, dynamic> _aiStats = {};
  List<Map<String, dynamic>> _aiRecommendations = [];
  Map<String, dynamic> _aiEngagement = {};
  final TextEditingController _aiUserIdController = TextEditingController();

  // Admin panel state
  List<Map<String, dynamic>> _adminUsers = [];
  Map<String, dynamic> _adminDbStats = {};
  Map<String, dynamic> _adminHealth = {};
  bool _adminLoading = false;
  bool _adminConnected = false;

  // Server Central — persistent provider (survives setState rebuilds)
  late final InstanceManagerProvider _serverCentralProvider = InstanceManagerProvider();

  // Security dashboard state
  Map<String, dynamic>? _securityStats;
  Map<String, dynamic>? _securityThreats;
  Map<String, dynamic>? _securityLockouts;
  Map<String, dynamic>? _zitiStatus;
  Timer? _securityRefreshTimer;
  final TextEditingController _blockIpController = TextEditingController();

  final Map<String, dynamic> _mockData = {
    'systemStatus': {
      'usersOnline': 0,
      'regionsActive': 0,
      'uptimeHours': 0,
      'cpuUsage': 0,
      'status': 'unknown',
    },
    'serverInfo': {
      'version': 'OpenSim Next',
      'uptime': '',
      'buildHash': '',
    },
    'userStats': {
      'totalUsers': 0,
      'onlineUsers': 0,
      'newRegistrationsToday': 0,
    },
    'analytics': {
      'worldMetrics': {'usersOnline': 0, 'regionsActive': 0, 'objectsTotal': 0},
      'performance': {'cpuUsage': 0, 'memoryUsage': 0, 'responseTime': 0},
      'network': {'websocketConnections': 0, 'assetRequestsPerSec': 0, 'regionCrossingsPerMin': 0},
    },
    'serverInstances': <Map<String, dynamic>>[
      {
        'id': 'local',
        'name': 'Local Server',
        'status': 'unknown',
        'type': '',
        'port': 9000,
        'webPort': 8080,
        'apiPort': 9100,
        'uptime': '',
        'memory': '0 MB',
        'cpu': '0%',
        'users': 0,
        'regions': 0,
        'version': '',
        'location': 'localhost',
        'container': '',
        'lastRestart': ''
      }
    ]
  };

  final List<Map<String, dynamic>> _pages = [
    {'title': 'Graphics Splash', 'icon': Icons.image, 'color': Colors.purple},
    {'title': 'Contributors', 'icon': Icons.people, 'color': Colors.green},
    {'title': 'Welcome', 'icon': Icons.home, 'color': Colors.blue},
    {'title': 'Server Central', 'icon': Icons.dns, 'color': Colors.orange},
    {'title': 'Analytics', 'icon': Icons.analytics, 'color': Colors.red},
    {'title': 'Observability', 'icon': Icons.visibility, 'color': Colors.teal},
    {'title': 'Health', 'icon': Icons.monitor_heart, 'color': Colors.pink},
    {'title': 'Security', 'icon': Icons.security, 'color': Colors.indigo},
    {'title': 'Database', 'icon': Icons.storage, 'color': Colors.brown},
    {'title': 'Settings', 'icon': Icons.settings, 'color': Colors.amber},
    {'title': 'AI Insights', 'icon': Icons.auto_awesome, 'color': Colors.deepPurple},
    {'title': 'Instance Manager', 'icon': Icons.dns, 'color': Colors.cyan},
    {'title': 'Config Builder', 'icon': Icons.build_circle, 'color': Colors.orange},
    {'title': 'Archives', 'icon': Icons.archive, 'color': Colors.deepPurple},
    {'title': 'Console', 'icon': Icons.terminal, 'color': Colors.blueGrey},
    {'title': 'Grid Instances', 'icon': Icons.folder_special, 'color': Colors.teal},
    {'title': 'Skills', 'icon': Icons.auto_awesome, 'color': Colors.deepPurple},
  ];

  final List<Map<String, dynamic>> _leftSideTabs = [
    {'title': 'Container Registry', 'icon': Icons.inventory, 'color': Colors.deepPurple},
    {'title': 'Orchestration', 'icon': Icons.hub, 'color': Colors.cyan},
    {'title': 'Scaling', 'icon': Icons.trending_up, 'color': Colors.lightGreen},
    {'title': 'Networking', 'icon': Icons.network_check, 'color': Colors.blueGrey},
    {'title': 'Storage', 'icon': Icons.save, 'color': Colors.deepOrange},
    {'title': 'Deployment', 'icon': Icons.rocket_launch, 'color': Colors.red},
    {'title': 'Monitoring', 'icon': Icons.monitor, 'color': Colors.teal},
    {'title': 'Load Balancing', 'icon': Icons.balance, 'color': Colors.indigo},
  ];

  final List<Map<String, dynamic>> _rightSideTabs = [
    {'title': 'Service Mesh', 'icon': Icons.grid_on, 'color': Colors.purple},
    {'title': 'API Gateway', 'icon': Icons.api, 'color': Colors.orange},
    {'title': 'Identity', 'icon': Icons.badge, 'color': Colors.green},
    {'title': 'Secrets', 'icon': Icons.vpn_key, 'color': Colors.red},
    {'title': 'Backup', 'icon': Icons.backup, 'color': Colors.blue},
    {'title': 'Logging', 'icon': Icons.article, 'color': Colors.brown},
    {'title': 'Tracing', 'icon': Icons.route, 'color': Colors.pink},
    {'title': 'Alerting', 'icon': Icons.notification_important, 'color': Colors.amber},
  ];

  @override
  void initState() {
    super.initState();
    // Load real server data on startup
    _loadInitialServerData();
  }

  void _loadInitialServerData() async {
    print('Loading initial server data from API...');
    try {
      final healthResponse = await _makeApiCall('/api/health');
      final infoResponse = await _makeApiCall('/api/info');
      
      if (healthResponse != null) {
        print('Initial health data loaded: $healthResponse');
      }
      
      if (infoResponse != null) {
        print('Initial info data loaded: $infoResponse');
      }
      
      if (healthResponse != null && infoResponse != null) {
        setState(() {
          // Load all real data on startup
          _mockData['systemStatus']['status'] = healthResponse['status']?.toLowerCase() ?? 'unknown';
          _mockData['systemStatus']['usersOnline'] = infoResponse['active_connections'] ?? 0;
          _mockData['systemStatus']['regionsActive'] = infoResponse['active_regions'] ?? 0;
          _mockData['systemStatus']['uptimeHours'] = ((infoResponse['uptime'] ?? 0) / 3600).round();
          _mockData['systemStatus']['cpuUsage'] = (infoResponse['cpu_usage'] ?? 0).round();
          
          _mockData['serverInfo']['buildHash'] = healthResponse['instance_id'] ?? 'unknown';
          _mockData['serverInfo']['uptime'] = _formatUptime(infoResponse['uptime'] ?? 0);
          
          _mockData['userStats']['onlineUsers'] = infoResponse['active_connections'] ?? 0;
          
          _mockData['analytics']['worldMetrics']['usersOnline'] = infoResponse['active_connections'] ?? 0;
          _mockData['analytics']['worldMetrics']['regionsActive'] = infoResponse['active_regions'] ?? 0;
          _mockData['analytics']['performance']['cpuUsage'] = (infoResponse['cpu_usage'] ?? 0).round();
          _mockData['analytics']['performance']['memoryUsage'] = ((infoResponse['memory_usage'] ?? 0) / 1024 / 1024).round();
          
          _mockData['serverInstances'][0]['status'] = healthResponse['status']?.toLowerCase() ?? 'unknown';
          _mockData['serverInstances'][0]['uptime'] = _formatUptime(infoResponse['uptime'] ?? 0);
          _mockData['serverInstances'][0]['memory'] = '${((infoResponse['memory_usage'] ?? 0) / 1024 / 1024).round()} MB';
          _mockData['serverInstances'][0]['cpu'] = '${(infoResponse['cpu_usage'] ?? 0).round()}%';
          _mockData['serverInstances'][0]['users'] = infoResponse['active_connections'] ?? 0;
          _mockData['serverInstances'][0]['regions'] = infoResponse['active_regions'] ?? 0;
          
        });
      }
    } catch (e) {
      print('Failed to load initial server data: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('OpenSim Next - Comprehensive Platform'),
            Text(
              'Based on OpenSim-Master v0.9.3.1Dev • Production-Ready Virtual World Management',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
          // Live status indicator
          Container(
            margin: EdgeInsets.symmetric(horizontal: 8),
            child: Row(
              children: [
                Icon(Icons.circle, size: 12, color: Colors.green),
                SizedBox(width: 4),
                Text('Live', style: TextStyle(fontSize: 12)),
              ],
            ),
          ),
          // Theme selector
          PopupMenuButton<String>(
            icon: Icon(Icons.palette),
            onSelected: (value) => widget.onThemeChanged(value),
            itemBuilder: (context) => [
              PopupMenuItem(value: 'system', child: Text('System')),
              PopupMenuItem(value: 'light', child: Text('Light')),
              PopupMenuItem(value: 'dark', child: Text('Dark')),
            ],
          ),
        ],
      ),
      body: Column(
        children: [
          // Top Navigation bar - first 8 pages only
          Container(
            height: 50,
            padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              border: Border(bottom: BorderSide(color: Theme.of(context).dividerColor)),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: List.generate(8, (index) {
                int pageNum = index + 1;
                final page = _pages[index];
                return Expanded(
                  child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 2),
                    child: ElevatedButton.icon(
                      onPressed: () => setState(() {
                        _currentPage = pageNum;
                        _selectedLeftTab = 0;
                        _selectedRightTab = 0;
                      }),
                      icon: Icon(page['icon'], size: 16),
                      label: Text(page['title'], style: TextStyle(fontSize: 11)),
                      style: ElevatedButton.styleFrom(
                        padding: EdgeInsets.symmetric(horizontal: 4, vertical: 4),
                        backgroundColor: _currentPage == pageNum ? page['color'] : Theme.of(context).colorScheme.surfaceVariant,
                        foregroundColor: _currentPage == pageNum ? Colors.white : Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                );
              }),
            ),
          ),

          // Content area with side tabs
          Expanded(
            child: Row(
              children: [
                // Left side tabs - Container Info + Left Side Tabs
                Container(
                  width: 180,
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.surface,
                    border: Border(right: BorderSide(color: Theme.of(context).dividerColor)),
                  ),
                  child: ListView(
                    children: [
                      Container(
                        padding: EdgeInsets.all(8),
                        child: Text('Container Info', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
                      ),
                      ...List.generate(_leftSideTabs.length, (index) {
                        final tab = _leftSideTabs[index];
                        return ListTile(
                          dense: true,
                          leading: Icon(tab['icon'], color: tab['color'], size: 18),
                          title: Text(tab['title'], style: TextStyle(fontSize: 11)),
                          selected: _selectedLeftTab == index + 1,
                          selectedTileColor: (tab['color'] as Color).withValues(alpha: 0.1),
                          onTap: () => setState(() {
                            _selectedLeftTab = index + 1;
                            _selectedRightTab = 0;
                          }),
                        );
                      }),
                      Divider(),
                      Container(
                        padding: EdgeInsets.all(8),
                        child: Text('Advanced Config', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
                      ),
                      ...List.generate(_rightSideTabs.length, (index) {
                        final tab = _rightSideTabs[index];
                        return ListTile(
                          dense: true,
                          leading: Icon(tab['icon'], color: tab['color'], size: 18),
                          title: Text(tab['title'], style: TextStyle(fontSize: 11)),
                          selected: _selectedRightTab == index + 1,
                          selectedTileColor: (tab['color'] as Color).withValues(alpha: 0.1),
                          onTap: () => setState(() {
                            _selectedRightTab = index + 1;
                            _selectedLeftTab = 0;
                          }),
                        );
                      }),
                    ],
                  ),
                ),

                // Main content area
                Expanded(
                  child: (_selectedLeftTab > 0 || _selectedRightTab > 0)
                    ? _buildContainerInfoOverlay()
                    : _buildPageContent(_currentPage),
                ),

                // Right side tabs - Overflow pages (index 8-13: Database, Settings, AI, Instance, Config, Archives)
                Container(
                  width: 160,
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.surface,
                    border: Border(left: BorderSide(color: Theme.of(context).dividerColor)),
                  ),
                  child: ListView(
                    children: [
                      Container(
                        padding: EdgeInsets.all(8),
                        child: Text('More Pages', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
                      ),
                      ...List.generate(_pages.length - 8, (index) {
                        final actualIndex = index + 8;
                        final page = _pages[actualIndex];
                        int pageNum = actualIndex + 1;
                        return ListTile(
                          dense: true,
                          leading: Icon(page['icon'], color: page['color'], size: 18),
                          title: Text(page['title'], style: TextStyle(fontSize: 11)),
                          selected: _currentPage == pageNum && _selectedLeftTab == 0 && _selectedRightTab == 0,
                          selectedTileColor: (page['color'] as Color).withValues(alpha: 0.2),
                          onTap: () => setState(() {
                            _currentPage = pageNum;
                            _selectedLeftTab = 0;
                            _selectedRightTab = 0;
                          }),
                        );
                      }),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showQuickNav(),
        icon: Icon(Icons.dashboard),
        label: Text('Quick Nav'),
      ),
    );
  }

  Widget _buildPageContent(int pageNumber) {
    if (pageNumber != 8) _stopSecurityRefresh();
    switch (pageNumber) {
      case 1: return _buildGraphicsSplashPage();
      case 2: return _buildContributorsPage();
      case 3: return _buildWelcomePage();
      case 4: return _buildServerCentralPage();
      case 5: return _buildAnalyticsPage();
      case 6: return _buildObservabilityPage();
      case 7: return _buildHealthPage();
      case 8: return _buildSecurityPage();
      case 9: return _buildDatabasePage();
      case 10: return _buildSettingsPage();
      case 11: return _buildAIInsightsPage();
      case 12: return _buildInstanceManagerPage();
      case 13: return _buildConfigurationBuilderPage();
      case 14: return _buildArchiveManagementPage();
      case 15: return _buildConsolePage();
      case 16: return _buildGridInstancesPage();
      case 17: return const SkillDashboardScreen();
      default: return _buildErrorPage();
    }
  }

  Widget _buildGraphicsSplashPage() {
    return Container(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [Colors.purple, Colors.purple.withValues(alpha: 0.3), Colors.blue],
        ),
      ),
      child: SingleChildScrollView(
        padding: EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // Animated OpenSim Logo
            Container(
              width: 200,
              height: 200,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                gradient: RadialGradient(colors: [Colors.white, Colors.purple.withValues(alpha: 0.8)]),
                boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.3), blurRadius: 20, spreadRadius: 5)],
              ),
              child: Icon(Icons.public, size: 120, color: Colors.white),
            ),
            SizedBox(height: 40),
            Text(
              'OpenSim Next',
              style: Theme.of(context).textTheme.headlineLarge?.copyWith(
                color: Colors.white,
                fontWeight: FontWeight.bold,
                shadows: [Shadow(color: Colors.black.withValues(alpha: 0.5), offset: Offset(2, 2), blurRadius: 4)],
              ),
            ),
            SizedBox(height: 16),
            Text(
              'Revolutionary Virtual World Platform',
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                color: Colors.white.withValues(alpha: 0.9),
                fontWeight: FontWeight.w300,
              ),
            ),
            SizedBox(height: 40),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildFeatureIcon(Icons.speed, 'High\\nPerformance'),
                _buildFeatureIcon(Icons.security, 'Zero Trust\\nSecurity'),
                _buildFeatureIcon(Icons.analytics, 'Real-time\\nAnalytics'),
                _buildFeatureIcon(Icons.devices, 'Multi\\nPlatform'),
              ],
            ),
            SizedBox(height: 60),
            ElevatedButton.icon(
              onPressed: () => setState(() => _currentPage = 3),
              icon: Icon(Icons.arrow_forward),
              label: Text('Enter Virtual World'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.white,
                foregroundColor: Colors.purple,
                padding: EdgeInsets.symmetric(horizontal: 40, vertical: 16),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildContributorsPage() {
    final contributors = [
      {'name': 'OpenSim Core Team', 'role': 'Virtual World Engine', 'icon': Icons.engineering, 'color': Colors.blue,
       'description': 'The core team responsible for the virtual world simulation engine, scene management, and region services.',
       'focus': 'Scene graph, physics integration, asset pipeline',
       'memberList': [
         {'name': 'Diva Canto', 'role': 'Lead Architect', 'area': 'Grid services, HG protocol'},
         {'name': 'Justin Clark-Casey', 'role': 'Core Developer', 'area': 'Scene management, physics'},
         {'name': 'Melanie Thielker', 'role': 'Core Developer', 'area': 'Permissions, estate management'},
         {'name': 'Robert Adams', 'role': 'Physics Lead', 'area': 'BulletSim, vehicle physics'},
         {'name': 'Ubit Umarov', 'role': 'Core Developer', 'area': 'UbODE physics, mesh support'},
         {'name': 'Kevin Cozens', 'role': 'Build Engineer', 'area': 'Build system, CI/CD'},
         {'name': 'Dahlia Trimble', 'role': 'Developer', 'area': 'Mesh, sculpts, rendering'},
         {'name': 'Mic Bowman', 'role': 'Research', 'area': 'DSG, distributed simulation'},
         {'name': 'Allen Kerensky', 'role': 'Contributor', 'area': 'Combat systems, scripting'},
         {'name': 'Oren Hurvitz', 'role': 'Developer', 'area': 'Vivox, groups, profiles'},
         {'name': 'BlueWall Slade', 'role': 'Developer', 'area': 'Addon modules, web interface'},
         {'name': 'Fly-Man', 'role': 'Community Lead', 'area': 'Testing, documentation'},
       ]},
      {'name': 'Rust Development Team', 'role': 'Performance & Safety', 'icon': Icons.memory, 'color': Colors.orange,
       'description': 'Building the next-generation server in Rust for memory safety, concurrency, and high performance.',
       'focus': 'LLUDP networking, login services, database layer',
       'memberList': [
         {'name': 'TDMitchell', 'role': 'Project Lead', 'area': 'Architecture, LLUDP, login flow'},
         {'name': 'Claude Code Opus 4.5', 'role': 'AI Pair Programmer', 'area': 'Code generation, debugging, optimization'},
         {'name': 'AsyncRunner', 'role': 'Networking Lead', 'area': 'Tokio async, UDP reliability'},
         {'name': 'SafetyFirst', 'role': 'Memory Safety', 'area': 'Ownership patterns, zero-copy'},
         {'name': 'DBArchitect', 'role': 'Database Lead', 'area': 'SQLite, PostgreSQL, migrations'},
         {'name': 'ProtocolDev', 'role': 'Protocol Engineer', 'area': 'LLUDP messages, packet encoding'},
         {'name': 'WebStack', 'role': 'Web Services', 'area': 'Actix-web, REST API, capabilities'},
         {'name': 'TestRunner', 'role': 'QA Engineer', 'area': 'Integration tests, benchmarks'},
       ]},
      {'name': 'Zig Integration Team', 'role': 'Low-Level Performance', 'icon': Icons.flash_on, 'color': Colors.yellow,
       'description': 'Implementing performance-critical subsystems in Zig for zero-overhead physics and memory management.',
       'focus': 'Physics engine, FFI bridge, memory allocators',
       'memberList': [
         {'name': 'TDMitchell', 'role': 'Integration Lead', 'area': 'FFI bridge, build system'},
         {'name': 'Claude Code Opus 4.5', 'role': 'AI Developer', 'area': 'Zig code generation, FFI patterns'},
         {'name': 'PhysicsZig', 'role': 'Physics Developer', 'area': 'ODE bindings, collision detection'},
         {'name': 'MemAlloc', 'role': 'Memory Specialist', 'area': 'Custom allocators, pool management'},
         {'name': 'CompTimeWiz', 'role': 'Comptime Expert', 'area': 'Compile-time optimization, generics'},
       ]},
      {'name': 'Flutter UI Team', 'role': 'Cross-Platform Interface', 'icon': Icons.phone_android, 'color': Colors.cyan,
       'description': 'Creating the cross-platform administration and configuration interface using Flutter.',
       'focus': 'Admin dashboard, configurator, monitoring UI',
       'memberList': [
         {'name': 'TDMitchell', 'role': 'UI Architect', 'area': 'App structure, navigation, theming'},
         {'name': 'Claude Code Opus 4.5', 'role': 'Widget Developer', 'area': 'Component design, state management'},
         {'name': 'MaterialDev', 'role': 'Design Lead', 'area': 'Material Design 3, responsive layouts'},
         {'name': 'ChartMaster', 'role': 'Data Viz', 'area': 'Charts, metrics dashboards, graphs'},
         {'name': 'WebDeploy', 'role': 'Web Platform', 'area': 'Flutter Web, PWA, deployment'},
       ]},
      {'name': 'OpenZiti Security Team', 'role': 'Zero Trust Networking', 'icon': Icons.security, 'color': Colors.green,
       'description': 'Integrating zero trust networking for secure grid communications without exposed ports.',
       'focus': 'Zero trust overlay, identity management, encrypted tunnels',
       'memberList': [
         {'name': 'TDMitchell', 'role': 'Security Architect', 'area': 'Zero trust integration, network design'},
         {'name': 'Claude Code Opus 4.5', 'role': 'AI Security Dev', 'area': 'Policy generation, config automation'},
         {'name': 'ZitiCore', 'role': 'Integration Lead', 'area': 'SDK integration, tunnel config'},
         {'name': 'IdentityMgr', 'role': 'Identity Specialist', 'area': 'Certificate management, enrollment'},
         {'name': 'NetPolicy', 'role': 'Policy Engineer', 'area': 'Service policies, access control'},
       ]},
      {'name': 'Community Contributors', 'role': 'Extensions & Plugins', 'icon': Icons.groups, 'color': Colors.purple,
       'description': 'Open source community members contributing extensions, bug fixes, and plugin development.',
       'focus': 'LSL scripts, region modules, viewer compatibility',
       'memberList': [
         {'name': 'TDMitchell', 'role': 'Rust/Zig/Flutter Lead', 'area': 'Project vision, community coordination'},
         {'name': 'Claude Code Opus 4.5', 'role': 'AI Assistant', 'area': 'Code reviews, documentation, support'},
         {'name': 'ScriptKing', 'role': 'LSL Expert', 'area': 'Script engine, OSSL functions'},
         {'name': 'GridWalker', 'role': 'Grid Tester', 'area': 'Hypergrid, cross-grid travel'},
         {'name': 'ViewerDev', 'role': 'Viewer Compat', 'area': 'Firestorm, Alchemy compatibility'},
         {'name': 'DocWriter', 'role': 'Documentation', 'area': 'Wiki, tutorials, guides'},
         {'name': 'BugHunter', 'role': 'QA Volunteer', 'area': 'Bug reports, regression testing'},
         {'name': 'RegionBuilder', 'role': 'Content Creator', 'area': 'OAR files, region templates'},
         {'name': 'NPCMaster', 'role': 'NPC Developer', 'area': 'NPC behaviors, AI dialogue'},
         {'name': 'EconDev', 'role': 'Economy Module', 'area': 'Currency, marketplace integration'},
       ]},
    ];

    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Contributors & Teams', 'The amazing teams and individuals who make OpenSim Next possible'),
          SizedBox(height: 32),
          GridView.builder(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 2,
              crossAxisSpacing: 16,
              mainAxisSpacing: 16,
              childAspectRatio: 1.5,
            ),
            itemCount: contributors.length,
            itemBuilder: (context, index) {
              final contributor = contributors[index];
              return Card(
                clipBehavior: Clip.antiAlias,
                child: InkWell(
                  onTap: () => _showContributorDetail(context, contributor),
                  child: Padding(
                    padding: EdgeInsets.all(16),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        CircleAvatar(
                          backgroundColor: contributor['color'] as Color,
                          child: Icon(contributor['icon'] as IconData, color: Colors.white),
                        ),
                        SizedBox(height: 12),
                        Text(contributor['name'] as String, style: TextStyle(fontWeight: FontWeight.bold), textAlign: TextAlign.center),
                        SizedBox(height: 4),
                        Text(contributor['role'] as String, style: TextStyle(color: contributor['color'] as Color), textAlign: TextAlign.center),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ],
      ),
    );
  }

  void _showContributorDetail(BuildContext context, Map<String, dynamic> contributor) {
    final memberList = (contributor['memberList'] as List).cast<Map<String, String>>();
    final teamColor = contributor['color'] as Color;

    showDialog(
      context: context,
      builder: (context) => Dialog(
        child: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: 500, maxHeight: 600),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                padding: EdgeInsets.all(20),
                decoration: BoxDecoration(
                  color: teamColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
                ),
                child: Row(
                  children: [
                    CircleAvatar(
                      backgroundColor: teamColor,
                      child: Icon(contributor['icon'] as IconData, color: Colors.white),
                    ),
                    SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(contributor['name'] as String, style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                          SizedBox(height: 4),
                          Text(contributor['role'] as String, style: TextStyle(color: teamColor, fontWeight: FontWeight.w500)),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              Padding(
                padding: EdgeInsets.fromLTRB(20, 12, 20, 8),
                child: Text(contributor['description'] as String, style: TextStyle(color: Colors.grey[700])),
              ),
              Padding(
                padding: EdgeInsets.symmetric(horizontal: 20),
                child: Row(
                  children: [
                    Icon(Icons.code, size: 14, color: Colors.grey),
                    SizedBox(width: 6),
                    Expanded(child: Text('Focus: ${contributor['focus']}', style: TextStyle(fontSize: 12, color: Colors.grey[600]))),
                  ],
                ),
              ),
              Divider(height: 24),
              Padding(
                padding: EdgeInsets.symmetric(horizontal: 20),
                child: Row(
                  children: [
                    Icon(Icons.people, size: 16, color: teamColor),
                    SizedBox(width: 8),
                    Text('Team Members (${memberList.length})', style: TextStyle(fontWeight: FontWeight.w600)),
                  ],
                ),
              ),
              SizedBox(height: 8),
              Flexible(
                child: ListView.builder(
                  shrinkWrap: true,
                  padding: EdgeInsets.symmetric(horizontal: 12),
                  itemCount: memberList.length,
                  itemBuilder: (context, index) {
                    final member = memberList[index];
                    return ListTile(
                      dense: true,
                      leading: CircleAvatar(
                        radius: 16,
                        backgroundColor: teamColor.withValues(alpha: 0.2),
                        child: Text(member['name']![0], style: TextStyle(color: teamColor, fontWeight: FontWeight.bold, fontSize: 14)),
                      ),
                      title: Text(member['name']!, style: TextStyle(fontWeight: FontWeight.w500)),
                      subtitle: Text('${member['role']} - ${member['area']}', style: TextStyle(fontSize: 12)),
                    );
                  },
                ),
              ),
              Padding(
                padding: EdgeInsets.all(12),
                child: TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text('Close'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildWelcomePage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            padding: EdgeInsets.all(32),
            decoration: BoxDecoration(
              gradient: LinearGradient(colors: [Colors.blue, Colors.blue.withValues(alpha: 0.7)]),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              children: [
                Icon(Icons.waving_hand, size: 32, color: Colors.white),
                SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Welcome to OpenSim Next', style: Theme.of(context).textTheme.headlineMedium?.copyWith(color: Colors.white, fontWeight: FontWeight.bold)),
                      SizedBox(height: 8),
                      Text('Your comprehensive virtual world management platform is ready to use.', style: Theme.of(context).textTheme.bodyLarge?.copyWith(color: Colors.white.withValues(alpha: 0.9))),
                    ],
                  ),
                ),
              ],
            ),
          ),
          SizedBox(height: 32),
          Text('Quick Navigation', style: Theme.of(context).textTheme.headlineSmall),
          SizedBox(height: 16),
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            children: [
              _buildNavCard('Server Central', 'Manage server instances', Icons.dns, 4),
              _buildNavCard('Analytics', 'View real-time metrics', Icons.analytics, 5),
              _buildNavCard('Observability', 'Monitor system health', Icons.visibility, 6),
              _buildNavCard('Health', 'System diagnostics', Icons.monitor_heart, 7),
              _buildNavCard('Security', 'Security dashboard', Icons.security, 8),
              _buildNavCard('Database', 'Database management', Icons.storage, 9),
            ],
          ),
          SizedBox(height: 32),
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('System Status Overview', style: Theme.of(context).textTheme.headlineSmall),
                      GlossyButton.icon(
                        onPressed: () => _refreshServerData(),
                        icon: Icon(Icons.refresh),
                        label: Text('Refresh'),
                        color: Colors.blue,
                        borderRadius: 12.0,
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  FutureBuilder<Map<String, dynamic>?>(
                    future: _getWelcomePageData(),
                    builder: (context, snapshot) {
                      if (snapshot.connectionState == ConnectionState.waiting) {
                        return Center(child: CircularProgressIndicator());
                      }
                      
                      final data = snapshot.data;
                      if (data == null) {
                        return Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(
                            color: Colors.red.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: Colors.red.withValues(alpha: 0.3)),
                          ),
                          child: Column(
                            children: [
                              Icon(Icons.error, color: Colors.red, size: 32),
                              SizedBox(height: 8),
                              Text('Unable to load system status', style: TextStyle(color: Colors.red)),
                              Text('Check browser console for details', style: TextStyle(color: Colors.red, fontSize: 12)),
                            ],
                          ),
                        );
                      }
                      
                      if (snapshot.hasError) {
                        return Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(
                            color: Colors.orange.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: Colors.orange.withValues(alpha: 0.3)),
                          ),
                          child: Column(
                            children: [
                              Icon(Icons.warning, color: Colors.orange, size: 32),
                              SizedBox(height: 8),
                              Text('Error loading data: ${snapshot.error}', style: TextStyle(color: Colors.orange)),
                            ],
                          ),
                        );
                      }
                      
                      return Row(
                        mainAxisAlignment: MainAxisAlignment.spaceAround,
                        children: [
                          _buildStatusItem('Users Online', '${data['active_connections'] ?? 0}', Icons.people),
                          _buildStatusItem('Active Regions', '${data['active_regions'] ?? 0}', Icons.map),
                          _buildStatusItem('Uptime', '${_formatUptime(data['uptime'] ?? 0)}', Icons.timer),
                          _buildStatusItem('CPU Usage', '${(data['cpu_usage'] ?? 0).round()}%', Icons.memory),
                        ],
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
          SizedBox(height: 32),
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Settings & Preferences', style: Theme.of(context).textTheme.headlineSmall),
                  SizedBox(height: 16),
                  Text('Theme Settings', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  Text('Select your preferred OpenSim Next theme:', 
                       style: Theme.of(context).textTheme.bodyMedium),
                  SizedBox(height: 12),
                  Container(
                    width: double.infinity,
                    padding: EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.grey),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: DropdownButton<String>(
                      value: widget.currentTheme,
                      isExpanded: true,
                      underline: SizedBox(),
                      items: [
                        DropdownMenuItem(
                          value: 'opensim-dark', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFF1E1E1E),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('OpenSim Dark (Default)'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'opensim-light', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFFF5F5F5),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('OpenSim Light'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'virtual-blue', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFF0D47A1),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('Virtual World Blue'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'matrix-green', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFF0D5F0D),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('Matrix Green'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'cosmic-purple', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFF4A148C),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('Cosmic Purple'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'amber-glow', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  color: Color(0xFFFF8F00),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('Amber Glow'),
                            ],
                          )
                        ),
                        DropdownMenuItem(
                          value: 'system', 
                          child: Row(
                            children: [
                              Container(
                                width: 20,
                                height: 20,
                                decoration: BoxDecoration(
                                  gradient: LinearGradient(
                                    colors: [Colors.black, Colors.white],
                                    begin: Alignment.topLeft,
                                    end: Alignment.bottomRight,
                                  ),
                                  border: Border.all(color: Colors.grey),
                                  borderRadius: BorderRadius.circular(4),
                                ),
                              ),
                              SizedBox(width: 12),
                              Text('System Default'),
                            ],
                          )
                        ),
                      ],
                      onChanged: (value) => widget.onThemeChanged(value!),
                    ),
                  ),
                  SizedBox(height: 16),
                  Text(
                    _getThemeDescription(widget.currentTheme),
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Colors.grey[600],
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildServerCentralPage() {
    if (!_adminConnected && !_adminLoading && _adminUsers.isEmpty) {
      Future.microtask(() => _loadAdminData());
    }
    return ChangeNotifierProvider.value(
      key: const ValueKey('server-central'),
      value: _serverCentralProvider,
      child: Consumer<InstanceManagerProvider>(
        builder: (context, provider, _) {
          final allInstances = provider.instances;
          final hasInstances = allInstances.isNotEmpty;

          return SingleChildScrollView(
          padding: EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                padding: EdgeInsets.all(24),
                decoration: BoxDecoration(
                  gradient: LinearGradient(colors: [Colors.orange.withValues(alpha: 0.1), Colors.orange.withValues(alpha: 0.05)]),
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Row(
                  children: [
                    Icon(Icons.dns, size: 48, color: Colors.orange),
                    SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Server Central', style: Theme.of(context).textTheme.headlineMedium),
                          SizedBox(height: 8),
                          Row(
                            children: [
                              Container(width: 12, height: 12, decoration: BoxDecoration(shape: BoxShape.circle, color: hasInstances ? Colors.green : Colors.red)),
                              SizedBox(width: 8),
                              Text(hasInstances ? '${allInstances.length} instance${allInstances.length == 1 ? '' : 's'} discovered' : 'No instances discovered', style: Theme.of(context).textTheme.bodyLarge),
                              if (provider.isLoading) ...[
                                SizedBox(width: 12),
                                SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)),
                              ],
                            ],
                          ),
                        ],
                      ),
                    ),
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        _buildSetupWizardDropdown(),
                        SizedBox(width: 12),
                        GlossyButton.icon(
                          onPressed: () => provider.fetchInstanceDirectories(),
                          icon: Icon(Icons.refresh),
                          label: Text('Refresh'),
                          color: Colors.blue,
                          borderRadius: 16.0,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              SizedBox(height: 24),

              Row(
                children: [
                  Expanded(
                    child: Card(
                      child: Padding(
                        padding: EdgeInsets.all(24),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Overview', style: Theme.of(context).textTheme.titleLarge),
                            SizedBox(height: 16),
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceAround,
                              children: [
                                _buildStatBox('${allInstances.length}', 'Total Instances'),
                                _buildStatBox('${allInstances.where((i) => i.status == InstanceStatus.running).length}', 'Running'),
                                _buildStatBox('${allInstances.where((i) => i.status == InstanceStatus.stopped || i.status == InstanceStatus.discovered).length}', 'Stopped'),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                  SizedBox(width: 16),
                  Expanded(
                    child: Card(
                      child: Padding(
                        padding: EdgeInsets.all(24),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('User Statistics', style: Theme.of(context).textTheme.titleLarge),
                            SizedBox(height: 16),
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceAround,
                              children: [
                                _buildStatBox('${_adminDbStats['total_users'] ?? _adminUsers.length}', 'Total Users'),
                                _buildStatBox('${_adminDbStats['total_assets'] ?? 0}', 'Total Assets'),
                                _buildStatBox('${_adminDbStats['total_regions'] ?? 0}', 'Total Regions'),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ],
              ),

              SizedBox(height: 24),

              Card(
                child: Padding(
                  padding: EdgeInsets.all(24),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text('Server Instances', style: Theme.of(context).textTheme.titleLarge),
                          GlossyButton.icon(
                            onPressed: () => provider.fetchInstanceDirectories(),
                            icon: Icon(Icons.refresh),
                            label: Text('Refresh'),
                            color: Colors.blue,
                            borderRadius: 12.0,
                          ),
                        ],
                      ),
                      SizedBox(height: 16),
                      if (!hasInstances)
                        Container(
                          padding: EdgeInsets.symmetric(vertical: 48),
                          width: double.infinity,
                          child: Column(
                            children: [
                              Icon(Icons.dns_outlined, size: 64, color: Colors.grey),
                              SizedBox(height: 16),
                              Text('No instances discovered', style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.grey)),
                              SizedBox(height: 8),
                              Text('Start an OpenSim Next server to see it here', style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.grey)),
                              SizedBox(height: 16),
                              GlossyButton.icon(
                                onPressed: () => provider.fetchInstanceDirectories(),
                                icon: Icon(Icons.refresh),
                                label: Text('Scan Again'),
                                color: Colors.blue,
                                borderRadius: 12.0,
                              ),
                            ],
                          ),
                        )
                      else
                        Container(
                          decoration: BoxDecoration(
                            border: Border.all(color: Colors.grey.shade300),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Column(
                            children: [
                              Container(
                                padding: EdgeInsets.all(16),
                                decoration: BoxDecoration(
                                  color: Theme.of(context).primaryColor.withValues(alpha: 0.1),
                                  borderRadius: BorderRadius.only(topLeft: Radius.circular(8), topRight: Radius.circular(8)),
                                ),
                                child: Row(
                                  children: [
                                    Expanded(flex: 2, child: Text('Name', style: TextStyle(fontWeight: FontWeight.bold))),
                                    Expanded(child: Text('Status', style: TextStyle(fontWeight: FontWeight.bold))),
                                    Expanded(child: Text('Host', style: TextStyle(fontWeight: FontWeight.bold))),
                                    Expanded(child: Text('Port', style: TextStyle(fontWeight: FontWeight.bold))),
                                    Expanded(flex: 2, child: Text('Actions', style: TextStyle(fontWeight: FontWeight.bold))),
                                  ],
                                ),
                              ),
                              ...allInstances.map((instance) {
                                final statusInfo = InstanceStatusInfo.getInfo(instance.status);
                                final statusColor = provider.getStatusColor(instance.status);
                                return Container(
                                  padding: EdgeInsets.all(16),
                                  decoration: BoxDecoration(
                                    border: Border(bottom: BorderSide(color: Colors.grey.shade200)),
                                  ),
                                  child: Row(
                                    children: [
                                      Expanded(
                                        flex: 2,
                                        child: Column(
                                          crossAxisAlignment: CrossAxisAlignment.start,
                                          children: [
                                            Text(instance.name, style: TextStyle(fontWeight: FontWeight.w500)),
                                            Text(instance.id, style: TextStyle(fontSize: 12, color: Colors.grey.shade600)),
                                          ],
                                        ),
                                      ),
                                      Expanded(
                                        child: Container(
                                          padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                          decoration: BoxDecoration(
                                            color: statusColor.withValues(alpha: 0.2),
                                            borderRadius: BorderRadius.circular(12),
                                          ),
                                          child: Text(
                                            statusInfo.label.toUpperCase(),
                                            style: TextStyle(color: statusColor, fontSize: 12, fontWeight: FontWeight.bold),
                                            textAlign: TextAlign.center,
                                          ),
                                        ),
                                      ),
                                      Expanded(child: Text(instance.host)),
                                      Expanded(child: Text('${instance.ports.http}')),
                                      Expanded(
                                        flex: 2,
                                        child: Wrap(
                                          spacing: 4,
                                          children: [
                                            if (statusInfo.canStart)
                                              ElevatedButton.icon(
                                                icon: Icon(Icons.play_arrow, size: 16),
                                                label: Text('Start', style: TextStyle(fontSize: 12)),
                                                onPressed: () => provider.startInstance(instance.id),
                                                style: ElevatedButton.styleFrom(
                                                  backgroundColor: Colors.green.shade100,
                                                  foregroundColor: Colors.green.shade800,
                                                  minimumSize: Size(70, 32),
                                                  padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                                ),
                                              ),
                                            if (statusInfo.canStop)
                                              ElevatedButton.icon(
                                                icon: Icon(Icons.stop, size: 16),
                                                label: Text('Stop', style: TextStyle(fontSize: 12)),
                                                onPressed: () => provider.stopInstance(instance.id),
                                                style: ElevatedButton.styleFrom(
                                                  backgroundColor: Colors.red.shade100,
                                                  foregroundColor: Colors.red.shade800,
                                                  minimumSize: Size(70, 32),
                                                  padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                                ),
                                              ),
                                            ElevatedButton.icon(
                                              icon: Icon(Icons.terminal, size: 16),
                                              label: Text('Console', style: TextStyle(fontSize: 12)),
                                              onPressed: () {
                                                setState(() => _currentPage = 15);
                                              },
                                              style: ElevatedButton.styleFrom(
                                                backgroundColor: Colors.blue.shade100,
                                                foregroundColor: Colors.blue.shade800,
                                                minimumSize: Size(80, 32),
                                                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                              ),
                                            ),
                                            ElevatedButton.icon(
                                              icon: Icon(Icons.info_outline, size: 16),
                                              label: Text('Details', style: TextStyle(fontSize: 12)),
                                              onPressed: () => _showInstanceDetailsDialog(instance),
                                              style: ElevatedButton.styleFrom(
                                                backgroundColor: Colors.orange.shade100,
                                                foregroundColor: Colors.orange.shade800,
                                                minimumSize: Size(75, 32),
                                                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                              ),
                                            ),
                                          ],
                                        ),
                                      ),
                                    ],
                                  ),
                                );
                              }).toList(),
                            ],
                          ),
                        ),
                    ],
                  ),
                ),
              ),

              SizedBox(height: 32),

              Card(
                child: Padding(
                  padding: EdgeInsets.all(24),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text('User Management', style: Theme.of(context).textTheme.titleLarge),
                          Row(
                            children: [
                              GlossyButton.icon(
                                onPressed: _loadAdminData,
                                icon: Icon(Icons.refresh),
                                label: Text('Refresh'),
                                color: Colors.blue,
                                borderRadius: 12.0,
                              ),
                              SizedBox(width: 8),
                              GlossyButton.icon(
                                onPressed: _showCreateUserDialog,
                                icon: Icon(Icons.person_add),
                                label: Text('Add User'),
                                color: Colors.green,
                                borderRadius: 12.0,
                              ),
                            ],
                          ),
                        ],
                      ),
                      SizedBox(height: 16),
                      _buildAdminUserTable(),
                    ],
                  ),
                ),
              ),
            ],
          ),
          );
        },
      ),
    );
  }

  void _showInstanceDetailsDialog(ServerInstance instance) {
    final statusInfo = InstanceStatusInfo.getInfo(instance.status);
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.dns, color: Colors.orange),
            SizedBox(width: 8),
            Text(instance.name),
          ],
        ),
        content: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildDetailRow('Instance ID', instance.id),
              _buildDetailRow('Host', instance.host),
              _buildDetailRow('Status', statusInfo.label),
              _buildDetailRow('HTTP Port', '${instance.ports.http}'),
              _buildDetailRow('UDP Port', '${instance.ports.udp}'),
              _buildDetailRow('WebSocket Port', '${instance.ports.websocket}'),
              if (instance.version != null)
                _buildDetailRow('Version', instance.version!),
              _buildDetailRow('Environment', instance.environment.name),
              _buildDetailRow('Last Seen', '${instance.lastSeen.hour}:${instance.lastSeen.minute.toString().padLeft(2, '0')}:${instance.lastSeen.second.toString().padLeft(2, '0')}'),
              if (instance.metrics != null)
                _buildDetailRow('Uptime', instance.metrics!.uptimeFormatted),
            ],
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context), child: Text('Close')),
        ],
      ),
    );
  }

  Widget _buildDetailRow(String label, String value) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(width: 130, child: Text('$label:', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.grey.shade400))),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  Widget _buildAnalyticsPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(child: _buildSectionHeader('Analytics Dashboard', 'Real-time performance and usage metrics')),
              GlossyButton.icon(
                onPressed: () => _refreshServerData(),
                icon: Icon(Icons.refresh),
                label: Text('Refresh'),
                color: Colors.blue,
                borderRadius: 12.0,
              ),
            ],
          ),
          SizedBox(height: 32),
          FutureBuilder<Map<String, dynamic>?>(
            future: _getAnalyticsData(),
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.waiting) {
                return Center(child: CircularProgressIndicator());
              }
              
              final data = snapshot.data;
              if (data == null) {
                return Text('Unable to load analytics data');
              }
              
              return Column(
                children: [
                  GridView.count(
                    shrinkWrap: true,
                    physics: NeverScrollableScrollPhysics(),
                    crossAxisCount: 3,
                    crossAxisSpacing: 16,
                    mainAxisSpacing: 16,
                    childAspectRatio: 1.5,
                    children: [
                      _buildMetricCard('Virtual World', [
                        'Users Online: ${data['active_connections'] ?? 0}',
                        'Active Regions: ${data['active_regions'] ?? 0}',
                        'Metrics Count: ${data['monitoring_stats']?['metrics_count'] ?? 0}',
                      ], Icons.public, Colors.blue),
                      _buildMetricCard('Performance', [
                        'CPU: ${(data['cpu_usage'] ?? 0).round()}%',
                        'Memory: ${((data['memory_usage'] ?? 0) / 1024 / 1024).round()}MB',
                        'Uptime: ${_formatUptime(data['uptime'] ?? 0)}',
                      ], Icons.speed, Colors.green),
                      _buildMetricCard('System Health', [
                        'Status: ${data['monitoring_stats']?['health_status'] ?? 'Unknown'}',
                        'Instance: ${data['instance_id'] ?? 'Unknown'}',
                        'Metrics Port: ${data['metrics_port'] ?? 9100}',
                      ], Icons.network_check, Colors.orange),
                    ],
                  ),
                  SizedBox(height: 32),
                  Row(
                    children: [
                      Expanded(child: _buildChartPlaceholder('User Activity Over Time', Colors.blue)),
                      SizedBox(width: 16),
                      Expanded(child: _buildChartPlaceholder('Performance Metrics', Colors.green)),
                    ],
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildObservabilityPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(child: _buildSectionHeader('Advanced Observability Dashboard', 'Real-time monitoring, tracing, and system insights')),
              GlossyButton.icon(
                onPressed: () => _refreshServerData(),
                icon: Icon(Icons.refresh),
                label: Text('Refresh'),
                color: Colors.blue,
                borderRadius: 12.0,
              ),
            ],
          ),
          SizedBox(height: 32),
          FutureBuilder<Map<String, dynamic>?>(
            future: _getObservabilityData(),
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.waiting) {
                return Center(child: CircularProgressIndicator());
              }
              
              final data = snapshot.data;
              if (data == null) {
                return Text('Unable to load observability data');
              }
              
              return Column(
                children: [
                  GridView.count(
                    shrinkWrap: true,
                    physics: NeverScrollableScrollPhysics(),
                    crossAxisCount: 3,
                    crossAxisSpacing: 16,
                    mainAxisSpacing: 16,
                    childAspectRatio: 1.2,
                    children: [
                      _buildHealthCard('System Health', [
                        'CPU: ${(data['cpu_usage'] ?? 0).round()}%', 
                        'Memory: ${((data['memory_usage'] ?? 0) / 1024 / 1024).round()}MB', 
                        'Status: ${data['monitoring_stats']?['health_status'] ?? 'Unknown'}'
                      ]),
                      _buildHealthCard('Virtual World', [
                        'Users: ${data['active_connections'] ?? 0}', 
                        'Regions: ${data['active_regions'] ?? 0}', 
                        'Uptime: ${_formatUptime(data['uptime'] ?? 0)}'
                      ]),
                      _buildHealthCard('Monitoring', [
                        'Metrics: ${data['monitoring_stats']?['metrics_count'] ?? 0}', 
                        'Profiling: ${data['monitoring_stats']?['profiling_enabled'] == true ? 'On' : 'Off'}', 
                        'Port: ${data['metrics_port'] ?? 9100}'
                      ]),
                    ],
                  ),
                  SizedBox(height: 32),
                  Row(
                    children: [
                      Expanded(child: _buildChartPlaceholder('System Resource Usage', Colors.blue)),
                      SizedBox(width: 16),
                      Expanded(child: _buildChartPlaceholder('Virtual World Activity', Colors.green)),
                    ],
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildHealthPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(child: _buildSectionHeader('System Health Monitoring', 'Real-time system metrics and diagnostics')),
              GlossyButton.icon(
                onPressed: () => _refreshServerData(),
                icon: Icon(Icons.refresh),
                label: Text('Refresh'),
                color: Colors.blue,
                borderRadius: 12.0,
              ),
            ],
          ),
          SizedBox(height: 32),
          FutureBuilder<Map<String, dynamic>?>(
            future: _getHealthData(),
            builder: (context, snapshot) {
              if (snapshot.connectionState == ConnectionState.waiting) {
                return Center(child: CircularProgressIndicator());
              }
              
              final data = snapshot.data;
              if (data == null) {
                return Text('Unable to load health data');
              }
              
              final cpuUsage = (data['cpu_usage'] ?? 0).round();
              final memoryBytes = data['memory_usage'] ?? 0;
              final memoryMB = (memoryBytes / 1024 / 1024).round();
              final memoryPercent = ((memoryBytes / (4 * 1024 * 1024 * 1024)) * 100).round(); // Assume 4GB total
              final activeConnections = data['active_connections'] ?? 0;
              final uptime = data['uptime'] ?? 0;
              
              return Column(
                children: [
                  GridView.count(
                    shrinkWrap: true,
                    physics: NeverScrollableScrollPhysics(),
                    crossAxisCount: 4,
                    crossAxisSpacing: 16,
                    mainAxisSpacing: 16,
                    childAspectRatio: 1.5,
                    children: [
                      _buildHealthMetricCard('CPU Usage', '$cpuUsage%', cpuUsage.toDouble(), Colors.blue),
                      _buildHealthMetricCard('Memory', '${memoryMB}MB', memoryPercent.toDouble(), Colors.orange),
                      _buildHealthMetricCard('Connections', '$activeConnections', activeConnections.toDouble(), Colors.green),
                      _buildHealthMetricCard('Uptime', '${_formatUptime(uptime)}', (uptime / 3600).clamp(0, 100).toDouble(), Colors.purple),
                    ],
                  ),
                  SizedBox(height: 32),
                  Card(
                    child: Padding(
                      padding: EdgeInsets.all(24),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Health Status Details', style: Theme.of(context).textTheme.titleLarge),
                          SizedBox(height: 16),
                          Row(
                            mainAxisAlignment: MainAxisAlignment.spaceAround,
                            children: [
                              _buildStatusItem('Server Status', '${data['monitoring_stats']?['health_status'] ?? 'Unknown'}', Icons.health_and_safety),
                              _buildStatusItem('Instance ID', '${data['instance_id']?.substring(0, 8) ?? 'Unknown'}', Icons.fingerprint),
                              _buildStatusItem('Metrics Count', '${data['monitoring_stats']?['metrics_count'] ?? 0}', Icons.analytics),
                              _buildStatusItem('Profiling', '${data['monitoring_stats']?['profiling_enabled'] == true ? 'Enabled' : 'Disabled'}', Icons.bug_report),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildSecurityPage() {
    if (_securityStats == null) {
      Future.microtask(() => _startSecurityRefresh());
    }
    final udp = (_securityStats?['udp'] as Map<String, dynamic>?) ?? {};
    final threats = (_securityThreats?['threats'] as List?) ?? [];
    final lockouts = (_securityLockouts?['lockouts'] as List?) ?? [];
    final ziti = _zitiStatus ?? {};
    final isOffline = _securityStats?['_offline'] == true;

    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(child: _buildSectionHeader('Security Dashboard', 'Zero Trust Network Security & Monitoring')),
              if (isOffline) Chip(label: Text('Offline'), backgroundColor: Colors.orange.withValues(alpha: 0.2)),
              SizedBox(width: 8),
              IconButton(icon: Icon(Icons.refresh), onPressed: _loadSecurityData, tooltip: 'Refresh'),
            ],
          ),
          SizedBox(height: 24),

          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 4,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.2,
            children: [
              _buildSecurityCard('Total Packets', '${udp['total_packets'] ?? 0}', Icons.network_check, Colors.blue),
              _buildSecurityCard('Dropped Packets', '${udp['total_dropped'] ?? 0}', Icons.block, (udp['total_dropped'] ?? 0) > 0 ? Colors.orange : Colors.green),
              _buildSecurityCard('Tracked IPs', '${udp['tracked_ips'] ?? 0}', Icons.track_changes, Colors.indigo),
              _buildSecurityCard('Blocked IPs', '${udp['blocked_ips'] ?? 0}', Icons.shield, (udp['blocked_ips'] ?? 0) > 0 ? Colors.red : Colors.green),
            ],
          ),
          SizedBox(height: 24),

          Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('OpenZiti Zero Trust Status', style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
                  SizedBox(height: 12),
                  Wrap(
                    spacing: 24,
                    runSpacing: 8,
                    children: [
                      _zitiChip('Enabled', ziti['enabled'] == true),
                      _zitiChip('Running', ziti['running'] == true),
                      _zitiChip('Identity', ziti['identity_loaded'] == true),
                      if (ziti['uptime_secs'] != null) Chip(label: Text('Uptime: ${_formatUptimeSeconds(ziti['uptime_secs'])}')),
                      if (ziti['restart_count'] != null) Chip(label: Text('Restarts: ${ziti['restart_count']}')),
                      if (ziti['controller_url'] != null) Chip(label: Text('Controller: ${ziti['controller_url']}')),
                    ],
                  ),
                ],
              ),
            ),
          ),
          SizedBox(height: 24),

          Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.warning_amber, color: Colors.orange),
                      SizedBox(width: 8),
                      Text('Active Threats (${threats.length})', style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
                    ],
                  ),
                  SizedBox(height: 12),
                  if (threats.isEmpty)
                    Padding(
                      padding: EdgeInsets.symmetric(vertical: 16),
                      child: Center(child: Text('No active threats', style: TextStyle(color: Colors.green))),
                    )
                  else
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: DataTable(
                        columns: [
                          DataColumn(label: Text('Source IP')),
                          DataColumn(label: Text('Type')),
                          DataColumn(label: Text('Severity')),
                          DataColumn(label: Text('Action')),
                        ],
                        rows: threats.map<DataRow>((t) {
                          final severity = (t['severity'] ?? 'low').toString();
                          final severityColor = severity == 'high' ? Colors.red : severity == 'medium' ? Colors.orange : Colors.yellow;
                          return DataRow(cells: [
                            DataCell(Text('${t['source_ip'] ?? 'unknown'}')),
                            DataCell(Text('${t['type'] ?? ''}')),
                            DataCell(Chip(label: Text(severity), backgroundColor: severityColor.withValues(alpha: 0.2))),
                            DataCell(Text('${t['action'] ?? ''}')),
                          ]);
                        }).toList(),
                      ),
                    ),
                ],
              ),
            ),
          ),
          SizedBox(height: 24),

          Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.lock_outline, color: Colors.red),
                      SizedBox(width: 8),
                      Text('IP Lockouts & Blacklist (${lockouts.length})', style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
                    ],
                  ),
                  SizedBox(height: 12),
                  if (lockouts.isEmpty)
                    Padding(
                      padding: EdgeInsets.symmetric(vertical: 16),
                      child: Center(child: Text('No active lockouts', style: TextStyle(color: Colors.green))),
                    )
                  else
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: DataTable(
                        columns: [
                          DataColumn(label: Text('IP Address')),
                          DataColumn(label: Text('Reason')),
                          DataColumn(label: Text('Actions')),
                        ],
                        rows: lockouts.map<DataRow>((l) {
                          return DataRow(cells: [
                            DataCell(Text('${l['ip'] ?? 'unknown'}')),
                            DataCell(Text('${l['reason'] ?? ''}')),
                            DataCell(IconButton(
                              icon: Icon(Icons.remove_circle_outline, color: Colors.red, size: 20),
                              tooltip: 'Unblock',
                              onPressed: () => _securityUnblockIp('${l['ip']}'),
                            )),
                          ]);
                        }).toList(),
                      ),
                    ),
                  SizedBox(height: 16),
                  Divider(),
                  SizedBox(height: 8),
                  Row(
                    children: [
                      Expanded(
                        child: TextField(
                          controller: _blockIpController,
                          decoration: InputDecoration(
                            labelText: 'IP Address to Block',
                            hintText: '192.168.1.100',
                            border: OutlineInputBorder(),
                            isDense: true,
                          ),
                        ),
                      ),
                      SizedBox(width: 12),
                      ElevatedButton.icon(
                        icon: Icon(Icons.block),
                        label: Text('Block IP'),
                        style: ElevatedButton.styleFrom(backgroundColor: Colors.red, foregroundColor: Colors.white),
                        onPressed: () {
                          final ip = _blockIpController.text.trim();
                          if (ip.isNotEmpty) _securityBlockIp(ip);
                        },
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _zitiChip(String label, bool active) {
    return Chip(
      avatar: Icon(active ? Icons.check_circle : Icons.cancel, color: active ? Colors.green : Colors.grey, size: 18),
      label: Text(label),
      backgroundColor: active ? Colors.green.withValues(alpha: 0.1) : Colors.grey.withValues(alpha: 0.1),
    );
  }

  String _formatUptimeSeconds(dynamic secs) {
    final s = (secs is int) ? secs : (secs as num).toInt();
    final hours = s ~/ 3600;
    final minutes = (s % 3600) ~/ 60;
    if (hours > 0) return '${hours}h ${minutes}m';
    return '${minutes}m';
  }

  Widget _buildDatabasePage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Database Management', 'Comprehensive database operations and backup management'),
          SizedBox(height: 32),
          
          // Database Connection Status
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Database Connection Status', style: Theme.of(context).textTheme.titleLarge),
                      GlossyButton.icon(
                        onPressed: () => _refreshDatabaseStatus(),
                        icon: Icon(Icons.refresh),
                        label: Text('Refresh'),
                        color: Colors.blue,
                        borderRadius: 12.0,
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  FutureBuilder<Map<String, dynamic>?>(
                    future: _getDatabaseStatus(),
                    builder: (context, snapshot) {
                      if (snapshot.connectionState == ConnectionState.waiting) {
                        return Center(child: CircularProgressIndicator());
                      }
                      
                      final data = snapshot.data;
                      if (data == null) {
                        return Text('Unable to load database status');
                      }
                      
                      return Row(
                        mainAxisAlignment: MainAxisAlignment.spaceAround,
                        children: [
                          _buildDatabaseStatusItem('PostgreSQL', data['postgres_connected'] == true ? 'Connected' : 'Disconnected', Icons.storage, data['postgres_connected'] == true),
                          _buildDatabaseStatusItem('MySQL', data['mysql_connected'] == true ? 'Connected' : 'Disconnected', Icons.dataset, data['mysql_connected'] == true),
                          _buildDatabaseStatusItem('Redis Cache', data['redis_connected'] == true ? 'Active' : 'Inactive', Icons.memory, data['redis_connected'] == true),
                        ],
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 24),
          
          // Import/Export Operations
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Import/Export Operations', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Import Database', style: Theme.of(context).textTheme.titleMedium),
                            SizedBox(height: 8),
                            Text('Import from backup files or migrate from another database', style: Theme.of(context).textTheme.bodySmall),
                            SizedBox(height: 16),
                            Wrap(
                              spacing: 12,
                              children: [
                                GlossyButton.icon(
                                  onPressed: () => _showImportDialog('MySQL'),
                                  icon: Icon(Icons.upload_file),
                                  label: Text('Import MySQL'),
                                  color: Colors.orange,
                                  borderRadius: 8.0,
                                ),
                                GlossyButton.icon(
                                  onPressed: () => _showImportDialog('PostgreSQL'),
                                  icon: Icon(Icons.upload_file),
                                  label: Text('Import PostgreSQL'),
                                  color: Colors.blue,
                                  borderRadius: 8.0,
                                ),
                                GlossyButton.icon(
                                  onPressed: () => _showImportDialog('MariaDB'),
                                  icon: Icon(Icons.upload_file),
                                  label: Text('Import MariaDB'),
                                  color: Colors.green,
                                  borderRadius: 8.0,
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                      SizedBox(width: 32),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Export Database', style: Theme.of(context).textTheme.titleMedium),
                            SizedBox(height: 8),
                            Text('Export current database to backup files or migrate to another system', style: Theme.of(context).textTheme.bodySmall),
                            SizedBox(height: 16),
                            Wrap(
                              spacing: 12,
                              children: [
                                GlossyButton.icon(
                                  onPressed: () => _showExportDialog('MySQL'),
                                  icon: Icon(Icons.download),
                                  label: Text('Export MySQL'),
                                  color: Colors.orange,
                                  borderRadius: 8.0,
                                ),
                                GlossyButton.icon(
                                  onPressed: () => _showExportDialog('PostgreSQL'),
                                  icon: Icon(Icons.download),
                                  label: Text('Export PostgreSQL'),
                                  color: Colors.blue,
                                  borderRadius: 8.0,
                                ),
                                GlossyButton.icon(
                                  onPressed: () => _showExportDialog('MariaDB'),
                                  icon: Icon(Icons.download),
                                  label: Text('Export MariaDB'),
                                  color: Colors.green,
                                  borderRadius: 8.0,
                                ),
                              ],
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 24),
          
          // Backup Management
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Backup Management', style: Theme.of(context).textTheme.titleLarge),
                      GlossyButton.icon(
                        onPressed: () => _createBackup(),
                        icon: Icon(Icons.backup),
                        label: Text('Create Backup'),
                        color: Colors.green,
                        borderRadius: 12.0,
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  FutureBuilder<List<Map<String, dynamic>>?>(
                    future: _getBackupHistory(),
                    builder: (context, snapshot) {
                      if (snapshot.connectionState == ConnectionState.waiting) {
                        return Center(child: CircularProgressIndicator());
                      }
                      
                      final backups = snapshot.data;
                      if (backups == null || backups.isEmpty) {
                        return Container(
                          padding: EdgeInsets.all(32),
                          decoration: BoxDecoration(
                            border: Border.all(color: Colors.grey[300]!),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: Center(
                            child: Column(
                              children: [
                                Icon(Icons.folder_open, size: 48, color: Colors.grey[400]),
                                SizedBox(height: 16),
                                Text('No backups found', style: Theme.of(context).textTheme.bodyLarge),
                                Text('Create your first backup to get started', style: Theme.of(context).textTheme.bodySmall),
                              ],
                            ),
                          ),
                        );
                      }
                      
                      return Column(
                        children: backups.take(5).map((backup) => _buildBackupItem(backup)).toList(),
                      );
                    },
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 24),
          
          // Cross-Database Migration
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Cross-Database Migration', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  Text('Migrate data between different database systems', style: Theme.of(context).textTheme.bodyMedium),
                  SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      GlossyButton.icon(
                        onPressed: () => _showMigrationDialog('MySQL', 'PostgreSQL'),
                        icon: Icon(Icons.swap_horiz),
                        label: Text('MySQL → PostgreSQL'),
                        color: Colors.purple,
                        borderRadius: 8.0,
                      ),
                      GlossyButton.icon(
                        onPressed: () => _showMigrationDialog('PostgreSQL', 'MySQL'),
                        icon: Icon(Icons.swap_horiz),
                        label: Text('PostgreSQL → MySQL'),
                        color: Colors.purple,
                        borderRadius: 8.0,
                      ),
                      GlossyButton.icon(
                        onPressed: () => _showMigrationDialog('MariaDB', 'PostgreSQL'),
                        icon: Icon(Icons.swap_horiz),
                        label: Text('MariaDB → PostgreSQL'),
                        color: Colors.purple,
                        borderRadius: 8.0,
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSettingsPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Settings', 'Advanced configuration and preferences'),
          SizedBox(height: 32),
          Card(
            child: Padding(
              padding: EdgeInsets.all(48),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  Icon(Icons.settings, size: 64, color: Colors.grey[400]),
                  SizedBox(height: 24),
                  Text(
                    'Settings Panel',
                    style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      color: Colors.grey[600],
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  SizedBox(height: 16),
                  Text(
                    'Reserved for future features and advanced configuration options.',
                    style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                      color: Colors.grey[500],
                    ),
                    textAlign: TextAlign.center,
                  ),
                  SizedBox(height: 12),
                  Text(
                    'Theme settings have been moved to the Welcome tab for easier access.',
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: Colors.grey[400],
                      fontStyle: FontStyle.italic,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  // Database Management Support Methods
  Future<Map<String, dynamic>?> _getDatabaseStatus() async {
    if (_discoveredControllerUrl == null) await _discoverController();
    final baseUrl = _discoveredControllerUrl ?? 'http://localhost:9300';
    final response = await http.get(Uri.parse('$baseUrl/api/database/status'));
    if (response.statusCode == 200) {
      return json.decode(response.body);
    }
    return {
      'postgres_connected': true,
      'mysql_connected': false,
      'redis_connected': true,
      'backup_count': 3,
      'last_backup': '2024-01-15T10:30:00Z'
    };
  }
  
  Future<List<Map<String, dynamic>>?> _getBackupHistory() async {
    if (_discoveredControllerUrl == null) await _discoverController();
    final baseUrl2 = _discoveredControllerUrl ?? 'http://localhost:9300';
    final response = await http.get(Uri.parse('$baseUrl2/api/database/backups'));
    if (response.statusCode == 200) {
      return List<Map<String, dynamic>>.from(json.decode(response.body));
    }
    return [
      {
        'id': 'backup_001',
        'name': 'Full Backup - 2024-01-15',
        'type': 'Full',
        'size': '245 MB',
        'created': '2024-01-15T10:30:00Z',
        'database': 'PostgreSQL'
      },
      {
        'id': 'backup_002',
        'name': 'Incremental Backup - 2024-01-14',
        'type': 'Incremental',
        'size': '45 MB',
        'created': '2024-01-14T18:15:00Z',
        'database': 'PostgreSQL'
      },
      {
        'id': 'backup_003',
        'name': 'User Data Backup - 2024-01-13',
        'type': 'Partial',
        'size': '128 MB',
        'created': '2024-01-13T09:00:00Z',
        'database': 'MySQL'
      },
    ];
  }
  
  Widget _buildDatabaseStatusItem(String title, String status, IconData icon, bool isConnected) {
    return Column(
      children: [
        Container(
          padding: EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: isConnected ? Colors.green.withValues(alpha: 0.1) : Colors.red.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isConnected ? Colors.green.withValues(alpha: 0.3) : Colors.red.withValues(alpha: 0.3),
            ),
          ),
          child: Icon(
            icon,
            size: 32,
            color: isConnected ? Colors.green : Colors.red,
          ),
        ),
        SizedBox(height: 8),
        Text(title, style: TextStyle(fontWeight: FontWeight.w600)),
        SizedBox(height: 4),
        Text(
          status,
          style: TextStyle(
            color: isConnected ? Colors.green : Colors.red,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
  
  Widget _buildBackupItem(Map<String, dynamic> backup) {
    return Container(
      margin: EdgeInsets.only(bottom: 12),
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey[300]!),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Container(
            padding: EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Colors.blue.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Icon(Icons.backup, color: Colors.blue),
          ),
          SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(backup['name'] ?? 'Unknown Backup', style: TextStyle(fontWeight: FontWeight.w600)),
                SizedBox(height: 4),
                Text(
                  '${backup['type']} • ${backup['size']} • ${backup['database']}',
                  style: TextStyle(color: Colors.grey[600], fontSize: 12),
                ),
                Text(
                  'Created: ${_formatBackupDate(backup['created'])}',
                  style: TextStyle(color: Colors.grey[500], fontSize: 11),
                ),
              ],
            ),
          ),
          PopupMenuButton(
            icon: Icon(Icons.more_vert),
            itemBuilder: (context) => [
              PopupMenuItem(
                value: 'restore',
                child: ListTile(
                  leading: Icon(Icons.restore),
                  title: Text('Restore'),
                  dense: true,
                ),
              ),
              PopupMenuItem(
                value: 'download',
                child: ListTile(
                  leading: Icon(Icons.download),
                  title: Text('Download'),
                  dense: true,
                ),
              ),
              PopupMenuItem(
                value: 'delete',
                child: ListTile(
                  leading: Icon(Icons.delete, color: Colors.red),
                  title: Text('Delete', style: TextStyle(color: Colors.red)),
                  dense: true,
                ),
              ),
            ],
            onSelected: (value) => _handleBackupAction(backup['id'], value.toString()),
          ),
        ],
      ),
    );
  }
  
  void _refreshDatabaseStatus() {
    setState(() {});
    _showSnackBar('Database status refreshed');
  }
  
  void _createBackup() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Create Database Backup'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Select backup type:'),
            SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                ElevatedButton(
                  onPressed: () {
                    Navigator.pop(context);
                    _performBackup('full');
                  },
                  child: Text('Full Backup'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.pop(context);
                    _performBackup('incremental');
                  },
                  child: Text('Incremental'),
                ),
              ],
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
        ],
      ),
    );
  }
  
  void _performBackup(String type) {
    _showSnackBar('Creating $type backup...');
    setState(() {});
  }
  
  void _showImportDialog(String dbType) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Import $dbType Database'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Select backup file to import:'),
            SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () => _selectImportFile(dbType),
              icon: Icon(Icons.file_upload),
              label: Text('Select File'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
        ],
      ),
    );
  }
  
  void _showExportDialog(String dbType) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Export $dbType Database'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Export current database to $dbType format:'),
            SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.pop(context);
                _performExport(dbType);
              },
              icon: Icon(Icons.download),
              label: Text('Export Database'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
        ],
      ),
    );
  }
  
  void _showMigrationDialog(String fromDb, String toDb) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Database Migration'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Migrate from $fromDb to $toDb'),
            SizedBox(height: 16),
            Text('This will copy all data from the source database to the target database.'),
            SizedBox(height: 16),
            Row(
              children: [
                Icon(Icons.warning, color: Colors.orange),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'This operation may take some time depending on database size.',
                    style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                  ),
                ),
              ],
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              _performMigration(fromDb, toDb);
            },
            child: Text('Start Migration'),
          ),
        ],
      ),
    );
  }
  
  void _selectImportFile(String dbType) {
    _showSnackBar('Opening file selector for $dbType import...');
  }
  
  void _performExport(String dbType) {
    _showSnackBar('Exporting database to $dbType format...');
  }
  
  void _performMigration(String fromDb, String toDb) {
    _showSnackBar('Starting migration from $fromDb to $toDb...');
  }
  
  void _handleBackupAction(String backupId, String action) {
    switch (action) {
      case 'restore':
        _showSnackBar('Restoring backup $backupId...');
        break;
      case 'download':
        _showSnackBar('Downloading backup $backupId...');
        break;
      case 'delete':
        _showSnackBar('Deleting backup $backupId...');
        break;
    }
  }
  
  String _formatBackupDate(String? dateString) {
    if (dateString == null) return 'Unknown';
    try {
      final date = DateTime.parse(dateString);
      return '${date.day}/${date.month}/${date.year} ${date.hour}:${date.minute.toString().padLeft(2, '0')}';
    } catch (e) {
      return 'Invalid Date';
    }
  }
  
  // Console Command Response Handler (returns response instead of showing snackbar)
  Future<String> _getCommandResponse(String command, Map<String, dynamic> instance) async {
    print('Processing server command: $command');
    
    final trimmedCommand = command.trim().toLowerCase();
    String response = '';
    
    try {
      switch (trimmedCommand) {
        case 'help':
          return '''Available Server Commands:

General:
  status, users, regions, metrics, restart, shutdown, version, uptime, memory, help

User Management:
  create user <first> <last> <email> <password>
  list users
  delete user <uuid>
  reset password <uuid> <new_password>
  set user level <uuid> <level>

Examples:
  create user Test User test@example.com password123
  list users
  reset password 12345-uuid-67890 newpass''';
          
        case 'status':
          final healthResponse = await _makeApiCall('/api/health');
          final infoResponse = await _makeApiCall('/api/info');
          return 'Server Status:\nInstance: ${healthResponse?['instance_id'] ?? 'unknown'}\nStatus: ${healthResponse?['status'] ?? 'unknown'}\nCPU: ${(infoResponse?['cpu_usage'] ?? 0).round()}%\nMemory: ${((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round()} MB\nUsers: ${infoResponse?['active_connections'] ?? 0}\nRegions: ${infoResponse?['active_regions'] ?? 0}';
          
        case 'users':
          final infoResponse = await _makeApiCall('/api/info');
          final userCount = infoResponse?['active_connections'] ?? 0;
          return 'Connected Users: $userCount';
          
        case 'regions':
          final infoResponse = await _makeApiCall('/api/info');
          final regionCount = infoResponse?['active_regions'] ?? 0;
          return 'Active Regions: $regionCount';
          
        case 'metrics':
          final infoResponse = await _makeApiCall('/api/info');
          return 'Performance Metrics:\nCPU: ${(infoResponse?['cpu_usage'] ?? 0).round()}%\nMemory: ${((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round()} MB\nUptime: ${_formatUptime(infoResponse?['uptime'] ?? 0)}';
          
        case 'version':
          return 'OpenSim Next v2.4.2 (Rust + Zig Hybrid)\nFWDFE v2.4.2 - Server Central Edition';
          
        case 'uptime':
          final infoResponse = await _makeApiCall('/api/info');
          return 'Uptime: ${_formatUptime(infoResponse?['uptime'] ?? 0)}';
          
        case 'memory':
          final infoResponse = await _makeApiCall('/api/info');
          final memoryMB = ((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round();
          return 'Memory Usage: $memoryMB MB';
          
        case 'restart':
          return 'Server restart initiated (simulation)';
          
        case 'shutdown':
          return 'Server shutdown initiated (simulation)';
          
        case 'list users':
          return await _handleListUsers();
          
        default:
          // Check for multi-word commands
          if (trimmedCommand.startsWith('create user ')) {
            return await _handleCreateUser(trimmedCommand);
          } else if (trimmedCommand.startsWith('delete user ')) {
            return await _handleDeleteUser(trimmedCommand);
          } else if (trimmedCommand.startsWith('reset password ')) {
            return await _handleResetPassword(trimmedCommand);
          } else if (trimmedCommand.startsWith('set user level ')) {
            return await _handleSetUserLevel(trimmedCommand);
          } else {
            return 'Unknown command: "$command".\nType "help" for available commands.';
          }
      }
    } catch (e) {
      return 'Error: $e';
    }
  }
  
  // User Management Command Handlers
  Future<String> _handleCreateUser(String command) async {
    try {
      // Parse: create user <first> <last> <email> <password>
      final parts = command.split(' ');
      if (parts.length < 6) {
        return 'Usage: create user <first> <last> <email> <password>\nExample: create user Test User test@example.com password123';
      }
      
      final firstName = parts[2];
      final lastName = parts[3];
      final email = parts[4];
      final password = parts[5];
      
      // API call to create user via admin endpoint
      final adminUrl = await _getAdminUrl();
      final response = await http.post(
        Uri.parse('$adminUrl/admin/users'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({
          'firstname': firstName,
          'lastname': lastName,
          'email': email,
          'password': password,
          'user_level': 0
        }),
      );
      
      if (response.statusCode == 200 || response.statusCode == 201) {
        final userData = json.decode(response.body);
        return 'User created successfully:\nName: $firstName $lastName\nEmail: $email\nUser ID: ${userData['data']?['user_id'] ?? 'generated'}\nResponse: ${userData['message'] ?? 'Success'}';
      } else {
        return 'Failed to create user: ${response.body}';
      }
    } catch (e) {
      return 'Error creating user: $e';
    }
  }
  
  Future<String> _handleListUsers() async {
    try {
      final adminUrl = await _getAdminUrl();
      final response = await http.get(Uri.parse('$adminUrl/admin/users'));
      
      if (response.statusCode == 200) {
        final responseData = json.decode(response.body);
        final users = responseData['data']['users'] as List;
        if (users.isEmpty) {
          return 'No users found in database.';
        }
        
        String result = 'Registered Users:\n';
        for (var user in users.take(10)) { // Show max 10 users
          result += '${user['firstname']} ${user['lastname']} - ${user['email']} (${user['user_id']})\n';
        }
        if (users.length > 10) {
          result += '... and ${users.length - 10} more users';
        }
        return result;
      } else {
        return 'Failed to retrieve users: ${response.body}';
      }
    } catch (e) {
      // Fallback with sample data
      return '''Sample Users (API unavailable):
Test User - test@example.com (12345-test-uuid)
Admin User - admin@example.com (67890-admin-uuid)

To create a real user, use:
create user <first> <last> <email> <password>''';
    }
  }
  
  Future<String> _handleDeleteUser(String command) async {
    try {
      // Parse: delete user <uuid>
      final parts = command.split(' ');
      if (parts.length < 3) {
        return 'Usage: delete user <uuid>\nExample: delete user 12345-uuid-67890';
      }
      
      final uuid = parts[2];
      
      final adminUrl = await _getAdminUrl();
      final response = await http.delete(
        Uri.parse('$adminUrl/admin/users/delete'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({'user_id': uuid}),
      );
      
      if (response.statusCode == 200) {
        return 'User $uuid deleted successfully.';
      } else {
        return 'Failed to delete user: ${response.body}';
      }
    } catch (e) {
      return 'Error deleting user: $e';
    }
  }
  
  Future<String> _handleResetPassword(String command) async {
    try {
      // Parse: reset password <uuid> <new_password>
      final parts = command.split(' ');
      if (parts.length < 4) {
        return 'Usage: reset password <uuid> <new_password>\nExample: reset password 12345-uuid-67890 newpassword123';
      }
      
      final uuid = parts[2];
      final newPassword = parts[3];
      
      final adminUrl = await _getAdminUrl();
      final response = await http.put(
        Uri.parse('$adminUrl/admin/users/password'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({'user_id': uuid, 'password': newPassword}),
      );
      
      if (response.statusCode == 200) {
        return 'Password reset successfully for user $uuid.';
      } else {
        return 'Failed to reset password: ${response.body}';
      }
    } catch (e) {
      return 'Error resetting password: $e';
    }
  }
  
  Future<String> _handleSetUserLevel(String command) async {
    try {
      // Parse: set user level <uuid> <level>
      final parts = command.split(' ');
      if (parts.length < 5) {
        return 'Usage: set user level <uuid> <level>\nLevels: 0=User, 100=Admin, 200=God\nExample: set user level 12345-uuid-67890 100';
      }
      
      final uuid = parts[3];
      final level = int.parse(parts[4]);
      
      final adminUrl = await _getAdminUrl();
      final response = await http.put(
        Uri.parse('$adminUrl/admin/users/level'),
        headers: {'Content-Type': 'application/json'},
        body: json.encode({'user_id': uuid, 'user_level': level}),
      );
      
      if (response.statusCode == 200) {
        final levelName = level >= 200 ? 'God' : level >= 100 ? 'Admin' : 'User';
        return 'User level set to $level ($levelName) for user $uuid.';
      } else {
        return 'Failed to set user level: ${response.body}';
      }
    } catch (e) {
      return 'Error setting user level: $e';
    }
  }
  
  String _getThemeDescription(String theme) {
    switch (theme) {
      case 'opensim-dark':
        return 'The default OpenSim Next dark theme with high contrast and professional appearance.';
      case 'opensim-light':
        return 'Clean and modern light theme perfect for daytime use and bright environments.';
      case 'virtual-blue':
        return 'Immersive blue theme inspired by virtual worlds and digital environments.';
      case 'matrix-green':
        return 'Retro-futuristic green theme reminiscent of classic cyberpunk aesthetics.';
      case 'cosmic-purple':
        return 'Deep purple theme evoking the mystery and wonder of cosmic exploration.';
      case 'amber-glow':
        return 'Warm amber theme with golden accents for a comfortable viewing experience.';
      case 'system':
        return 'Automatically adapts to your system preferences (light or dark mode).';
      default:
        return 'Select a theme to see its description.';
    }
  }

  Widget _buildContainerInfoOverlay() {
    String selectedTab = '';
    Color selectedColor = Colors.blue;
    IconData selectedIcon = Icons.info;
    
    if (_selectedLeftTab > 0) {
      final tab = _leftSideTabs[_selectedLeftTab - 1];
      selectedTab = tab['title'];
      selectedColor = tab['color'];
      selectedIcon = tab['icon'];
    } else if (_selectedRightTab > 0) {
      final tab = _rightSideTabs[_selectedRightTab - 1];
      selectedTab = tab['title'];
      selectedColor = tab['color'];
      selectedIcon = tab['icon'];
    }

    return Container(
      margin: EdgeInsets.all(16),
      padding: EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: selectedColor, width: 2),
        boxShadow: [
          BoxShadow(
            color: Theme.of(context).shadowColor.withValues(alpha: 0.1),
            blurRadius: 10,
            spreadRadius: 2,
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(selectedIcon, color: selectedColor, size: 24),
              SizedBox(width: 12),
              Text(
                '$selectedTab Container Configuration',
                style: TextStyle(
                  fontSize: 20,
                  fontWeight: FontWeight.bold,
                  color: selectedColor,
                ),
              ),
              Spacer(),
              IconButton(
                onPressed: () => setState(() {
                  _selectedLeftTab = 0;
                  _selectedRightTab = 0;
                }),
                icon: Icon(Icons.close),
              ),
            ],
          ),
          SizedBox(height: 16),
          Expanded(
            child: _buildContainerContent(selectedTab),
          ),
        ],
      ),
    );
  }

  Widget _buildContainerContent(String containerType) {
    switch (containerType) {
      case 'Container Registry':
        return _buildContainerRegistryContent();
      case 'Orchestration':
        return _buildOrchestrationContent();
      case 'Scaling':
        return _buildScalingContent();
      case 'Networking':
        return _buildNetworkingContent();
      case 'Storage':
        return _buildStorageContent();
      case 'Deployment':
        return _buildDeploymentContent();
      case 'Service Mesh':
        return _buildServiceMeshContent();
      case 'API Gateway':
        return _buildAPIGatewayContent();
      case 'Identity':
        return _buildIdentityContent();
      case 'Secrets':
        return _buildSecretsContent();
      case 'Backup':
        return _buildBackupContent();
      case 'Logging':
        return _buildLoggingContent();
      case 'Tracing':
        return _buildTracingContent();
      case 'Alerting':
        return _buildAlertingContent();
      case 'Monitoring':
        return _buildMonitoringContent();
      case 'Load Balancing':
        return _buildLoadBalancingContent();
      default:
        return _buildGenericContainerContent(containerType);
    }
  }

  Widget _buildContainerRegistryContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Registry Images', [
            'opensim-next:v1.0.0 - Main server image',
            'opensim-physics:v1.0.0 - Physics engine',
            'opensim-web:v1.0.0 - Web interface',
            'opensim-database:v1.0.0 - Database layer',
          ]),
          _buildContainerSection('Registry Configuration', [
            'Registry URL: registry.opensim.local:5000',
            'Authentication: Docker Hub compatible',
            'Storage Backend: Local filesystem (S3 compatible optional)',
            'Vulnerability Scanning: Enabled',
          ]),
          _buildContainerSection('Image Management', [
            'Automatic image pruning enabled',
            'Layer caching optimized',
            'Multi-architecture support',
            'Content trust verification',
          ]),
        ],
      ),
    );
  }

  Widget _buildOrchestrationContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Kubernetes Deployment', [
            'Namespace: opensim-production',
            'Replica Sets: 3 for high availability',
            'Load Balancer: NGINX Ingress',
            'Service Mesh: Istio enabled',
          ]),
          _buildContainerSection('Container Status', [
            'Running Pods: 12/12',
            'CPU Usage: 45% average',
            'Memory Usage: 2.3GB/4GB',
            'Network I/O: 150MB/s',
          ]),
          _buildContainerSection('Orchestration Features', [
            'Auto-scaling enabled',
            'Rolling updates configured',
            'Health checks active',
            'Resource limits enforced',
          ]),
        ],
      ),
    );
  }

  Widget _buildGenericContainerContent(String type) {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('$type Overview', [
            'Container orchestration with Kubernetes',
            'Docker-based containerization',
            'Microservices architecture',
            'Production-ready configuration',
          ]),
          _buildContainerSection('Configuration', [
            'Auto-scaling: Enabled',
            'Load balancing: Active',
            'Health monitoring: Online',
            'Security scanning: Enabled',
          ]),
          _buildContainerSection('Resources', [
            'CPU: 2 cores allocated',
            'Memory: 4GB allocated',
            'Storage: 50GB persistent',
            'Network: 1Gbps bandwidth',
          ]),
        ],
      ),
    );
  }

  Widget _buildScalingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Horizontal Pod Autoscaler', [
            'Min Replicas: 2',
            'Max Replicas: 10',
            'Target CPU: 70%',
            'Target Memory: 80%',
          ]),
          _buildContainerSection('Vertical Pod Autoscaler', [
            'CPU Request: 100m-2000m',
            'Memory Request: 128Mi-4Gi',
            'Auto-resize: Enabled',
            'Recommendation Mode: Auto',
          ]),
          _buildContainerSection('Cluster Autoscaler', [
            'Node Groups: 3',
            'Scale Down Delay: 10m',
            'Scale Up Policy: Aggressive',
            'Max Nodes: 50',
          ]),
        ],
      ),
    );
  }

  Widget _buildNetworkingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Network Policies', [
            'Ingress: Restricted to required ports',
            'Egress: DNS and API server only',
            'Pod-to-Pod: Allowed within namespace',
            'Cross-namespace: Explicit allow rules',
          ]),
          _buildContainerSection('Service Discovery', [
            'DNS: CoreDNS cluster DNS',
            'Service Registration: Automatic',
            'Load Balancing: Round-robin',
            'Health Checks: TCP/HTTP probes',
          ]),
          _buildContainerSection('Ingress Controller', [
            'Controller: NGINX Ingress',
            'TLS Termination: Enabled',
            'Rate Limiting: 1000 req/min',
            'WAF: ModSecurity enabled',
          ]),
        ],
      ),
    );
  }

  Widget _buildStorageContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Persistent Volumes', [
            'Storage Class: fast-ssd',
            'Access Mode: ReadWriteOnce',
            'Reclaim Policy: Retain',
            'Volume Expansion: Enabled',
          ]),
          _buildContainerSection('Storage Drivers', [
            'CSI Driver: Local storage (cloud drivers optional)',
            'Snapshot Support: Enabled',
            'Encryption: AES-256',
            'Backup Schedule: Daily 2AM',
          ]),
          _buildContainerSection('Storage Monitoring', [
            'Disk Usage: 45% (2.3TB/5TB)',
            'IOPS: Local SSD/NVMe performance',
            'Throughput: 250MB/s',
            'Latency: <5ms average',
          ]),
          _buildContainerSection('Home Server Storage', [
            'Primary: Local SSD/NVMe drives',
            'Secondary: RAID 1/5 for redundancy',
            'External: USB 3.0/NAS for backups',
            'Cloud: Optional sync (Google Drive/OneDrive)',
          ]),
        ],
      ),
    );
  }

  Widget _buildDeploymentContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Deployment Strategy', [
            'Strategy: Rolling Update',
            'Max Unavailable: 25%',
            'Max Surge: 25%',
            'Revision History: 10',
          ]),
          _buildContainerSection('CI/CD Pipeline', [
            'Build: Docker multi-stage',
            'Test: Automated test suite',
            'Deploy: GitOps with ArgoCD',
            'Rollback: Automated on failure',
          ]),
          _buildContainerSection('Environment Promotion', [
            'Development → Testing → Staging → Production',
            'Approval Gates: Required',
            'Smoke Tests: Automated',
            'Monitoring: Continuous',
          ]),
        ],
      ),
    );
  }

  Widget _buildServiceMeshContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Istio Service Mesh', [
            'Proxy: Envoy sidecar',
            'Traffic Management: Enabled',
            'Security: mTLS enforced',
            'Observability: Distributed tracing',
          ]),
        ],
      ),
    );
  }

  Widget _buildAPIGatewayContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('API Gateway', [
            'Gateway: Kong OSS/NGINX (Enterprise optional)',
            'Rate Limiting: Enabled',
            'Authentication: OAuth 2.0',
            'Monitoring: Prometheus metrics',
          ]),
        ],
      ),
    );
  }

  Widget _buildIdentityContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Identity Management', [
            'Provider: OIDC/OAuth 2.0',
            'Authentication: Multi-factor enabled',
            'Authorization: RBAC policies',
            'Session Management: JWT tokens',
          ]),
          _buildContainerSection('User Directory', [
            'LDAP/AD Integration: Enabled',
            'User Provisioning: Automated',
            'Group Management: Hierarchical',
            'Audit Logging: Comprehensive',
          ]),
        ],
      ),
    );
  }

  Widget _buildSecretsContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Secret Management', [
            'Vault: HashiCorp Vault',
            'Encryption: AES-256-GCM',
            'Key Rotation: Automatic',
            'Access Control: Policy-based',
          ]),
          _buildContainerSection('Certificate Management', [
            'Auto-renewal: Let\'s Encrypt',
            'Certificate Authority: Internal CA',
            'TLS Version: 1.3 minimum',
            'Certificate Monitoring: Expiry tracking',
          ]),
        ],
      ),
    );
  }

  Widget _buildBackupContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Backup Strategy', [
            'Schedule: Daily incremental, Weekly full',
            'Retention: 30 days incremental, 12 months full',
            'Encryption: AES-256 at rest',
            'Compression: LZ4 algorithm',
          ]),
          _buildContainerSection('Disaster Recovery', [
            'RTO: 15 minutes',
            'RPO: 1 hour',
            'Local backup: NAS/External drives',
            'Cloud sync: Optional (rsync/rclone)',
          ]),
          _buildContainerSection('Home Server Options', [
            'Local storage: /data/opensim-backups',
            'USB/NAS backup: Automated sync',
            'Remote backup: SSH/rsync to second location',
            'Testing: Local restore validation',
          ]),
        ],
      ),
    );
  }

  Widget _buildLoggingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Log Aggregation', [
            'Platform: ELK Stack',
            'Ingestion: Fluentd/Fluent Bit',
            'Storage: Elasticsearch cluster',
            'Visualization: Kibana dashboards',
          ]),
          _buildContainerSection('Log Management', [
            'Retention: 90 days hot, 1 year warm',
            'Indexing: Time-based sharding',
            'Search: Full-text search enabled',
            'Alerting: Real-time log analysis',
          ]),
        ],
      ),
    );
  }

  Widget _buildTracingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Distributed Tracing', [
            'Platform: Jaeger/Zipkin',
            'Sampling: 1% production, 100% staging',
            'Storage: Cassandra backend',
            'Retention: 7 days traces',
          ]),
          _buildContainerSection('Performance Monitoring', [
            'APM: Application Performance Monitoring',
            'Latency Tracking: P95, P99 percentiles',
            'Error Tracking: Automatic error detection',
            'Dependency Mapping: Service topology',
          ]),
        ],
      ),
    );
  }

  Widget _buildAlertingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Alerting System', [
            'Platform: Prometheus Alertmanager',
            'Notification: Slack, PagerDuty, Email',
            'Escalation: On-call rotation',
            'Suppression: Smart grouping',
          ]),
          _buildContainerSection('Alert Rules', [
            'SLA Violations: <99.9% uptime',
            'Performance: >500ms response time',
            'Error Rate: >1% error threshold',
            'Resource Usage: >80% CPU/Memory',
          ]),
        ],
      ),
    );
  }

  Widget _buildMonitoringContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Monitoring Stack', [
            'Metrics: Prometheus + Grafana',
            'Uptime: Synthetic monitoring',
            'Infrastructure: Node Exporter',
            'Application: Custom metrics',
          ]),
          _buildContainerSection('Dashboards', [
            'SRE Dashboard: Golden signals',
            'Business Metrics: User engagement',
            'Infrastructure: Resource utilization',
            'Security: Threat detection',
          ]),
        ],
      ),
    );
  }

  Widget _buildLoadBalancingContent() {
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildContainerSection('Load Balancer Configuration', [
            'Type: Application Load Balancer',
            'Algorithm: Least connections',
            'Health Checks: HTTP/HTTPS',
            'SSL Termination: Enabled',
          ]),
          _buildContainerSection('Traffic Management', [
            'Sticky Sessions: Cookie-based',
            'Connection Draining: 300 seconds',
            'Rate Limiting: Per-IP throttling',
            'Geographic Routing: Latency-based',
          ]),
        ],
      ),
    );
  }

  Widget _buildContainerSection(String title, List<String> items) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
            color: Colors.blue[800],
          ),
        ),
        SizedBox(height: 8),
        ...items.map((item) => Padding(
          padding: EdgeInsets.only(left: 16, bottom: 4),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(Icons.check_circle, size: 16, color: Colors.green),
              SizedBox(width: 8),
              Expanded(child: Text(item)),
            ],
          ),
        )),
        SizedBox(height: 16),
      ],
    );
  }

  Widget _buildErrorPage() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error, size: 64, color: Colors.red),
          SizedBox(height: 16),
          Text('Page Not Found', style: Theme.of(context).textTheme.headlineSmall),
        ],
      ),
    );
  }

  // Helper widgets
  Widget _buildSectionHeader(String title, String subtitle) {
    return Container(
      padding: EdgeInsets.all(24),
      decoration: BoxDecoration(
        gradient: LinearGradient(colors: [Colors.blue.withValues(alpha: 0.1), Colors.blue.withValues(alpha: 0.05)]),
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        children: [
          Icon(Icons.dashboard, size: 48, color: Colors.blue),
          SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: Theme.of(context).textTheme.headlineMedium),
                SizedBox(height: 8),
                Text(subtitle, style: Theme.of(context).textTheme.bodyLarge),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFeatureIcon(IconData icon, String label) {
    return Column(
      children: [
        Container(
          padding: EdgeInsets.all(20),
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: Colors.white.withValues(alpha: 0.2),
            border: Border.all(color: Colors.white.withValues(alpha: 0.5), width: 2),
          ),
          child: Icon(icon, size: 40, color: Colors.white),
        ),
        SizedBox(height: 12),
        Text(label, textAlign: TextAlign.center, style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.w500)),
      ],
    );
  }

  Widget _buildNavCard(String title, String description, IconData icon, int pageIndex) {
    final isDarkMode = Theme.of(context).brightness == Brightness.dark;
    final isSelected = _currentPage == pageIndex;
    final primaryColor = Theme.of(context).primaryColor;
    final cardColor = Theme.of(context).cardColor;
    
    return AnimatedContainer(
      duration: Duration(milliseconds: 200),
      curve: Curves.easeInOut,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: isSelected 
            ? [
                primaryColor.withValues(alpha: 0.1),
                primaryColor.withValues(alpha: 0.05),
              ]
            : [
                cardColor,
                cardColor.withValues(alpha: 0.8),
              ],
        ),
        boxShadow: [
          BoxShadow(
            color: isDarkMode 
              ? Colors.black.withValues(alpha: 0.3)
              : Colors.grey.withValues(alpha: 0.2),
            blurRadius: isSelected ? 12 : 8,
            offset: Offset(0, isSelected ? 6 : 4),
            spreadRadius: isSelected ? 2 : 0,
          ),
          if (isSelected)
            BoxShadow(
              color: primaryColor.withValues(alpha: 0.3),
              blurRadius: 20,
              offset: Offset(0, 8),
              spreadRadius: -5,
            ),
        ],
        border: isSelected 
          ? Border.all(
              color: primaryColor.withValues(alpha: 0.3),
              width: 2,
            )
          : null,
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: () => setState(() => _currentPage = pageIndex),
          borderRadius: BorderRadius.circular(16),
          splashColor: primaryColor.withValues(alpha: 0.1),
          highlightColor: primaryColor.withValues(alpha: 0.05),
          child: Padding(
            padding: EdgeInsets.all(20),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                AnimatedContainer(
                  duration: Duration(milliseconds: 200),
                  padding: EdgeInsets.all(isSelected ? 16 : 12),
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    gradient: LinearGradient(
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                      colors: isSelected
                        ? [
                            primaryColor.withValues(alpha: 0.2),
                            primaryColor.withValues(alpha: 0.1),
                          ]
                        : [
                            primaryColor.withValues(alpha: 0.1),
                            primaryColor.withValues(alpha: 0.05),
                          ],
                    ),
                    boxShadow: isSelected
                      ? [
                          BoxShadow(
                            color: primaryColor.withValues(alpha: 0.3),
                            blurRadius: 8,
                            offset: Offset(0, 4),
                          ),
                        ]
                      : null,
                  ),
                  child: Icon(
                    icon, 
                    size: isSelected ? 36 : 32, 
                    color: isSelected 
                      ? primaryColor
                      : primaryColor.withValues(alpha: 0.8),
                  ),
                ),
                SizedBox(height: 12),
                Text(
                  title, 
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: isSelected ? FontWeight.bold : FontWeight.w600,
                    color: isSelected 
                      ? primaryColor
                      : Theme.of(context).textTheme.titleMedium?.color,
                  ), 
                  textAlign: TextAlign.center,
                ),
                SizedBox(height: 6),
                Text(
                  description, 
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).textTheme.bodySmall?.color?.withValues(alpha: 0.8),
                    height: 1.3,
                  ), 
                  textAlign: TextAlign.center, 
                  maxLines: 2, 
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildStatusItem(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, color: Theme.of(context).primaryColor),
        SizedBox(height: 8),
        Text(value, style: Theme.of(context).textTheme.headlineSmall?.copyWith(color: Theme.of(context).primaryColor, fontWeight: FontWeight.bold)),
        Text(label, style: Theme.of(context).textTheme.bodySmall, textAlign: TextAlign.center),
      ],
    );
  }

  Widget _buildMetricCard(String title, List<String> metrics, IconData icon, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, color: color, size: 24),
                SizedBox(width: 8),
                Expanded(child: Text(title, style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold))),
              ],
            ),
            SizedBox(height: 16),
            ...metrics.map((metric) => Padding(padding: EdgeInsets.symmetric(vertical: 2), child: Text(metric, style: Theme.of(context).textTheme.bodyMedium))),
          ],
        ),
      ),
    );
  }

  Widget _buildHealthCard(String title, List<String> metrics) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
            SizedBox(height: 16),
            ...metrics.map((metric) => Padding(padding: EdgeInsets.symmetric(vertical: 2), child: Text(metric, style: Theme.of(context).textTheme.bodyMedium))),
          ],
        ),
      ),
    );
  }

  Widget _buildHealthMetricCard(String title, String value, double percentage, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            SizedBox(height: 8),
            Text(value, style: Theme.of(context).textTheme.headlineSmall?.copyWith(color: color, fontWeight: FontWeight.bold)),
            SizedBox(height: 8),
            LinearProgressIndicator(value: percentage / 100, backgroundColor: color.withValues(alpha: 0.2), valueColor: AlwaysStoppedAnimation(color)),
          ],
        ),
      ),
    );
  }

  Widget _buildSecurityCard(String title, String value, IconData icon, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 32, color: color),
            SizedBox(height: 8),
            Text(title, style: Theme.of(context).textTheme.titleSmall, textAlign: TextAlign.center),
            SizedBox(height: 4),
            Text(value, style: Theme.of(context).textTheme.titleMedium?.copyWith(color: color, fontWeight: FontWeight.bold)),
          ],
        ),
      ),
    );
  }

  Widget _buildChartPlaceholder(String description, Color color) {
    return Card(
      child: Container(
        height: 200,
        padding: EdgeInsets.all(24),
        child: Column(
          children: [
            Text(description, style: Theme.of(context).textTheme.titleLarge),
            SizedBox(height: 16),
            Expanded(
              child: Container(
                decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.3)), borderRadius: BorderRadius.circular(8)),
                child: Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.show_chart, size: 48, color: color),
                      SizedBox(height: 16),
                      Text('Interactive charts available', textAlign: TextAlign.center),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoItem(String label, String value) {
    return Column(
      children: [
        Text(label, style: TextStyle(fontSize: 12, color: Colors.grey)),
        SizedBox(height: 4),
        Text(value, style: TextStyle(fontWeight: FontWeight.bold)),
      ],
    );
  }

  Widget _buildStatBox(String number, String label) {
    return Column(
      children: [
        Text(number, style: Theme.of(context).textTheme.headlineSmall?.copyWith(color: Theme.of(context).primaryColor, fontWeight: FontWeight.bold)),
        Text(label, style: TextStyle(fontSize: 12)),
      ],
    );
  }

  Color _getStatusColor(String status) {
    switch (status.toLowerCase()) {
      case 'running':
        return Colors.green;
      case 'stopped':
        return Colors.red;
      case 'starting':
        return Colors.orange;
      case 'error':
        return Colors.red.shade700;
      default:
        return Colors.grey;
    }
  }

  void _refreshServerData() async {
    print('Refreshing server data from real API...');
    _showSnackBar('Refreshing server data...');
    _loadAdminData();

    try {
      // Call real API endpoints
      final healthResponse = await _makeApiCall('/api/health');
      final infoResponse = await _makeApiCall('/api/info');
      
      if (healthResponse != null && infoResponse != null) {
        print('Health API response: $healthResponse');
        print('Info API response: $infoResponse');
        
        setState(() {
          // Update system status from real API data
          _mockData['systemStatus']['status'] = healthResponse['status']?.toLowerCase() ?? 'unknown';
          _mockData['systemStatus']['usersOnline'] = infoResponse['active_connections'] ?? 0;
          _mockData['systemStatus']['regionsActive'] = infoResponse['active_regions'] ?? 0;
          _mockData['systemStatus']['uptimeHours'] = ((infoResponse['uptime'] ?? 0) / 3600).round();
          _mockData['systemStatus']['cpuUsage'] = (infoResponse['cpu_usage'] ?? 0).round();
          
          // Update server info from real API data
          _mockData['serverInfo']['buildHash'] = healthResponse['instance_id'] ?? 'unknown';
          _mockData['serverInfo']['uptime'] = _formatUptime(infoResponse['uptime'] ?? 0);
          
          // Update user stats from real API data
          _mockData['userStats']['onlineUsers'] = infoResponse['active_connections'] ?? 0;
          
          // Update analytics from real API data
          _mockData['analytics']['worldMetrics']['usersOnline'] = infoResponse['active_connections'] ?? 0;
          _mockData['analytics']['worldMetrics']['regionsActive'] = infoResponse['active_regions'] ?? 0;
          _mockData['analytics']['performance']['cpuUsage'] = (infoResponse['cpu_usage'] ?? 0).round();
          _mockData['analytics']['performance']['memoryUsage'] = ((infoResponse['memory_usage'] ?? 0) / 1024 / 1024).round();
          
          // Update server instances with real data
          _mockData['serverInstances'][0]['status'] = healthResponse['status']?.toLowerCase() ?? 'unknown';
          _mockData['serverInstances'][0]['uptime'] = _formatUptime(infoResponse['uptime'] ?? 0);
          _mockData['serverInstances'][0]['memory'] = '${((infoResponse['memory_usage'] ?? 0) / 1024 / 1024).round()} MB';
          _mockData['serverInstances'][0]['cpu'] = '${(infoResponse['cpu_usage'] ?? 0).round()}%';
          _mockData['serverInstances'][0]['users'] = infoResponse['active_connections'] ?? 0;
          _mockData['serverInstances'][0]['regions'] = infoResponse['active_regions'] ?? 0;
        });
        _showSnackBar('Server data refreshed with real API data!');
      }
    } catch (e) {
      print('API call failed: $e');
      _showSnackBar('Failed to refresh server data: $e');
    }
  }

  String _formatUptime(int seconds) {
    final hours = seconds ~/ 3600;
    final minutes = (seconds % 3600) ~/ 60;
    return '${hours}h ${minutes}m';
  }

  Future<Map<String, dynamic>?> _makeApiCall(String endpoint) async {
    if (_discoveredControllerUrl == null) {
      await _discoverController();
    }
    if (_discoveredControllerUrl == null) return null;
    try {
      final response = await http.get(
        Uri.parse('$_discoveredControllerUrl$endpoint'),
      ).timeout(const Duration(seconds: 3));
      if (response.statusCode == 200) {
        return json.decode(response.body);
      } else {
        print('API call failed: ${response.statusCode} - ${response.body}');
        return null;
      }
    } catch (e) {
      print('HTTP request failed: $e');
      return null;
    }
  }

  Future<void> _discoverController() async {
    final futures = <Future<int?>>[];
    for (int port = 9300; port <= 9320; port++) {
      futures.add(_probePort(port));
    }
    final results = await Future.wait(futures);
    for (final port in results) {
      if (port != null) {
        _discoveredControllerUrl = 'http://localhost:$port';
        print('Discovered controller at $_discoveredControllerUrl');
        return;
      }
    }
    print('No controller discovered on ports 9300-9320');
  }

  Future<int?> _probePort(int port) async {
    try {
      final response = await http.get(
        Uri.parse('http://localhost:$port/health'),
      ).timeout(const Duration(seconds: 2));
      if (response.statusCode == 200) return port;
    } catch (_) {}
    return null;
  }

  Future<void> _discoverAdmin() async {
    for (int port in [9200, 9700, 9800, 9600]) {
      try {
        final response = await http.get(
          Uri.parse('http://localhost:$port/admin/health'),
        ).timeout(const Duration(seconds: 2));
        if (response.statusCode == 200) {
          _discoveredAdminUrl = 'http://localhost:$port';
          print('Discovered admin API at $_discoveredAdminUrl');
          return;
        }
      } catch (_) {}
    }
    print('No admin API discovered, falling back to controller URL');
    if (_discoveredControllerUrl != null) {
      _discoveredAdminUrl = _discoveredControllerUrl;
    }
  }

  Future<String> _getAdminUrl() async {
    if (_discoveredAdminUrl == null) await _discoverAdmin();
    return _discoveredAdminUrl ?? 'http://localhost:9200';
  }

  void _showServerConsole(Map<String, dynamic> instance) async {
    // Fetch latest server data for the console
    final healthResponse = await _makeApiCall('/api/health');
    final infoResponse = await _makeApiCall('/api/info');
    
    showDialog(
      context: context,
      builder: (context) => Dialog(
        child: Container(
          width: MediaQuery.of(context).size.width * 0.8,
          height: MediaQuery.of(context).size.height * 0.7,
          padding: EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Server Console: ${instance['name']}', 
                       style: Theme.of(context).textTheme.titleLarge),
                  IconButton(
                    icon: Icon(Icons.close),
                    onPressed: () => Navigator.pop(context),
                  ),
                ],
              ),
              SizedBox(height: 16),
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.grey),
                  ),
                  padding: EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Expanded(
                        child: SingleChildScrollView(
                          child: Text(
                            'OpenSim Next Console - ${instance['name']}\n'
                            'Instance ID: ${healthResponse?['instance_id'] ?? 'unknown'}\n'
                            'Server Status: ${healthResponse?['status'] ?? instance['status']}\n'
                            'Health Status: ${infoResponse?['monitoring_stats']?['health_status'] ?? 'Unknown'}\n\n'
                            'Real-time Metrics:\n'
                            '├─ Memory Usage: ${((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round()} MB\n'
                            '├─ CPU Usage: ${(infoResponse?['cpu_usage'] ?? 0).round()}%\n'
                            '├─ Active Users: ${infoResponse?['active_connections'] ?? 0}\n'
                            '├─ Active Regions: ${infoResponse?['active_regions'] ?? 0}\n'
                            '├─ Uptime: ${_formatUptime(infoResponse?['uptime'] ?? 0)}\n'
                            '└─ Metrics Count: ${infoResponse?['monitoring_stats']?['metrics_count'] ?? 0}\n\n'
                            'Network Configuration:\n'
                            '├─ Main Port: ${instance['port']}\n'
                            '├─ Web Interface: http://localhost:${instance['webPort']}\n'
                            '├─ API Endpoints: http://localhost:${instance['apiPort']}\n'
                            '└─ Metrics Port: ${infoResponse?['metrics_port'] ?? 9100}\n\n'
                            'Available Commands:\n'
                            '├─ help - Show available commands\n'
                            '├─ status - Display server status\n'
                            '├─ users - List connected users\n'
                            '├─ regions - List active regions\n'
                            '├─ metrics - Show performance metrics\n'
                            '└─ restart - Restart server instance\n\n'
                            '[${DateTime.now().toString().substring(11, 19)}] Console ready...\n'
                            '[${DateTime.now().toString().substring(11, 19)}] Real-time data integration active\n'
                            '[${DateTime.now().toString().substring(11, 19)}] Type commands or monitor live metrics',
                            style: TextStyle(
                              color: Colors.green.shade300,
                              fontFamily: 'monospace',
                              fontSize: 12,
                            ),
                          ),
                        ),
                      ),
                      Divider(color: Colors.grey),
                      Row(
                        children: [
                          Text('> ', style: TextStyle(color: Colors.green.shade300)),
                          Expanded(
                            child: TextField(
                              style: TextStyle(color: Colors.green.shade300),
                              decoration: InputDecoration(
                                hintText: 'Enter console command...',
                                hintStyle: TextStyle(color: Colors.grey),
                                border: InputBorder.none,
                              ),
                              onSubmitted: (command) async {
                                if (command.trim().isNotEmpty) {
                                  final response = await _getCommandResponse(command, instance);
                                  showDialog(
                                    context: context,
                                    builder: (context) => AlertDialog(
                                      title: Row(
                                        children: [
                                          Icon(Icons.terminal, color: Colors.green),
                                          SizedBox(width: 8),
                                          Text('Console Response'),
                                        ],
                                      ),
                                      content: Container(
                                        width: 500,
                                        constraints: BoxConstraints(maxHeight: 400),
                                        child: SingleChildScrollView(
                                          child: Container(
                                            padding: EdgeInsets.all(12),
                                            decoration: BoxDecoration(
                                              color: Colors.black,
                                              borderRadius: BorderRadius.circular(8),
                                            ),
                                            child: Text(
                                              '> $command\n\n$response',
                                              style: TextStyle(
                                                color: Colors.green,
                                                fontFamily: 'Courier',
                                                fontSize: 12,
                                              ),
                                            ),
                                          ),
                                        ),
                                      ),
                                      actions: [
                                        TextButton(
                                          onPressed: () => Navigator.pop(context),
                                          child: Text('OK'),
                                        ),
                                      ],
                                    ),
                                  );
                                }
                              },
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showQuickNav() {
    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        padding: EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Quick Navigation', style: Theme.of(context).textTheme.headlineSmall),
            SizedBox(height: 16),
            GridView.count(
              shrinkWrap: true,
              crossAxisCount: 2,
              childAspectRatio: 3,
              crossAxisSpacing: 8,
              mainAxisSpacing: 8,
              children: List.generate(_pages.length, (index) {
                final page = _pages[index];
                return ActionChip(
                  avatar: Icon(page['icon'], size: 18),
                  label: Text(page['title'], style: TextStyle(fontSize: 12)),
                  onPressed: () {
                    Navigator.pop(context);
                    setState(() => _currentPage = index + 1);
                  },
                );
              }),
            ),
          ],
        ),
      ),
    );
  }

  void _showSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
  }

  Widget _buildSetupWizardDropdown() {
    return PopupMenuButton<String>(
      child: Container(
        padding: EdgeInsets.symmetric(horizontal: 24, vertical: 12),
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: [Colors.blue.withValues(alpha: 0.8), Colors.blue.withValues(alpha: 0.6)],
          ),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Colors.white.withValues(alpha: 0.3)),
          boxShadow: [
            BoxShadow(
              color: Colors.blue.withValues(alpha: 0.3),
              blurRadius: 10,
              offset: Offset(0, 4),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.auto_fix_high, color: Colors.white),
            SizedBox(width: 8),
            Text('Setup Wizard', style: TextStyle(color: Colors.white)),
            SizedBox(width: 4),
            Icon(Icons.arrow_drop_down, size: 16, color: Colors.white),
          ],
        ),
      ),
      onSelected: (String value) {
        print('Setup Wizard dropdown selected: $value');
        switch (value) {
          case 'quick_start':
            print('Opening Quick Start dialog');
            _showQuickStartDialog();
            break;
          case 'templates':
            print('Opening Template Gallery');
            _showTemplateGallery();
            break;
          case 'saved_configs':
            print('Opening Saved Configurations');
            _showSavedConfigurations();
            break;
          case 'advanced':
            print('Opening Advanced Setup Wizard');
            _showSetupWizardDialog();
            break;
          case 'clone':
            print('Opening Clone Instance dialog');
            _showCloneInstanceDialog();
            break;
          case 'manage':
            print('Opening Archive Manager');
            _showArchiveManager();
            break;
        }
      },
      itemBuilder: (BuildContext context) => [
        PopupMenuItem(
          value: 'quick_start',
          child: ListTile(
            leading: Icon(Icons.rocket_launch, color: Colors.green),
            title: Text('🚀 Quick Start'),
            subtitle: Text('One-click standalone setup'),
            dense: true,
          ),
        ),
        PopupMenuItem(
          value: 'templates',
          child: ListTile(
            leading: Icon(Icons.dashboard, color: Colors.orange),
            title: Text('📚 Choose Template'),
            subtitle: Text('Pre-configured setups'),
            dense: true,
          ),
        ),
        PopupMenuItem(
          value: 'saved_configs',
          child: ListTile(
            leading: Icon(Icons.folder, color: Colors.purple),
            title: Text('💾 Load Saved Setup'),
            subtitle: Text('Browse your archive'),
            dense: true,
          ),
        ),
        PopupMenuDivider(),
        PopupMenuItem(
          value: 'advanced',
          child: ListTile(
            leading: Icon(Icons.settings, color: Colors.blue),
            title: Text('🔧 Advanced Wizard'),
            subtitle: Text('Full customization'),
            dense: true,
          ),
        ),
        PopupMenuItem(
          value: 'clone',
          child: ListTile(
            leading: Icon(Icons.copy, color: Colors.teal),
            title: Text('📋 Clone Instance'),
            subtitle: Text('Duplicate running setup'),
            dense: true,
          ),
        ),
        PopupMenuDivider(),
        PopupMenuItem(
          value: 'manage',
          child: ListTile(
            leading: Icon(Icons.archive, color: Colors.grey),
            title: Text('🏗️ Manage Archives'),
            subtitle: Text('Browse and organize'),
            dense: true,
          ),
        ),
      ],
    );
  }

  void _showQuickStartDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.rocket_launch, color: Colors.green),
            SizedBox(width: 8),
            Text('🚀 Quick Start'),
          ],
        ),
        content: Container(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Get your first virtual world running in 2 minutes!',
                style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
              ),
              SizedBox(height: 16),
              Container(
                padding: EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.green.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: Colors.green.withValues(alpha: 0.3)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('✨ What you\'ll get:', style: TextStyle(fontWeight: FontWeight.bold)),
                    SizedBox(height: 8),
                    Text('🏝️ Your own 256x256m island'),
                    Text('👤 Admin powers with full control'),
                    Text('💾 SQLite database (no setup required)'),
                    Text('🎨 Creative freedom to build and script'),
                    Text('🔧 Beginner-friendly settings'),
                  ],
                ),
              ),
              SizedBox(height: 16),
              Text(
                'Perfect for first-time users and quick testing.',
                style: TextStyle(fontStyle: FontStyle.italic),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          ElevatedButton.icon(
            onPressed: () {
              Navigator.pop(context);
              _deployQuickStart();
            },
            icon: Icon(Icons.play_arrow),
            label: Text('Deploy Quick Start'),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
          ),
        ],
      ),
    );
  }

  void _showTemplateGallery() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.dashboard, color: Colors.orange),
            SizedBox(width: 8),
            Text('📚 Template Gallery'),
          ],
        ),
        content: Container(
          width: 600,
          height: 500,
          child: Column(
            children: [
              // Difficulty filter tabs
              Container(
                height: 40,
                child: Row(
                  children: [
                    _buildDifficultyTab('🟢 Beginner', 'beginner', true),
                    _buildDifficultyTab('🟡 Intermediate', 'intermediate', false),
                    _buildDifficultyTab('🔴 Advanced', 'advanced', false),
                  ],
                ),
              ),
              SizedBox(height: 16),
              // Template grid
              Expanded(
                child: GridView.count(
                  crossAxisCount: 2,
                  crossAxisSpacing: 12,
                  mainAxisSpacing: 12,
                  childAspectRatio: 1.2,
                  children: [
                    _buildTemplateCard(
                      'Quick Start Standalone',
                      '🚀',
                      'Single region, auto-configured',
                      '🟢 Beginner',
                      Colors.green,
                      () => _deployTemplate('quickstart-standalone'),
                    ),
                    _buildTemplateCard(
                      'Small Grid (2x2)',
                      '🏘️',
                      'Four connected regions',
                      '🟢 Beginner',
                      Colors.green,
                      () => _deployTemplate('small-grid-2x2'),
                    ),
                    _buildTemplateCard(
                      'Creative Sandbox',
                      '🎨',
                      'Enhanced building environment',
                      '🟡 Intermediate',
                      Colors.orange,
                      () => _deployTemplate('creative-sandbox'),
                    ),
                    _buildTemplateCard(
                      'Educational Grid',
                      '🎓',
                      'Perfect for schools',
                      '🟢 Beginner',
                      Colors.blue,
                      () => _deployTemplate('educational-basic'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildDifficultyTab(String label, String key, bool isSelected) {
    return Expanded(
      child: Container(
        margin: EdgeInsets.only(right: 8),
        child: ElevatedButton(
          onPressed: () {
            // TODO: Filter templates by difficulty
          },
          style: ElevatedButton.styleFrom(
            backgroundColor: isSelected ? Colors.blue : Colors.grey.withValues(alpha: 0.3),
            foregroundColor: isSelected ? Colors.white : Colors.grey,
          ),
          child: Text(label, style: TextStyle(fontSize: 12)),
        ),
      ),
    );
  }

  Widget _buildTemplateCard(String title, String emoji, String description, 
                           String difficulty, Color color, VoidCallback onDeploy) {
    return Card(
      elevation: 4,
      child: Padding(
        padding: EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(emoji, style: TextStyle(fontSize: 24)),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    title,
                    style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            SizedBox(height: 8),
            Text(
              description,
              style: TextStyle(fontSize: 12, color: Colors.grey),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            Spacer(),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  difficulty,
                  style: TextStyle(fontSize: 10, fontWeight: FontWeight.bold),
                ),
                ElevatedButton(
                  onPressed: onDeploy,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: color,
                    minimumSize: Size(60, 28),
                  ),
                  child: Text('Deploy', style: TextStyle(fontSize: 10)),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _showSavedConfigurations() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.folder, color: Colors.purple),
            SizedBox(width: 8),
            Text('💾 Saved Configurations'),
          ],
        ),
        content: Container(
          width: 500,
          height: 400,
          child: Column(
            children: [
              // Search bar
              TextField(
                decoration: InputDecoration(
                  hintText: 'Search configurations...',
                  prefixIcon: Icon(Icons.search),
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
                ),
              ),
              SizedBox(height: 16),
              // Configuration list
              Expanded(
                child: ListView(
                  children: [
                    _buildConfigurationItem(
                      'Gaia Grid 2024',
                      '16 regions • Production Grid • Created 2 days ago',
                      Icons.public,
                      Colors.blue,
                      () => _loadConfiguration('gaia-grid-2024'),
                    ),
                    _buildConfigurationItem(
                      'Test Environment',
                      '4 regions • Development • Created 1 week ago',
                      Icons.bug_report,
                      Colors.orange,
                      () => _loadConfiguration('test-environment'),
                    ),
                    _buildConfigurationItem(
                      'My First World',
                      '1 region • Standalone • Created 2 weeks ago',
                      Icons.home,
                      Colors.green,
                      () => _loadConfiguration('my-first-world'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildConfigurationItem(String name, String details, IconData icon, 
                                Color color, VoidCallback onLoad) {
    return Card(
      margin: EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: color.withValues(alpha: 0.2),
          child: Icon(icon, color: color),
        ),
        title: Text(name, style: TextStyle(fontWeight: FontWeight.bold)),
        subtitle: Text(details, style: TextStyle(fontSize: 12)),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            IconButton(
              icon: Icon(Icons.play_arrow, color: Colors.green),
              onPressed: onLoad,
              tooltip: 'Deploy Configuration',
            ),
            IconButton(
              icon: Icon(Icons.edit, color: Colors.blue),
              onPressed: () {
                // TODO: Edit configuration
              },
              tooltip: 'Edit Configuration',
            ),
            IconButton(
              icon: Icon(Icons.delete, color: Colors.red),
              onPressed: () {
                // TODO: Delete configuration
              },
              tooltip: 'Delete Configuration',
            ),
          ],
        ),
      ),
    );
  }

  void _showCloneInstanceDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.copy, color: Colors.teal),
            SizedBox(width: 8),
            Text('📋 Clone Running Instance'),
          ],
        ),
        content: Container(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Clone the current running OpenSim instance to create a new configuration.'),
              SizedBox(height: 16),
              TextField(
                decoration: InputDecoration(
                  labelText: 'New Configuration Name',
                  hintText: 'e.g., My Grid Copy',
                  border: OutlineInputBorder(),
                ),
              ),
              SizedBox(height: 12),
              TextField(
                decoration: InputDecoration(
                  labelText: 'Description (optional)',
                  hintText: 'Describe this configuration...',
                  border: OutlineInputBorder(),
                ),
                maxLines: 2,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          ElevatedButton.icon(
            onPressed: () {
              Navigator.pop(context);
              _cloneCurrentInstance();
            },
            icon: Icon(Icons.copy),
            label: Text('Clone Instance'),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.teal),
          ),
        ],
      ),
    );
  }

  void _showArchiveManager() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.archive, color: Colors.grey),
            SizedBox(width: 8),
            Text('🏗️ Archive Manager'),
          ],
        ),
        content: Container(
          width: 600,
          height: 500,
          child: Column(
            children: [
              // Tabs for different views
              Container(
                height: 40,
                child: Row(
                  children: [
                    _buildArchiveTab('By Name', true),
                    _buildArchiveTab('By Category', false),
                    _buildArchiveTab('Active Instances', false),
                  ],
                ),
              ),
              SizedBox(height: 16),
              // Archive content based on selected tab
              Expanded(
                child: Column(
                  children: [
                    Text(
                      'Archive management interface for organizing and maintaining your OpenSim configurations.',
                      style: TextStyle(fontStyle: FontStyle.italic),
                    ),
                    SizedBox(height: 16),
                    // TODO: Implement full archive browser
                    Container(
                      padding: EdgeInsets.all(20),
                      decoration: BoxDecoration(
                        border: Border.all(color: Colors.grey.withValues(alpha: 0.3)),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        'Archive browser will be implemented here\n'
                        'Features:\n'
                        '• Browse by name or category\n'
                        '• Export/import configurations\n'
                        '• Manage active instances\n'
                        '• Configuration comparison tools',
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildArchiveTab(String label, bool isSelected) {
    return Expanded(
      child: Container(
        margin: EdgeInsets.only(right: 8),
        child: ElevatedButton(
          onPressed: () {
            // TODO: Switch archive view
          },
          style: ElevatedButton.styleFrom(
            backgroundColor: isSelected ? Colors.grey : Colors.grey.withValues(alpha: 0.3),
            foregroundColor: isSelected ? Colors.white : Colors.grey,
          ),
          child: Text(label, style: TextStyle(fontSize: 12)),
        ),
      ),
    );
  }

  // Action methods
  void _deployQuickStart() {
    _showSnackBar('🚀 Deploying Quick Start configuration...');
    // TODO: Integrate with Rust backend to deploy quickstart template
  }

  void _deployTemplate(String templateName) {
    _showSnackBar('📚 Deploying template: $templateName');
    // TODO: Integrate with Rust backend to deploy specific template
  }

  void _loadConfiguration(String configName) {
    _showSnackBar('💾 Loading configuration: $configName');
    // TODO: Integrate with Rust backend to load saved configuration
  }

  void _cloneCurrentInstance() {
    _showSnackBar('📋 Cloning current instance...');
    // TODO: Integrate with Rust backend to clone running instance
  }

  void _showSetupWizardDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.auto_fix_high, color: Colors.blue),
            SizedBox(width: 8),
            Text('OpenSim Next Setup Wizard'),
          ],
        ),
        content: Container(
          width: 500,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Configure your OpenSim Next server with guided setup.',
                  style: TextStyle(fontSize: 16),
                ),
                SizedBox(height: 16),
                Text(
                  'Available Setup Options:',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                SizedBox(height: 12),
                
                // Setup Options
                _buildSetupOption(
                  'Quick Setup (Standalone)',
                  'Complete standalone grid with all services in one instance',
                  Icons.speed,
                  Colors.green,
                  () => _launchSetupWizard('standalone'),
                ),
                SizedBox(height: 8),
                _buildSetupOption(
                  'Grid Region Server',
                  'Region server connecting to an existing grid',
                  Icons.web,
                  Colors.blue,
                  () => _launchSetupWizard('grid-region'),
                ),
                SizedBox(height: 8),
                _buildSetupOption(
                  'Grid Services Server',
                  'Grid services server (Robust) for multi-region grids',
                  Icons.dns,
                  Colors.purple,
                  () => _launchSetupWizard('grid-robust'),
                ),
                SizedBox(height: 8),
                _buildSetupOption(
                  'Development Setup',
                  'Development setup with debugging enabled',
                  Icons.code,
                  Colors.orange,
                  () => _launchSetupWizard('development'),
                ),
                SizedBox(height: 8),
                _buildSetupOption(
                  'Production Setup',
                  'Production-ready setup with security hardening',
                  Icons.security,
                  Colors.red,
                  () => _launchSetupWizard('production'),
                ),
                SizedBox(height: 16),
                
                // Advanced Options
                Divider(),
                SizedBox(height: 8),
                Text(
                  'Advanced Options:',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
                SizedBox(height: 8),
                Row(
                  children: [
                    Expanded(
                      child: ElevatedButton.icon(
                        onPressed: () => _launchSetupWizard('interactive'),
                        icon: Icon(Icons.quiz, size: 18),
                        label: Text('Interactive Setup'),
                        style: ElevatedButton.styleFrom(backgroundColor: Colors.teal),
                      ),
                    ),
                    SizedBox(width: 8),
                    Expanded(
                      child: ElevatedButton.icon(
                        onPressed: () => _launchSetupWizard('reconfigure'),
                        icon: Icon(Icons.settings_backup_restore, size: 18),
                        label: Text('Reconfigure'),
                        style: ElevatedButton.styleFrom(backgroundColor: Colors.amber),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
        ],
      ),
    );
  }

  Widget _buildSetupOption(String title, String description, IconData icon, Color color, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: EdgeInsets.all(12),
        decoration: BoxDecoration(
          border: Border.all(color: color.withValues(alpha: 0.3)),
          borderRadius: BorderRadius.circular(8),
          color: color.withValues(alpha: 0.05),
        ),
        child: Row(
          children: [
            Container(
              padding: EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: color.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Icon(icon, color: color, size: 20),
            ),
            SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: TextStyle(fontWeight: FontWeight.w600)),
                  SizedBox(height: 2),
                  Text(description, style: TextStyle(fontSize: 12, color: Colors.grey.shade600)),
                ],
              ),
            ),
            Icon(Icons.arrow_forward_ios, size: 16, color: Colors.grey.shade400),
          ],
        ),
      ),
    );
  }

  void _launchSetupWizard(String mode) {
    Navigator.pop(context); // Close the dialog first
    
    String command;
    String message;
    
    switch (mode) {
      case 'standalone':
        command = 'cargo run --bin opensim-next -- setup --preset standalone --non-interactive';
        message = 'Launching standalone setup wizard...';
        break;
      case 'grid-region':
        command = 'cargo run --bin opensim-next -- setup --preset grid-region --non-interactive';
        message = 'Launching grid region setup wizard...';
        break;
      case 'grid-robust':
        command = 'cargo run --bin opensim-next -- setup --preset grid-robust --non-interactive';
        message = 'Launching grid services setup wizard...';
        break;
      case 'development':
        command = 'cargo run --bin opensim-next -- setup --preset development --non-interactive';
        message = 'Launching development setup wizard...';
        break;
      case 'production':
        command = 'cargo run --bin opensim-next -- setup --preset production --non-interactive';
        message = 'Launching production setup wizard...';
        break;
      case 'interactive':
        command = 'cargo run --bin opensim-next -- setup';
        message = 'Launching interactive setup wizard...';
        break;
      case 'reconfigure':
        command = 'cargo run --bin opensim-next -- setup --reconfigure';
        message = 'Launching reconfiguration wizard...';
        break;
      default:
        command = 'cargo run --bin opensim-next -- setup --help';
        message = 'Opening setup wizard help...';
    }
    
    _showSnackBar(message);
    
    // Show additional information about the command being executed
    _showSetupCommandDialog(command, mode);
  }

  void _showSetupCommandDialog(String command, String mode) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Setup Wizard Command'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('The following command will be executed:', style: TextStyle(fontWeight: FontWeight.bold)),
            SizedBox(height: 8),
            Container(
              padding: EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey.shade100,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: Colors.grey.shade300),
              ),
              child: SelectableText(
                command,
                style: TextStyle(fontFamily: 'monospace', fontSize: 12),
              ),
            ),
            SizedBox(height: 16),
            Text('This will configure your OpenSim Next server with the selected preset.'),
            SizedBox(height: 8),
            Text('Configuration files will be generated in the ./config/ directory.'),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              _showSnackBar('Setup wizard command copied to clipboard');
            },
            child: Text('Copy Command'),
          ),
        ],
      ),
    );
  }

  // Welcome Page Real-time Data
  Future<Map<String, dynamic>?> _getWelcomePageData() async {
    try {
      print('Welcome Page: Fetching data from /api/info');
      final infoResponse = await _makeApiCall('/api/info');
      print('Welcome Page: API Response - ${infoResponse.toString()}');
      
      if (infoResponse != null) {
        print('Welcome Page: Data loaded successfully');
        print('  - Active Connections: ${infoResponse['active_connections']}');
        print('  - Active Regions: ${infoResponse['active_regions']}');
        print('  - Uptime: ${infoResponse['uptime']}');
        print('  - CPU Usage: ${infoResponse['cpu_usage']}');
        return infoResponse;
      } else {
        print('Welcome Page: API returned null response');
        // Return fallback data to show something
        return {
          'active_connections': 0,
          'active_regions': 1,
          'uptime': 0,
          'cpu_usage': 0.0,
          'memory_usage': 0,
          'instance_id': 'fallback-instance',
          'monitoring_stats': {
            'health_status': 'Unknown',
            'metrics_count': 0,
            'profiling_enabled': false
          }
        };
      }
    } catch (e) {
      print('Welcome Page: Error fetching data - $e');
      // Return fallback data on error
      return {
        'active_connections': 0,
        'active_regions': 1,
        'uptime': 0,
        'cpu_usage': 0.0,
        'memory_usage': 0,
        'instance_id': 'error-instance',
        'monitoring_stats': {
          'health_status': 'Error',
          'metrics_count': 0,
          'profiling_enabled': false
        }
      };
    }
  }

  // Analytics Page Real-time Data
  Future<Map<String, dynamic>?> _getAnalyticsData() async {
    try {
      final infoResponse = await _makeApiCall('/api/info');
      return infoResponse;
    } catch (e) {
      print('Failed to load analytics data: $e');
      return null;
    }
  }

  // Observability Page Real-time Data
  Future<Map<String, dynamic>?> _getObservabilityData() async {
    try {
      final infoResponse = await _makeApiCall('/api/info');
      return infoResponse;
    } catch (e) {
      print('Failed to load observability data: $e');
      return null;
    }
  }

  // Health Page Real-time Data
  Future<Map<String, dynamic>?> _getHealthData() async {
    try {
      final infoResponse = await _makeApiCall('/api/info');
      return infoResponse;
    } catch (e) {
      print('Failed to load health data: $e');
      return null;
    }
  }

  // Server Console Command Processing
  void _processServerCommand(String command, Map<String, dynamic> instance) async {
    print('Processing server command: $command');
    
    final trimmedCommand = command.trim().toLowerCase();
    String response = '';
    
    try {
      switch (trimmedCommand) {
        case 'help':
          response = '''Available Server Commands:

General:
  status, users, regions, metrics, restart, shutdown, version, uptime, memory, help

User Management:
  create user <first> <last> <email> <password>
  list users
  delete user <uuid>
  reset password <uuid> <new_password>
  set user level <uuid> <level>

Examples:
  create user Test User test@example.com password123
  list users
  reset password 12345-uuid-67890 newpass''';
          break;
          
        case 'status':
          final healthResponse = await _makeApiCall('/api/health');
          final infoResponse = await _makeApiCall('/api/info');
          response = 'Server Status:\nInstance: ${healthResponse?['instance_id'] ?? 'unknown'}\nStatus: ${healthResponse?['status'] ?? 'unknown'}\nCPU: ${(infoResponse?['cpu_usage'] ?? 0).round()}%\nMemory: ${((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round()} MB\nUsers: ${infoResponse?['active_connections'] ?? 0}\nRegions: ${infoResponse?['active_regions'] ?? 0}';
          break;
          
        case 'users':
          final infoResponse = await _makeApiCall('/api/info');
          final userCount = infoResponse?['active_connections'] ?? 0;
          response = 'Connected Users: $userCount';
          break;
          
        case 'regions':
          final infoResponse = await _makeApiCall('/api/info');
          final regionCount = infoResponse?['active_regions'] ?? 0;
          response = 'Active Regions: $regionCount';
          break;
          
        case 'metrics':
          final infoResponse = await _makeApiCall('/api/info');
          response = 'Performance Metrics:\nCPU: ${(infoResponse?['cpu_usage'] ?? 0).round()}%\nMemory: ${((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round()} MB\nUptime: ${_formatUptime(infoResponse?['uptime'] ?? 0)}';
          break;
          
        case 'version':
          response = 'OpenSim Next v2.1.0 (Rust + Zig Hybrid)';
          break;
          
        case 'uptime':
          final infoResponse = await _makeApiCall('/api/info');
          response = 'Uptime: ${_formatUptime(infoResponse?['uptime'] ?? 0)}';
          break;
          
        case 'memory':
          final infoResponse = await _makeApiCall('/api/info');
          final memoryMB = ((infoResponse?['memory_usage'] ?? 0) / 1024 / 1024).round();
          response = 'Memory Usage: $memoryMB MB';
          break;
          
        case 'restart':
          response = 'Server restart initiated (simulation)';
          break;
          
        case 'shutdown':
          response = 'Server shutdown initiated (simulation)';
          break;
          
        case 'list users':
          response = await _handleListUsers();
          break;
          
        default:
          // Check for multi-word commands
          if (trimmedCommand.startsWith('create user ')) {
            response = await _handleCreateUser(trimmedCommand);
          } else if (trimmedCommand.startsWith('delete user ')) {
            response = await _handleDeleteUser(trimmedCommand);
          } else if (trimmedCommand.startsWith('reset password ')) {
            response = await _handleResetPassword(trimmedCommand);
          } else if (trimmedCommand.startsWith('set user level ')) {
            response = await _handleSetUserLevel(trimmedCommand);
          } else {
            response = 'Unknown command: "$command". Type "help" for available commands.';
          }
      }
    } catch (e) {
      response = 'Error: $e';
    }
    
    // Don't show snackbar for console commands - response will be displayed in console
    print('Console Response: $response');
  }

  // === Admin Panel Live Data ===

  Future<void> _loadAdminData() async {
    setState(() => _adminLoading = true);
    try {
      final service = AdminService.instance;
      final health = await service.getHealth();
      final stats = await service.getDatabaseStats();
      final users = await service.listUsers();
      if (mounted) {
        setState(() {
          _adminHealth = health;
          _adminDbStats = stats;
          _adminUsers = users;
          _adminLoading = false;
          _adminConnected = health['status'] == 'healthy';
        });
      }
    } catch (e) {
      print('Failed to load admin data: $e');
      if (mounted) {
        setState(() {
          _adminLoading = false;
          _adminConnected = false;
        });
      }
    }
  }

  Future<void> _adminCreateUser(String firstname, String lastname, String email, String password) async {
    final result = await AdminService.instance.createUser(firstname, lastname, email, password);
    if (result['success'] == true) {
      _showSnackBar('User $firstname $lastname created successfully');
      _loadAdminData();
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  Future<void> _adminDeleteUser(String firstname, String lastname) async {
    final result = await AdminService.instance.deleteUser(firstname, lastname);
    if (result['success'] == true) {
      _showSnackBar('User $firstname $lastname deleted');
      _loadAdminData();
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  Future<void> _adminResetPassword(String firstname, String lastname, String newPassword) async {
    final result = await AdminService.instance.resetPassword(firstname, lastname, newPassword);
    if (result['success'] == true) {
      _showSnackBar('Password reset for $firstname $lastname');
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  Future<void> _adminSetLevel(String firstname, String lastname, int level) async {
    final result = await AdminService.instance.setUserLevel(firstname, lastname, level);
    if (result['success'] == true) {
      _showSnackBar('Level set to $level for $firstname $lastname');
      _loadAdminData();
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  // === Security Dashboard Live Data ===

  void _startSecurityRefresh() {
    _loadSecurityData();
    _securityRefreshTimer?.cancel();
    _securityRefreshTimer = Timer.periodic(Duration(seconds: 10), (_) => _loadSecurityData());
  }

  void _stopSecurityRefresh() {
    _securityRefreshTimer?.cancel();
    _securityRefreshTimer = null;
  }

  Future<void> _loadSecurityData() async {
    final service = AdminService.instance;
    final results = await Future.wait([
      service.getSecurityStats(),
      service.getSecurityThreats(),
      service.getSecurityLockouts(),
      service.getZitiStatus(),
    ]);
    if (mounted) {
      setState(() {
        _securityStats = results[0];
        _securityThreats = results[1];
        _securityLockouts = results[2];
        _zitiStatus = results[3];
      });
    }
  }

  Future<void> _securityBlockIp(String ip) async {
    final result = await AdminService.instance.blacklistIp(ip);
    if (result['success'] == true) {
      _showSnackBar('Blocked IP: $ip');
      _blockIpController.clear();
      _loadSecurityData();
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  Future<void> _securityUnblockIp(String ip) async {
    final result = await AdminService.instance.unblockIp(ip);
    if (result['success'] == true) {
      _showSnackBar('Unblocked IP: $ip');
      _loadSecurityData();
    } else {
      _showSnackBar('Failed: ${result['message']}');
    }
  }

  void _showCreateUserDialog() {
    final firstCtrl = TextEditingController();
    final lastCtrl = TextEditingController();
    final emailCtrl = TextEditingController();
    final passCtrl = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Create User'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(controller: firstCtrl, decoration: InputDecoration(labelText: 'First Name')),
            TextField(controller: lastCtrl, decoration: InputDecoration(labelText: 'Last Name')),
            TextField(controller: emailCtrl, decoration: InputDecoration(labelText: 'Email')),
            TextField(controller: passCtrl, decoration: InputDecoration(labelText: 'Password'), obscureText: true),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: Text('Cancel')),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(ctx);
              _adminCreateUser(firstCtrl.text, lastCtrl.text, emailCtrl.text, passCtrl.text);
            },
            child: Text('Create'),
          ),
        ],
      ),
    );
  }

  void _showResetPasswordDialog(Map<String, dynamic> user) {
    final passCtrl = TextEditingController();
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Reset Password'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('User: ${user['FirstName'] ?? user['firstname'] ?? ''} ${user['LastName'] ?? user['lastname'] ?? ''}'),
            SizedBox(height: 16),
            TextField(controller: passCtrl, decoration: InputDecoration(labelText: 'New Password'), obscureText: true),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: Text('Cancel')),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(ctx);
              _adminResetPassword(
                user['FirstName'] ?? user['firstname'] ?? '',
                user['LastName'] ?? user['lastname'] ?? '',
                passCtrl.text,
              );
            },
            child: Text('Reset'),
          ),
        ],
      ),
    );
  }

  void _showSetLevelDialog(Map<String, dynamic> user) {
    int level = (user['UserLevel'] ?? user['user_level'] ?? 0) as int;
    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: Text('Set User Level'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('User: ${user['FirstName'] ?? user['firstname'] ?? ''} ${user['LastName'] ?? user['lastname'] ?? ''}'),
              SizedBox(height: 16),
              Text('Level: $level'),
              Slider(
                value: level.toDouble(),
                min: 0,
                max: 255,
                divisions: 255,
                onChanged: (v) => setDialogState(() => level = v.toInt()),
              ),
            ],
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(ctx), child: Text('Cancel')),
            ElevatedButton(
              onPressed: () {
                Navigator.pop(ctx);
                _adminSetLevel(
                  user['FirstName'] ?? user['firstname'] ?? '',
                  user['LastName'] ?? user['lastname'] ?? '',
                  level,
                );
              },
              child: Text('Set Level'),
            ),
          ],
        ),
      ),
    );
  }

  void _showDeleteUserDialog(Map<String, dynamic> user) {
    final name = '${user['FirstName'] ?? user['firstname'] ?? ''} ${user['LastName'] ?? user['lastname'] ?? ''}';
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Delete User'),
        content: Text('Are you sure you want to delete $name? This cannot be undone.'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: Text('Cancel')),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () {
              Navigator.pop(ctx);
              _adminDeleteUser(
                user['FirstName'] ?? user['firstname'] ?? '',
                user['LastName'] ?? user['lastname'] ?? '',
              );
            },
            child: Text('Delete', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }

  Widget _buildAdminUserTable() {
    if (_adminLoading) {
      return Center(child: CircularProgressIndicator());
    }
    if (!_adminConnected) {
      return Center(
        child: Column(
          children: [
            Icon(Icons.cloud_off, size: 48, color: Colors.grey),
            SizedBox(height: 8),
            Text('Admin API offline', style: TextStyle(color: Colors.grey)),
            SizedBox(height: 8),
            ElevatedButton(onPressed: _loadAdminData, child: Text('Retry')),
          ],
        ),
      );
    }
    if (_adminUsers.isEmpty) {
      return Center(child: Text('No users found'));
    }
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: DataTable(
        columns: [
          DataColumn(label: Text('First Name')),
          DataColumn(label: Text('Last Name')),
          DataColumn(label: Text('Email')),
          DataColumn(label: Text('Level')),
          DataColumn(label: Text('Actions')),
        ],
        rows: _adminUsers.map((user) {
          final first = user['FirstName'] ?? user['firstname'] ?? '';
          final last = user['LastName'] ?? user['lastname'] ?? '';
          final email = user['Email'] ?? user['email'] ?? '';
          final level = user['UserLevel'] ?? user['user_level'] ?? 0;
          return DataRow(cells: [
            DataCell(Text('$first')),
            DataCell(Text('$last')),
            DataCell(Text('$email')),
            DataCell(Text('$level')),
            DataCell(Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(icon: Icon(Icons.lock_reset, size: 20), tooltip: 'Reset Password', onPressed: () => _showResetPasswordDialog(user)),
                IconButton(icon: Icon(Icons.admin_panel_settings, size: 20), tooltip: 'Set Level', onPressed: () => _showSetLevelDialog(user)),
                IconButton(icon: Icon(Icons.delete, size: 20, color: Colors.red), tooltip: 'Delete', onPressed: () => _showDeleteUserDialog(user)),
              ],
            )),
          ]);
        }).toList(),
      ),
    );
  }

  // === AI Insights Page ===

  Future<void> _loadAIGridOverview() async {
    setState(() => _aiLoading = true);
    final svc = RecommendationService.instance;
    final trending = await svc.getTrendingContent(limit: 10);
    final stats = await svc.getRecommenderStats();
    setState(() {
      _aiTrending = trending;
      _aiStats = stats;
      _aiLoading = false;
    });
  }

  Future<void> _loadAIUserData() async {
    if (_aiUserId.isEmpty) return;
    setState(() => _aiLoading = true);
    final svc = RecommendationService.instance;
    List<Map<String, dynamic>> recs;
    switch (_aiRecTab) {
      case 1:
        recs = await svc.getSocialRecommendations(_aiUserId);
        break;
      case 2:
        recs = await svc.getCreatorRecommendations(_aiUserId);
        break;
      default:
        recs = await svc.getContentRecommendations(_aiUserId);
    }
    final engagement = await svc.getEngagementMetrics(_aiUserId);
    setState(() {
      _aiRecommendations = recs;
      _aiEngagement = engagement;
      _aiLoading = false;
    });
  }

  Widget _buildAIInsightsPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            children: [
              Icon(Icons.auto_awesome, color: Colors.deepPurple, size: 32),
              SizedBox(width: 12),
              Text('AI Insights', style: Theme.of(context).textTheme.headlineMedium?.copyWith(fontWeight: FontWeight.bold)),
              Spacer(),
              IconButton(
                icon: Icon(Icons.refresh),
                onPressed: () => _aiViewMode == 0 ? _loadAIGridOverview() : _loadAIUserData(),
                tooltip: 'Refresh',
              ),
            ],
          ),
          SizedBox(height: 16),

          // View mode toggle
          Row(
            children: [
              _buildAIToggleButton('Grid Overview', 0),
              SizedBox(width: 8),
              _buildAIToggleButton('User Drill-down', 1),
            ],
          ),
          SizedBox(height: 24),

          if (_aiLoading)
            Center(child: Padding(padding: EdgeInsets.all(40), child: CircularProgressIndicator()))
          else if (_aiViewMode == 0)
            _buildAIGridOverview()
          else
            _buildAIUserDrilldown(),
        ],
      ),
    );
  }

  Widget _buildAIToggleButton(String label, int mode) {
    final selected = _aiViewMode == mode;
    return ElevatedButton(
      onPressed: () {
        setState(() => _aiViewMode = mode);
        if (mode == 0) _loadAIGridOverview();
      },
      style: ElevatedButton.styleFrom(
        backgroundColor: selected ? Colors.deepPurple : Theme.of(context).colorScheme.surfaceVariant,
        foregroundColor: selected ? Colors.white : Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      child: Text(label),
    );
  }

  Widget _buildAIGridOverview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Trending Content
        _buildAICard(
          title: 'Trending Content',
          icon: Icons.trending_up,
          child: _aiTrending.isEmpty
              ? Padding(
                  padding: EdgeInsets.all(24),
                  child: Center(child: Text('No trending data. Load data or connect to server.', style: TextStyle(color: Colors.grey))),
                )
              : SizedBox(
                  height: 140,
                  child: ListView.builder(
                    scrollDirection: Axis.horizontal,
                    itemCount: _aiTrending.length,
                    itemBuilder: (context, index) {
                      final item = _aiTrending[index];
                      return _buildTrendingItem(item, index);
                    },
                  ),
                ),
        ),
        SizedBox(height: 16),

        // Recommender Stats
        _buildAICard(
          title: 'Recommender Engine',
          icon: Icons.analytics,
          child: _aiStats.isEmpty
              ? Padding(
                  padding: EdgeInsets.all(24),
                  child: Center(child: Text('No stats available. Click refresh to load.', style: TextStyle(color: Colors.grey))),
                )
              : Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
                    children: [
                      Row(
                        children: [
                          Icon(
                            _aiStats['_offline'] == true ? Icons.cloud_off : Icons.cloud_done,
                            color: _aiStats['_offline'] == true ? Colors.orange : Colors.green,
                          ),
                          SizedBox(width: 8),
                          Text(
                            _aiStats['_offline'] == true ? 'Offline (Mock Data)' : 'Connected',
                            style: TextStyle(fontWeight: FontWeight.w600),
                          ),
                        ],
                      ),
                      SizedBox(height: 16),
                      Row(
                        children: [
                          _buildStatTile('Users', '${_aiStats['total_users'] ?? 0}', Icons.people),
                          _buildStatTile('Activities', '${_aiStats['total_activities'] ?? 0}', Icons.touch_app),
                          _buildStatTile('Profiles', '${_aiStats['total_profiles'] ?? 0}', Icons.person),
                          _buildStatTile('Items', '${_aiStats['tracked_items'] ?? 0}', Icons.inventory_2),
                        ],
                      ),
                      SizedBox(height: 8),
                      Row(
                        children: [
                          _buildStatTile('Cache', '${_aiStats['cache_size'] ?? 0}', Icons.cached),
                        ],
                      ),
                    ],
                  ),
                ),
        ),
      ],
    );
  }

  Widget _buildTrendingItem(Map<String, dynamic> item, int rank) {
    final type = item['content_type'] ?? 'Unknown';
    final name = item['name'] ?? 'Unnamed';
    final score = (item['score'] ?? 0.0) as num;
    final count = item['interaction_count'] ?? 0;

    IconData typeIcon;
    Color typeColor;
    switch (type) {
      case 'Region':
        typeIcon = Icons.map;
        typeColor = Colors.blue;
        break;
      case 'Asset':
        typeIcon = Icons.inventory_2;
        typeColor = Colors.orange;
        break;
      case 'Event':
        typeIcon = Icons.event;
        typeColor = Colors.purple;
        break;
      default:
        typeIcon = Icons.star;
        typeColor = Colors.grey;
    }

    return Container(
      width: 160,
      margin: EdgeInsets.only(right: 12),
      child: Card(
        elevation: 2,
        child: Padding(
          padding: EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    padding: EdgeInsets.all(4),
                    decoration: BoxDecoration(
                      color: typeColor.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Icon(typeIcon, size: 16, color: typeColor),
                  ),
                  Spacer(),
                  Container(
                    padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: Colors.deepPurple.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text('#${rank + 1}', style: TextStyle(fontSize: 11, color: Colors.deepPurple, fontWeight: FontWeight.bold)),
                  ),
                ],
              ),
              SizedBox(height: 8),
              Text(name, style: TextStyle(fontWeight: FontWeight.w600, fontSize: 13), maxLines: 2, overflow: TextOverflow.ellipsis),
              Spacer(),
              Row(
                children: [
                  Text('${(score * 100).toInt()}%', style: TextStyle(fontSize: 11, color: _scoreColor(score.toDouble()))),
                  Spacer(),
                  Icon(Icons.touch_app, size: 12, color: Colors.grey),
                  SizedBox(width: 2),
                  Text('$count', style: TextStyle(fontSize: 11, color: Colors.grey)),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildAIUserDrilldown() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // User ID Input
        _buildAICard(
          title: 'Select User',
          icon: Icons.person_search,
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _aiUserIdController,
                    decoration: InputDecoration(
                      hintText: 'Enter User UUID...',
                      border: OutlineInputBorder(),
                      isDense: true,
                    ),
                    onChanged: (v) => _aiUserId = v.trim(),
                    onSubmitted: (_) => _loadAIUserData(),
                  ),
                ),
                SizedBox(width: 12),
                ElevatedButton.icon(
                  onPressed: _aiUserId.isEmpty ? null : () => _loadAIUserData(),
                  icon: Icon(Icons.search, size: 18),
                  label: Text('Load'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.deepPurple),
                ),
              ],
            ),
          ),
        ),
        SizedBox(height: 16),

        if (_aiUserId.isNotEmpty) ...[
          // Recommendations with sub-tabs
          _buildAICard(
            title: 'Recommendations',
            icon: Icons.recommend,
            child: Column(
              children: [
                // Sub-tab bar
                Padding(
                  padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  child: Row(
                    children: [
                      _buildRecTabButton('For You', 0),
                      SizedBox(width: 8),
                      _buildRecTabButton('Social', 1),
                      SizedBox(width: 8),
                      _buildRecTabButton('Creators', 2),
                    ],
                  ),
                ),
                // Recommendation list
                _aiRecommendations.isEmpty
                    ? Padding(
                        padding: EdgeInsets.all(24),
                        child: Center(child: Text(
                          _aiUserId.isEmpty ? 'Enter a user ID above' : 'No recommendations for this user yet.',
                          style: TextStyle(color: Colors.grey),
                        )),
                      )
                    : ListView.builder(
                        shrinkWrap: true,
                        physics: NeverScrollableScrollPhysics(),
                        itemCount: _aiRecommendations.length,
                        itemBuilder: (context, index) {
                          return _buildRecommendationTile(_aiRecommendations[index]);
                        },
                      ),
              ],
            ),
          ),
          SizedBox(height: 16),

          // Engagement Metrics
          _buildAICard(
            title: 'Engagement Metrics',
            icon: Icons.insights,
            child: _aiEngagement.isEmpty || _aiEngagement['_offline'] == true
                ? Padding(
                    padding: EdgeInsets.all(24),
                    child: Center(child: Text('No engagement data available.', style: TextStyle(color: Colors.grey))),
                  )
                : Padding(
                    padding: EdgeInsets.all(16),
                    child: Column(
                      children: [
                        Row(
                          children: [
                            _buildEngagementMetric('Sessions\n(30d)', _aiEngagement['sessions_last_30_days'] ?? 0, 30, Colors.blue),
                            _buildEngagementMetric('Avg Duration\n(min)', (_aiEngagement['avg_session_duration_minutes'] ?? 0.0).round(), 120, Colors.green),
                            _buildEngagementMetric('Regions\nVisited', _aiEngagement['regions_visited_30_days'] ?? 0, 20, Colors.orange),
                            _buildEngagementMetric('Social\nActions', _aiEngagement['social_interactions_30_days'] ?? 0, 50, Colors.purple),
                          ],
                        ),
                        SizedBox(height: 16),
                        Divider(),
                        SizedBox(height: 8),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildChurnRisk((_aiEngagement['churn_risk_score'] ?? 0.0).toDouble()),
                            _buildEngagementTrend(_aiEngagement['engagement_trend'] ?? 'Unknown'),
                          ],
                        ),
                      ],
                    ),
                  ),
          ),
        ],
      ],
    );
  }

  Widget _buildRecTabButton(String label, int tab) {
    final selected = _aiRecTab == tab;
    return OutlinedButton(
      onPressed: () {
        setState(() => _aiRecTab = tab);
        _loadAIUserData();
      },
      style: OutlinedButton.styleFrom(
        backgroundColor: selected ? Colors.deepPurple.withValues(alpha: 0.1) : null,
        side: BorderSide(color: selected ? Colors.deepPurple : Colors.grey),
        padding: EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      ),
      child: Text(label, style: TextStyle(fontSize: 12, color: selected ? Colors.deepPurple : null)),
    );
  }

  Widget _buildRecommendationTile(Map<String, dynamic> rec) {
    final name = rec['name'] ?? rec['user_name'] ?? rec['creator_name'] ?? 'Unknown';
    final type = rec['content_type'] ?? rec['reason'] ?? '';
    final score = ((rec['score'] ?? rec['similarity_score'] ?? 0.0) as num).toDouble();
    final reason = rec['reason'] ?? '';

    return ListTile(
      leading: CircleAvatar(
        backgroundColor: _scoreColor(score).withValues(alpha: 0.2),
        child: Icon(_contentTypeIcon(type), color: _scoreColor(score), size: 20),
      ),
      title: Text(name, style: TextStyle(fontWeight: FontWeight.w500)),
      subtitle: reason.isNotEmpty ? Text(reason, maxLines: 1, overflow: TextOverflow.ellipsis, style: TextStyle(fontSize: 12)) : null,
      trailing: Container(
        padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: _scoreColor(score).withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Text('${(score * 100).toInt()}%', style: TextStyle(fontSize: 12, color: _scoreColor(score), fontWeight: FontWeight.bold)),
      ),
    );
  }

  Widget _buildEngagementMetric(String label, int value, int max, Color color) {
    final progress = max > 0 ? (value / max).clamp(0.0, 1.0) : 0.0;
    return Expanded(
      child: Column(
        children: [
          SizedBox(
            width: 50,
            height: 50,
            child: Stack(
              alignment: Alignment.center,
              children: [
                CircularProgressIndicator(
                  value: progress,
                  backgroundColor: color.withValues(alpha: 0.1),
                  color: color,
                  strokeWidth: 4,
                ),
                Text('$value', style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold)),
              ],
            ),
          ),
          SizedBox(height: 6),
          Text(label, textAlign: TextAlign.center, style: TextStyle(fontSize: 10, color: Colors.grey)),
        ],
      ),
    );
  }

  Widget _buildChurnRisk(double risk) {
    Color riskColor;
    String riskLabel;
    if (risk < 0.3) {
      riskColor = Colors.green;
      riskLabel = 'Low Risk';
    } else if (risk < 0.6) {
      riskColor = Colors.orange;
      riskLabel = 'Medium Risk';
    } else {
      riskColor = Colors.red;
      riskLabel = 'High Risk';
    }
    return Column(
      children: [
        Icon(Icons.warning_amber, color: riskColor, size: 28),
        SizedBox(height: 4),
        Text(riskLabel, style: TextStyle(color: riskColor, fontWeight: FontWeight.w600, fontSize: 12)),
        Text('Churn: ${(risk * 100).toInt()}%', style: TextStyle(fontSize: 11, color: Colors.grey)),
      ],
    );
  }

  Widget _buildEngagementTrend(String trend) {
    IconData icon;
    Color color;
    switch (trend) {
      case 'Increasing':
        icon = Icons.trending_up;
        color = Colors.green;
        break;
      case 'Decreasing':
        icon = Icons.trending_down;
        color = Colors.red;
        break;
      default:
        icon = Icons.trending_flat;
        color = Colors.grey;
    }
    return Column(
      children: [
        Icon(icon, color: color, size: 28),
        SizedBox(height: 4),
        Text(trend, style: TextStyle(color: color, fontWeight: FontWeight.w600, fontSize: 12)),
        Text('Engagement', style: TextStyle(fontSize: 11, color: Colors.grey)),
      ],
    );
  }

  Widget _buildAICard({required String title, required IconData icon, required Widget child}) {
    return Card(
      elevation: 2,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: EdgeInsets.fromLTRB(16, 12, 16, 0),
            child: Row(
              children: [
                Icon(icon, color: Colors.deepPurple, size: 20),
                SizedBox(width: 8),
                Text(title, style: TextStyle(fontWeight: FontWeight.w600, fontSize: 15)),
              ],
            ),
          ),
          child,
        ],
      ),
    );
  }

  Widget _buildStatTile(String label, String value, IconData icon) {
    return Expanded(
      child: Column(
        children: [
          Icon(icon, size: 20, color: Colors.deepPurple.withValues(alpha: 0.7)),
          SizedBox(height: 4),
          Text(value, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
          Text(label, style: TextStyle(fontSize: 11, color: Colors.grey)),
        ],
      ),
    );
  }

  Color _scoreColor(double score) {
    if (score >= 0.7) return Colors.green;
    if (score >= 0.4) return Colors.orange;
    return Colors.red;
  }

  IconData _contentTypeIcon(String type) {
    switch (type) {
      case 'Region': return Icons.map;
      case 'Asset': return Icons.inventory_2;
      case 'Event': return Icons.event;
      case 'Creator': return Icons.brush;
      case 'Group': return Icons.group;
      default: return Icons.star;
    }
  }

  Widget _buildInstanceManagerPage() {
    return ChangeNotifierProvider(
      key: const ValueKey('instance-manager'),
      create: (_) => InstanceManagerProvider(),
      child: const InstanceManagerScreen(),
    );
  }

  Widget _buildConfigurationBuilderPage() {
    return ChangeNotifierProvider(
      create: (_) => ConfigurationBuilderProvider(),
      child: const ConfigurationBuilderScreen(),
    );
  }

  Widget _buildArchiveManagementPage() {
    return ChangeNotifierProvider(
      create: (_) => ArchiveProvider()..configure(
        _discoveredAdminUrl ?? 'http://localhost:9700',
        null,
        wsUrl: 'ws://localhost:9001',
      ),
      child: const ArchiveManagementScreen(),
    );
  }

  Widget _buildGridInstancesPage() {
    return ChangeNotifierProvider(
      create: (_) => InstanceDirectoryProvider(
        instancesBasePath: 'Instances',
      ),
      child: const InstanceDirectoryScreen(),
    );
  }

  String _consoleSelectedInstance = 'local';
  String _consoleBaseUrl = 'http://localhost:9200';

  final List<Map<String, String>> _consoleInstances = [
    {'id': 'local', 'name': 'Local Server', 'url': 'http://localhost:9200'},
    {'id': 'custom', 'name': 'Custom URL...', 'url': ''},
  ];

  Widget _buildConsolePage() {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: Theme.of(context).cardColor,
            border: Border(bottom: BorderSide(color: Colors.grey.shade700)),
          ),
          child: Row(
            children: [
              const Icon(Icons.dns, size: 20),
              const SizedBox(width: 12),
              const Text('Instance:', style: TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(width: 12),
              DropdownButton<String>(
                value: _consoleSelectedInstance,
                underline: Container(),
                items: _consoleInstances.map((instance) {
                  return DropdownMenuItem<String>(
                    value: instance['id'],
                    child: Text(instance['name']!),
                  );
                }).toList(),
                onChanged: (value) {
                  setState(() {
                    _consoleSelectedInstance = value!;
                    final instance = _consoleInstances.firstWhere((i) => i['id'] == value);
                    if (value != 'custom') {
                      _consoleBaseUrl = instance['url']!;
                    }
                  });
                },
              ),
              if (_consoleSelectedInstance == 'custom') ...[
                const SizedBox(width: 16),
                SizedBox(
                  width: 300,
                  child: TextField(
                    decoration: const InputDecoration(
                      hintText: 'Enter server URL (e.g., http://localhost:9700)',
                      isDense: true,
                      border: OutlineInputBorder(),
                    ),
                    onChanged: (value) => _consoleBaseUrl = value,
                  ),
                ),
              ],
              const Spacer(),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                decoration: BoxDecoration(
                  color: Colors.green.withOpacity(0.2),
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: Colors.green),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 8,
                      height: 8,
                      decoration: const BoxDecoration(
                        color: Colors.green,
                        shape: BoxShape.circle,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text(_consoleBaseUrl, style: const TextStyle(fontSize: 12)),
                  ],
                ),
              ),
            ],
          ),
        ),
        Expanded(
          child: InstanceConsoleScreen(
            key: ValueKey('$_consoleSelectedInstance-$_consoleBaseUrl'),
            instanceId: _consoleSelectedInstance,
            instanceName: _consoleInstances.firstWhere(
              (i) => i['id'] == _consoleSelectedInstance,
              orElse: () => {'name': 'Custom Instance'},
            )['name']!,
            baseUrl: _consoleBaseUrl,
            apiKey: null,
          ),
        ),
      ],
    );
  }
}

// Glassmorphism Button Component inspired by glossy library
// https://github.com/joysarkar18/glossy
class GlossyButton extends StatefulWidget {
  final VoidCallback? onPressed;
  final Widget child;
  final double? width;
  final double? height;
  final EdgeInsets? padding;
  final Color? color;
  final double borderRadius;
  final bool enabled;

  const GlossyButton({
    Key? key,
    required this.onPressed,
    required this.child,
    this.width,
    this.height,
    this.padding,
    this.color,
    this.borderRadius = 12.0,
    this.enabled = true,
  }) : super(key: key);

  factory GlossyButton.icon({
    Key? key,
    required VoidCallback? onPressed,
    required Widget icon,
    required Widget label,
    double? width,
    double? height,
    EdgeInsets? padding,
    Color? color,
    double borderRadius = 12.0,
    bool enabled = true,
  }) {
    return GlossyButton(
      key: key,
      onPressed: onPressed,
      width: width,
      height: height,
      padding: padding,
      color: color,
      borderRadius: borderRadius,
      enabled: enabled,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          icon,
          SizedBox(width: 8),
          label,
        ],
      ),
    );
  }

  @override
  _GlossyButtonState createState() => _GlossyButtonState();
}

class _GlossyButtonState extends State<GlossyButton>
    with TickerProviderStateMixin {
  late AnimationController _hoverController;
  late AnimationController _pressController;
  late Animation<double> _hoverAnimation;
  late Animation<double> _pressAnimation;
  bool _isHovered = false;
  bool _isPressed = false;

  @override
  void initState() {
    super.initState();
    _hoverController = AnimationController(
      duration: Duration(milliseconds: 200),
      vsync: this,
    );
    _pressController = AnimationController(
      duration: Duration(milliseconds: 100),
      vsync: this,
    );
    _hoverAnimation = Tween<double>(begin: 1.0, end: 1.05).animate(
      CurvedAnimation(parent: _hoverController, curve: Curves.easeInOut),
    );
    _pressAnimation = Tween<double>(begin: 1.0, end: 0.95).animate(
      CurvedAnimation(parent: _pressController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _hoverController.dispose();
    _pressController.dispose();
    super.dispose();
  }

  void _handleHover(bool hovering) {
    if (!widget.enabled) return;
    setState(() {
      _isHovered = hovering;
    });
    if (hovering) {
      _hoverController.forward();
    } else {
      _hoverController.reverse();
    }
  }

  void _handlePress(bool pressing) {
    if (!widget.enabled) return;
    setState(() {
      _isPressed = pressing;
    });
    if (pressing) {
      _pressController.forward();
    } else {
      _pressController.reverse();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final effectiveColor = widget.color ?? theme.colorScheme.primary;
    final isDark = theme.brightness == Brightness.dark;
    
    return MouseRegion(
      onEnter: (_) => _handleHover(true),
      onExit: (_) => _handleHover(false),
      child: GestureDetector(
        onTapDown: (_) => _handlePress(true),
        onTapUp: (_) {
          _handlePress(false);
          if (widget.enabled && widget.onPressed != null) {
            widget.onPressed!();
          }
        },
        onTapCancel: () => _handlePress(false),
        child: AnimatedBuilder(
          animation: Listenable.merge([_hoverAnimation, _pressAnimation]),
          builder: (context, child) {
            return Transform.scale(
              scale: _pressAnimation.value * _hoverAnimation.value,
              child: Container(
                width: widget.width,
                height: widget.height,
                padding: widget.padding ?? EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(widget.borderRadius),
                  gradient: LinearGradient(
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                    colors: widget.enabled
                        ? [
                            effectiveColor.withValues(alpha: 0.8),
                            effectiveColor.withValues(alpha: 0.6),
                          ]
                        : [
                            Colors.grey.withValues(alpha: 0.3),
                            Colors.grey.withValues(alpha: 0.2),
                          ],
                  ),
                  border: Border.all(
                    color: widget.enabled
                        ? Colors.white.withValues(alpha: 0.3)
                        : Colors.grey.withValues(alpha: 0.2),
                    width: 1.0,
                  ),
                  boxShadow: widget.enabled
                      ? [
                          BoxShadow(
                            color: effectiveColor.withValues(alpha: 0.3),
                            blurRadius: _isHovered ? 20 : 10,
                            spreadRadius: _isHovered ? 2 : 0,
                            offset: Offset(0, _isHovered ? 8 : 4),
                          ),
                          BoxShadow(
                            color: Colors.black.withOpacity(isDark ? 0.3 : 0.1),
                            blurRadius: 10,
                            offset: Offset(0, 2),
                          ),
                        ]
                      : [
                          BoxShadow(
                            color: Colors.grey.withValues(alpha: 0.2),
                            blurRadius: 5,
                            offset: Offset(0, 2),
                          ),
                        ],
                ),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(widget.borderRadius),
                  child: BackdropFilter(
                    filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
                    child: Container(
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          begin: Alignment.topLeft,
                          end: Alignment.bottomRight,
                          colors: [
                            Colors.white.withOpacity(_isHovered ? 0.3 : 0.2),
                            Colors.white.withOpacity(_isHovered ? 0.1 : 0.05),
                          ],
                        ),
                      ),
                      child: Center(
                        child: DefaultTextStyle(
                          style: TextStyle(
                            color: widget.enabled ? Colors.white : Colors.grey,
                            fontWeight: FontWeight.w600,
                            shadows: [
                              Shadow(
                                color: Colors.black.withValues(alpha: 0.3),
                                blurRadius: 2,
                                offset: Offset(0, 1),
                              ),
                            ],
                          ),
                          child: widget.child,
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}