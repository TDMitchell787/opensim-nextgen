// Test database type detection logic

fn from_url(url: &str) -> Result<String, String> {
    if url.starts_with("postgresql://") || url.starts_with("postgres://") {
        Ok("PostgreSQL".to_string())
    } else if url.starts_with("mysql://") {
        Ok("MySQL".to_string())
    } else if url.starts_with("mariadb://") {
        Ok("MariaDB".to_string())
    } else if url.starts_with("sqlite://") {
        Ok("SQLite".to_string())
    } else {
        Err(format!("Unsupported database URL format: {}", url))
    }
}

fn main() {
    let test_urls = vec![
        "postgresql://localhost/opensim",
        "postgres://localhost/opensim",
        "mysql://localhost/opensim",
        "mariadb://localhost/opensim",
        "sqlite://opensim.db",
        "redis://localhost/opensim", // Should fail
    ];

    for url in test_urls {
        match from_url(url) {
            Ok(db_type) => println!("✅ {} -> {}", url, db_type),
            Err(e) => println!("❌ {} -> {}", url, e),
        }
    }
}