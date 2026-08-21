# Security and Authentication

This document covers the authentication and authorization options available in the JHipster Rust generator.

## Overview

The generator supports two authentication strategies:

| Strategy        | Description                                      | Best For                               |
| --------------- | ------------------------------------------------ | -------------------------------------- |
| **JWT**         | Self-contained token authentication              | Standalone applications, microservices |
| **OAuth2/OIDC** | Delegated authentication with identity providers | Enterprise SSO, Keycloak, Okta         |

## Authentication Types

### JWT Authentication

JWT (JSON Web Token) authentication is a stateless authentication mechanism where the server generates a signed token containing user claims.

**How it works:**

1. User submits credentials to `/api/authenticate`
2. Server validates credentials against database
3. Server generates signed JWT with user claims
4. Client includes token in `Authorization: Bearer <token>` header
5. Server validates token signature on each request

**Configuration:**

Set these environment variables in your `.env` file:

```bash
# Required: Secret key for signing tokens (use a strong random string)
JWT_SECRET=your-secret-key-minimum-256-bits-for-security

# Optional: Token expiration (default: 24 hours)
JWT_EXPIRATION_HOURS=24
```

**Token Structure:**

```json
{
  "sub": "username",
  "auth": ["ROLE_USER", "ROLE_ADMIN"],
  "iat": 1704067200,
  "exp": 1704153600
}
```

**Remember Me:**

When the `remember_me` flag is set during authentication, tokens are issued with a 30-day expiration instead of the default 24 hours.

### OAuth2/OIDC Authentication

OAuth2 with OpenID Connect (OIDC) delegates authentication to an external identity provider.

**Supported Providers:**

- Keycloak (see [Keycloak Integration](./KEYCLOAK.md) for detailed setup)
- Okta
- Any standard OIDC-compliant provider

**How it works:**

1. User clicks login, redirected to identity provider
2. User authenticates with provider (username/password, SSO, MFA)
3. Provider redirects back with authorization code
4. Server exchanges code for access token and ID token
5. Server validates tokens using provider's public keys (JWKS)
6. Access token stored in HTTP-only cookie

**Configuration:**

Set these environment variables in your `.env` file:

```bash
# Required: OIDC issuer URL
OIDC_ISSUER_URI=http://localhost:9080/realms/jhipster

# Required: OAuth2 client credentials
OIDC_CLIENT_ID=web_app
OIDC_CLIENT_SECRET=web_app

# Required: Frontend URL for redirects
FRONTEND_URL=http://localhost:8080
```

**Automatic Endpoint Discovery:**

The generator automatically constructs standard OIDC endpoints from the issuer URI:

- Authorization endpoint: `{issuer}/protocol/openid-connect/auth`
- Token endpoint: `{issuer}/protocol/openid-connect/token`
- JWKS endpoint: `{issuer}/protocol/openid-connect/certs`
- Logout endpoint: `{issuer}/protocol/openid-connect/logout`

For detailed Keycloak setup including realm configuration, client setup, and user management, see [Keycloak Integration](./KEYCLOAK.md).

## Password Security

### Argon2 Hashing

User passwords are hashed using Argon2, the winner of the Password Hashing Competition and recommended by OWASP.

**Features:**

- Memory-hard algorithm resistant to GPU attacks
- Cryptographically random salt per password
- Stored in standard PHC format: `$argon2id$v=19$m=...`

**Implementation:**

```rust
// Password hashing (during registration/password change)
let hash = UserService::hash_password("user_password")?;

// Password verification (during authentication)
let is_valid = UserService::verify_password("user_password", &stored_hash)?;
```

### Password Change

JWT mode includes a password change endpoint:

```bash
POST /api/account/change-password
Content-Type: application/json
Authorization: Bearer <token>

{
  "currentPassword": "old_password",
  "newPassword": "new_password"
}
```

The endpoint:

1. Validates the current password
2. Hashes the new password with Argon2
3. Updates the stored hash

### Password Reset (Email Enabled)

When email is enabled, users can reset forgotten passwords:

**Step 1: Request Reset**

```bash
POST /api/account/reset-password/init
Content-Type: text/plain

user@example.com
```

The server:

1. Generates a random 20-character reset key
2. Stores the key with a timestamp
3. Sends reset email with link

**Step 2: Complete Reset**

```bash
POST /api/account/reset-password/finish
Content-Type: application/json

{
  "key": "abc123resetkey...",
  "newPassword": "new_secure_password"
}
```

The server:

1. Validates the reset key exists
2. Checks key is not expired (24-hour window)
3. Hashes and stores new password
4. Clears the reset key

For email configuration, see [Email Integration](./EMAIL_INTEGRATION.md).

## Authorization

### Role-Based Access Control (RBAC)

The generator implements role-based authorization with two default roles:

| Role         | Description           |
| ------------ | --------------------- |
| `ROLE_USER`  | Standard user access  |
| `ROLE_ADMIN` | Administrative access |

**Database Structure:**

Users are linked to roles through the `user_authorities` join table:

```sql
CREATE TABLE user_authorities (
    user_id INTEGER NOT NULL,
    authority_name VARCHAR(50) NOT NULL,
    PRIMARY KEY (user_id, authority_name),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### Checking Authorization

In handlers, use the `AuthUser` extracted from the request:

```rust
use crate::middleware::auth::AuthUser;

async fn admin_only(
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if !auth_user.has_authority("ROLE_ADMIN") {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    // ... handler logic
}
```

### Route Protection

The generator provides middleware for protecting routes:

```rust
// Require authentication (any role)
.layer(middleware::from_fn(require_auth))

// Require specific role
.layer(middleware::from_fn(|req, next| require_role(req, next, "ROLE_ADMIN")))
```

## Security Middleware

### Authentication Middleware

The `auth_middleware` runs on all requests and:

1. Extracts token from `Authorization` header (JWT) or cookies (OAuth2)
2. Validates token signature and expiration
3. Extracts user claims (username, roles)
4. Inserts `AuthUser` into request extensions

```rust
pub struct AuthUser {
    pub login: String,
    pub authorities: Vec<String>,
    pub email: Option<String>,        // OAuth2 only
    pub first_name: Option<String>,   // OAuth2 only
    pub last_name: Option<String>,    // OAuth2 only
}
```

### Require Auth Middleware

Returns `401 Unauthorized` if the user is not authenticated:

```rust
async fn require_auth<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError>
```

### Require Role Middleware

Returns `403 Forbidden` if the user lacks the required role:

```rust
async fn require_role<B>(
    request: Request<B>,
    next: Next<B>,
    required_role: &'static str,
) -> Result<Response, AppError>
```

## API Endpoints

### JWT Mode Endpoints

| Method | Endpoint                             | Description                 | Auth Required |
| ------ | ------------------------------------ | --------------------------- | ------------- |
| POST   | `/api/authenticate`                  | Login with credentials      | No            |
| GET    | `/api/account`                       | Get current user            | Yes           |
| POST   | `/api/account`                       | Update current user account | Yes           |
| POST   | `/api/account/change-password`       | Change password             | Yes           |
| POST   | `/api/register`                      | Register new user*          | No            |
| GET    | `/api/activate`                      | Activate user account*      | No            |
| POST   | `/api/account/reset-password/init`   | Request password reset*     | No            |
| POST   | `/api/account/reset-password/finish` | Complete password reset*    | No            |

*Email-enabled endpoints: When email is disabled, `/api/register` creates already-activated users, and password reset endpoints are not available.

### OAuth2 Mode Endpoints

| Method | Endpoint                     | Description                 | Auth Required |
| ------ | ---------------------------- | --------------------------- | ------------- |
| GET    | `/oauth2/authorization/oidc` | Initiate OAuth2 login       | No            |
| GET    | `/login/oauth2/code/oidc`    | OAuth2 callback             | No            |
| GET    | `/api/oauth2/info`           | Get OAuth2 config           | No            |
| GET    | `/api/account`               | Get current user            | Yes           |
| POST   | `/api/logout`                | Logout (returns logout URL) | Yes           |

### Common Protected Endpoints

| Method | Endpoint               | Required Role |
| ------ | ---------------------- | ------------- |
| GET    | `/api/users`           | ROLE_USER     |
| GET    | `/api/admin/users`     | ROLE_ADMIN    |
| POST   | `/api/admin/users`     | ROLE_ADMIN    |
| PUT    | `/api/admin/users/:id` | ROLE_ADMIN    |
| DELETE | `/api/admin/users/:id` | ROLE_ADMIN    |

## OAuth2 Token Handling

### Token Validation

OAuth2 tokens are validated using the provider's JSON Web Key Set (JWKS):

1. Fetch public keys from `{issuer}/protocol/openid-connect/certs`
2. Cache keys for 5 minutes to reduce provider calls
3. Match token's `kid` (key ID) header to signing key
4. Validate signature using RS256/RS384/RS512
5. Verify `iss` (issuer) matches expected issuer
6. Verify `azp` (authorized party) matches client ID

### Claims Extraction

The generator extracts user information from OIDC tokens:

| Claim                         | Usage                  |
| ----------------------------- | ---------------------- |
| `preferred_username` or `sub` | Username               |
| `email`                       | Email address          |
| `given_name`                  | First name             |
| `family_name`                 | Last name              |
| `realm_access.roles`          | Keycloak roles         |
| `groups`                      | Role groups (fallback) |

### Cookie Security

OAuth2 tokens are stored in secure cookies:

```rust
Set-Cookie: access_token=<token>; HttpOnly; SameSite=Lax; Path=/
Set-Cookie: id_token=<token>; HttpOnly; SameSite=Lax; Path=/
```

- **HttpOnly**: Prevents JavaScript access (XSS protection)
- **SameSite=Lax**: CSRF protection while allowing navigation

## CSRF Protection

### OAuth2 State Parameter

The OAuth2 flow uses a `state` parameter to prevent CSRF attacks:

1. Generate random state value before redirect
2. Include state in authorization URL
3. Validate state matches on callback

### Cookie Attributes

All authentication cookies use `SameSite=Lax` to prevent cross-site request forgery.

## Security Best Practices

### JWT Secret Management

```bash
# Generate a strong secret (256+ bits)
openssl rand -base64 32

# Store in environment, not in code
JWT_SECRET=<generated-secret>
```

### Production Checklist

1. **Use HTTPS**: Always use TLS in production
2. **Strong Secrets**: Use cryptographically random JWT secrets (256+ bits)
3. **Secure Cookies**: Enable `Secure` flag for cookies in production
4. **Token Expiration**: Use short-lived tokens (24 hours or less)
5. **Password Policy**: Enforce strong passwords at the application level
6. **Rate Limiting**: Add rate limiting to authentication endpoints
7. **Audit Logging**: Monitor authentication events
8. **CORS Configuration**: Restrict allowed origins in production

### Environment-Specific Configuration

**Development:**

```bash
JWT_SECRET=dev-secret-not-for-production
JWT_EXPIRATION_HOURS=168  # 1 week for convenience
```

**Production:**

```bash
JWT_SECRET=<strong-random-secret>
JWT_EXPIRATION_HOURS=24
```

## Troubleshooting

### JWT Issues

**Token expired:**

```
Error: token has expired
```

Solution: Re-authenticate to get a new token.

**Invalid signature:**

```
Error: invalid token signature
```

Solution: Ensure `JWT_SECRET` matches between token generation and validation.

### OAuth2 Issues

**Invalid redirect URI:**

```
Error: invalid_redirect_uri
```

Solution: Ensure the callback URL is registered in your identity provider.

**Token validation failed:**

```
Error: token validation failed
```

Solution: Check that `OIDC_ISSUER_URI` matches the token issuer.

For Keycloak-specific troubleshooting, see [Keycloak Integration - Troubleshooting](./KEYCLOAK.md#troubleshooting).

## Dependencies

The security implementation uses these Rust crates:

| Crate          | Purpose                                |
| -------------- | -------------------------------------- |
| `argon2`       | Password hashing                       |
| `jsonwebtoken` | JWT creation and validation            |
| `rand`         | Cryptographic random number generation |
| `chrono`       | Token expiration handling              |
| `reqwest`      | HTTP client for OAuth2 token exchange  |
| `tower-http`   | CORS middleware                        |
