# Entity Generation

This document covers entity generation options, field types, validations, pagination, and relationships for the JHipster Rust generator.

## Overview

The JHipster Rust generator creates complete CRUD functionality for entities including:

- Diesel ORM models with proper type mappings
- Axum HTTP handlers with RESTful endpoints
- Service layer with business logic
- DTOs with validation
- Database migrations with indexes and constraints

## Generating Entities

### Using JDL (JHipster Domain Language)

The recommended way to define entities is using JDL files:

```bash
# Create a .jdl file with your entity definitions
jhipster-rust jdl your-entities.jdl
```

### Interactive Entity Generation

You can also generate entities interactively:

```bash
jhipster-rust entity YourEntityName
```

## Field Types

The generator supports the following field types with automatic mapping to Rust types and database columns:

| JDL Type      | Rust Type                | PostgreSQL                 | MySQL           | SQLite      |
| ------------- | ------------------------ | -------------------------- | --------------- | ----------- |
| Boolean       | `bool`                   | `BOOLEAN`                  | `BOOLEAN`       | `BOOLEAN`   |
| Integer       | `i32`                    | `INTEGER`                  | `INTEGER`       | `INTEGER`   |
| Long          | `i64`                    | `BIGINT`                   | `BIGINT`        | `BIGINT`    |
| Float         | `f32`                    | `REAL`                     | `FLOAT`         | `REAL`      |
| Double        | `f64`                    | `DOUBLE PRECISION`         | `DOUBLE`        | `REAL`      |
| BigDecimal    | `bigdecimal::BigDecimal` | `DECIMAL`                  | `DECIMAL(21,2)` | `DECIMAL`   |
| String        | `String`                 | `VARCHAR(255)`             | `VARCHAR(255)`  | `TEXT`      |
| UUID          | `uuid::Uuid`             | `UUID`                     | `VARCHAR(36)`   | `TEXT`      |
| LocalDate     | `chrono::NaiveDate`      | `DATE`                     | `DATE`          | `DATE`      |
| Instant       | `chrono::NaiveDateTime`  | `TIMESTAMP`                | `DATETIME`      | `TIMESTAMP` |
| ZonedDateTime | `chrono::NaiveDateTime`  | `TIMESTAMP WITH TIME ZONE` | `DATETIME`      | `TIMESTAMP` |
| Duration      | `i64`                    | `BIGINT`                   | `BIGINT`        | `BIGINT`    |
| TextBlob      | `String`                 | `TEXT`                     | `TEXT`          | `TEXT`      |
| Blob          | `Vec<u8>`                | `BYTEA`                    | `LONGBLOB`      | `BLOB`      |
| AnyBlob       | `Vec<u8>`                | `BYTEA`                    | `LONGBLOB`      | `BLOB`      |
| ImageBlob     | `Vec<u8>`                | `BYTEA`                    | `LONGBLOB`      | `BLOB`      |

### Example JDL with Field Types

```jdl
entity Product {
  name String required
  description TextBlob
  price BigDecimal required
  quantity Integer
  available Boolean
  createdAt Instant
  sku UUID
}
```

## Field Validations

The generator supports comprehensive field validations that are enforced both at the API level (using the `validator` crate) and at the database level (using constraints).

### Available Validations

| Validation  | Description                       | Example                           |
| ----------- | --------------------------------- | --------------------------------- |
| `required`  | Field cannot be null              | `name String required`            |
| `minlength` | Minimum string length             | `name String minlength(2)`        |
| `maxlength` | Maximum string length             | `name String maxlength(100)`      |
| `pattern`   | Regex pattern matching            | `code String pattern(/^[A-Z]+$/)` |
| `min`       | Minimum numeric value             | `age Integer min(0)`              |
| `max`       | Maximum numeric value             | `age Integer max(150)`            |
| `unique`    | Unique constraint (creates index) | `email String unique`             |

### Generated Validation Code

For a field defined as:

```jdl
entity User {
  username String required minlength(3) maxlength(50) pattern(/^[a-zA-Z0-9_]+$/)
  age Integer min(0) max(150)
}
```

The generator creates DTO validation:

```rust
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 50), regex(path = *USERNAME_REGEX))]
    pub username: String,

    #[validate(range(min = 0, max = 150))]
    pub age: Option<i32>,
}

lazy_static::lazy_static! {
    static ref USERNAME_REGEX: regex::Regex =
        regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
}
```

### Validation Example

```jdl
entity Employee {
  firstName String required minlength(1) maxlength(50)
  lastName String required minlength(1) maxlength(50)
  email String required unique pattern(/^[^@]+@[^@]+\.[^@]+$/)
  salary BigDecimal min(0)
  hireDate LocalDate required
}
```

## Pagination

All entity list endpoints support pagination with the following query parameters:

### Query Parameters

| Parameter | Default  | Description                     |
| --------- | -------- | ------------------------------- |
| `page`    | 0        | Zero-based page number          |
| `size`    | 20       | Items per page (max 100)        |
| `sort`    | `id,asc` | Sort specification (repeatable) |

### Sort Specification

Sort parameters follow the format `field,direction`:

```bash
# Single sort
GET /api/products?sort=name,asc

# Multiple sorts (first takes priority)
GET /api/products?sort=category,asc&sort=price,desc

# Direction is optional (defaults to asc)
GET /api/products?sort=name
```

### Response Headers

Paginated responses include:

- `X-Total-Count` - Total number of records matching the query

### Example Requests

```bash
# Get first page of 10 products sorted by name
curl "http://localhost:8080/api/products?page=0&size=10&sort=name,asc"

# Get second page with default size
curl "http://localhost:8080/api/products?page=1"

# Sort by multiple fields
curl "http://localhost:8080/api/products?sort=category,asc&sort=createdDate,desc"
```

### Pagination in Rust

The generated code uses a `PageRequest` struct:

```rust
#[derive(Debug, Deserialize)]
pub struct PageRequest {
    #[serde(default)]
    pub page: i64,
    #[serde(default = "default_size")]
    pub size: i64,
    #[serde(default)]
    pub sort: Vec<String>,
}

impl PageRequest {
    pub fn offset(&self) -> i64 {
        self.page * self.limit()
    }

    pub fn limit(&self) -> i64 {
        self.size.min(100).max(1)
    }
}
```

## Relationships

The generator supports three types of entity relationships.

### ManyToOne / OneToMany

A ManyToOne relationship creates a foreign key in the owning entity.

```jdl
entity Blog {
  name String required
  handle String required
}

entity Post {
  title String required
  content TextBlob required
  date Instant required
}

// Post has a blog_id foreign key
relationship ManyToOne {
  Post{blog(name)} to Blog
}
```

**Generated Structure:**

- `Post` model includes `blog_id: Option<i32>`
- Foreign key constraint with ON DELETE SET NULL
- Index on `blog_id` for query performance
- `PostDto` includes optional `blog: Option<BlogDto>` for nested serialization

### OneToOne

A OneToOne relationship works similarly to ManyToOne but enforces uniqueness.

```jdl
entity User {
  login String required
}

entity Profile {
  bio TextBlob
  website String
}

relationship OneToOne {
  Profile{user(login)} to User
}
```

**Generated Structure:**

- `Profile` model includes `user_id: Option<i32>`
- Unique constraint on the foreign key
- Index on `user_id`

### ManyToMany

A ManyToMany relationship creates a join table.

```jdl
entity Post {
  title String required
  content TextBlob required
}

entity Tag {
  name String required
}

relationship ManyToMany {
  Post{tag(name)} to Tag{post}
}
```

**Generated Structure:**

Join table `rel_post__tag`:

```sql
CREATE TABLE rel_post__tag (
  post_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (post_id, tag_id),
  FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

**Service Methods for ManyToMany:**

```rust
impl PostService {
    // Add a single tag to a post
    pub fn add_tag(conn: &mut DbConnection, post_id: i32, tag_id: i32) -> Result<()>;

    // Remove a tag from a post
    pub fn remove_tag(conn: &mut DbConnection, post_id: i32, tag_id: i32) -> Result<()>;

    // Get all tags for a post
    pub fn get_tags(conn: &mut DbConnection, post_id: i32) -> Result<Vec<Tag>>;

    // Replace all tags (sync)
    pub fn sync_tags(conn: &mut DbConnection, post_id: i32, tag_ids: &[i32]) -> Result<()>;
}
```

### Complete JDL Example

```jdl
application {
  config {
    baseName myapp
    applicationType monolith
    databaseType sql
    devDatabaseType postgresql
    prodDatabaseType postgresql
    authenticationType jwt
    buildTool cargo
  }
  entities *
}

entity Blog {
  name String required minlength(3) maxlength(100)
  handle String required unique pattern(/^[a-z0-9-]+$/)
}

entity Post {
  title String required minlength(1) maxlength(200)
  content TextBlob required
  date Instant required
  published Boolean
}

entity Tag {
  name String required minlength(2) maxlength(40) unique
}

entity Comment {
  content TextBlob required
  date Instant required
}

relationship ManyToOne {
  Post{blog(name) required} to Blog
  Comment{post(title) required} to Post
}

relationship ManyToMany {
  Post{tag(name)} to Tag{post}
}

paginate Post, Tag with pagination
```

## Generated Files

For each entity, the generator creates:

| File                                     | Description                           |
| ---------------------------------------- | ------------------------------------- |
| `src/models/{entity}.rs`                 | Diesel model structs                  |
| `src/dto/{entity}_dto.rs`                | Request/response DTOs with validation |
| `src/services/{entity}_service.rs`       | Business logic and database queries   |
| `src/handlers/{entity}.rs`               | Axum HTTP handlers                    |
| `migrations/{timestamp}_create_{table}/` | SQL migration files                   |

### Generated API Endpoints

| Method | Endpoint              | Description            |
| ------ | --------------------- | ---------------------- |
| GET    | `/api/{entities}`     | List with pagination   |
| GET    | `/api/{entities}/:id` | Get by ID              |
| POST   | `/api/{entities}`     | Create new entity      |
| PUT    | `/api/{entities}/:id` | Update existing entity |
| DELETE | `/api/{entities}/:id` | Delete entity          |

## Audit Fields

Every entity automatically includes audit fields:

| Field                | Type                    | Description                     |
| -------------------- | ----------------------- | ------------------------------- |
| `created_by`         | `Option<String>`        | Username who created the entity |
| `created_date`       | `Option<NaiveDateTime>` | Creation timestamp              |
| `last_modified_by`   | `Option<String>`        | Username who last modified      |
| `last_modified_date` | `Option<NaiveDateTime>` | Last modification timestamp     |

These fields are automatically populated by the service layer based on the authenticated user.

## Running Migrations

After generating entities, run the database migrations:

```bash
cd server
diesel migration run
```

To revert the last migration:

```bash
diesel migration revert
```

## Testing Generated Entities

Each generated entity includes integration tests. Run them with:

```bash
cd server

# Run all tests (use single thread for database tests)
cargo test -- --test-threads=1

# Run tests for a specific entity
cargo test post -- --test-threads=1
```

## Best Practices

1. **Use JDL for complex schemas**: JDL files are easier to maintain and version control than interactive entity generation.

2. **Define relationships carefully**: Consider which side owns the relationship and whether cascade deletes are appropriate.

3. **Add indexes for foreign keys**: The generator automatically creates indexes for foreign keys and unique constraints.

4. **Use appropriate field types**: Choose the smallest type that fits your data (e.g., `Integer` instead of `Long` for counts).

5. **Validate at all layers**: Field validations are enforced at the DTO level, but also consider adding database-level constraints for critical fields.

6. **Test with realistic data**: The generated integration tests provide a starting point, but add tests with edge cases specific to your domain.
