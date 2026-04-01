import 'package:flutter/material.dart';
import '../../models/archive_models.dart';

class ArchiveJobDetailsWidget extends StatelessWidget {
  final ArchiveJob job;
  final VoidCallback? onCancel;
  final VoidCallback? onDownload;

  const ArchiveJobDetailsWidget({
    super.key,
    required this.job,
    this.onCancel,
    this.onDownload,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildHeader(context),
          const SizedBox(height: 24),
          _buildStatusCard(context),
          const SizedBox(height: 16),
          _buildDetailsCard(context),
          if (job.result != null) ...[
            const SizedBox(height: 16),
            _buildResultCard(context),
          ],
          if (job.error != null) ...[
            const SizedBox(height: 16),
            _buildErrorCard(context),
          ],
          const SizedBox(height: 16),
          _buildActionsCard(context),
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        _buildTypeIcon(),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                job.displayName,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              if (job.targetName != null)
                Text(
                  job.targetName!,
                  style: TextStyle(color: Colors.grey[600]),
                ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildTypeIcon() {
    final icon = job.archiveType == ArchiveType.iar
        ? Icons.inventory_2
        : Icons.landscape;
    final color = job.archiveType == ArchiveType.iar
        ? Colors.purple
        : Colors.teal;

    return CircleAvatar(
      radius: 32,
      backgroundColor: color.withOpacity(0.2),
      child: Icon(icon, size: 32, color: color),
    );
  }

  Widget _buildStatusCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.info_outline),
                const SizedBox(width: 8),
                Text(
                  'Status',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const Divider(),
            if (job.isActive) ...[
              const SizedBox(height: 8),
              LinearProgressIndicator(value: job.progress),
              const SizedBox(height: 8),
              Text(
                job.progressMessage ?? 'Processing...',
                style: TextStyle(color: Colors.grey[600]),
              ),
              Text(
                '${(job.progress * 100).toStringAsFixed(1)}% complete',
                style: const TextStyle(fontWeight: FontWeight.bold),
              ),
            ] else ...[
              ListTile(
                leading: _buildStatusIcon(),
                title: Text(_statusLabel(job.status)),
                subtitle: Text('Elapsed: ${job.elapsedFormatted}'),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildDetailsCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.list_alt),
                const SizedBox(width: 8),
                Text(
                  'Details',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const Divider(),
            _buildDetailRow('Job ID', job.id),
            _buildDetailRow('Type', job.typeLabel),
            _buildDetailRow('Operation', job.operationLabel),
            if (job.targetId != null)
              _buildDetailRow('Target ID', job.targetId!),
            if (job.filePath != null)
              _buildDetailRow('File Path', job.filePath!),
            _buildDetailRow('Created', _formatDateTime(job.createdAt)),
            if (job.startedAt != null)
              _buildDetailRow('Started', _formatDateTime(job.startedAt!)),
            if (job.completedAt != null)
              _buildDetailRow('Completed', _formatDateTime(job.completedAt!)),
          ],
        ),
      ),
    );
  }

  Widget _buildResultCard(BuildContext context) {
    final result = job.result!;

    return Card(
      color: Colors.green[50],
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.green),
                const SizedBox(width: 8),
                Text(
                  'Results',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const Divider(),
            if (result.assetsProcessed != null)
              _buildDetailRow('Assets', '${result.assetsProcessed}'),
            if (result.foldersProcessed != null)
              _buildDetailRow('Folders', '${result.foldersProcessed}'),
            if (result.itemsProcessed != null)
              _buildDetailRow('Items', '${result.itemsProcessed}'),
            if (result.objectsProcessed != null)
              _buildDetailRow('Objects', '${result.objectsProcessed}'),
            if (result.parcelsProcessed != null)
              _buildDetailRow('Parcels', '${result.parcelsProcessed}'),
            if (result.terrainProcessed != null)
              _buildDetailRow('Terrain', result.terrainProcessed! ? 'Yes' : 'No'),
            if (result.archiveSizeBytes != null)
              _buildDetailRow('Archive Size', result.archiveSizeFormatted),
          ],
        ),
      ),
    );
  }

  Widget _buildErrorCard(BuildContext context) {
    return Card(
      color: Colors.red[50],
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.error_outline, color: Colors.red),
                const SizedBox(width: 8),
                Text(
                  'Error',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const Divider(),
            SelectableText(
              job.error!,
              style: const TextStyle(color: Colors.red),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActionsCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.play_arrow),
                const SizedBox(width: 8),
                Text(
                  'Actions',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const Divider(),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                if (job.isActive && onCancel != null)
                  ElevatedButton.icon(
                    onPressed: onCancel,
                    icon: const Icon(Icons.cancel),
                    label: const Text('Cancel Job'),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.orange,
                      foregroundColor: Colors.white,
                    ),
                  ),
                if (job.isComplete &&
                    job.operation == JobOperation.save &&
                    onDownload != null)
                  ElevatedButton.icon(
                    onPressed: onDownload,
                    icon: const Icon(Icons.download),
                    label: const Text('Download Archive'),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.green,
                      foregroundColor: Colors.white,
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDetailRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(
            child: SelectableText(value),
          ),
        ],
      ),
    );
  }

  Widget _buildStatusIcon() {
    Color color;
    IconData icon;

    switch (job.status) {
      case JobStatus.queued:
        color = Colors.grey;
        icon = Icons.schedule;
        break;
      case JobStatus.running:
        color = Colors.blue;
        icon = Icons.sync;
        break;
      case JobStatus.completed:
        color = Colors.green;
        icon = Icons.check_circle;
        break;
      case JobStatus.failed:
        color = Colors.red;
        icon = Icons.error;
        break;
      case JobStatus.cancelled:
        color = Colors.orange;
        icon = Icons.cancel;
        break;
    }

    return Icon(icon, color: color, size: 32);
  }

  String _statusLabel(JobStatus status) {
    switch (status) {
      case JobStatus.queued:
        return 'Queued';
      case JobStatus.running:
        return 'Running';
      case JobStatus.completed:
        return 'Completed';
      case JobStatus.failed:
        return 'Failed';
      case JobStatus.cancelled:
        return 'Cancelled';
    }
  }

  String _formatDateTime(DateTime dt) {
    return '${dt.year}-${dt.month.toString().padLeft(2, '0')}-${dt.day.toString().padLeft(2, '0')} '
        '${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}:${dt.second.toString().padLeft(2, '0')}';
  }
}
