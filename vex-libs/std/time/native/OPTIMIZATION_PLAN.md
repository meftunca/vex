# vex_time Optimization & Feature Completion Plan

**Current Status**: ⚠️ SIMD not effective, missing Go-layout support

---

## 🐛 Current Problems

### 1. SIMD Performance Issues

| Operation | Expected | Actual | Problem |
|-----------|----------|--------|---------|
| **Parse** | 1.5-2x | **0.94x** ❌ | Slower! |
| **Format** | 1.5-2x | 1.35x ⚠️ | Marginal |

**Root Causes**:
1. `strlen()` overhead at start
2. Store/load operations expensive
3. `timegm()` bottleneck (not SIMD-able)
4. Branch mispredictions

### 2. Missing Features

- ❌ No Go-layout parsing (`January`, `Monday`, etc.)
- ❌ No custom format strings
- ❌ Only RFC3339 supported
- ❌ No strftime-compatible formats

---

## 🎯 Solution Strategy

### Phase 1: Fix SIMD Performance (Priority: HIGH)

#### A. Remove `strlen()` overhead
```c
// Before: ❌
if (strlen(s) < 20) return -1;

// After: ✅
if (s[0] == '\0' || s[19] == '\0') return -1;
```

#### B. Inline hot paths
```c
__attribute__((always_inline))
static inline int fast_parse_digits(const uint8_t* s) {
    // Direct computation without store
    return (s[0] - '0') * 1000 + 
           (s[1] - '0') * 100 +
           (s[2] - '0') * 10 + 
           (s[3] - '0');
}
```

#### C. Use SWAR (SIMD Within A Register)
```c
// Parse 4 digits in one 32-bit operation
uint32_t packed = *(uint32_t*)s;
uint32_t digits = (packed - 0x30303030) & 0x0F0F0F0F;
int year = (digits >> 24) * 1000 + 
           ((digits >> 16) & 0xFF) * 100 +
           ((digits >> 8) & 0xFF) * 10 +
           (digits & 0xFF);
```

#### D. Optimize `timegm()` bottleneck
```c
// Use fast epoch calculation instead of timegm()
// Based on: days since epoch formula
static inline int64_t fast_epoch(int y, int m, int d, int h, int min, int s) {
    // Simplified Gregorian calendar calculation
    int64_t days = (y - 1970) * 365 + leap_days(y);
    days += month_day_offset[m - 1] + d - 1;
    return days * 86400 + h * 3600 + min * 60 + s;
}
```

**Expected Speedup**: 2-3x for RFC3339 parse

---

### Phase 2: Add Go-Layout Support (Priority: HIGH)

#### A. Implement Layout Token System

```c
typedef enum {
    // Year
    TOK_2006,  // 2006 (4-digit year)
    TOK_06,    // 06 (2-digit year)
    
    // Month
    TOK_January, TOK_Jan,  // Full/short name
    TOK_01, TOK_1,         // 2-digit/1-digit
    
    // Day
    TOK_02, TOK_2, TOK__2, // 2-digit/1-digit/space-padded
    TOK_002,               // Day of year
    
    // Weekday
    TOK_Monday, TOK_Mon,
    
    // Hour
    TOK_15,    // 24-hour
    TOK_03, TOK_3, TOK_PM, // 12-hour + AM/PM
    
    // Minute/Second
    TOK_04, TOK_4,         // Minute
    TOK_05, TOK_5,         // Second
    
    // Fraction
    TOK_000, TOK_000000, TOK_000000000,
    
    // Zone
    TOK_MST, TOK_Z07, TOK_Z0700, TOK_Z07_00,
    
    // Literal
    TOK_LITERAL
} LayoutToken;
```

#### B. Layout Parser

```c
typedef struct {
    LayoutToken type;
    const char* literal;
    int len;
} ParsedToken;

// Compile layout once, use many times
ParsedToken* compile_layout(const char* layout);

// Fast path matching
int match_token(const char** input, ParsedToken* token, ParsedValue* out);
```

#### C. Name Lookup Tables (SIMD-friendly)

```c
static const char* month_names[] = {
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
};

// SIMD comparison for month names
int simd_match_month_name(const char* s) {
    // Load 9 bytes (max month name length)
    __m128i input = _mm_loadu_si128((__m128i*)s);
    
    for (int i = 0; i < 12; i++) {
        __m128i month = _mm_loadu_si128((__m128i*)month_names[i]);
        __m128i cmp = _mm_cmpeq_epi8(input, month);
        if (_mm_movemask_epi8(cmp) == 0xFFFF) return i + 1;
    }
    return -1;
}
```

---

### Phase 3: Performance Targets

| Operation | Current | Target | Method |
|-----------|---------|--------|--------|
| RFC3339 Parse | 2700 ns | **800 ns** | SWAR + fast epoch |
| Go-Layout Parse | N/A | **1200 ns** | Token cache + SIMD names |
| RFC3339 Format | 220 ns | **150 ns** | Inline + LUT |
| Go-Layout Format | N/A | **300 ns** | Template cache |

**vs Go/Rust**:
- ✅ RFC3339: Faster than Go (800 vs 1000 ns)
- ✅ Go-Layout: Equal to Go (1200 vs 1200 ns)

---

## 📋 Implementation Checklist

### Quick Wins (Today)
- [ ] Remove `strlen()` calls
- [ ] Inline digit parsing
- [ ] Use SWAR for 4-digit year
- [ ] Fast epoch calculation (no `timegm`)

### Go-Layout Support (This Week)
- [ ] Token enum and structs
- [ ] Layout compiler
- [ ] Month/weekday name tables
- [ ] SIMD name matching
- [ ] Token parser/formatter

### Advanced Optimizations (Future)
- [ ] Layout cache (LRU)
- [ ] PGO-guided optimization
- [ ] Custom allocator for temp buffers
- [ ] SIMD shuffle for formatting

---

## 🔬 Benchmark Strategy

### Test Cases

```c
// RFC3339
"2024-11-07T12:34:56.123456789Z"           // Full precision
"2024-11-07T12:34:56Z"                      // No fraction
"2024-11-07T12:34:56+03:00"                 // With timezone

// Go-Layout  
"Monday, January 02, 2006 3:04:05 PM MST"  // Full names
"2006-01-02 15:04:05"                      // ISO-like
"Jan 2, 2006 at 3:04pm (MST)"              // Casual
```

### Measurement

```c
// Per operation
Median, P50, P99, P999
Min, Max, StdDev

// Comparison
Speedup vs scalar
Speedup vs Go
Speedup vs Rust
```

---

## 🎯 Success Criteria

**Minimum**:
- ✅ RFC3339 parse: < 1000 ns (competitive with Go)
- ✅ Go-layout support: All common tokens
- ✅ No regressions in scalar path

**Target**:
- 🎯 RFC3339 parse: < 800 ns (faster than Go!)
- 🎯 Go-layout parse: < 1500 ns
- 🎯 Format: < 200 ns

**Stretch**:
- 🚀 RFC3339 parse: < 500 ns (2x faster than Go!)
- 🚀 Strftime compatibility
- 🚀 Locale support

---

## 💡 Alternative: Use Specialized Library?

**Option**: Port [simdjson](https://github.com/simdjson/simdjson) techniques

**Pros**:
- Proven SIMD patterns
- Extreme performance
- Well-tested

**Cons**:
- Complex
- Not Vex-specific
- Over-engineering for dates

**Decision**: Implement custom, Vex-optimized solution first.

---

## 📚 References

- [SWAR: SIMD Within A Register](http://0x80.pl/articles/simd-parsing-int-sequences.html)
- [Fast Epoch Calculation](https://howardhinnant.github.io/date_algorithms.html)
- [Go time package source](https://github.com/golang/go/blob/master/src/time/format.go)
- [Rust chrono optimization](https://github.com/chronotope/chrono)

---

**Status**: 🔴 NOT PRODUCTION READY  
**Action**: Implement Phase 1 ASAP!

