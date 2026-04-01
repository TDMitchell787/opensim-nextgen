import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

class InstanceDirectoryEntry {
  final String name;
  final String path;
  final String mode;
  final int loginPort;
  final int robustPort;
  final int regionCount;
  final String databaseUrl;
  final bool hypergridEnabled;
  final String homeUri;
  final bool hasStartScript;
  final bool hasPreflight;

  InstanceDirectoryEntry({
    required this.name,
    required this.path,
    this.mode = 'standalone',
    this.loginPort = 9000,
    this.robustPort = 8003,
    this.regionCount = 0,
    this.databaseUrl = '',
    this.hypergridEnabled = false,
    this.homeUri = '',
    this.hasStartScript = false,
    this.hasPreflight = false,
  });

  String get modeDisplay {
    switch (mode) {
      case 'grid': return 'Grid';
      case 'robust': return 'Robust';
      case 'standalone': return 'Standalone';
      default: return mode;
    }
  }

  IconData get modeIcon {
    switch (mode) {
      case 'grid': return Icons.grid_on;
      case 'robust': return Icons.dns;
      case 'standalone': return Icons.computer;
      default: return Icons.help_outline;
    }
  }

  Color get modeColor {
    switch (mode) {
      case 'grid': return Colors.blue;
      case 'robust': return Colors.purple;
      case 'standalone': return Colors.green;
      default: return Colors.grey;
    }
  }
}

class InstanceDirectoryProvider extends ChangeNotifier {
  final String instancesBasePath;
  List<InstanceDirectoryEntry> _entries = [];
  bool _isScanning = false;
  Timer? _scanTimer;

  List<InstanceDirectoryEntry> get entries => List.unmodifiable(_entries);
  bool get isScanning => _isScanning;
  int get totalInstances => _entries.length;
  int get gridInstances => _entries.where((e) => e.mode == 'grid').length;
  int get standaloneInstances => _entries.where((e) => e.mode == 'standalone').length;

  InstanceDirectoryProvider({required this.instancesBasePath}) {
    scanInstances();
    _scanTimer = Timer.periodic(const Duration(seconds: 60), (_) => scanInstances());
  }

  @override
  void dispose() {
    _scanTimer?.cancel();
    super.dispose();
  }

  Future<void> scanInstances() async {
    if (_isScanning) return;
    _isScanning = true;
    notifyListeners();

    try {
      final dir = Directory(instancesBasePath);
      if (!dir.existsSync()) {
        _entries = [];
        return;
      }

      final newEntries = <InstanceDirectoryEntry>[];

      for (final entity in dir.listSync()) {
        if (entity is! Directory) continue;
        final name = entity.path.split('/').last;
        if (name == 'template' || name.startsWith('.')) continue;

        final envFile = File('${entity.path}/.env');
        if (!envFile.existsSync()) continue;

        final env = _parseEnvFile(envFile.readAsStringSync());

        int regionCount = 0;
        final regionsDir = Directory('${entity.path}/Regions');
        if (regionsDir.existsSync()) {
          final regionsIni = File('${entity.path}/Regions/Regions.ini');
          if (regionsIni.existsSync()) {
            regionCount = regionsIni.readAsLinesSync()
                .where((l) => l.trim().startsWith('['))
                .length;
          }
        }

        newEntries.add(InstanceDirectoryEntry(
          name: name,
          path: entity.path,
          mode: env['OPENSIM_SERVICE_MODE'] ?? 'standalone',
          loginPort: int.tryParse(env['OPENSIM_LOGIN_PORT'] ?? '') ?? 9000,
          robustPort: int.tryParse(env['OPENSIM_ROBUST_PORT'] ?? '') ?? 8003,
          regionCount: regionCount,
          databaseUrl: env['DATABASE_URL'] ?? '',
          hypergridEnabled: env['OPENSIM_HYPERGRID_ENABLED'] == 'true',
          homeUri: env['OPENSIM_HOME_URI'] ?? '',
          hasStartScript: File('${entity.path}/start.sh').existsSync(),
          hasPreflight: File('${entity.path}/preflight.sh').existsSync(),
        ));
      }

      newEntries.sort((a, b) => a.name.compareTo(b.name));
      _entries = newEntries;
    } finally {
      _isScanning = false;
      notifyListeners();
    }
  }

  Map<String, String> _parseEnvFile(String content) {
    final env = <String, String>{};
    for (final line in content.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith('#')) continue;
      final idx = trimmed.indexOf('=');
      if (idx > 0) {
        env[trimmed.substring(0, idx).trim()] = trimmed.substring(idx + 1).trim();
      }
    }
    return env;
  }

  Future<void> scanInstancesViaController(String controllerUrl) async {
    if (_isScanning) return;
    _isScanning = true;
    notifyListeners();

    try {
      final response = await http.get(
        Uri.parse('$controllerUrl/api/instance-dirs'),
      );

      if (response.statusCode == 200) {
        final data = json.decode(response.body);
        final dirs = data['directories'] as List? ?? [];
        final newEntries = <InstanceDirectoryEntry>[];

        for (final dir in dirs) {
          newEntries.add(InstanceDirectoryEntry(
            name: dir['name'] ?? '',
            path: dir['path'] ?? '',
            mode: dir['service_mode'] ?? 'standalone',
            loginPort: dir['login_port'] ?? 9000,
            robustPort: dir['robust_port'] ?? 8003,
            regionCount: dir['region_count'] ?? 0,
            databaseUrl: dir['database_url'] ?? '',
            hypergridEnabled: dir['hypergrid_enabled'] ?? false,
          ));
        }

        newEntries.sort((a, b) => a.name.compareTo(b.name));
        _entries = newEntries;
      }
    } catch (e) {
      debugPrint('Failed to scan via controller: $e');
    } finally {
      _isScanning = false;
      notifyListeners();
    }
  }
}
