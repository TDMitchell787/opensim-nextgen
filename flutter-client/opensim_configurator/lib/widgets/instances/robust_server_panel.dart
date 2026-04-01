import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

class RobustServiceStatus {
  final String name;
  final bool healthy;
  final String endpoint;
  final int? responseTimeMs;

  RobustServiceStatus({
    required this.name,
    required this.healthy,
    required this.endpoint,
    this.responseTimeMs,
  });
}

class RobustServerPanel extends StatefulWidget {
  final String robustUrl;
  final String instanceName;

  const RobustServerPanel({
    super.key,
    required this.robustUrl,
    required this.instanceName,
  });

  @override
  State<RobustServerPanel> createState() => _RobustServerPanelState();
}

class _RobustServerPanelState extends State<RobustServerPanel> {
  bool _isChecking = false;
  bool _robustRunning = false;
  List<RobustServiceStatus> _services = [];
  Timer? _refreshTimer;

  final List<String> _serviceEndpoints = [
    'grid',
    'auth',
    'accounts',
    'asset',
    'inventory',
    'presence',
    'avatar',
    'friends',
  ];

  @override
  void initState() {
    super.initState();
    _checkServices();
    _refreshTimer = Timer.periodic(
      const Duration(seconds: 30),
      (_) => _checkServices(),
    );
  }

  @override
  void dispose() {
    _refreshTimer?.cancel();
    super.dispose();
  }

  Future<void> _checkServices() async {
    if (_isChecking) return;
    setState(() => _isChecking = true);

    final results = <RobustServiceStatus>[];
    bool anyHealthy = false;

    for (final endpoint in _serviceEndpoints) {
      final url = '${widget.robustUrl}/$endpoint';
      try {
        final sw = Stopwatch()..start();
        final response = await http.get(Uri.parse(url)).timeout(
          const Duration(seconds: 5),
        );
        sw.stop();
        final healthy = response.statusCode == 200 || response.statusCode == 400;
        if (healthy) anyHealthy = true;
        results.add(RobustServiceStatus(
          name: endpoint.substring(0, 1).toUpperCase() + endpoint.substring(1),
          healthy: healthy,
          endpoint: url,
          responseTimeMs: sw.elapsedMilliseconds,
        ));
      } catch (_) {
        results.add(RobustServiceStatus(
          name: endpoint.substring(0, 1).toUpperCase() + endpoint.substring(1),
          healthy: false,
          endpoint: url,
        ));
      }
    }

    if (mounted) {
      setState(() {
        _services = results;
        _robustRunning = anyHealthy;
        _isChecking = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final healthyCount = _services.where((s) => s.healthy).length;
    final totalCount = _services.length;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  _robustRunning ? Icons.cloud_done : Icons.cloud_off,
                  color: _robustRunning ? Colors.green : Colors.red,
                  size: 28,
                ),
                const SizedBox(width: 12),
                Text(
                  'Robust Server',
                  style: theme.textTheme.titleLarge,
                ),
                const Spacer(),
                if (_isChecking)
                  const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                const SizedBox(width: 8),
                Text(
                  _robustRunning ? 'Running' : 'Stopped',
                  style: TextStyle(
                    color: _robustRunning ? Colors.green : Colors.red,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              '${widget.instanceName} - ${widget.robustUrl}',
              style: theme.textTheme.bodySmall,
            ),
            if (totalCount > 0) ...[
              const SizedBox(height: 4),
              LinearProgressIndicator(
                value: totalCount > 0 ? healthyCount / totalCount : 0,
                backgroundColor: Colors.red.withOpacity(0.2),
                valueColor: AlwaysStoppedAnimation<Color>(Colors.green),
              ),
              const SizedBox(height: 4),
              Text(
                '$healthyCount / $totalCount services healthy',
                style: theme.textTheme.bodySmall,
              ),
            ],
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 4,
              children: _services.map((svc) {
                return Chip(
                  avatar: Icon(
                    svc.healthy ? Icons.check_circle : Icons.cancel,
                    size: 16,
                    color: svc.healthy ? Colors.green : Colors.red,
                  ),
                  label: Text(
                    svc.responseTimeMs != null
                        ? '${svc.name} (${svc.responseTimeMs}ms)'
                        : svc.name,
                    style: const TextStyle(fontSize: 12),
                  ),
                  backgroundColor: svc.healthy
                      ? Colors.green.withOpacity(0.1)
                      : Colors.red.withOpacity(0.1),
                );
              }).toList(),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton.icon(
                  onPressed: _checkServices,
                  icon: const Icon(Icons.refresh, size: 16),
                  label: const Text('Refresh'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
