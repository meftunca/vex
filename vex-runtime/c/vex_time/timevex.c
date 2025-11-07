// timevex.c - see header for license/notes
#define _POSIX_C_SOURCE 200809L
#include "timevex.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <pthread.h>
#include <sys/time.h>
#include <unistd.h>
#include <math.h>

// ------------------------- Internal helpers -------------------------

static pthread_mutex_t tz_mutex = PTHREAD_MUTEX_INITIALIZER;

static int64_t clamp_i64(int64_t v, int64_t lo, int64_t hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

static int64_t ns_add(int64_t sec, int64_t nsec) {
    return sec*1000000000LL + nsec;
}

static int is_leap(int y) {
    return (y%4==0 && y%100!=0) || (y%400==0);
}

static int days_in_month(int y, int m) {
    static const int dm[12] = {31,28,31,30,31,30,31,31,30,31,30,31};
    if (m==2) return 28 + is_leap(y);
    return dm[m-1];
}

// timegm portable fallback
static time_t timegm_portable(struct tm *tm) {
#if defined(_BSD_SOURCE) || defined(_GNU_SOURCE) || defined(__USE_MISC)
    return timegm(tm);
#else
    // Save TZ
    char *old_tz = getenv("TZ");
    pthread_mutex_lock(&tz_mutex);
    setenv("TZ", "UTC", 1); tzset();
    time_t ret = mktime(tm);
    if (old_tz) setenv("TZ", old_tz, 1); else unsetenv("TZ");
    tzset();
    pthread_mutex_unlock(&tz_mutex);
    return ret;
#endif
}

// Set TZ temporarily to name; if name==NULL restore.
static void tz_swap(const char *name, char *prev, size_t prevsz) {
    pthread_mutex_lock(&tz_mutex);
    const char *cur = getenv("TZ");
    if (cur) snprintf(prev, prevsz, "%s", cur);
    else prev[0] = 0;
    if (name) setenv("TZ", name, 1); else {
        if (prev[0]) setenv("TZ", prev, 1);
        else unsetenv("TZ");
    }
    tzset();
    pthread_mutex_unlock(&tz_mutex);
}

// ----------------------------- Errors --------------------------------

enum { TVX_OK=0, TVX_EPARSE=1, TVX_ERANGE=2 };

const char* tvx_StrError(int code) {
    switch(code){
        case 0: return "ok";
        case 1: return "parse error";
        case 2: return "out of range";
        default: return "error";
    }
}

const char* tvx_LastOSError(void) {
    static __thread char buf[256];
#if defined(__GLIBC__) && !defined(_GNU_SOURCE)
#define _GNU_SOURCE
#endif
    strerror_r(errno, buf, sizeof(buf));
    return buf;
}

// --------------------------- Locations -------------------------------

tvx_location tvx_UTC(void){ tvx_location l; snprintf(l.name, sizeof(l.name), "UTC"); return l; }
tvx_location tvx_Local(void){ tvx_location l; snprintf(l.name, sizeof(l.name), "Local"); return l; }

tvx_location tvx_FixedZone(const char *name, int32_t offset_seconds) {
    tvx_location l;
    if (!name) name = "Fixed";
    int sign = offset_seconds>=0 ? 1 : -1;
    int32_t a = offset_seconds*sign;
    int hh = a/3600, mm = (a%3600)/60;
    snprintf(l.name, sizeof(l.name), "%s%+03d:%02d", name, sign*hh, mm);
    return l;
}

int tvx_LoadLocation(const char *iana, tvx_location *out){
    if (!iana || !out) return TVX_EPARSE;
    // We don't validate against system tzdb here; defer to OS.
    snprintf(out->name, sizeof(out->name), "%s", iana);
    return TVX_OK;
}

// ------------------------------ Time ---------------------------------

static int64_t monotonic_now_ns(void){
#if defined(CLOCK_MONOTONIC)
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return ns_add(ts.tv_sec, ts.tv_nsec);
#else
    struct timeval tv; gettimeofday(&tv, NULL);
    return ns_add(tv.tv_sec, (int64_t)tv.tv_usec*1000);
#endif
}

int64_t tvx_MonotonicNow(void){ return monotonic_now_ns(); }

static tvx_time make_time(time_t sec, int32_t nsec, tvx_location loc) {
    tvx_time t;
    t.unix_sec = (int64_t)sec;
    t.nsec = (int32_t)clamp_i64(nsec, 0, 999999999);
    t.mono_ns = monotonic_now_ns();
    t.loc = loc;
    return t;
}

tvx_time tvx_Now(void){
    struct timespec ts;
#if defined(CLOCK_REALTIME)
    clock_gettime(CLOCK_REALTIME, &ts);
#else
    struct timeval tv; gettimeofday(&tv,NULL); ts.tv_sec=tv.tv_sec; ts.tv_nsec=tv.tv_usec*1000;
#endif
    return make_time(ts.tv_sec, ts.tv_nsec, tvx_UTC());
}

tvx_time tvx_NowIn(tvx_location loc){
    tvx_time t = tvx_Now(); t.loc = loc; return t;
}

tvx_time tvx_Unix(int64_t sec, int64_t nsec, tvx_location loc){
    int64_t s = sec + nsec/1000000000LL;
    int64_t ns = nsec % 1000000000LL; if (ns<0){ ns+=1000000000LL; s -= 1; }
    return make_time((time_t)s, (int32_t)ns, loc);
}

int64_t tvx_UnixSeconds(tvx_time t){ return t.unix_sec; }
int64_t tvx_UnixNano(tvx_time t){ return ns_add(t.unix_sec, t.nsec); }

tvx_time tvx_In(tvx_time t, tvx_location loc){ t.loc = loc; return t; }
tvx_time tvx_UTCTo(tvx_time t, tvx_location loc){ (void)loc; return t; /* same epoch; loc only affects presentation */ }

int tvx_Before(tvx_time a, tvx_time b){
    if (a.unix_sec != b.unix_sec) return a.unix_sec < b.unix_sec;
    return a.nsec < b.nsec;
}
int tvx_After(tvx_time a, tvx_time b){ return tvx_Before(b,a); }
int tvx_Equal(tvx_time a, tvx_time b){ return a.unix_sec==b.unix_sec && a.nsec==b.nsec; }

tvx_time tvx_Add(tvx_time t, tvx_duration d){
    int64_t ns = t.nsec + (d % 1000000000LL);
    int64_t carry = ns / 1000000000LL; if (ns<0){ ns += 1000000000LL; carry -= 1; }
    int64_t sec = t.unix_sec + (d / 1000000000LL) + carry;
    return tvx_Unix(sec, ns, t.loc);
}

tvx_duration tvx_Sub(tvx_time a, tvx_time b){
    return (a.unix_sec - b.unix_sec)*1000000000LL + (a.nsec - b.nsec);
}

tvx_duration tvx_Since(tvx_time t){ return tvx_Sub(tvx_Now(), t); }
tvx_duration tvx_Until(tvx_time t){ return tvx_Sub(t, tvx_Now()); }

// Calendar extraction in a given location
static void to_tm(tvx_time t, struct tm *out){
    time_t s = (time_t)t.unix_sec;
    if (strcmp(t.loc.name,"UTC")==0) {
        gmtime_r(&s, out);
    } else if (strcmp(t.loc.name,"Local")==0) {
        localtime_r(&s, out);
    } else {
        char prev[256]={0};
        tz_swap(t.loc.name, prev, sizeof(prev));
        localtime_r(&s, out);
        tz_swap(NULL, prev, sizeof(prev));
    }
}

int tvx_Year(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_year+1900; }
int tvx_Month(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_mon+1; }
int tvx_Day(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_mday; }
int tvx_Hour(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_hour; }
int tvx_Minute(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_min; }
int tvx_Second(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_sec; }
int tvx_Nanosecond(tvx_time t){ return t.nsec; }
tvx_weekday tvx_Weekday(tvx_time t){ struct tm tm; to_tm(t,&tm); return (tvx_weekday)tm.tm_wday; }
int tvx_YearDay(tvx_time t){ struct tm tm; to_tm(t,&tm); return tm.tm_yday+1; }

void tvx_ISOWeek(tvx_time t, int *y, int *w){
    struct tm tm; to_tm(t,&tm);
    // ISO week algo
    int yy = tm.tm_year + 1900;
    int doy = tm.tm_yday + 1;
    int dow = (tm.tm_wday==0)?7:tm.tm_wday; // Mon=1..Sun=7
    int thursday = doy + (4 - dow);
    int iso_week = (thursday + 6)/7;
    int iso_year = yy;
    if (iso_week == 0){
        iso_year = yy - 1;
        iso_week = 52 + ((is_leap(iso_year) && (tm.tm_wday==5 || (tm.tm_wday==6 && is_leap(yy))))?1:0);
    } else {
        int weeks_in_year = 52 + ((is_leap(yy) && (tm.tm_wday==4 || tm.tm_wday==3))?1:0);
        if (iso_week > weeks_in_year){ iso_week = 1; iso_year = yy + 1; }
    }
    if (y) *y = iso_year; if (w) *w = iso_week;
}

// AddDate similar to Go: normalize via tm then back to epoch
tvx_time tvx_AddDate(tvx_time t, int years, int months, int days){
    struct tm tm; to_tm(t,&tm);
    tm.tm_year += years;
    int m = tm.tm_mon + months;
    tm.tm_year += m/12;
    tm.tm_mon = m%12; if (tm.tm_mon<0){ tm.tm_mon += 12; tm.tm_year -= 1; }
    tm.tm_mday += days;
    // normalize using mktime/timegm depending on location
    time_t s;
    if (strcmp(t.loc.name,"UTC")==0){
        s = timegm_portable(&tm);
    } else if (strcmp(t.loc.name,"Local")==0){
        s = mktime(&tm);
    } else {
        char prev[256]={0};
        tz_swap(t.loc.name, prev, sizeof(prev));
        s = mktime(&tm);
        tz_swap(NULL, prev, sizeof(prev));
    }
    return tvx_Unix((int64_t)s, t.nsec, t.loc);
}

// Truncate/Round on absolute ns since epoch
tvx_time tvx_Truncate(tvx_time t, tvx_duration d){
    if (d<=0) return t;
    int64_t ns = tvx_UnixNano(t);
    int64_t rem = ns % d; if (rem<0) rem += d;
    return tvx_Unix((ns - rem)/1000000000LL, (ns - rem)%1000000000LL, t.loc);
}
tvx_time tvx_Round(tvx_time t, tvx_duration d){
    if (d<=0) return t;
    int64_t ns = tvx_UnixNano(t);
    int64_t rem = ns % d; if (rem<0) rem += d;
    int64_t half = d/2;
    int64_t adj = (rem < half) ? -rem : (d - rem);
    int64_t n = ns + adj;
    return tvx_Unix(n/1000000000LL, n%1000000000LL, t.loc);
}

// ---------------------- Duration parse/format ------------------------

static int64_t parse_int(const char **ps){
    const char *s = *ps; int64_t v=0; int any=0;
    while (*s>='0' && *s<='9'){ v = v*10 + (*s - '0'); s++; any=1; }
    *ps = s; return any? v : -1;
}

int tvx_ParseDuration(const char *s, tvx_duration *out){
    if (!s||!out) return TVX_EPARSE;
    int neg=0; if (*s=='-'){ neg=1; s++; } else if (*s=='+'){ s++; }
    long double total = 0.0L;
    int any=0;
    while (*s){
        // number (int or float)
        const char *start = s;
        int64_t iv = parse_int(&s);
        long double val = 0.0L;
        if (iv>=0){ val = (long double)iv;
            if (*s=='.'){
                s++;
                const char *fstart = s;
                int64_t frac = 0; int scale=1, fdigits=0;
                while (*s>='0' && *s<='9'){ frac = frac*10 + (*s - '0'); s++; fdigits++; scale *= 10; if (fdigits>9) break; }
                val += ((long double)frac)/((long double)scale);
            }
        } else return TVX_EPARSE;
        // unit
        int matched=1;
        if (strncmp(s,"ns",2)==0){ total += val; s+=2; }
        else if (strncmp(s,"us",2)==0 || strncmp(s,"µs",2)==0){ total += val*1000.0L; s+=2; }
        else if (strncmp(s,"ms",2)==0){ total += val*1e6L; s+=2; }
        else if (*s=='s'){ total += val*1e9L; s++; }
        else if (*s=='m'){ total += val*60.0L*1e9L; s++; }
        else if (*s=='h'){ total += val*3600.0L*1e9L; s++; }
        else { matched=0; }
        if (!matched) return TVX_EPARSE;
        any=1;
    }
    if (!any) return TVX_EPARSE;
    if (neg) total = -total;
    if (total > (long double)INT64_MAX || total < (long double)INT64_MIN) return TVX_ERANGE;
    *out = (tvx_duration)(total + (total>=0?0.5L:-0.5L));
    return TVX_OK;
}

int tvx_FormatDuration(tvx_duration d, char *buf, size_t buflen){
    if (!buf||buflen<2) return TVX_ERANGE;
    if (d==0){ snprintf(buf, buflen, "0s"); return TVX_OK; }
    int neg = d<0; long long v = neg? -(long long)d : (long long)d;
    // Prefer h, m, s, ms, us, ns mixed like Go
    long long ns = v % 1000000000LL; long long sec = v / 1000000000LL;
    long long h = sec/3600; sec%=3600; long long m = sec/60; sec%=60;
    if (ns==0){
        if (h>0) snprintf(buf, buflen, "%s%lldh%lldm%llds", neg?"-":"", h,m,sec);
        else if (m>0) snprintf(buf, buflen, "%s%lldm%llds", neg?"-":"", m,sec);
        else snprintf(buf, buflen, "%s%llds", neg?"-":"", sec);
        return TVX_OK;
    } else {
        // print fractional seconds up to 9 digits, trimming trailing zeros
        char frac[16]; snprintf(frac, sizeof(frac), "%09lld", ns);
        for (int i=8;i>0;i--) if (frac[i]=='0') frac[i]=0; else break;
        snprintf(buf, buflen, "%s%lld.%ss", neg?"-":"", v/1000000000LL, frac);
        return TVX_OK;
    }
}

// ----------------------- RFC3339 parse/format ------------------------

// Minimal but robust RFC3339 / RFC3339Nano parser.
static int parse_2(const char *s){ return (s[0]-'0')*10 + (s[1]-'0'); }
static int parse_4(const char *s){ return (s[0]-'0')*1000 + (s[1]-'0')*100 + (s[2]-'0')*10 + (s[3]-'0'); }

int tvx_ParseRFC3339(const char *s, tvx_time *out){
    if (!s||!out) return TVX_EPARSE;
    size_t n = strlen(s);
    if (n < 20) return TVX_EPARSE;
    // YYYY-MM-DD[T ]HH:MM:SS(.nnn)?(Z|+hh:mm|-hh:mm)
    int year = parse_4(s);
    int mon = parse_2(s+5);
    int day = parse_2(s+8);
    int hour = parse_2(s+11);
    int min = parse_2(s+14);
    int sec = parse_2(s+17);
    int idx = 19;
    int32_t nsec = 0;
    if (s[10]!='T' && s[10]!='t' && s[10]!=' ' ) return TVX_EPARSE;
    if (s[4]!='-'||s[7]!='-'||s[13]!=':'||s[16]!=':') return TVX_EPARSE;
    if (idx < (int)n && s[idx]=='.'){
        idx++;
        int digits=0;
        while (idx<(int)n && s[idx]>='0'&&s[idx]<='9' && digits<9){
            nsec = nsec*10 + (s[idx]-'0'); idx++; digits++;
        }
        while (digits++<9) nsec*=10;
    }
    int offset_sign = 0;
    int off_h=0, off_m=0;
    if (idx<(int)n && (s[idx]=='Z'||s[idx]=='z')){ idx++; offset_sign=0; }
    else if (idx+5 <= (int)n && (s[idx]=='+'||s[idx]=='-')){
        offset_sign = (s[idx]=='+')?+1:-1; idx++;
        off_h = parse_2(s+idx); idx+=2;
        if (s[idx]!=':') return TVX_EPARSE; idx++;
        off_m = parse_2(s+idx); idx+=2;
    } else return TVX_EPARSE;

    if (mon<1||mon>12) return TVX_ERANGE;
    if (day<1||day>31) return TVX_ERANGE;
    if (hour<0||hour>23) return TVX_ERANGE;
    if (min<0||min>59) return TVX_ERANGE;
    if (sec<0||sec>60) return TVX_ERANGE;

    struct tm tm = {0};
    tm.tm_year = year-1900; tm.tm_mon = mon-1; tm.tm_mday = day;
    tm.tm_hour = hour; tm.tm_min = min; tm.tm_sec = sec;
    time_t base = timegm_portable(&tm); // treat fields as UTC, then apply offset
    int off = offset_sign * (off_h*3600 + off_m*60);
    time_t final = base - off;
    tvx_location loc = offset_sign==0 ? tvx_UTC() : tvx_FixedZone("UTC", -off);
    *out = tvx_Unix((int64_t)final, nsec, loc);
    return TVX_OK;
}

int tvx_FormatRFC3339(tvx_time t, int nano, char *buf, size_t buflen){
    if (!buf||buflen<32) return TVX_ERANGE;
    struct tm tm;
    int offset = 0;
    if (strcmp(t.loc.name,"UTC")==0){
        gmtime_r((time_t*)&t.unix_sec, &tm);
    } else if (strcmp(t.loc.name,"Local")==0){
        localtime_r((time_t*)&t.unix_sec, &tm);
        // Derive offset from tm_gmtoff if available
#ifdef __APPLE__
        offset = (int)tm.tm_gmtoff;
#elif defined(__GLIBC__)
        offset = (int)tm.tm_gmtoff;
#else
        // Fallback: recompute using mktime vs timegm
        struct tm tm2 = tm; time_t lt = mktime(&tm2);
        struct tm g; gmtime_r((time_t*)&t.unix_sec, &g);
        time_t ut = timegm_portable(&g);
        offset = (int)difftime(lt, ut);
#endif
    } else {
        char prev[256]={0}; tz_swap(t.loc.name, prev, sizeof(prev));
        localtime_r((time_t*)&t.unix_sec, &tm);
#ifdef __APPLE__
        offset = (int)tm.tm_gmtoff;
#elif defined(__GLIBC__)
        offset = (int)tm.tm_gmtoff;
#else
        struct tm tm2 = tm; time_t lt = mktime(&tm2);
        struct tm g; gmtime_r((time_t*)&t.unix_sec, &g);
        time_t ut = timegm_portable(&g);
        offset = (int)difftime(lt, ut);
#endif
        tz_swap(NULL, prev, sizeof(prev));
    }
    int year = tm.tm_year+1900;
    int mon = tm.tm_mon+1;
    int day = tm.tm_mday;
    int hh = tm.tm_hour, mm = tm.tm_min, ss = tm.tm_sec;
    char frac[16]="";
    if (nano){
        // print nsec with trimming
        char tmp[16]; snprintf(tmp,sizeof(tmp), "%09d", t.nsec);
        int end = 8; while (end>=0 && tmp[end]=='0') end--;
        if (end>=0){ tmp[end+1]=0; snprintf(frac,sizeof(frac), ".%s", tmp); }
    }
    if (offset==0){
        snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02d%sZ",
            year,mon,day,hh,mm,ss, frac);
    } else {
        int sign = offset>=0?+1:-1;
        int a = sign*offset;
        int oh = a/3600, om = (a%3600)/60;
        snprintf(buf, buflen, "%04d-%02d-%02dT%02d:%02d:%02d%s%c%02d:%02d",
            year,mon,day,hh,mm,ss, frac, sign>=0?'+':'-', oh, om);
    }
    return TVX_OK;
}

// ---------------------------- Sleep ----------------------------------

void tvx_Sleep(tvx_duration d){
    if (d<=0){ sched_yield(); return; }
    struct timespec ts;
    ts.tv_sec = d/1000000000LL;
    ts.tv_nsec = d%1000000000LL;
    while (nanosleep(&ts, &ts)==-1 && errno==EINTR) { /* retry */ }
}

// ------------------------- Timers/Tickers ----------------------------

struct tvx_timer {
    pthread_t th;
    pthread_mutex_t mu;
    pthread_cond_t cv;
    int active; // 1 if waiting, 0 otherwise
    int stop;   // stop requested
    tvx_duration dur;
    tvx_callback cb;
    void *user;
};

static void* timer_worker(void *arg){
    tvx_timer *t = (tvx_timer*)arg;
    pthread_mutex_lock(&t->mu);
    while (!t->stop){
        if (!t->active){
            pthread_cond_wait(&t->cv, &t->mu);
            continue;
        }
        tvx_duration d = t->dur;
        pthread_mutex_unlock(&t->mu);
        tvx_Sleep(d);
        pthread_mutex_lock(&t->mu);
        if (!t->stop && t->active){
            t->active = 0;
            tvx_callback cb = t->cb; void *user = t->user;
            pthread_mutex_unlock(&t->mu);
            if (cb) cb(user);
            pthread_mutex_lock(&t->mu);
        }
    }
    pthread_mutex_unlock(&t->mu);
    return NULL;
}

tvx_timer* tvx_NewTimer(tvx_duration d, tvx_callback cb, void *user){
    tvx_timer *t = (tvx_timer*)calloc(1, sizeof(*t));
    if (!t) return NULL;
    pthread_mutex_init(&t->mu, NULL);
    pthread_cond_init(&t->cv, NULL);
    t->active = 1; t->stop = 0; t->dur = d; t->cb = cb; t->user = user;
    pthread_create(&t->th, NULL, timer_worker, t);
    pthread_mutex_lock(&t->mu);
    pthread_cond_signal(&t->cv);
    pthread_mutex_unlock(&t->mu);
    return t;
}

int tvx_TimerReset(tvx_timer *t, tvx_duration d){
    if (!t) return 0;
    pthread_mutex_lock(&t->mu);
    int was_active = t->active;
    t->dur = d; t->active = 1;
    pthread_cond_signal(&t->cv);
    pthread_mutex_unlock(&t->mu);
    return was_active;
}

int tvx_TimerStop(tvx_timer *t){
    if (!t) return 0;
    pthread_mutex_lock(&t->mu);
    int was_active = t->active;
    t->active = 0;
    pthread_mutex_unlock(&t->mu);
    return was_active;
}

void tvx_TimerFree(tvx_timer *t){
    if (!t) return;
    pthread_mutex_lock(&t->mu);
    t->stop = 1;
    pthread_cond_signal(&t->cv);
    pthread_mutex_unlock(&t->mu);
    pthread_join(t->th, NULL);
    pthread_mutex_destroy(&t->mu);
    pthread_cond_destroy(&t->cv);
    free(t);
}

struct tvx_ticker {
    pthread_t th;
    pthread_mutex_t mu;
    pthread_cond_t cv;
    int running;
    int stop;
    tvx_duration period;
    tvx_callback cb;
    void *user;
};

static void* ticker_worker(void *arg){
    tvx_ticker *tk = (tvx_ticker*)arg;
    pthread_mutex_lock(&tk->mu);
    while (!tk->stop){
        if (!tk->running){
            pthread_cond_wait(&tk->cv, &tk->mu);
            continue;
        }
        tvx_duration p = tk->period;
        pthread_mutex_unlock(&tk->mu);
        tvx_Sleep(p);
        pthread_mutex_lock(&tk->mu);
        if (!tk->stop && tk->running){
            tvx_callback cb = tk->cb; void *user = tk->user;
            pthread_mutex_unlock(&tk->mu);
            if (cb) cb(user);
            pthread_mutex_lock(&tk->mu);
        }
    }
    pthread_mutex_unlock(&tk->mu);
    return NULL;
}

tvx_ticker* tvx_NewTicker(tvx_duration period_ns, tvx_callback cb, void *user){
    if (period_ns < 1000000LL) period_ns = 1000000LL; // >=1ms
    tvx_ticker *tk = (tvx_ticker*)calloc(1, sizeof(*tk));
    if (!tk) return NULL;
    pthread_mutex_init(&tk->mu, NULL);
    pthread_cond_init(&tk->cv, NULL);
    tk->running = 1; tk->stop = 0; tk->period = period_ns; tk->cb = cb; tk->user = user;
    pthread_create(&tk->th, NULL, ticker_worker, tk);
    pthread_mutex_lock(&tk->mu);
    pthread_cond_signal(&tk->cv);
    pthread_mutex_unlock(&tk->mu);
    return tk;
}

int tvx_TickerReset(tvx_ticker *tk, tvx_duration period_ns){
    if (!tk) return 0;
    if (period_ns < 1000000LL) period_ns = 1000000LL;
    pthread_mutex_lock(&tk->mu);
    tk->period = period_ns;
    tk->running = 1;
    pthread_cond_signal(&tk->cv);
    pthread_mutex_unlock(&tk->mu);
    return 1;
}

int tvx_TickerStop(tvx_ticker *tk){
    if (!tk) return 0;
    pthread_mutex_lock(&tk->mu);
    int was = tk->running;
    tk->running = 0;
    pthread_mutex_unlock(&tk->mu);
    return was;
}

void tvx_TickerFree(tvx_ticker *tk){
    if (!tk) return;
    pthread_mutex_lock(&tk->mu);
    tk->stop = 1;
    pthread_cond_signal(&tk->cv);
    pthread_mutex_unlock(&tk->mu);
    pthread_join(tk->th, NULL);
    pthread_mutex_destroy(&tk->mu);
    pthread_cond_destroy(&tk->cv);
    free(tk);
}
