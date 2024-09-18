#!/usr/bin/env python3

import sys
import datetime
import struct


def time_decode(byte_list: [int]):
    unpacked, = struct.unpack('<q', bytes(byte_list))
    print(datetime.timedelta(microseconds=unpacked/10.))


if __name__ == "__main__":
    n = [int(x) for x in sys.argv[1].split(',')]
    s = time_decode(n)
    print(s)
