#define _GNU_SOURCE
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* Linux runner: report the measured child's rusage, excluding this wrapper.
 * Forking from a small native wrapper avoids attributing Python's pre-exec
 * resident set to the measured child. stdout/stderr remain the child's streams.
 */
static double seconds(struct timespec t) { return t.tv_sec + t.tv_nsec / 1e9; }
static double cpu_ms(struct timeval t) { return t.tv_sec * 1000.0 + t.tv_usec / 1000.0; }

int main(int argc, char **argv) {
    if (argc < 3) { fprintf(stderr, "usage: resource-measure REPORT COMMAND [ARGS...]\n"); return 2; }
    struct timespec start, end;
    struct rusage usage;
    int status;
    if (clock_gettime(CLOCK_MONOTONIC, &start)) { perror("clock_gettime"); return 2; }
    pid_t pid = fork();
    if (pid < 0) { perror("fork"); return 2; }
    if (pid == 0) {
        execvp(argv[2], &argv[2]);
        perror("execvp");
        _exit(127);
    }
    while (wait4(pid, &status, 0, &usage) < 0) {
        if (errno != EINTR) { perror("wait4"); return 2; }
    }
    if (clock_gettime(CLOCK_MONOTONIC, &end)) { perror("clock_gettime"); return 2; }
    double wall = (seconds(end) - seconds(start)) * 1000.0;
    double user = cpu_ms(usage.ru_utime), system = cpu_ms(usage.ru_stime);
    int code = WIFEXITED(status) ? WEXITSTATUS(status) : 128 + WTERMSIG(status);
    FILE *out = fopen(argv[1], "w");
    if (!out) { perror("fopen"); return 2; }
    fprintf(out,
        "{\"wall_ms\":%.9f,\"user_ms\":%.6f,\"system_ms\":%.6f,"
        "\"cpu_ms\":%.6f,\"cpu_percent\":%.6f,\"peak_rss_kib\":%ld,"
        "\"minor_faults\":%ld,\"major_faults\":%ld,\"voluntary_context_switches\":%ld,"
        "\"involuntary_context_switches\":%ld,\"input_blocks\":%ld,\"output_blocks\":%ld,\"exit_code\":%d}\n",
        wall, user, system, user + system, wall > 0 ? (user + system) * 100.0 / wall : 0,
        usage.ru_maxrss, usage.ru_minflt, usage.ru_majflt, usage.ru_nvcsw,
        usage.ru_nivcsw, usage.ru_inblock, usage.ru_oublock, code);
    if (fclose(out)) { perror("fclose"); return 2; }
    return code;
}
