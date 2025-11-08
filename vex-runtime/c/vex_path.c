// vex_path.c - Cross-platform path manipulation and file system operations
// Supports: Unix/Linux/macOS/Windows
#include "vex.h"
#include <string.h>
#include <stdlib.h>
#include <limits.h>
#include <sys/stat.h>
#include <stdio.h>
#include <errno.h>
#include <ctype.h>

#ifdef _WIN32
#include <windows.h>
#include <direct.h>
#include <io.h>
#define PATH_SEP '\\'
#define PATH_SEP_STR "\\"
#define ALT_PATH_SEP '/'
#define mkdir(path, mode) _mkdir(path)
#define rmdir(path) _rmdir(path)
#define stat _stat
#define S_ISDIR(m) (((m) & S_IFMT) == S_IFDIR)
#define S_ISREG(m) (((m) & S_IFMT) == S_IFREG)
#else
#include <unistd.h>
#include <dirent.h>
#include <libgen.h>
#define PATH_SEP '/'
#define PATH_SEP_STR "/"
#define ALT_PATH_SEP '\0'
#endif

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

// ============================================================================
// PLATFORM UTILITIES
// ============================================================================

static inline bool is_path_separator(char c)
{
#ifdef _WIN32
  return c == '/' || c == '\\';
#else
  return c == '/';
#endif
}

static inline char get_path_separator(void)
{
  return PATH_SEP;
}

const char *vex_path_separator(void)
{
  return PATH_SEP_STR;
}

// Normalize separators to platform-native
static char *normalize_separators(char *path)
{
  if (!path)
    return NULL;

#ifdef _WIN32
  // Convert / to \ on Windows
  for (char *p = path; *p; p++)
  {
    if (*p == '/')
      *p = '\\';
  }
#endif

  return path;
}

// ============================================================================
// PATH NORMALIZATION
// ============================================================================

char *vex_path_normalize(const char *path)
{
  if (!path || !*path)
  {
    return vex_strdup(".");
  }

  size_t len = vex_strlen(path);
  char *result = (char *)vex_malloc(len + 2); // +2 for safety
  char *out = result;
  const char *p = path;

  bool is_absolute = false;

#ifdef _WIN32
  // Handle Windows absolute paths: C:\ or \\server\share
  if (len >= 2 && isalpha(path[0]) && path[1] == ':')
  {
    *out++ = *p++;
    *out++ = *p++;
    is_absolute = true;
  }
  else if (len >= 2 && is_path_separator(path[0]) && is_path_separator(path[1]))
  {
    // UNC path
    *out++ = PATH_SEP;
    *out++ = PATH_SEP;
    p += 2;
    is_absolute = true;
  }
  else if (is_path_separator(path[0]))
  {
    *out++ = PATH_SEP;
    p++;
    is_absolute = true;
  }
#else
  if (path[0] == '/')
  {
    *out++ = '/';
    p++;
    is_absolute = true;
  }
#endif

  // Stack for path components
  const char *components[256];
  int component_count = 0;

  // Parse components
  while (*p)
  {
    // Skip separators
    while (*p && is_path_separator(*p))
      p++;

    if (!*p)
      break;

    const char *component_start = p;
    while (*p && !is_path_separator(*p))
      p++;

    size_t component_len = p - component_start;

    if (component_len == 1 && component_start[0] == '.')
    {
      // Skip "."
      continue;
    }
    else if (component_len == 2 && component_start[0] == '.' && component_start[1] == '.')
    {
      // ".." - pop previous component if not at root
      if (component_count > 0 && !(components[component_count - 1][0] == '.' && components[component_count - 1][1] == '.'))
      {
        component_count--;
      }
      else if (!is_absolute)
      {
        // Keep ".." for relative paths
        components[component_count++] = component_start;
      }
    }
    else
    {
      // Normal component
      components[component_count++] = component_start;
    }
  }

  // Build result
  if (component_count == 0)
  {
    if (!is_absolute)
    {
      *out++ = '.';
    }
  }
  else
  {
    for (int i = 0; i < component_count; i++)
    {
      if (i > 0 || (is_absolute && out > result))
      {
        *out++ = PATH_SEP;
      }

      // Find component length
      const char *comp = components[i];
      const char *comp_end = comp;
      while (*comp_end && !is_path_separator(*comp_end))
        comp_end++;

      size_t comp_len = comp_end - comp;
      vex_memcpy(out, comp, comp_len);
      out += comp_len;
    }
  }

  *out = '\0';
  return result;
}

char *vex_path_clean(const char *path)
{
  // Alias for normalize
  return vex_path_normalize(path);
}

// ============================================================================
// PATH VALIDATION & SANITIZATION
// ============================================================================

bool vex_path_is_valid(const char *path)
{
  if (!path || !*path)
    return false;

  // Check for null bytes
  for (const char *p = path; *p; p++)
  {
    if (*p == '\0')
      return false;
  }

  // Check for invalid characters
#ifdef _WIN32
  // Windows invalid chars: < > : " | ? *
  const char *invalid = "<>:\"|?*";
  for (const char *p = path; *p; p++)
  {
    for (const char *inv = invalid; *inv; inv++)
    {
      if (*p == *inv && *p != ':') // Allow : for drive letter
        return false;
    }
  }
#endif

  return true;
}

char *vex_path_sanitize(const char *path)
{
  if (!path)
    return NULL;

  size_t len = vex_strlen(path);
  char *result = (char *)vex_malloc(len + 1);
  char *out = result;

  for (const char *p = path; *p; p++)
  {
    char c = *p;

#ifdef _WIN32
    // Replace invalid Windows chars with _
    if (c == '<' || c == '>' || c == '"' || c == '|' || c == '?' || c == '*')
    {
      *out++ = '_';
    }
    else if (c == ':' && p != path + 1) // Allow drive letter colon
    {
      *out++ = '_';
    }
    else
    {
      *out++ = c;
    }
#else
    // Unix: only null byte is truly invalid
    if (c == '\0')
      break;
    *out++ = c;
#endif
  }

  *out = '\0';
  return result;
}

// ============================================================================
// PATH MANIPULATION
// ============================================================================

char *vex_path_join(const char *path1, const char *path2)
{
  if (!path1 || !path2)
  {
    vex_panic("vex_path_join: NULL path");
  }

  // If path2 is absolute, return it
  if (vex_path_is_absolute(path2))
  {
    return vex_strdup(path2);
  }

  size_t len1 = vex_strlen(path1);
  size_t len2 = vex_strlen(path2);

  // Remove trailing separators from path1
  while (len1 > 0 && is_path_separator(path1[len1 - 1]))
    len1--;

  // Remove leading separators from path2
  while (len2 > 0 && is_path_separator(path2[0]))
  {
    path2++;
    len2--;
  }

  // Allocate: path1 + separator + path2 + '\0'
  char *result = (char *)vex_malloc(len1 + 1 + len2 + 1);
  vex_memcpy(result, path1, len1);
  result[len1] = PATH_SEP;
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

#ifdef _WIN32
  char *path_copy = vex_strdup(path);
  size_t len = vex_strlen(path_copy);

  // Find last separator
  char *last_sep = NULL;
  for (size_t i = 0; i < len; i++)
  {
    if (is_path_separator(path_copy[i]))
      last_sep = &path_copy[i];
  }

  if (!last_sep)
  {
    vex_free(path_copy);
    return vex_strdup(".");
  }

  // Handle root: C:\ or \
  if (last_sep == path_copy || (len >= 2 && path_copy[1] == ':' && last_sep == &path_copy[2]))
  {
    last_sep[1] = '\0';
    return path_copy;
  }

  *last_sep = '\0';
  return path_copy;
#else
  char *path_copy = vex_strdup(path);
  char *dir = dirname(path_copy);
  char *result = vex_strdup(dir);
  vex_free(path_copy);
  return result;
#endif
}

char *vex_path_basename(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_basename: NULL path");
  }

#ifdef _WIN32
  const char *last_sep = NULL;
  for (const char *p = path; *p; p++)
  {
    if (is_path_separator(*p))
      last_sep = p;
  }

  if (!last_sep)
    return vex_strdup(path);

  return vex_strdup(last_sep + 1);
#else
  char *path_copy = vex_strdup(path);
  char *base = basename(path_copy);
  char *result = vex_strdup(base);
  vex_free(path_copy);
  return result;
#endif
}

char *vex_path_extension(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_extension: NULL path");
  }

  const char *dot = NULL;
  const char *sep = NULL;

  for (const char *p = path; *p; p++)
  {
    if (*p == '.')
      dot = p;
    if (is_path_separator(*p))
    {
      sep = p;
      dot = NULL; // Reset dot after separator
    }
  }

  // Dot must exist, not be first char, and come after last separator
  if (dot && dot != path && (!sep || dot > sep))
  {
    return vex_strdup(dot); // Includes the dot
  }

  return vex_strdup(""); // No extension
}

char *vex_path_stem(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_stem: NULL path");
  }

  char *base = vex_path_basename(path);
  char *dot = strrchr(base, '.');

  if (dot && dot != base)
  {
    *dot = '\0';
  }

  return base;
}

char *vex_path_absolute(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_absolute: NULL path");
  }

#ifdef _WIN32
  char resolved[PATH_MAX];
  if (_fullpath(resolved, path, PATH_MAX) == NULL)
  {
    return NULL;
  }
  return vex_strdup(resolved);
#else
  char resolved[PATH_MAX];
  if (realpath(path, resolved) == NULL)
  {
    return NULL;
  }
  return vex_strdup(resolved);
#endif
}

bool vex_path_is_absolute(const char *path)
{
  if (!path || path[0] == '\0')
    return false;

#ifdef _WIN32
  // C:\ or \\server\share
  if (vex_strlen(path) >= 2)
  {
    if (isalpha(path[0]) && path[1] == ':')
      return true;
    if (is_path_separator(path[0]) && is_path_separator(path[1]))
      return true;
  }
  return is_path_separator(path[0]);
#else
  return path[0] == '/';
#endif
}

// ============================================================================
// PATH COMPONENTS
// ============================================================================

VexArray *vex_path_components(const char *path)
{
  if (!path)
  {
    vex_panic("vex_path_components: NULL path");
  }

  VexArray *components = NULL;
  char *path_copy = vex_strdup(path);
  char *p = path_copy;

#ifdef _WIN32
  // Handle drive letter
  if (vex_strlen(path) >= 2 && isalpha(path[0]) && path[1] == ':')
  {
    char drive[3] = {path[0], ':', '\0'};
    char *drive_copy = vex_strdup(drive);
    components = vex_array_append(components, &drive_copy, sizeof(char *));
    p += 2;
  }
#endif

  // Skip leading separators
  while (*p && is_path_separator(*p))
    p++;

  // Split by separators
  char *component_start = p;
  while (*p)
  {
    if (is_path_separator(*p))
    {
      *p = '\0';
      if (*component_start)
      {
        char *comp = vex_strdup(component_start);
        components = vex_array_append(components, &comp, sizeof(char *));
      }
      p++;
      while (*p && is_path_separator(*p))
        p++;
      component_start = p;
    }
    else
    {
      p++;
    }
  }

  // Last component
  if (*component_start)
  {
    char *comp = vex_strdup(component_start);
    components = vex_array_append(components, &comp, sizeof(char *));
  }

  vex_free(path_copy);
  return components;
}

char *vex_path_parent(const char *path)
{
  // Alias for dirname (but more explicit name)
  return vex_path_dirname(path);
}

// ============================================================================
// PATH COMPARISON
// ============================================================================

bool vex_path_equals(const char *path1, const char *path2)
{
  if (!path1 || !path2)
    return false;

  char *norm1 = vex_path_normalize(path1);
  char *norm2 = vex_path_normalize(path2);

  bool result = (vex_strcmp(norm1, norm2) == 0);

  vex_free(norm1);
  vex_free(norm2);
  return result;
}

bool vex_path_starts_with(const char *path, const char *prefix)
{
  if (!path || !prefix)
    return false;

  char *norm_path = vex_path_normalize(path);
  char *norm_prefix = vex_path_normalize(prefix);

  size_t path_len = vex_strlen(norm_path);
  size_t prefix_len = vex_strlen(norm_prefix);

  bool result = false;
  if (prefix_len <= path_len)
  {
    if (vex_strncmp(norm_path, norm_prefix, prefix_len) == 0)
    {
      // Check it's at a component boundary
      if (prefix_len == path_len || is_path_separator(norm_path[prefix_len]))
      {
        result = true;
      }
    }
  }

  vex_free(norm_path);
  vex_free(norm_prefix);
  return result;
}

bool vex_path_ends_with(const char *path, const char *suffix)
{
  if (!path || !suffix)
    return false;

  size_t path_len = vex_strlen(path);
  size_t suffix_len = vex_strlen(suffix);

  if (suffix_len > path_len)
    return false;

  return vex_strcmp(path + path_len - suffix_len, suffix) == 0;
}

// ============================================================================
// PATH TYPE DETECTION
// ============================================================================

bool vex_path_exists(const char *path)
{
  if (!path)
    return false;

#ifdef _WIN32
  return _access(path, 0) == 0;
#else
  struct stat st;
  return lstat(path, &st) == 0;
#endif
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

bool vex_path_is_symlink(const char *path)
{
  if (!path)
    return false;

#ifdef _WIN32
  // Windows symlinks are complex; simplified check
  DWORD attrs = GetFileAttributesA(path);
  if (attrs == INVALID_FILE_ATTRIBUTES)
    return false;
  return (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
#else
  struct stat st;
  if (lstat(path, &st) != 0)
    return false;
  return S_ISLNK(st.st_mode);
#endif
}

bool vex_path_is_readable(const char *path)
{
  if (!path)
    return false;

#ifdef _WIN32
  return _access(path, 4) == 0; // R_OK
#else
  return access(path, R_OK) == 0;
#endif
}

bool vex_path_is_writable(const char *path)
{
  if (!path)
    return false;

#ifdef _WIN32
  return _access(path, 2) == 0; // W_OK
#else
  return access(path, W_OK) == 0;
#endif
}

bool vex_path_is_executable(const char *path)
{
  if (!path)
    return false;

#ifdef _WIN32
  // On Windows, check extension or FILE_ATTRIBUTE
  const char *ext = vex_path_extension(path);
  bool is_exec = (vex_strcmp(ext, ".exe") == 0 || vex_strcmp(ext, ".bat") == 0 || vex_strcmp(ext, ".cmd") == 0);
  vex_free((void *)ext);
  return is_exec;
#else
  return access(path, X_OK) == 0;
#endif
}

// ============================================================================
// DIRECTORY OPERATIONS
// ============================================================================

bool vex_dir_create(const char *path, int mode)
{
  if (!path)
  {
    vex_panic("vex_dir_create: NULL path");
  }

#ifdef _WIN32
  (void)mode; // Windows doesn't use mode
  return _mkdir(path) == 0;
#else
  return mkdir(path, (mode_t)mode) == 0;
#endif
}

bool vex_dir_create_all(const char *path, int mode)
{
  if (!path || !*path)
  {
    vex_panic("vex_dir_create_all: NULL or empty path");
  }

  // If already exists, success
  if (vex_path_is_dir(path))
    return true;

  char tmp[PATH_MAX];
  char *p = NULL;
  size_t len;

  vex_sprintf(tmp, "%s", path);
  len = vex_strlen(tmp);

  // Remove trailing separator
  if (len > 0 && is_path_separator(tmp[len - 1]))
    tmp[len - 1] = '\0';

#ifdef _WIN32
  // Skip drive letter
  p = tmp;
  if (len >= 2 && isalpha(tmp[0]) && tmp[1] == ':')
    p += 2;
  if (*p == PATH_SEP)
    p++;
#else
  p = tmp + 1; // Skip leading /
#endif

  for (; *p; p++)
  {
    if (is_path_separator(*p))
    {
      *p = '\0';
      if (!vex_path_exists(tmp))
      {
#ifdef _WIN32
        _mkdir(tmp);
#else
        mkdir(tmp, (mode_t)mode);
#endif
      }
      *p = PATH_SEP;
    }
  }

  // Create final directory
#ifdef _WIN32
  return _mkdir(tmp) == 0 || errno == EEXIST;
#else
  return mkdir(tmp, (mode_t)mode) == 0 || errno == EEXIST;
#endif
}

bool vex_dir_remove(const char *path)
{
  if (!path)
  {
    vex_panic("vex_dir_remove: NULL path");
  }

  return rmdir(path) == 0;
}

static bool remove_directory_recursive(const char *path)
{
#ifdef _WIN32
  WIN32_FIND_DATAA find_data;
  char search_path[PATH_MAX];
  vex_snprintf(search_path, sizeof(search_path), "%s\\*", path);

  HANDLE hFind = FindFirstFileA(search_path, &find_data);
  if (hFind == INVALID_HANDLE_VALUE)
    return false;

  do
  {
    if (vex_strcmp(find_data.cFileName, ".") == 0 || vex_strcmp(find_data.cFileName, "..") == 0)
      continue;

    char full_path[PATH_MAX];
    vex_snprintf(full_path, sizeof(full_path), "%s\\%s", path, find_data.cFileName);

    if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
    {
      if (!remove_directory_recursive(full_path))
      {
        FindClose(hFind);
        return false;
      }
    }
    else
    {
      if (!DeleteFileA(full_path))
      {
        FindClose(hFind);
        return false;
      }
    }
  } while (FindNextFileA(hFind, &find_data));

  FindClose(hFind);
  return RemoveDirectoryA(path) != 0;
#else
  DIR *dir = opendir(path);
  if (!dir)
    return false;

  struct dirent *entry;
  bool success = true;

  while ((entry = readdir(dir)) != NULL && success)
  {
    if (vex_strcmp(entry->d_name, ".") == 0 || vex_strcmp(entry->d_name, "..") == 0)
      continue;

    char *full_path = vex_path_join(path, entry->d_name);

    struct stat st;
    if (lstat(full_path, &st) == 0)
    {
      if (S_ISDIR(st.st_mode))
      {
        success = remove_directory_recursive(full_path);
      }
      else
      {
        success = (unlink(full_path) == 0);
      }
    }

    vex_free(full_path);
  }

  closedir(dir);

  if (success)
  {
    success = (rmdir(path) == 0);
  }

  return success;
#endif
}

bool vex_dir_remove_all(const char *path)
{
  if (!path)
  {
    vex_panic("vex_dir_remove_all: NULL path");
  }

  if (!vex_path_exists(path))
    return true; // Already doesn't exist

  return remove_directory_recursive(path);
}

// ============================================================================
// GLOB/PATTERN MATCHING
// ============================================================================

// Simple glob matching (*, ?, [...])
static bool match_pattern(const char *pattern, const char *str, bool case_sensitive)
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
        if (match_pattern(pattern, str, case_sensitive))
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
          char c = case_sensitive ? *str : tolower(*str);
          char start = case_sensitive ? pattern[0] : tolower(pattern[0]);
          char end = case_sensitive ? pattern[2] : tolower(pattern[2]);
          if (c >= start && c <= end)
          {
            matched = true;
          }
          pattern += 3;
        }
        else
        {
          // Single char
          char c = case_sensitive ? *str : tolower(*str);
          char p = case_sensitive ? *pattern : tolower(*pattern);
          if (c == p)
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
      char c1 = case_sensitive ? *pattern : tolower(*pattern);
      char c2 = case_sensitive ? *str : tolower(*str);
      if (c1 != c2)
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

bool vex_path_match_glob(const char *path, const char *pattern)
{
  if (!path || !pattern)
    return false;

  return match_pattern(pattern, path, true);
}

VexArray *vex_path_glob(const char *pattern)
{
  if (!pattern)
  {
    vex_panic("vex_path_glob: NULL pattern");
  }

  VexArray *results = NULL;

#ifdef _WIN32
  WIN32_FIND_DATAA find_data;
  HANDLE hFind = FindFirstFileA("*", &find_data);
  if (hFind == INVALID_HANDLE_VALUE)
    return results;

  do
  {
    if (vex_strcmp(find_data.cFileName, ".") == 0 || vex_strcmp(find_data.cFileName, "..") == 0)
      continue;

    if (match_pattern(pattern, find_data.cFileName, true))
    {
      char *matched_path = vex_strdup(find_data.cFileName);
      results = vex_array_append(results, &matched_path, sizeof(char *));
    }
  } while (FindNextFileA(hFind, &find_data));

  FindClose(hFind);
#else
  DIR *dir = opendir(".");
  if (!dir)
    return results;

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL)
  {
    if (entry->d_name[0] == '.')
      continue; // Skip hidden files

    if (match_pattern(pattern, entry->d_name, true))
    {
      char *matched_path = vex_strdup(entry->d_name);
      results = vex_array_append(results, &matched_path, sizeof(char *));
    }
  }

  closedir(dir);
#endif

  return results;
}

static void glob_recursive_internal(const char *dir_path, const char *pattern, VexArray **results)
{
#ifdef _WIN32
  WIN32_FIND_DATAA find_data;
  char search_path[PATH_MAX];
  vex_snprintf(search_path, sizeof(search_path), "%s\\*", dir_path);

  HANDLE hFind = FindFirstFileA(search_path, &find_data);
  if (hFind == INVALID_HANDLE_VALUE)
    return;

  do
  {
    if (vex_strcmp(find_data.cFileName, ".") == 0 || vex_strcmp(find_data.cFileName, "..") == 0)
      continue;

    char *full_path = vex_path_join(dir_path, find_data.cFileName);

    if (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
    {
      glob_recursive_internal(full_path, pattern, results);
    }
    else
    {
      if (match_pattern(pattern, find_data.cFileName, true))
      {
        char *path_copy = vex_strdup(full_path);
        *results = vex_array_append(*results, &path_copy, sizeof(char *));
      }
    }

    vex_free(full_path);
  } while (FindNextFileA(hFind, &find_data));

  FindClose(hFind);
#else
  DIR *dir = opendir(dir_path);
  if (!dir)
    return;

  struct dirent *entry;
  while ((entry = readdir(dir)) != NULL)
  {
    if (entry->d_name[0] == '.')
      continue;

    char *full_path = vex_path_join(dir_path, entry->d_name);

    if (entry->d_type == DT_DIR)
    {
      glob_recursive_internal(full_path, pattern, results);
    }
    else if (entry->d_type == DT_REG)
    {
      if (match_pattern(pattern, entry->d_name, true))
      {
        char *path_copy = vex_strdup(full_path);
        *results = vex_array_append(*results, &path_copy, sizeof(char *));
      }
    }

    vex_free(full_path);
  }

  closedir(dir);
#endif
}

VexArray *vex_path_glob_recursive(const char *dir_path, const char *pattern)
{
  if (!dir_path || !pattern)
  {
    vex_panic("vex_path_glob_recursive: NULL parameter");
  }

  VexArray *results = NULL;
  glob_recursive_internal(dir_path, pattern, &results);
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

#ifdef _WIN32
  WIN32_FIND_DATAA find_data;
  char search_path[PATH_MAX];
  vex_snprintf(search_path, sizeof(search_path), "%s\\*", dir_path);

  HANDLE hFind = FindFirstFileA(search_path, &find_data);
  if (hFind == INVALID_HANDLE_VALUE)
    return entries;

  do
  {
    if (vex_strcmp(find_data.cFileName, ".") == 0 || vex_strcmp(find_data.cFileName, "..") == 0)
      continue;

    char *full_path = vex_path_join(dir_path, find_data.cFileName);

    VexDirEntry *dir_entry = (VexDirEntry *)vex_malloc(sizeof(VexDirEntry));
    dir_entry->path = full_path;
    dir_entry->is_dir = (find_data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
    dir_entry->size = dir_entry->is_dir ? 0 : ((size_t)find_data.nFileSizeHigh << 32) | find_data.nFileSizeLow;

    entries = vex_array_append(entries, &dir_entry, sizeof(VexDirEntry *));
  } while (FindNextFileA(hFind, &find_data));

  FindClose(hFind);
#else
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
#endif

  return entries;
}

// ============================================================================
// METADATA
// ============================================================================

VexPathMetadata *vex_path_metadata(const char *path)
{
  if (!path)
    return NULL;

  VexPathMetadata *meta = (VexPathMetadata *)vex_calloc(1, sizeof(VexPathMetadata));

#ifdef _WIN32
  WIN32_FILE_ATTRIBUTE_DATA attrs;
  if (!GetFileAttributesExA(path, GetFileExInfoStandard, &attrs))
  {
    vex_free(meta);
    return NULL;
  }

  meta->size = ((uint64_t)attrs.nFileSizeHigh << 32) | attrs.nFileSizeLow;
  meta->is_dir = (attrs.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
  meta->is_file = !meta->is_dir;
  meta->is_symlink = (attrs.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0;

  // Convert FILETIME to Unix timestamp
  ULARGE_INTEGER ull;
  ull.LowPart = attrs.ftLastWriteTime.dwLowDateTime;
  ull.HighPart = attrs.ftLastWriteTime.dwHighDateTime;
  meta->modified_time = (int64_t)(ull.QuadPart / 10000000ULL - 11644473600ULL);

  ull.LowPart = attrs.ftCreationTime.dwLowDateTime;
  ull.HighPart = attrs.ftCreationTime.dwHighDateTime;
  meta->created_time = (int64_t)(ull.QuadPart / 10000000ULL - 11644473600ULL);

  ull.LowPart = attrs.ftLastAccessTime.dwLowDateTime;
  ull.HighPart = attrs.ftLastAccessTime.dwHighDateTime;
  meta->accessed_time = (int64_t)(ull.QuadPart / 10000000ULL - 11644473600ULL);

  meta->mode = 0; // Windows doesn't have Unix permissions
#else
  struct stat st;
  if (lstat(path, &st) != 0)
  {
    vex_free(meta);
    return NULL;
  }

  meta->size = (uint64_t)st.st_size;
  meta->modified_time = (int64_t)st.st_mtime;
  meta->created_time = (int64_t)st.st_ctime;
  meta->accessed_time = (int64_t)st.st_atime;
  meta->mode = (uint32_t)st.st_mode;
  meta->is_dir = S_ISDIR(st.st_mode);
  meta->is_file = S_ISREG(st.st_mode);
  meta->is_symlink = S_ISLNK(st.st_mode);
#endif

  return meta;
}

uint32_t vex_path_permissions(const char *path)
{
  if (!path)
    return 0;

#ifdef _WIN32
  // Windows doesn't have Unix permissions
  return 0;
#else
  struct stat st;
  if (stat(path, &st) != 0)
    return 0;
  return (uint32_t)(st.st_mode & 0777);
#endif
}

bool vex_path_set_permissions(const char *path, uint32_t mode)
{
  if (!path)
    return false;

#ifdef _WIN32
  // Windows doesn't support chmod
  return false;
#else
  return chmod(path, (mode_t)mode) == 0;
#endif
}

// ============================================================================
// SYMLINK OPERATIONS
// ============================================================================

bool vex_symlink_create(const char *target, const char *link_path)
{
  if (!target || !link_path)
    return false;

#ifdef _WIN32
  // Windows requires admin privileges for symlinks
  return CreateSymbolicLinkA(link_path, target, 0) != 0;
#else
  return symlink(target, link_path) == 0;
#endif
}

char *vex_symlink_read(const char *link_path)
{
  if (!link_path)
    return NULL;

#ifdef _WIN32
  // Windows symlink reading is complex; simplified
  char buffer[PATH_MAX];
  DWORD len = GetFinalPathNameByHandleA(
      CreateFileA(link_path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL),
      buffer, PATH_MAX, FILE_NAME_NORMALIZED);
  if (len == 0)
    return NULL;
  return vex_strdup(buffer);
#else
  char buffer[PATH_MAX];
  ssize_t len = readlink(link_path, buffer, sizeof(buffer) - 1);
  if (len < 0)
    return NULL;
  buffer[len] = '\0';
  return vex_strdup(buffer);
#endif
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

  size_t src_size;
  char *data = vex_file_read_all(src, &src_size);
  if (!data)
    return false;

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
#ifdef _WIN32
  char temp_path[PATH_MAX];
  char temp_file[PATH_MAX];

  if (GetTempPathA(PATH_MAX, temp_path) == 0)
    return NULL;

  const char *use_prefix = prefix ? prefix : "vex";

  if (GetTempFileNameA(temp_path, use_prefix, 0, temp_file) == 0)
    return NULL;

  return vex_strdup(temp_file);
#else
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
#endif
}

char *vex_path_temp_dir(const char *prefix)
{
#ifdef _WIN32
  char temp_path[PATH_MAX];
  if (GetTempPathA(PATH_MAX, temp_path) == 0)
    return NULL;

  const char *use_prefix = prefix ? prefix : "vex";

  char temp_dir[PATH_MAX];
  vex_snprintf(temp_dir, sizeof(temp_dir), "%s%s_%u", temp_path, use_prefix, (unsigned)GetTickCount());

  if (!CreateDirectoryA(temp_dir, NULL))
    return NULL;

  return vex_strdup(temp_dir);
#else
  const char *tmpdir = getenv("TMPDIR");
  if (!tmpdir)
    tmpdir = "/tmp";

  const char *use_prefix = prefix ? prefix : "vex";

  char template[PATH_MAX];
  snprintf(template, sizeof(template), "%s/%s_XXXXXX", tmpdir, use_prefix);

  if (mkdtemp(template) == NULL)
    return NULL;

  return vex_strdup(template);
#endif
}
