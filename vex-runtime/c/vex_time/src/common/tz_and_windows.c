#include "vex_time.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

#if !defined(_WIN32)
#include <time.h>
#include <unistd.h>
#include <sys/stat.h>
#endif

struct VexTz {
  int is_fixed;
  int fixed_offset;
  char fixed_name[32];

  /* tzfile */
  int has_tzif;
  int timecnt, typecnt, charcnt;
  int64_t* trans;
  unsigned char* trans_type;
  struct Ttinfo { int32_t gmtoff; unsigned char isdst; unsigned char abbr_index; } *ttis;
  char* abbrs;

  /* opaque data for memory-loaded tzif */
  unsigned char* mem_blob;
  size_t mem_len;
};

static const char* g_tzdir = NULL;
void vt_tz_set_dir(const char* path){ g_tzdir = path; }

/* ---------------- TZ helpers ---------------- */
static int read_be32(const unsigned char* p){ return (int)((p[0]<<24)|(p[1]<<16)|(p[2]<<8)|p[3]); }
static int64_t read_be64(const unsigned char* p){ int64_t v=0; for (int i=0;i<8;i++){ v=(v<<8)|p[i]; } return v; }

static VexTz* tz_utc_singleton(void){
  static VexTz utc = {0}; static int inited=0;
  if (!inited){ utc.is_fixed=1; utc.fixed_offset=0; snprintf(utc.fixed_name,sizeof(utc.fixed_name),"UTC"); inited=1; }
  return &utc;
}
VexTz* vt_tz_utc(void){ return tz_utc_singleton(); }

VexTz* vt_tz_fixed(const char* name, int offset_sec){
  VexTz* z=(VexTz*)calloc(1,sizeof(VexTz));
  z->is_fixed=1; z->fixed_offset=offset_sec;
  snprintf(z->fixed_name,sizeof(z->fixed_name),"%s", name?name:"FIX");
  return z;
}

/* Helper for reading from file or memory */
typedef struct {
  FILE* f;
  const unsigned char* mem;
  size_t len;
  size_t off;
} ReadContext;

static int readn(ReadContext* ctx, void* dst, size_t n){
  if (ctx->f){ return fread(dst,1,n,ctx->f)==n?0:-1; }
  if (ctx->off+n>ctx->len) return -1;
  memcpy(dst, ctx->mem+ctx->off, n);
  ctx->off+=n;
  return 0;
}

/* Parse TZif v2/v3 either from file or memory */
static VexTz* load_tzif_stream(FILE* f, const unsigned char* mem, size_t len){
  unsigned char hdr[44];
  ReadContext ctx = {f, mem, len, 0};

  if (readn(&ctx,hdr,44)!=0) return NULL;
  if (memcmp(hdr,"TZif",4)!=0) return NULL;
  int tzh_ttisgmtcnt = read_be32(hdr+20);
  int tzh_ttisstdcnt = read_be32(hdr+24);
  int tzh_leapcnt    = read_be32(hdr+28);
  int tzh_timecnt    = read_be32(hdr+32);
  int tzh_typecnt    = read_be32(hdr+36);
  int tzh_charcnt    = read_be32(hdr+40);

  size_t skip = (size_t)tzh_timecnt*4 + tzh_timecnt + (size_t)tzh_typecnt*6 + tzh_charcnt + (size_t)tzh_leapcnt*8 + tzh_ttisstdcnt + tzh_ttisgmtcnt;
  if (f) fseek(f, (long)(44 + skip), SEEK_SET); else ctx.off = 44 + skip;

  if (readn(&ctx,hdr,44)!=0) return NULL;
  if (memcmp(hdr,"TZif",4)!=0) return NULL;

  tzh_ttisgmtcnt = read_be32(hdr+20);
  tzh_ttisstdcnt = read_be32(hdr+24);
  tzh_leapcnt    = read_be32(hdr+28);
  tzh_timecnt    = read_be32(hdr+32);
  tzh_typecnt    = read_be32(hdr+36);
  tzh_charcnt    = read_be32(hdr+40);

  VexTz* z=(VexTz*)calloc(1,sizeof(VexTz)); z->has_tzif=1;
  z->timecnt=tzh_timecnt; z->typecnt=tzh_typecnt; z->charcnt=tzh_charcnt;
  z->trans = (int64_t*)calloc(tzh_timecnt, sizeof(int64_t));
  z->trans_type = (unsigned char*)calloc(tzh_timecnt, 1);
  z->ttis = (struct Ttinfo*)calloc(tzh_typecnt, sizeof(struct Ttinfo));
  z->abbrs = (char*)calloc(tzh_charcnt+1,1);

  for (int i=0;i<tzh_timecnt;i++){ unsigned char buf[8]; if (readn(&ctx,buf,8)!=0){ vt_tz_release(z); return NULL; } z->trans[i] = read_be64(buf); }
  if (readn(&ctx,z->trans_type, tzh_timecnt)!=0){ vt_tz_release(z); return NULL; }
  for (int i=0;i<tzh_typecnt;i++){
    unsigned char tt[6]; if (readn(&ctx,tt,6)!=0){ vt_tz_release(z); return NULL; }
    z->ttis[i].gmtoff = (int32_t)read_be32(tt);
    z->ttis[i].isdst = tt[4];
    z->ttis[i].abbr_index = tt[5];
  }
  if (readn(&ctx,z->abbrs, tzh_charcnt)!=0){ vt_tz_release(z); return NULL; }
  z->abbrs[tzh_charcnt]='\0';
  /* skip trailing sections */
  size_t tail = (size_t)tzh_leapcnt*12 + tzh_ttisstdcnt + tzh_ttisgmtcnt;
  if (f) fseek(f, (long)tail, SEEK_CUR); else ctx.off += tail;
  return z;
}

static const char* getenv_or(const char* k, const char* defv){
#if defined(_WIN32)
  static char buf[1024];
  DWORD n = GetEnvironmentVariableA(k, buf, sizeof buf);
  if (n>0 && n<sizeof buf) return buf;
  return defv;
#else
  const char* v = getenv(k); return v ? v : defv;
#endif
}

static VexTz* load_tzif_file(const char* path){
  FILE* f = fopen(path,"rb"); if (!f) return NULL;
  VexTz* z = load_tzif_stream(f, NULL, 0);
  fclose(f); return z;
}

VexTz* vt_tz_load_from_memory(const char* name, const unsigned char* tzif, size_t len){
  (void)name;
  return load_tzif_stream(NULL, tzif, len);
}

VexTz* vt_tz_load(const char* name){
  if (!name || strcmp(name,"UTC")==0) return vt_tz_utc();
  const char* dir = g_tzdir ? g_tzdir : getenv_or("VT_TZDIR", NULL);
  char path[768];
  if (dir){ snprintf(path,sizeof(path), "%s/%s", dir, name); VexTz* z = load_tzif_file(path); if (z) return z; }
#if !defined(_WIN32)
  const char* default_dir = "/usr/share/zoneinfo";
  snprintf(path,sizeof(path), "%s/%s", default_dir, name);
  VexTz* z = load_tzif_file(path); if (z) return z;
  /* try absolute path */
  z = load_tzif_file(name); if (z) return z;
#endif
  return NULL;
}

void vt_tz_release(VexTz* tz){
  if (!tz) return;
  if (tz==vt_tz_utc()) return;
  if (!tz->is_fixed && tz->has_tzif){
    free(tz->trans); free(tz->trans_type); free(tz->ttis); free(tz->abbrs);
    if (tz->mem_blob) free(tz->mem_blob);
  }
  free(tz);
}

int vt_tz_offset_at(const VexTz* tz, VexInstant utc, int* offset_sec, const char** abbr){
  if (!tz){ if (offset_sec) *offset_sec=0; if (abbr) *abbr="UTC"; return 0; }
  if (tz->is_fixed){ if (offset_sec) *offset_sec = tz->fixed_offset; if (abbr) *abbr=tz->fixed_name; return 0; }
  if (!tz->has_tzif){ if (offset_sec) *offset_sec = 0; if (abbr) *abbr="UTC"; return 0; }
  int idx=-1;
  for (int i=0;i<tz->timecnt;i++){ if (utc.unix_sec >= tz->trans[i]) idx=i; else break; }
  int type = (idx>=0 && idx<tz->timecnt) ? tz->trans_type[idx] : 0;
  if (type >= tz->typecnt) type=0;
  int off = tz->ttis[type].gmtoff;
  const char* a = tz->abbrs + tz->ttis[type].abbr_index;
  if (offset_sec) *offset_sec = off; if (abbr) *abbr=a;
  return 0;
}

VexInstant vt_utc_to_tz(const VexTz* tz, VexInstant utc){
  int off=0; vt_tz_offset_at(tz, utc, &off, NULL);
  return vt_instant_from_unix(utc.unix_sec + off, utc.nsec);
}

/* ---------- vt_tz_local ---------- */
#if defined(_WIN32)
#include <windows.h>
static const struct { const char* win; const char* iana; } WIN_TO_IANA[] = {
  {"UTC", "UTC"},
  {"Greenwich Standard Time", "Etc/GMT"},
  {"GMT Standard Time", "Europe/London"},
  {"Morocco Standard Time", "Africa/Casablanca"},
  {"W. Europe Standard Time", "Europe/Berlin"},
  {"Central Europe Standard Time", "Europe/Budapest"},
  {"E. Europe Standard Time", "Europe/Bucharest"},
  {"Turkey Standard Time", "Europe/Istanbul"},
  {"GTB Standard Time", "Europe/Athens"},
  {"Russian Standard Time", "Europe/Moscow"},
  {"Israel Standard Time", "Asia/Jerusalem"},
  {"Egypt Standard Time", "Africa/Cairo"},
  {"Arabic Standard Time", "Asia/Baghdad"},
  {"Arabian Standard Time", "Asia/Dubai"},
  {"India Standard Time", "Asia/Kolkata"},
  {"SE Asia Standard Time", "Asia/Bangkok"},
  {"China Standard Time", "Asia/Shanghai"},
  {"Tokyo Standard Time", "Asia/Tokyo"},
  {"AUS Eastern Standard Time", "Australia/Sydney"},
  {"E. Australia Standard Time", "Australia/Brisbane"},
  {"New Zealand Standard Time", "Pacific/Auckland"},
  {"Pacific SA Standard Time", "America/Santiago"},
  {"Argentina Standard Time", "America/Argentina/Buenos_Aires"},
  {"SA Eastern Standard Time", "America/Sao_Paulo"},
  {"Eastern Standard Time", "America/New_York"},
  {"Central Standard Time", "America/Chicago"},
  {"Mountain Standard Time", "America/Denver"},
  {"US Mountain Standard Time", "America/Phoenix"},
  {"Pacific Standard Time", "America/Los_Angeles"},
  {"Alaskan Standard Time", "America/Anchorage"},
  {"Hawaiian Standard Time", "Pacific/Honolulu"},
};
static const char* map_win_to_iana(const char* win){
  for (size_t i=0;i<sizeof(WIN_TO_IANA)/sizeof(WIN_TO_IANA[0]); i++){
    if (_stricmp(WIN_TO_IANA[i].win, win)==0) return WIN_TO_IANA[i].iana;
  }
  return NULL;
}
VexTz* vt_tz_local(void){
  DYNAMIC_TIME_ZONE_INFORMATION dtzi; GetDynamicTimeZoneInformation(&dtzi);
  /* Prefer registry key if available (on Win10+) */
  const char* iana = NULL;
  /* Try dynamic key name first */
  char keyname[256]; WideCharToMultiByte(CP_UTF8,0, dtzi.TimeZoneKeyName, -1, keyname, sizeof keyname, NULL, NULL);
  if (keyname[0]) iana = map_win_to_iana(keyname);
  if (!iana){
    char stdname[256]; WideCharToMultiByte(CP_UTF8,0, dtzi.StandardName, -1, stdname, sizeof stdname, NULL, NULL);
    if (stdname[0]) iana = map_win_to_iana(stdname);
  }
  if (iana){
    VexTz* z = vt_tz_load(iana);
    if (z) return z;
  }
  /* Fallback: fixed current bias */
  int bias_minutes = (int)dtzi.Bias; /* minutes west of UTC */
  return vt_tz_fixed("LOCAL", -bias_minutes*60);
}

#else /* POSIX */
#include <sys/stat.h>
#include <limits.h>

static int readlink_into(const char* path, char* buf, size_t buflen){
  ssize_t n = readlink(path, buf, buflen-1);
  if (n<0) return -1; buf[n]=0; return 0;
}

VexTz* vt_tz_local(void){
  /* Try /etc/timezone name */
  char linkbuf[PATH_MAX]; linkbuf[0]=0;
  struct stat st;
  if (lstat("/etc/localtime",&st)==0 && S_ISLNK(st.st_mode)){
    if (readlink_into("/etc/localtime", linkbuf, sizeof linkbuf)==0){
      const char* p = strstr(linkbuf, "zoneinfo/");
      if (p) p += 9; else p = linkbuf;
      VexTz* z = vt_tz_load(p);
      if (z) return z;
    }
  }
  /* Fallback: current offset via localtime */
  time_t now = time(NULL);
  struct tm lt; localtime_r(&now, &lt);
  struct tm gt; gmtime_r(&now, &gt);
  time_t lt_as_utc = timegm(&lt);
  int offset = (int)difftime(now, lt_as_utc);
  return vt_tz_fixed(lt.tm_isdst?"LOCAL-DST":"LOCAL", offset);
}
#endif
