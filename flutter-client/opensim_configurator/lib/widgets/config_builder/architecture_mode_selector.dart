import 'package:flutter/material.dart';
import '../../models/deployment_models.dart';

class ArchitectureModeSelector extends StatelessWidget {
  final ArchitectureMode selectedMode;
  final ValueChanged<ArchitectureMode> onModeChanged;
  final ArchitectureConfig? config;

  const ArchitectureModeSelector({
    super.key,
    required this.selectedMode,
    required this.onModeChanged,
    this.config,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildArchitectureBanner(context),
        const SizedBox(height: 16),
        _buildSectionHeader(context, 'Architecture Mode'),
        const SizedBox(height: 12),
        _buildModeCards(context),
        const SizedBox(height: 24),
        if (selectedMode == ArchitectureMode.gridServer) ...[
          _buildSectionHeader(context, 'Grid Server Configuration'),
          const SizedBox(height: 12),
          _buildGridServerConfig(context),
        ] else if (selectedMode == ArchitectureMode.regionServer) ...[
          _buildSectionHeader(context, 'Region Server Configuration'),
          const SizedBox(height: 12),
          _buildRegionServerConfig(context),
        ] else ...[
          _buildSectionHeader(context, 'Standalone Configuration'),
          const SizedBox(height: 12),
          _buildStandaloneConfig(context),
        ],
        const SizedBox(height: 24),
        _buildSectionHeader(context, 'What This Mode Creates'),
        const SizedBox(height: 12),
        _buildModeDetails(context),
      ],
    );
  }

  Widget _buildArchitectureBanner(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [Colors.indigo[400]!, Colors.indigo[600]!],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Colors.white.withOpacity(0.2),
              borderRadius: BorderRadius.circular(8),
            ),
            child: const Icon(
              Icons.account_tree,
              color: Colors.white,
              size: 32,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Architecture Mode',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 18,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Choose how services are distributed: all on one server, or split between grid and region servers.',
                  style: TextStyle(
                    color: Colors.white.withOpacity(0.9),
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            _getSectionIcon(title),
            size: 18,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getSectionIcon(String title) {
    switch (title) {
      case 'Architecture Mode':
        return Icons.account_tree;
      case 'Grid Server Configuration':
        return Icons.dns;
      case 'Region Server Configuration':
        return Icons.map;
      case 'Standalone Configuration':
        return Icons.computer;
      case 'What This Mode Creates':
        return Icons.list_alt;
      default:
        return Icons.settings;
    }
  }

  Widget _buildModeCards(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            _buildModeCard(
              context,
              ArchitectureMode.standalone,
              'Standalone',
              'All services on a single server',
              Icons.computer,
              Colors.green,
              [
                'Users, inventory, assets local',
                'Region simulator built-in',
                'Simplest to set up',
              ],
            ),
            const SizedBox(height: 12),
            _buildModeCard(
              context,
              ArchitectureMode.gridServer,
              'Grid Server',
              'Central hub for multi-region grid',
              Icons.dns,
              Colors.blue,
              [
                'Central authentication',
                'Shared inventory & assets',
                'Region registry',
              ],
            ),
            const SizedBox(height: 12),
            _buildModeCard(
              context,
              ArchitectureMode.regionServer,
              'Region Server',
              'Connects to existing grid server',
              Icons.map,
              Colors.orange,
              [
                'Local prims & terrain',
                'Connects to grid services',
                'Lightweight setup',
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildModeCard(
    BuildContext context,
    ArchitectureMode mode,
    String title,
    String subtitle,
    IconData icon,
    Color color,
    List<String> features,
  ) {
    final isSelected = mode == selectedMode;

    return Material(
      color: isSelected ? color.withOpacity(0.1) : Colors.transparent,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        onTap: () => onModeChanged(mode),
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isSelected ? color : Colors.grey[300]!,
              width: isSelected ? 2 : 1,
            ),
          ),
          child: Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: color.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Icon(icon, color: color, size: 28),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          title,
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 16,
                            color: isSelected ? color : null,
                          ),
                        ),
                        if (isSelected) ...[
                          const SizedBox(width: 8),
                          Icon(Icons.check_circle, color: color, size: 18),
                        ],
                      ],
                    ),
                    Text(
                      subtitle,
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.grey[600],
                      ),
                    ),
                    const SizedBox(height: 8),
                    Wrap(
                      spacing: 8,
                      runSpacing: 4,
                      children: features
                          .map((f) => Chip(
                                label: Text(f),
                                labelStyle: const TextStyle(fontSize: 10),
                                padding: EdgeInsets.zero,
                                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                              ))
                          .toList(),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStandaloneConfig(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.green[50],
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.green[200]!),
              ),
              child: Row(
                children: [
                  Icon(Icons.info, color: Colors.green[700], size: 20),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Standalone Mode',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Colors.green[900],
                          ),
                        ),
                        Text(
                          'All services run on this server: user accounts, inventory, assets, and region simulator. Perfect for personal grids, testing, or small communities.',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.green[700],
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'postgresql://opensim@localhost/opensim_standalone',
              decoration: const InputDecoration(
                labelText: 'Database URL',
                hintText: 'postgresql://user@host/database',
                prefixIcon: Icon(Icons.storage),
                border: OutlineInputBorder(),
                helperText: 'Single database for all services',
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildGridServerConfig(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.blue[50],
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.blue[200]!),
              ),
              child: Row(
                children: [
                  Icon(Icons.info, color: Colors.blue[700], size: 20),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Grid Server Mode',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Colors.blue[900],
                          ),
                        ),
                        Text(
                          'This server becomes the central hub. Region servers will connect here for authentication, inventory, and assets.',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.blue[700],
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'My Grid',
              decoration: const InputDecoration(
                labelText: 'Grid Name',
                hintText: 'My Virtual World',
                prefixIcon: Icon(Icons.label),
                border: OutlineInputBorder(),
                helperText: 'Displayed to users and in viewer grid list',
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'http://localhost:9000',
              decoration: const InputDecoration(
                labelText: 'Public Base URL',
                hintText: 'http://grid.example.com:9000',
                prefixIcon: Icon(Icons.link),
                border: OutlineInputBorder(),
                helperText: 'URL region servers use to connect',
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'postgresql://opensim@localhost/opensim_grid',
              decoration: const InputDecoration(
                labelText: 'Database URL',
                hintText: 'postgresql://user@host/database',
                prefixIcon: Icon(Icons.storage),
                border: OutlineInputBorder(),
                helperText: 'Database for users, inventory, and assets',
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRegionServerConfig(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.orange[50],
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.orange[200]!),
              ),
              child: Row(
                children: [
                  Icon(Icons.info, color: Colors.orange[700], size: 20),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Region Server Mode',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Colors.orange[900],
                          ),
                        ),
                        Text(
                          'This server runs region simulators and connects to an existing grid server for user services.',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.orange[700],
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'http://localhost:9000',
              decoration: const InputDecoration(
                labelText: 'Grid Server URL',
                hintText: 'http://grid.example.com:9000',
                prefixIcon: Icon(Icons.dns),
                border: OutlineInputBorder(),
                helperText: 'URL of the grid server to connect to',
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'My Region',
              decoration: const InputDecoration(
                labelText: 'Region Name',
                hintText: 'Welcome Island',
                prefixIcon: Icon(Icons.map),
                border: OutlineInputBorder(),
                helperText: 'Name displayed on the grid map',
              ),
            ),
            const SizedBox(height: 16),
            TextFormField(
              initialValue: 'postgresql://opensim@localhost/opensim_region',
              decoration: const InputDecoration(
                labelText: 'Database URL',
                hintText: 'postgresql://user@host/database',
                prefixIcon: Icon(Icons.storage),
                border: OutlineInputBorder(),
                helperText: 'Local database for prims and terrain only',
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildModeDetails(BuildContext context) {
    final info = ArchitectureModeInfo.getInfo(selectedMode);
    if (info == null) return const SizedBox.shrink();

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  info.icon,
                  style: const TextStyle(fontSize: 24),
                ),
                const SizedBox(width: 12),
                Text(
                  info.name,
                  style: const TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 18,
                  ),
                ),
              ],
            ),
            const Divider(),
            const Text(
              'Database Tables Created:',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            _buildTablesList(),
            const SizedBox(height: 16),
            const Text(
              'Features:',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            ...info.features.map((f) => Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Row(
                children: [
                  const Icon(Icons.check, color: Colors.green, size: 16),
                  const SizedBox(width: 8),
                  Expanded(child: Text(f, style: const TextStyle(fontSize: 13))),
                ],
              ),
            )),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey[100],
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  const Icon(Icons.schedule, size: 16),
                  const SizedBox(width: 8),
                  Text(
                    'Setup time: ${info.setupTime}',
                    style: const TextStyle(fontSize: 13),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTablesList() {
    final tables = _getTablesForMode();
    return Wrap(
      spacing: 8,
      runSpacing: 4,
      children: tables.map((table) => Chip(
        label: Text(table),
        labelStyle: const TextStyle(fontSize: 11),
        backgroundColor: _getTableColor(table),
        padding: EdgeInsets.zero,
        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      )).toList(),
    );
  }

  List<String> _getTablesForMode() {
    switch (selectedMode) {
      case ArchitectureMode.standalone:
        return [
          'useraccounts', 'auth', 'inventoryfolders', 'inventoryitems',
          'assets', 'avatars', 'griduser', 'presence', 'friends', 'regions',
          'prims', 'primshapes', 'terrain', 'land', 'regionsettings',
        ];
      case ArchitectureMode.gridServer:
        return [
          'useraccounts', 'auth', 'inventoryfolders', 'inventoryitems',
          'assets', 'avatars', 'griduser', 'presence', 'friends', 'regions',
        ];
      case ArchitectureMode.regionServer:
        return [
          'prims', 'primshapes', 'terrain', 'land', 'landaccesslist',
          'regionsettings', 'regionenvironment', 'spawnpoints',
        ];
    }
  }

  Color _getTableColor(String table) {
    final gridTables = {'useraccounts', 'auth', 'inventoryfolders', 'inventoryitems',
        'assets', 'avatars', 'griduser', 'presence', 'friends', 'regions'};
    final regionTables = {'prims', 'primshapes', 'terrain', 'land', 'landaccesslist',
        'regionsettings', 'regionenvironment', 'spawnpoints'};

    if (gridTables.contains(table)) {
      return Colors.blue[100]!;
    } else if (regionTables.contains(table)) {
      return Colors.orange[100]!;
    }
    return Colors.grey[200]!;
  }
}
