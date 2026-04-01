//! Python SDK generator for OpenSim client library

use std::{collections::HashMap, path::PathBuf};
use anyhow::Result;

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema, TypeDefinition, HttpMethod},
    generator::{LanguageGenerator, GeneratorConfig, GeneratedFile, GeneratedFileType, GenerationResult, PackageInfo, CodeGenUtils, TargetLanguage},
};

/// Python-specific SDK generator
pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_data_class(&self, data_type: &DataTypeSchema) -> String {
        let mut code = String::new();
        
        // Add docstring
        code.push_str(&format!("class {}:\n", data_type.name));
        code.push_str(&format!("    \"\"\"{}\"\"\"\n\n", data_type.description));
        
        match &data_type.type_definition {
            TypeDefinition::Object { properties } => {
                // Generate __init__ method
                code.push_str("    def __init__(self");
                
                for property in properties {
                    let field_name = CodeGenUtils::to_snake_case(&property.name);
                    let field_type = self.map_type_to_python(&property.property_type.type_name);
                    
                    if property.required {
                        code.push_str(&format!(", {}: {}", field_name, field_type));
                    } else {
                        code.push_str(&format!(", {}: Optional[{}] = None", field_name, field_type));
                    }
                }
                
                code.push_str("):\n");
                
                // Assign properties
                for property in properties {
                    let field_name = CodeGenUtils::to_snake_case(&property.name);
                    code.push_str(&format!("        self.{} = {}\n", field_name, field_name));
                }
                
                code.push_str("\n");
                
                // Generate to_dict method
                code.push_str("    def to_dict(self) -> Dict[str, Any]:\n");
                code.push_str("        \"\"\"Convert to dictionary.\"\"\"\n");
                code.push_str("        return {\n");
                
                for property in properties {
                    let field_name = CodeGenUtils::to_snake_case(&property.name);
                    let original_name = &property.name;
                    code.push_str(&format!("            '{}': self.{},\n", original_name, field_name));
                }
                
                code.push_str("        }\n\n");
                
                // Generate from_dict class method
                code.push_str("    @classmethod\n");
                code.push_str(&format!("    def from_dict(cls, data: Dict[str, Any]) -> '{}':\n", data_type.name));
                code.push_str("        \"\"\"Create from dictionary.\"\"\"\n");
                code.push_str("        return cls(\n");
                
                for property in properties {
                    let field_name = CodeGenUtils::to_snake_case(&property.name);
                    let original_name = &property.name;
                    code.push_str(&format!("            {}=data.get('{}'),\n", field_name, original_name));
                }
                
                code.push_str("        )\n\n");
            }
            TypeDefinition::Enum { variants } => {
                code.push_str("    \"\"\"Enumeration values\"\"\"\n");
                
                for variant in variants {
                    let variant_name = variant.name.to_uppercase();
                    if let Some(value) = variant.value.as_str() {
                        code.push_str(&format!("    {} = '{}'\n", variant_name, value));
                    } else {
                        code.push_str(&format!("    {} = '{}'\n", variant_name, variant.name));
                    }
                }
                
                code.push_str("\n");
            }
            _ => {
                code.push_str("    pass  # TODO: Implement specific type\n\n");
            }
        }
        
        code
    }

    fn generate_endpoint_method(&self, endpoint: &EndpointSchema) -> String {
        let mut code = String::new();
        
        let method_name = CodeGenUtils::to_snake_case(&endpoint.name.replace(" ", "_"));
        
        // Method signature
        code.push_str(&format!("    async def {}(self", method_name));
        
        // Add path parameters
        for param in &endpoint.path_parameters {
            let param_name = CodeGenUtils::to_snake_case(&param.name);
            let param_type = self.map_type_to_python(&param.parameter_type.type_name);
            code.push_str(&format!(", {}: {}", param_name, param_type));
        }
        
        // Add query parameters
        for param in &endpoint.query_parameters {
            if !param.required {
                let param_name = CodeGenUtils::to_snake_case(&param.name);
                let param_type = self.map_type_to_python(&param.parameter_type.type_name);
                code.push_str(&format!(", {}: Optional[{}] = None", param_name, param_type));
            }
        }
        
        // Add request body parameter
        if let Some(ref request_body) = endpoint.request_body {
            let body_type = self.map_type_to_python(&request_body.type_name);
            code.push_str(&format!(", request: {}", body_type));
        }
        
        let return_type = self.map_type_to_python(&endpoint.response_body.type_name);
        code.push_str(&format!(") -> {}:\n", return_type));
        
        // Method docstring
        code.push_str(&format!("        \"\"\"{}\"\"\"\n", endpoint.description));
        
        // Method implementation
        let path = self.generate_path_building(&endpoint.path, &endpoint.path_parameters);
        code.push_str(&format!("        url = self._build_url('{}')\n", path));
        
        let http_method = match endpoint.method {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            _ => "GET",
        };
        
        if endpoint.request_body.is_some() {
            code.push_str(&format!("        response = await self._request('{}', url, json=request.to_dict() if hasattr(request, 'to_dict') else request)\n", http_method));
        } else {
            code.push_str(&format!("        response = await self._request('{}', url)\n", http_method));
        }
        
        code.push_str(&format!("        return {}.from_dict(response) if hasattr({}, 'from_dict') else response\n\n", return_type, return_type));
        
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

    fn map_type_to_python(&self, type_name: &str) -> String {
        match type_name {
            "String" => "str",
            "Integer" => "int",
            "Float" => "float",
            "Boolean" => "bool",
            "DateTime" => "datetime",
            "UUID" => "str",  // or use uuid.UUID
            "Binary" => "bytes",
            _ => type_name, // Custom types
        }.to_string()
    }

    fn generate_client_class(&self, schema: &APISchema) -> String {
        let mut code = String::new();
        
        code.push_str("class OpenSimClient:\n");
        code.push_str("    \"\"\"OpenSim API client for Python.\"\"\"\n\n");
        
        // Constructor
        code.push_str("    def __init__(self, base_url: str, access_token: Optional[str] = None):\n");
        code.push_str("        \"\"\"Initialize the OpenSim client.\n\n");
        code.push_str("        Args:\n");
        code.push_str("            base_url: The base URL of the OpenSim API\n");
        code.push_str("            access_token: Optional access token for authentication\n");
        code.push_str("        \"\"\"\n");
        code.push_str("        self.base_url = base_url.rstrip('/')\n");
        code.push_str("        self.access_token = access_token\n");
        code.push_str("        self.session = aiohttp.ClientSession()\n\n");
        
        // Helper methods
        code.push_str("    def _build_url(self, path: str) -> str:\n");
        code.push_str("        \"\"\"Build full URL from path.\"\"\"\n");
        code.push_str("        return f'{self.base_url}{path}'\n\n");
        
        code.push_str("    def _get_headers(self) -> Dict[str, str]:\n");
        code.push_str("        \"\"\"Get default headers.\"\"\"\n");
        code.push_str("        headers = {'Content-Type': 'application/json'}\n");
        code.push_str("        if self.access_token:\n");
        code.push_str("            headers['Authorization'] = f'Bearer {self.access_token}'\n");
        code.push_str("        return headers\n\n");
        
        code.push_str("    async def _request(self, method: str, url: str, **kwargs) -> Dict[str, Any]:\n");
        code.push_str("        \"\"\"Make HTTP request.\"\"\"\n");
        code.push_str("        headers = self._get_headers()\n");
        code.push_str("        kwargs.setdefault('headers', {}).update(headers)\n");
        code.push_str("        \n");
        code.push_str("        async with self.session.request(method, url, **kwargs) as response:\n");
        code.push_str("            response.raise_for_status()\n");
        code.push_str("            return await response.json()\n\n");
        
        code.push_str("    async def close(self):\n");
        code.push_str("        \"\"\"Close the client session.\"\"\"\n");
        code.push_str("        await self.session.close()\n\n");
        
        code.push_str("    async def __aenter__(self):\n");
        code.push_str("        return self\n\n");
        
        code.push_str("    async def __aexit__(self, exc_type, exc_val, exc_tb):\n");
        code.push_str("        await self.close()\n\n");
        
        code
    }

    fn generate_setup_py(&self, config: &GeneratorConfig) -> String {
        format!(r#"from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="{}",
    version="{}",
    author="{}",
    description="OpenSim client library for Python",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="{}",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    install_requires=[
        "aiohttp>=3.8.0",
        "typing-extensions>=4.0.0",
    ],
    extras_require={{
        "dev": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.20.0",
            "black>=22.0.0",
            "mypy>=0.900",
        ],
    }},
)
"#,
            config.package_name,
            config.package_version,
            config.author,
            config.repository_url.as_deref().unwrap_or("https://github.com/opensim/opensim-next")
        )
    }
}

impl LanguageGenerator for PythonGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        let mut generated_files = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Generate main client file
        let mut client_content = String::new();
        client_content.push_str("\"\"\"OpenSim Client SDK for Python.\"\"\"\n\n");
        client_content.push_str("import asyncio\n");
        client_content.push_str("from datetime import datetime\n");
        client_content.push_str("from typing import Dict, List, Optional, Any\n");
        client_content.push_str("import aiohttp\n\n");

        // Generate data types
        for data_type in &schema.data_types {
            client_content.push_str(&self.generate_data_class(data_type));
        }

        // Generate client class
        client_content.push_str(&self.generate_client_class(schema));

        // Generate API methods
        for endpoint in &schema.endpoints {
            client_content.push_str(&self.generate_endpoint_method(endpoint));
        }

        generated_files.push(GeneratedFile {
            path: PathBuf::from(format!("{}/client.py", config.package_name)),
            content: client_content,
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate __init__.py
        let init_content = format!(r#""""OpenSim Client SDK for Python."""

from .client import OpenSimClient

__version__ = "{}"
__all__ = ["OpenSimClient"]
"#, config.package_version);

        generated_files.push(GeneratedFile {
            path: PathBuf::from(format!("{}/__init__.py", config.package_name)),
            content: init_content,
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate setup.py
        generated_files.push(GeneratedFile {
            path: PathBuf::from("setup.py"),
            content: self.generate_setup_py(config),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate README
        if config.include_documentation {
            let readme_content = format!(r#"# {}

OpenSim client library for Python.

## Installation

```bash
pip install {}
```

## Usage

```python
import asyncio
from {} import OpenSimClient

async def main():
    async with OpenSimClient("https://api.opensim.org", "your_token") as client:
        # Use the client...
        pass

if __name__ == "__main__":
    asyncio.run(main())
```

## License

{}
"#,
                config.package_name,
                config.package_name,
                config.package_name,
                config.license
            );

            generated_files.push(GeneratedFile {
                path: PathBuf::from("README.md"),
                content: readme_content,
                file_type: GeneratedFileType::Documentation,
            });
        }

        // Generate example
        if config.include_examples {
            let example_content = format!(r#"""Basic usage example for OpenSim Python client."""

import asyncio
from {} import OpenSimClient

async def main():
    """Main example function."""
    async with OpenSimClient("https://api.opensim.org") as client:
        # Authenticate
        auth_response = await client.auth_login({{
            "username": "your_username",
            "password": "your_password"
        }})
        
        client.access_token = auth_response.access_token
        
        # List regions
        regions = await client.list_regions()
        print(f"Found {{len(regions)}} regions")

if __name__ == "__main__":
    asyncio.run(main())
"#, config.package_name);

            generated_files.push(GeneratedFile {
                path: PathBuf::from("examples/basic.py"),
                content: example_content,
                file_type: GeneratedFileType::Example,
            });
        }

        let package_info = PackageInfo {
            name: config.package_name.clone(),
            version: config.package_version.clone(),
            description: "OpenSim client library for Python".to_string(),
            main_file: Some(PathBuf::from(format!("{}/__init__.py", config.package_name))),
            entry_points: vec!["OpenSimClient".to_string()],
            dependencies: vec![
                "aiohttp".to_string(),
                "typing-extensions".to_string(),
            ],
        };

        Ok(GenerationResult {
            target_language: TargetLanguage::Python,
            generated_files,
            package_info,
            warnings,
            errors,
        })
    }

    fn generate_data_types(&self, types: &[DataTypeSchema], config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let mut content = String::new();
        content.push_str("\"\"\"Data type definitions.\"\"\"\n\n");
        content.push_str("from datetime import datetime\n");
        content.push_str("from typing import Dict, List, Optional, Any\n\n");

        for data_type in types {
            content.push_str(&self.generate_data_class(data_type));
        }

        Ok(vec![GeneratedFile {
            path: PathBuf::from(format!("{}/types.py", config.package_name)),
            content,
            file_type: GeneratedFileType::SourceCode,
        }])
    }

    fn generate_api_methods(&self, endpoints: &[EndpointSchema], config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let mut content = String::new();
        content.push_str("\"\"\"API endpoint methods.\"\"\"\n\n");
        content.push_str("from typing import Dict, List, Optional, Any\n");
        content.push_str("from .types import *\n\n");

        content.push_str("class APIMethodsMixin:\n");
        content.push_str("    \"\"\"Mixin class for API methods.\"\"\"\n\n");
        
        for endpoint in endpoints {
            content.push_str(&self.generate_endpoint_method(endpoint));
        }

        Ok(vec![GeneratedFile {
            path: PathBuf::from(format!("{}/api.py", config.package_name)),
            content,
            file_type: GeneratedFileType::SourceCode,
        }])
    }

    fn generate_authentication(&self, _schema: &APISchema, config: &GeneratorConfig) -> Result<GeneratedFile> {
        let content = format!(r#""""Authentication helpers."""

from typing import Dict, Any

class AuthMixin:
    """Authentication mixin for OpenSim client."""
    
    async def authenticate(self, username: str, password: str) -> Dict[str, Any]:
        """Authenticate with username and password."""
        auth_data = {{
            "username": username,
            "password": password
        }}
        
        response = await self._request("POST", self._build_url("/auth/login"), json=auth_data)
        self.access_token = response.get("access_token")
        return response
    
    def set_token(self, token: str):
        """Set access token."""
        self.access_token = token
"#);

        Ok(GeneratedFile {
            path: PathBuf::from(format!("{}/auth.py", config.package_name)),
            content,
            file_type: GeneratedFileType::SourceCode,
        })
    }

    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![GeneratedFile {
            path: PathBuf::from("setup.py"),
            content: self.generate_setup_py(config),
            file_type: GeneratedFileType::Configuration,
        }])
    }

    fn generate_examples(&self, _schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let example_content = format!(r#"""Example usage of OpenSim Python client."""

import asyncio
from {} import OpenSimClient

async def main():
    """Main example function."""
    client = OpenSimClient("https://api.opensim.org")
    
    try:
        # Authenticate
        await client.authenticate("username", "password")
        print("Authentication successful")
        
        # Your API calls here...
        
    finally:
        await client.close()

if __name__ == "__main__":
    asyncio.run(main())
"#, config.package_name);

        Ok(vec![GeneratedFile {
            path: PathBuf::from("examples/basic.py"),
            content: example_content,
            file_type: GeneratedFileType::Example,
        }])
    }

    fn generate_tests(&self, _schema: &APISchema, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let test_content = format!(r#"""Tests for OpenSim Python client."""

import pytest
from {} import OpenSimClient

@pytest.mark.asyncio
async def test_client_creation():
    """Test client creation."""
    client = OpenSimClient("https://api.example.com")
    assert client.base_url == "https://api.example.com"
    await client.close()

@pytest.mark.asyncio
async def test_authentication():
    """Test authentication flow."""
    client = OpenSimClient("https://api.example.com")
    
    # This would require a test server or mocking
    assert client.access_token is None
    
    await client.close()
"#, config.package_name);

        Ok(vec![GeneratedFile {
            path: PathBuf::from("tests/test_client.py"),
            content: test_content,
            file_type: GeneratedFileType::Test,
        }])
    }

    fn get_file_extension(&self) -> &'static str {
        ".py"
    }

    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile> {
        Ok(GeneratedFile {
            path: PathBuf::from("setup.py"),
            content: self.generate_setup_py(config),
            file_type: GeneratedFileType::Configuration,
        })
    }
}