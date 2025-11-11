# Control Flow

Version: 0.1.0 
Last Updated: November 3, 2025

This document defines control flow constructs in the Vex programming language.

---

## Conditional Statements

### If Expression

Basic Syntax:

```vex
if condition {
    // body
}
```

Properties:

- Condition must be `bool` type (no implicit conversion)
- Braces are required (no braceless syntax)
- Body is a new scope

Example:

```vex
let x = 10;
if x > 5 {
    // x is greater than 5
}
```

### If-Else

```vex
if condition {
    // true branch
} else {
    // false branch
}
```

Answer: `select` statement (High Priority) - Go-style channel selection

```vex
select {
    case msg = <-ch1:
        println("Received from ch1");
    case ch2 <- value:
        println("Sent to ch2");
    default:
        println("No channel ready");
}
```

Channel implementation ile birlikte gelecek. Şu an channels implement edilmemiş, o yüzden bu dokümanda yok. 13_Concurrency.md'de mention edilmeli.

Example:

```vex
let age = 18;
if age >= 18 {
    // adult
} else {
    // minor
}
```

### If-Elif-Else Chain (v0.1)

Use `elif` for else-if chains:

```vex
if condition1 {
    // first branch
} elif condition2 {
    // second branch
} elif condition3 {
    // third branch
} else {
    // default branch
}
```

Example:

```vex
let score = 85;
if score >= 90 {
    // grade A
} elif score >= 80 {
    // grade B
} elif score >= 70 {
    // grade C
} else {
    // grade F
}
```

Note: `elif` keyword introduced in v0.1 (replaces older `else if` syntax)

### Nested If

```vex
if outer_condition {
    if inner_condition {
        // nested body
    }
}
```

Example:

```vex
let age = 20;
let has_license = true;

if age >= 18 {
    if has_license {
        // can drive
    }
}
```

### If as Expression (Future)

```vex
let value = if condition { 10 } else { 20 };
```

---

## Pattern Matching

### Match Expression

Syntax:

```vex
match value {
    pattern1 => { body1 }
    pattern2 => { body2 }
    _ => { default }
}
```

Properties:

- Must be exhaustive (all cases covered)
- Evaluates top-to-bottom (first match wins)
- `_` is wildcard pattern (matches anything)

### Literal Patterns

```vex
match x {
    0 => { /* zero */ }
    1 => { /* one */ }
    2 => { /* two */ }
    _ => { /* other */ }
}
```

Example:

```vex
let day = 3;
match day {
    1 => { /* Monday */ }
    2 => { /* Tuesday */ }
    3 => { /* Wednesday */ }
    4 => { /* Thursday */ }
    5 => { /* Friday */ }
    6 => { /* Saturday */ }
    7 => { /* Sunday */ }
    _ => { /* invalid */ }
}
```

### Enum Patterns

```vex
enum Color {
    Red,
    Green,
    Blue,
}

let color = Color.Red;
match color {
    Color.Red => { /* red */ }
    Color.Green => { /* green */ }
    Color.Blue => { /* blue */ }
}
```

Exhaustiveness Check:

```vex
match color {
    Color.Red => { }
    Color.Green => { }
    // ERROR: Missing Color.Blue case
}
```

### Or Patterns (v0.1)

Match multiple patterns with `|`:

```vex
match x {
    1 | 2 | 3 => { /* low */ }
    4 | 5 | 6 => { /* medium */ }
    7 | 8 | 9 => { /* high */ }
    _ => { /* other */ }
}
```

Example:

```vex
match day {
    1 | 2 | 3 | 4 | 5 => { /* weekday */ }
    6 | 7 => { /* weekend */ }
    _ => { /* invalid */ }
}
```

### Tuple Patterns

```vex
let point = (10, 20);
match point {
    (0, 0) => { /* origin */ }
    (0, y) => { /* on y-axis */ }
    (x, 0) => { /* on x-axis */ }
    (x, y) => { /* general point */ }
}
```

Destructuring:

```vex
let pair = (1, 2);
match pair {
    (a, b) => {
        // a = 1, b = 2
    }
}
```

### Struct Patterns (Future)

```vex
struct Point { x: i32, y: i32 }

let p = Point { x: 10, y: 20 };
match p {
    Point { x: 0, y: 0 } => { /* origin */ }
    Point { x, y: 0 } => { /* on x-axis, x = 10 */ }
    Point { x, y } => { /* general, x=10, y=20 */ }
}
```

### Range Patterns (Future)

```vex
match age {
    0..=12 => { /* child */ }
    13..=17 => { /* teen */ }
    18..=64 => { /* adult */ }
    65.. => { /* senior */ }
}
```

### Guards (Future)

Add conditions to patterns:

```vex
match x {
    n if n < 0 => { /* negative */ }
    n if n == 0 => { /* zero */ }
    n if n > 0 => { /* positive */ }
}
```

### Data-Carrying Enum Patterns (Future)

```vex
enum Option<T> {
    Some(T),
    None,
}

let value = Some(42);
match value {
    Some(x) => { /* x = 42 */ }
    None => { /* no value */ }
}
```

---

## Loops

### While Loop

Syntax:

```vex
while condition {
    // body
}
```

Example:

```vex
let! counter = 0;
while counter < 10 {
    counter = counter + 1;
}
```

Infinite Loop:

```vex
while true {
    // runs forever (until break)
}
```

### For Loop

Syntax:

```vex
for variable in start..end {
    // body
}
```

Range-Based:

```vex
for i in 0..10 {
    // i = 0, 1, 2, ..., 9
}
```

Example:

```vex
let! sum = 0;
for i in 1..11 {
    sum = sum + i;
}
// sum = 55 (1+2+...+10)
```

Inclusive Range:

```vex
for i in 0..=10 {
    // i = 0, 1, 2, ..., 10 (includes 10)
}
```

Operators:

- `..` - Exclusive range: `0..10` → 0, 1, 2, ..., 9
- `..=` - Inclusive range: `0..=10` → 0, 1, 2, ..., 10

### Loop (Infinite Loop) (Future)

```vex
loop {
    // runs forever
    if condition {
        break;
    }
}
```

Equivalent to:

```vex
while true {
    // body
}
```

### For-Each (Future)

Iterate over collections:

```vex
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    // num = 1, then 2, then 3, ...
}
```

With Index:

```vex
for (index, value) in numbers.enumerate() {
    // index = 0, 1, 2, ...
    // value = 1, 2, 3, ...
}
```

---

## Control Transfer

### Break

Exit from loop early:

```vex
let! i = 0;
while i < 10 {
    if i == 5 {
        break;  // Exit loop
    }
    i = i + 1;
}
// i = 5
```

In Match (Future):

```vex
while true {
    match get_input() {
        "quit" => { break; }
        cmd => { process(cmd); }
    }
}
```

### Continue

Skip to next iteration:

```vex
for i in 0..10 {
    if i % 2 == 0 {
        continue;  // Skip even numbers
    }
    // Only odd numbers reach here
}
```

Example:

```vex
let! count = 0;
for i in 1..101 {
    if i % 3 == 0 {
        continue;  // Skip multiples of 3
    }
    count = count + 1;
}
// count = 67 (100 - 33 multiples of 3)
```

### Return

Exit from function:

```vex
fn find(arr: [i32; 10], target: i32): i32 {
    for i in 0..10 {
        if arr[i] == target {
            return i;  // Found, exit function
        }
    }
    return -1;  // Not found
}
```

Early Return:

```vex
fn validate(x: i32): bool {
    if x < 0 {
        return false;  // Early exit
    }
    if x > 100 {
        return false;  // Early exit
    }
    return true;
}
```

### Labeled Breaks (Future)

Break from nested loops:

```vex
'outer: for i in 0..10 {
    for j in 0..10 {
        if i * j > 50 {
            break 'outer;  // Break outer loop
        }
    }
}
```

---

## Error Handling

### Result Type (Future)

Use union types for error handling:

```vex
type Result<T> = (T | error);

fn divide(a: i32, b: i32): Result<i32> {
    if b == 0 {
        return "Division by zero";
    }
    return a / b;
}
```

Pattern Matching on Result:

```vex
let result = divide(10, 2);
match result {
    value when value is i32 => {
        // Success: value = 5
    }
    err when err is error => {
        // Error: handle err
    }
}
```

### Option Type (Future)

Represent optional values:

```vex
type Option<T> = (T | nil);

fn find(arr: [i32], target: i32): Option<i32> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return i;
        }
    }
    return nil;
}
```

Unwrapping:

```vex
let result = find([1, 2, 3], 2);
match result {
    index when index is i32 => { /* found at index */ }
    nil => { /* not found */ }
}
```

### Try-Catch (Future Consideration)

```vex
try {
    let result = risky_operation();
    process(result);
} catch err {
    handle_error(err);
}
```

### Panic

Abort program execution:

```vex
fn unreachable_code() {
    @unreachable();  // Compiler hint
}

fn assert_positive(x: i32) {
    if x <= 0 {
        panic("Value must be positive");
    }
}
```

---

## Examples

### If-Elif-Else

```vex
fn classify_age(age: i32): i32 {
    if age < 0 {
        return -1;  // Invalid
    } elif age < 13 {
        return 0;   // Child
    } elif age < 20 {
        return 1;   // Teen
    } elif age < 65 {
        return 2;   // Adult
    } else {
        return 3;   // Senior
    }
}
```

### Match with Enums

```vex
enum Status {
    Active = 0,
    Inactive = 1,
    Pending = 2,
}

fn handle_status(status: Status): i32 {
    match status {
        Active => {
            return 1;
        }
        Inactive => {
            return 0;
        }
        Pending => {
            return -1;
        }
    }
}
```

### While Loop

```vex
fn count_down(n: i32): i32 {
    let! counter = n;
    while counter > 0 {
        counter = counter - 1;
    }
    return counter;  // 0
}
```

### For Loop

```vex
fn sum_range(start: i32, end: i32): i32 {
    let! sum = 0;
    for i in start..end {
        sum = sum + i;
    }
    return sum;
}

fn main(): i32 {
    return sum_range(1, 11);  // 55
}
```

### Break and Continue

```vex
fn find_first_even(numbers: [i32; 10]): i32 {
    for i in 0..10 {
        if numbers[i] % 2 == 1 {
            continue;  // Skip odd numbers
        }
        return numbers[i];  // Return first even
    }
    return -1;  // No even number found
}
```

---

## Defer Statement

### Syntax

Purpose: Execute code when function exits, regardless of how it exits.

Status: Fully implemented - deferred statements execute in LIFO order on function exit

Keyword: `defer`

```vex
fn example() {
    defer cleanup();  // Executes when function returns
    // ... function body
}
```

### Basic Usage

```vex
fn read_file(path: string): string {
    let file = open(path);
    defer close(file);  // Always closes, even on error

    if !file.is_valid() {
        return "";  // defer executes before return
    }

    return file.read_all();
}  // defer executes here
```

### Multiple Defer Statements

Execution Order: LIFO (Last In, First Out) - Reverse order of declaration

```vex
fn process_data() {
    defer println("Step 3: Final cleanup");
    defer println("Step 2: Release lock");
    defer println("Step 1: Close connection");

    // Function body
    println("Processing...");
}

// Output:
// Processing...
// Step 1: Close connection
// Step 2: Release lock
// Step 3: Final cleanup
```

### Resource Management

File Handling:

```vex
fn copy_file(src: string, dst: string): bool {
    let src_file = open(src);
    defer close(src_file);

    let dst_file = create(dst);
    defer close(dst_file);

    // Both files automatically closed on return
    return copy_content(src_file, dst_file);
}
```

Memory Management:

```vex
fn process_buffer(): i32 {
    let buffer = allocate(1024);
    defer free(buffer);

    // Use buffer...
    let result = compute(buffer);

    return result;
}  // buffer freed automatically
```

Lock Management:

```vex
fn update_shared_data(mutex: &Mutex!, data: i32) {
    mutex.lock();
    defer mutex.unlock();

    // Critical section
    shared_value = data;

    // mutex unlocked automatically, even if panic occurs
}
```

### Defer with Closures (Future)

```vex
fn complex_cleanup() {
    let! counter = 0;
    defer {
        // Closure can access function variables
        println("Counter was: " + counter);
    };

    counter = 42;
}  // Prints: "Counter was: 42"
```

### Error Handling with Defer

```vex
fn risky_operation(): (i32 | error) {
    let resource = acquire();
    defer release(resource);

    if problem() {
        return "Error occurred";  // defer runs before return
    }

    return 42;
}
```

### Common Patterns

1. RAII-style Resource Management:

```vex
fn database_transaction(): bool {
    let tx = db.begin_transaction();
    defer tx.rollback();  // Safety net

    if !tx.insert(...) {
        return false;  // Rollback happens
    }

    tx.commit();
    return true;
}
```

2. Cleanup Stack:

```vex
fn multi_step_process(): i32 {
    let step1 = init_step1();
    defer cleanup_step1(step1);

    let step2 = init_step2();
    defer cleanup_step2(step2);

    let step3 = init_step3();
    defer cleanup_step3(step3);

    return execute();
}  // Cleanup in reverse: step3, step2, step1
```

3. Timing and Logging:

```vex
fn measured_operation() {
    let start_time = now();
    defer {
        let elapsed = now() - start_time;
        println("Operation took: " + elapsed + "ms");
    };

    // Expensive operation
    compute_heavy_task();
}
```

### Comparison with Other Languages

• Feature — Vex — Go — Rust — C++
| Keyword | `defer` | `defer` | N/A | N/A |
• RAII — Manual — Manual — Automatic — Manual
• Execution — On exit — On exit — On drop — On scope
• Order — LIFO — LIFO — LIFO (drop) — LIFO
• Closures — Yes — Yes — Yes (Drop) — Lambda

### Implementation Status

- Keyword reserved (`defer`)
- Parser support (COMPLETE - Nov 9, 2025)
- Codegen implemented (LIFO execution)
- Stack unwinding integration working
- Priority: COMPLETE

Examples: See `examples/defer_*.vx` for working demonstrations

---

### Nested Loops

```vex
fn matrix_sum(rows: i32, cols: i32): i32 {
    let! sum = 0;
    for i in 0..rows {
        for j in 0..cols {
            sum = sum + (i * cols + j);
        }
    }
    return sum;
}
```

### Early Return

```vex
fn is_prime(n: i32): bool {
    if n <= 1 {
        return false;  // Early return
    }
    if n == 2 {
        return true;   // Early return
    }
    if n % 2 == 0 {
        return false;  // Early return
    }

    // Check odd divisors
    let! i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i = i + 2;
    }
    return true;
}
```

---

## Best Practices

### 1. Use Match Over If Chains

```vex
// Good: Clear, exhaustive
match status {
    Active => { }
    Inactive => { }
    Pending => { }
}

// Bad: Verbose, error-prone
if status == Active {
    // ...
} elif status == Inactive {
    // ...
} elif status == Pending {
    // ...
}
```

### 2. Prefer Early Returns

```vex
// Good: Early validation
fn process(x: i32): i32 {
    if x < 0 {
        return -1;
    }
    if x == 0 {
        return 0;
    }
    // Main logic
    return x * 2;
}

// Bad: Deep nesting
fn process(x: i32): i32 {
    if x >= 0 {
        if x != 0 {
            // Main logic
            return x * 2;
        } else {
            return 0;
        }
    } else {
        return -1;
    }
}
```

### 3. Avoid Deep Nesting

```vex
// Good: Flat structure
fn validate(x: i32, y: i32, z: i32): bool {
    if x < 0 { return false; }
    if y < 0 { return false; }
    if z < 0 { return false; }
    return true;
}

// Bad: Deep nesting
fn validate(x: i32, y: i32, z: i32): bool {
    if x >= 0 {
        if y >= 0 {
            if z >= 0 {
                return true;
            }
        }
    }
    return false;
}
```

### 4. Use Descriptive Conditions

```vex
// Good: Named condition
let is_adult = age >= 18;
let has_permission = role == "admin";

if is_adult && has_permission {
    // Clear intent
}

// Bad: Complex inline condition
if age >= 18 && role == "admin" && status == "active" {
    // What does this check?
}
```

### 5. Limit Loop Complexity

```vex
// Good: Simple loop body
for i in 0..10 {
    process_item(i);
}

// Bad: Complex logic in loop
for i in 0..10 {
    if condition1 {
        if condition2 {
            for j in 0..5 {
                // Too complex
            }
        }
    }
}
```

---

---

## Select Statement (Future)

### Syntax (Go-style)

Purpose: Wait on multiple channel operations

```vex
select {
    case msg = <-ch1:
        println("Received from ch1");
    case ch2 <- value:
        println("Sent to ch2");
    case msg = <-ch3:
        println("Received from ch3");
    default:
        println("No channel ready");
}
```

### Semantics

- Blocks until one case is ready
- If multiple cases ready, randomly chooses one
- `default` case executes immediately if no channel ready
- Without `default`, blocks forever if no channel ready

### Example: Timeout Pattern

```vex
import { channel, timeout } from "sync";

fn fetch_with_timeout(): (string | error) {
    let result_ch = channel<string>();
    let timeout_ch = timeout(5000); // 5 seconds

    go fetch_data(result_ch);

    select {
        case data = <-result_ch:
            return data;
        case <-timeout_ch:
            return "Timeout error";
    }
}
```

### Current Status

Syntax: `select` keyword reserved 
Parser: Partial (keyword recognized, AST node exists) 
Channels: MPSC channels implemented (lock-free ring buffer) 
Priority: � Medium (Channel infrastructure complete, select syntax pending)

Note: Basic channel operations (`send`, `recv`, `close`) fully working. Multi-channel `select` syntax planned.

See 13_Concurrency.md for full concurrency model.

### Switch Statement

C-style switch with integer values:

Syntax: `switch value { case val: { } default: { } }`

```vex
switch day {
    case 1:
        println("Monday");
    case 2:
        println("Tuesday");
    case 3:
        println("Wednesday");
    case 4:
        println("Thursday");
    case 5:
        println("Friday");
    case 6:
        println("Saturday");
    case 7:
        println("Sunday");
    default:
        println("Invalid day");
}
```

Properties:

- Only works with integer types (i32, u32, etc.)
- No implicit fallthrough (unlike C)
- Must have `default` case (unlike C)
- Each case must be a compile-time constant

Differences from C:

- No fallthrough by default
- Requires `default` case
- Only integer types supported
- No expression cases (use `match` instead)

---

## Control Flow Summary

• Construct — Syntax — Use Case — Status
| If | `if cond { }` | Simple branching | |
| If-Else | `if cond { } else { }` | Binary choice | |
| If-Elif-Else | `if { } elif { } else { }` | Multiple conditions | |
| Match | `match val { pat => { } }` | Pattern matching | |
| Switch | `switch val { case ... }` | Integer switching | |
| While | `while cond { }` | Condition-based loop | |
| For | `for i in range { }` | Iteration | |
| Defer | `defer cleanup();` | LIFO cleanup | |
| Select | `select { case ... }` | Channel multiplexing | |
| Break | `break;` | Exit loop | |
| Continue | `continue;` | Skip iteration | |
| Return | `return value;` | Exit function | |

---

Previous: 05Functionsand_Methods.md 
Next: 07StructsandDataTypes.md
