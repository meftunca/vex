# Policy System

Version: 0.1.2
Last Updated: November 2025

This document describes Vex's policy system, which provides metadata annotations for structs and fields, enabling features like serialization, validation, and code generation.

---

## Overview

The policy system allows you to define reusable metadata templates that can be applied to structs and their fields. This enables:

- Serialization: JSON, XML, database mappings
- Validation: Field constraints and rules
- Code Generation: Automatic CRUD operations, API endpoints
- Documentation: Field descriptions and examples
- UI Generation: Form layouts and input types

### Key Concepts

- Policy: A named collection of field metadata
- Metadata: Key-value annotations using backtick syntax
- Composition: Policies can inherit from parent policies
- Application: Structs apply policies using `with` clause
- Resolution: Metadata is merged with precedence rules

---

## Policy Declarations

### Basic Policy Syntax

```vex
policy PolicyName {
    field_name `key:"value"`,
    another_field `type:"string" required:"true"`
}
```

Components:

- `policy` keyword
- Policy name (identifier)
- Field declarations with metadata in backticks
- Comma-separated fields

### Metadata Syntax

Metadata uses a simple key-value format within backticks:

```vex
`key1:"value1" key2:"value2" key3:"true"`
```

Rules:

- Keys and values are strings
- Multiple key-value pairs separated by spaces
- Values can contain special characters
- No nested structures (flat key-value pairs)

### Example Policies

```vex
policy APIModel {
    id `json:"id" db:"id" required:"true"`,
    name `json:"name" db:"name" max_length:"100"`,
    email `json:"email" db:"email" format:"email"`,
    created_at `json:"created_at" db:"created_at" type:"datetime"`
}

policy ValidationRules {
    age `min:"0" max:"150"`,
    salary `min:"0"`,
    phone `pattern:"^\\+?[1-9]\\d{1,14}$"`
}
```

---

## Policy Composition

### Parent Policies

Policies can inherit metadata from parent policies:

```vex
policy BaseModel {
    id `db:"id" indexed:"true"`,
    created_at `db:"created_at"`,
    updated_at `db:"updated_at"`
}

policy APIModel with BaseModel {
    id `json:"id"`,  // Overrides db:"id" with json:"id"
    name `json:"name"`,
    email `json:"email"`
}
```

Inheritance Rules:

1. Child policies inherit all fields from parent policies
2. Child field metadata overrides parent metadata for the same field
3. Multiple inheritance is supported with comma separation

### Multiple Inheritance

```vex
policy Timestamped {
    created_at `db:"created_at"`,
    updated_at `db:"updated_at"`
}

policy SoftDelete {
    deleted_at `db:"deleted_at"`,
    is_deleted `db:"is_deleted" default:"false"`
}

policy FullModel with Timestamped, SoftDelete {
    id `json:"id" db:"id"`,
    name `json:"name" db:"name"`
}
```

Resolution Order:

1. First parent policies processed left-to-right
2. Child policy fields override parent fields
3. Later parents can override earlier parents

---

## Struct Application

### Basic Application

Apply policies to structs using the `with` clause:

```vex
struct User with APIModel {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
}

struct Product with APIModel, ValidationRules {
    id: i32,
    name: string,
    price: f64,
    category: string,
}
```

Effects:

- All policy fields must exist in the struct
- Metadata is applied to matching fields
- Struct gains the combined metadata from all policies

### Field Requirements

When a policy is applied, the struct must contain all fields defined in the policy:

```vex
policy RequiredFields {
    id `required:"true"`,
    name `required:"true"`
}

// ✅ Valid - has both required fields
struct User with RequiredFields {
    id: i32,
    name: string,
    email: string,  // Extra fields allowed
}

// ❌ Invalid - missing 'name' field
struct Incomplete with RequiredFields {
    id: i32,
    // name field missing
}
```

---

## Metadata Resolution

### Merge Order

When multiple sources define metadata for the same field, they are merged with this precedence:

1. Inline metadata (highest precedence)
2. Child policy metadata
3. Parent policy metadata (lowest precedence)

### Example Resolution

```vex
policy Base {
    id `db:"id"`,
    name `db:"name"`
}

policy API with Base {
    id `json:"id"`,     // Overrides db:"id"
    name `json:"name"`  // Overrides db:"name"
}

struct User with API {
    id: i32 `primary_key:"true"`,  // Overrides json:"id"
    name: string,                  // Uses json:"name"
}
```

Final metadata for `id` field:

- `primary_key:"true"` (from inline)
- `json:"id"` (from API policy, but overridden by inline)

Final metadata for `name` field:

- `json:"name"` (from API policy)

---

## Inline Metadata

### Field-Level Metadata

You can add metadata directly to struct fields:

```vex
struct User with APIModel {
    id: i32 `primary_key:"true" auto_increment:"true"`,
    name: string `max_length:"100"`,
    email: string `unique:"true"`,
    created_at: i64 `default:"now()"`,
}
```

Use Cases:

- Field-specific overrides
- Additional constraints not in policies
- Database-specific annotations
- Validation rules

### Metadata Combination

Inline metadata is merged with policy metadata:

```vex
policy APIModel {
    id `json:"id"`,
    name `json:"name"`
}

struct User with APIModel {
    id: i32 `db:"user_id" primary_key:"true"`,  // Combines with json:"id"
    name: string `max_length:"50"`,             // Adds to json:"name"
}
```

Result for `id`:

- `json:"id"` (from policy)
- `db:"user_id"` (from inline)
- `primary_key:"true"` (from inline)

---

## Use Cases

### 1. API Serialization

```vex
policy JSONAPI {
    id `json:"id"`,
    name `json:"name"`,
    email `json:"email"`,
    created_at `json:"created_at"`,
    updated_at `json:"updated_at"`
}

struct User with JSONAPI {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
    updated_at: i64,
}
```

### 2. Database Mapping

```vex
policy DatabaseModel {
    id `db:"id" type:"INTEGER" primary_key:"true"`,
    name `db:"name" type:"VARCHAR(100)" not_null:"true"`,
    email `db:"email" type:"VARCHAR(255)" unique:"true"`,
    created_at `db:"created_at" type:"TIMESTAMP" default:"CURRENT_TIMESTAMP"`
}

struct User with DatabaseModel {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
}
```

### 3. Validation Rules

```vex
policy Validation {
    age `min:"0" max:"150"`,
    email `format:"email" required:"true"`,
    phone `pattern:"^\\+?[1-9]\\d{1,14}$"`,
    salary `min:"0"`
}

struct Employee with Validation {
    name: string,
    age: i32,
    email: string,
    phone: string,
    salary: f64,
}
```

### 4. UI Generation

```vex
policy FormUI {
    name `ui:"text" label:"Full Name" required:"true"`,
    email `ui:"email" label:"Email Address" required:"true"`,
    age `ui:"number" label:"Age" min:"0" max:"150"`,
    department `ui:"select" label:"Department" options:"Engineering,Sales,Marketing"`
}

struct Employee with FormUI {
    name: string,
    email: string,
    age: i32,
    department: string,
}
```

### 5. Multi-Format Support

```vex
policy MultiFormat {
    id `json:"id" xml:"id" db:"id"`,
    name `json:"name" xml:"name" db:"name"`,
    data `json:"data" xml:"data" db:"data" type:"JSONB"`
}

struct Document with MultiFormat {
    id: i32,
    name: string,
    data: string,  // JSON string
}
```

---

## Examples

### Complete API Model

```vex
// Base model with common fields
policy BaseModel {
    id `json:"id" db:"id" type:"INTEGER" primary_key:"true"`,
    created_at `json:"created_at" db:"created_at" type:"TIMESTAMP"`,
    updated_at `json:"updated_at" db:"updated_at" type:"TIMESTAMP"`
}

// API-specific extensions
policy APIModel with BaseModel {
    id `xml:"id"`,  // Add XML support
    name `json:"name" xml:"name" db:"name" required:"true"`,
    email `json:"email" xml:"email" db:"email" format:"email"`
}

// Validation rules
policy UserValidation {
    name `min_length:"2" max_length:"100"`,
    email `format:"email" unique:"true"`,
    age `min:"13" max:"150"`
}

// Complete user model
struct User with APIModel, UserValidation {
    id: i32,
    name: string,
    email: string,
    age: i32 `json:"age" db:"age"`,  // Additional inline metadata
    created_at: i64,
    updated_at: i64,
}

fn main(): i32 {
    let user = User {
        id: 1,
        name: "Alice Johnson",
        email: "alice@example.com",
        age: 30,
        created_at: 1699000000,
        updated_at: 1699000000,
    };

    // At compile time, the metadata is available for:
    // - JSON serialization/deserialization
    // - Database ORM operations
    // - API documentation generation
    // - Form validation
    // - UI component generation

    return 0;
}
```

### Policy Inheritance Chain

```vex
// Foundation policies
policy Identifiable {
    id `type:"uuid" required:"true"`
}

policy Timestamped with Identifiable {
    created_at `type:"datetime"`,
    updated_at `type:"datetime"`
}

policy SoftDelete with Timestamped {
    deleted_at `type:"datetime"`,
    is_deleted `type:"boolean" default:"false"`
}

// Domain policies
policy APIModel with SoftDelete {
    id `json:"id"`,
    name `json:"name"`,
    description `json:"description"`
}

policy DatabaseModel with SoftDelete {
    id `db:"id" primary_key:"true"`,
    name `db:"name" not_null:"true"`,
    description `db:"description"`
}

// Concrete usage
struct Product with APIModel, DatabaseModel {
    id: string,
    name: string,
    description: string,
    created_at: i64,
    updated_at: i64,
    deleted_at: i64,
    is_deleted: bool,
}
```

### Metadata-Driven Code Generation

```vex
policy CRUD {
    id `primary_key:"true" auto_increment:"true"`,
    name `unique:"true" index:"name_idx"`,
    created_at `default:"CURRENT_TIMESTAMP"`,
    updated_at `on_update:"CURRENT_TIMESTAMP"`
}

policy APIEndpoints {
    id `route:"/api/{id}" methods:"GET,PUT,DELETE"`,
    name `route:"/api/search" methods:"GET" query:"name"`
}

struct User with CRUD, APIEndpoints {
    id: i32,
    name: string,
    email: string,
    created_at: i64,
    updated_at: i64,
}

// This could generate:
// 1. Database table creation
// 2. CRUD repository methods
// 3. REST API endpoints
// 4. OpenAPI documentation
// 5. Frontend forms and components
```

---

Previous: 19PackageManager.md
