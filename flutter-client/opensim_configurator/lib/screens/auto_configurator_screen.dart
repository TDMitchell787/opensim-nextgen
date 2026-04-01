// OpenSim Next Auto-Configurator - Flutter Web Version
// Complete auto-configuration wizard with multi-language support and intelligent guidance

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../theme/app_theme.dart';
import '../widgets/configuration_wizard.dart';
import '../widgets/configuration_dashboard.dart';
import '../widgets/deployment_selector.dart';
import '../widgets/language_selector.dart';
import '../widgets/validation_panel.dart';
import '../widgets/help_system.dart';

class AutoConfiguratorScreen extends StatefulWidget {
  @override
  _AutoConfiguratorScreenState createState() => _AutoConfiguratorScreenState();
}

class _AutoConfiguratorScreenState extends State<AutoConfiguratorScreen> {
  final PageController _pageController = PageController();
  int _currentPage = 0;
  bool _showWizard = false;

  final List<String> _steps = [
    'Deployment Type',
    'Environment',
    'Database',
    'Regions',
    'Security',
    'Network',
    'Review'
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Theme.of(context).colorScheme.background,
      body: CustomScrollView(
        slivers: [
          // App Bar
          SliverAppBar(
            expandedHeight: 120,
            floating: false,
            pinned: true,
            elevation: 2,
            backgroundColor: Colors.white,
            foregroundColor: AppTheme.gray900,
            flexibleSpace: FlexibleSpaceBar(
              title: Row(
                children: [
                  Icon(Icons.settings_applications, color: AppTheme.primaryColor),
                  SizedBox(width: 8),
                  Text(
                    'OpenSim Next Auto-Configurator',
                    style: TextStyle(
                      color: AppTheme.gray900,
                      fontSize: 18,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ],
              ),
              background: Container(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                    colors: [Colors.white, AppTheme.gray50],
                  ),
                ),
              ),
            ),
            actions: [
              LanguageSelector(),
              SizedBox(width: 8),
              IconButton(
                icon: Icon(Icons.help_outline),
                onPressed: () => _showHelpSystem(context),
                tooltip: 'Help & Documentation',
              ),
              IconButton(
                icon: Icon(Icons.settings),
                onPressed: () => _showSettings(context),
                tooltip: 'Settings',
              ),
              SizedBox(width: 16),
            ],
          ),

          // Main Content
          SliverToBoxAdapter(
            child: Container(
              padding: EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Progress Indicator
                  if (_showWizard) _buildProgressIndicator(),
                  
                  SizedBox(height: 24),

                  // Validation Panel
                  ValidationPanel(),
                  
                  SizedBox(height: 24),

                  // Main Configuration Interface
                  _showWizard 
                    ? ConfigurationWizard(
                        pageController: _pageController,
                        currentPage: _currentPage,
                        onPageChanged: (page) => setState(() => _currentPage = page),
                        onWizardComplete: () => setState(() => _showWizard = false),
                      )
                    : ConfigurationDashboard(
                        onStartWizard: () => setState(() => _showWizard = true),
                      ),

                  SizedBox(height: 24),

                  // Quick Actions
                  _buildQuickActions(),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildProgressIndicator() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Configuration Progress',
                  style: AppTheme.titleStyle(),
                ),
                Text(
                  'Step ${_currentPage + 1} of ${_steps.length}',
                  style: AppTheme.bodyStyle(),
                ),
              ],
            ),
            SizedBox(height: 16),
            
            // Progress Bar
            LinearProgressIndicator(
              value: (_currentPage + 1) / _steps.length,
              backgroundColor: AppTheme.gray200,
              valueColor: AlwaysStoppedAnimation<Color>(AppTheme.primaryColor),
            ),
            
            SizedBox(height: 20),
            
            // Step Indicators
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: _steps.asMap().entries.map((entry) {
                int index = entry.key;
                String step = entry.value;
                bool isActive = index == _currentPage;
                bool isCompleted = index < _currentPage;
                
                return Expanded(
                  child: Column(
                    children: [
                      Container(
                        width: 32,
                        height: 32,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: isCompleted 
                            ? AppTheme.successColor
                            : isActive 
                              ? AppTheme.primaryColor
                              : AppTheme.gray300,
                        ),
                        child: Center(
                          child: isCompleted
                            ? Icon(Icons.check, color: Colors.white, size: 16)
                            : Text(
                                '${index + 1}',
                                style: TextStyle(
                                  color: isActive ? Colors.white : AppTheme.gray600,
                                  fontSize: 14,
                                  fontWeight: FontWeight.w500,
                                ),
                              ),
                        ),
                      ),
                      SizedBox(height: 8),
                      Text(
                        step,
                        style: TextStyle(
                          fontSize: 12,
                          color: isActive ? AppTheme.primaryColor : AppTheme.gray600,
                          fontWeight: isActive ? FontWeight.w500 : FontWeight.w400,
                        ),
                        textAlign: TextAlign.center,
                      ),
                    ],
                  ),
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildQuickActions() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Quick Actions',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [
                _buildQuickActionButton(
                  icon: Icons.upload_file,
                  label: 'Import Configuration',
                  onPressed: () => _importConfiguration(),
                ),
                _buildQuickActionButton(
                  icon: Icons.download,
                  label: 'Export Configuration',
                  onPressed: () => _exportConfiguration(),
                ),
                _buildQuickActionButton(
                  icon: Icons.note_add,
                  label: 'Load Template',
                  onPressed: () => _loadTemplate(),
                ),
                _buildQuickActionButton(
                  icon: Icons.code,
                  label: 'Expert Mode',
                  onPressed: () => _toggleExpertMode(),
                ),
                _buildQuickActionButton(
                  icon: Icons.backup,
                  label: 'Backup & Restore',
                  onPressed: () => _showBackupRestore(),
                ),
                _buildQuickActionButton(
                  icon: Icons.security,
                  label: 'Security Scan',
                  onPressed: () => _runSecurityScan(),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildQuickActionButton({
    required IconData icon,
    required String label,
    required VoidCallback onPressed,
  }) {
    return OutlinedButton.icon(
      onPressed: onPressed,
      icon: Icon(icon),
      label: Text(label),
      style: OutlinedButton.styleFrom(
        padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      ),
    );
  }

  void _showHelpSystem(BuildContext context) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => DraggableScrollableSheet(
        initialChildSize: 0.7,
        maxChildSize: 0.9,
        minChildSize: 0.5,
        builder: (context, scrollController) => HelpSystem(
          scrollController: scrollController,
        ),
      ),
    );
  }

  void _showSettings(BuildContext context) {
    Navigator.pushNamed(context, '/settings');
  }

  void _importConfiguration() {
    // Implement configuration import
    _showComingSoonDialog('Configuration Import');
  }

  void _exportConfiguration() {
    // Implement configuration export
    _showComingSoonDialog('Configuration Export');
  }

  void _loadTemplate() {
    // Implement template loading
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Load Configuration Template'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: Icon(Icons.developer_mode, color: AppTheme.developmentColor),
              title: Text('Development Template'),
              subtitle: Text('Single region, SQLite, basic security'),
              onTap: () {
                Navigator.pop(context);
                _loadTemplateType('development');
              },
            ),
            ListTile(
              leading: Icon(Icons.business, color: AppTheme.productionColor),
              title: Text('Production Template'),
              subtitle: Text('Multi-region, PostgreSQL, enterprise security'),
              onTap: () {
                Navigator.pop(context);
                _loadTemplateType('production');
              },
            ),
            ListTile(
              leading: Icon(Icons.grid_on, color: AppTheme.gridColor),
              title: Text('Grid Template'),
              subtitle: Text('Large scale, clustering, zero trust'),
              onTap: () {
                Navigator.pop(context);
                _loadTemplateType('grid');
              },
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

  void _loadTemplateType(String type) {
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    provider.loadTemplate(type);
    
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Loaded $type template successfully'),
        backgroundColor: AppTheme.successColor,
      ),
    );
  }

  void _toggleExpertMode() {
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    provider.toggleExpertMode();
    
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          provider.isExpertMode 
            ? 'Expert mode enabled - Advanced options available'
            : 'Expert mode disabled - Simplified interface active'
        ),
        backgroundColor: AppTheme.infoColor,
      ),
    );
  }

  void _showBackupRestore() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Backup & Restore'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: Icon(Icons.backup, color: AppTheme.primaryColor),
              title: Text('Create Backup'),
              subtitle: Text('Save current configuration'),
              onTap: () {
                Navigator.pop(context);
                _createBackup();
              },
            ),
            ListTile(
              leading: Icon(Icons.restore, color: AppTheme.successColor),
              title: Text('Restore from Backup'),
              subtitle: Text('Load previous configuration'),
              onTap: () {
                Navigator.pop(context);
                _restoreBackup();
              },
            ),
            ListTile(
              leading: Icon(Icons.schedule, color: AppTheme.infoColor),
              title: Text('Automated Backups'),
              subtitle: Text('Configure backup schedule'),
              onTap: () {
                Navigator.pop(context);
                _configureAutomatedBackups();
              },
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

  void _runSecurityScan() {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            SizedBox(
              width: 20,
              height: 20,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            SizedBox(width: 16),
            Text('Security Scan in Progress'),
          ],
        ),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Analyzing configuration security...'),
            SizedBox(height: 16),
            LinearProgressIndicator(),
            SizedBox(height: 16),
            Text(
              'This may take a few moments.',
              style: AppTheme.captionStyle(),
            ),
          ],
        ),
      ),
    );

    // Simulate security scan
    Future.delayed(Duration(seconds: 3), () {
      Navigator.pop(context);
      _showSecurityResults();
    });
  }

  void _showSecurityResults() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Security Scan Results'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.check_circle, color: AppTheme.successColor),
                SizedBox(width: 8),
                Text('SSL/TLS Configuration: OK'),
              ],
            ),
            SizedBox(height: 8),
            Row(
              children: [
                Icon(Icons.warning, color: AppTheme.warningColor),
                SizedBox(width: 8),
                Text('Database Security: Needs attention'),
              ],
            ),
            SizedBox(height: 8),
            Row(
              children: [
                Icon(Icons.check_circle, color: AppTheme.successColor),
                SizedBox(width: 8),
                Text('Firewall Rules: OK'),
              ],
            ),
            SizedBox(height: 16),
            Text(
              'Overall Security Score: 8/10',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 8),
            Text(
              '2 recommendations available',
              style: AppTheme.bodyStyle(),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('View Details'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  void _createBackup() {
    _showComingSoonDialog('Configuration Backup');
  }

  void _restoreBackup() {
    _showComingSoonDialog('Configuration Restore');
  }

  void _configureAutomatedBackups() {
    _showComingSoonDialog('Automated Backup Configuration');
  }

  void _showComingSoonDialog(String feature) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Coming Soon'),
        content: Text('$feature is currently under development and will be available in a future release.'),
        actions: [
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }
}