# Vex Stdlib Planning - 05: Math and Random

**Priority:** 5
**Status:** Partial (math exists, others missing)
**Dependencies:** builtin

## 📦 Packages in This Category

### 5.1 math (extend existing)
**Status:** ✅ Exists (extend with missing functions)
**Description:** Mathematical functions and constants

#### Current Implementation
- Basic arithmetic functions exist via C runtime

#### Required Extensions
```vex
// Constants
const E: f64 = 2.718281828459045
const PI: f64 = 3.141592653589793
const PHI: f64 = 1.618033988749895
const SQRT2: f64 = 1.414213562373095
const SQRT_E: f64 = 1.648721271936509
const LN2: f64 = 0.693147180559945
const LN10: f64 = 2.302585092994046

// Additional functions
fn abs(x: f64): f64
fn signbit(x: f64): bool
fn copysign(x: f64, y: f64): f64
fn dim(x: f64, y: f64): f64
fn max(x: f64, y: f64): f64
fn min(x: f64, y: f64): f64

// Rounding
fn ceil(x: f64): f64
fn floor(x: f64): f64
fn trunc(x: f64): f64
fn round(x: f64): f64
fn round_to_even(x: f64): f64

// Exponents and logarithms
fn exp(x: f64): f64
fn exp2(x: f64): f64
fn expm1(x: f64): f64
fn log(x: f64): f64
fn log2(x: f64): f64
fn log10(x: f64): f64
fn log1p(x: f64): f64
fn log_b(x: f64, base: f64): f64

// Powers and roots
fn pow(x: f64, y: f64): f64
fn pow10(e: int): f64
fn sqrt(x: f64): f64
fn cbrt(x: f64): f64
fn hypot(p: f64, q: f64): f64

// Trigonometric
fn sin(x: f64): f64
fn cos(x: f64): f64
fn tan(x: f64): f64
fn asin(x: f64): f64
fn acos(x: f64): f64
fn atan(x: f64): f64
fn atan2(y: f64, x: f64): f64

// Hyperbolic
fn sinh(x: f64): f64
fn cosh(x: f64): f64
fn tanh(x: f64): f64
fn asinh(x: f64): f64
fn acosh(x: f64): f64
fn atanh(x: f64): f64

// Special functions
fn erf(x: f64): f64
fn erfc(x: f64): f64
fn gamma(x: f64): f64
fn lgamma(x: f64): f64

// Integer functions
fn gcd(a: i64, b: i64): i64
fn lcm(a: i64, b: i64): i64
fn factorial(n: u64): u64
```

#### Dependencies
- builtin

### 5.2 rand
**Status:** ❌ Missing (critical for many applications)
**Description:** Random number generation

#### Required Types
```vex
trait Source {
    fn int63(self: &mut Self): i64
    fn seed(self: &mut Self, seed: i64)
}

trait Rand {
    fn int(self: &mut Self): int
    fn intn(self: &mut Self, n: int): int
    fn int63(self: &mut Self): i64
    fn int63n(self: &mut Self, n: i64): i64
    fn float64(self: &mut Self): f64
    fn perm(self: &mut Self, n: int): []int
    fn shuffle(self: &mut Self, n: int, swap: fn(i: int, j: int))
}

struct Rand {
    src: Source,
    // internal state
}
```

#### Required Functions
```vex
// Global random generator
static global_rand: Rand

// Basic random functions
fn int(): int
fn intn(n: int): int
fn float64(): f64
fn perm(n: int): []int

// Seeding
fn seed(seed: i64)

// Custom generators
fn new(src: Source): Rand
fn new_source(seed: i64): Source
fn new_zipf(src: Source, s: f64, v: f64, imax: u64): Zipf
fn new_normal(src: Source, mean: f64, stddev: f64): Normal
fn new_exp(src: Source, lambda: f64): Exp
```

#### Dependencies
- builtin
- math

#### Notes
- **Security:** Cryptographically secure random needed
- **Performance:** Fast pseudo-random generation

### 5.3 big
**Status:** ❌ Missing (important for cryptography)
**Description:** Arbitrary-precision arithmetic

#### Required Types
```vex
struct Int {
    neg: bool,
    abs: Vec<u32>,
}

struct Float {
    prec: u32,
    mode: RoundingMode,
    acc: Accuracy,
    form: Form,
    neg: bool,
    mant: Vec<u32>,
    exp: i32,
}

struct Rat {
    num: Int,
    den: Int,
}

enum RoundingMode {
    ToNearestEven,
    ToNearestAway,
    ToZero,
    AwayFromZero,
    ToNegativeInf,
    ToPositiveInf,
}

enum Accuracy {
    Exact,
    Below,
    Above,
}
```

#### Required Functions
```vex
// Int operations
fn new_int(x: i64): Int
fn set_string(s: str, base: int): Result<Int, Error>
fn string(z: &Int, base: int): str
fn add(x: &Int, y: &Int): Int
fn sub(x: &Int, y: &Int): Int
fn mul(x: &Int, y: &Int): Int
fn div(x: &Int, y: &Int): Int
fn mod(x: &Int, y: &Int): Int
fn cmp(x: &Int, y: &Int): int
fn abs(x: &Int): Int
fn neg(x: &Int): Int

// Float operations
fn new_float(x: f64): Float
fn set_prec(prec: u32): u32
fn add(x: &Float, y: &Float): Float
fn sub(x: &Float, y: &Float): Float
fn mul(x: &Float, y: &Float): Float
fn quot(x: &Float, y: &Float): Float

// Rat operations
fn new_rat(a: i64, b: i64): Rat
fn add(x: &Rat, y: &Rat): Rat
fn mul(x: &Rat, y: &Rat): Rat
```

#### Dependencies
- builtin
- strings
- strconv

### 5.4 cmplx
**Status:** ❌ Missing (specialized use)
**Description:** Complex number operations

#### Required Types
```vex
struct Complex {
    real: f64,
    imag: f64,
}
```

#### Required Functions
```vex
// Construction
fn rect(real: f64, imag: f64): Complex
fn polar(r: f64, theta: f64): Complex

// Access
fn real(c: Complex): f64
fn imag(c: Complex): f64

// Operations
fn abs(c: Complex): f64
fn phase(c: Complex): f64
fn conj(c: Complex): Complex
fn add(a: Complex, b: Complex): Complex
fn sub(a: Complex, b: Complex): Complex
fn mul(a: Complex, b: Complex): Complex
fn div(a: Complex, b: Complex): Complex

// Functions
fn sqrt(c: Complex): Complex
fn exp(c: Complex): Complex
fn log(c: Complex): Complex
fn log10(c: Complex): Complex
fn pow(a: Complex, b: Complex): Complex
fn sin(c: Complex): Complex
fn cos(c: Complex): Complex
fn tan(c: Complex): Complex
```

#### Dependencies
- builtin
- math

## 🎯 Implementation Priority

1. **math extensions** - Complete basic math functions
2. **rand** - Pseudo-random number generation
3. **big** - Arbitrary precision arithmetic
4. **cmplx** - Complex numbers

## ⚠️ Language Feature Issues

- **Operator Overloading:** Complex numbers need +, -, *, / operators
- **Static Variables:** Global random generator state
- **Large Structs:** Big Int/Float need efficient memory handling

## 📋 Missing Critical Dependencies

- **Complex Type:** Built-in complex number support
- **Big Integer Literals:** Support for large number literals
- **Math Intrinsics:** Hardware-accelerated math functions

## 🚀 Next Steps

1. Extend math package with missing functions
2. Implement rand package with basic PRNG
3. Add big.Int for arbitrary precision
4. Implement complex number operations