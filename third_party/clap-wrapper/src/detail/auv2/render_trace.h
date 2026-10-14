#pragma once
// Diagnostic branch only. All file setup and page prefaulting happen in Initialize.
// Render writes fixed-size records to preallocated shared memory: no file calls,
// allocation, locks, formatting, waiting or retry loops in the trace writer.
#include <clap/clap.h>
#include <algorithm>
#include <atomic>
#include <cstdint>
#include <cstring>
#include <cstdlib>
#include <cstdio>
#include <mutex>
#include <new>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <mach/mach_time.h>
#include <pthread.h>
namespace ca72_trace {
constexpr uint64_t capacity = 262144;
struct Record { std::atomic<uint64_t> committed; uint64_t fields[15]; };
struct Header {
  uint64_t magic, version, count, record_size, numer, denom, pid, ppid;
  std::atomic<uint64_t> next, dropped;
};
static_assert(sizeof(Record) == 128);
static_assert(std::atomic<uint64_t>::is_always_lock_free);
inline std::atomic<void*> memory{nullptr};
inline std::once_flag prepared;
inline uint64_t bits(double v) { uint64_t b; std::memcpy(&b, &v, sizeof b); return b; }
inline void prepare() {
  std::call_once(prepared, [] {
    auto directory = std::getenv("CA72_AU_TRACE_DIR");
    if (!directory) return;
    char path[2048];
    if (std::snprintf(path, sizeof path, "%s/au-render-%d.bin", directory, getpid()) >= int(sizeof path)) std::abort();
    int fd = open(path, O_CREAT | O_EXCL | O_RDWR, 0600);
    const size_t bytes = 4096 + capacity * sizeof(Record);
    if (fd < 0 || ftruncate(fd, bytes)) std::abort();
    auto p = mmap(nullptr, bytes, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    close(fd);
    if (p == MAP_FAILED) std::abort();
    std::memset(p, 0, bytes); // prefault outside rendering
    auto h = new(p) Header{};
    mach_timebase_info_data_t clock{}; mach_timebase_info(&clock);
    h->magic = 0x4341373254524143ULL; h->version = 1; h->count = capacity;
    h->record_size = sizeof(Record); h->numer = clock.numer; h->denom = clock.denom;
    h->pid = getpid(); h->ppid = getppid();
    auto records = reinterpret_cast<Record*>(static_cast<char*>(p) + 4096);
    for (uint64_t i = 0; i < capacity; ++i) new(&records[i]) Record{};
    memory.store(p, std::memory_order_release);
  });
}
inline void put(uint64_t kind, const void* object, uint64_t a=0, uint64_t b=0,
                uint64_t c=0, uint64_t d=0, uint64_t e=0, uint64_t f=0,
                uint64_t g=0, uint64_t hvalue=0, uint64_t i=0, uint64_t j=0,
                uint64_t k=0) {
  auto p = memory.load(std::memory_order_acquire); if (!p) return;
  auto h = static_cast<Header*>(p);
  auto n = h->next.fetch_add(1, std::memory_order_relaxed);
  if (n >= capacity) { h->dropped.fetch_add(1, std::memory_order_relaxed); return; }
  auto r = reinterpret_cast<Record*>(static_cast<char*>(p) + 4096) + n;
  auto &v = r->fields;
  v[0]=mach_absolute_time(); v[1]=kind; v[2]=reinterpret_cast<uintptr_t>(object);
  v[3]=pthread_mach_thread_np(pthread_self()); v[4]=a; v[5]=b; v[6]=c;
  v[7]=d; v[8]=e; v[9]=f; v[10]=g; v[11]=hvalue; v[12]=i; v[13]=j; v[14]=k;
  r->committed.store(n+1, std::memory_order_release);
}
inline void snapshot(const void* owner, const clap_plugin_t* plugin, const clap_plugin_params_t* params) {
  if (!params) return;
  const auto count = params->count(plugin);
  for (uint32_t index = 0; index < count; ++index) {
    clap_param_info_t info{};
    if (!params->get_info(plugin, index, &info)) continue;
    double value = 0; params->get_value(plugin, info.id, &value);
    uint64_t name[4]{};
    std::memcpy(name, info.name, std::min(sizeof name, strnlen(info.name, sizeof info.name)));
    put(10, owner, info.id, bits(value), name[0], name[1], name[2], name[3]);
  }
}
} // namespace ca72_trace
