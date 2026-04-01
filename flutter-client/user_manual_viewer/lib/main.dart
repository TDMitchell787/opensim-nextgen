import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown/flutter_markdown.dart';
import 'package:url_launcher/url_launcher.dart';

void main() {
  runApp(const UserManualApp());
}

class UserManualApp extends StatelessWidget {
  const UserManualApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'OpenSim Next User Manual',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorSchemeSeed: const Color(0xFF1565C0),
        brightness: Brightness.light,
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: const Color(0xFF42A5F5),
        brightness: Brightness.dark,
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const ManualViewer(),
    );
  }
}

class TocEntry {
  final String title;
  final int level;
  final int lineIndex;
  final List<TocEntry> children;

  TocEntry({
    required this.title,
    required this.level,
    required this.lineIndex,
    List<TocEntry>? children,
  }) : children = children ?? [];
}

class ManualViewer extends StatefulWidget {
  const ManualViewer({super.key});

  @override
  State<ManualViewer> createState() => _ManualViewerState();
}

class _ManualViewerState extends State<ManualViewer> {
  String _displayMarkdown = '';
  List<String> _lines = [];
  List<TocEntry> _toc = [];
  List<TocEntry> _filteredToc = [];
  String _searchQuery = '';
  bool _isLoading = true;
  final ScrollController _scrollController = ScrollController();
  final TextEditingController _searchController = TextEditingController();
  String _currentSection = '';
  bool _sidebarCollapsed = false;
  List<_SearchResult> _searchResults = [];
  int _currentSearchIndex = -1;
  Map<String, int> _slugToLineIndex = {};
  Map<String, String> _slugToTitle = {};
  bool _isLiveSource = false;

  @override
  void initState() {
    super.initState();
    _loadManual();
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _loadManual() async {
    String content;
    bool live = false;

    final masterFile = _findMasterManual();
    if (masterFile != null && masterFile.existsSync()) {
      content = masterFile.readAsStringSync();
      live = true;
    } else {
      content = await rootBundle.loadString('assets/USER_MANUAL.md');
    }

    final lines = content.split('\n');
    final toc = _buildToc(lines);
    final slugMaps = _buildSlugMap(lines);

    setState(() {
      _displayMarkdown = content;
      _lines = lines;
      _toc = toc;
      _filteredToc = toc;
      _slugToLineIndex = slugMaps.$1;
      _slugToTitle = slugMaps.$2;
      _isLiveSource = live;
      _isLoading = false;
      _currentSection = 'Full Manual';
    });
  }

  File? _findMasterManual() {
    final envPath = Platform.environment['OPENSIM_MANUAL_PATH'];
    if (envPath != null && envPath.isNotEmpty) {
      final f = File(envPath);
      if (f.existsSync()) return f;
    }

    final exe = Platform.resolvedExecutable;
    var dir = Directory(exe).parent;
    for (int i = 0; i < 10; i++) {
      final candidate = File('${dir.path}/USER_MANUAL.md');
      if (candidate.existsSync()) return candidate;
      final nested = File('${dir.path}/opensim-next/USER_MANUAL.md');
      if (nested.existsSync()) return nested;
      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }
    return null;
  }

  static String _slugify(String heading) {
    return heading
        .toLowerCase()
        .replaceAll(RegExp(r'[^\w\s-]'), '')
        .replaceAll(RegExp(r'\s+'), '-')
        .replaceAll(RegExp(r'-+'), '-')
        .replaceAll(RegExp(r'^-|-$'), '');
  }

  (Map<String, int>, Map<String, String>) _buildSlugMap(List<String> lines) {
    final Map<String, int> slugToLine = {};
    final Map<String, String> slugToTitle = {};
    for (int i = 0; i < lines.length; i++) {
      final line = lines[i].trim();
      if (!line.startsWith('#')) continue;
      int level = 0;
      while (level < line.length && line[level] == '#') {
        level++;
      }
      if (level >= line.length) continue;
      final title = line.substring(level).trim();
      if (title.isEmpty) continue;
      final slug = _slugify(title);
      if (!slugToLine.containsKey(slug)) {
        slugToLine[slug] = i;
        slugToTitle[slug] = title;
      }
    }
    return (slugToLine, slugToTitle);
  }

  List<TocEntry> _buildToc(List<String> lines) {
    final List<TocEntry> roots = [];
    final List<TocEntry> stack = [];

    for (int i = 0; i < lines.length; i++) {
      final line = lines[i].trim();
      if (!line.startsWith('#')) continue;

      int level = 0;
      while (level < line.length && line[level] == '#') {
        level++;
      }
      if (level > 4 || level >= line.length) continue;

      final title = line.substring(level).trim();
      if (title.isEmpty) continue;

      final entry = TocEntry(title: title, level: level, lineIndex: i);

      if (level == 1) {
        roots.add(entry);
        stack.clear();
        stack.add(entry);
      } else {
        while (stack.isNotEmpty && stack.last.level >= level) {
          stack.removeLast();
        }
        if (stack.isNotEmpty) {
          stack.last.children.add(entry);
        } else {
          roots.add(entry);
        }
        stack.add(entry);
      }
    }
    return roots;
  }

  void _navigateToSection(int lineIndex, String title) {
    final sectionContent = _extractSection(lineIndex);
    setState(() {
      _currentSection = title;
      _displayMarkdown = sectionContent;
      _searchResults = [];
      _currentSearchIndex = -1;
    });
    if (_scrollController.hasClients) {
      _scrollController.animateTo(
        0,
        duration: const Duration(milliseconds: 300),
        curve: Curves.easeInOut,
      );
    }
  }

  String _extractSection(int startLine) {
    int endLine = _lines.length;
    final startLevel = _getHeadingLevel(_lines[startLine]);

    for (int i = startLine + 1; i < _lines.length; i++) {
      final line = _lines[i].trim();
      if (line.startsWith('#')) {
        final level = _getHeadingLevel(line);
        if (level <= startLevel) {
          endLine = i;
          break;
        }
      }
    }
    return _lines.sublist(startLine, endLine).join('\n');
  }

  int _getHeadingLevel(String line) {
    final trimmed = line.trim();
    int level = 0;
    while (level < trimmed.length && trimmed[level] == '#') {
      level++;
    }
    return level;
  }

  void _onSearchChanged(String query) {
    setState(() {
      _searchQuery = query;
      if (query.isEmpty) {
        _filteredToc = _toc;
        _searchResults = [];
        _currentSearchIndex = -1;
      } else {
        _filteredToc = _filterTocEntries(_toc, query.toLowerCase());
        _searchResults = _findInContent(query);
        _currentSearchIndex = _searchResults.isNotEmpty ? 0 : -1;
      }
    });
  }

  List<TocEntry> _filterTocEntries(List<TocEntry> entries, String query) {
    final List<TocEntry> result = [];
    for (final entry in entries) {
      final filteredChildren = _filterTocEntries(entry.children, query);
      if (entry.title.toLowerCase().contains(query) ||
          filteredChildren.isNotEmpty) {
        result.add(TocEntry(
          title: entry.title,
          level: entry.level,
          lineIndex: entry.lineIndex,
          children: filteredChildren,
        ));
      }
    }
    return result;
  }

  List<_SearchResult> _findInContent(String query) {
    final results = <_SearchResult>[];
    final lowerQuery = query.toLowerCase();
    for (int i = 0; i < _lines.length; i++) {
      if (_lines[i].toLowerCase().contains(lowerQuery)) {
        String context = _lines[i].trim();
        if (context.length > 120) {
          context = '${context.substring(0, 120)}...';
        }
        String section = '';
        for (int j = i; j >= 0; j--) {
          if (_lines[j].trim().startsWith('#')) {
            section = _lines[j].trim().replaceAll(RegExp(r'^#+\s*'), '');
            break;
          }
        }
        results.add(_SearchResult(lineIndex: i, context: context, section: section));
      }
    }
    return results;
  }

  void _showFullManual() {
    setState(() {
      _displayMarkdown = _lines.join('\n');
      _currentSection = 'Full Manual';
    });
    if (_scrollController.hasClients) {
      _scrollController.animateTo(0,
          duration: const Duration(milliseconds: 300), curve: Curves.easeInOut);
    }
  }

  void _navigateToSearchResult(_SearchResult result) {
    for (int j = result.lineIndex; j >= 0; j--) {
      if (_lines[j].trim().startsWith('#')) {
        _navigateToSection(j, result.section);
        return;
      }
    }
    _navigateToSection(result.lineIndex, result.section);
  }

  void _handleAnchorLink(String slug) {
    if (_slugToLineIndex.containsKey(slug)) {
      final lineIndex = _slugToLineIndex[slug]!;
      final title = _slugToTitle[slug] ?? slug;
      _showFullManual();
      Future.delayed(const Duration(milliseconds: 100), () {
        _navigateToSection(lineIndex, title);
      });
      return;
    }
    final normalizedSlug = slug.replaceAll(RegExp(r'-+'), '-');
    for (final entry in _slugToLineIndex.entries) {
      if (entry.key.startsWith(normalizedSlug) ||
          normalizedSlug.startsWith(entry.key)) {
        final title = _slugToTitle[entry.key] ?? entry.key;
        _showFullManual();
        Future.delayed(const Duration(milliseconds: 100), () {
          _navigateToSection(entry.value, title);
        });
        return;
      }
    }
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Section not found: $slug'),
          duration: const Duration(seconds: 3),
        ),
      );
    }
  }

  Future<void> _reloadManual() async {
    setState(() => _isLoading = true);
    await _loadManual();
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }

    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Scaffold(
      body: Row(
        children: [
          if (!_sidebarCollapsed)
            SizedBox(
              width: 320,
              child: _buildSidebar(theme, isDark),
            ),
          if (!_sidebarCollapsed)
            VerticalDivider(width: 1, color: theme.dividerColor),
          Expanded(child: _buildContentArea(theme, isDark)),
        ],
      ),
    );
  }

  Widget _buildSidebar(ThemeData theme, bool isDark) {
    return Container(
      color: isDark ? const Color(0xFF1E1E2E) : const Color(0xFFF5F7FA),
      child: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: isDark ? const Color(0xFF252540) : const Color(0xFF1565C0),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Icon(Icons.menu_book, color: Colors.white, size: 28),
                    const SizedBox(width: 10),
                    Expanded(
                      child: Text('OpenSim Next',
                          style: TextStyle(
                              color: Colors.white,
                              fontSize: 18,
                              fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
                const SizedBox(height: 4),
                Text('User Manual v3.0.0',
                    style: TextStyle(
                        color: Colors.white.withValues(alpha: 0.8),
                        fontSize: 12)),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: TextField(
              controller: _searchController,
              onChanged: _onSearchChanged,
              decoration: InputDecoration(
                hintText: 'Search manual...',
                prefixIcon: const Icon(Icons.search, size: 20),
                suffixIcon: _searchQuery.isNotEmpty
                    ? IconButton(
                        icon: const Icon(Icons.clear, size: 18),
                        onPressed: () {
                          _searchController.clear();
                          _onSearchChanged('');
                        },
                      )
                    : null,
                isDense: true,
                contentPadding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(8)),
                filled: true,
                fillColor: isDark ? const Color(0xFF2A2A40) : Colors.white,
              ),
              style: const TextStyle(fontSize: 14),
            ),
          ),
          if (_searchResults.isNotEmpty) ...[
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              child: Row(
                children: [
                  Text('${_searchResults.length} results',
                      style: TextStyle(
                          fontSize: 12,
                          color: theme.colorScheme.primary,
                          fontWeight: FontWeight.w500)),
                  const Spacer(),
                  IconButton(
                    icon: const Icon(Icons.arrow_upward, size: 16),
                    onPressed: _currentSearchIndex > 0
                        ? () {
                            setState(() => _currentSearchIndex--);
                            _navigateToSearchResult(
                                _searchResults[_currentSearchIndex]);
                          }
                        : null,
                    constraints:
                        const BoxConstraints(minWidth: 28, minHeight: 28),
                  ),
                  Text('${_currentSearchIndex + 1}/${_searchResults.length}',
                      style: const TextStyle(fontSize: 11)),
                  IconButton(
                    icon: const Icon(Icons.arrow_downward, size: 16),
                    onPressed:
                        _currentSearchIndex < _searchResults.length - 1
                            ? () {
                                setState(() => _currentSearchIndex++);
                                _navigateToSearchResult(
                                    _searchResults[_currentSearchIndex]);
                              }
                            : null,
                    constraints:
                        const BoxConstraints(minWidth: 28, minHeight: 28),
                  ),
                ],
              ),
            ),
            SizedBox(
              height: 160,
              child: ListView.builder(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                itemCount: _searchResults.length,
                itemBuilder: (context, index) {
                  final result = _searchResults[index];
                  final isSelected = index == _currentSearchIndex;
                  return InkWell(
                    onTap: () {
                      setState(() => _currentSearchIndex = index);
                      _navigateToSearchResult(result);
                    },
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 6),
                      decoration: BoxDecoration(
                        color: isSelected
                            ? theme.colorScheme.primaryContainer
                            : null,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(result.section,
                              style: TextStyle(
                                  fontSize: 10,
                                  color: theme.colorScheme.primary,
                                  fontWeight: FontWeight.w600),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis),
                          Text(result.context,
                              style: const TextStyle(fontSize: 11),
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis),
                        ],
                      ),
                    ),
                  );
                },
              ),
            ),
            const Divider(height: 1),
          ],
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            child: InkWell(
              onTap: _showFullManual,
              borderRadius: BorderRadius.circular(6),
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                decoration: BoxDecoration(
                  color: _currentSection == 'Full Manual'
                      ? theme.colorScheme.primaryContainer
                      : null,
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Row(
                  children: [
                    Icon(Icons.article,
                        size: 16, color: theme.colorScheme.primary),
                    const SizedBox(width: 8),
                    const Text('Full Manual',
                        style: TextStyle(
                            fontSize: 13, fontWeight: FontWeight.w600)),
                  ],
                ),
              ),
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(vertical: 4),
              children: _filteredToc
                  .map((entry) => _buildTocItem(entry, theme, isDark))
                  .toList(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTocItem(TocEntry entry, ThemeData theme, bool isDark) {
    final isSelected = entry.title == _currentSection;
    final indent = (entry.level - 1) * 16.0;
    final isChapter = entry.level == 1;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => _navigateToSection(entry.lineIndex, entry.title),
          child: Container(
            padding: EdgeInsets.only(
              left: 12 + indent,
              right: 8,
              top: isChapter ? 8 : 4,
              bottom: isChapter ? 8 : 4,
            ),
            decoration: BoxDecoration(
              color: isSelected ? theme.colorScheme.primaryContainer : null,
              border: isSelected
                  ? Border(
                      left: BorderSide(
                          color: theme.colorScheme.primary, width: 3))
                  : null,
            ),
            child: Row(
              children: [
                if (entry.children.isNotEmpty)
                  Icon(Icons.chevron_right,
                      size: 14,
                      color: isDark ? Colors.white38 : Colors.black38),
                if (entry.children.isNotEmpty) const SizedBox(width: 4),
                Expanded(
                  child: Text(
                    entry.title,
                    style: TextStyle(
                      fontSize: isChapter ? 13 : 12,
                      fontWeight:
                          isChapter ? FontWeight.w600 : FontWeight.normal,
                      color: isSelected
                          ? theme.colorScheme.primary
                          : (isDark ? Colors.white70 : Colors.black87),
                    ),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        ),
        ...entry.children.map((child) => _buildTocItem(child, theme, isDark)),
      ],
    );
  }

  Widget _buildContentArea(ThemeData theme, bool isDark) {
    return Column(
      children: [
        Container(
          height: 48,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          decoration: BoxDecoration(
            color: isDark ? const Color(0xFF1E1E2E) : Colors.white,
            border: Border(bottom: BorderSide(color: theme.dividerColor)),
          ),
          child: Row(
            children: [
              IconButton(
                icon: Icon(
                    _sidebarCollapsed ? Icons.menu : Icons.menu_open,
                    size: 20),
                onPressed: () =>
                    setState(() => _sidebarCollapsed = !_sidebarCollapsed),
                tooltip:
                    _sidebarCollapsed ? 'Show sidebar' : 'Hide sidebar',
              ),
              const SizedBox(width: 8),
              Icon(Icons.description_outlined,
                  size: 18, color: theme.colorScheme.primary),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  _currentSection,
                  style: const TextStyle(
                      fontSize: 15, fontWeight: FontWeight.w500),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: _isLiveSource
                      ? (isDark ? const Color(0xFF1B5E20) : const Color(0xFFE8F5E9))
                      : (isDark ? const Color(0xFF4E342E) : const Color(0xFFFFF3E0)),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  _isLiveSource ? 'Live' : 'Bundled',
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: _isLiveSource
                        ? (isDark ? const Color(0xFF81C784) : const Color(0xFF2E7D32))
                        : (isDark ? const Color(0xFFFFCC80) : const Color(0xFFE65100)),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Text('${_lines.length} lines',
                  style: TextStyle(
                      fontSize: 12,
                      color: isDark ? Colors.white38 : Colors.black38)),
              const SizedBox(width: 4),
              IconButton(
                icon: const Icon(Icons.refresh, size: 18),
                onPressed: _reloadManual,
                tooltip: 'Reload manual',
              ),
              IconButton(
                icon: const Icon(Icons.keyboard_arrow_up, size: 20),
                onPressed: () => _scrollController.animateTo(0,
                    duration: const Duration(milliseconds: 400),
                    curve: Curves.easeOut),
                tooltip: 'Scroll to top',
              ),
            ],
          ),
        ),
        Expanded(
          child: Container(
            color: isDark ? const Color(0xFF16161E) : const Color(0xFFFAFAFC),
            child: Scrollbar(
              controller: _scrollController,
              thumbVisibility: true,
              child: SingleChildScrollView(
                controller: _scrollController,
                padding: const EdgeInsets.all(32),
                child: Center(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(maxWidth: 900),
                    child: MarkdownBody(
                      data: _displayMarkdown,
                      selectable: true,
                      onTapLink: (text, href, title) {
                        if (href == null) return;
                        if (href.startsWith('#')) {
                          _handleAnchorLink(href.substring(1));
                        } else {
                          launchUrl(Uri.parse(href));
                        }
                      },
                      styleSheet: MarkdownStyleSheet(
                        h1: TextStyle(
                            fontSize: 28,
                            fontWeight: FontWeight.bold,
                            color: theme.colorScheme.primary,
                            height: 1.4),
                        h2: TextStyle(
                            fontSize: 22,
                            fontWeight: FontWeight.bold,
                            color: isDark ? Colors.white : Colors.black87,
                            height: 1.4),
                        h3: TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w600,
                            color: isDark ? Colors.white70 : Colors.black87,
                            height: 1.4),
                        h4: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                            color: isDark ? Colors.white60 : Colors.black87,
                            height: 1.4),
                        p: TextStyle(
                            fontSize: 15,
                            height: 1.7,
                            color: isDark ? Colors.white70 : Colors.black87),
                        listBullet: TextStyle(
                            fontSize: 15,
                            color: isDark ? Colors.white70 : Colors.black87),
                        code: TextStyle(
                            fontSize: 13,
                            fontFamily: 'Menlo',
                            backgroundColor: isDark
                                ? const Color(0xFF2A2A40)
                                : const Color(0xFFF0F0F5),
                            color: isDark
                                ? const Color(0xFFE0E0FF)
                                : const Color(0xFF333355)),
                        codeblockDecoration: BoxDecoration(
                          color: isDark
                              ? const Color(0xFF1A1A2E)
                              : const Color(0xFFF5F5FA),
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                              color: isDark
                                  ? const Color(0xFF333355)
                                  : const Color(0xFFDDDDE5)),
                        ),
                        codeblockPadding: const EdgeInsets.all(16),
                        blockquoteDecoration: BoxDecoration(
                          border: Border(
                              left: BorderSide(
                                  color: theme.colorScheme.primary, width: 4)),
                          color: isDark
                              ? const Color(0xFF1E1E30)
                              : const Color(0xFFF0F4FF),
                        ),
                        blockquotePadding: const EdgeInsets.all(12),
                        tableBorder: TableBorder.all(
                            color: isDark
                                ? const Color(0xFF333355)
                                : const Color(0xFFDDDDE5),
                            width: 1),
                        tableHead: TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                            color: isDark ? Colors.white : Colors.black87),
                        tableBody: TextStyle(
                            fontSize: 14,
                            color: isDark ? Colors.white70 : Colors.black87),
                        tableCellsPadding: const EdgeInsets.all(8),
                        horizontalRuleDecoration: BoxDecoration(
                          border: Border(
                              top: BorderSide(
                                  color: isDark
                                      ? const Color(0xFF333355)
                                      : const Color(0xFFDDDDE5),
                                  width: 1)),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _SearchResult {
  final int lineIndex;
  final String context;
  final String section;

  _SearchResult({
    required this.lineIndex,
    required this.context,
    required this.section,
  });
}
