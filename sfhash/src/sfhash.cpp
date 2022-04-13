
#include <cstdlib>
#include "./sfhash.h"

typedef int32_t i32;
typedef uint8_t u8;
typedef uint32_t u32;
typedef uint64_t u64;

const u64 MAGIC_SEED = 0x5894f1d70e083f7fULL;
const u64 MAGIC_SHIFT_1 = 0x65a9e6f3500c3625ULL;
const u64 MAGIC_SHIFT_2 = 0x2d2d50b5dffcf597ULL;
const u64 MAGIC_SHIFT_3 = 0xd96af23194673fc3ULL;
const u64 MAGIC_SHIFT_4 = 0x5051caa88ad84c69ULL;
const u64 MAGIC_OFFSET_1 = 0x55e392f6495f951fULL;
const u64 MAGIC_OFFSET_2 = 0xf587327f9c3575f1ULL;
const u64 MAGIC_MIX_1 = 0xe1c91479797ffbffULL;
const u64 MAGIC_MIX_2 = 0x34f821a6d42ee0c1ULL;
const u64 MAGIC_MIX_3 = 0xdca0986e9a623025ULL;
const u64 MAGIC_MIX_4 = 0x452ff2dc5bc471b3ULL;

static inline u64 mix(u64 v) {
    v ^= v >> 17;
    v *= MAGIC_MIX_1;
    v ^= v >> 49;
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

    u64 h3 = MAGIC_SEED ^ (len * MAGIC_SHIFT_1);
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
            // v *= mix
            v1 *= MAGIC_MIX_1;
            v2 *= MAGIC_MIX_2;
            v3 *= MAGIC_MIX_3;
            v4 *= MAGIC_MIX_4;
            // h += v;
            h1 = rotate_right(h1 + v1, 31);
            h2 = rotate_right(h1 + v2, 31);
            h3 = rotate_right(h1 + v3, 31);
            h4 = rotate_right(h1 + v4, 31);
            // h *= shift;
            h1 *= MAGIC_SHIFT_1;
            h2 *= MAGIC_SHIFT_2;
            h3 *= MAGIC_SHIFT_3;
            h4 *= MAGIC_SHIFT_4;
            // ptr++;
            ptr += 4;
        }
        h = rotate_right(h1, 1);
        h += rotate_right(h2, 3);
        h += rotate_right(h3, 6);
        h += rotate_right(h4, 11);
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
