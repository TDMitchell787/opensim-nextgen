import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';
import '../models/deployment_models.dart';

class ConfigurationWizard extends StatefulWidget {
  final PageController pageController;
  final int currentPage;
  final ValueChanged<int> onPageChanged;
  final VoidCallback onWizardComplete;

  const ConfigurationWizard({
    required this.pageController,
    required this.currentPage,
    required this.onPageChanged,
    required this.onWizardComplete,
  });

  @override
  State<ConfigurationWizard> createState() => _ConfigurationWizardState();
}

class _ConfigurationWizardState extends State<ConfigurationWizard> {
  final _formKey = GlobalKey<FormState>();

  final _httpPortController = TextEditingController();
  final _httpsPortController = TextEditingController();
  final _hostnameController = TextEditingController();
  final _internalIpController = TextEditingController();
  final _dbConnectionController = TextEditingController();
  final _sslCertController = TextEditingController();
  final _sslKeyController = TextEditingController();
  final _maxPrimsController = TextEditingController();
  final _maxScriptsController = TextEditingController();
  final _scriptTimeoutController = TextEditingController();
  final _cacheTimeoutController = TextEditingController();

  bool _httpsEnabled = false;
  bool _passwordComplexity = true;
  bool _bruteForceProtection = true;
  bool _cacheAssets = true;
  int _sessionTimeout = 3600;
  String _databaseType = 'SQLite';

  static const _stepTitles = [
    'Deployment Type',
    'Network',
    'Database',
    'Security',
    'Performance',
  ];

  @override
  void initState() {
    super.initState();
    _syncFromProvider();
  }

  void _syncFromProvider() {
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    final config = provider.currentConfig;
    if (config != null) {
      _httpPortController.text = config.networkConfig.httpPort.toString();
      _httpsPortController.text = config.networkConfig.httpsPort.toString();
      _hostnameController.text = config.networkConfig.externalHostname;
      _internalIpController.text = config.networkConfig.internalIp;
      _httpsEnabled = config.networkConfig.httpsEnabled;
      _databaseType = config.databaseType;
      _dbConnectionController.text = config.databaseConnection;
      _passwordComplexity = config.securityConfig.passwordComplexity;
      _bruteForceProtection = config.securityConfig.bruteForceProtection;
      _sessionTimeout = config.securityConfig.sessionTimeout;
      _sslCertController.text = config.securityConfig.sslCertificatePath;
      _sslKeyController.text = config.securityConfig.sslPrivateKeyPath;
      _maxPrimsController.text = config.performanceConfig.maxPrims.toString();
      _maxScriptsController.text = config.performanceConfig.maxScripts.toString();
      _scriptTimeoutController.text = config.performanceConfig.scriptTimeout.toString();
      _cacheAssets = config.performanceConfig.cacheAssets;
      _cacheTimeoutController.text = config.performanceConfig.cacheTimeout.toString();
    } else {
      _httpPortController.text = '9000';
      _httpsPortController.text = '9001';
      _hostnameController.text = 'localhost';
      _internalIpController.text = '0.0.0.0';
      _dbConnectionController.text = '';
      _maxPrimsController.text = '15000';
      _maxScriptsController.text = '5000';
      _scriptTimeoutController.text = '30';
      _cacheTimeoutController.text = '24';
    }
  }

  @override
  void dispose() {
    _httpPortController.dispose();
    _httpsPortController.dispose();
    _hostnameController.dispose();
    _internalIpController.dispose();
    _dbConnectionController.dispose();
    _sslCertController.dispose();
    _sslKeyController.dispose();
    _maxPrimsController.dispose();
    _maxScriptsController.dispose();
    _scriptTimeoutController.dispose();
    _cacheTimeoutController.dispose();
    super.dispose();
  }

  void _saveCurrentStep() {
    final provider = Provider.of<ConfiguratorProvider>(context, listen: false);
    final config = provider.currentConfig;
    if (config == null) return;

    switch (widget.currentPage) {
      case 1:
        provider.updateConfiguration(config.copyWith(
          networkConfig: config.networkConfig.copyWith(
            httpPort: int.tryParse(_httpPortController.text) ?? 9000,
            httpsPort: int.tryParse(_httpsPortController.text) ?? 9001,
            httpsEnabled: _httpsEnabled,
            externalHostname: _hostnameController.text,
            internalIp: _internalIpController.text,
          ),
        ));
        break;
      case 2:
        provider.updateConfiguration(config.copyWith(
          databaseType: _databaseType,
          databaseConnection: _dbConnectionController.text,
        ));
        break;
      case 3:
        provider.updateConfiguration(config.copyWith(
          securityConfig: config.securityConfig.copyWith(
            passwordComplexity: _passwordComplexity,
            bruteForceProtection: _bruteForceProtection,
            sessionTimeout: _sessionTimeout,
            sslCertificatePath: _sslCertController.text,
            sslPrivateKeyPath: _sslKeyController.text,
          ),
        ));
        break;
      case 4:
        provider.updateConfiguration(config.copyWith(
          performanceConfig: config.performanceConfig.copyWith(
            maxPrims: int.tryParse(_maxPrimsController.text) ?? 15000,
            maxScripts: int.tryParse(_maxScriptsController.text) ?? 5000,
            scriptTimeout: int.tryParse(_scriptTimeoutController.text) ?? 30,
            cacheAssets: _cacheAssets,
            cacheTimeout: int.tryParse(_cacheTimeoutController.text) ?? 24,
          ),
        ));
        break;
    }
  }

  void _goNext() {
    _saveCurrentStep();
    if (widget.currentPage < 4) {
      widget.onPageChanged(widget.currentPage + 1);
    } else {
      widget.onWizardComplete();
    }
  }

  void _goPrevious() {
    _saveCurrentStep();
    if (widget.currentPage > 0) {
      widget.onPageChanged(widget.currentPage - 1);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildStepIndicator(),
            SizedBox(height: 24),
            Text(_stepTitles[widget.currentPage],
                style: Theme.of(context).textTheme.titleLarge),
            SizedBox(height: 16),
            Form(
              key: _formKey,
              child: _buildStepContent(),
            ),
            SizedBox(height: 24),
            _buildNavigation(),
          ],
        ),
      ),
    );
  }

  Widget _buildStepIndicator() {
    return Row(
      children: List.generate(5, (index) {
        final isActive = index == widget.currentPage;
        final isCompleted = index < widget.currentPage;
        return Expanded(
          child: Container(
            margin: EdgeInsets.symmetric(horizontal: 2),
            height: 4,
            decoration: BoxDecoration(
              color: isCompleted
                  ? Colors.green
                  : isActive
                      ? Theme.of(context).colorScheme.primary
                      : Colors.grey[300],
              borderRadius: BorderRadius.circular(2),
            ),
          ),
        );
      }),
    );
  }

  Widget _buildStepContent() {
    switch (widget.currentPage) {
      case 0: return _buildDeploymentStep();
      case 1: return _buildNetworkStep();
      case 2: return _buildDatabaseStep();
      case 3: return _buildSecurityStep();
      case 4: return _buildPerformanceStep();
      default: return SizedBox.shrink();
    }
  }

  Widget _buildDeploymentStep() {
    return Consumer<ConfiguratorProvider>(
      builder: (context, provider, _) {
        final types = DeploymentTypeInfo.getAllTypes();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Choose the deployment type that best matches your needs:',
                style: Theme.of(context).textTheme.bodyMedium),
            SizedBox(height: 16),
            ...types.map((info) => _buildDeploymentCard(info, provider)),
          ],
        );
      },
    );
  }

  Widget _buildDeploymentCard(DeploymentTypeInfo info, ConfiguratorProvider provider) {
    final isSelected = provider.selectedDeploymentType == info.type;
    return Card(
      elevation: isSelected ? 4 : 1,
      margin: EdgeInsets.only(bottom: 12),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: isSelected ? provider.getDeploymentTypeColor(info.type) : Colors.transparent,
          width: 2,
        ),
      ),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: () => provider.selectDeploymentType(info.type),
        child: Padding(
          padding: EdgeInsets.all(16),
          child: Row(
            children: [
              Text(info.icon, style: TextStyle(fontSize: 32)),
              SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(info.name, style: Theme.of(context).textTheme.titleMedium),
                    SizedBox(height: 4),
                    Text(info.description, style: Theme.of(context).textTheme.bodySmall),
                    SizedBox(height: 8),
                    Wrap(
                      spacing: 8,
                      children: info.features.take(3).map((f) => Chip(
                        label: Text(f, style: TextStyle(fontSize: 10)),
                        visualDensity: VisualDensity.compact,
                        padding: EdgeInsets.zero,
                      )).toList(),
                    ),
                  ],
                ),
              ),
              if (isSelected)
                Icon(Icons.check_circle, color: provider.getDeploymentTypeColor(info.type)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildNetworkStep() {
    return Column(
      children: [
        Row(
          children: [
            Expanded(
              child: TextFormField(
                controller: _httpPortController,
                decoration: InputDecoration(labelText: 'HTTP Port', helperText: 'Default: 9000'),
                keyboardType: TextInputType.number,
              ),
            ),
            SizedBox(width: 16),
            Expanded(
              child: TextFormField(
                controller: _httpsPortController,
                decoration: InputDecoration(labelText: 'HTTPS Port', helperText: 'Default: 9001'),
                keyboardType: TextInputType.number,
              ),
            ),
          ],
        ),
        SizedBox(height: 16),
        SwitchListTile(
          title: Text('Enable HTTPS'),
          subtitle: Text('Encrypt connections with SSL/TLS'),
          value: _httpsEnabled,
          onChanged: (v) => setState(() => _httpsEnabled = v),
        ),
        SizedBox(height: 8),
        TextFormField(
          controller: _hostnameController,
          decoration: InputDecoration(
            labelText: 'External Hostname',
            helperText: 'Public hostname or IP for viewer connections',
            prefixIcon: Icon(Icons.dns),
          ),
        ),
        SizedBox(height: 16),
        TextFormField(
          controller: _internalIpController,
          decoration: InputDecoration(
            labelText: 'Internal IP',
            helperText: 'Bind address (0.0.0.0 for all interfaces)',
            prefixIcon: Icon(Icons.lan),
          ),
        ),
      ],
    );
  }

  Widget _buildDatabaseStep() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DropdownButtonFormField<String>(
          value: _databaseType,
          decoration: InputDecoration(
            labelText: 'Database Type',
            prefixIcon: Icon(Icons.storage),
          ),
          items: ['SQLite', 'PostgreSQL', 'MySQL', 'MariaDB'].map((type) => DropdownMenuItem(
            value: type,
            child: Text(type),
          )).toList(),
          onChanged: (v) => setState(() => _databaseType = v ?? 'SQLite'),
        ),
        SizedBox(height: 16),
        TextFormField(
          controller: _dbConnectionController,
          decoration: InputDecoration(
            labelText: 'Connection String',
            helperText: _databaseType == 'SQLite'
                ? 'File path (e.g., ./opensim.db)'
                : 'postgresql://user:pass@localhost/opensim',
            prefixIcon: Icon(Icons.link),
          ),
          maxLines: 1,
        ),
        SizedBox(height: 16),
        Card(
          color: Colors.blue[50],
          child: Padding(
            padding: EdgeInsets.all(12),
            child: Row(
              children: [
                Icon(Icons.info_outline, color: Colors.blue, size: 20),
                SizedBox(width: 12),
                Expanded(
                  child: Text(
                    _databaseType == 'SQLite'
                        ? 'SQLite is recommended for development. No server setup required.'
                        : _databaseType == 'PostgreSQL'
                            ? 'PostgreSQL provides best performance for production deployments.'
                            : 'MySQL/MariaDB provides enterprise compatibility.',
                    style: TextStyle(fontSize: 12, color: Colors.blue[800]),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildSecurityStep() {
    return Column(
      children: [
        SwitchListTile(
          title: Text('Password Complexity'),
          subtitle: Text('Require strong passwords for user accounts'),
          value: _passwordComplexity,
          onChanged: (v) => setState(() => _passwordComplexity = v),
        ),
        SwitchListTile(
          title: Text('Brute Force Protection'),
          subtitle: Text('Lock accounts after failed login attempts'),
          value: _bruteForceProtection,
          onChanged: (v) => setState(() => _bruteForceProtection = v),
        ),
        SizedBox(height: 8),
        ListTile(
          title: Text('Session Timeout'),
          subtitle: Text('${(_sessionTimeout / 60).round()} minutes'),
          trailing: SizedBox(
            width: 200,
            child: Slider(
              value: _sessionTimeout.toDouble(),
              min: 300,
              max: 86400,
              divisions: 20,
              label: '${(_sessionTimeout / 60).round()} min',
              onChanged: (v) => setState(() => _sessionTimeout = v.round()),
            ),
          ),
        ),
        SizedBox(height: 8),
        TextFormField(
          controller: _sslCertController,
          decoration: InputDecoration(
            labelText: 'SSL Certificate Path',
            helperText: 'Path to .pem certificate file',
            prefixIcon: Icon(Icons.security),
          ),
        ),
        SizedBox(height: 16),
        TextFormField(
          controller: _sslKeyController,
          decoration: InputDecoration(
            labelText: 'SSL Private Key Path',
            helperText: 'Path to private key file',
            prefixIcon: Icon(Icons.vpn_key),
          ),
        ),
      ],
    );
  }

  Widget _buildPerformanceStep() {
    return Column(
      children: [
        Row(
          children: [
            Expanded(
              child: TextFormField(
                controller: _maxPrimsController,
                decoration: InputDecoration(labelText: 'Max Prims', helperText: 'Per region'),
                keyboardType: TextInputType.number,
              ),
            ),
            SizedBox(width: 16),
            Expanded(
              child: TextFormField(
                controller: _maxScriptsController,
                decoration: InputDecoration(labelText: 'Max Scripts', helperText: 'Per region'),
                keyboardType: TextInputType.number,
              ),
            ),
          ],
        ),
        SizedBox(height: 16),
        TextFormField(
          controller: _scriptTimeoutController,
          decoration: InputDecoration(
            labelText: 'Script Timeout (seconds)',
            helperText: 'Max execution time per script event',
            prefixIcon: Icon(Icons.timer),
          ),
          keyboardType: TextInputType.number,
        ),
        SizedBox(height: 16),
        SwitchListTile(
          title: Text('Asset Caching'),
          subtitle: Text('Cache assets locally for faster access'),
          value: _cacheAssets,
          onChanged: (v) => setState(() => _cacheAssets = v),
        ),
        if (_cacheAssets) ...[
          SizedBox(height: 8),
          TextFormField(
            controller: _cacheTimeoutController,
            decoration: InputDecoration(
              labelText: 'Cache Timeout (hours)',
              helperText: 'How long to keep cached assets',
              prefixIcon: Icon(Icons.access_time),
            ),
            keyboardType: TextInputType.number,
          ),
        ],
      ],
    );
  }

  Widget _buildNavigation() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        if (widget.currentPage > 0)
          OutlinedButton.icon(
            onPressed: _goPrevious,
            icon: Icon(Icons.arrow_back, size: 18),
            label: Text('Previous'),
          )
        else
          SizedBox.shrink(),
        ElevatedButton.icon(
          onPressed: _goNext,
          icon: Icon(widget.currentPage < 4 ? Icons.arrow_forward : Icons.check, size: 18),
          label: Text(widget.currentPage < 4 ? 'Next' : 'Finish'),
        ),
      ],
    );
  }
}
