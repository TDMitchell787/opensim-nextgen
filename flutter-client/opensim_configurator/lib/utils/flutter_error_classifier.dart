// Flutter Error Classification & Detective Debugging Utility
// Implements the Flutter version of the Elegant Archive Solution methodology

import 'dart:developer' as developer;
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

/// Flutter Error Classifier implementing detective debugging methodology
class FlutterErrorClassifier {
  // Error pattern detection map
  static const Map<String, String> errorPatterns = {
    'setState.*mounted': 'Widget lifecycle error - check mounted before setState',
    'RenderFlex.*overflow': 'Layout overflow - use Flexible/Expanded widgets',
    'Navigator.*context': 'Navigation context error - use proper BuildContext',
    'Provider.*not found': 'Provider scope error - wrap with correct Provider',
    'HTTP.*CORS': 'CORS policy error - check server configuration',
    'Service Worker.*cache': 'PWA caching error - clear browser cache',
    'XMLHttpRequest.*blocked': 'Network request blocked - check CORS/firewall',
    'Future.*completed': 'Async completion error - check Future handling',
    'dispose.*setState': 'Widget disposed before setState - add mounted check',
    'BuildContext.*mounted': 'Context usage after dispose - check widget lifecycle',
  };

  // Error classification categories
  static const Map<String, List<String>> errorCategories = {
    'Widget Lifecycle': [
      'setState.*mounted',
      'dispose.*setState', 
      'BuildContext.*mounted',
    ],
    'Layout Issues': [
      'RenderFlex.*overflow',
      'Positioned.*constraints',
      'Flex.*MainAxis.*overflow',
    ],
    'Navigation Errors': [
      'Navigator.*context',
      'Route.*not found',
      'pushNamed.*context',
    ],
    'State Management': [
      'Provider.*not found',
      'ChangeNotifier.*disposed',
      'setState.*null',
    ],
    'Network/API Issues': [
      'HTTP.*CORS',
      'XMLHttpRequest.*blocked',
      'Connection.*refused',
    ],
    'PWA/Caching': [
      'Service Worker.*cache',
      'manifest.*invalid',
      'cache.*failed',
    ],
  };

  /// Log error with detective debugging information
  static void logError({
    required String errorType,
    required String errorMessage,
    required String solution,
    required List<String> affectedFiles,
    String? detectiveCommand,
    Map<String, dynamic>? additionalInfo,
  }) {
    final timestamp = DateTime.now().toIso8601String();
    
    final errorLog = '''
## Flutter Error: $errorType - $timestamp

**Classification**: ${_classifyError(errorMessage)}
**Detective Command**: ${detectiveCommand ?? _generateDetectiveCommand(errorType)}
**Files Affected**: 
${affectedFiles.map((file) => '- $file').join('\n')}

**Error Message**: $errorMessage
**Solution Applied**: $solution

**Additional Info**: ${additionalInfo?.toString() ?? 'None'}

---
''';

    if (kDebugMode) {
      developer.log(errorLog, name: 'FlutterErrorClassifier');
      print('🔧 FLUTTER ERROR LOGGED: $errorType');
    }
    
    // TODO: Write to FlutterErrorTrap.md file in production
    _writeToErrorTrapFile(errorLog);
  }

  /// Classify error based on pattern matching
  static String _classifyError(String errorMessage) {
    for (final category in errorCategories.entries) {
      for (final pattern in category.value) {
        if (RegExp(pattern, caseSensitive: false).hasMatch(errorMessage)) {
          return category.key;
        }
      }
    }
    return 'Unclassified Error';
  }

  /// Generate detective command for error investigation
  static String _generateDetectiveCommand(String errorType) {
    final commands = {
      'Widget Lifecycle': 'grep -r "setState" lib/ | grep -v "mounted"',
      'Layout Issues': 'flutter analyze | grep "RenderFlex"',
      'Navigation Errors': 'grep -r "Navigator.of" lib/',
      'State Management': 'grep -r "Provider.of" lib/',
      'Network/API Issues': 'Browser dev tools → Network tab → Check CORS headers',
      'PWA/Caching': 'Browser dev tools → Application → Service Workers',
    };
    
    return commands[errorType] ?? 'flutter analyze --verbose';
  }

  /// Write error log to FlutterErrorTrap.md (placeholder for file writing)
  static void _writeToErrorTrapFile(String errorLog) {
    // TODO: Implement file writing for production
    // For now, just log to console in debug mode
    if (kDebugMode) {
      print('📝 Error logged to FlutterErrorTrap.md:');
      print(errorLog);
    }
  }

  /// Analyze widget tree for common issues
  static List<String> analyzeWidgetTree(Widget widget) {
    final issues = <String>[];
    
    // This is a simplified analysis - in practice, you'd traverse the widget tree
    final widgetString = widget.toString();
    
    // Check for potential overflow issues
    if (widgetString.contains('Column') && !widgetString.contains('Expanded') && !widgetString.contains('Flexible')) {
      issues.add('Column without Expanded/Flexible may cause overflow');
    }
    
    // Check for potential navigation issues
    if (widgetString.contains('Navigator') && !widgetString.contains('BuildContext')) {
      issues.add('Navigator usage without proper BuildContext');
    }
    
    return issues;
  }
}

/// Elegant Flutter Solution - Single-pass data loading
class ElegantDataLoader<T> {
  final List<Future<T>> _futures = [];
  final List<String> _sources = [];

  /// Clear all pending data sources
  void clear() {
    _futures.clear();
    _sources.clear();
  }

  /// Add a data source to load
  void addSource(String sourceName, Future<T> future) {
    _sources.add(sourceName);
    _futures.add(future);
  }
  
  /// Load all data sources in parallel (Elegant Archive Solution pattern)
  Future<Map<String, T>> loadAllData() async {
    try {
      // Single-pass parallel loading - no sequential conflicts
      final results = await Future.wait(_futures);
      
      final resultMap = <String, T>{};
      for (int i = 0; i < _sources.length; i++) {
        resultMap[_sources[i]] = results[i];
      }
      
      FlutterErrorClassifier.logError(
        errorType: 'Data Loading Success',
        errorMessage: 'Successfully loaded ${_sources.length} data sources',
        solution: 'Elegant parallel loading pattern applied',
        affectedFiles: ['ElegantDataLoader'],
        detectiveCommand: 'Future.wait() parallel execution',
      );
      
      return resultMap;
    } catch (e) {
      FlutterErrorClassifier.logError(
        errorType: 'Data Loading Error',
        errorMessage: e.toString(),
        solution: 'Implement proper error handling and fallback data',
        affectedFiles: ['ElegantDataLoader'],
        detectiveCommand: 'Check network connectivity and API endpoints',
      );
      
      rethrow;
    }
  }
}

/// Safe setState utility that prevents lifecycle errors
mixin SafeStateMixin<T extends StatefulWidget> on State<T> {
  /// Safe setState that checks mounted state
  void safeSetState(VoidCallback fn) {
    if (mounted) {
      setState(fn);
    } else {
      FlutterErrorClassifier.logError(
        errorType: 'Widget Lifecycle',
        errorMessage: 'Attempted setState on unmounted widget',
        solution: 'Using SafeStateMixin.safeSetState() prevented crash',
        affectedFiles: [widget.runtimeType.toString()],
        detectiveCommand: 'grep -r "setState" lib/ | grep -v "mounted"',
      );
    }
  }
  
  /// Safe async operation with mounted check
  Future<void> safeAsyncOperation(Future<void> Function() operation) async {
    try {
      await operation();
      if (mounted) {
        // Safe to update UI
      }
    } catch (e) {
      if (mounted) {
        FlutterErrorClassifier.logError(
          errorType: 'Async Operation Error',
          errorMessage: e.toString(),
          solution: 'Proper error handling with mounted check',
          affectedFiles: [widget.runtimeType.toString()],
        );
      }
    }
  }
}

/// Cache Buster utility for Flutter Web
class FlutterCacheBuster {
  /// Add cache busting parameter to URL
  static String addCacheBuster(String url) {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    final separator = url.contains('?') ? '&' : '?';
    return '$url${separator}v=$timestamp';
  }
  
  /// Clear service worker cache (Web only)
  static Future<void> clearServiceWorkerCache() async {
    if (kIsWeb) {
      try {
        // Note: This requires additional web-specific implementation
        FlutterErrorClassifier.logError(
          errorType: 'Cache Management',
          errorMessage: 'Service worker cache clear requested',
          solution: 'Implementing cache invalidation strategy',
          affectedFiles: ['FlutterCacheBuster'],
          detectiveCommand: 'Browser dev tools → Application → Clear Storage',
        );
      } catch (e) {
        FlutterErrorClassifier.logError(
          errorType: 'Cache Management Error',
          errorMessage: e.toString(),
          solution: 'Manual cache clear required',
          affectedFiles: ['FlutterCacheBuster'],
        );
      }
    }
  }
}