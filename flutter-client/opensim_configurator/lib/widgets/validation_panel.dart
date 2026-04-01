import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/configurator_provider.dart';

class ValidationPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Consumer<ConfiguratorProvider>(
      builder: (context, provider, child) {
        final result = provider.validationResult;

        if (provider.currentConfig == null) {
          return Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Row(
                children: [
                  Icon(Icons.info_outline, color: Colors.grey),
                  SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      'Select a deployment type to enable configuration validation.',
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Colors.grey[600]),
                    ),
                  ),
                ],
              ),
            ),
          );
        }

        if (result == null) {
          return Card(
            child: Padding(
              padding: EdgeInsets.all(16),
              child: Row(
                children: [
                  Icon(Icons.pending_outlined, color: Colors.amber),
                  SizedBox(width: 12),
                  Expanded(
                    child: Text('Validation pending', style: Theme.of(context).textTheme.bodyMedium),
                  ),
                  ElevatedButton.icon(
                    onPressed: provider.isLoading ? null : () => provider.validateConfiguration(),
                    icon: Icon(Icons.play_arrow, size: 18),
                    label: Text('Run Validation'),
                  ),
                ],
              ),
            ),
          );
        }

        final scoreColor = provider.getValidationScoreColor(result.overallScore);

        return Card(
          child: Padding(
            padding: EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    SizedBox(
                      width: 48,
                      height: 48,
                      child: Stack(
                        alignment: Alignment.center,
                        children: [
                          CircularProgressIndicator(
                            value: result.overallScore / 100.0,
                            strokeWidth: 4,
                            backgroundColor: Colors.grey[200],
                            valueColor: AlwaysStoppedAnimation<Color>(scoreColor),
                          ),
                          Text(
                            '${result.overallScore}',
                            style: TextStyle(fontWeight: FontWeight.bold, fontSize: 14, color: scoreColor),
                          ),
                        ],
                      ),
                    ),
                    SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('Configuration Validation', style: Theme.of(context).textTheme.titleMedium),
                          SizedBox(height: 4),
                          _buildStatusChip(context, provider.validationStatus),
                        ],
                      ),
                    ),
                    ElevatedButton.icon(
                      onPressed: provider.isLoading ? null : () => provider.validateConfiguration(),
                      icon: Icon(Icons.refresh, size: 18),
                      label: Text('Re-validate'),
                    ),
                  ],
                ),

                if (result.errors.isNotEmpty) ...[
                  SizedBox(height: 16),
                  _buildSection(context, 'Errors', result.errors, Icons.error, Colors.red),
                ],

                if (result.warnings.isNotEmpty) ...[
                  SizedBox(height: 12),
                  _buildSection(context, 'Warnings', result.warnings, Icons.warning, Colors.amber[700]!),
                ],

                if (result.recommendations.isNotEmpty) ...[
                  SizedBox(height: 12),
                  _buildSection(context, 'Recommendations', result.recommendations, Icons.lightbulb_outline, Colors.blue),
                ],

                if (result.isValid && result.errors.isEmpty && result.warnings.isEmpty) ...[
                  SizedBox(height: 12),
                  Row(
                    children: [
                      Icon(Icons.check_circle, color: Colors.green, size: 20),
                      SizedBox(width: 8),
                      Text('Configuration is valid and ready to apply.',
                          style: TextStyle(color: Colors.green[700])),
                    ],
                  ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildStatusChip(BuildContext context, String status) {
    Color chipColor;
    IconData chipIcon;
    switch (status) {
      case 'Complete':
        chipColor = Colors.green;
        chipIcon = Icons.check_circle;
        break;
      case 'Warning':
        chipColor = Colors.amber[700]!;
        chipIcon = Icons.warning;
        break;
      case 'Error':
        chipColor = Colors.red;
        chipIcon = Icons.error;
        break;
      default:
        chipColor = Colors.grey;
        chipIcon = Icons.pending;
    }
    return Chip(
      avatar: Icon(chipIcon, size: 16, color: chipColor),
      label: Text(status, style: TextStyle(fontSize: 12, color: chipColor)),
      backgroundColor: chipColor.withValues(alpha: 0.1),
      side: BorderSide.none,
      padding: EdgeInsets.zero,
      visualDensity: VisualDensity.compact,
    );
  }

  Widget _buildSection(BuildContext context, String title, List<String> items, IconData icon, Color color) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: TextStyle(fontWeight: FontWeight.w600, fontSize: 13, color: color)),
        SizedBox(height: 4),
        ...items.map((item) => Padding(
          padding: EdgeInsets.only(bottom: 4),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, size: 16, color: color),
              SizedBox(width: 8),
              Expanded(child: Text(item, style: Theme.of(context).textTheme.bodySmall)),
            ],
          ),
        )),
      ],
    );
  }
}
