// OpenSim Next Configurator - Settings Screen
// App settings and server connection configuration

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../theme/app_theme.dart';

class SettingsScreen extends StatefulWidget {
  @override
  _SettingsScreenState createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  final _formKey = GlobalKey<FormState>();
  late TextEditingController _serverUrlController;
  late TextEditingController _apiKeyController;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    _serverUrlController = TextEditingController(text: provider.serverUrl);
    _apiKeyController = TextEditingController(text: provider.apiKey);
  }

  @override
  void dispose() {
    _serverUrlController.dispose();
    _apiKeyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Settings'),
      ),
      body: Consumer<ConfiguratorProvider>(
        builder: (context, provider, child) {
          return SingleChildScrollView(
            padding: EdgeInsets.all(16),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Connection Settings
                  _buildSectionHeader('Server Connection'),
                  SizedBox(height: 16),
                  Card(
                    child: Padding(
                      padding: EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          TextFormField(
                            controller: _serverUrlController,
                            decoration: InputDecoration(
                              labelText: 'Server URL',
                              hintText: 'http://localhost:9000',
                              prefixIcon: Icon(Icons.computer),
                              helperText: 'URL of your OpenSim Next server',
                            ),
                            validator: (value) {
                              if (value == null || value.isEmpty) {
                                return 'Server URL is required';
                              }
                              if (Uri.tryParse(value)?.hasAbsolutePath != true) {
                                return 'Invalid URL format';
                              }
                              return null;
                            },
                          ),
                          SizedBox(height: 16),
                          TextFormField(
                            controller: _apiKeyController,
                            decoration: InputDecoration(
                              labelText: 'API Key',
                              hintText: 'default-key-change-me',
                              prefixIcon: Icon(Icons.key),
                              helperText: 'API key for server authentication',
                            ),
                            obscureText: true,
                            validator: (value) {
                              if (value == null || value.isEmpty) {
                                return 'API key is required';
                              }
                              return null;
                            },
                          ),
                          SizedBox(height: 16),
                          Row(
                            children: [
                              Expanded(
                                child: OutlinedButton(
                                  onPressed: _isLoading ? null : _testConnection,
                                  child: _isLoading
                                      ? Row(
                                          mainAxisAlignment: MainAxisAlignment.center,
                                          mainAxisSize: MainAxisSize.min,
                                          children: [
                                            SizedBox(
                                              width: 16,
                                              height: 16,
                                              child: CircularProgressIndicator(strokeWidth: 2),
                                            ),
                                            SizedBox(width: 8),
                                            Text('Testing...'),
                                          ],
                                        )
                                      : Text('Test Connection'),
                                ),
                              ),
                              SizedBox(width: 12),
                              Expanded(
                                child: ElevatedButton(
                                  onPressed: _isLoading ? null : _saveConnection,
                                  child: Text('Save'),
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                  SizedBox(height: 24),

                  // Connection Status
                  _buildSectionHeader('Connection Status'),
                  SizedBox(height: 16),
                  Card(
                    child: Padding(
                      padding: EdgeInsets.all(16),
                      child: Row(
                        children: [
                          Container(
                            width: 12,
                            height: 12,
                            decoration: BoxDecoration(
                              color: provider.getServerStatusColor(),
                              shape: BoxShape.circle,
                            ),
                          ),
                          SizedBox(width: 12),
                          Expanded(
                            child: Text(
                              provider.isConnected ? 'Connected' : 'Disconnected',
                              style: AppTheme.bodyStyle(),
                            ),
                          ),
                          if (provider.serverStatus != null)
                            Text(
                              'v${provider.serverStatus?.uptimeSeconds != null ? "1.0.0" : "Unknown"}',
                              style: AppTheme.captionStyle(),
                            ),
                        ],
                      ),
                    ),
                  ),
                  SizedBox(height: 24),

                  // App Information
                  _buildSectionHeader('App Information'),
                  SizedBox(height: 16),
                  Card(
                    child: Column(
                      children: [
                        ListTile(
                          leading: Icon(Icons.info_outline),
                          title: Text('Version'),
                          subtitle: Text('1.0.0'),
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.build),
                          title: Text('Build'),
                          subtitle: Text('Development'),
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.flutter_dash),
                          title: Text('Framework'),
                          subtitle: Text('Flutter + Rust FFI'),
                        ),
                      ],
                    ),
                  ),
                  SizedBox(height: 24),

                  // Actions
                  _buildSectionHeader('Actions'),
                  SizedBox(height: 16),
                  Card(
                    child: Column(
                      children: [
                        ListTile(
                          leading: Icon(Icons.refresh, color: AppTheme.primaryColor),
                          title: Text('Reset Configuration'),
                          subtitle: Text('Clear all configuration data'),
                          onTap: _showResetDialog,
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.file_download, color: AppTheme.infoColor),
                          title: Text('Export Configuration'),
                          subtitle: Text('Save configuration to file'),
                          onTap: provider.currentConfig != null ? _exportConfiguration : null,
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.file_upload, color: AppTheme.successColor),
                          title: Text('Import Configuration'),
                          subtitle: Text('Load configuration from file'),
                          onTap: _importConfiguration,
                        ),
                      ],
                    ),
                  ),
                  SizedBox(height: 24),

                  // Support
                  _buildSectionHeader('Support'),
                  SizedBox(height: 16),
                  Card(
                    child: Column(
                      children: [
                        ListTile(
                          leading: Icon(Icons.help_outline, color: AppTheme.infoColor),
                          title: Text('Documentation'),
                          subtitle: Text('View user guide and documentation'),
                          onTap: _openDocumentation,
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.bug_report, color: AppTheme.warningColor),
                          title: Text('Report Issue'),
                          subtitle: Text('Report bugs or request features'),
                          onTap: _reportIssue,
                        ),
                        Divider(height: 1),
                        ListTile(
                          leading: Icon(Icons.code, color: AppTheme.primaryColor),
                          title: Text('Source Code'),
                          subtitle: Text('View on GitHub'),
                          onTap: _openSourceCode,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Text(
      title,
      style: AppTheme.titleStyle().copyWith(
        color: AppTheme.primaryColor,
        fontSize: 18,
      ),
    );
  }

  Future<void> _testConnection() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _isLoading = true;
    });

    try {
      final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
      await provider.updateConnectionSettings(
        _serverUrlController.text,
        _apiKeyController.text,
      );

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            provider.isConnected 
                ? 'Connection successful!' 
                : 'Connection failed: ${provider.errorMessage ?? "Unknown error"}',
          ),
          backgroundColor: provider.isConnected ? AppTheme.successColor : AppTheme.errorColor,
        ),
      );
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  Future<void> _saveConnection() async {
    if (!_formKey.currentState!.validate()) return;

    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    await provider.updateConnectionSettings(
      _serverUrlController.text,
      _apiKeyController.text,
    );

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Connection settings saved'),
        backgroundColor: AppTheme.successColor,
      ),
    );
  }

  void _showResetDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Reset Configuration'),
        content: Text('This will clear all configuration data. Are you sure?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Provider.of<ConfiguratorProvider>(context, listen: false).resetConfiguration();
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('Configuration reset'),
                  backgroundColor: AppTheme.successColor,
                ),
              );
            },
            child: Text('Reset'),
          ),
        ],
      ),
    );
  }

  void _exportConfiguration() {
    // Show coming soon dialog
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Coming Soon'),
        content: Text('Configuration export feature is currently under development.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  void _importConfiguration() {
    // Show coming soon dialog
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Coming Soon'),
        content: Text('Configuration import feature is currently under development.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  void _openDocumentation() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Documentation'),
        content: Text('Documentation will be available at docs.opensim.org when released.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  void _reportIssue() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Report Issue'),
        content: Text('Issues can be reported at github.com/opensim/opensim-next when available.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text('OK'),
          ),
        ],
      ),
    );
  }

  void _openSourceCode() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Source Code'),
        content: Text('Source code will be available at github.com/opensim/opensim-next when released.'),
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