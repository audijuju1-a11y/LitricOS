/*
 * litric-drmpresent -- KMS scanout for the LitricOS shell.
 *
 * The shell's raster is safe Rust (ADR-0017). What it cannot do without
 * `unsafe` is talk ioctl to the display controller, and until now it wrote
 * finished frames through fbdev, which the DRM driver only mirrors to the
 * screen by luck: QEMU's drm_fb_helper acknowledges write(2) and leaves the
 * scanout alone. This helper is the one audited place where the unsafe part
 * of the output path lives, and it is deliberately smaller than the fbdev one
 * it replaces.
 *
 * # What it does
 *
 * Opens one card node, takes DRM master, finds a connected connector, picks
 * its preferred mode, allocates two dumb buffers, sets the CRTC, and then
 * receives damage bands from stdin. Every band is copied into *both*
 * buffers, so a partial update can never be shown with an older frame's stale
 * rows behind it, and each `P` asks the driver for a page flip. The flip is
 * hardware: the display controller changes which buffer it is scanning out at
 * the next vertical blank, so nothing is torn and no pixels travel through a
 * pipe.
 *
 * # Protocol (line control, then raw pixels)
 *
 *   B <y> <rows>\n      rows * pitch bytes of XRGB8888 follow
 *   P\n                 flip and wait (bounded) for the completion event
 *   Q\n                 clean exit
 *   EOF                 clean exit
 *
 * It has no parser for anything else, no filesystem access beyond the card
 * node it was given, no network, and no commands that move, scale or restyle
 * the picture. A malformed command is fatal rather than guessed at: a display
 * helper that improvises is a display helper that shows the wrong thing.
 *
 * # What is NOT verified here
 *
 * This file is compiled with -Werror but has never been run against a real
 * DRM device in this repository's evidence: the build host has no card node
 * and the image has not been booted with this path. `--check` exists so the
 * first boot can prove connector, mode and format without taking the screen.
 */

#define _POSIX_C_SOURCE 200809L

#include <drm/drm.h>
#include <drm/drm_fourcc.h>
#include <drm/drm_mode.h>

#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>

#define TAG "litric-drmpresent"

/* How long a flip completion may take before the frame is given up on. A
 * helper that blocks here takes the whole desktop with it, and a missed
 * completion event is a diagnostic, not a reason to stop drawing. */
#define FLIP_TIMEOUT_MS 100

struct output {
    int fd;
    uint32_t conn_id;
    uint32_t crtc_id;
    struct drm_mode_modeinfo mode;
    uint32_t fb[2];
    uint32_t handle[2];
    uint8_t *map[2];
    size_t bytes[2];
    uint32_t pitch;
    int front;
    int flip_ok;
    char name[64];
};

static void fail(const char *what) {
    fprintf(stderr, "%s: %s: %s\n", TAG, what, strerror(errno));
}

static int xioctl(int fd, unsigned long request, void *arg) {
    int r;
    do {
        r = ioctl(fd, request, arg);
    } while (r < 0 && errno == EINTR);
    return r;
}

/* DRM connector types, for a name a person can match against the machine. */
static const char *connector_name(uint32_t type) {
    switch (type) {
    case 1: return "VGA";
    case 2: return "DVI-I";
    case 3: return "DVI-D";
    case 4: return "DVI-A";
    case 5: return "Composite";
    case 7: return "LVDS";
    case 9: return "DIN";
    case 10: return "DP";
    case 11: return "HDMI-A";
    case 12: return "HDMI-B";
    case 14: return "eDP";
    case 15: return "Virtual";
    case 16: return "DSI";
    case 18: return "Writeback";
    case 19: return "SPI";
    case 20: return "USB";
    default: return "Connector";
    }
}

/* One control line, byte at a time: it is at most a few dozen bytes and the
 * payload that follows is read in bulk. Returns the length, 0 at a clean EOF,
 * -1 on error or an over-long line. */
static int read_line(int fd, char *buf, size_t cap) {
    size_t n = 0;
    for (;;) {
        char c;
        ssize_t got = read(fd, &c, 1);
        if (got == 0) {
            return n == 0 ? 0 : -1;
        }
        if (got < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        if (c == '\n') {
            if (n >= cap) return -1;
            buf[n] = '\0';
            return (int)n;
        }
        if (n + 1 >= cap) return -1;
        buf[n++] = c;
    }
}

/* Read exactly `len` bytes. Returns 1 when full, 0 at EOF before any byte, and
 * -1 for a truncated band: half a band is a band that must not be shown. */
static int read_full(int fd, void *buf, size_t len) {
    size_t offset = 0;
    while (offset < len) {
        ssize_t n = read(fd, (uint8_t *)buf + offset, len - offset);
        if (n == 0) return offset == 0 ? 0 : -1;
        if (n < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        offset += (size_t)n;
    }
    return 1;
}

static int wait_flip(int fd) {
    struct pollfd p = {.fd = fd, .events = POLLIN, .revents = 0};
    int ready = poll(&p, 1, FLIP_TIMEOUT_MS);
    if (ready <= 0) return 0;
    uint8_t buf[256];
    ssize_t n = read(fd, buf, sizeof buf);
    if (n < 8) return 0;
    uint32_t type = 0;
    memcpy(&type, buf, sizeof type);
    return type == DRM_EVENT_FLIP_COMPLETE;
}

static int add_dumb(struct output *o, int index) {
    struct drm_mode_create_dumb create;
    memset(&create, 0, sizeof create);
    create.width = o->mode.hdisplay;
    create.height = o->mode.vdisplay;
    create.bpp = 32;
    if (xioctl(o->fd, DRM_IOCTL_MODE_CREATE_DUMB, &create) < 0) {
        fail("create dumb buffer");
        return -1;
    }
    o->handle[index] = create.handle;
    o->pitch = create.pitch;

    struct drm_mode_fb_cmd2 fb;
    memset(&fb, 0, sizeof fb);
    fb.width = create.width;
    fb.height = create.height;
    fb.pixel_format = DRM_FORMAT_XRGB8888;
    fb.handles[0] = create.handle;
    fb.pitches[0] = create.pitch;
    if (xioctl(o->fd, DRM_IOCTL_MODE_ADDFB2, &fb) < 0) {
        fail("add framebuffer");
        return -1;
    }
    o->fb[index] = fb.fb_id;

    struct drm_mode_map_dumb map;
    memset(&map, 0, sizeof map);
    map.handle = create.handle;
    if (xioctl(o->fd, DRM_IOCTL_MODE_MAP_DUMB, &map) < 0) {
        fail("map dumb buffer");
        return -1;
    }
    void *mapped = mmap(NULL, create.size, PROT_READ | PROT_WRITE, MAP_SHARED,
                        o->fd, (off_t)map.offset);
    if (mapped == MAP_FAILED) {
        fail("mmap dumb buffer");
        return -1;
    }
    o->map[index] = mapped;
    o->bytes[index] = create.size;
    /* Zeroed once so a band that does not cover the screen shows black rather
     * than whatever the allocator left behind. */
    memset(mapped, 0, create.size);
    return 0;
}

/* Find a connector to drive and the mode to drive it at. */
static int choose(struct output *o, uint32_t *connectors, uint32_t count_connectors,
                  uint32_t *crtcs, uint32_t count_crtcs) {
    int found = 0;
    for (uint32_t i = 0; i < count_connectors && !found; i++) {
        struct drm_mode_get_connector conn;
        memset(&conn, 0, sizeof conn);
        conn.connector_id = connectors[i];
        if (xioctl(o->fd, DRM_IOCTL_MODE_GETCONNECTOR, &conn) < 0) continue;
        uint32_t count_modes = conn.count_modes;
        uint32_t connector_encoders = conn.count_encoders;
        if (conn.connection != 1 /* DRM_MODE_CONNECTED */ || count_modes == 0) {
            continue;
        }
        struct drm_mode_modeinfo *modes = calloc(count_modes, sizeof *modes);
        uint32_t *conn_enc = calloc(connector_encoders ? connector_encoders : 1,
                                    sizeof *conn_enc);
        if (modes == NULL || conn_enc == NULL) {
            free(modes);
            free(conn_enc);
            continue;
        }
        memset(&conn, 0, sizeof conn);
        conn.connector_id = connectors[i];
        conn.count_modes = count_modes;
        conn.modes_ptr = (uint64_t)(uintptr_t)modes;
        conn.count_encoders = connector_encoders;
        conn.encoders_ptr = (uint64_t)(uintptr_t)conn_enc;
        if (xioctl(o->fd, DRM_IOCTL_MODE_GETCONNECTOR, &conn) < 0) {
            free(modes);
            free(conn_enc);
            continue;
        }

        /* Preferred mode first; otherwise the first mode the driver listed,
         * which is the order the connector advertises. Inventing a resolution
         * from the framebuffer's old geometry would be a guess about hardware
         * we can simply ask. */
        int pick = -1;
        for (uint32_t m = 0; m < count_modes; m++) {
            if (modes[m].type & 8 /* DRM_MODE_TYPE_PREFERRED */) {
                pick = (int)m;
                break;
            }
        }
        if (pick < 0) {
            for (uint32_t m = 0; m < count_modes; m++) {
                if (modes[m].hdisplay > 0 && modes[m].vdisplay > 0) {
                    pick = (int)m;
                    break;
                }
            }
        }
        if (pick < 0) {
            free(modes);
            free(conn_enc);
            continue;
        }

        /* The encoder the connector is wired to, else any encoder that can
         * drive one of this card's CRTCs. */
        uint32_t encoder_id = conn.encoder_id;
        if (encoder_id == 0) {
            for (uint32_t e = 0; e < connector_encoders; e++) {
                struct drm_mode_get_encoder enc;
                memset(&enc, 0, sizeof enc);
                enc.encoder_id = conn_enc[e];
                if (xioctl(o->fd, DRM_IOCTL_MODE_GETENCODER, &enc) < 0) continue;
                if (enc.possible_crtcs != 0) {
                    encoder_id = enc.encoder_id;
                    break;
                }
            }
        }
        struct drm_mode_get_encoder enc;
        memset(&enc, 0, sizeof enc);
        enc.encoder_id = encoder_id;
        if (encoder_id == 0 || xioctl(o->fd, DRM_IOCTL_MODE_GETENCODER, &enc) < 0) {
            free(modes);
            free(conn_enc);
            continue;
        }
        uint32_t crtc_id = enc.crtc_id;
        if (crtc_id == 0) {
            for (uint32_t c = 0; c < count_crtcs; c++) {
                if (enc.possible_crtcs & (1u << c)) {
                    crtc_id = crtcs[c];
                    break;
                }
            }
        }
        if (crtc_id == 0) {
            free(modes);
            free(conn_enc);
            continue;
        }

        o->conn_id = conn.connector_id;
        o->crtc_id = crtc_id;
        o->mode = modes[pick];
        snprintf(o->name, sizeof o->name, "%s-%u",
                 connector_name(conn.connector_type), conn.connector_type_id);
        found = 1;
        free(modes);
        free(conn_enc);
    }
    return found ? 0 : -1;
}

/* Read the card's resources and pick an output. Returns 0 and fills `o` on
 * success; leaves the card untouched beyond the resource queries. */
static int probe(int fd, struct output *o) {
    struct drm_mode_card_res res;
    memset(&res, 0, sizeof res);
    if (xioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &res) < 0) {
        fail("get resources");
        return -1;
    }
    uint32_t count_connectors = res.count_connectors;
    uint32_t count_crtcs = res.count_crtcs;
    if (count_connectors == 0 || count_crtcs == 0) {
        fprintf(stderr, "%s: this card reports no connector or no CRTC\n", TAG);
        return -1;
    }
    uint32_t *connectors = calloc(count_connectors, sizeof *connectors);
    uint32_t *crtcs = calloc(count_crtcs, sizeof *crtcs);
    if (connectors == NULL || crtcs == NULL) {
        fprintf(stderr, "%s: out of memory reading resources\n", TAG);
        free(connectors);
        free(crtcs);
        return -1;
    }
    memset(&res, 0, sizeof res);
    res.count_connectors = count_connectors;
    res.connector_id_ptr = (uint64_t)(uintptr_t)connectors;
    res.count_crtcs = count_crtcs;
    res.crtc_id_ptr = (uint64_t)(uintptr_t)crtcs;
    if (xioctl(fd, DRM_IOCTL_MODE_GETRESOURCES, &res) < 0) {
        fail("get resources (arrays)");
        free(connectors);
        free(crtcs);
        return -1;
    }
    int rc = choose(o, connectors, count_connectors, crtcs, count_crtcs);
    free(connectors);
    free(crtcs);
    if (rc < 0) {
        fprintf(stderr, "%s: no connected output with a mode\n", TAG);
    }
    return rc;
}

static void release(struct output *o) {
    for (int i = 0; i < 2; i++) {
        if (o->map[i] != NULL && o->bytes[i] > 0) {
            munmap(o->map[i], o->bytes[i]);
            o->map[i] = NULL;
        }
        if (o->fb[i] != 0) {
            /* The framebuffer has to go before the memory behind it, or the
             * card keeps a reference to a buffer nobody owns. */
            uint32_t fb = o->fb[i];
            (void)xioctl(o->fd, DRM_IOCTL_MODE_RMFB, &fb);
            struct drm_mode_destroy_dumb destroy;
            memset(&destroy, 0, sizeof destroy);
            destroy.handle = o->handle[i];
            (void)xioctl(o->fd, DRM_IOCTL_MODE_DESTROY_DUMB, &destroy);
            o->fb[i] = 0;
        }
    }
    (void)xioctl(o->fd, DRM_IOCTL_DROP_MASTER, NULL);
    if (o->fd >= 0) close(o->fd);
    o->fd = -1;
}

static int present(struct output *o) {
    if (!o->flip_ok) {
        /* Both buffers always hold the current frame, so the one being
         * scanned out is already right and there is nothing to flip. */
        return 0;
    }
    int back = 1 - o->front;
    struct drm_mode_crtc_page_flip flip;
    memset(&flip, 0, sizeof flip);
    flip.crtc_id = o->crtc_id;
    flip.fb_id = o->fb[back];
    flip.flags = DRM_MODE_PAGE_FLIP_EVENT;
    if (xioctl(o->fd, DRM_IOCTL_MODE_PAGE_FLIP, &flip) < 0) {
        if (errno == EINVAL || errno == ENOSYS) {
            /* Some drivers refuse to flip a dumb buffer. The picture is still
             * correct because the bands went into both buffers; record the
             * downgrade once and keep drawing. */
            fprintf(stderr, "%s: page flip unsupported, scanning out in place\n", TAG);
            o->flip_ok = 0;
            return 0;
        }
        fail("page flip");
        return -1;
    }
    if (!wait_flip(o->fd)) {
        fprintf(stderr, "%s: no flip completion within %d ms\n", TAG, FLIP_TIMEOUT_MS);
    }
    o->front = back;
    return 0;
}

int main(int argc, char **argv) {
    int check_only = 0;
    const char *card = NULL;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--check") == 0) {
            check_only = 1;
        } else if (card == NULL) {
            card = argv[i];
        } else {
            fprintf(stderr, "usage: %s [--check] /dev/dri/cardN\n", TAG);
            return 64;
        }
    }
    if (card == NULL) {
        fprintf(stderr, "usage: %s [--check] /dev/dri/cardN\n", TAG);
        return 64;
    }

    struct output o;
    memset(&o, 0, sizeof o);
    o.fd = -1;
    o.flip_ok = 1;

    o.fd = open(card, O_RDWR | O_CLOEXEC);
    if (o.fd < 0) {
        fail("open card");
        return 1;
    }
    if (probe(o.fd, &o) < 0) {
        close(o.fd);
        return 1;
    }
    if (check_only) {
        printf("LITRIC-DRM/1 %u %u %u %s\n", o.mode.hdisplay, o.mode.vdisplay,
               o.mode.hdisplay * 4u, o.name);
        fflush(stdout);
        close(o.fd);
        return 0;
    }

    /* Master is what allows mode setting. Without it SETCRTC is refused, and
     * drawing to a buffer nobody scans out is worse than saying so. */
    if (xioctl(o.fd, DRM_IOCTL_SET_MASTER, NULL) < 0) {
        if (errno != EINVAL) {
            fail("become DRM master");
            close(o.fd);
            return 1;
        }
    }

    if (add_dumb(&o, 0) < 0 || add_dumb(&o, 1) < 0) {
        release(&o);
        return 1;
    }

    struct drm_mode_crtc set;
    memset(&set, 0, sizeof set);
    set.set_connectors_ptr = (uint64_t)(uintptr_t)&o.conn_id;
    set.count_connectors = 1;
    set.crtc_id = o.crtc_id;
    set.fb_id = o.fb[0];
    set.mode = o.mode;
    set.mode_valid = 1;
    if (xioctl(o.fd, DRM_IOCTL_MODE_SETCRTC, &set) < 0) {
        fail("set CRTC");
        release(&o);
        return 1;
    }

    printf("LITRIC-DRM/1 %u %u %u %s\n", o.mode.hdisplay, o.mode.vdisplay,
           o.pitch, o.name);
    fflush(stdout);

    uint8_t *band = malloc((size_t)o.pitch * o.mode.vdisplay);
    if (band == NULL) {
        fprintf(stderr, "%s: out of memory for a band\n", TAG);
        release(&o);
        return 1;
    }

    int rc = 0;
    for (;;) {
        char line[128];
        int n = read_line(STDIN_FILENO, line, sizeof line);
        if (n == 0) break;
        if (n < 0) {
            fprintf(stderr, "%s: malformed or truncated command\n", TAG);
            rc = 2;
            break;
        }
        if (line[0] == 'Q') break;
        if (line[0] == 'P') {
            if (present(&o) < 0) {
                rc = 1;
                break;
            }
            continue;
        }
        if (line[0] == 'B') {
            unsigned long y = 0;
            unsigned long rows = 0;
            char extra = 0;
            if (sscanf(line, "B %lu %lu%c", &y, &rows, &extra) != 2) {
                fprintf(stderr, "%s: bad band header '%s'\n", TAG, line);
                rc = 2;
                break;
            }
            if (rows == 0 || y + rows > o.mode.vdisplay) {
                fprintf(stderr, "%s: band %lu+%lu outside %u rows\n", TAG, y, rows,
                        o.mode.vdisplay);
                rc = 2;
                break;
            }
            size_t length = (size_t)rows * o.pitch;
            int got = read_full(STDIN_FILENO, band, length);
            if (got <= 0) {
                fprintf(stderr, "%s: incomplete band (%d)\n", TAG, got);
                rc = got == 0 ? 0 : 2;
                break;
            }
            memcpy(o.map[0] + (size_t)y * o.pitch, band, length);
            memcpy(o.map[1] + (size_t)y * o.pitch, band, length);
            continue;
        }
        fprintf(stderr, "%s: unknown command '%s'\n", TAG, line);
        rc = 2;
        break;
    }

    free(band);
    release(&o);
    return rc;
}
