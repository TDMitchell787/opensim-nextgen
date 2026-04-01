import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/configuration_builder_models.dart';
import '../providers/configuration_builder_provider.dart';
import '../widgets/config_builder/template_selector.dart';
import '../widgets/config_builder/system_requirements_card.dart';
import '../widgets/config_builder/grid_planner_widget.dart';
import '../widgets/config_builder/deployment_cart_widget.dart';

class ConfigurationBuilderScreen extends StatefulWidget {
  const ConfigurationBuilderScreen({super.key});

  @override
  State<ConfigurationBuilderScreen> createState() => _ConfigurationBuilderScreenState();
}

class _ConfigurationBuilderScreenState extends State<ConfigurationBuilderScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _formKey = GlobalKey<FormState>();
  final _cartKey = GlobalKey<DeploymentCartWidgetState>();
  final _gridPlannerKey = GlobalKey<GridPlannerWidgetState>();
  bool _showCart = true;
  int? _nextAvailablePort;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 5, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  void _showTopSnackBar(BuildContext context, Widget content, {Color? backgroundColor, Duration duration = const Duration(seconds: 4)}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: content,
        backgroundColor: backgroundColor,
        duration: duration,
        behavior: SnackBarBehavior.floating,
        margin: EdgeInsets.only(
          bottom: MediaQuery.of(context).size.height - 150,
          left: 20,
          right: 20,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ConfigurationBuilderProvider>();

    return Scaffold(
      body: Row(
        children: [
          SizedBox(
            width: 280,
            child: _buildLeftPanel(context, provider),
          ),
          const VerticalDivider(width: 1),
          Expanded(
            child: Column(
              children: [
                _buildHeader(context, provider),
                TabBar(
                  controller: _tabController,
                  tabs: const [
                    Tab(text: 'Server', icon: Icon(Icons.dns)),
                    Tab(text: 'Regions', icon: Icon(Icons.map)),
                    Tab(text: 'OSSL', icon: Icon(Icons.security)),
                    Tab(text: 'Includes', icon: Icon(Icons.folder_special)),
                    Tab(text: 'Deploy', icon: Icon(Icons.rocket_launch)),
                  ],
                  isScrollable: false,
                  labelColor: Theme.of(context).colorScheme.primary,
                  unselectedLabelColor: Colors.grey,
                ),
                Expanded(
                  child: provider.currentConfiguration == null
                      ? _buildNoConfigurationSelected(context)
                      : TabBarView(
                          controller: _tabController,
                          children: [
                            _buildServerTab(context, provider),
                            _buildRegionsTab(context, provider),
                            _buildOsslTab(context, provider),
                            _buildIncludesTab(context, provider),
                            _buildDeployTab(context, provider),
                          ],
                        ),
                ),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          _buildCartPanel(context),
        ],
      ),
    );
  }

  Widget _buildCartPanel(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      width: _showCart ? 380 : 48,
      child: _showCart
          ? Column(
              children: [
                _buildCartHeader(context),
                Expanded(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.all(8),
                    child: DeploymentCartWidget(
                      key: _cartKey,
                      onDeploy: () => _showCartDeployDialog(context),
                      onClear: () {
                        _showTopSnackBar(context, const Text('Cart cleared'));
                      },
                    ),
                  ),
                ),
              ],
            )
          : _buildCollapsedCart(context),
    );
  }

  Widget _buildCartHeader(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      color: theme.colorScheme.surface,
      child: Row(
        children: [
          Icon(Icons.shopping_cart, color: theme.colorScheme.primary),
          const SizedBox(width: 8),
          const Expanded(
            child: Text(
              'Deployment Cart',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.chevron_right),
            onPressed: () => setState(() => _showCart = false),
            tooltip: 'Collapse cart',
          ),
        ],
      ),
    );
  }

  Widget _buildCollapsedCart(BuildContext context) {
    final theme = Theme.of(context);
    return InkWell(
      onTap: () => setState(() => _showCart = true),
      child: Container(
        color: theme.colorScheme.surface,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          children: [
            const SizedBox(height: 16),
            RotatedBox(
              quarterTurns: 1,
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.shopping_cart, color: theme.colorScheme.primary, size: 18),
                  const SizedBox(width: 8),
                  const Text(
                    'Cart',
                    style: TextStyle(fontWeight: FontWeight.bold),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            Icon(Icons.chevron_left, color: theme.colorScheme.outline),
          ],
        ),
      ),
    );
  }

  void _showCartDeployDialog(BuildContext context) {
    final cartState = _cartKey.currentState;
    if (cartState == null) return;

    final deployType = cartState.deploymentType;
    final targetPath = cartState.targetPath;
    final autoStart = cartState.autoStart;
    final worldName = cartState.worldName;
    final items = cartState.items;
    final totalRegions = items.fold<int>(0, (sum, item) => sum + item.regionCount);

    String deployTypeLabel;
    IconData deployTypeIcon;
    Color deployTypeColor;

    switch (deployType) {
      case DeploymentType.native:
        deployTypeLabel = 'Native';
        deployTypeIcon = Icons.computer;
        deployTypeColor = Colors.green;
        break;
      case DeploymentType.docker:
        deployTypeLabel = 'Docker';
        deployTypeIcon = Icons.widgets;
        deployTypeColor = Colors.blue;
        break;
      case DeploymentType.kubernetes:
        deployTypeLabel = 'Kubernetes';
        deployTypeIcon = Icons.cloud;
        deployTypeColor = Colors.purple;
        break;
    }

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(Icons.rocket_launch, color: Theme.of(context).colorScheme.primary),
            const SizedBox(width: 8),
            const Text('Deploy World'),
          ],
        ),
        content: SizedBox(
          width: 450,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: deployTypeColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: deployTypeColor.withValues(alpha: 0.3)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(deployTypeIcon, color: deployTypeColor),
                        const SizedBox(width: 8),
                        Text(
                          '$deployTypeLabel Deployment',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: deployTypeColor,
                            fontSize: 16,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 12),
                    _buildDeployInfoRow('World Name', worldName),
                    _buildDeployInfoRow('Total Regions', '$totalRegions regions'),
                    _buildDeployInfoRow('Grid Items', '${items.length} configurations'),
                    _buildDeployInfoRow('Target', targetPath),
                    _buildDeployInfoRow('Auto-start', autoStart ? 'Yes' : 'No'),
                  ],
                ),
              ),
              const SizedBox(height: 16),
              const Text(
                'This will:',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              if (deployType == DeploymentType.native) ...[
                _buildDeployStep('Generate Region.ini files for each grid'),
                _buildDeployStep('Create OpenSim.ini with merged settings'),
                _buildDeployStep('Register instances with Instance Manager'),
                if (autoStart) _buildDeployStep('Start the opensim-next server'),
              ],
              if (deployType == DeploymentType.docker) ...[
                _buildDeployStep('Generate docker-compose.yml with ${items.length} services'),
                _buildDeployStep('Create volume mounts for region configs'),
                _buildDeployStep('Configure container networking'),
                if (autoStart) _buildDeployStep('Run docker-compose up -d'),
              ],
              if (deployType == DeploymentType.kubernetes) ...[
                _buildDeployStep('Generate Helm chart values.yaml'),
                _buildDeployStep('Create ConfigMaps for region configurations'),
                _buildDeployStep('Deploy StatefulSet for persistent storage'),
                if (autoStart) _buildDeployStep('Run helm upgrade --install'),
              ],
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton.icon(
            onPressed: () {
              Navigator.pop(context);
              _executeDeployment(context, deployType, targetPath, autoStart, worldName, items);
            },
            icon: const Icon(Icons.rocket_launch),
            label: Text('Deploy $totalRegions Regions'),
          ),
        ],
      ),
    );
  }

  Widget _buildDeployInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 100,
            child: Text(
              '$label:',
              style: TextStyle(color: Colors.grey[600], fontSize: 13),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildDeployStep(String step) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.check_circle_outline, size: 16, color: Colors.green),
          const SizedBox(width: 8),
          Expanded(child: Text(step, style: const TextStyle(fontSize: 13))),
        ],
      ),
    );
  }

  void _executeDeployment(
    BuildContext context,
    DeploymentType deployType,
    String targetPath,
    bool autoStart,
    String worldName,
    List<CartItem> items,
  ) {
    _showTopSnackBar(
      context,
      Row(
        children: [
          const SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
          ),
          const SizedBox(width: 16),
          Text('Deploying $worldName (${items.fold<int>(0, (s, i) => s + i.regionCount)} regions)...'),
        ],
      ),
      duration: const Duration(seconds: 5),
    );

    // TODO: Call backend API to execute deployment
    // For now, simulate deployment
    Future.delayed(const Duration(seconds: 2), () {
      if (context.mounted) {
        _showTopSnackBar(
          context,
          Row(
            children: [
              const Icon(Icons.check_circle, color: Colors.white),
              const SizedBox(width: 8),
              Text('$worldName deployed successfully!'),
            ],
          ),
          backgroundColor: Colors.green,
        );
        _cartKey.currentState?.clear();
      }
    });
  }

  Widget _buildLeftPanel(BuildContext context, ConfigurationBuilderProvider provider) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          color: Theme.of(context).colorScheme.surface,
          child: Row(
            children: [
              Icon(Icons.build_circle, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 8),
              const Expanded(
                child: Text(
                  'Configuration Builder',
                  style: TextStyle(fontWeight: FontWeight.bold),
                ),
              ),
            ],
          ),
        ),
        const Divider(height: 1),
        Expanded(
          child: DefaultTabController(
            length: 2,
            child: Column(
              children: [
                const TabBar(
                  tabs: [
                    Tab(text: 'Templates'),
                    Tab(text: 'Saved'),
                  ],
                ),
                Expanded(
                  child: TabBarView(
                    children: [
                      TemplateSelector(showSystemRequirements: true),
                      _buildSavedConfigsList(context, provider),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        const Divider(height: 1),
        Padding(
          padding: const EdgeInsets.all(12),
          child: SizedBox(
            width: double.infinity,
            child: ElevatedButton.icon(
              onPressed: () => provider.createNewConfiguration(),
              icon: const Icon(Icons.add),
              label: const Text('New Configuration'),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildSavedConfigsList(BuildContext context, ConfigurationBuilderProvider provider) {
    final configs = provider.savedConfigurations;

    if (configs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.folder_open, size: 48, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'No saved configurations',
              style: TextStyle(color: Colors.grey[600]),
            ),
            const SizedBox(height: 8),
            Text(
              'Select a template and save it',
              style: TextStyle(fontSize: 12, color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: configs.length,
      itemBuilder: (context, index) {
        final config = configs[index];
        final isSelected = provider.selectedConfigurationId == config.id;

        return Card(
          margin: const EdgeInsets.symmetric(vertical: 4),
          elevation: isSelected ? 4 : 1,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
            side: isSelected
                ? BorderSide(color: Theme.of(context).colorScheme.primary, width: 2)
                : BorderSide.none,
          ),
          child: ListTile(
            leading: Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: provider.getDeploymentTypeColor(config.deploymentType).withOpacity(0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(
                provider.getDeploymentTypeIcon(config.deploymentType),
                color: provider.getDeploymentTypeColor(config.deploymentType),
              ),
            ),
            title: Text(config.name, style: const TextStyle(fontSize: 14)),
            subtitle: Row(
              children: [
                _buildStatusBadge(config.deploymentStatus),
                const SizedBox(width: 8),
                Text(
                  config.deploymentTypeLabel,
                  style: const TextStyle(fontSize: 11),
                ),
              ],
            ),
            trailing: PopupMenuButton<String>(
              onSelected: (value) {
                if (value == 'delete') {
                  provider.deleteConfiguration(config.id);
                }
              },
              itemBuilder: (context) => [
                const PopupMenuItem(
                  value: 'delete',
                  child: Row(
                    children: [
                      Icon(Icons.delete, size: 18),
                      SizedBox(width: 8),
                      Text('Delete'),
                    ],
                  ),
                ),
              ],
            ),
            onTap: () => provider.selectConfiguration(config.id),
          ),
        );
      },
    );
  }

  Widget _buildStatusBadge(DeploymentStatus status) {
    Color color;
    String label;

    switch (status) {
      case DeploymentStatus.draft:
        color = Colors.grey;
        label = 'Draft';
        break;
      case DeploymentStatus.ready:
        color = Colors.blue;
        label = 'Ready';
        break;
      case DeploymentStatus.deployed:
        color = Colors.green;
        label = 'Deployed';
        break;
      case DeploymentStatus.failed:
        color = Colors.red;
        label = 'Failed';
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(fontSize: 10, color: color, fontWeight: FontWeight.bold),
      ),
    );
  }

  Widget _buildHeader(BuildContext context, ConfigurationBuilderProvider provider) {
    return Container(
      padding: const EdgeInsets.all(16),
      color: Theme.of(context).colorScheme.surface,
      child: Row(
        children: [
          if (provider.currentConfiguration != null) ...[
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    provider.currentConfiguration!.name,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                  if (provider.selectedTemplate != null)
                    Text(
                      'Based on: ${provider.selectedTemplate!.name}',
                      style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                    ),
                ],
              ),
            ),
          ] else
            const Expanded(child: Text('Select a template to begin')),
          if (provider.currentConfiguration != null) ...[
            OutlinedButton.icon(
              onPressed: () => _showSaveDialog(context, provider),
              icon: const Icon(Icons.save),
              label: const Text('Save'),
            ),
            const SizedBox(width: 8),
            ElevatedButton.icon(
              onPressed: () => _showDeployDialog(context, provider),
              icon: const Icon(Icons.rocket_launch),
              label: const Text('Deploy'),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildNoConfigurationSelected(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.build_circle_outlined, size: 80, color: Colors.grey[400]),
          const SizedBox(height: 24),
          Text(
            'No Configuration Selected',
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
              color: Colors.grey[600],
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'Select a template from the left panel to start configuring',
            style: TextStyle(color: Colors.grey[500]),
          ),
        ],
      ),
    );
  }

  Widget _buildServerTab(BuildContext context, ConfigurationBuilderProvider provider) {
    final config = provider.currentConfiguration!.opensimIni;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (provider.selectedTemplate != null) ...[
              SystemRequirementsCard(
                requirements: provider.selectedTemplate!.systemRequirements,
                title: 'System Requirements',
              ),
              const SizedBox(height: 24),
            ],
            _buildSectionCard(
              context,
              title: 'Startup',
              icon: Icons.play_circle,
              children: [
                TextFormField(
                  initialValue: config.startup.gridName,
                  decoration: const InputDecoration(
                    labelText: 'Grid Name',
                    hintText: 'Enter your grid name',
                  ),
                ),
                const SizedBox(height: 16),
                TextFormField(
                  initialValue: config.startup.welcomeMessage,
                  decoration: const InputDecoration(
                    labelText: 'Welcome Message',
                    hintText: 'Message shown to new users',
                  ),
                  maxLines: 2,
                ),
                const SizedBox(height: 16),
                DropdownButtonFormField<PhysicsEngine>(
                  value: config.startup.physicsEngine,
                  decoration: const InputDecoration(labelText: 'Physics Engine'),
                  items: PhysicsEngine.values.map((engine) {
                    return DropdownMenuItem(
                      value: engine,
                      child: Text(engine.name),
                    );
                  }).toList(),
                  onChanged: (_) {},
                ),
              ],
            ),
            const SizedBox(height: 24),
            _buildSectionCard(
              context,
              title: 'Network',
              icon: Icons.network_check,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: TextFormField(
                        initialValue: config.network.httpPort.toString(),
                        decoration: const InputDecoration(labelText: 'HTTP Port'),
                        keyboardType: TextInputType.number,
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: TextFormField(
                        initialValue: config.network.externalHostName,
                        decoration: const InputDecoration(labelText: 'External Hostname'),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Enable HTTPS/SSL'),
                  value: config.network.httpSSL,
                  onChanged: (_) {},
                ),
                SwitchListTile(
                  title: const Text('Allow Remote Admin'),
                  value: config.network.allowRemoteAdmin,
                  onChanged: (_) {},
                ),
              ],
            ),
            const SizedBox(height: 24),
            _buildSectionCard(
              context,
              title: 'Database',
              icon: Icons.storage,
              children: [
                DropdownButtonFormField<DatabaseProvider>(
                  value: config.database.provider,
                  decoration: const InputDecoration(labelText: 'Database Provider'),
                  items: DatabaseProvider.values.map((db) {
                    return DropdownMenuItem(
                      value: db,
                      child: Text(db.name),
                    );
                  }).toList(),
                  onChanged: (_) {},
                ),
                const SizedBox(height: 16),
                TextFormField(
                  initialValue: config.database.connectionString,
                  decoration: const InputDecoration(
                    labelText: 'Connection String',
                    hintText: 'Database connection string',
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRegionsTab(BuildContext context, ConfigurationBuilderProvider provider) {
    final config = provider.currentConfiguration!.regionIni;
    final templateType = provider.currentConfiguration!.basedOnTemplate ?? 'mainland';

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionCard(
            context,
            title: 'Multi-Region Grid Planner',
            icon: Icons.grid_view,
            children: [
              GridPlannerWidget(
                key: _gridPlannerKey,
                templateType: templateType,
                gridName: config.regionName,
                suggestedBasePort: _nextAvailablePort,
                onConfigChanged: (layout, terrainType, terrainHeight) {
                },
                onAddToCart: (layout, terrainType, terrainHeight, simType, maxAgents) {
                  final cartItem = CartItem(
                    id: DateTime.now().millisecondsSinceEpoch.toString(),
                    name: '${config.regionName} $simType',
                    worldName: provider.currentConfiguration!.name,
                    simulatorType: simType,
                    regionCount: layout.totalRegions,
                    portStart: layout.basePort,
                    portEnd: layout.portRangeEnd,
                    terrainType: terrainType.name,
                    terrainHeight: terrainHeight,
                    maxAgents: maxAgents * layout.totalRegions,
                  );
                  _cartKey.currentState?.addItem(cartItem);

                  final nextPort = _cartKey.currentState?.getNextAvailablePort() ?? 9000;
                  _gridPlannerKey.currentState?.setBasePort(nextPort);

                  setState(() {
                    _nextAvailablePort = nextPort;
                    if (!_showCart) {
                      _showCart = true;
                    }
                  });
                },
              ),
            ],
          ),
          const SizedBox(height: 24),
          _buildSectionCard(
            context,
            title: 'Single Region Settings (Legacy)',
            icon: Icons.map,
            children: [
              TextFormField(
                initialValue: config.regionName,
                decoration: const InputDecoration(labelText: 'Region Name'),
              ),
              const SizedBox(height: 16),
              TextFormField(
                initialValue: config.regionUuid,
                decoration: const InputDecoration(labelText: 'Region UUID'),
                enabled: false,
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: TextFormField(
                      initialValue: config.locationX.toString(),
                      decoration: const InputDecoration(labelText: 'Location X'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: TextFormField(
                      initialValue: config.locationY.toString(),
                      decoration: const InputDecoration(labelText: 'Location Y'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: TextFormField(
                      initialValue: config.sizeX.toString(),
                      decoration: const InputDecoration(labelText: 'Size X'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: TextFormField(
                      initialValue: config.sizeY.toString(),
                      decoration: const InputDecoration(labelText: 'Size Y'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 24),
          _buildSectionCard(
            context,
            title: 'Capacity',
            icon: Icons.group,
            children: [
              Row(
                children: [
                  Expanded(
                    child: TextFormField(
                      initialValue: config.maxAgents.toString(),
                      decoration: const InputDecoration(labelText: 'Max Agents'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: TextFormField(
                      initialValue: config.maxPrims.toString(),
                      decoration: const InputDecoration(labelText: 'Max Prims'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: TextFormField(
                      initialValue: config.internalPort.toString(),
                      decoration: const InputDecoration(labelText: 'Internal Port'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: TextFormField(
                      initialValue: config.waterHeight.toString(),
                      decoration: const InputDecoration(labelText: 'Water Height'),
                      keyboardType: TextInputType.number,
                    ),
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 24),
          _buildSectionCard(
            context,
            title: 'Estate',
            icon: Icons.home_work,
            children: [
              TextFormField(
                initialValue: config.estate.estateName,
                decoration: const InputDecoration(labelText: 'Estate Name'),
              ),
              const SizedBox(height: 16),
              TextFormField(
                initialValue: config.estate.estateOwner,
                decoration: const InputDecoration(labelText: 'Estate Owner'),
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                title: const Text('Allow Voice'),
                value: config.estate.allowVoice,
                onChanged: (_) {},
              ),
              SwitchListTile(
                title: const Text('Allow Flying'),
                value: config.estate.allowFly,
                onChanged: (_) {},
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildOsslTab(BuildContext context, ConfigurationBuilderProvider provider) {
    final config = provider.currentConfiguration!.osslConfig;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionCard(
            context,
            title: 'OSSL Settings',
            icon: Icons.security,
            children: [
              SwitchListTile(
                title: const Text('Allow OSSL Functions'),
                subtitle: const Text('Enable OpenSimulator Scripting Language functions'),
                value: config.allowOSFunctions,
                onChanged: (_) {},
              ),
              const SizedBox(height: 16),
              DropdownButtonFormField<OsslThreatLevel>(
                value: config.osslThreatLevel,
                decoration: const InputDecoration(labelText: 'Threat Level'),
                items: OsslThreatLevel.values.map((level) {
                  return DropdownMenuItem(
                    value: level,
                    child: Text(level.name),
                  );
                }).toList(),
                onChanged: (_) {},
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                title: const Text('Allow MOD Functions'),
                subtitle: const Text('Enable module-specific functions'),
                value: config.allowMODFunctions,
                onChanged: (_) {},
              ),
              SwitchListTile(
                title: const Text('Allow LightShare'),
                subtitle: const Text('Enable Windlight environment sharing'),
                value: config.allowLightShare,
                onChanged: (_) {},
              ),
              SwitchListTile(
                title: const Text('Allow WindLight'),
                subtitle: const Text('Enable WindLight environment controls'),
                value: config.allowWindLight,
                onChanged: (_) {},
              ),
            ],
          ),
          const SizedBox(height: 24),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.warning_amber, color: Colors.orange[700]),
                      const SizedBox(width: 8),
                      const Text(
                        'Threat Level Guide',
                        style: TextStyle(fontWeight: FontWeight.bold),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _buildThreatLevelRow('None', 'Only basic, safe functions'),
                  _buildThreatLevelRow('Nuisance', 'May cause minor annoyances'),
                  _buildThreatLevelRow('VeryLow', 'Minimal security risk'),
                  _buildThreatLevelRow('Low', 'Low security risk'),
                  _buildThreatLevelRow('Moderate', 'Moderate security risk'),
                  _buildThreatLevelRow('High', 'High security risk - trusted scripts only'),
                  _buildThreatLevelRow('VeryHigh', 'Very high risk - admin scripts only'),
                  _buildThreatLevelRow('Severe', 'Maximum access - dangerous'),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildThreatLevelRow(String level, String description) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              level,
              style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 12),
            ),
          ),
          Expanded(
            child: Text(
              description,
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildIncludesTab(BuildContext context, ConfigurationBuilderProvider provider) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionCard(
            context,
            title: 'Configuration Includes',
            icon: Icons.folder_special,
            children: [
              ListTile(
                leading: const Icon(Icons.insert_drive_file),
                title: const Text('Standalone.ini'),
                subtitle: const Text('Standalone mode configuration'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
              ListTile(
                leading: const Icon(Icons.insert_drive_file),
                title: const Text('GridCommon.ini'),
                subtitle: const Text('Grid mode configuration'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
              ListTile(
                leading: const Icon(Icons.insert_drive_file),
                title: const Text('osslEnable.ini'),
                subtitle: const Text('OSSL function overrides'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
            ],
          ),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: () {},
            icon: const Icon(Icons.add),
            label: const Text('Add Include File'),
          ),
        ],
      ),
    );
  }

  Widget _buildDeployTab(BuildContext context, ConfigurationBuilderProvider provider) {
    final config = provider.currentConfiguration!;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionCard(
            context,
            title: 'Deployment Type',
            icon: Icons.rocket_launch,
            children: [
              _buildDeploymentTypeOption(
                context,
                provider,
                DeploymentType.native,
                'Native',
                'Run directly on the host machine',
                Icons.computer,
              ),
              const SizedBox(height: 12),
              _buildDeploymentTypeOption(
                context,
                provider,
                DeploymentType.docker,
                'Docker',
                'Run in a Docker container',
                Icons.widgets,
              ),
              const SizedBox(height: 12),
              _buildDeploymentTypeOption(
                context,
                provider,
                DeploymentType.kubernetes,
                'Kubernetes',
                'Deploy to Kubernetes cluster',
                Icons.cloud,
              ),
            ],
          ),
          const SizedBox(height: 24),
          if (config.deploymentType == DeploymentType.docker)
            _buildDockerSettings(context, provider),
          if (config.deploymentType == DeploymentType.kubernetes)
            _buildKubernetesSettings(context, provider),
          const SizedBox(height: 24),
          _buildSectionCard(
            context,
            title: 'Deployment Options',
            icon: Icons.settings,
            children: [
              TextFormField(
                decoration: const InputDecoration(
                  labelText: 'Target Path',
                  hintText: '/opt/opensim/instances/myinstance',
                ),
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                title: const Text('Auto-start after deployment'),
                value: true,
                onChanged: (_) {},
              ),
              SwitchListTile(
                title: const Text('Register with Instance Manager'),
                value: true,
                onChanged: (_) {},
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDeploymentTypeOption(
    BuildContext context,
    ConfigurationBuilderProvider provider,
    DeploymentType type,
    String title,
    String description,
    IconData icon,
  ) {
    final isSelected = provider.currentConfiguration?.deploymentType == type;
    final color = provider.getDeploymentTypeColor(type);

    return InkWell(
      onTap: () => provider.updateDeploymentSettings(type, null),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          border: Border.all(
            color: isSelected ? color : Colors.grey[300]!,
            width: isSelected ? 2 : 1,
          ),
          borderRadius: BorderRadius.circular(12),
          color: isSelected ? color.withOpacity(0.05) : null,
        ),
        child: Row(
          children: [
            Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: color.withOpacity(0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(icon, color: color),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: const TextStyle(fontWeight: FontWeight.bold),
                  ),
                  Text(
                    description,
                    style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                  ),
                ],
              ),
            ),
            if (isSelected)
              Icon(Icons.check_circle, color: color),
          ],
        ),
      ),
    );
  }

  Widget _buildDockerSettings(BuildContext context, ConfigurationBuilderProvider provider) {
    return _buildSectionCard(
      context,
      title: 'Docker Settings',
      icon: Icons.widgets,
      children: [
        TextFormField(
          initialValue: 'opensim-next:latest',
          decoration: const InputDecoration(labelText: 'Docker Image'),
        ),
        const SizedBox(height: 16),
        Row(
          children: [
            Expanded(
              child: TextFormField(
                initialValue: '2048',
                decoration: const InputDecoration(labelText: 'Memory Limit (MB)'),
                keyboardType: TextInputType.number,
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: TextFormField(
                initialValue: '2.0',
                decoration: const InputDecoration(labelText: 'CPU Limit'),
                keyboardType: TextInputType.number,
              ),
            ),
          ],
        ),
        const SizedBox(height: 16),
        DropdownButtonFormField<String>(
          value: 'unless-stopped',
          decoration: const InputDecoration(labelText: 'Restart Policy'),
          items: const [
            DropdownMenuItem(value: 'no', child: Text('No')),
            DropdownMenuItem(value: 'always', child: Text('Always')),
            DropdownMenuItem(value: 'on-failure', child: Text('On Failure')),
            DropdownMenuItem(value: 'unless-stopped', child: Text('Unless Stopped')),
          ],
          onChanged: (_) {},
        ),
      ],
    );
  }

  Widget _buildKubernetesSettings(BuildContext context, ConfigurationBuilderProvider provider) {
    return _buildSectionCard(
      context,
      title: 'Kubernetes Settings',
      icon: Icons.cloud,
      children: [
        TextFormField(
          initialValue: 'opensim',
          decoration: const InputDecoration(labelText: 'Namespace'),
        ),
        const SizedBox(height: 16),
        Row(
          children: [
            Expanded(
              child: TextFormField(
                initialValue: '1',
                decoration: const InputDecoration(labelText: 'Replicas'),
                keyboardType: TextInputType.number,
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: SwitchListTile(
                title: const Text('Enable HPA'),
                value: false,
                onChanged: (_) {},
                contentPadding: EdgeInsets.zero,
              ),
            ),
          ],
        ),
        const SizedBox(height: 16),
        TextFormField(
          decoration: const InputDecoration(
            labelText: 'Ingress Host',
            hintText: 'opensim.example.com',
          ),
        ),
        const SizedBox(height: 16),
        SwitchListTile(
          title: const Text('Enable TLS'),
          value: false,
          onChanged: (_) {},
        ),
      ],
    );
  }

  Widget _buildSectionCard(
    BuildContext context, {
    required String title,
    required IconData icon,
    required List<Widget> children,
  }) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, color: Theme.of(context).colorScheme.primary),
                const SizedBox(width: 12),
                Text(
                  title,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const Divider(height: 24),
            ...children,
          ],
        ),
      ),
    );
  }

  void _showSaveDialog(BuildContext context, ConfigurationBuilderProvider provider) {
    final nameController = TextEditingController(
      text: provider.currentConfiguration?.name ?? '',
    );
    final descController = TextEditingController(
      text: provider.currentConfiguration?.description ?? '',
    );

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Save Configuration'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(labelText: 'Configuration Name'),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: descController,
              decoration: const InputDecoration(labelText: 'Description'),
              maxLines: 2,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              provider.saveConfiguration(
                name: nameController.text,
                description: descController.text,
              );
              Navigator.pop(context);
              _showTopSnackBar(context, const Text('Configuration saved'));
            },
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _showDeployDialog(BuildContext context, ConfigurationBuilderProvider provider) {
    final pathController = TextEditingController(
      text: '/opt/opensim/instances/${provider.currentConfiguration?.name.toLowerCase().replaceAll(' ', '_') ?? 'instance'}',
    );
    bool autoStart = true;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: const Text('Deploy Configuration'),
          content: SizedBox(
            width: 400,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Configuration: ${provider.currentConfiguration?.name}',
                  style: const TextStyle(fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 8),
                Text(
                  'Type: ${provider.currentConfiguration?.deploymentTypeLabel}',
                  style: TextStyle(color: Colors.grey[600]),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: pathController,
                  decoration: const InputDecoration(
                    labelText: 'Target Path',
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 16),
                CheckboxListTile(
                  title: const Text('Auto-start after deployment'),
                  value: autoStart,
                  onChanged: (v) => setState(() => autoStart = v ?? true),
                  controlAffinity: ListTileControlAffinity.leading,
                  contentPadding: EdgeInsets.zero,
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () async {
                Navigator.pop(context);
                final result = await provider.deployConfiguration(
                  targetPath: pathController.text,
                  autoStart: autoStart,
                );
                if (context.mounted) {
                  _showTopSnackBar(
                    context,
                    Text(result.message),
                    backgroundColor: result.success ? Colors.green : Colors.red,
                  );
                }
              },
              child: const Text('Deploy'),
            ),
          ],
        ),
      ),
    );
  }
}
