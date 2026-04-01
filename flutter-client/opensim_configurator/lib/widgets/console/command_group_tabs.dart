import 'package:flutter/material.dart';
import '../../models/console_command_models.dart';

class CommandGroupTabs extends StatelessWidget {
  final TabController controller;
  final Map<CommandGroup, List<ConsoleCommand>> commandsByGroup;
  final bool showImplementedCount;

  const CommandGroupTabs({
    super.key,
    required this.controller,
    required this.commandsByGroup,
    this.showImplementedCount = true,
  });

  @override
  Widget build(BuildContext context) {
    return TabBar(
      controller: controller,
      isScrollable: true,
      tabs: CommandGroup.values.map((group) {
        final commands = commandsByGroup[group] ?? [];
        return _CommandGroupTab(
          group: group,
          totalCount: commands.length,
          implementedCount: commands.where((c) => c.implemented).length,
          showImplementedCount: showImplementedCount,
        );
      }).toList(),
    );
  }
}

class _CommandGroupTab extends StatelessWidget {
  final CommandGroup group;
  final int totalCount;
  final int implementedCount;
  final bool showImplementedCount;

  const _CommandGroupTab({
    required this.group,
    required this.totalCount,
    required this.implementedCount,
    required this.showImplementedCount,
  });

  @override
  Widget build(BuildContext context) {
    final isComplete = implementedCount == totalCount && totalCount > 0;

    return Tab(
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(_getGroupLabel(group)),
          if (showImplementedCount) ...[
            const SizedBox(width: 6),
            _buildCountBadge(isComplete),
          ],
        ],
      ),
    );
  }

  Widget _buildCountBadge(bool isComplete) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: isComplete ? Colors.green[100] : Colors.grey[200],
        borderRadius: BorderRadius.circular(10),
      ),
      child: Text(
        '$implementedCount/$totalCount',
        style: TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.w500,
          color: isComplete ? Colors.green[800] : Colors.grey[700],
        ),
      ),
    );
  }

  String _getGroupLabel(CommandGroup group) {
    const labels = {
      CommandGroup.users: 'Users',
      CommandGroup.regions: 'Regions',
      CommandGroup.terrain: 'Terrain',
      CommandGroup.objects: 'Objects',
      CommandGroup.estates: 'Estates',
      CommandGroup.archiving: 'Archiving',
      CommandGroup.assets: 'Assets',
      CommandGroup.comms: 'Comms',
      CommandGroup.hypergrid: 'Hypergrid',
      CommandGroup.general: 'General',
      CommandGroup.database: 'Database',
    };
    return labels[group] ?? group.name;
  }
}

class CommandGroupSelector extends StatelessWidget {
  final CommandGroup? selectedGroup;
  final Map<CommandGroup, List<ConsoleCommand>> commandsByGroup;
  final ValueChanged<CommandGroup?> onGroupChanged;

  const CommandGroupSelector({
    super.key,
    required this.selectedGroup,
    required this.commandsByGroup,
    required this.onGroupChanged,
  });

  @override
  Widget build(BuildContext context) {
    return DropdownButton<CommandGroup>(
      value: selectedGroup,
      hint: const Text('Select Group'),
      isExpanded: true,
      items: CommandGroup.values.map((group) {
        final commands = commandsByGroup[group] ?? [];
        final implementedCount = commands.where((c) => c.implemented).length;

        return DropdownMenuItem(
          value: group,
          child: Row(
            children: [
              Expanded(child: Text(_getGroupLabel(group))),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: implementedCount == commands.length && commands.isNotEmpty
                      ? Colors.green[100]
                      : Colors.grey[200],
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(
                  '$implementedCount/${commands.length}',
                  style: TextStyle(
                    fontSize: 10,
                    color: implementedCount == commands.length && commands.isNotEmpty
                        ? Colors.green[800]
                        : Colors.grey[700],
                  ),
                ),
              ),
            ],
          ),
        );
      }).toList(),
      onChanged: onGroupChanged,
    );
  }

  String _getGroupLabel(CommandGroup group) {
    const labels = {
      CommandGroup.users: 'Users',
      CommandGroup.regions: 'Regions',
      CommandGroup.terrain: 'Terrain',
      CommandGroup.objects: 'Objects',
      CommandGroup.estates: 'Estates',
      CommandGroup.archiving: 'Archiving',
      CommandGroup.assets: 'Assets',
      CommandGroup.comms: 'Comms',
      CommandGroup.hypergrid: 'Hypergrid',
      CommandGroup.general: 'General',
      CommandGroup.database: 'Database',
    };
    return labels[group] ?? group.name;
  }
}

class CommandGroupChips extends StatelessWidget {
  final CommandGroup? selectedGroup;
  final Map<CommandGroup, List<ConsoleCommand>> commandsByGroup;
  final ValueChanged<CommandGroup?> onGroupChanged;

  const CommandGroupChips({
    super.key,
    required this.selectedGroup,
    required this.commandsByGroup,
    required this.onGroupChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: CommandGroup.values.map((group) {
        final commands = commandsByGroup[group] ?? [];
        final implementedCount = commands.where((c) => c.implemented).length;
        final isSelected = selectedGroup == group;
        final isComplete = implementedCount == commands.length && commands.isNotEmpty;

        return FilterChip(
          selected: isSelected,
          label: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(_getGroupLabel(group)),
              const SizedBox(width: 4),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                decoration: BoxDecoration(
                  color: isComplete
                      ? Colors.green[isSelected ? 200 : 100]
                      : Colors.grey[isSelected ? 300 : 200],
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(
                  '$implementedCount/${commands.length}',
                  style: TextStyle(
                    fontSize: 9,
                    color: isComplete ? Colors.green[800] : Colors.grey[700],
                  ),
                ),
              ),
            ],
          ),
          onSelected: (selected) {
            onGroupChanged(selected ? group : null);
          },
        );
      }).toList(),
    );
  }

  String _getGroupLabel(CommandGroup group) {
    const labels = {
      CommandGroup.users: 'Users',
      CommandGroup.regions: 'Regions',
      CommandGroup.terrain: 'Terrain',
      CommandGroup.objects: 'Objects',
      CommandGroup.estates: 'Estates',
      CommandGroup.archiving: 'Archiving',
      CommandGroup.assets: 'Assets',
      CommandGroup.comms: 'Comms',
      CommandGroup.hypergrid: 'Hypergrid',
      CommandGroup.general: 'General',
      CommandGroup.database: 'Database',
    };
    return labels[group] ?? group.name;
  }
}
