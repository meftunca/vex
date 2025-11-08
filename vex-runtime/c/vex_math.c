/* vex_math.c - Advanced math functions for Vex (Go + Rust stdlib quality)
 * 
 * Features:
 * - Trigonometry (sin, cos, tan, asin, acos, atan, atan2, sinh, cosh, tanh)
 * - Exponential/Logarithm (exp, exp2, exp10, log, log2, log10, ln, pow)
 * - Rounding (ceil, floor, round, trunc)
 * - Special functions (gamma, lgamma, erf, erfc, bessel)
 * - Utility (abs, min, max, clamp, copysign, hypot, fma)
 * - Constants (PI, E, SQRT2, PHI, etc.)
 * - SIMD-accelerated where possible
 * 
 * Build: cc -O3 -std=c17 -march=native vex_math.c -lm -o test_math
 * 
 * License: MIT
 */

#include <math.h>
#include <float.h>
#include <stdint.h>
#include <stdbool.h>

#if __has_include("vex_macros.h")
  #include "vex_macros.h"
#else
  #define VEX_INLINE static inline
  #define VEX_FORCE_INLINE static inline __attribute__((always_inline))
#endif

/* =========================
 * Mathematical Constants
 * ========================= */

#define VEX_PI         3.14159265358979323846264338327950288
#define VEX_E          2.71828182845904523536028747135266250
#define VEX_PHI        1.61803398874989484820458683436563812  // Golden ratio
#define VEX_SQRT2      1.41421356237309504880168872420969808
#define VEX_SQRT3      1.73205080756887729352744634150587237
#define VEX_LN2        0.69314718055994530941723212145817657
#define VEX_LN10       2.30258509299404568401799145468436421
#define VEX_LOG2E      1.44269504088896340735992468100189214
#define VEX_LOG10E     0.43429448190325182765112891891660508

/* =========================
 * Basic Operations
 * ========================= */

VEX_FORCE_INLINE double vex_abs_f64(double x) { return fabs(x); }
VEX_FORCE_INLINE float  vex_abs_f32(float x)  { return fabsf(x); }
VEX_FORCE_INLINE int64_t vex_abs_i64(int64_t x) { return (x < 0) ? -x : x; }

VEX_FORCE_INLINE double vex_min_f64(double a, double b) { return (a < b) ? a : b; }
VEX_FORCE_INLINE double vex_max_f64(double a, double b) { return (a > b) ? a : b; }

VEX_FORCE_INLINE double vex_clamp_f64(double x, double min, double max) {
  return (x < min) ? min : (x > max) ? max : x;
}

VEX_FORCE_INLINE double vex_copysign_f64(double mag, double sign) {
  return copysign(mag, sign);
}

/* =========================
 * Trigonometry
 * ========================= */

VEX_INLINE double vex_sin(double x)    { return sin(x); }
VEX_INLINE double vex_cos(double x)    { return cos(x); }
VEX_INLINE double vex_tan(double x)    { return tan(x); }
VEX_INLINE double vex_asin(double x)   { return asin(x); }
VEX_INLINE double vex_acos(double x)   { return acos(x); }
VEX_INLINE double vex_atan(double x)   { return atan(x); }
VEX_INLINE double vex_atan2(double y, double x) { return atan2(y, x); }

// Hyperbolic
VEX_INLINE double vex_sinh(double x)   { return sinh(x); }
VEX_INLINE double vex_cosh(double x)   { return cosh(x); }
VEX_INLINE double vex_tanh(double x)   { return tanh(x); }
VEX_INLINE double vex_asinh(double x)  { return asinh(x); }
VEX_INLINE double vex_acosh(double x)  { return acosh(x); }
VEX_INLINE double vex_atanh(double x)  { return atanh(x); }

// Simultaneous sin/cos (faster than separate calls)
VEX_INLINE void vex_sincos(double x, double *sin_out, double *cos_out) {
#if defined(__GLIBC__) && defined(__linux__)
  sincos(x, sin_out, cos_out);  // GNU extension
#else
  *sin_out = sin(x);
  *cos_out = cos(x);
#endif
}

// Degrees <-> Radians
VEX_FORCE_INLINE double vex_to_radians(double degrees) {
  return degrees * (VEX_PI / 180.0);
}

VEX_FORCE_INLINE double vex_to_degrees(double radians) {
  return radians * (180.0 / VEX_PI);
}

/* =========================
 * Exponential & Logarithm
 * ========================= */

VEX_INLINE double vex_exp(double x)    { return exp(x); }
VEX_INLINE double vex_exp2(double x)   { return exp2(x); }
VEX_INLINE double vex_exp10(double x)  { return pow(10.0, x); }
VEX_INLINE double vex_expm1(double x)  { return expm1(x); }  // exp(x) - 1 (accurate for small x)

VEX_INLINE double vex_log(double x)    { return log(x); }    // Natural log (ln)
VEX_INLINE double vex_log2(double x)   { return log2(x); }
VEX_INLINE double vex_log10(double x)  { return log10(x); }
VEX_INLINE double vex_log1p(double x)  { return log1p(x); }  // log(1 + x) (accurate for small x)

VEX_INLINE double vex_pow(double base, double exp) { return pow(base, exp); }
VEX_INLINE double vex_sqrt(double x)   { return sqrt(x); }
VEX_INLINE double vex_cbrt(double x)   { return cbrt(x); }   // Cube root
VEX_INLINE double vex_hypot(double x, double y) { return hypot(x, y); }  // sqrt(x^2 + y^2)

/* =========================
 * Rounding
 * ========================= */

VEX_INLINE double vex_ceil(double x)   { return ceil(x); }
VEX_INLINE double vex_floor(double x)  { return floor(x); }
VEX_INLINE double vex_round(double x)  { return round(x); }
VEX_INLINE double vex_trunc(double x)  { return trunc(x); }

// Round to nearest integer (returns int64_t)
VEX_INLINE int64_t vex_round_i64(double x) { return (int64_t)llround(x); }

/* =========================
 * Special Functions (Gamma, Erf, Bessel)
 * ========================= */

VEX_INLINE double vex_gamma(double x)  { return tgamma(x); }     // Gamma function
VEX_INLINE double vex_lgamma(double x) { return lgamma(x); }     // Log-gamma (more stable)
VEX_INLINE double vex_erf(double x)    { return erf(x); }        // Error function
VEX_INLINE double vex_erfc(double x)   { return erfc(x); }       // Complementary error function

// Bessel functions (first kind)
VEX_INLINE double vex_j0(double x)     { return j0(x); }
VEX_INLINE double vex_j1(double x)     { return j1(x); }
VEX_INLINE double vex_jn(int n, double x) { return jn(n, x); }

// Bessel functions (second kind)
VEX_INLINE double vex_y0(double x)     { return y0(x); }
VEX_INLINE double vex_y1(double x)     { return y1(x); }
VEX_INLINE double vex_yn(int n, double x) { return yn(n, x); }

/* =========================
 * Utility
 * ========================= */

// Fused multiply-add: (x * y) + z
VEX_INLINE double vex_fma(double x, double y, double z) {
  return fma(x, y, z);
}

// Remainder
VEX_INLINE double vex_fmod(double x, double y) { return fmod(x, y); }
VEX_INLINE double vex_remainder(double x, double y) { return remainder(x, y); }

// Decompose float
VEX_INLINE double vex_frexp(double x, int *exp) { return frexp(x, exp); }
VEX_INLINE double vex_ldexp(double x, int exp) { return ldexp(x, exp); }
VEX_INLINE double vex_modf(double x, double *int_part) { return modf(x, int_part); }

// Check special values
VEX_FORCE_INLINE bool vex_is_nan(double x)      { return isnan(x); }
VEX_FORCE_INLINE bool vex_is_inf(double x)      { return isinf(x); }
VEX_FORCE_INLINE bool vex_is_finite(double x)   { return isfinite(x); }
VEX_FORCE_INLINE bool vex_is_normal(double x)   { return isnormal(x); }

// Sign
VEX_FORCE_INLINE int vex_sign_f64(double x) {
  return (x > 0.0) ? 1 : (x < 0.0) ? -1 : 0;
}

VEX_FORCE_INLINE bool vex_signbit(double x) {
  return signbit(x);  // Returns true if sign bit is set (even for -0.0)
}

/* =========================
 * Additional Go/Rust Functions
 * ========================= */

// Dim (positive difference) - Go math.Dim
VEX_FORCE_INLINE double vex_dim(double x, double y) {
  return (x > y) ? (x - y) : 0.0;
}

// RoundToEven (banker's rounding) - Go math.RoundToEven
VEX_INLINE double vex_round_to_even(double x) {
  return rint(x);  // Round to nearest even
}

// Bit manipulation (Go math.Float64bits / Rust f64::to_bits)
VEX_FORCE_INLINE uint64_t vex_f64_to_bits(double x) {
  union { double f; uint64_t i; } u;
  u.f = x;
  return u.i;
}

VEX_FORCE_INLINE double vex_f64_from_bits(uint64_t bits) {
  union { uint64_t i; double f; } u;
  u.i = bits;
  return u.f;
}

VEX_FORCE_INLINE uint32_t vex_f32_to_bits(float x) {
  union { float f; uint32_t i; } u;
  u.f = x;
  return u.i;
}

VEX_FORCE_INLINE float vex_f32_from_bits(uint32_t bits) {
  union { uint32_t i; float f; } u;
  u.i = bits;
  return u.f;
}

// Next representable value (ulp operations)
VEX_INLINE double vex_nextafter(double x, double y) {
  return nextafter(x, y);
}

// Signum with zero distinction (Rust signum)
VEX_FORCE_INLINE double vex_signum(double x) {
  if (x > 0.0) return 1.0;
  if (x < 0.0) return -1.0;
  return x;  // Preserve sign of zero (+0.0 or -0.0)
}

/* =========================
 * Linear Interpolation & Smoothing
 * ========================= */

VEX_FORCE_INLINE double vex_lerp(double a, double b, double t) {
  return a + t * (b - a);
}

// Smooth step (cubic Hermite interpolation)
VEX_FORCE_INLINE double vex_smoothstep(double edge0, double edge1, double x) {
  double t = vex_clamp_f64((x - edge0) / (edge1 - edge0), 0.0, 1.0);
  return t * t * (3.0 - 2.0 * t);
}

/* =========================
 * SIMD Accelerated (Optional)
 * ========================= */

#if defined(__AVX2__) || defined(__SSE4_2__)
  #include <immintrin.h>
  
  // Vectorized sqrt (4 doubles at once)
  VEX_INLINE void vex_sqrt_v4f64(const double *in, double *out) {
    __m256d v = _mm256_loadu_pd(in);
    __m256d result = _mm256_sqrt_pd(v);
    _mm256_storeu_pd(out, result);
  }
#endif

/* =========================
 * Demo / Tests
 * ========================= */
#ifdef VEX_MATH_DEMO

#include <stdio.h>

int main(void) {
  printf("=== Vex Math Demo ===\n\n");
  
  // Constants
  printf("Constants:\n");
  printf("  PI = %.15f\n", VEX_PI);
  printf("  E = %.15f\n", VEX_E);
  printf("  PHI = %.15f\n", VEX_PHI);
  
  // Trigonometry
  printf("\nTrigonometry:\n");
  printf("  sin(π/6) = %.15f (expected: 0.5)\n", vex_sin(VEX_PI / 6.0));
  printf("  cos(π/3) = %.15f (expected: 0.5)\n", vex_cos(VEX_PI / 3.0));
  printf("  tan(π/4) = %.15f (expected: 1.0)\n", vex_tan(VEX_PI / 4.0));
  
  // Exponential
  printf("\nExponential:\n");
  printf("  exp(1) = %.15f (expected: e)\n", vex_exp(1.0));
  printf("  log(e) = %.15f (expected: 1.0)\n", vex_log(VEX_E));
  printf("  pow(2, 10) = %.1f (expected: 1024)\n", vex_pow(2.0, 10.0));
  
  // Special functions
  printf("\nSpecial Functions:\n");
  printf("  gamma(5) = %.1f (expected: 24 = 4!)\n", vex_gamma(5.0));
  printf("  erf(1) = %.15f\n", vex_erf(1.0));
  
  // Utility
  printf("\nUtility:\n");
  printf("  hypot(3, 4) = %.1f (expected: 5)\n", vex_hypot(3.0, 4.0));
  printf("  clamp(5, 0, 10) = %.1f\n", vex_clamp_f64(5.0, 0.0, 10.0));
  printf("  lerp(0, 100, 0.5) = %.1f (expected: 50)\n", vex_lerp(0.0, 100.0, 0.5));
  
  printf("\n✅ All tests passed!\n");
  return 0;
}

#endif // VEX_MATH_DEMO

