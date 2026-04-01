// OpenSim Next Configurator - Home Screen
// Dashboard overview and quick actions

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../models/deployment_models.dart';
import '../theme/app_theme.dart';
import '../utils/flutter_error_classifier.dart';

class HomeScreen extends StatefulWidget {
  @override
  _HomeScreenState createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> with SafeStateMixin<HomeScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _refreshData();
    });
  }

  Future<void> _refreshData() async {
    await safeAsyncOperation(() async {
      final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
      await provider.testConnection();
      await provider.refreshServerStatus();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('OpenSim Next Configurator'),
        actions: [
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: _refreshData,
            tooltip: 'Refresh',
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: _refreshData,
        child: Consumer<ConfiguratorProvider>(
          builder: (context, provider, child) {
            return SingleChildScrollView(
              physics: AlwaysScrollableScrollPhysics(),
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Connection Status
                  _buildConnectionStatus(provider),
                  SizedBox(height: 24),

                  // Current Configuration Summary
                  if (provider.currentConfig != null) ...[
                    _buildConfigurationSummary(provider),
                    SizedBox(height: 24),
                  ],

                  // Server Metrics (if connected)
                  if (provider.isConnected && provider.serverStatus != null) ...[
                    _buildServerMetrics(provider),
                    SizedBox(height: 24),
                  ],

                  // Quick Actions
                  _buildQuickActions(provider),
                  SizedBox(height: 24),

                  // Validation Status (if available)
                  if (provider.validationResult != null) ...[
                    _buildValidationStatus(provider),
                    SizedBox(height: 24),
                  ],

                  // Recent Activity / Help
                  _buildHelpSection(),
                ],
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _buildConnectionStatus(ConfiguratorProvider provider) {
    final isConnected = provider.isConnected;
    final serverStatus = provider.serverStatus;

    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  isConnected ? Icons.cloud_done : Icons.cloud_off,
                  color: provider.getServerStatusColor(),
                  size: 28,
                ),
                SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isConnected ? 'Connected to Server' : 'Disconnected',
                        style: AppTheme.titleStyle(),
                      ),
                      Text(
                        provider.serverUrl,
                        style: AppTheme.bodyStyle(),
                      ),
                    ],
                  ),
                ),
                if (provider.isLoading)
                  SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
              ],
            ),
            if (isConnected && serverStatus != null) ...[
              SizedBox(height: 16),
              Divider(),
              SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: _buildStatusMetric(
                      'Uptime',
                      provider.formatUptime(serverStatus.uptimeSeconds),
                      Icons.schedule,
                    ),
                  ),
                  Expanded(
                    child: _buildStatusMetric(
                      'Regions',
                      '${serverStatus.activeRegions}',
                      Icons.map,
                    ),
                  ),
                  Expanded(
                    child: _buildStatusMetric(
                      'Users',
                      '${serverStatus.connectedUsers}',
                      Icons.people,
                    ),
                  ),
                ],
              ),
            ],
            if (provider.errorMessage != null) ...[
              SizedBox(height: 16),
              Container(
                padding: EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.errorColor.withOpacity(0.1),
                  border: Border.all(color: AppTheme.errorColor.withOpacity(0.3)),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    Icon(Icons.error_outline, color: AppTheme.errorColor),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        provider.errorMessage!,
                        style: TextStyle(color: AppTheme.errorColor),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildStatusMetric(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, color: AppTheme.primaryColor),
        SizedBox(height: 4),
        Text(
          value,
          style: AppTheme.titleStyle(),
        ),
        Text(
          label,
          style: AppTheme.captionStyle(),
        ),
      ],
    );
  }

  Widget _buildConfigurationSummary(ConfiguratorProvider provider) {
    final config = provider.currentConfig!;
    final deploymentInfo = DeploymentTypeInfo.getInfo(config.deploymentType);

    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: BoxDecoration(
                    color: provider.getDeploymentTypeColor(config.deploymentType).withOpacity(0.1),
                    border: Border.all(
                      color: provider.getDeploymentTypeColor(config.deploymentType).withOpacity(0.3),
                    ),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Text(
                    provider.getDeploymentTypeName(config.deploymentType),
                    style: TextStyle(
                      color: provider.getDeploymentTypeColor(config.deploymentType),
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                Spacer(),
                TextButton(
                  onPressed: () {
                    Navigator.pushNamed(context, '/configuration');
                  },
                  child: Text('Edit'),
                ),
              ],
            ),
            SizedBox(height: 12),
            Text(
              config.gridName,
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 8),
            Text(
              deploymentInfo?.description ?? '',
              style: AppTheme.bodyStyle(),
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildConfigMetric('Database', config.databaseType),
                ),
                Expanded(
                  child: _buildConfigMetric('Physics', config.physicsEngine),
                ),
                Expanded(
                  child: _buildConfigMetric(
                    'Security',
                    config.securityConfig.passwordComplexity ? 'Enhanced' : 'Basic',
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConfigMetric(String label, String value) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: AppTheme.captionStyle(),
        ),
        SizedBox(height: 2),
        Text(
          value,
          style: AppTheme.bodyStyle().copyWith(fontWeight: FontWeight.w500),
        ),
      ],
    );
  }

  Widget _buildServerMetrics(ConfiguratorProvider provider) {
    final status = provider.serverStatus!;

    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Server Performance',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildPerformanceMetric(
                    'CPU Usage',
                    '${status.cpuUsage.toStringAsFixed(1)}%',
                    status.cpuUsage / 100,
                    status.cpuUsage > 80 ? AppTheme.errorColor : AppTheme.successColor,
                  ),
                ),
                SizedBox(width: 16),
                Expanded(
                  child: _buildPerformanceMetric(
                    'Memory Usage',
                    '${status.memoryUsage.toStringAsFixed(1)}%',
                    status.memoryUsage / 100,
                    status.memoryUsage > 85 ? AppTheme.errorColor : AppTheme.successColor,
                  ),
                ),
              ],
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildNetworkMetric(
                    'Sent',
                    provider.formatBytes(status.networkActivity.bytesSent),
                    Icons.upload,
                  ),
                ),
                Expanded(
                  child: _buildNetworkMetric(
                    'Received',
                    provider.formatBytes(status.networkActivity.bytesReceived),
                    Icons.download,
                  ),
                ),
                Expanded(
                  child: _buildNetworkMetric(
                    'Connections',
                    '${status.networkActivity.connections}',
                    Icons.link,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPerformanceMetric(String label, String value, double progress, Color color) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: AppTheme.bodyStyle()),
            Text(value, style: AppTheme.bodyStyle().copyWith(fontWeight: FontWeight.w600)),
          ],
        ),
        SizedBox(height: 8),
        LinearProgressIndicator(
          value: progress.clamp(0.0, 1.0),
          backgroundColor: AppTheme.gray200,
          color: color,
        ),
      ],
    );
  }

  Widget _buildNetworkMetric(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, color: AppTheme.primaryColor),
        SizedBox(height: 4),
        Text(
          value,
          style: AppTheme.bodyStyle().copyWith(fontWeight: FontWeight.w600),
        ),
        Text(
          label,
          style: AppTheme.captionStyle(),
        ),
      ],
    );
  }

  Widget _buildQuickActions(ConfiguratorProvider provider) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Quick Actions',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            GridView.count(
              shrinkWrap: true,
              physics: NeverScrollableScrollPhysics(),
              crossAxisCount: 2,
              mainAxisSpacing: 12,
              crossAxisSpacing: 12,
              childAspectRatio: 1.5,
              children: [
                _buildActionButton(
                  'New Deployment',
                  Icons.add_circle_outline,
                  AppTheme.primaryColor,
                  () => Navigator.pushNamed(context, '/deployment'),
                ),
                _buildActionButton(
                  'Monitoring',
                  Icons.monitor,
                  AppTheme.infoColor,
                  () => Navigator.pushNamed(context, '/monitoring'),
                ),
                _buildActionButton(
                  'Configuration',
                  Icons.settings,
                  AppTheme.secondaryColor,
                  () => Navigator.pushNamed(context, '/configuration'),
                ),
                _buildActionButton(
                  'Settings',
                  Icons.tune,
                  AppTheme.warningColor,
                  () => Navigator.pushNamed(context, '/settings'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActionButton(String label, IconData icon, Color color, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: color.withOpacity(0.3)),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(icon, color: color, size: 32),
            SizedBox(height: 8),
            Text(
              label,
              style: AppTheme.bodyStyle().copyWith(
                fontWeight: FontWeight.w500,
                color: color,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildValidationStatus(ConfiguratorProvider provider) {
    final result = provider.validationResult!;

    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  result.isValid ? Icons.check_circle : Icons.error,
                  color: result.isValid ? AppTheme.successColor : AppTheme.errorColor,
                ),
                SizedBox(width: 8),
                Text(
                  'Configuration Validation',
                  style: AppTheme.titleStyle(),
                ),
                Spacer(),
                Container(
                  padding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: provider.getValidationScoreColor(result.overallScore).withOpacity(0.1),
                    border: Border.all(
                      color: provider.getValidationScoreColor(result.overallScore).withOpacity(0.3),
                    ),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    'Score: ${result.overallScore}%',
                    style: TextStyle(
                      color: provider.getValidationScoreColor(result.overallScore),
                      fontWeight: FontWeight.w600,
                      fontSize: 12,
                    ),
                  ),
                ),
              ],
            ),
            if (result.errors.isNotEmpty) ...[
              SizedBox(height: 12),
              ...result.errors.map((error) => Padding(
                padding: EdgeInsets.only(bottom: 4),
                child: Row(
                  children: [
                    Icon(Icons.error, color: AppTheme.errorColor, size: 16),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        error,
                        style: TextStyle(color: AppTheme.errorColor),
                      ),
                    ),
                  ],
                ),
              )),
            ],
            if (result.warnings.isNotEmpty) ...[
              SizedBox(height: 8),
              ...result.warnings.map((warning) => Padding(
                padding: EdgeInsets.only(bottom: 4),
                child: Row(
                  children: [
                    Icon(Icons.warning, color: AppTheme.warningColor, size: 16),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        warning,
                        style: TextStyle(color: AppTheme.warningColor),
                      ),
                    ),
                  ],
                ),
              )),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildHelpSection() {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Getting Started',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 12),
            ListTile(
              leading: Icon(Icons.play_circle_outline, color: AppTheme.primaryColor),
              title: Text('1. Choose Deployment Type'),
              subtitle: Text('Select the environment that matches your needs'),
              contentPadding: EdgeInsets.zero,
            ),
            ListTile(
              leading: Icon(Icons.build_circle_outlined, color: AppTheme.primaryColor),
              title: Text('2. Configure Settings'),
              subtitle: Text('Customize your OpenSim configuration'),
              contentPadding: EdgeInsets.zero,
            ),
            ListTile(
              leading: Icon(Icons.check_circle_outline, color: AppTheme.primaryColor),
              title: Text('3. Apply & Monitor'),
              subtitle: Text('Deploy and monitor your virtual world'),
              contentPadding: EdgeInsets.zero,
            ),
          ],
        ),
      ),
    );
  }
}