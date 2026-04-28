//! Knowledge base system for OpenSim community
//!
//! Provides comprehensive documentation and FAQ system with:
//! - Structured articles and guides
//! - Search and categorization
//! - Version management
//! - User contributions
//! - Analytics and feedback

use super::{CommunityConfig, ComponentHealth};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Knowledge base main structure
pub struct KnowledgeBase {
    config: CommunityConfig,
    articles: Arc<RwLock<HashMap<String, Article>>>,
    categories: Arc<RwLock<HashMap<String, KBCategory>>>,
    search_index: Arc<RwLock<SearchIndex>>,
    analytics: Arc<RwLock<KBAnalytics>>,
    feedback_system: Arc<RwLock<FeedbackSystem>>,
}

impl KnowledgeBase {
    /// Create new knowledge base
    pub async fn new(config: CommunityConfig) -> Result<Self> {
        let articles = Arc::new(RwLock::new(HashMap::new()));
        let categories = Arc::new(RwLock::new(HashMap::new()));
        let search_index = Arc::new(RwLock::new(SearchIndex::new()));
        let analytics = Arc::new(RwLock::new(KBAnalytics::new()));
        let feedback_system = Arc::new(RwLock::new(FeedbackSystem::new()));

        Ok(Self {
            config,
            articles,
            categories,
            search_index,
            analytics,
            feedback_system,
        })
    }

    /// Initialize the knowledge base
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing knowledge base");

        // Create default categories and articles
        self.create_default_structure().await?;

        // Initialize search index
        self.search_index.write().await.initialize().await?;

        // Initialize analytics
        self.analytics.write().await.initialize().await?;

        tracing::info!("Knowledge base initialized successfully");
        Ok(())
    }

    /// Create default knowledge base structure
    async fn create_default_structure(&self) -> Result<()> {
        // Create categories
        self.create_default_categories().await?;

        // Create initial articles
        self.create_default_articles().await?;

        Ok(())
    }

    /// Create default categories
    async fn create_default_categories(&self) -> Result<()> {
        let mut categories = self.categories.write().await;

        categories.insert(
            "getting-started".to_string(),
            KBCategory {
                id: "getting-started".to_string(),
                name: "Getting Started".to_string(),
                description: "Essential guides for new users".to_string(),
                icon: "🚀".to_string(),
                sort_order: 1,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        categories.insert(
            "api-reference".to_string(),
            KBCategory {
                id: "api-reference".to_string(),
                name: "API Reference".to_string(),
                description: "Complete API documentation".to_string(),
                icon: "📚".to_string(),
                sort_order: 2,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        categories.insert(
            "tutorials".to_string(),
            KBCategory {
                id: "tutorials".to_string(),
                name: "Tutorials".to_string(),
                description: "Step-by-step tutorials and guides".to_string(),
                icon: "🎓".to_string(),
                sort_order: 3,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        categories.insert(
            "troubleshooting".to_string(),
            KBCategory {
                id: "troubleshooting".to_string(),
                name: "Troubleshooting".to_string(),
                description: "Common issues and solutions".to_string(),
                icon: "🔧".to_string(),
                sort_order: 4,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        categories.insert(
            "best-practices".to_string(),
            KBCategory {
                id: "best-practices".to_string(),
                name: "Best Practices".to_string(),
                description: "Recommended patterns and practices".to_string(),
                icon: "⭐".to_string(),
                sort_order: 5,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        categories.insert(
            "faq".to_string(),
            KBCategory {
                id: "faq".to_string(),
                name: "FAQ".to_string(),
                description: "Frequently asked questions".to_string(),
                icon: "❓".to_string(),
                sort_order: 6,
                article_count: 0,
                parent_id: None,
                subcategories: Vec::new(),
                created_at: get_current_timestamp(),
            },
        );

        Ok(())
    }

    /// Create default articles
    async fn create_default_articles(&self) -> Result<()> {
        let mut articles = self.articles.write().await;

        // Getting Started articles
        articles.insert(
            "quick-start".to_string(),
            Article {
                id: "quick-start".to_string(),
                title: "Quick Start Guide".to_string(),
                summary: "Get up and running with OpenSim in 5 minutes".to_string(),
                content: self.generate_quick_start_content(),
                category_id: "getting-started".to_string(),
                author_id: "system".to_string(),
                author_name: "OpenSim Team".to_string(),
                status: ArticleStatus::Published,
                version: 1,
                created_at: get_current_timestamp(),
                updated_at: get_current_timestamp(),
                published_at: Some(get_current_timestamp()),
                view_count: 0,
                like_count: 0,
                tags: vec![
                    "installation".to_string(),
                    "setup".to_string(),
                    "beginner".to_string(),
                ],
                related_articles: Vec::new(),
                attachments: Vec::new(),
                change_log: Vec::new(),
            },
        );

        articles.insert(
            "system-requirements".to_string(),
            Article {
                id: "system-requirements".to_string(),
                title: "System Requirements".to_string(),
                summary: "Hardware and software requirements for OpenSim".to_string(),
                content: self.generate_system_requirements_content(),
                category_id: "getting-started".to_string(),
                author_id: "system".to_string(),
                author_name: "OpenSim Team".to_string(),
                status: ArticleStatus::Published,
                version: 1,
                created_at: get_current_timestamp(),
                updated_at: get_current_timestamp(),
                published_at: Some(get_current_timestamp()),
                view_count: 0,
                like_count: 0,
                tags: vec![
                    "requirements".to_string(),
                    "hardware".to_string(),
                    "system".to_string(),
                ],
                related_articles: vec!["quick-start".to_string()],
                attachments: Vec::new(),
                change_log: Vec::new(),
            },
        );

        // FAQ articles
        articles.insert(
            "common-questions".to_string(),
            Article {
                id: "common-questions".to_string(),
                title: "Common Questions".to_string(),
                summary: "Answers to the most frequently asked questions".to_string(),
                content: self.generate_faq_content(),
                category_id: "faq".to_string(),
                author_id: "system".to_string(),
                author_name: "OpenSim Team".to_string(),
                status: ArticleStatus::Published,
                version: 1,
                created_at: get_current_timestamp(),
                updated_at: get_current_timestamp(),
                published_at: Some(get_current_timestamp()),
                view_count: 0,
                like_count: 0,
                tags: vec![
                    "faq".to_string(),
                    "questions".to_string(),
                    "help".to_string(),
                ],
                related_articles: Vec::new(),
                attachments: Vec::new(),
                change_log: Vec::new(),
            },
        );

        // Troubleshooting articles
        articles.insert(
            "installation-issues".to_string(),
            Article {
                id: "installation-issues".to_string(),
                title: "Installation Issues".to_string(),
                summary: "Common installation problems and solutions".to_string(),
                content: self.generate_troubleshooting_content(),
                category_id: "troubleshooting".to_string(),
                author_id: "system".to_string(),
                author_name: "OpenSim Team".to_string(),
                status: ArticleStatus::Published,
                version: 1,
                created_at: get_current_timestamp(),
                updated_at: get_current_timestamp(),
                published_at: Some(get_current_timestamp()),
                view_count: 0,
                like_count: 0,
                tags: vec![
                    "installation".to_string(),
                    "troubleshooting".to_string(),
                    "issues".to_string(),
                ],
                related_articles: vec![
                    "quick-start".to_string(),
                    "system-requirements".to_string(),
                ],
                attachments: Vec::new(),
                change_log: Vec::new(),
            },
        );

        // Update category article counts
        self.update_category_counts().await?;

        Ok(())
    }

    /// Update category article counts
    async fn update_category_counts(&self) -> Result<()> {
        let articles = self.articles.read().await;
        let mut categories = self.categories.write().await;

        // Reset counts
        for category in categories.values_mut() {
            category.article_count = 0;
        }

        // Count articles per category
        for article in articles.values() {
            if let Some(category) = categories.get_mut(&article.category_id) {
                category.article_count += 1;
            }
        }

        Ok(())
    }

    /// Create a new article
    pub async fn create_article(&self, article_data: CreateArticleRequest) -> Result<Article> {
        let article_id = generate_id();
        let article = Article {
            id: article_id.clone(),
            title: article_data.title,
            summary: article_data.summary,
            content: article_data.content,
            category_id: article_data.category_id.clone(),
            author_id: article_data.author_id.clone(),
            author_name: article_data.author_name.clone(),
            status: ArticleStatus::Draft,
            version: 1,
            created_at: get_current_timestamp(),
            updated_at: get_current_timestamp(),
            published_at: None,
            view_count: 0,
            like_count: 0,
            tags: article_data.tags.unwrap_or_default(),
            related_articles: Vec::new(),
            attachments: Vec::new(),
            change_log: vec![ChangeLogEntry {
                version: 1,
                author_name: article_data.author_name,
                changes: "Initial creation".to_string(),
                timestamp: get_current_timestamp(),
            }],
        };

        // Add the article
        self.articles
            .write()
            .await
            .insert(article_id.clone(), article.clone());

        // Update search index
        self.search_index
            .write()
            .await
            .add_article(&article)
            .await?;

        // Update category count if published
        if article.status == ArticleStatus::Published {
            if let Some(category) = self
                .categories
                .write()
                .await
                .get_mut(&article_data.category_id)
            {
                category.article_count += 1;
            }
        }

        Ok(article)
    }

    /// Search knowledge base
    pub async fn search(&self, query: &str, filters: KBSearchFilters) -> Result<KBSearchResults> {
        self.search_index.read().await.search(query, filters).await
    }

    /// Get knowledge base statistics
    pub async fn get_stats(&self) -> Result<KBStats> {
        let articles = self.articles.read().await;
        let categories = self.categories.read().await;
        let analytics = self.analytics.read().await;

        let total_articles = articles.len() as u64;
        let published_articles = articles
            .values()
            .filter(|a| a.status == ArticleStatus::Published)
            .count() as u64;

        let total_views = articles.values().map(|a| a.view_count).sum();
        let total_likes = articles.values().map(|a| a.like_count).sum();

        Ok(KBStats {
            total_articles,
            published_articles,
            total_categories: categories.len() as u64,
            total_views,
            total_likes,
            articles_created_today: 0, // TODO: Implement
            popular_articles: analytics.get_popular_articles(),
        })
    }

    /// Get knowledge base health status
    pub async fn health_check(&self) -> Result<ComponentHealth> {
        let start_time = SystemTime::now();

        // Test all components
        let _articles_count = self.articles.read().await.len();
        let _categories_count = self.categories.read().await.len();
        let _search_healthy = self.search_index.read().await.is_healthy();

        let response_time = start_time.elapsed().unwrap().as_millis() as u64;

        Ok(ComponentHealth {
            status: "healthy".to_string(),
            response_time_ms: response_time,
            last_error: None,
        })
    }

    /// Generate content for articles
    fn generate_quick_start_content(&self) -> String {
        r#"# Quick Start Guide

Welcome to OpenSim! This guide will help you get started quickly.

## Prerequisites

Before you begin, ensure you have:
- A supported operating system (Windows 10+, macOS 10.15+, or Linux)
- At least 4GB of RAM
- Internet connection

## Installation

### Step 1: Download OpenSim

Choose your preferred installation method:

**Option A: Package Manager**
```bash
# Rust
cargo install opensim-client

# Python
pip install opensim-client

# Node.js
npm install opensim-client
```

**Option B: Binary Download**
Download the latest release from our [releases page](https://github.com/opensim/opensim-next/releases).

### Step 2: Verify Installation

```bash
opensim --version
opensim health-check
```

### Step 3: Create Your First Project

```bash
opensim new my-project
cd my-project
opensim run
```

## Next Steps

- [API Documentation](../api-reference/overview)
- [Tutorials](../tutorials/)
- [Join the Community](https://community.opensim.org)

## Need Help?

- Check our [FAQ](../faq/common-questions)
- Visit the [troubleshooting guide](../troubleshooting/installation-issues)
- Ask questions in our [forums](https://community.opensim.org/forums)
"#.to_string()
    }

    fn generate_system_requirements_content(&self) -> String {
        r#"# System Requirements

## Minimum Requirements

### Operating System
- **Windows**: Windows 10 (64-bit) or later
- **macOS**: macOS 10.15 (Catalina) or later
- **Linux**: Ubuntu 20.04 LTS or equivalent

### Hardware
- **CPU**: 2 GHz dual-core processor
- **Memory**: 4 GB RAM
- **Storage**: 2 GB available space
- **Network**: Broadband internet connection

## Recommended Requirements

### Hardware
- **CPU**: 3 GHz quad-core processor or better
- **Memory**: 8 GB RAM or more
- **Storage**: 10 GB available space (SSD recommended)
- **Graphics**: Dedicated graphics card for 3D applications
- **Network**: High-speed internet connection

## Software Dependencies

### Development Tools
- **Rust**: Version 1.70 or later (for Rust development)
- **Python**: Version 3.8 or later (for Python development)
- **Node.js**: Version 16 or later (for JavaScript development)

### Runtime Dependencies
- **OpenSSL**: For secure connections
- **SQLite**: For local data storage
- **Redis**: For caching (optional but recommended)

## Platform-Specific Notes

### Windows
- Visual Studio Build Tools may be required for some features
- Windows Defender exclusions recommended for better performance

### macOS
- Xcode Command Line Tools required
- May require additional permissions for network access

### Linux
- Package manager dependencies may vary by distribution
- Some distributions may require additional development packages

## Cloud Deployment

### Supported Platforms
- **AWS**: EC2, ECS, Lambda
- **Google Cloud**: Compute Engine, Cloud Run
- **Azure**: Virtual Machines, Container Instances
- **Docker**: All major container platforms

### Resource Recommendations
- **Small deployment**: 2 vCPUs, 4 GB RAM
- **Medium deployment**: 4 vCPUs, 8 GB RAM
- **Large deployment**: 8+ vCPUs, 16+ GB RAM

## Performance Considerations

- SSD storage significantly improves performance
- More RAM allows for larger worlds and more concurrent users
- Network latency affects real-time features
- Load balancing recommended for production deployments
"#
        .to_string()
    }

    fn generate_faq_content(&self) -> String {
        r#"# Frequently Asked Questions

## General Questions

### What is OpenSim?
OpenSim is an open-source virtual world platform that allows you to create immersive 3D environments and applications.

### Is OpenSim free to use?
Yes, OpenSim is completely free and open-source under the MIT license.

### What programming languages are supported?
We provide official SDKs for:
- Rust
- Python
- JavaScript/TypeScript
- C#
- Java
- Go
- PHP
- Ruby

## Installation & Setup

### How do I install OpenSim?
See our [Quick Start Guide](../getting-started/quick-start) for detailed installation instructions.

### What are the system requirements?
Check our [System Requirements](../getting-started/system-requirements) page for detailed specifications.

### Can I run OpenSim on a Raspberry Pi?
Yes, but performance may be limited. We recommend at least a Raspberry Pi 4 with 4GB RAM.

## Development

### How do I create my first application?
Follow our [First Steps Guide](../tutorials/first-application) for a step-by-step walkthrough.

### Where can I find code examples?
Browse our [examples repository](https://github.com/opensim/examples) for complete example projects.

### How do I contribute to OpenSim?
See our [contribution guidelines](https://github.com/opensim/opensim-next/blob/main/CONTRIBUTING.md) for information on how to contribute.

## Troubleshooting

### OpenSim won't start - what should I check?
1. Verify system requirements are met
2. Check if required ports are available
3. Review the error logs
4. See our [troubleshooting guide](../troubleshooting/installation-issues)

### How do I report a bug?
Please report bugs on our [GitHub issues page](https://github.com/opensim/opensim-next/issues).

### Where can I get help?
- [Community Forums](https://community.opensim.org/forums)
- [Discord Server](https://discord.gg/opensim)
- [Support Email](mailto:support@opensim.org)

## Community

### How can I stay updated on OpenSim news?
- Follow our [blog](https://blog.opensim.org)
- Join our [newsletter](https://opensim.org/newsletter)
- Follow us on [Twitter](https://twitter.com/opensim)

### Can I contribute documentation?
Yes! Documentation contributions are very welcome. See our [documentation guidelines](../contributing/documentation) for details.
"#.to_string()
    }

    fn generate_troubleshooting_content(&self) -> String {
        r#"# Installation Issues

Common installation problems and their solutions.

## Permission Errors

### Windows
**Error**: "Access denied" or "Permission denied"

**Solution**:
1. Run Command Prompt as Administrator
2. Ensure antivirus software isn't blocking the installation
3. Add OpenSim to Windows Defender exclusions

### macOS
**Error**: "Permission denied" or "Operation not permitted"

**Solution**:
1. Use `sudo` for system-wide installations
2. Grant necessary permissions in System Preferences > Security & Privacy
3. For Apple Silicon Macs, ensure ARM64 compatibility

### Linux
**Error**: Permission or dependency errors

**Solution**:
```bash
# Update package lists
sudo apt update

# Install required dependencies
sudo apt install build-essential curl pkg-config libssl-dev

# Use sudo for system-wide installation
sudo cargo install opensim-client
```

## Network Issues

### Port Conflicts
**Error**: "Port already in use" or "Address already in use"

**Solution**:
1. Check which process is using the port:
   ```bash
   # Windows
   netstat -ano | findstr :9000
   
   # macOS/Linux
   lsof -i :9000
   ```
2. Stop the conflicting process or change OpenSim's port
3. Configure firewall to allow OpenSim ports

### Firewall Blocking
**Error**: Connection timeouts or "Connection refused"

**Solution**:
1. **Windows**: Add OpenSim to Windows Firewall exceptions
2. **macOS**: Allow OpenSim in Security & Privacy > Firewall
3. **Linux**: Configure iptables or ufw to allow OpenSim ports

## Build Errors

### Rust Compilation Issues
**Error**: Rust compiler errors or missing dependencies

**Solution**:
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Install missing system dependencies
# Ubuntu/Debian:
sudo apt install pkg-config libssl-dev

# macOS:
brew install pkg-config openssl
```

### Python Package Issues
**Error**: "No module named" or pip installation failures

**Solution**:
```bash
# Upgrade pip
python -m pip install --upgrade pip

# Use virtual environment
python -m venv opensim-env
source opensim-env/bin/activate  # Linux/macOS
opensim-env\Scripts\activate     # Windows

# Install with specific Python version
python3.8 -m pip install opensim-client
```

## Platform-Specific Issues

### Windows Subsystem for Linux (WSL)
**Issue**: Performance problems or GUI issues

**Solution**:
1. Use WSL 2 for better performance
2. Install VcXsrv for GUI support
3. Configure display forwarding:
   ```bash
   export DISPLAY=:0
   ```

### Docker Issues
**Error**: Container startup failures

**Solution**:
```bash
# Pull latest image
docker pull opensim/opensim-next:latest

# Run with proper port mapping
docker run -p 9000:9000 -p 9100:9100 opensim/opensim-next

# Check container logs
docker logs <container-id>
```

## Performance Issues

### High Memory Usage
**Symptoms**: System becomes slow, out of memory errors

**Solutions**:
1. Increase system RAM or swap space
2. Reduce world complexity
3. Enable caching to reduce memory pressure
4. Use production-optimized builds

### Slow Startup
**Symptoms**: OpenSim takes a long time to start

**Solutions**:
1. Use SSD storage for better I/O performance
2. Disable unnecessary features during development
3. Optimize database queries
4. Use connection pooling

## Getting Additional Help

If these solutions don't resolve your issue:

1. **Check the logs**: Look for error messages in OpenSim logs
2. **Search existing issues**: Check our [GitHub issues](https://github.com/opensim/opensim-next/issues)
3. **Ask the community**: Post in our [forums](https://community.opensim.org/forums)
4. **Contact support**: Email [support@opensim.org](mailto:support@opensim.org)

### Information to Include When Asking for Help

- Operating system and version
- OpenSim version
- Complete error messages
- Steps to reproduce the issue
- Relevant log files
"#.to_string()
    }
}

// Supporting structures and implementations...

/// Knowledge base article structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub category_id: String,
    pub author_id: String,
    pub author_name: String,
    pub status: ArticleStatus,
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub published_at: Option<u64>,
    pub view_count: u64,
    pub like_count: u64,
    pub tags: Vec<String>,
    pub related_articles: Vec<String>,
    pub attachments: Vec<Attachment>,
    pub change_log: Vec<ChangeLogEntry>,
}

/// Knowledge base category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KBCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub sort_order: i32,
    pub article_count: u64,
    pub parent_id: Option<String>,
    pub subcategories: Vec<String>,
    pub created_at: u64,
}

/// Article status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArticleStatus {
    Draft,
    Review,
    Published,
    Archived,
}

/// Article attachment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub mime_type: String,
}

/// Change log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeLogEntry {
    pub version: u32,
    pub author_name: String,
    pub changes: String,
    pub timestamp: u64,
}

/// Request for creating articles
#[derive(Debug, Deserialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub summary: String,
    pub content: String,
    pub category_id: String,
    pub author_id: String,
    pub author_name: String,
    pub tags: Option<Vec<String>>,
}

/// Knowledge base statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct KBStats {
    pub total_articles: u64,
    pub published_articles: u64,
    pub total_categories: u64,
    pub total_views: u64,
    pub total_likes: u64,
    pub articles_created_today: u64,
    pub popular_articles: Vec<(String, u64)>,
}

// Supporting systems...

/// Search index for knowledge base
pub struct SearchIndex {
    // Search implementation would go here
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn add_article(&mut self, _article: &Article) -> Result<()> {
        Ok(())
    }

    pub async fn search(&self, _query: &str, _filters: KBSearchFilters) -> Result<KBSearchResults> {
        Ok(KBSearchResults {
            articles: Vec::new(),
            total_results: 0,
            page: 1,
            per_page: 20,
        })
    }

    pub fn is_healthy(&self) -> bool {
        true
    }
}

/// Knowledge base analytics
pub struct KBAnalytics {
    popular_articles: Vec<(String, u64)>,
}

impl KBAnalytics {
    pub fn new() -> Self {
        Self {
            popular_articles: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn get_popular_articles(&self) -> Vec<(String, u64)> {
        self.popular_articles.clone()
    }
}

/// Feedback system for articles
pub struct FeedbackSystem {
    // Feedback implementation would go here
}

impl FeedbackSystem {
    pub fn new() -> Self {
        Self {}
    }
}

/// Search filters for knowledge base
#[derive(Debug, Deserialize)]
pub struct KBSearchFilters {
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<ArticleStatus>,
    pub author_id: Option<String>,
}

/// Search results for knowledge base
#[derive(Debug, Serialize)]
pub struct KBSearchResults {
    pub articles: Vec<Article>,
    pub total_results: u64,
    pub page: u32,
    pub per_page: u32,
}

/// Utility functions
fn generate_id() -> String {
    format!("kb_{}", get_current_timestamp())
}

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
