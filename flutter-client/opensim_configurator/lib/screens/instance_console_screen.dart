import 'package:flutter/material.dart';
import '../models/console_command_models.dart';
import '../services/console_service.dart';

class InstanceConsoleScreen extends StatefulWidget {
  final String instanceId;
  final String instanceName;
  final String baseUrl;
  final String? apiKey;

  const InstanceConsoleScreen({
    super.key,
    required this.instanceId,
    required this.instanceName,
    required this.baseUrl,
    this.apiKey,
  });

  @override
  State<InstanceConsoleScreen> createState() => _InstanceConsoleScreenState();
}

class _InstanceConsoleScreenState extends State<InstanceConsoleScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  late ConsoleService _consoleService;
  late Map<CommandGroup, List<ConsoleCommand>> _commandsByGroup;
  final List<CommandExecution> _executionHistory = [];
  String? _filterText;
  bool _showOnlyImplemented = false;

  @override
  void initState() {
    super.initState();
    _consoleService = ConsoleService(
      baseUrl: widget.baseUrl,
      apiKey: widget.apiKey,
    );
    _commandsByGroup = _consoleService.getCommandsByGroup();
    _tabController = TabController(
      length: CommandGroup.values.length,
      vsync: this,
    );
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  List<ConsoleCommand> _getFilteredCommands(CommandGroup group) {
    var commands = _commandsByGroup[group] ?? [];

    if (_showOnlyImplemented) {
      commands = commands.where((c) => c.implemented).toList();
    }

    if (_filterText != null && _filterText!.isNotEmpty) {
      final filter = _filterText!.toLowerCase();
      commands = commands.where((c) =>
        c.name.toLowerCase().contains(filter) ||
        c.description.toLowerCase().contains(filter)
      ).toList();
    }

    return commands;
  }

  Future<void> _executeCommand(ConsoleCommand command, Map<String, dynamic> params) async {
    final execution = CommandExecution(
      command: command.name,
      params: params,
    );

    setState(() {
      _executionHistory.insert(0, execution);
    });

    try {
      final result = await _consoleService.executeCommand(command, params);
      setState(() {
        _executionHistory[0] = CommandExecution(
          command: command.name,
          params: params,
          result: result,
          executedAt: execution.executedAt,
        );
      });

      _showResultDialog(command, result);
    } catch (e) {
      final errorResult = CommandResult(
        success: false,
        message: 'Command execution failed',
        error: e.toString(),
      );
      setState(() {
        _executionHistory[0] = CommandExecution(
          command: command.name,
          params: params,
          result: errorResult,
          executedAt: execution.executedAt,
        );
      });

      _showResultDialog(command, errorResult);
    }
  }

  void _showCommandDialog(ConsoleCommand command) {
    final formKey = GlobalKey<FormState>();
    final paramValues = <String, dynamic>{};

    for (final param in command.params) {
      if (param.defaultValue != null) {
        paramValues[param.name] = param.defaultValue;
      }
    }

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Row(
          children: [
            Icon(
              command.implemented ? Icons.check_circle : Icons.pending,
              color: command.implemented ? Colors.green : Colors.orange,
              size: 20,
            ),
            const SizedBox(width: 8),
            Expanded(child: Text(command.name)),
          ],
        ),
        content: SizedBox(
          width: 500,
          child: SingleChildScrollView(
            child: Form(
              key: formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    command.description,
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: Colors.grey[100],
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      'Syntax: ${command.syntax}',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ),
                  if (command.params.isNotEmpty) ...[
                    const SizedBox(height: 16),
                    const Divider(),
                    const SizedBox(height: 8),
                    Text(
                      'Parameters',
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                    const SizedBox(height: 8),
                    ...command.params.map((param) => _buildParamField(param, paramValues)),
                  ],
                  if (!command.implemented) ...[
                    const SizedBox(height: 16),
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.orange[50],
                        borderRadius: BorderRadius.circular(8),
                        border: Border.all(color: Colors.orange[200]!),
                      ),
                      child: Row(
                        children: [
                          Icon(Icons.warning_amber, color: Colors.orange[700]),
                          const SizedBox(width: 8),
                          const Expanded(
                            child: Text(
                              'This command is not yet implemented in opensim-next',
                              style: TextStyle(fontSize: 12),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: command.implemented
                ? () {
                    if (formKey.currentState!.validate()) {
                      formKey.currentState!.save();
                      Navigator.pop(context);
                      _executeCommand(command, paramValues);
                    }
                  }
                : null,
            child: const Text('Execute'),
          ),
        ],
      ),
    );
  }

  Widget _buildParamField(CommandParam param, Map<String, dynamic> paramValues) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                param.name,
                style: const TextStyle(fontWeight: FontWeight.w500),
              ),
              if (param.required)
                const Text(' *', style: TextStyle(color: Colors.red)),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            param.description,
            style: TextStyle(fontSize: 12, color: Colors.grey[600]),
          ),
          const SizedBox(height: 4),
          _buildInputForParamType(param, paramValues),
        ],
      ),
    );
  }

  Widget _buildInputForParamType(CommandParam param, Map<String, dynamic> paramValues) {
    switch (param.type) {
      case ParamType.boolean:
        return StatefulBuilder(
          builder: (context, setState) {
            return SwitchListTile(
              value: paramValues[param.name] == 'true' || paramValues[param.name] == true,
              onChanged: (value) {
                setState(() {
                  paramValues[param.name] = value;
                });
              },
              title: Text(param.name),
              contentPadding: EdgeInsets.zero,
            );
          },
        );

      case ParamType.choice:
        return DropdownButtonFormField<String>(
          value: paramValues[param.name] as String?,
          items: param.choices?.map((choice) => DropdownMenuItem(
            value: choice,
            child: Text(choice),
          )).toList() ?? [],
          onChanged: (value) {
            paramValues[param.name] = value;
          },
          onSaved: (value) {
            paramValues[param.name] = value;
          },
          validator: param.required
              ? (value) => value == null || value.isEmpty ? 'Required' : null
              : null,
          decoration: InputDecoration(
            hintText: param.placeholder ?? 'Select ${param.name}',
            isDense: true,
            border: const OutlineInputBorder(),
          ),
        );

      case ParamType.integer:
      case ParamType.number:
        return TextFormField(
          initialValue: paramValues[param.name]?.toString(),
          keyboardType: TextInputType.number,
          decoration: InputDecoration(
            hintText: param.placeholder ?? 'Enter ${param.name}',
            isDense: true,
            border: const OutlineInputBorder(),
          ),
          validator: param.required
              ? (value) => value == null || value.isEmpty ? 'Required' : null
              : null,
          onSaved: (value) {
            if (value != null && value.isNotEmpty) {
              paramValues[param.name] = param.type == ParamType.integer
                  ? int.tryParse(value) ?? value
                  : double.tryParse(value) ?? value;
            }
          },
        );

      case ParamType.file:
      case ParamType.path:
        return Row(
          children: [
            Expanded(
              child: TextFormField(
                initialValue: paramValues[param.name] as String?,
                decoration: InputDecoration(
                  hintText: param.placeholder ?? 'Enter path',
                  isDense: true,
                  border: const OutlineInputBorder(),
                ),
                validator: param.required
                    ? (value) => value == null || value.isEmpty ? 'Required' : null
                    : null,
                onSaved: (value) {
                  paramValues[param.name] = value;
                },
              ),
            ),
            const SizedBox(width: 8),
            IconButton(
              icon: const Icon(Icons.folder_open),
              onPressed: () {
                // File picker would go here in production
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('File browser not implemented in web')),
                );
              },
              tooltip: 'Browse',
            ),
          ],
        );

      case ParamType.uuid:
        return TextFormField(
          initialValue: paramValues[param.name] as String?,
          decoration: InputDecoration(
            hintText: param.placeholder ?? '00000000-0000-0000-0000-000000000000',
            isDense: true,
            border: const OutlineInputBorder(),
          ),
          validator: (value) {
            if (param.required && (value == null || value.isEmpty)) {
              return 'Required';
            }
            if (value != null && value.isNotEmpty) {
              final uuidRegex = RegExp(
                r'^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$',
              );
              if (!uuidRegex.hasMatch(value)) {
                return 'Invalid UUID format';
              }
            }
            return null;
          },
          onSaved: (value) {
            paramValues[param.name] = value;
          },
        );

      case ParamType.string:
      default:
        return TextFormField(
          initialValue: paramValues[param.name] as String?,
          decoration: InputDecoration(
            hintText: param.placeholder ?? 'Enter ${param.name}',
            isDense: true,
            border: const OutlineInputBorder(),
          ),
          validator: param.required
              ? (value) => value == null || value.isEmpty ? 'Required' : null
              : null,
          onSaved: (value) {
            paramValues[param.name] = value;
          },
        );
    }
  }

  void _showResultDialog(ConsoleCommand command, CommandResult result) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
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
                Container(
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
                ),
                const SizedBox(height: 16),
                Text(
                  'Result:',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const SizedBox(height: 8),
                Container(
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
                        Text(
                          result.data.toString(),
                          style: const TextStyle(
                            fontFamily: 'monospace',
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Executed at: ${result.timestamp.toLocal()}',
                  style: TextStyle(fontSize: 11, color: Colors.grey[600]),
                ),
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
      ),
    );
  }

  void _showHistoryDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Command History'),
        content: SizedBox(
          width: 600,
          height: 400,
          child: _executionHistory.isEmpty
              ? const Center(child: Text('No commands executed yet'))
              : ListView.builder(
                  itemCount: _executionHistory.length,
                  itemBuilder: (context, index) {
                    final exec = _executionHistory[index];
                    return Card(
                      child: ListTile(
                        leading: Icon(
                          exec.result == null
                              ? Icons.hourglass_empty
                              : exec.result!.success
                                  ? Icons.check_circle
                                  : Icons.error,
                          color: exec.result == null
                              ? Colors.grey
                              : exec.result!.success
                                  ? Colors.green
                                  : Colors.red,
                        ),
                        title: Text(exec.command),
                        subtitle: Text(
                          exec.result?.message ?? 'Executing...',
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        trailing: Text(
                          '${exec.executedAt.hour}:${exec.executedAt.minute.toString().padLeft(2, '0')}',
                          style: TextStyle(color: Colors.grey[600], fontSize: 12),
                        ),
                      ),
                    );
                  },
                ),
        ),
        actions: [
          TextButton(
            onPressed: () {
              setState(() {
                _executionHistory.clear();
              });
              Navigator.pop(context);
            },
            child: const Text('Clear History'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('Console Commands'),
            Text(
              widget.instanceName,
              style: const TextStyle(fontSize: 12, fontWeight: FontWeight.normal),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: Badge(
              label: Text('${_executionHistory.length}'),
              isLabelVisible: _executionHistory.isNotEmpty,
              child: const Icon(Icons.history),
            ),
            onPressed: _showHistoryDialog,
            tooltip: 'Command History',
          ),
          IconButton(
            icon: Icon(
              _showOnlyImplemented ? Icons.filter_alt : Icons.filter_alt_outlined,
            ),
            onPressed: () {
              setState(() {
                _showOnlyImplemented = !_showOnlyImplemented;
              });
            },
            tooltip: _showOnlyImplemented ? 'Show All' : 'Show Implemented Only',
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              setState(() {
                _commandsByGroup = _consoleService.getCommandsByGroup();
              });
            },
            tooltip: 'Refresh',
          ),
        ],
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(100),
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                child: TextField(
                  decoration: InputDecoration(
                    hintText: 'Filter commands...',
                    prefixIcon: const Icon(Icons.search),
                    isDense: true,
                    filled: true,
                    fillColor: Colors.white,
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(8),
                      borderSide: BorderSide.none,
                    ),
                  ),
                  onChanged: (value) {
                    setState(() {
                      _filterText = value;
                    });
                  },
                ),
              ),
              TabBar(
                controller: _tabController,
                isScrollable: true,
                tabs: CommandGroup.values.map((group) {
                  final commands = _getFilteredCommands(group);
                  final implementedCount = commands.where((c) => c.implemented).length;
                  return Tab(
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(_getGroupLabel(group)),
                        const SizedBox(width: 4),
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
              ),
            ],
          ),
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: CommandGroup.values.map((group) {
          final commands = _getFilteredCommands(group);
          if (commands.isEmpty) {
            return Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.search_off, size: 64, color: Colors.grey[400]),
                  const SizedBox(height: 16),
                  Text(
                    _filterText != null && _filterText!.isNotEmpty
                        ? 'No commands match "$_filterText"'
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
              return Card(
                child: ListTile(
                  leading: Icon(
                    command.implemented ? Icons.check_circle : Icons.pending,
                    color: command.implemented ? Colors.green : Colors.orange,
                  ),
                  title: Text(
                    command.name,
                    style: const TextStyle(fontWeight: FontWeight.w500),
                  ),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(command.description),
                      const SizedBox(height: 4),
                      Text(
                        command.syntax,
                        style: TextStyle(
                          fontFamily: 'monospace',
                          fontSize: 11,
                          color: Colors.grey[600],
                        ),
                      ),
                    ],
                  ),
                  trailing: command.params.isNotEmpty
                      ? Chip(
                          label: Text('${command.params.length} params'),
                          padding: EdgeInsets.zero,
                          labelStyle: const TextStyle(fontSize: 10),
                        )
                      : null,
                  onTap: () => _showCommandDialog(command),
                ),
              );
            },
          );
        }).toList(),
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
