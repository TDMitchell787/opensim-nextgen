// OpenSim Next Configurator - Monitoring Screen
// Real-time server monitoring and metrics

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../theme/app_theme.dart';
import '../utils/flutter_error_classifier.dart';

class MonitoringScreen extends StatefulWidget {
  @override
  _MonitoringScreenState createState() => _MonitoringScreenState();
}

class _MonitoringScreenState extends State<MonitoringScreen> with SafeStateMixin<MonitoringScreen> {
  @override
  void initState() {
    super.initState();
    _startRefreshTimer();
  }

  void _startRefreshTimer() {
    // Refresh every 5 seconds when screen is active - using safe async operation
    safeAsyncOperation(() async {
      await Future.delayed(Duration(seconds: 5));
      if (mounted) {
        Provider.of<ConfiguratorProvider>(context, listen: false).refreshServerStatus();
        _startRefreshTimer();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Monitoring'),
        actions: [
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: () => Provider.of<ConfiguratorProvider>(context, listen: false).refreshServerStatus(),
          ),
        ],
      ),
      body: Consumer<ConfiguratorProvider>(
        builder: (context, provider, child) {
          if (!provider.isConnected) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.cloud_off, size: 64, color: AppTheme.gray400),
                  SizedBox(height: 16),
                  Text(
                    'Server Not Connected',
                    style: AppTheme.titleStyle(),
                  ),
                  SizedBox(height: 8),
                  Text(
                    'Please check your connection settings',
                    style: AppTheme.bodyStyle(),
                  ),
                  SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: () => Provider.of<ConfiguratorProvider>(context, listen: false).testConnection(),
                    child: Text('Test Connection'),
                  ),
                ],
              ),
            );
          }

          final status = provider.serverStatus;
          if (status == null) {
            return Center(child: CircularProgressIndicator());
          }

          return RefreshIndicator(
            onRefresh: () => provider.refreshServerStatus(),
            child: SingleChildScrollView(
              physics: AlwaysScrollableScrollPhysics(),
              padding: EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Server Status Overview
                  _buildStatusOverview(provider, status),
                  SizedBox(height: 24),

                  // Performance Metrics
                  _buildPerformanceMetrics(provider, status),
                  SizedBox(height: 24),

                  // Network Activity
                  _buildNetworkActivity(provider, status),
                  SizedBox(height: 24),

                  // Region Information
                  _buildRegionInfo(status),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildStatusOverview(ConfiguratorProvider provider, status) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(
                    color: provider.getServerStatusColor(),
                    shape: BoxShape.circle,
                  ),
                ),
                SizedBox(width: 8),
                Text(
                  status.isRunning ? 'Server Online' : 'Server Offline',
                  style: AppTheme.titleStyle(),
                ),
                Spacer(),
                Text(
                  'Last updated: ${DateTime.now().toString().substring(11, 19)}',
                  style: AppTheme.captionStyle(),
                ),
              ],
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildOverviewMetric(
                    'Uptime',
                    provider.formatUptime(status.uptimeSeconds),
                    Icons.schedule,
                  ),
                ),
                Expanded(
                  child: _buildOverviewMetric(
                    'Active Regions',
                    '${status.activeRegions}',
                    Icons.map,
                  ),
                ),
                Expanded(
                  child: _buildOverviewMetric(
                    'Connected Users',
                    '${status.connectedUsers}',
                    Icons.people,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildOverviewMetric(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, color: AppTheme.primaryColor, size: 24),
        SizedBox(height: 8),
        Text(
          value,
          style: AppTheme.titleStyle(),
        ),
        Text(
          label,
          style: AppTheme.captionStyle(),
          textAlign: TextAlign.center,
        ),
      ],
    );
  }

  Widget _buildPerformanceMetrics(ConfiguratorProvider provider, status) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Performance Metrics',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            _buildPerformanceBar(
              'CPU Usage',
              status.cpuUsage,
              '${status.cpuUsage.toStringAsFixed(1)}%',
              status.cpuUsage > 80 ? AppTheme.errorColor : 
              status.cpuUsage > 60 ? AppTheme.warningColor : AppTheme.successColor,
            ),
            SizedBox(height: 16),
            _buildPerformanceBar(
              'Memory Usage',
              status.memoryUsage,
              '${status.memoryUsage.toStringAsFixed(1)}%',
              status.memoryUsage > 85 ? AppTheme.errorColor : 
              status.memoryUsage > 70 ? AppTheme.warningColor : AppTheme.successColor,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPerformanceBar(String label, double value, String displayValue, Color color) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              label,
              style: AppTheme.bodyStyle(),
            ),
            Text(
              displayValue,
              style: AppTheme.bodyStyle().copyWith(
                fontWeight: FontWeight.w600,
                color: color,
              ),
            ),
          ],
        ),
        SizedBox(height: 8),
        LinearProgressIndicator(
          value: (value / 100).clamp(0.0, 1.0),
          backgroundColor: AppTheme.gray200,
          color: color,
        ),
      ],
    );
  }

  Widget _buildNetworkActivity(ConfiguratorProvider provider, status) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Network Activity',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildNetworkMetric(
                    'Data Sent',
                    provider.formatBytes(status.networkActivity.bytesSent),
                    Icons.upload,
                    AppTheme.primaryColor,
                  ),
                ),
                Expanded(
                  child: _buildNetworkMetric(
                    'Data Received',
                    provider.formatBytes(status.networkActivity.bytesReceived),
                    Icons.download,
                    AppTheme.successColor,
                  ),
                ),
                Expanded(
                  child: _buildNetworkMetric(
                    'Connections',
                    '${status.networkActivity.connections}',
                    Icons.link,
                    AppTheme.infoColor,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNetworkMetric(String label, String value, IconData icon, Color color) {
    return Container(
      padding: EdgeInsets.all(12),
      margin: EdgeInsets.symmetric(horizontal: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        border: Border.all(color: color.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        children: [
          Icon(icon, color: color, size: 24),
          SizedBox(height: 8),
          Text(
            value,
            style: AppTheme.bodyStyle().copyWith(
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
          Text(
            label,
            style: AppTheme.captionStyle(),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildRegionInfo(status) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Region Information',
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 16),
            if (status.activeRegions == 0) ...[
              Center(
                child: Column(
                  children: [
                    Icon(Icons.map, size: 48, color: AppTheme.gray400),
                    SizedBox(height: 8),
                    Text(
                      'No Active Regions',
                      style: AppTheme.bodyStyle(),
                    ),
                  ],
                ),
              ),
            ] else ...[
              Text(
                'Active Regions: ${status.activeRegions}',
                style: AppTheme.bodyStyle(),
              ),
              SizedBox(height: 8),
              Text(
                'Connected Users: ${status.connectedUsers}',
                style: AppTheme.bodyStyle(),
              ),
              SizedBox(height: 16),
              Container(
                padding: EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: AppTheme.infoColor.withOpacity(0.1),
                  border: Border.all(color: AppTheme.infoColor.withOpacity(0.3)),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Row(
                  children: [
                    Icon(Icons.info_outline, color: AppTheme.infoColor),
                    SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        'Detailed region management features are coming soon.',
                        style: TextStyle(color: AppTheme.infoColor),
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
}