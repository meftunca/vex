// vex_range.c - Range and RangeInclusive iterator implementation

#include "vex.h"
#include <stdio.h>
#include <stdlib.h>

/**
 * Create a Range (exclusive end): 0..10
 * @param start Start value (inclusive)
 * @param end End value (exclusive)
 * @return Range structure
 */
VexRange vex_range_new(int64_t start, int64_t end)
{
  VexRange range;
  range.start = start;
  range.end = end;
  range.current = start;
  return range;
}

/**
 * Create a RangeInclusive: 0..=10
 * @param start Start value (inclusive)
 * @param end End value (inclusive)
 * @return RangeInclusive structure
 */
VexRangeInclusive vex_range_inclusive_new(int64_t start, int64_t end)
{
  VexRangeInclusive range;
  range.start = start;
  range.end = end;
  range.current = start;
  return range;
}

/**
 * Get next value from Range iterator
 * @param range Range to iterate
 * @param out_value Output for next value
 * @return true if value available, false if exhausted
 */
bool vex_range_next(VexRange *range, int64_t *out_value)
{
  if (range->current < range->end)
  {
    *out_value = range->current;
    range->current++;
    return true;
  }
  return false;
}

/**
 * Get next value from RangeInclusive iterator
 * @param range RangeInclusive to iterate
 * @param out_value Output for next value
 * @return true if value available, false if exhausted
 */
bool vex_range_inclusive_next(VexRangeInclusive *range, int64_t *out_value)
{
  if (range->current <= range->end)
  {
    *out_value = range->current;
    range->current++;
    return true;
  }
  return false;
}

/**
 * Get length of Range
 * @param range Range to measure
 * @return Number of elements (end - start, clamped to 0)
 */
int64_t vex_range_len(const VexRange *range)
{
  int64_t len = range->end - range->start;
  return len > 0 ? len : 0;
}

/**
 * Get length of RangeInclusive
 * @param range RangeInclusive to measure
 * @return Number of elements (end - start + 1, clamped to 0)
 */
int64_t vex_range_inclusive_len(const VexRangeInclusive *range)
{
  int64_t len = range->end - range->start + 1;
  return len > 0 ? len : 0;
}
