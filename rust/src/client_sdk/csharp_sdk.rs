//! C# SDK generator for OpenSim client library

use std::path::PathBuf;
use anyhow::Result;

use super::{
    api_schema::{APISchema, EndpointSchema, DataTypeSchema},
    generator::{LanguageGenerator, GeneratorConfig, GeneratedFile, GeneratedFileType, GenerationResult, PackageInfo, TargetLanguage},
};

/// C# SDK generator
pub struct CSharpGenerator;

impl CSharpGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_csproj(&self, config: &GeneratorConfig) -> String {
        let namespace = config.namespace.as_deref().unwrap_or("OpenSim.Client");
        
        format!(r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net6.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <PackageId>{}</PackageId>
    <Version>{}</Version>
    <Authors>{}</Authors>
    <Description>OpenSim client library for .NET</Description>
    <PackageLicenseExpression>{}</PackageLicenseExpression>
    <RepositoryUrl>{}</RepositoryUrl>
    <RootNamespace>{}</RootNamespace>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
    <PackageReference Include="System.Net.Http" Version="4.3.4" />
  </ItemGroup>

</Project>
"#,
            config.package_name,
            config.package_version,
            config.author,
            config.license,
            config.repository_url.as_deref().unwrap_or("https://github.com/opensim/opensim-next"),
            namespace
        )
    }
}

impl LanguageGenerator for CSharpGenerator {
    fn generate_client(&self, schema: &APISchema, config: &GeneratorConfig) -> Result<GenerationResult> {
        let mut generated_files = Vec::new();
        let namespace = config.namespace.as_deref().unwrap_or("OpenSim.Client");

        // Generate main client class
        let client_content = format!(r#"using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace {}
{{
    /// <summary>
    /// OpenSim API client for .NET
    /// </summary>
    public class OpenSimClient : IDisposable
    {{
        private readonly HttpClient _httpClient;
        private readonly string _baseUrl;
        private string? _accessToken;

        /// <summary>
        /// Initializes a new instance of the OpenSimClient class
        /// </summary>
        /// <param name="baseUrl">The base URL of the OpenSim API</param>
        public OpenSimClient(string baseUrl)
        {{
            _baseUrl = baseUrl.TrimEnd('/');
            _httpClient = new HttpClient();
            _httpClient.DefaultRequestHeaders.Add("User-Agent", "OpenSim-CSharp-Client/1.0");
        }}

        /// <summary>
        /// Sets the access token for authentication
        /// </summary>
        /// <param name="token">The access token</param>
        public void SetAccessToken(string token)
        {{
            _accessToken = token;
            _httpClient.DefaultRequestHeaders.Authorization = 
                new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);
        }}

        /// <summary>
        /// Authenticates with username and password
        /// </summary>
        /// <param name="username">Username</param>
        /// <param name="password">Password</param>
        /// <returns>Authentication response</returns>
        public async Task<AuthResponse> AuthenticateAsync(string username, string password)
        {{
            var request = new AuthRequest
            {{
                Username = username,
                Password = password
            }};

            var response = await PostAsync<AuthRequest, AuthResponse>("/auth/login", request);
            SetAccessToken(response.AccessToken);
            return response;
        }}

        /// <summary>
        /// Gets user profile by ID
        /// </summary>
        /// <param name="userId">User ID</param>
        /// <returns>User profile</returns>
        public async Task<UserProfile> GetUserProfileAsync(string userId)
        {{
            return await GetAsync<UserProfile>($"/users/{{userId}}");
        }}

        /// <summary>
        /// Lists available regions
        /// </summary>
        /// <param name="limit">Maximum number of regions to return</param>
        /// <returns>List of regions</returns>
        public async Task<List<Region>> ListRegionsAsync(int? limit = null)
        {{
            var url = "/regions";
            if (limit.HasValue)
            {{
                url += $"?limit={{limit.Value}}";
            }}
            
            var response = await GetAsync<RegionListResponse>(url);
            return response.Regions;
        }}

        private async Task<T> GetAsync<T>(string path)
        {{
            var response = await _httpClient.GetAsync(_baseUrl + path);
            response.EnsureSuccessStatusCode();
            var json = await response.Content.ReadAsStringAsync();
            return JsonConvert.DeserializeObject<T>(json) ?? throw new InvalidOperationException("Failed to deserialize response");
        }}

        private async Task<TResponse> PostAsync<TRequest, TResponse>(string path, TRequest request)
        {{
            var json = JsonConvert.SerializeObject(request);
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            
            var response = await _httpClient.PostAsync(_baseUrl + path, content);
            response.EnsureSuccessStatusCode();
            
            var responseJson = await response.Content.ReadAsStringAsync();
            return JsonConvert.DeserializeObject<TResponse>(responseJson) ?? throw new InvalidOperationException("Failed to deserialize response");
        }}

        /// <summary>
        /// Disposes the client
        /// </summary>
        public void Dispose()
        {{
            _httpClient?.Dispose();
        }}
    }}

    // Data models
    public class AuthRequest
    {{
        [JsonProperty("username")]
        public string Username {{ get; set; }} = string.Empty;

        [JsonProperty("password")]
        public string Password {{ get; set; }} = string.Empty;
    }}

    public class AuthResponse
    {{
        [JsonProperty("access_token")]
        public string AccessToken {{ get; set; }} = string.Empty;

        [JsonProperty("refresh_token")]
        public string RefreshToken {{ get; set; }} = string.Empty;

        [JsonProperty("expires_in")]
        public int ExpiresIn {{ get; set; }}

        [JsonProperty("token_type")]
        public string TokenType {{ get; set; }} = string.Empty;
    }}

    public class UserProfile
    {{
        [JsonProperty("id")]
        public string Id {{ get; set; }} = string.Empty;

        [JsonProperty("username")]
        public string Username {{ get; set; }} = string.Empty;

        [JsonProperty("email")]
        public string? Email {{ get; set; }}

        [JsonProperty("created_at")]
        public DateTime CreatedAt {{ get; set; }}

        [JsonProperty("last_login")]
        public DateTime? LastLogin {{ get; set; }}
    }}

    public class Region
    {{
        [JsonProperty("id")]
        public string Id {{ get; set; }} = string.Empty;

        [JsonProperty("name")]
        public string Name {{ get; set; }} = string.Empty;

        [JsonProperty("position")]
        public Vector2 Position {{ get; set; }} = new Vector2();

        [JsonProperty("size")]
        public Vector2 Size {{ get; set; }} = new Vector2();

        [JsonProperty("online_users")]
        public int OnlineUsers {{ get; set; }}
    }}

    public class Vector2
    {{
        [JsonProperty("x")]
        public float X {{ get; set; }}

        [JsonProperty("y")]
        public float Y {{ get; set; }}
    }}

    public class RegionListResponse
    {{
        [JsonProperty("regions")]
        public List<Region> Regions {{ get; set; }} = new List<Region>();

        [JsonProperty("total")]
        public int Total {{ get; set; }}
    }}
}}
"#, namespace);

        generated_files.push(GeneratedFile {
            path: PathBuf::from("OpenSimClient.cs"),
            content: client_content,
            file_type: GeneratedFileType::SourceCode,
        });

        // Generate project file
        generated_files.push(GeneratedFile {
            path: PathBuf::from(format!("{}.csproj", config.package_name)),
            content: self.generate_csproj(config),
            file_type: GeneratedFileType::Configuration,
        });

        // Generate README if requested
        if config.include_documentation {
            let readme_content = format!(r#"# {}

OpenSim client library for .NET.

## Installation

```xml
<PackageReference Include="{}" Version="{}" />
```

## Usage

```csharp
using {};

var client = new OpenSimClient("https://api.opensim.org");

// Authenticate
var auth = await client.AuthenticateAsync("username", "password");
Console.WriteLine($"Authenticated: {{auth.AccessToken}}");

// Use the client...
```

## License

{}
"#,
                config.package_name,
                config.package_name,
                config.package_version,
                namespace,
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
            description: "OpenSim client library for .NET".to_string(),
            main_file: Some(PathBuf::from("OpenSimClient.cs")),
            entry_points: vec!["OpenSimClient".to_string()],
            dependencies: vec!["Newtonsoft.Json".to_string(), "System.Net.Http".to_string()],
        };

        Ok(GenerationResult {
            target_language: TargetLanguage::CSharp,
            generated_files,
            package_info,
            warnings: vec![],
            errors: vec![],
        })
    }

    fn generate_data_types(&self, _types: &[DataTypeSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // C# classes would be generated here
        Ok(vec![])
    }

    fn generate_api_methods(&self, _endpoints: &[EndpointSchema], _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        // API method implementations would be generated here
        Ok(vec![])
    }

    fn generate_authentication(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<GeneratedFile> {
        let content = "// Authentication methods would be generated here";
        Ok(GeneratedFile {
            path: PathBuf::from("Auth.cs"),
            content: content.to_string(),
            file_type: GeneratedFileType::SourceCode,
        })
    }

    fn generate_configuration(&self, config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![GeneratedFile {
            path: PathBuf::from(format!("{}.csproj", config.package_name)),
            content: self.generate_csproj(config),
            file_type: GeneratedFileType::Configuration,
        }])
    }

    fn generate_examples(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let example_content = "// Examples would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("Examples/BasicExample.cs"),
            content: example_content.to_string(),
            file_type: GeneratedFileType::Example,
        }])
    }

    fn generate_tests(&self, _schema: &APISchema, _config: &GeneratorConfig) -> Result<Vec<GeneratedFile>> {
        let test_content = "// Tests would be generated here";
        Ok(vec![GeneratedFile {
            path: PathBuf::from("Tests/ClientTests.cs"),
            content: test_content.to_string(),
            file_type: GeneratedFileType::Test,
        }])
    }

    fn get_file_extension(&self) -> &'static str {
        ".cs"
    }

    fn get_package_manager_config(&self, config: &GeneratorConfig) -> Result<GeneratedFile> {
        Ok(GeneratedFile {
            path: PathBuf::from(format!("{}.csproj", config.package_name)),
            content: self.generate_csproj(config),
            file_type: GeneratedFileType::Configuration,
        })
    }
}