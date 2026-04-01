import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/instance_models.dart';
import '../../providers/instance_manager_provider.dart';

class InstanceSelector extends StatelessWidget {
  final String? selectedInstanceId;
  final ValueChanged<String?>? onChanged;
  final bool showStatus;
  final bool showConnectedOnly;

  const InstanceSelector({
    super.key,
    this.selectedInstanceId,
    this.onChanged,
    this.showStatus = true,
    this.showConnectedOnly = false,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instances = showConnectedOnly
        ? provider.connectedInstances
        : provider.instances;

    if (instances.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          border: Border.all(color: Colors.grey[300]!),
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Center(
          child: Text(
            'No instances available',
            style: TextStyle(color: Colors.grey),
          ),
        ),
      );
    }

    return DropdownButtonFormField<String>(
      value: selectedInstanceId ?? provider.selectedInstanceId,
      decoration: InputDecoration(
        labelText: 'Select Instance',
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
        ),
        prefixIcon: const Icon(Icons.dns),
      ),
      items: instances.map((instance) {
        return DropdownMenuItem<String>(
          value: instance.id,
          child: _buildInstanceItem(context, instance, provider),
        );
      }).toList(),
      onChanged: (value) {
        if (onChanged != null) {
          onChanged!(value);
        } else {
          provider.selectInstance(value);
        }
      },
      selectedItemBuilder: (context) {
        return instances.map((instance) {
          return Row(
            children: [
              Container(
                width: 8,
                height: 8,
                decoration: BoxDecoration(
                  color: provider.getStatusColor(instance.status),
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 8),
              Text(instance.name),
            ],
          );
        }).toList();
      },
    );
  }

  Widget _buildInstanceItem(
    BuildContext context,
    ServerInstance instance,
    InstanceManagerProvider provider,
  ) {
    final statusColor = provider.getStatusColor(instance.status);

    return Row(
      children: [
        Container(
          width: 10,
          height: 10,
          decoration: BoxDecoration(
            color: statusColor,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                instance.name,
                style: const TextStyle(fontWeight: FontWeight.w500),
              ),
              if (showStatus)
                Text(
                  '${instance.host} - ${InstanceStatusInfo.getInfo(instance.status).label}',
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.grey[600],
                  ),
                ),
            ],
          ),
        ),
        if (instance.connected)
          Icon(Icons.link, color: Colors.green[400], size: 16),
      ],
    );
  }
}

class InstanceSelectorChips extends StatelessWidget {
  final String? selectedInstanceId;
  final ValueChanged<String?>? onChanged;
  final bool showConnectedOnly;

  const InstanceSelectorChips({
    super.key,
    this.selectedInstanceId,
    this.onChanged,
    this.showConnectedOnly = false,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instances = showConnectedOnly
        ? provider.connectedInstances
        : provider.instances;

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: instances.map((instance) {
          final isSelected = instance.id == (selectedInstanceId ?? provider.selectedInstanceId);
          final statusColor = provider.getStatusColor(instance.status);

          return Padding(
            padding: const EdgeInsets.only(right: 8),
            child: FilterChip(
              avatar: Container(
                width: 8,
                height: 8,
                decoration: BoxDecoration(
                  color: statusColor,
                  shape: BoxShape.circle,
                ),
              ),
              label: Text(instance.name),
              selected: isSelected,
              onSelected: (_) {
                if (onChanged != null) {
                  onChanged!(instance.id);
                } else {
                  provider.selectInstance(instance.id);
                }
              },
            ),
          );
        }).toList(),
      ),
    );
  }
}

class InstanceSelectorList extends StatelessWidget {
  final String? selectedInstanceId;
  final ValueChanged<String?>? onChanged;
  final bool showConnectedOnly;

  const InstanceSelectorList({
    super.key,
    this.selectedInstanceId,
    this.onChanged,
    this.showConnectedOnly = false,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instances = showConnectedOnly
        ? provider.connectedInstances
        : provider.instances;

    if (instances.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.dns_outlined, size: 48, color: Colors.grey),
              SizedBox(height: 16),
              Text(
                'No instances configured',
                style: TextStyle(color: Colors.grey),
              ),
            ],
          ),
        ),
      );
    }

    return ListView.separated(
      shrinkWrap: true,
      itemCount: instances.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final instance = instances[index];
        final isSelected = instance.id == (selectedInstanceId ?? provider.selectedInstanceId);
        final statusColor = provider.getStatusColor(instance.status);

        return ListTile(
          selected: isSelected,
          leading: Container(
            width: 40,
            height: 40,
            decoration: BoxDecoration(
              color: statusColor.withOpacity(0.1),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Icon(
              instance.connected ? Icons.dns : Icons.dns_outlined,
              color: statusColor,
            ),
          ),
          title: Text(instance.name),
          subtitle: Text(instance.host),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildEnvironmentBadge(instance.environment),
              const SizedBox(width: 8),
              Icon(
                instance.connected ? Icons.link : Icons.link_off,
                color: instance.connected ? Colors.green : Colors.grey,
                size: 20,
              ),
            ],
          ),
          onTap: () {
            if (onChanged != null) {
              onChanged!(instance.id);
            } else {
              provider.selectInstance(instance.id);
            }
          },
        );
      },
    );
  }

  Widget _buildEnvironmentBadge(InstanceEnvironment env) {
    String label;
    Color color;

    switch (env) {
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
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
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
}
