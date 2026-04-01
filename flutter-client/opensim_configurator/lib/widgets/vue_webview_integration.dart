import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';

/// Widget for integrating Vue.js components into Flutter pages
/// Provides seamless embedding of the Vue.js dashboard components
class VueWebViewIntegration extends StatefulWidget {
  final String vuePath;
  final String title;
  final Map<String, String>? queryParams;
  final Function(String)? onPageFinished;
  final Function(String)? onPageStarted;
  
  const VueWebViewIntegration({
    Key? key,
    required this.vuePath,
    required this.title,
    this.queryParams,
    this.onPageFinished,
    this.onPageStarted,
  }) : super(key: key);
  
  @override
  _VueWebViewIntegrationState createState() => _VueWebViewIntegrationState();
}

class _VueWebViewIntegrationState extends State<VueWebViewIntegration> {
  late final WebViewController _controller;
  bool _isLoading = true;
  String? _error;
  
  @override
  void initState() {
    super.initState();
    _initializeWebView();
  }
  
  void _initializeWebView() {
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(Theme.of(context).scaffoldBackgroundColor)
      ..setNavigationDelegate(
        NavigationDelegate(
          onPageStarted: (String url) {
            setState(() {
              _isLoading = true;
              _error = null;
            });
            widget.onPageStarted?.call(url);
          },
          onPageFinished: (String url) {
            setState(() {
              _isLoading = false;
            });
            widget.onPageFinished?.call(url);
            _injectFlutterBridge();
          },
          onHttpError: (HttpResponseError error) {
            setState(() {
              _error = 'HTTP Error: ${error.response?.statusCode}';
              _isLoading = false;
            });
          },
          onWebResourceError: (WebResourceError error) {
            setState(() {
              _error = 'Resource Error: ${error.description}';
              _isLoading = false;
            });
          },
        ),
      );
    
    _loadUrl();
  }
  
  void _loadUrl() {
    String baseUrl = 'http://localhost:8080';
    String fullUrl = '$baseUrl${widget.vuePath}';
    
    // Add query parameters if provided
    if (widget.queryParams != null && widget.queryParams!.isNotEmpty) {
      final params = widget.queryParams!.entries
          .map((e) => '${e.key}=${Uri.encodeComponent(e.value)}')
          .join('&');
      fullUrl += '?$params';
    }
    
    _controller.loadRequest(Uri.parse(fullUrl));
  }
  
  /// Inject JavaScript bridge for Flutter-Vue.js communication
  void _injectFlutterBridge() {
    _controller.runJavaScript('''
      window.flutterBridge = {
        // Theme synchronization
        setTheme: function(theme) {
          if (window.vueApp && window.vueApp.changeTheme) {
            window.vueApp.changeTheme(theme);
          }
        },
        
        // Data refresh trigger
        refreshData: function() {
          if (window.vueApp && window.vueApp.refreshAllData) {
            window.vueApp.refreshAllData();
          }
        },
        
        // Navigation trigger
        navigateToTab: function(tabId) {
          if (window.vueApp && window.vueApp.setActiveTab) {
            window.vueApp.setActiveTab(tabId);
          }
        },
        
        // Get current data
        getCurrentData: function() {
          if (window.vueApp) {
            return JSON.stringify(window.vueApp.getCurrentData());
          }
          return '{}';
        }
      };
      
      // Notify Flutter when Vue.js is ready
      if (window.vueApp) {
        console.log('Vue.js app detected, bridge ready');
      }
    ''');
  }
  
  /// Refresh the WebView content
  void refresh() {
    _controller.reload();
  }
  
  /// Execute JavaScript in the WebView
  Future<void> executeJavaScript(String script) async {
    await _controller.runJavaScript(script);
  }
  
  /// Set theme in the Vue.js app
  void setTheme(String theme) {
    executeJavaScript('window.flutterBridge.setTheme("$theme")');
  }
  
  /// Navigate to specific tab in Vue.js app
  void navigateToTab(String tabId) {
    executeJavaScript('window.flutterBridge.navigateToTab("$tabId")');
  }
  
  /// Trigger data refresh in Vue.js app
  void refreshData() {
    executeJavaScript('window.flutterBridge.refreshData()');
  }
  
  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return _buildErrorWidget();
    }
    
    return Stack(
      children: [
        WebViewWidget(controller: _controller),
        if (_isLoading) _buildLoadingWidget(),
      ],
    );
  }
  
  Widget _buildLoadingWidget() {
    return Container(
      color: Theme.of(context).scaffoldBackgroundColor.withValues(alpha: 0.8),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 16),
            Text(
              'Loading ${widget.title}...',
              style: Theme.of(context).textTheme.bodyLarge,
            ),
          ],
        ),
      ),
    );
  }
  
  Widget _buildErrorWidget() {
    return Container(
      padding: EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.error_outline,
            size: 64,
            color: Theme.of(context).colorScheme.error,
          ),
          SizedBox(height: 16),
          Text(
            'Failed to load ${widget.title}',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          SizedBox(height: 8),
          Text(
            _error ?? 'Unknown error occurred',
            style: Theme.of(context).textTheme.bodyMedium,
            textAlign: TextAlign.center,
          ),
          SizedBox(height: 24),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceEvenly,
            children: [
              ElevatedButton.icon(
                onPressed: () {
                  setState(() {
                    _error = null;
                  });
                  _loadUrl();
                },
                icon: Icon(Icons.refresh),
                label: Text('Retry'),
              ),
              TextButton.icon(
                onPressed: () {
                  // Fallback to native Flutter implementation
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('Switching to native Flutter interface'),
                      duration: Duration(seconds: 2),
                    ),
                  );
                },
                icon: Icon(Icons.mobile_friendly),
                label: Text('Use Native'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// Enhanced wrapper for Vue.js dashboard integration
class VueJsDashboardIntegration extends StatefulWidget {
  final String tabId;
  final String title;
  final Widget? fallbackWidget;
  
  const VueJsDashboardIntegration({
    Key? key,
    required this.tabId,
    required this.title,
    this.fallbackWidget,
  }) : super(key: key);
  
  @override
  _VueJsDashboardIntegrationState createState() => _VueJsDashboardIntegrationState();
}

class _VueJsDashboardIntegrationState extends State<VueJsDashboardIntegration> {
  bool _useWebView = true;
  
  @override
  Widget build(BuildContext context) {
    if (!_useWebView && widget.fallbackWidget != null) {
      return widget.fallbackWidget!;
    }
    
    return Column(
      children: [
        // Toggle between WebView and native implementation
        Container(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              Text('Interface: '),
              SegmentedButton<bool>(
                segments: [
                  ButtonSegment(
                    value: true,
                    label: Text('Vue.js'),
                    icon: Icon(Icons.web),
                  ),
                  ButtonSegment(
                    value: false,
                    label: Text('Native'),
                    icon: Icon(Icons.mobile_friendly),
                  ),
                ],
                selected: {_useWebView},
                onSelectionChanged: (Set<bool> selection) {
                  setState(() {
                    _useWebView = selection.first;
                  });
                },
              ),
            ],
          ),
        ),
        Expanded(
          child: _useWebView
              ? VueWebViewIntegration(
                  vuePath: '/',
                  title: widget.title,
                  queryParams: {'tab': widget.tabId},
                  onPageFinished: (url) {
                    // Auto-navigate to the correct tab when page loads
                    Future.delayed(Duration(milliseconds: 1000), () {
                      // This will be implemented when WebView is ready
                    });
                  },
                )
              : widget.fallbackWidget ?? _buildPlaceholder(),
        ),
      ],
    );
  }
  
  Widget _buildPlaceholder() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.construction, size: 64),
          SizedBox(height: 16),
          Text('Native ${widget.title} interface coming soon!'),
        ],
      ),
    );
  }
}