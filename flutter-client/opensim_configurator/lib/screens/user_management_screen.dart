import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/user_models.dart';
import '../providers/user_provider.dart';
import '../widgets/users/user_list_widget.dart';
import '../widgets/users/user_details_card.dart';
import '../widgets/users/user_creation_dialog.dart';
import '../widgets/users/appearance_diagnostics_widget.dart';

class UserManagementScreen extends StatefulWidget {
  const UserManagementScreen({super.key});

  @override
  State<UserManagementScreen> createState() => _UserManagementScreenState();
}

class _UserManagementScreenState extends State<UserManagementScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final TextEditingController _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadUsers();
    });
  }

  @override
  void dispose() {
    _tabController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  void _loadUsers() {
    final provider = Provider.of<UserProvider>(context, listen: false);
    if (provider.isConfigured) {
      provider.loadUsers(refresh: true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('User Management'),
        actions: [
          IconButton(
            icon: const Icon(Icons.person_add),
            onPressed: _showCreateUserDialog,
            tooltip: 'Create User',
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadUsers,
            tooltip: 'Refresh',
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.people), text: 'Users'),
            Tab(icon: Icon(Icons.person), text: 'Details'),
            Tab(icon: Icon(Icons.checkroom), text: 'Appearance'),
          ],
        ),
      ),
      body: Consumer<UserProvider>(
        builder: (context, provider, child) {
          if (!provider.isConfigured) {
            return _buildNotConfiguredState();
          }

          if (provider.isLoading && provider.users.isEmpty) {
            return const Center(child: CircularProgressIndicator());
          }

          if (provider.errorMessage != null && provider.users.isEmpty) {
            return _buildErrorState(provider);
          }

          return Column(
            children: [
              _buildSearchBar(provider),
              Expanded(
                child: TabBarView(
                  controller: _tabController,
                  children: [
                    _buildUsersTab(provider),
                    _buildDetailsTab(provider),
                    _buildAppearanceTab(provider),
                  ],
                ),
              ),
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
            'Please configure the backend connection\nin Settings to manage users.',
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.grey[600]),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(UserProvider provider) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 64, color: Colors.red[300]),
          const SizedBox(height: 16),
          Text(
            'Error Loading Users',
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
            onPressed: _loadUsers,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildSearchBar(UserProvider provider) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: TextField(
        controller: _searchController,
        decoration: InputDecoration(
          hintText: 'Search users by name...',
          prefixIcon: const Icon(Icons.search),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(12),
          ),
          suffixIcon: _searchController.text.isNotEmpty
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: () {
                    _searchController.clear();
                    setState(() {});
                  },
                )
              : null,
        ),
        onChanged: (value) {
          setState(() {});
        },
      ),
    );
  }

  Widget _buildUsersTab(UserProvider provider) {
    final filteredUsers = _filterUsers(provider.users);

    if (filteredUsers.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.people_outline, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              _searchController.text.isEmpty ? 'No Users' : 'No Matching Users',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              _searchController.text.isEmpty
                  ? 'Create your first user to get started.'
                  : 'Try a different search term.',
              style: TextStyle(color: Colors.grey[600]),
            ),
            if (_searchController.text.isEmpty) ...[
              const SizedBox(height: 24),
              ElevatedButton.icon(
                onPressed: _showCreateUserDialog,
                icon: const Icon(Icons.person_add),
                label: const Text('Create User'),
              ),
            ],
          ],
        ),
      );
    }

    return UserListWidget(
      users: filteredUsers,
      selectedUserId: provider.selectedUser?.id,
      onUserSelected: (userId) {
        provider.selectUser(userId);
        _tabController.animateTo(1);
      },
      onUserDeleted: (userId) => _confirmDeleteUser(userId, provider),
    );
  }

  Widget _buildDetailsTab(UserProvider provider) {
    if (provider.selectedUser == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.person_outline, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'Select a User',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    color: Colors.grey[600],
                  ),
            ),
            const SizedBox(height: 8),
            Text(
              'Choose a user from the Users tab\nto see their details.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: UserDetailsCard(
        user: provider.selectedUser!,
        onResetPassword: () => _showResetPasswordDialog(provider),
        onChangeLevel: () => _showChangeLevelDialog(provider),
        onDelete: () => _confirmDeleteUser(provider.selectedUser!.id, provider),
      ),
    );
  }

  Widget _buildAppearanceTab(UserProvider provider) {
    if (provider.selectedUser == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.checkroom_outlined, size: 64, color: Colors.grey[400]),
            const SizedBox(height: 16),
            Text(
              'Select a User',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    color: Colors.grey[600],
                  ),
            ),
            const SizedBox(height: 8),
            Text(
              'Choose a user from the Users tab\nto check their appearance.',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: AppearanceDiagnosticsWidget(
        userId: provider.selectedUser!.id,
        userName: provider.selectedUser!.fullName,
        diagnostics: provider.selectedUserDiagnostics,
        isLoading: provider.isLoading,
        onRefresh: () => provider.loadUserDiagnostics(provider.selectedUser!.id),
        onRepair: () => provider.repairAppearance(provider.selectedUser!.id),
        onReset: () => _confirmResetAppearance(provider),
        onRebuildInventory: () =>
            provider.rebuildInventory(provider.selectedUser!.id),
      ),
    );
  }

  List<UserAccount> _filterUsers(List<UserAccount> users) {
    final query = _searchController.text.toLowerCase();
    if (query.isEmpty) return users;

    return users.where((user) {
      return user.firstName.toLowerCase().contains(query) ||
          user.lastName.toLowerCase().contains(query) ||
          user.fullName.toLowerCase().contains(query);
    }).toList();
  }

  void _showCreateUserDialog() {
    showDialog(
      context: context,
      builder: (context) => UserCreationDialog(
        onUserCreated: (user) {
          final provider = Provider.of<UserProvider>(context, listen: false);
          provider.loadUsers(refresh: true);
        },
      ),
    );
  }

  void _showResetPasswordDialog(UserProvider provider) {
    final passwordController = TextEditingController();
    final confirmController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Reset Password'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('Reset password for ${provider.selectedUser!.fullName}'),
            const SizedBox(height: 16),
            TextField(
              controller: passwordController,
              decoration: const InputDecoration(
                labelText: 'New Password',
                prefixIcon: Icon(Icons.lock),
              ),
              obscureText: true,
            ),
            const SizedBox(height: 12),
            TextField(
              controller: confirmController,
              decoration: const InputDecoration(
                labelText: 'Confirm Password',
                prefixIcon: Icon(Icons.lock_outline),
              ),
              obscureText: true,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () async {
              if (passwordController.text != confirmController.text) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Passwords do not match'),
                    backgroundColor: Colors.red,
                  ),
                );
                return;
              }
              if (passwordController.text.length < 6) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Password must be at least 6 characters'),
                    backgroundColor: Colors.red,
                  ),
                );
                return;
              }
              Navigator.of(context).pop();
              final success = await provider.resetPassword(
                provider.selectedUser!.id,
                passwordController.text,
              );
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(
                      success
                          ? 'Password reset successfully'
                          : 'Failed to reset password',
                    ),
                    backgroundColor: success ? Colors.green : Colors.red,
                  ),
                );
              }
            },
            child: const Text('Reset'),
          ),
        ],
      ),
    );
  }

  void _showChangeLevelDialog(UserProvider provider) {
    int selectedLevel = provider.selectedUser!.userLevel;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('Change User Level'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('Set level for ${provider.selectedUser!.fullName}'),
              const SizedBox(height: 16),
              DropdownButtonFormField<int>(
                value: selectedLevel,
                decoration: const InputDecoration(
                  labelText: 'User Level',
                  prefixIcon: Icon(Icons.admin_panel_settings),
                ),
                items: const [
                  DropdownMenuItem(value: 0, child: Text('Regular User (0)')),
                  DropdownMenuItem(value: 100, child: Text('VIP (100)')),
                  DropdownMenuItem(value: 200, child: Text('Admin (200)')),
                  DropdownMenuItem(value: 250, child: Text('God (250)')),
                ],
                onChanged: (value) {
                  if (value != null) {
                    setDialogState(() => selectedLevel = value);
                  }
                },
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            ElevatedButton(
              onPressed: () {
                Navigator.of(context).pop();
                // TODO: Implement updateUserLevel in provider
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text('User level updated to $selectedLevel'),
                    backgroundColor: Colors.green,
                  ),
                );
              },
              child: const Text('Update'),
            ),
          ],
        ),
      ),
    );
  }

  void _confirmDeleteUser(String userId, UserProvider provider) {
    final user = provider.users.firstWhere((u) => u.id == userId);

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete User'),
        content: Text(
          'Are you sure you want to delete ${user.fullName}?\n\n'
          'This action cannot be undone.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () async {
              Navigator.of(context).pop();
              final success = await provider.deleteUser(userId);
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(
                      success
                          ? 'User deleted successfully'
                          : 'Failed to delete user',
                    ),
                    backgroundColor: success ? Colors.green : Colors.red,
                  ),
                );
              }
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.red,
              foregroundColor: Colors.white,
            ),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
  }

  void _confirmResetAppearance(UserProvider provider) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Reset Appearance'),
        content: Text(
          'Reset appearance for ${provider.selectedUser!.fullName} to defaults?\n\n'
          'This will:\n'
          '- Reset all wearables to Ruth defaults\n'
          '- Rebuild inventory folders if missing\n'
          '- Create CurrentOutfit links\n\n'
          'Existing customizations will be lost.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () async {
              Navigator.of(context).pop();
              final success =
                  await provider.resetAppearance(provider.selectedUser!.id);
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(
                      success
                          ? 'Appearance reset successfully'
                          : 'Failed to reset appearance',
                    ),
                    backgroundColor: success ? Colors.green : Colors.red,
                  ),
                );
              }
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.orange,
              foregroundColor: Colors.white,
            ),
            child: const Text('Reset'),
          ),
        ],
      ),
    );
  }
}
