#!/usr/bin/env python3

import sys
import struct
import decode_int

# possible that size of segment data is stored in a key-val
# with key 0 in big-endian integer.


class Chunk:
    path = str
    instruction = int
    reference = str
    size = int
    payload_b = bytes
    payload = str

    def init(self, line):
        if line.startswith("sector"):
            return -1
        parts = line.rstrip('\n').split('::')
        pairs = []

        for p in parts:
            pairs.append(p.split(':'))

        self.path = pairs[0][1]
        self.reference = pairs[1][1]

        s = (pairs[2][1].lstrip('[').rstrip(']').strip('').split(", "))
        payload = [int(n).to_bytes() for n in s if n.strip().isdigit()]
        print(payload, s)
        self.payload_b = b''.join(payload)
        self.payload = s
        self.size = int(pairs[3][1])
        self.instruction = int(pairs[4][1], 16)


def flush_segment(seg_map, total):
    with open("segment." + str(total), 'wb') as file:
        for key in sorted(seg_map.keys()):
            payload = seg_map[key]
            file.write(payload)


def concat_segments(path):
    with open(path) as file:
        seg_map = {}
        projected_size = 0
        current_size = 0
        current_path = ""
        total_datas = 0
        in_segment = False

        for line in file:
            chunk = Chunk()
            res = chunk.init(line)
            if res == -1:
                continue

            if chunk.instruction == 0x03:
                in_segment = False
                current_size = 0
                projected_size = 0
                if chunk.reference == "0":
                    n = decode_int.get_int(chunk.payload)
                    projected_size = n
                    current_path = chunk.path
            elif chunk.instruction not in [0x7, 0x20, 0x38, 0x40, 0x80]:
                if in_segment is True:
                    in_segment = False
                    print(f"segment: {current_path}"
                          "size: {current_size} / {projected_size}")
                    current_path = ""
                    current_size = 0
                    projected_size = 0
                    flush_segment(seg_map, total_datas)
                    seg_map.clear()
                    total_datas += 1
            elif chunk.instruction == 0x7:
                if chunk.path != current_path:
                    projected_size = 0

                seg_map[chunk.reference] = (chunk.payload_b)
                current_path = chunk.path
                current_size += chunk.size

                if in_segment is False:

                    in_segment = True


if __name__ == "__main__":
    path = sys.argv[1]
    concat_segments(path)
