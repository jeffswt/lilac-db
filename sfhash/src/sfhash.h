
#ifndef _SFHASH64_H
#define _SFHASH64_H

#include <stdint.h>
#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

void sfhash64(const void *buf, int len, uint32_t seed, void *out);
void sfhash32(const void *buf, int len, uint32_t seed, void *out);

#ifdef __cplusplus
}
#endif

#endif
