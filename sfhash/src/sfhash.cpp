
#include <cstdlib>
#include "./sfhash.h"

typedef int32_t i32;
typedef uint8_t u8;
typedef uint32_t u32;
typedef uint64_t u64;

const u64 MAGIC_1 = 0xf123456789abcdefULL;
const u64 MAGIC_2 = 0x1727267628383583ULL;
const u64 MAGIC_3 = 0x2159f3532159f353ULL;
const u64 MAGIC_4 = 0x8352ed958352ed95ULL;
const u32 MAGIC_5 = 0xaaaababaUL;
const u64 MAGIC_5_64 = MAGIC_5 ^ ((u64)MAGIC_5 << 32);

static inline u64 rotate_right(u64 val, u64 digs) {
    u64 mask = (1 << digs) - 1;
    return (val >> digs) ^ ((val & mask) << (64 - digs));
}

void sfhash64(const void *buf, i32 len, u32 _seed, void *out) {
    // hash the first 32x bytes in groups
    u64 *ptr = (u64*)buf;
    u64 *end = ptr + ((len >> 5) << 2); // align to 32 bytes
    u64 *end2 = ptr + (len >> 3); // align to 8 bytes

    // initial hashes for 4 parallel processes
    u64 h3 = MAGIC_1 ^ (len * MAGIC_2);
    u64 h2 = h3 - MAGIC_3;
    u64 h1 = h2 + MAGIC_4;
    u64 h4 = h3 ^ MAGIC_4;

    // start running
    while (ptr != end) {
        // fetch data
        u64 v1 = *ptr;
        u64 v2 = *(ptr + 1);
        u64 v3 = *(ptr + 2);
        u64 v4 = *(ptr + 3);
        // rshift
        v1 = v1 ^ (v1 >> 13);
        v2 = v2 ^ (v2 >> 13);
        v3 = v3 ^ (v3 >> 13);
        v4 = v4 ^ (v4 >> 13);
        // multiply
        // vectors can be directly cast from 64x4 to and from 32x8 since the
        // endian does not change throughout the reversible process
        u32 v1_1 = v1 >> 32,
            v1_2 = (u32)v1;
        u32 v2_1 = v2 >> 32,
            v2_2 = (u32)v2;
        u32 v3_1 = v3 >> 32,
            v3_2 = (u32)v3;
        u32 v4_1 = v4 >> 32,
            v4_2 = (u32)v4;
        v1_1 *= MAGIC_5;
        v1_2 *= MAGIC_5;
        v2_1 *= MAGIC_5;
        v2_2 *= MAGIC_5;
        v3_1 *= MAGIC_5;
        v3_2 *= MAGIC_5;
        v4_1 *= MAGIC_5;
        v4_2 *= MAGIC_5;
        v1 = ((u64)v1_1 << 32) ^ v1_2;
        v2 = ((u64)v2_1 << 32) ^ v2_2;
        v3 = ((u64)v3_1 << 32) ^ v3_2;
        v4 = ((u64)v4_1 << 32) ^ v4_2;
        // rshift again
        h1 = h1 ^ v1 ^ (v1 >> 37);
        h2 = h2 ^ v2 ^ (v2 >> 37);
        h3 = h3 ^ v3 ^ (v3 >> 37);
        h4 = h4 ^ v4 ^ (v4 >> 37);
        // skip 4 every time
        ptr += 4;
    }

    // last 0 / 8 / 16 / 24 bytes
    // fetch data
    u64 v1, v2, v3, v4;
    if (ptr == end2) {
        v1 = 0; v2 = 0; v3 = 0; v4 = 0;
    } else if (ptr + 1 == end2) {
        v1 = *ptr; v2 = 0; v3 = 0; v4 = 0;
        ptr += 1;
    } else if (ptr + 1 == end2) {
        v1 = *ptr; v2 = *(ptr + 1); v3 = 0; v4 = 0;
        ptr += 2;
    } else {
        v1 = *ptr; v2 = *(ptr + 1); v3 = *(ptr + 2); v4 = 0;
        ptr += 3;
    }
    // rshift
    v1 = v1 ^ (v1 >> 13);
    v2 = v2 ^ (v2 >> 13);
    v3 = v3 ^ (v3 >> 13);
    v4 = v4 ^ (v4 >> 13);
    // multiply
    u32 v1_1 = v1 >> 32,
        v1_2 = (u32)v1;
    u32 v2_1 = v2 >> 32,
        v2_2 = (u32)v2;
    u32 v3_1 = v3 >> 32,
        v3_2 = (u32)v3;
    u32 v4_1 = v4 >> 32,
        v4_2 = (u32)v4;
    v1_1 *= MAGIC_5;
    v1_2 *= MAGIC_5;
    v2_1 *= MAGIC_5;
    v2_2 *= MAGIC_5;
    v3_1 *= MAGIC_5;
    v3_2 *= MAGIC_5;
    v4_1 *= MAGIC_5;
    v4_2 *= MAGIC_5;
    v1 = ((u64)v1_1 << 32) ^ v1_2;
    v2 = ((u64)v2_1 << 32) ^ v2_2;
    v3 = ((u64)v3_1 << 32) ^ v3_2;
    v4 = ((u64)v4_1 << 32) ^ v4_2;
    // rshift again
    h1 = h1 ^ v1 ^ (v1 >> 37);
    h2 = h2 ^ v2 ^ (v2 >> 37);
    h3 = h3 ^ v3 ^ (v3 >> 37);
    h4 = h4 ^ v4 ^ (v4 >> 37);

    // hash the last 8 bytes
    u8 *ptr2 = (u8*)ptr;
    u64 h = (rotate_right(h1, 1) ^ rotate_right(h2, 3)) ^ (rotate_right(h3, 6) ^ rotate_right(h4, 11));
    u64 v = 0;
    switch (len & 7) {
        case 7: v ^= (u64)ptr2[6] << 48;
        case 6: v ^= (u64)ptr2[5] << 40;
        case 5: v ^= (u64)ptr2[4] << 32;
        case 4: v ^= (u64)ptr2[3] << 24;
        case 3: v ^= (u64)ptr2[2] << 16;
        case 2: v ^= (u64)ptr2[1] << 8;
        case 1: v ^= (u64)ptr2[0];
	}
    v = v ^ (v >> 13);
    v *= MAGIC_3;
    h = h ^ v ^ (v >> 37);

    // final flush
    h = h ^ (h >> 13);
    h *= MAGIC_4;
    h = h ^ (h >> 37);
    h *= MAGIC_3;

    // writeback
    *(u64*)out = h;
}
