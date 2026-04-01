import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/configuration_builder_models.dart';
import '../../providers/configuration_builder_provider.dart';

class RegionIniEditor extends StatefulWidget {
  final RegionIniConfig config;
  final ValueChanged<RegionIniConfig>? onChanged;

  const RegionIniEditor({
    super.key,
    required this.config,
    this.onChanged,
  });

  @override
  State<RegionIniEditor> createState() => _RegionIniEditorState();
}

class _RegionIniEditorState extends State<RegionIniEditor> {
  late TextEditingController _regionNameController;
  late TextEditingController _regionUuidController;
  late TextEditingController _locationXController;
  late TextEditingController _locationYController;
  late TextEditingController _sizeXController;
  late TextEditingController _sizeYController;
  late TextEditingController _internalPortController;
  late TextEditingController _maxAgentsController;
  late TextEditingController _maxPrimsController;
  late TextEditingController _estateNameController;
  late TextEditingController _estateOwnerController;

  @override
  void initState() {
    super.initState();
    _initControllers();
  }

  void _initControllers() {
    _regionNameController = TextEditingController(text: widget.config.regionName);
    _regionUuidController = TextEditingController(text: widget.config.regionUuid);
    _locationXController = TextEditingController(text: widget.config.locationX.toString());
    _locationYController = TextEditingController(text: widget.config.locationY.toString());
    _sizeXController = TextEditingController(text: widget.config.sizeX.toString());
    _sizeYController = TextEditingController(text: widget.config.sizeY.toString());
    _internalPortController = TextEditingController(text: widget.config.internalPort.toString());
    _maxAgentsController = TextEditingController(text: widget.config.maxAgents.toString());
    _maxPrimsController = TextEditingController(text: widget.config.maxPrims.toString());
    _estateNameController = TextEditingController(text: widget.config.estateName);
    _estateOwnerController = TextEditingController(text: widget.config.estateOwner);
  }

  @override
  void didUpdateWidget(RegionIniEditor oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.config != widget.config) {
      _regionNameController.text = widget.config.regionName;
      _regionUuidController.text = widget.config.regionUuid;
      _locationXController.text = widget.config.locationX.toString();
      _locationYController.text = widget.config.locationY.toString();
      _sizeXController.text = widget.config.sizeX.toString();
      _sizeYController.text = widget.config.sizeY.toString();
      _internalPortController.text = widget.config.internalPort.toString();
      _maxAgentsController.text = widget.config.maxAgents.toString();
      _maxPrimsController.text = widget.config.maxPrims.toString();
      _estateNameController.text = widget.config.estateName;
      _estateOwnerController.text = widget.config.estateOwner;
    }
  }

  @override
  void dispose() {
    _regionNameController.dispose();
    _regionUuidController.dispose();
    _locationXController.dispose();
    _locationYController.dispose();
    _sizeXController.dispose();
    _sizeYController.dispose();
    _internalPortController.dispose();
    _maxAgentsController.dispose();
    _maxPrimsController.dispose();
    _estateNameController.dispose();
    _estateOwnerController.dispose();
    super.dispose();
  }

  void _notifyChange() {
    final provider = context.read<ConfigurationBuilderProvider>();
    provider.updateRegionIni(RegionIniConfig(
      regionName: _regionNameController.text,
      regionUuid: _regionUuidController.text,
      locationX: int.tryParse(_locationXController.text) ?? 1000,
      locationY: int.tryParse(_locationYController.text) ?? 1000,
      sizeX: int.tryParse(_sizeXController.text) ?? 256,
      sizeY: int.tryParse(_sizeYController.text) ?? 256,
      internalPort: int.tryParse(_internalPortController.text) ?? 9000,
      maxAgents: int.tryParse(_maxAgentsController.text) ?? 40,
      maxPrims: int.tryParse(_maxPrimsController.text) ?? 45000,
      estateName: _estateNameController.text,
      estateOwner: _estateOwnerController.text,
    ));
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildSectionHeader('Region Identity'),
          const SizedBox(height: 12),
          _buildIdentitySection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Location & Size'),
          const SizedBox(height: 12),
          _buildLocationSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Capacity Settings'),
          const SizedBox(height: 12),
          _buildCapacitySection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Estate Configuration'),
          const SizedBox(height: 12),
          _buildEstateSection(),
          const SizedBox(height: 24),
          _buildSectionHeader('Region Preview'),
          const SizedBox(height: 12),
          _buildRegionPreview(),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            _getSectionIcon(title),
            size: 18,
            color: Theme.of(context).colorScheme.onSecondaryContainer,
          ),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.onSecondaryContainer,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getSectionIcon(String title) {
    switch (title) {
      case 'Region Identity':
        return Icons.badge;
      case 'Location & Size':
        return Icons.place;
      case 'Capacity Settings':
        return Icons.people;
      case 'Estate Configuration':
        return Icons.home_work;
      case 'Region Preview':
        return Icons.preview;
      default:
        return Icons.settings;
    }
  }

  Widget _buildIdentitySection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextFormField(
              controller: _regionNameController,
              decoration: const InputDecoration(
                labelText: 'Region Name',
                hintText: 'Enter region name',
                prefixIcon: Icon(Icons.terrain),
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => _notifyChange(),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _regionUuidController,
                    decoration: const InputDecoration(
                      labelText: 'Region UUID',
                      hintText: 'Auto-generated if empty',
                      prefixIcon: Icon(Icons.fingerprint),
                      border: OutlineInputBorder(),
                    ),
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton.filled(
                  icon: const Icon(Icons.refresh),
                  tooltip: 'Generate new UUID',
                  onPressed: _generateUuid,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _generateUuid() {
    final uuid = _createUuid();
    _regionUuidController.text = uuid;
    _notifyChange();
  }

  String _createUuid() {
    final random = DateTime.now().millisecondsSinceEpoch;
    final hex = random.toRadixString(16).padLeft(12, '0');
    return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-4${hex.substring(0, 3)}-a${hex.substring(3, 6)}-${hex}';
  }

  Widget _buildLocationSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _locationXController,
                    decoration: const InputDecoration(
                      labelText: 'Location X',
                      hintText: '1000',
                      prefixIcon: Icon(Icons.east),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _locationYController,
                    decoration: const InputDecoration(
                      labelText: 'Location Y',
                      hintText: '1000',
                      prefixIcon: Icon(Icons.north),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _sizeXController,
                    decoration: const InputDecoration(
                      labelText: 'Size X (meters)',
                      hintText: '256',
                      prefixIcon: Icon(Icons.width_normal),
                      border: OutlineInputBorder(),
                      helperText: 'Standard: 256, Var: 512, 1024',
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _sizeYController,
                    decoration: const InputDecoration(
                      labelText: 'Size Y (meters)',
                      hintText: '256',
                      prefixIcon: Icon(Icons.height),
                      border: OutlineInputBorder(),
                      helperText: 'Standard: 256, Var: 512, 1024',
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildSizePresets(),
          ],
        ),
      ),
    );
  }

  Widget _buildSizePresets() {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        _buildSizePresetChip('Standard', 256, 256),
        _buildSizePresetChip('Medium', 512, 512),
        _buildSizePresetChip('Large', 1024, 1024),
        _buildSizePresetChip('Wide', 512, 256),
        _buildSizePresetChip('Tall', 256, 512),
      ],
    );
  }

  Widget _buildSizePresetChip(String label, int sizeX, int sizeY) {
    final isSelected = int.tryParse(_sizeXController.text) == sizeX &&
        int.tryParse(_sizeYController.text) == sizeY;

    return ChoiceChip(
      label: Text('$label (${sizeX}x$sizeY)'),
      selected: isSelected,
      onSelected: (selected) {
        if (selected) {
          _sizeXController.text = sizeX.toString();
          _sizeYController.text = sizeY.toString();
          _notifyChange();
        }
      },
    );
  }

  Widget _buildCapacitySection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextFormField(
              controller: _internalPortController,
              decoration: const InputDecoration(
                labelText: 'Internal Port',
                hintText: '9000',
                prefixIcon: Icon(Icons.router),
                border: OutlineInputBorder(),
                helperText: 'UDP port for region communication',
              ),
              keyboardType: TextInputType.number,
              onChanged: (_) => _notifyChange(),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _maxAgentsController,
                    decoration: const InputDecoration(
                      labelText: 'Max Agents',
                      hintText: '40',
                      prefixIcon: Icon(Icons.people),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _maxPrimsController,
                    decoration: const InputDecoration(
                      labelText: 'Max Prims',
                      hintText: '45000',
                      prefixIcon: Icon(Icons.category),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (_) => _notifyChange(),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildCapacityPresets(),
          ],
        ),
      ),
    );
  }

  Widget _buildCapacityPresets() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Quick Presets',
          style: TextStyle(
            fontSize: 12,
            color: Colors.grey[600],
            fontWeight: FontWeight.w500,
          ),
        ),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            _buildCapacityPresetChip('Low Traffic', 20, 15000),
            _buildCapacityPresetChip('Standard', 40, 45000),
            _buildCapacityPresetChip('High Traffic', 80, 60000),
            _buildCapacityPresetChip('Event', 200, 30000),
          ],
        ),
      ],
    );
  }

  Widget _buildCapacityPresetChip(String label, int maxAgents, int maxPrims) {
    return ActionChip(
      avatar: Icon(
        label == 'Event' ? Icons.celebration : Icons.tune,
        size: 16,
      ),
      label: Text('$label ($maxAgents users / ${maxPrims ~/ 1000}K prims)'),
      onPressed: () {
        _maxAgentsController.text = maxAgents.toString();
        _maxPrimsController.text = maxPrims.toString();
        _notifyChange();
      },
    );
  }

  Widget _buildEstateSection() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextFormField(
              controller: _estateNameController,
              decoration: const InputDecoration(
                labelText: 'Estate Name',
                hintText: 'My Estate',
                prefixIcon: Icon(Icons.home_work),
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => _notifyChange(),
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _estateOwnerController,
              decoration: const InputDecoration(
                labelText: 'Estate Owner',
                hintText: 'Grid Administrator',
                prefixIcon: Icon(Icons.person),
                border: OutlineInputBorder(),
                helperText: 'Owner username or UUID',
              ),
              onChanged: (_) => _notifyChange(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRegionPreview() {
    final sizeX = int.tryParse(_sizeXController.text) ?? 256;
    final sizeY = int.tryParse(_sizeYController.text) ?? 256;
    final locationX = int.tryParse(_locationXController.text) ?? 1000;
    final locationY = int.tryParse(_locationYController.text) ?? 1000;
    final maxAgents = int.tryParse(_maxAgentsController.text) ?? 40;
    final maxPrims = int.tryParse(_maxPrimsController.text) ?? 45000;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  width: 120,
                  height: 120,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topLeft,
                      end: Alignment.bottomRight,
                      colors: [
                        Colors.green[300]!,
                        Colors.green[600]!,
                      ],
                    ),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.green[800]!, width: 2),
                  ),
                  child: Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Text(
                          _regionNameController.text.isEmpty
                              ? 'Region'
                              : _regionNameController.text,
                          style: const TextStyle(
                            color: Colors.white,
                            fontWeight: FontWeight.bold,
                            fontSize: 12,
                          ),
                          textAlign: TextAlign.center,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          '${sizeX}x$sizeY',
                          style: const TextStyle(
                            color: Colors.white70,
                            fontSize: 10,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _buildPreviewRow('Name', _regionNameController.text.isEmpty
                          ? 'Not set'
                          : _regionNameController.text),
                      _buildPreviewRow('Location', '($locationX, $locationY)'),
                      _buildPreviewRow('Size', '${sizeX}x$sizeY meters'),
                      _buildPreviewRow('Capacity', '$maxAgents users / $maxPrims prims'),
                      _buildPreviewRow('Estate', _estateNameController.text.isEmpty
                          ? 'Not set'
                          : _estateNameController.text),
                      _buildPreviewRow('Port', _internalPortController.text),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildResourceEstimate(sizeX, sizeY, maxAgents, maxPrims),
          ],
        ),
      ),
    );
  }

  Widget _buildPreviewRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: TextStyle(
                color: Colors.grey[600],
                fontSize: 12,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontWeight: FontWeight.w500,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildResourceEstimate(int sizeX, int sizeY, int maxAgents, int maxPrims) {
    final regionMultiplier = (sizeX * sizeY) / (256 * 256);
    final baseMemory = 512;
    final agentMemory = maxAgents * 10;
    final primMemory = maxPrims ~/ 100;
    final estimatedMemory = ((baseMemory + agentMemory + primMemory) * regionMultiplier).round();

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.blue[50],
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.blue[200]!),
      ),
      child: Row(
        children: [
          Icon(Icons.memory, size: 20, color: Colors.blue[700]),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Estimated Memory: ~${estimatedMemory}MB',
                  style: TextStyle(
                    fontWeight: FontWeight.w500,
                    color: Colors.blue[900],
                  ),
                ),
                Text(
                  'Based on region size, capacity, and prim count',
                  style: TextStyle(
                    fontSize: 11,
                    color: Colors.blue[700],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
