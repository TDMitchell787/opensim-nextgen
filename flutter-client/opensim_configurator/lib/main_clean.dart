import 'package:flutter/material.dart';

void main() {
  runApp(OpenSimApp());
}

class OpenSimApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'OpenSim Next - Comprehensive Platform',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      home: OpenSimDashboard(),
    );
  }
}

class OpenSimDashboard extends StatefulWidget {
  @override
  _OpenSimDashboardState createState() => _OpenSimDashboardState();
}

class _OpenSimDashboardState extends State<OpenSimDashboard> {
  int _currentPage = 1;

  final List<Map<String, dynamic>> _pages = [
    {'title': 'Graphics Splash', 'icon': Icons.image, 'color': Colors.purple},
    {'title': 'Contributors', 'icon': Icons.people, 'color': Colors.green},
    {'title': 'Welcome', 'icon': Icons.home, 'color': Colors.blue},
    {'title': 'Web Admin', 'icon': Icons.admin_panel_settings, 'color': Colors.orange},
    {'title': 'Analytics', 'icon': Icons.analytics, 'color': Colors.red},
    {'title': 'Observability', 'icon': Icons.visibility, 'color': Colors.teal},
    {'title': 'Health', 'icon': Icons.monitor_heart, 'color': Colors.pink},
    {'title': 'Security', 'icon': Icons.security, 'color': Colors.indigo},
    {'title': 'Database', 'icon': Icons.storage, 'color': Colors.brown},
    {'title': 'Settings', 'icon': Icons.settings, 'color': Colors.amber},
    {'title': 'Containers', 'icon': Icons.developer_board, 'color': Colors.deepPurple},
    {'title': 'Orchestration', 'icon': Icons.account_tree, 'color': Colors.teal[700]!},
    {'title': 'Deployment', 'icon': Icons.rocket_launch, 'color': Colors.deepOrange},
    {'title': 'Extensions', 'icon': Icons.extension, 'color': Colors.cyan[600]!},
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('OpenSim Next - Comprehensive Platform'),
            Text(
              'Production-Ready Virtual World Management',
              style: Theme.of(context).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
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
        ],
      ),
      body: Column(
        children: [
          // Navigation bar
          Container(
            height: 60,
            padding: EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: Colors.grey[100],
              border: Border(bottom: BorderSide(color: Colors.grey[300]!)),
            ),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: List.generate(_pages.length, (index) {
                  int pageNum = index + 1;
                  final page = _pages[index];
                  return Padding(
                    padding: EdgeInsets.symmetric(horizontal: 4),
                    child: ElevatedButton.icon(
                      onPressed: () => setState(() => _currentPage = pageNum),
                      icon: Icon(page['icon'], size: 18),
                      label: Text(page['title']),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: _currentPage == pageNum ? page['color'] : Colors.grey[300],
                        foregroundColor: _currentPage == pageNum ? Colors.white : Colors.black87,
                      ),
                    ),
                  );
                }),
              ),
            ),
          ),
          
          // Content area
          Expanded(
            child: _buildPageContent(_currentPage),
          ),
        ],
      ),
    );
  }

  Widget _buildPageContent(int pageNumber) {
    switch (pageNumber) {
      case 1: return _buildGraphicsSplashPage();
      case 2: return _buildContributorsPage();
      case 3: return _buildWelcomePage();
      case 4: return _buildWebAdminPage();
      case 5: return _buildAnalyticsPage();
      case 6: return _buildObservabilityPage();
      case 7: return _buildHealthPage();
      case 8: return _buildSecurityPage();
      case 9: return _buildDatabasePage();
      case 10: return _buildSettingsPage();
      case 11: return _buildContainersPage();
      case 12: return _buildOrchestrationPage();
      case 13: return _buildDeploymentPage();
      case 14: return _buildExtensionsPage();
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
      {'name': 'OpenSim Core Team', 'role': 'Virtual World Engine', 'icon': Icons.engineering, 'color': Colors.blue},
      {'name': 'Rust Development Team', 'role': 'Performance & Safety', 'icon': Icons.memory, 'color': Colors.orange},
      {'name': 'Zig Integration Team', 'role': 'Low-Level Performance', 'icon': Icons.flash_on, 'color': Colors.yellow},
      {'name': 'Flutter UI Team', 'role': 'Cross-Platform Interface', 'icon': Icons.phone_android, 'color': Colors.cyan},
      {'name': 'OpenZiti Security Team', 'role': 'Zero Trust Networking', 'icon': Icons.security, 'color': Colors.green},
      {'name': 'Community Contributors', 'role': 'Extensions & Plugins', 'icon': Icons.groups, 'color': Colors.purple},
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
                child: Padding(
                  padding: EdgeInsets.all(16),
                  child: Column(
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
              );
            },
          ),
        ],
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
              _buildNavCard('Web Admin', 'Manage server settings', Icons.admin_panel_settings, 4),
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
                  Text('System Status Overview', style: Theme.of(context).textTheme.headlineSmall),
                  SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceAround,
                    children: [
                      _buildStatusItem('Users Online', '42', Icons.people),
                      _buildStatusItem('Active Regions', '8', Icons.map),
                      _buildStatusItem('Uptime', '120h', Icons.timer),
                      _buildStatusItem('CPU Usage', '25%', Icons.memory),
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

  Widget _buildWebAdminPage() {
    final regions = [
      {'id': 1, 'name': 'Welcome Region', 'status': 'online', 'users': 8, 'load': 'low'},
      {'id': 2, 'name': 'Sandbox', 'status': 'online', 'users': 12, 'load': 'medium'},
      {'id': 3, 'name': 'Event Space', 'status': 'online', 'users': 22, 'load': 'high'},
      {'id': 4, 'name': 'Shopping District', 'status': 'maintenance', 'users': 0, 'load': 'none'},
    ];

    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Admin header with auto configurator
          Container(
            padding: EdgeInsets.all(24),
            decoration: BoxDecoration(
              gradient: LinearGradient(colors: [Colors.orange.withValues(alpha: 0.1), Colors.orange.withValues(alpha: 0.05)]),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              children: [
                Icon(Icons.admin_panel_settings, size: 48, color: Colors.orange),
                SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('System Administration', style: Theme.of(context).textTheme.headlineMedium),
                      SizedBox(height: 8),
                      Row(
                        children: [
                          Container(width: 12, height: 12, decoration: BoxDecoration(shape: BoxShape.circle, color: Colors.green)),
                          SizedBox(width: 8),
                          Text('System Status: ONLINE', style: Theme.of(context).textTheme.bodyLarge),
                        ],
                      ),
                    ],
                  ),
                ),
                ElevatedButton.icon(
                  onPressed: () => _showSnackBar('Opening Auto Configurator at /configurator'),
                  icon: Icon(Icons.settings_applications),
                  label: Text('Auto Configurator'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                ),
              ],
            ),
          ),
          SizedBox(height: 32),
          
          // Server Instances Management Table
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Server Instances Management', style: Theme.of(context).textTheme.titleLarge),
                      ElevatedButton.icon(
                        onPressed: () => _showCreateServerInstanceDialog(),
                        icon: Icon(Icons.add_circle),
                        label: Text('Add Server Instance'),
                        style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.3)), borderRadius: BorderRadius.circular(8)),
                    child: Column(
                      children: [
                        // Server instances table header
                        Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(color: Colors.blue.withValues(alpha: 0.1), borderRadius: BorderRadius.only(topLeft: Radius.circular(8), topRight: Radius.circular(8))),
                          child: Row(
                            children: [
                              Expanded(flex: 2, child: Text('Server Instance', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Type', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Status', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Port', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Config File', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Actions', style: TextStyle(fontWeight: FontWeight.bold))),
                            ],
                          ),
                        ),
                        // Server instances rows
                        ..._getServerInstances().map((server) => _buildServerInstanceRow(server)),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 32),
          
          // Region management table - THE SERVER TABLE YOU WANTED!
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
                      ElevatedButton.icon(onPressed: () => _showSnackBar('Opening region creation wizard...'), icon: Icon(Icons.add), label: Text('Add Region')),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.3)), borderRadius: BorderRadius.circular(8)),
                    child: Column(
                      children: [
                        // Table header
                        Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(color: Theme.of(context).primaryColor.withValues(alpha: 0.1), borderRadius: BorderRadius.only(topLeft: Radius.circular(8), topRight: Radius.circular(8))),
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
                        ...regions.map((region) => _buildRegionRow(region)),
                      ],
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

  Widget _buildAnalyticsPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Analytics Dashboard', 'Real-time performance and usage metrics'),
          SizedBox(height: 32),
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.5,
            children: [
              _buildMetricCard('Virtual World', ['Users Online: 45', 'Active Regions: 8', 'Objects: 15420'], Icons.public, Colors.blue),
              _buildMetricCard('Performance', ['CPU: 28%', 'Memory: 1250MB', 'Response: 18ms'], Icons.speed, Colors.green),
              _buildMetricCard('Network', ['WebSocket: 45', 'Assets/sec: 150', 'Crossings/min: 12'], Icons.network_check, Colors.orange),
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
      ),
    );
  }

  Widget _buildObservabilityPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Advanced Observability Dashboard', 'Real-time monitoring, tracing, and system insights'),
          SizedBox(height: 32),
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 3,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.2,
            children: [
              _buildHealthCard('System Health', ['CPU: 25%', 'Memory: 40%', 'Disk I/O: 15%']),
              _buildHealthCard('Virtual World', ['Users: 42', 'Regions: 8', 'Physics Bodies: 1520']),
              _buildHealthCard('Performance', ['Response: 15ms', 'DB Query: 5ms', 'Throughput: 450 req/s']),
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
      ),
    );
  }

  Widget _buildHealthPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('System Health Monitoring', 'Real-time system metrics and diagnostics'),
          SizedBox(height: 32),
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 4,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.5,
            children: [
              _buildHealthMetricCard('CPU Usage', '25%', 25, Colors.blue),
              _buildHealthMetricCard('Memory', '40%', 40, Colors.orange),
              _buildHealthMetricCard('Disk Usage', '15%', 15, Colors.green),
              _buildHealthMetricCard('Network I/O', '8 MB/s', 30, Colors.purple),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildSecurityPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Security Dashboard', 'Zero Trust Network Security & Monitoring'),
          SizedBox(height: 32),
          GridView.count(
            shrinkWrap: true,
            physics: NeverScrollableScrollPhysics(),
            crossAxisCount: 4,
            crossAxisSpacing: 16,
            mainAxisSpacing: 16,
            childAspectRatio: 1.2,
            children: [
              _buildSecurityCard('Active Threats', '0', Icons.warning, Colors.green),
              _buildSecurityCard('Blocked Attempts', '23', Icons.block, Colors.orange),
              _buildSecurityCard('Trusted Connections', '42', Icons.verified_user, Colors.green),
              _buildSecurityCard('Encryption Status', 'Active', Icons.lock, Colors.blue),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDatabasePage() {
    final databases = _getDatabaseInstances();
    final backups = _getRecentBackups();
    
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Database Management', 'Live multi-backend database operations and backup strategies'),
          SizedBox(height: 32),
          
          // Database Instances Overview
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(Icons.storage, color: Colors.blue),
                            SizedBox(width: 8),
                            Text('Database Instances', style: Theme.of(context).textTheme.titleLarge),
                            Spacer(),
                            Container(
                              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                              decoration: BoxDecoration(color: Colors.green.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
                              child: Text('3 Active', style: TextStyle(color: Colors.green, fontSize: 12, fontWeight: FontWeight.bold)),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildDbStatusCard('PostgreSQL', 'Connected', '15.2GB', Icons.storage, Colors.blue),
                            _buildDbStatusCard('MySQL', 'Connected', '8.7GB', Icons.dns, Colors.orange),
                            _buildDbStatusCard('SQLite', 'Connected', '2.1GB', Icons.folder, Colors.green),
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
                        Row(
                          children: [
                            Icon(Icons.backup, color: Colors.purple),
                            SizedBox(width: 8),
                            Text('Backup Status', style: Theme.of(context).textTheme.titleLarge),
                            Spacer(),
                            ElevatedButton.icon(
                              onPressed: () => _showCreateBackupDialog(),
                              icon: Icon(Icons.backup, size: 16),
                              label: Text('Create Backup'),
                              style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildBackupStatCard('Last Backup', '2h ago', Icons.schedule, Colors.green),
                            _buildBackupStatCard('Total Backups', '247', Icons.archive, Colors.blue),
                            _buildBackupStatCard('Backup Size', '125GB', Icons.storage, Colors.orange),
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
          
          // Live Database Connections Table
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Live Database Connections', style: Theme.of(context).textTheme.titleLarge),
                      Row(
                        children: [
                          ElevatedButton.icon(
                            onPressed: () => _showAddDatabaseDialog(),
                            icon: Icon(Icons.add),
                            label: Text('Add Database'),
                            style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                          ),
                          SizedBox(width: 8),
                          ElevatedButton.icon(
                            onPressed: () => _showRestoreBackupDialog(),
                            icon: Icon(Icons.restore),
                            label: Text('Restore Backup'),
                            style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                          ),
                          SizedBox(width: 8),
                          ElevatedButton.icon(
                            onPressed: () => _showMergeDatabaseDialog(),
                            icon: Icon(Icons.merge),
                            label: Text('Merge Database'),
                            style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
                          ),
                          SizedBox(width: 8),
                          ElevatedButton.icon(
                            onPressed: () => _showSnackBar('Refreshing database connections...'),
                            icon: Icon(Icons.refresh),
                            label: Text('Refresh'),
                          ),
                        ],
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.3)), borderRadius: BorderRadius.circular(8)),
                    child: Column(
                      children: [
                        // Database table header
                        Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(color: Colors.blue.withValues(alpha: 0.1), borderRadius: BorderRadius.only(topLeft: Radius.circular(8), topRight: Radius.circular(8))),
                          child: Row(
                            children: [
                              Expanded(flex: 2, child: Text('Database', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Type', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Status', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Size', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Tables', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Last Backup', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Actions', style: TextStyle(fontWeight: FontWeight.bold))),
                            ],
                          ),
                        ),
                        // Database rows
                        ...databases.map((db) => _buildDatabaseRow(db)),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 32),
          
          // Backup Strategy & Recent Backups
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Backup Strategy', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        _buildBackupStrategyItem('Automated Daily Backups', '3:00 AM UTC', Icons.schedule, true),
                        _buildBackupStrategyItem('Weekly Full Backup', 'Sundays 2:00 AM', Icons.backup, true),
                        _buildBackupStrategyItem('Real-time Replication', 'Continuous', Icons.sync, true),
                        _buildBackupStrategyItem('Offsite Cloud Storage', 'AWS S3', Icons.cloud, true),
                        _buildBackupStrategyItem('Disaster Recovery', '4-hour RTO', Icons.security, true),
                        SizedBox(height: 16),
                        ElevatedButton.icon(
                          onPressed: () => _showBackupStrategyDialog(),
                          icon: Icon(Icons.settings),
                          label: Text('Configure Backup Strategy'),
                          style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
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
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text('Recent Backups', style: Theme.of(context).textTheme.titleLarge),
                            TextButton(
                              onPressed: () => _showAllBackupsDialog(),
                              child: Text('View All'),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        Container(
                          height: 300,
                          child: ListView.builder(
                            itemCount: backups.length,
                            itemBuilder: (context, index) {
                              final backup = backups[index];
                              return _buildBackupItem(backup);
                            },
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
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
          _buildSectionHeader('Settings', 'Application configuration and preferences'),
          SizedBox(height: 32),
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Application Settings', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  ListTile(
                    leading: Icon(Icons.palette),
                    title: Text('Theme'),
                    subtitle: Text('Choose your preferred theme'),
                    trailing: DropdownButton<String>(
                      value: 'System',
                      items: ['System', 'Light', 'Dark'].map((theme) => DropdownMenuItem(value: theme, child: Text(theme))).toList(),
                      onChanged: (value) => _showSnackBar('Theme changed to $value'),
                    ),
                  ),
                  ListTile(
                    leading: Icon(Icons.notifications),
                    title: Text('Notifications'),
                    subtitle: Text('Manage notification preferences'),
                    trailing: Switch(value: true, onChanged: (value) => _showSnackBar('Notifications ${value ? 'enabled' : 'disabled'}')),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildContainersPage() {
    final containers = _getContainerInstances();
    
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Container Management', 'Docker and Kubernetes container orchestration'),
          SizedBox(height: 32),
          
          // Container Overview
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Icon(Icons.developer_board, color: Colors.deepPurple),
                            SizedBox(width: 8),
                            Text('Container Status', style: Theme.of(context).textTheme.titleLarge),
                            Spacer(),
                            Container(
                              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                              decoration: BoxDecoration(color: Colors.green.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
                              child: Text('5 Running', style: TextStyle(color: Colors.green, fontSize: 12, fontWeight: FontWeight.bold)),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildContainerStatCard('Running', '5', Icons.play_circle, Colors.green),
                            _buildContainerStatCard('Stopped', '2', Icons.stop_circle, Colors.red),
                            _buildContainerStatCard('Images', '12', Icons.layers, Colors.blue),
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
                        Row(
                          children: [
                            Icon(Icons.memory, color: Colors.orange),
                            SizedBox(width: 8),
                            Text('Resource Usage', style: Theme.of(context).textTheme.titleLarge),
                          ],
                        ),
                        SizedBox(height: 16),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceAround,
                          children: [
                            _buildResourceCard('CPU', '35%', Icons.computer, Colors.orange),
                            _buildResourceCard('Memory', '2.8GB', Icons.memory, Colors.blue),
                            _buildResourceCard('Storage', '45GB', Icons.storage, Colors.green),
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
          
          // Container Management Table
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Container Instances', style: Theme.of(context).textTheme.titleLarge),
                      Row(
                        children: [
                          ElevatedButton.icon(
                            onPressed: () => _showCreateContainerDialog(),
                            icon: Icon(Icons.add),
                            label: Text('Create Container'),
                            style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                          ),
                          SizedBox(width: 8),
                          ElevatedButton.icon(
                            onPressed: () => _showSnackBar('Pulling latest images...'),
                            icon: Icon(Icons.download),
                            label: Text('Pull Images'),
                          ),
                        ],
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.3)), borderRadius: BorderRadius.circular(8)),
                    child: Column(
                      children: [
                        // Container table header
                        Container(
                          padding: EdgeInsets.all(16),
                          decoration: BoxDecoration(color: Colors.deepPurple.withValues(alpha: 0.1), borderRadius: BorderRadius.only(topLeft: Radius.circular(8), topRight: Radius.circular(8))),
                          child: Row(
                            children: [
                              Expanded(flex: 2, child: Text('Container', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Image', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Status', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Port', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('CPU/Memory', style: TextStyle(fontWeight: FontWeight.bold))),
                              Expanded(child: Text('Actions', style: TextStyle(fontWeight: FontWeight.bold))),
                            ],
                          ),
                        ),
                        // Container rows
                        ...containers.map((container) => _buildContainerRow(container)),
                      ],
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

  Widget _buildOrchestrationPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Container Orchestration', 'Kubernetes and Docker Swarm management'),
          SizedBox(height: 32),
          
          // Kubernetes Clusters
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text('Kubernetes Clusters', style: Theme.of(context).textTheme.titleLarge),
                      ElevatedButton.icon(
                        onPressed: () => _showCreateClusterDialog(),
                        icon: Icon(Icons.add_circle),
                        label: Text('Add Cluster'),
                        style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(child: _buildClusterCard({'name': 'Production Cluster', 'status': 'running', 'nodes': 5, 'pods': 12, 'services': 8, 'cpu': '45%', 'memory': '12.5GB', 'version': 'v1.28.2'})),
                      SizedBox(width: 16),
                      Expanded(child: _buildClusterCard({'name': 'Staging Cluster', 'status': 'running', 'nodes': 2, 'pods': 6, 'services': 4, 'cpu': '25%', 'memory': '6.2GB', 'version': 'v1.28.2'})),
                      SizedBox(width: 16),
                      Expanded(child: _buildClusterCard({'name': 'Development Cluster', 'status': 'stopped', 'nodes': 0, 'pods': 3, 'services': 0, 'cpu': '0%', 'memory': '0GB', 'version': 'v1.28.2'})),
                    ],
                  ),
                ],
              ),
            ),
          ),
          
          SizedBox(height: 32),
          
          // Service Mesh
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Service Mesh & Load Balancing', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  GridView.count(
                    shrinkWrap: true,
                    physics: NeverScrollableScrollPhysics(),
                    crossAxisCount: 2,
                    crossAxisSpacing: 16,
                    mainAxisSpacing: 16,
                    childAspectRatio: 2,
                    children: [
                      _buildServiceCard('Istio Service Mesh', 'Active', 'Traffic routing and security', Icons.account_tree, Colors.blue),
                      _buildServiceCard('NGINX Load Balancer', 'Running', '5 backend servers', Icons.balance, Colors.green),
                      _buildServiceCard('Consul Service Discovery', 'Connected', '12 registered services', Icons.explore, Colors.purple),
                      _buildServiceCard('Prometheus Monitoring', 'Collecting', '847 metrics/sec', Icons.monitor, Colors.orange),
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

  Widget _buildDeploymentPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Deployment Pipeline', 'CI/CD and automated deployment management'),
          SizedBox(height: 32),
          
          // Deployment Status
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Current Deployments', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        _buildDeploymentItem('opensim-next-v1.2.3', 'Production', 'success', '15 minutes ago'),
                        _buildDeploymentItem('opensim-next-v1.2.4-rc1', 'Staging', 'running', '2 minutes ago'),
                        _buildDeploymentItem('opensim-next-v1.2.4-rc2', 'Development', 'pending', 'Queued'),
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
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text('Quick Deploy', style: Theme.of(context).textTheme.titleLarge),
                            ElevatedButton.icon(
                              onPressed: () => _showDeploymentWizard(),
                              icon: Icon(Icons.rocket_launch),
                              label: Text('Deploy Now'),
                              style: ElevatedButton.styleFrom(backgroundColor: Colors.deepOrange),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        _buildDeployOption('🚀 Production Deploy', 'Deploy to live environment'),
                        _buildDeployOption('🧪 Staging Deploy', 'Deploy to staging for testing'),
                        _buildDeployOption('💻 Dev Deploy', 'Deploy to development'),
                        _buildDeployOption('🔄 Rollback', 'Rollback to previous version'),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // CI/CD Pipeline
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('CI/CD Pipeline Status', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  _buildPipelineStage('Source', 'GitHub main branch', 'completed', Colors.green),
                  _buildPipelineStage('Build', 'Rust + Flutter compilation', 'completed', Colors.green),
                  _buildPipelineStage('Test', 'Unit & integration tests', 'running', Colors.orange),
                  _buildPipelineStage('Security Scan', 'Vulnerability assessment', 'pending', Colors.grey),
                  _buildPipelineStage('Deploy', 'Container deployment', 'pending', Colors.grey),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildExtensionsPage() {
    return SingleChildScrollView(
      padding: EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Extensions & Plugins', 'Manage OpenSim extensions and custom modules'),
          SizedBox(height: 32),
          
          // Extension Categories
          Row(
            children: [
              Expanded(
                child: Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Installed Extensions', style: Theme.of(context).textTheme.titleLarge),
                        SizedBox(height: 16),
                        _buildExtensionItem('OpenZiti Security', 'v2.1.0', 'Zero Trust Networking', true, Colors.green),
                        _buildExtensionItem('Advanced Physics', 'v1.5.2', 'Enhanced physics engine', true, Colors.blue),
                        _buildExtensionItem('VR Integration', 'v0.9.1', 'Virtual reality support', false, Colors.orange),
                        _buildExtensionItem('AI NPCs', 'v1.2.0', 'AI-powered characters', true, Colors.purple),
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
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text('Extension Store', style: Theme.of(context).textTheme.titleLarge),
                            ElevatedButton.icon(
                              onPressed: () => _showExtensionStoreDialog(),
                              icon: Icon(Icons.store),
                              label: Text('Browse Store'),
                              style: ElevatedButton.styleFrom(backgroundColor: Colors.cyan[600]),
                            ),
                          ],
                        ),
                        SizedBox(height: 16),
                        _buildStoreExtension('Blockchain Assets', 'NFT and crypto integration', '4.8★', Colors.amber),
                        _buildStoreExtension('Voice Chat Pro', 'Spatial audio system', '4.9★', Colors.green),
                        _buildStoreExtension('Analytics Plus', 'Advanced user analytics', '4.7★', Colors.blue),
                        _buildStoreExtension('Custom Weather', 'Dynamic weather system', '4.6★', Colors.cyan),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
          
          SizedBox(height: 32),
          
          // Custom Extension Development
          Card(
            child: Padding(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Extension Development', style: Theme.of(context).textTheme.titleLarge),
                  SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton.icon(
                          onPressed: () => _showInstallExtensionDialog(),
                          icon: Icon(Icons.code),
                          label: Text('Create New Extension'),
                          style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                        ),
                      ),
                      SizedBox(width: 16),
                      Expanded(
                        child: ElevatedButton.icon(
                          onPressed: () => _showSnackBar('Opening extension SDK documentation...'),
                          icon: Icon(Icons.book),
                          label: Text('SDK Documentation'),
                        ),
                      ),
                      SizedBox(width: 16),
                      Expanded(
                        child: ElevatedButton.icon(
                          onPressed: () => _showSnackBar('Opening extension marketplace...'),
                          icon: Icon(Icons.publish),
                          label: Text('Publish Extension'),
                          style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
                        ),
                      ),
                    ],
                  ),
                  SizedBox(height: 16),
                  Container(
                    padding: EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: Colors.blue.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Extension API Support:', style: TextStyle(fontWeight: FontWeight.bold)),
                        SizedBox(height: 8),
                        Text('• Rust native extensions for high performance'),
                        Text('• JavaScript/TypeScript for web extensions'),
                        Text('• Python scripting for automation'),
                        Text('• C# compatibility for legacy OpenSim modules'),
                        Text('• WebAssembly (WASM) for cross-platform modules'),
                      ],
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
    return Card(
      child: InkWell(
        onTap: () => setState(() => _currentPage = pageIndex),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: EdgeInsets.all(16),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(icon, size: 32, color: Theme.of(context).primaryColor),
              SizedBox(height: 8),
              Text(title, style: Theme.of(context).textTheme.titleMedium, textAlign: TextAlign.center),
              SizedBox(height: 4),
              Text(description, style: Theme.of(context).textTheme.bodySmall, textAlign: TextAlign.center, maxLines: 2, overflow: TextOverflow.ellipsis),
            ],
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

  Widget _buildRegionRow(Map<String, dynamic> region) {
    Color statusColor = region['status'] == 'online' ? Colors.green : region['status'] == 'maintenance' ? Colors.orange : Colors.red;
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.2)))),
      child: Row(
        children: [
          Expanded(flex: 2, child: Text(region['name'], style: TextStyle(fontWeight: FontWeight.w500))),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(region['status'], style: TextStyle(color: statusColor, fontSize: 12), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text('${region['users']}', textAlign: TextAlign.center)),
          Expanded(child: Text('${region['load']}', textAlign: TextAlign.center)),
          Expanded(
            child: Wrap(
              spacing: 4,
              children: [
                _buildActionButton(
                  icon: Icons.terminal,
                  tooltip: 'Terminal',
                  color: Colors.black,
                  onPressed: () => _showSnackBar('Opening terminal for ${region['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.play_arrow,
                  tooltip: 'Start',
                  color: Colors.green,
                  onPressed: () => _showSnackBar('Starting ${region['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.stop,
                  tooltip: 'Stop',
                  color: Colors.red,
                  onPressed: () => _showSnackBar('Stopping ${region['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.settings,
                  tooltip: 'Config',
                  color: Colors.blue,
                  onPressed: () => _showRegionConfigDialog(region),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionButton({
    required IconData icon,
    required String tooltip,
    required Color color,
    required VoidCallback onPressed,
  }) {
    return Container(
      width: 32,
      height: 32,
      child: IconButton(
        onPressed: onPressed,
        icon: Icon(icon, size: 16),
        tooltip: tooltip,
        style: IconButton.styleFrom(
          backgroundColor: color.withValues(alpha: 0.1),
          foregroundColor: color,
          padding: EdgeInsets.zero,
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

  void _showSnackBar(String message) {
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
  }

  void _showServerAction(String action) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('$action Server'),
        content: Text('Are you sure you want to $action the OpenSim server?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context), child: Text('Cancel')),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              _showSnackBar('$action initiated...');
            },
            child: Text(action),
          ),
        ],
      ),
    );
  }

  void _showRegionConfigDialog(Map<String, dynamic> region) {
    String regionIniContent = _getRegionIniContent(region);
    
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.settings, color: Colors.blue),
              SizedBox(width: 8),
              Text('${region['name']}.ini'),
              Spacer(),
              Text('Region ${region['id']}', style: TextStyle(fontSize: 14, color: Colors.grey)),
            ],
          ),
          content: Container(
            width: double.maxFinite,
            height: 500,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: Colors.grey[100],
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.folder, size: 16, color: Colors.orange),
                      SizedBox(width: 4),
                      Text('config/regions/${region['name']}.ini', style: TextStyle(fontSize: 12, fontFamily: 'monospace')),
                      Spacer(),
                      Container(
                        padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: _getStatusColor(region['status']),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Text(region['status'].toUpperCase(), style: TextStyle(fontSize: 10, color: Colors.white, fontWeight: FontWeight.bold)),
                      ),
                    ],
                  ),
                ),
                SizedBox(height: 16),
                Expanded(
                  child: Container(
                    padding: EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Colors.grey[50],
                      border: Border.all(color: Colors.grey[300]!),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: SingleChildScrollView(
                      child: SelectableText(
                        regionIniContent,
                        style: TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 13,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Close'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Opening ${region['name']}.ini in external editor...');
              },
              icon: Icon(Icons.edit, size: 16),
              label: Text('Edit File'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Reloading configuration for ${region['name']}...');
              },
              icon: Icon(Icons.refresh, size: 16),
              label: Text('Reload Config'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
            ),
          ],
        );
      },
    );
  }

  String _getRegionIniContent(Map<String, dynamic> region) {
    return '''[Startup]
; Region Configuration for ${region['name']}
; Generated by OpenSim Next Web Admin

[Network]
; Network interface for client connections
http_listener_port = 9000
region_port = 9000

; External hostname that clients will use to connect
ExternalHostName = localhost

; Internal IP address and port
InternalAddress = 0.0.0.0
InternalPort = 9000

[DatabaseService]
; Database connection for this region
StorageProvider = OpenSim.Data.SQLite.dll
ConnectionString = "Data Source=config/regions/${region['name'].toLowerCase().replaceAll(' ', '_')}.db;Version=3;New=True;"

[Region]
; The unique region ID for this region
RegionUUID = 00000000-0000-0000-0000-00000000000${region['id'].toString().padLeft(1, '0')}

; Location on the grid
RegionLocationX = 1000
RegionLocationY = 1000

; Size of region in meters (Standard OpenSim regions are 256x256)
RegionSizeX = 256
RegionSizeY = 256

; Region name as it appears to residents
RegionName = ${region['name']}

; Estate settings
EstateManager = admin@opensim.local
MaxAgents = 100
MaxPrims = 15000

[Physics]
; Physics engine to use
physics = OpenDynamicsEngine

; Enable physical prim movement
physical_prim = true

; Gravity settings (m/s squared)
gravity = -9.8

[ScriptEngine]
; Scripting engine
DefaultScriptEngine = XEngine

; Enable scripting
Enabled = true

; Maximum script execution time (seconds)
ScriptExecutionTimeout = 30

[Permissions]
; Allow land changes
AllowedScript = true

; Enable god mode
GodMode = true

; Enable admin commands
AdminCommandsEnabled = true

[Terrain]
; Default terrain settings
InitialTerrain = flat

; Water height (in meters)
WaterHeight = 20.0

[Voice]
; Voice chat settings
enabled = false

[Economy]
; Economy module
EconomyModule = BetaGridLikeMoneyModule

; Enable economy
enabled = true

[Status]
; Current status: ${region['status']}
; Active users: ${region['users']}
; Server load: ${region['load']}
; Last updated: ${DateTime.now().toString()}

[Chat]
; Chat settings
whisper_distance = 10
say_distance = 20
shout_distance = 100

[World]
; World map settings
GenerateMaptiles = true
MaptileRefresh = 0

; Day/night cycle
SunHour = 14
''';
  }

  Widget _buildConfigSection(String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
        SizedBox(height: 8),
        ...children,
      ],
    );
  }

  Widget _buildConfigField(String label, String value) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Text(label, style: TextStyle(fontSize: 14)),
          ),
          Expanded(
            flex: 3,
            child: TextFormField(
              initialValue: value,
              decoration: InputDecoration(
                isDense: true,
                border: OutlineInputBorder(),
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 8),
              ),
              style: TextStyle(fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildConfigDropdown(String label, String value, List<String> options) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Text(label, style: TextStyle(fontSize: 14)),
          ),
          Expanded(
            flex: 3,
            child: DropdownButtonFormField<String>(
              value: value,
              decoration: InputDecoration(
                isDense: true,
                border: OutlineInputBorder(),
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 8),
              ),
              items: options.map((String option) {
                return DropdownMenuItem<String>(
                  value: option,
                  child: Text(option, style: TextStyle(fontSize: 13)),
                );
              }).toList(),
              onChanged: (String? newValue) {},
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRow(String label, String value, Color color) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Text(label, style: TextStyle(fontSize: 14)),
          ),
          Expanded(
            flex: 3,
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: color.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: color.withValues(alpha: 0.3)),
              ),
              child: Text(value, style: TextStyle(fontSize: 13, color: color, fontWeight: FontWeight.w500)),
            ),
          ),
        ],
      ),
    );
  }

  Color _getStatusColor(String status) {
    switch (status.toLowerCase()) {
      case 'online': return Colors.green;
      case 'maintenance': return Colors.orange;
      case 'offline': return Colors.red;
      default: return Colors.grey;
    }
  }

  Color _getLoadColor(String load) {
    switch (load.toLowerCase()) {
      case 'low': return Colors.green;
      case 'medium': return Colors.orange;
      case 'high': return Colors.red;
      case 'none': return Colors.grey;
      default: return Colors.blue;
    }
  }

  List<Map<String, dynamic>> _getServerInstances() {
    return [
      {
        'id': 1,
        'name': 'Main Grid Server',
        'type': 'Grid',
        'status': 'running',
        'port': 8002,
        'configFile': 'config/grid.ini',
        'uptime': '15h 32m',
        'memory': '2.4GB',
        'cpu': '15%'
      },
      {
        'id': 2,
        'name': 'Standalone Dev Server',
        'type': 'Standalone',
        'status': 'running',
        'port': 9000,
        'configFile': 'config/standalone.ini',
        'uptime': '3h 45m',
        'memory': '1.8GB',
        'cpu': '8%'
      },
      {
        'id': 3,
        'name': 'Test Environment',
        'type': 'Standalone',
        'status': 'stopped',
        'port': 9001,
        'configFile': 'config/test-standalone.ini',
        'uptime': '0m',
        'memory': '0MB',
        'cpu': '0%'
      },
      {
        'id': 4,
        'name': 'Backup Grid Instance',
        'type': 'Grid',
        'status': 'maintenance',
        'port': 8003,
        'configFile': 'config/backup-grid.ini',
        'uptime': '72h 15m',
        'memory': '3.1GB',
        'cpu': '22%'
      },
    ];
  }

  Widget _buildServerInstanceRow(Map<String, dynamic> server) {
    Color statusColor = _getStatusColor(server['status']);
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.2)))),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(server['name'], style: TextStyle(fontWeight: FontWeight.w500)),
                SizedBox(height: 4),
                Text('CPU: ${server['cpu']} | Memory: ${server['memory']} | Uptime: ${server['uptime']}', 
                     style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: server['type'] == 'Grid' ? Colors.blue.withValues(alpha: 0.2) : Colors.purple.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                server['type'],
                style: TextStyle(
                  color: server['type'] == 'Grid' ? Colors.blue : Colors.purple,
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                ),
                textAlign: TextAlign.center,
              ),
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(server['status'], style: TextStyle(color: statusColor, fontSize: 12), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text('${server['port']}', textAlign: TextAlign.center, style: TextStyle(fontFamily: 'monospace'))),
          Expanded(
            child: Text(
              server['configFile'].split('/').last,
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12, fontFamily: 'monospace', color: Colors.blue),
            ),
          ),
          Expanded(
            child: Wrap(
              spacing: 4,
              children: [
                _buildActionButton(
                  icon: Icons.terminal,
                  tooltip: 'Console',
                  color: Colors.black,
                  onPressed: () => _showSnackBar('Opening console for ${server['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.play_arrow,
                  tooltip: 'Start',
                  color: Colors.green,
                  onPressed: () => _showSnackBar('Starting ${server['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.stop,
                  tooltip: 'Stop',
                  color: Colors.red,
                  onPressed: () => _showSnackBar('Stopping ${server['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.settings,
                  tooltip: 'Config',
                  color: Colors.blue,
                  onPressed: () => _showServerConfigDialog(server),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showCreateServerInstanceDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String selectedType = 'Standalone';
        String serverName = '';
        String port = '9000';
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.add_circle, color: Colors.green),
                  SizedBox(width: 8),
                  Text('Create New Server Instance'),
                ],
              ),
              content: Container(
                width: 400,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Server Instance Name',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) => serverName = value,
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: selectedType,
                      decoration: InputDecoration(
                        labelText: 'Server Type',
                        border: OutlineInputBorder(),
                      ),
                      items: ['Standalone', 'Grid'].map((String type) {
                        return DropdownMenuItem<String>(
                          value: type,
                          child: Text(type),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          selectedType = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Port Number',
                        border: OutlineInputBorder(),
                      ),
                      keyboardType: TextInputType.number,
                      onChanged: (value) => port = value,
                    ),
                    SizedBox(height: 16),
                    Container(
                      padding: EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.blue.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Configuration File:', style: TextStyle(fontWeight: FontWeight.bold)),
                          SizedBox(height: 4),
                          Text('config/${selectedType.toLowerCase()}.ini', style: TextStyle(fontFamily: 'monospace', color: Colors.blue)),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Creating $selectedType server instance: $serverName on port $port');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                  child: Text('Create Server'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showServerConfigDialog(Map<String, dynamic> server) {
    String configContent = _getServerConfigContent(server);
    
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.settings, color: Colors.blue),
              SizedBox(width: 8),
              Text('${server['configFile'].split('/').last}'),
              Spacer(),
              Container(
                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: server['type'] == 'Grid' ? Colors.blue : Colors.purple,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(server['type'], style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
              ),
            ],
          ),
          content: Container(
            width: double.maxFinite,
            height: 500,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: Colors.grey[100],
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.folder, size: 16, color: Colors.orange),
                      SizedBox(width: 4),
                      Text(server['configFile'], style: TextStyle(fontSize: 12, fontFamily: 'monospace')),
                      Spacer(),
                      Container(
                        padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: _getStatusColor(server['status']),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Text(server['status'].toUpperCase(), style: TextStyle(fontSize: 10, color: Colors.white, fontWeight: FontWeight.bold)),
                      ),
                    ],
                  ),
                ),
                SizedBox(height: 16),
                Expanded(
                  child: Container(
                    padding: EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Colors.grey[50],
                      border: Border.all(color: Colors.grey[300]!),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: SingleChildScrollView(
                      child: SelectableText(
                        configContent,
                        style: TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 13,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Close'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Opening ${server['configFile']} in external editor...');
              },
              icon: Icon(Icons.edit, size: 16),
              label: Text('Edit File'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Reloading ${server['type']} configuration for ${server['name']}...');
              },
              icon: Icon(Icons.refresh, size: 16),
              label: Text('Reload Config'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
            ),
          ],
        );
      },
    );
  }

  List<Map<String, dynamic>> _getDatabaseInstances() {
    return [
      {
        'id': 1,
        'name': 'Main Grid Database',
        'type': 'PostgreSQL',
        'host': 'localhost:5432',
        'status': 'connected',
        'size': '15.2GB',
        'tables': 47,
        'lastBackup': '2h ago',
        'connections': 23,
        'version': '15.3'
      },
      {
        'id': 2,
        'name': 'User Data MySQL',
        'type': 'MySQL',
        'host': 'localhost:3306',
        'status': 'connected',
        'size': '8.7GB',
        'tables': 31,
        'lastBackup': '1h ago',
        'connections': 15,
        'version': '8.0.33'
      },
      {
        'id': 3,
        'name': 'Local SQLite Cache',
        'type': 'SQLite',
        'host': 'local file',
        'status': 'connected',
        'size': '2.1GB',
        'tables': 18,
        'lastBackup': '4h ago',
        'connections': 8,
        'version': '3.42.0'
      },
      {
        'id': 4,
        'name': 'Analytics Database',
        'type': 'PostgreSQL',
        'host': 'analytics.db:5432',
        'status': 'maintenance',
        'size': '25.8GB',
        'tables': 62,
        'lastBackup': '6h ago',
        'connections': 0,
        'version': '15.3'
      },
    ];
  }

  List<Map<String, dynamic>> _getRecentBackups() {
    return [
      {
        'name': 'main-grid-backup-20250702-030000.sql',
        'database': 'Main Grid Database',
        'size': '4.2GB',
        'date': '2h ago',
        'type': 'Full',
        'status': 'completed',
      },
      {
        'name': 'user-data-backup-20250702-020000.sql',
        'database': 'User Data MySQL',
        'size': '1.8GB',
        'date': '3h ago',
        'type': 'Incremental',
        'status': 'completed',
      },
      {
        'name': 'analytics-backup-20250701-030000.sql',
        'database': 'Analytics Database',
        'size': '8.1GB',
        'date': '1d ago',
        'type': 'Full',
        'status': 'completed',
      },
      {
        'name': 'sqlite-cache-backup-20250701-030000.db',
        'database': 'Local SQLite Cache',
        'size': '512MB',
        'date': '1d ago',
        'type': 'Full',
        'status': 'completed',
      },
    ];
  }

  Widget _buildDbStatusCard(String name, String status, String size, IconData icon, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(height: 8),
          Text(name, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
          Text(status, style: TextStyle(color: color, fontSize: 11)),
          Text(size, style: TextStyle(fontSize: 10, color: Colors.grey[600])),
        ],
      ),
    );
  }

  Widget _buildBackupStatCard(String label, String value, IconData icon, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(height: 8),
          Text(value, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16, color: color)),
          Text(label, style: TextStyle(fontSize: 11, color: Colors.grey[600])),
        ],
      ),
    );
  }

  Widget _buildDatabaseRow(Map<String, dynamic> db) {
    Color statusColor = _getStatusColor(db['status']);
    Color typeColor = db['type'] == 'PostgreSQL' ? Colors.blue : 
                     db['type'] == 'MySQL' ? Colors.orange : Colors.green;
    
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.2)))),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(db['name'], style: TextStyle(fontWeight: FontWeight.w500)),
                SizedBox(height: 4),
                Text('${db['host']} • ${db['connections']} connections • v${db['version']}', 
                     style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: typeColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(db['type'], style: TextStyle(color: typeColor, fontSize: 12, fontWeight: FontWeight.bold), textAlign: TextAlign.center),
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(db['status'], style: TextStyle(color: statusColor, fontSize: 12), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text('${db['size']}', textAlign: TextAlign.center, style: TextStyle(fontFamily: 'monospace'))),
          Expanded(child: Text('${db['tables']}', textAlign: TextAlign.center)),
          Expanded(child: Text('${db['lastBackup']}', textAlign: TextAlign.center, style: TextStyle(fontSize: 12))),
          Expanded(
            child: Wrap(
              spacing: 4,
              children: [
                _buildActionButton(
                  icon: Icons.monitor,
                  tooltip: 'Monitor',
                  color: Colors.blue,
                  onPressed: () => _showSnackBar('Opening database monitor for ${db['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.backup,
                  tooltip: 'Backup',
                  color: Colors.purple,
                  onPressed: () => _showSnackBar('Creating backup for ${db['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.restore,
                  tooltip: 'Restore',
                  color: Colors.orange,
                  onPressed: () => _showRestoreBackupDialog(targetDatabase: db),
                ),
                _buildActionButton(
                  icon: Icons.merge,
                  tooltip: 'Merge',
                  color: Colors.deepPurple,
                  onPressed: () => _showMergeDatabaseDialog(targetDatabase: db),
                ),
                _buildActionButton(
                  icon: Icons.settings,
                  tooltip: 'Config',
                  color: Colors.grey,
                  onPressed: () => _showDatabaseConfigDialog(db),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBackupStrategyItem(String name, String schedule, IconData icon, bool enabled) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Icon(icon, size: 16, color: enabled ? Colors.green : Colors.grey),
          SizedBox(width: 8),
          Expanded(child: Text(name, style: TextStyle(fontSize: 14))),
          Text(schedule, style: TextStyle(fontSize: 12, color: Colors.grey[600])),
          SizedBox(width: 8),
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: enabled ? Colors.green : Colors.grey,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildBackupItem(Map<String, dynamic> backup) {
    return Container(
      margin: EdgeInsets.only(bottom: 8),
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.grey[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[200]!),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.archive, size: 16, color: Colors.blue),
              SizedBox(width: 8),
              Expanded(
                child: Text(backup['name'], style: TextStyle(fontSize: 12, fontFamily: 'monospace')),
              ),
              Container(
                padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: backup['type'] == 'Full' ? Colors.blue.withValues(alpha: 0.2) : Colors.orange.withValues(alpha: 0.2),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(backup['type'], style: TextStyle(fontSize: 10, fontWeight: FontWeight.bold)),
              ),
            ],
          ),
          SizedBox(height: 4),
          Row(
            children: [
              Text(backup['database'], style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              Spacer(),
              Text('${backup['size']} • ${backup['date']}', style: TextStyle(fontSize: 10, color: Colors.grey[500])),
            ],
          ),
        ],
      ),
    );
  }

  void _showCreateBackupDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String selectedDatabase = 'Main Grid Database';
        String backupType = 'Full';
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.backup, color: Colors.purple),
                  SizedBox(width: 8),
                  Text('Create Database Backup'),
                ],
              ),
              content: Container(
                width: 400,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    DropdownButtonFormField<String>(
                      value: selectedDatabase,
                      decoration: InputDecoration(
                        labelText: 'Select Database',
                        border: OutlineInputBorder(),
                      ),
                      items: _getDatabaseInstances().map((db) {
                        return DropdownMenuItem<String>(
                          value: db['name'],
                          child: Text(db['name']),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          selectedDatabase = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: backupType,
                      decoration: InputDecoration(
                        labelText: 'Backup Type',
                        border: OutlineInputBorder(),
                      ),
                      items: ['Full', 'Incremental', 'Differential'].map((String type) {
                        return DropdownMenuItem<String>(
                          value: type,
                          child: Text(type),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          backupType = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    Container(
                      padding: EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.blue.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Backup Details:', style: TextStyle(fontWeight: FontWeight.bold)),
                          SizedBox(height: 4),
                          Text('Location: /backups/${DateTime.now().toString().substring(0, 10)}/'),
                          Text('Compression: gzip'),
                          Text('Encryption: AES-256'),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Creating $backupType backup for $selectedDatabase...');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
                  child: Text('Create Backup'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showAddDatabaseDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String dbType = 'PostgreSQL';
        String dbName = '';
        String host = 'localhost';
        String port = '5432';
        String username = '';
        String password = '';
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.add, color: Colors.green),
                  SizedBox(width: 8),
                  Text('Add Database Connection'),
                ],
              ),
              content: Container(
                width: 400,
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      DropdownButtonFormField<String>(
                        value: dbType,
                        decoration: InputDecoration(
                          labelText: 'Database Type',
                          border: OutlineInputBorder(),
                        ),
                        items: ['PostgreSQL', 'MySQL', 'SQLite'].map((String type) {
                          return DropdownMenuItem<String>(
                            value: type,
                            child: Text(type),
                          );
                        }).toList(),
                        onChanged: (String? newValue) {
                          setState(() {
                            dbType = newValue!;
                            port = dbType == 'PostgreSQL' ? '5432' : 
                                   dbType == 'MySQL' ? '3306' : '0';
                          });
                        },
                      ),
                      SizedBox(height: 16),
                      TextField(
                        decoration: InputDecoration(
                          labelText: 'Database Name',
                          border: OutlineInputBorder(),
                        ),
                        onChanged: (value) => dbName = value,
                      ),
                      SizedBox(height: 16),
                      Row(
                        children: [
                          Expanded(
                            flex: 3,
                            child: TextField(
                              decoration: InputDecoration(
                                labelText: 'Host',
                                border: OutlineInputBorder(),
                              ),
                              onChanged: (value) => host = value,
                            ),
                          ),
                          SizedBox(width: 8),
                          Expanded(
                            flex: 1,
                            child: TextField(
                              decoration: InputDecoration(
                                labelText: 'Port',
                                border: OutlineInputBorder(),
                              ),
                              keyboardType: TextInputType.number,
                              controller: TextEditingController(text: port),
                              onChanged: (value) => port = value,
                            ),
                          ),
                        ],
                      ),
                      SizedBox(height: 16),
                      TextField(
                        decoration: InputDecoration(
                          labelText: 'Username',
                          border: OutlineInputBorder(),
                        ),
                        onChanged: (value) => username = value,
                      ),
                      SizedBox(height: 16),
                      TextField(
                        decoration: InputDecoration(
                          labelText: 'Password',
                          border: OutlineInputBorder(),
                        ),
                        obscureText: true,
                        onChanged: (value) => password = value,
                      ),
                    ],
                  ),
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Testing connection to $dbType database $dbName...');
                  },
                  child: Text('Test Connection'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Adding $dbType database connection: $dbName');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                  child: Text('Add Database'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showDatabaseConfigDialog(Map<String, dynamic> db) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.settings, color: Colors.orange),
              SizedBox(width: 8),
              Text('Database Configuration'),
              Spacer(),
              Container(
                padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: db['type'] == 'PostgreSQL' ? Colors.blue : 
                         db['type'] == 'MySQL' ? Colors.orange : Colors.green,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(db['type'], style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
              ),
            ],
          ),
          content: Container(
            width: double.maxFinite,
            height: 400,
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Connection Settings', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Database Name', db['name']),
                  _buildConfigField('Host', db['host']),
                  _buildConfigField('Version', db['version']),
                  _buildConfigField('Current Connections', '${db['connections']}'),
                  
                  SizedBox(height: 16),
                  Text('Performance Settings', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Max Connections', '100'),
                  _buildConfigField('Connection Timeout', '30s'),
                  _buildConfigField('Query Timeout', '300s'),
                  _buildConfigField('Pool Size', '20'),
                  
                  SizedBox(height: 16),
                  Text('Backup Settings', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Backup Schedule', 'Daily 3:00 AM'),
                  _buildConfigField('Retention Period', '30 days'),
                  _buildConfigField('Backup Location', '/backups/${db['name'].toLowerCase().replaceAll(' ', '_')}/'),
                  _buildConfigField('Last Backup', db['lastBackup']),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Close'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Testing connection to ${db['name']}...');
              },
              icon: Icon(Icons.wifi, size: 16),
              label: Text('Test Connection'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Reloading configuration for ${db['name']}...');
              },
              icon: Icon(Icons.refresh, size: 16),
              label: Text('Reload Config'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
            ),
          ],
        );
      },
    );
  }

  void _showBackupStrategyDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.settings, color: Colors.purple),
              SizedBox(width: 8),
              Text('Configure Backup Strategy'),
            ],
          ),
          content: Container(
            width: 500,
            height: 400,
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Automated Backup Schedule', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Daily Backup Time', '3:00 AM UTC'),
                  _buildConfigField('Weekly Full Backup', 'Sundays 2:00 AM UTC'),
                  _buildConfigField('Monthly Archive', '1st of each month'),
                  
                  SizedBox(height: 16),
                  Text('Retention Policy', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Daily Backups', 'Keep 7 days'),
                  _buildConfigField('Weekly Backups', 'Keep 4 weeks'),
                  _buildConfigField('Monthly Archives', 'Keep 12 months'),
                  _buildConfigField('Yearly Archives', 'Keep 5 years'),
                  
                  SizedBox(height: 16),
                  Text('Storage Configuration', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Primary Location', '/backups/local/'),
                  _buildConfigField('Cloud Storage', 'AWS S3 (us-east-1)'),
                  _buildConfigField('Encryption', 'AES-256'),
                  _buildConfigField('Compression', 'gzip -9'),
                  
                  SizedBox(height: 16),
                  Text('Disaster Recovery', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                  SizedBox(height: 8),
                  _buildConfigField('Recovery Time Objective', '4 hours'),
                  _buildConfigField('Recovery Point Objective', '1 hour'),
                  _buildConfigField('Offsite Replication', 'Real-time to AWS'),
                  _buildConfigField('Failover Testing', 'Monthly'),
                ],
              ),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Cancel'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Testing backup strategy configuration...');
              },
              icon: Icon(Icons.play_arrow, size: 16),
              label: Text('Test Strategy'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Backup strategy configuration saved successfully');
              },
              icon: Icon(Icons.save, size: 16),
              label: Text('Save Strategy'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
            ),
          ],
        );
      },
    );
  }

  void _showAllBackupsDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.archive, color: Colors.blue),
              SizedBox(width: 8),
              Text('All Database Backups'),
            ],
          ),
          content: Container(
            width: double.maxFinite,
            height: 500,
            child: ListView.builder(
              itemCount: _getRecentBackups().length * 3, // Show more backups
              itemBuilder: (context, index) {
                final backup = _getRecentBackups()[index % _getRecentBackups().length];
                return _buildBackupItem(backup);
              },
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Close'),
            ),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                _showSnackBar('Cleaning up old backups...');
              },
              icon: Icon(Icons.cleaning_services, size: 16),
              label: Text('Cleanup Old'),
            ),
          ],
        );
      },
    );
  }

  void _showRestoreBackupDialog({Map<String, dynamic>? targetDatabase}) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String selectedBackup = _getRecentBackups().first['name'];
        String selectedDatabase = targetDatabase?['name'] ?? _getDatabaseInstances().first['name'];
        String restoreMode = 'Complete Replace';
        bool createBackupFirst = true;
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.restore, color: Colors.orange),
                  SizedBox(width: 8),
                  Text('Restore Database Backup'),
                ],
              ),
              content: Container(
                width: 500,
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Backup Selection
                      Text('Source Backup', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: selectedBackup,
                        decoration: InputDecoration(
                          labelText: 'Select Backup File',
                          border: OutlineInputBorder(),
                        ),
                        items: [
                          ..._getRecentBackups().map((backup) {
                            return DropdownMenuItem<String>(
                              value: backup['name'],
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(backup['name'], style: TextStyle(fontSize: 12, fontFamily: 'monospace')),
                                  Text('${backup['database']} • ${backup['size']} • ${backup['date']}', 
                                       style: TextStyle(fontSize: 10, color: Colors.grey[600])),
                                ],
                              ),
                            );
                          }),
                          DropdownMenuItem<String>(
                            value: 'custom',
                            child: Text('🗂️ Browse for backup file...', style: TextStyle(color: Colors.blue)),
                          ),
                        ],
                        onChanged: (String? newValue) {
                          setState(() {
                            selectedBackup = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Target Database
                      Text('Target Database', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: selectedDatabase,
                        decoration: InputDecoration(
                          labelText: 'Target Database',
                          border: OutlineInputBorder(),
                        ),
                        items: _getDatabaseInstances().map((db) {
                          return DropdownMenuItem<String>(
                            value: db['name'],
                            child: Row(
                              children: [
                                Container(
                                  width: 8,
                                  height: 8,
                                  decoration: BoxDecoration(
                                    shape: BoxShape.circle,
                                    color: db['type'] == 'PostgreSQL' ? Colors.blue : 
                                           db['type'] == 'MySQL' ? Colors.orange : Colors.green,
                                  ),
                                ),
                                SizedBox(width: 8),
                                Expanded(child: Text(db['name'])),
                                Text(db['type'], style: TextStyle(fontSize: 12, color: Colors.grey[600])),
                              ],
                            ),
                          );
                        }).toList(),
                        onChanged: (String? newValue) {
                          setState(() {
                            selectedDatabase = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Restore Mode
                      Text('Restore Mode', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: restoreMode,
                        decoration: InputDecoration(
                          labelText: 'Restore Method',
                          border: OutlineInputBorder(),
                        ),
                        items: [
                          DropdownMenuItem(value: 'Complete Replace', child: Text('Complete Replace (⚠️ Overwrites all data)')),
                          DropdownMenuItem(value: 'Merge Data', child: Text('Merge Data (Combines with existing)')),
                          DropdownMenuItem(value: 'Selective Tables', child: Text('Selective Tables (Choose specific tables)')),
                          DropdownMenuItem(value: 'Schema Only', child: Text('Schema Only (Structure without data)')),
                          DropdownMenuItem(value: 'Data Only', child: Text('Data Only (Data without structure)')),
                        ],
                        onChanged: (String? newValue) {
                          setState(() {
                            restoreMode = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Safety Options
                      CheckboxListTile(
                        title: Text('Create backup before restore'),
                        subtitle: Text('Recommended: Create safety backup of current data'),
                        value: createBackupFirst,
                        onChanged: (bool? value) {
                          setState(() {
                            createBackupFirst = value ?? true;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Warning/Info Box
                      Container(
                        padding: EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: restoreMode == 'Complete Replace' ? Colors.red.withValues(alpha: 0.1) : Colors.blue.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: restoreMode == 'Complete Replace' ? Colors.red.withValues(alpha: 0.3) : Colors.blue.withValues(alpha: 0.3),
                          ),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              children: [
                                Icon(
                                  restoreMode == 'Complete Replace' ? Icons.warning : Icons.info,
                                  color: restoreMode == 'Complete Replace' ? Colors.red : Colors.blue,
                                  size: 16,
                                ),
                                SizedBox(width: 8),
                                Text(
                                  restoreMode == 'Complete Replace' ? 'Destructive Operation' : 'Restore Information',
                                  style: TextStyle(fontWeight: FontWeight.bold),
                                ),
                              ],
                            ),
                            SizedBox(height: 4),
                            Text(
                              restoreMode == 'Complete Replace' 
                                ? 'This will permanently delete all current data in $selectedDatabase and replace it with backup data.'
                                : 'Backup will be restored to $selectedDatabase using $restoreMode method.',
                              style: TextStyle(fontSize: 12),
                            ),
                            if (createBackupFirst) ...[
                              SizedBox(height: 4),
                              Text(
                                '✅ Safety backup will be created first: ${selectedDatabase}_pre_restore_${DateTime.now().millisecondsSinceEpoch}',
                                style: TextStyle(fontSize: 11, color: Colors.green[700]),
                              ),
                            ],
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                if (restoreMode == 'Selective Tables')
                  ElevatedButton.icon(
                    onPressed: () {
                      Navigator.of(context).pop();
                      _showSelectTablesDialog(selectedBackup, selectedDatabase);
                    },
                    icon: Icon(Icons.table_chart, size: 16),
                    label: Text('Select Tables'),
                    style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
                  ),
                ElevatedButton.icon(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('${createBackupFirst ? 'Creating safety backup and r' : 'R'}estoring $selectedBackup to $selectedDatabase using $restoreMode...');
                  },
                  icon: Icon(Icons.restore, size: 16),
                  label: Text('Start Restore'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: restoreMode == 'Complete Replace' ? Colors.red : Colors.orange,
                  ),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showMergeDatabaseDialog({Map<String, dynamic>? targetDatabase}) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String sourceDatabase = _getDatabaseInstances().first['name'];
        String targetDb = targetDatabase?['name'] ?? _getDatabaseInstances()[1]['name'];
        String mergeStrategy = 'Smart Merge';
        String conflictResolution = 'Source Wins';
        bool preserveTargetSchema = true;
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.merge, color: Colors.purple),
                  SizedBox(width: 8),
                  Text('Merge Database'),
                ],
              ),
              content: Container(
                width: 500,
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Source Database
                      Text('Source Database (merge FROM)', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: sourceDatabase,
                        decoration: InputDecoration(
                          labelText: 'Source Database',
                          border: OutlineInputBorder(),
                        ),
                        items: _getDatabaseInstances().map((db) {
                          return DropdownMenuItem<String>(
                            value: db['name'],
                            child: Row(
                              children: [
                                Container(
                                  width: 8,
                                  height: 8,
                                  decoration: BoxDecoration(
                                    shape: BoxShape.circle,
                                    color: db['type'] == 'PostgreSQL' ? Colors.blue : 
                                           db['type'] == 'MySQL' ? Colors.orange : Colors.green,
                                  ),
                                ),
                                SizedBox(width: 8),
                                Expanded(child: Text(db['name'])),
                                Text('${db['tables']} tables • ${db['size']}', style: TextStyle(fontSize: 12, color: Colors.grey[600])),
                              ],
                            ),
                          );
                        }).toList(),
                        onChanged: (String? newValue) {
                          setState(() {
                            sourceDatabase = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Target Database
                      Text('Target Database (merge INTO)', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: targetDb,
                        decoration: InputDecoration(
                          labelText: 'Target Database',
                          border: OutlineInputBorder(),
                        ),
                        items: _getDatabaseInstances().map((db) {
                          return DropdownMenuItem<String>(
                            value: db['name'],
                            child: Row(
                              children: [
                                Container(
                                  width: 8,
                                  height: 8,
                                  decoration: BoxDecoration(
                                    shape: BoxShape.circle,
                                    color: db['type'] == 'PostgreSQL' ? Colors.blue : 
                                           db['type'] == 'MySQL' ? Colors.orange : Colors.green,
                                  ),
                                ),
                                SizedBox(width: 8),
                                Expanded(child: Text(db['name'])),
                                Text('${db['tables']} tables • ${db['size']}', style: TextStyle(fontSize: 12, color: Colors.grey[600])),
                              ],
                            ),
                          );
                        }).toList(),
                        onChanged: (String? newValue) {
                          setState(() {
                            targetDb = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Merge Strategy
                      Text('Merge Strategy', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: mergeStrategy,
                        decoration: InputDecoration(
                          labelText: 'How to merge data',
                          border: OutlineInputBorder(),
                        ),
                        items: [
                          DropdownMenuItem(value: 'Smart Merge', child: Text('Smart Merge (Automatic conflict detection)')),
                          DropdownMenuItem(value: 'Append Only', child: Text('Append Only (Add new records only)')),
                          DropdownMenuItem(value: 'Schema Merge', child: Text('Schema Merge (Tables and structure only)')),
                          DropdownMenuItem(value: 'Cross-Platform', child: Text('Cross-Platform (MySQL ↔ PostgreSQL)')),
                          DropdownMenuItem(value: 'Custom Rules', child: Text('Custom Rules (User-defined mapping)')),
                        ],
                        onChanged: (String? newValue) {
                          setState(() {
                            mergeStrategy = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Conflict Resolution
                      Text('Conflict Resolution', style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue)),
                      SizedBox(height: 8),
                      DropdownButtonFormField<String>(
                        value: conflictResolution,
                        decoration: InputDecoration(
                          labelText: 'When records conflict',
                          border: OutlineInputBorder(),
                        ),
                        items: [
                          DropdownMenuItem(value: 'Source Wins', child: Text('Source Wins (Overwrite with source data)')),
                          DropdownMenuItem(value: 'Target Wins', child: Text('Target Wins (Keep existing target data)')),
                          DropdownMenuItem(value: 'Newest Wins', child: Text('Newest Wins (Based on timestamp)')),
                          DropdownMenuItem(value: 'Manual Review', child: Text('Manual Review (Flag conflicts for review)')),
                          DropdownMenuItem(value: 'Create Duplicates', child: Text('Create Duplicates (Keep both versions)')),
                        ],
                        onChanged: (String? newValue) {
                          setState(() {
                            conflictResolution = newValue!;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Options
                      CheckboxListTile(
                        title: Text('Preserve target database schema'),
                        subtitle: Text('Keep existing table structures intact'),
                        value: preserveTargetSchema,
                        onChanged: (bool? value) {
                          setState(() {
                            preserveTargetSchema = value ?? true;
                          });
                        },
                      ),
                      
                      SizedBox(height: 16),
                      
                      // Migration Preview
                      Container(
                        padding: EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: Colors.blue.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Merge Preview:', style: TextStyle(fontWeight: FontWeight.bold)),
                            SizedBox(height: 4),
                            Text('📤 Source: $sourceDatabase'),
                            Text('📥 Target: $targetDb'),
                            Text('🔄 Strategy: $mergeStrategy'),
                            Text('⚖️ Conflicts: $conflictResolution'),
                            if (sourceDatabase != targetDb && mergeStrategy == 'Cross-Platform') ...[
                              SizedBox(height: 4),
                              Text('🔄 Cross-platform migration detected', style: TextStyle(color: Colors.orange[700])),
                            ],
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton.icon(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Analyzing merge compatibility between $sourceDatabase and $targetDb...');
                  },
                  icon: Icon(Icons.analytics, size: 16),
                  label: Text('Analyze'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
                ),
                ElevatedButton.icon(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Starting database merge: $sourceDatabase → $targetDb using $mergeStrategy strategy...');
                  },
                  icon: Icon(Icons.merge, size: 16),
                  label: Text('Start Merge'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.purple),
                ),
              ],
            );
          },
        );
      },
    );
  }

  void _showSelectTablesDialog(String backupFile, String targetDatabase) {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        // Mock table list from backup
        List<Map<String, dynamic>> availableTables = [
          {'name': 'users', 'records': 1247, 'size': '2.1MB', 'selected': true},
          {'name': 'regions', 'records': 8, 'size': '45KB', 'selected': true},
          {'name': 'assets', 'records': 15420, 'size': '8.7GB', 'selected': false},
          {'name': 'inventory', 'records': 9876, 'size': '1.2GB', 'selected': true},
          {'name': 'transactions', 'records': 5432, 'size': '876MB', 'selected': false},
          {'name': 'friends', 'records': 2341, 'size': '124KB', 'selected': true},
          {'name': 'groups', 'records': 156, 'size': '67KB', 'selected': true},
          {'name': 'estates', 'records': 23, 'size': '12KB', 'selected': true},
        ];
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.table_chart, color: Colors.blue),
                  SizedBox(width: 8),
                  Text('Select Tables to Restore'),
                ],
              ),
              content: Container(
                width: 500,
                height: 400,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Backup: $backupFile', style: TextStyle(fontFamily: 'monospace', fontSize: 12)),
                    Text('Target: $targetDatabase', style: TextStyle(fontFamily: 'monospace', fontSize: 12)),
                    SizedBox(height: 16),
                    Row(
                      children: [
                        ElevatedButton(
                          onPressed: () {
                            setState(() {
                              for (var table in availableTables) {
                                table['selected'] = true;
                              }
                            });
                          },
                          child: Text('Select All'),
                        ),
                        SizedBox(width: 8),
                        ElevatedButton(
                          onPressed: () {
                            setState(() {
                              for (var table in availableTables) {
                                table['selected'] = false;
                              }
                            });
                          },
                          child: Text('Select None'),
                        ),
                      ],
                    ),
                    SizedBox(height: 16),
                    Expanded(
                      child: ListView.builder(
                        itemCount: availableTables.length,
                        itemBuilder: (context, index) {
                          final table = availableTables[index];
                          return CheckboxListTile(
                            title: Text(table['name'], style: TextStyle(fontFamily: 'monospace')),
                            subtitle: Text('${table['records']} records • ${table['size']}'),
                            value: table['selected'],
                            onChanged: (bool? value) {
                              setState(() {
                                table['selected'] = value ?? false;
                              });
                            },
                          );
                        },
                      ),
                    ),
                    Container(
                      padding: EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: Colors.blue.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        'Selected: ${availableTables.where((t) => t['selected']).length} tables',
                        style: TextStyle(fontWeight: FontWeight.bold),
                      ),
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton.icon(
                  onPressed: () {
                    final selectedTables = availableTables.where((t) => t['selected']).map((t) => t['name']).toList();
                    Navigator.of(context).pop();
                    _showSnackBar('Restoring ${selectedTables.length} selected tables: ${selectedTables.join(', ')}');
                  },
                  icon: Icon(Icons.restore, size: 16),
                  label: Text('Restore Selected'),
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                ),
              ],
            );
          },
        );
      },
    );
  }

  String _getServerConfigContent(Map<String, dynamic> server) {
    if (server['type'] == 'Grid') {
      return '''[Startup]
; OpenSim Grid Configuration - ${server['name']}
; Generated by OpenSim Next Web Admin

[Const]
GridName = "OpenSim Next Grid"
GridOwner = "Grid Administrator"
GridURL = "http://localhost:${server['port']}"

[Network]
; Grid server network configuration
http_listener_port = ${server['port']}
console_port = ${server['port'] + 1000}

; External hostname for grid services
ExternalHostName = localhost

[GridService]
; Main grid service configuration
GridService = "OpenSim.Services.GridService.dll:GridService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/grid.db;Version=3;New=True;"

; Default region settings
DefaultRegionSize = 256
MaxRegionSize = 4096
RegionDefaultLocation = 1000,1000

[PresenceService]
; User presence tracking
PresenceService = "OpenSim.Services.PresenceService.dll:PresenceService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/presence.db;Version=3;New=True;"

[UserAccountService]
; User account management
UserAccountService = "OpenSim.Services.UserAccountService.dll:UserAccountService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/useraccounts.db;Version=3;New=True;"

[AuthenticationService]
; User authentication
AuthenticationService = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/auth.db;Version=3;New=True;"

[AvatarService]
; Avatar appearance service
AvatarService = "OpenSim.Services.AvatarService.dll:AvatarService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/avatars.db;Version=3;New=True;"

[InventoryService]
; Inventory management
InventoryService = "OpenSim.Services.InventoryService.dll:XInventoryService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/inventory.db;Version=3;New=True;"

[AssetService]
; Asset service configuration
AssetService = "OpenSim.Services.AssetService.dll:AssetService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/assets.db;Version=3;New=True;"

[FriendsService]
; Friends list management
FriendsService = "OpenSim.Services.FriendsService.dll:FriendsService"
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/friends.db;Version=3;New=True;"

[GridInfoService]
; Grid information service
GridInfoService = "OpenSim.Services.GridInfoService.dll:GridInfoService"
GridName = "OpenSim Next Grid"
GridNick = "osng"
GridOwner = "Grid Administrator"
GridOwnerEmail = "admin@opensim.local"

[LoginService]
; Login service configuration
LoginService = "OpenSim.Services.LLLoginService.dll:LLLoginService"
UserAccountService = UserAccountService
GridUserService = GridUserService
AuthenticationService = AuthenticationService
InventoryService = InventoryService
PresenceService = PresenceService
GridService = GridService
AvatarService = AvatarService
FriendsService = FriendsService

; Welcome message
WelcomeMessage = "Welcome to OpenSim Next Grid!"

; Status: ${server['status']}
; Port: ${server['port']}
; Uptime: ${server['uptime']}
; Memory: ${server['memory']}
; CPU: ${server['cpu']}
''';
    } else {
      return '''[Startup]
; OpenSim Standalone Configuration - ${server['name']}
; Generated by OpenSim Next Web Admin

[Const]
; Basic standalone server settings
BaseURL = "http://localhost"
PublicPort = "${server['port']}"

[Network]
; Network configuration
http_listener_port = ${server['port']}
console_port = ${server['port'] + 1000}

; External hostname that clients will use to connect
ExternalHostName = localhost

[DatabaseService]
; Standalone database configuration
StorageProvider = "OpenSim.Data.SQLite.dll"
ConnectionString = "Data Source=config/standalone.db;Version=3;New=True;"

[Architecture]
; Architecture modules for standalone mode
Include-Architecture = "config/StandaloneCommon.ini"

[Modules]
; Core modules
AssetServices = "LocalAssetServicesConnector"
InventoryServices = "LocalInventoryServicesConnector"
NeighbourServices = "LocalNeighbourServicesConnector"
AuthenticationServices = "LocalAuthenticationServicesConnector"
AuthorizationServices = "LocalAuthorizationServicesConnector"
GridServices = "LocalGridServicesConnector"
PresenceServices = "LocalPresenceServicesConnector"
UserAccountServices = "LocalUserAccountServicesConnector"
GridUserServices = "LocalGridUserServicesConnector"
AvatarServices = "LocalAvatarServicesConnector"
FriendsServices = "LocalFriendsServicesConnector"

[LocalServiceModule]
; Local services for standalone operation
LocalServiceModule = "OpenSim.Services.AssetService.dll:AssetService"
LocalServiceModule = "OpenSim.Services.InventoryService.dll:XInventoryService"
LocalServiceModule = "OpenSim.Services.GridService.dll:GridService"
LocalServiceModule = "OpenSim.Services.PresenceService.dll:PresenceService"
LocalServiceModule = "OpenSim.Services.UserAccountService.dll:UserAccountService"
LocalServiceModule = "OpenSim.Services.AuthenticationService.dll:PasswordAuthenticationService"
LocalServiceModule = "OpenSim.Services.AvatarService.dll:AvatarService"
LocalServiceModule = "OpenSim.Services.FriendsService.dll:FriendsService"

[LibraryService]
; Asset library
LibraryName = "OpenSim Library"
DefaultLibrary = "./inventory/Libraries.xml"

[LoginService]
; Login service for standalone
WelcomeMessage = "Welcome to OpenSim Standalone Server!"
AllowAnonymousLogin = false
AllowDuplicatePresences = false

[Hypergrid]
; Hypergrid settings for connecting to other grids
Enabled = true
HomeURI = "http://localhost:${server['port']}"
GatekeeperURI = "http://localhost:${server['port']}"

[UserProfiles]
; User profiles module
ProfileServiceURL = "http://localhost:${server['port']}"

[Search]
; Search module
Module = BasicSearchModule

[Economy]
; Economy settings
EconomyModule = BetaGridLikeMoneyModule
CurrencySymbol = "OS\$"
PriceUpload = 0

[Physics]
; Physics engine
DefaultPhysicsEngine = OpenDynamicsEngine

[Status]
; Current server status
; Status: ${server['status']}
; Port: ${server['port']}
; Uptime: ${server['uptime']}
; Memory: ${server['memory']}
; CPU: ${server['cpu']}
''';
    }
  }

  // Container Management Methods
  List<Map<String, dynamic>> _getContainerInstances() {
    return [
      {
        'id': 1,
        'name': 'opensim-grid-server',
        'image': 'opensim/grid:latest',
        'status': 'running',
        'port': '9000:9000',
        'cpu': '15%',
        'memory': '1.2GB',
        'uptime': '2d 4h',
        'restart': 'always'
      },
      {
        'id': 2,
        'name': 'opensim-web-interface',
        'image': 'opensim/web:v2.1',
        'status': 'running',
        'port': '8080:80',
        'cpu': '8%',
        'memory': '512MB',
        'uptime': '2d 4h',
        'restart': 'always'
      },
      {
        'id': 3,
        'name': 'redis-cache',
        'image': 'redis:alpine',
        'status': 'running',
        'port': '6379:6379',
        'cpu': '3%',
        'memory': '128MB',
        'uptime': '2d 4h',
        'restart': 'always'
      },
      {
        'id': 4,
        'name': 'postgres-main',
        'image': 'postgres:15-alpine',
        'status': 'running',
        'port': '5432:5432',
        'cpu': '12%',
        'memory': '800MB',
        'uptime': '2d 4h',
        'restart': 'always'
      },
      {
        'id': 5,
        'name': 'opensim-assets',
        'image': 'opensim/assets:latest',
        'status': 'running',
        'port': '8003:8003',
        'cpu': '5%',
        'memory': '256MB',
        'uptime': '2d 4h',
        'restart': 'always'
      },
      {
        'id': 6,
        'name': 'nginx-proxy',
        'image': 'nginx:alpine',
        'status': 'stopped',
        'port': '80:80,443:443',
        'cpu': '0%',
        'memory': '0MB',
        'uptime': '0h',
        'restart': 'no'
      },
      {
        'id': 7,
        'name': 'monitoring-stack',
        'image': 'prom/prometheus:latest',
        'status': 'stopped',
        'port': '9090:9090',
        'cpu': '0%',
        'memory': '0MB',
        'uptime': '0h',
        'restart': 'no'
      },
    ];
  }

  Widget _buildContainerStatCard(String label, String value, IconData icon, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(height: 8),
          Text(value, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16, color: color)),
          Text(label, style: TextStyle(fontSize: 11, color: Colors.grey[600])),
        ],
      ),
    );
  }

  Widget _buildResourceCard(String label, String value, IconData icon, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(height: 8),
          Text(value, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16, color: color)),
          Text(label, style: TextStyle(fontSize: 11, color: Colors.grey[600])),
        ],
      ),
    );
  }

  Widget _buildContainerRow(Map<String, dynamic> container) {
    Color statusColor = container['status'] == 'running' ? Colors.green : Colors.red;
    
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.2)))),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(container['name'], style: TextStyle(fontWeight: FontWeight.w500)),
                SizedBox(height: 4),
                Text('Uptime: ${container['uptime']} • Restart: ${container['restart']}', 
                     style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
          ),
          Expanded(
            child: Text(
              container['image'],
              style: TextStyle(fontSize: 12, fontFamily: 'monospace', color: Colors.blue),
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(container['status'], style: TextStyle(color: statusColor, fontSize: 12), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text(container['port'], textAlign: TextAlign.center, style: TextStyle(fontFamily: 'monospace', fontSize: 12))),
          Expanded(child: Text('${container['cpu']} / ${container['memory']}', textAlign: TextAlign.center, style: TextStyle(fontSize: 12))),
          Expanded(
            child: Wrap(
              spacing: 4,
              children: [
                _buildActionButton(
                  icon: container['status'] == 'running' ? Icons.stop : Icons.play_arrow,
                  tooltip: container['status'] == 'running' ? 'Stop' : 'Start',
                  color: container['status'] == 'running' ? Colors.red : Colors.green,
                  onPressed: () => _showSnackBar('${container['status'] == 'running' ? 'Stopping' : 'Starting'} ${container['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.refresh,
                  tooltip: 'Restart',
                  color: Colors.orange,
                  onPressed: () => _showSnackBar('Restarting ${container['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.terminal,
                  tooltip: 'Logs',
                  color: Colors.blue,
                  onPressed: () => _showSnackBar('Opening logs for ${container['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.delete,
                  tooltip: 'Remove',
                  color: Colors.grey,
                  onPressed: () => _showSnackBar('Removing ${container['name']}...'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showCreateContainerDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String containerName = '';
        String selectedImage = 'opensim/grid:latest';
        String portMapping = '9000:9000';
        String restartPolicy = 'always';
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.add_circle, color: Colors.green),
                  SizedBox(width: 8),
                  Text('Create New Container'),
                ],
              ),
              content: Container(
                width: 400,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Container Name',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) => containerName = value,
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: selectedImage,
                      decoration: InputDecoration(
                        labelText: 'Docker Image',
                        border: OutlineInputBorder(),
                      ),
                      items: [
                        'opensim/grid:latest',
                        'opensim/standalone:latest',
                        'opensim/web:latest',
                        'opensim/assets:latest',
                        'postgres:15-alpine',
                        'redis:alpine',
                        'nginx:alpine',
                      ].map((String image) {
                        return DropdownMenuItem<String>(
                          value: image,
                          child: Text(image),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          selectedImage = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Port Mapping (host:container)',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) => portMapping = value,
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: restartPolicy,
                      decoration: InputDecoration(
                        labelText: 'Restart Policy',
                        border: OutlineInputBorder(),
                      ),
                      items: ['no', 'always', 'on-failure', 'unless-stopped'].map((String policy) {
                        return DropdownMenuItem<String>(
                          value: policy,
                          child: Text(policy),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          restartPolicy = newValue!;
                        });
                      },
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Creating container: $containerName with image $selectedImage');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                  child: Text('Create Container'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  // Cluster Management Methods
  List<Map<String, dynamic>> _getKubernetesClusters() {
    return [
      {
        'name': 'opensim-production',
        'status': 'running',
        'nodes': 3,
        'pods': 12,
        'services': 8,
        'cpu': '45%',
        'memory': '12.5GB',
        'version': 'v1.28.2'
      },
      {
        'name': 'opensim-staging',
        'status': 'running',
        'nodes': 2,
        'pods': 6,
        'services': 4,
        'cpu': '25%',
        'memory': '6.2GB',
        'version': 'v1.28.2'
      },
      {
        'name': 'opensim-development',
        'status': 'stopped',
        'nodes': 1,
        'pods': 0,
        'services': 0,
        'cpu': '0%',
        'memory': '0GB',
        'version': 'v1.28.2'
      },
    ];
  }

  Widget _buildClusterCard(Map<String, dynamic> cluster) {
    Color statusColor = cluster['status'] == 'running' ? Colors.green : Colors.red;
    
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(cluster['name'], style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
                  child: Text(cluster['status'], style: TextStyle(color: statusColor, fontSize: 12)),
                ),
              ],
            ),
            SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                _buildClusterStat('Nodes', cluster['nodes'].toString(), Icons.dns),
                _buildClusterStat('Pods', cluster['pods'].toString(), Icons.widgets),
                _buildClusterStat('Services', cluster['services'].toString(), Icons.cloud),
              ],
            ),
            SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('CPU: ${cluster['cpu']}', style: TextStyle(fontSize: 12)),
                Text('Memory: ${cluster['memory']}', style: TextStyle(fontSize: 12)),
                Text('Version: ${cluster['version']}', style: TextStyle(fontSize: 12)),
              ],
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton.icon(
                    onPressed: () => _showSnackBar('Managing cluster: ${cluster['name']}...'),
                    icon: Icon(Icons.settings, size: 16),
                    label: Text('Manage'),
                    style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
                  ),
                ),
                SizedBox(width: 8),
                Expanded(
                  child: ElevatedButton.icon(
                    onPressed: () => _showSnackBar('Scaling cluster: ${cluster['name']}...'),
                    icon: Icon(Icons.trending_up, size: 16),
                    label: Text('Scale'),
                    style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildClusterStat(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, size: 20, color: Colors.blue),
        SizedBox(height: 4),
        Text(value, style: TextStyle(fontWeight: FontWeight.bold)),
        Text(label, style: TextStyle(fontSize: 11, color: Colors.grey[600])),
      ],
    );
  }

  void _showCreateClusterDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String clusterName = '';
        int nodeCount = 3;
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.add_circle, color: Colors.green),
                  SizedBox(width: 8),
                  Text('Create Kubernetes Cluster'),
                ],
              ),
              content: Container(
                width: 400,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Cluster Name',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) => clusterName = value,
                    ),
                    SizedBox(height: 16),
                    Row(
                      children: [
                        Text('Node Count: '),
                        Expanded(
                          child: Slider(
                            value: nodeCount.toDouble(),
                            min: 1,
                            max: 10,
                            divisions: 9,
                            label: nodeCount.toString(),
                            onChanged: (value) => setState(() => nodeCount = value.round()),
                          ),
                        ),
                        Text(nodeCount.toString()),
                      ],
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Creating Kubernetes cluster: $clusterName with $nodeCount nodes');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                  child: Text('Create Cluster'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  // Deployment Methods
  List<Map<String, dynamic>> _getActiveDeployments() {
    return [
      {
        'name': 'opensim-grid-deployment',
        'environment': 'production',
        'version': 'v2.1.3',
        'status': 'running',
        'replicas': '3/3',
        'uptime': '5d 12h',
        'health': 'healthy'
      },
      {
        'name': 'opensim-web-deployment',
        'environment': 'production',
        'version': 'v1.8.2',
        'status': 'running',
        'replicas': '2/2',
        'uptime': '5d 12h',
        'health': 'healthy'
      },
      {
        'name': 'opensim-staging-deployment',
        'environment': 'staging',
        'version': 'v2.2.0-beta',
        'status': 'updating',
        'replicas': '1/2',
        'uptime': '2h 15m',
        'health': 'degraded'
      },
      {
        'name': 'opensim-dev-deployment',
        'environment': 'development',
        'version': 'v2.2.0-dev',
        'status': 'stopped',
        'replicas': '0/1',
        'uptime': '0m',
        'health': 'offline'
      },
    ];
  }

  Widget _buildDeploymentRow(Map<String, dynamic> deployment) {
    Color statusColor = _getStatusColor(deployment['status']);
    Color envColor = deployment['environment'] == 'production' ? Colors.red : 
                    deployment['environment'] == 'staging' ? Colors.orange : Colors.blue;
    
    return Container(
      padding: EdgeInsets.all(16),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.grey.withValues(alpha: 0.2)))),
      child: Row(
        children: [
          Expanded(
            flex: 2,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(deployment['name'], style: TextStyle(fontWeight: FontWeight.w500)),
                SizedBox(height: 4),
                Text('Uptime: ${deployment['uptime']} • Health: ${deployment['health']}', 
                     style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: envColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(deployment['environment'], style: TextStyle(color: envColor, fontSize: 12, fontWeight: FontWeight.bold), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text(deployment['version'], textAlign: TextAlign.center, style: TextStyle(fontFamily: 'monospace', fontSize: 12))),
          Expanded(
            child: Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(deployment['status'], style: TextStyle(color: statusColor, fontSize: 12), textAlign: TextAlign.center),
            ),
          ),
          Expanded(child: Text(deployment['replicas'], textAlign: TextAlign.center, style: TextStyle(fontFamily: 'monospace'))),
          Expanded(
            child: Wrap(
              spacing: 4,
              children: [
                _buildActionButton(
                  icon: Icons.visibility,
                  tooltip: 'Monitor',
                  color: Colors.blue,
                  onPressed: () => _showSnackBar('Opening deployment monitor for ${deployment['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.update,
                  tooltip: 'Update',
                  color: Colors.orange,
                  onPressed: () => _showSnackBar('Updating deployment: ${deployment['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.scale,
                  tooltip: 'Scale',
                  color: Colors.purple,
                  onPressed: () => _showSnackBar('Scaling deployment: ${deployment['name']}...'),
                ),
                _buildActionButton(
                  icon: Icons.restart_alt,
                  tooltip: 'Restart',
                  color: Colors.grey,
                  onPressed: () => _showSnackBar('Restarting deployment: ${deployment['name']}...'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showDeploymentWizard() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String deploymentName = '';
        String selectedEnvironment = 'development';
        String selectedVersion = 'latest';
        int replicas = 1;
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.rocket_launch, color: Colors.deepOrange),
                  SizedBox(width: 8),
                  Text('Deployment Wizard'),
                ],
              ),
              content: Container(
                width: 450,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextField(
                      decoration: InputDecoration(
                        labelText: 'Deployment Name',
                        border: OutlineInputBorder(),
                      ),
                      onChanged: (value) => deploymentName = value,
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: selectedEnvironment,
                      decoration: InputDecoration(
                        labelText: 'Environment',
                        border: OutlineInputBorder(),
                      ),
                      items: ['development', 'staging', 'production'].map((String env) {
                        return DropdownMenuItem<String>(
                          value: env,
                          child: Text(env),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          selectedEnvironment = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    DropdownButtonFormField<String>(
                      value: selectedVersion,
                      decoration: InputDecoration(
                        labelText: 'Version',
                        border: OutlineInputBorder(),
                      ),
                      items: ['latest', 'v2.2.0', 'v2.1.3', 'v2.1.2', 'v2.0.0'].map((String version) {
                        return DropdownMenuItem<String>(
                          value: version,
                          child: Text(version),
                        );
                      }).toList(),
                      onChanged: (String? newValue) {
                        setState(() {
                          selectedVersion = newValue!;
                        });
                      },
                    ),
                    SizedBox(height: 16),
                    Row(
                      children: [
                        Text('Replicas: '),
                        Expanded(
                          child: Slider(
                            value: replicas.toDouble(),
                            min: 1,
                            max: 10,
                            divisions: 9,
                            label: replicas.toString(),
                            onChanged: (value) => setState(() => replicas = value.round()),
                          ),
                        ),
                        Text(replicas.toString()),
                      ],
                    ),
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Deploying $deploymentName v$selectedVersion to $selectedEnvironment with $replicas replicas');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.deepOrange),
                  child: Text('Deploy'),
                ),
              ],
            );
          },
        );
      },
    );
  }

  // Extension Management Methods
  List<Map<String, dynamic>> _getInstalledExtensions() {
    return [
      {
        'name': 'Advanced Physics Engine',
        'version': 'v3.2.1',
        'status': 'active',
        'author': 'OpenSim Community',
        'description': 'Enhanced physics simulation with improved collision detection',
        'size': '15MB',
        'lastUpdated': '2 weeks ago'
      },
      {
        'name': 'Economy Module Plus',
        'version': 'v2.4.0',
        'status': 'active',
        'author': 'Economy Team',
        'description': 'Advanced economy features with marketplace integration',
        'size': '8MB',
        'lastUpdated': '1 month ago'
      },
      {
        'name': 'Chat Enhancement Suite',
        'version': 'v1.8.3',
        'status': 'active',
        'author': 'Communication Inc',
        'description': 'Rich text chat, voice integration, and moderation tools',
        'size': '12MB',
        'lastUpdated': '3 weeks ago'
      },
      {
        'name': 'Advanced Scripting API',
        'version': 'v4.1.2',
        'status': 'inactive',
        'author': 'Script Foundation',
        'description': 'Extended LSL scripting capabilities and debugging tools',
        'size': '22MB',
        'lastUpdated': '2 months ago'
      },
      {
        'name': 'Web Interface Extensions',
        'version': 'v2.0.1',
        'status': 'active',
        'author': 'Web Team',
        'description': 'Additional web-based administration and user interfaces',
        'size': '18MB',
        'lastUpdated': '1 week ago'
      },
    ];
  }

  Widget _buildExtensionCard(Map<String, dynamic> extension) {
    Color statusColor = extension['status'] == 'active' ? Colors.green : Colors.grey;
    
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(
                    extension['name'],
                    style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
                  child: Text(extension['status'], style: TextStyle(color: statusColor, fontSize: 12)),
                ),
              ],
            ),
            SizedBox(height: 8),
            Text(
              extension['description'],
              style: TextStyle(color: Colors.grey[600], fontSize: 13),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('v${extension['version']}', style: TextStyle(fontSize: 12, fontFamily: 'monospace')),
                Text(extension['size'], style: TextStyle(fontSize: 12)),
              ],
            ),
            SizedBox(height: 4),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('by ${extension['author']}', style: TextStyle(fontSize: 11, color: Colors.grey[600])),
                Text(extension['lastUpdated'], style: TextStyle(fontSize: 11, color: Colors.grey[600])),
              ],
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton.icon(
                    onPressed: () => _showSnackBar('${extension['status'] == 'active' ? 'Deactivating' : 'Activating'} ${extension['name']}...'),
                    icon: Icon(extension['status'] == 'active' ? Icons.stop : Icons.play_arrow, size: 16),
                    label: Text(extension['status'] == 'active' ? 'Deactivate' : 'Activate'),
                    style: ElevatedButton.styleFrom(backgroundColor: extension['status'] == 'active' ? Colors.orange : Colors.green),
                  ),
                ),
                SizedBox(width: 8),
                Expanded(
                  child: ElevatedButton.icon(
                    onPressed: () => _showSnackBar('Configuring ${extension['name']}...'),
                    icon: Icon(Icons.settings, size: 16),
                    label: Text('Configure'),
                    style: ElevatedButton.styleFrom(backgroundColor: Colors.blue),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildServiceCard(String title, String status, String description, IconData icon, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          children: [
            Icon(icon, color: color, size: 32),
            SizedBox(height: 8),
            Text(title, style: TextStyle(fontWeight: FontWeight.bold), textAlign: TextAlign.center),
            SizedBox(height: 4),
            Container(
              padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(color: color.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
              child: Text(status, style: TextStyle(color: color, fontSize: 12)),
            ),
            SizedBox(height: 8),
            Text(description, style: TextStyle(fontSize: 11, color: Colors.grey[600]), textAlign: TextAlign.center),
          ],
        ),
      ),
    );
  }

  Widget _buildStoreExtension(String name, String description, String rating, Color color) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(name, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14), overflow: TextOverflow.ellipsis),
                ),
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(color: color.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(8)),
                  child: Text(rating, style: TextStyle(color: color, fontSize: 11, fontWeight: FontWeight.bold)),
                ),
              ],
            ),
            SizedBox(height: 4),
            Text(description, style: TextStyle(fontSize: 11, color: Colors.grey[600]), maxLines: 2, overflow: TextOverflow.ellipsis),
            SizedBox(height: 8),
            ElevatedButton(
              onPressed: () => _showSnackBar('Installing $name from marketplace...'),
              child: Text('Install', style: TextStyle(fontSize: 11)),
              style: ElevatedButton.styleFrom(backgroundColor: color, minimumSize: Size(double.infinity, 28)),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDeploymentItem(String name, String environment, String status, String time) {
    Color statusColor = status == 'success' ? Colors.green : status == 'running' ? Colors.blue : Colors.orange;
    Color envColor = environment == 'Production' ? Colors.red : environment == 'Staging' ? Colors.orange : Colors.blue;
    
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withValues(alpha: 0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(name, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14)),
                SizedBox(height: 4),
                Row(
                  children: [
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(color: envColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(8)),
                      child: Text(environment, style: TextStyle(color: envColor, fontSize: 11, fontWeight: FontWeight.bold)),
                    ),
                    SizedBox(width: 8),
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(color: statusColor.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(8)),
                      child: Text(status, style: TextStyle(color: statusColor, fontSize: 11)),
                    ),
                  ],
                ),
              ],
            ),
          ),
          Text(time, style: TextStyle(fontSize: 11, color: Colors.grey[600])),
        ],
      ),
    );
  }

  Widget _buildDeployOption(String title, String description) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
            SizedBox(height: 8),
            Text(description, style: TextStyle(color: Colors.grey[600], fontSize: 13)),
            SizedBox(height: 12),
            ElevatedButton(
              onPressed: () => _showSnackBar('Starting $title...'),
              child: Text('Deploy'),
              style: ElevatedButton.styleFrom(backgroundColor: Colors.deepOrange),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildExtensionItem(String name, String version, String description, bool enabled, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.withValues(alpha: 0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(name, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14)),
                SizedBox(height: 4),
                Text('v$version', style: TextStyle(fontSize: 11, fontFamily: 'monospace', color: Colors.grey[600])),
                SizedBox(height: 4),
                Text(description, style: TextStyle(fontSize: 12, color: Colors.grey[600])),
              ],
            ),
          ),
          Container(
            padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: enabled ? Colors.green.withValues(alpha: 0.2) : Colors.grey.withValues(alpha: 0.2),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(
              enabled ? 'Enabled' : 'Disabled',
              style: TextStyle(
                color: enabled ? Colors.green : Colors.grey,
                fontSize: 11,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPipelineStage(String name, String description, String status, Color color) {
    IconData icon = status == 'completed' ? Icons.check_circle : 
                   status == 'running' ? Icons.play_circle : Icons.pending;
    
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        border: Border.all(color: color.withValues(alpha: 0.3)),
        borderRadius: BorderRadius.circular(8),
        color: color.withValues(alpha: 0.05),
      ),
      child: Row(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(name, style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14)),
                SizedBox(height: 4),
                Text(description, style: TextStyle(fontSize: 12, color: Colors.grey[600])),
              ],
            ),
          ),
          Container(
            padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(color: color.withValues(alpha: 0.2), borderRadius: BorderRadius.circular(12)),
            child: Text(status, style: TextStyle(color: color, fontSize: 11, fontWeight: FontWeight.bold)),
          ),
        ],
      ),
    );
  }

  void _showExtensionStoreDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return AlertDialog(
          title: Row(
            children: [
              Icon(Icons.store, color: Colors.blue),
              SizedBox(width: 8),
              Text('Extension Store'),
            ],
          ),
          content: Container(
            width: 500,
            height: 400,
            child: Column(
              children: [
                TextField(
                  decoration: InputDecoration(
                    labelText: 'Search Extensions',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.search),
                  ),
                ),
                SizedBox(height: 16),
                Expanded(
                  child: GridView.count(
                    crossAxisCount: 2,
                    crossAxisSpacing: 8,
                    mainAxisSpacing: 8,
                    childAspectRatio: 1.2,
                    children: [
                      _buildStoreExtension('Advanced Analytics', 'Real-time user behavior analysis', '4.8★', Colors.blue),
                      _buildStoreExtension('Voice Chat Pro', 'High-quality spatial audio', '4.9★', Colors.green),
                      _buildStoreExtension('Custom Weather', 'Dynamic weather systems', '4.6★', Colors.cyan),
                      _buildStoreExtension('AI Companions', 'Intelligent NPCs and bots', '4.7★', Colors.purple),
                    ],
                  ),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('Close'),
            ),
          ],
        );
      },
    );
  }

  void _showInstallExtensionDialog() {
    showDialog(
      context: context,
      builder: (BuildContext context) {
        String extensionUrl = '';
        String installMethod = 'marketplace';
        
        return StatefulBuilder(
          builder: (context, setState) {
            return AlertDialog(
              title: Row(
                children: [
                  Icon(Icons.add_circle, color: Colors.green),
                  SizedBox(width: 8),
                  Text('Install Extension'),
                ],
              ),
              content: Container(
                width: 400,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    DropdownButtonFormField<String>(
                      value: installMethod,
                      decoration: InputDecoration(
                        labelText: 'Installation Method',
                        border: OutlineInputBorder(),
                      ),
                      items: [
                        DropdownMenuItem(value: 'marketplace', child: Text('OpenSim Marketplace')),
                        DropdownMenuItem(value: 'url', child: Text('Direct URL')),
                        DropdownMenuItem(value: 'file', child: Text('Local File')),
                      ],
                      onChanged: (String? newValue) {
                        setState(() {
                          installMethod = newValue!;
                        });
                      },
                    ),
                    if (installMethod == 'url') ...[
                      SizedBox(height: 16),
                      TextField(
                        decoration: InputDecoration(
                          labelText: 'Extension URL',
                          border: OutlineInputBorder(),
                          hintText: 'https://example.com/extension.zip',
                        ),
                        onChanged: (value) => extensionUrl = value,
                      ),
                    ],
                    if (installMethod == 'marketplace') ...[
                      SizedBox(height: 16),
                      Container(
                        padding: EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: Colors.blue.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Text('Browse available extensions in the OpenSim Marketplace', 
                                   style: TextStyle(color: Colors.blue)),
                      ),
                    ],
                    if (installMethod == 'file') ...[
                      SizedBox(height: 16),
                      Container(
                        padding: EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: Colors.orange.withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Text('Select a local extension file (.zip or .dll)', 
                                   style: TextStyle(color: Colors.orange)),
                      ),
                    ],
                  ],
                ),
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: Text('Cancel'),
                ),
                ElevatedButton(
                  onPressed: () {
                    Navigator.of(context).pop();
                    _showSnackBar('Installing extension via $installMethod...');
                  },
                  style: ElevatedButton.styleFrom(backgroundColor: Colors.green),
                  child: Text('Install'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}