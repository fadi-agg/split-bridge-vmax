#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <sys/mman.h>
#define THRESHOLD (10 * 1024 * 1024)

#define EXPANSION_RATIO 0.10

void* malloc(size_t size) {
    static void* (*real_malloc)(size_t) = NULL;
    if (!real_malloc) {
        real_malloc = dlsym(RTLD_NEXT, "malloc");
    }

    if (size < THRESHOLD || size > (1024ULL * 1024 * 1024 * 4)) {
        return real_malloc(size);
    }

    size_t expanded_size = size + (size_t)(size * EXPANSION_RATIO);

    fprintf(stderr, "🧪 Split VMAX: Original Request: %zu MB | Expanded Pipe (10%%): %zu MB\n",
            size / (1024 * 1024), expanded_size / (1024 * 1024));

    void* ptr = real_malloc(expanded_size);

    if (ptr) {

        madvise(ptr, expanded_size, MADV_WILLNEED);
    }

    return ptr;
}
