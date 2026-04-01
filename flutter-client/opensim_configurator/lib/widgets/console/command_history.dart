import 'package:flutter/material.dart';
import '../../models/console_command_models.dart';

class CommandHistoryDialog extends StatelessWidget {
  final List<CommandExecution> executionHistory;
  final VoidCallback onClear;

  const CommandHistoryDialog({
    super.key,
    required this.executionHistory,
    required this.onClear,
  });

  static Future<void> show(
    BuildContext context,
    List<CommandExecution> executionHistory,
    VoidCallback onClear,
  ) {
    return showDialog(
      context: context,
      builder: (context) => CommandHistoryDialog(
        executionHistory: executionHistory,
        onClear: onClear,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Row(
        children: [
          const Icon(Icons.history),
          const SizedBox(width: 8),
          const Text('Command History'),
          const Spacer(),
          Text(
            '${executionHistory.length} commands',
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.normal,
              color: Colors.grey[600],
            ),
          ),
        ],
      ),
      content: SizedBox(
        width: 600,
        height: 400,
        child: executionHistory.isEmpty
            ? _buildEmptyState()
            : _buildHistoryList(),
      ),
      actions: [
        if (executionHistory.isNotEmpty)
          TextButton.icon(
            onPressed: () {
              onClear();
              Navigator.pop(context);
            },
            icon: const Icon(Icons.delete_sweep, size: 18),
            label: const Text('Clear History'),
          ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.history, size: 64, color: Colors.grey[400]),
          const SizedBox(height: 16),
          Text(
            'No commands executed yet',
            style: TextStyle(color: Colors.grey[600], fontSize: 16),
          ),
          const SizedBox(height: 8),
          Text(
            'Commands you execute will appear here',
            style: TextStyle(color: Colors.grey[500], fontSize: 12),
          ),
        ],
      ),
    );
  }

  Widget _buildHistoryList() {
    return ListView.builder(
      itemCount: executionHistory.length,
      itemBuilder: (context, index) {
        final exec = executionHistory[index];
        return CommandHistoryItem(execution: exec);
      },
    );
  }
}

class CommandHistoryItem extends StatelessWidget {
  final CommandExecution execution;

  const CommandHistoryItem({
    super.key,
    required this.execution,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ExpansionTile(
        leading: _buildStatusIcon(),
        title: Text(
          execution.command,
          style: const TextStyle(fontWeight: FontWeight.w500),
        ),
        subtitle: Text(
          execution.result?.message ?? 'Executing...',
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
        trailing: Text(
          _formatTime(execution.executedAt),
          style: TextStyle(color: Colors.grey[600], fontSize: 12),
        ),
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (execution.params.isNotEmpty) ...[
                  Text(
                    'Parameters:',
                    style: TextStyle(
                      fontWeight: FontWeight.w500,
                      color: Colors.grey[700],
                      fontSize: 12,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Colors.grey[100],
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      execution.params.entries
                          .map((e) => '${e.key}: ${e.value}')
                          .join('\n'),
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 11,
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                ],
                if (execution.result != null) ...[
                  Text(
                    'Result:',
                    style: TextStyle(
                      fontWeight: FontWeight.w500,
                      color: Colors.grey[700],
                      fontSize: 12,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: execution.result!.success
                          ? Colors.green[50]
                          : Colors.red[50],
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(
                        color: execution.result!.success
                            ? Colors.green[200]!
                            : Colors.red[200]!,
                      ),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          execution.result!.message,
                          style: const TextStyle(fontSize: 12),
                        ),
                        if (execution.result!.error != null) ...[
                          const SizedBox(height: 4),
                          Text(
                            'Error: ${execution.result!.error}',
                            style: const TextStyle(
                              color: Colors.red,
                              fontSize: 11,
                            ),
                          ),
                        ],
                        if (execution.result!.data != null) ...[
                          const SizedBox(height: 4),
                          Text(
                            execution.result!.data.toString(),
                            style: const TextStyle(
                              fontFamily: 'monospace',
                              fontSize: 10,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIcon() {
    if (execution.result == null) {
      return const SizedBox(
        width: 20,
        height: 20,
        child: CircularProgressIndicator(strokeWidth: 2),
      );
    }

    return Icon(
      execution.result!.success ? Icons.check_circle : Icons.error,
      color: execution.result!.success ? Colors.green : Colors.red,
      size: 20,
    );
  }

  String _formatTime(DateTime time) {
    return '${time.hour.toString().padLeft(2, '0')}:'
        '${time.minute.toString().padLeft(2, '0')}:'
        '${time.second.toString().padLeft(2, '0')}';
  }
}

class CommandHistoryBadge extends StatelessWidget {
  final int count;
  final VoidCallback onTap;

  const CommandHistoryBadge({
    super.key,
    required this.count,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Badge(
        label: Text('$count'),
        isLabelVisible: count > 0,
        child: const Icon(Icons.history),
      ),
      onPressed: onTap,
      tooltip: 'Command History ($count)',
    );
  }
}
