import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/instance_directory_provider.dart';
import '../widgets/instances/robust_server_panel.dart';
import '../widgets/instances/instance_launcher_widget.dart';

class InstanceDirectoryScreen extends StatelessWidget {
  const InstanceDirectoryScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<InstanceDirectoryProvider>();
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Instance Directory'),
        actions: [
          IconButton(
            onPressed: () => provider.scanInstances(),
            icon: provider.isScanning
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.refresh),
            tooltip: 'Rescan',
          ),
        ],
      ),
      body: provider.entries.isEmpty
          ? _buildEmptyState(theme)
          : _buildInstanceList(context, provider),
    );
  }

  Widget _buildEmptyState(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.folder_off, size: 64, color: theme.disabledColor),
          const SizedBox(height: 16),
          Text(
            'No instances found',
            style: theme.textTheme.headlineSmall,
          ),
          const SizedBox(height: 8),
          Text(
            'Create an instance in the Instances/ directory',
            style: theme.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  Widget _buildInstanceList(BuildContext context, InstanceDirectoryProvider provider) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSummaryRow(context, provider),
          const SizedBox(height: 16),
          ...provider.entries.map((entry) => _buildInstanceCard(context, entry)),
        ],
      ),
    );
  }

  Widget _buildSummaryRow(BuildContext context, InstanceDirectoryProvider provider) {
    return Row(
      children: [
        _summaryChip(
          Icons.folder,
          '${provider.totalInstances} Instances',
          Colors.blue,
        ),
        const SizedBox(width: 8),
        _summaryChip(
          Icons.grid_on,
          '${provider.gridInstances} Grid',
          Colors.purple,
        ),
        const SizedBox(width: 8),
        _summaryChip(
          Icons.computer,
          '${provider.standaloneInstances} Standalone',
          Colors.green,
        ),
      ],
    );
  }

  Widget _summaryChip(IconData icon, String label, Color color) {
    return Chip(
      avatar: Icon(icon, size: 16, color: color),
      label: Text(label, style: const TextStyle(fontSize: 12)),
    );
  }

  Widget _buildInstanceCard(BuildContext context, InstanceDirectoryEntry entry) {
    final theme = Theme.of(context);

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: ExpansionTile(
        leading: Icon(entry.modeIcon, color: entry.modeColor),
        title: Text(entry.name, style: const TextStyle(fontWeight: FontWeight.bold)),
        subtitle: Row(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: entry.modeColor.withOpacity(0.2),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                entry.modeDisplay,
                style: TextStyle(fontSize: 11, color: entry.modeColor),
              ),
            ),
            const SizedBox(width: 8),
            Text(
              '${entry.regionCount} regions  |  Port ${entry.loginPort}',
              style: theme.textTheme.bodySmall,
            ),
            if (entry.hypergridEnabled) ...[
              const SizedBox(width: 8),
              const Icon(Icons.language, size: 14, color: Colors.orange),
              const Text(' HG', style: TextStyle(fontSize: 11, color: Colors.orange)),
            ],
          ],
        ),
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _detailRow('Path', entry.path),
                _detailRow('Login Port', entry.loginPort.toString()),
                if (entry.mode == 'grid')
                  _detailRow('Robust Port', entry.robustPort.toString()),
                _detailRow('Regions', entry.regionCount.toString()),
                _detailRow('Database', entry.databaseUrl.isNotEmpty
                    ? entry.databaseUrl.replaceAll(RegExp(r'://.*@'), '://***@')
                    : 'Not configured'),
                if (entry.hypergridEnabled)
                  _detailRow('Home URI', entry.homeUri),
                const SizedBox(height: 12),
                if (entry.mode == 'grid') ...[
                  RobustServerPanel(
                    robustUrl: 'http://localhost:${entry.robustPort}',
                    instanceName: entry.name,
                  ),
                  const SizedBox(height: 8),
                ],
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    if (entry.hasPreflight)
                      OutlinedButton.icon(
                        onPressed: () => _showPreflightDialog(context, entry),
                        icon: const Icon(Icons.checklist, size: 16),
                        label: const Text('Preflight'),
                      ),
                    const SizedBox(width: 8),
                    if (entry.hasStartScript)
                      ElevatedButton.icon(
                        onPressed: () => _showLaunchConfirmation(context, entry),
                        icon: const Icon(Icons.play_arrow, size: 16),
                        label: const Text('Start'),
                      ),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _detailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(label,
                style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 12)),
          ),
          Expanded(
            child: Text(value, style: const TextStyle(fontSize: 12)),
          ),
        ],
      ),
    );
  }

  void _showPreflightDialog(BuildContext context, InstanceDirectoryEntry entry) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Preflight: ${entry.name}'),
        content: const Text(
          'Run preflight checks to verify all prerequisites are met.\n\n'
          'Use the CLI command:\n'
          'opensim-next preflight --instance <name>',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  void _showLaunchConfirmation(BuildContext context, InstanceDirectoryEntry entry) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Start ${entry.name}?'),
        content: Text(
          'Mode: ${entry.modeDisplay}\n'
          'Port: ${entry.loginPort}\n'
          'Regions: ${entry.regionCount}\n\n'
          'Start using the terminal:\n'
          './Instances/${entry.name}/start.sh',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}
