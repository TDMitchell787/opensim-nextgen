import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/instance_models.dart';
import '../../providers/instance_manager_provider.dart';

class InstanceCard extends StatelessWidget {
  final ServerInstance instance;
  final bool isSelected;
  final VoidCallback? onTap;
  final VoidCallback? onConnect;
  final VoidCallback? onDisconnect;

  const InstanceCard({
    super.key,
    required this.instance,
    this.isSelected = false,
    this.onTap,
    this.onConnect,
    this.onDisconnect,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final statusInfo = InstanceStatusInfo.getInfo(instance.status);
    final statusColor = provider.getStatusColor(instance.status);
    final healthColor = instance.health != null
        ? provider.getHealthColor(instance.health!.overall)
        : Colors.grey;

    return Card(
      elevation: isSelected ? 4 : 1,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: isSelected ? Theme.of(context).colorScheme.primary : Colors.transparent,
          width: 2,
        ),
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 12,
                    height: 12,
                    decoration: BoxDecoration(
                      color: statusColor,
                      shape: BoxShape.circle,
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      instance.name,
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  _buildEnvironmentBadge(context),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                instance.host,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Colors.grey[600],
                ),
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  _buildStatusChip(statusInfo, statusColor),
                  const SizedBox(width: 8),
                  if (instance.health != null)
                    _buildHealthChip(context, instance.health!, healthColor),
                ],
              ),
              if (instance.metrics != null) ...[
                const SizedBox(height: 12),
                _buildMetricsRow(context, instance.metrics!),
              ],
              const SizedBox(height: 12),
              _buildActionButtons(context, statusInfo),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildEnvironmentBadge(BuildContext context) {
    String label;
    Color color;

    switch (instance.environment) {
      case InstanceEnvironment.development:
        label = 'DEV';
        color = Colors.green;
        break;
      case InstanceEnvironment.staging:
        label = 'STG';
        color = Colors.orange;
        break;
      case InstanceEnvironment.production:
        label = 'PROD';
        color = Colors.red;
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withOpacity(0.5)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 10,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  Widget _buildStatusChip(InstanceStatusInfo statusInfo, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(statusInfo.icon, style: const TextStyle(fontSize: 12)),
          const SizedBox(width: 4),
          Text(
            statusInfo.label,
            style: TextStyle(
              color: color,
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHealthChip(BuildContext context, HealthStatus health, Color color) {
    String label;
    switch (health.overall) {
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
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }

  Widget _buildMetricsRow(BuildContext context, InstanceMetrics metrics) {
    return Row(
      children: [
        _buildMetricItem(
          context,
          Icons.people,
          '${metrics.activeUsers}',
          'Users',
        ),
        const SizedBox(width: 16),
        _buildMetricItem(
          context,
          Icons.memory,
          '${metrics.memoryUsagePercent.toStringAsFixed(0)}%',
          'Memory',
        ),
        const SizedBox(width: 16),
        _buildMetricItem(
          context,
          Icons.speed,
          '${metrics.cpuUsage.toStringAsFixed(0)}%',
          'CPU',
        ),
      ],
    );
  }

  Widget _buildMetricItem(BuildContext context, IconData icon, String value, String label) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(icon, size: 14, color: Colors.grey[600]),
        const SizedBox(width: 4),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              value,
              style: const TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.bold,
              ),
            ),
            Text(
              label,
              style: TextStyle(
                fontSize: 10,
                color: Colors.grey[500],
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildActionButtons(BuildContext context, InstanceStatusInfo statusInfo) {
    final provider = context.read<InstanceManagerProvider>();

    if (!instance.connected) {
      return SizedBox(
        width: double.infinity,
        child: ElevatedButton.icon(
          onPressed: onConnect ?? () => provider.connectToInstance(instance.id),
          icon: const Icon(Icons.link, size: 18),
          label: const Text('Connect'),
          style: ElevatedButton.styleFrom(
            backgroundColor: Theme.of(context).colorScheme.primary,
            foregroundColor: Colors.white,
          ),
        ),
      );
    }

    return Row(
      children: [
        if (statusInfo.canStart)
          Expanded(
            child: _buildActionButton(
              context,
              Icons.play_arrow,
              'Start',
              Colors.green,
              () => provider.sendCommand(instance.id, InstanceCommand.start),
            ),
          ),
        if (statusInfo.canStop) ...[
          if (statusInfo.canStart) const SizedBox(width: 8),
          Expanded(
            child: _buildActionButton(
              context,
              Icons.stop,
              'Stop',
              Colors.red,
              () => provider.sendCommand(instance.id, InstanceCommand.stop),
            ),
          ),
        ],
        if (statusInfo.canRestart) ...[
          const SizedBox(width: 8),
          Expanded(
            child: _buildActionButton(
              context,
              Icons.refresh,
              'Restart',
              Colors.orange,
              () => provider.sendCommand(instance.id, InstanceCommand.restart),
            ),
          ),
        ],
        const SizedBox(width: 8),
        IconButton(
          onPressed: onDisconnect ?? () => provider.disconnectFromInstance(instance.id),
          icon: const Icon(Icons.link_off),
          tooltip: 'Disconnect',
          iconSize: 20,
        ),
      ],
    );
  }

  Widget _buildActionButton(
    BuildContext context,
    IconData icon,
    String label,
    Color color,
    VoidCallback onPressed,
  ) {
    return OutlinedButton.icon(
      onPressed: onPressed,
      icon: Icon(icon, size: 16, color: color),
      label: Text(label, style: TextStyle(color: color)),
      style: OutlinedButton.styleFrom(
        side: BorderSide(color: color.withOpacity(0.5)),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      ),
    );
  }
}

class InstanceCardCompact extends StatelessWidget {
  final ServerInstance instance;
  final bool isSelected;
  final VoidCallback? onTap;

  const InstanceCardCompact({
    super.key,
    required this.instance,
    this.isSelected = false,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final statusColor = provider.getStatusColor(instance.status);

    return ListTile(
      selected: isSelected,
      leading: Container(
        width: 12,
        height: 12,
        decoration: BoxDecoration(
          color: statusColor,
          shape: BoxShape.circle,
        ),
      ),
      title: Text(instance.name),
      subtitle: Text(instance.host),
      trailing: instance.connected
          ? const Icon(Icons.link, color: Colors.green, size: 20)
          : const Icon(Icons.link_off, color: Colors.grey, size: 20),
      onTap: onTap,
    );
  }
}
