import 'package:flutter/material.dart';
import '../../models/configuration_builder_models.dart';

class SystemRequirementsCard extends StatelessWidget {
  final SystemRequirements requirements;
  final String? title;
  final bool showNotes;
  final bool compact;

  const SystemRequirementsCard({
    super.key,
    required this.requirements,
    this.title,
    this.showNotes = true,
    this.compact = false,
  });

  @override
  Widget build(BuildContext context) {
    if (compact) {
      return _buildCompactCard(context);
    }
    return _buildFullCard(context);
  }

  Widget _buildFullCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (title != null) ...[
              Text(
                title!,
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 16),
            ],
            Row(
              children: [
                Expanded(
                  child: _buildRequirementItem(
                    context,
                    icon: Icons.memory,
                    label: 'Memory',
                    value: requirements.memoryDisplay,
                    subValue: 'min: ${(requirements.minMemoryMB / 1024).toStringAsFixed(1)}GB',
                    color: const Color(0xFF3B82F6),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: _buildRequirementItem(
                    context,
                    icon: Icons.developer_board,
                    label: 'CPU',
                    value: requirements.cpuDisplay,
                    subValue: 'min: ${requirements.minCpuCores} cores',
                    color: const Color(0xFF10B981),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildRequirementItem(
                    context,
                    icon: Icons.network_check,
                    label: 'Network',
                    value: requirements.networkDisplay,
                    subValue: 'recommended',
                    color: const Color(0xFF8B5CF6),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: _buildRequirementItem(
                    context,
                    icon: Icons.storage,
                    label: 'Disk',
                    value: requirements.diskDisplay,
                    subValue: 'minimum',
                    color: const Color(0xFFF59E0B),
                  ),
                ),
              ],
            ),
            if (showNotes && requirements.notes != null) ...[
              const SizedBox(height: 16),
              const Divider(),
              const SizedBox(height: 12),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(
                    Icons.info_outline,
                    size: 16,
                    color: Colors.grey[600],
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      requirements.notes!,
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.grey[600],
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildCompactCard(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey[300]!),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildCompactItem(Icons.memory, requirements.memoryDisplay),
          _buildCompactDivider(),
          _buildCompactItem(Icons.developer_board, requirements.cpuDisplay),
          _buildCompactDivider(),
          _buildCompactItem(Icons.network_check, requirements.networkDisplay),
          _buildCompactDivider(),
          _buildCompactItem(Icons.storage, requirements.diskDisplay),
        ],
      ),
    );
  }

  Widget _buildRequirementItem(
    BuildContext context, {
    required IconData icon,
    required String label,
    required String value,
    required String subValue,
    required Color color,
  }) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 18, color: color),
              const SizedBox(width: 8),
              Text(
                label,
                style: TextStyle(
                  fontSize: 12,
                  color: Colors.grey[600],
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            value,
            style: TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.bold,
              color: color,
            ),
          ),
          const SizedBox(height: 2),
          Text(
            subValue,
            style: TextStyle(
              fontSize: 10,
              color: Colors.grey[500],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCompactItem(IconData icon, String value) {
    return Row(
      children: [
        Icon(icon, size: 14, color: Colors.grey[600]),
        const SizedBox(width: 4),
        Text(
          value,
          style: const TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w500,
          ),
        ),
      ],
    );
  }

  Widget _buildCompactDivider() {
    return Container(
      height: 20,
      width: 1,
      color: Colors.grey[300],
    );
  }
}

class SystemRequirementsBadge extends StatelessWidget {
  final SystemRequirements requirements;

  const SystemRequirementsBadge({
    super.key,
    required this.requirements,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _buildBadge(Icons.memory, requirements.memoryDisplay),
        const SizedBox(width: 8),
        _buildBadge(Icons.developer_board, requirements.cpuDisplay),
      ],
    );
  }

  Widget _buildBadge(IconData icon, String value) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.grey[100],
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 12, color: Colors.grey[600]),
          const SizedBox(width: 4),
          Text(
            value,
            style: TextStyle(
              fontSize: 11,
              color: Colors.grey[700],
            ),
          ),
        ],
      ),
    );
  }
}

class SystemRequirementsExpanded extends StatelessWidget {
  final SystemRequirements requirements;

  const SystemRequirementsExpanded({
    super.key,
    required this.requirements,
  });

  @override
  Widget build(BuildContext context) {
    return ExpansionTile(
      leading: const Icon(Icons.settings_system_daydream),
      title: const Text('System Requirements'),
      subtitle: Text(
        '${requirements.memoryDisplay} RAM, ${requirements.cpuDisplay}',
        style: TextStyle(fontSize: 12, color: Colors.grey[600]),
      ),
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              _buildExpandedRow('Memory (Recommended)', requirements.memoryDisplay),
              _buildExpandedRow('Memory (Minimum)', '${(requirements.minMemoryMB / 1024).toStringAsFixed(1)}GB'),
              _buildExpandedRow('CPU Cores (Recommended)', requirements.recommendedCpuCores.toString()),
              _buildExpandedRow('CPU Cores (Minimum)', requirements.minCpuCores.toString()),
              _buildExpandedRow('Network Bandwidth', requirements.networkDisplay),
              _buildExpandedRow('Disk Space', requirements.diskDisplay),
              if (requirements.notes != null) ...[
                const SizedBox(height: 8),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Colors.blue[50],
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(Icons.info, size: 16, color: Colors.blue[700]),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          requirements.notes!,
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.blue[800],
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildExpandedRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: TextStyle(
              fontSize: 13,
              color: Colors.grey[600],
            ),
          ),
          Text(
            value,
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }
}
