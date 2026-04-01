import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/configuration_builder_models.dart';
import '../../providers/configuration_builder_provider.dart';

class OpenSimIniEditor extends StatefulWidget {
  final OpenSimIniConfig config;
  final ValueChanged<OpenSimIniConfig>? onChanged;

  const OpenSimIniEditor({
    super.key,
    required this.config,
    this.onChanged,
  });

  @override
  State<OpenSimIniEditor> createState() => _OpenSimIniEditorState();
}

class _OpenSimIniEditorState extends State<OpenSimIniEditor> {
  late TextEditingController _gridNameController;
  late TextEditingController _welcomeMessageController;
  late TextEditingController _httpPortController;
  late TextEditingController _externalHostController;
  late TextEditingController _internalIpController;
  late TextEditingController _connectionStringController;

  @override
  void initState() {
    super.initState();
    _initControllers();
  }

  void _initControllers() {
    _gridNameController = TextEditingController(text: widget.config.gridName);
    _welcomeMessageController = TextEditingController(text: widget.config.welcomeMessage);
    _httpPortController = TextEditingController(text: widget.config.httpPort.toString());
    _externalHostController = TextEditingController(text: widget.config.externalHostName);
    _internalIpController = TextEditingController(text: widget.config.internalIp);
    _connectionStringController = TextEditingController(text: widget.config.connectionString);
  }

  @override
  void didUpdateWidget(OpenSimIniEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.config != widget.config) {
      _gridNameController.text = widget.config.gridName;
      _welcomeMessageController.text = widget.config.welcomeMessage;
      _httpPortController.text = widget.config.httpPort.toString();
      _externalHostController.text = widget.config.externalHostName;
      _internalIpController.text = widget.config.internalIp;
      _connectionStringController.text = widget.config.connectionString;
    }
  }

  @override
  void dispose() {
    _gridNameController.dispose();
    _welcomeMessageController.dispose();
    _httpPortController.dispose();
    _externalHostController.dispose();
    _internalIpController.dispose();
    _connectionStringController.dispose();
    super.dispose();
  }

  void _notifyChange() {
    final provider = context.read<ConfigurationBuilderProvider>();
    provider.updateOpenSimIni(OpenSimIniConfig(
      gridName: _gridNameController.text,
      welcomeMessage: _welcomeMessageController.text,
      allowAnonymousLogin: widget.config.allowAnonymousLogin,
      httpPort: int.tryParse(_httpPortController.text) ?? 9000,
      externalHostName: _externalHostController.text,
      internalIp: _internalIpController.text,
      databaseProvider: widget.config.databaseProvider,
      connectionString: _connectionStringController.text,
      physicsEngine: widget.config.physicsEngine,
      enableVoice: widget.config.enableVoice,
      enableSearch: widget.config.enableSearch,
      enableCurrency: widget.config.enableCurrency,
      additionalSections: widget.config.additionalSections,
    ));
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Startup Configuration'),
          const SizedBox(height: 12),
          _buildStartupSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Network Configuration'),
          const SizedBox(height: 12),
          _buildNetworkSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Database Configuration'),
          const SizedBox(height: 12),
          _buildDatabaseSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Features'),
          const SizedBox(height: 12),
          _buildFeaturesSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Physics Engine'),
          const SizedBox(height: 12),
          _buildPhysicsSection(),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            _getSectionIcon(title),
            size: 18,
            color: Theme.of(context).colorScheme.onPrimaryContainer,
          ),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.onPrimaryContainer,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getSectionIcon(String title) {
    switch (title) {
      case 'Startup Configuration':
        return Icons.play_arrow;
      case 'Network Configuration':
        return Icons.network_check;
      case 'Database Configuration':
        return Icons.storage;
      case 'Features':
        return Icons.extension;
      case 'Physics Engine':
        return Icons.science;
      default:
        return Icons.settings;
    }
  }

  Widget _buildStartupSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextFormField(
              controller: _gridNameController,
              decoration: const InputDecoration(
                labelText: 'Grid Name',
                hintText: 'Enter your grid name',
                prefixIcon: Icon(Icons.grid_on),
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => _notifyChange(),
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _welcomeMessageController,
              decoration: const InputDecoration(
                labelText: 'Welcome Message',
                hintText: 'Message shown to users on login',
                prefixIcon: Icon(Icons.message),
                border: OutlineInputBorder(),
              ),
              maxLines: 2,
              onChanged: (_) => _notifyChange(),
            ),
            const SizedBox(height: 16),
            Consumer<ConfigurationBuilderProvider>(
              builder: (context, provider, _) {
                final config = provider.currentOpenSimIni;
                return SwitchListTile(
                  title: const Text('Allow Anonymous Login'),
                  subtitle: const Text('Enable guest access without registration'),
                  value: config?.allowAnonymousLogin ?? false,
                  onChanged: (value) {
                    if (config != null) {
                      provider.updateOpenSimIni(OpenSimIniConfig(
                        gridName: config.gridName,
                        welcomeMessage: config.welcomeMessage,
                        allowAnonymousLogin: value,
                        httpPort: config.httpPort,
                        externalHostName: config.externalHostName,
                        internalIp: config.internalIp,
                        databaseProvider: config.databaseProvider,
                        connectionString: config.connectionString,
                        physicsEngine: config.physicsEngine,
                        enableVoice: config.enableVoice,
                        enableSearch: config.enableSearch,
                        enableCurrency: config.enableCurrency,
                        additionalSections: config.additionalSections,
                      ));
                    }
                  },
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNetworkSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _httpPortController,
                    decoration: const InputDecoration(
                      labelText: 'HTTP Port',
                      hintText: '9000',
                      prefixIcon: Icon(Icons.http),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _internalIpController,
                    decoration: const InputDecoration(
                      labelText: 'Internal IP',
                      hintText: '0.0.0.0',
                      prefixIcon: Icon(Icons.computer),
                      border: OutlineInputBorder(),
                    ),
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _externalHostController,
              decoration: const InputDecoration(
                labelText: 'External Hostname',
                hintText: 'yourgrid.example.com',
                prefixIcon: Icon(Icons.public),
                border: OutlineInputBorder(),
                helperText: 'Public hostname or IP for external access',
              ),
              onChanged: (_) => _notifyChange(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDatabaseSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Consumer<ConfigurationBuilderProvider>(
              builder: (context, provider, _) {
                final config = provider.currentOpenSimIni;
                return DropdownButtonFormField<DatabaseProvider>(
                  value: config?.databaseProvider ?? DatabaseProvider.sqlite,
                  decoration: const InputDecoration(
                    labelText: 'Database Provider',
                    prefixIcon: Icon(Icons.storage),
                    border: OutlineInputBorder(),
                  ),
                  items: DatabaseProvider.values.map((provider) {
                    return DropdownMenuItem(
                      value: provider,
                      child: Row(
                        children: [
                          Icon(
                            _getDatabaseIcon(provider),
                            size: 18,
                            color: _getDatabaseColor(provider),
                          ),
                          const SizedBox(width: 8),
                          Text(_getDatabaseName(provider)),
                        ],
                      ),
                    );
                  }).toList(),
                  onChanged: (value) {
                    if (config != null && value != null) {
                      provider.updateOpenSimIni(OpenSimIniConfig(
                        gridName: config.gridName,
                        welcomeMessage: config.welcomeMessage,
                        allowAnonymousLogin: config.allowAnonymousLogin,
                        httpPort: config.httpPort,
                        externalHostName: config.externalHostName,
                        internalIp: config.internalIp,
                        databaseProvider: value,
                        connectionString: _getDefaultConnectionString(value),
                        physicsEngine: config.physicsEngine,
                        enableVoice: config.enableVoice,
                        enableSearch: config.enableSearch,
                        enableCurrency: config.enableCurrency,
                        additionalSections: config.additionalSections,
                      ));
                      _connectionStringController.text = _getDefaultConnectionString(value);
                    }
                  },
                );
              },
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _connectionStringController,
              decoration: const InputDecoration(
                labelText: 'Connection String',
                prefixIcon: Icon(Icons.link),
                border: OutlineInputBorder(),
                helperText: 'Database connection string',
              ),
              onChanged: (_) => _notifyChange(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildFeaturesSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Consumer<ConfigurationBuilderProvider>(
          builder: (context, provider, _) {
            final config = provider.currentOpenSimIni;
            if (config == null) return const SizedBox.shrink();

            return Column(
              children: [
                _buildFeatureSwitch(
                  'Voice Chat',
                  'Enable voice communication between users',
                  Icons.mic,
                  config.enableVoice,
                  (value) => _updateFeature(config, enableVoice: value),
                ),
                const Divider(),
                _buildFeatureSwitch(
                  'Search',
                  'Enable grid-wide search functionality',
                  Icons.search,
                  config.enableSearch,
                  (value) => _updateFeature(config, enableSearch: value),
                ),
                const Divider(),
                _buildFeatureSwitch(
                  'Currency System',
                  'Enable in-world economy and transactions',
                  Icons.attach_money,
                  config.enableCurrency,
                  (value) => _updateFeature(config, enableCurrency: value),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildFeatureSwitch(
    String title,
    String subtitle,
    IconData icon,
    bool value,
    ValueChanged<bool> onChanged,
  ) {
    return SwitchListTile(
      title: Row(
        children: [
          Icon(icon, size: 20),
          const SizedBox(width: 8),
          Text(title),
        ],
      ),
      subtitle: Text(subtitle),
      value: value,
      onChanged: onChanged,
    );
  }

  void _updateFeature(
    OpenSimIniConfig config, {
    bool? enableVoice,
    bool? enableSearch,
    bool? enableCurrency,
  }) {
    final provider = context.read<ConfigurationBuilderProvider>();
    provider.updateOpenSimIni(OpenSimIniConfig(
      gridName: config.gridName,
      welcomeMessage: config.welcomeMessage,
      allowAnonymousLogin: config.allowAnonymousLogin,
      httpPort: config.httpPort,
      externalHostName: config.externalHostName,
      internalIp: config.internalIp,
      databaseProvider: config.databaseProvider,
      connectionString: config.connectionString,
      physicsEngine: config.physicsEngine,
      enableVoice: enableVoice ?? config.enableVoice,
      enableSearch: enableSearch ?? config.enableSearch,
      enableCurrency: enableCurrency ?? config.enableCurrency,
      additionalSections: config.additionalSections,
    ));
  }

  Widget _buildPhysicsSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Consumer<ConfigurationBuilderProvider>(
          builder: (context, provider, _) {
            final config = provider.currentOpenSimIni;
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DropdownButtonFormField<PhysicsEngine>(
                  value: config?.physicsEngine ?? PhysicsEngine.ubOde,
                  decoration: const InputDecoration(
                    labelText: 'Physics Engine',
                    prefixIcon: Icon(Icons.science),
                    border: OutlineInputBorder(),
                  ),
                  items: PhysicsEngine.values.map((engine) {
                    return DropdownMenuItem(
                      value: engine,
                      child: Row(
                        children: [
                          _buildPhysicsIcon(engine),
                          const SizedBox(width: 8),
                          Text(_getPhysicsName(engine)),
                        ],
                      ),
                    );
                  }).toList(),
                  onChanged: (value) {
                    if (config != null && value != null) {
                      provider.updateOpenSimIni(OpenSimIniConfig(
                        gridName: config.gridName,
                        welcomeMessage: config.welcomeMessage,
                        allowAnonymousLogin: config.allowAnonymousLogin,
                        httpPort: config.httpPort,
                        externalHostName: config.externalHostName,
                        internalIp: config.internalIp,
                        databaseProvider: config.databaseProvider,
                        connectionString: config.connectionString,
                        physicsEngine: value,
                        enableVoice: config.enableVoice,
                        enableSearch: config.enableSearch,
                        enableCurrency: config.enableCurrency,
                        additionalSections: config.additionalSections,
                      ));
                    }
                  },
                ),
                const SizedBox(height: 16),
                _buildPhysicsInfo(config?.physicsEngine ?? PhysicsEngine.ubOde),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildPhysicsIcon(PhysicsEngine engine) {
    IconData icon;
    Color color;
    switch (engine) {
      case PhysicsEngine.ubOde:
        icon = Icons.speed;
        color = Colors.green;
        break;
      case PhysicsEngine.bulletSim:
        icon = Icons.flash_on;
        color = Colors.orange;
        break;
      case PhysicsEngine.basicPhysics:
        icon = Icons.square;
        color = Colors.blue;
        break;
      case PhysicsEngine.posPlugin:
        icon = Icons.extension;
        color = Colors.purple;
        break;
    }
    return Icon(icon, size: 18, color: color);
  }

  String _getPhysicsName(PhysicsEngine engine) {
    switch (engine) {
      case PhysicsEngine.ubOde:
        return 'ubODE (Recommended)';
      case PhysicsEngine.bulletSim:
        return 'BulletSim';
      case PhysicsEngine.basicPhysics:
        return 'Basic Physics';
      case PhysicsEngine.posPlugin:
        return 'POS Plugin';
    }
  }

  Widget _buildPhysicsInfo(PhysicsEngine engine) {
    String description;
    Color color;
    IconData icon;

    switch (engine) {
      case PhysicsEngine.ubOde:
        description = 'High-performance physics with excellent collision detection. Best for most use cases.';
        color = Colors.green;
        icon = Icons.check_circle;
        break;
      case PhysicsEngine.bulletSim:
        description = 'Good for vehicle physics and mesh collisions. May use more resources.';
        color = Colors.orange;
        icon = Icons.info;
        break;
      case PhysicsEngine.basicPhysics:
        description = 'Minimal physics for low-resource environments. Limited collision support.';
        color = Colors.blue;
        icon = Icons.info;
        break;
      case PhysicsEngine.posPlugin:
        description = 'Position-only physics. No collision detection. For testing only.';
        color = Colors.purple;
        icon = Icons.warning;
        break;
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Row(
        children: [
          Icon(icon, color: color, size: 20),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              description,
              style: TextStyle(
                fontSize: 13,
                color: Colors.grey[700],
              ),
            ),
          ),
        ],
      ),
    );
  }

  IconData _getDatabaseIcon(DatabaseProvider provider) {
    switch (provider) {
      case DatabaseProvider.sqlite:
        return Icons.storage;
      case DatabaseProvider.postgresql:
        return Icons.cloud;
      case DatabaseProvider.mysql:
        return Icons.dns;
      case DatabaseProvider.mariadb:
        return Icons.storage;
    }
  }

  Color _getDatabaseColor(DatabaseProvider provider) {
    switch (provider) {
      case DatabaseProvider.sqlite:
        return Colors.blue;
      case DatabaseProvider.postgresql:
        return Colors.indigo;
      case DatabaseProvider.mysql:
        return Colors.orange;
      case DatabaseProvider.mariadb:
        return Colors.teal;
    }
  }

  String _getDatabaseName(DatabaseProvider provider) {
    switch (provider) {
      case DatabaseProvider.sqlite:
        return 'SQLite (Embedded)';
      case DatabaseProvider.postgresql:
        return 'PostgreSQL';
      case DatabaseProvider.mysql:
        return 'MySQL';
      case DatabaseProvider.mariadb:
        return 'MariaDB';
    }
  }

  String _getDefaultConnectionString(DatabaseProvider provider) {
    switch (provider) {
      case DatabaseProvider.sqlite:
        return 'Data Source=opensim.db;Version=3';
      case DatabaseProvider.postgresql:
        return 'Host=localhost;Database=opensim;Username=opensim;Password=password';
      case DatabaseProvider.mysql:
        return 'Server=localhost;Database=opensim;Uid=opensim;Pwd=password';
      case DatabaseProvider.mariadb:
        return 'Server=localhost;Database=opensim;Uid=opensim;Pwd=password';
    }
  }
}
