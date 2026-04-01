import 'package:flutter/material.dart';
import '../../models/console_command_models.dart';

class CommandResultDialog extends StatelessWidget {
  final ConsoleCommand command;
  final CommandResult result;

  const CommandResultDialog({
    super.key,
    required this.command,
    required this.result,
  });

  static Future<void> show(
    BuildContext context,
    ConsoleCommand command,
    CommandResult result,
  ) {
    return showDialog(
      context: context,
      builder: (context) => CommandResultDialog(
        command: command,
        result: result,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Row(
        children: [
          Icon(
            result.success ? Icons.check_circle : Icons.error,
            color: result.success ? Colors.green : Colors.red,
          ),
          const SizedBox(width: 8),
          Text(result.success ? 'Command Executed' : 'Command Failed'),
        ],
      ),
      content: SizedBox(
        width: 500,
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildCommandBox(),
              const SizedBox(height: 16),
              Text(
                'Result:',
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: 8),
              _buildResultBox(),
              const SizedBox(height: 8),
              _buildTimestamp(),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }

  Widget _buildCommandBox() {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.grey[100],
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        command.name,
        style: const TextStyle(
          fontFamily: 'monospace',
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  Widget _buildResultBox() {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: result.success ? Colors.green[50] : Colors.red[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: result.success ? Colors.green[200]! : Colors.red[200]!,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(result.message),
          if (result.error != null) ...[
            const SizedBox(height: 8),
            Text(
              'Error: ${result.error}',
              style: const TextStyle(color: Colors.red, fontSize: 12),
            ),
          ],
          if (result.data != null) ...[
            const SizedBox(height: 8),
            const Divider(),
            const SizedBox(height: 8),
            _buildDataDisplay(),
          ],
        ],
      ),
    );
  }

  Widget _buildDataDisplay() {
    if (result.data is Map) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final entry in (result.data as Map).entries)
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    '${entry.key}: ',
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontWeight: FontWeight.bold,
                      fontSize: 12,
                    ),
                  ),
                  Expanded(
                    child: Text(
                      entry.value.toString(),
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ),
                ],
              ),
            ),
        ],
      );
    } else if (result.data is List) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final item in (result.data as List))
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Text(
                '• $item',
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12,
                ),
              ),
            ),
        ],
      );
    } else {
      return Text(
        result.data.toString(),
        style: const TextStyle(
          fontFamily: 'monospace',
          fontSize: 12,
        ),
      );
    }
  }

  Widget _buildTimestamp() {
    return Text(
      'Executed at: ${result.timestamp.toLocal()}',
      style: TextStyle(fontSize: 11, color: Colors.grey[600]),
    );
  }
}
