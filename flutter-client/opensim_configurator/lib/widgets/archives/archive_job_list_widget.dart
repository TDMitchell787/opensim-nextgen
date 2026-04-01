import 'package:flutter/material.dart';
import '../../models/archive_models.dart';

class ArchiveJobListWidget extends StatelessWidget {
  final List<ArchiveJob> jobs;
  final ArchiveJob? selectedJob;
  final Function(ArchiveJob) onJobSelected;
  final Function(ArchiveJob)? onJobCancelled;

  const ArchiveJobListWidget({
    super.key,
    required this.jobs,
    this.selectedJob,
    required this.onJobSelected,
    this.onJobCancelled,
  });

  @override
  Widget build(BuildContext context) {
    if (jobs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.archive_outlined, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'No Archive Jobs',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'Start an IAR or OAR operation to see jobs here.',
              style: TextStyle(color: Colors.grey[600]),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: jobs.length,
      itemBuilder: (context, index) {
        final job = jobs[index];
        final isSelected = selectedJob?.id == job.id;

        return Card(
          margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          color: isSelected
              ? Theme.of(context).colorScheme.primaryContainer
              : null,
          child: ListTile(
            leading: _buildStatusIcon(job),
            title: Text(
              job.displayName,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            subtitle: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (job.targetName != null)
                  Text(job.targetName!,
                      style: TextStyle(color: Colors.grey[600])),
                if (job.isActive) ...[
                  const SizedBox(height: 4),
                  LinearProgressIndicator(value: job.progress),
                  const SizedBox(height: 2),
                  Text(
                    job.progressMessage ?? '${(job.progress * 100).toStringAsFixed(0)}%',
                    style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                  ),
                ] else ...[
                  Text(
                    '${_statusLabel(job.status)} - ${job.elapsedFormatted}',
                    style: TextStyle(
                      fontSize: 12,
                      color: _statusColor(job.status),
                    ),
                  ),
                ],
              ],
            ),
            trailing: job.isActive && onJobCancelled != null
                ? IconButton(
                    icon: const Icon(Icons.cancel, color: Colors.orange),
                    onPressed: () => onJobCancelled!(job),
                    tooltip: 'Cancel Job',
                  )
                : _buildStatusBadge(job),
            onTap: () => onJobSelected(job),
          ),
        );
      },
    );
  }

  Widget _buildStatusIcon(ArchiveJob job) {
    IconData icon;
    Color color;

    if (job.archiveType == ArchiveType.iar) {
      icon = Icons.inventory_2;
    } else {
      icon = Icons.landscape;
    }

    switch (job.status) {
      case JobStatus.queued:
        color = Colors.grey;
        break;
      case JobStatus.running:
        color = Colors.blue;
        break;
      case JobStatus.completed:
        color = Colors.green;
        break;
      case JobStatus.failed:
        color = Colors.red;
        break;
      case JobStatus.cancelled:
        color = Colors.orange;
        break;
    }

    return CircleAvatar(
      backgroundColor: color.withOpacity(0.2),
      child: Icon(icon, color: color),
    );
  }

  Widget _buildStatusBadge(ArchiveJob job) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _statusColor(job.status).withOpacity(0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        _statusLabel(job.status),
        style: TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.bold,
          color: _statusColor(job.status),
        ),
      ),
    );
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

  Color _statusColor(JobStatus status) {
    switch (status) {
      case JobStatus.queued:
        return Colors.grey;
      case JobStatus.running:
        return Colors.blue;
      case JobStatus.completed:
        return Colors.green;
      case JobStatus.failed:
        return Colors.red;
      case JobStatus.cancelled:
        return Colors.orange;
    }
  }
}
