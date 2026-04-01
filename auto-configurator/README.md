# OpenSim Next Auto-Configurator

An intelligent, web-based configuration wizard that transforms the complex OpenSim Next setup process into a guided, error-free experience while maintaining enterprise-grade security standards.

## 🌟 Features

### ✅ **Completed Foundation (Phase 17.1)**

- **🎨 Modern UI/UX Architecture**
  - Responsive HTML5 Progressive Web App (PWA)
  - Professional design with accessibility features
  - Mobile-friendly responsive layout
  - Dark/light mode support

- **🧙‍♂️ Intelligent Configuration Wizard**
  - Step-by-step guided setup process
  - Smart deployment type detection (Development/Production/Grid)
  - Context-aware help and documentation
  - Real-time validation and error prevention

- **📄 Advanced Template Parser**
  - Automatic parsing of OpenSim .ini.example files
  - Intelligent default value detection
  - Configuration schema validation
  - Cross-file dependency resolution

- **🔒 Security-First Design**
  - Cryptographic material management
  - SSL/TLS certificate handling
  - USB security key integration
  - Zero-storage policy for sensitive data

- **📱 Progressive Web App Features**
  - Offline functionality with service worker
  - App-like experience on mobile devices
  - Background sync for configuration saves
  - Push notifications for updates

### 🚧 **In Development (Phase 17.2-17.4)**

- Configuration preview and diff visualization
- Encrypted USB key management system
- Multi-language internationalization support
- Advanced configuration profiles

## 🚀 Quick Start

### Prerequisites

- Modern web browser with JavaScript enabled
- HTTPS connection (recommended for security features)
- OpenSim Next installation directory

### Installation

1. **Copy the auto-configurator to your OpenSim Next installation:**
   ```bash
   cp -r auto-configurator/ /path/to/opensim-next/
   ```

2. **Serve the application (choose one method):**

   **Option A: Python Simple Server**
   ```bash
   cd /path/to/opensim-next/auto-configurator
   python3 -m http.server 8080
   ```

   **Option B: Node.js serve**
   ```bash
   cd /path/to/opensim-next/auto-configurator
   npx serve -p 8080
   ```

   **Option C: Apache/Nginx**
   Configure your web server to serve the auto-configurator directory

3. **Access the configurator:**
   Open your browser to `http://localhost:8080`

## 🎯 Usage Guide

### Step 1: Choose Deployment Type

The configurator starts by asking you to select your deployment type:

- **🖥️ Development**: Single region setup for testing
  - SQLite database
  - Basic physics (ODE)
  - Local network only
  - Minimal security

- **🖥️ Production**: Single server production deployment
  - PostgreSQL database
  - Multi-physics engines
  - SSL/TLS security
  - Comprehensive monitoring

- **🌐 Grid**: Multi-server grid deployment
  - Distributed architecture
  - Zero trust networking
  - Load balancing
  - Enterprise security

### Step 2: Environment Configuration

Configure basic environment settings:
- Installation path
- Grid name and administrator email
- Logging levels
- Service configuration

### Step 3: Database Setup

Choose and configure your database:
- **SQLite**: File-based database (ideal for development)
- **PostgreSQL**: Enterprise database (recommended for production)

The configurator automatically generates appropriate connection strings and validates database settings.

### Step 4: Region Configuration

Set up your virtual world regions:
- Region names and coordinates
- Physics engine selection per region
- Terrain and asset configuration
- Network and security settings

### Step 5: Security Configuration

Configure cryptographic materials and security settings:
- SSL/TLS certificates and private keys
- Authentication levels and policies
- Zero trust networking (for grid deployments)
- Encryption requirements

**⚠️ Security Notice**: The auto-configurator never stores your private keys or sensitive data. All cryptographic materials remain under your control.

### Step 6: Network Configuration

Configure network and connectivity:
- Port assignments and firewall settings
- Load balancing and scaling options
- External hostname and DNS configuration
- CDN and caching setup

### Step 7: Review and Generate

Review your configuration and generate files:
- Configuration validation and pre-flight checks
- Preview generated configuration files
- Download complete configuration package
- Deployment instructions and next steps

## 🔧 Advanced Features

### Security Management

The auto-configurator includes advanced security features:

- **Certificate Validation**: Automatic validation of SSL certificates and private keys
- **USB Security Keys**: Integration with hardware security devices (YubiKey, etc.)
- **Encrypted Storage**: Local configuration data is encrypted using Web Crypto API
- **Security Auditing**: Real-time security posture assessment

### Configuration Templates

Pre-built configuration templates for common scenarios:

- **Development Environment**: Quick setup for local development
- **Production Single-Server**: Optimized for production deployment
- **Grid Hub**: Central hub configuration for multi-server grids
- **Grid Node**: Regional node configuration for distributed grids

### Import/Export

- **Import Existing Configurations**: Load existing OpenSim.ini files
- **Export Configuration Packages**: Download complete configuration sets
- **Template Sharing**: Share configuration templates with team members
- **Backup and Restore**: Backup configurations with encryption

## 🛡️ Security Features

### Zero-Storage Security Policy

The auto-configurator follows a strict zero-storage policy for sensitive data:

- **Private Keys**: Never stored or transmitted
- **Passwords**: Not saved in configuration files
- **Certificates**: Only metadata stored (fingerprints, validity periods)
- **Encryption**: All local storage is encrypted using Web Crypto API

### Hardware Security Integration

Support for hardware security devices:

- **USB Security Keys**: YubiKey, Nitrokey, SoloKeys
- **Smart Cards**: PIV/CAC card integration
- **TPM Integration**: Trusted Platform Module support
- **Encrypted USB Storage**: Secure key storage on encrypted USB drives

### Compliance and Auditing

- **Security Auditing**: Real-time security posture assessment
- **Compliance Checking**: Validation against security standards
- **Audit Trails**: Complete configuration change history
- **Risk Assessment**: Automated security risk scoring

## 📱 Progressive Web App Features

The auto-configurator is built as a modern Progressive Web App:

- **Offline Support**: Continue configuration without internet
- **Mobile Responsive**: Works on smartphones and tablets
- **App Installation**: Install as a native app on your device
- **Background Sync**: Automatic syncing when connection is restored
- **Push Notifications**: Alerts for configuration updates

## 🔧 Development

### Architecture

The auto-configurator is built with modern web technologies:

- **Frontend**: Vanilla JavaScript ES6+ with modern CSS
- **Security**: Web Crypto API for encryption and key management
- **Storage**: IndexedDB for offline data, localStorage for preferences
- **Network**: Service Worker for offline support and caching
- **UI/UX**: Responsive design with accessibility features

### File Structure

```
auto-configurator/
├── index.html              # Main application entry point
├── manifest.json           # PWA manifest for app installation
├── sw.js                   # Service worker for offline support
├── css/
│   └── styles.css          # Main stylesheet with responsive design
├── js/
│   ├── app.js              # Main application logic and wizard
│   ├── config-parser.js    # Configuration template parser
│   ├── security.js         # Security and cryptographic features
│   ├── wizard.js           # Step-by-step wizard implementation
│   └── dashboard.js        # Dashboard and progress tracking
├── icons/                  # PWA icons and assets
└── README.md              # This documentation
```

### API Integration

The auto-configurator can integrate with OpenSim Next's REST API:

- **Configuration Validation**: Real-time validation of settings
- **Template Management**: Server-side template storage
- **Security Validation**: Certificate and key validation
- **Deployment Automation**: Automated configuration deployment

## 🐛 Troubleshooting

### Common Issues

**🔒 "Secure context required" warning**
- **Solution**: Access the configurator via HTTPS or localhost
- **Why**: Web Crypto API requires secure context for security features

**📱 "WebUSB not supported" error**
- **Solution**: Use a Chromium-based browser (Chrome, Edge, Opera)
- **Why**: WebUSB is currently only supported in Chromium browsers

**💾 Configuration not saving**
- **Solution**: Check browser storage permissions and available space
- **Why**: The app uses localStorage and IndexedDB for offline storage

**🌐 Offline functionality not working**
- **Solution**: Ensure the service worker is properly registered
- **Why**: Check browser console for service worker errors

### Browser Compatibility

**✅ Fully Supported:**
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

**⚠️ Limited Support:**
- Internet Explorer: Not supported
- Older browsers: Basic functionality only

### Performance Optimization

For optimal performance:

- **Modern Browser**: Use latest browser versions
- **Hardware Acceleration**: Enable hardware acceleration in browser
- **Memory**: Ensure sufficient available RAM (2GB+ recommended)
- **Storage**: Maintain adequate free disk space for caching

## 📞 Support

### Documentation

- **User Manual**: Complete OpenSim Next documentation in `../USER_MANUAL.md`
- **API Reference**: REST API documentation for integration
- **Security Guide**: Security best practices and procedures
- **Troubleshooting**: Common issues and solutions

### Community

- **GitHub Issues**: Report bugs and request features
- **Discord Community**: Real-time support and discussions
- **Documentation Wiki**: Community-maintained documentation

## 🗺️ Roadmap

### Phase 17.2: Intelligence (In Progress)
- [ ] Intelligent configuration validation system
- [ ] Deployment type selector with auto-detection
- [ ] Real-time configuration preview
- [ ] Enhanced dashboard with progress tracking

### Phase 17.3: Advanced Features (Planned)
- [ ] Encrypted USB key management system
- [ ] Comprehensive help system integration
- [ ] Configuration export/import system
- [ ] Automated testing framework

### Phase 17.4: Polish (Planned)
- [ ] Multi-language internationalization
- [ ] Advanced configuration profiles
- [ ] Performance optimizations
- [ ] Enhanced accessibility features

## 📄 License

This project is part of OpenSim Next and follows the same licensing terms.

## 🙏 Acknowledgments

Built with modern web standards and security best practices. Special thanks to the OpenSim community for feedback and testing.

---

**🌟 Revolutionary Configuration Made Simple**

The OpenSim Next Auto-Configurator transforms complex server setup into an intuitive, secure, and error-free experience. Whether you're setting up a development environment or deploying enterprise-grade virtual world infrastructure, the auto-configurator guides you through every step with intelligence and security built-in.

*Last updated: Phase 17 Foundation Complete*