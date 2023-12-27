//! Partitioned hashers.
//!
//! See the [crate documentation](../index.html) for more details.

mod hasher;

pub use hasher::Hasher;

/// A wrapper around a [partitioned hasher](crate::partitioned::Hasher) that implements
/// [`std::hash::Hasher`].
///
/// See the [crate documentation](../index.html) for more details.
#[derive(Clone, Debug)]
pub struct PartitionedHasher<const N: usize, H: Hasher<N>> {
    hasher: H,
    buffer: Vec<u8>,
}

impl<const N: usize, H: Hasher<N>> PartitionedHasher<N, H> {
    /// Creates a new [`PartitionedHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::hash::Hasher as _;
    ///
    /// use bufhash::partitioned::Hasher;
    /// use bufhash::PartitionedHasher;
    ///
    /// #[derive(Debug, Default)]
    /// pub struct Simple(u64);
    ///
    /// impl Hasher<8> for Simple {
    ///     fn write(&mut self, bytes: &[u8; 8]) {
    ///         let data = u64::from_le_bytes(*bytes);
    ///         self.0 = self.0.wrapping_add(data);
    ///     }
    ///
    ///     fn finish(&self, bytes: &[u8]) -> u64 {
    ///         self.0 << bytes.len()
    ///     }
    /// }
    ///
    ///
    /// let mut hasher = PartitionedHasher::new(Simple::default());
    /// hasher.write(b"Hello, world!");
    /// assert_eq!(hasher.finish(), 0xE405_8DED_8D8C_A900);
    /// ```
    pub fn new(hasher: H) -> Self {
        Self {
            hasher,
            buffer: Vec::with_capacity(N),
        }
    }

    #[cfg(test)]
    pub fn inner_vec(&self) -> &Vec<u8> {
        &self.buffer
    }
}

impl<const N: usize, H: Hasher<N>> std::default::Default for PartitionedHasher<N, H>
where
    H: std::default::Default,
{
    fn default() -> Self {
        Self {
            hasher: Default::default(),
            buffer: Vec::with_capacity(N),
        }
    }
}

impl<const N: usize, H: Hasher<N>> core::hash::Hasher for PartitionedHasher<N, H> {
    fn write(&mut self, bytes: &[u8]) {
        let mut bytes = bytes;

        // (1) Checks the buffer to see if any bytes are waiting to be processed. If
        // they are, attempt to process them.
        if !self.buffer.is_empty() {
            let needed = N - self.buffer.len();
            let borrowed = bytes.get(..needed).unwrap_or(bytes);

            self.buffer.extend_from_slice(borrowed);

            // Not enough elements to fill up the buffer, so just return (as the new
            // elements were just added to [`self.buffer`]).
            if self.buffer.len() != N {
                return;
            }

            let mut buffer = [0u8; N];
            buffer.copy_from_slice(&self.buffer[..N]);
            self.hasher.write(&buffer);
            self.buffer.clear();

            // Advance the bytes pointer by the number of elements we consumed to
            // complete the buffer.
            bytes = &bytes[borrowed.len()..]
        }

        // (2) Consume as many of the bytes as is possible.
        while bytes.len() >= N {
            let (chunk, remaining) = bytes.split_at(N);
            // SAFETY: we just ensured that bytes has at least N elements and that
            // `chunk` was split exactly at N elements. As such, this cast will always
            // unwrap.
            self.hasher.write(chunk.try_into().unwrap());
            bytes = remaining;
        }

        // (3) If there are remaining bytes, add them, to the buffer for the next write.
        if !bytes.is_empty() {
            self.buffer.extend_from_slice(bytes);
        }
    }

    fn finish(&self) -> u64 {
        self.hasher.finish(&self.buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher as _;

    use super::*;

    #[derive(Debug, Default)]
    pub struct Simple(u64);

    impl Hasher<8> for Simple {
        fn write(&mut self, bytes: &[u8; 8]) {
            let data = u64::from_le_bytes(*bytes);
            self.0 = self.0.wrapping_add(data);
        }

        fn finish(&self, bytes: &[u8]) -> u64 {
            self.0 << bytes.len()
        }
    }

    #[test]
    fn it_works_with_single_call() {
        let mut hasher = PartitionedHasher::new(Simple::default());
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 0);

        hasher.write(b"Hello, world!");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 5);

        assert_eq!(hasher.finish(), 0xE405_8DED_8D8C_A900);
    }

    #[test]
    fn it_works_with_multiple_calls() {
        let mut hasher = PartitionedHasher::new(Simple::default());
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 0);

        hasher.write(b"H");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 1);

        hasher.write(b"e");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 2);

        hasher.write(b"l");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 3);

        hasher.write(b"l");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 4);

        hasher.write(b"o");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 5);

        hasher.write(b",");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 6);

        hasher.write(b" ");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 7);

        hasher.write(b"w");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 0);

        hasher.write(b"o");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 1);

        hasher.write(b"r");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 2);

        hasher.write(b"l");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 3);

        hasher.write(b"d");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 4);

        hasher.write(b"!");
        assert_eq!(hasher.inner_vec().capacity(), 8);
        assert_eq!(hasher.inner_vec().len(), 5);

        assert_eq!(hasher.finish(), 0xE405_8DED_8D8C_A900);
    }
}
