// vex_fs.h - Header for Vex FS native module
#ifndef VEX_FS_H
#define VEX_FS_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// File operations
char *vex_file_read_all_str(const char *path, size_t *out_size);
bool vex_file_write_all_str(const char *path, const void *data, size_t size);
bool vex_file_exists_str(const char *path);
bool vex_file_remove_str(const char *path);
bool vex_file_rename_str(const char *old_path, const char *new_path);
bool vex_file_copy_str(const char *src, const char *dst);
bool vex_file_move_str(const char *src, const char *dst);

// Directory operations
bool vex_dir_create_str(const char *path);
bool vex_dir_remove_str(const char *path);
bool vex_dir_exists_str(const char *path);

// String helpers
const char *vex_str_to_cstr(const char *s);
const char *vex_cstr_to_str(const char *ptr);
// vex_strlen is provided by vex_runtime

#ifdef __cplusplus
}
#endif

#endif // VEX_FS_H
