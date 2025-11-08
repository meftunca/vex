#include "vex_time.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <math.h>

/* ---- Conversions ---- */
VexInstant vt_instant_from_unix(int64_t sec, int32_t nsec){
  VexInstant t; t.unix_sec = sec; t.nsec = nsec; t._pad = 0; return t;
}
void vt_instant_to_unix(VexInstant t, int64_t* sec, int32_t* nsec){
  if (sec) *sec = t.unix_sec; if (nsec) *nsec = t.nsec;
}

/* ---- Duration parse/format ---- */
static int unit_scale_ns(char u){
  switch (u){
    case 'h': return 3600*1000000000LL;
    case 'm': return 60*1000000000LL;
    case 's': return 1000000000LL;
    default: return 0;
  }
}
/* Accepts chains like 1h2m3.5s, 250ms, -1.25h, 500us, 10ns */
int vt_parse_duration(const char* s, VexDuration* out_ns){
  if (!s || !out_ns) return -1;
  const char* p = s;
  int neg = 0; if (*p=='+' || *p=='-'){ neg = (*p=='-'); p++; }
  long double total = 0.0L;
  while (*p){
    char* end=NULL;
    long double v = strtold(p, &end);
    if (end==p) return -1;
    p = end;
    /* read unit token: ns, us/µs, ms, s, m, h */
    if (strncmp(p,"ns",2)==0){ total += v; p+=2; }
    else if (strncmp(p,"us",2)==0 || strncmp(p,"\xC2\xB5""s",3)==0){ total += v*1000.0L; p += (p[0]=='u'?2:3); }
    else if (strncmp(p,"µs",2)==0){ total += v*1000.0L; p+=2; }
    else if (strncmp(p,"ms",2)==0){ total += v*1000000.0L; p+=2; }
    else if (*p=='s'){ total += v*1000000000.0L; p+=1; }
    else if (*p=='m'){ total += v*60.0L*1000000000.0L; p+=1; }
    else if (*p=='h'){ total += v*3600.0L*1000000000.0L; p+=1; }
    else return -1;
  }
  if (neg) total = -total;
  /* clamp to int64 */
  long double minv = -(9.22e18L); long double maxv = +(9.22e18L);
  if (total<minv) total=minv; if (total>maxv) total=maxv;
  *out_ns = (VexDuration)(total);
  return 0;
}

int vt_format_duration(VexDuration ns, char* buf, size_t buflen){
  if (!buf || buflen<2) return -1;
  if (ns==0){ snprintf(buf, buflen, "0s"); return 0; }
  char sign = (ns<0)?'-':'+'; if (ns<0) ns = -ns;
  long long hours = (long long)(ns / 1000000000LL / 3600LL);
  long long rem = (long long)(ns - hours*3600LL*1000000000LL);
  long long mins = rem / 1000000000LL / 60LL; rem -= mins*60LL*1000000000LL;
  long long secs = rem / 1000000000LL; rem -= secs*1000000000LL;
  long long ms = rem / 1000000LL; rem -= ms*1000000LL;
  long long us = rem / 1000LL; rem -= us*1000LL;
  long long nss = rem;
  /* produce compact */
  if (hours>0) snprintf(buf, buflen, "%c%lldh%lldm%llds", sign, hours, mins, secs);
  else if (mins>0) snprintf(buf, buflen, "%c%lldm%lld.%03llds", sign, mins, secs, ms);
  else if (secs>0) snprintf(buf, buflen, "%c%lld.%03llds", sign, secs, ms);
  else if (ms>0)   snprintf(buf, buflen, "%c%lldms", sign, ms);
  else if (us>0)   snprintf(buf, buflen, "%c%lldus", sign, us);
  else             snprintf(buf, buflen, "%cllldns", sign, (long long)nss);
  return 0;
}

/* ---- Arithmetic ---- */
VexTime vt_add(VexTime t, VexDuration d){
  VexTime r = t;
  int64_t sec = r.wall.unix_sec;
  int64_t nsec = (int64_t)r.wall.nsec + d;
  sec += nsec / 1000000000LL;
  nsec %= 1000000000LL; if (nsec<0){ nsec += 1000000000LL; sec -= 1; }
  r.wall.unix_sec = sec; r.wall.nsec = (int32_t)nsec;
  r.mono_ns += (d>=0)?(uint64_t)d:(uint64_t)(-d); /* monotonic isn't signed, but relative delta is fine */
  return r;
}
VexDuration vt_sub(VexTime t, VexTime u){
  /* prefer monotonic if both carry it */
  if (t.mono_ns && u.mono_ns){
    int64_t diff = (int64_t)(t.mono_ns - u.mono_ns);
    return diff;
  }
  /* fallback to wall */
  int64_t ds = t.wall.unix_sec - u.wall.unix_sec;
  int64_t dns = (int64_t)t.wall.nsec - (int64_t)u.wall.nsec;
  return ds*1000000000LL + dns;
}
VexDuration vt_since(VexTime t){
  VexTime now; vt_now(&now); return vt_sub(now, t);
}
VexDuration vt_until(VexTime t){
  VexTime now; vt_now(&now); return vt_sub(t, now);
}

/* ---- RFC3339 (UTC default) ---- */
static int two(const char* s){ return (s[0]>='0'&&s[0]<='9'&&s[1]>='0'&&s[1]<='9'); }
int vt_format_rfc3339_utc(VexInstant t, char* buf, size_t buflen){
  /* format: YYYY-MM-DDTHH:MM:SS[.nnnnnnnnn]Z */
  if (!buf) return -1;
  /* naive UTC via gmtime */
  time_t sec = (time_t)t.unix_sec;
#if defined(_WIN32)
  struct tm tmv; gmtime_s(&tmv, &sec);
#else
  struct tm tmv; gmtime_r(&sec, &tmv);
#endif
  if (t.nsec) snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02d.%09dZ",
    tmv.tm_year+1900, tmv.tm_mon+1, tmv.tm_mday, tmv.tm_hour, tmv.tm_min, tmv.tm_sec, t.nsec);
  else snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02dZ",
    tmv.tm_year+1900, tmv.tm_mon+1, tmv.tm_mday, tmv.tm_hour, tmv.tm_min, tmv.tm_sec);
  return 0;
}

int vt_parse_rfc3339(const char* s, VexInstant* out){
  if (!s || !out) return -1;
  int Y,M,D,h,m,sec; Y=M=D=h=m=sec=0; int nsec=0;
  /* minimal parser: YYYY-MM-DDTHH:MM:SS(.frac)?(Z|±HH:MM) */
  if (!(strlen(s)>=20)) return -1;
  if (sscanf(s,"%4d-%2d-%2dT%2d:%2d:%2d",&Y,&M,&D,&h,&m,&sec)!=6) return -1;
  const char* p = s + 19;
  if (*p=='.'){
    p++;
    int digits=0; nsec=0;
    while (*p>='0' && *p<='9' && digits<9){ nsec = nsec*10 + (*p-'0'); digits++; p++; }
    while (digits<9){ nsec *= 10; digits++; }
  }
  int tzsign=0, tzh=0, tzm=0;
  if (*p=='Z'){ tzsign=0; p++; }
  else if (*p=='+' || *p=='-'){ tzsign = (*p=='-')?-1:1; p++;
    if (!(two(p) && p[2]==':' && two(p+3))) return -1;
    tzh = (p[0]-'0')*10 + (p[1]-'0'); tzm = (p[3]-'0')*10 + (p[4]-'0'); p+=5;
  } else return -1;

  /* to epoch seconds (UTC) — use naive civil time conv */
  struct tm tmb; memset(&tmb,0,sizeof(tmb));
  tmb.tm_year = Y-1900; tmb.tm_mon = M-1; tmb.tm_mday = D;
  tmb.tm_hour = h; tmb.tm_min = m; tmb.tm_sec = sec;
#if defined(_WIN32)
  time_t loc = _mkgmtime(&tmb);
#else
  time_t loc = timegm(&tmb);
#endif
  if (loc==(time_t)-1) return -1;
  int64_t epoch = (int64_t)loc;
  int tzofs = tzsign * (tzh*3600 + tzm*60);
  epoch -= tzofs;
  out->unix_sec = epoch; out->nsec = nsec; out->_pad=0;
  return 0;
}
