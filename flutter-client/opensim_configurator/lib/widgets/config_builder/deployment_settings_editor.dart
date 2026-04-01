import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../models/configuration_builder_models.dart';
import '../../providers/configuration_builder_provider.dart';

class DeploymentSettingsEditor extends StatefulWidget {
  final ContainerConfig config;
  final ValueChanged<ContainerConfig>? onChanged;

  const DeploymentSettingsEditor({
    super.key,
    required this.config,
    this.onChanged,
  });

  @override
  State<DeploymentSettingsEditor> createState() => _DeploymentSettingsEditorState();
}

class _DeploymentSettingsEditorState extends State<DeploymentSettingsEditor> {
  late TextEditingController _dockerImageController;
  late TextEditingController _memoryLimitController;
  late TextEditingController _cpuLimitController;
  late TextEditingController _namespaceController;
  late TextEditingController _replicasController;
  late TextEditingController _ingressHostController;
  late TextEditingController _targetPathController;

  @override
  void initState() {
    super.initState();
    _initControllers();
  }

  void _initControllers() {
    _dockerImageController = TextEditingController(text: widget.config.dockerImage ?? 'opensim-next:latest');
    _memoryLimitController = TextEditingController(text: widget.config.memoryLimitMB.toString());
    _cpuLimitController = TextEditingController(text: widget.config.cpuLimit.toString());
    _namespaceController = TextEditingController(text: widget.config.namespace ?? 'opensim');
    _replicasController = TextEditingController(text: widget.config.replicas.toString());
    _ingressHostController = TextEditingController(text: widget.config.ingressHost ?? '');
    _targetPathController = TextEditingController(text: '/opt/opensim/instances/new-instance');
  }

  @override
  void dispose() {
    _dockerImageController.dispose();
    _memoryLimitController.dispose();
    _cpuLimitController.dispose();
    _namespaceController.dispose();
    _replicasController.dispose();
    _ingressHostController.dispose();
    _targetPathController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Consumer<ConfigurationBuilderProvider>(
        builder: (context, provider, _) {
          final deploymentType = provider.currentDeploymentType;

          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildDeploymentTypeBanner(context),
              const SizedBox(height: 16),
              _buildSectionHeader(context, 'Deployment Type'),
              const SizedBox(height: 12),
              _buildDeploymentTypeSelector(context, provider, deploymentType),
              const SizedBox(height: 24),
              if (deploymentType == DeploymentType.native) ...[
                _buildSectionHeader(context, 'Native Deployment'),
                const SizedBox(height: 12),
                _buildNativeSettings(context, provider),
              ] else if (deploymentType == DeploymentType.docker) ...[
                _buildSectionHeader(context, 'Docker Settings'),
                const SizedBox(height: 12),
                _buildDockerSettings(context, provider),
              ] else if (deploymentType == DeploymentType.kubernetes) ...[
                _buildSectionHeader(context, 'Kubernetes Settings'),
                const SizedBox(height: 12),
                _buildKubernetesSettings(context, provider),
              ],
              const SizedBox(height: 24),
              _buildSectionHeader(context, 'Deployment Options'),
              const SizedBox(height: 12),
              _buildDeploymentOptions(context, provider),
            ],
          );
        },
      ),
    );
  }

  Widget _buildDeploymentTypeBanner(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: [Colors.deepPurple[400]!, Colors.deepPurple[600]!],
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
              Icons.rocket_launch,
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
                  'Deployment Configuration',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.bold,
                    fontSize: 18,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Choose how to deploy your OpenSim instance: native binary, Docker container, or Kubernetes orchestration.',
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

  Widget _buildSectionHeader(BuildContext context, String title) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            _getSectionIcon(title),
            size: 18,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              fontWeight: FontWeight.bold,
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getSectionIcon(String title) {
    switch (title) {
      case 'Deployment Type':
        return Icons.category;
      case 'Native Deployment':
        return Icons.computer;
      case 'Docker Settings':
        return Icons.inventory_2;
      case 'Kubernetes Settings':
        return Icons.hub;
      case 'Deployment Options':
        return Icons.settings;
      default:
        return Icons.settings;
    }
  }

  Widget _buildDeploymentTypeSelector(
    BuildContext context,
    ConfigurationBuilderProvider provider,
    DeploymentType currentType,
  ) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            _buildDeploymentTypeCard(
              context,
              provider,
              DeploymentType.native,
              currentType,
              'Native Binary',
              'Direct execution on host system',
              Icons.computer,
              Colors.blue,
              [
                'Best for development and testing',
                'Direct file system access',
                'Lower overhead',
              ],
            ),
            const SizedBox(height: 12),
            _buildDeploymentTypeCard(
              context,
              provider,
              DeploymentType.docker,
              currentType,
              'Docker Container',
              'Containerized deployment with Docker/Podman',
              Icons.inventory_2,
              Colors.cyan,
              [
                'Isolated environment',
                'Easy scaling and replication',
                'Portable across systems',
              ],
            ),
            const SizedBox(height: 12),
            _buildDeploymentTypeCard(
              context,
              provider,
              DeploymentType.kubernetes,
              currentType,
              'Kubernetes',
              'Enterprise orchestration with K8s/K3s',
              Icons.hub,
              Colors.purple,
              [
                'Auto-scaling and healing',
                'High availability',
                'Production-grade orchestration',
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDeploymentTypeCard(
    BuildContext context,
    ConfigurationBuilderProvider provider,
    DeploymentType type,
    DeploymentType currentType,
    String title,
    String subtitle,
    IconData icon,
    Color color,
    List<String> features,
  ) {
    final isSelected = type == currentType;

    return Material(
      color: isSelected ? color.withOpacity(0.1) : Colors.transparent,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        onTap: () => provider.updateDeploymentSettings(type),
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isSelected ? color : Colors.grey[300]!,
              width: isSelected ? 2 : 1,
            ),
          ),
          child: Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: color.withOpacity(0.1),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Icon(icon, color: color, size: 28),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          title,
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 16,
                            color: isSelected ? color : null,
                          ),
                        ),
                        if (isSelected) ...[
                          const SizedBox(width: 8),
                          Icon(Icons.check_circle, color: color, size: 18),
                        ],
                      ],
                    ),
                    Text(
                      subtitle,
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.grey[600],
                      ),
                    ),
                    const SizedBox(height: 8),
                    Wrap(
                      spacing: 8,
                      runSpacing: 4,
                      children: features
                          .map((f) => Chip(
                                label: Text(f),
                                labelStyle: const TextStyle(fontSize: 10),
                                padding: EdgeInsets.zero,
                                materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                              ))
                          .toList(),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildNativeSettings(BuildContext context, ConfigurationBuilderProvider provider) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            TextFormField(
              controller: _targetPathController,
              decoration: const InputDecoration(
                labelText: 'Target Path',
                hintText: '/opt/opensim/instances/my-instance',
                prefixIcon: Icon(Icons.folder),
                border: OutlineInputBorder(),
                helperText: 'Directory where INI files will be generated',
              ),
            ),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Colors.blue[50],
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.blue[200]!),
              ),
              child: Row(
                children: [
                  Icon(Icons.info, color: Colors.blue[700], size: 20),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Native Deployment',
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            color: Colors.blue[900],
                          ),
                        ),
                        Text(
                          'Configuration files will be written directly to the filesystem. The OpenSim binary will be executed directly without containerization.',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.blue[700],
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDockerSettings(BuildContext context, ConfigurationBuilderProvider provider) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextFormField(
              controller: _dockerImageController,
              decoration: const InputDecoration(
                labelText: 'Docker Image',
                hintText: 'opensim-next:latest',
                prefixIcon: Icon(Icons.inventory_2),
                border: OutlineInputBorder(),
                helperText: 'Container image to use',
              ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _memoryLimitController,
                    decoration: const InputDecoration(
                      labelText: 'Memory Limit (MB)',
                      hintText: '2048',
                      prefixIcon: Icon(Icons.memory),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _cpuLimitController,
                    decoration: const InputDecoration(
                      labelText: 'CPU Limit (cores)',
                      hintText: '2.0',
                      prefixIcon: Icon(Icons.developer_board),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildDockerRestartPolicy(provider),
            const SizedBox(height: 16),
            _buildDockerNetworkMode(provider),
          ],
        ),
      ),
    );
  }

  Widget _buildDockerRestartPolicy(ConfigurationBuilderProvider provider) {
    return DropdownButtonFormField<String>(
      value: 'unless-stopped',
      decoration: const InputDecoration(
        labelText: 'Restart Policy',
        prefixIcon: Icon(Icons.refresh),
        border: OutlineInputBorder(),
      ),
      items: const [
        DropdownMenuItem(value: 'no', child: Text('No')),
        DropdownMenuItem(value: 'always', child: Text('Always')),
        DropdownMenuItem(value: 'unless-stopped', child: Text('Unless Stopped')),
        DropdownMenuItem(value: 'on-failure', child: Text('On Failure')),
      ],
      onChanged: (value) {},
    );
  }

  Widget _buildDockerNetworkMode(ConfigurationBuilderProvider provider) {
    return DropdownButtonFormField<String>(
      value: 'opensim-network',
      decoration: const InputDecoration(
        labelText: 'Network Mode',
        prefixIcon: Icon(Icons.network_check),
        border: OutlineInputBorder(),
      ),
      items: const [
        DropdownMenuItem(value: 'opensim-network', child: Text('opensim-network (Custom)')),
        DropdownMenuItem(value: 'bridge', child: Text('Bridge')),
        DropdownMenuItem(value: 'host', child: Text('Host')),
        DropdownMenuItem(value: 'none', child: Text('None')),
      ],
      onChanged: (value) {},
    );
  }

  Widget _buildKubernetesSettings(BuildContext context, ConfigurationBuilderProvider provider) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _namespaceController,
                    decoration: const InputDecoration(
                      labelText: 'Namespace',
                      hintText: 'opensim',
                      prefixIcon: Icon(Icons.folder_special),
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _replicasController,
                    decoration: const InputDecoration(
                      labelText: 'Replicas',
                      hintText: '1',
                      prefixIcon: Icon(Icons.copy),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _memoryLimitController,
                    decoration: const InputDecoration(
                      labelText: 'Memory Limit (MB)',
                      hintText: '2048',
                      prefixIcon: Icon(Icons.memory),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _cpuLimitController,
                    decoration: const InputDecoration(
                      labelText: 'CPU Limit (cores)',
                      hintText: '2.0',
                      prefixIcon: Icon(Icons.developer_board),
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _ingressHostController,
              decoration: const InputDecoration(
                labelText: 'Ingress Host',
                hintText: 'grid.example.com',
                prefixIcon: Icon(Icons.public),
                border: OutlineInputBorder(),
                helperText: 'Public hostname for external access',
              ),
            ),
            const SizedBox(height: 16),
            _buildKubernetesOptions(provider),
          ],
        ),
      ),
    );
  }

  Widget _buildKubernetesOptions(ConfigurationBuilderProvider provider) {
    return Column(
      children: [
        SwitchListTile(
          title: const Row(
            children: [
              Icon(Icons.auto_graph, size: 20),
              SizedBox(width: 8),
              Text('Enable HPA'),
            ],
          ),
          subtitle: const Text('Horizontal Pod Autoscaler for automatic scaling'),
          value: true,
          onChanged: (value) {},
        ),
        const Divider(),
        SwitchListTile(
          title: const Row(
            children: [
              Icon(Icons.https, size: 20),
              SizedBox(width: 8),
              Text('Enable TLS'),
            ],
          ),
          subtitle: const Text('HTTPS/TLS termination at ingress'),
          value: true,
          onChanged: (value) {},
        ),
        const Divider(),
        SwitchListTile(
          title: const Row(
            children: [
              Icon(Icons.storage, size: 20),
              SizedBox(width: 8),
              Text('Persistent Storage'),
            ],
          ),
          subtitle: const Text('Use PersistentVolumeClaim for data'),
          value: true,
          onChanged: (value) {},
        ),
      ],
    );
  }

  Widget _buildDeploymentOptions(BuildContext context, ConfigurationBuilderProvider provider) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            SwitchListTile(
              title: const Row(
                children: [
                  Icon(Icons.play_arrow, size: 20),
                  SizedBox(width: 8),
                  Text('Auto-start after deployment'),
                ],
              ),
              subtitle: const Text('Automatically start the instance after deployment completes'),
              value: true,
              onChanged: (value) {},
            ),
            const Divider(),
            SwitchListTile(
              title: const Row(
                children: [
                  Icon(Icons.app_registration, size: 20),
                  SizedBox(width: 8),
                  Text('Register with Instance Manager'),
                ],
              ),
              subtitle: const Text('Add to instances.toml for monitoring and management'),
              value: true,
              onChanged: (value) {},
            ),
            const Divider(),
            SwitchListTile(
              title: const Row(
                children: [
                  Icon(Icons.backup, size: 20),
                  SizedBox(width: 8),
                  Text('Create backup before deploy'),
                ],
              ),
              subtitle: const Text('Backup existing configuration if updating'),
              value: false,
              onChanged: (value) {},
            ),
          ],
        ),
      ),
    );
  }
}

class DeploymentPreview extends StatelessWidget {
  final DeploymentType deploymentType;
  final ContainerConfig config;

  const DeploymentPreview({
    super.key,
    required this.deploymentType,
    required this.config,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  _getDeploymentIcon(),
                  color: _getDeploymentColor(),
                ),
                const SizedBox(width: 8),
                Text(
                  'Deployment Preview',
                  style: const TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 16,
                  ),
                ),
              ],
            ),
            const Divider(),
            _buildPreviewContent(),
          ],
        ),
      ),
    );
  }

  IconData _getDeploymentIcon() {
    switch (deploymentType) {
      case DeploymentType.native:
        return Icons.computer;
      case DeploymentType.docker:
        return Icons.inventory_2;
      case DeploymentType.kubernetes:
        return Icons.hub;
    }
  }

  Color _getDeploymentColor() {
    switch (deploymentType) {
      case DeploymentType.native:
        return Colors.blue;
      case DeploymentType.docker:
        return Colors.cyan;
      case DeploymentType.kubernetes:
        return Colors.purple;
    }
  }

  Widget _buildPreviewContent() {
    switch (deploymentType) {
      case DeploymentType.native:
        return _buildNativePreview();
      case DeploymentType.docker:
        return _buildDockerPreview();
      case DeploymentType.kubernetes:
        return _buildKubernetesPreview();
    }
  }

  Widget _buildNativePreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildPreviewRow('Type', 'Native Binary Execution'),
        _buildPreviewRow('Binary', 'opensim-next'),
        _buildPreviewRow('Config Path', '/opt/opensim/instances/'),
      ],
    );
  }

  Widget _buildDockerPreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildPreviewRow('Type', 'Docker Container'),
        _buildPreviewRow('Image', config.dockerImage ?? 'opensim-next:latest'),
        _buildPreviewRow('Memory', '${config.memoryLimitMB}MB'),
        _buildPreviewRow('CPU', '${config.cpuLimit} cores'),
      ],
    );
  }

  Widget _buildKubernetesPreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildPreviewRow('Type', 'Kubernetes Deployment'),
        _buildPreviewRow('Namespace', config.namespace ?? 'opensim'),
        _buildPreviewRow('Replicas', config.replicas.toString()),
        _buildPreviewRow('Memory', '${config.memoryLimitMB}MB'),
        _buildPreviewRow('Ingress', config.ingressHost ?? 'Not configured'),
      ],
    );
  }

  Widget _buildPreviewRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 100,
            child: Text(
              label,
              style: TextStyle(
                color: Colors.grey[600],
                fontSize: 13,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(
                fontWeight: FontWeight.w500,
                fontSize: 13,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
