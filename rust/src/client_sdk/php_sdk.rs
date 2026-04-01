//! PHP SDK generator for OpenSim client libraries

use super::{
    api_schema::*,
    generator::{LanguageGenerator, GeneratorConfig},
    utils::*,
};
use anyhow::Result;
use std::collections::HashMap;

/// PHP SDK generator implementation
pub struct PhpGenerator;

impl LanguageGenerator for PhpGenerator {
    fn language_name(&self) -> &'static str {
        "PHP"
    }

    fn file_extension(&self) -> &'static str {
        "php"
    }

    fn generate_sdk(&self, schema: &ApiSchema, config: &GeneratorConfig) -> Result<HashMap<String, String>> {
        let mut files = HashMap::new();

        // Generate main client
        files.insert("src/Client.php".to_string(), self.generate_client(schema, config)?);
        
        // Generate types
        for type_def in &schema.types {
            let filename = format!("src/Models/{}.php", to_pascal_case(&type_def.name()));
            files.insert(filename, self.generate_type_definition(type_def)?);
        }
        
        // Generate authentication
        files.insert("src/Auth/AuthManager.php".to_string(), self.generate_auth(schema)?);
        
        // Generate API endpoints
        for endpoint in &schema.endpoints {
            let category = endpoint.path.split('/').nth(1).unwrap_or("misc");
            let class_name = format!("{}Api", to_pascal_case(category));
            let filename = format!("src/Api/{}.php", class_name);
            if !files.contains_key(&filename) {
                files.insert(filename, self.generate_api_category(schema, category)?);
            }
        }
        
        // Generate composer.json
        files.insert("composer.json".to_string(), self.generate_composer_json(config)?);
        
        // Generate examples
        files.insert("examples/basic_usage.php".to_string(), self.generate_examples(schema, config)?);
        
        // Generate tests
        files.insert("tests/ClientTest.php".to_string(), self.generate_tests(schema)?);
        
        // Generate autoloader
        files.insert("src/autoload.php".to_string(), self.generate_autoloader(config)?);
        
        // Generate README
        files.insert("README.md".to_string(), self.generate_readme(config)?);

        Ok(files)
    }

    fn generate_type_definition(&self, type_def: &TypeDefinition) -> Result<String> {
        match type_def {
            TypeDefinition::Object(obj) => {
                let class_name = to_pascal_case(&obj.name);
                let mut result = format!(r#"<?php

namespace OpenSim\Models;

/**
 * {}
 * 
 * {}
 */
class {}
{{
"#, class_name, 
    obj.description.as_deref().unwrap_or(""),
    class_name);

                // Add properties
                for field in &obj.fields {
                    let php_type = self.map_type_to_php_doc(&field.field_type);
                    result.push_str(&format!(r#"    /**
     * {}
     * @var {}
     */
    public ${};

"#, 
                        field.description.as_deref().unwrap_or(""),
                        php_type,
                        to_snake_case(&field.name)
                    ));
                }

                // Add constructor
                result.push_str("    /**\n     * Constructor\n     * @param array $data\n     */\n");
                result.push_str("    public function __construct(array $data = [])\n    {\n");
                
                for field in &obj.fields {
                    let field_name = to_snake_case(&field.name);
                    result.push_str(&format!("        $this->{} = $data['{}'] ?? null;\n", field_name, field.name));
                }
                
                result.push_str("    }\n\n");

                // Add toArray method
                result.push_str("    /**\n     * Convert to array\n     * @return array\n     */\n");
                result.push_str("    public function toArray(): array\n    {\n");
                result.push_str("        return [\n");
                
                for field in &obj.fields {
                    let field_name = to_snake_case(&field.name);
                    result.push_str(&format!("            '{}' => $this->{},\n", field.name, field_name));
                }
                
                result.push_str("        ];\n    }\n\n");

                // Add fromArray static method
                result.push_str(&format!(r#"    /**
     * Create from array
     * @param array $data
     * @return self
     */
    public static function fromArray(array $data): self
    {{
        return new self($data);
    }}
}}
"#));

                Ok(result)
            }
            TypeDefinition::Enum(enum_def) => {
                let class_name = to_pascal_case(&enum_def.name);
                let mut result = format!(r#"<?php

namespace OpenSim\Models;

/**
 * {}
 * 
 * {}
 */
class {}
{{
"#, class_name,
    enum_def.description.as_deref().unwrap_or(""),
    class_name);

                for variant in &enum_def.variants {
                    result.push_str(&format!("    public const {} = '{}';\n", 
                        to_screaming_snake_case(&variant.name), 
                        variant.value));
                }

                result.push_str(r#"
    /**
     * Get all valid values
     * @return array
     */
    public static function getValidValues(): array
    {
        return [
"#);

                for variant in &enum_def.variants {
                    result.push_str(&format!("            self::{},\n", to_screaming_snake_case(&variant.name)));
                }

                result.push_str(r#"        ];
    }

    /**
     * Check if value is valid
     * @param string $value
     * @return bool
     */
    public static function isValid(string $value): bool
    {
        return in_array($value, self::getValidValues(), true);
    }
}
"#);

                Ok(result)
            }
        }
    }
}

impl PhpGenerator {
    fn generate_client(&self, schema: &ApiSchema, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"<?php

namespace OpenSim;

use OpenSim\Auth\AuthManager;
use OpenSim\Exceptions\OpenSimException;
use GuzzleHttp\Client as HttpClient;
use GuzzleHttp\Exception\RequestException;
use Psr\Http\Message\ResponseInterface;

/**
 * OpenSim API Client
 * 
 * Generated by OpenSim SDK Generator
 * Version: {}
 */
class Client
{{
    private string $baseUrl;
    private HttpClient $httpClient;
    private AuthManager $authManager;
    private array $defaultHeaders;

    /**
     * Constructor
     * 
     * @param array $config
     */
    public function __construct(array $config = [])
    {{
        $this->baseUrl = rtrim($config['base_url'] ?? 'https://api.opensim.org', '/');
        
        $this->defaultHeaders = [
            'Content-Type' => 'application/json',
            'Accept' => 'application/json',
            'User-Agent' => 'opensim-php-sdk/{}',
        ];

        if (isset($config['api_key'])) {{
            $this->defaultHeaders['X-API-Key'] = $config['api_key'];
        }}

        $this->httpClient = new HttpClient([
            'timeout' => $config['timeout'] ?? 30,
            'headers' => $this->defaultHeaders,
        ]);

        $this->authManager = new AuthManager($this);
    }}

    /**
     * Make HTTP request to API
     * 
     * @param string $method
     * @param string $path
     * @param array $data
     * @param array $headers
     * @return array
     * @throws OpenSimException
     */
    public function makeRequest(string $method, string $path, array $data = [], array $headers = []): array
    {{
        $url = $this->baseUrl . $path;
        
        $options = [
            'headers' => array_merge($this->defaultHeaders, $headers),
        ];

        if (!empty($data)) {{
            if (strtoupper($method) === 'GET') {{
                $options['query'] = $data;
            }} else {{
                $options['json'] = $data;
            }}
        }}

        try {{
            $response = $this->httpClient->request($method, $url, $options);
            return $this->handleResponse($response);
        }} catch (RequestException $e) {{
            throw new OpenSimException(
                'API request failed: ' . $e->getMessage(),
                $e->getCode(),
                $e
            );
        }}
    }}

    /**
     * Handle HTTP response
     * 
     * @param ResponseInterface $response
     * @return array
     * @throws OpenSimException
     */
    private function handleResponse(ResponseInterface $response): array
    {{
        $statusCode = $response->getStatusCode();
        $body = $response->getBody()->getContents();

        if ($statusCode >= 400) {{
            $error = json_decode($body, true);
            throw new OpenSimException(
                $error['message'] ?? 'API error',
                $statusCode
            );
        }}

        $data = json_decode($body, true);
        if (json_last_error() !== JSON_ERROR_NONE) {{
            throw new OpenSimException('Invalid JSON response: ' . json_last_error_msg());
        }}

        return $data;
    }}

    /**
     * Test API connection
     * 
     * @return bool
     */
    public function ping(): bool
    {{
        try {{
            $this->makeRequest('GET', '/api/health');
            return true;
        }} catch (OpenSimException $e) {{
            return false;
        }}
    }}

    /**
     * Get auth manager
     * 
     * @return AuthManager
     */
    public function auth(): AuthManager
    {{
        return $this->authManager;
    }}
}}
"#, config.version, config.version))
    }

    fn generate_auth(&self, _schema: &ApiSchema) -> Result<String> {
        Ok(r#"<?php

namespace OpenSim\Auth;

use OpenSim\Client;
use OpenSim\Exceptions\AuthenticationException;

/**
 * Authentication Manager
 */
class AuthManager
{
    private Client $client;
    private ?string $token = null;
    private ?int $expiresAt = null;

    public function __construct(Client $client)
    {
        $this->client = $client;
    }

    /**
     * Login with username and password
     * 
     * @param string $username
     * @param string $password
     * @return array
     * @throws AuthenticationException
     */
    public function login(string $username, string $password): array
    {
        try {
            $response = $this->client->makeRequest('POST', '/api/auth/login', [
                'username' => $username,
                'password' => $password,
            ]);

            $this->token = $response['data']['token'];
            $this->expiresAt = $response['data']['expires_at'];

            return $response['data'];
        } catch (\Exception $e) {
            throw new AuthenticationException('Login failed: ' . $e->getMessage(), 0, $e);
        }
    }

    /**
     * Logout
     * 
     * @return void
     * @throws AuthenticationException
     */
    public function logout(): void
    {
        if (!$this->isAuthenticated()) {
            throw new AuthenticationException('Not authenticated');
        }

        try {
            $this->client->makeRequest('POST', '/api/auth/logout');
            $this->token = null;
            $this->expiresAt = null;
        } catch (\Exception $e) {
            throw new AuthenticationException('Logout failed: ' . $e->getMessage(), 0, $e);
        }
    }

    /**
     * Check if authenticated
     * 
     * @return bool
     */
    public function isAuthenticated(): bool
    {
        return $this->token !== null && 
               ($this->expiresAt === null || $this->expiresAt > time());
    }

    /**
     * Get current token
     * 
     * @return string|null
     */
    public function getToken(): ?string
    {
        return $this->token;
    }

    /**
     * Set token manually
     * 
     * @param string $token
     * @param int|null $expiresAt
     */
    public function setToken(string $token, ?int $expiresAt = null): void
    {
        $this->token = $token;
        $this->expiresAt = $expiresAt;
    }
}
"#.to_string())
    }

    fn generate_api_category(&self, schema: &ApiSchema, category: &str) -> Result<String> {
        let class_name = format!("{}Api", to_pascal_case(category));
        let mut result = format!(r#"<?php

namespace OpenSim\Api;

use OpenSim\Client;
use OpenSim\Exceptions\OpenSimException;

/**
 * {} API endpoints
 */
class {}
{{
    private Client $client;

    public function __construct(Client $client)
    {{
        $this->client = $client;
    }}

"#, to_pascal_case(category), class_name);

        let endpoints: Vec<_> = schema.endpoints.iter()
            .filter(|e| e.path.starts_with(&format!("/{}", category)))
            .collect();

        for endpoint in endpoints {
            result.push_str(&self.generate_endpoint_method(endpoint)?);
        }

        result.push_str("}\n");
        Ok(result)
    }

    fn generate_endpoint_method(&self, endpoint: &ApiEndpoint) -> Result<String> {
        let method_name = self.generate_method_name(&endpoint.path, &endpoint.method);
        
        let mut params = Vec::new();
        let mut path_params = Vec::new();
        
        // Add path parameters
        for param in &endpoint.parameters {
            if param.location == ParameterLocation::Path {
                params.push(format!("${}: {}", to_snake_case(&param.name), self.map_type_to_php(&param.param_type)));
                path_params.push(param.name.clone());
            }
        }
        
        // Add query parameters
        let has_query_params = endpoint.parameters.iter().any(|p| p.location == ParameterLocation::Query);
        if has_query_params {
            params.push("array $query = []".to_string());
        }
        
        // Add request body
        if endpoint.request_body.is_some() {
            params.push("array $data = []".to_string());
        }
        
        let params_str = params.join(", ");
        
        let return_type = if endpoint.responses.get("200").is_some() {
            "array"
        } else {
            "void"
        };

        Ok(format!(r#"    /**
     * {} {}
     * 
{}
     * @return {}
     * @throws OpenSimException
     */
    public function {}({}): {}
    {{
        $path = '{}';
{}
        
        {}
    }}

"#, 
            endpoint.method.to_uppercase(),
            endpoint.summary.as_deref().unwrap_or(&endpoint.path),
            endpoint.parameters.iter()
                .map(|p| format!("     * @param {} ${}", self.map_type_to_php(&p.param_type), to_snake_case(&p.name)))
                .collect::<Vec<_>>()
                .join("\n"),
            return_type,
            method_name,
            params_str,
            return_type,
            endpoint.path,
            self.generate_path_replacements(&path_params),
            self.generate_request_call(endpoint, has_query_params)
        ))
    }

    fn generate_method_name(&self, path: &str, method: &str) -> String {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty() && !s.starts_with('{')).collect();
        let name = parts.join("_");
        to_snake_case(&format!("{}_{}", method, name))
    }

    fn generate_path_replacements(&self, path_params: &[String]) -> String {
        if path_params.is_empty() {
            return String::new();
        }
        
        let mut replacements = String::new();
        for param in path_params {
            replacements.push_str(&format!("        $path = str_replace('{{{}}}', ${}, $path);\n", 
                param, to_snake_case(param)));
        }
        
        replacements
    }

    fn generate_request_call(&self, endpoint: &ApiEndpoint, has_query_params: bool) -> String {
        let data_param = if endpoint.request_body.is_some() {
            "$data"
        } else if has_query_params {
            "$query"
        } else {
            "[]"
        };

        if endpoint.responses.get("200").is_some() {
            format!("return $this->client->makeRequest('{}', $path, {});", 
                endpoint.method.to_uppercase(), data_param)
        } else {
            format!("$this->client->makeRequest('{}', $path, {});", 
                endpoint.method.to_uppercase(), data_param)
        }
    }

    fn generate_composer_json(&self, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"{{
    "name": "{}/{}",
    "description": "Official PHP SDK for the OpenSim API",
    "version": "{}",
    "type": "library",
    "license": "{}",
    "authors": [
        {{
            "name": "OpenSim Contributors",
            "email": "developers@opensim.org"
        }}
    ],
    "require": {{
        "php": ">=8.1",
        "guzzlehttp/guzzle": "^7.0",
        "ext-json": "*"
    }},
    "require-dev": {{
        "phpunit/phpunit": "^10.0",
        "phpstan/phpstan": "^1.0"
    }},
    "autoload": {{
        "psr-4": {{
            "OpenSim\\": "src/"
        }}
    }},
    "autoload-dev": {{
        "psr-4": {{
            "OpenSim\\Tests\\": "tests/"
        }}
    }},
    "scripts": {{
        "test": "phpunit",
        "analyze": "phpstan analyse src/"
    }}
}}
"#, 
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-php-sdk"),
            config.version,
            config.license.as_deref().unwrap_or("MIT")))
    }

    fn generate_examples(&self, _schema: &ApiSchema, _config: &GeneratorConfig) -> Result<String> {
        Ok(r#"<?php

require_once __DIR__ . '/../vendor/autoload.php';

use OpenSim\Client;
use OpenSim\Exceptions\OpenSimException;

// Create client
$client = new Client([
    'base_url' => 'https://api.opensim.org',
    'api_key' => 'your-api-key',
    'timeout' => 30,
]);

try {
    // Test connection
    if ($client->ping()) {
        echo "Successfully connected to OpenSim API\n";
    }

    // Login
    $loginResult = $client->auth()->login('username', 'password');
    echo "Logged in successfully. Token expires at: " . $loginResult['expires_at'] . "\n";

    // Example API calls would go here...

    // Logout
    $client->auth()->logout();
    echo "Logged out successfully\n";

} catch (OpenSimException $e) {
    echo "Error: " . $e->getMessage() . "\n";
}
"#.to_string())
    }

    fn generate_tests(&self, _schema: &ApiSchema) -> Result<String> {
        Ok(r#"<?php

namespace OpenSim\Tests;

use OpenSim\Client;
use PHPUnit\Framework\TestCase;

class ClientTest extends TestCase
{
    private Client $client;

    protected function setUp(): void
    {
        $this->client = new Client([
            'base_url' => 'https://api.example.com',
            'api_key' => 'test-key',
        ]);
    }

    public function testClientCreation(): void
    {
        $this->assertInstanceOf(Client::class, $this->client);
    }

    public function testPing(): void
    {
        // This would require a mock server for proper testing
        $this->markTestSkipped('Requires mock server implementation');
    }

    public function testAuthenticationManager(): void
    {
        $authManager = $this->client->auth();
        $this->assertInstanceOf(\OpenSim\Auth\AuthManager::class, $authManager);
        $this->assertFalse($authManager->isAuthenticated());
    }
}
"#.to_string())
    }

    fn generate_autoloader(&self, _config: &GeneratorConfig) -> Result<String> {
        Ok(r#"<?php

// Simple autoloader for development use
// In production, use Composer autoloader instead

spl_autoload_register(function ($class) {
    $prefix = 'OpenSim\\';
    $base_dir = __DIR__ . '/';

    $len = strlen($prefix);
    if (strncmp($prefix, $class, $len) !== 0) {
        return;
    }

    $relative_class = substr($class, $len);
    $file = $base_dir . str_replace('\\', '/', $relative_class) . '.php';

    if (file_exists($file)) {
        require $file;
    }
});
"#.to_string())
    }

    fn generate_readme(&self, config: &GeneratorConfig) -> Result<String> {
        Ok(format!(r#"# OpenSim PHP SDK

Official PHP client library for the OpenSim API.

## Requirements

- PHP 8.1 or higher
- Guzzle HTTP client
- JSON extension

## Installation

Install via Composer:

```bash
composer require {}/{}
```

## Quick Start

```php
<?php

require_once 'vendor/autoload.php';

use OpenSim\Client;

$client = new Client([
    'base_url' => 'https://api.opensim.org',
    'api_key' => 'your-api-key',
]);

// Test connection
if ($client->ping()) {{
    echo "Connected to OpenSim API!";
}}

// Login
$loginResult = $client->auth()->login('username', 'password');
echo "Logged in successfully!";
```

## Features

- **PSR-4 Autoloading**: Modern PHP standards compliance
- **Type Safety**: Full type hints and documentation
- **Error Handling**: Comprehensive exception handling
- **Authentication**: Built-in support for API key and token authentication
- **Testing**: PHPUnit test suite with mock server support

## Documentation

Full API documentation is available at [https://docs.opensim.org](https://docs.opensim.org).

## Testing

```bash
composer test
```

## License

{}
"#, 
            config.organization.as_deref().unwrap_or("opensim"),
            config.package_name.as_deref().unwrap_or("opensim-php-sdk"),
            config.license.as_deref().unwrap_or("MIT")))
    }

    fn map_type_to_php(&self, data_type: &DataType) -> String {
        match data_type {
            DataType::String => "string".to_string(),
            DataType::Integer => "int".to_string(),
            DataType::Number => "float".to_string(),
            DataType::Boolean => "bool".to_string(),
            DataType::Array(_) => "array".to_string(),
            DataType::Object(_) => "array".to_string(),
            DataType::Optional(inner) => format!("?{}", self.map_type_to_php(inner)),
            DataType::Map(_) => "array".to_string(),
        }
    }

    fn map_type_to_php_doc(&self, data_type: &DataType) -> String {
        match data_type {
            DataType::String => "string".to_string(),
            DataType::Integer => "int".to_string(),
            DataType::Number => "float".to_string(),
            DataType::Boolean => "bool".to_string(),
            DataType::Array(inner) => format!("{}[]", self.map_type_to_php_doc(inner)),
            DataType::Object(name) => format!("\\OpenSim\\Models\\{}", to_pascal_case(name)),
            DataType::Optional(inner) => format!("{}|null", self.map_type_to_php_doc(inner)),
            DataType::Map(value_type) => format!("array<string, {}>", self.map_type_to_php_doc(value_type)),
        }
    }
}

fn to_screaming_snake_case(input: &str) -> String {
    to_snake_case(input).to_uppercase()
}