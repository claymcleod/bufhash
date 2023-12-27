use std::hash::Hasher as _;

use bufhash::partitioned::Hasher;
use bufhash::PartitionedHasher;

#[derive(Debug, Default)]
pub struct Murmur3 {
    hash: u32,
    num_bytes: usize,
}

const C1: u32 = 0xCC9E_2D51;
const C2: u32 = 0x1B87_3593;
const R1: u32 = 15;
const R2: u32 = 13;
const M: u32 = 5;
const N: u32 = 0xE654_6B64;

impl Hasher<4> for Murmur3 {
    fn write(&mut self, bytes: &[u8; 4]) {
        let mut k = u32::from_le_bytes(*bytes);

        k = k.wrapping_mul(C1);
        k = k.rotate_left(R1);
        k = k.wrapping_mul(C2);

        self.hash ^= k;
        self.hash = self.hash.rotate_left(R2);
        self.hash = self.hash.wrapping_mul(M).wrapping_add(N);

        self.num_bytes += 4;
    }

    fn finish(&self, bytes: &[u8]) -> u64 {
        let mut hash = self.hash;
        let mut num_bytes = self.num_bytes;

        if !bytes.is_empty() {
            let mut buffer = [0u8; 4];

            for (i, &byte) in bytes.iter().enumerate() {
                buffer[i] = byte;
            }
            let mut remaining = u32::from_le_bytes(buffer);

            remaining = remaining.wrapping_mul(C1);
            remaining = remaining.rotate_left(R1);
            remaining = remaining.wrapping_mul(C2);

            hash ^= remaining;
            num_bytes += bytes.len();
        }

        hash ^= num_bytes as u32;

        hash ^= hash >> 16;
        hash = hash.wrapping_mul(0x85EB_CA6B);
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0xC2B2_AE35);
        hash ^= hash >> 16;

        hash as u64
    }
}

pub fn main() {
    let mut hasher = PartitionedHasher::new(Murmur3::default());
    hasher.write(b"Hello, world!");

    let result = hasher.finish();
    println!("Single-call result:   0x{:X}", &result);
    assert_eq!(result, 0xC036_3E43);

    let mut hasher = PartitionedHasher::new(Murmur3::default());
    hasher.write(b"H");
    hasher.write(b"e");
    hasher.write(b"l");
    hasher.write(b"l");
    hasher.write(b"o");
    hasher.write(b",");
    hasher.write(b" ");
    hasher.write(b"w");
    hasher.write(b"o");
    hasher.write(b"r");
    hasher.write(b"l");
    hasher.write(b"d");
    hasher.write(b"!");

    let result = hasher.finish();
    println!("Multiple-call result: 0x{:X}", &result);
    assert_eq!(result, 0xC036_3E43);
}
