
        #define _GNU_SOURCE
        #include <stdio.h>
        #include <stdlib.h>
        #include <dlfcn.h>
        #include <sys/mman.h>
        #define THRESHOLD (16 * 1024 * 1024)
        #define SAFETY_MARGIN (20 * 1024 * 1024)
        void* malloc(size_t size) {
        static void* (*real_malloc)(size_t) = NULL;
        if (!real_malloc) real_malloc = dlsym(RTLD_NEXT, "malloc");
        if (size < THRESHOLD || size > (1024ULL * 1024 * 1024 * 4)) return real_malloc(size);
        size_t expanded_size = size + SAFETY_MARGIN;
        void* ptr = real_malloc(expanded_size);
        if (ptr) madvise(ptr, expanded_size, MADV_WILLNEED);
        return ptr;
    }
    