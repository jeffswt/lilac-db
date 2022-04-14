
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
const u32 MAGIC_SHIFT32_1 = 0xc4afca95UL;
const u32 MAGIC_SHIFT32_2 = 0xbaf1c985UL;
const u32 MAGIC_SHIFT32_3 = 0xee1abb0fUL;
const u32 MAGIC_SHIFT32_4 = 0xdcbffdebUL;
const u64 MAGIC_OFFSET_1 = 0xff43a9d0c1c914cdULL;
const u64 MAGIC_OFFSET_2 = 0xf049ed58f79e6153ULL;
const u64 MAGIC_MIX = 0xed27a0e9f72a6d47ULL;

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
    const u64 *end2 = ptr + (len >> 3);
    u64 h3 = MAGIC_SEED ^ _seed ^ ((u64)_seed << 32) ^ (len * MAGIC_SHIFT_1);
    u64 h, v;
    
    // get rid of the excessive operations
    if (len < 32) {
        h = h3;

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
        return;
    }

    const u64 *end1 = ptr + ((len >> 5) << 2);
    
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

// sfHash32 uses AVX2 for vector multiplication over 32*8 uints.
void sfhash32(const void *buf, i32 len, u32 _seed, void *out) {
    const u64 *ptr = (const u64*)(buf);
    const u64 *end2 = ptr + (len >> 3);
    u64 h3 = MAGIC_SEED ^ _seed ^ ((u64)_seed << 32) ^ (len * MAGIC_SHIFT_1);
    u64 h, v;
    
    // get rid of the excessive operations
    if (len < 32) {
        h = h3;

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
        return;
    }

    const u64 *end1 = ptr + ((len >> 5) << 2);
    
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
            u32 h1_1 = h1 >> 32, h1_2 = h1,
                h2_1 = h2 >> 32, h2_2 = h2,
                h3_1 = h3 >> 32, h3_2 = h3,
                h4_1 = h4 >> 32, h4_2 = h4;
            h1_1 *= MAGIC_SHIFT32_1;
            h1_2 *= MAGIC_SHIFT32_1;
            h2_1 *= MAGIC_SHIFT32_2;
            h2_2 *= MAGIC_SHIFT32_2;
            h3_1 *= MAGIC_SHIFT32_3;
            h3_2 *= MAGIC_SHIFT32_3;
            h4_1 *= MAGIC_SHIFT32_4;
            h4_2 *= MAGIC_SHIFT32_4;
            h1 = ((u64)h1_1 << 32) ^ h1_2,
            h2 = ((u64)h2_1 << 32) ^ h2_2,
            h3 = ((u64)h3_1 << 32) ^ h3_2,
            h4 = ((u64)h4_1 << 32) ^ h4_2,
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

    u32 h_32 = h ^ (h >> 32);
    *(u32*)out = h_32;
}
