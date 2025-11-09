#include "vex_time.h"
#include "fast_parse.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>

/* ---- Conversions ---- */
VexInstant vt_instant_from_unix(int64_t sec, int32_t nsec){
  VexInstant t; t.unix_sec = sec; t.nsec = nsec; t._pad = 0; return t;
}
void vt_instant_to_unix(VexInstant t, int64_t* sec, int32_t* nsec){
  if (sec) *sec = t.unix_sec; if (nsec) *nsec = t.nsec;
}

/* ---- Duration parse/format ---- */
int vt_parse_duration(const char* s, VexDuration* out_ns){
  if (!s || !out_ns) return -1;
  const char* p = s; int neg=0; if (*p=='+'||*p=='-'){ neg=(*p=='-'); p++; }
  long double total=0.0L;
  while (*p){
    char* end=NULL; long double v = strtold(p,&end); if (end==p) return -1; p=end;
    if (strncmp(p,"ns",2)==0){ total += v; p+=2; }
    else if (strncmp(p,"us",2)==0){ total += v*1000.0L; p+=2; }
    else if ((unsigned char)p[0]==0xC2 && (unsigned char)p[1]==0xB5 && p[2]=='s'){ total += v*1000.0L; p+=3; }
    else if (strncmp(p,"µs",2)==0){ total += v*1000.0L; p+=2; }
    else if (strncmp(p,"ms",2)==0){ total += v*1000000.0L; p+=2; }
    else if (*p=='s'){ total += v*1000000000.0L; p+=1; }
    else if (*p=='m'){ total += v*60.0L*1000000000.0L; p+=1; }
    else if (*p=='h'){ total += v*3600.0L*1000000000.0L; p+=1; }
    else return -1;
  }
  if (neg) total=-total;
  if (total>9.22e18L) total=9.22e18L; if (total<-9.22e18L) total=-9.22e18L;
  *out_ns = (VexDuration)(total);
  return 0;
}

/* Fast integer-to-string (no snprintf overhead) */
static inline char* fast_itoa(char* p, long long val) {
  if (val == 0) { *p++ = '0'; return p; }
  char* start = p;
  int neg = (val < 0);
  if (neg) {
    *p++ = '-';
    val = -val;
    start = p;
  }
  
  /* Write digits in reverse */
  char* end = p;
  do {
    *end++ = '0' + (val % 10);
    val /= 10;
  } while (val > 0);
  *end = '\0';
  
  /* Reverse string */
  char* rev_end = end - 1;
  while (start < rev_end) {
    char tmp = *start;
    *start = *rev_end;
    *rev_end = tmp;
    start++;
    rev_end--;
  }
  
  return end;
}

/* Fast integer-to-string with zero-padding */
static inline char* fast_itoa_pad(char* p, long long val, int width) {
  char* start = p;
  if (val == 0) {
    for (int i = 0; i < width; i++) *p++ = '0';
    return p;
  }
  
  char* end = p;
  do {
    *end++ = '0' + (val % 10);
    val /= 10;
  } while (val > 0);
  
  /* Pad with zeros */
  while (end - start < width) *end++ = '0';
  *end = '\0';
  
  /* Reverse */
  char* rev_end = end - 1;
  while (start < rev_end) {
    char tmp = *start;
    *start = *rev_end;
    *rev_end = tmp;
    start++;
    rev_end--;
  }
  
  return end;
}

int vt_format_duration(VexDuration ns, char* buf, size_t buflen){
  if (!buf || buflen<2) return -1;
  if (ns==0){ buf[0]='0'; buf[1]='s'; buf[2]='\0'; return 0; }
  
  char* p = buf;
  size_t remain = buflen;
  
  char sign = (ns<0)?'-':'+'; 
  if (ns<0) ns = -ns;
  
  /* Calculate components (optimized: avoid repeated divisions) */
  const int64_t NS_PER_SEC = 1000000000LL;
  const int64_t NS_PER_MIN = 60LL * NS_PER_SEC;
  const int64_t NS_PER_HOUR = 3600LL * NS_PER_SEC;
  
  int64_t hours = (int64_t)(ns / NS_PER_HOUR);
  int64_t rem = (int64_t)(ns - hours * NS_PER_HOUR);
  int64_t mins = rem / NS_PER_MIN;
  rem -= mins * NS_PER_MIN;
  int64_t secs = rem / NS_PER_SEC;
  rem -= secs * NS_PER_SEC;
  int64_t ms = rem / 1000000LL;
  rem -= ms * 1000000LL;
  int64_t us = rem / 1000LL;
  int64_t nss = rem - us * 1000LL;
  
  /* Format based on largest component (manual formatting, no snprintf) */
  if (hours > 0) {
    if (remain < 20) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, hours);
    *p++ = 'h';
    p = fast_itoa(p, mins);
    *p++ = 'm';
    p = fast_itoa(p, secs);
    *p++ = 's';
    *p = '\0';
  } else if (mins > 0) {
    if (remain < 25) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, mins);
    *p++ = 'm';
    p = fast_itoa(p, secs);
    *p++ = '.';
    p = fast_itoa_pad(p, ms, 3);
    *p++ = 's';
    *p = '\0';
  } else if (secs > 0) {
    if (remain < 20) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, secs);
    *p++ = '.';
    p = fast_itoa_pad(p, ms, 3);
    *p++ = 's';
    *p = '\0';
  } else if (ms > 0) {
    if (remain < 15) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, ms);
    *p++ = 'm';
    *p++ = 's';
    *p = '\0';
  } else if (us > 0) {
    if (remain < 15) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, us);
    *p++ = 'u';
    *p++ = 's';
    *p = '\0';
  } else {
    if (remain < 15) return -1;
    if (sign == '-') *p++ = '-';
    p = fast_itoa(p, (long long)nss);
    *p++ = 'n';
    *p++ = 's';
    *p = '\0';
  }
  
  return 0;
}

/* ---- RFC3339 ---- */
#if defined(_WIN32)
#include <windows.h>
#else
#include <time.h>
#endif
#if !defined(_WIN32)
static time_t timegm_compat(struct tm* t){ return timegm(t); }
#else
#include <stdlib.h>
static time_t timegm_compat(struct tm* t){ return _mkgmtime(t); }
#endif

int vt_format_rfc3339_utc(VexInstant t, char* buf, size_t buflen){
  /* Use SWAR-optimized fast version */
  return vt_format_rfc3339_utc_fast(t, buf, buflen);
}

int vt_parse_rfc3339(const char* s, VexInstant* out){
  /* Use SWAR-optimized fast version */
  return vt_parse_rfc3339_fast(s, out);
}

/* ---- Arithmetic ---- */
VexTime vt_add(VexTime t, VexDuration d){
  VexTime r = t;
  int64_t sec = r.wall.unix_sec;
  int64_t nsec = (int64_t)r.wall.nsec + d;
  sec += nsec / 1000000000LL;
  nsec %= 1000000000LL; if (nsec<0){ nsec += 1000000000LL; sec -= 1; }
  r.wall.unix_sec = sec; r.wall.nsec = (int32_t)nsec;
  r.mono_ns += (d>=0)?(uint64_t)d:(uint64_t)(-d);
  return r;
}
VexDuration vt_sub(VexTime t, VexTime u){
  if (t.mono_ns && u.mono_ns){
    int64_t diff = (int64_t)(t.mono_ns - u.mono_ns);
    return diff;
  }
  int64_t ds = t.wall.unix_sec - u.wall.unix_sec;
  int64_t dns = (int64_t)t.wall.nsec - (int64_t)u.wall.nsec;
  return ds*1000000000LL + dns;
}
VexDuration vt_since(VexTime t){ VexTime now; vt_now(&now); return vt_sub(now,t); }
VexDuration vt_until(VexTime t){ VexTime now; vt_now(&now); return vt_sub(t,now); }

/* ---- Go layout helpers ---- */
#if !defined(_WIN32)
#include <time.h>
static void gmtime_s_compat2(const time_t* tt, struct tm* out){ gmtime_r(tt, out); }
#else
#include <time.h>
static void gmtime_s_compat2(const time_t* tt, struct tm* out){ gmtime_s(out, tt); }
#endif

static void append(char** p, size_t* r, const char* s){
  while (*s && *r>1){ **p = *s; (*p)++; (*r)--; s++; }
}
static void append_int_w(char** p, size_t* r, int v, int width){
  char buf[32]; snprintf(buf,sizeof(buf), "%0*d", width, v); append(p,r,buf);
}
static void append_int_no(char** p, size_t* r, int v){
  char buf[32]; snprintf(buf,sizeof(buf), "%d", v); append(p,r,buf);
}
static void append_frac(char** p, size_t* r, int nsec, int digits){
  char buf[16]; int nn = nsec;
  if (digits==9) snprintf(buf,sizeof(buf), "%09d", nn);
  else if (digits==6) snprintf(buf,sizeof(buf), "%06d", nn/1000);
  else if (digits==3) snprintf(buf,sizeof(buf), "%03d", nn/1000000);
  else snprintf(buf,sizeof(buf), "%09d", nn);
  buf[digits] = '\0'; append(p,r,buf);
}
static void append_tz(char** p, size_t* r, int offset_sec, int colon, int z_if_zero){
  if (z_if_zero && offset_sec==0){ append(p,r,"Z"); return; }
  char sign = offset_sec>=0?'+':'-';
  int a = offset_sec>=0?offset_sec:-offset_sec;
  int hh = a/3600; int mm = (a/60)%60;
  char buf[8];
  if (colon) snprintf(buf,sizeof(buf), "%c%02d:%02d", sign, hh, mm);
  else snprintf(buf,sizeof(buf), "%c%02d%02d", sign, hh, mm);
  append(p,r,buf);
}

static const char* MONTHS_FULL[12]={"January","February","March","April","May","June","July","August","September","October","November","December"};
static const char* MONTHS_ABR[12] ={"Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"};
static const char* WDAYS_FULL[7]  ={"Sunday","Monday","Tuesday","Wednesday","Thursday","Friday","Saturday"};
static const char* WDAYS_ABR[7]   ={"Sun","Mon","Tue","Wed","Thu","Fri","Sat"};

int vt_format_go(VexInstant utc, const VexTz* tz, const char* layout, char* out, size_t outlen){
  if (!out || outlen==0) return -1;
  const char* abbr="UTC"; int off=0;
  vt_tz_offset_at(tz, utc, &off, &abbr);
  VexInstant loc = vt_utc_to_tz(tz, utc);
  time_t sec = (time_t)loc.unix_sec; struct tm tmv; gmtime_s_compat2(&sec, &tmv);

  char* p = out; size_t rem = outlen; *p=0;
  const char* s = layout;
  while (*s && rem>1){
    if (strncmp(s,"Monday",6)==0){ append(&p,&rem, WDAYS_FULL[tmv.tm_wday]); s+=6; continue; }
    if (strncmp(s,"Mon",3)==0){ append(&p,&rem, WDAYS_ABR[tmv.tm_wday]); s+=3; continue; }
    if (strncmp(s,"January",7)==0){ append(&p,&rem, MONTHS_FULL[tmv.tm_mon]); s+=7; continue; }
    if (strncmp(s,"Jan",3)==0){ append(&p,&rem, MONTHS_ABR[tmv.tm_mon]); s+=3; continue; }
    if (strncmp(s,"2006",4)==0){ append_int_w(&p,&rem, tmv.tm_year+1900, 4); s+=4; continue; }
    if (strncmp(s,"06",2)==0){ append_int_w(&p,&rem, (tmv.tm_year+1900)%100, 2); s+=2; continue; }
    if (strncmp(s,"01",2)==0){ append_int_w(&p,&rem, tmv.tm_mon+1, 2); s+=2; continue; }
    if (strncmp(s,"1",1)==0){ append_int_no(&p,&rem, tmv.tm_mon+1); s+=1; continue; }
    if (strncmp(s,"02",2)==0){ append_int_w(&p,&rem, tmv.tm_mday, 2); s+=2; continue; }
    if (strncmp(s,"_2",2)==0){ char buf[3]; snprintf(buf,sizeof(buf), "%2d", tmv.tm_mday); append(&p,&rem, buf); s+=2; continue; }
    if (strncmp(s,"2",1)==0){ append_int_no(&p,&rem, tmv.tm_mday); s+=1; continue; }
    if (strncmp(s,"002",3)==0){ /* day of year */ int yday = tmv.tm_yday+1; char buf[8]; snprintf(buf,sizeof(buf), "%03d", yday); append(&p,&rem, buf); s+=3; continue; }
    if (strncmp(s,"15",2)==0){ append_int_w(&p,&rem, tmv.tm_hour, 2); s+=2; continue; }
    if (strncmp(s,"03",2)==0){ int h=(tmv.tm_hour%12); if(h==0)h=12; append_int_w(&p,&rem,h,2); s+=2; continue; }
    if (strncmp(s,"3",1)==0){ int h=(tmv.tm_hour%12); if(h==0)h=12; append_int_no(&p,&rem,h); s+=1; continue; }
    if (strncmp(s,"04",2)==0){ append_int_w(&p,&rem, tmv.tm_min, 2); s+=2; continue; }
    if (strncmp(s,"4",1)==0){ append_int_no(&p,&rem, tmv.tm_min); s+=1; continue; }
    if (strncmp(s,"05",2)==0){ append_int_w(&p,&rem, tmv.tm_sec, 2); s+=2; continue; }
    if (strncmp(s,"5",1)==0){ append_int_no(&p,&rem, tmv.tm_sec); s+=1; continue; }
    if (strncmp(s,"PM",2)==0){ append(&p,&rem, tmv.tm_hour>=12?"PM":"AM"); s+=2; continue; }
    if (strncmp(s,"pm",2)==0){ append(&p,&rem, tmv.tm_hour>=12?"pm":"am"); s+=2; continue; }
    if (strncmp(s,".000000000",10)==0){ append(&p,&rem,"."); append_frac(&p,&rem, utc.nsec, 9); s+=10; continue; }
    if (strncmp(s,".000000",7)==0){ append(&p,&rem,"."); append_frac(&p,&rem, utc.nsec, 6); s+=7; continue; }
    if (strncmp(s,".000",4)==0){ append(&p,&rem,"."); append_frac(&p,&rem, utc.nsec, 3); s+=4; continue; }
    if (strncmp(s,"Z07:00",6)==0){ append_tz(&p,&rem, off, 1, 1); s+=6; continue; }
    if (strncmp(s,"-07:00",6)==0){ append_tz(&p,&rem, off, 1, 0); s+=6; continue; }
    if (strncmp(s,"-0700",5)==0){ append_tz(&p,&rem, off, 0, 0); s+=5; continue; }
    if (strncmp(s,"MST",3)==0){ append(&p,&rem, tz? (const char*) (abbr?abbr:"UTC") : "UTC"); s+=3; continue; }
    /* copy literal */
    *p++ = *s++; rem--;
  }
  *p = '\0';
  return 0;
}

static int read_int_n(const char** sp, int min_d, int max_d, int* out){
  const char* s=*sp; int d=0; int v=0;
  while (*s>='0'&&*s<='9' && d<max_d){ v = v*10 + (*s-'0'); s++; d++; }
  if (d<min_d) return -1;
  *out = v; *sp = s; return 0;
}
static int match_word_ci(const char** sp, const char* const words[], int n, int* idx){
  const char* s=*sp;
  for (int i=0;i<n;i++){
    size_t len=strlen(words[i]); int ok=1;
    for (size_t j=0;j<len;j++){
      char c1 = s[j], c2 = words[i][j];
      if (tolower((unsigned char)c1)!=tolower((unsigned char)c2)){ ok=0; break; }
    }
    if (ok){ *idx=i; *sp = s+len; return 0; }
  }
  return -1;
}
static int parse_frac(const char** sp, int* nsec){
  const char* s=*sp; if (*s!='.') return 0;
  s++;
  int d=0; int n=0;
  while (*s>='0'&&*s<='9' && d<9){ n = n*10 + (*s-'0'); s++; d++; }
  while (d<9){ n*=10; d++; }
  *nsec = n; *sp = s; return 0;
}
static int parse_tz(const char** sp, int* offset_sec){
  const char* s=*sp;
  if (*s=='Z'){ *offset_sec=0; *sp=s+1; return 0; }
  int sign = (*s=='-')?-1:(*s=='+')?+1:0; if (!sign) return -1; s++;
  int hh=0,mm=0; if (read_int_n(&s,2,2,&hh)!=0) return -1;
  if (*s==':') s++;
  if (read_int_n(&s,2,2,&mm)!=0) return -1;
  *offset_sec = sign*(hh*3600 + mm*60);
  *sp = s; return 0;
}

static int yday_to_monthday(int year, int yday, int* out_m, int* out_d){
  static const int mdays[12]={31,28,31,30,31,30,31,31,30,31,30,31};
  int leap = ( (year%4==0 && year%100!=0) || (year%400==0) );
  int dleft = yday;
  for (int m=0;m<12;m++){
    int d = mdays[m] + (m==1 && leap);
    if (dleft <= d){ *out_m = m+1; *out_d = dleft; return 0; }
    dleft -= d;
  }
  return -1;
}

int vt_parse_go(const char* layout, const char* value, const VexTz* tz, VexInstant* out){
  int Y=0,M=1,D=1,h=0,mn=0,sc=0, nsec=0;
  int has_zone=0, zone_ofs=0; int have_yday=0, yday=0; int have_pm=0, pm=0;
  const char *L=layout, *V=value;
  while (*L && *V){
    if (strncmp(L,"2006",4)==0){ if (read_int_n(&V,4,4,&Y)!=0) return -1; L+=4; continue; }
    if (strncmp(L,"06",2)==0){ if (read_int_n(&V,2,2,&Y)!=0) return -1; Y += (Y>=69?1900:2000); L+=2; continue; }
    if (strncmp(L,"January",7)==0){ int idx; if (match_word_ci(&V, MONTHS_FULL, 12, &idx)!=0) return -1; M=idx+1; L+=7; continue; }
    if (strncmp(L,"Jan",3)==0){ int idx; if (match_word_ci(&V, MONTHS_ABR, 12, &idx)!=0) return -1; M=idx+1; L+=3; continue; }
    if (strncmp(L,"01",2)==0){ if (read_int_n(&V,2,2,&M)!=0) return -1; L+=2; continue; }
    if (strncmp(L,"1",1)==0){ if (read_int_n(&V,1,2,&M)!=0) return -1; L+=1; continue; }
    if (strncmp(L,"002",3)==0){ if (read_int_n(&V,3,3,&yday)!=0) return -1; have_yday=1; L+=3; continue; }
    if (strncmp(L,"02",2)==0){ if (read_int_n(&V,2,2,&D)!=0) return -1; L+=2; continue; }
    if (strncmp(L,"_2",2)==0){ while (*V==' ') V++; if (read_int_n(&V,1,2,&D)!=0) return -1; L+=2; continue; }
    if (strncmp(L,"2",1)==0){ if (read_int_n(&V,1,2,&D)!=0) return -1; L+=1; continue; }
    if (strncmp(L,"Monday",6)==0){ int idx; if (match_word_ci(&V, WDAYS_FULL, 7, &idx)!=0) return -1; L+=6; continue; }
    if (strncmp(L,"Mon",3)==0){ int idx; if (match_word_ci(&V, WDAYS_ABR, 7, &idx)!=0) return -1; L+=3; continue; }
    if (strncmp(L,"15",2)==0){ if (read_int_n(&V,2,2,&h)!=0) return -1; L+=2; continue; }
    if (strncmp(L,"03",2)==0 || strncmp(L,"3",1)==0){
      int hh=0; if (read_int_n(&V,(L[1]=='3')?2:1,2,&hh)!=0) return -1; h = hh%12; L += (L[0]=='0'&&L[1]=='3')?2:1; continue;
    }
    if (strncmp(L,"PM",2)==0 || strncmp(L,"pm",2)==0){
      have_pm=1; pm = (V[0]=='P'||V[0]=='p'); V+=2; L+=2; continue;
    }
    if (strncmp(L,"04",2)==0 || strncmp(L,"4",1)==0){ if (read_int_n(&V,(L[1]=='4')?2:1,2,&mn)!=0) return -1; L+=(L[0]=='0'&&L[1]=='4')?2:1; continue; }
    if (strncmp(L,"05",2)==0 || strncmp(L,"5",1)==0){ if (read_int_n(&V,(L[1]=='5')?2:1,2,&sc)!=0) return -1; L+=(L[0]=='0'&&L[1]=='5')?2:1; continue; }
    if (strncmp(L,".000000000",10)==0 || strncmp(L,".000000",7)==0 || strncmp(L,".000",4)==0){
      if (*V!='.') return -1; parse_frac(&V,&nsec);
      L += (L[1]=='0' && L[2]=='0' && L[3]=='0' && L[4]=='0' && L[5]=='0' && L[6]=='0' && L[7]=='0' && L[8]=='0' && L[9]=='0')?10:
           (L[1]=='0' && L[2]=='0' && L[3]=='0' && L[4]=='0' && L[5]=='0' && L[6]=='0')?7:4;
      continue;
    }
    if (strncmp(L,"Z07:00",6)==0 || strncmp(L,"-07:00",6)==0 || strncmp(L,"-0700",5)==0){
      if (parse_tz(&V,&zone_ofs)!=0) return -1; has_zone=1; L += (L[0]=='Z')?6:(L[4]==':')?6:5; continue;
    }
    if (strncmp(L,"MST",3)==0){
      /* accept zone abbreviation: read letters 1..5 (ignore for offset; tz arg applies) */
      int n=0; while (isalpha((unsigned char)*V) && n<5){ V++; n++; }
      if (n==0) return -1;
      L+=3; continue;
    }
    /* literal match */
    if (*L == *V){ L++; V++; continue; }
    return -1;
  }
  if (*L) return -1;
  if (have_pm){ if (pm && h<12) h+=12; else if (!pm && h==12) h=0; }
  if (have_yday){ if (Y==0) return -1; if (yday<=0) return -1; int m=0,d=0; if (yday_to_monthday(Y,yday, &m,&d)!=0) return -1; M=m; D=d; }
  struct tm tmv; memset(&tmv,0,sizeof(tmv));
  tmv.tm_year = Y-1900; tmv.tm_mon = M-1; tmv.tm_mday=D; tmv.tm_hour=h; tmv.tm_min=mn; tmv.tm_sec=sc;
  time_t base = timegm_compat(&tmv); if (base==(time_t)-1) return -1;
  int ofs = 0;
  if (has_zone) ofs = zone_ofs;
  else { if (tz){ vt_tz_offset_at(tz, vt_instant_from_unix(base,0), &ofs, NULL); } else ofs = 0; }
  int64_t epoch = (int64_t)base - ofs;
  out->unix_sec = epoch; out->nsec = nsec; out->_pad=0;
  return 0;
}

/* ==== Component extraction (fast_date_from_epoch usage) ==== */
/* External function declared in fast_parse.h */
extern void fast_date_from_epoch(int64_t unix_sec, int* year, int* month, int* day, 
                                  int* hour, int* minute, int* second, int* weekday);

void vt_instant_date(VexInstant t, int* year, int* month, int* day, int* hour, int* minute, int* second, int* nsec) {
  int weekday_unused;
  fast_date_from_epoch(t.unix_sec, year, month, day, hour, minute, second, &weekday_unused);
  if (nsec) *nsec = t.nsec;
}

void vt_instant_clock(VexInstant t, int* hour, int* minute, int* second) {
  int y, m, d, w;
  fast_date_from_epoch(t.unix_sec, &y, &m, &d, hour, minute, second, &w);
}

int vt_instant_yearday(VexInstant t) {
  int year, month, day, hour, minute, second, weekday;
  fast_date_from_epoch(t.unix_sec, &year, &month, &day, &hour, &minute, &second, &weekday);
  
  /* Days in each month for non-leap year */
  static const int days_in_month[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
  
  /* Check if leap year */
  int is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
  
  /* Calculate day of year */
  int yday = day;
  for (int i = 0; i < month - 1; i++) {
    yday += days_in_month[i];
    if (i == 1 && is_leap) yday++;  /* Add leap day after February */
  }
  return yday;
}

int vt_instant_weekday(VexInstant t) {
  int year, month, day, hour, minute, second, weekday;
  fast_date_from_epoch(t.unix_sec, &year, &month, &day, &hour, &minute, &second, &weekday);
  return weekday;  /* 0=Sunday, 6=Saturday */
}

int vt_instant_isoweek(VexInstant t, int* iso_year) {
  int year, month, day, hour, minute, second, weekday;
  int dummy_y, dummy_m, dummy_d, dummy_h, dummy_min, dummy_s;
  fast_date_from_epoch(t.unix_sec, &year, &month, &day, &hour, &minute, &second, &weekday);
  
  /* ISO 8601 week date: week starts on Monday (1), ends on Sunday (7) */
  /* Thursday is day 4, so weeks are determined by Thursday's date */
  
  /* Convert weekday: 0=Sunday -> 7, 1=Monday -> 1, ..., 6=Saturday -> 6 */
  int iso_weekday = (weekday == 0) ? 7 : weekday;
  
  /* Find January 4 of this year (which is always in week 1) */
  struct tm tm_jan4;
  memset(&tm_jan4, 0, sizeof(tm_jan4));
  tm_jan4.tm_year = year - 1900;
  tm_jan4.tm_mon = 0;
  tm_jan4.tm_mday = 4;
  time_t t_jan4 = timegm_compat(&tm_jan4);
  
  int y_jan4, m_jan4, d_jan4, h_jan4, min_jan4, s_jan4, wd_jan4;
  fast_date_from_epoch(t_jan4, &y_jan4, &m_jan4, &d_jan4, &h_jan4, &min_jan4, &s_jan4, &wd_jan4);
  
  int jan4_iso_weekday = (wd_jan4 == 0) ? 7 : wd_jan4;
  
  /* Find Monday of week containing January 4 */
  time_t week1_monday = t_jan4 - (int64_t)(jan4_iso_weekday - 1) * 86400;
  
  /* If this date is before week 1 Monday, it belongs to previous year's last week */
  if (t.unix_sec < week1_monday) {
    /* Date is in previous year's last week */
    if (iso_year) *iso_year = year - 1;
    
    /* Find week 1 of previous year */
    struct tm tm_prev_jan4;
    memset(&tm_prev_jan4, 0, sizeof(tm_prev_jan4));
    tm_prev_jan4.tm_year = (year - 1) - 1900;
    tm_prev_jan4.tm_mon = 0;
    tm_prev_jan4.tm_mday = 4;
    time_t t_prev_jan4 = timegm_compat(&tm_prev_jan4);
    
    int wd_prev_jan4;
    fast_date_from_epoch(t_prev_jan4, &dummy_y, &dummy_m, &dummy_d, &dummy_h, &dummy_min, &dummy_s, &wd_prev_jan4);
    
    int prev_jan4_iso_weekday = (wd_prev_jan4 == 0) ? 7 : wd_prev_jan4;
    time_t prev_week1_monday = t_prev_jan4 - (int64_t)(prev_jan4_iso_weekday - 1) * 86400;
    
    return 1 + (int)((t.unix_sec - prev_week1_monday) / 604800);
  }
  
  /* Find next year's week 1 Monday to check if we're in the first week of next year */
  struct tm tm_next_jan4;
  memset(&tm_next_jan4, 0, sizeof(tm_next_jan4));
  tm_next_jan4.tm_year = (year + 1) - 1900;
  tm_next_jan4.tm_mon = 0;
  tm_next_jan4.tm_mday = 4;
  time_t t_next_jan4 = timegm_compat(&tm_next_jan4);
  
  int wd_next_jan4;
  fast_date_from_epoch(t_next_jan4, &dummy_y, &dummy_m, &dummy_d, &dummy_h, &dummy_min, &dummy_s, &wd_next_jan4);
  
  int next_jan4_iso_weekday = (wd_next_jan4 == 0) ? 7 : wd_next_jan4;
  time_t next_week1_monday = t_next_jan4 - (int64_t)(next_jan4_iso_weekday - 1) * 86400;
  
  if (t.unix_sec >= next_week1_monday) {
    if (iso_year) *iso_year = year + 1;
    return 1;
  }
  
  if (iso_year) *iso_year = year;
  return 1 + (int)((t.unix_sec - week1_monday) / 604800);
}

/* ==== Comparison operators ==== */
int vt_instant_compare(VexInstant a, VexInstant b) {
  if (a.unix_sec < b.unix_sec) return -1;
  if (a.unix_sec > b.unix_sec) return 1;
  if (a.nsec < b.nsec) return -1;
  if (a.nsec > b.nsec) return 1;
  return 0;
}

int vt_instant_equal(VexInstant a, VexInstant b) {
  return (a.unix_sec == b.unix_sec && a.nsec == b.nsec) ? 1 : 0;
}

int vt_instant_before(VexInstant a, VexInstant b) {
  return (vt_instant_compare(a, b) < 0) ? 1 : 0;
}

int vt_instant_after(VexInstant a, VexInstant b) {
  return (vt_instant_compare(a, b) > 0) ? 1 : 0;
}

/* ==== Time truncation and rounding ==== */
VexInstant vt_instant_truncate(VexInstant t, VexDuration d) {
  if (d <= 0) return t;
  
  int64_t sec_and_ns = (t.unix_sec * 1000000000LL) + (int64_t)t.nsec;
  int64_t truncated = (sec_and_ns / d) * d;
  
  VexInstant result;
  result.unix_sec = truncated / 1000000000LL;
  result.nsec = (int32_t)(truncated % 1000000000LL);
  result._pad = 0;
  return result;
}

VexInstant vt_instant_round(VexInstant t, VexDuration d) {
  if (d <= 0) return t;
  
  int64_t sec_and_ns = (t.unix_sec * 1000000000LL) + (int64_t)t.nsec;
  int64_t rounded = ((sec_and_ns + d/2) / d) * d;
  
  VexInstant result;
  result.unix_sec = rounded / 1000000000LL;
  result.nsec = (int32_t)(rounded % 1000000000LL);
  result._pad = 0;
  return result;
}

/* ==== Unix timestamp variants ==== */
int64_t vt_instant_unix_milli(VexInstant t) {
  return (t.unix_sec * 1000LL) + (t.nsec / 1000000);
}

int64_t vt_instant_unix_micro(VexInstant t) {
  return (t.unix_sec * 1000000LL) + (t.nsec / 1000);
}

