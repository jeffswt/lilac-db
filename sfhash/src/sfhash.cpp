
#include <cstdlib>
#include "./sfhash.h"

typedef int32_t i32;
typedef uint8_t u8;
typedef uint32_t u32;
typedef uint64_t u64;

const u64 MAGIC_1 = 0xf123456789abcdefULL;
const u64 MAGIC_2 = 0x1727267628383583ULL;
const u64 MAGIC_3 = 0x7654567654323456ULL;

const u32 MAGIC_MIX = 0xf4325c37UL;

static inline u64 mix(u64 v) {
	v ^= v >> 23;
    u32 v_1 = v >> 32,
        v_2 = v;
    v_1 *= 0xf4325c37UL;
    v_2 *= 0xf4325c37UL;
    v = ((u64)v_1 << 32) ^ v_2;
	v ^= v >> 47;
	return v;
}

static inline u64 rotate_right(u64 val, u64 digs) {
    u64 mask = (1 << digs) - 1;
    return (val >> digs) ^ ((val & mask) << (64 - digs));
}

void sfhash64(const void *buf, i32 len, u32 _seed, void *out) {
    const u64 seed = ((u64)_seed << 32) ^ _seed;
    const u64 magic = 0x880355f21e6d1965ULL;

    const u64 *ptr = (const u64*)(buf);
    const u64 *end1 = ptr + ((len >> 5) << 2);
    const u64 *end2 = ptr + (len >> 3);

    u64 h3 = seed ^ (len * magic),
        h2 = h3 - MAGIC_1,
        h1 = h2 + MAGIC_2,
        h4 = h3 + MAGIC_1;
    u64 v1, v2, v3, v4, v;

    while (ptr != end1) {
        // v = *ptr;
        v1 = *ptr;
        v2 = *(ptr + 1);
        v3 = *(ptr + 2);
        v4 = *(ptr + 3);
        // v ^= v >> 23;
        v1 ^= v1 >> 23;
        v2 ^= v2 >> 23;
        v3 ^= v3 >> 23;
        v4 ^= v4 >> 23;
        // u32 v_1 = v >> 32,
        //     v_2 = v;
        u32 v1_1 = v1 >> 32,
            v1_2 = v1;
        u32 v2_1 = v2 >> 32,
            v2_2 = v2;
        u32 v3_1 = v3 >> 32,
            v3_2 = v3;
        u32 v4_1 = v4 >> 32,
            v4_2 = v4;
        // v_1 *= MAGIC_MIX;
        // v_2 *= MAGIC_MIX;
        v1_1 *= MAGIC_MIX;
        v1_2 *= MAGIC_MIX;
        v2_1 *= MAGIC_MIX;
        v2_2 *= MAGIC_MIX;
        v3_1 *= MAGIC_MIX;
        v3_2 *= MAGIC_MIX;
        v4_1 *= MAGIC_MIX;
        v4_2 *= MAGIC_MIX;
        // v = ((u64)v_1 << 32) ^ v_2;
        v1 = ((u64)v1_1 << 32) ^ v1_2;
        v2 = ((u64)v2_1 << 32) ^ v2_2;
        v3 = ((u64)v3_1 << 32) ^ v3_2;
        v4 = ((u64)v4_1 << 32) ^ v4_2;
        // v ^= v >> 47;
        // h ^= mix(v);
        h1 ^= v1 ^ (v1 >> 47);
        h2 ^= v2 ^ (v2 >> 47);
        h3 ^= v3 ^ (v3 >> 47);
        h4 ^= v4 ^ (v4 >> 47);
        // h *= magic;
        h1 *= magic;
        h2 *= magic;
        h3 *= magic;
        h4 *= magic;
        // ptr++;
        ptr += 4;
    }

    u64 h = rotate_right(h1, 1);
    h *= MAGIC_3;
    h ^= rotate_right(h2, 3);
    h *= MAGIC_3;
    h ^= rotate_right(h3, 6);
    h *= MAGIC_3;
    h ^= rotate_right(h4, 11);

	while (ptr != end2) {
		v = *ptr;
		h ^= mix(v);
		h *= magic;
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
    h *= magic;

    *(u64*)out = mix(h);
}
