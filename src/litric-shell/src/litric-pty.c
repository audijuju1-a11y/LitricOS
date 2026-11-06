/*
 * litric-pty — the PTY owner for the LitricOS Terminal.
 *
 * The Rust shell forbids unsafe code. This helper is the only place that needs
 * a pseudo-terminal, fork and ioctl. It keeps the child on a real PTY and
 * polls a 0600 size file written by the shell; when the numbers change it
 * applies TIOCSWINSZ and sends SIGWINCH to the foreground process group.
 *
 * Usage: litric-pty /path/to/session.size
 * Size file: "columns rows\n" ("100 30\n"). Invalid or missing input keeps the
 * current size.
 */
#define _GNU_SOURCE

#include <errno.h>
#include <poll.h>
#include <pty.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/prctl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <termios.h>
#include <unistd.h>
#include <utmp.h>

#define INITIAL_COLS 100
#define INITIAL_ROWS 30
#define MIN_COLS 20
#define MAX_COLS 500
#define MIN_ROWS 5
#define MAX_ROWS 300

static volatile sig_atomic_t stop_requested = 0;

static void on_signal(int signo) {
    (void)signo;
    stop_requested = 1;
}

static int write_all(int fd, const char *bytes, size_t length) {
    size_t done = 0;
    while (done < length) {
        ssize_t wrote = write(fd, bytes + done, length - done);
        if (wrote < 0) {
            if (errno == EINTR) {
                continue;
            }
            return -1;
        }
        if (wrote == 0) {
            return -1;
        }
        done += (size_t)wrote;
    }
    return 0;
}

static int parse_size(const char *path, int *cols, int *rows) {
    FILE *file = fopen(path, "r");
    if (file == NULL) {
        return 0;
    }
    int c = 0;
    int r = 0;
    int matched = fscanf(file, "%d %d", &c, &r);
    fclose(file);
    if (matched != 2 || c < MIN_COLS || c > MAX_COLS || r < MIN_ROWS || r > MAX_ROWS) {
        return 0;
    }
    *cols = c;
    *rows = r;
    return 1;
}

static void apply_size(int master, pid_t child, int cols, int rows) {
    struct winsize window;
    memset(&window, 0, sizeof(window));
    window.ws_col = (unsigned short)cols;
    window.ws_row = (unsigned short)rows;
    if (ioctl(master, TIOCSWINSZ, &window) != 0) {
        return;
    }
    pid_t foreground = tcgetpgrp(master);
    if (foreground <= 0) {
        foreground = child;
    }
    (void)kill(-foreground, SIGWINCH);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s SIZE_FILE\n", argv[0]);
        return 2;
    }
    const char *size_path = argv[1];
    int cols = INITIAL_COLS;
    int rows = INITIAL_ROWS;
    (void)parse_size(size_path, &cols, &rows);

    struct winsize window;
    memset(&window, 0, sizeof(window));
    window.ws_col = (unsigned short)cols;
    window.ws_row = (unsigned short)rows;

    int master = -1;
    int slave = -1;
    if (openpty(&master, &slave, NULL, NULL, &window) != 0) {
        perror("litric-pty: openpty");
        return 1;
    }

    pid_t child = fork();
    if (child < 0) {
        perror("litric-pty: fork");
        close(master);
        close(slave);
        return 1;
    }

    if (child == 0) {
        close(master);
        if (login_tty(slave) != 0) {
            _exit(126);
        }
        if (slave > STDERR_FILENO) {
            close(slave);
        }
        (void)prctl(PR_SET_PDEATHSIG, SIGTERM);
        if (getppid() == 1) {
            _exit(1);
        }
        struct termios terminal;
        if (tcgetattr(STDIN_FILENO, &terminal) == 0) {
            terminal.c_lflag &= (tcflag_t)~(ECHO | ECHONL);
            (void)tcsetattr(STDIN_FILENO, TCSANOW, &terminal);
        }
        (void)setenv("TERM", "xterm-256color", 1);
        execl("/bin/sh", "sh", "-c",
              "PS1='__LITRIC_READY__'; PS2=''; export PS1 PS2; exec /bin/sh -i",
              (char *)NULL);
        _exit(127);
    }

    close(slave);
    (void)signal(SIGPIPE, SIG_IGN);
    (void)signal(SIGTERM, on_signal);
    (void)signal(SIGINT, on_signal);
    (void)signal(SIGHUP, on_signal);

    int stdin_open = 1;
    int child_done = 0;
    while (!stop_requested) {
        struct pollfd fds[2];
        fds[0].fd = stdin_open ? STDIN_FILENO : -1;
        fds[0].events = POLLIN;
        fds[0].revents = 0;
        fds[1].fd = master;
        fds[1].events = POLLIN;
        fds[1].revents = 0;

        int ready = poll(fds, 2, 100);
        if (ready < 0) {
            if (errno == EINTR) {
                continue;
            }
            break;
        }

        if (ready > 0) {
            if (stdin_open && (fds[0].revents & POLLIN) != 0) {
                char input[4096];
                ssize_t count = read(STDIN_FILENO, input, sizeof(input));
                if (count <= 0) {
                    char eot = 0x04;
                    stdin_open = 0;
                    (void)write_all(master, &eot, 1);
                } else if (write_all(master, input, (size_t)count) != 0) {
                    break;
                }
            }
            if ((fds[1].revents & (POLLIN | POLLHUP)) != 0) {
                char output[8192];
                ssize_t count = read(master, output, sizeof(output));
                if (count <= 0) {
                    break;
                }
                if (write_all(STDOUT_FILENO, output, (size_t)count) != 0) {
                    break;
                }
            }
        }

        int next_cols = cols;
        int next_rows = rows;
        if (parse_size(size_path, &next_cols, &next_rows) != 0 &&
            (next_cols != cols || next_rows != rows)) {
            cols = next_cols;
            rows = next_rows;
            apply_size(master, child, cols, rows);
        }

        if (!child_done) {
            int status = 0;
            pid_t reaped = waitpid(child, &status, WNOHANG);
            if (reaped == child || (reaped < 0 && errno == ECHILD)) {
                child_done = 1;
                for (;;) {
                    char last[8192];
                    ssize_t count = read(master, last, sizeof(last));
                    if (count <= 0) {
                        break;
                    }
                    if (write_all(STDOUT_FILENO, last, (size_t)count) != 0) {
                        break;
                    }
                }
                break;
            }
        }
    }

    if (!child_done) {
        (void)kill(-child, SIGTERM);
        (void)kill(child, SIGTERM);
        (void)waitpid(child, NULL, 0);
    }
    close(master);
    return 0;
}
