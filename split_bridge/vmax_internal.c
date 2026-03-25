
        #define _GNU_SOURCE
        #include <stdio.h>
        #include <stdlib.h>
        #include <dlfcn.h>
        void* malloc(size_t size) {
        static void* (*real_malloc)(size_t) = NULL;
        if (!real_malloc) real_malloc = dlsym(RTLD_NEXT, "malloc");
        if (size > 512000) { return real_malloc(size + (1024 * 1024)); }
        return real_malloc(size);
    }
    