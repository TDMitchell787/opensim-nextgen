import 'package:flutter/material.dart';

class HelpSystem extends StatefulWidget {
  final ScrollController? scrollController;

  const HelpSystem({this.scrollController});

  @override
  State<HelpSystem> createState() => _HelpSystemState();
}

class _HelpSystemState extends State<HelpSystem> {
  String _searchQuery = '';
  final _searchController = TextEditingController();

  static const _helpSections = <_HelpSection>[
    _HelpSection(
      icon: Icons.play_arrow,
      title: 'Getting Started',
      items: [
        _HelpItem(
          title: 'Prerequisites',
          content: 'Before configuring OpenSim Next, ensure you have:\n'
              '- Rust 1.70+ and Cargo installed\n'
              '- Zig 0.14+ for physics engine\n'
              '- At least 4GB RAM (8GB recommended)\n'
              '- 2+ CPU cores\n'
              '- Network access on ports 9000-9200',
        ),
        _HelpItem(
          title: 'First-Time Setup',
          content: '1. Select a Deployment Type (Development recommended for beginners)\n'
              '2. Configure network settings (defaults work for local testing)\n'
              '3. Choose a database (SQLite for development, PostgreSQL for production)\n'
              '4. Review security settings\n'
              '5. Set performance limits appropriate for your hardware\n'
              '6. Run validation to check your configuration\n'
              '7. Apply configuration and start the server',
        ),
        _HelpItem(
          title: 'Creating Your First Region',
          content: 'After the server starts:\n'
              '1. Open the Web Admin panel (port 9200)\n'
              '2. Create a user account\n'
              '3. The default region is automatically created\n'
              '4. Connect with a Second Life compatible viewer\n'
              '5. Login with: firstname lastname / password\n'
              '6. Grid URI: localhost:9000',
        ),
      ],
    ),
    _HelpSection(
      icon: Icons.category,
      title: 'Deployment Types',
      items: [
        _HelpItem(
          title: 'Development',
          content: 'Best for: Learning, testing, and local development\n\n'
              'Specs: 1-10 concurrent users, 1-4 regions\n'
              'Hardware: 4 cores, 8GB RAM minimum\n'
              'Database: SQLite (file-based, zero setup)\n'
              'Physics: ODE (stable, well-tested)\n'
              'Network: localhost only, no SSL required\n\n'
              'This mode optimizes for ease of setup and fast iteration.',
        ),
        _HelpItem(
          title: 'Production',
          content: 'Best for: Live servers with real users\n\n'
              'Specs: 10-500 concurrent users, 4-32 regions\n'
              'Hardware: 16 cores, 32GB RAM recommended\n'
              'Database: PostgreSQL (performance, reliability)\n'
              'Physics: Bullet or UBODE (better physics simulation)\n'
              'Network: Public hostname, SSL recommended\n\n'
              'This mode enables all security features and performance optimizations.',
        ),
        _HelpItem(
          title: 'Grid',
          content: 'Best for: Large-scale multi-region deployments\n\n'
              'Specs: 100-10,000+ concurrent users, 32-1000+ regions\n'
              'Hardware: 64+ cores, 128GB+ RAM\n'
              'Database: PostgreSQL with clustering\n'
              'Physics: POS with GPU acceleration\n'
              'Network: Load balancing, distributed architecture\n\n'
              'Grid mode supports hypergrid connectivity and multi-server federation.',
        ),
      ],
    ),
    _HelpSection(
      icon: Icons.dns,
      title: 'Network Setup',
      items: [
        _HelpItem(
          title: 'Port Configuration',
          content: 'OpenSim Next uses multiple ports:\n\n'
              '- 9000: Main LLUDP server (viewer connections)\n'
              '- 8080: Web client interface\n'
              '- 9100: Prometheus monitoring metrics\n'
              '- 9200: Admin API (REST endpoints)\n\n'
              'Ensure these ports are not blocked by your firewall.\n'
              'For remote access, forward port 9000 (UDP+TCP) on your router.',
        ),
        _HelpItem(
          title: 'External Hostname',
          content: 'The external hostname is what viewers use to connect:\n\n'
              '- Local testing: Use "localhost" or "127.0.0.1"\n'
              '- LAN access: Use your local IP (e.g., 192.168.1.x)\n'
              '- Internet access: Use your public IP or domain name\n\n'
              'Important: The hostname must be resolvable by connecting viewers.',
        ),
        _HelpItem(
          title: 'SSL/TLS Configuration',
          content: 'For production deployments, enable HTTPS:\n\n'
              '1. Obtain an SSL certificate (Let\'s Encrypt is free)\n'
              '2. Set the certificate path (.pem file)\n'
              '3. Set the private key path\n'
              '4. Enable HTTPS in network settings\n'
              '5. Configure HTTPS port (default: 9001)\n\n'
              'SSL encrypts web interface and API communications.',
        ),
      ],
    ),
    _HelpSection(
      icon: Icons.storage,
      title: 'Database',
      items: [
        _HelpItem(
          title: 'SQLite',
          content: 'Zero-configuration embedded database.\n\n'
              'Pros:\n'
              '- No server installation needed\n'
              '- Single file storage (easy backup)\n'
              '- Fast for small deployments\n\n'
              'Cons:\n'
              '- Not suitable for concurrent writes\n'
              '- Limited to single-server deployments\n'
              '- Performance degrades with large datasets\n\n'
              'Connection: ./opensim.db (relative path)',
        ),
        _HelpItem(
          title: 'PostgreSQL',
          content: 'High-performance production database.\n\n'
              'Pros:\n'
              '- Excellent concurrent performance\n'
              '- Advanced features (JSONB, full-text search)\n'
              '- Supports clustering and replication\n\n'
              'Connection format:\n'
              'postgresql://user:password@host:5432/dbname\n\n'
              'Recommended for 10+ concurrent users.',
        ),
        _HelpItem(
          title: 'MySQL / MariaDB',
          content: 'Enterprise-compatible database.\n\n'
              'Pros:\n'
              '- Wide hosting support\n'
              '- Compatible with existing OpenSim deployments\n'
              '- Good replication options\n\n'
              'Connection format:\n'
              'mysql://user:password@host:3306/dbname\n\n'
              'Use when migrating from existing OpenSim installations.',
        ),
      ],
    ),
    _HelpSection(
      icon: Icons.security,
      title: 'Security',
      items: [
        _HelpItem(
          title: 'Password Policies',
          content: 'When Password Complexity is enabled:\n\n'
              '- Minimum 8 characters\n'
              '- Must contain uppercase and lowercase\n'
              '- Must contain at least one number\n'
              '- Must contain at least one special character\n\n'
              'Brute Force Protection:\n'
              '- Locks account after 5 failed attempts\n'
              '- 15-minute lockout period\n'
              '- IP-based rate limiting',
        ),
        _HelpItem(
          title: 'Session Management',
          content: 'Session timeout controls how long a user stays logged in:\n\n'
              '- Development: 24 hours (convenience)\n'
              '- Production: 1-4 hours (security)\n'
              '- Grid: 30-60 minutes (high security)\n\n'
              'Expired sessions require re-authentication.\n'
              'Active viewer connections are not affected by web session timeout.',
        ),
        _HelpItem(
          title: 'SSL Certificates',
          content: 'SSL certificates encrypt communications:\n\n'
              'Free options:\n'
              '- Let\'s Encrypt (automated renewal)\n'
              '- Self-signed (for development/testing)\n\n'
              'Certificate files needed:\n'
              '- Certificate: .pem or .crt file\n'
              '- Private key: .key or .pem file\n\n'
              'Note: Viewer LLUDP connections use their own encryption.',
        ),
      ],
    ),
    _HelpSection(
      icon: Icons.speed,
      title: 'Performance',
      items: [
        _HelpItem(
          title: 'Prim Limits',
          content: 'Prims are the building blocks of virtual world objects:\n\n'
              '- Development: 15,000 per region\n'
              '- Production: 45,000 per region\n'
              '- Grid: 100,000+ per region\n\n'
              'Higher limits require more RAM and CPU.\n'
              'Each prim uses approximately 1KB of memory.',
        ),
        _HelpItem(
          title: 'Script Optimization',
          content: 'Scripts run LSL/OSSL code in the virtual world:\n\n'
              'Max Scripts per region:\n'
              '- Development: 5,000\n'
              '- Production: 20,000\n'
              '- Grid: 100,000+\n\n'
              'Script Timeout: Maximum time per event handler.\n'
              '- Default: 30 seconds\n'
              '- Increase for complex operations\n'
              '- Lower for better responsiveness',
        ),
        _HelpItem(
          title: 'Asset Caching',
          content: 'Caching stores frequently accessed assets locally:\n\n'
              'Benefits:\n'
              '- Faster texture loading\n'
              '- Reduced database queries\n'
              '- Lower network usage\n\n'
              'Cache Timeout:\n'
              '- Development: 1-4 hours\n'
              '- Production: 24-72 hours\n'
              '- Grid: 72-168 hours\n\n'
              'Cache uses disk space proportional to unique assets.',
        ),
      ],
    ),
  ];

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  List<_HelpSection> get _filteredSections {
    if (_searchQuery.isEmpty) return _helpSections;
    final query = _searchQuery.toLowerCase();
    return _helpSections
        .map((section) {
          final matchingItems = section.items
              .where((item) =>
                  item.title.toLowerCase().contains(query) ||
                  item.content.toLowerCase().contains(query))
              .toList();
          if (matchingItems.isEmpty && !section.title.toLowerCase().contains(query)) {
            return null;
          }
          return _HelpSection(
            icon: section.icon,
            title: section.title,
            items: matchingItems.isEmpty ? section.items : matchingItems,
          );
        })
        .whereType<_HelpSection>()
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    final sections = _filteredSections;

    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      child: Column(
        children: [
          Center(
            child: Container(
              width: 40, height: 4,
              margin: EdgeInsets.only(top: 12, bottom: 8),
              decoration: BoxDecoration(
                color: Colors.grey[400],
                borderRadius: BorderRadius.circular(2),
              ),
            ),
          ),
          Padding(
            padding: EdgeInsets.symmetric(horizontal: 24, vertical: 8),
            child: Row(
              children: [
                Icon(Icons.help_outline, size: 24),
                SizedBox(width: 12),
                Text('Help & Documentation', style: Theme.of(context).textTheme.titleLarge),
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.symmetric(horizontal: 24, vertical: 8),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: 'Search help topics...',
                prefixIcon: Icon(Icons.search, size: 20),
                isDense: true,
                border: OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
                contentPadding: EdgeInsets.symmetric(vertical: 8),
                suffixIcon: _searchQuery.isNotEmpty
                    ? IconButton(
                        icon: Icon(Icons.clear, size: 18),
                        onPressed: () {
                          _searchController.clear();
                          setState(() => _searchQuery = '');
                        },
                      )
                    : null,
              ),
              onChanged: (v) => setState(() => _searchQuery = v),
            ),
          ),
          Expanded(
            child: ListView.builder(
              controller: widget.scrollController,
              padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              itemCount: sections.length,
              itemBuilder: (context, index) => _buildSection(context, sections[index]),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSection(BuildContext context, _HelpSection section) {
    return Card(
      margin: EdgeInsets.only(bottom: 8),
      child: ExpansionTile(
        leading: Icon(section.icon, color: Theme.of(context).colorScheme.primary),
        title: Text(section.title, style: TextStyle(fontWeight: FontWeight.w600)),
        children: section.items.map((item) => _buildHelpItem(context, item)).toList(),
      ),
    );
  }

  Widget _buildHelpItem(BuildContext context, _HelpItem item) {
    return ExpansionTile(
      tilePadding: EdgeInsets.only(left: 56, right: 16),
      title: Text(item.title, style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.w500)),
      children: [
        Padding(
          padding: EdgeInsets.fromLTRB(56, 0, 16, 16),
          child: Align(
            alignment: Alignment.topLeft,
            child: Text(
              item.content,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(height: 1.5),
            ),
          ),
        ),
      ],
    );
  }
}

class _HelpSection {
  final IconData icon;
  final String title;
  final List<_HelpItem> items;

  const _HelpSection({required this.icon, required this.title, required this.items});
}

class _HelpItem {
  final String title;
  final String content;

  const _HelpItem({required this.title, required this.content});
}
