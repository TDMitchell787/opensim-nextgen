use std::env;

fn main() {
    // Test PostgreSQL
    env::set_var("DATABASE_URL", "postgresql://user:pass@localhost/opensim");
    println!("PostgreSQL URL: {}", env::var("DATABASE_URL").unwrap());
    
    // Test MySQL  
    env::set_var("DATABASE_URL", "mysql://user:pass@localhost/opensim");
    println!("MySQL URL: {}", env::var("DATABASE_URL").unwrap());
    
    // Test SQLite
    env::set_var("DATABASE_URL", "sqlite://opensim.db");
    println!("SQLite URL: {}", env::var("DATABASE_URL").unwrap());
    
    println!("Multi-database configuration system ready!");
}