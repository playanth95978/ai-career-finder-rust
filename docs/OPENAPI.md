# OpenAPI and Swagger Documentation Guide

This guide explains the OpenAPI/Swagger integration in JHipster Rust applications, including Scalar UI for API documentation.

## Overview

JHipster Rust applications include automatic API documentation using:

| Component      | Library         | Purpose                                       |
| -------------- | --------------- | --------------------------------------------- |
| OpenAPI Schema | `utoipa`        | Generate OpenAPI 3.0 spec from Rust code      |
| Scalar UI      | `utoipa-scalar` | Modern, interactive API documentation         |
| Swagger UI     | JHipster bundle | Classic Swagger interface (from static files) |

## Documentation Endpoints

When Swagger/OpenAPI is enabled, these endpoints are available:

| Endpoint       | Description                    | Notes                             |
| -------------- | ------------------------------ | --------------------------------- |
| `/scalar`      | Scalar UI - Modern API docs    | Interactive, dark mode support    |
| `/swagger-ui`  | Swagger UI - Classic interface | Served from JHipster static files |
| `/v3/api-docs` | OpenAPI 3.0 JSON specification | Spring Boot compatible endpoint   |

## Enabling OpenAPI Documentation

OpenAPI documentation is controlled by the `enableSwaggerCodegen` option during project generation:

```bash
jhipster-rust
# Select "Yes" when prompted for Swagger/OpenAPI documentation
```

Or in `.yo-rc.json`:

```json
{
  "generator-jhipster": {
    "enableSwaggerCodegen": true
  }
}
```

## Scalar UI

[Scalar](https://github.com/scalar/scalar) is a modern, open-source API documentation viewer that provides a clean, interactive interface for exploring your API.

### Features

- Dark/light mode support
- Interactive "Try it out" functionality
- Modern, responsive design
- Syntax highlighting for request/response bodies
- Authentication support (JWT Bearer)

### Accessing Scalar UI

Navigate to `http://localhost:8080/scalar` after starting your application.

### Authentication in Scalar

1. Click the "Authentication" button in the sidebar
2. Select "Bearer" authentication
3. Enter your JWT token (without the "Bearer " prefix)
4. All subsequent requests will include the Authorization header

## Swagger UI

JHipster Rust serves the classic Swagger UI from static files, providing the familiar JHipster Swagger experience with authentication integration.

### Accessing Swagger UI

Navigate to `http://localhost:8080/swagger-ui` after starting your application.

### Authentication in Swagger UI

1. Click the "Authorize" button (lock icon)
2. Enter your JWT token in the "Value" field
3. Click "Authorize" to apply
4. All endpoints will now include the Bearer token

## OpenAPI JSON Specification

The raw OpenAPI 3.0 specification is available at `/v3/api-docs`:

```bash
curl http://localhost:8080/v3/api-docs | jq .
```

This endpoint returns the complete API specification including:

- API info (title, version, description)
- Server URLs
- All paths with operations
- Request/response schemas
- Security schemes
- Tags for grouping endpoints

## Architecture

### How It Works

```
┌────────────────────────────────────────────────────────────────┐
│                         Rust Backend                           │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ #[utoipa::   │───▶│   ApiDoc     │───▶│  OpenAPI JSON    │  │
│  │    path]     │    │   struct     │    │  /v3/api-docs    │  │
│  │  on handlers │    │              │    │                  │  │
│  └──────────────┘    └──────────────┘    └────────┬─────────┘  │
│                                                   │            │
│  ┌──────────────┐                                 │            │
│  │ #[derive(    │                                 │            │
│  │  ToSchema)]  │                                 ▼            │
│  │  on DTOs     │                    ┌────────────────────────┐│
│  └──────────────┘                    │   Scalar UI (/scalar)  ││
│                                      │   Swagger UI           ││
│                                      │   (/swagger-ui)        ││
│                                      └────────────────────────┘│
└────────────────────────────────────────────────────────────────┘
```

### Key Components

**1. openapi.rs - API Document Definition**

```rust
#[derive(OpenApi)]
#[openapi(
    info(
        title = "MyApp API",
        version = "1.0.0",
        description = "MyApp REST API documentation"
    ),
    paths(
        handlers::account::get_account,
        handlers::user::get_all_users,
        // ... more paths
    ),
    components(schemas(UserDto, CreateUserDto, ...)),
    tags(
        (name = "account", description = "Account management"),
        (name = "user-management", description = "User administration")
    )
)]
pub struct ApiDoc;
```

**2. Handler Documentation**

```rust
/// Get current user account information
#[utoipa::path(
    get,
    path = "/api/account",
    tag = "account",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user account info", body = UserDto),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn get_account(...) -> Result<Json<UserDto>, AppError> {
    // Implementation
}
```

**3. DTO Schema Documentation**

```rust
/// User data transfer object
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    /// Unique user identifier
    #[schema(example = 1)]
    pub id: i32,

    /// User login name
    #[schema(example = "johndoe")]
    pub login: String,

    /// User email address
    #[schema(example = "john@example.com")]
    pub email: String,

    // ... more fields
}
```

## Security Schemes

### JWT Bearer Authentication

The generated API uses JWT Bearer authentication:

```rust
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Enter your JWT token"))
                    .build(),
            ),
        );
    }
}
```

### Applying Security to Endpoints

Protected endpoints include the security requirement:

```rust
#[utoipa::path(
    get,
    path = "/api/admin/users",
    security(("bearer_auth" = [])),  // Requires authentication
    // ...
)]
```

Public endpoints omit the security attribute:

```rust
#[utoipa::path(
    post,
    path = "/api/authenticate",
    // No security attribute = public endpoint
    // ...
)]
```

## Entity Documentation

When you generate entities, they automatically get OpenAPI documentation:

### Entity Endpoints

Each entity generates these documented endpoints:

| Method | Endpoint               | Description          |
| ------ | ---------------------- | -------------------- |
| GET    | `/api/{entities}`      | List with pagination |
| GET    | `/api/{entities}/{id}` | Get by ID            |
| POST   | `/api/{entities}`      | Create new           |
| PUT    | `/api/{entities}/{id}` | Update existing      |
| DELETE | `/api/{entities}/{id}` | Delete by ID         |

### Entity Handler Example

```rust
/// Get all products with pagination
#[utoipa::path(
    get,
    path = "/api/products",
    tag = "products",
    security(("bearer_auth" = [])),
    params(
        ("page" = Option<i64>, Query, description = "Page number (0-indexed)"),
        ("size" = Option<i64>, Query, description = "Page size"),
        ("sort" = Option<String>, Query, description = "Sort field and direction")
    ),
    responses(
        (status = 200, description = "List of products", body = Vec<ProductDto>,
            headers(
                ("X-Total-Count" = i64, description = "Total number of items")
            )
        ),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_all(...) { ... }
```

### Entity DTO Example

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductDto {
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = "Widget")]
    pub name: String,

    #[schema(example = 29.99)]
    pub price: f64,

    #[schema(example = true)]
    pub available: bool,
}
```

## Needle System

JHipster uses needles to automatically register new entities in the OpenAPI specification:

**openapi.rs needles:**

```rust
paths(
    // ... existing paths ...
    // jhipster-needle-add-openapi-path - JHipster will add paths here
),
components(
    schemas(
        // ... existing schemas ...
        // jhipster-needle-add-openapi-schema - JHipster will add schemas here
    )
),
tags(
    // ... existing tags ...
    // jhipster-needle-add-openapi-tag - JHipster will add tags here
)
```

When you generate a new entity, JHipster automatically:

1. Adds the entity handler paths
2. Adds the entity DTO schemas
3. Adds a tag for the entity

## Common Schema Annotations

### Basic Types

```rust
#[schema(example = "john")]           // String example
#[schema(example = 42)]               // Integer example
#[schema(example = 3.14)]             // Float example
#[schema(example = true)]             // Boolean example
#[schema(example = json!(["a", "b"]))] // Array example
```

### Validation

```rust
#[schema(minimum = 0, maximum = 100)]  // Number range
#[schema(min_length = 1, max_length = 50)] // String length
#[schema(pattern = r"^[a-z]+$")]       // Regex pattern
```

### Optional Fields

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub optional_field: Option<String>,
```

### Enum Values

```rust
#[derive(ToSchema)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}
```

## Customizing Documentation

### API Info

Edit `openapi.rs` to customize the API information:

```rust
#[openapi(
    info(
        title = "My Custom API",
        description = "Detailed API description with **markdown** support",
        version = "2.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(
            name = "API Support",
            email = "support@myapi.com",
            url = "https://myapi.com/support"
        )
    ),
    // ...
)]
```

### Server URLs

```rust
servers(
    (url = "http://localhost:8080", description = "Local development"),
    (url = "https://staging.myapp.com", description = "Staging environment"),
    (url = "https://api.myapp.com", description = "Production")
)
```

### Custom Tags

```rust
tags(
    (name = "health", description = "Health check endpoints"),
    (name = "account", description = "Account management"),
    (name = "admin", description = "Administrative operations",
     external_docs(url = "https://docs.myapp.com/admin"))
)
```

## Exporting the Specification

### JSON Export

```bash
# Export to file
curl http://localhost:8080/v3/api-docs > openapi.json

# Pretty print
curl http://localhost:8080/v3/api-docs | jq . > openapi.json
```

### Using with Other Tools

The exported OpenAPI spec can be used with:

- **Postman**: Import the JSON for API testing
- **Insomnia**: Import for API development
- **Code generators**: Generate client SDKs
- **API gateways**: Configure Kong, AWS API Gateway, etc.

## Troubleshooting

### Scalar UI Not Loading

**Symptom**: Blank page at `/scalar`

**Solutions**:

1. Verify `enableSwaggerCodegen` is `true`
2. Check server logs for errors
3. Clear browser cache

### Missing Endpoints in Documentation

**Symptom**: Some handlers not appearing in docs

**Solutions**:

1. Ensure `#[utoipa::path]` macro is present on handler
2. Verify path is added to `openapi.rs` paths list
3. Check for compilation errors in handler

### Authentication Not Working in Scalar

**Symptom**: 401 errors even after entering token

**Solutions**:

1. Ensure token doesn't include "Bearer " prefix
2. Verify token is valid and not expired
3. Check that endpoint has `security(("bearer_auth" = []))` attribute

### Schema Not Appearing

**Symptom**: Request/response body shows as empty

**Solutions**:

1. Add `#[derive(ToSchema)]` to the DTO
2. Add schema to `components(schemas(...))` in openapi.rs
3. Ensure `#[serde(rename_all = "camelCase")]` matches API expectations

## Dependencies

The OpenAPI implementation uses these crates:

| Crate           | Version | Purpose               |
| --------------- | ------- | --------------------- |
| `utoipa`        | 5.x     | OpenAPI derive macros |
| `utoipa-scalar` | 0.2.x   | Scalar UI integration |

Dependencies are conditionally included based on `enableSwaggerCodegen`:

```toml
# In Cargo.toml (when Swagger is enabled)
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-scalar = { version = "0.2", features = ["axum"] }
```

## Comparison: Scalar vs Swagger UI

| Feature              | Scalar UI | Swagger UI       |
| -------------------- | --------- | ---------------- |
| Modern design        | ✅        | ❌               |
| Dark mode            | ✅        | ❌ (limited)     |
| Performance          | Fast      | Moderate         |
| Customization        | Limited   | Extensive        |
| JHipster integration | Basic     | Full (with auth) |
| Try it out           | ✅        | ✅               |
| Code samples         | ✅        | ✅               |

**Recommendation**: Use Scalar UI for development and API exploration. Use Swagger UI for documentation that matches JHipster's frontend integration.
