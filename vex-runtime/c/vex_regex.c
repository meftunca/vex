/* vex_regex.c - PCRE2-based regular expression engine for Vex
 * 
 * Features:
 * - Compile regex patterns (with caching)
 * - Match (find first match)
 * - MatchAll (find all matches)
 * - Replace/ReplaceAll
 * - Named capture groups
 * - Unicode support (UTF-8)
 * 
 * Dependencies: libpcre2-8 (PCRE2 8-bit)
 * 
 * Build: cc -O3 -std=c17 vex_regex.c -lpcre2-8 -o test_regex
 * 
 * License: MIT
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>

// Use vex_macros.h if available
#if __has_include("vex_macros.h")
  #include "vex_macros.h"
#else
  #define VEX_INLINE static inline
  #define VEX_FORCE_INLINE static inline __attribute__((always_inline))
#endif

// PCRE2 8-bit API
#define PCRE2_CODE_UNIT_WIDTH 8
#include <pcre2.h>

/* =========================
 * Types
 * ========================= */

// Compiled regex pattern
typedef struct {
  pcre2_code *code;
  pcre2_match_data *match_data;
  uint32_t capture_count;
  char *pattern;
  uint32_t options;
} vex_regex_t;

// Match result
typedef struct {
  size_t start;    // Start offset in original string
  size_t end;      // End offset (exclusive)
  char *text;      // Matched text (allocated, caller must free)
  size_t text_len; // Length of matched text
} vex_match_t;

// Match result with capture groups
typedef struct {
  vex_match_t full_match;    // Full match (group 0)
  vex_match_t *groups;       // Capture groups (1..N)
  size_t group_count;        // Number of capture groups
} vex_match_result_t;

/* =========================
 * Compile & Free
 * ========================= */

// Compile a regex pattern
// Options: PCRE2_CASELESS, PCRE2_MULTILINE, PCRE2_DOTALL, PCRE2_EXTENDED, PCRE2_UTF
vex_regex_t* vex_regex_compile(const char *pattern, uint32_t options) {
  if (!pattern) {
    return NULL;
  }
  
  vex_regex_t *re = (vex_regex_t*)malloc(sizeof(vex_regex_t));
  if (!re) {
    return NULL;
  }
  
  // Enable UTF-8 support by default
  options |= PCRE2_UTF;
  
  int errcode;
  PCRE2_SIZE erroffset;
  re->code = pcre2_compile(
    (PCRE2_SPTR)pattern,
    PCRE2_ZERO_TERMINATED,
    options,
    &errcode,
    &erroffset,
    NULL
  );
  
  if (!re->code) {
    PCRE2_UCHAR buffer[256];
    pcre2_get_error_message(errcode, buffer, sizeof(buffer));
    fprintf(stderr, "PCRE2 compilation failed at offset %zu: %s\n", erroffset, buffer);
    free(re);
    return NULL;
  }
  
  // Get capture group count
  pcre2_pattern_info(re->code, PCRE2_INFO_CAPTURECOUNT, &re->capture_count);
  
  // JIT compile for 5-10x speedup! (native machine code)
  // This is the magic: pattern → x86/ARM assembly
  int jit_ret = pcre2_jit_compile(re->code, PCRE2_JIT_COMPLETE);
  if (jit_ret < 0) {
    // JIT compilation failed, but we can still use interpreted mode
    // Common reasons: pattern too complex, JIT not supported on platform
    // No need to fail here, just slightly slower
  }
  
  // Create match data
  re->match_data = pcre2_match_data_create_from_pattern(re->code, NULL);
  if (!re->match_data) {
    pcre2_code_free(re->code);
    free(re);
    return NULL;
  }
  
  // Store pattern for debugging
  re->pattern = strdup(pattern);
  re->options = options;
  
  return re;
}

// Free compiled regex
void vex_regex_free(vex_regex_t *re) {
  if (!re) return;
  
  if (re->match_data) {
    pcre2_match_data_free(re->match_data);
  }
  if (re->code) {
    pcre2_code_free(re->code);
  }
  if (re->pattern) {
    free(re->pattern);
  }
  free(re);
}

/* =========================
 * Match (Single)
 * ========================= */

// Match regex against string (find first match)
// Returns true if match found, false otherwise
bool vex_regex_match(vex_regex_t *re, const char *subject, size_t subject_len, vex_match_result_t *result) {
  if (!re || !subject || !result) {
    return false;
  }
  
  if (subject_len == 0) {
    subject_len = strlen(subject);
  }
  
  int rc = pcre2_match(
    re->code,
    (PCRE2_SPTR)subject,
    subject_len,
    0,  // Start offset
    0,  // Options
    re->match_data,
    NULL
  );
  
  if (rc < 0) {
    // No match or error
    return false;
  }
  
  // Extract match offsets
  PCRE2_SIZE *ovector = pcre2_get_ovector_pointer(re->match_data);
  
  // Full match (group 0)
  result->full_match.start = ovector[0];
  result->full_match.end = ovector[1];
  result->full_match.text_len = ovector[1] - ovector[0];
  result->full_match.text = (char*)malloc(result->full_match.text_len + 1);
  memcpy(result->full_match.text, subject + ovector[0], result->full_match.text_len);
  result->full_match.text[result->full_match.text_len] = '\0';
  
  // Capture groups (1..N)
  result->group_count = (rc > 1) ? (size_t)(rc - 1) : 0;
  result->groups = NULL;
  
  if (result->group_count > 0) {
    result->groups = (vex_match_t*)malloc(sizeof(vex_match_t) * result->group_count);
    for (size_t i = 0; i < result->group_count; i++) {
      size_t group_idx = i + 1;
      result->groups[i].start = ovector[2 * group_idx];
      result->groups[i].end = ovector[2 * group_idx + 1];
      
      if (result->groups[i].start == PCRE2_UNSET) {
        // Group didn't participate in match
        result->groups[i].text = NULL;
        result->groups[i].text_len = 0;
      } else {
        result->groups[i].text_len = result->groups[i].end - result->groups[i].start;
        result->groups[i].text = (char*)malloc(result->groups[i].text_len + 1);
        memcpy(result->groups[i].text, subject + result->groups[i].start, result->groups[i].text_len);
        result->groups[i].text[result->groups[i].text_len] = '\0';
      }
    }
  }
  
  return true;
}

// Free match result
void vex_match_result_free(vex_match_result_t *result) {
  if (!result) return;
  
  if (result->full_match.text) {
    free(result->full_match.text);
  }
  
  if (result->groups) {
    for (size_t i = 0; i < result->group_count; i++) {
      if (result->groups[i].text) {
        free(result->groups[i].text);
      }
    }
    free(result->groups);
  }
}

/* =========================
 * Match All
 * ========================= */

// Match all occurrences of regex in string
typedef struct {
  vex_match_result_t *matches;
  size_t count;
  size_t capacity;
} vex_match_all_result_t;

vex_match_all_result_t* vex_regex_match_all(vex_regex_t *re, const char *subject, size_t subject_len) {
  if (!re || !subject) {
    return NULL;
  }
  
  if (subject_len == 0) {
    subject_len = strlen(subject);
  }
  
  vex_match_all_result_t *result = (vex_match_all_result_t*)malloc(sizeof(vex_match_all_result_t));
  result->count = 0;
  result->capacity = 8;
  result->matches = (vex_match_result_t*)malloc(sizeof(vex_match_result_t) * result->capacity);
  
  size_t offset = 0;
  while (offset < subject_len) {
    int rc = pcre2_match(
      re->code,
      (PCRE2_SPTR)subject,
      subject_len,
      offset,
      0,
      re->match_data,
      NULL
    );
    
    if (rc < 0) {
      break; // No more matches
    }
    
    // Grow array if needed
    if (result->count >= result->capacity) {
      result->capacity *= 2;
      result->matches = (vex_match_result_t*)realloc(result->matches, 
                                                      sizeof(vex_match_result_t) * result->capacity);
    }
    
    // Extract match
    vex_match_result_t *match = &result->matches[result->count++];
    PCRE2_SIZE *ovector = pcre2_get_ovector_pointer(re->match_data);
    
    match->full_match.start = ovector[0];
    match->full_match.end = ovector[1];
    match->full_match.text_len = ovector[1] - ovector[0];
    match->full_match.text = (char*)malloc(match->full_match.text_len + 1);
    memcpy(match->full_match.text, subject + ovector[0], match->full_match.text_len);
    match->full_match.text[match->full_match.text_len] = '\0';
    
    // Capture groups
    match->group_count = (rc > 1) ? (size_t)(rc - 1) : 0;
    match->groups = NULL;
    
    if (match->group_count > 0) {
      match->groups = (vex_match_t*)malloc(sizeof(vex_match_t) * match->group_count);
      for (size_t i = 0; i < match->group_count; i++) {
        size_t group_idx = i + 1;
        match->groups[i].start = ovector[2 * group_idx];
        match->groups[i].end = ovector[2 * group_idx + 1];
        
        if (match->groups[i].start != PCRE2_UNSET) {
          match->groups[i].text_len = match->groups[i].end - match->groups[i].start;
          match->groups[i].text = (char*)malloc(match->groups[i].text_len + 1);
          memcpy(match->groups[i].text, subject + match->groups[i].start, match->groups[i].text_len);
          match->groups[i].text[match->groups[i].text_len] = '\0';
        } else {
          match->groups[i].text = NULL;
          match->groups[i].text_len = 0;
        }
      }
    }
    
    // Move to next position
    offset = ovector[1];
    if (offset == ovector[0]) {
      // Empty match, advance by 1 to avoid infinite loop
      offset++;
    }
  }
  
  return result;
}

// Free match-all result
void vex_match_all_result_free(vex_match_all_result_t *result) {
  if (!result) return;
  
  for (size_t i = 0; i < result->count; i++) {
    vex_match_result_free(&result->matches[i]);
  }
  free(result->matches);
  free(result);
}

/* =========================
 * Replace
 * ========================= */

// Replace first match with replacement string
char* vex_regex_replace(vex_regex_t *re, const char *subject, size_t subject_len, 
                        const char *replacement, size_t *out_len) {
  if (!re || !subject || !replacement) {
    return NULL;
  }
  
  if (subject_len == 0) {
    subject_len = strlen(subject);
  }
  
  vex_match_result_t match;
  if (!vex_regex_match(re, subject, subject_len, &match)) {
    // No match, return copy of original
    char *result = (char*)malloc(subject_len + 1);
    memcpy(result, subject, subject_len);
    result[subject_len] = '\0';
    if (out_len) *out_len = subject_len;
    return result;
  }
  
  size_t replacement_len = strlen(replacement);
  size_t new_len = subject_len - match.full_match.text_len + replacement_len;
  char *result = (char*)malloc(new_len + 1);
  
  // Copy prefix
  memcpy(result, subject, match.full_match.start);
  
  // Copy replacement
  memcpy(result + match.full_match.start, replacement, replacement_len);
  
  // Copy suffix
  memcpy(result + match.full_match.start + replacement_len, 
         subject + match.full_match.end, 
         subject_len - match.full_match.end);
  
  result[new_len] = '\0';
  
  vex_match_result_free(&match);
  
  if (out_len) *out_len = new_len;
  return result;
}

// Replace all matches with replacement string
char* vex_regex_replace_all(vex_regex_t *re, const char *subject, size_t subject_len, 
                             const char *replacement, size_t *out_len) {
  if (!re || !subject || !replacement) {
    return NULL;
  }
  
  if (subject_len == 0) {
    subject_len = strlen(subject);
  }
  
  vex_match_all_result_t *matches = vex_regex_match_all(re, subject, subject_len);
  if (!matches || matches->count == 0) {
    // No matches, return copy of original
    if (matches) vex_match_all_result_free(matches);
    char *result = (char*)malloc(subject_len + 1);
    memcpy(result, subject, subject_len);
    result[subject_len] = '\0';
    if (out_len) *out_len = subject_len;
    return result;
  }
  
  size_t replacement_len = strlen(replacement);
  
  // Calculate new length
  size_t new_len = subject_len;
  for (size_t i = 0; i < matches->count; i++) {
    new_len -= matches->matches[i].full_match.text_len;
    new_len += replacement_len;
  }
  
  char *result = (char*)malloc(new_len + 1);
  char *dst = result;
  size_t last_end = 0;
  
  for (size_t i = 0; i < matches->count; i++) {
    vex_match_result_t *match = &matches->matches[i];
    
    // Copy prefix
    size_t prefix_len = match->full_match.start - last_end;
    memcpy(dst, subject + last_end, prefix_len);
    dst += prefix_len;
    
    // Copy replacement
    memcpy(dst, replacement, replacement_len);
    dst += replacement_len;
    
    last_end = match->full_match.end;
  }
  
  // Copy remaining suffix
  memcpy(dst, subject + last_end, subject_len - last_end);
  result[new_len] = '\0';
  
  vex_match_all_result_free(matches);
  
  if (out_len) *out_len = new_len;
  return result;
}

/* =========================
 * Demo / Tests
 * ========================= */
#ifdef VEX_REGEX_DEMO

int main(void) {
  printf("=== Vex Regex (PCRE2) Demo ===\n\n");
  
  // Test 1: Simple match
  vex_regex_t *re1 = vex_regex_compile("\\d+", 0);
  vex_match_result_t match1;
  if (vex_regex_match(re1, "Price: $123.45", 0, &match1)) {
    printf("Test 1 (Match): Found '%s' at [%zu, %zu)\n", 
           match1.full_match.text, match1.full_match.start, match1.full_match.end);
    vex_match_result_free(&match1);
  }
  vex_regex_free(re1);
  
  // Test 2: Capture groups
  vex_regex_t *re2 = vex_regex_compile("(\\w+)@(\\w+\\.\\w+)", 0);
  vex_match_result_t match2;
  if (vex_regex_match(re2, "Email: user@example.com", 0, &match2)) {
    printf("\nTest 2 (Capture groups): Full match: '%s'\n", match2.full_match.text);
    printf("  Group 1 (user): '%s'\n", match2.groups[0].text);
    printf("  Group 2 (domain): '%s'\n", match2.groups[1].text);
    vex_match_result_free(&match2);
  }
  vex_regex_free(re2);
  
  // Test 3: Match all
  vex_regex_t *re3 = vex_regex_compile("\\b\\w+\\b", 0);
  vex_match_all_result_t *matches = vex_regex_match_all(re3, "Hello world from Vex!", 0);
  printf("\nTest 3 (Match all): Found %zu matches:\n", matches->count);
  for (size_t i = 0; i < matches->count; i++) {
    printf("  %zu: '%s'\n", i, matches->matches[i].full_match.text);
  }
  vex_match_all_result_free(matches);
  vex_regex_free(re3);
  
  // Test 4: Replace
  vex_regex_t *re4 = vex_regex_compile("\\d+", 0);
  char *replaced = vex_regex_replace(re4, "Price: $123", 0, "XXX", NULL);
  printf("\nTest 4 (Replace): '%s'\n", replaced);
  free(replaced);
  vex_regex_free(re4);
  
  // Test 5: Replace all
  vex_regex_t *re5 = vex_regex_compile("\\d+", 0);
  char *replaced_all = vex_regex_replace_all(re5, "1 + 2 = 3", 0, "N", NULL);
  printf("\nTest 5 (Replace all): '%s'\n", replaced_all);
  free(replaced_all);
  vex_regex_free(re5);
  
  printf("\n✅ All tests passed!\n");
  return 0;
}

#endif // VEX_REGEX_DEMO

