# Policy System

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's policy system, which provides metadata annotations for structs and fields, enabling features like serialization, validation, and code generation.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1
8. \1

---

## Overview

The policy system allows you to define reusable metadata templates that can be applied to structs and their fields. This enables:

- **Serialization**: JSON, XML, database mappings
- **Validation**: Field constraints and rules
- **Code Generation**: Automatic CRUD operations, API endpoints
- **Documentation**: Field descriptions and examples
- **UI Generation**: Form layouts and input types

### Key Concepts

- **Policy**: A named collection of field metadata
- **Metadata**: Key-value annotations using backtick syntax
- **Composition**: Policies can inherit from parent policies
- **Application**: Structs apply policies using `with` clause
- **Resolution**: Metadata is merged with precedence rules

---

## Policy Declarations

### Basic Policy Syntax

``````vex
policy PolicyName {
    field_name `key:"value"`,
    another_field `type:"string" required:"true"`
}
```

**Components:**

- `policy` keyword
- Policy name (identifier)
- Field declarations with metadata in backticks
- Comma-separated fields

### Metadata Syntax

Metadata uses a simple key-value format within backticks:

``````vex
`key1:"value1" key2:"value2" key3:"true"`
```

**Rules:**

- Keys and values are strings
- Multiple key-value pairs separated by spaces
- Values can contain special characters
- No nested structures (flat key-value pairs)

### Example Policies

[12 lines code: ```vex]

---

## Policy Composition

### Parent Policies

Policies can inherit metadata from parent policies:

[11 lines code: ```vex]

**Inheritance Rules:**

1. Child policies inherit all fields from parent policies
2. Child field metadata overrides parent metadata for the same field
3. Multiple inheritance is supported with comma separation

### Multiple Inheritance

[14 lines code: ```vex]

**Resolution Order:**

1. First parent policies processed left-to-right
2. Child policy fields override parent fields
3. Later parents can override earlier parents

---

## Struct Application

### Basic Application

Apply policies to structs using the `with` clause:

[13 lines code: ```vex]

**Effects:**

- All policy fields must exist in the struct
- Metadata is applied to matching fields
- Struct gains the combined metadata from all policies

### Field Requirements

When a policy is applied, the struct must contain all fields defined in the policy:

[17 lines code: ```vex]

---

## Metadata Resolution

### Merge Order

When multiple sources define metadata for the same field, they are merged with this precedence:

1. **Inline metadata** (highest precedence)
2. **Child policy metadata**
3. **Parent policy metadata** (lowest precedence)

### Example Resolution

[14 lines code: ```vex]

**Final metadata for `id` field:**

- `primary_key:"true"` (from inline)
- `json:"id"` (from API policy, but overridden by inline)

**Final metadata for `name` field:**

- `json:"name"` (from API policy)

---

## Inline Metadata

### Field-Level Metadata

You can add metadata directly to struct fields:

``````vex
struct User with APIModel {
    id: i32 `primary_key:"true" auto_increment:"true"`,
    name: string `max_length:"100"`,
    email: string `unique:"true"`,
    created_at: i64 `default:"now()"`,
}
```

**Use Cases:**

- Field-specific overrides
- Additional constraints not in policies
- Database-specific annotations
- Validation rules

### Metadata Combination

Inline metadata is merged with policy metadata:

[9 lines code: ```vex]

**Result for `id`:**

- `json:"id"` (from policy)
- `db:"user_id"` (from inline)
- `primary_key:"true"` (from inline)

---

## Use Cases

### 1. API Serialization

[15 lines code: ```vex]

### 2. Database Mapping

[13 lines code: ```vex]

### 3. Validation Rules

[14 lines code: ```vex]

### 4. UI Generation

[13 lines code: ```vex]

### 5. Multi-Format Support

[11 lines code: ```vex]

---

## Examples

### Complete API Model

[50 lines code: ```vex]

### Policy Inheritance Chain

[38 lines code: ```vex]

### Metadata-Driven Code Generation

[26 lines code: ```vex]

---

**Previous**: \1

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/20_Policy_System.md
