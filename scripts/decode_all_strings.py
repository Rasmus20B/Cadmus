#!/usr/bin/env python3

import decode_fmstring
import sys


def decode_file(path):
    with open(path, 'r') as file:
        for line in file:
            parts = line.rstrip('\n').split('::')
            pairs = []
            for p in parts:
                pairs.append(p.split(':'))

            if pairs[1][1] == "Some(16)":
                text = pairs[2][1].lstrip('[').rstrip(']')
                n = [int(x) for x in text.split(',')]
                string = decode_fmstring.string_decrypt(n)
                print(f"{pairs[0][1]}: {string}")


if __name__ == "__main__":
    path = sys.argv[1]
    decode_file(path)
