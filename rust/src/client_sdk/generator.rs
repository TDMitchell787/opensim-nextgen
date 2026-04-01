//! SDK generator for multiple programming languages

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};

use super::api_schema::{APISchema, EndpointSchema, DataTypeSchema, TypeDefinition};

/// Supported target languages for SDK generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    CSharp,
    Java,
    Go,
    PHP,
    Ruby,
    Swift,
    Kotlin,
}

/// SDK generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    pub target_language: TargetLanguage,
    pub output_directory: PathBuf,
    pub package_name: String,
    pub package_version: String,
    pub namespace: Option<String>,
    pub author: String,
    pub license: String,
    pub repository_url: Option<String>,
    pub include_examples: bool,
    pub include_documentation: bool,
    pub async_support: bool,
    pub custom_templates: HashMap<String, String>,
    pub additional_dependencies: Vec<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            target_language: TargetLanguage::Rust,
            output_directory: PathBuf::from("./generated-sdk"),
            package_name: "opensim-client".to_string(),
            package_version: "1.0.0".to_string(),
            namespace: None,
            author: "OpenSim Community".to_string(),
            license: "MIT".to_string(),
            repository_url: Some("https://github.com/opensim/opensim-next".to_string()),
            include_examples: true,
            include_documentation: true,
            async_support: true,
            custom_templates: HashMap::new(),
            additional_dependencies: Vec::new(),
        }
    }
}

/// Generated file information
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
    pub file_type: GeneratedFileType,
}

/// Types of generated files
#[derive(Debug, Clone)]
pub enum GeneratedFileType {
    SourceCode,
    Documentation,
    Configuration,
    Example,
    Test,
}

/// SDK generation result
#[derive(Debug, Clone)]
pub struct GenerationResult {
    pub target_language: TargetLanguage,
    pub generated_files: Vec<GeneratedFile>,
    pub package_info: PackageInfo,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main_file: Option<PathBuf>,
    pub entry_points: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Language-specific configuration and templates
pub trait LanguageGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult>;
    fn generate_data_types(&self, types: &[DataTypeSchema], config: &GeneratorConfig) -> Result<Vec<GeneratedFile>>;
    fn generate_api_methods(&self, endpoints: &[EndpointSchema], config: &GeneratorConfig) -> Result<Vec<GeneratedFile>>;
    fn generate_authentication(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GeneratedFile>;
    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>>;
    fn generate_examples(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>>;
    fn generate_tests(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>>;
    fn get_file_extension(&self) -> &'static str;
    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile>;
}

/// Main SDK generator
pub struct SDKGenerator {
    generators: HashMap<TargetLanguage, Box<dyn LanguageGenerator>>,
}

impl SDKGenerator {
    /// Create a new SDK generator with all supported languages
    pub fn new() -> Self {
        let mut generators: HashMap<TargetLanguage, Box<dyn LanguageGenerator>> = HashMap::new();
        
        generators.insert(TargetLanguage::Rust, Box::new(super::rust_sdk::RustGenerator::new()));
        generators.insert(TargetLanguage::Python, Box::new(super::python_sdk::PythonGenerator::new()));
        generators.insert(TargetLanguage::JavaScript, Box::new(super::javascript_sdk::JavaScriptGenerator::new()));
        generators.insert(TargetLanguage::CSharp, Box::new(super::csharp_sdk::CSharpGenerator::new()));
        generators.insert(TargetLanguage::Java, Box::new(super::java_sdk::JavaGenerator::new()));
        generators.insert(TargetLanguage::Go, Box::new(super::go_sdk::GoGenerator));
        generators.insert(TargetLanguage::PHP, Box::new(super::php_sdk::PhpGenerator));
        generators.insert(TargetLanguage::Ruby, Box::new(super::ruby_sdk::RubyGenerator));
        
        Self { generators }
    }

    /// Generate SDK for specified language
    pub async fn generate_sdk(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        info!("Generating SDK for {:?}", config.target_language);

        let generator = self.generators.get(&config.target_language)
            .ok_or_else(|| anyhow!("No generator available for language: {:?}", config.target_language))?;

        // Ensure output directory exists
        self.ensure_output_directory(&config.output_directory).await?;

        // Generate the SDK
        let mut result = generator.generate_client(schema, config)?;

        // Write files to disk
        self.write_generated_files(&result.generated_files, &config.output_directory).await?;

        info!("SDK generation completed for {:?}. Generated {} files.", 
               config.target_language, result.generated_files.len());

        Ok(result)
    }

    /// Generate SDKs for multiple languages
    pub async fn generate_multiple_sdks(&self, schema: &APISchema, configs: Vec<GeneratorConfig>) -> Result<Vec<GenerationResult>> {
        let mut results = Vec::new();

        for config in configs {
            match self.generate_sdk(schema, &config).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    error!("Failed to generate SDK for {:?}: {}", config.target_language, e);
                    // Continue with other languages
                }
            }
        }

        Ok(results)
    }

    /// Validate API schema before generation
    pub fn validate_schema(&self, schema: &APISchema) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for missing required fields
        if schema.endpoints.is_empty() {
            warnings.push("No endpoints defined in schema".to_string());
        }

        if schema.data_types.is_empty() {
            warnings.push("No data types defined in schema".to_string());
        }

        // Validate endpoint references
        let type_names: Vec<_> = schema.data_types.iter().map(|t| &t.name).collect();
        
        for endpoint in &schema.endpoints {
            if let Some(ref request_body) = endpoint.request_body {
                if !type_names.contains(&&request_body.type_name) && !self.is_primitive_type(&request_body.type_name) {
                    warnings.push(format!("Endpoint '{}' references undefined type '{}'", 
                                        endpoint.name, request_body.type_name));
                }
            }

            if !type_names.contains(&&endpoint.response_body.type_name) && !self.is_primitive_type(&endpoint.response_body.type_name) {
                warnings.push(format!("Endpoint '{}' response references undefined type '{}'", 
                                    endpoint.name, endpoint.response_body.type_name));
            }
        }

        // Validate circular dependencies
        self.check_circular_dependencies(&schema.data_types, &mut warnings);

        Ok(warnings)
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<TargetLanguage> {
        self.generators.keys().cloned().collect()
    }

    /// Create configuration for all supported languages
    pub fn create_multi_language_configs(&self, base_config: &GeneratorConfig) -> Vec<GeneratorConfig> {
        self.supported_languages().into_iter().map(|lang| {
            let mut config = base_config.clone();
            config.target_language = lang.clone();
            config.output_directory = base_config.output_directory.join(format!("{:?}", lang).to_lowercase());
            
            // Language-specific adjustments
            match lang {
                TargetLanguage::Python => {
                    config.package_name = config.package_name.replace("-", "_");
                }
                TargetLanguage::CSharp => {
                    config.namespace = Some("OpenSim.Client".to_string());
                }
                TargetLanguage::Java => {
                    config.namespace = Some("org.opensim.client".to_string());
                }
                _ => {}
            }
            
            config
        }).collect()
    }

    async fn ensure_output_directory(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
            debug!("Created output directory: {:?}", path);
        }
        Ok(())
    }

    async fn write_generated_files(&self, files: &[GeneratedFile], base_path: &Path) -> Result<()> {
        for file in files {
            let full_path = base_path.join(&file.path);
            
            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            tokio::fs::write(&full_path, &file.content).await?;
            debug!("Generated file: {:?}", full_path);
        }
        Ok(())
    }

    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(type_name, 
            "String" | "Integer" | "Float" | "Boolean" | "DateTime" | "UUID" | "Binary" |
            "Object" | "Array" | "Map"
        )
    }

    fn check_circular_dependencies(&self, types: &[DataTypeSchema], warnings: &mut Vec<String>) {
        // Simple circular dependency detection
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for data_type in types {
            if !visited.contains(&data_type.name) {
                self.detect_circular_dependency(&data_type.name, types, &mut visited, &mut recursion_stack, warnings);
            }
        }
    }

    fn detect_circular_dependency(
        &self,
        type_name: &str,
        types: &[DataTypeSchema],
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
        warnings: &mut Vec<String>,
    ) {
        visited.insert(type_name.to_string());
        recursion_stack.insert(type_name.to_string());

        if let Some(data_type) = types.iter().find(|t| t.name == type_name) {
            if let TypeDefinition::Object { properties } = &data_type.type_definition {
                for property in properties {
                    let referenced_type = &property.property_type.type_name;
                    
                    if recursion_stack.contains(referenced_type) {
                        warnings.push(format!("Circular dependency detected: {} -> {}", type_name, referenced_type));
                    } else if !visited.contains(referenced_type) && !self.is_primitive_type(referenced_type) {
                        self.detect_circular_dependency(referenced_type, types, visited, recursion_stack, warnings);
                    }
                }
            }
        }

        recursion_stack.remove(type_name);
    }
}

/// Utility functions for code generation
pub struct CodeGenUtils;

impl CodeGenUtils {
    /// Convert snake_case to PascalCase
    pub fn to_pascal_case(input: &str) -> String {
        input.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect()
    }

    /// Convert snake_case to camelCase
    pub fn to_camel_case(input: &str) -> String {
        let pascal = Self::to_pascal_case(input);
        if let Some(first_char) = pascal.chars().next() {
            first_char.to_lowercase().collect::<String>() + &pascal[1..]
        } else {
            pascal
        }
    }

    /// Convert PascalCase/camelCase to snake_case
    pub fn to_snake_case(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }

        result
    }

    /// Convert string to valid identifier for the given language
    pub fn to_valid_identifier(input: &str, language: &TargetLanguage) -> String {
        let reserved_words = Self::get_reserved_words(language);
        let mut identifier = input.replace("-", "_").replace(" ", "_");

        // Remove invalid characters
        identifier = identifier.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        // Ensure it doesn't start with a number
        if identifier.chars().next().map_or(false, |c| c.is_numeric()) {
            identifier = format!("_{}", identifier);
        }

        // Handle reserved words
        if reserved_words.contains(&identifier.as_str()) {
            identifier = format!("{}_", identifier);
        }

        identifier
    }

    /// Get reserved words for a programming language
    pub fn get_reserved_words(language: &TargetLanguage) -> Vec<&'static str> {
        match language {
            TargetLanguage::Rust => vec![
                "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
                "static", "struct", "super", "trait", "true", "type", "unsafe",
                "use", "where", "while", "async", "await", "dyn",
            ],
            TargetLanguage::Python => vec![
                "and", "as", "assert", "break", "class", "continue", "def", "del",
                "elif", "else", "except", "finally", "for", "from", "global",
                "if", "import", "in", "is", "lambda", "not", "or", "pass",
                "raise", "return", "try", "while", "with", "yield", "async", "await",
            ],
            TargetLanguage::JavaScript | TargetLanguage::TypeScript => vec![
                "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "export", "extends", "finally",
                "for", "function", "if", "import", "in", "instanceof", "new",
                "return", "super", "switch", "this", "throw", "try", "typeof",
                "var", "void", "while", "with", "yield", "async", "await",
            ],
            TargetLanguage::Java => vec![
                "abstract", "assert", "boolean", "break", "byte", "case", "catch",
                "char", "class", "const", "continue", "default", "do", "double",
                "else", "enum", "extends", "final", "finally", "float", "for",
                "goto", "if", "implements", "import", "instanceof", "int",
                "interface", "long", "native", "new", "package", "private",
                "protected", "public", "return", "short", "static", "strictfp",
                "super", "switch", "synchronized", "this", "throw", "throws",
                "transient", "try", "void", "volatile", "while",
            ],
            TargetLanguage::CSharp => vec![
                "abstract", "as", "base", "bool", "break", "byte", "case", "catch",
                "char", "checked", "class", "const", "continue", "decimal", "default",
                "delegate", "do", "double", "else", "enum", "event", "explicit",
                "extern", "false", "finally", "fixed", "float", "for", "foreach",
                "goto", "if", "implicit", "in", "int", "interface", "internal",
                "is", "lock", "long", "namespace", "new", "null", "object",
                "operator", "out", "override", "params", "private", "protected",
                "public", "readonly", "ref", "return", "sbyte", "sealed", "short",
                "sizeof", "stackalloc", "static", "string", "struct", "switch",
                "this", "throw", "true", "try", "typeof", "uint", "ulong",
                "unchecked", "unsafe", "ushort", "using", "virtual", "void",
                "volatile", "while", "async", "await",
            ],
            _ => vec![], // Add more languages as needed
        }
    }

    /// Generate documentation comment for the target language
    pub fn generate_doc_comment(text: &str, language: &TargetLanguage) -> String {
        match language {
            TargetLanguage::Rust => format!("/// {}", text),
            TargetLanguage::Python => format!("\"\"\"{}\"\"\"", text),
            TargetLanguage::JavaScript | TargetLanguage::TypeScript => format!("/**\n * {}\n */", text),
            TargetLanguage::Java => format!("/**\n * {}\n */", text),
            TargetLanguage::CSharp => format!("/// <summary>\n/// {}\n/// </summary>", text),
            _ => format!("// {}", text),
        }
    }

    /// Escape string literal for the target language
    pub fn escape_string_literal(text: &str, language: &TargetLanguage) -> String {
        match language {
            TargetLanguage::Rust => format!("\"{}\"", text.replace("\"", "\\\"")),
            TargetLanguage::Python => format!("\"{}\"", text.replace("\"", "\\\"")),
            TargetLanguage::JavaScript | TargetLanguage::TypeScript => format!("\"{}\"", text.replace("\"", "\\\"")),
            TargetLanguage::Java => format!("\"{}\"", text.replace("\"", "\\\"")),
            TargetLanguage::CSharp => format!("\"{}\"", text.replace("\"", "\\\"")),
            _ => format!("\"{}\"", text.replace("\"", "\\\"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_conversions() {
        assert_eq!(CodeGenUtils::to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(CodeGenUtils::to_camel_case("hello_world"), "helloWorld");
        assert_eq!(CodeGenUtils::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(CodeGenUtils::to_snake_case("helloWorld"), "hello_world");
    }

    #[test]
    fn test_valid_identifier() {
        assert_eq!(CodeGenUtils::to_valid_identifier("class", &TargetLanguage::Python), "class_");
        assert_eq!(CodeGenUtils::to_valid_identifier("123test", &TargetLanguage::Rust), "_123test");
        assert_eq!(CodeGenUtils::to_valid_identifier("hello-world", &TargetLanguage::Java), "hello_world");
    }

    #[tokio::test]
    async fn test_sdk_generator() -> Result<()> {
        let generator = SDKGenerator::new();
        let languages = generator.supported_languages();
        
        assert!(!languages.is_empty());
        assert!(languages.contains(&TargetLanguage::Rust));
        assert!(languages.contains(&TargetLanguage::Python));
        
        Ok(())
    }
}