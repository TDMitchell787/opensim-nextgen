import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'services/unified_backend_service.dart';
import 'models/deployment_models.dart';
import 'theme/observability_themes.dart';
import 'package:fl_chart/fl_chart.dart';
import 'dart:async';
import 'dart:math' as math;

void main() {
  runApp(ReorderedOpenSimApp());
}

class ReorderedOpenSimApp extends StatefulWidget {
  @override
  _ReorderedOpenSimAppState createState() => _ReorderedOpenSimAppState();
}

class _ReorderedOpenSimAppState extends State<ReorderedOpenSimApp> {
  String _currentTheme = ObservabilityThemes.system;

  @override
  void initState() {
    super.initState();
    UnifiedBackendService.instance.initializeWebSocket();
  }

  @override
  void dispose() {
    UnifiedBackendService.instance.closeWebSocket();
    super.dispose();
  }

  void _changeTheme(String themeName) {
    setState(() {
      _currentTheme = themeName;
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'OpenSim Next - Comprehensive 10-Page Platform',
      theme: _currentTheme == ObservabilityThemes.system || _currentTheme == ObservabilityThemes.light
          ? ObservabilityThemes.lightTheme
          : ObservabilityThemes.getTheme(_currentTheme),
      darkTheme: _currentTheme == ObservabilityThemes.system
          ? ObservabilityThemes.darkTheme
          : ObservabilityThemes.getTheme(_currentTheme),
      themeMode: ObservabilityThemes.getThemeMode(_currentTheme),
      home: ComprehensiveDashboard(
        currentTheme: _currentTheme,
        onThemeChanged: _changeTheme,
      ),
    );
  }
}

class ComprehensiveDashboard extends StatefulWidget {
  final String currentTheme;
  final Function(String) onThemeChanged;

  const ComprehensiveDashboard({
    Key? key,
    required this.currentTheme,
    required this.onThemeChanged,
  }) : super(key: key);

  @override
  _ComprehensiveDashboardState createState() => _ComprehensiveDashboardState();
}

class _ComprehensiveDashboardState extends State<ComprehensiveDashboard> with TickerProviderStateMixin {
  late TabController _tabController;
  late StreamSubscription _realtimeSubscription;
  Map<String, dynamic> _realtimeData = {};

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 10, vsync: this);
    
    _realtimeSubscription = UnifiedBackendService.instance.realTimeData.listen((data) {
      if (mounted) {
        setState(() {
          _realtimeData = data;
        });
      }
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    _realtimeSubscription.cancel();
    super.dispose();
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
              'Reordered 10-Page Virtual World Management',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
          Container(
            margin: EdgeInsets.symmetric(horizontal: 8),
            child: Row(
              children: [
                Icon(
                  Icons.circle,
                  size: 12,
                  color: _realtimeData.isNotEmpty ? Colors.green : Colors.red,
                ),
                SizedBox(width: 4),
                Text('Live', style: TextStyle(fontSize: 12)),
              ],
            ),
          ),
          PopupMenuButton<String>(
            icon: Icon(Icons.palette),
            onSelected: widget.onThemeChanged,
            itemBuilder: (context) => ObservabilityThemes.allThemes.map((theme) {
              return PopupMenuItem(
                value: theme,
                child: Row(
                  children: [
                    if (widget.currentTheme == theme) Icon(Icons.check, size: 16),
                    SizedBox(width: 8),
                    Text(ObservabilityThemes.themeNames[theme] ?? theme),
                  ],
                ),
              );
            }).toList(),
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          isScrollable: true,
          tabs: [
            Tab(icon: Icon(Icons.image), text: 'Graphics Splash'),
            Tab(icon: Icon(Icons.people), text: 'Contributors'),  
            Tab(icon: Icon(Icons.home), text: 'Welcome'),
            Tab(icon: Icon(Icons.admin_panel_settings), text: 'Web Admin'),
            Tab(icon: Icon(Icons.analytics), text: 'Analytics'),
            Tab(icon: Icon(Icons.visibility), text: 'Observability'),
            Tab(icon: Icon(Icons.monitor_heart), text: 'Health'),
            Tab(icon: Icon(Icons.security), text: 'Security'),
            Tab(icon: Icon(Icons.storage), text: 'Database'),
            Tab(icon: Icon(Icons.settings), text: 'Settings'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          GraphicsSplashPage(),           // Page 1: Splash page for Graphics
          ContributorGraphicPage(),       // Page 2: Contributor Graphic  
          WelcomePage(),                  // Page 3: Welcome
          WebAdminPage(),                 // Page 4: Web Admin
          AnalyticsDashboardPage(),       // Page 5: Analytics Dashboard
          ObservabilityDashboardPage(),   // Page 6: Observability Dashboard
          HealthMonitoringPage(),         // Page 7: Health Monitoring
          SecurityDashboardPage(),        // Page 8: Security Dashboard
          DatabaseManagementPage(),       // Page 9: Database Management
          SettingsPage(),                 // Page 10: Settings
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showQuickNav(context),
        icon: Icon(Icons.dashboard),
        label: Text('Quick Nav'),
      ),
    );
  }

  void _showQuickNav(BuildContext context) {
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
              children: [
                _buildNavChip(context, 0, Icons.image, 'Graphics Splash'),
                _buildNavChip(context, 1, Icons.people, 'Contributors'),
                _buildNavChip(context, 2, Icons.home, 'Welcome'),
                _buildNavChip(context, 3, Icons.admin_panel_settings, 'Web Admin'),
                _buildNavChip(context, 4, Icons.analytics, 'Analytics'),
                _buildNavChip(context, 5, Icons.visibility, 'Observability'),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNavChip(BuildContext context, int index, IconData icon, String label) {
    return ActionChip(
      avatar: Icon(icon, size: 18),
      label: Text(label, style: TextStyle(fontSize: 12)),
      onPressed: () {
        Navigator.pop(context);
        _tabController.animateTo(index);
      },
    );
  }
}

// Page 1: Graphics Splash Page
class GraphicsSplashPage extends StatefulWidget {
  @override
  _GraphicsSplashPageState createState() => _GraphicsSplashPageState();
}

class _GraphicsSplashPageState extends State<GraphicsSplashPage> with TickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _fadeAnimation;
  late Animation<double> _scaleAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: Duration(seconds: 2),
      vsync: this,
    );
    
    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeIn),
    );
    
    _scaleAnimation = Tween<double>(begin: 0.8, end: 1.0).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.elasticOut),
    );
    
    _animationController.forward();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            Theme.of(context).primaryColor,
            Theme.of(context).primaryColor.withOpacity(0.3),
            Theme.of(context).colorScheme.secondary,
          ],
        ),
      ),
      child: AnimatedBuilder(
        animation: _animationController,
        builder: (context, child) {
          return FadeTransition(
            opacity: _fadeAnimation,
            child: ScaleTransition(
              scale: _scaleAnimation,
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
                        gradient: RadialGradient(
                          colors: [
                            Colors.white,
                            Theme.of(context).primaryColor.withOpacity(0.8),
                          ],
                        ),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.3),
                            blurRadius: 20,
                            spreadRadius: 5,
                          ),
                        ],
                      ),
                      child: Icon(
                        Icons.public,
                        size: 120,
                        color: Colors.white,
                      ),
                    ),
                    
                    SizedBox(height: 40),
                    
                    // Title with Animated Text
                    Text(
                      'OpenSim Next',
                      style: Theme.of(context).textTheme.headlineLarge?.copyWith(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                        shadows: [
                          Shadow(
                            color: Colors.black.withOpacity(0.5),
                            offset: Offset(2, 2),
                            blurRadius: 4,
                          ),
                        ],
                      ),
                    ),
                    
                    SizedBox(height: 16),
                    
                    Text(
                      'Revolutionary Virtual World Platform',
                      style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                        color: Colors.white.withOpacity(0.9),
                        fontWeight: FontWeight.w300,
                      ),
                    ),
                    
                    SizedBox(height: 40),
                    
                    // Feature highlights with graphics
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                      children: [
                        _buildFeatureIcon(Icons.speed, 'High\nPerformance'),
                        _buildFeatureIcon(Icons.security, 'Zero Trust\nSecurity'),
                        _buildFeatureIcon(Icons.analytics, 'Real-time\nAnalytics'),
                        _buildFeatureIcon(Icons.devices, 'Multi\nPlatform'),
                      ],
                    ),
                    
                    SizedBox(height: 60),
                    
                    // Animated call-to-action
                    Container(
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(30),
                        gradient: LinearGradient(
                          colors: [Colors.white, Colors.white.withOpacity(0.8)],
                        ),
                        boxShadow: [
                          BoxShadow(
                            color: Colors.black.withOpacity(0.2),
                            blurRadius: 15,
                            spreadRadius: 2,
                          ),
                        ],
                      ),
                      child: MaterialButton(
                        onPressed: () {
                          // Navigate to welcome page
                          DefaultTabController.of(context)?.animateTo(2);
                        },
                        padding: EdgeInsets.symmetric(horizontal: 40, vertical: 16),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.arrow_forward, color: Theme.of(context).primaryColor),
                            SizedBox(width: 8),
                            Text(
                              'Enter Virtual World',
                              style: TextStyle(
                                fontSize: 18,
                                fontWeight: FontWeight.bold,
                                color: Theme.of(context).primaryColor,
                              ),
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
        },
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
            color: Colors.white.withOpacity(0.2),
            border: Border.all(color: Colors.white.withOpacity(0.5), width: 2),
          ),
          child: Icon(icon, size: 40, color: Colors.white),
        ),
        SizedBox(height: 12),
        Text(
          label,
          textAlign: TextAlign.center,
          style: TextStyle(
            color: Colors.white,
            fontSize: 12,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }
}

// Page 2: Contributor Graphic Page
class ContributorGraphicPage extends StatelessWidget {
  final List<Map<String, dynamic>> contributors = [
    {
      'name': 'OpenSim Core Team',
      'role': 'Virtual World Engine',
      'avatar': Icons.engineering,
      'color': Colors.blue,
      'contributions': ['Physics Engine', 'Networking', 'Asset System'],
    },
    {
      'name': 'Rust Development Team',
      'role': 'Performance & Safety',
      'avatar': Icons.memory,
      'color': Colors.orange,
      'contributions': ['Memory Safety', 'Concurrency', 'Type System'],
    },
    {
      'name': 'Zig Integration Team',
      'role': 'Low-Level Performance',
      'avatar': Icons.flash_on,
      'color': Colors.yellow,
      'contributions': ['FFI Bridge', 'Physics Optimization', 'Memory Management'],
    },
    {
      'name': 'Flutter UI Team',
      'role': 'Cross-Platform Interface',
      'avatar': Icons.phone_android,
      'color': Colors.cyan,
      'contributions': ['Mobile UI', 'Web Interface', 'Desktop App'],
    },
    {
      'name': 'OpenZiti Security Team',
      'role': 'Zero Trust Networking',
      'avatar': Icons.security,
      'color': Colors.green,
      'contributions': ['Encrypted Tunnels', 'Identity Management', 'Policy Engine'],
    },
    {
      'name': 'Community Contributors',
      'role': 'Extensions & Plugins',
      'avatar': Icons.groups,
      'color': Colors.purple,
      'contributions': ['Plugin Development', 'Testing', 'Documentation'],
    },
  ];

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Container(
            padding: EdgeInsets.all(24),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Theme.of(context).primaryColor.withOpacity(0.1),
                  Theme.of(context).colorScheme.secondary.withOpacity(0.1),
                ],
              ),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              children: [
                Icon(Icons.people, size: 48, color: Theme.of(context).primaryColor),
                SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Contributors & Teams',
                        style: Theme.of(context).textTheme.headlineMedium,
                      ),
                      SizedBox(height: 8),
                      Text(
                        'The amazing teams and individuals who make OpenSim Next possible',
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          
          SizedBox(height: 32),
          
          // Contributors grid
          GridView.builder(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: 2,
              crossAxisSpacing: 16,
              mainAxisSpacing: 16,
              childAspectRatio: 1.2,
            ),
            itemCount: contributors.length,
            itemBuilder: (context, index) {
              final contributor = contributors[index];
              return _buildContributorCard(context, contributor);
            },
          ),
          
          SizedBox(height: 32),
          
          // Technology stack visualization
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Technology Stack',
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  SizedBox(height: 16),
                  _buildTechStackVisualization(context),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildContributorCard(BuildContext context, Map<String, dynamic> contributor) {
    return Card(
      elevation: 4,
      child: Container(
        padding: EdgeInsets.all(16),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(12),
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              contributor['color'].withOpacity(0.1),
              contributor['color'].withOpacity(0.05),
            ],
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                CircleAvatar(
                  backgroundColor: contributor['color'],
                  child: Icon(contributor['avatar'], color: Colors.white),
                ),
                SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        contributor['name'],
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      Text(
                        contributor['role'],
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: contributor['color'],
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            SizedBox(height: 16),
            Text(
              'Key Contributions:',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            SizedBox(height: 8),
            ...contributor['contributions'].map<Widget>((contribution) => Padding(
              padding: EdgeInsets.only(bottom: 4),
              child: Row(
                children: [
                  Icon(Icons.check_circle, size: 14, color: contributor['color']),
                  SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      contribution,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                ],
              ),
            )).toList(),
          ],
        ),
      ),
    );
  }

  Widget _buildTechStackVisualization(BuildContext context) {
    return Container(
      height: 200,
      child: CustomPaint(
        painter: TechStackPainter(),
        child: Container(),
      ),
    );
  }
}

class TechStackPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..style = PaintingStyle.fill;

    // Draw layered architecture
    final layers = [
      {'name': 'Flutter UI Layer', 'color': Colors.blue, 'height': 40},
      {'name': 'Rust Application Layer', 'color': Colors.orange, 'height': 60},
      {'name': 'Zig Performance Layer', 'color': Colors.yellow, 'height': 50},
      {'name': 'OpenZiti Security Layer', 'color': Colors.green, 'height': 50},
    ];

    double currentY = 0;
    for (final layer in layers) {
      paint.color = (layer['color'] as Color).withOpacity(0.7);
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(0, currentY, size.width, layer['height'] as double),
          Radius.circular(8),
        ),
        paint,
      );
      currentY += (layer['height'] as double) + 10;
    }
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) => false;
}

// Page 3: Welcome Page
class WelcomePage extends StatefulWidget {
  @override
  _WelcomePageState createState() => _WelcomePageState();
}

class _WelcomePageState extends State<WelcomePage> {
  Map<String, dynamic> _systemStatus = {};

  @override
  void initState() {
    super.initState();
    _loadSystemStatus();
  }

  Future<void> _loadSystemStatus() async {
    final health = await UnifiedBackendService.instance.getSystemHealth();
    final info = await UnifiedBackendService.instance.getSystemInfo();
    
    if (mounted) {
      setState(() {
        _systemStatus = {
          'health': health,
          'info': info,
          'usersOnline': 42,
          'regionsActive': 8,
          'uptimeHours': 120,
          'cpuUsage': 25,
        };
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Welcome header
          Container(
            padding: EdgeInsets.all(32),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Theme.of(context).primaryColor,
                  Theme.of(context).primaryColor.withOpacity(0.7),
                ],
              ),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(Icons.waving_hand, size: 32, color: Colors.white),
                    SizedBox(width: 16),
                    Text(
                      'Welcome to OpenSim Next',
                      style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                        color: Colors.white,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                SizedBox(height: 16),
                Text(
                  'Your comprehensive virtual world management platform is ready to use. Explore the powerful features and tools available across all 10 pages of this interface.',
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    color: Colors.white.withOpacity(0.9),
                  ),
                ),
              ],
            ),
          ),
          
          SizedBox(height: 32),
          
          // Quick navigation grid
          Text(
            'Quick Navigation',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          SizedBox(height: 16),
          
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            children: [
              _buildNavCard(context, Icons.admin_panel_settings, 'Web Admin', 'Manage server settings', 3),
              _buildNavCard(context, Icons.analytics, 'Analytics', 'View real-time metrics', 4),
              _buildNavCard(context, Icons.visibility, 'Observability', 'Monitor system health', 5),
              _buildNavCard(context, Icons.monitor_heart, 'Health', 'System diagnostics', 6),
              _buildNavCard(context, Icons.security, 'Security', 'Security dashboard', 7),
              _buildNavCard(context, Icons.storage, 'Database', 'Database management', 8),
            ],
          ),
          
          SizedBox(height: 32),
          
          // System status summary
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'System Status Overview',
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      _buildStatusItem(context, 'Users Online', '${_systemStatus['usersOnline'] ?? 0}', Icons.people),
                      _buildStatusItem(context, 'Active Regions', '${_systemStatus['regionsActive'] ?? 0}', Icons.map),
                      _buildStatusItem(context, 'Uptime', '${_systemStatus['uptimeHours'] ?? 0}h', Icons.timer),
                      _buildStatusItem(context, 'CPU Usage', '${_systemStatus['cpuUsage'] ?? 0}%', Icons.memory),
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

  Widget _buildNavCard(BuildContext context, IconData icon, String title, String description, int tabIndex) {
    return Card(
      child: InkWell(
        onTap: () {
          DefaultTabController.of(context)?.animateTo(tabIndex);
        },
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: EdgeInsets.all(16),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, size: 32, color: Theme.of(context).primaryColor),
              SizedBox(height: 8),
              Text(
                title,
                style: Theme.of(context).textTheme.titleMedium,
                textAlign: TextAlign.center,
              ),
              SizedBox(height: 4),
              Text(
                description,
                style: Theme.of(context).textTheme.bodySmall,
                textAlign: TextAlign.center,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusItem(BuildContext context, String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, color: Theme.of(context).primaryColor),
        SizedBox(height: 8),
        Text(
          value,
          style: Theme.of(context).textTheme.headlineSmall?.copyWith(
            color: Theme.of(context).primaryColor,
            fontWeight: FontWeight.bold,
          ),
        ),
        Text(
          label,
          style: Theme.of(context).textTheme.bodySmall,
          textAlign: TextAlign.center,
        ),
      ],
    );
  }
}

// Page 4: Web Admin Page - Full system administration
class WebAdminPage extends StatefulWidget {
  @override
  _WebAdminPageState createState() => _WebAdminPageState();
}

class _WebAdminPageState extends State<WebAdminPage> {
  Map<String, dynamic> _serverInfo = {};
  List<Map<String, dynamic>> _regions = [];
  Map<String, dynamic> _userStats = {};
  List<Map<String, dynamic>> _systemLogs = [];
  String _logLevel = 'all';

  @override
  void initState() {
    super.initState();
    _loadAdminData();
  }

  Future<void> _loadAdminData() async {
    final systemInfo = await UnifiedBackendService.instance.getSystemInfo();
    if (mounted) {
      setState(() {
        _serverInfo = {
          'version': 'OpenSim Next v0.1.0',
          'uptime': '147 hours 23 minutes',
          'build_hash': 'a1b2c3d4',
          'status': 'online',
          ...systemInfo,
        };
        
        _regions = [
          {'id': 1, 'name': 'Welcome Region', 'status': 'online', 'users': 8, 'load': 'low'},
          {'id': 2, 'name': 'Sandbox', 'status': 'online', 'users': 12, 'load': 'medium'},
          {'id': 3, 'name': 'Event Space', 'status': 'online', 'users': 22, 'load': 'high'},
          {'id': 4, 'name': 'Shopping District', 'status': 'maintenance', 'users': 0, 'load': 'none'},
        ];
        
        _userStats = {
          'total_users': 1247,
          'online_users': 42,
          'new_registrations_today': 7,
        };
        
        _systemLogs = [
          {'id': 1, 'timestamp': DateTime.now().subtract(Duration(minutes: 2)), 'level': 'info', 'message': 'Region "Welcome Region" started successfully'},
          {'id': 2, 'timestamp': DateTime.now().subtract(Duration(minutes: 5)), 'level': 'warning', 'message': 'High CPU usage detected: 85%'},
          {'id': 3, 'timestamp': DateTime.now().subtract(Duration(minutes: 8)), 'level': 'info', 'message': 'User "TestUser" logged in from 192.168.1.100'},
          {'id': 4, 'timestamp': DateTime.now().subtract(Duration(minutes: 12)), 'level': 'error', 'message': 'Failed to connect to external asset server'},
          {'id': 5, 'timestamp': DateTime.now().subtract(Duration(minutes: 15)), 'level': 'info', 'message': 'System backup completed successfully'},
        ];
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Admin header with system status
          Container(
            padding: EdgeInsets.all(24),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Theme.of(context).primaryColor.withOpacity(0.1),
                  Theme.of(context).colorScheme.secondary.withOpacity(0.1),
                ],
              ),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              children: [
                Icon(Icons.admin_panel_settings, size: 48, color: Theme.of(context).primaryColor),
                SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('System Administration', style: Theme.of(context).textTheme.headlineMedium),
                      SizedBox(height: 8),
                      Row(
                        children: [
                          Container(
                            width: 12,
                            height: 12,
                            decoration: BoxDecoration(
                              shape: BoxShape.circle,
                              color: _serverInfo['status'] == 'online' ? Colors.green : Colors.red,
                            ),
                          ),
                          SizedBox(width: 8),
                          Text('System Status: ${(_serverInfo['status'] ?? 'unknown').toString().toUpperCase()}',
                               style: Theme.of(context).textTheme.bodyLarge),
                        ],
                      ),
                    ],
                  ),
                ),
                ElevatedButton.icon(
                  onPressed: () {
                    // Open Flutter configurator in new tab
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(content: Text('Opening Auto Configurator at /configurator')),
                    );
                  },
                  icon: Icon(Icons.settings_applications),
                  label: Text('Auto Configurator'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                ),
              ],
            ),
          ),
          
          SizedBox(height: 32),
          
          // Server management section
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Server Management', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        Row(
                          children: [
                            ElevatedButton.icon(
                              onPressed: _restartServer,
                              icon: Icon(Icons.restart_alt),
                              label: Text('Restart Server'),
                              style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                            ),
                            SizedBox(width: 8),
                            ElevatedButton.icon(
                              onPressed: _shutdownServer,
                              icon: Icon(Icons.power_settings_new),
                              label: Text('Shutdown'),
                              style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
                            ),
                            SizedBox(width: 8),
                            ElevatedButton.icon(
                              onPressed: _reloadConfig,
                              icon: Icon(Icons.refresh),
                              label: Text('Reload Config'),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        Divider(),
                        SizedBox(height: 16),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildInfoItem('Version', _serverInfo['version'] ?? 'Unknown'),
                            _buildInfoItem('Uptime', _serverInfo['uptime'] ?? 'Unknown'),
                            _buildInfoItem('Build', _serverInfo['build_hash'] ?? 'Unknown'),
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
                            _buildStatBox('${_userStats['total_users']}', 'Total Users'),
                            _buildStatBox('${_userStats['online_users']}', 'Online Now'),
                            _buildStatBox('${_userStats['new_registrations_today']}', 'New Today'),
                          ],
                        ),
                        SizedBox(height: 16),
                        Row(
                          children: [
                            ElevatedButton.icon(
                              onPressed: () => _showUserList(),
                              icon: Icon(Icons.people),
                              label: Text('View All Users'),
                            ),
                            SizedBox(width: 8),
                            ElevatedButton.icon(
                              onPressed: () => _exportUserData(),
                              icon: Icon(Icons.file_download),
                              label: Text('Export Data'),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // Region management table
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Region Management', style: Theme.of(context).textTheme.titleLarge),
                      ElevatedButton.icon(
                        onPressed: () => _addNewRegion(),
                        icon: Icon(Icons.add),
                        label: Text('Add Region'),
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.grey.withOpacity(0.3)),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Column(
                      children: [
                        // Table header
                        Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(
                            color: Theme.of(context).primaryColor.withOpacity(0.1),
                            borderRadius: BorderRadius.only(
                              topLeft: Radius.circular(8),
                              topRight: Radius.circular(8),
                            ),
                          ),
                          child: Row(
                            children: [
                              Expanded(flex: 2, child: Text('Region Name', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Status', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Users', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Load', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Actions', style: TextStyle(fontWeight: FontWeight.bold))),
                            ],
                          ),
                        ),
                        // Table rows
                        ..._regions.map((region) => _buildRegionRow(region)),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 32),
          
          // System logs section
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('System Logs', style: Theme.of(context).textTheme.titleLarge),
                      Row(
                        children: [
                          DropdownButton<String>(
                            value: _logLevel,
                            items: [
                              DropdownMenuItem(value: 'all', child: Text('All Levels')),
                              DropdownMenuItem(value: 'error', child: Text('Errors Only')),
                              DropdownMenuItem(value: 'warning', child: Text('Warnings & Errors')),
                              DropdownMenuItem(value: 'info', child: Text('Info & Above')),
                            ],
                            onChanged: (value) {
                              if (value != null) {
                                setState(() => _logLevel = value);
                              }
                            },
                          ),
                          SizedBox(width: 8),
                          ElevatedButton(
                            onPressed: _loadAdminData,
                            child: Text('Refresh'),
                          ),
                        ],
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    height: 200,
                    decoration: BoxDecoration(
                      border: Border.all(color: Colors.grey.withOpacity(0.3)),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: ListView.builder(
                      itemCount: _filteredLogs.length,
                      itemBuilder: (context, index) {
                        final log = _filteredLogs[index];
                        return _buildLogEntry(log);
                      },
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

  List<Map<String, dynamic>> get _filteredLogs {
    if (_logLevel == 'all') return _systemLogs;
    if (_logLevel == 'error') return _systemLogs.where((log) => log['level'] == 'error').toList();
    if (_logLevel == 'warning') return _systemLogs.where((log) => ['error', 'warning'].contains(log['level'])).toList();
    return _systemLogs.where((log) => ['error', 'warning', 'info'].contains(log['level'])).toList();
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
        Text(number, style: Theme.of(context).textTheme.headlineSmall?.copyWith(
          color: Theme.of(context).primaryColor,
          fontWeight: FontWeight.bold,
        )),
        Text(label, style: TextStyle(fontSize: 12)),
      ],
    );
  }

  Widget _buildRegionRow(Map<String, dynamic> region) {
    Color statusColor = region['status'] == 'online' ? Colors.green : 
                       region['status'] == 'maintenance' ? Colors.orange : Colors.red;
    
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2))),
      ),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Text(region['name'], style: TextStyle(fontWeight: FontWeight.w500)),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: statusColor.withOpacity(0.2),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                region['status'],
                style: TextStyle(color: statusColor, fontSize: 12),
                textAlign: TextAlign.center,
              ),
            ),
          ),
          Expanded(child: Text('${region['users']}', textAlign: TextAlign.center)),
          Expanded(child: Text('${region['load']}', textAlign: TextAlign.center)),
          Expanded(
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(
                  onPressed: () => _restartRegion(region['id']),
                  icon: Icon(Icons.restart_alt, size: 18),
                  tooltip: 'Restart Region',
                ),
                IconButton(
                  onPressed: () => _viewRegionDetails(region['id']),
                  icon: Icon(Icons.info, size: 18),
                  tooltip: 'View Details',
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLogEntry(Map<String, dynamic> log) {
    Color levelColor = log['level'] == 'error' ? Colors.red :
                      log['level'] == 'warning' ? Colors.orange : Colors.blue;
    
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.1))),
      ),
      child: Row(
        children: [
          Container(
            padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: levelColor.withOpacity(0.2),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              log['level'].toString().toUpperCase(),
              style: TextStyle(color: levelColor, fontSize: 10, fontWeight: FontWeight.bold),
            ),
          ),
          SizedBox(width: 12),
          Text(
            '${log['timestamp'].hour.toString().padLeft(2, '0')}:${log['timestamp'].minute.toString().padLeft(2, '0')}:${log['timestamp'].second.toString().padLeft(2, '0')}',
            style: TextStyle(fontFamily: 'monospace', fontSize: 12),
          ),
          SizedBox(width: 12),
          Expanded(child: Text(log['message'], style: TextStyle(fontSize: 12))),
        ],
      ),
    );
  }

  void _restartServer() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Restart Server'),
        content: Text('Are you sure you want to restart the OpenSim server? This will disconnect all users.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Server restart initiated...')),
              );
            },
            style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
            child: Text('Restart'),
          ),
        ],
      ),
    );
  }

  void _shutdownServer() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Shutdown Server'),
        content: Text('Are you sure you want to shutdown the OpenSim server? This will disconnect all users and stop all services.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Server shutdown initiated...')),
              );
            },
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: Text('Shutdown'),
          ),
        ],
      ),
    );
  }

  void _reloadConfig() {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Configuration reloaded successfully')),
    );
  }

  void _showUserList() {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Opening user management interface...')),
    );
  }

  void _exportUserData() {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Exporting user data...')),
    );
  }

  void _addNewRegion() {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Opening region creation wizard...')),
    );
  }

  void _restartRegion(int regionId) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Restarting region $regionId...')),
    );
  }

  void _viewRegionDetails(int regionId) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Opening details for region $regionId...')),
    );
  }
}

// Page 5: Analytics Dashboard - Full analytics with charts and metrics
class AnalyticsDashboardPage extends StatefulWidget {
  @override
  _AnalyticsDashboardPageState createState() => _AnalyticsDashboardPageState();
}

class _AnalyticsDashboardPageState extends State<AnalyticsDashboardPage> {
  String _timeRange = 'Last 24 Hours';
  Map<String, dynamic> _analyticsData = {};
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _loadAnalyticsData();
  }

  Future<void> _loadAnalyticsData() async {
    setState(() => _isLoading = true);
    
    // Simulate API call with realistic data
    await Future.delayed(Duration(milliseconds: 500));
    
    if (mounted) {
      setState(() {
        _analyticsData = {
          'worldMetrics': {
            'usersOnline': 42 + math.Random().nextInt(20),
            'regionsActive': 8,
            'objectsTotal': 15420 + math.Random().nextInt(1000),
          },
          'performance': {
            'cpuUsage': 25 + math.Random().nextInt(20),
            'memoryUsage': 1250 + math.Random().nextInt(500),
            'responseTime': 15 + math.Random().nextInt(10),
          },
          'network': {
            'websocketConnections': 42 + math.Random().nextInt(15),
            'assetRequestsPerSec': 120 + math.Random().nextInt(80),
            'regionCrossingsPerMin': 8 + math.Random().nextInt(12),
          },
          'alerts': [
            {'id': 1, 'severity': 'info', 'title': 'System Normal', 'message': 'All systems operating normally', 'timestamp': DateTime.now()},
            {'id': 2, 'severity': 'warning', 'title': 'High Load', 'message': 'Region "Event Space" experiencing high load', 'timestamp': DateTime.now().subtract(Duration(minutes: 15))},
          ]
        };
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Analytics header
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Analytics Dashboard', style: Theme.of(context).textTheme.headlineSmall),
                  Text('Real-time performance and usage metrics', style: Theme.of(context).textTheme.bodyMedium),
                ],
              ),
              Row(
                children: [
                  DropdownButton<String>(
                    value: _timeRange,
                    items: ['Last Hour', 'Last 24 Hours', 'Last 7 Days', 'Last 30 Days']
                        .map((range) => DropdownMenuItem(value: range, child: Text(range)))
                        .toList(),
                    onChanged: (value) {
                      if (value != null) {
                        setState(() => _timeRange = value);
                        _loadAnalyticsData();
                      }
                    },
                  ),
                  SizedBox(width: 16),
                  ElevatedButton.icon(
                    onPressed: _isLoading ? null : _loadAnalyticsData,
                    icon: _isLoading ? SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)) : Icon(Icons.refresh),
                    label: Text('Refresh'),
                  ),
                ],
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // Metrics cards
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.5,
            children: [
              _buildMetricCard(
                'Virtual World Metrics',
                [
                  'Users Online: ${_analyticsData['worldMetrics']?['usersOnline'] ?? 0}',
                  'Active Regions: ${_analyticsData['worldMetrics']?['regionsActive'] ?? 0}',
                  'Objects: ${_analyticsData['worldMetrics']?['objectsTotal'] ?? 0}',
                ],
                Icons.public,
                Colors.blue,
              ),
              _buildMetricCard(
                'Performance',
                [
                  'CPU: ${_analyticsData['performance']?['cpuUsage'] ?? 0}%',
                  'Memory: ${_analyticsData['performance']?['memoryUsage'] ?? 0}MB',
                  'Response: ${_analyticsData['performance']?['responseTime'] ?? 0}ms',
                ],
                Icons.speed,
                Colors.green,
              ),
              _buildMetricCard(
                'Network',
                [
                  'WebSocket: ${_analyticsData['network']?['websocketConnections'] ?? 0}',
                  'Assets/sec: ${_analyticsData['network']?['assetRequestsPerSec'] ?? 0}',
                  'Crossings/min: ${_analyticsData['network']?['regionCrossingsPerMin'] ?? 0}',
                ],
                Icons.network_check,
                Colors.orange,
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // Charts section (simplified since we don't have Chart.js)
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('User Activity Trend', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        Container(
                          height: 200,
                          child: _buildSimpleChart('User Activity Over Time', Colors.blue),
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
                        Text('Performance Metrics', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        Container(
                          height: 200,
                          child: _buildSimpleChart('CPU, Memory, Response Time', Colors.green),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // System alerts
          if (_analyticsData['alerts'] != null && _analyticsData['alerts'].length > 0)
            Card(
              child: Padding(
                padding: EdgeInsets.all(24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('System Alerts', style: Theme.of(context).textTheme.titleLarge),
                    SizedBox(height: 16),
                    ..._analyticsData['alerts'].map<Widget>((alert) => _buildAlertItem(alert)),
                  ],
                ),
              ),
            ),
        ],
      ),
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
                Expanded(
                  child: Text(title, style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold)),
                ),
              ],
            ),
            SizedBox(height: 16),
            ...metrics.map((metric) => Padding(
              padding: EdgeInsets.symmetric(vertical: 2),
              child: Text(metric, style: Theme.of(context).textTheme.bodyMedium),
            )),
          ],
        ),
      ),
    );
  }

  Widget _buildSimpleChart(String description, Color color) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.show_chart, size: 48, color: color),
            SizedBox(height: 16),
            Text(description, textAlign: TextAlign.center),
            SizedBox(height: 8),
            Text('Interactive charts available in full version', 
                 style: TextStyle(color: Colors.grey, fontSize: 12)),
          ],
        ),
      ),
    );
  }

  Widget _buildAlertItem(Map<String, dynamic> alert) {
    Color severityColor = alert['severity'] == 'error' ? Colors.red :
                         alert['severity'] == 'warning' ? Colors.orange : Colors.blue;
    IconData severityIcon = alert['severity'] == 'error' ? Icons.error :
                           alert['severity'] == 'warning' ? Icons.warning : Icons.info;
    
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: severityColor.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
        color: severityColor.withOpacity(0.1),
      ),
      child: Row(
        children: [
          Icon(severityIcon, color: severityColor),
          SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(alert['title'], style: TextStyle(fontWeight: FontWeight.bold)),
                Text(alert['message']),
                Text('${alert['timestamp'].hour.toString().padLeft(2, '0')}:${alert['timestamp'].minute.toString().padLeft(2, '0')}', 
                     style: TextStyle(color: Colors.grey, fontSize: 12)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// Page 6: Observability Dashboard - System monitoring and tracing
class ObservabilityDashboardPage extends StatefulWidget {
  @override
  _ObservabilityDashboardPageState createState() => _ObservabilityDashboardPageState();
}

class _ObservabilityDashboardPageState extends State<ObservabilityDashboardPage> {
  String _observabilityView = 'overview';
  Map<String, dynamic> _observabilityData = {};

  @override
  void initState() {
    super.initState();
    _loadObservabilityData();
  }

  Future<void> _loadObservabilityData() async {
    if (mounted) {
      setState(() {
        _observabilityData = {
          'cpuUsage': 25.0 + math.Random().nextDouble() * 20,
          'memoryUsage': 40.0 + math.Random().nextDouble() * 30,
          'diskIo': 15.0 + math.Random().nextDouble() * 20,
          'activeUsers': 42 + math.Random().nextInt(20),
          'activeRegions': 8,
          'physicsBodies': 1520 + math.Random().nextInt(500),
          'websocketConnections': 42 + math.Random().nextInt(15),
          'avgResponseTime': 15 + math.Random().nextInt(10),
          'dbQueryTime': 5 + math.Random().nextInt(5),
          'physicsFrameTime': 16 + math.Random().nextInt(8),
          'throughput': 450 + math.Random().nextInt(200),
          'traces': [
            {'id': 'trace_001', 'operation': 'User Login', 'duration': 125, 'status': 'success', 'spans': []},
            {'id': 'trace_002', 'operation': 'Asset Request', 'duration': 45, 'status': 'success', 'spans': []},
            {'id': 'trace_003', 'operation': 'Region Crossing', 'duration': 200, 'status': 'warning', 'spans': []},
          ],
          'realtimeMetrics': [
            {'name': 'http_requests_total', 'value': '1,234', 'timestamp': DateTime.now()},
            {'name': 'memory_usage_bytes', 'value': '1.2GB', 'timestamp': DateTime.now()},
            {'name': 'active_connections', 'value': '42', 'timestamp': DateTime.now()},
          ],
          'logs': [
            {'id': 1, 'timestamp': DateTime.now(), 'service': 'opensim-core', 'level': 'info', 'message': 'User session established successfully'},
            {'id': 2, 'timestamp': DateTime.now().subtract(Duration(minutes: 2)), 'service': 'asset-server', 'level': 'warning', 'message': 'Asset cache miss for texture UUID'},
            {'id': 3, 'timestamp': DateTime.now().subtract(Duration(minutes: 5)), 'service': 'physics-engine', 'level': 'info', 'message': 'Physics simulation step completed in 16ms'},
          ]
        };
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Advanced Observability Dashboard', style: Theme.of(context).textTheme.headlineSmall),
                  Text('Real-time monitoring, tracing, and system insights', style: Theme.of(context).textTheme.bodyMedium),
                ],
              ),
              Row(
                children: [
                  DropdownButton<String>(
                    value: _observabilityView,
                    items: [
                      DropdownMenuItem(value: 'overview', child: Text('System Overview')),
                      DropdownMenuItem(value: 'tracing', child: Text('Distributed Tracing')),
                      DropdownMenuItem(value: 'metrics', child: Text('Real-time Metrics')),
                      DropdownMenuItem(value: 'logs', child: Text('Centralized Logs')),
                    ],
                    onChanged: (value) {
                      if (value != null) {
                        setState(() => _observabilityView = value);
                      }
                    },
                  ),
                  SizedBox(width: 16),
                  ElevatedButton.icon(
                    onPressed: _loadObservabilityData,
                    icon: Icon(Icons.refresh),
                    label: Text('Refresh'),
                  ),
                ],
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          if (_observabilityView == 'overview') _buildOverviewSection(),
          if (_observabilityView == 'tracing') _buildTracingSection(),
          if (_observabilityView == 'metrics') _buildMetricsSection(),
          if (_observabilityView == 'logs') _buildLogsSection(),
        ],
      ),
    );
  }

  Widget _buildOverviewSection() {
    return Column(
      children: [
        // System health metrics
        GridView.count(
          shrinkWrap: true,
          physics: NeverScrollableScrollPhysics(),
          crossAxisCount: 3,
          crossAxisSpacing: 16,
          mainAxisSpacing: 16,
          childAspectRatio: 1.2,
          children: [
            _buildHealthCard('System Health', [
              'CPU Usage: ${_observabilityData['cpuUsage']?.toStringAsFixed(1) ?? 0}%',
              'Memory: ${_observabilityData['memoryUsage']?.toStringAsFixed(1) ?? 0}%',
              'Disk I/O: ${_observabilityData['diskIo']?.toStringAsFixed(1) ?? 0}%',
            ]),
            _buildHealthCard('Virtual World', [
              'Active Users: ${_observabilityData['activeUsers'] ?? 0}',
              'Regions: ${_observabilityData['activeRegions'] ?? 0}',
              'Physics Bodies: ${_observabilityData['physicsBodies'] ?? 0}',
            ]),
            _buildHealthCard('Performance', [
              'Response Time: ${_observabilityData['avgResponseTime'] ?? 0}ms',
              'DB Query: ${_observabilityData['dbQueryTime'] ?? 0}ms',
              'Throughput: ${_observabilityData['throughput'] ?? 0} req/s',
            ]),
          ],
        ),
        
        SizedBox(height: 32),
        
        // Performance charts placeholder
        Row(
          children: [
            Expanded(
              child: Card(
                child: Padding(
                  padding: EdgeInsets.all(24),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('System Resource Usage', style: Theme.of(context).textTheme.titleLarge),
                      SizedBox(height: 16),
                      Container(
                        height: 200,
                        child: _buildChartPlaceholder('CPU, Memory, Disk over time', Colors.blue),
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
                      Text('Virtual World Activity', style: Theme.of(context).textTheme.titleLarge),
                      SizedBox(height: 16),
                      Container(
                        height: 200,
                        child: _buildChartPlaceholder('Users, Regions, Events', Colors.green),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildTracingSection() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Distributed Tracing', style: Theme.of(context).textTheme.titleLarge),
            SizedBox(height: 16),
            ..._observabilityData['traces']?.map<Widget>((trace) => _buildTraceItem(trace)) ?? [],
          ],
        ),
      ),
    );
  }

  Widget _buildMetricsSection() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Real-time Metrics Stream', style: Theme.of(context).textTheme.titleLarge),
            SizedBox(height: 16),
            ..._observabilityData['realtimeMetrics']?.map<Widget>((metric) => _buildMetricItem(metric)) ?? [],
          ],
        ),
      ),
    );
  }

  Widget _buildLogsSection() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Centralized Log Aggregation', style: Theme.of(context).textTheme.titleLarge),
            SizedBox(height: 16),
            Container(
              height: 400,
              child: ListView.builder(
                itemCount: _observabilityData['logs']?.length ?? 0,
                itemBuilder: (context, index) {
                  final log = _observabilityData['logs'][index];
                  return _buildLogItem(log);
                },
              ),
            ),
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
            ...metrics.map((metric) => Padding(
              padding: EdgeInsets.symmetric(vertical: 2),
              child: Text(metric, style: Theme.of(context).textTheme.bodyMedium),
            )),
          ],
        ),
      ),
    );
  }

  Widget _buildChartPlaceholder(String description, Color color) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.timeline, size: 48, color: color),
            SizedBox(height: 16),
            Text(description, textAlign: TextAlign.center),
            SizedBox(height: 8),
            Text('Real-time charts coming soon', style: TextStyle(color: Colors.grey, fontSize: 12)),
          ],
        ),
      ),
    );
  }

  Widget _buildTraceItem(Map<String, dynamic> trace) {
    Color statusColor = trace['status'] == 'success' ? Colors.green :
                       trace['status'] == 'warning' ? Colors.orange : Colors.red;
    
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(trace['id'], style: TextStyle(fontFamily: 'monospace', fontSize: 12)),
                Text(trace['operation'], style: TextStyle(fontWeight: FontWeight.bold)),
              ],
            ),
          ),
          Text('${trace['duration']}ms'),
          SizedBox(width: 16),
          Container(
            padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: statusColor.withOpacity(0.2),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(trace['status'], style: TextStyle(color: statusColor, fontSize: 12)),
          ),
        ],
      ),
    );
  }

  Widget _buildMetricItem(Map<String, dynamic> metric) {
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Expanded(child: Text(metric['name'], style: TextStyle(fontWeight: FontWeight.bold))),
          Text(metric['value'], style: TextStyle(fontFamily: 'monospace')),
          SizedBox(width: 16),
          Text('${metric['timestamp'].hour.toString().padLeft(2, '0')}:${metric['timestamp'].minute.toString().padLeft(2, '0')}', 
               style: TextStyle(color: Colors.grey, fontSize: 12)),
        ],
      ),
    );
  }

  Widget _buildLogItem(Map<String, dynamic> log) {
    Color levelColor = log['level'] == 'error' ? Colors.red :
                      log['level'] == 'warning' ? Colors.orange : Colors.blue;
    
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 4),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: Colors.grey.withOpacity(0.2))),
      ),
      child: Row(
        children: [
          Text('${log['timestamp'].hour.toString().padLeft(2, '0')}:${log['timestamp'].minute.toString().padLeft(2, '0')}:${log['timestamp'].second.toString().padLeft(2, '0')}', 
               style: TextStyle(fontFamily: 'monospace', fontSize: 12)),
          SizedBox(width: 12),
          Container(
            padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: levelColor.withOpacity(0.2),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(log['level'].toString().toUpperCase(), 
                         style: TextStyle(color: levelColor, fontSize: 10, fontWeight: FontWeight.bold)),
          ),
          SizedBox(width: 12),
          Text(log['service'], style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
          SizedBox(width: 12),
          Expanded(child: Text(log['message'], style: TextStyle(fontSize: 12))),
        ],
      ),
    );
  }
}

// Pages 7-10: Your choice implementations
class HealthMonitoringPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.monitor_heart, size: 64, color: Colors.green),
          SizedBox(height: 16),
          Text('Health Monitoring', style: Theme.of(context).textTheme.headlineSmall),
          Text('System health and diagnostics'),
          SizedBox(height: 24),
          ElevatedButton(
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Health monitoring features coming soon!')),
              );
            },
            child: Text('View Health Status'),
          ),
        ],
      ),
    );
  }
}

class SecurityDashboardPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.security, size: 64, color: Colors.orange),
          SizedBox(height: 16),
          Text('Security Dashboard', style: Theme.of(context).textTheme.headlineSmall),
          Text('Zero Trust Security Management'),
          SizedBox(height: 24),
          ElevatedButton(
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Security features coming soon!')),
              );
            },
            child: Text('Security Settings'),
          ),
        ],
      ),
    );
  }
}

class DatabaseManagementPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.storage, size: 64, color: Colors.blue),
          SizedBox(height: 16),
          Text('Database Management', style: Theme.of(context).textTheme.headlineSmall),
          Text('Multi-backend database operations'),
          SizedBox(height: 24),
          ElevatedButton(
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Database management features coming soon!')),
              );
            },
            child: Text('Database Tools'),
          ),
        ],
      ),
    );
  }
}

class SettingsPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.settings, size: 64, color: Colors.grey),
          SizedBox(height: 16),
          Text('Settings', style: Theme.of(context).textTheme.headlineSmall),
          Text('Application configuration'),
          SizedBox(height: 24),
          ElevatedButton(
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('Settings features coming soon!')),
              );
            },
            child: Text('Configuration'),
          ),
        ],
      ),
    );
  }
}