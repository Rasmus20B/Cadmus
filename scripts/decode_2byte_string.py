#!/usr/bin/env python3.12

import sys

ENCODING_MAPPING2 = {
        (0x0, 0x0): '0',
        (0x2, 0xa): ' ',
        (0x2, 0x1d): '_',
        (0x12, 0xf): 'a',
        (0x12, 0x25): 'b',
        (0x12, 0x3d): 'c',
        (0x12, 0x50): 'd',
        (0x12, 0x6b): 'e',
        (0x12, 0xa3): 'f',
        (0x12, 0xb0): 'g',
        (0x12, 0xd3): 'h',
        (0x12, 0xec): 'i',
        (0x13, 0x5): 'j',
        (0x13, 0x1e): 'k',
        (0x13, 0x30): 'l',
        (0x13, 0x5f): 'm',
        (0x13, 0x6d): 'n',
        (0x13, 0x8e): 'o',
        (0x13, 0xb3): 'p',
        (0x13, 0xc8): 'q',
        (0x13, 0xda): 'r',
        (0x14, 0x10): 's',
        (0x14, 0x33): 't',
        (0x14, 0x53): 'u',
        (0x14, 0x7b): 'v',
        (0x14, 0x8d): 'w',
        (0x14, 0x97): 'x',
        (0x14, 0x9c): 'y',
        (0x14, 0xad): 'z',
        }


def decode(data: [int]):
    res = ""
    for a, b in zip(data[0::2], data[1::2]):
        res += ENCODING_MAPPING2[(a, b)]
    return res


if __name__ == "__main__":
    n = [int(x) for x in sys.argv[1].split(',')]
    s = decode(n)
    print(s)