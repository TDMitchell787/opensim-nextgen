//! API schema definitions for SDK generation

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Complete API schema for the OpenSim server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APISchema {
    pub version: String,
    pub base_url: String,
    pub authentication: AuthenticationSchema,
    pub endpoints: Vec<EndpointSchema>,
    pub data_types: Vec<DataTypeSchema>,
    pub error_codes: Vec<ErrorCodeSchema>,
    pub rate_limits: RateLimitSchema,
    pub webhooks: Vec<WebhookSchema>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSchema {
    pub auth_type: AuthenticationType,
    pub token_endpoint: Option<String>,
    pub refresh_endpoint: Option<String>,
    pub scopes: Vec<String>,
    pub headers: HashMap<String, String>,
}

/// Types of authentication supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationType {
    Bearer,
    ApiKey,
    OAuth2,
    Basic,
    Custom { scheme: String },
}

/// API endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSchema {
    pub id: String,
    pub name: String,
    pub description: String,
    pub method: HttpMethod,
    pub path: String,
    pub path_parameters: Vec<ParameterSchema>,
    pub query_parameters: Vec<ParameterSchema>,
    pub request_body: Option<DataTypeReference>,
    pub response_body: DataTypeReference,
    pub error_responses: Vec<ErrorResponseSchema>,
    pub rate_limit: Option<String>,
    pub authentication_required: bool,
    pub scopes_required: Vec<String>,
    pub deprecated: bool,
    pub examples: Vec<ExampleSchema>,
}

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub description: String,
    pub parameter_type: DataTypeReference,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ValidationSchema>,
}

/// Data type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypeSchema {
    pub name: String,
    pub description: String,
    pub type_definition: TypeDefinition,
    pub examples: Vec<serde_json::Value>,
    pub deprecated: bool,
}

/// Type definition variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDefinition {
    Primitive { primitive_type: PrimitiveType },
    Array { item_type: Box<DataTypeReference> },
    Object { properties: Vec<PropertySchema> },
    Union { variants: Vec<DataTypeReference> },
    Enum { variants: Vec<EnumVariant> },
    Map { key_type: Box<DataTypeReference>, value_type: Box<DataTypeReference> },
}

/// Primitive data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    UUID,
    Binary,
}

/// Object property definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub name: String,
    pub description: String,
    pub property_type: DataTypeReference,
    pub required: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub validation: Option<ValidationSchema>,
}

/// Enum variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: serde_json::Value,
    pub description: String,
}

/// Reference to a data type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypeReference {
    pub type_name: String,
    pub nullable: bool,
    pub array: bool,
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSchema {
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub pattern: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub format: Option<String>,
}

/// Error response definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponseSchema {
    pub status_code: u16,
    pub error_type: String,
    pub description: String,
    pub response_body: Option<DataTypeReference>,
}

/// Error code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeSchema {
    pub code: String,
    pub http_status: u16,
    pub message: String,
    pub description: String,
    pub resolution_steps: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSchema {
    pub default_limit: u32,
    pub default_window_seconds: u32,
    pub endpoint_limits: HashMap<String, EndpointRateLimit>,
    pub headers: Vec<String>,
}

/// Per-endpoint rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimit {
    pub requests_per_window: u32,
    pub window_seconds: u32,
    pub burst_limit: Option<u32>,
}

/// Webhook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSchema {
    pub event_type: String,
    pub description: String,
    pub payload_schema: DataTypeReference,
    pub retry_policy: RetryPolicy,
}

/// Webhook retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_seconds: u32,
    pub max_delay_seconds: u32,
    pub backoff_multiplier: f64,
}

/// API usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleSchema {
    pub name: String,
    pub description: String,
    pub request: ExampleRequest,
    pub response: ExampleResponse,
}

/// Example request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleRequest {
    pub path_parameters: HashMap<String, serde_json::Value>,
    pub query_parameters: HashMap<String, serde_json::Value>,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

/// Example response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}

impl APISchema {
    /// Create a comprehensive API schema for OpenSim
    pub fn create_opensim_schema() -> Self {
        Self {
            version: "1.0.0".to_string(),
            base_url: "https://api.opensim.org".to_string(),
            authentication: AuthenticationSchema {
                auth_type: AuthenticationType::Bearer,
                token_endpoint: Some("/auth/token".to_string()),
                refresh_endpoint: Some("/auth/refresh".to_string()),
                scopes: vec![
                    "read:regions".to_string(),
                    "write:regions".to_string(),
                    "read:users".to_string(),
                    "write:users".to_string(),
                    "read:assets".to_string(),
                    "write:assets".to_string(),
                    "admin".to_string(),
                ],
                headers: HashMap::from([
                    ("Authorization".to_string(), "Bearer {token}".to_string()),
                    ("User-Agent".to_string(), "OpenSim-SDK/{version}".to_string()),
                ]),
            },
            endpoints: Self::create_endpoints(),
            data_types: Self::create_data_types(),
            error_codes: Self::create_error_codes(),
            rate_limits: RateLimitSchema {
                default_limit: 1000,
                default_window_seconds: 3600,
                endpoint_limits: HashMap::from([
                    ("auth".to_string(), EndpointRateLimit {
                        requests_per_window: 10,
                        window_seconds: 60,
                        burst_limit: Some(5),
                    }),
                    ("assets".to_string(), EndpointRateLimit {
                        requests_per_window: 100,
                        window_seconds: 60,
                        burst_limit: Some(20),
                    }),
                ]),
                headers: vec![
                    "X-RateLimit-Limit".to_string(),
                    "X-RateLimit-Remaining".to_string(),
                    "X-RateLimit-Reset".to_string(),
                ],
            },
            webhooks: Self::create_webhooks(),
        }
    }

    fn create_endpoints() -> Vec<EndpointSchema> {
        vec![
            // Authentication endpoints
            EndpointSchema {
                id: "auth_login".to_string(),
                name: "User Login".to_string(),
                description: "Authenticate user and receive access token".to_string(),
                method: HttpMethod::POST,
                path: "/auth/login".to_string(),
                path_parameters: vec![],
                query_parameters: vec![],
                request_body: Some(DataTypeReference {
                    type_name: "LoginRequest".to_string(),
                    nullable: false,
                    array: false,
                }),
                response_body: DataTypeReference {
                    type_name: "AuthResponse".to_string(),
                    nullable: false,
                    array: false,
                },
                error_responses: vec![
                    ErrorResponseSchema {
                        status_code: 401,
                        error_type: "INVALID_CREDENTIALS".to_string(),
                        description: "Invalid username or password".to_string(),
                        response_body: Some(DataTypeReference {
                            type_name: "ErrorResponse".to_string(),
                            nullable: false,
                            array: false,
                        }),
                    },
                ],
                rate_limit: Some("auth".to_string()),
                authentication_required: false,
                scopes_required: vec![],
                deprecated: false,
                examples: vec![
                    ExampleSchema {
                        name: "Basic Login".to_string(),
                        description: "Login with username and password".to_string(),
                        request: ExampleRequest {
                            path_parameters: HashMap::new(),
                            query_parameters: HashMap::new(),
                            headers: HashMap::from([
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ]),
                            body: Some(serde_json::json!({
                                "username": "john_doe",
                                "password": "secure_password123"
                            })),
                        },
                        response: ExampleResponse {
                            status_code: 200,
                            headers: HashMap::from([
                                ("Content-Type".to_string(), "application/json".to_string()),
                            ]),
                            body: serde_json::json!({
                                "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                                "refresh_token": "def50200a1b2c3d4e5f6...",
                                "expires_in": 3600,
                                "token_type": "Bearer"
                            }),
                        },
                    },
                ],
            },
            
            // Region endpoints
            EndpointSchema {
                id: "list_regions".to_string(),
                name: "List Regions".to_string(),
                description: "Get a list of all available regions".to_string(),
                method: HttpMethod::GET,
                path: "/regions".to_string(),
                path_parameters: vec![],
                query_parameters: vec![
                    ParameterSchema {
                        name: "limit".to_string(),
                        description: "Maximum number of regions to return".to_string(),
                        parameter_type: DataTypeReference {
                            type_name: "Integer".to_string(),
                            nullable: true,
                            array: false,
                        },
                        required: false,
                        default_value: Some(serde_json::Value::Number(serde_json::Number::from(50))),
                        validation: Some(ValidationSchema {
                            minimum: Some(1.0),
                            maximum: Some(1000.0),
                            min_length: None,
                            max_length: None,
                            pattern: None,
                            format: None,
                        }),
                    },
                ],
                request_body: None,
                response_body: DataTypeReference {
                    type_name: "RegionListResponse".to_string(),
                    nullable: false,
                    array: false,
                },
                error_responses: vec![],
                rate_limit: None,
                authentication_required: true,
                scopes_required: vec!["read:regions".to_string()],
                deprecated: false,
                examples: vec![],
            },

            // User endpoints
            EndpointSchema {
                id: "get_user_profile".to_string(),
                name: "Get User Profile".to_string(),
                description: "Retrieve user profile information".to_string(),
                method: HttpMethod::GET,
                path: "/users/{user_id}".to_string(),
                path_parameters: vec![
                    ParameterSchema {
                        name: "user_id".to_string(),
                        description: "Unique identifier for the user".to_string(),
                        parameter_type: DataTypeReference {
                            type_name: "UUID".to_string(),
                            nullable: false,
                            array: false,
                        },
                        required: true,
                        default_value: None,
                        validation: None,
                    },
                ],
                query_parameters: vec![],
                request_body: None,
                response_body: DataTypeReference {
                    type_name: "UserProfile".to_string(),
                    nullable: false,
                    array: false,
                },
                error_responses: vec![
                    ErrorResponseSchema {
                        status_code: 404,
                        error_type: "USER_NOT_FOUND".to_string(),
                        description: "User with specified ID does not exist".to_string(),
                        response_body: Some(DataTypeReference {
                            type_name: "ErrorResponse".to_string(),
                            nullable: false,
                            array: false,
                        }),
                    },
                ],
                rate_limit: None,
                authentication_required: true,
                scopes_required: vec!["read:users".to_string()],
                deprecated: false,
                examples: vec![],
            },

            // Asset endpoints
            EndpointSchema {
                id: "upload_asset".to_string(),
                name: "Upload Asset".to_string(),
                description: "Upload a new asset to the system".to_string(),
                method: HttpMethod::POST,
                path: "/assets".to_string(),
                path_parameters: vec![],
                query_parameters: vec![],
                request_body: Some(DataTypeReference {
                    type_name: "AssetUploadRequest".to_string(),
                    nullable: false,
                    array: false,
                }),
                response_body: DataTypeReference {
                    type_name: "AssetUploadResponse".to_string(),
                    nullable: false,
                    array: false,
                },
                error_responses: vec![
                    ErrorResponseSchema {
                        status_code: 413,
                        error_type: "ASSET_TOO_LARGE".to_string(),
                        description: "Asset exceeds maximum allowed size".to_string(),
                        response_body: Some(DataTypeReference {
                            type_name: "ErrorResponse".to_string(),
                            nullable: false,
                            array: false,
                        }),
                    },
                ],
                rate_limit: Some("assets".to_string()),
                authentication_required: true,
                scopes_required: vec!["write:assets".to_string()],
                deprecated: false,
                examples: vec![],
            },
        ]
    }

    fn create_data_types() -> Vec<DataTypeSchema> {
        vec![
            DataTypeSchema {
                name: "LoginRequest".to_string(),
                description: "Request payload for user authentication".to_string(),
                type_definition: TypeDefinition::Object {
                    properties: vec![
                        PropertySchema {
                            name: "username".to_string(),
                            description: "User's login name".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: false,
                            write_only: false,
                            validation: Some(ValidationSchema {
                                min_length: Some(3),
                                max_length: Some(50),
                                pattern: Some("^[a-zA-Z0-9_]+$".to_string()),
                                minimum: None,
                                maximum: None,
                                format: None,
                            }),
                        },
                        PropertySchema {
                            name: "password".to_string(),
                            description: "User's password".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: false,
                            write_only: true,
                            validation: Some(ValidationSchema {
                                min_length: Some(8),
                                max_length: Some(128),
                                pattern: None,
                                minimum: None,
                                maximum: None,
                                format: None,
                            }),
                        },
                    ],
                },
                examples: vec![
                    serde_json::json!({
                        "username": "john_doe",
                        "password": "secure_password123"
                    }),
                ],
                deprecated: false,
            },

            DataTypeSchema {
                name: "AuthResponse".to_string(),
                description: "Response containing authentication tokens".to_string(),
                type_definition: TypeDefinition::Object {
                    properties: vec![
                        PropertySchema {
                            name: "access_token".to_string(),
                            description: "JWT access token".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "refresh_token".to_string(),
                            description: "Refresh token for obtaining new access tokens".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "expires_in".to_string(),
                            description: "Token expiration time in seconds".to_string(),
                            property_type: DataTypeReference {
                                type_name: "Integer".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "token_type".to_string(),
                            description: "Type of token (always 'Bearer')".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                    ],
                },
                examples: vec![
                    serde_json::json!({
                        "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
                        "refresh_token": "def50200a1b2c3d4e5f6...",
                        "expires_in": 3600,
                        "token_type": "Bearer"
                    }),
                ],
                deprecated: false,
            },

            DataTypeSchema {
                name: "UserProfile".to_string(),
                description: "User profile information".to_string(),
                type_definition: TypeDefinition::Object {
                    properties: vec![
                        PropertySchema {
                            name: "id".to_string(),
                            description: "Unique user identifier".to_string(),
                            property_type: DataTypeReference {
                                type_name: "UUID".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "username".to_string(),
                            description: "User's login name".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "email".to_string(),
                            description: "User's email address".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: true,
                                array: false,
                            },
                            required: false,
                            read_only: false,
                            write_only: false,
                            validation: Some(ValidationSchema {
                                pattern: Some(r"^[^@\s]+@[^@\s]+\.[^@\s]+$".to_string()),
                                format: Some("email".to_string()),
                                min_length: None,
                                max_length: None,
                                minimum: None,
                                maximum: None,
                            }),
                        },
                        PropertySchema {
                            name: "created_at".to_string(),
                            description: "Account creation timestamp".to_string(),
                            property_type: DataTypeReference {
                                type_name: "DateTime".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "last_login".to_string(),
                            description: "Last login timestamp".to_string(),
                            property_type: DataTypeReference {
                                type_name: "DateTime".to_string(),
                                nullable: true,
                                array: false,
                            },
                            required: false,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                    ],
                },
                examples: vec![
                    serde_json::json!({
                        "id": "123e4567-e89b-12d3-a456-426614174000",
                        "username": "john_doe",
                        "email": "john@example.com",
                        "created_at": "2024-01-01T00:00:00Z",
                        "last_login": "2024-06-20T12:00:00Z"
                    }),
                ],
                deprecated: false,
            },

            DataTypeSchema {
                name: "Region".to_string(),
                description: "Virtual world region information".to_string(),
                type_definition: TypeDefinition::Object {
                    properties: vec![
                        PropertySchema {
                            name: "id".to_string(),
                            description: "Unique region identifier".to_string(),
                            property_type: DataTypeReference {
                                type_name: "UUID".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "name".to_string(),
                            description: "Region display name".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: false,
                            write_only: false,
                            validation: Some(ValidationSchema {
                                min_length: Some(1),
                                max_length: Some(64),
                                pattern: None,
                                minimum: None,
                                maximum: None,
                                format: None,
                            }),
                        },
                        PropertySchema {
                            name: "position".to_string(),
                            description: "Region position in grid coordinates".to_string(),
                            property_type: DataTypeReference {
                                type_name: "Vector2".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: false,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "size".to_string(),
                            description: "Region size in meters".to_string(),
                            property_type: DataTypeReference {
                                type_name: "Vector2".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: false,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "online_users".to_string(),
                            description: "Number of users currently in the region".to_string(),
                            property_type: DataTypeReference {
                                type_name: "Integer".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                    ],
                },
                examples: vec![
                    serde_json::json!({
                        "id": "987fcdeb-51a2-4567-8901-23456789abcd",
                        "name": "Welcome Island",
                        "position": {"x": 1000, "y": 1000},
                        "size": {"x": 256, "y": 256},
                        "online_users": 15
                    }),
                ],
                deprecated: false,
            },

            DataTypeSchema {
                name: "ErrorResponse".to_string(),
                description: "Standard error response format".to_string(),
                type_definition: TypeDefinition::Object {
                    properties: vec![
                        PropertySchema {
                            name: "error".to_string(),
                            description: "Error code identifier".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "message".to_string(),
                            description: "Human-readable error message".to_string(),
                            property_type: DataTypeReference {
                                type_name: "String".to_string(),
                                nullable: false,
                                array: false,
                            },
                            required: true,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                        PropertySchema {
                            name: "details".to_string(),
                            description: "Additional error details".to_string(),
                            property_type: DataTypeReference {
                                type_name: "Object".to_string(),
                                nullable: true,
                                array: false,
                            },
                            required: false,
                            read_only: true,
                            write_only: false,
                            validation: None,
                        },
                    ],
                },
                examples: vec![
                    serde_json::json!({
                        "error": "INVALID_CREDENTIALS",
                        "message": "The provided username or password is incorrect",
                        "details": {
                            "code": 1001,
                            "timestamp": "2024-06-20T12:00:00Z"
                        }
                    }),
                ],
                deprecated: false,
            },
        ]
    }

    fn create_error_codes() -> Vec<ErrorCodeSchema> {
        vec![
            ErrorCodeSchema {
                code: "INVALID_CREDENTIALS".to_string(),
                http_status: 401,
                message: "Invalid username or password".to_string(),
                description: "The provided authentication credentials are incorrect".to_string(),
                resolution_steps: vec![
                    "Verify username and password are correct".to_string(),
                    "Check if account is active and not locked".to_string(),
                    "Contact support if problem persists".to_string(),
                ],
            },
            ErrorCodeSchema {
                code: "USER_NOT_FOUND".to_string(),
                http_status: 404,
                message: "User not found".to_string(),
                description: "No user exists with the specified identifier".to_string(),
                resolution_steps: vec![
                    "Verify the user ID is correct".to_string(),
                    "Check if the user account has been deleted".to_string(),
                ],
            },
            ErrorCodeSchema {
                code: "ASSET_TOO_LARGE".to_string(),
                http_status: 413,
                message: "Asset exceeds maximum size limit".to_string(),
                description: "The uploaded asset is larger than the allowed maximum size".to_string(),
                resolution_steps: vec![
                    "Reduce the asset file size".to_string(),
                    "Compress the asset using appropriate tools".to_string(),
                    "Contact admin to increase size limits if necessary".to_string(),
                ],
            },
            ErrorCodeSchema {
                code: "RATE_LIMIT_EXCEEDED".to_string(),
                http_status: 429,
                message: "Rate limit exceeded".to_string(),
                description: "Too many requests have been made in the current time window".to_string(),
                resolution_steps: vec![
                    "Wait before making additional requests".to_string(),
                    "Implement exponential backoff in your client".to_string(),
                    "Consider upgrading to a higher rate limit tier".to_string(),
                ],
            },
        ]
    }

    fn create_webhooks() -> Vec<WebhookSchema> {
        vec![
            WebhookSchema {
                event_type: "user.login".to_string(),
                description: "Triggered when a user successfully logs in".to_string(),
                payload_schema: DataTypeReference {
                    type_name: "UserLoginEvent".to_string(),
                    nullable: false,
                    array: false,
                },
                retry_policy: RetryPolicy {
                    max_attempts: 3,
                    initial_delay_seconds: 60,
                    max_delay_seconds: 3600,
                    backoff_multiplier: 2.0,
                },
            },
            WebhookSchema {
                event_type: "region.user_joined".to_string(),
                description: "Triggered when a user enters a region".to_string(),
                payload_schema: DataTypeReference {
                    type_name: "RegionUserJoinedEvent".to_string(),
                    nullable: false,
                    array: false,
                },
                retry_policy: RetryPolicy {
                    max_attempts: 3,
                    initial_delay_seconds: 30,
                    max_delay_seconds: 1800,
                    backoff_multiplier: 2.0,
                },
            },
            WebhookSchema {
                event_type: "asset.uploaded".to_string(),
                description: "Triggered when a new asset is successfully uploaded".to_string(),
                payload_schema: DataTypeReference {
                    type_name: "AssetUploadedEvent".to_string(),
                    nullable: false,
                    array: false,
                },
                retry_policy: RetryPolicy {
                    max_attempts: 5,
                    initial_delay_seconds: 120,
                    max_delay_seconds: 7200,
                    backoff_multiplier: 1.5,
                },
            },
        ]
    }
}