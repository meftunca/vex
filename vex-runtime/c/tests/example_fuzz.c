/* example_fuzz.c - Fuzzing example for vex_testing.c
 * Demonstrates libFuzzer integration for testing C code.
 *
 * Build with libFuzzer:
 *   clang -O2 -g -fsanitize=fuzzer,address -DVEX_FUZZ_TARGET \
 *     example_fuzz.c -o example_fuzz
 *
 * Run fuzzer:
 *   ./example_fuzz -max_total_time=30  # Run for 30 seconds
 *   ./example_fuzz corpus/  # Use corpus directory
 *
 * Build with AFL++:
 *   afl-clang-fast -O2 -g -DVEX_FUZZ_TARGET example_fuzz.c -o example_fuzz_afl
 *   afl-fuzz -i corpus/ -o findings/ -- ./example_fuzz_afl
 */

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

// Example: Simple parser (intentionally has bugs for fuzzing demo)
typedef struct
{
  int type; // 0 = int, 1 = string
  union
  {
    int64_t i;
    char *s;
  } value;
} Token;

static Token parse_token(const char *input, size_t len)
{
  Token tok = {0};

  if (len == 0)
  {
    return tok;
  }

  // Try to parse as integer
  if (input[0] >= '0' && input[0] <= '9')
  {
    tok.type = 0;
    tok.value.i = atoll(input);
    return tok;
  }

  // Parse as string
  tok.type = 1;
  tok.value.s = (char *)vex_malloc(len + 1);
  memcpy(tok.value.s, input, len);
  tok.value.s[len] = '\0';

  return tok;
}

static void free_token(Token *tok)
{
  if (tok->type == 1 && tok->value.s)
  {
    vex_free(tok->value.s);
    tok->value.s = NULL;
  }
}

// Fuzz target: Test parser with random input
#ifdef VEX_FUZZ_TARGET

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
  // Skip invalid sizes
  if (size < 1 || size > 1024)
  {
    return 0;
  }

  // Create null-terminated string
  char *input = (char *)vex_malloc(size + 1);
  if (!input)
  {
    return 0;
  }
  memcpy(input, data, size);
  input[size] = '\0';

  // Parse token
  Token tok = parse_token(input, size);

  // Validate result
  if (tok.type == 0)
  {
    // Integer: check range
    if (tok.value.i < -1000000 || tok.value.i > 1000000)
    {
      // Potential overflow?
    }
  }
  else if (tok.type == 1)
  {
    // String: check length
    if (tok.value.s && strlen(tok.value.s) != size)
    {
      // Length mismatch?
    }
  }

  // Cleanup
  free_token(&tok);
  vex_free(input);

  return 0;
}

#else

// Standalone mode: Run with predefined inputs
int main(int argc, char **argv)
{
  if (argc < 2)
  {
    fprintf(stderr, "Usage: %s <input_file>\n", argv[0]);
    return 1;
  }

  FILE *f = fopen(argv[1], "rb");
  if (!f)
  {
    perror("fopen");
    return 1;
  }

  fseek(f, 0, SEEK_END);
  long fsize = ftell(f);
  fseek(f, 0, SEEK_SET);

  uint8_t *data = (uint8_t *)vex_malloc(fsize);
  fread(data, 1, fsize, f);
  fclose(f);

  // Run fuzz test
  Token tok = parse_token((const char *)data, (size_t)fsize);

  printf("Parsed token: type=%d\n", tok.type);
  if (tok.type == 0)
  {
    printf("  value.i = %ld\n", tok.value.i);
  }
  else if (tok.type == 1)
  {
    printf("  value.s = '%s'\n", tok.value.s);
  }

  free_token(&tok);
  vex_free(data);

  return 0;
}

#endif
