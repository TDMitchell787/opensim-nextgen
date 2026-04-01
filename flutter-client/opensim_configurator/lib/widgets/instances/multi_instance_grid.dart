import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/instance_models.dart';
import '../../providers/instance_manager_provider.dart';
import 'instance_card.dart';

class MultiInstanceGrid extends StatelessWidget {
  final int crossAxisCount;
  final double childAspectRatio;
  final bool showQuickStats;

  const MultiInstanceGrid({
    super.key,
    this.crossAxisCount = 2,
    this.childAspectRatio = 1.2,
    this.showQuickStats = true,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instances = provider.instances;

    return Column(
      children: [
        if (showQuickStats) ...[
          _buildQuickStats(context, provider),
          const SizedBox(height: 16),
        ],
        if (instances.isEmpty)
          _buildEmptyState(context)
        else
          Expanded(
            child: GridView.builder(
              padding: const EdgeInsets.all(16),
              gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: crossAxisCount,
                childAspectRatio: childAspectRatio,
                crossAxisSpacing: 16,
                mainAxisSpacing: 16,
              ),
              itemCount: instances.length,
              itemBuilder: (context, index) {
                final instance = instances[index];
                return InstanceCard(
                  instance: instance,
                  isSelected: instance.id == provider.selectedInstanceId,
                  onTap: () => provider.selectInstance(instance.id),
                );
              },
            ),
          ),
      ],
    );
  }

  Widget _buildQuickStats(BuildContext context, InstanceManagerProvider provider) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(12),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.05),
            blurRadius: 10,
            offset: const Offset(0, 2),
          ),
        ],
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildStatItem(
            context,
            Icons.dns,
            '${provider.totalInstances}',
            'Total',
            Colors.blue,
          ),
          _buildStatDivider(),
          _buildStatItem(
            context,
            Icons.link,
            '${provider.connectedCount}',
            'Connected',
            Colors.green,
          ),
          _buildStatDivider(),
          _buildStatItem(
            context,
            Icons.check_circle,
            '${provider.healthyCount}',
            'Healthy',
            const Color(0xFF10B981),
          ),
          _buildStatDivider(),
          _buildStatItem(
            context,
            Icons.warning,
            '${provider.needsAttentionCount}',
            'Attention',
            Colors.orange,
          ),
        ],
      ),
    );
  }

  Widget _buildStatItem(
    BuildContext context,
    IconData icon,
    String value,
    String label,
    Color color,
  ) {
    return Column(
      children: [
        Icon(icon, color: color, size: 24),
        const SizedBox(height: 4),
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

  Widget _buildStatDivider() {
    return Container(
      height: 40,
      width: 1,
      color: Colors.grey[300],
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Expanded(
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.dns_outlined,
              size: 64,
              color: Colors.grey[400],
            ),
            const SizedBox(height: 16),
            Text(
              'No Instances Configured',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Add instances to your instances.toml configuration\nor connect to a new instance.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey[500]),
            ),
            const SizedBox(height: 24),
            ElevatedButton.icon(
              onPressed: () {},
              icon: const Icon(Icons.add),
              label: const Text('Add Instance'),
            ),
          ],
        ),
      ),
    );
  }
}

class MultiInstanceList extends StatelessWidget {
  final bool showQuickActions;

  const MultiInstanceList({
    super.key,
    this.showQuickActions = true,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instances = provider.instances;

    if (instances.isEmpty) {
      return const Center(
        child: Text('No instances configured'),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: instances.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, index) {
        final instance = instances[index];
        final isSelected = instance.id == provider.selectedInstanceId;
        final statusColor = provider.getStatusColor(instance.status);

        return Card(
          elevation: isSelected ? 2 : 0,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
            side: BorderSide(
              color: isSelected
                  ? Theme.of(context).colorScheme.primary
                  : Colors.grey[300]!,
            ),
          ),
          child: ListTile(
            leading: Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: statusColor.withOpacity(0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Icon(Icons.dns, color: statusColor),
            ),
            title: Text(instance.name),
            subtitle: Text(
              '${instance.host} - ${InstanceStatusInfo.getInfo(instance.status).label}',
            ),
            trailing: showQuickActions
                ? _buildQuickActions(context, provider, instance)
                : Icon(
                    instance.connected ? Icons.link : Icons.link_off,
                    color: instance.connected ? Colors.green : Colors.grey,
                  ),
            onTap: () => provider.selectInstance(instance.id),
          ),
        );
      },
    );
  }

  Widget _buildQuickActions(
    BuildContext context,
    InstanceManagerProvider provider,
    ServerInstance instance,
  ) {
    final statusInfo = InstanceStatusInfo.getInfo(instance.status);

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (!instance.connected)
          IconButton(
            icon: const Icon(Icons.link),
            onPressed: () => provider.connectToInstance(instance.id),
            tooltip: 'Connect',
            iconSize: 20,
          )
        else ...[
          if (statusInfo.canStart)
            IconButton(
              icon: const Icon(Icons.play_arrow, color: Colors.green),
              onPressed: () => provider.sendCommand(instance.id, InstanceCommand.start),
              tooltip: 'Start',
              iconSize: 20,
            ),
          if (statusInfo.canStop)
            IconButton(
              icon: const Icon(Icons.stop, color: Colors.red),
              onPressed: () => provider.sendCommand(instance.id, InstanceCommand.stop),
              tooltip: 'Stop',
              iconSize: 20,
            ),
          if (statusInfo.canRestart)
            IconButton(
              icon: const Icon(Icons.refresh, color: Colors.orange),
              onPressed: () => provider.sendCommand(instance.id, InstanceCommand.restart),
              tooltip: 'Restart',
              iconSize: 20,
            ),
        ],
      ],
    );
  }
}

class InstanceQuickActions extends StatelessWidget {
  final String instanceId;
  final bool compact;

  const InstanceQuickActions({
    super.key,
    required this.instanceId,
    this.compact = false,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instance = provider.instances.firstWhere(
      (i) => i.id == instanceId,
      orElse: () => throw Exception('Instance not found'),
    );
    final statusInfo = InstanceStatusInfo.getInfo(instance.status);

    if (!instance.connected) {
      return ElevatedButton.icon(
        onPressed: () => provider.connectToInstance(instanceId),
        icon: const Icon(Icons.link),
        label: const Text('Connect'),
      );
    }

    if (compact) {
      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (statusInfo.canStart)
            IconButton(
              icon: const Icon(Icons.play_arrow),
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.start),
              tooltip: 'Start',
              color: Colors.green,
            ),
          if (statusInfo.canStop)
            IconButton(
              icon: const Icon(Icons.stop),
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.stop),
              tooltip: 'Stop',
              color: Colors.red,
            ),
          if (statusInfo.canRestart)
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.restart),
              tooltip: 'Restart',
              color: Colors.orange,
            ),
        ],
      );
    }

    return Row(
      children: [
        if (statusInfo.canStart)
          Expanded(
            child: ElevatedButton.icon(
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.start),
              icon: const Icon(Icons.play_arrow),
              label: const Text('Start'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.green,
                foregroundColor: Colors.white,
              ),
            ),
          ),
        if (statusInfo.canStop) ...[
          if (statusInfo.canStart) const SizedBox(width: 8),
          Expanded(
            child: ElevatedButton.icon(
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.stop),
              icon: const Icon(Icons.stop),
              label: const Text('Stop'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.red,
                foregroundColor: Colors.white,
              ),
            ),
          ),
        ],
        if (statusInfo.canRestart) ...[
          const SizedBox(width: 8),
          Expanded(
            child: ElevatedButton.icon(
              onPressed: () => provider.sendCommand(instanceId, InstanceCommand.restart),
              icon: const Icon(Icons.refresh),
              label: const Text('Restart'),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.orange,
                foregroundColor: Colors.white,
              ),
            ),
          ),
        ],
      ],
    );
  }
}
