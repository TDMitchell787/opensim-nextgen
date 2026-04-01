import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/instance_models.dart';
import '../providers/instance_manager_provider.dart';
import '../widgets/instances/multi_instance_grid.dart';
import '../widgets/instances/instance_card.dart';
import '../widgets/instances/instance_selector.dart';
import '../widgets/instances/console_terminal.dart';

class InstanceManagerScreen extends StatefulWidget {
  const InstanceManagerScreen({super.key});

  @override
  State<InstanceManagerScreen> createState() => _InstanceManagerScreenState();
}

class _InstanceManagerScreenState extends State<InstanceManagerScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  bool _showConsole = false;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      context.read<InstanceManagerProvider>().fetchInstanceDirectories();
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Instance Manager'),
        actions: [
          IconButton(
            icon: Icon(_showConsole ? Icons.terminal : Icons.terminal_outlined),
            onPressed: () => setState(() => _showConsole = !_showConsole),
            tooltip: 'Toggle Console',
          ),
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: _showAddInstanceDialog,
            tooltip: 'Add Instance',
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _refreshAllInstances,
            tooltip: 'Refresh All',
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.grid_view), text: 'Grid'),
            Tab(icon: Icon(Icons.list), text: 'List'),
            Tab(icon: Icon(Icons.dashboard), text: 'Detail'),
          ],
        ),
      ),
      body: Consumer<InstanceManagerProvider>(
        builder: (context, provider, child) {
          if (provider.isLoading) {
            return const Center(child: CircularProgressIndicator());
          }

          if (provider.errorMessage != null) {
            return _buildErrorState(provider);
          }

          return Column(
            children: [
              Expanded(
                child: TabBarView(
                  controller: _tabController,
                  children: [
                    _buildGridView(provider),
                    _buildListView(provider),
                    _buildDetailView(provider),
                  ],
                ),
              ),
              if (_showConsole && provider.selectedInstanceId != null)
                SizedBox(
                  height: 250,
                  child: ConsoleTerminal(
                    instanceId: provider.selectedInstanceId!,
                    showHeader: true,
                  ),
                ),
            ],
          );
        },
      ),
      floatingActionButton: _buildFAB(),
    );
  }

  Widget _buildErrorState(InstanceManagerProvider provider) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
          const SizedBox(height: 16),
          Text(
            'Error',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Text(
            provider.errorMessage ?? 'Unknown error',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: _refreshAllInstances,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildGridView(InstanceManagerProvider provider) {
    return const MultiInstanceGrid(
      crossAxisCount: 2,
      childAspectRatio: 1.1,
      showQuickStats: true,
    );
  }

  Widget _buildListView(InstanceManagerProvider provider) {
    return const MultiInstanceList(
      showQuickActions: true,
    );
  }

  Widget _buildDetailView(InstanceManagerProvider provider) {
    final selectedInstance = provider.selectedInstance;

    if (selectedInstance == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.touch_app, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'Select an Instance',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    color: Colors.grey[600],
                  ),
            ),
            const SizedBox(height: 8),
            Text(
              'Choose an instance from the grid or list view\nto see detailed information.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey[500]),
            ),
            if (provider.instances.isNotEmpty) ...[
              const SizedBox(height: 24),
              SizedBox(
                width: 300,
                child: InstanceSelector(
                  onChanged: (id) => provider.selectInstance(id),
                ),
              ),
            ],
          ],
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildInstanceHeader(provider, selectedInstance),
          const SizedBox(height: 16),
          InstanceCard(
            instance: selectedInstance,
            isSelected: true,
            onTap: () {},
          ),
          const SizedBox(height: 16),
          _buildQuickActionsCard(provider, selectedInstance),
          const SizedBox(height: 16),
          if (selectedInstance.connected) ...[
            _buildMetricsCard(provider, selectedInstance),
            const SizedBox(height: 16),
            _buildHealthCard(provider, selectedInstance),
            const SizedBox(height: 16),
          ],
          _buildConnectionInfoCard(selectedInstance),
        ],
      ),
    );
  }

  Widget _buildInstanceHeader(
      InstanceManagerProvider provider, ServerInstance instance) {
    return Row(
      children: [
        Container(
          width: 16,
          height: 16,
          decoration: BoxDecoration(
            color: provider.getStatusColor(instance.status),
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                instance.name,
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              Text(
                instance.host,
                style: TextStyle(color: Colors.grey[600]),
              ),
            ],
          ),
        ),
        _buildEnvironmentChip(instance.environment),
      ],
    );
  }

  Widget _buildEnvironmentChip(InstanceEnvironment env) {
    String label;
    Color color;

    switch (env) {
      case InstanceEnvironment.development:
        label = 'Development';
        color = Colors.green;
        break;
      case InstanceEnvironment.staging:
        label = 'Staging';
        color = Colors.orange;
        break;
      case InstanceEnvironment.production:
        label = 'Production';
        color = Colors.red;
        break;
    }

    return Chip(
      label: Text(label, style: TextStyle(color: color)),
      backgroundColor: color.withOpacity(0.1),
      side: BorderSide(color: color.withOpacity(0.3)),
    );
  }

  Widget _buildQuickActionsCard(
      InstanceManagerProvider provider, ServerInstance instance) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Quick Actions',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            InstanceQuickActions(
              instanceId: instance.id,
              compact: false,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetricsCard(
      InstanceManagerProvider provider, ServerInstance instance) {
    final metrics = instance.metrics;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Performance Metrics',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            if (metrics != null) ...[
              _buildMetricRow('Active Users', '${metrics.activeUsers}',
                  Icons.people, Colors.blue),
              const SizedBox(height: 12),
              _buildMetricRow(
                  'Active Regions',
                  '${metrics.activeRegions}',
                  Icons.map,
                  Colors.green),
              const SizedBox(height: 12),
              _buildMetricProgress(
                  'CPU Usage',
                  metrics.cpuUsage,
                  Icons.speed,
                  _getUsageColor(metrics.cpuUsage)),
              const SizedBox(height: 12),
              _buildMetricProgress(
                  'Memory Usage',
                  metrics.memoryUsagePercent,
                  Icons.memory,
                  _getUsageColor(metrics.memoryUsagePercent)),
            ] else ...[
              Center(
                child: Column(
                  children: [
                    Icon(Icons.analytics_outlined,
                        size: 48, color: Colors.grey[400]),
                    const SizedBox(height: 8),
                    Text(
                      'No metrics available',
                      style: TextStyle(color: Colors.grey[500]),
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

  Color _getUsageColor(double value) {
    if (value >= 80) return Colors.red;
    if (value >= 60) return Colors.orange;
    return Colors.green;
  }

  Widget _buildMetricRow(
      String label, String value, IconData icon, Color color) {
    return Row(
      children: [
        Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: color.withOpacity(0.1),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(icon, color: color, size: 20),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Text(
            label,
            style: const TextStyle(fontWeight: FontWeight.w500),
          ),
        ),
        Text(
          value,
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
  }

  Widget _buildMetricProgress(
      String label, double value, IconData icon, Color color) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, color: color, size: 20),
            const SizedBox(width: 8),
            Expanded(child: Text(label)),
            Text(
              '${value.toStringAsFixed(1)}%',
              style: TextStyle(fontWeight: FontWeight.bold, color: color),
            ),
          ],
        ),
        const SizedBox(height: 8),
        LinearProgressIndicator(
          value: value / 100,
          backgroundColor: Colors.grey[200],
          valueColor: AlwaysStoppedAnimation<Color>(color),
        ),
      ],
    );
  }

  Widget _buildHealthCard(
      InstanceManagerProvider provider, ServerInstance instance) {
    final health = instance.health;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  'Health Status',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                const Spacer(),
                if (health != null)
                  _buildHealthBadge(health.overall, provider),
              ],
            ),
            const SizedBox(height: 16),
            if (health != null) ...[
              ...health.components.entries.map((entry) => Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: _buildComponentHealthRow(
                        entry.key, entry.value, provider),
                  )),
              if (health.lastCheck != null) ...[
                const SizedBox(height: 8),
                Text(
                  'Last check: ${_formatDateTime(health.lastCheck!)}',
                  style: TextStyle(color: Colors.grey[500], fontSize: 12),
                ),
              ],
            ] else ...[
              Center(
                child: Column(
                  children: [
                    Icon(Icons.health_and_safety_outlined,
                        size: 48, color: Colors.grey[400]),
                    const SizedBox(height: 8),
                    Text(
                      'Health data not available',
                      style: TextStyle(color: Colors.grey[500]),
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

  Widget _buildHealthBadge(HealthState state, InstanceManagerProvider provider) {
    final color = provider.getHealthColor(state);
    String label;

    switch (state) {
      case HealthState.healthy:
        label = 'Healthy';
        break;
      case HealthState.degraded:
        label = 'Degraded';
        break;
      case HealthState.unhealthy:
        label = 'Unhealthy';
        break;
      case HealthState.unknown:
        label = 'Unknown';
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w600,
          fontSize: 12,
        ),
      ),
    );
  }

  Widget _buildComponentHealthRow(
      String name, ComponentHealth component, InstanceManagerProvider provider) {
    final color = provider.getHealthColor(component.status);

    return Row(
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(child: Text(name)),
        if (component.responseTimeMs != null)
          Text(
            '${component.responseTimeMs}ms',
            style: TextStyle(color: Colors.grey[500], fontSize: 12),
          ),
      ],
    );
  }

  Widget _buildConnectionInfoCard(ServerInstance instance) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Connection Information',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            _buildInfoRow('Host', instance.host),
            _buildInfoRow('WebSocket Port', '${instance.ports.websocket}'),
            _buildInfoRow('Admin Port', '${instance.ports.admin}'),
            _buildInfoRow('HTTP Port', '${instance.ports.http}'),
            if (instance.version != null)
              _buildInfoRow('Version', instance.version!),
            if (instance.lastSeen != null)
              _buildInfoRow('Last Seen', _formatDateTime(instance.lastSeen!)),
            if (instance.tags.isNotEmpty)
              _buildInfoRow('Tags', instance.tags.join(', ')),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: TextStyle(
                color: Colors.grey[600],
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(fontWeight: FontWeight.w500),
            ),
          ),
        ],
      ),
    );
  }

  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}-${dateTime.month.toString().padLeft(2, '0')}-'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}:'
        '${dateTime.second.toString().padLeft(2, '0')}';
  }

  Widget? _buildFAB() {
    final provider = context.watch<InstanceManagerProvider>();
    if (provider.instances.isEmpty) return null;

    return FloatingActionButton.extended(
      onPressed: _showBatchActionsDialog,
      icon: const Icon(Icons.layers),
      label: const Text('Batch Actions'),
    );
  }

  void _refreshAllInstances() {
    final provider =
        Provider.of<InstanceManagerProvider>(context, listen: false);
    provider.fetchInstanceDirectories();
    for (final instance in provider.instances) {
      if (instance.connected) {
        provider.sendCommand(instance.id, InstanceCommand.getStatus);
      }
    }
  }

  void _showAddInstanceDialog() {
    final nameController = TextEditingController();
    final hostController = TextEditingController();
    final apiKeyController = TextEditingController();
    InstanceEnvironment environment = InstanceEnvironment.development;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('Add New Instance'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: nameController,
                  decoration: const InputDecoration(
                    labelText: 'Instance Name',
                    hintText: 'My Grid Server',
                    prefixIcon: Icon(Icons.label),
                  ),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: hostController,
                  decoration: const InputDecoration(
                    labelText: 'Host',
                    hintText: 'grid.example.com',
                    prefixIcon: Icon(Icons.dns),
                  ),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: apiKeyController,
                  decoration: const InputDecoration(
                    labelText: 'API Key',
                    hintText: 'Enter API key',
                    prefixIcon: Icon(Icons.key),
                  ),
                  obscureText: true,
                ),
                const SizedBox(height: 16),
                DropdownButtonFormField<InstanceEnvironment>(
                  value: environment,
                  decoration: const InputDecoration(
                    labelText: 'Environment',
                    prefixIcon: Icon(Icons.settings),
                  ),
                  items: InstanceEnvironment.values.map((env) {
                    String label;
                    switch (env) {
                      case InstanceEnvironment.development:
                        label = 'Development';
                        break;
                      case InstanceEnvironment.staging:
                        label = 'Staging';
                        break;
                      case InstanceEnvironment.production:
                        label = 'Production';
                        break;
                    }
                    return DropdownMenuItem(value: env, child: Text(label));
                  }).toList(),
                  onChanged: (value) {
                    if (value != null) {
                      setDialogState(() => environment = value);
                    }
                  },
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () {
                if (nameController.text.isNotEmpty &&
                    hostController.text.isNotEmpty) {
                  final instance = ServerInstance(
                    id: DateTime.now().millisecondsSinceEpoch.toString(),
                    name: nameController.text,
                    host: hostController.text,
                    ports: InstancePorts(),
                    environment: environment,
                    apiKey: apiKeyController.text,
                    autoConnect: true,
                  );
                  Provider.of<InstanceManagerProvider>(context, listen: false)
                      .addInstance(instance);
                  Navigator.of(context).pop();
                }
              },
              child: const Text('Add'),
            ),
          ],
        ),
      ),
    );
  }

  void _showBatchActionsDialog() {
    showDialog(
      context: context,
      builder: (context) => Consumer<InstanceManagerProvider>(
        builder: (context, provider, child) => AlertDialog(
          title: const Text('Batch Actions'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Apply action to ${provider.connectedCount} connected instances',
                style: TextStyle(color: Colors.grey[600]),
              ),
              const SizedBox(height: 24),
              _buildBatchActionButton(
                'Start All',
                Icons.play_arrow,
                Colors.green,
                () => _executeBatchAction(InstanceCommand.start),
              ),
              const SizedBox(height: 8),
              _buildBatchActionButton(
                'Stop All',
                Icons.stop,
                Colors.red,
                () => _executeBatchAction(InstanceCommand.stop),
              ),
              const SizedBox(height: 8),
              _buildBatchActionButton(
                'Restart All',
                Icons.refresh,
                Colors.orange,
                () => _executeBatchAction(InstanceCommand.restart),
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Close'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBatchActionButton(
      String label, IconData icon, Color color, VoidCallback onPressed) {
    return SizedBox(
      width: double.infinity,
      child: OutlinedButton.icon(
        onPressed: () {
          Navigator.of(context).pop();
          onPressed();
        },
        icon: Icon(icon, color: color),
        label: Text(label, style: TextStyle(color: color)),
        style: OutlinedButton.styleFrom(
          side: BorderSide(color: color.withOpacity(0.5)),
          padding: const EdgeInsets.all(12),
        ),
      ),
    );
  }

  void _executeBatchAction(InstanceCommand command) async {
    final provider =
        Provider.of<InstanceManagerProvider>(context, listen: false);
    final results = await provider.broadcastCommand(command);

    if (!mounted) return;

    final successCount = results.where((r) => r.status == 'success').length;
    final failCount = results.length - successCount;

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'Batch action completed: $successCount succeeded, $failCount failed',
        ),
        backgroundColor: failCount == 0 ? Colors.green : Colors.orange,
      ),
    );
  }
}
