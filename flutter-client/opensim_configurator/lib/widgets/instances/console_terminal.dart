import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import '../../models/instance_models.dart';
import '../../providers/instance_manager_provider.dart';

class ConsoleTerminal extends StatefulWidget {
  final String instanceId;
  final double? height;
  final bool showHeader;
  final bool autoscroll;

  const ConsoleTerminal({
    super.key,
    required this.instanceId,
    this.height,
    this.showHeader = true,
    this.autoscroll = true,
  });

  @override
  State<ConsoleTerminal> createState() => _ConsoleTerminalState();
}

class _ConsoleTerminalState extends State<ConsoleTerminal> {
  final TextEditingController _commandController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  final FocusNode _focusNode = FocusNode();
  final List<String> _commandHistory = [];
  int _historyIndex = -1;

  @override
  void dispose() {
    _commandController.dispose();
    _scrollController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    if (widget.autoscroll && _scrollController.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 100),
          curve: Curves.easeOut,
        );
      });
    }
  }

  void _submitCommand() {
    final command = _commandController.text.trim();
    if (command.isEmpty) return;

    final provider = context.read<InstanceManagerProvider>();
    provider.sendConsoleCommand(widget.instanceId, command);

    _commandHistory.add(command);
    _historyIndex = _commandHistory.length;
    _commandController.clear();
    _scrollToBottom();
  }

  void _navigateHistory(int direction) {
    if (_commandHistory.isEmpty) return;

    setState(() {
      _historyIndex += direction;
      _historyIndex = _historyIndex.clamp(0, _commandHistory.length);

      if (_historyIndex < _commandHistory.length) {
        _commandController.text = _commandHistory[_historyIndex];
        _commandController.selection = TextSelection.fromPosition(
          TextPosition(offset: _commandController.text.length),
        );
      } else {
        _commandController.clear();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final instance = provider.instances.firstWhere(
      (i) => i.id == widget.instanceId,
      orElse: () => throw Exception('Instance not found'),
    );
    final history = provider.getConsoleHistory(widget.instanceId);

    WidgetsBinding.instance.addPostFrameCallback((_) => _scrollToBottom());

    return Container(
      height: widget.height,
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[800]!),
      ),
      child: Column(
        children: [
          if (widget.showHeader) _buildHeader(context, instance),
          Expanded(
            child: _buildConsoleOutput(history),
          ),
          _buildCommandInput(context, instance),
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context, ServerInstance instance) {
    final provider = context.read<InstanceManagerProvider>();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.grey[900],
        borderRadius: const BorderRadius.vertical(top: Radius.circular(8)),
      ),
      child: Row(
        children: [
          Container(
            width: 12,
            height: 12,
            decoration: BoxDecoration(
              color: provider.getStatusColor(instance.status),
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 8),
          Text(
            '${instance.name} Console',
            style: const TextStyle(
              color: Colors.white,
              fontWeight: FontWeight.w500,
              fontSize: 14,
            ),
          ),
          const Spacer(),
          IconButton(
            icon: const Icon(Icons.delete_outline, color: Colors.grey, size: 18),
            onPressed: () => provider.clearConsoleHistory(widget.instanceId),
            tooltip: 'Clear console',
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(),
          ),
          const SizedBox(width: 8),
          IconButton(
            icon: const Icon(Icons.copy, color: Colors.grey, size: 18),
            onPressed: () => _copyToClipboard(context),
            tooltip: 'Copy output',
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(),
          ),
        ],
      ),
    );
  }

  Widget _buildConsoleOutput(List<ConsoleEntry> history) {
    if (history.isEmpty) {
      return const Center(
        child: Text(
          'Console output will appear here...',
          style: TextStyle(
            color: Colors.grey,
            fontFamily: 'monospace',
          ),
        ),
      );
    }

    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.all(12),
      itemCount: history.length,
      itemBuilder: (context, index) {
        final entry = history[index];
        return _buildConsoleEntry(entry);
      },
    );
  }

  Widget _buildConsoleEntry(ConsoleEntry entry) {
    Color textColor;
    String prefix = '';

    switch (entry.outputType) {
      case ConsoleOutputType.stdout:
        textColor = Colors.white;
        break;
      case ConsoleOutputType.stderr:
        textColor = Colors.red[300]!;
        break;
      case ConsoleOutputType.info:
        textColor = Colors.blue[300]!;
        prefix = '[INFO] ';
        break;
      case ConsoleOutputType.warning:
        textColor = Colors.yellow[300]!;
        prefix = '[WARN] ';
        break;
      case ConsoleOutputType.error:
        textColor = Colors.red[400]!;
        prefix = '[ERROR] ';
        break;
      case ConsoleOutputType.debug:
        textColor = Colors.grey[400]!;
        prefix = '[DEBUG] ';
        break;
      case ConsoleOutputType.command:
        textColor = Colors.green[300]!;
        break;
    }

    final timestamp = '${entry.timestamp.hour.toString().padLeft(2, '0')}:'
        '${entry.timestamp.minute.toString().padLeft(2, '0')}:'
        '${entry.timestamp.second.toString().padLeft(2, '0')}';

    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '[$timestamp] ',
            style: TextStyle(
              color: Colors.grey[600],
              fontFamily: 'monospace',
              fontSize: 12,
            ),
          ),
          Expanded(
            child: SelectableText(
              '$prefix${entry.content}',
              style: TextStyle(
                color: textColor,
                fontFamily: 'monospace',
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCommandInput(BuildContext context, ServerInstance instance) {
    final isConnected = instance.connected;

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.grey[900],
        borderRadius: const BorderRadius.vertical(bottom: Radius.circular(8)),
      ),
      child: RawKeyboardListener(
        focusNode: _focusNode,
        onKey: (event) {
          if (event is RawKeyDownEvent) {
            if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
              _navigateHistory(-1);
            } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
              _navigateHistory(1);
            }
          }
        },
        child: Row(
          children: [
            const Text(
              '\$ ',
              style: TextStyle(
                color: Colors.green,
                fontFamily: 'monospace',
                fontWeight: FontWeight.bold,
              ),
            ),
            Expanded(
              child: TextField(
                controller: _commandController,
                enabled: isConnected,
                style: const TextStyle(
                  color: Colors.white,
                  fontFamily: 'monospace',
                  fontSize: 14,
                ),
                decoration: InputDecoration(
                  hintText: isConnected ? 'Enter command...' : 'Not connected',
                  hintStyle: TextStyle(color: Colors.grey[600]),
                  border: InputBorder.none,
                  contentPadding: EdgeInsets.zero,
                  isDense: true,
                ),
                onSubmitted: (_) => _submitCommand(),
              ),
            ),
            IconButton(
              icon: Icon(
                Icons.send,
                color: isConnected ? Colors.green : Colors.grey,
                size: 20,
              ),
              onPressed: isConnected ? _submitCommand : null,
              tooltip: 'Send command',
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
            ),
          ],
        ),
      ),
    );
  }

  void _copyToClipboard(BuildContext context) {
    final provider = context.read<InstanceManagerProvider>();
    final history = provider.getConsoleHistory(widget.instanceId);
    final text = history.map((e) => e.content).join('\n');

    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Console output copied to clipboard'),
        duration: Duration(seconds: 2),
      ),
    );
  }
}

class ConsoleTerminalMini extends StatelessWidget {
  final String instanceId;
  final int maxLines;

  const ConsoleTerminalMini({
    super.key,
    required this.instanceId,
    this.maxLines = 5,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceManagerProvider>();
    final history = provider.getConsoleHistory(instanceId);
    final recentEntries = history.length > maxLines
        ? history.sublist(history.length - maxLines)
        : history;

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: recentEntries.map((entry) {
          Color textColor = Colors.white;
          if (entry.outputType == ConsoleOutputType.error) {
            textColor = Colors.red[300]!;
          } else if (entry.outputType == ConsoleOutputType.warning) {
            textColor = Colors.yellow[300]!;
          } else if (entry.outputType == ConsoleOutputType.command) {
            textColor = Colors.green[300]!;
          }

          return Text(
            entry.content,
            style: TextStyle(
              color: textColor,
              fontFamily: 'monospace',
              fontSize: 11,
            ),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          );
        }).toList(),
      ),
    );
  }
}
