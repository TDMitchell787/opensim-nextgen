import 'package:flutter/material.dart';
import '../services/skill_service.dart';

class SkillDashboardScreen extends StatefulWidget {
  const SkillDashboardScreen({super.key});

  @override
  State<SkillDashboardScreen> createState() => _SkillDashboardScreenState();
}

class _SkillDashboardScreenState extends State<SkillDashboardScreen> {
  Map<String, dynamic> _dashboard = {};
  Map<String, dynamic>? _selectedDomain;
  List<dynamic> _domainSkills = [];
  Map<String, dynamic>? _selectedSkill;
  String _searchQuery = '';
  List<dynamic> _searchResults = [];
  bool _isLoading = true;
  bool _isOffline = false;
  final _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _loadDashboard();
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _loadDashboard() async {
    setState(() => _isLoading = true);
    final data = await SkillService.instance.getDashboard();
    if (mounted) {
      setState(() {
        _dashboard = data;
        _isOffline = data['offline'] == true;
        _isLoading = false;
      });
    }
  }

  Future<void> _loadDomainSkills(String domainId) async {
    final data = await SkillService.instance.getDomainSkills(domainId);
    if (mounted) {
      setState(() {
        _selectedDomain = data;
        _domainSkills = List<dynamic>.from(data['skills'] ?? []);
        _selectedSkill = null;
      });
    }
  }

  Future<void> _loadSkillDetail(String domainId, String skillId) async {
    final data = await SkillService.instance.getSkillDetail(domainId, skillId);
    if (mounted) {
      setState(() => _selectedSkill = data);
    }
  }

  Future<void> _performSearch(String query) async {
    if (query.isEmpty) {
      setState(() {
        _searchResults = [];
        _searchQuery = '';
      });
      return;
    }
    final data = await SkillService.instance.searchSkills(query);
    if (mounted) {
      setState(() {
        _searchQuery = query;
        _searchResults = List<dynamic>.from(data['results'] ?? []);
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0F172A),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _isOffline
              ? _buildOfflineState()
              : Row(
                  children: [
                    SizedBox(width: 320, child: _buildDomainPanel()),
                    Expanded(child: _buildMainContent()),
                  ],
                ),
    );
  }

  Widget _buildOfflineState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.cloud_off, size: 64, color: Colors.grey[600]),
          const SizedBox(height: 16),
          Text('Server Offline',
              style: TextStyle(
                  color: Colors.grey[400],
                  fontSize: 20,
                  fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          Text('Start the server to view the Skill Dashboard',
              style: TextStyle(color: Colors.grey[600])),
          const SizedBox(height: 24),
          ElevatedButton.icon(
            onPressed: _loadDashboard,
            icon: const Icon(Icons.refresh),
            label: const Text('Retry'),
          ),
        ],
      ),
    );
  }

  Widget _buildDomainPanel() {
    final domains = List<dynamic>.from(_dashboard['domains'] ?? []);
    final totalSkills = _dashboard['total_skills'] ?? 0;
    final overallScore = _dashboard['overall_score'] ?? 0;

    return Container(
      color: const Color(0xFF1E293B),
      child: Column(
        children: [
          _buildScoreHeader(totalSkills, overallScore),
          _buildSearchBar(),
          Expanded(
            child: _searchQuery.isNotEmpty
                ? _buildSearchResults()
                : _buildDomainList(domains),
          ),
        ],
      ),
    );
  }

  Widget _buildScoreHeader(int totalSkills, int overallScore) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: const BoxDecoration(
        gradient: LinearGradient(
          colors: [Color(0xFF1E40AF), Color(0xFF7C3AED)],
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.auto_awesome, color: Colors.white, size: 28),
              const SizedBox(width: 12),
              const Text('Skill Engine',
                  style: TextStyle(
                      color: Colors.white,
                      fontSize: 20,
                      fontWeight: FontWeight.bold)),
            ],
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              _buildStatBadge('$totalSkills', 'Skills'),
              const SizedBox(width: 12),
              _buildStatBadge('${_dashboard['total_domains'] ?? 0}', 'Domains'),
              const SizedBox(width: 12),
              _buildStatBadge('$overallScore%', 'Maturity'),
            ],
          ),
          const SizedBox(height: 12),
          ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: LinearProgressIndicator(
              value: overallScore / 100.0,
              backgroundColor: Colors.white.withValues(alpha: 0.2),
              valueColor:
                  const AlwaysStoppedAnimation<Color>(Colors.greenAccent),
              minHeight: 6,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStatBadge(String value, String label) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        children: [
          Text(value,
              style: const TextStyle(
                  color: Colors.white,
                  fontSize: 16,
                  fontWeight: FontWeight.bold)),
          Text(label,
              style: TextStyle(
                  color: Colors.white.withValues(alpha: 0.7), fontSize: 11)),
        ],
      ),
    );
  }

  Widget _buildSearchBar() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: TextField(
        controller: _searchController,
        style: const TextStyle(color: Colors.white),
        decoration: InputDecoration(
          hintText: 'Search skills...',
          hintStyle: TextStyle(color: Colors.grey[500]),
          prefixIcon: Icon(Icons.search, color: Colors.grey[500]),
          suffixIcon: _searchQuery.isNotEmpty
              ? IconButton(
                  icon: Icon(Icons.clear, color: Colors.grey[500]),
                  onPressed: () {
                    _searchController.clear();
                    _performSearch('');
                  },
                )
              : null,
          filled: true,
          fillColor: const Color(0xFF0F172A),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
            borderSide: BorderSide.none,
          ),
          contentPadding: const EdgeInsets.symmetric(vertical: 10),
        ),
        onChanged: (v) => _performSearch(v),
      ),
    );
  }

  Widget _buildSearchResults() {
    if (_searchResults.isEmpty) {
      return Center(
          child: Text('No results for "$_searchQuery"',
              style: TextStyle(color: Colors.grey[500])));
    }
    return ListView.builder(
      itemCount: _searchResults.length,
      itemBuilder: (ctx, i) {
        final skill = _searchResults[i];
        return _buildSkillListTile(skill);
      },
    );
  }

  Widget _buildDomainList(List<dynamic> domains) {
    return ListView.builder(
      itemCount: domains.length,
      itemBuilder: (ctx, i) {
        final d = domains[i];
        final score = d['score'] ?? 0;
        final total = d['total_skills'] ?? 0;
        final isSelected = _selectedDomain?['domain'] == d['domain'];
        return _buildDomainTile(d, score, total, isSelected);
      },
    );
  }

  Widget _buildDomainTile(
      Map<String, dynamic> d, int score, int total, bool isSelected) {
    final color = _domainColor(d['domain'] ?? '');
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: isSelected
            ? color.withValues(alpha: 0.15)
            : Colors.transparent,
        borderRadius: BorderRadius.circular(8),
        border: isSelected
            ? Border.all(color: color.withValues(alpha: 0.4))
            : null,
      ),
      child: ListTile(
        dense: true,
        leading: Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.15),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(_domainIcon(d['domain'] ?? ''), color: color, size: 18),
        ),
        title: Text(d['display_name'] ?? d['domain'] ?? '',
            style: const TextStyle(
                color: Colors.white, fontWeight: FontWeight.w500)),
        subtitle: Row(
          children: [
            Text('$total skills',
                style: TextStyle(color: Colors.grey[500], fontSize: 12)),
            const SizedBox(width: 8),
            _buildMaturityBadge(score),
          ],
        ),
        trailing: Icon(Icons.chevron_right, color: Colors.grey[600], size: 18),
        onTap: () => _loadDomainSkills(d['domain']),
      ),
    );
  }

  Widget _buildMaturityBadge(int score) {
    final color = score >= 80
        ? Colors.green
        : score >= 50
            ? Colors.amber
            : score > 0
                ? Colors.orange
                : Colors.grey;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text('$score%',
          style: TextStyle(
              color: color, fontSize: 11, fontWeight: FontWeight.w600)),
    );
  }

  Widget _buildMainContent() {
    if (_selectedSkill != null && _selectedSkill!.isNotEmpty) {
      return _buildSkillDetailView();
    }
    if (_selectedDomain != null) {
      return _buildDomainDetailView();
    }
    return _buildOverviewView();
  }

  Widget _buildOverviewView() {
    final domains = List<dynamic>.from(_dashboard['domains'] ?? []);
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Maturity Dashboard',
              style: TextStyle(
                  color: Colors.white,
                  fontSize: 24,
                  fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          Text(
              '14 domains across the Skill Engine — select a domain to explore',
              style: TextStyle(color: Colors.grey[400])),
          const SizedBox(height: 24),
          _buildMaturityGrid(domains),
          const SizedBox(height: 32),
          _buildLevelLegend(),
        ],
      ),
    );
  }

  Widget _buildMaturityGrid(List<dynamic> domains) {
    return Wrap(
      spacing: 16,
      runSpacing: 16,
      children: domains.map<Widget>((d) => _buildDomainCard(d)).toList(),
    );
  }

  Widget _buildDomainCard(Map<String, dynamic> d) {
    final score = d['score'] ?? 0;
    final total = d['total_skills'] ?? 0;
    final byLevel = List<int>.from(d['by_level'] ?? [0, 0, 0, 0, 0, 0, 0, 0]);
    final color = _domainColor(d['domain'] ?? '');

    return GestureDetector(
      onTap: () => _loadDomainSkills(d['domain']),
      child: Container(
        width: 220,
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: const Color(0xFF1E293B),
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
              color: color.withValues(alpha: 0.3)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(_domainIcon(d['domain'] ?? ''), color: color, size: 20),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(d['display_name'] ?? '',
                      style: const TextStyle(
                          color: Colors.white,
                          fontWeight: FontWeight.w600,
                          fontSize: 14)),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('$total skills',
                    style: TextStyle(color: Colors.grey[400], fontSize: 12)),
                _buildMaturityBadge(score),
              ],
            ),
            const SizedBox(height: 8),
            ClipRRect(
              borderRadius: BorderRadius.circular(3),
              child: LinearProgressIndicator(
                value: score / 100.0,
                backgroundColor: Colors.grey[800],
                valueColor: AlwaysStoppedAnimation<Color>(color),
                minHeight: 4,
              ),
            ),
            const SizedBox(height: 10),
            _buildLevelBar(byLevel, total),
          ],
        ),
      ),
    );
  }

  Widget _buildLevelBar(List<int> byLevel, int total) {
    if (total == 0) return const SizedBox.shrink();
    final colors = [
      Colors.grey[700]!,
      Colors.grey[500]!,
      Colors.blue[800]!,
      Colors.blue[600]!,
      Colors.amber[700]!,
      Colors.orange[600]!,
      Colors.green[600]!,
      Colors.green[400]!,
    ];
    return SizedBox(
      height: 6,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(3),
        child: Row(
          children: List.generate(8, (i) {
            final w = byLevel[i] / total;
            if (w == 0) return const SizedBox.shrink();
            return Expanded(
              flex: (w * 100).round().clamp(1, 100),
              child: Container(color: colors[i]),
            );
          }),
        ),
      ),
    );
  }

  Widget _buildLevelLegend() {
    final levels = [
      ('L0', 'Seed', Colors.grey[700]!),
      ('L1', 'Defined', Colors.grey[500]!),
      ('L2', 'Stubbed', Colors.blue[800]!),
      ('L3', 'Functional', Colors.blue[600]!),
      ('L4', 'Robust', Colors.amber[700]!),
      ('L5', 'Integrated', Colors.orange[600]!),
      ('L6', 'Verified', Colors.green[600]!),
      ('L7', 'Production', Colors.green[400]!),
    ];
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: const Color(0xFF1E293B),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Maturity Levels',
              style: TextStyle(
                  color: Colors.white,
                  fontWeight: FontWeight.w600,
                  fontSize: 14)),
          const SizedBox(height: 12),
          Wrap(
            spacing: 16,
            runSpacing: 8,
            children: levels
                .map((l) => Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Container(
                            width: 12,
                            height: 12,
                            decoration: BoxDecoration(
                                color: l.$3,
                                borderRadius: BorderRadius.circular(3))),
                        const SizedBox(width: 6),
                        Text('${l.$1} ${l.$2}',
                            style: TextStyle(
                                color: Colors.grey[400], fontSize: 12)),
                      ],
                    ))
                .toList(),
          ),
        ],
      ),
    );
  }

  Widget _buildDomainDetailView() {
    final domain = _selectedDomain!;
    final color = _domainColor(domain['domain'] ?? '');
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildBreadcrumb(domain['display_name'] ?? ''),
          const SizedBox(height: 16),
          Row(
            children: [
              Icon(_domainIcon(domain['domain'] ?? ''), color: color, size: 28),
              const SizedBox(width: 12),
              Text(domain['display_name'] ?? '',
                  style: const TextStyle(
                      color: Colors.white,
                      fontSize: 22,
                      fontWeight: FontWeight.bold)),
              const SizedBox(width: 16),
              _buildMaturityBadge(domain['score'] ?? 0),
            ],
          ),
          const SizedBox(height: 20),
          ..._domainSkills.map<Widget>((s) => _buildSkillListTile(s)),
        ],
      ),
    );
  }

  Widget _buildSkillListTile(Map<String, dynamic> skill) {
    final maturity = skill['maturity'] ?? 0;
    final matLabel = skill['maturity_label'] ?? 'Seed';
    final color = _levelColor(maturity);
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      decoration: BoxDecoration(
        color: const Color(0xFF1E293B),
        borderRadius: BorderRadius.circular(8),
      ),
      child: ListTile(
        dense: true,
        leading: Container(
          width: 32,
          height: 32,
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.15),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Center(
              child: Text('L$maturity',
                  style: TextStyle(
                      color: color,
                      fontSize: 12,
                      fontWeight: FontWeight.bold))),
        ),
        title: Text(skill['display_name'] ?? skill['id'] ?? '',
            style: const TextStyle(color: Colors.white, fontSize: 14)),
        subtitle: Text(skill['description'] ?? '',
            style: TextStyle(color: Colors.grey[500], fontSize: 12),
            maxLines: 1,
            overflow: TextOverflow.ellipsis),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: color.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(matLabel,
                  style: TextStyle(color: color, fontSize: 11)),
            ),
            const SizedBox(width: 8),
            Icon(Icons.chevron_right, color: Colors.grey[600], size: 18),
          ],
        ),
        onTap: () =>
            _loadSkillDetail(skill['domain'] ?? '', skill['id'] ?? ''),
      ),
    );
  }

  Widget _buildSkillDetailView() {
    final s = _selectedSkill!;
    final maturity = s['maturity'] ?? 0;
    final color = _levelColor(maturity);
    final params = List<dynamic>.from(s['params'] ?? []);
    final tags = List<dynamic>.from(s['tags'] ?? []);
    final examples = List<dynamic>.from(s['examples'] ?? []);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildBreadcrumb(
              '${s['domain'] ?? ''} / ${s['display_name'] ?? ''}',
              onBack: () => setState(() => _selectedSkill = null)),
          const SizedBox(height: 16),
          Row(
            children: [
              Text(s['display_name'] ?? '',
                  style: const TextStyle(
                      color: Colors.white,
                      fontSize: 22,
                      fontWeight: FontWeight.bold)),
              const SizedBox(width: 12),
              Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: color.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Text('L$maturity ${s['maturity_label'] ?? ''}',
                    style: TextStyle(
                        color: color,
                        fontWeight: FontWeight.w600,
                        fontSize: 13)),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(s['description'] ?? '',
              style: TextStyle(color: Colors.grey[400], fontSize: 14)),
          const SizedBox(height: 6),
          Text('Phase: ${s['phase'] ?? 'N/A'}',
              style: TextStyle(color: Colors.grey[600], fontSize: 12)),
          const SizedBox(height: 16),
          if (tags.isNotEmpty)
            Wrap(
              spacing: 6,
              runSpacing: 6,
              children: tags
                  .map<Widget>((t) => Chip(
                        label: Text('$t',
                            style: const TextStyle(fontSize: 11)),
                        backgroundColor: const Color(0xFF334155),
                        labelStyle: TextStyle(color: Colors.grey[300]),
                        padding: EdgeInsets.zero,
                        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      ))
                  .toList(),
            ),
          const SizedBox(height: 24),
          if (params.isNotEmpty) ...[
            const Text('Parameters',
                style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            ...params.map<Widget>((p) => _buildParamRow(p)),
          ],
          if (examples.isNotEmpty) ...[
            const SizedBox(height: 24),
            const Text('Examples',
                style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            ...examples.map<Widget>((e) => _buildExampleCard(e)),
          ],
          const SizedBox(height: 16),
          Row(
            children: [
              _buildInfoChip(Icons.location_on, 'Region',
                  s['requires_region'] == true),
              const SizedBox(width: 8),
              _buildInfoChip(
                  Icons.person, 'Agent', s['requires_agent'] == true),
              const SizedBox(width: 8),
              _buildInfoChip(
                  Icons.admin_panel_settings, 'Admin',
                  s['requires_admin'] == true),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildParamRow(Map<String, dynamic> p) {
    final required = p['required'] == true;
    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF1E293B),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Row(
        children: [
          SizedBox(
            width: 140,
            child: Row(
              children: [
                Text(p['name'] ?? '',
                    style: TextStyle(
                        color: required ? Colors.white : Colors.grey[400],
                        fontWeight:
                            required ? FontWeight.w600 : FontWeight.normal,
                        fontSize: 13,
                        fontFamily: 'monospace')),
                if (required)
                  const Text(' *',
                      style: TextStyle(color: Colors.red, fontSize: 12)),
              ],
            ),
          ),
          SizedBox(
            width: 80,
            child: Text(p['param_type'] ?? '',
                style: TextStyle(
                    color: Colors.blue[300],
                    fontSize: 12,
                    fontFamily: 'monospace')),
          ),
          Expanded(
            child: Text(p['description'] ?? '',
                style: TextStyle(color: Colors.grey[500], fontSize: 12)),
          ),
          if (p['default_value'] != null)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.grey[800],
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text('=${p['default_value']}',
                  style: TextStyle(
                      color: Colors.grey[400],
                      fontSize: 11,
                      fontFamily: 'monospace')),
            ),
        ],
      ),
    );
  }

  Widget _buildExampleCard(Map<String, dynamic> e) {
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF1E293B),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(e['description'] ?? '',
              style: const TextStyle(
                  color: Colors.white, fontWeight: FontWeight.w500)),
          const SizedBox(height: 8),
          _buildCodeBlock('Input', e['input'] ?? ''),
          const SizedBox(height: 4),
          _buildCodeBlock('Output', e['output'] ?? ''),
        ],
      ),
    );
  }

  Widget _buildCodeBlock(String label, String code) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label,
            style: TextStyle(
                color: Colors.grey[500],
                fontSize: 11,
                fontWeight: FontWeight.w600)),
        const SizedBox(height: 2),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: const Color(0xFF0F172A),
            borderRadius: BorderRadius.circular(4),
          ),
          child: Text(code,
              style: TextStyle(
                  color: Colors.green[300],
                  fontSize: 12,
                  fontFamily: 'monospace')),
        ),
      ],
    );
  }

  Widget _buildInfoChip(IconData icon, String label, bool active) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: active
            ? Colors.blue.withValues(alpha: 0.1)
            : Colors.grey.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(
            color: active
                ? Colors.blue.withValues(alpha: 0.3)
                : Colors.grey.withValues(alpha: 0.2)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: active ? Colors.blue : Colors.grey[600]),
          const SizedBox(width: 4),
          Text(label,
              style: TextStyle(
                  color: active ? Colors.blue : Colors.grey[600],
                  fontSize: 12)),
        ],
      ),
    );
  }

  Widget _buildBreadcrumb(String path, {VoidCallback? onBack}) {
    return Row(
      children: [
        InkWell(
          onTap: () => setState(() {
            _selectedDomain = null;
            _selectedSkill = null;
          }),
          child: Text('Dashboard',
              style: TextStyle(color: Colors.blue[400], fontSize: 13)),
        ),
        if (path.isNotEmpty) ...[
          Icon(Icons.chevron_right, color: Colors.grey[600], size: 16),
          InkWell(
            onTap: onBack,
            child: Text(path,
                style: TextStyle(color: Colors.grey[400], fontSize: 13)),
          ),
        ],
      ],
    );
  }

  Color _levelColor(int level) {
    switch (level) {
      case 7:
        return Colors.green[400]!;
      case 6:
        return Colors.green[600]!;
      case 5:
        return Colors.orange[600]!;
      case 4:
        return Colors.amber[700]!;
      case 3:
        return Colors.blue[600]!;
      case 2:
        return Colors.blue[800]!;
      case 1:
        return Colors.grey[500]!;
      default:
        return Colors.grey[700]!;
    }
  }

  Color _domainColor(String domain) {
    switch (domain) {
      case 'building':
        return Colors.blue;
      case 'scripting':
        return Colors.purple;
      case 'landscaping':
        return Colors.green;
      case 'vehicles':
        return Colors.orange;
      case 'media':
        return Colors.red;
      case 'clothing':
        return Colors.pink;
      case 'navigation':
        return Colors.cyan;
      case 'estate':
        return Colors.indigo;
      case 'economy':
        return Colors.amber;
      case 'social':
        return Colors.teal;
      case 'animation':
        return Colors.deepPurple;
      case 'inventory':
        return Colors.brown;
      case 'npc_management':
        return Colors.deepOrange;
      case 'tutorial':
        return Colors.lightGreen;
      default:
        return Colors.grey;
    }
  }

  IconData _domainIcon(String domain) {
    switch (domain) {
      case 'building':
        return Icons.construction;
      case 'scripting':
        return Icons.code;
      case 'landscaping':
        return Icons.terrain;
      case 'vehicles':
        return Icons.directions_car;
      case 'media':
        return Icons.videocam;
      case 'clothing':
        return Icons.checkroom;
      case 'navigation':
        return Icons.explore;
      case 'estate':
        return Icons.admin_panel_settings;
      case 'economy':
        return Icons.account_balance;
      case 'social':
        return Icons.groups;
      case 'animation':
        return Icons.animation;
      case 'inventory':
        return Icons.inventory;
      case 'npc_management':
        return Icons.smart_toy;
      case 'tutorial':
        return Icons.school;
      default:
        return Icons.extension;
    }
  }
}
