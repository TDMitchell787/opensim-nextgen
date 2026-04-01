import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

enum TerrainType { water, land, mixed, void_ }

class GridLayout {
  int gridWidth;
  int gridHeight;
  int baseLocationX;
  int baseLocationY;
  int basePort;
  int regionSize;
  String namingPattern;

  GridLayout({
    this.gridWidth = 1,
    this.gridHeight = 1,
    this.baseLocationX = 1000,
    this.baseLocationY = 1000,
    this.basePort = 9000,
    this.regionSize = 256,
    this.namingPattern = '{name}{index:02}',
  });

  int get totalRegions => gridWidth * gridHeight;
  int get portRangeEnd => basePort + totalRegions - 1;
  int get worldSizeX => gridWidth * regionSize;
  int get worldSizeY => gridHeight * regionSize;
}

class SimulatorTypeDefaults {
  final int typicalMinSims;
  final int typicalMaxSims;
  final String recommendedLayout;
  final TerrainType terrainType;
  final int terrainHeight;
  final int suggestedPortRangeStart;

  const SimulatorTypeDefaults({
    required this.typicalMinSims,
    required this.typicalMaxSims,
    required this.recommendedLayout,
    required this.terrainType,
    required this.terrainHeight,
    required this.suggestedPortRangeStart,
  });

  static SimulatorTypeDefaults forType(String type) {
    switch (type.toLowerCase()) {
      case 'marina':
        return const SimulatorTypeDefaults(
          typicalMinSims: 16, typicalMaxSims: 64,
          recommendedLayout: '4x4 to 8x8', terrainType: TerrainType.water,
          terrainHeight: -30, suggestedPortRangeStart: 9000,
        );
      case 'mainland':
        return const SimulatorTypeDefaults(
          typicalMinSims: 1, typicalMaxSims: 100,
          recommendedLayout: 'variable', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9100,
        );
      case 'event':
        return const SimulatorTypeDefaults(
          typicalMinSims: 4, typicalMaxSims: 4,
          recommendedLayout: '2x2', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9200,
        );
      case 'welcome':
        return const SimulatorTypeDefaults(
          typicalMinSims: 8, typicalMaxSims: 8,
          recommendedLayout: '2x4', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9210,
        );
      case 'sandbox':
        return const SimulatorTypeDefaults(
          typicalMinSims: 1, typicalMaxSims: 4,
          recommendedLayout: '1x1 to 2x2', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9220,
        );
      case 'shopping':
        return const SimulatorTypeDefaults(
          typicalMinSims: 4, typicalMaxSims: 16,
          recommendedLayout: '2x2 to 4x4', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9230,
        );
      default:
        return const SimulatorTypeDefaults(
          typicalMinSims: 1, typicalMaxSims: 16,
          recommendedLayout: 'variable', terrainType: TerrainType.land,
          terrainHeight: 22, suggestedPortRangeStart: 9000,
        );
    }
  }
}

class GridPlannerWidget extends StatefulWidget {
  final String templateType;
  final String gridName;
  final Function(GridLayout, TerrainType, int) onConfigChanged;
  final Function(GridLayout, TerrainType, int, String, int)? onAddToCart;
  final int? suggestedBasePort;

  const GridPlannerWidget({
    super.key,
    required this.templateType,
    required this.gridName,
    required this.onConfigChanged,
    this.onAddToCart,
    this.suggestedBasePort,
  });

  @override
  State<GridPlannerWidget> createState() => GridPlannerWidgetState();
}

class GridPlannerWidgetState extends State<GridPlannerWidget> {
  late GridLayout _layout;
  late TerrainType _terrainType;
  late int _terrainHeight;
  late SimulatorTypeDefaults _defaults;

  final _widthController = TextEditingController();
  final _heightController = TextEditingController();
  final _baseXController = TextEditingController();
  final _baseYController = TextEditingController();
  final _basePortController = TextEditingController();
  final _namingController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _defaults = SimulatorTypeDefaults.forType(widget.templateType);
    _initializeFromDefaults();
  }

  void _initializeFromDefaults() {
    int suggestedWidth = 1, suggestedHeight = 1;
    final minSims = _defaults.typicalMinSims;
    if (minSims >= 64) { suggestedWidth = 8; suggestedHeight = 8; }
    else if (minSims >= 16) { suggestedWidth = 4; suggestedHeight = 4; }
    else if (minSims >= 8) { suggestedWidth = 2; suggestedHeight = 4; }
    else if (minSims >= 4) { suggestedWidth = 2; suggestedHeight = 2; }

    final basePort = widget.suggestedBasePort ?? _defaults.suggestedPortRangeStart;

    _layout = GridLayout(
      gridWidth: suggestedWidth,
      gridHeight: suggestedHeight,
      baseLocationX: 1000,
      baseLocationY: 1000,
      basePort: basePort,
      regionSize: 256,
      namingPattern: '{name} {index:02}',
    );
    _terrainType = _defaults.terrainType;
    _terrainHeight = _defaults.terrainHeight;

    _widthController.text = _layout.gridWidth.toString();
    _heightController.text = _layout.gridHeight.toString();
    _baseXController.text = _layout.baseLocationX.toString();
    _baseYController.text = _layout.baseLocationY.toString();
    _basePortController.text = _layout.basePort.toString();
    _namingController.text = _layout.namingPattern;
  }

  @override
  void didUpdateWidget(GridPlannerWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.templateType != widget.templateType) {
      _defaults = SimulatorTypeDefaults.forType(widget.templateType);
      _initializeFromDefaults();
      setState(() {});
    } else if (oldWidget.suggestedBasePort != widget.suggestedBasePort && widget.suggestedBasePort != null) {
      setBasePort(widget.suggestedBasePort!);
    }
  }

  void _notifyChange() {
    widget.onConfigChanged(_layout, _terrainType, _terrainHeight);
  }

  void setBasePort(int port) {
    setState(() {
      _layout.basePort = port;
      _basePortController.text = port.toString();
    });
    _notifyChange();
  }

  int get currentPortRangeEnd => _layout.portRangeEnd;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildTemplateInfo(),
        const SizedBox(height: 16),
        _buildGridSizeSection(),
        const SizedBox(height: 16),
        _buildLocationSection(),
        const SizedBox(height: 16),
        _buildPortSection(),
        const SizedBox(height: 16),
        _buildTerrainSection(),
        const SizedBox(height: 16),
        _buildNamingSection(),
        const SizedBox(height: 16),
        _buildGridPreview(),
        const SizedBox(height: 16),
        _buildCapacityWarnings(),
        if (widget.onAddToCart != null) ...[
          const SizedBox(height: 24),
          _buildAddToCartSection(context),
        ],
      ],
    );
  }

  Widget _buildAddToCartSection(BuildContext context) {
    final theme = Theme.of(context);
    final hasPortError = _layout.portRangeEnd > 65535;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: theme.colorScheme.primaryContainer.withOpacity(0.3),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: theme.colorScheme.primary.withOpacity(0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.add_shopping_cart, color: theme.colorScheme.primary),
              const SizedBox(width: 8),
              Text(
                'Add to Deployment Cart',
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: theme.colorScheme.primary,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '${_layout.totalRegions} ${widget.templateType} regions',
                      style: theme.textTheme.bodyLarge?.copyWith(
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                    Text(
                      'Ports ${_layout.basePort}-${_layout.portRangeEnd} | '
                      '${_terrainType.name} terrain (${_terrainHeight}m)',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.outline,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 16),
              FilledButton.icon(
                onPressed: hasPortError
                    ? null
                    : () {
                        widget.onAddToCart?.call(
                          _layout,
                          _terrainType,
                          _terrainHeight,
                          widget.templateType,
                          40,
                        );
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text(
                              'Added ${_layout.totalRegions} ${widget.templateType} regions to cart',
                            ),
                            behavior: SnackBarBehavior.floating,
                            margin: EdgeInsets.only(
                              bottom: MediaQuery.of(context).size.height - 150,
                              left: 20,
                              right: 20,
                            ),
                            action: SnackBarAction(
                              label: 'View Cart',
                              onPressed: () {},
                            ),
                          ),
                        );
                      },
                icon: const Icon(Icons.add_shopping_cart),
                label: const Text('Add to Cart'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTemplateInfo() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.blue.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.blue.withOpacity(0.3)),
      ),
      child: Row(
        children: [
          Icon(Icons.info_outline, color: Colors.blue[700]),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '${widget.templateType} Template Defaults',
                  style: TextStyle(fontWeight: FontWeight.bold, color: Colors.blue[700]),
                ),
                const SizedBox(height: 4),
                Text(
                  'Typical: ${_defaults.typicalMinSims}-${_defaults.typicalMaxSims} sims | '
                  'Layout: ${_defaults.recommendedLayout} | '
                  'Terrain: ${_defaults.terrainType.name} (${_defaults.terrainHeight}m)',
                  style: TextStyle(fontSize: 12, color: Colors.blue[600]),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildGridSizeSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Grid Size', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: _widthController,
                decoration: const InputDecoration(
                  labelText: 'Width (X)',
                  helperText: 'Regions wide',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (v) {
                  _layout.gridWidth = int.tryParse(v) ?? 1;
                  _notifyChange();
                  setState(() {});
                },
              ),
            ),
            const SizedBox(width: 8),
            const Text('x', style: TextStyle(fontSize: 20)),
            const SizedBox(width: 8),
            Expanded(
              child: TextField(
                controller: _heightController,
                decoration: const InputDecoration(
                  labelText: 'Height (Y)',
                  helperText: 'Regions tall',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (v) {
                  _layout.gridHeight = int.tryParse(v) ?? 1;
                  _notifyChange();
                  setState(() {});
                },
              ),
            ),
            const SizedBox(width: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.grey[200],
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                children: [
                  Text('${_layout.totalRegions}', style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
                  const Text('Total Regions', style: TextStyle(fontSize: 10)),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          children: [
            _buildPresetButton('1x1', 1, 1),
            _buildPresetButton('2x2', 2, 2),
            _buildPresetButton('2x4', 2, 4),
            _buildPresetButton('4x4', 4, 4),
            _buildPresetButton('8x8', 8, 8),
            _buildPresetButton('10x10', 10, 10),
          ],
        ),
      ],
    );
  }

  Widget _buildPresetButton(String label, int w, int h) {
    final isSelected = _layout.gridWidth == w && _layout.gridHeight == h;
    return ActionChip(
      label: Text(label),
      backgroundColor: isSelected ? Colors.blue : null,
      labelStyle: TextStyle(color: isSelected ? Colors.white : null),
      onPressed: () {
        _layout.gridWidth = w;
        _layout.gridHeight = h;
        _widthController.text = w.toString();
        _heightController.text = h.toString();
        _notifyChange();
        setState(() {});
      },
    );
  }

  Widget _buildLocationSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Base Grid Location', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const Text('Starting coordinates for the grid (256m per unit)', style: TextStyle(fontSize: 12, color: Colors.grey)),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: _baseXController,
                decoration: const InputDecoration(
                  labelText: 'X Coordinate',
                  border: OutlineInputBorder(),
                  prefixIcon: Icon(Icons.arrow_forward),
                ),
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (v) {
                  _layout.baseLocationX = int.tryParse(v) ?? 1000;
                  _notifyChange();
                },
              ),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: TextField(
                controller: _baseYController,
                decoration: const InputDecoration(
                  labelText: 'Y Coordinate',
                  border: OutlineInputBorder(),
                  prefixIcon: Icon(Icons.arrow_upward),
                ),
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (v) {
                  _layout.baseLocationY = int.tryParse(v) ?? 1000;
                  _notifyChange();
                },
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildPortSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Port Allocation', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: _basePortController,
                decoration: const InputDecoration(
                  labelText: 'Base Port',
                  border: OutlineInputBorder(),
                  prefixIcon: Icon(Icons.router),
                ),
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (v) {
                  _layout.basePort = int.tryParse(v) ?? 9000;
                  _notifyChange();
                  setState(() {});
                },
              ),
            ),
            const SizedBox(width: 16),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: _layout.portRangeEnd > 65535 ? Colors.red[100] : Colors.green[100],
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                children: [
                  Text(
                    'Port Range',
                    style: TextStyle(fontSize: 10, color: Colors.grey[700]),
                  ),
                  Text(
                    '${_layout.basePort} - ${_layout.portRangeEnd}',
                    style: TextStyle(
                      fontWeight: FontWeight.bold,
                      color: _layout.portRangeEnd > 65535 ? Colors.red : Colors.green[700],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildTerrainSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Terrain Configuration', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: DropdownButtonFormField<TerrainType>(
                value: _terrainType,
                decoration: const InputDecoration(
                  labelText: 'Terrain Type',
                  border: OutlineInputBorder(),
                ),
                items: TerrainType.values.map((t) => DropdownMenuItem(
                  value: t,
                  child: Row(
                    children: [
                      Icon(_getTerrainIcon(t), size: 20),
                      const SizedBox(width: 8),
                      Text(t == TerrainType.void_ ? 'Void' : t.name[0].toUpperCase() + t.name.substring(1)),
                    ],
                  ),
                )).toList(),
                onChanged: (v) {
                  if (v != null) {
                    _terrainType = v;
                    _terrainHeight = _getDefaultHeight(v);
                    _notifyChange();
                    setState(() {});
                  }
                },
              ),
            ),
            const SizedBox(width: 16),
            SizedBox(
              width: 120,
              child: TextField(
                decoration: const InputDecoration(
                  labelText: 'Height (m)',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.numberWithOptions(signed: true),
                controller: TextEditingController(text: _terrainHeight.toString()),
                onChanged: (v) {
                  _terrainHeight = int.tryParse(v) ?? 0;
                  _notifyChange();
                },
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Text(
          _getTerrainDescription(_terrainType),
          style: TextStyle(fontSize: 12, color: Colors.grey[600]),
        ),
      ],
    );
  }

  IconData _getTerrainIcon(TerrainType t) {
    switch (t) {
      case TerrainType.water: return Icons.water;
      case TerrainType.land: return Icons.landscape;
      case TerrainType.mixed: return Icons.terrain;
      case TerrainType.void_: return Icons.crop_square;
    }
  }

  int _getDefaultHeight(TerrainType t) {
    switch (t) {
      case TerrainType.water: return -30;
      case TerrainType.land: return 22;
      case TerrainType.mixed: return 0;
      case TerrainType.void_: return 0;
    }
  }

  String _getTerrainDescription(TerrainType t) {
    switch (t) {
      case TerrainType.water: return 'Underwater terrain for boats and marine activities (recommended: -30m)';
      case TerrainType.land: return 'Above-ground terrain for buildings and landscaping (recommended: 22m+)';
      case TerrainType.mixed: return 'Variable terrain with both water and land areas';
      case TerrainType.void_: return 'Empty/flat terrain for custom heightmaps';
    }
  }

  Widget _buildNamingSection() {
    final exampleName = _layout.namingPattern
        .replaceAll('{name}', widget.gridName)
        .replaceAll('{index:02}', '01')
        .replaceAll('{index}', '1');

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Region Naming Pattern', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const SizedBox(height: 8),
        TextField(
          controller: _namingController,
          decoration: InputDecoration(
            labelText: 'Naming Pattern',
            border: const OutlineInputBorder(),
            helperText: 'Use {name} for grid name, {index} or {index:02} for number',
            suffixIcon: IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: () {
                _namingController.text = '{name} {index:02}';
                _layout.namingPattern = '{name} {index:02}';
                _notifyChange();
                setState(() {});
              },
            ),
          ),
          onChanged: (v) {
            _layout.namingPattern = v;
            _notifyChange();
            setState(() {});
          },
        ),
        const SizedBox(height: 4),
        Text(
          'Example: $exampleName.ini',
          style: TextStyle(fontSize: 12, color: Colors.grey[600], fontStyle: FontStyle.italic),
        ),
      ],
    );
  }

  Widget _buildGridPreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Grid Layout Preview', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16)),
        const SizedBox(height: 8),
        Container(
          height: 200,
          decoration: BoxDecoration(
            color: _terrainType == TerrainType.water ? Colors.blue[50] : Colors.green[50],
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.grey[300]!),
          ),
          child: LayoutBuilder(
            builder: (context, constraints) {
              final cellWidth = (constraints.maxWidth - 20) / _layout.gridWidth.clamp(1, 16);
              final cellHeight = (constraints.maxHeight - 20) / _layout.gridHeight.clamp(1, 16);
              final cellSize = cellWidth < cellHeight ? cellWidth : cellHeight;

              return Center(
                child: Wrap(
                  direction: Axis.vertical,
                  children: List.generate(_layout.gridHeight.clamp(1, 16), (y) {
                    return Row(
                      mainAxisSize: MainAxisSize.min,
                      children: List.generate(_layout.gridWidth.clamp(1, 16), (x) {
                        final index = y * _layout.gridWidth + x + 1;
                        final port = _layout.basePort + index - 1;
                        return Container(
                          width: cellSize - 2,
                          height: cellSize - 2,
                          margin: const EdgeInsets.all(1),
                          decoration: BoxDecoration(
                            color: _terrainType == TerrainType.water
                                ? Colors.blue[200]
                                : Colors.green[200],
                            borderRadius: BorderRadius.circular(2),
                            border: Border.all(color: Colors.grey[400]!, width: 0.5),
                          ),
                          child: cellSize > 30 ? Center(
                            child: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                Text(
                                  '${index.toString().padLeft(2, '0')}',
                                  style: TextStyle(
                                    fontSize: cellSize > 50 ? 10 : 8,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                                if (cellSize > 50)
                                  Text(
                                    ':$port',
                                    style: const TextStyle(fontSize: 7),
                                  ),
                              ],
                            ),
                          ) : null,
                        );
                      }),
                    );
                  }),
                ),
              );
            },
          ),
        ),
        const SizedBox(height: 8),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              'World Size: ${_layout.worldSizeX}m x ${_layout.worldSizeY}m',
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
            Text(
              'Locations: (${_layout.baseLocationX},${_layout.baseLocationY}) to '
              '(${_layout.baseLocationX + _layout.gridWidth - 1},${_layout.baseLocationY + _layout.gridHeight - 1})',
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildCapacityWarnings() {
    final warnings = <Widget>[];

    if (_layout.totalRegions > 100) {
      warnings.add(_buildWarning(
        Icons.warning,
        Colors.red,
        '${_layout.totalRegions} regions is very large - ensure adequate server capacity',
      ));
    } else if (_layout.totalRegions > 50) {
      warnings.add(_buildWarning(
        Icons.warning_amber,
        Colors.orange,
        'Large grid (${_layout.totalRegions} regions) - consider server resources',
      ));
    }

    if (_layout.portRangeEnd > 65535) {
      warnings.add(_buildWarning(
        Icons.error,
        Colors.red,
        'Port range exceeds maximum (would need port ${_layout.portRangeEnd})',
      ));
    }

    if (_layout.basePort < 1024) {
      warnings.add(_buildWarning(
        Icons.info,
        Colors.blue,
        'Ports below 1024 require elevated privileges',
      ));
    }

    if (_layout.totalRegions < _defaults.typicalMinSims) {
      warnings.add(_buildWarning(
        Icons.info,
        Colors.blue,
        '${widget.templateType} typically uses ${_defaults.typicalMinSims}+ regions',
      ));
    }

    if (warnings.isEmpty) {
      return Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.green[50],
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          children: [
            Icon(Icons.check_circle, color: Colors.green[700]),
            const SizedBox(width: 8),
            Text('Configuration looks good!', style: TextStyle(color: Colors.green[700])),
          ],
        ),
      );
    }

    return Column(children: warnings);
  }

  Widget _buildWarning(IconData icon, Color color, String message) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Row(
        children: [
          Icon(icon, color: color),
          const SizedBox(width: 8),
          Expanded(child: Text(message, style: TextStyle(color: color))),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _widthController.dispose();
    _heightController.dispose();
    _baseXController.dispose();
    _baseYController.dispose();
    _basePortController.dispose();
    _namingController.dispose();
    super.dispose();
  }
}
