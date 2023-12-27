/// A partitioned hasher.
///
/// See the [crate documentation](../index.html) for more details.
#[allow(missing_docs)]
pub trait Hasher<const S: usize> {
    fn write(&mut self, bytes: &[u8; S]);
    fn finish(&self, bytes: &[u8]) -> u64;
}
