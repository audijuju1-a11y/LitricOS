/*
 * litric-fbpresent -- the deliberately tiny unsafe boundary for DRM fbdev.
 *
 * The Litric shell is safe Rust. Some DRM fbdev implementations acknowledge
 * write(2) yet do not refresh a QEMU scanout. This helper maps exactly the
 * device named by its parent, copies fixed-size frames from stdin, and asks
 * fbdev to pan the same visible buffer. It accepts no commands and has no
 * filesystem or network protocol: EOF exits, a partial frame is fatal.
 */
#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <fcntl.h>
#include <linux/fb.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

static int read_frame(int fd, unsigned char *buffer, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        ssize_t n = read(fd, buffer + offset, length - offset);
        if (n == 0) return offset == 0 ? 0 : -1;
        if (n < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        offset += (size_t)n;
    }
    return 1;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fputs("usage: litric-fbpresent /dev/fbN frame-bytes\n", stderr);
        return 64;
    }
    char *end = NULL;
    errno = 0;
    unsigned long long parsed = strtoull(argv[2], &end, 10);
    if (errno != 0 || end == argv[2] || *end != '\0' || parsed == 0 ||
        parsed > (unsigned long long)SIZE_MAX) {
        fputs("litric-fbpresent: invalid frame size\n", stderr);
        return 64;
    }
    size_t length = (size_t)parsed;
    int fb = open(argv[1], O_RDWR | O_CLOEXEC);
    if (fb < 0) {
        perror("litric-fbpresent: open framebuffer");
        return 1;
    }
    void *mapping = mmap(NULL, length, PROT_READ | PROT_WRITE, MAP_SHARED, fb, 0);
    if (mapping == MAP_FAILED) {
        perror("litric-fbpresent: mmap framebuffer");
        close(fb);
        return 1;
    }
    struct fb_var_screeninfo visible;
    int can_pan = ioctl(fb, FBIOGET_VSCREENINFO, &visible) == 0;
    for (;;) {
        int frame = read_frame(STDIN_FILENO, mapping, length);
        if (frame == 0) break;
        if (frame < 0) {
            fputs("litric-fbpresent: incomplete frame\n", stderr);
            munmap(mapping, length);
            close(fb);
            return 1;
        }
        /*
         * drm_fb_helper may back fbdev with deferred I/O. A MAP_SHARED write
         * dirties the shmem pages, while this explicit sync asks that layer to
         * schedule its flush before we request the visible-buffer pan. EINVAL
         * is documented for device mappings that do not implement msync and is
         * harmless there; their pan is still attempted below.
         */
        if (msync(mapping, length, MS_SYNC) < 0 && errno != EINVAL) {
            perror("litric-fbpresent: msync framebuffer");
            munmap(mapping, length);
            close(fb);
            return 1;
        }
        /* A no-op pan makes drivers commit the mapped front buffer. */
        if (can_pan) (void)ioctl(fb, FBIOPAN_DISPLAY, &visible);
    }
    munmap(mapping, length);
    close(fb);
    return 0;
}
