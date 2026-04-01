import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'services/unified_backend_service.dart';
import 'widgets/vue_webview_integration.dart';
import 'models/deployment_models.dart';
import 'theme/observability_themes.dart';
import 'package:fl_chart/fl_chart.dart';
import 'dart:async';

void main() {
  runApp(EnhancedOpenSimApp());
}

class EnhancedOpenSimApp extends StatefulWidget {
  @override
  _EnhancedOpenSimAppState createState() => _EnhancedOpenSimAppState();
}

class _EnhancedOpenSimAppState extends State<EnhancedOpenSimApp> {
  String _currentTheme = ObservabilityThemes.system;

  @override
  void initState() {
    super.initState();
    // Initialize backend service
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
      title: 'OpenSim Next - Unified 10-Page Platform',
      theme: _currentTheme == ObservabilityThemes.system || _currentTheme == ObservabilityThemes.light
          ? ObservabilityThemes.lightTheme
          : ObservabilityThemes.getTheme(_currentTheme),
      darkTheme: _currentTheme == ObservabilityThemes.system
          ? ObservabilityThemes.darkTheme
          : ObservabilityThemes.getTheme(_currentTheme),
      themeMode: ObservabilityThemes.getThemeMode(_currentTheme),
      home: UnifiedDashboard(
        currentTheme: _currentTheme,
        onThemeChanged: _changeTheme,
      ),
    );
  }
}

class UnifiedDashboard extends StatefulWidget {
  final String currentTheme;
  final Function(String) onThemeChanged;

  const UnifiedDashboard({
    Key? key,
    required this.currentTheme,
    required this.onThemeChanged,
  }) : super(key: key);

  @override
  _UnifiedDashboardState createState() => _UnifiedDashboardState();
}

class _UnifiedDashboardState extends State<UnifiedDashboard> with TickerProviderStateMixin {
  late TabController _tabController;
  late StreamSubscription _realtimeSubscription;
  Map<String, dynamic> _realtimeData = {};

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 10, vsync: this);
    
    // Subscribe to real-time data
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
            Text('OpenSim Next - Unified Platform'),
            Text(
              '10-Page Comprehensive Virtual World Management',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
          // Real-time status indicator
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
          // Theme selector
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
          // Refresh button
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: () {
              // Refresh all data
              setState(() {});
            },
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          isScrollable: true,
          tabs: [
            Tab(icon: Icon(Icons.home), text: 'Splash'),
            Tab(icon: Icon(Icons.analytics), text: 'Analytics'),
            Tab(icon: Icon(Icons.admin_panel_settings), text: 'Admin'),
            Tab(icon: Icon(Icons.visibility), text: 'Observability'),
            Tab(icon: Icon(Icons.monitor_heart), text: 'Health'),
            Tab(icon: Icon(Icons.security), text: 'Security'),
            Tab(icon: Icon(Icons.storage), text: 'Database'),
            Tab(icon: Icon(Icons.network_check), text: 'Network'),
            Tab(icon: Icon(Icons.settings), text: 'Settings'),
            Tab(icon: Icon(Icons.help), text: 'Documentation'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          EnhancedSplashPage(),
          EnhancedAnalyticsPage(),
          EnhancedAdminPage(),
          EnhancedObservabilityPage(),
          EnhancedHealthPage(),
          EnhancedSecurityPage(),
          EnhancedDatabasePage(),
          EnhancedNetworkPage(),
          EnhancedSettingsPage(),
          EnhancedDocumentationPage(),
        ],
      ),
      // Floating action for quick access
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () {
          _showQuickActionsMenu(context);
        },
        icon: Icon(Icons.dashboard),
        label: Text('Quick Actions'),
      ),
    );
  }

  void _showQuickActionsMenu(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        padding: EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Quick Actions', style: Theme.of(context).textTheme.headlineSmall),
            SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                ActionChip(
                  avatar: Icon(Icons.refresh),
                  label: Text('Refresh All Data'),
                  onPressed: () {
                    Navigator.pop(context);
                    setState(() {});
                  },
                ),
                ActionChip(
                  avatar: Icon(Icons.web),
                  label: Text('Open Vue.js Dashboard'),
                  onPressed: () {
                    Navigator.pop(context);
                    // Navigate to analytics tab with Vue.js integration
                    _tabController.animateTo(1);
                  },
                ),
                ActionChip(
                  avatar: Icon(Icons.admin_panel_settings),
                  label: Text('Admin Panel'),
                  onPressed: () {
                    Navigator.pop(context);
                    _tabController.animateTo(2);
                  },
                ),
                ActionChip(
                  avatar: Icon(Icons.visibility),
                  label: Text('Observability'),
                  onPressed: () {
                    Navigator.pop(context);
                    _tabController.animateTo(3);
                  },
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

// Enhanced Splash Page with Vue.js integration
class EnhancedSplashPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return VueJsDashboardIntegration(
      tabId: 'splash',
      title: 'Welcome Splash',
      fallbackWidget: NativeSplashPage(),
    );
  }
}

// Enhanced Analytics Page with Vue.js integration
class EnhancedAnalyticsPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return VueJsDashboardIntegration(
      tabId: 'analytics',
      title: 'Analytics Dashboard',
      fallbackWidget: NativeAnalyticsPage(),
    );
  }
}

// Enhanced Admin Page with Vue.js integration
class EnhancedAdminPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return VueJsDashboardIntegration(
      tabId: 'admin',
      title: 'Web Admin Panel',
      fallbackWidget: NativeAdminPage(),
    );
  }
}

// Enhanced Observability Page with Vue.js integration
class EnhancedObservabilityPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return VueJsDashboardIntegration(
      tabId: 'observability',
      title: 'Observability Dashboard',
      fallbackWidget: NativeObservabilityPage(),
    );
  }
}

// Native Flutter implementations as fallbacks
class NativeSplashPage extends StatefulWidget {
  @override
  _NativeSplashPageState createState() => _NativeSplashPageState();
}

class _NativeSplashPageState extends State<NativeSplashPage> {
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
          // Hero section
          Container(
            padding: EdgeInsets.all(32),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Theme.of(context).primaryColor,
                  Theme.of(context).primaryColor.withValues(alpha: 0.7),
                ],
              ),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Welcome to OpenSim Next',
                  style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                SizedBox(height: 8),
                Text(
                  'Unified 10-Page Virtual World Management Platform',
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    color: Colors.white.withValues(alpha: 0.9),
                  ),
                ),
              ],
            ),
          ),
          
          SizedBox(height: 32),
          
          // Feature grid
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 2,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            children: [
              _buildFeatureCard(
                context,
                '🚀',
                'High Performance',
                'Rust/Zig hybrid architecture',
              ),
              _buildFeatureCard(
                context,
                '🔐',
                'Zero Trust Security',
                'OpenZiti encrypted overlay',
              ),
              _buildFeatureCard(
                context,
                '📊',
                'Real-time Analytics',
                'Live data streaming',
              ),
              _buildFeatureCard(
                context,
                '🌐',
                'Multi-Protocol',
                'Web, mobile, SL viewers',
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // System status
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'System Status',
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      _buildStatusItem(context, 'Users Online', '${_systemStatus['usersOnline'] ?? 0}'),
                      _buildStatusItem(context, 'Active Regions', '${_systemStatus['regionsActive'] ?? 0}'),
                      _buildStatusItem(context, 'Uptime', '${_systemStatus['uptimeHours'] ?? 0}h'),
                      _buildStatusItem(context, 'CPU Usage', '${_systemStatus['cpuUsage'] ?? 0}%'),
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

  Widget _buildFeatureCard(BuildContext context, String icon, String title, String description) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          children: [
            Text(icon, style: TextStyle(fontSize: 32)),
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
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusItem(BuildContext context, String label, String value) {
    return Column(
      children: [
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
        ),
      ],
    );
  }
}

// Additional native implementations for other pages
class NativeAnalyticsPage extends StatefulWidget {
  @override
  _NativeAnalyticsPageState createState() => _NativeAnalyticsPageState();
}

class _NativeAnalyticsPageState extends State<NativeAnalyticsPage> {
  String _timeRange = 'Last 24 Hours';
  Map<String, dynamic> _analyticsData = {};
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadAnalyticsData();
  }

  Future<void> _loadAnalyticsData() async {
    setState(() => _isLoading = true);
    
    final data = await UnifiedBackendService.instance.getAnalyticsData(_timeRange);
    
    if (mounted) {
      setState(() {
        _analyticsData = data;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Center(child: CircularProgressIndicator());
    }

    return SingleChildScrollView(
      padding: EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Analytics Dashboard',
                style: Theme.of(context).textTheme.headlineSmall,
              ),
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
            ],
          ),
          
          SizedBox(height: 24),
          
          // Metrics cards
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            children: [
              _buildMetricCard(
                'World Metrics',
                [
                  'Users Online: ${_analyticsData['worldMetrics']?['usersOnline'] ?? 0}',
                  'Active Regions: ${_analyticsData['worldMetrics']?['regionsActive'] ?? 0}',
                  'Objects: ${_analyticsData['worldMetrics']?['objectsTotal'] ?? 0}',
                ],
              ),
              _buildMetricCard(
                'Performance',
                [
                  'CPU: ${_analyticsData['performance']?['cpuUsage'] ?? 0}%',
                  'Memory: ${_analyticsData['performance']?['memoryUsage'] ?? 0}MB',
                  'Response: ${_analyticsData['performance']?['responseTime'] ?? 0}ms',
                ],
              ),
              _buildMetricCard(
                'Network',
                [
                  'WebSocket: ${_analyticsData['network']?['websocketConnections'] ?? 0}',
                  'Assets/sec: ${_analyticsData['network']?['assetRequestsPerSec'] ?? 0}',
                  'Crossings/min: ${_analyticsData['network']?['regionCrossingsPerMin'] ?? 0}',
                ],
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // Simple chart placeholder
          Card(
            child: Container(
              height: 200,
              padding: EdgeInsets.all(16),
              child: Column(
                children: [
                  Text('Analytics Charts', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  Expanded(
                    child: Center(
                      child: Text('Chart visualization would go here\n(Real charts available in Vue.js version)'),
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

  Widget _buildMetricCard(String title, List<String> metrics) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            SizedBox(height: 12),
            ...metrics.map((metric) => Padding(
              padding: EdgeInsets.symmetric(vertical: 2),
              child: Text(metric, style: Theme.of(context).textTheme.bodySmall),
            )),
          ],
        ),
      ),
    );
  }
}

// Placeholder implementations for other pages
class NativeAdminPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.admin_panel_settings, size: 64),
          SizedBox(height: 16),
          Text('Native Admin Panel', style: Theme.of(context).textTheme.headlineSmall),
          Text('Full implementation available in Vue.js version'),
        ],
      ),
    );
  }
}

class NativeObservabilityPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.visibility, size: 64),
          SizedBox(height: 16),
          Text('Native Observability Dashboard', style: Theme.of(context).textTheme.headlineSmall),
          Text('Full implementation available in Vue.js version'),
        ],
      ),
    );
  }
}

// Placeholder pages for the remaining tabs
class EnhancedHealthPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Health Monitoring Page'));
  }
}

class EnhancedSecurityPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Security Dashboard Page'));
  }
}

class EnhancedDatabasePage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Database Management Page'));
  }
}

class EnhancedNetworkPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Network Management Page'));
  }
}

class EnhancedSettingsPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Settings Page'));
  }
}

class EnhancedDocumentationPage extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Center(child: Text('Enhanced Documentation Page'));
  }
}