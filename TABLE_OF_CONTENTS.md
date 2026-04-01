# OpenSim Next - Documentation Table of Contents

*Complete guide organized by development phases: Core OpenSim Port vs Enhanced Features*

---

## 🚀 Quick Start & Essential Documentation

### **[CLAUDE.md](CLAUDE.md)**
**Primary development guide** - Essential information for developers including technology stack, build instructions, and development guidelines. Start here for development work.

### **[USER_MANUAL.md](USER_MANUAL.md)**
**Complete deployment guide** - Comprehensive manual for installation, configuration, and operation. Essential for administrators and end users.

### **[ARCHITECTURE_GLOSSARY.md](ARCHITECTURE_GLOSSARY.md)**
**Complete system overview** - Concise descriptions of ALL architectural components (both core OpenSim port and enhanced features). Universal reference for understanding the entire codebase.

---

## 📦 CORE OPENSIM PORT (Phases 1-28)
*Essential functionality migration from OpenSim-master - Production-ready virtual world platform*

### **Foundation Documentation**

#### **[ROADMAP.rules.md](ROADMAP.rules.md)** - Phases 1-28
**Core development progress** - Systematic port completion including networking, physics, database, avatar systems, and basic virtual world functionality.

#### **[docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md)**
**Legacy OpenSim transition** - Complete guide for migrating from existing OpenSim installations to OpenSim Next.

#### **[docs/QUICK_START_GUIDE.md](docs/QUICK_START_GUIDE.md)**
**Fast core setup** - Rapid deployment of essential OpenSim functionality.

### **Core System Documentation**

#### **[docs/REST_API_GUIDE.md](docs/REST_API_GUIDE.md)**
**Core API endpoints** - Essential REST API for basic virtual world operations and legacy compatibility.

#### **[docs/WEBSOCKET_WEB_CLIENT_GUIDE.md](docs/WEBSOCKET_WEB_CLIENT_GUIDE.md)**
**Web client integration** - WebSocket protocol for browser-based access to virtual worlds.

#### **[docs/SECURITY_HARDENING_GUIDE.md](docs/SECURITY_HARDENING_GUIDE.md)**
**Production security** - Essential security configuration for production virtual world deployment.

#### **Reference: [ARCHITECTURE_GLOSSARY.md](ARCHITECTURE_GLOSSARY.md)**
**Core systems reference** - See sections: Network Layer, Physics Engine, Database Layer, Region Management, Avatar System, OpenSim Compatibility.

### **Core Testing & Validation**

#### **[rust/tools/phase28-testing/](rust/tools/phase28-testing/)**
**Core functionality testing** - Comprehensive test suite validating essential OpenSim port completion through Phase 28.

#### **[Phase_26_Schema_Audit_Report.md](Phase_26_Schema_Audit_Report.md)**
**Database foundation audit** - Core database schema validation and optimization for production readiness.

---

## ⚡ ENHANCED FEATURES (Phase 29+)
*Next-generation virtual world capabilities - Advanced features beyond core OpenSim*

### **Enhanced Platform Documentation**

#### **[ROADMAP.rules.md](ROADMAP.rules.md)** - Phases 29+
**Advanced feature development** - Business intelligence, mobile integration, AI capabilities, and enterprise-grade enhancements.

#### **[Phase 24.md](Phase%2024.md)** - Analytics & Reporting System
**Business intelligence platform** - Advanced analytics, predictive modeling, and automated reporting capabilities.

#### **[Phase 25.md](Phase%2025.md)** - Advanced System Features
**Extended functionality** - Enhanced capabilities and system optimizations beyond core OpenSim.

#### **[Phase 26.md](Phase%2026.md)** - Performance & Optimization
**Enterprise-grade performance** - Advanced optimization and scalability features.

#### **[Phase 27.md](Phase%2027.md)** - Latest Enhancements
**Cutting-edge features** - Most recent advanced development achievements.

### **Advanced System Operations**

#### **[docs/MONITORING_ADMINISTRATION_GUIDE.md](docs/MONITORING_ADMINISTRATION_GUIDE.md)**
**Enterprise monitoring** - Advanced system monitoring, analytics, and business intelligence administration.

#### **[docs/REMOTE_ADMIN_GUIDE.md](docs/REMOTE_ADMIN_GUIDE.md)**
**Advanced administration** - Enterprise-grade remote management capabilities.

#### **Reference: [ARCHITECTURE_GLOSSARY.md](ARCHITECTURE_GLOSSARY.md)**
**Enhanced systems reference** - See sections: Mobile Integration, Reporting & Analytics, AI Integration, Security Framework, Performance Systems, PWA Integration.

### **Modern Interface & Mobile**

#### **[auto-configurator/](auto-configurator/)**
**Progressive Web App configurator** - Modern web-based configuration and management interface with mobile support.

#### **[flutter-client/](flutter-client/)**
**Cross-platform management UI** - Flutter-based configuration interfaces for desktop, web, and mobile.

#### **[rust/web-frontend/](rust/web-frontend/)**
**Advanced web interface** - Modern browser-based access with enhanced capabilities.

---

## 🚢 DEPLOYMENT & OPERATIONS
*Production deployment for both core and enhanced features*

### **[deploy/](deploy/)**
**Complete deployment infrastructure**

#### **Core Deployment** (Phases 1-28)
- **[docker/](deploy/docker/)** - Basic containerization for core OpenSim functionality
- **[kubernetes/manifests/](deploy/kubernetes/manifests/)** - Essential Kubernetes deployment
- **[scripts/](deploy/scripts/)** - Core deployment and validation scripts

#### **Enhanced Deployment** (Phase 29+)
- **[deploy/PHASE-29.5-SUMMARY.md](deploy/PHASE-29.5-SUMMARY.md)** - Advanced deployment features
- **[kubernetes/helm/](deploy/kubernetes/helm/)** - Advanced Helm charts with enterprise features
- **[terraform/](deploy/terraform/)** - Infrastructure as Code for enterprise deployment
- **[cicd/](deploy/cicd/)** - Advanced CI/CD pipelines for multiple platforms
- **[monitoring/](deploy/monitoring/)** - Enterprise monitoring stack (Prometheus, Grafana, Alertmanager)

---

## 🛠️ DEVELOPMENT & METHODOLOGY

### **[SDS-EADS.md](SDS-EADS.md)**
**Development methodology** - Systematic Development Standards and Elegant Archive Development System used throughout all phases.

### **[tools/git-management/](tools/git-management/)**
**Development workflow** - OGMS (OpenSim Git Management System) for systematic development across all phases.

### **[file_line_counts.txt](file_line_counts.txt)**
**Project metrics** - Complete codebase analysis showing growth from core port through enhanced features.

---

## 📊 MIGRATION & ADOPTION STRATEGY

### **Core OpenSim Replacement Path** (Phases 1-28)
1. **[docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md)** - Legacy migration
2. **[docs/QUICK_START_GUIDE.md](docs/QUICK_START_GUIDE.md)** - Core deployment
3. **[docs/SECURITY_HARDENING_GUIDE.md](docs/SECURITY_HARDENING_GUIDE.md)** - Production hardening
4. **[rust/tools/phase28-testing/](rust/tools/phase28-testing/)** - Validation testing

### **Enhanced Features Adoption Path** (Phase 29+)
1. **[Phase 24.md](Phase%2024.md)** - Add business intelligence
2. **[auto-configurator/](auto-configurator/)** - Deploy modern web interface
3. **[deploy/monitoring/](deploy/monitoring/)** - Enable enterprise monitoring
4. **[flutter-client/](flutter-client/)** - Add cross-platform management

---

## 🎯 DOCUMENTATION BY AUDIENCE

### **OpenSim Administrators** (Core Replacement)
- `USER_MANUAL.md` → `docs/MIGRATION_GUIDE.md` → `docs/SECURITY_HARDENING_GUIDE.md`
- Focus: Phases 1-28 core functionality

### **Enterprise Operators** (Enhanced Features)  
- `USER_MANUAL.md` → `Phase 24.md` → `docs/MONITORING_ADMINISTRATION_GUIDE.md`
- Focus: Phase 29+ business intelligence and analytics

### **Developers** (Core Contribution)
- `CLAUDE.md` → `ARCHITECTURE_GLOSSARY.md` → `SDS-EADS.md`
- Focus: Core OpenSim port development

### **Advanced Developers** (Feature Enhancement)
- `CLAUDE.md` → Phase documentation → modern interface development
- Focus: Next-generation virtual world features

---

*The clear separation between **Core OpenSim Port** (Phases 1-28) and **Enhanced Features** (Phase 29+) helps organizations choose their adoption path based on needs: essential virtual world functionality vs cutting-edge enterprise capabilities.*