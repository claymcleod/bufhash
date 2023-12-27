use std::hash::Hasher as _;

#[derive(Debug, Default)]
pub struct Simple(u64);

impl bufhash::partitioned::Hasher<8> for Simple {
    fn write(&mut self, bytes: &[u8; 8]) {
        let data = u64::from_le_bytes(*bytes);
        self.0 = self.0.wrapping_add(data);
    }

    fn finish(&self, bytes: &[u8]) -> u64 {
        self.0 << bytes.len()
    }
}

pub fn main() {
    let mut hasher = bufhash::PartitionedHasher::new(Simple::default());
    hasher.write(b"Hello, world!");
    println!("Result: {:#X}", hasher.finish());
}
