//! JavaScript/TypeScript SDK generator for OpenSim client library

use std::path::PathBuf;
use anyhow::Result;

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema},
    generator::{LanguageGenerator, GeneratorConfig, GeneratedFile, GeneratedFileType, GenerationResult, PackageInfo, TargetLanguage},
};

/// JavaScript/TypeScript SDK generator
pub struct JavaScriptGenerator;

impl JavaScriptGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_package_json(&self, config: &GeneratorConfig) -> String {
        format!(r#"{{
  "name": "{}",
  "version": "{}",
  "description": "OpenSim client library for JavaScript/TypeScript",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "author": "{}",
  "license": "{}",
  "repository": "{}",
  "keywords": ["opensim", "client", "sdk", "api"],
  "scripts": {{
    "build": "tsc",
    "test": "jest",
    "lint": "eslint src/**/*.ts",
    "prepublishOnly": "npm run build"
  }},
  "dependencies": {{
    "axios": "^1.0.0"
  }},
  "devDependencies": {{
    "@types/node": "^18.0.0",
    "typescript": "^4.9.0",
    "jest": "^29.0.0",
    "@types/jest": "^29.0.0",
    "eslint": "^8.0.0",
    "@typescript-eslint/eslint-plugin": "^5.0.0",
    "@typescript-eslint/parser": "^5.0.0"
  }},
  "files": [
    "dist/**/*"
  ]
}}
"#,
            config.package_name,
            config.package_version,
            config.author,
            config.license,
            config.repository_url.as_deref().unwrap_or("https://github.com/opensim/opensim-next")
        )
    }
}

impl LanguageGenerator for JavaScriptGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        let mut generated_files = Vec::new();

        // Generate main client TypeScript file
        let client_content = r#"/**
 * OpenSim Client SDK for TypeScript/JavaScript
 */

import axios, { AxiosInstance, AxiosResponse } from 'axios';

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
}

export interface UserProfile {
  id: string;
  username: string;
  email?: string;
  created_at: string;
  last_login?: string;
}

export class OpenSimClient {
  private client: AxiosInstance;
  private accessToken?: string;

  constructor(baseURL: string, accessToken?: string) {
    this.client = axios.create({
      baseURL: baseURL.replace(/\/$/, ''),
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (accessToken) {
      this.setAccessToken(accessToken);
    }
  }

  setAccessToken(token: string): void {
    this.accessToken = token;
    this.client.defaults.headers.common['Authorization'] = `Bearer ${token}`;
  }

  async authenticate(username: string, password: string): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>('/auth/login', {
      username,
      password,
    });

    this.setAccessToken(response.data.access_token);
    return response.data;
  }

  async getUserProfile(userId: string): Promise<UserProfile> {
    const response = await this.client.get<UserProfile>(`/users/${userId}`);
    return response.data;
  }

  async listRegions(limit?: number): Promise<any[]> {
    const params = limit ? { limit } : {};
    const response = await this.client.get('/regions', { params });
    return response.data;
  }
}

export default OpenSimClient;
"#;

        generated_files.push(GeneratedFile {
            path: PathBuf::from("src/index.ts"),
            content: client_content.to_string(),
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate package.json
        generated_files.push(GeneratedFile {
            path: PathBuf::from("package.json"),
            content: self.generate_package_json(config),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate TypeScript config
        let tsconfig_content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
"#;

        generated_files.push(GeneratedFile {
            path: PathBuf::from("tsconfig.json"),
            content: tsconfig_content.to_string(),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate README if requested
        if config.include_documentation {
            let readme_content = format!(r#"# {}

OpenSim client library for JavaScript/TypeScript.

## Installation

```bash
npm install {}
```

## Usage

```typescript
import {{ OpenSimClient }} from '{}';

const client = new OpenSimClient('https://api.opensim.org');

// Authenticate
const auth = await client.authenticate('username', 'password');
console.log('Authenticated:', auth.access_token);

// Use the client...
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

        let package_info = PackageInfo {
            name: config.package_name.clone(),
            version: config.package_version.clone(),
            description: "OpenSim client library for JavaScript/TypeScript".to_string(),
            main_file: Some(PathBuf::from("src/index.ts")),
            entry_points: vec!["OpenSimClient".to_string()],
            dependencies: vec!["axios".to_string()],
        };

        Ok(GenerationResult {
            target_language: TargetLanguage::JavaScript,
            generated_files,
            package_info,
            warnings: vec![],
            errors: vec![],
        })
    }

    fn generate_data_types(&self, _types: &[DataTypeSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // TypeScript interfaces would be generated here
        Ok(vec![])
    }

    fn generate_api_methods(&self, _endpoints: &[EndpointSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // API method implementations would be generated here
        Ok(vec![])
    }

    fn generate_authentication(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<GeneratedFile> {
        let content = "// Authentication methods would be generated here";
        Ok(GeneratedFile {
            path: PathBuf::from("src/auth.ts"),
            content: content.to_string(),
            file_type: GeneratedFileType::SourceCode,
        })
    }

    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![GeneratedFile {
            path: PathBuf::from("package.json"),
            content: self.generate_package_json(config),
            file_type: GeneratedFileType::Configuration,
        }])
    }

    fn generate_examples(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let example_content = "// Examples would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("examples/basic.js"),
            content: example_content.to_string(),
            file_type: GeneratedFileType::Example,
        }])
    }

    fn generate_tests(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let test_content = "// Tests would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("tests/client.test.ts"),
            content: test_content.to_string(),
            file_type: GeneratedFileType::Test,
        }])
    }

    fn get_file_extension(&self) -> &'static str {
        ".ts"
    }

    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile> {
        Ok(GeneratedFile {
            path: PathBuf::from("package.json"),
            content: self.generate_package_json(config),
            file_type: GeneratedFileType::Configuration,
        })
    }
}