#include "vex_fastenc.h"
#include <stdint.h>
#include <stddef.h>

#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <bcrypt.h>
#pragma comment(lib, "bcrypt.lib")
int vex_os_random(void* dst, size_t n){
  NTSTATUS st = BCryptGenRandom(NULL, (PUCHAR)dst, (ULONG)n, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
  return st==0?0:-1;
}
#else
#include <fcntl.h>
#include <unistd.h>
int vex_os_random(void* dst, size_t n){
  int fd = open("/dev/urandom", O_RDONLY);
  if (fd<0) return -1;
  uint8_t* p=(uint8_t*)dst; size_t r=0;
  while (r<n){
    ssize_t k = read(fd, p+r, n-r);
    if (k<=0){ close(fd); return -1; }
    r += (size_t)k;
  }
  close(fd);
  return 0;
}
#endif
