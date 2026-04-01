import 'package:flutter/material.dart';
import '../../models/configuration_builder_models.dart' show DeploymentType;

class CartItem {
  final String id;
  final String name;
  final String worldName;
  final String simulatorType;
  final int regionCount;
  final int portStart;
  final int portEnd;
  final String terrainType;
  final int terrainHeight;
  final int maxAgents;

  CartItem({
    required this.id,
    required this.name,
    required this.worldName,
    required this.simulatorType,
    required this.regionCount,
    required this.portStart,
    required this.portEnd,
    required this.terrainType,
    required this.terrainHeight,
    required this.maxAgents,
  });
}

class UsageProfile {
  final String name;
  final double avgAvatarsPerRegion;
  final double emptyRegionPercent;
  final double peakMultiplier;

  const UsageProfile({
    required this.name,
    required this.avgAvatarsPerRegion,
    required this.emptyRegionPercent,
    required this.peakMultiplier,
  });

  static const light = UsageProfile(
    name: 'Light (mostly empty)',
    avgAvatarsPerRegion: 1.0,
    emptyRegionPercent: 0.8,
    peakMultiplier: 2.0,
  );

  static const moderate = UsageProfile(
    name: 'Moderate (2 avg/region)',
    avgAvatarsPerRegion: 2.0,
    emptyRegionPercent: 0.6,
    peakMultiplier: 3.0,
  );

  static const heavy = UsageProfile(
    name: 'Heavy (5 avg/region)',
    avgAvatarsPerRegion: 5.0,
    emptyRegionPercent: 0.3,
    peakMultiplier: 4.0,
  );

  static const event = UsageProfile(
    name: 'Event (crowded)',
    avgAvatarsPerRegion: 20.0,
    emptyRegionPercent: 0.1,
    peakMultiplier: 5.0,
  );

  static const List<UsageProfile> all = [light, moderate, heavy, event];
}

class CartAggregation {
  final int totalRegions;
  final int totalItems;
  final int maxMemoryMb;
  final int realisticMemoryMb;
  final int maxCpuCores;
  final int realisticCpuCores;
  final int maxBandwidthMbps;
  final int realisticBandwidthMbps;
  final int maxAvatars;
  final int realisticAvatars;
  final int peakAvatars;
  final int totalMaxPrims;
  final Map<String, int> regionsByType;
  final Map<String, int> terrainSummary;
  final List<String> worlds;
  final List<String> warnings;
  final List<(String, String, int)> portConflicts;

  CartAggregation({
    required this.totalRegions,
    required this.totalItems,
    required this.maxMemoryMb,
    required this.realisticMemoryMb,
    required this.maxCpuCores,
    required this.realisticCpuCores,
    required this.maxBandwidthMbps,
    required this.realisticBandwidthMbps,
    required this.maxAvatars,
    required this.realisticAvatars,
    required this.peakAvatars,
    required this.totalMaxPrims,
    required this.regionsByType,
    required this.terrainSummary,
    required this.worlds,
    required this.warnings,
    required this.portConflicts,
  });
}

class DeploymentCartWidget extends StatefulWidget {
  final Function(CartItem)? onItemRemoved;
  final Function()? onDeploy;
  final Function()? onClear;
  final Function(CartItem)? onItemAdded;

  const DeploymentCartWidget({
    super.key,
    this.onItemRemoved,
    this.onDeploy,
    this.onClear,
    this.onItemAdded,
  });

  @override
  State<DeploymentCartWidget> createState() => DeploymentCartWidgetState();
}

class DeploymentCartWidgetState extends State<DeploymentCartWidget> {
  final List<CartItem> _items = [];
  UsageProfile _selectedProfile = UsageProfile.moderate;
  String _worldName = 'My Virtual World';
  bool _showRealisticStats = true;
  DeploymentType _deploymentType = DeploymentType.native;
  String _targetPath = '/opt/opensim/instances';
  bool _autoStart = true;
  final _targetPathController = TextEditingController();
  final _worldNameController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _targetPathController.text = _targetPath;
    _worldNameController.text = _worldName;
  }

  @override
  void dispose() {
    _targetPathController.dispose();
    _worldNameController.dispose();
    super.dispose();
  }

  void addItem(CartItem item) {
    setState(() {
      _items.add(item);
    });
    widget.onItemAdded?.call(item);
  }

  List<CartItem> get items => List.unmodifiable(_items);
  int get itemCount => _items.length;

  int getNextAvailablePort() {
    if (_items.isEmpty) {
      return 9000;
    }
    int highestPort = 0;
    for (final item in _items) {
      if (item.portEnd > highestPort) {
        highestPort = item.portEnd;
      }
    }
    return highestPort + 1;
  }

  void removeItem(String itemId) {
    setState(() {
      final item = _items.firstWhere((i) => i.id == itemId);
      _items.removeWhere((i) => i.id == itemId);
      widget.onItemRemoved?.call(item);
    });
  }

  void clear() {
    setState(() {
      _items.clear();
    });
    widget.onClear?.call();
  }

  CartAggregation _calculateAggregation() {
    if (_items.isEmpty) {
      return CartAggregation(
        totalRegions: 0,
        totalItems: 0,
        maxMemoryMb: 0,
        realisticMemoryMb: 0,
        maxCpuCores: 0,
        realisticCpuCores: 0,
        maxBandwidthMbps: 0,
        realisticBandwidthMbps: 0,
        maxAvatars: 0,
        realisticAvatars: 0,
        peakAvatars: 0,
        totalMaxPrims: 0,
        regionsByType: {},
        terrainSummary: {},
        worlds: [],
        warnings: [],
        portConflicts: [],
      );
    }

    int totalRegions = 0;
    int totalMaxAgents = 0;
    int totalMaxPrims = 0;
    int maxMemory = 0;
    int maxBandwidth = 0;
    final regionsByType = <String, int>{};
    final terrainSummary = <String, int>{};
    final worldsSet = <String>{};
    final warnings = <String>[];
    final portConflicts = <(String, String, int)>[];

    final portMap = <int, String>{};

    for (final item in _items) {
      totalRegions += item.regionCount;
      totalMaxAgents += item.maxAgents;
      totalMaxPrims += item.regionCount * 45000;

      final memPerRegion = _getMemoryForType(item.simulatorType);
      maxMemory += (memPerRegion * item.regionCount * 0.85).round();
      maxBandwidth += item.regionCount * 75;

      regionsByType[item.simulatorType] =
          (regionsByType[item.simulatorType] ?? 0) + item.regionCount;
      terrainSummary[item.terrainType] =
          (terrainSummary[item.terrainType] ?? 0) + item.regionCount;
      worldsSet.add(item.worldName);

      for (int port = item.portStart; port <= item.portEnd; port++) {
        if (portMap.containsKey(port)) {
          portConflicts.add((portMap[port]!, item.name, port));
        } else {
          portMap[port] = item.name;
        }
      }
    }

    if (portConflicts.isNotEmpty) {
      warnings.add('${portConflicts.length} port conflicts detected');
    }

    if (maxMemory > 65536) {
      warnings.add(
          'Max memory (${(maxMemory / 1024).round()} GB) exceeds typical server capacity');
    }

    if (totalRegions > 256) {
      warnings
          .add('$totalRegions regions is very large - consider multi-server');
    }

    final activeRegions =
        (totalRegions * (1.0 - _selectedProfile.emptyRegionPercent)).round();
    final realisticAvatars =
        (activeRegions * _selectedProfile.avgAvatarsPerRegion).round();
    final peakAvatars =
        (realisticAvatars * _selectedProfile.peakMultiplier).round();

    const baseRegionMemory = 256;
    const memoryPerAvatar = 50;
    final realisticMemory = (totalRegions * baseRegionMemory) +
        (realisticAvatars * memoryPerAvatar) +
        (maxMemory * 0.3 * 0.3).round();

    final avatarRatio = totalMaxAgents > 0 ? realisticAvatars / totalMaxAgents : 0.0;
    final realisticCpu = ((maxMemory ~/ 2048).clamp(2, 16) * (0.3 + avatarRatio * 0.3)).ceil();
    final realisticBandwidth = (maxBandwidth * avatarRatio.clamp(0.1, 1.0)).ceil();

    return CartAggregation(
      totalRegions: totalRegions,
      totalItems: _items.length,
      maxMemoryMb: maxMemory,
      realisticMemoryMb: realisticMemory.clamp(1024, maxMemory),
      maxCpuCores: (maxMemory ~/ 2048).clamp(2, 32),
      realisticCpuCores: realisticCpu.clamp(2, 16),
      maxBandwidthMbps: maxBandwidth,
      realisticBandwidthMbps: realisticBandwidth.clamp(10, maxBandwidth),
      maxAvatars: totalMaxAgents,
      realisticAvatars: realisticAvatars,
      peakAvatars: peakAvatars.clamp(0, totalMaxAgents),
      totalMaxPrims: totalMaxPrims,
      regionsByType: regionsByType,
      terrainSummary: terrainSummary,
      worlds: worldsSet.toList(),
      warnings: warnings,
      portConflicts: portConflicts,
    );
  }

  int _getMemoryForType(String type) {
    switch (type.toLowerCase()) {
      case 'marina':
        return 2048;
      case 'mainland':
        return 2048;
      case 'event':
        return 8192;
      case 'welcome':
        return 3072;
      case 'sandbox':
        return 4096;
      case 'shopping':
        return 2048;
      case 'island':
        return 1536;
      case 'roleplay':
        return 2560;
      case 'residential':
        return 1024;
      default:
        return 2048;
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final aggregation = _calculateAggregation();

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildHeader(theme),
            const SizedBox(height: 16),
            if (_items.isEmpty)
              _buildEmptyState(theme)
            else ...[
              _buildWorldNameField(theme),
              const SizedBox(height: 16),
              _buildUsageProfileSelector(theme),
              const SizedBox(height: 16),
              _buildItemsList(theme),
              const SizedBox(height: 16),
              _buildAggregationSummary(theme, aggregation),
              if (aggregation.warnings.isNotEmpty) ...[
                const SizedBox(height: 16),
                _buildWarnings(theme, aggregation.warnings),
              ],
              const SizedBox(height: 20),
              _buildDeploymentSettings(theme),
              const SizedBox(height: 16),
              _buildActions(theme, aggregation),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(ThemeData theme) {
    return Row(
      children: [
        Icon(Icons.shopping_cart, color: theme.colorScheme.primary),
        const SizedBox(width: 8),
        Text(
          'Deployment Cart',
          style: theme.textTheme.titleLarge,
        ),
        const Spacer(),
        if (_items.isNotEmpty)
          Chip(
            label: Text('${_items.length} items'),
            backgroundColor: theme.colorScheme.primaryContainer,
          ),
      ],
    );
  }

  Widget _buildEmptyState(ThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(32),
      alignment: Alignment.center,
      child: Column(
        children: [
          Icon(
            Icons.add_shopping_cart,
            size: 48,
            color: theme.colorScheme.outline,
          ),
          const SizedBox(height: 16),
          Text(
            'No grid configurations added',
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.outline,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            'Use the Grid Planner to add regions to your deployment',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.outline,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWorldNameField(ThemeData theme) {
    return TextField(
      decoration: InputDecoration(
        labelText: 'World / Grid Name',
        hintText: 'Name for this virtual world deployment',
        prefixIcon: const Icon(Icons.public),
        border: const OutlineInputBorder(),
        filled: true,
        fillColor: theme.colorScheme.surfaceContainerHighest,
      ),
      controller: _worldNameController,
      onChanged: (value) {
        _worldName = value;
      },
    );
  }

  Widget _buildUsageProfileSelector(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text('Usage Profile', style: theme.textTheme.titleSmall),
            const SizedBox(width: 8),
            Tooltip(
              message:
                  'Affects realistic resource estimates based on expected avatar population',
              child: Icon(Icons.info_outline,
                  size: 16, color: theme.colorScheme.outline),
            ),
          ],
        ),
        const SizedBox(height: 8),
        SegmentedButton<UsageProfile>(
          segments: UsageProfile.all.map((profile) {
            return ButtonSegment<UsageProfile>(
              value: profile,
              label: Text(profile.name.split(' ').first),
              tooltip: profile.name,
            );
          }).toList(),
          selected: {_selectedProfile},
          onSelectionChanged: (selected) {
            setState(() {
              _selectedProfile = selected.first;
            });
          },
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Switch(
              value: _showRealisticStats,
              onChanged: (value) {
                setState(() {
                  _showRealisticStats = value;
                });
              },
            ),
            Text(
              _showRealisticStats
                  ? 'Showing realistic usage'
                  : 'Showing max capacity',
              style: theme.textTheme.bodySmall,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildItemsList(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Cart Items', style: theme.textTheme.titleSmall),
        const SizedBox(height: 8),
        ...List.generate(_items.length, (index) {
          final item = _items[index];
          return _buildItemCard(theme, item, index);
        }),
      ],
    );
  }

  Widget _buildItemCard(ThemeData theme, CartItem item, int index) {
    final typeColor = _getTypeColor(item.simulatorType);

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      color: theme.colorScheme.surfaceContainerHighest,
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: typeColor.withValues(alpha: 0.2),
          child: Icon(_getTypeIcon(item.simulatorType), color: typeColor),
        ),
        title: Text(item.name),
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '${item.regionCount} regions | Ports ${item.portStart}-${item.portEnd}',
              style: theme.textTheme.bodySmall,
            ),
            Text(
              '${item.simulatorType} | ${item.terrainType} (${item.terrainHeight}m)',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.outline,
              ),
            ),
          ],
        ),
        trailing: IconButton(
          icon: const Icon(Icons.delete_outline),
          onPressed: () => removeItem(item.id),
          tooltip: 'Remove from cart',
        ),
      ),
    );
  }

  Color _getTypeColor(String type) {
    switch (type.toLowerCase()) {
      case 'marina':
        return Colors.blue;
      case 'mainland':
        return Colors.green;
      case 'island':
        return Colors.teal;
      case 'event':
        return Colors.purple;
      case 'welcome':
        return Colors.orange;
      case 'sandbox':
        return Colors.amber;
      case 'shopping':
        return Colors.pink;
      case 'residential':
        return Colors.brown;
      default:
        return Colors.grey;
    }
  }

  IconData _getTypeIcon(String type) {
    switch (type.toLowerCase()) {
      case 'marina':
        return Icons.sailing;
      case 'mainland':
        return Icons.terrain;
      case 'island':
        return Icons.beach_access;
      case 'event':
        return Icons.celebration;
      case 'welcome':
        return Icons.waving_hand;
      case 'sandbox':
        return Icons.construction;
      case 'shopping':
        return Icons.storefront;
      case 'residential':
        return Icons.home;
      default:
        return Icons.grid_view;
    }
  }

  Widget _buildAggregationSummary(ThemeData theme, CartAggregation agg) {
    final showRealistic = _showRealisticStats;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Resource Summary', style: theme.textTheme.titleSmall),
        const SizedBox(height: 12),

        Row(
          children: [
            Expanded(
              child: _buildStatCard(
                theme,
                'Total Regions',
                '${agg.totalRegions}',
                Icons.grid_view,
                Colors.blue,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: _buildStatCard(
                theme,
                'Avatars',
                showRealistic
                    ? '${agg.realisticAvatars} avg / ${agg.peakAvatars} peak'
                    : '${agg.maxAvatars} max',
                Icons.people,
                Colors.green,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),

        Row(
          children: [
            Expanded(
              child: _buildStatCard(
                theme,
                'Memory',
                showRealistic
                    ? '${_formatMemory(agg.realisticMemoryMb)} realistic'
                    : '${_formatMemory(agg.maxMemoryMb)} max',
                Icons.memory,
                Colors.orange,
                subtitle: showRealistic
                    ? 'Max: ${_formatMemory(agg.maxMemoryMb)}'
                    : null,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: _buildStatCard(
                theme,
                'CPU Cores',
                showRealistic
                    ? '${agg.realisticCpuCores} cores'
                    : '${agg.maxCpuCores} cores',
                Icons.developer_board,
                Colors.purple,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),

        Row(
          children: [
            Expanded(
              child: _buildStatCard(
                theme,
                'Network',
                showRealistic
                    ? '${agg.realisticBandwidthMbps} Mbps'
                    : '${agg.maxBandwidthMbps} Mbps',
                Icons.wifi,
                Colors.teal,
              ),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: _buildStatCard(
                theme,
                'Prims',
                _formatNumber(agg.totalMaxPrims),
                Icons.view_in_ar,
                Colors.indigo,
              ),
            ),
          ],
        ),

        const SizedBox(height: 16),
        _buildBreakdownSection(theme, agg),
      ],
    );
  }

  String _formatMemory(int mb) {
    if (mb >= 1024) {
      return '${(mb / 1024).toStringAsFixed(1)} GB';
    }
    return '$mb MB';
  }

  String _formatNumber(int n) {
    if (n >= 1000000) {
      return '${(n / 1000000).toStringAsFixed(1)}M';
    }
    if (n >= 1000) {
      return '${(n / 1000).toStringAsFixed(0)}K';
    }
    return '$n';
  }

  Widget _buildStatCard(
    ThemeData theme,
    String label,
    String value,
    IconData icon,
    Color color, {
    String? subtitle,
  }) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 16, color: color),
              const SizedBox(width: 4),
              Text(
                label,
                style: theme.textTheme.bodySmall?.copyWith(color: color),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            value,
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
          if (subtitle != null)
            Text(
              subtitle,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.outline,
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildBreakdownSection(ThemeData theme, CartAggregation agg) {
    return ExpansionTile(
      title: const Text('Breakdown by Type'),
      initiallyExpanded: false,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              ...agg.regionsByType.entries.map((entry) {
                final color = _getTypeColor(entry.key);
                return Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: Row(
                    children: [
                      Icon(_getTypeIcon(entry.key), size: 20, color: color),
                      const SizedBox(width: 8),
                      Text(entry.key),
                      const Spacer(),
                      Text(
                        '${entry.value} regions',
                        style: theme.textTheme.bodyMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                );
              }),
              const Divider(),
              ...agg.terrainSummary.entries.map((entry) {
                return Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: Row(
                    children: [
                      Icon(
                        entry.key.toLowerCase() == 'water'
                            ? Icons.water
                            : Icons.landscape,
                        size: 16,
                        color: theme.colorScheme.outline,
                      ),
                      const SizedBox(width: 8),
                      Text(
                        '${entry.key}: ${entry.value} regions',
                        style: theme.textTheme.bodySmall,
                      ),
                    ],
                  ),
                );
              }),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildWarnings(ThemeData theme, List<String> warnings) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.orange.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.orange.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.warning_amber, color: Colors.orange, size: 20),
              const SizedBox(width: 8),
              Text(
                'Warnings',
                style: theme.textTheme.titleSmall?.copyWith(
                  color: Colors.orange,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          ...warnings.map((w) => Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('• '),
                    Expanded(child: Text(w)),
                  ],
                ),
              )),
        ],
      ),
    );
  }

  Widget _buildDeploymentSettings(ThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: theme.colorScheme.outline.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.settings, color: theme.colorScheme.primary, size: 20),
              const SizedBox(width: 8),
              Text(
                'Deployment Settings',
                style: theme.textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Text('Deployment Type', style: theme.textTheme.bodySmall),
          const SizedBox(height: 8),
          SegmentedButton<DeploymentType>(
            segments: [
              ButtonSegment<DeploymentType>(
                value: DeploymentType.native,
                label: const Text('Native'),
                icon: const Icon(Icons.computer, size: 18),
              ),
              ButtonSegment<DeploymentType>(
                value: DeploymentType.docker,
                label: const Text('Docker'),
                icon: const Icon(Icons.widgets, size: 18),
              ),
              ButtonSegment<DeploymentType>(
                value: DeploymentType.kubernetes,
                label: const Text('K8s'),
                icon: const Icon(Icons.cloud, size: 18),
              ),
            ],
            selected: {_deploymentType},
            onSelectionChanged: (selected) {
              setState(() {
                _deploymentType = selected.first;
              });
            },
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _targetPathController,
            decoration: InputDecoration(
              labelText: 'Target Path',
              hintText: _deploymentType == DeploymentType.native
                  ? '/opt/opensim/instances'
                  : _deploymentType == DeploymentType.docker
                      ? '/var/opensim/data'
                      : 'opensim-namespace',
              prefixIcon: Icon(
                _deploymentType == DeploymentType.kubernetes
                    ? Icons.cloud
                    : Icons.folder,
              ),
              border: const OutlineInputBorder(),
              isDense: true,
            ),
            onChanged: (v) => _targetPath = v,
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Checkbox(
                value: _autoStart,
                onChanged: (v) => setState(() => _autoStart = v ?? true),
              ),
              Expanded(
                child: Text(
                  'Auto-start after deployment',
                  style: theme.textTheme.bodySmall,
                ),
              ),
            ],
          ),
          if (_deploymentType == DeploymentType.docker) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Colors.blue.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  const Icon(Icons.info_outline, size: 16, color: Colors.blue),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Will generate docker-compose.yml with ${_items.length} services',
                      style: theme.textTheme.bodySmall?.copyWith(color: Colors.blue),
                    ),
                  ),
                ],
              ),
            ),
          ],
          if (_deploymentType == DeploymentType.kubernetes) ...[
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Colors.purple.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  const Icon(Icons.info_outline, size: 16, color: Colors.purple),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Will generate Helm chart with ConfigMaps for each region',
                      style: theme.textTheme.bodySmall?.copyWith(color: Colors.purple),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  DeploymentType get deploymentType => _deploymentType;
  String get targetPath => _targetPath;
  bool get autoStart => _autoStart;
  String get worldName => _worldName;

  Widget _buildActions(ThemeData theme, CartAggregation agg) {
    return Row(
      children: [
        OutlinedButton.icon(
          onPressed: _items.isEmpty ? null : clear,
          icon: const Icon(Icons.clear_all),
          label: const Text('Clear Cart'),
        ),
        const Spacer(),
        FilledButton.icon(
          onPressed: _items.isEmpty || agg.portConflicts.isNotEmpty
              ? null
              : () => widget.onDeploy?.call(),
          icon: const Icon(Icons.rocket_launch),
          label: Text('Deploy ${agg.totalRegions} Regions'),
        ),
      ],
    );
  }
}
