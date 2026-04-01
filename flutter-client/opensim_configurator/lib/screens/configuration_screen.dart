// OpenSim Next Configurator - Configuration Screen
// Configuration editing and validation

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../theme/app_theme.dart';

class ConfigurationScreen extends StatefulWidget {
  @override
  _ConfigurationScreenState createState() => _ConfigurationScreenState();
}

class _ConfigurationScreenState extends State<ConfigurationScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Configuration'),
        actions: [
          IconButton(
            icon: Icon(Icons.save),
            onPressed: () => _saveConfiguration(context),
          ),
        ],
      ),
      body: Consumer<ConfiguratorProvider>(
        builder: (context, provider, child) {
          if (provider.currentConfig == null) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.settings, size: 64, color: AppTheme.gray400),
                  SizedBox(height: 16),
                  Text(
                    'No Configuration Selected',
                    style: AppTheme.titleStyle(),
                  ),
                  SizedBox(height: 8),
                  Text(
                    'Please select a deployment type first',
                    style: AppTheme.bodyStyle(),
                  ),
                  SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: () => Navigator.pushNamed(context, '/deployment'),
                    child: Text('Select Deployment Type'),
                  ),
                ],
              ),
            );
          }

          return SingleChildScrollView(
            padding: EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'OpenSim Configuration',
                  style: AppTheme.headlineStyle(),
                ),
                SizedBox(height: 8),
                Text(
                  'Configure your OpenSim Next deployment settings',
                  style: AppTheme.bodyStyle(),
                ),
                SizedBox(height: 24),
                
                // Coming Soon Message
                Card(
                  child: Padding(
                    padding: EdgeInsets.all(24),
                    child: Column(
                      children: [
                        Icon(
                          Icons.construction,
                          size: 64,
                          color: AppTheme.warningColor,
                        ),
                        SizedBox(height: 16),
                        Text(
                          'Configuration Editor Coming Soon',
                          style: AppTheme.titleStyle(),
                          textAlign: TextAlign.center,
                        ),
                        SizedBox(height: 8),
                        Text(
                          'The visual configuration editor is currently under development. For now, you can view your current configuration below.',
                          style: AppTheme.bodyStyle(),
                          textAlign: TextAlign.center,
                        ),
                      ],
                    ),
                  ),
                ),
                SizedBox(height: 24),

                // Current Configuration Summary
                if (provider.currentConfig != null) ...[
                  Text(
                    'Current Configuration',
                    style: AppTheme.titleStyle(),
                  ),
                  SizedBox(height: 12),
                  _buildConfigurationSummary(provider),
                ],
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildConfigurationSummary(ConfiguratorProvider provider) {
    final config = provider.currentConfig!;
    
    return Column(
      children: [
        _buildConfigCard(
          'General',
          [
            _buildConfigRow('Grid Name', config.gridName),
            _buildConfigRow('Grid Nick', config.gridNick),
            _buildConfigRow('Deployment Type', provider.getDeploymentTypeName(config.deploymentType)),
            _buildConfigRow('Welcome Message', config.welcomeMessage),
          ],
        ),
        _buildConfigCard(
          'Database',
          [
            _buildConfigRow('Type', config.databaseType),
            _buildConfigRow('Connection', config.databaseConnection, isPassword: true),
          ],
        ),
        _buildConfigCard(
          'Physics',
          [
            _buildConfigRow('Engine', config.physicsEngine),
          ],
        ),
        _buildConfigCard(
          'Network',
          [
            _buildConfigRow('HTTP Port', config.networkConfig.httpPort.toString()),
            _buildConfigRow('HTTPS Port', config.networkConfig.httpsPort.toString()),
            _buildConfigRow('HTTPS Enabled', config.networkConfig.httpsEnabled ? 'Yes' : 'No'),
            _buildConfigRow('External Hostname', config.networkConfig.externalHostname),
            _buildConfigRow('Internal IP', config.networkConfig.internalIp),
          ],
        ),
        _buildConfigCard(
          'Security',
          [
            _buildConfigRow('Password Complexity', config.securityConfig.passwordComplexity ? 'Enabled' : 'Disabled'),
            _buildConfigRow('Session Timeout', '${config.securityConfig.sessionTimeout}s'),
            _buildConfigRow('Brute Force Protection', config.securityConfig.bruteForceProtection ? 'Enabled' : 'Disabled'),
            _buildConfigRow('SSL Certificate', config.securityConfig.sslCertificatePath.isEmpty ? 'Not configured' : config.securityConfig.sslCertificatePath),
          ],
        ),
        _buildConfigCard(
          'Performance',
          [
            _buildConfigRow('Max Prims', config.performanceConfig.maxPrims.toString()),
            _buildConfigRow('Max Scripts', config.performanceConfig.maxScripts.toString()),
            _buildConfigRow('Script Timeout', '${config.performanceConfig.scriptTimeout}s'),
            _buildConfigRow('Cache Assets', config.performanceConfig.cacheAssets ? 'Enabled' : 'Disabled'),
            _buildConfigRow('Cache Timeout', '${config.performanceConfig.cacheTimeout}h'),
          ],
        ),
      ],
    );
  }

  Widget _buildConfigCard(String title, List<Widget> children) {
    return Card(
      margin: EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: AppTheme.titleStyle(),
            ),
            SizedBox(height: 12),
            ...children,
          ],
        ),
      ),
    );
  }

  Widget _buildConfigRow(String label, String value, {bool isPassword = false}) {
    return Padding(
      padding: EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: AppTheme.captionStyle(),
            ),
          ),
          Expanded(
            child: Text(
              isPassword && value.isNotEmpty ? '••••••••' : value,
              style: AppTheme.bodyStyle(),
            ),
          ),
        ],
      ),
    );
  }

  void _saveConfiguration(BuildContext context) {
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    
    // Show coming soon dialog
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Coming Soon'),
        content: Text('Configuration editing and saving features are currently under development.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }
}