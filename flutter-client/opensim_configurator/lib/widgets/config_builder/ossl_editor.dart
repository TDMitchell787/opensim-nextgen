import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/configuration_builder_models.dart';
import '../../providers/configuration_builder_provider.dart';

class OsslEditor extends StatelessWidget {
  final OsslConfig config;
  final ValueChanged<OsslConfig>? onChanged;

  const OsslEditor({
    super.key,
    required this.config,
    this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildInfoBanner(context),
          const SizedBox(height: 16),
          _buildThreatLevelGuide(context),
          const SizedBox(height: 24),
          _buildSectionHeader(context, 'Default Threat Level'),
          const SizedBox(height: 12),
          _buildDefaultThreatLevel(context),
          const SizedBox(height: 24),
          _buildSectionHeader(context, 'Function Permissions'),
          const SizedBox(height: 12),
          _buildFunctionPermissions(context),
          const SizedBox(height: 24),
          _buildSectionHeader(context, 'Quick Presets'),
          const SizedBox(height: 12),
          _buildPresets(context),
        ],
      ),
    );
  }

  Widget _buildInfoBanner(BuildContext context) {
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
              Icons.security,
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
                  'OSSL Security Settings',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 18,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Configure which OpenSim Script Language functions are available to scripts. Higher threat levels enable more powerful but potentially dangerous functions.',
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

  Widget _buildThreatLevelGuide(BuildContext context) {
    return Card(
      child: ExpansionTile(
        leading: const Icon(Icons.help_outline),
        title: const Text('Threat Level Guide'),
        subtitle: const Text('Tap to expand'),
        children: [
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                _buildThreatLevelRow(
                  OsslThreatLevel.none,
                  'No OSSL functions enabled',
                  'Most restrictive - only LSL functions work',
                  Colors.grey,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.nuisance,
                  'Minor annoyance functions',
                  'Functions that could be annoying but harmless',
                  Colors.blue,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.veryLow,
                  'Very safe functions',
                  'Useful functions with minimal risk',
                  Colors.green,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.low,
                  'Low risk functions',
                  'Common utility functions, good default',
                  Colors.teal,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.moderate,
                  'Moderate risk functions',
                  'More powerful but require trust',
                  Colors.amber,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.high,
                  'High risk functions',
                  'Powerful functions, trusted users only',
                  Colors.orange,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.veryHigh,
                  'Very high risk functions',
                  'Can affect region performance',
                  Colors.deepOrange,
                ),
                const Divider(),
                _buildThreatLevelRow(
                  OsslThreatLevel.severe,
                  'Severe risk functions',
                  'Can significantly impact server, admin only',
                  Colors.red,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildThreatLevelRow(
    OsslThreatLevel level,
    String title,
    String description,
    Color color,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Container(
            width: 12,
            height: 12,
            decoration: BoxDecoration(
              color: color,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _getThreatLevelName(level),
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
                ),
                Text(
                  title,
                  style: const TextStyle(fontSize: 13),
                ),
                Text(
                  description,
                  style: TextStyle(
                    fontSize: 11,
                    color: Colors.grey[600],
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
        color: Theme.of(context).colorScheme.tertiaryContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            _getSectionIcon(title),
            size: 18,
            color: Theme.of(context).colorScheme.onTertiaryContainer,
          ),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.onTertiaryContainer,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getSectionIcon(String title) {
    switch (title) {
      case 'Default Threat Level':
        return Icons.shield;
      case 'Function Permissions':
        return Icons.functions;
      case 'Quick Presets':
        return Icons.tune;
      default:
        return Icons.settings;
    }
  }

  Widget _buildDefaultThreatLevel(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Consumer<ConfigurationBuilderProvider>(
          builder: (context, provider, _) {
            final config = provider.currentOsslConfig;
            final currentLevel = config?.defaultThreatLevel ?? OsslThreatLevel.low;

            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DropdownButtonFormField<OsslThreatLevel>(
                  value: currentLevel,
                  decoration: const InputDecoration(
                    labelText: 'Default Threat Level',
                    prefixIcon: Icon(Icons.security),
                    border: OutlineInputBorder(),
                    helperText: 'Maximum threat level allowed by default',
                  ),
                  items: OsslThreatLevel.values.map((level) {
                    return DropdownMenuItem(
                      value: level,
                      child: Row(
                        children: [
                          Container(
                            width: 12,
                            height: 12,
                            decoration: BoxDecoration(
                              color: _getThreatColor(level),
                              shape: BoxShape.circle,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Text(_getThreatLevelName(level)),
                        ],
                      ),
                    );
                  }).toList(),
                  onChanged: (value) {
                    if (config != null && value != null) {
                      provider.updateOsslConfig(OsslConfig(
                        defaultThreatLevel: value,
                        allowedFunctions: config.allowedFunctions,
                        blockedFunctions: config.blockedFunctions,
                        enableNpc: config.enableNpc,
                        enableTeleport: config.enableTeleport,
                        enableDynamicTextures: config.enableDynamicTextures,
                        enableJsonStore: config.enableJsonStore,
                      ));
                    }
                  },
                ),
                const SizedBox(height: 16),
                _buildThreatLevelIndicator(currentLevel),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildThreatLevelIndicator(OsslThreatLevel level) {
    final index = OsslThreatLevel.values.indexOf(level);
    final progress = (index + 1) / OsslThreatLevel.values.length;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text('Restrictive'),
            const Text('Permissive'),
          ],
        ),
        const SizedBox(height: 4),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: progress,
            minHeight: 8,
            backgroundColor: Colors.grey[200],
            valueColor: AlwaysStoppedAnimation(_getThreatColor(level)),
          ),
        ),
      ],
    );
  }

  Widget _buildFunctionPermissions(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Consumer<ConfigurationBuilderProvider>(
          builder: (context, provider, _) {
            final config = provider.currentOsslConfig;
            if (config == null) return const SizedBox.shrink();

            return Column(
              children: [
                _buildFunctionToggle(
                  context,
                  'NPC Functions',
                  'Enable Non-Player Character creation and control',
                  Icons.smart_toy,
                  config.enableNpc,
                  (value) => _updateOsslConfig(provider, config, enableNpc: value),
                ),
                const Divider(),
                _buildFunctionToggle(
                  context,
                  'Teleport Functions',
                  'Allow scripts to teleport avatars',
                  Icons.swap_horiz,
                  config.enableTeleport,
                  (value) => _updateOsslConfig(provider, config, enableTeleport: value),
                ),
                const Divider(),
                _buildFunctionToggle(
                  context,
                  'Dynamic Textures',
                  'Allow scripts to generate textures dynamically',
                  Icons.image,
                  config.enableDynamicTextures,
                  (value) => _updateOsslConfig(provider, config, enableDynamicTextures: value),
                ),
                const Divider(),
                _buildFunctionToggle(
                  context,
                  'JSON Store',
                  'Enable JSON data storage functions',
                  Icons.data_object,
                  config.enableJsonStore,
                  (value) => _updateOsslConfig(provider, config, enableJsonStore: value),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildFunctionToggle(
    BuildContext context,
    String title,
    String subtitle,
    IconData icon,
    bool value,
    ValueChanged<bool> onChanged,
  ) {
    return SwitchListTile(
      title: Row(
        children: [
          Icon(icon, size: 20),
          const SizedBox(width: 8),
          Text(title),
        ],
      ),
      subtitle: Text(subtitle),
      value: value,
      onChanged: onChanged,
    );
  }

  void _updateOsslConfig(
    ConfigurationBuilderProvider provider,
    OsslConfig config, {
    bool? enableNpc,
    bool? enableTeleport,
    bool? enableDynamicTextures,
    bool? enableJsonStore,
  }) {
    provider.updateOsslConfig(OsslConfig(
      defaultThreatLevel: config.defaultThreatLevel,
      allowedFunctions: config.allowedFunctions,
      blockedFunctions: config.blockedFunctions,
      enableNpc: enableNpc ?? config.enableNpc,
      enableTeleport: enableTeleport ?? config.enableTeleport,
      enableDynamicTextures: enableDynamicTextures ?? config.enableDynamicTextures,
      enableJsonStore: enableJsonStore ?? config.enableJsonStore,
    ));
  }

  Widget _buildPresets(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Consumer<ConfigurationBuilderProvider>(
          builder: (context, provider, _) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Apply common security configurations',
                  style: TextStyle(
                    color: Colors.grey[600],
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 16),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    _buildPresetButton(
                      context,
                      provider,
                      'Lockdown',
                      'Maximum security - minimal functions',
                      Icons.lock,
                      Colors.red,
                      OsslConfig(
                        defaultThreatLevel: OsslThreatLevel.none,
                        enableNpc: false,
                        enableTeleport: false,
                        enableDynamicTextures: false,
                        enableJsonStore: false,
                      ),
                    ),
                    _buildPresetButton(
                      context,
                      provider,
                      'Cautious',
                      'Low risk functions only',
                      Icons.shield,
                      Colors.amber,
                      OsslConfig(
                        defaultThreatLevel: OsslThreatLevel.low,
                        enableNpc: false,
                        enableTeleport: false,
                        enableDynamicTextures: true,
                        enableJsonStore: true,
                      ),
                    ),
                    _buildPresetButton(
                      context,
                      provider,
                      'Balanced',
                      'Good balance of features and security',
                      Icons.balance,
                      Colors.green,
                      OsslConfig(
                        defaultThreatLevel: OsslThreatLevel.moderate,
                        enableNpc: true,
                        enableTeleport: true,
                        enableDynamicTextures: true,
                        enableJsonStore: true,
                      ),
                    ),
                    _buildPresetButton(
                      context,
                      provider,
                      'Permissive',
                      'Most functions enabled for trusted users',
                      Icons.verified_user,
                      Colors.blue,
                      OsslConfig(
                        defaultThreatLevel: OsslThreatLevel.high,
                        enableNpc: true,
                        enableTeleport: true,
                        enableDynamicTextures: true,
                        enableJsonStore: true,
                      ),
                    ),
                    _buildPresetButton(
                      context,
                      provider,
                      'Developer',
                      'All functions enabled for development',
                      Icons.code,
                      Colors.purple,
                      OsslConfig(
                        defaultThreatLevel: OsslThreatLevel.severe,
                        enableNpc: true,
                        enableTeleport: true,
                        enableDynamicTextures: true,
                        enableJsonStore: true,
                      ),
                    ),
                  ],
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildPresetButton(
    BuildContext context,
    ConfigurationBuilderProvider provider,
    String name,
    String description,
    IconData icon,
    Color color,
    OsslConfig preset,
  ) {
    return SizedBox(
      width: 160,
      child: OutlinedButton(
        style: OutlinedButton.styleFrom(
          side: BorderSide(color: color),
          padding: const EdgeInsets.all(12),
        ),
        onPressed: () {
          provider.updateOsslConfig(preset);
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Applied "$name" preset'),
              duration: const Duration(seconds: 2),
            ),
          );
        },
        child: Column(
          children: [
            Icon(icon, color: color, size: 24),
            const SizedBox(height: 4),
            Text(
              name,
              style: TextStyle(
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
            const SizedBox(height: 2),
            Text(
              description,
              style: TextStyle(
                fontSize: 10,
                color: Colors.grey[600],
              ),
              textAlign: TextAlign.center,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }

  String _getThreatLevelName(OsslThreatLevel level) {
    switch (level) {
      case OsslThreatLevel.none:
        return 'None';
      case OsslThreatLevel.nuisance:
        return 'Nuisance';
      case OsslThreatLevel.veryLow:
        return 'Very Low';
      case OsslThreatLevel.low:
        return 'Low';
      case OsslThreatLevel.moderate:
        return 'Moderate';
      case OsslThreatLevel.high:
        return 'High';
      case OsslThreatLevel.veryHigh:
        return 'Very High';
      case OsslThreatLevel.severe:
        return 'Severe';
    }
  }

  Color _getThreatColor(OsslThreatLevel level) {
    switch (level) {
      case OsslThreatLevel.none:
        return Colors.grey;
      case OsslThreatLevel.nuisance:
        return Colors.blue;
      case OsslThreatLevel.veryLow:
        return Colors.green;
      case OsslThreatLevel.low:
        return Colors.teal;
      case OsslThreatLevel.moderate:
        return Colors.amber;
      case OsslThreatLevel.high:
        return Colors.orange;
      case OsslThreatLevel.veryHigh:
        return Colors.deepOrange;
      case OsslThreatLevel.severe:
        return Colors.red;
    }
  }
}
