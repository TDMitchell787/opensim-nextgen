import 'package:flutter/material.dart';
import '../../models/user_models.dart';

class AppearanceDiagnosticsWidget extends StatelessWidget {
  final String userId;
  final String userName;
  final AppearanceDiagnostics? diagnostics;
  final bool isLoading;
  final VoidCallback onRefresh;
  final VoidCallback onRepair;
  final VoidCallback onReset;
  final VoidCallback onRebuildInventory;

  const AppearanceDiagnosticsWidget({
    super.key,
    required this.userId,
    required this.userName,
    this.diagnostics,
    required this.isLoading,
    required this.onRefresh,
    required this.onRepair,
    required this.onReset,
    required this.onRebuildInventory,
  });

  @override
  Widget build(BuildContext context) {
    if (isLoading && diagnostics == null) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text('Loading diagnostics...'),
          ],
        ),
      );
    }

    if (diagnostics == null) {
      return _buildNoDiagnosticsState(context);
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildHeader(context),
        const SizedBox(height: 16),
        _buildOverviewCard(context),
        const SizedBox(height: 16),
        _buildWearablesCard(context),
        const SizedBox(height: 16),
        _buildFoldersCard(context),
        const SizedBox(height: 16),
        if (diagnostics!.missingFolders.isNotEmpty ||
            diagnostics!.missingWearables.isNotEmpty ||
            diagnostics!.invalidItems.isNotEmpty)
          _buildIssuesCard(context),
        const SizedBox(height: 16),
        _buildActionsCard(context),
      ],
    );
  }

  Widget _buildNoDiagnosticsState(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.medical_information_outlined,
              size: 64, color: Colors.grey[400]),
          const SizedBox(height: 16),
          Text(
            'No Diagnostics Available',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Text(
            'Run diagnostics to check appearance health.',
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: onRefresh,
            icon: const Icon(Icons.play_arrow),
            label: const Text('Run Diagnostics'),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: _getStatusColor().withOpacity(0.1),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Icon(
            _getStatusIcon(),
            color: _getStatusColor(),
            size: 32,
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Appearance: $userName',
                style: Theme.of(context).textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              const SizedBox(height: 4),
              _buildStatusBadge(),
            ],
          ),
        ),
        IconButton(
          onPressed: onRefresh,
          icon: const Icon(Icons.refresh),
          tooltip: 'Refresh Diagnostics',
        ),
      ],
    );
  }

  Widget _buildOverviewCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Overview',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildProgressMetric(
                    'Folders',
                    diagnostics!.actualFolderCount,
                    diagnostics!.expectedFolderCount,
                    diagnostics!.folderCompleteness,
                    Colors.blue,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: _buildProgressMetric(
                    'Wearables',
                    diagnostics!.actualWearableCount,
                    diagnostics!.expectedWearableCount,
                    diagnostics!.wearableCompleteness,
                    Colors.purple,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                _buildCheckItem(
                  'Body Parts',
                  diagnostics!.hasBodyParts,
                  'Shape, Skin, Hair, Eyes',
                ),
                const SizedBox(width: 24),
                _buildCheckItem(
                  'Clothing',
                  diagnostics!.hasClothing,
                  'Shirt, Pants',
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(
              'Last checked: ${_formatDateTime(diagnostics!.checkedAt)}',
              style: TextStyle(color: Colors.grey[500], fontSize: 12),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildProgressMetric(
    String label,
    int actual,
    int expected,
    double progress,
    Color color,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: const TextStyle(fontWeight: FontWeight.w500)),
            Text(
              '$actual / $expected',
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: progress >= 1.0 ? Colors.green : Colors.orange,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: progress.clamp(0.0, 1.0),
            backgroundColor: Colors.grey[200],
            valueColor: AlwaysStoppedAnimation<Color>(
              progress >= 1.0 ? Colors.green : color,
            ),
            minHeight: 8,
          ),
        ),
      ],
    );
  }

  Widget _buildCheckItem(String label, bool isOk, String description) {
    return Row(
      children: [
        Icon(
          isOk ? Icons.check_circle : Icons.cancel,
          color: isOk ? Colors.green : Colors.red,
          size: 20,
        ),
        const SizedBox(width: 8),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: const TextStyle(fontWeight: FontWeight.w500)),
            Text(
              description,
              style: TextStyle(color: Colors.grey[600], fontSize: 11),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildWearablesCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Wearables',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            if (diagnostics!.wearables.isEmpty)
              Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    children: [
                      Icon(Icons.warning_amber,
                          size: 48, color: Colors.orange[400]),
                      const SizedBox(height: 8),
                      const Text('No wearables found'),
                    ],
                  ),
                ),
              )
            else
              ...diagnostics!.wearables.map((w) => _buildWearableRow(w)),
          ],
        ),
      ),
    );
  }

  Widget _buildWearableRow(WearableEntry wearable) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: wearable.isValid ? Colors.green : Colors.red,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 12),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: wearable.isBodyPart
                  ? Colors.blue.withOpacity(0.1)
                  : Colors.purple.withOpacity(0.1),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              wearable.typeName,
              style: TextStyle(
                fontSize: 12,
                color: wearable.isBodyPart ? Colors.blue : Colors.purple,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              wearable.name ?? 'Default ${wearable.typeName}',
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (!wearable.isValid)
            const Tooltip(
              message: 'Item or asset missing',
              child: Icon(Icons.warning, color: Colors.orange, size: 16),
            ),
        ],
      ),
    );
  }

  Widget _buildFoldersCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Inventory Folders',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
                Text(
                  '${diagnostics!.actualFolderCount} / ${diagnostics!.expectedFolderCount}',
                  style: TextStyle(
                    color: diagnostics!.folderCompleteness >= 1.0
                        ? Colors.green
                        : Colors.orange,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            if (diagnostics!.folders.isEmpty)
              Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    children: [
                      Icon(Icons.folder_off, size: 48, color: Colors.red[400]),
                      const SizedBox(height: 8),
                      const Text('No folders found - Critical issue!'),
                    ],
                  ),
                ),
              )
            else
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: diagnostics!.folders
                    .map((f) => _buildFolderChip(f))
                    .toList(),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildFolderChip(InventoryFolder folder) {
    return Chip(
      label: Text(
        folder.folderName,
        style: const TextStyle(fontSize: 12),
      ),
      avatar: Icon(
        _getFolderIcon(folder.type),
        size: 16,
      ),
      backgroundColor: Colors.grey[100],
    );
  }

  IconData _getFolderIcon(int type) {
    switch (type) {
      case 0:
        return Icons.image;
      case 1:
        return Icons.audiotrack;
      case 5:
        return Icons.checkroom;
      case 6:
        return Icons.category;
      case 7:
        return Icons.note;
      case 8:
        return Icons.folder;
      case 13:
        return Icons.accessibility;
      case 14:
        return Icons.delete;
      case 20:
        return Icons.animation;
      case 21:
        return Icons.gesture;
      case 47:
        return Icons.style;
      default:
        return Icons.folder;
    }
  }

  Widget _buildIssuesCard(BuildContext context) {
    return Card(
      color: Colors.orange.withOpacity(0.05),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.warning_amber, color: Colors.orange),
                const SizedBox(width: 8),
                Text(
                  'Issues Detected',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                        color: Colors.orange[800],
                      ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            if (diagnostics!.missingFolders.isNotEmpty) ...[
              _buildIssueSection(
                'Missing Folders',
                diagnostics!.missingFolders,
                Icons.folder_off,
                Colors.red,
              ),
              const SizedBox(height: 12),
            ],
            if (diagnostics!.missingWearables.isNotEmpty) ...[
              _buildIssueSection(
                'Missing Wearables',
                diagnostics!.missingWearables,
                Icons.checkroom,
                Colors.orange,
              ),
              const SizedBox(height: 12),
            ],
            if (diagnostics!.invalidItems.isNotEmpty)
              _buildIssueSection(
                'Invalid Items',
                diagnostics!.invalidItems,
                Icons.error_outline,
                Colors.red,
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildIssueSection(
    String title,
    List<String> items,
    IconData icon,
    Color color,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, size: 16, color: color),
            const SizedBox(width: 8),
            Text(
              title,
              style: TextStyle(fontWeight: FontWeight.w600, color: color),
            ),
          ],
        ),
        const SizedBox(height: 4),
        ...items.map((item) => Padding(
              padding: const EdgeInsets.only(left: 24, top: 4),
              child: Text('- $item', style: TextStyle(color: Colors.grey[700])),
            )),
      ],
    );
  }

  Widget _buildActionsCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Repair Actions',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                OutlinedButton.icon(
                  onPressed: onRepair,
                  icon: const Icon(Icons.build),
                  label: const Text('Auto-Repair'),
                ),
                OutlinedButton.icon(
                  onPressed: onRebuildInventory,
                  icon: const Icon(Icons.folder_copy),
                  label: const Text('Rebuild Folders'),
                ),
                OutlinedButton.icon(
                  onPressed: onReset,
                  icon: const Icon(Icons.restart_alt, color: Colors.orange),
                  label: const Text('Reset to Defaults',
                      style: TextStyle(color: Colors.orange)),
                  style: OutlinedButton.styleFrom(
                    side: const BorderSide(color: Colors.orange),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(
              'Auto-Repair: Fixes missing items while preserving existing customizations.\n'
              'Reset to Defaults: Completely resets appearance to Ruth defaults.',
              style: TextStyle(color: Colors.grey[600], fontSize: 12),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusBadge() {
    final color = _getStatusColor();
    final label = _getStatusLabel();

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }

  Color _getStatusColor() {
    if (diagnostics == null) return Colors.grey;
    switch (diagnostics!.status) {
      case AppearanceStatus.complete:
        return Colors.green;
      case AppearanceStatus.incomplete:
        return Colors.orange;
      case AppearanceStatus.missing:
        return Colors.red;
      case AppearanceStatus.error:
        return Colors.purple;
    }
  }

  IconData _getStatusIcon() {
    if (diagnostics == null) return Icons.help_outline;
    switch (diagnostics!.status) {
      case AppearanceStatus.complete:
        return Icons.check_circle;
      case AppearanceStatus.incomplete:
        return Icons.warning;
      case AppearanceStatus.missing:
        return Icons.cancel;
      case AppearanceStatus.error:
        return Icons.error;
    }
  }

  String _getStatusLabel() {
    if (diagnostics == null) return 'Unknown';
    switch (diagnostics!.status) {
      case AppearanceStatus.complete:
        return 'Healthy';
      case AppearanceStatus.incomplete:
        return 'Incomplete';
      case AppearanceStatus.missing:
        return 'Missing';
      case AppearanceStatus.error:
        return 'Error';
    }
  }

  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}-${dateTime.month.toString().padLeft(2, '0')}-'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
