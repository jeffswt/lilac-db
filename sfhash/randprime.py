# python3 -m pip install sympy
import random
import sympy


def genprime(bits: int) -> int:
    return sympy.randprime(2**(bits - 3), 2**bits)


if __name__ == '__main__':
    random.seed(23333)
    print(f'const u64 MAGIC_SEED = {hex(genprime(64))}ULL;')
    print(f'const u32 MAGIC_SHIFT_32 = {hex(genprime(32))}UL;')
    print(f'const u64 MAGIC_SHIFT_64 = {hex(genprime(64))}ULL;')
    print(f'const u64 MAGIC_OFFSET_1 = {hex(genprime(64))}ULL;')
    print(f'const u64 MAGIC_OFFSET_2 = {hex(genprime(64))}ULL;')
    print(f'const u32 MAGIC_MIX_32 = {hex(genprime(32))}UL;')
    print(f'const u64 MAGIC_MIX_64 = {hex(genprime(64))}ULL;')
