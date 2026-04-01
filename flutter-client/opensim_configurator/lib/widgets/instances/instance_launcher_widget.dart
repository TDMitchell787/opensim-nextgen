import 'dart:io';
import 'package:flutter/material.dart';

class InstanceEnvConfig {
  final String instanceDir;
  final String serviceMode;
  final String databaseUrl;
  final int loginPort;
  final int robustPort;
  final String gridName;
  final bool hypergridEnabled;
  final String homeUri;
  final int regionCount;

  InstanceEnvConfig({
    required this.instanceDir,
    this.serviceMode = 'standalone',
    this.databaseUrl = '',
    this.loginPort = 9000,
    this.robustPort = 8003,
    this.gridName = 'OpenSim',
    this.hypergridEnabled = false,
    this.homeUri = '',
    this.regionCount = 0,
  });

  factory InstanceEnvConfig.fromEnvFile(String path) {
    final file = File(path);
    if (!file.existsSync()) return InstanceEnvConfig(instanceDir: path);

    final lines = file.readAsLinesSync();
    final env = <String, String>{};
    for (final line in lines) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) continue;
      final idx = trimmed.indexOf('=');
      if (idx > 0) {
        env[trimmed.substring(0, idx).trim()] = trimmed.substring(idx + 1).trim();
      }
    }

    final dir = Directory(File(path).parent.path);
    int regionCount = 0;
    final regionsIni = File('${dir.path}/Regions/Regions.ini');
    if (regionsIni.existsSync()) {
      regionCount = regionsIni.readAsLinesSync()
          .where((l) => l.trim().startsWith('['))
          .length;
    }

    return InstanceEnvConfig(
      instanceDir: dir.path,
      serviceMode: env['OPENSIM_SERVICE_MODE'] ?? 'standalone',
      databaseUrl: env['DATABASE_URL'] ?? '',
      loginPort: int.tryParse(env['OPENSIM_LOGIN_PORT'] ?? '') ?? 9000,
      robustPort: int.tryParse(env['OPENSIM_ROBUST_PORT'] ?? '') ?? 8003,
      gridName: env['OPENSIM_INSTANCE_ID'] ?? 'OpenSim',
      hypergridEnabled: env['OPENSIM_HYPERGRID_ENABLED'] == 'true',
      homeUri: env['OPENSIM_HOME_URI'] ?? '',
      regionCount: regionCount,
    );
  }
}

class InstanceLauncherWidget extends StatefulWidget {
  final String instancesDir;
  final Function(String instanceName, String mode)? onLaunch;

  const InstanceLauncherWidget({
    super.key,
    required this.instancesDir,
    this.onLaunch,
  });

  @override
  State<InstanceLauncherWidget> createState() => _InstanceLauncherWidgetState();
}

class _InstanceLauncherWidgetState extends State<InstanceLauncherWidget> {
  List<String> _instanceNames = [];
  String? _selectedInstance;
  InstanceEnvConfig? _selectedConfig;

  @override
  void initState() {
    super.initState();
    _scanInstances();
  }

  void _scanInstances() {
    final dir = Directory(widget.instancesDir);
    if (!dir.existsSync()) {
      setState(() => _instanceNames = []);
      return;
    }

    final names = dir.listSync()
        .whereType<Directory>()
        .where((d) {
          final name = d.path.split('/').last;
          return name != 'template' && File('${d.path}/.env').existsSync();
        })
        .map((d) => d.path.split('/').last)
        .toList()
      ..sort();

    setState(() => _instanceNames = names);
  }

  void _selectInstance(String? name) {
    if (name == null) return;
    final envPath = '${widget.instancesDir}/$name/.env';
    setState(() {
      _selectedInstance = name;
      _selectedConfig = InstanceEnvConfig.fromEnvFile(envPath);
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.rocket_launch, size: 24),
                const SizedBox(width: 8),
                Text('Instance Launcher', style: theme.textTheme.titleLarge),
                const Spacer(),
                IconButton(
                  onPressed: _scanInstances,
                  icon: const Icon(Icons.refresh),
                  tooltip: 'Rescan Instances',
                ),
              ],
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              value: _selectedInstance,
              decoration: const InputDecoration(
                labelText: 'Select Instance',
                border: OutlineInputBorder(),
              ),
              items: _instanceNames.map((name) {
                return DropdownMenuItem(value: name, child: Text(name));
              }).toList(),
              onChanged: _selectInstance,
            ),
            if (_selectedConfig != null) ...[
              const SizedBox(height: 12),
              _buildConfigSummary(_selectedConfig!),
              const SizedBox(height: 12),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  if (_selectedConfig!.serviceMode == 'grid') ...[
                    ElevatedButton.icon(
                      onPressed: () => widget.onLaunch?.call(_selectedInstance!, 'robust'),
                      icon: const Icon(Icons.dns),
                      label: const Text('Launch Robust'),
                    ),
                    const SizedBox(width: 8),
                  ],
                  ElevatedButton.icon(
                    onPressed: () => widget.onLaunch?.call(
                      _selectedInstance!,
                      _selectedConfig!.serviceMode,
                    ),
                    icon: const Icon(Icons.play_arrow),
                    label: Text('Launch ${_selectedConfig!.serviceMode.toUpperCase()}'),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildConfigSummary(InstanceEnvConfig config) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _infoRow('Mode', config.serviceMode.toUpperCase()),
        _infoRow('Login Port', config.loginPort.toString()),
        if (config.serviceMode == 'grid')
          _infoRow('Robust Port', config.robustPort.toString()),
        _infoRow('Regions', config.regionCount.toString()),
        _infoRow('Database', config.databaseUrl.isNotEmpty
            ? config.databaseUrl.replaceAll(RegExp(r'://.*@'), '://***@')
            : 'Not configured'),
        if (config.hypergridEnabled)
          _infoRow('Hypergrid', config.homeUri),
      ],
    );
  }

  Widget _infoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 100,
            child: Text(label, style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
          ),
          Expanded(child: Text(value, style: const TextStyle(fontSize: 12))),
        ],
      ),
    );
  }
}
