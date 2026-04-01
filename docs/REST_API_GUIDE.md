# OpenSim Next REST API Guide

The OpenSim Next REST API provides a comprehensive HTTP interface for all virtual world services, enabling modern web applications, mobile apps, and third-party integrations to interact with the virtual world server.

## Overview

OpenSim Next's REST API offers:

- **Complete Virtual World Access**: Full CRUD operations for all virtual world entities
- **Modern HTTP Standards**: RESTful design with JSON payloads and proper HTTP status codes
- **Comprehensive Authentication**: JWT-based authentication with role-based access control
- **Real-time Capabilities**: WebSocket upgrades for real-time communication
- **Developer-Friendly**: OpenAPI/Swagger documentation and client SDKs
- **Production-Ready**: Rate limiting, caching, monitoring, and security features

## Quick Start

### 1. Enable REST API

Edit `config-include/RestAPI.ini`:

```ini
[RestAPI]
Enabled = true
Port = 8080
BindAddress = "0.0.0.0"
RequireAuthentication = true
```

### 2. Start the Server

```bash
cargo run --bin opensim_next
```

### 3. Authenticate

```bash
# Login to get access token
curl -X POST http://localhost:8080/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}'
```

### 4. Use the API

```bash
# List assets (using token from login)
curl -X GET http://localhost:8080/v1/assets \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

## API Architecture

### Base URL Structure

```
{protocol}://{host}:{port}/{version}/{resource}
```

**Examples:**
- `http://localhost:8080/v1/assets`
- `https://api.opensim.example.com/v1/users`
- `http://region-server:8080/v1/regions`

### Response Format

All API responses follow a consistent format:

```json
{
  "success": true,
  "data": {
    // Response data here
  },
  "error": null,
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "v1"
}
```

**Error Response:**
```json
{
  "success": false,
  "data": null,
  "error": "Detailed error message",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "v1"
}
```

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, PUT, DELETE |
| 201 | Created | Successful POST |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Resource already exists or conflict |
| 422 | Unprocessable Entity | Valid format but invalid data |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |

## Authentication

### JWT-Based Authentication

OpenSim Next uses JSON Web Tokens (JWT) for stateless authentication.

#### Login Process

```bash
POST /v1/auth/login
Content-Type: application/json

{
  "username": "your_username",
  "password": "your_password",
  "remember_me": false
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 86400,
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "john_doe",
      "email": "john@example.com",
      "user_level": 10
    }
  }
}
```

#### Using Access Tokens

Include the access token in the Authorization header:

```bash
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Token Refresh

```bash
POST /v1/auth/refresh
Authorization: Bearer {refresh_token}
```

#### Logout

```bash
POST /v1/auth/logout
Authorization: Bearer {access_token}
```

### Permission Levels

| Level | Role | Permissions |
|-------|------|-------------|
| 0 | Guest | Read-only public data |
| 10 | User | Own inventory, basic operations |
| 50 | Moderator | User management, region monitoring |
| 100 | Admin | Full system access |

## API Endpoints

### Asset Management

Assets are digital content items like textures, sounds, meshes, and scripts.

#### List Assets

```bash
GET /v1/assets?page=1&limit=50&asset_type=texture&query=landscape
Authorization: Bearer {token}
```

**Query Parameters:**
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 50, max: 100)
- `query`: Search query string
- `asset_type`: Filter by type (texture, sound, mesh, animation, script, etc.)
- `creator_id`: Filter by creator UUID
- `is_public`: Filter public/private assets
- `tags`: Comma-separated tags

**Response:**
```json
{
  "success": true,
  "data": {
    "assets": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Mountain Landscape",
        "description": "Beautiful mountain scenery texture",
        "asset_type": "texture",
        "content_type": "image/png",
        "size": 2048576,
        "created": "2024-01-15T10:00:00Z",
        "updated": "2024-01-15T10:00:00Z",
        "creator_id": "123e4567-e89b-12d3-a456-426614174000",
        "is_public": true,
        "tags": ["landscape", "mountain", "nature"]
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 50,
      "total": 1,
      "pages": 1
    }
  }
}
```

#### Get Asset

```bash
GET /v1/assets/{asset_id}
Authorization: Bearer {token}
```

#### Upload Asset

```bash
POST /v1/assets
Authorization: Bearer {token}
Content-Type: multipart/form-data

file: <binary_data>
metadata: {
  "name": "My Texture",
  "description": "Custom texture for buildings",
  "asset_type": "texture",
  "is_public": false,
  "tags": ["custom", "building", "texture"]
}
```

#### Download Asset Data

```bash
GET /v1/assets/{asset_id}/data
Authorization: Bearer {token}
```

Returns the binary asset data with appropriate Content-Type header.

#### Update Asset Metadata

```bash
PUT /v1/assets/{asset_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Updated Asset Name",
  "description": "Updated description",
  "is_public": true,
  "tags": ["updated", "tags"]
}
```

#### Delete Asset

```bash
DELETE /v1/assets/{asset_id}
Authorization: Bearer {token}
```

#### Search Assets

```bash
GET /v1/assets/search?query=mountain&asset_type=texture&tags=landscape
Authorization: Bearer {token}
```

### Inventory Management

User inventory contains folders and items that reference assets.

#### List User Inventory Items

```bash
GET /v1/inventory/users/{user_id}/items?page=1&limit=50
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "My Favorite Texture",
        "description": "A texture I use often",
        "asset_id": "123e4567-e89b-12d3-a456-426614174000",
        "folder_id": "987fcdeb-51d2-43f8-b567-123456789abc",
        "owner_id": "456e7890-e12b-34d5-a678-901234567def",
        "creator_id": "123e4567-e89b-12d3-a456-426614174000",
        "item_type": "texture",
        "permissions": {
          "next_perms": 532480,
          "current_perms": 647168,
          "base_perms": 647168,
          "everyone_perms": 0,
          "group_perms": 0
        },
        "created": "2024-01-15T10:00:00Z",
        "updated": "2024-01-15T11:00:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 50,
      "total": 1,
      "pages": 1
    }
  }
}
```

#### List User Inventory Folders

```bash
GET /v1/inventory/users/{user_id}/folders
Authorization: Bearer {token}
```

#### Get Inventory Item

```bash
GET /v1/inventory/items/{item_id}
Authorization: Bearer {token}
```

#### Create Inventory Item

```bash
POST /v1/inventory/users/{user_id}/items
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "New Inventory Item",
  "description": "Description of the item",
  "folder_id": "987fcdeb-51d2-43f8-b567-123456789abc",
  "asset_id": "123e4567-e89b-12d3-a456-426614174000",
  "item_type": "texture"
}
```

#### Update Inventory Item

```bash
PUT /v1/inventory/items/{item_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Updated Item Name",
  "description": "Updated description",
  "folder_id": "new-folder-id"
}
```

#### Delete Inventory Item

```bash
DELETE /v1/inventory/items/{item_id}
Authorization: Bearer {token}
```

#### Create Inventory Folder

```bash
POST /v1/inventory/users/{user_id}/folders
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "New Folder",
  "parent_id": "parent-folder-id",
  "folder_type": "texture"
}
```

### User Management

#### List Users (Admin Only)

```bash
GET /v1/users?page=1&limit=50
Authorization: Bearer {admin_token}
```

#### Get User

```bash
GET /v1/users/{user_id}
Authorization: Bearer {token}
```

#### Get User Profile

```bash
GET /v1/users/{user_id}/profile
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "john_doe",
    "email": "john@example.com",
    "first_name": "John",
    "last_name": "Doe",
    "display_name": "John Doe",
    "profile_image": "https://cdn.example.com/profiles/john_doe.png",
    "created": "2024-01-01T00:00:00Z",
    "last_login": "2024-01-15T10:00:00Z",
    "is_active": true,
    "user_level": 10
  }
}
```

#### Create User (Admin Only)

```bash
POST /v1/users
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "username": "new_user",
  "email": "newuser@example.com",
  "password": "secure_password",
  "first_name": "New",
  "last_name": "User"
}
```

#### Update User Profile

```bash
PUT /v1/users/{user_id}/profile
Authorization: Bearer {token}
Content-Type: application/json

{
  "first_name": "Updated First",
  "last_name": "Updated Last",
  "display_name": "Updated Display Name",
  "profile_image": "https://cdn.example.com/new_image.png"
}
```

#### Delete User (Admin Only)

```bash
DELETE /v1/users/{user_id}
Authorization: Bearer {admin_token}
```

#### Get User Inventory Summary

```bash
GET /v1/users/{user_id}/inventory
Authorization: Bearer {token}
```

### Region Management

#### List Regions

```bash
GET /v1/regions?page=1&limit=50
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "regions": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Welcome Region",
        "location_x": 1000,
        "location_y": 1000,
        "size_x": 256,
        "size_y": 256,
        "estate_id": 1,
        "owner_id": "123e4567-e89b-12d3-a456-426614174000",
        "is_online": true,
        "agent_count": 5,
        "prim_count": 1250,
        "script_count": 45,
        "created": "2024-01-01T00:00:00Z",
        "last_heartbeat": "2024-01-15T10:30:00Z"
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 50,
      "total": 1,
      "pages": 1
    }
  }
}
```

#### Get Region

```bash
GET /v1/regions/{region_id}
Authorization: Bearer {token}
```

#### Create Region (Admin Only)

```bash
POST /v1/regions
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "New Region",
  "location_x": 2000,
  "location_y": 2000,
  "size_x": 256,
  "size_y": 256,
  "estate_id": 1
}
```

#### Update Region (Admin Only)

```bash
PUT /v1/regions/{region_id}
Authorization: Bearer {admin_token}
Content-Type: application/json

{
  "name": "Updated Region Name",
  "location_x": 2100,
  "location_y": 2100
}
```

#### Delete Region (Admin Only)

```bash
DELETE /v1/regions/{region_id}
Authorization: Bearer {admin_token}
```

#### Restart Region (Admin Only)

```bash
POST /v1/regions/{region_id}/restart
Authorization: Bearer {admin_token}
```

#### List Agents in Region

```bash
GET /v1/regions/{region_id}/agents
Authorization: Bearer {token}
```

#### List Objects in Region

```bash
GET /v1/regions/{region_id}/objects?page=1&limit=50
Authorization: Bearer {token}
```

### Statistics and Monitoring

#### Get Overview Statistics

```bash
GET /v1/stats/overview
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "users": {
      "total": 1000,
      "online": 50,
      "new_today": 5
    },
    "regions": {
      "total": 25,
      "online": 24,
      "offline": 1
    },
    "assets": {
      "total": 50000,
      "size_mb": 15000,
      "uploads_today": 25
    },
    "performance": {
      "uptime_seconds": 86400,
      "memory_usage_mb": 2048,
      "cpu_usage_percent": 15.5
    },
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### Get Asset Statistics

```bash
GET /v1/stats/assets
Authorization: Bearer {token}
```

#### Get User Statistics

```bash
GET /v1/stats/users
Authorization: Bearer {token}
```

#### Get Region Statistics

```bash
GET /v1/stats/regions
Authorization: Bearer {token}
```

## Client Tools

### Rust CLI Client

OpenSim Next includes a powerful Rust CLI client:

```bash
# Build the client
cargo build --bin rest_api_client

# Login and get token
cargo run --bin rest_api_client --username admin --password admin auth login

# List assets
cargo run --bin rest_api_client --token YOUR_TOKEN assets list

# Upload asset
cargo run --bin rest_api_client --token YOUR_TOKEN assets upload \
  --file texture.png --name "My Texture" --asset-type texture --public

# Create user (admin only)
cargo run --bin rest_api_client --token ADMIN_TOKEN users create \
  --username newuser --email user@example.com --password password123 \
  --first-name John --last-name Doe

# Get region statistics
cargo run --bin rest_api_client --token YOUR_TOKEN stats regions
```

### Environment Variables

Set common options via environment variables:

```bash
export OPENSIM_API_TOKEN="your_token_here"
export OPENSIM_USERNAME="admin"
export OPENSIM_PASSWORD="admin_password"
export OPENSIM_API_SERVER="http://localhost:8080"

# Now you can use the client without flags
cargo run --bin rest_api_client assets list
```

## Development

### OpenAPI/Swagger Documentation

Access interactive API documentation at:
```
http://localhost:8080/docs
```

### Rate Limiting

The API implements rate limiting to prevent abuse:

- **Default**: 100 requests per minute per IP address
- **Authenticated Users**: Higher limits based on user level
- **Rate Limit Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### Caching

Response caching improves performance:

- **Asset Metadata**: 5 minutes
- **User Profiles**: 15 minutes
- **Region Information**: 2 minutes
- **Statistics**: 1 minute

Cache headers: `Cache-Control`, `ETag`, `Last-Modified`

### Error Handling

Comprehensive error responses with debug information:

```json
{
  "success": false,
  "error": "Validation failed",
  "details": {
    "field": "email",
    "message": "Invalid email format",
    "code": "VALIDATION_ERROR"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "v1"
}
```

### Monitoring and Metrics

Prometheus metrics available at `/metrics`:

- Request duration histograms
- Request count by endpoint and status
- Authentication success/failure rates
- Active sessions count
- Cache hit/miss ratios

## Security Best Practices

### 1. Authentication

- Always use HTTPS in production
- Store JWT secrets securely
- Implement token rotation
- Use strong passwords

### 2. Authorization

- Follow principle of least privilege
- Validate user permissions for each request
- Log access attempts
- Implement session management

### 3. Input Validation

- Validate all input parameters
- Sanitize file uploads
- Check file types and sizes
- Prevent SQL injection

### 4. Rate Limiting

- Configure appropriate limits
- Monitor for abuse patterns
- Implement progressive delays
- Use IP whitelisting for trusted sources

### 5. Monitoring

- Log all API requests
- Monitor error rates
- Set up alerts for anomalies
- Regular security audits

## Production Deployment

### Configuration

```ini
[RestAPI]
Enabled = true
Port = 8080
BindAddress = "0.0.0.0"
EnableSSL = true
SSLCertPath = "/path/to/cert.pem"
SSLKeyPath = "/path/to/key.pem"

[RestAPI_Authentication]
JWTSecret = "${JWT_SECRET_ENV_VAR}"
RequireEmailVerification = true
MaxFailedAttempts = 3

[RestAPI_Security]
LogRequests = true
TrustedProxies = "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16"

[RestAPI_Advanced]
MaxConcurrentConnections = 1000
EnableHTTP2 = true
EnableCompression = true
```

### Load Balancing

Use a reverse proxy like Nginx:

```nginx
upstream opensim_api {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
    server 127.0.0.1:8082;
}

server {
    listen 443 ssl http2;
    server_name api.opensim.example.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://opensim_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/opensim_next /usr/local/bin/
EXPOSE 8080
CMD ["opensim_next"]
```

## Troubleshooting

### Common Issues

1. **Authentication Errors**
   - Check JWT secret configuration
   - Verify token format
   - Check token expiration

2. **Permission Denied**
   - Verify user permissions
   - Check endpoint requirements
   - Review access logs

3. **Rate Limiting**
   - Check rate limit configuration
   - Monitor request patterns
   - Implement client-side throttling

4. **Performance Issues**
   - Enable caching
   - Monitor database performance
   - Check resource usage

### Debug Mode

Enable verbose logging:

```ini
[RestAPI]
LogRequests = true
LogRequestBodies = true
LogResponseBodies = true
```

### Health Checks

Monitor API health:

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed status
curl -H "Authorization: Bearer TOKEN" http://localhost:8080/v1/stats/overview
```

---

**OpenSim Next REST API**: Modern HTTP interface for virtual world services with enterprise-grade security, performance, and developer experience.