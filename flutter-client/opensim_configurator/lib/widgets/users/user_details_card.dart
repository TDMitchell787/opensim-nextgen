import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../models/user_models.dart';

class UserDetailsCard extends StatelessWidget {
  final UserAccount user;
  final VoidCallback onResetPassword;
  final VoidCallback onChangeLevel;
  final VoidCallback onDelete;

  const UserDetailsCard({
    super.key,
    required this.user,
    required this.onResetPassword,
    required this.onChangeLevel,
    required this.onDelete,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildHeader(context),
        const SizedBox(height: 16),
        _buildInfoCard(context),
        const SizedBox(height: 16),
        _buildActionsCard(context),
      ],
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Row(
      children: [
        CircleAvatar(
          radius: 32,
          backgroundColor: _getUserColor(),
          child: Text(
            user.firstName.substring(0, 1).toUpperCase(),
            style: const TextStyle(
              color: Colors.white,
              fontWeight: FontWeight.bold,
              fontSize: 24,
            ),
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                user.fullName,
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              const SizedBox(height: 4),
              Row(
                children: [
                  _buildStatusBadge(),
                  const SizedBox(width: 8),
                  if (user.isAdmin) _buildRoleBadge(),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildInfoCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Account Information',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            _buildInfoRow(context, 'User ID', user.id, canCopy: true),
            _buildInfoRow(context, 'First Name', user.firstName),
            _buildInfoRow(context, 'Last Name', user.lastName),
            _buildInfoRow(context, 'Email', user.email ?? 'Not set'),
            _buildInfoRow(context, 'Created', user.createdFormatted),
            _buildInfoRow(context, 'User Level', '${user.userLevel}'),
            _buildInfoRow(context, 'User Flags', '${user.userFlags}'),
            if (user.userTitle != null && user.userTitle!.isNotEmpty)
              _buildInfoRow(context, 'Title', user.userTitle!),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(BuildContext context, String label, String value,
      {bool canCopy = false}) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 100,
            child: Text(
              label,
              style: TextStyle(
                color: Colors.grey[600],
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          Expanded(
            child: Row(
              children: [
                Expanded(
                  child: SelectableText(
                    value,
                    style: const TextStyle(fontWeight: FontWeight.w500),
                  ),
                ),
                if (canCopy)
                  IconButton(
                    icon: const Icon(Icons.copy, size: 16),
                    onPressed: () {
                      Clipboard.setData(ClipboardData(text: value));
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(
                          content: Text('Copied to clipboard'),
                          duration: Duration(seconds: 1),
                        ),
                      );
                    },
                    tooltip: 'Copy',
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActionsCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Actions',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            const SizedBox(height: 16),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                OutlinedButton.icon(
                  onPressed: onResetPassword,
                  icon: const Icon(Icons.lock_reset),
                  label: const Text('Reset Password'),
                ),
                OutlinedButton.icon(
                  onPressed: onChangeLevel,
                  icon: const Icon(Icons.admin_panel_settings),
                  label: const Text('Change Level'),
                ),
                OutlinedButton.icon(
                  onPressed: onDelete,
                  icon: const Icon(Icons.delete, color: Colors.red),
                  label: const Text('Delete User',
                      style: TextStyle(color: Colors.red)),
                  style: OutlinedButton.styleFrom(
                    side: const BorderSide(color: Colors.red),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Color _getUserColor() {
    if (user.isGod) return Colors.purple;
    if (user.isAdmin) return Colors.indigo;
    switch (user.status) {
      case UserStatus.active:
        return Colors.green;
      case UserStatus.inactive:
        return Colors.grey;
      case UserStatus.suspended:
        return Colors.red;
      case UserStatus.pending:
        return Colors.orange;
    }
  }

  Widget _buildStatusBadge() {
    Color color;
    String label;

    switch (user.status) {
      case UserStatus.active:
        color = Colors.green;
        label = 'Active';
        break;
      case UserStatus.inactive:
        color = Colors.grey;
        label = 'Inactive';
        break;
      case UserStatus.suspended:
        color = Colors.red;
        label = 'Suspended';
        break;
      case UserStatus.pending:
        color = Colors.orange;
        label = 'Pending';
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }

  Widget _buildRoleBadge() {
    final color = Colors.purple;
    final label = user.isGod ? 'God' : 'Admin';

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
