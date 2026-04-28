//! Web-based client interface for OpenSim Next
//!
//! This module provides a simple web interface for testing and demonstrating
//! the WebSocket functionality of OpenSim Next.

use anyhow::Result;
use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Web client server for serving the browser-based interface
pub struct WebClientServer {
    port: u16,
    ai_router: Option<Router>,
}

impl WebClientServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            ai_router: None,
        }
    }

    pub fn with_ai_router(mut self, router: Router) -> Self {
        self.ai_router = Some(router);
        self
    }

    /// Start the web client server
    pub async fn start(self) -> Result<()> {
        let port = self.port;
        info!("Starting web client server on port {}", port);

        let app = self.create_router().await;

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Web client available at: http://{}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn create_router(self) -> Router {
        let mut router = Router::new()
            .route("/", get(flutter_configurator_handler))
            .route("/index.html", get(flutter_configurator_handler))
            .route("/flutter", get(flutter_configurator_handler))
            .route("/dashboard", get(dashboard_handler))
            .route("/css/styles.css", get(styles_handler))
            .route("/js/app.js", get(app_js_handler))
            .route("/js/app-v2.js", get(app_js_v2_handler))
            .route(
                "/js/multi-instance-server-manager.js",
                get(multi_instance_js_handler),
            )
            .route("/client.html", get(client_handler))
            .route("/client.js", get(client_js_handler))
            .route("/health", get(|| async { "OK" }))
            // Flutter Web Assets (served from root)
            .route("/main.dart.js", get(flutter_main_dart_js_handler))
            .route("/flutter.js", get(flutter_js_handler))
            .route("/flutter_bootstrap.js", get(flutter_bootstrap_handler))
            .route(
                "/flutter_service_worker.js",
                get(flutter_service_worker_handler),
            )
            .route("/manifest.json", get(flutter_manifest_handler))
            .route("/assets/*path", get(flutter_assets_assets_handler))
            .route("/canvaskit/*path", get(flutter_canvaskit_handler))
            .route("/icons/*path", get(flutter_icons_handler))
            // Flutter Web Configurator Routes (legacy)
            .route("/configurator", get(flutter_configurator_handler))
            .route("/configurator/*path", get(flutter_assets_handler))
            // User Manual Integration
            .route("/user-manual", get(user_manual_handler))
            .route("/api/user-manual", get(user_manual_api_handler))
            // API Proxy endpoints for dashboard (no CORS issues)
            .route("/api/health", get(api_health_proxy))
            .route("/api/metrics", get(api_metrics_proxy))
            .route("/api/info", get(api_info_proxy));

        if let Some(ai) = self.ai_router {
            use tower_http::cors::{Any, CorsLayer};
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            router = router.nest("/api/ai", ai.layer(cors));
            info!("AI API mounted at /api/ai/*");
        }

        router
    }
}

/// Dashboard page handler - serves the main frontend
async fn dashboard_handler() -> impl IntoResponse {
    let html = include_str!("../../web-frontend/index.html");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
}

/// Styles handler - serves the CSS
async fn styles_handler() -> impl IntoResponse {
    let css = include_str!("../../web-frontend/css/styles.css");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        css,
    )
}

/// App JavaScript handler - serves the main application
async fn app_js_handler() -> impl IntoResponse {
    let js = include_str!("../../web-frontend/js/app.js");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        js,
    )
}

/// App JavaScript v2 handler - serves the updated application
async fn app_js_v2_handler() -> impl IntoResponse {
    let js = include_str!("../../web-frontend/js/app-v2.js");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        js,
    )
}

/// Phase 20 Multi-Instance Server Manager handler - serves the Phase 20 functionality
async fn multi_instance_js_handler() -> impl IntoResponse {
    let js = include_str!("../../web-frontend/js/multi-instance-server-manager.js");
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        js,
    )
}

/// Legacy index page handler
async fn index_handler() -> impl IntoResponse {
    let html_content = r###"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim Next - Web Interface</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            border-radius: 10px;
            text-align: center;
            margin-bottom: 30px;
        }
        .card {
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .btn {
            background: #667eea;
            color: white;
            padding: 12px 24px;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            margin: 5px;
        }
        .btn:hover {
            background: #5a67d8;
        }
        .feature-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }
        .status {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: bold;
        }
        .status.completed {
            background: #10b981;
            color: white;
        }
        .status.in-progress {
            background: #f59e0b;
            color: white;
        }
        .status.future {
            background: #6b7280;
            color: white;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 OpenSim Next</h1>
        <p>Production-Ready Virtual World Server</p>
        <p><strong>Phase 13: WebSocket & Web Client Support</strong></p>
    </div>
    
    <div class="card">
        <h2>🌐 Web Client Interface</h2>
        <p>Experience OpenSim Next directly in your browser with real-time WebSocket communication.</p>
        <a href="/client.html" class="btn">Launch Web Client</a>
    </div>
    
    <div class="feature-grid">
        <div class="card">
            <h3>🔌 WebSocket Features</h3>
            <ul>
                <li><span class="status completed">✓</span> Real-time bidirectional communication</li>
                <li><span class="status completed">✓</span> LLSD protocol support</li>
                <li><span class="status completed">✓</span> Authentication and session management</li>
                <li><span class="status completed">✓</span> Message routing and handling</li>
                <li><span class="status in-progress">⚠</span> Avatar updates and chat</li>
                <li><span class="status future">⏳</span> WebRTC voice/video</li>
            </ul>
        </div>
        
        <div class="card">
            <h3>🌍 Virtual World Features</h3>
            <ul>
                <li><span class="status completed">✓</span> Second Life viewer compatibility</li>
                <li><span class="status completed">✓</span> Multi-physics engine system</li>
                <li><span class="status completed">✓</span> Complete user systems</li>
                <li><span class="status completed">✓</span> Real-time monitoring</li>
                <li><span class="status completed">✓</span> Enterprise-grade stability</li>
                <li><span class="status completed">✓</span> Production deployment ready</li>
            </ul>
        </div>
        
        <div class="card">
            <h3>🔧 Technical Specifications</h3>
            <ul>
                <li><strong>Architecture:</strong> Rust/Zig hybrid</li>
                <li><strong>Physics:</strong> ODE, Bullet, UBODE, POS, Basic</li>
                <li><strong>Protocols:</strong> LLSD, WebSocket, HTTP</li>
                <li><strong>Database:</strong> PostgreSQL, SQLite</li>
                <li><strong>Monitoring:</strong> Prometheus, WebSocket</li>
                <li><strong>SDKs:</strong> 8 languages supported</li>
            </ul>
        </div>
        
        <div class="card">
            <h3>📊 Server Status</h3>
            <div id="server-status">
                <p>Loading server status...</p>
            </div>
        </div>
    </div>
    
    <div class="card">
        <h2>📖 Documentation & Resources</h2>
        <p>Comprehensive guides and resources for developers and users.</p>
        <a href="#" class="btn">API Documentation</a>
        <a href="#" class="btn">Developer Guide</a>
        <a href="#" class="btn">GitHub Repository</a>
    </div>
    
    <script>
        // Load server status
        async function loadServerStatus() {
            try {
                const response = await fetch('/health');
                const status = await response.text();
                document.getElementById('server-status').innerHTML = \`
                    <p><span class="status completed">✓</span> Server: \${status}</p>
                    <p><span class="status completed">✓</span> WebSocket: Available on port 9001</p>
                    <p><span class="status completed">✓</span> Main Server: Running on port 9000</p>
                    <p><span class="status completed">✓</span> Monitoring: Available on port 9100</p>
                \`;
            } catch (error) {
                document.getElementById('server-status').innerHTML = \`
                    <p><span class="status future">⚠</span> Server: Connection failed</p>
                    <p>Please ensure the OpenSim Next server is running.</p>
                \`;
            }
        }
        
        loadServerStatus();
        setInterval(loadServerStatus, 10000); // Update every 10 seconds
    </script>
</body>
</html>
    "###;
    Html(html_content)
}

/// Client page handler
async fn client_handler() -> impl IntoResponse {
    let html_content = r###"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim Next - Web Client</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #1a1a1a;
            color: #ffffff;
        }
        .client-container {
            display: grid;
            grid-template-columns: 300px 1fr;
            gap: 20px;
            height: calc(100vh - 40px);
        }
        .sidebar {
            background: #2d2d2d;
            border-radius: 10px;
            padding: 20px;
            overflow-y: auto;
        }
        .main-area {
            background: #2d2d2d;
            border-radius: 10px;
            padding: 20px;
            display: flex;
            flex-direction: column;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            text-align: center;
        }
        .connection-status {
            padding: 10px;
            border-radius: 5px;
            margin-bottom: 20px;
            text-align: center;
        }
        .connected {
            background: #10b981;
        }
        .disconnected {
            background: #ef4444;
        }
        .connecting {
            background: #f59e0b;
        }
        .message-area {
            flex: 1;
            background: #1a1a1a;
            border-radius: 5px;
            padding: 15px;
            overflow-y: auto;
            margin-bottom: 20px;
            border: 1px solid #444;
        }
        .message-input {
            display: flex;
            gap: 10px;
        }
        .message-input input {
            flex: 1;
            padding: 10px;
            border: 1px solid #444;
            border-radius: 5px;
            background: #1a1a1a;
            color: white;
        }
        .btn {
            background: #667eea;
            color: white;
            padding: 10px 20px;
            border: none;
            border-radius: 5px;
            cursor: pointer;
        }
        .btn:hover {
            background: #5a67d8;
        }
        .btn:disabled {
            background: #666;
            cursor: not-allowed;
        }
        .message {
            margin-bottom: 10px;
            padding: 8px;
            border-radius: 4px;
        }
        .message.sent {
            background: #667eea;
            text-align: right;
        }
        .message.received {
            background: #374151;
        }
        .message.system {
            background: #065f46;
            font-style: italic;
        }
        .message.error {
            background: #7f1d1d;
        }
        .controls {
            margin-bottom: 20px;
        }
        .controls button {
            margin-right: 10px;
            margin-bottom: 10px;
        }
        .stats {
            background: #1a1a1a;
            padding: 10px;
            border-radius: 5px;
            font-size: 12px;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
    <div class="client-container">
        <div class="sidebar">
            <div class="header">
                <h3>OpenSim Web Client</h3>
            </div>
            
            <div id="connection-status" class="connection-status disconnected">
                Disconnected
            </div>
            
            <div class="controls">
                <button id="connect-btn" class="btn">Connect</button>
                <button id="disconnect-btn" class="btn" disabled>Disconnect</button>
            </div>
            
            <div class="stats">
                <h4>Connection Stats</h4>
                <div id="stats-content">
                    <p>Messages Sent: <span id="sent-count">0</span></p>
                    <p>Messages Received: <span id="received-count">0</span></p>
                    <p>Uptime: <span id="uptime">00:00:00</span></p>
                </div>
            </div>
            
            <div>
                <h4>Quick Actions</h4>
                <button id="heartbeat-btn" class="btn" disabled>Send Heartbeat</button>
                <button id="auth-btn" class="btn" disabled>Authenticate</button>
                <button id="clear-btn" class="btn">Clear Messages</button>
            </div>
        </div>
        
        <div class="main-area">
            <div class="header">
                <h2>Real-time Communication</h2>
                <p>WebSocket connection to OpenSim Next server</p>
            </div>
            
            <div id="message-area" class="message-area">
                <div class="message system">Welcome to OpenSim Next Web Client!</div>
                <div class="message system">Click "Connect" to establish WebSocket connection.</div>
            </div>
            
            <div class="message-input">
                <input type="text" id="message-input" placeholder="Enter message or JSON..." disabled>
                <button id="send-btn" class="btn" disabled>Send</button>
            </div>
        </div>
    </div>
    
    <script src="/client.js"></script>
</body>
</html>
    "###;
    Html(html_content)
}

/// Client JavaScript handler
async fn client_js_handler() -> impl IntoResponse {
    let js_content = r#"
class OpenSimWebClient {
    constructor() {
        this.socket = null;
        this.connected = false;
        this.messagesSent = 0;
        this.messagesReceived = 0;
        this.connectTime = null;
        this.uptimeInterval = null;
        
        this.initializeUI();
        this.setupEventListeners();
    }
    
    initializeUI() {
        this.statusEl = document.getElementById('connection-status');
        this.messageArea = document.getElementById('message-area');
        this.messageInput = document.getElementById('message-input');
        this.connectBtn = document.getElementById('connect-btn');
        this.disconnectBtn = document.getElementById('disconnect-btn');
        this.sendBtn = document.getElementById('send-btn');
        this.heartbeatBtn = document.getElementById('heartbeat-btn');
        this.authBtn = document.getElementById('auth-btn');
        this.clearBtn = document.getElementById('clear-btn');
        this.sentCountEl = document.getElementById('sent-count');
        this.receivedCountEl = document.getElementById('received-count');
        this.uptimeEl = document.getElementById('uptime');
    }
    
    setupEventListeners() {
        this.connectBtn.addEventListener('click', () => this.connect());
        this.disconnectBtn.addEventListener('click', () => this.disconnect());
        this.sendBtn.addEventListener('click', () => this.sendMessage());
        this.heartbeatBtn.addEventListener('click', () => this.sendHeartbeat());
        this.authBtn.addEventListener('click', () => this.authenticate());
        this.clearBtn.addEventListener('click', () => this.clearMessages());
        
        this.messageInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.sendMessage();
            }
        });
    }
    
    connect() {
        if (this.connected) return;
        
        this.updateStatus('connecting', 'Connecting...');
        
        // Connect to WebSocket server on port 9001
        const wsUrl = `ws://${window.location.hostname}:9001/ws`;
        this.socket = new WebSocket(wsUrl);
        
        this.socket.onopen = () => {
            this.connected = true;
            this.connectTime = Date.now();
            this.updateStatus('connected', 'Connected');
            this.addMessage('system', 'Connected to OpenSim Next WebSocket server');
            this.updateButtons();
            this.startUptimeTimer();
        };
        
        this.socket.onclose = () => {
            this.connected = false;
            this.updateStatus('disconnected', 'Disconnected');
            this.addMessage('system', 'Disconnected from server');
            this.updateButtons();
            this.stopUptimeTimer();
        };
        
        this.socket.onerror = (error) => {
            this.addMessage('error', `Connection error: ${error.message || 'Unknown error'}`);
            this.updateStatus('disconnected', 'Connection Error');
        };
        
        this.socket.onmessage = (event) => {
            this.messagesReceived++;
            this.updateStats();
            
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (e) {
                this.addMessage('received', `Raw: ${event.data}`);
            }
        };
    }
    
    disconnect() {
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
    }
    
    sendMessage() {
        const text = this.messageInput.value.trim();
        if (!text || !this.connected) return;
        
        try {
            // Try to parse as JSON first
            const message = JSON.parse(text);
            this.sendJSON(message);
        } catch (e) {
            // Send as simple text message wrapped in WebSocket format
            const wrappedMessage = {
                id: this.generateId(),
                timestamp: Math.floor(Date.now() / 1000),
                message: {
                    type: "ChatMessage",
                    from: "WebClient",
                    message: text,
                    channel: 0
                }
            };
            this.sendJSON(wrappedMessage);
        }
        
        this.messageInput.value = '';
    }
    
    sendJSON(obj) {
        if (!this.connected) return;
        
        const json = JSON.stringify(obj);
        this.socket.send(json);
        this.messagesSent++;
        this.updateStats();
        this.addMessage('sent', JSON.stringify(obj, null, 2));
    }
    
    sendHeartbeat() {
        const heartbeat = {
            id: this.generateId(),
            timestamp: Math.floor(Date.now() / 1000),
            message: {
                type: "Heartbeat"
            }
        };
        this.sendJSON(heartbeat);
    }
    
    authenticate() {
        const auth = {
            id: this.generateId(),
            timestamp: Math.floor(Date.now() / 1000),
            message: {
                type: "Auth",
                token: null,
                session_id: null
            }
        };
        this.sendJSON(auth);
    }
    
    handleMessage(message) {
        let displayText = JSON.stringify(message, null, 2);
        
        // Handle specific message types
        if (message.message) {
            switch (message.message.type) {
                case 'AuthResponse':
                    if (message.message.success) {
                        this.addMessage('system', `Authentication successful! Session: ${message.message.session_id}`);
                    } else {
                        this.addMessage('error', `Authentication failed: ${message.message.error}`);
                    }
                    break;
                case 'Heartbeat':
                    this.addMessage('system', 'Heartbeat received from server');
                    // Send pong response
                    this.sendJSON({
                        id: this.generateId(),
                        timestamp: Math.floor(Date.now() / 1000),
                        message: { type: "Pong" }
                    });
                    break;
                case 'Error':
                    this.addMessage('error', `Server error: ${message.message.message}`);
                    break;
                default:
                    this.addMessage('received', displayText);
            }
        } else {
            this.addMessage('received', displayText);
        }
    }
    
    updateStatus(className, text) {
        this.statusEl.className = `connection-status ${className}`;
        this.statusEl.textContent = text;
    }
    
    updateButtons() {
        this.connectBtn.disabled = this.connected;
        this.disconnectBtn.disabled = !this.connected;
        this.sendBtn.disabled = !this.connected;
        this.messageInput.disabled = !this.connected;
        this.heartbeatBtn.disabled = !this.connected;
        this.authBtn.disabled = !this.connected;
    }
    
    addMessage(type, content) {
        const messageDiv = document.createElement('div');
        messageDiv.className = `message ${type}`;
        
        const timestamp = new Date().toLocaleTimeString();
        messageDiv.innerHTML = `<small>${timestamp}</small><br>${this.escapeHtml(content)}`;
        
        this.messageArea.appendChild(messageDiv);
        this.messageArea.scrollTop = this.messageArea.scrollHeight;
    }
    
    clearMessages() {
        this.messageArea.innerHTML = '';
        this.addMessage('system', 'Messages cleared');
    }
    
    updateStats() {
        this.sentCountEl.textContent = this.messagesSent;
        this.receivedCountEl.textContent = this.messagesReceived;
    }
    
    startUptimeTimer() {
        this.uptimeInterval = setInterval(() => {
            if (this.connectTime) {
                const uptime = Date.now() - this.connectTime;
                const seconds = Math.floor(uptime / 1000) % 60;
                const minutes = Math.floor(uptime / 60000) % 60;
                const hours = Math.floor(uptime / 3600000);
                
                this.uptimeEl.textContent = 
                    `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
            }
        }, 1000);
    }
    
    stopUptimeTimer() {
        if (this.uptimeInterval) {
            clearInterval(this.uptimeInterval);
            this.uptimeInterval = null;
        }
    }
    
    generateId() {
        return Math.random().toString(36).substr(2, 9);
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize the client when the page loads
document.addEventListener('DOMContentLoaded', () => {
    new OpenSimWebClient();
});
    "#;

    ([("Content-Type", "application/javascript")], js_content)
}

/// Flutter Web configurator handler - serves the Flutter Web app
async fn flutter_configurator_handler() -> impl IntoResponse {
    // Try to serve the built Flutter Web app from the served directory
    match std::fs::read_to_string("./rust/web-frontend/flutter/index.html") {
        Ok(html) => {
            // Serve Flutter web app from root path
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    (header::PRAGMA, "no-cache"),
                    (header::EXPIRES, "0"),
                ],
                html,
            )
                .into_response()
        }
        Err(_) => {
            // Fallback to a redirect message if Flutter Web isn't built yet
            let fallback_html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Flutter Web Configurator - Building...</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            background: rgba(255, 255, 255, 0.1);
            padding: 3rem;
            border-radius: 20px;
            backdrop-filter: blur(10px);
            max-width: 500px;
        }
        h1 { margin-bottom: 1rem; }
        .spinner {
            width: 40px;
            height: 40px;
            border: 4px solid rgba(255, 255, 255, 0.3);
            border-top: 4px solid white;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin: 2rem auto;
        }
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        .instructions {
            text-align: left;
            background: rgba(0, 0, 0, 0.2);
            padding: 1rem;
            border-radius: 10px;
            margin-top: 2rem;
            font-family: monospace;
            font-size: 0.9rem;
        }
        .back-button {
            display: inline-block;
            margin-top: 2rem;
            padding: 0.75rem 1.5rem;
            background: rgba(255, 255, 255, 0.2);
            color: white;
            text-decoration: none;
            border-radius: 10px;
            border: 1px solid rgba(255, 255, 255, 0.3);
            transition: all 0.3s;
        }
        .back-button:hover {
            background: rgba(255, 255, 255, 0.3);
            transform: translateY(-2px);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🛠️ Flutter Web Configurator</h1>
        <div class="spinner"></div>
        <p>The Flutter Web configurator is not yet built.</p>
        
        <div class="instructions">
            <strong>To build the Flutter Web configurator:</strong><br><br>
            1. cd ../flutter-client/opensim_configurator/<br>
            2. flutter pub get<br>
            3. flutter build web<br>
            4. Restart OpenSim Next server
        </div>
        
        <a href="/" class="back-button">← Back to Main Dashboard</a>
        
        <script>
            // Auto-refresh every 5 seconds to check if Flutter Web is built
            setTimeout(() => window.location.reload(), 5000);
        </script>
    </div>
</body>
</html>
            "#;
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                fallback_html.to_string(),
            )
                .into_response()
        }
    }
}

/// Flutter Web assets handler - serves Flutter Web static assets
async fn flutter_assets_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let file_path = format!("./rust/web-frontend/flutter/{}", path);

    match std::fs::read(&file_path) {
        Ok(content) => {
            let content_type = match path.split('.').last() {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("woff") | Some("woff2") => "font/woff2",
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    (header::PRAGMA, "no-cache"),
                    (header::EXPIRES, "0"),
                ],
                content,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Flutter assets/assets handler - serves files from assets/ subdirectory
async fn flutter_assets_assets_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let file_path = format!("./rust/web-frontend/flutter/assets/{}", path);

    match std::fs::read(&file_path) {
        Ok(content) => {
            let content_type = match path.split('.').last() {
                Some("js") => "application/javascript",
                Some("css") => "text/css",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("woff") | Some("woff2") => "font/woff2",
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    (header::PRAGMA, "no-cache"),
                    (header::EXPIRES, "0"),
                ],
                content,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Flutter canvaskit handler - serves files from canvaskit/ subdirectory
async fn flutter_canvaskit_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let file_path = format!("./rust/web-frontend/flutter/canvaskit/{}", path);

    match std::fs::read(&file_path) {
        Ok(content) => {
            let content_type = match path.split('.').last() {
                Some("js") => "application/javascript",
                Some("wasm") => "application/wasm",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    (header::PRAGMA, "no-cache"),
                    (header::EXPIRES, "0"),
                ],
                content,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Flutter icons handler - serves files from icons/ subdirectory
async fn flutter_icons_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let file_path = format!("./rust/web-frontend/flutter/icons/{}", path);

    match std::fs::read(&file_path) {
        Ok(content) => {
            let content_type = match path.split('.').last() {
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                    (header::PRAGMA, "no-cache"),
                    (header::EXPIRES, "0"),
                ],
                content,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

/// Flutter main.dart.js handler
async fn flutter_main_dart_js_handler() -> impl IntoResponse {
    match std::fs::read("./rust/web-frontend/flutter/main.dart.js") {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            content,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "main.dart.js not found").into_response(),
    }
}

/// Flutter.js handler
async fn flutter_js_handler() -> impl IntoResponse {
    match std::fs::read("./rust/web-frontend/flutter/flutter.js") {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            content,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "flutter.js not found").into_response(),
    }
}

/// Flutter bootstrap handler
async fn flutter_bootstrap_handler() -> impl IntoResponse {
    match std::fs::read("./rust/web-frontend/flutter/flutter_bootstrap.js") {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            content,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "flutter_bootstrap.js not found").into_response(),
    }
}

/// Flutter service worker handler
async fn flutter_service_worker_handler() -> impl IntoResponse {
    // Provide a minimal service worker that doesn't try to cache resources
    // This prevents the cache errors while still allowing Flutter to function
    let minimal_service_worker = r#"
// Minimal service worker for Flutter Web configurator
// Disables caching to prevent resource fetch errors

const TEMP_CACHE_NAME = 'flutter-temp-cache';
const CACHE_NAME = 'flutter-app-cache';

self.addEventListener('activate', function(event) {
  // Clean up old caches
  event.waitUntil(
    caches.keys().then(function(cacheNames) {
      return Promise.all(
        cacheNames.map(function(cacheName) {
          if (cacheName !== CACHE_NAME) {
            console.log('Deleting cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
});

self.addEventListener('fetch', function(event) {
  // Simply fetch from network without caching to avoid resource errors
  event.respondWith(
    fetch(event.request).catch(function() {
      // If network fails, try cache
      return caches.match(event.request);
    })
  );
});

self.addEventListener('message', function(event) {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});
"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/javascript")],
        minimal_service_worker,
    )
        .into_response()
}

/// Flutter manifest handler
async fn flutter_manifest_handler() -> impl IntoResponse {
    match std::fs::read_to_string("./rust/web-frontend/flutter/manifest.json") {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Manifest not found").into_response(),
    }
}

/// User Manual HTML viewer handler
async fn user_manual_handler() -> impl IntoResponse {
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OpenSim Next - User Manual</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .header {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            padding: 1rem;
            border-bottom: 1px solid rgba(0,0,0,0.1);
            position: sticky;
            top: 0;
            z-index: 100;
        }
        
        .header-content {
            max-width: 1200px;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .logo {
            font-size: 1.5rem;
            font-weight: 700;
            color: #2563eb;
        }
        
        .nav-buttons {
            display: flex;
            gap: 1rem;
        }
        
        .btn {
            padding: 0.5rem 1rem;
            background: #2563eb;
            color: white;
            border: none;
            border-radius: 6px;
            text-decoration: none;
            cursor: pointer;
            transition: background 0.2s;
        }
        
        .btn:hover {
            background: #1d4ed8;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
            display: grid;
            grid-template-columns: 300px 1fr;
            gap: 2rem;
            min-height: calc(100vh - 80px);
        }
        
        .sidebar {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 12px;
            padding: 1.5rem;
            height: fit-content;
            position: sticky;
            top: 100px;
        }
        
        .sidebar h3 {
            margin-top: 0;
            color: #1e293b;
            border-bottom: 2px solid #2563eb;
            padding-bottom: 0.5rem;
        }
        
        .toc {
            list-style: none;
            padding: 0;
        }
        
        .toc li {
            margin: 0.5rem 0;
        }
        
        .toc a {
            color: #64748b;
            text-decoration: none;
            display: block;
            padding: 0.5rem;
            border-radius: 6px;
            transition: all 0.2s;
        }
        
        .toc a:hover, .toc a.active {
            background: #2563eb;
            color: white;
        }
        
        .content {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 12px;
            padding: 2rem;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        }
        
        .content h1, .content h2, .content h3, .content h4 {
            color: #1e293b;
            margin-top: 2rem;
            margin-bottom: 1rem;
        }
        
        .content h1 {
            border-bottom: 3px solid #2563eb;
            padding-bottom: 0.5rem;
        }
        
        .content h2 {
            border-left: 4px solid #2563eb;
            padding-left: 1rem;
        }
        
        .content pre {
            background: #1e293b;
            color: #e2e8f0;
            padding: 1rem;
            border-radius: 6px;
            overflow-x: auto;
        }
        
        .content code {
            background: #f1f5f9;
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            font-family: 'JetBrains Mono', monospace;
        }
        
        .content blockquote {
            border-left: 4px solid #10b981;
            background: #f0fdf4;
            padding: 1rem;
            margin: 1rem 0;
            border-radius: 6px;
        }
        
        .loading {
            text-align: center;
            padding: 2rem;
            color: #64748b;
        }
        
        .spinner {
            width: 40px;
            height: 40px;
            border: 4px solid #e2e8f0;
            border-top: 4px solid #2563eb;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin: 1rem auto;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        @media (max-width: 768px) {
            .container {
                grid-template-columns: 1fr;
                padding: 1rem;
            }
            
            .sidebar {
                position: static;
            }
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="header-content">
            <div class="logo">📚 OpenSim Next User Manual</div>
            <div class="nav-buttons">
                <a href="/" class="btn">← Back to Dashboard</a>
                <a href="/configurator" class="btn">⚙️ Configurator</a>
            </div>
        </div>
    </div>
    
    <div class="container">
        <div class="sidebar">
            <h3>📋 Table of Contents</h3>
            <ul class="toc" id="toc">
                <li><div class="loading">Loading manual...</div></li>
            </ul>
        </div>
        
        <div class="content">
            <div class="loading">
                <div class="spinner"></div>
                <p>Loading OpenSim Next User Manual...</p>
            </div>
        </div>
    </div>
    
    <script>
        class UserManualViewer {
            constructor() {
                this.loadManual();
                this.setupNavigation();
            }
            
            async loadManual() {
                try {
                    const response = await fetch('/api/user-manual');
                    const manual = await response.text();
                    this.renderManual(manual);
                } catch (error) {
                    this.showError('Failed to load user manual: ' + error.message);
                }
            }
            
            renderManual(markdown) {
                // Simple markdown to HTML conversion
                let html = markdown
                    // Headers
                    .replace(/^### (.*$)/gm, '<h3>$1</h3>')
                    .replace(/^## (.*$)/gm, '<h2 id="' + this.slugify('$1') + '">$1</h2>')
                    .replace(/^# (.*$)/gm, '<h1 id="' + this.slugify('$1') + '">$1</h1>')
                    
                    // Code blocks
                    .replace(/```([^`]+)```/g, '<pre><code>$1</code></pre>')
                    .replace(/`([^`]+)`/g, '<code>$1</code>')
                    
                    // Lists
                    .replace(/^\* (.*$)/gm, '<li>$1</li>')
                    .replace(/^- (.*$)/gm, '<li>$1</li>')
                    
                    // Bold and italic
                    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
                    .replace(/\*(.*?)\*/g, '<em>$1</em>')
                    
                    // Line breaks
                    .replace(/\n\n/g, '</p><p>')
                    .replace(/\n/g, '<br>');
                
                // Wrap in paragraphs
                html = '<p>' + html + '</p>';
                
                // Clean up
                html = html
                    .replace(/<p><h/g, '<h')
                    .replace(/<\/h([1-6])><\/p>/g, '</h$1>')
                    .replace(/<p><pre/g, '<pre')
                    .replace(/<\/pre><\/p>/g, '</pre>')
                    .replace(/<p><li>/g, '<ul><li>')
                    .replace(/<\/li><\/p>/g, '</li></ul>');
                
                document.querySelector('.content').innerHTML = html;
                this.generateTOC();
            }
            
            generateTOC() {
                const headers = document.querySelectorAll('h1, h2, h3');
                const toc = document.getElementById('toc');
                
                if (headers.length === 0) {
                    toc.innerHTML = '<li>No sections found</li>';
                    return;
                }
                
                let tocHTML = '';
                headers.forEach(header => {
                    const id = this.slugify(header.textContent);
                    header.id = id;
                    const level = header.tagName.toLowerCase();
                    const listItem = document.createElement('li');
                    const link = document.createElement('a');
                    link.href = '#' + id;
                    link.textContent = header.textContent;
                    listItem.appendChild(link);
                    tocHTML += listItem.outerHTML;
                });
                
                toc.innerHTML = tocHTML;
            }
            
            slugify(text) {
                return text.toLowerCase()
                    .replace(/[^\\\\w\\\\s-]/g, '')
                    .replace(/[\\\\s_-]+/g, '-')
                    .replace(/^-+|-+$/g, '');
            }
            
            setupNavigation() {
                // Smooth scrolling for TOC links
                document.addEventListener('click', (e) => {
                    if (e.target.matches('.toc a')) {
                        e.preventDefault();
                        const targetId = e.target.getAttribute('href').substring(1);
                        const target = document.getElementById(targetId);
                        if (target) {
                            target.scrollIntoView({ behavior: 'smooth', block: 'start' });
                            
                            // Update active state
                            document.querySelectorAll('.toc a').forEach(a => a.classList.remove('active'));
                            e.target.classList.add('active');
                        }
                    }
                });
                
                // Update active TOC item on scroll
                window.addEventListener('scroll', () => {
                    const headers = document.querySelectorAll('h1, h2, h3');
                    let current = '';
                    
                    headers.forEach(header => {
                        const rect = header.getBoundingClientRect();
                        if (rect.top <= 100) {
                            current = header.id;
                        }
                    });
                    
                    document.querySelectorAll('.toc a').forEach(a => {
                        a.classList.remove('active');
                        if (a.getAttribute('href') === '#' + current) {
                            a.classList.add('active');
                        }
                    });
                });
            }
            
            showError(message) {
                document.querySelector('.content').innerHTML = `
                    <div style="text-align: center; color: #ef4444; padding: 2rem;">
                        <h2>❌ Error Loading Manual</h2>
                        <p>${message}</p>
                        <button onclick="location.reload()" class="btn">🔄 Retry</button>
                    </div>
                `;
            }
        }
        
        // Initialize when page loads
        document.addEventListener('DOMContentLoaded', () => {
            new UserManualViewer();
        });
    </script>
</body>
</html>
    "#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html_content,
    )
}

/// User Manual API handler - serves the raw markdown content
async fn user_manual_api_handler() -> impl IntoResponse {
    match std::fs::read_to_string("../USER_MANUAL.md") {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            content,
        )
            .into_response(),
        Err(_) => {
            let fallback_content = r#"# OpenSim Next - User Manual

## Manual Not Found

The USER_MANUAL.md file could not be loaded. This may be because:

1. The file is not in the expected location (../USER_MANUAL.md)
2. The server doesn't have read permissions
3. The file path needs to be adjusted

## Quick Start

While the full manual loads, here are the essential commands:

### Starting the Server
```bash
RUST_LOG=info DATABASE_URL="postgresql://user:pass@localhost:5432/opensim" cargo run --bin opensim-next
```

### Available Interfaces
- **Main Dashboard**: http://localhost:8080/
- **Web Client**: http://localhost:8080/client.html
- **Flutter Configurator**: http://localhost:8080/configurator
- **User Manual**: http://localhost:8080/user-manual

### Database Support
- PostgreSQL (recommended for production)
- MySQL/MariaDB (legacy compatibility)
- SQLite (development/testing)

### Multi-Protocol Support
- **Second Life Viewers**: Port 9000 (LLUDP)
- **WebSocket**: Port 9001 (Web browsers)
- **Web Interface**: Port 8080 (HTTP)
- **Monitoring**: Port 9100 (Prometheus)

## Getting Help

If you need assistance:
1. Check the troubleshooting section
2. Review the configuration examples
3. Visit the GitHub repository
4. Join the community Discord

---

*This is a fallback manual. The complete USER_MANUAL.md should be located in the project root.*
"#;
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                fallback_content.to_string(),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_client_server_creation() {
        let server = WebClientServer::new(8080);
        assert_eq!(server.port, 8080);
    }
}

/// API proxy handlers to avoid CORS issues
async fn api_health_proxy() -> impl IntoResponse {
    match reqwest::Client::new()
        .get("http://localhost:9100/health")
        .header("Authorization", "Bearer default-key-change-me")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json")],
                    text,
                )
                    .into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read response").into_response()
            }
        }
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Monitoring service unavailable",
        )
            .into_response(),
    }
}

async fn api_metrics_proxy() -> impl IntoResponse {
    match reqwest::Client::new()
        .get("http://localhost:9100/metrics")
        .header("Authorization", "Bearer default-key-change-me")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                (StatusCode::OK, [(header::CONTENT_TYPE, "text/plain")], text).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read response").into_response()
            }
        }
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Monitoring service unavailable",
        )
            .into_response(),
    }
}

async fn api_info_proxy() -> impl IntoResponse {
    match reqwest::Client::new()
        .get("http://localhost:9100/info")
        .header("Authorization", "Bearer default-key-change-me")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json")],
                    text,
                )
                    .into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read response").into_response()
            }
        }
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Monitoring service unavailable",
        )
            .into_response(),
    }
}
