import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:provider/provider.dart';
import '../models/archive_models.dart';
import '../providers/archive_provider.dart';
import '../services/admin_service.dart';
import '../widgets/archives/archive_job_list_widget.dart';
import '../widgets/archives/archive_job_details_widget.dart';
import '../widgets/archives/archive_operation_dialogs.dart';

class ArchiveManagementScreen extends StatefulWidget {
  const ArchiveManagementScreen({super.key});

  @override
  State<ArchiveManagementScreen> createState() => _ArchiveManagementScreenState();
}

class _ArchiveManagementScreenState extends State<ArchiveManagementScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  List<Map<String, String>> _users = [];
  List<Map<String, String>> _regions = [];
  List<Map<String, dynamic>> _oarFiles = [];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadJobs();
      _fetchUsersAndRegions();
    });
  }

  Future<void> _fetchUsersAndRegions() async {
    await AdminService.instance.ensureDiscovered();
    final adminUrl = AdminService.instance.adminUrl;

    try {
      final regionsResp = await http.get(
        Uri.parse('$adminUrl/console/regions'),
      );
      if (regionsResp.statusCode == 200) {
        final data = jsonDecode(regionsResp.body) as Map<String, dynamic>;
        final regionList = data['data']?['regions'] as List<dynamic>? ?? [];
        setState(() {
          _regions = regionList.map((r) {
            final m = r as Map<String, dynamic>;
            return {
              'id': (m['uuid'] ?? '').toString(),
              'name': (m['name'] ?? '').toString(),
            };
          }).toList();
        });
      }
    } catch (e) {
      debugPrint('Failed to fetch regions: $e');
    }

    try {
      final oarFilesResp = await http.get(
        Uri.parse('$adminUrl/admin/archives/oar/files'),
      );
      if (oarFilesResp.statusCode == 200) {
        final data = jsonDecode(oarFilesResp.body) as Map<String, dynamic>;
        final fileList = data['data']?['files'] as List<dynamic>? ?? [];
        setState(() {
          _oarFiles = fileList.cast<Map<String, dynamic>>();
        });
      }
    } catch (e) {
      debugPrint('Failed to fetch OAR files: $e');
    }

    try {
      final usersResp = await http.get(
        Uri.parse('$adminUrl/admin/users'),
      );
      if (usersResp.statusCode == 200) {
        final data = jsonDecode(usersResp.body) as Map<String, dynamic>;
        final userList = data['data']?['users'] as List<dynamic>? ?? [];
        setState(() {
          _users = userList
              .map((u) {
                final m = u as Map<String, dynamic>;
                final first = (m['firstname'] ?? '').toString().trim();
                final last = (m['lastname'] ?? '').toString().trim();
                final fullName = '$first $last'.trim();
                return {
                  'id': (m['user_id'] ?? '').toString(),
                  'name': fullName,
                };
              })
              .where((u) => u['name']!.isNotEmpty && u['id']!.isNotEmpty)
              .toList();
        });
      }
    } catch (e) {
      debugPrint('Failed to fetch users: $e');
    }
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  void _loadJobs() {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    if (provider.isConfigured) {
      provider.loadJobs(refresh: true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Archive Management'),
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.add),
            tooltip: 'New Archive Operation',
            onSelected: _handleMenuAction,
            itemBuilder: (context) => [
              const PopupMenuItem(
                value: 'load_iar',
                child: ListTile(
                  leading: Icon(Icons.upload_file, color: Colors.purple),
                  title: Text('Load IAR'),
                  subtitle: Text('Import inventory archive'),
                ),
              ),
              const PopupMenuItem(
                value: 'save_iar',
                child: ListTile(
                  leading: Icon(Icons.download, color: Colors.purple),
                  title: Text('Save IAR'),
                  subtitle: Text('Export inventory archive'),
                ),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(
                value: 'load_oar',
                child: ListTile(
                  leading: Icon(Icons.upload_file, color: Colors.teal),
                  title: Text('Load OAR'),
                  subtitle: Text('Import region archive'),
                ),
              ),
              const PopupMenuItem(
                value: 'save_oar',
                child: ListTile(
                  leading: Icon(Icons.download, color: Colors.teal),
                  title: Text('Save OAR'),
                  subtitle: Text('Export region archive'),
                ),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(
                value: 'clear_region',
                child: ListTile(
                  leading: Icon(Icons.delete_sweep, color: Colors.red),
                  title: Text('Clear Region'),
                  subtitle: Text('Remove all objects from region'),
                ),
              ),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadJobs,
            tooltip: 'Refresh',
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.list), text: 'All Jobs'),
            Tab(icon: Icon(Icons.sync), text: 'Active'),
            Tab(icon: Icon(Icons.check_circle), text: 'Completed'),
            Tab(icon: Icon(Icons.info), text: 'Details'),
          ],
        ),
      ),
      body: Consumer<ArchiveProvider>(
        builder: (context, provider, child) {
          if (!provider.isConfigured) {
            return _buildNotConfiguredState();
          }

          if (provider.isLoading && provider.jobs.isEmpty) {
            return const Center(child: CircularProgressIndicator());
          }

          if (provider.errorMessage != null && provider.jobs.isEmpty) {
            return _buildErrorState(provider);
          }

          return TabBarView(
            controller: _tabController,
            children: [
              _buildAllJobsTab(provider),
              _buildActiveJobsTab(provider),
              _buildCompletedJobsTab(provider),
              _buildDetailsTab(provider),
            ],
          );
        },
      ),
    );
  }

  Widget _buildNotConfiguredState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.settings_outlined, size: 64, color: Colors.grey[400]),
          const SizedBox(height: 16),
          Text(
            'Service Not Configured',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Text(
            'Please configure the backend connection\nin Settings to manage archives.',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.grey[600]),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(ArchiveProvider provider) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
          const SizedBox(height: 16),
          Text(
            'Error Loading Jobs',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Text(
            provider.errorMessage ?? 'Unknown error',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.grey[600]),
          ),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: _loadJobs,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildAllJobsTab(ArchiveProvider provider) {
    return Column(
      children: [
        if (provider.activeJobCount > 0)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(8),
            color: Colors.blue[50],
            child: Row(
              children: [
                const Icon(Icons.sync, color: Colors.blue, size: 20),
                const SizedBox(width: 8),
                Text(
                  '${provider.activeJobCount} active job${provider.activeJobCount > 1 ? 's' : ''}',
                  style: const TextStyle(
                    color: Colors.blue,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
          ),
        Expanded(
          child: ArchiveJobListWidget(
            jobs: provider.jobs,
            selectedJob: provider.selectedJob,
            onJobSelected: _selectJob,
            onJobCancelled: _cancelJob,
          ),
        ),
      ],
    );
  }

  Widget _buildActiveJobsTab(ArchiveProvider provider) {
    final activeJobs = [...provider.pendingJobs, ...provider.runningJobs];

    if (activeJobs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.check_circle_outline, size: 64, color: Colors.green[300]),
            const SizedBox(height: 16),
            Text(
              'No Active Jobs',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'All archive operations have completed.',
              style: TextStyle(color: Colors.grey[600]),
            ),
          ],
        ),
      );
    }

    return ArchiveJobListWidget(
      jobs: activeJobs,
      selectedJob: provider.selectedJob,
      onJobSelected: _selectJob,
      onJobCancelled: _cancelJob,
    );
  }

  Widget _buildCompletedJobsTab(ArchiveProvider provider) {
    final completedJobs = [...provider.completedJobs, ...provider.failedJobs];

    if (completedJobs.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.history, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'No Completed Jobs',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'Completed archive operations will appear here.',
              style: TextStyle(color: Colors.grey[600]),
            ),
          ],
        ),
      );
    }

    return ArchiveJobListWidget(
      jobs: completedJobs,
      selectedJob: provider.selectedJob,
      onJobSelected: _selectJob,
    );
  }

  Widget _buildDetailsTab(ArchiveProvider provider) {
    if (provider.selectedJob == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.touch_app, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'Select a Job',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              'Select an archive job from the list to view details.',
              style: TextStyle(color: Colors.grey[600]),
            ),
          ],
        ),
      );
    }

    return ArchiveJobDetailsWidget(
      job: provider.selectedJob!,
      onCancel: provider.selectedJob!.isActive ? () => _cancelJob(provider.selectedJob!) : null,
      onDownload: provider.selectedJob!.isComplete &&
              provider.selectedJob!.operation == JobOperation.save
          ? () => _downloadArchive(provider.selectedJob!)
          : null,
    );
  }

  void _handleMenuAction(String action) {
    switch (action) {
      case 'load_iar':
        _showLoadIarDialog();
        break;
      case 'save_iar':
        _showSaveIarDialog();
        break;
      case 'load_oar':
        _showLoadOarDialog();
        break;
      case 'save_oar':
        _showSaveOarDialog();
        break;
      case 'clear_region':
        _showClearRegionDialog();
        break;
    }
  }

  void _showLoadIarDialog() {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    showDialog(
      context: context,
      builder: (dialogContext) => IarLoadDialog(
        users: _users,
        onSubmit: (userFirstname, userLastname, filePath, merge, {bool createUser = false, String? userId, String? email, String? password}) async {
          final response = await provider.loadIar(
            userFirstname: userFirstname,
            userLastname: userLastname,
            filePath: filePath,
            merge: merge,
            createUserIfMissing: createUser,
            userId: userId,
            userEmail: email,
            userPassword: password,
          );
          _showResultSnackBar(response);
        },
      ),
    );
  }

  void _showSaveIarDialog() {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    showDialog(
      context: context,
      builder: (dialogContext) => IarSaveDialog(
        users: _users,
        onSubmit: (userFirstname, userLastname, includeAssets) async {
          final response = await provider.saveIar(
            userFirstname: userFirstname,
            userLastname: userLastname,
            outputPath: '/tmp/${userFirstname}_${userLastname}_inventory.iar',
            includeAssets: includeAssets,
          );
          _showResultSnackBar(response);
        },
      ),
    );
  }

  void _showLoadOarDialog() {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    showDialog(
      context: context,
      builder: (dialogContext) => OarLoadDialog(
        regions: _regions,
        users: _users,
        oarFiles: _oarFiles,
        onSubmit: (regionName, filePath, merge, loadTerrain, loadObjects,
            loadParcels, {String? defaultUserFirstname, String? defaultUserLastname}) async {
          final response = await provider.loadOar(
            regionName: regionName,
            filePath: filePath,
            merge: merge,
            defaultUserFirstname: defaultUserFirstname,
            defaultUserLastname: defaultUserLastname,
          );
          _showResultSnackBar(response);
        },
      ),
    );
  }

  void _showSaveOarDialog() {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    showDialog(
      context: context,
      builder: (dialogContext) => OarSaveDialog(
        regions: _regions,
        onSubmit: (regionName, includeAssets, includeTerrain, includeObjects,
            includeParcels) async {
          final response = await provider.saveOar(
            regionName: regionName,
            outputPath: '/tmp/${regionName.replaceAll(' ', '_')}_region.oar',
            includeAssets: includeAssets,
            includeTerrain: includeTerrain,
            includeObjects: includeObjects,
            includeParcels: includeParcels,
          );
          _showResultSnackBar(response);
        },
      ),
    );
  }

  void _showClearRegionDialog() async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Clear Region?'),
        content: const Text(
          'This will remove ALL objects from the region (database and live scene). '
          'Connected viewers will need to relog to see the change.\n\n'
          'This action cannot be undone.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Clear Region'),
          ),
        ],
      ),
    );

    if (confirm == true) {
      final provider = Provider.of<ArchiveProvider>(context, listen: false);
      final response = await provider.clearRegion();
      _showResultSnackBar(response);
    }
  }

  void _showResultSnackBar(ArchiveJobResponse response) {
    final messenger = ScaffoldMessenger.of(context);
    if (response.success) {
      messenger.showSnackBar(
        SnackBar(
          content: Text(response.message ?? 'Job started successfully'),
          backgroundColor: Colors.green,
        ),
      );
    } else {
      messenger.showSnackBar(
        SnackBar(
          content: Text(response.error ?? 'Operation failed'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  void _selectJob(ArchiveJob job) {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    provider.selectJob(job.id);
    _tabController.animateTo(3);
  }

  void _cancelJob(ArchiveJob job) async {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);

    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Cancel Job?'),
        content: Text('Are you sure you want to cancel ${job.displayName}?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('No'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: ElevatedButton.styleFrom(backgroundColor: Colors.orange),
            child: const Text('Yes, Cancel'),
          ),
        ],
      ),
    );

    if (confirm == true) {
      final success = await provider.cancelJob(job.id);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(success ? 'Job cancelled' : 'Failed to cancel job'),
            backgroundColor: success ? Colors.orange : Colors.red,
          ),
        );
      }
    }
  }

  void _downloadArchive(ArchiveJob job) async {
    final provider = Provider.of<ArchiveProvider>(context, listen: false);
    final url = await provider.downloadArchive(job.id);

    if (mounted) {
      if (url != null) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Download ready: $url'),
            backgroundColor: Colors.green,
            action: SnackBarAction(
              label: 'Copy',
              textColor: Colors.white,
              onPressed: () {},
            ),
          ),
        );
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Failed to get download URL'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }
}
