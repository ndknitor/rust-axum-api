# Rust Axum API

A REST API built with Axum featuring JWT and cookie-based authentication, OpenAPI documentation, and API versioning.

## Quick Start

```bash
# Run in development mode (enables Swagger UI)
ENVIRONMENT=development cargo run

# Run in production mode (Swagger UI disabled)
cargo run

# Server starts at http://localhost:3000
# Swagger UI (dev only): http://localhost:3000/swagger-ui
```

## Configuration

Environment variables:

```bash
ENVIRONMENT=development  # If set: enables Swagger UI. If not set: production mode
HOST=0.0.0.0             # Default: 0.0.0.0
PORT=3000                # Default: 3000
JWT_SECRET=secret        # Default: default-secret-change-in-production
JWT_TTL=24               # Token expiration in hours (Default: 24)
RUST_LOG=debug           # Log level
```

### Environment Modes

| ENVIRONMENT | Swagger UI | OpenAPI |
|-------------|------------|---------|
| Not set (production) | Disabled | Disabled |
| `development` | Enabled | Enabled |
| `staging` | Enabled | Enabled |
| Any value | Enabled | Enabled |

## API Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/v1/auth/jwt` | None | Login, returns JWT token |
| POST | `/api/v1/auth/cookie` | None | Login, sets auth cookie |
| POST | `/api/v1/auth/logout` | None | Clears auth cookie |
| GET | `/api/v1/protected` | JWT or Cookie | Protected endpoint |
| GET | `/socket` | None | WebSocket endpoint (echo server) |

## Authentication

### JWT Authentication

```bash
# 1. Get a token
curl -X POST http://localhost:3000/api/v1/auth/jwt \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass"}'

# Response: {"token":"eyJ..."}

# 2. Use the token
curl http://localhost:3000/api/v1/protected \
  -H "Authorization: Bearer eyJ..."
```

### Cookie Authentication

```bash
# 1. Login (cookie is set automatically)
curl -X POST http://localhost:3000/api/v1/auth/cookie \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass"}' \
  -c cookies.txt

# 2. Access protected routes
curl http://localhost:3000/api/v1/protected -b cookies.txt

# 3. Logout
curl -X POST http://localhost:3000/api/v1/auth/logout -b cookies.txt
```

## WebSocket

The `/socket` endpoint provides a WebSocket echo server that echoes back any message you send.

### Using websocat

```bash
# Install websocat
cargo install websocat

# Connect and send messages
websocat ws://localhost:3000/socket
# Type messages and press Enter - they will be echoed back
```

### Using curl (WebSocket)

```bash
curl --include \
  --no-buffer \
  --header "Connection: Upgrade" \
  --header "Upgrade: websocket" \
  --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  --header "Sec-WebSocket-Version: 13" \
  http://localhost:3000/socket
```

### JavaScript Example

```javascript
const ws = new WebSocket('ws://localhost:3000/socket');

ws.onopen = () => {
  console.log('Connected');
  ws.send('Hello, server!');
};

ws.onmessage = (event) => {
  console.log('Received:', event.data);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

## Static Files

Static files are served from the `wwwroot` folder. Any file placed in this folder will be accessible at the root URL.

### File Mapping

| File Path | URL |
|-----------|-----|
| `wwwroot/index.html` | `http://localhost:3000/index.html` |
| `wwwroot/css/style.css` | `http://localhost:3000/css/style.css` |
| `wwwroot/js/app.js` | `http://localhost:3000/js/app.js` |
| `wwwroot/images/logo.png` | `http://localhost:3000/images/logo.png` |

### Folder Structure

```
wwwroot/
├── index.html
├── css/
│   └── style.css
├── js/
│   └── app.js
└── images/
    └── logo.png
```

### Notes

- Static files are served as a fallback (API routes take priority)
- The `wwwroot` folder must exist in the working directory
- For Docker deployments, the folder is copied to `/app/wwwroot`

---

## Adding a New Controller

### Step 1: Create the Controller File

Create a new file `src/controllers/v1/users.rs`:

```rust
use axum::{Json, Router, routing::{get, post}};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::middlewares::Authorize;

pub fn router() -> Router {
    Router::new()
        // Public endpoints (no auth required)
        .route("/", get(list_users))
        // Private endpoints (auth required)
        .route("/me", get(get_current_user))
        .route("/", post(create_user))
}

// ============================================
// Public Endpoint (no authentication)
// ============================================

#[derive(Serialize, ToSchema)]
pub struct User {
    pub id: u64,
    pub username: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>)
    ),
    tag = "users"
)]
pub async fn list_users() -> Json<Vec<User>> {
    // This endpoint is public - no Auth extractor
    Json(vec![
        User { id: 1, username: "alice".to_string() },
        User { id: 2, username: "bob".to_string() },
    ])
}

// ============================================
// Private Endpoint (JWT or Cookie auth)
// ============================================

#[derive(Serialize, ToSchema)]
pub struct CurrentUser {
    pub username: String,
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",TTL
    responses(
        (status = 200, description = "Get current user", body = CurrentUser),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_current_user(Authorize(claims): Authorize) -> Json<CurrentUser> {
    // Authorize extractor validates JWT or cookie automatically
    // claims.sub contains the username from the token
    Json(CurrentUser {
        username: claims.sub,
        message: "You are authenticated!".to_string(),
    })
}

// ============================================
// Private Endpoint with Request Body
// ============================================

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub id: u64,
    pub username: String,
    pub created_by: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = CreateUserResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "users"
)]
pub async fn create_user(
    Authorize(claims): Authorize,
    Json(payload): Json<CreateUserRequest>,
) -> Json<CreateUserResponse> {
    Json(CreateUserResponse {
        id: 123,
        username: payload.username,
        created_by: claims.sub,
    })
}
```

### Step 2: Register the Controller

Update `src/controllers/v1/mod.rs`:

```rust
pub mod auth;
pub mod protected;
pub mod root;
pub mod users;  // Add this line

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(root::router())
        .nest("/auth", auth::router())
        .nest("/protected", protected::router())
        .nest("/users", users::router())  // Add this line
}
```

### Step 3: Add to OpenAPI Documentation

Update `src/main.rs`:

```rust
#[derive(OpenApi)]
#[openapi(
    info(title = "Rust Axum API", version = "1.0.0"),
    paths(
        // ... existing paths ...
        controllers::v1::users::list_users,
        controllers::v1::users::get_current_user,
        controllers::v1::users::create_user,
    ),
    components(
        schemas(
            // ... existing schemas ...
            controllers::v1::users::User,
            controllers::v1::users::CurrentUser,
            controllers::v1::users::CreateUserRequest,
            controllers::v1::users::CreateUserResponse,
        )
    ),
    tags(
        (name = "v1", description = "API version 1"),
        (name = "v2", description = "API version 2"),
        (name = "auth", description = "Authentication"),
        (name = "users", description = "User management"),  // Add this
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;
```

---

## Authorization

### Extractors

| Extractor | Description |
|-----------|-------------|
| `Authorize` | Accepts **both** JWT Bearer token and cookie (tries JWT first) |
| `JwtAuth` | JWT Bearer token **only** |
| `CookieAuth` | Cookie **only** |

### Claims Structure

JWT tokens now include `roles` and `policies`:

```rust
pub struct Claims {
    pub sub: String,      // Subject (username)
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub roles: Vec<String>,    // User roles
    pub policies: Vec<String>, // User policies
}
```

### Basic Usage (Authentication Only)

```rust
use crate::middlewares::{Authorize, JwtAuth, CookieAuth};

// Accepts both JWT and cookie
async fn flexible_auth(Authorize(claims): Authorize) -> String {
    format!("Hello, {}", claims.sub)
}

// JWT only
async fn jwt_only(JwtAuth(claims): JwtAuth) -> String {
    format!("Hello, {}", claims.sub)
}

// Cookie only
async fn cookie_only(CookieAuth(claims): CookieAuth) -> String {
    format!("Hello, {}", claims.sub)
}
```

### Role-Based Authorization (OR Logic)

Use the `require_roles!` macro to protect routes. User needs **at least one** of the specified roles:

```rust
use axum::{Router, routing::get};

// User must have "admin" OR "moderator" role
let app = Router::new()
    .route("/admin", get(admin_handler))
    .layer(require_roles!("admin", "moderator"));
```

### Policy-Based Authorization (AND Logic)

Use the `require_policies!` macro. User needs **all** specified policies:

```rust
// User must have BOTH "read" AND "write" policies
let app = Router::new()
    .route("/manage", get(manage_handler))
    .layer(require_policies!("read", "write"));
```

### Combined Roles and Policies

Use the `require_auth!` macro for both:

```rust
// User must have ("admin" OR "superadmin") AND ("read" AND "write")
let app = Router::new()
    .route("/super", get(super_handler))
    .layer(require_auth!(
        roles: ["admin", "superadmin"],
        policies: ["read", "write"]
    ));
```

### Create Claims with roles and policies

```rust
  // Without roles/policies                                                                                                                                                  
  let claims = Claims::new(username, 24, None, None);                                                                                                                        
                                                                                                                                                                             
  // With roles only                                                                                                                                                         
  let claims = Claims::new(username, 24, Some(vec!["admin".into()]), None);                                                                                                  
                                                                                                                                                                             
  // With both                                                                                                                                                               
  let claims = Claims::new(                                                                                                                                                  
      username,                                                                                                                                                              
      24,                                                                                                                                                                    
      Some(vec!["admin".into(), "user".into()]),                                                                                                                             
      Some(vec!["read".into(), "write".into()]),                                                                                                                             
  );                                                                                                                                                                         
                                                                                                                                                                             
  // Builder pattern still works too                                                                                                                                         
  let claims = Claims::new(username, 24, None, None)                                                                                                                         
      .with_roles(vec!["admin".into()])                                                                                                                                      
      .with_policies(vec!["read".into()]);
```

### Manual Check in Handlers

You can also check roles/policies manually in handlers:

```rust
use crate::middlewares::{Authorize, AuthorizeError};

async fn admin_handler(
    Authorize(claims): Authorize
) -> Result<String, AuthorizeError> {
    // Check roles (OR logic)
    if !claims.has_any_role(&["admin", "moderator"]) {
        return Err(AuthorizeError::Forbidden);
    }

    // Check policies (AND logic)
    if !claims.has_all_policies(&["read", "write"]) {
        return Err(AuthorizeError::Forbidden);
    }

    Ok(format!("Welcome admin: {}", claims.sub))
}
```

### Complete Controller Example

Here's a full example of a controller with different authorization levels:

```rust
// src/controllers/v1/admin.rs
use axum::{Json, Router, routing::{get, post, delete}};
use serde::Serialize;

use crate::middlewares::Authorize;
use crate::require_roles;

pub fn router() -> Router {
    Router::new()
        // Public route - no authentication
        .route("/status", get(status))
        // Authenticated routes - any logged-in user
        .route("/profile", get(profile))
        // Admin-only routes - requires "admin" role
        .nest("/manage", admin_routes())
}

fn admin_routes() -> Router {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/:id", delete(delete_user))
        // Apply role check to all routes in this group
        .layer(require_roles!("admin", "superadmin"))
}

// Public - no auth required
async fn status() -> &'static str {
    "OK"
}

#[derive(Serialize)]
pub struct Profile {
    username: String,
    roles: Vec<String>,
    policies: Vec<String>,
}

// Authenticated - any valid token
async fn profile(Authorize(claims): Authorize) -> Json<Profile> {
    Json(Profile {
        username: claims.sub,
        roles: claims.roles,
        policies: claims.policies,
    })
}

// Admin only - protected by layer
async fn list_users(Authorize(claims): Authorize) -> String {
    format!("Admin {} listing users", claims.sub)
}

async fn create_user(Authorize(claims): Authorize) -> String {
    format!("Admin {} creating user", claims.sub)
}

async fn delete_user(Authorize(claims): Authorize) -> String {
    format!("Admin {} deleting user", claims.sub)
}
```

Register in `src/controllers/v1/mod.rs`:

```rust
pub mod admin;

pub fn router() -> Router {
    Router::new()
        // ... existing routes ...
        .nest("/admin", admin::router())
}
```

---

## Project Structure

```
├── wwwroot/                # Static files (HTML, CSS, JS, images)
│   └── index.html
└── src/
    ├── main.rs             # App entry point, OpenAPI config
    ├── openapi.rs          # OpenAPI documentation config
    ├── middlewares/
│   ├── mod.rs
│   ├── authorize.rs        # Authorize extractor + role/policy middleware
│   ├── cookie.rs           # CookieAuth extractor
│   └── jwt.rs              # JwtAuth extractor, Claims with roles/policies
└── controllers/
    ├── mod.rs              # Router setup, /api prefix
    ├── socket.rs           # WebSocket endpoint /socket
    ├── v1/
    │   ├── mod.rs          # v1 router
    │   ├── root.rs         # GET /api/v1
    │   ├── auth.rs         # /api/v1/auth/*
    │   └── protected.rs    # /api/v1/protected
    └── v2/
        ├── mod.rs          # v2 router
        └── root.rs         # /api/v2/*
```

## Commands

```bash
cargo build              # Build the project
cargo run                # Run the server
cargo test               # Run tests
cargo check              # Fast type checking
cargo clippy             # Run linter
cargo fmt                # Format code
```

## Docker

### Build and Run

```bash
# Build the image
docker build -t rust-axum-api .

# Run in production mode (Swagger disabled)
docker run -p 3000:3000 rust-axum-api

# Run in development mode (Swagger enabled)
docker run -p 3000:3000 -e ENVIRONMENT=development rust-axum-api

# Run with custom JWT secret
docker run -p 3000:3000 \
  -e ENVIRONMENT=development \
  -e JWT_SECRET=your-secret-key \
  rust-axum-api
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - ENVIRONMENT=development
      - JWT_SECRET=your-secret-key
      - RUST_LOG=info
```

Run with:

```bash
docker compose up --build
```
