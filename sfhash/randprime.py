# python3 -m pip install sympy
import random
import sympy


def genprime(bits: int) -> int:
    return sympy.randprime(2**(bits - 3) * 5, 2**bits)


if __name__ == '__main__':
    random.seed(23333)
    fields = [
        ('MAGIC_SEED', 64),
        ('MAGIC_SHIFT_1', 64),
        ('MAGIC_SHIFT_2', 64),
        ('MAGIC_SHIFT_3', 64),
        ('MAGIC_SHIFT_4', 64),
        ('MAGIC_SHIFT32_1', 32),
        ('MAGIC_SHIFT32_2', 32),
        ('MAGIC_SHIFT32_3', 32),
        ('MAGIC_SHIFT32_4', 32),
        ('MAGIC_OFFSET_1', 64),
        ('MAGIC_OFFSET_2', 64),
        ('MAGIC_MIX', 64),
    ]
    for name, bits in fields:
        print(f'const u{bits} {name} = {hex(genprime(bits))}ULL;')
    pass
