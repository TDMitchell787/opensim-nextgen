import 'package:flutter/material.dart';

class IarLoadDialog extends StatefulWidget {
  final List<Map<String, String>> users;
  final Function(String userFirstname, String userLastname, String filePath, bool merge, {bool createUser, String? userId, String? email, String? password}) onSubmit;

  const IarLoadDialog({
    super.key,
    required this.users,
    required this.onSubmit,
  });

  @override
  State<IarLoadDialog> createState() => _IarLoadDialogState();
}

class _IarLoadDialogState extends State<IarLoadDialog> {
  final _formKey = GlobalKey<FormState>();
  final _filePathController = TextEditingController();
  final _firstnameController = TextEditingController();
  final _lastnameController = TextEditingController();
  final _userIdController = TextEditingController();
  final _emailController = TextEditingController();
  final _passwordController = TextEditingController(text: 'changeme');
  String? _selectedUserName;
  bool _merge = false;
  bool _createNewUser = false;

  @override
  void dispose() {
    _filePathController.dispose();
    _firstnameController.dispose();
    _lastnameController.dispose();
    _userIdController.dispose();
    _emailController.dispose();
    _passwordController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.upload_file, color: Colors.purple),
          SizedBox(width: 8),
          Text('Load IAR (Inventory Archive)'),
        ],
      ),
      content: Form(
        key: _formKey,
        child: SizedBox(
          width: 450,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                SwitchListTile(
                  title: const Text('Create new user for import'),
                  subtitle: const Text('Use when importing for a user not yet in the grid'),
                  value: _createNewUser,
                  onChanged: (value) => setState(() => _createNewUser = value),
                ),
                const SizedBox(height: 12),
                if (!_createNewUser)
                  DropdownButtonFormField<String>(
                    decoration: const InputDecoration(
                      labelText: 'Existing User',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.person),
                    ),
                    value: _selectedUserName,
                    items: widget.users
                        .map((u) => DropdownMenuItem(
                              value: u['name'],
                              child: Text(u['name'] ?? u['id']!),
                            ))
                        .toList(),
                    onChanged: (value) => setState(() => _selectedUserName = value),
                    validator: (value) =>
                        !_createNewUser && value == null ? 'Please select a user' : null,
                  ),
                if (_createNewUser) ...[
                  TextFormField(
                    controller: _firstnameController,
                    decoration: const InputDecoration(
                      labelText: 'First Name',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.person),
                    ),
                    validator: (value) =>
                        _createNewUser && (value?.isEmpty ?? true) ? 'Required' : null,
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _lastnameController,
                    decoration: const InputDecoration(
                      labelText: 'Last Name',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.person_outline),
                    ),
                    validator: (value) =>
                        _createNewUser && (value?.isEmpty ?? true) ? 'Required' : null,
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _userIdController,
                    decoration: const InputDecoration(
                      labelText: 'Original UUID (from IAR source grid)',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.fingerprint),
                      hintText: '00000000-0000-0000-0000-000000000000',
                      helperText: 'Preserves asset ownership — paste from source grid',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _emailController,
                    decoration: const InputDecoration(
                      labelText: 'Email',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.email),
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _passwordController,
                    decoration: const InputDecoration(
                      labelText: 'Password',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.lock),
                    ),
                  ),
                ],
                const SizedBox(height: 16),
                TextFormField(
                  controller: _filePathController,
                  decoration: const InputDecoration(
                    labelText: 'IAR File Path',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.folder),
                    hintText: '/path/to/archive.iar',
                  ),
                  validator: (value) =>
                      value?.isEmpty ?? true ? 'Please enter file path' : null,
                ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Merge with existing inventory'),
                  subtitle: const Text('Keep existing items instead of replacing'),
                  value: _merge,
                  onChanged: (value) => setState(() => _merge = value),
                ),
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.upload),
          label: const Text('Load IAR'),
        ),
      ],
    );
  }

  void _submit() {
    if (_formKey.currentState!.validate()) {
      String firstname;
      String lastname;
      if (_createNewUser) {
        firstname = _firstnameController.text.trim();
        lastname = _lastnameController.text.trim();
      } else {
        final parts = _selectedUserName!.split(' ');
        firstname = parts.first;
        lastname = parts.length > 1 ? parts.sublist(1).join(' ') : '';
      }
      widget.onSubmit(
        firstname, lastname, _filePathController.text, _merge,
        createUser: _createNewUser,
        userId: _userIdController.text.trim().isEmpty ? null : _userIdController.text.trim(),
        email: _emailController.text.trim().isEmpty ? null : _emailController.text.trim(),
        password: _passwordController.text.trim().isEmpty ? null : _passwordController.text.trim(),
      );
      Navigator.of(context).pop();
    }
  }
}

class IarSaveDialog extends StatefulWidget {
  final List<Map<String, String>> users;
  final Function(String userFirstname, String userLastname, bool includeAssets) onSubmit;

  const IarSaveDialog({
    super.key,
    required this.users,
    required this.onSubmit,
  });

  @override
  State<IarSaveDialog> createState() => _IarSaveDialogState();
}

class _IarSaveDialogState extends State<IarSaveDialog> {
  final _formKey = GlobalKey<FormState>();
  String? _selectedUserName;
  bool _includeAssets = true;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.download, color: Colors.purple),
          SizedBox(width: 8),
          Text('Save IAR (Inventory Archive)'),
        ],
      ),
      content: Form(
        key: _formKey,
        child: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              DropdownButtonFormField<String>(
                decoration: const InputDecoration(
                  labelText: 'User',
                  border: OutlineInputBorder(),
                  prefixIcon: Icon(Icons.person),
                ),
                value: _selectedUserName,
                items: widget.users
                    .map((u) => DropdownMenuItem(
                          value: u['name'],
                          child: Text(u['name'] ?? u['id']!),
                        ))
                    .toList(),
                onChanged: (value) => setState(() => _selectedUserName = value),
                validator: (value) =>
                    value == null ? 'Please select a user' : null,
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                title: const Text('Include Assets'),
                subtitle: const Text('Include texture, sound, and other asset data'),
                value: _includeAssets,
                onChanged: (value) => setState(() => _includeAssets = value),
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.download),
          label: const Text('Save IAR'),
        ),
      ],
    );
  }

  void _submit() {
    if (_formKey.currentState!.validate()) {
      final parts = _selectedUserName!.split(' ');
      final firstname = parts.first;
      final lastname = parts.length > 1 ? parts.sublist(1).join(' ') : '';
      widget.onSubmit(firstname, lastname, _includeAssets);
      Navigator.of(context).pop();
    }
  }
}

class OarLoadDialog extends StatefulWidget {
  final List<Map<String, String>> regions;
  final List<Map<String, String>> users;
  final List<Map<String, dynamic>> oarFiles;
  final Function(String regionName, String filePath, bool merge, bool loadTerrain,
      bool loadObjects, bool loadParcels, {String? defaultUserFirstname, String? defaultUserLastname}) onSubmit;

  const OarLoadDialog({
    super.key,
    required this.regions,
    required this.users,
    this.oarFiles = const [],
    required this.onSubmit,
  });

  @override
  State<OarLoadDialog> createState() => _OarLoadDialogState();
}

class _OarLoadDialogState extends State<OarLoadDialog> {
  final _formKey = GlobalKey<FormState>();
  String? _selectedRegionName;
  String? _selectedOarFile;
  String? _selectedDefaultUser;
  bool _reassignOwnership = false;
  bool _merge = false;
  bool _loadTerrain = true;
  bool _loadObjects = true;
  bool _loadParcels = true;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.upload_file, color: Colors.teal),
          SizedBox(width: 8),
          Text('Load OAR (Region Archive)'),
        ],
      ),
      content: Form(
        key: _formKey,
        child: SizedBox(
          width: 450,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButtonFormField<String>(
                  decoration: const InputDecoration(
                    labelText: 'Region',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.landscape),
                  ),
                  value: _selectedRegionName,
                  items: widget.regions
                      .map((r) => DropdownMenuItem(
                            value: r['name'],
                            child: Text(r['name'] ?? r['id']!),
                          ))
                      .toList(),
                  onChanged: (value) =>
                      setState(() => _selectedRegionName = value),
                  validator: (value) =>
                      value == null ? 'Please select a region' : null,
                ),
                const SizedBox(height: 16),
                if (widget.oarFiles.isNotEmpty)
                  DropdownButtonFormField<String>(
                    decoration: const InputDecoration(
                      labelText: 'OAR File',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.folder),
                    ),
                    value: _selectedOarFile,
                    items: widget.oarFiles.map((f) {
                      final name = f['name'] as String? ?? '';
                      final size = f['size'] as num? ?? 0;
                      final sizeMb = (size / (1024 * 1024)).toStringAsFixed(1);
                      return DropdownMenuItem(
                        value: f['path'] as String? ?? name,
                        child: Text('$name ($sizeMb MB)'),
                      );
                    }).toList(),
                    onChanged: (value) => setState(() => _selectedOarFile = value),
                    validator: (value) =>
                        value == null ? 'Please select an OAR file' : null,
                  )
                else
                  const Text(
                    'No OAR files found in instance OAR directory',
                    style: TextStyle(color: Colors.orange),
                  ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Reassign object ownership'),
                  subtitle: const Text('Assign all imported objects to a local grid user'),
                  value: _reassignOwnership,
                  onChanged: (value) => setState(() => _reassignOwnership = value),
                ),
                if (_reassignOwnership) ...[
                  const SizedBox(height: 12),
                  DropdownButtonFormField<String>(
                    decoration: const InputDecoration(
                      labelText: 'Default Owner',
                      border: OutlineInputBorder(),
                      prefixIcon: Icon(Icons.person),
                      helperText: 'All objects and parcels will be owned by this user',
                    ),
                    value: _selectedDefaultUser,
                    items: widget.users
                        .map((u) => DropdownMenuItem(
                              value: u['name'],
                              child: Text(u['name'] ?? u['id']!),
                            ))
                        .toList(),
                    onChanged: (value) => setState(() => _selectedDefaultUser = value),
                    validator: (value) =>
                        _reassignOwnership && value == null ? 'Please select an owner' : null,
                  ),
                ],
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Merge with existing content'),
                  value: _merge,
                  onChanged: (value) => setState(() => _merge = value),
                ),
                SwitchListTile(
                  title: const Text('Load Terrain'),
                  value: _loadTerrain,
                  onChanged: (value) => setState(() => _loadTerrain = value),
                ),
                SwitchListTile(
                  title: const Text('Load Objects'),
                  value: _loadObjects,
                  onChanged: (value) => setState(() => _loadObjects = value),
                ),
                SwitchListTile(
                  title: const Text('Load Parcels'),
                  value: _loadParcels,
                  onChanged: (value) => setState(() => _loadParcels = value),
                ),
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.upload),
          label: const Text('Load OAR'),
        ),
      ],
    );
  }

  void _submit() {
    if (_formKey.currentState!.validate()) {
      String? defaultFirstname;
      String? defaultLastname;
      if (_reassignOwnership && _selectedDefaultUser != null) {
        final parts = _selectedDefaultUser!.split(' ');
        defaultFirstname = parts.first;
        defaultLastname = parts.length > 1 ? parts.sublist(1).join(' ') : '';
      }
      widget.onSubmit(
        _selectedRegionName!,
        _selectedOarFile ?? '',
        _merge,
        _loadTerrain,
        _loadObjects,
        _loadParcels,
        defaultUserFirstname: defaultFirstname,
        defaultUserLastname: defaultLastname,
      );
      Navigator.of(context).pop();
    }
  }
}

class OarSaveDialog extends StatefulWidget {
  final List<Map<String, String>> regions;
  final Function(String regionName, bool includeAssets, bool includeTerrain,
      bool includeObjects, bool includeParcels) onSubmit;

  const OarSaveDialog({
    super.key,
    required this.regions,
    required this.onSubmit,
  });

  @override
  State<OarSaveDialog> createState() => _OarSaveDialogState();
}

class _OarSaveDialogState extends State<OarSaveDialog> {
  final _formKey = GlobalKey<FormState>();
  String? _selectedRegionName;
  bool _includeAssets = true;
  bool _includeTerrain = true;
  bool _includeObjects = true;
  bool _includeParcels = true;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Row(
        children: [
          Icon(Icons.download, color: Colors.teal),
          SizedBox(width: 8),
          Text('Save OAR (Region Archive)'),
        ],
      ),
      content: Form(
        key: _formKey,
        child: SizedBox(
          width: 400,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButtonFormField<String>(
                  decoration: const InputDecoration(
                    labelText: 'Region',
                    border: OutlineInputBorder(),
                    prefixIcon: Icon(Icons.landscape),
                  ),
                  value: _selectedRegionName,
                  items: widget.regions
                      .map((r) => DropdownMenuItem(
                            value: r['name'],
                            child: Text(r['name'] ?? r['id']!),
                          ))
                      .toList(),
                  onChanged: (value) =>
                      setState(() => _selectedRegionName = value),
                  validator: (value) =>
                      value == null ? 'Please select a region' : null,
                ),
                const SizedBox(height: 16),
                SwitchListTile(
                  title: const Text('Include Assets'),
                  value: _includeAssets,
                  onChanged: (value) => setState(() => _includeAssets = value),
                ),
                SwitchListTile(
                  title: const Text('Include Terrain'),
                  value: _includeTerrain,
                  onChanged: (value) => setState(() => _includeTerrain = value),
                ),
                SwitchListTile(
                  title: const Text('Include Objects'),
                  value: _includeObjects,
                  onChanged: (value) => setState(() => _includeObjects = value),
                ),
                SwitchListTile(
                  title: const Text('Include Parcels'),
                  value: _includeParcels,
                  onChanged: (value) => setState(() => _includeParcels = value),
                ),
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        ElevatedButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.download),
          label: const Text('Save OAR'),
        ),
      ],
    );
  }

  void _submit() {
    if (_formKey.currentState!.validate()) {
      widget.onSubmit(
        _selectedRegionName!,
        _includeAssets,
        _includeTerrain,
        _includeObjects,
        _includeParcels,
      );
      Navigator.of(context).pop();
    }
  }
}
