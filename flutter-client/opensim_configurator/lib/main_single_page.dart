import 'package:flutter/material.dart';

void main() {
  runApp(SinglePageApp());
}

class SinglePageApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Single Page Test',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: SinglePageManager(),
    );
  }
}

class SinglePageManager extends StatefulWidget {
  @override
  _SinglePageManagerState createState() => _SinglePageManagerState();
}

class _SinglePageManagerState extends State<SinglePageManager> {
  int _currentPage = 1;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Single Page Manager - Page $_currentPage'),
      ),
      body: Column(
        children: [
          // Navigation bar
          Container(
            height: 60,
            padding: EdgeInsets.all(8),
            color: Colors.grey[100],
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: List.generate(10, (index) {
                  int pageNum = index + 1;
                  return Padding(
                    padding: EdgeInsets.symmetric(horizontal: 4),
                    child: ElevatedButton(
                      onPressed: () {
                        setState(() {
                          _currentPage = pageNum;
                        });
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: _currentPage == pageNum ? Colors.blue : Colors.grey,
                      ),
                      child: Text('Page $pageNum'),
                    ),
                  );
                }),
              ),
            ),
          ),
          
          Divider(),
          
          // Single content area with conditional rendering
          Expanded(
            child: _buildPageContent(_currentPage),
          ),
        ],
      ),
    );
  }

  Widget _buildPageContent(int pageNumber) {
    switch (pageNumber) {
      case 1:
        return _buildPage('Graphics Splash', Colors.purple, Icons.image, 'Animated welcome with feature highlights');
      case 2:
        return _buildPage('Contributors', Colors.green, Icons.people, 'Team showcase with technology stack');
      case 3:
        return _buildPage('Welcome', Colors.blue, Icons.home, 'System overview with navigation');
      case 4:
        return _buildPage('Web Admin', Colors.orange, Icons.admin_panel_settings, 'Server table and configurator');
      case 5:
        return _buildPage('Analytics', Colors.red, Icons.analytics, 'Real-time metrics and charts');
      case 6:
        return _buildPage('Observability', Colors.teal, Icons.visibility, 'System monitoring and tracing');
      case 7:
        return _buildPage('Health', Colors.pink, Icons.monitor_heart, 'Live health metrics');
      case 8:
        return _buildPage('Security', Colors.indigo, Icons.security, 'Zero trust security status');
      case 9:
        return _buildPage('Database', Colors.brown, Icons.storage, 'Database management interface');
      case 10:
        return _buildPage('Settings', Colors.amber, Icons.settings, 'Configuration options');
      default:
        return _buildPage('Unknown', Colors.grey, Icons.error, 'Unknown page');
    }
  }

  Widget _buildPage(String title, Color color, IconData icon, String description) {
    return Container(
      color: color.withOpacity(0.1),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, size: 80, color: color),
            SizedBox(height: 24),
            Text(
              title,
              style: Theme.of(context).textTheme.headlineLarge?.copyWith(
                color: color,
                fontWeight: FontWeight.bold,
              ),
            ),
            SizedBox(height: 16),
            Container(
              padding: EdgeInsets.all(16),
              margin: EdgeInsets.symmetric(horizontal: 32),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(12),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withOpacity(0.1),
                    blurRadius: 8,
                    spreadRadius: 2,
                  ),
                ],
              ),
              child: Column(
                children: [
                  Text(
                    description,
                    style: Theme.of(context).textTheme.bodyLarge,
                    textAlign: TextAlign.center,
                  ),
                  SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      _buildStatCard('Users', '42', color),
                      _buildStatCard('Regions', '8', color),
                      _buildStatCard('Uptime', '120h', color),
                    ],
                  ),
                  SizedBox(height: 16),
                  ElevatedButton.icon(
                    onPressed: () {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('$title functionality working!')),
                      );
                    },
                    icon: Icon(icon),
                    label: Text('Test $title'),
                    style: ElevatedButton.styleFrom(backgroundColor: color),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatCard(String label, String value, Color color) {
    return Column(
      children: [
        Text(
          value,
          style: TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
        Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: Colors.grey[600],
          ),
        ),
      ],
    );
  }
}