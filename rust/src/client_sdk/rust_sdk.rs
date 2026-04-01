//! Rust SDK generator for OpenSim client library

use std::{collections::HashMap, path::PathBuf};
use anyhow::Result;

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema, TypeDefinition, PropertySchema, HttpMethod},
    generator::{LanguageGenerator, GeneratorConfig, GeneratedFile, GeneratedFileType, GenerationResult, PackageInfo, CodeGenUtils},
};

/// Rust-specific SDK generator
pub struct RustGenerator;

impl RustGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_data_type(&self, data_type: &DataTypeSchema) -> String {
        let mut code = String::new();
        
        // Add documentation
        code.push_str(&format!("/// {}\n", data_type.description));
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        
        match &data_type.type_definition {
            TypeDefinition::Object { properties } => {
                code.push_str(&format!("pub struct {} {{\n", data_type.name));
                
                for property in properties {
                    code.push_str(&format!("    /// {}\n", property.description));
                    
                    let field_type = self.map_type_to_rust(&property.property_type.type_name, property.property_type.nullable);
                    let field_name = CodeGenUtils::to_snake_case(&property.name);
                    
                    if property.required {
                        code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
                    } else {
                        code.push_str(&format!("    pub {}: Option<{}>,\n", field_name, field_type));
                    }
                }
                
                code.push_str("}\n\n");
            }
            TypeDefinition::Enum { variants } => {
                code.push_str(&format!("pub enum {} {{\n", data_type.name));
                
                for variant in variants {
                    code.push_str(&format!("    /// {}\n", variant.description));
                    code.push_str(&format!("    {},\n", CodeGenUtils::to_pascal_case(&variant.name)));
                }
                
                code.push_str("}\n\n");
            }
            _ => {
                // Handle other type definitions as needed
                code.push_str(&format!("pub type {} = String; // TODO: Implement specific type\n\n", data_type.name));
            }
        }
        
        code
    }

    fn generate_endpoint_method(&self, endpoint: &EndpointSchema) -> String {
        let mut code = String::new();
        
        // Method documentation
        code.push_str(&format!("    /// {}\n", endpoint.description));
        
        // Method signature
        let method_name = CodeGenUtils::to_snake_case(&endpoint.name.replace(" ", "_"));
        let return_type = self.map_type_to_rust(&endpoint.response_body.type_name, false);
        
        code.push_str(&format!("    pub async fn {}(&self", method_name));
        
        // Add path parameters
        for param in &endpoint.path_parameters {
            let param_name = CodeGenUtils::to_snake_case(&param.name);
            let param_type = self.map_type_to_rust(&param.parameter_type.type_name, false);
            code.push_str(&format!(", {}: {}", param_name, param_type));
        }
        
        // Add query parameters
        for param in &endpoint.query_parameters {
            if !param.required {
                let param_name = CodeGenUtils::to_snake_case(&param.name);
                let param_type = self.map_type_to_rust(&param.parameter_type.type_name, false);
                code.push_str(&format!(", {}: Option<{}>", param_name, param_type));
            }
        }
        
        // Add request body parameter
        if let Some(ref request_body) = endpoint.request_body {
            let body_type = self.map_type_to_rust(&request_body.type_name, false);
            code.push_str(&format!(", request: {}", body_type));
        }
        
        code.push_str(&format!(") -> Result<{}> {{\n", return_type));
        
        // Method implementation
        let path = self.generate_path_building(&endpoint.path, &endpoint.path_parameters);
        code.push_str(&format!("        let url = self.build_url(\"{}\");\n", path));
        
        match endpoint.method {
            HttpMethod::GET => {
                code.push_str("        let response = self.client.get(&url)\n");
            }
            HttpMethod::POST => {
                code.push_str("        let response = self.client.post(&url)\n");
                if endpoint.request_body.is_some() {
                    code.push_str("            .json(&request)\n");
                }
            }
            HttpMethod::PUT => {
                code.push_str("        let response = self.client.put(&url)\n");
                if endpoint.request_body.is_some() {
                    code.push_str("            .json(&request)\n");
                }
            }
            HttpMethod::DELETE => {
                code.push_str("        let response = self.client.delete(&url)\n");
            }
            _ => {
                code.push_str("        let response = self.client.request(Method::GET, &url)\n");
            }
        }
        
        code.push_str("            .headers(self.default_headers())\n");
        code.push_str("            .send()\n");
        code.push_str("            .await?\n");
        code.push_str("            .error_for_status()?;\n\n");
        code.push_str("        let result = response.json().await?;\n");
        code.push_str("        Ok(result)\n");
        code.push_str("    }\n\n");
        
        code
    }

    fn generate_path_building(&self, path: &str, path_params: &[super::api_schema::ParameterSchema]) -> String {
        let mut result = path.to_string();
        
        for param in path_params {
            let placeholder = format!("{{{}}}", param.name);
            let param_name = CodeGenUtils::to_snake_case(&param.name);
            result = result.replace(&placeholder, &format!("{{{}}}", param_name));
        }
        
        result
    }

    fn map_type_to_rust(&self, type_name: &str, nullable: bool) -> String {
        let rust_type = match type_name {
            "String" => "String",
            "Integer" => "i64",
            "Float" => "f64",
            "Boolean" => "bool",
            "DateTime" => "chrono::DateTime<chrono::Utc>",
            "UUID" => "uuid::Uuid",
            "Binary" => "Vec<u8>",
            _ => type_name, // Custom types
        };

        if nullable {
            format!("Option<{}>", rust_type)
        } else {
            rust_type.to_string()
        }
    }

    fn generate_client_struct(&self, schema: &APISchema) -> String {
        let mut code = String::new();
        
        code.push_str("/// OpenSim API client\n");
        code.push_str("#[derive(Debug, Clone)]\n");
        code.push_str("pub struct OpenSimClient {\n");
        code.push_str("    client: reqwest::Client,\n");
        code.push_str("    base_url: String,\n");
        code.push_str("    access_token: Option<String>,\n");
        code.push_str("}\n\n");
        
        code.push_str("impl OpenSimClient {\n");
        
        // Constructor
        code.push_str("    /// Create a new OpenSim client\n");
        code.push_str("    pub fn new(base_url: impl Into<String>) -> Self {\n");
        code.push_str("        Self {\n");
        code.push_str("            client: reqwest::Client::new(),\n");
        code.push_str("            base_url: base_url.into(),\n");
        code.push_str("            access_token: None,\n");
        code.push_str("        }\n");
        code.push_str("    }\n\n");
        
        // Authentication method
        code.push_str("    /// Set access token for authentication\n");
        code.push_str("    pub fn with_token(mut self, token: impl Into<String>) -> Self {\n");
        code.push_str("        self.access_token = Some(token.into());\n");
        code.push_str("        self\n");
        code.push_str("    }\n\n");
        
        // Helper methods
        code.push_str("    fn build_url(&self, path: &str) -> String {\n");
        code.push_str("        format!(\"{}{}\", self.base_url, path)\n");
        code.push_str("    }\n\n");
        
        code.push_str("    fn default_headers(&self) -> reqwest::header::HeaderMap {\n");
        code.push_str("        let mut headers = reqwest::header::HeaderMap::new();\n");
        code.push_str("        headers.insert(reqwest::header::CONTENT_TYPE, \"application/json\".parse().unwrap());\n");
        code.push_str("        \n");
        code.push_str("        if let Some(ref token) = self.access_token {\n");
        code.push_str("            let auth_value = format!(\"Bearer {}\", token);\n");
        code.push_str("            headers.insert(reqwest::header::AUTHORIZATION, auth_value.parse().unwrap());\n");
        code.push_str("        }\n");
        code.push_str("        \n");
        code.push_str("        headers\n");
        code.push_str("    }\n\n");
        
        code
    }

    fn generate_cargo_toml(&self, config: &GeneratorConfig) -> String {
        format!(r#"[package]
name = "{}"
version = "{}"
edition = "2021"
authors = ["{}"]
license = "{}"
description = "OpenSim client library for Rust"
repository = "{}"

[dependencies]
reqwest = {{ version = "0.11", features = ["json"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
anyhow = "1.0"
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
"#,
            config.package_name,
            config.package_version,
            config.author,
            config.license,
            config.repository_url.as_deref().unwrap_or("https://github.com/opensim/opensim-next")
        )
    }
}

impl LanguageGenerator for RustGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        let mut generated_files = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Generate main library file
        let mut lib_content = String::new();
        lib_content.push_str("//! OpenSim Client SDK for Rust\n\n");
        lib_content.push_str("use anyhow::Result;\n");
        lib_content.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Generate data types
        for data_type in &schema.data_types {
            lib_content.push_str(&self.generate_data_type(data_type));
        }

        // Generate client struct
        lib_content.push_str(&self.generate_client_struct(schema));

        // Generate API methods
        for endpoint in &schema.endpoints {
            lib_content.push_str(&self.generate_endpoint_method(endpoint));
        }

        lib_content.push_str("}\n"); // Close impl block

        generated_files.push(GeneratedFile {
            path: PathBuf::from("src/lib.rs"),
            content: lib_content,
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate Cargo.toml
        generated_files.push(GeneratedFile {
            path: PathBuf::from("Cargo.toml"),
            content: self.generate_cargo_toml(config),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate README
        if config.include_documentation {
            let readme_content = format!(r#"# {}

OpenSim client library for Rust.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
{} = "{}"
```

## Usage

```rust
use {}::OpenSimClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let client = OpenSimClient::new("https://api.opensim.org")
        .with_token("your_access_token");
    
    // Use the client...
    
    Ok(())
}}
```

## License

{}
"#,
                config.package_name,
                config.package_name,
                config.package_version,
                config.package_name.replace("-", "_"),
                config.license
            );

            generated_files.push(GeneratedFile {
                path: PathBuf::from("README.md"),
                content: readme_content,
                file_type: GeneratedFileType::Documentation,
            });
        }

        // Generate examples
        if config.include_examples {
            let example_content = format!(r#"//! Basic usage example

use anyhow::Result;
use {}::OpenSimClient;

#[tokio::main]
async fn main() -> Result<()> {{
    let client = OpenSimClient::new("https://api.opensim.org")
        .with_token("your_access_token_here");
    
    // List regions
    let regions = client.list_regions(Some(10), None).await?;
    println!("Found {{}} regions", regions.len());
    
    Ok(())
}}
"#, config.package_name.replace("-", "_"));

            generated_files.push(GeneratedFile {
                path: PathBuf::from("examples/basic.rs"),
                content: example_content,
                file_type: GeneratedFileType::Example,
            });
        }

        let package_info = PackageInfo {
            name: config.package_name.clone(),
            version: config.package_version.clone(),
            description: "OpenSim client library for Rust".to_string(),
            main_file: Some(PathBuf::from("src/lib.rs")),
            entry_points: vec!["OpenSimClient".to_string()],
            dependencies: vec![
                "reqwest".to_string(),
                "serde".to_string(),
                "tokio".to_string(),
                "anyhow".to_string(),
                "chrono".to_string(),
                "uuid".to_string(),
            ],
        };

        Ok(GenerationResult {
            target_language: super::generator::TargetLanguage::Rust,
            generated_files,
            package_info,
            warnings,
            errors,
        })
    }

    fn generate_data_types(&self, types: &[DataTypeSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let mut content = String::new();
        content.push_str("//! Data type definitions\n\n");
        content.push_str("use serde::{Deserialize, Serialize};\n\n");

        for data_type in types {
            content.push_str(&self.generate_data_type(data_type));
        }

        Ok(vec![GeneratedFile {
            path: PathBuf::from("src/types.rs"),
            content,
            file_type: GeneratedFileType::SourceCode,
        }])
    }

    fn generate_api_methods(&self, endpoints: &[EndpointSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let mut content = String::new();
        content.push_str("//! API endpoint methods\n\n");
        content.push_str("use anyhow::Result;\n");
        content.push_str("use crate::types::*;\n\n");

        content.push_str("impl OpenSimClient {\n");
        for endpoint in endpoints {
            content.push_str(&self.generate_endpoint_method(endpoint));
        }
        content.push_str("}\n");

        Ok(vec![GeneratedFile {
            path: PathBuf::from("src/api.rs"),
            content,
            file_type: GeneratedFileType::SourceCode,
        }])
    }

    fn generate_authentication(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<GeneratedFile> {
        let content = r#"//! Authentication helpers

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

impl OpenSimClient {
    /// Authenticate with username and password
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<AuthResponse> {
        let request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        let response = self.client
            .post(&self.build_url("/auth/login"))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;
            
        let auth_response: AuthResponse = response.json().await?;
        self.access_token = Some(auth_response.access_token.clone());
        
        Ok(auth_response)
    }
}
"#;

        Ok(GeneratedFile {
            path: PathBuf::from("src/auth.rs"),
            content: content.to_string(),
            file_type: GeneratedFileType::SourceCode,
        })
    }

    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![GeneratedFile {
            path: PathBuf::from("Cargo.toml"),
            content: self.generate_cargo_toml(config),
            file_type: GeneratedFileType::Configuration,
        }])
    }

    fn generate_examples(&self, _schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let basic_example = format!(r#"//! Basic usage example

use anyhow::Result;
use {}::OpenSimClient;

#[tokio::main]
async fn main() -> Result<()> {{
    let client = OpenSimClient::new("https://api.opensim.org");
    
    // Authenticate
    let mut client = client;
    let auth_response = client.authenticate("username", "password").await?;
    println!("Authenticated successfully, token expires in {{}} seconds", auth_response.expires_in);
    
    Ok(())
}}
"#, config.package_name.replace("-", "_"));

        Ok(vec![GeneratedFile {
            path: PathBuf::from("examples/basic.rs"),
            content: basic_example,
            file_type: GeneratedFileType::Example,
        }])
    }

    fn generate_tests(&self, _schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let test_content = format!(r#"//! Integration tests

use anyhow::Result;
use {}::OpenSimClient;

#[tokio::test]
async fn test_client_creation() {{
    let client = OpenSimClient::new("https://api.example.com");
    assert!(!client.base_url.is_empty());
}}

#[tokio::test]
async fn test_authentication() -> Result<()> {{
    // This would require a test server or mocking
    // For now, just test that the method exists
    let mut client = OpenSimClient::new("https://api.example.com");
    
    // This would fail without a real server, so we just verify the client exists
    assert!(client.access_token.is_none());
    
    Ok(())
}}
"#, config.package_name.replace("-", "_"));

        Ok(vec![GeneratedFile {
            path: PathBuf::from("tests/integration.rs"),
            content: test_content,
            file_type: GeneratedFileType::Test,
        }])
    }

    fn get_file_extension(&self) -> &'static str {
        ".rs"
    }

    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile> {
        Ok(GeneratedFile {
            path: PathBuf::from("Cargo.toml"),
            content: self.generate_cargo_toml(config),
            file_type: GeneratedFileType::Configuration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_sdk::api_schema::APISchema;

    #[test]
    fn test_rust_generator() {
        let generator = RustGenerator::new();
        let schema = APISchema::create_opensim_schema();
        let config = GeneratorConfig {
            target_language: super::super::generator::TargetLanguage::Rust,
            ..Default::default()
        };

        let result = generator.generate_client(&schema, &config).unwrap();
        assert!(!result.generated_files.is_empty());
        assert_eq!(result.target_language, super::super::generator::TargetLanguage::Rust);
    }
}