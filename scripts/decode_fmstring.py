#!/usr/bin/env python3

import sys


def punctuation(c):
    match c:
        case 50:
            return []
        case 65:
            return [58 ^ 0x5a]
        case 116:
            return [46 ^ 0x5a]
        case 218:
            return [117]
        case 219:
            return [46 ^ 0x5a, 46 ^ 0x5a]
        case _:
            return [c]


def string_decrypt(byte_list: [int]):
    print(len(byte_list) - 12)
    punctuated = map(punctuation, byte_list)
    flatten = [c for string in punctuated for c in string]
    full = ([chr(c ^ 0x5A) for c in flatten])
    return "".join([c for c in full if c.isprintable()])


if __name__ == "__main__":
    n = [int(x) for x in sys.argv[1].split(',')]
    s = string_decrypt(n)
    print(s)
