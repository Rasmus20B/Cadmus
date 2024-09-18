#!/usr/bin/env python3

import sys


# pub fn get_int(bytes: &[u8]) -> usize {
#     return match bytes.len() {
#         1 => bytes[0] as usize,
#         2 => ((bytes[0] as usize) << 8) + (bytes[1] as usize),
#         4 => (get_int(&bytes[0..2]) << 16) + get_int(&bytes[2..4]),
#         _ => 0
#     }
# }

def get_int(n):
    ns = [int(x) for x in n]
    match ns.__len__():
        case 1:
            return ns[0]
        case 2:
            return ((ns[0]) << 8) + (ns[1])
        case 4:
            return (get_int(ns[0:2]) << 16) + get_int(ns[2:4])


if __name__ == "__main__":
    n = sys.argv[1]
    ns = n.split(", ")
    print(get_int(ns))
    pass
