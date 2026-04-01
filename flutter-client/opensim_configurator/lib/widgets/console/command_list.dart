import 'package:flutter/material.dart';
import '../../models/console_command_models.dart';

class CommandList extends StatelessWidget {
  final List<ConsoleCommand> commands;
  final Function(ConsoleCommand) onCommandTap;
  final String? filterText;

  const CommandList({
    super.key,
    required this.commands,
    required this.onCommandTap,
    this.filterText,
  });

  @override
  Widget build(BuildContext context) {
    if (commands.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search_off, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              filterText != null && filterText!.isNotEmpty
                  ? 'No commands match "$filterText"'
                  : 'No commands in this group',
              style: TextStyle(color: Colors.grey[600]),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: commands.length,
      itemBuilder: (context, index) {
        final command = commands[index];
        return CommandListItem(
          command: command,
          onTap: () => onCommandTap(command),
        );
      },
    );
  }
}

class CommandListItem extends StatelessWidget {
  final ConsoleCommand command;
  final VoidCallback onTap;

  const CommandListItem({
    super.key,
    required this.command,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildStatusIcon(),
              const SizedBox(width: 12),
              Expanded(child: _buildContent(context)),
              if (command.params.isNotEmpty) _buildParamsChip(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusIcon() {
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: Icon(
        command.implemented ? Icons.check_circle : Icons.pending,
        color: command.implemented ? Colors.green : Colors.orange,
        size: 20,
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          command.name,
          style: const TextStyle(
            fontWeight: FontWeight.w600,
            fontSize: 15,
          ),
        ),
        const SizedBox(height: 4),
        Text(
          command.description,
          style: TextStyle(
            color: Colors.grey[700],
            fontSize: 13,
          ),
        ),
        const SizedBox(height: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: Colors.grey[100],
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(
            command.syntax,
            style: TextStyle(
              fontFamily: 'monospace',
              fontSize: 11,
              color: Colors.grey[700],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildParamsChip() {
    return Container(
      margin: const EdgeInsets.only(left: 8),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.blue[50],
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.blue[200]!),
      ),
      child: Text(
        '${command.params.length} param${command.params.length == 1 ? '' : 's'}',
        style: TextStyle(
          fontSize: 10,
          color: Colors.blue[700],
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }
}

class CommandListCompact extends StatelessWidget {
  final List<ConsoleCommand> commands;
  final Function(ConsoleCommand) onCommandTap;

  const CommandListCompact({
    super.key,
    required this.commands,
    required this.onCommandTap,
  });

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: commands.map((command) {
        return ActionChip(
          avatar: Icon(
            command.implemented ? Icons.check_circle : Icons.pending,
            size: 16,
            color: command.implemented ? Colors.green : Colors.orange,
          ),
          label: Text(command.name),
          onPressed: () => onCommandTap(command),
        );
      }).toList(),
    );
  }
}
