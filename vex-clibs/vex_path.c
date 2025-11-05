// vex_path.c - Path manipulation and glob operations
#include "vex.h"
#include <string.h>
#include <stdlib.h>
#include <limits.h>
#include <unistd.h>
#include <sys/stat.h>
#include <dirent.h>
#include <libgen.h>
#include <stdio.h>

// ============================================================================
// PATH MANIPULATION
// ============================================================================

char *vex_path_join(const char *path1, const char *path2)
{
  if (!path1 || !path2)
  {
    vex_panic("vex_path_join: NULL path");
  }

  size_t len1 = vex_strlen(path1);
  size_t len2 = vex_strlen(path2);

  // Remove trailing slash from path1
  while (len1 > 0 && path1[len1 - 1] == '/')
    len1--;

  // Remove leading slash from path2
  while (len2 > 0 && path2[0] == '/')
  {
    path2++;
    len2--;
  }

  // Allocate: path1 + '/' + path2 + '\0'
  char *result = (char *)vex_malloc(len1 + 1 + len2 + 1);
  vex_memcpy(result, path1, len1);
  result[len1] = '/';
  vex_memcpy(result + len1 + 1, path2, len2);
  result[len1 + 1 + len2] = '\0';

  return result;
}

char *vex_path_dirname(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_dirname: NULL path");
  }

  char *path_copy = vex_strdup(path);
  char *dir = dirname(path_copy);
  char *result = vex_strdup(dir);
  vex_free(path_copy);

  return result;
}

char *vex_path_basename(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_basename: NULL path");
  }

  char *path_copy = vex_strdup(path);
  char *base = basename(path_copy);
  char *result = vex_strdup(base);
  vex_free(path_copy);

  return result;
}

char *vex_path_extension(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_extension: NULL path");
  }

  const char *dot = strrchr(path, '.');
  const char *slash = strrchr(path, '/');

  // Dot must come after last slash (if any) and not be first char
  if (dot && (!slash || dot > slash) && dot != path)
  {
    return vex_strdup(dot); // Includes the dot
  }

  return vex_strdup(""); // No extension
}

char *vex_path_absolute(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_absolute: NULL path");
  }

  char resolved[PATH_MAX];
  if (realpath(path, resolved) == NULL)
  {
    return NULL; // Failed to resolve
  }

  return vex_strdup(resolved);
}

bool vex_path_is_absolute(const char *path)
{
  if (!path || path[0] == '\0')
    return false;
  return path[0] == '/';
}

bool vex_path_is_dir(const char *path)
{
  if (!path)
    return false;

  struct stat st;
  if (stat(path, &st) != 0)
    return false;
  return S_ISDIR(st.st_mode);
}

bool vex_path_is_file(const char *path)
{
  if (!path)
    return false;

  struct stat st;
  if (stat(path, &st) != 0)
    return false;
  return S_ISREG(st.st_mode);
}

// ============================================================================
// GLOB/PATTERN MATCHING
// ============================================================================

// Simple glob matching (*, ?, [...])
static bool match_pattern(const char *pattern, const char *str)
{
  while (*pattern && *str)
  {
    if (*pattern == '*')
    {
      // Skip consecutive *
      while (*pattern == '*')
        pattern++;
      if (!*pattern)
        return true; // * at end matches everything

      // Try matching rest of pattern at each position
      while (*str)
      {
        if (match_pattern(pattern, str))
          return true;
        str++;
      }
      return false;
    }
    else if (*pattern == '?')
    {
      // ? matches any single character
      pattern++;
      str++;
    }
    else if (*pattern == '[')
    {
      // Character class [abc] or [a-z]
      pattern++;
      bool negate = false;
      if (*pattern == '!')
      {
        negate = true;
        pattern++;
      }

      bool matched = false;
      while (*pattern && *pattern != ']')
      {
        if (pattern[1] == '-' && pattern[2] != ']')
        {
          // Range: a-z
          if (*str >= pattern[0] && *str <= pattern[2])
          {
            matched = true;
          }
          pattern += 3;
        }
        else
        {
          // Single char
          if (*str == *pattern)
          {
            matched = true;
          }
          pattern++;
        }
      }

      if (*pattern == ']')
        pattern++;
      if (negate)
        matched = !matched;
      if (!matched)
        return false;
      str++;
    }
    else
    {
      // Literal character
      if (*pattern != *str)
        return false;
      pattern++;
      str++;
    }
  }

  // Skip trailing *
  while (*pattern == '*')
    pattern++;

  return !*pattern && !*str;
}

VexArray *vex_path_glob(const char *pattern)
{
  if (!pattern)
  {
    vex_panic("vex_path_glob: NULL pattern");
  }

  // Simple implementation: glob in current directory
  // For full implementation, should handle path separators and recursive **

  VexArray *results = NULL; // Will be created by vex_array_append

  DIR *dir = opendir(".");
  if (!dir)
    return results;

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL)
  {
    if (entry->d_name[0] == '.')
      continue; // Skip hidden files

    if (match_pattern(pattern, entry->d_name))
    {
      char *matched_path = vex_strdup(entry->d_name);
      results = vex_array_append(results, &matched_path, sizeof(char *));
    }
  }

  closedir(dir);
  return results;
}

VexArray *vex_path_glob_recursive(const char *dir_path, const char *pattern)
{
  if (!dir_path || !pattern)
  {
    vex_panic("vex_path_glob_recursive: NULL parameter");
  }

  VexArray *results = NULL;

  DIR *dir = opendir(dir_path);
  if (!dir)
    return results;

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL)
  {
    if (entry->d_name[0] == '.')
      continue;

    char *full_path = vex_path_join(dir_path, entry->d_name);

    if (entry->d_type == DT_DIR)
    {
      // Recursively search subdirectories
      VexArray *sub_results = vex_path_glob_recursive(full_path, pattern);

      // Merge results
      if (sub_results)
      {
        size_t sub_len = vex_array_len(sub_results);
        for (size_t i = 0; i < sub_len; i++)
        {
          char **path_ptr = (char **)vex_array_get(sub_results, i, sizeof(char *));
          char *path_copy = vex_strdup(*path_ptr);
          results = vex_array_append(results, &path_copy, sizeof(char *));
        }
      }
    }
    else if (entry->d_type == DT_REG)
    {
      // Regular file - check pattern
      if (match_pattern(pattern, entry->d_name))
      {
        char *path_copy = vex_strdup(full_path);
        results = vex_array_append(results, &path_copy, sizeof(char *));
      }
    }

    vex_free(full_path);
  }

  closedir(dir);
  return results;
}

// ============================================================================
// DIRECTORY WALKING
// ============================================================================

typedef struct
{
  const char *path;
  bool is_dir;
  size_t size;
} VexDirEntry;

VexArray *vex_path_list_dir(const char *dir_path)
{
  if (!dir_path)
  {
    vex_panic("vex_path_list_dir: NULL path");
  }

  VexArray *entries = NULL;

  DIR *dir = opendir(dir_path);
  if (!dir)
    return entries;

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL)
  {
    if (entry->d_name[0] == '.')
      continue;

    char *full_path = vex_path_join(dir_path, entry->d_name);

    VexDirEntry *dir_entry = (VexDirEntry *)vex_malloc(sizeof(VexDirEntry));
    dir_entry->path = full_path;
    dir_entry->is_dir = (entry->d_type == DT_DIR);

    // Get size for files
    if (!dir_entry->is_dir)
    {
      struct stat st;
      if (stat(full_path, &st) == 0)
      {
        dir_entry->size = (size_t)st.st_size;
      }
      else
      {
        dir_entry->size = 0;
      }
    }
    else
    {
      dir_entry->size = 0;
    }

    entries = vex_array_append(entries, &dir_entry, sizeof(VexDirEntry *));
  }

  closedir(dir);
  return entries;
}

// ============================================================================
// FILE COPY/MOVE
// ============================================================================

bool vex_file_copy(const char *src, const char *dst)
{
  if (!src || !dst)
  {
    vex_panic("vex_file_copy: NULL path");
  }

  // Read source
  size_t src_size;
  char *data = vex_file_read_all(src, &src_size);
  if (!data)
    return false;

  // Write destination
  bool success = vex_file_write_all(dst, data, src_size);
  vex_free(data);

  return success;
}

bool vex_file_move(const char *src, const char *dst)
{
  if (!src || !dst)
  {
    vex_panic("vex_file_move: NULL path");
  }

  // Try rename first (fast if on same filesystem)
  if (vex_file_rename(src, dst))
  {
    return true;
  }

  // Fallback: copy + delete
  if (vex_file_copy(src, dst))
  {
    return vex_file_remove(src);
  }

  return false;
}

// ============================================================================
// TEMP FILE/DIR
// ============================================================================

char *vex_path_temp_file(const char *prefix)
{
  const char *tmpdir = getenv("TMPDIR");
  if (!tmpdir)
    tmpdir = "/tmp";

  const char *use_prefix = prefix ? prefix : "vex";

  char template[PATH_MAX];
  snprintf(template, sizeof(template), "%s/%s_XXXXXX", tmpdir, use_prefix);

  int fd = mkstemp(template);
  if (fd < 0)
    return NULL;

  close(fd);
  return vex_strdup(template);
}

char *vex_path_temp_dir(const char *prefix)
{
  const char *tmpdir = getenv("TMPDIR");
  if (!tmpdir)
    tmpdir = "/tmp";

  const char *use_prefix = prefix ? prefix : "vex";

  char template[PATH_MAX];
  snprintf(template, sizeof(template), "%s/%s_XXXXXX", tmpdir, use_prefix);

  if (mkdtemp(template) == NULL)
    return NULL;

  return vex_strdup(template);
}
