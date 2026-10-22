// SPDX-License-Identifier: Apache-2.0
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>
int unlink(const char *p) { if (getenv("RACOON_RESTORE_UNLINK_FAIL") && strstr(p, "r03-live.db-wal")) { errno = EACCES; return -1; } return ((int (*)(const char *))dlsym(RTLD_NEXT, "unlink"))(p); }
int rename(const char *from, const char *to) {
    const char *fault = getenv("RACOON_RESTORE_RENAME_FAIL");
    if (fault && ((!strcmp(fault, "wal") && strstr(from, "r03-live.db-wal")) ||
        ((!strcmp(fault, "shm") || !strcmp(fault, "rollback")) && strstr(from, "r03-live.db-shm")) ||
        ((!strcmp(fault, "journal") || !strcmp(fault, "rollback-shm")) && strstr(from, "r03-live.db-journal")) ||
        (!strcmp(fault, "rollback-shm") && strstr(from, "detached.db-shm") && strstr(to, "r03-live.db-shm")) ||
        (!strcmp(fault, "publish") && strstr(from, "replacement.db")) ||
        (!strcmp(fault, "rollback") && (strstr(from, "detached.db-wal") || strstr(from, "recovering.db-wal")) && strstr(to, "r03-live.db-wal")))) {
        errno = EACCES; return -1;
    }
    return ((int (*)(const char *, const char *))dlsym(RTLD_NEXT, "rename"))(from, to);
}
