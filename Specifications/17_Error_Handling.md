# Error Handling

**Version:** 0.9.2
**Last Updated:** November 2025

This document describes Vex's error handling system, including the `Result<T, E>` and `Option<T>` types, pattern matching for error handling, and best practices.

---

## Table of Contents

1. [Overview](#overview)
2. [Option<T> Type](#optiont-type)
3. [Result<T, E> Type](#resultt-e-type)
4. [Pattern Matching for Errors](#pattern-matching-for-errors)
5. [Error Propagation](#error-propagation)
6. [Custom Error Types](#custom-error-types)
7. [Best Practices](#best-practices)

---

## Overview

Vex provides two primary types for handling potential failure:

- **`Option<T>`**: Represents a value that may or may not exist
- **`Result<T, E>`**: Represents either success (`T`) or failure (`E`)

Both types encourage explicit handling of potential errors rather than exceptions or null pointers.

---

## Option<T> Type

The `Option<T>` type represents an optional value. It has two variants:

```vex
enum Option<T> {
    Some(T),
    None
}
```

### Basic Usage

```vex
// Function that may not find a value
fn find_user(id: i32): Option<User> {
    if id == 42 {
        return Some(User { id: 42, name: "Alice" });
    }
    return None;
}

// Using the result
let user = find_user(42);
match user {
    Some(u) => println("Found user: {}", u.name),
    None => println("User not found")
}
```

### Common Methods

```vex
let maybe_value: Option<i32> = Some(42);

// Check if value exists
if maybe_value.is_some() {
    println("Value exists");
}

// Get value with default
let value = maybe_value.unwrap_or(0); // Returns 42

// Transform option
let doubled = maybe_value.map(|x| x * 2); // Some(84)

// Chain operations
let result = maybe_value
    .filter(|x| x > 40)
    .map(|x| x + 1); // Some(43)
```

---

## Result<T, E> Type

The `Result<T, E>` type represents either success or failure. It has two variants:

```vex
enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

### Basic Usage

```vex
// Function that may fail
fn divide(a: i32, b: i32): Result<i32, String> {
    if b == 0 {
        return Err("Division by zero".to_string());
    }
    return Ok(a / b);
}

// Using the result
let result = divide(10, 2);
match result {
    Ok(value) => println("Result: {}", value),
    Err(error) => println("Error: {}", error)
}
```

### Common Methods

```vex
let result: Result<i32, String> = Ok(42);

// Check result type
if result.is_ok() {
    println("Success!");
}

// Get values
let value = result.unwrap(); // Panics if Err
let value_or_default = result.unwrap_or(0);

// Transform results
let doubled = result.map(|x| x * 2); // Ok(84)

// Handle errors
let error_message = result.unwrap_err(); // Panics if Ok

// Convert Option to Result
let option: Option<i32> = Some(42);
let result = option.ok_or("Value not found"); // Ok(42)
```

---

## Pattern Matching for Errors

Pattern matching provides elegant error handling:

```vex
fn process_data(data: Vec<i32>): Result<String, String> {
    // Validate input
    if data.is_empty() {
        return Err("No data provided".to_string());
    }

    // Process each item
    for item in data {
        match validate_item(item) {
            Ok(processed) => println("Processed: {}", processed),
            Err(error) => return Err(format("Validation failed: {}", error))
        }
    }

    Ok("All data processed successfully".to_string())
}

fn validate_item(item: i32): Result<i32, String> {
    if item < 0 {
        Err("Negative values not allowed".to_string())
    } else if item > 100 {
        Err("Values too large".to_string())
    } else {
        Ok(item * 2)
    }
}
```

### Nested Matching

```vex
fn complex_operation(): Result<i32, String> {
    let config = load_config()?;
    let data = fetch_data(config)?;
    let result = process_data(data)?;

    Ok(result)
}

// Equivalent to:
fn complex_operation_manual(): Result<i32, String> {
    match load_config() {
        Ok(config) => match fetch_data(config) {
            Ok(data) => match process_data(data) {
                Ok(result) => Ok(result),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        },
        Err(e) => Err(e)
    }
}
```

---

## Error Propagation

Vex supports the `?` operator for concise error propagation:

```vex
fn read_and_process_file(filename: String): Result<String, String> {
    let content = read_file(filename)?;      // Propagates file errors
    let data = parse_json(content)?;         // Propagates parse errors
    let result = validate_data(data)?;       // Propagates validation errors

    Ok(result)
}

// Equivalent without ? operator
fn read_and_process_file_manual(filename: String): Result<String, String> {
    match read_file(filename) {
        Ok(content) => match parse_json(content) {
            Ok(data) => match validate_data(data) {
                Ok(result) => Ok(result),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        },
        Err(e) => Err(e)
    }
}
```

### Early Returns

```vex
fn authenticate_user(username: String, password: String): Result<User, AuthError> {
    // Validate input
    if username.is_empty() {
        return Err(AuthError::InvalidUsername);
    }

    if password.is_empty() {
        return Err(AuthError::InvalidPassword);
    }

    // Attempt authentication
    let user = find_user(username)?;
    verify_password(user, password)?;

    Ok(user)
}
```

---

## Custom Error Types

Define custom error types using enums:

```vex
enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(i32)
}

enum ApiError {
    NetworkError(String),
    AuthenticationFailed,
    ValidationError(Vec<String>),
    DatabaseError(DatabaseError)
}

fn fetch_user(id: i32): Result<User, ApiError> {
    // Attempt database connection
    let connection = connect_to_db().map_err(|e| ApiError::DatabaseError(e))?;

    // Execute query
    match connection.query_user(id) {
        Ok(user) => Ok(user),
        Err(DatabaseError::NotFound(_)) => Err(ApiError::NotFound),
        Err(e) => Err(ApiError::DatabaseError(e))
    }
}
```

### Error Traits

Implement common error traits for better ergonomics:

```vex
trait Error {
    fn message(&self): String;
}

impl Error for DatabaseError {
    fn message(&self): String {
        match self {
            ConnectionFailed(msg) => format("Connection failed: {}", msg),
            QueryFailed(msg) => format("Query failed: {}", msg),
            NotFound(id) => format("User {} not found", id)
        }
    }
}
```

---

## Best Practices

### 1. Use Result for Operations That Can Fail

```vex
// Good: Explicit error handling
fn parse_number(input: String): Result<i32, ParseError> {
    // Implementation
}

// Bad: Using Option when you need error details
fn parse_number_bad(input: String): Option<i32> {
    // Can't provide specific error information
}
```

### 2. Define Specific Error Types

```vex
// Good: Specific errors
enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(Vec<String>)
}

// Bad: Generic string errors
fn load_config(): Result<Config, String> {
    // Error details lost in generic strings
}
```

### 3. Use ? for Early Returns

```vex
// Good: Clear control flow
fn process_request(req: Request): Result<Response, Error> {
    let user = authenticate(req)?;
    let data = validate_input(req)?;
    let result = perform_business_logic(user, data)?;

    Ok(create_response(result))
}
```

### 4. Handle Errors at Appropriate Levels

```vex
// Good: Handle errors where you can respond appropriately
fn handle_request(req: Request): Response {
    match process_request(req) {
        Ok(data) => Response::success(data),
        Err(Error::ValidationError(fields)) => Response::bad_request(fields),
        Err(Error::NotFound) => Response::not_found(),
        Err(_) => Response::internal_error()
    }
}
```

### 5. Avoid unwrap() in Production

```vex
// Bad: Will panic in production
let value = result.unwrap();

// Good: Handle the error
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e) // or provide default
};
```

### 6. Use expect() for Programming Errors

```vex
// Good: Document assumptions
let config = load_config().expect("Config file should always exist");

// Bad: Silent unwrap
let config = load_config().unwrap();
```

---

**Previous**: [16_Standard_Library.md](./16_Standard_Library.md)  
**Next**: [18_Raw_Pointers_and_FFI.md](./18_Raw_Pointers_and_FFI.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/17_Error_Handling.md
