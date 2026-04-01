import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/configuration_builder_models.dart';
import '../../providers/configuration_builder_provider.dart';
import 'system_requirements_card.dart';

class TemplateSelector extends StatelessWidget {
  final bool showSystemRequirements;
  final ValueChanged<SimulatorTemplate>? onTemplateSelected;

  const TemplateSelector({
    super.key,
    this.showSystemRequirements = true,
    this.onTemplateSelected,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ConfigurationBuilderProvider>();
    final templates = provider.builtInTemplates;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Simulator Templates',
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            itemCount: templates.length,
            itemBuilder: (context, index) {
              final template = templates[index];
              final isSelected = provider.selectedTemplate?.id == template.id;
              final typeInfo = SimulatorTypeInfo.getInfo(template.templateType);

              return _TemplateCard(
                template: template,
                typeInfo: typeInfo,
                isSelected: isSelected,
                showRequirements: showSystemRequirements,
                onTap: () {
                  provider.selectTemplate(template);
                  onTemplateSelected?.call(template);
                },
              );
            },
          ),
        ),
      ],
    );
  }
}

class _TemplateCard extends StatelessWidget {
  final SimulatorTemplate template;
  final SimulatorTypeInfo typeInfo;
  final bool isSelected;
  final bool showRequirements;
  final VoidCallback onTap;

  const _TemplateCard({
    required this.template,
    required this.typeInfo,
    required this.isSelected,
    required this.showRequirements,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.read<ConfigurationBuilderProvider>();
    final color = provider.getSimulatorTypeColor(template.templateType);

    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      elevation: isSelected ? 4 : 1,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: isSelected
            ? BorderSide(color: color, width: 2)
            : BorderSide.none,
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 40,
                    height: 40,
                    decoration: BoxDecoration(
                      color: color.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Icon(
                      _getIconData(typeInfo.icon),
                      color: color,
                      size: 24,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          template.name,
                          style: const TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                          ),
                        ),
                        if (showRequirements) ...[
                          const SizedBox(height: 2),
                          Text(
                            '${template.systemRequirements.memoryDisplay} / ${template.systemRequirements.cpuDisplay}',
                            style: TextStyle(
                              fontSize: 11,
                              color: Colors.grey[600],
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                  if (isSelected)
                    Icon(Icons.check_circle, color: color, size: 20),
                ],
              ),
              if (!showRequirements) ...[
                const SizedBox(height: 8),
                Text(
                  template.description,
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.grey[600],
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  IconData _getIconData(String iconName) {
    switch (iconName) {
      case 'terrain':
        return Icons.terrain;
      case 'beach_access':
        return Icons.beach_access;
      case 'directions_boat':
        return Icons.directions_boat;
      case 'construction':
        return Icons.construction;
      case 'waving_hand':
        return Icons.waving_hand;
      case 'celebration':
        return Icons.celebration;
      case 'shopping_cart':
        return Icons.shopping_cart;
      case 'theater_comedy':
        return Icons.theater_comedy;
      case 'home':
        return Icons.home;
      case 'grid_off':
        return Icons.grid_off;
      case 'landscape':
        return Icons.landscape;
      default:
        return Icons.public;
    }
  }
}

class TemplateSelectorGrid extends StatelessWidget {
  final int crossAxisCount;
  final ValueChanged<SimulatorTemplate>? onTemplateSelected;

  const TemplateSelectorGrid({
    super.key,
    this.crossAxisCount = 3,
    this.onTemplateSelected,
  });

  @override
  Widget build(BuildContext context) {
    final provider = context.watch<ConfigurationBuilderProvider>();
    final templates = provider.builtInTemplates;

    return GridView.builder(
      padding: const EdgeInsets.all(16),
      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: crossAxisCount,
        childAspectRatio: 1.5,
        crossAxisSpacing: 12,
        mainAxisSpacing: 12,
      ),
      itemCount: templates.length,
      itemBuilder: (context, index) {
        final template = templates[index];
        final isSelected = provider.selectedTemplate?.id == template.id;
        final color = provider.getSimulatorTypeColor(template.templateType);
        final typeInfo = SimulatorTypeInfo.getInfo(template.templateType);

        return _TemplateGridCard(
          template: template,
          typeInfo: typeInfo,
          isSelected: isSelected,
          color: color,
          onTap: () {
            provider.selectTemplate(template);
            onTemplateSelected?.call(template);
          },
        );
      },
    );
  }
}

class _TemplateGridCard extends StatelessWidget {
  final SimulatorTemplate template;
  final SimulatorTypeInfo typeInfo;
  final bool isSelected;
  final Color color;
  final VoidCallback onTap;

  const _TemplateGridCard({
    required this.template,
    required this.typeInfo,
    required this.isSelected,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: isSelected ? 4 : 1,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: isSelected
            ? BorderSide(color: color, width: 2)
            : BorderSide.none,
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 48,
                    height: 48,
                    decoration: BoxDecoration(
                      color: color.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Icon(
                      _getIconData(typeInfo.icon),
                      color: color,
                      size: 28,
                    ),
                  ),
                  const Spacer(),
                  if (isSelected)
                    Icon(Icons.check_circle, color: color, size: 24),
                ],
              ),
              const Spacer(),
              Text(
                template.name,
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 16,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                template.description,
                style: TextStyle(
                  fontSize: 12,
                  color: Colors.grey[600],
                ),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
              const Spacer(),
              Row(
                children: [
                  _buildRequirementChip(
                    Icons.memory,
                    template.systemRequirements.memoryDisplay,
                  ),
                  const SizedBox(width: 8),
                  _buildRequirementChip(
                    Icons.developer_board,
                    template.systemRequirements.cpuDisplay,
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildRequirementChip(IconData icon, String label) {
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
            label,
            style: TextStyle(
              fontSize: 10,
              color: Colors.grey[700],
            ),
          ),
        ],
      ),
    );
  }

  IconData _getIconData(String iconName) {
    switch (iconName) {
      case 'terrain':
        return Icons.terrain;
      case 'beach_access':
        return Icons.beach_access;
      case 'directions_boat':
        return Icons.directions_boat;
      case 'construction':
        return Icons.construction;
      case 'waving_hand':
        return Icons.waving_hand;
      case 'celebration':
        return Icons.celebration;
      case 'shopping_cart':
        return Icons.shopping_cart;
      case 'theater_comedy':
        return Icons.theater_comedy;
      case 'home':
        return Icons.home;
      case 'grid_off':
        return Icons.grid_off;
      case 'landscape':
        return Icons.landscape;
      default:
        return Icons.public;
    }
  }
}
