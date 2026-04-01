import 'package:flutter/material.dart';
import '../../models/console_command_models.dart';

class CommandDialog extends StatefulWidget {
  final ConsoleCommand command;
  final Function(Map<String, dynamic>) onExecute;

  const CommandDialog({
    super.key,
    required this.command,
    required this.onExecute,
  });

  @override
  State<CommandDialog> createState() => _CommandDialogState();

  static Future<void> show(
    BuildContext context,
    ConsoleCommand command,
    Function(Map<String, dynamic>) onExecute,
  ) {
    return showDialog(
      context: context,
      builder: (context) => CommandDialog(
        command: command,
        onExecute: onExecute,
      ),
    );
  }
}

class _CommandDialogState extends State<CommandDialog> {
  final _formKey = GlobalKey<FormState>();
  final _paramValues = <String, dynamic>{};

  @override
  void initState() {
    super.initState();
    for (final param in widget.command.params) {
      if (param.defaultValue != null) {
        _paramValues[param.name] = param.defaultValue;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Row(
        children: [
          Icon(
            widget.command.implemented ? Icons.check_circle : Icons.pending,
            color: widget.command.implemented ? Colors.green : Colors.orange,
            size: 20,
          ),
          const SizedBox(width: 8),
          Expanded(child: Text(widget.command.name)),
        ],
      ),
      content: SizedBox(
        width: 500,
        child: SingleChildScrollView(
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  widget.command.description,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 8),
                _buildSyntaxBox(),
                if (widget.command.params.isNotEmpty) ...[
                  const SizedBox(height: 16),
                  const Divider(),
                  const SizedBox(height: 8),
                  Text(
                    'Parameters',
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: 8),
                  ...widget.command.params.map(_buildParamField),
                ],
                if (!widget.command.implemented) _buildNotImplementedWarning(),
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
          onPressed: widget.command.implemented ? _onExecute : null,
          child: const Text('Execute'),
        ),
      ],
    );
  }

  Widget _buildSyntaxBox() {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.grey[100],
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        'Syntax: ${widget.command.syntax}',
        style: const TextStyle(
          fontFamily: 'monospace',
          fontSize: 12,
        ),
      ),
    );
  }

  Widget _buildNotImplementedWarning() {
    return Padding(
      padding: const EdgeInsets.only(top: 16),
      child: Container(
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
    );
  }

  Widget _buildParamField(CommandParam param) {
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
          _buildInputWidget(param),
        ],
      ),
    );
  }

  Widget _buildInputWidget(CommandParam param) {
    switch (param.type) {
      case ParamType.boolean:
        return _buildBooleanInput(param);
      case ParamType.choice:
        return _buildChoiceInput(param);
      case ParamType.integer:
      case ParamType.number:
        return _buildNumberInput(param);
      case ParamType.file:
      case ParamType.path:
        return _buildPathInput(param);
      case ParamType.uuid:
        return _buildUuidInput(param);
      case ParamType.string:
      default:
        return _buildTextInput(param);
    }
  }

  Widget _buildBooleanInput(CommandParam param) {
    return StatefulBuilder(
      builder: (context, setState) {
        return SwitchListTile(
          value: _paramValues[param.name] == 'true' || _paramValues[param.name] == true,
          onChanged: (value) {
            setState(() {
              _paramValues[param.name] = value;
            });
          },
          title: Text(param.name),
          contentPadding: EdgeInsets.zero,
        );
      },
    );
  }

  Widget _buildChoiceInput(CommandParam param) {
    return DropdownButtonFormField<String>(
      value: _paramValues[param.name] as String?,
      items: param.choices?.map((choice) => DropdownMenuItem(
        value: choice,
        child: Text(choice),
      )).toList() ?? [],
      onChanged: (value) => _paramValues[param.name] = value,
      onSaved: (value) => _paramValues[param.name] = value,
      validator: param.required
          ? (value) => value == null || value.isEmpty ? 'Required' : null
          : null,
      decoration: InputDecoration(
        hintText: param.placeholder ?? 'Select ${param.name}',
        isDense: true,
        border: const OutlineInputBorder(),
      ),
    );
  }

  Widget _buildNumberInput(CommandParam param) {
    return TextFormField(
      initialValue: _paramValues[param.name]?.toString(),
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
          _paramValues[param.name] = param.type == ParamType.integer
              ? int.tryParse(value) ?? value
              : double.tryParse(value) ?? value;
        }
      },
    );
  }

  Widget _buildPathInput(CommandParam param) {
    return Row(
      children: [
        Expanded(
          child: TextFormField(
            initialValue: _paramValues[param.name] as String?,
            decoration: InputDecoration(
              hintText: param.placeholder ?? 'Enter path',
              isDense: true,
              border: const OutlineInputBorder(),
            ),
            validator: param.required
                ? (value) => value == null || value.isEmpty ? 'Required' : null
                : null,
            onSaved: (value) => _paramValues[param.name] = value,
          ),
        ),
        const SizedBox(width: 8),
        IconButton(
          icon: const Icon(Icons.folder_open),
          onPressed: () {
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(content: Text('File browser not implemented in web')),
            );
          },
          tooltip: 'Browse',
        ),
      ],
    );
  }

  Widget _buildUuidInput(CommandParam param) {
    return TextFormField(
      initialValue: _paramValues[param.name] as String?,
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
      onSaved: (value) => _paramValues[param.name] = value,
    );
  }

  Widget _buildTextInput(CommandParam param) {
    return TextFormField(
      initialValue: _paramValues[param.name] as String?,
      decoration: InputDecoration(
        hintText: param.placeholder ?? 'Enter ${param.name}',
        isDense: true,
        border: const OutlineInputBorder(),
      ),
      validator: param.required
          ? (value) => value == null || value.isEmpty ? 'Required' : null
          : null,
      onSaved: (value) => _paramValues[param.name] = value,
    );
  }

  void _onExecute() {
    if (_formKey.currentState!.validate()) {
      _formKey.currentState!.save();
      Navigator.pop(context);
      widget.onExecute(_paramValues);
    }
  }
}
