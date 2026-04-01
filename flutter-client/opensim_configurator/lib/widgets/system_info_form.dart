// OpenSim Next Configurator - System Info Form Widget
// Form for collecting system information for auto-detection

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/deployment_models.dart';
import '../theme/app_theme.dart';

class SystemInfoForm extends StatefulWidget {
  final SystemInfo? initialData;
  final Function(SystemInfo) onChanged;

  const SystemInfoForm({
    Key? key,
    this.initialData,
    required this.onChanged,
  }) : super(key: key);

  @override
  _SystemInfoFormState createState() => _SystemInfoFormState();
}

class _SystemInfoFormState extends State<SystemInfoForm> {
  final _formKey = GlobalKey<FormState>();
  
  late TextEditingController _memoryController;
  late TextEditingController _cpuController;
  late TextEditingController _bandwidthController;
  late TextEditingController _domainController;
  late TextEditingController _usersController;
  late TextEditingController _regionsController;
  
  bool _hasPublicIp = false;
  bool _isCommercial = false;

  @override
  void initState() {
    super.initState();
    
    final initial = widget.initialData ?? SystemInfo(
      memoryGb: 8.0,
      cpuCores: 4,
      hasPublicIp: false,
      bandwidthMbps: 100,
      domain: 'localhost',
      expectedUsers: 5,
      expectedRegions: 1,
      isCommercial: false,
    );
    
    _memoryController = TextEditingController(text: initial.memoryGb.toString());
    _cpuController = TextEditingController(text: initial.cpuCores.toString());
    _bandwidthController = TextEditingController(text: initial.bandwidthMbps.toString());
    _domainController = TextEditingController(text: initial.domain);
    _usersController = TextEditingController(text: initial.expectedUsers.toString());
    _regionsController = TextEditingController(text: initial.expectedRegions.toString());
    
    _hasPublicIp = initial.hasPublicIp;
    _isCommercial = initial.isCommercial;
    
    // Add listeners to notify parent of changes
    _memoryController.addListener(_notifyChanges);
    _cpuController.addListener(_notifyChanges);
    _bandwidthController.addListener(_notifyChanges);
    _domainController.addListener(_notifyChanges);
    _usersController.addListener(_notifyChanges);
    _regionsController.addListener(_notifyChanges);
  }

  @override
  void dispose() {
    _memoryController.dispose();
    _cpuController.dispose();
    _bandwidthController.dispose();
    _domainController.dispose();
    _usersController.dispose();
    _regionsController.dispose();
    super.dispose();
  }

  void _notifyChanges() {
    if (_formKey.currentState?.validate() == true) {
      final systemInfo = SystemInfo(
        memoryGb: double.tryParse(_memoryController.text) ?? 8.0,
        cpuCores: int.tryParse(_cpuController.text) ?? 4,
        hasPublicIp: _hasPublicIp,
        bandwidthMbps: int.tryParse(_bandwidthController.text) ?? 100,
        domain: _domainController.text,
        expectedUsers: int.tryParse(_usersController.text) ?? 5,
        expectedRegions: int.tryParse(_regionsController.text) ?? 1,
        isCommercial: _isCommercial,
      );
      widget.onChanged(systemInfo);
    }
  }

  bool validate() {
    return _formKey.currentState?.validate() == true;
  }

  @override
  Widget build(BuildContext context) {
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Hardware Section
          _buildSectionHeader('Hardware Information'),
          SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: _memoryController,
                  decoration: InputDecoration(
                    labelText: 'Memory (GB)',
                    hintText: '8.0',
                    prefixIcon: Icon(Icons.memory),
                  ),
                  keyboardType: TextInputType.numberWithOptions(decimal: true),
                  inputFormatters: [
                    FilteringTextInputFormatter.allow(RegExp(r'^\d*\.?\d*')),
                  ],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Required';
                    }
                    final memory = double.tryParse(value);
                    if (memory == null || memory <= 0) {
                      return 'Invalid memory size';
                    }
                    return null;
                  },
                ),
              ),
              SizedBox(width: 16),
              Expanded(
                child: TextFormField(
                  controller: _cpuController,
                  decoration: InputDecoration(
                    labelText: 'CPU Cores',
                    hintText: '4',
                    prefixIcon: Icon(Icons.computer),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                  ],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Required';
                    }
                    final cores = int.tryParse(value);
                    if (cores == null || cores <= 0) {
                      return 'Invalid CPU cores';
                    }
                    return null;
                  },
                ),
              ),
            ],
          ),
          SizedBox(height: 16),

          // Network Section
          _buildSectionHeader('Network Information'),
          SizedBox(height: 12),
          TextFormField(
            controller: _domainController,
            decoration: InputDecoration(
              labelText: 'Domain/Hostname',
              hintText: 'localhost',
              prefixIcon: Icon(Icons.language),
            ),
            validator: (value) {
              if (value == null || value.isEmpty) {
                return 'Required';
              }
              return null;
            },
          ),
          SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: _bandwidthController,
                  decoration: InputDecoration(
                    labelText: 'Bandwidth (Mbps)',
                    hintText: '100',
                    prefixIcon: Icon(Icons.network_check),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                  ],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Required';
                    }
                    final bandwidth = int.tryParse(value);
                    if (bandwidth == null || bandwidth <= 0) {
                      return 'Invalid bandwidth';
                    }
                    return null;
                  },
                ),
              ),
              SizedBox(width: 16),
              Expanded(
                child: SwitchListTile(
                  title: Text('Public IP'),
                  subtitle: Text('Server has public IP address'),
                  value: _hasPublicIp,
                  onChanged: (value) {
                    setState(() {
                      _hasPublicIp = value;
                    });
                    _notifyChanges();
                  },
                  contentPadding: EdgeInsets.zero,
                ),
              ),
            ],
          ),
          SizedBox(height: 16),

          // Usage Section
          _buildSectionHeader('Expected Usage'),
          SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: _usersController,
                  decoration: InputDecoration(
                    labelText: 'Concurrent Users',
                    hintText: '5',
                    prefixIcon: Icon(Icons.people),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                  ],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Required';
                    }
                    final users = int.tryParse(value);
                    if (users == null || users <= 0) {
                      return 'Invalid user count';
                    }
                    return null;
                  },
                ),
              ),
              SizedBox(width: 16),
              Expanded(
                child: TextFormField(
                  controller: _regionsController,
                  decoration: InputDecoration(
                    labelText: 'Regions',
                    hintText: '1',
                    prefixIcon: Icon(Icons.map),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                  ],
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Required';
                    }
                    final regions = int.tryParse(value);
                    if (regions == null || regions <= 0) {
                      return 'Invalid region count';
                    }
                    return null;
                  },
                ),
              ),
            ],
          ),
          SizedBox(height: 16),

          // Commercial Use Toggle
          SwitchListTile(
            title: Text('Commercial Use'),
            subtitle: Text('This will be used for commercial purposes'),
            value: _isCommercial,
            onChanged: (value) {
              setState(() {
                _isCommercial = value;
              });
              _notifyChanges();
            },
            contentPadding: EdgeInsets.zero,
          ),

          // Help Text
          SizedBox(height: 16),
          Container(
            padding: EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: AppTheme.infoColor.withValues(alpha: 0.1),
              border: Border.all(color: AppTheme.infoColor.withValues(alpha: 0.3)),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Row(
              children: [
                Icon(Icons.info_outline, color: AppTheme.infoColor),
                SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'This information helps us recommend the best deployment type for your needs. All data stays on your device.',
                    style: TextStyle(
                      color: AppTheme.infoColor,
                      fontSize: 12,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Text(
      title,
      style: AppTheme.titleStyle().copyWith(
        color: AppTheme.primaryColor,
        fontSize: 16,
      ),
    );
  }
}