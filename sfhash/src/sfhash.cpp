
#include <cstdlib>
#include "./sfhash.h"

typedef int32_t i32;
typedef uint8_t u8;
typedef uint32_t u32;
typedef uint64_t u64;

const u64 MAGIC_SEED = 0xbc4a78eb0e083fb5ULL;
const u64 MAGIC_SHIFT_1 = 0xc2d4f379500c363fULL;
const u64 MAGIC_SHIFT_2 = 0xa696a85adffcf585ULL;
const u64 MAGIC_SHIFT_3 = 0xfcb5791894673fd3ULL;
const u64 MAGIC_SHIFT_4 = 0xb828e5548ad84c69ULL;
const u64 MAGIC_OFFSET_1 = 0xbaf1c97b495f954fULL;
const u64 MAGIC_OFFSET_2 = 0xaa7c10d3d42ee0b7ULL;
const u64 MAGIC_MIX = 0xfe504c379a62302fULL;

static inline u64 mix(u64 v) {
    v ^= v >> 23;
    v *= MAGIC_MIX;
    v ^= v >> 47;
    return v;
}

static inline u64 rotate_right(u64 val, u64 digs) {
    u64 mask = (1 << digs) - 1;
    return (val >> digs) ^ ((val & mask) << (64 - digs));
}

void sfhash64(const void *buf, i32 len, u32 _seed, void *out) {
    const u64 *ptr = (const u64*)(buf);
    const u64 *end1 = ptr + ((len >> 5) << 2);
    const u64 *end2 = ptr + (len >> 3);

    u64 h3 = MAGIC_SEED ^ _seed ^ (len * MAGIC_SHIFT_1);
    u64 h, v;
    if (ptr != end1) {
        u64 h2 = h3 + MAGIC_OFFSET_1,
            h1 = h2 + MAGIC_OFFSET_2,
            h4 = h3 - MAGIC_OFFSET_2;
        u64 v1, v2, v3, v4, v;

        while (ptr != end1) {
            // v = *ptr;
            v1 = *ptr;
            v2 = *(ptr + 1);
            v3 = *(ptr + 2);
            v4 = *(ptr + 3);
            // v ^= v >> 23
            v1 ^= v1 >> 23;
            v2 ^= v2 >> 23;
            v3 ^= v3 >> 23;
            v4 ^= v4 >> 23;
            // v ^= v >> 47
            v1 ^= v1 >> 47;
            v2 ^= v2 >> 47;
            v3 ^= v3 >> 47;
            v4 ^= v4 >> 47;
            // h ^= v;
            h1 ^= v1;
            h2 ^= v2;
            h3 ^= v3;
            h4 ^= v4;
            // h *= magic;
            h1 *= MAGIC_SHIFT_1;
            h2 *= MAGIC_SHIFT_2;
            h3 *= MAGIC_SHIFT_3;
            h4 *= MAGIC_SHIFT_4;
            // ptr++;
            ptr += 4;
        }
        h = rotate_right(h1, 1);
        h ^= rotate_right(h2, 3);
        h ^= rotate_right(h3, 6);
        h ^= rotate_right(h4, 11);
    } else {
        h = h3;
    }

    // remaining at most 31 bytes
    // branch predictors should be able to predict this loop
    // so we aren't unwrapping it anyway
    while (ptr != end2) {
		v = *ptr;
        h ^= mix(v);
        h *= MAGIC_SHIFT_1;
        ptr++;
	}

    const u8 *ptr2 = (const u8*)ptr;
    v = 0;

    switch (len & 7) {
        case 7: v ^= (u64)ptr2[6] << 48;
        case 6: v ^= (u64)ptr2[5] << 40;
        case 5: v ^= (u64)ptr2[4] << 32;
        case 4: v ^= (u64)ptr2[3] << 24;
        case 3: v ^= (u64)ptr2[2] << 16;
        case 2: v ^= (u64)ptr2[1] << 8;
        case 1: v ^= (u64)ptr2[0];
    }

    h ^= mix(v);
    h *= MAGIC_SHIFT_4;

    *(u64*)out = mix(h);
}
