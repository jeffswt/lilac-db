
#include <cstdlib>
#include "./sfhash.h"

typedef int32_t i32;
typedef uint8_t u8;
typedef uint32_t u32;
typedef uint64_t u64;

const u64 MAGIC_1 = 0xf123456789abcdefULL;
const u64 MAGIC_2 = 0x1727267628383583ULL;
const u64 MAGIC_3 = 0x7654567654323456ULL;

static inline u64 mix(u64 h) {				
	h ^= h >> 23;		
    u32 h_1 = h >> 32,
        h_2 = h;
    h_1 *= 0xf4325c37UL;
    h_2 *= 0xf4325c37UL;
    h = ((u64)h_1 << 32) ^ h_2;
	h ^= h >> 47;
	return h;
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
        v1 = *ptr;
        v2 = *(ptr + 1);
        v3 = *(ptr + 2);
        v4 = *(ptr + 3);
        h1 ^= mix(v1);
        h2 ^= mix(v2);
        h3 ^= mix(v3);
        h4 ^= mix(v4);
        h1 *= magic;
        h2 *= magic;
        h3 *= magic;
        h4 *= magic;
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
