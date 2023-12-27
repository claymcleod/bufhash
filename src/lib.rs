//! Buffered hashing facilities.
//!
//! This crate provides facilities for writing hashers that take advantage of buffering
//! in various ways. Most notably, it provides the concept of **partitioned** hashers,
//! which are a class of hashers that split data into fixed-sized chunks before hashing.
//! This crate, and all of the facilities therein, are written in 100% Rust and are
//! designed to integrate with the existing Rust traits, such as [`std::hash::Hasher`].
//!
//! ## Overview
//!
//! Partitioned hashers are a class of hashers that work on fixed-size chunks of data
//! (for example, [MurmurHash3]). Fundamentally, the facilities provided under the
//! [`partitioned`] namespace exist to solve the problem of accounting while
//! partitioning data during incremental hashing. Underneath the hood, partitioned
//! hashers keep track of chunking streaming inputs into the appropriate sizes and
//! storing excess data within an internal buffer to be used in the next
//! [`write()`](std::hash::Hasher::write) call.
//!
//! For example, consider the following invocation of a fictional hashing algorithm that
//! works in chunks of four (4) bytes at a time:
//!
//! ```compile_fail
//! let mut hasher = MyHasher::default();
//!
//! hasher.write(b"Hi,");
//! hasher.write(b"there ");
//! hasher.write(b"world!");
//!
//! println!("Result: 0x{:X}", hasher.finish());
//! ```
//!
//! In this example, the data being hashed is incrementally fed to the hasher over three
//! different [`write()`](std::hash::Hasher::write) calls. Recalling that our hasher
//! works on four byte chunks, the first three characters in the byte string literal
//! (i.e., `b"Hi,"`) are not sufficient to fill out a full block of data to be hashed.
//! However, we similarly do not want to treat these three characters as if they are the
//! end of the data stream, as more data may be coming (and, in fact, _is_ in our
//! example. With the facilities provided by the Rust standard library, it is up to the
//! implementer to keep track of these various edge cases and ensure the hasher works
//! appropriately.
//!
//! To ease implementation of hashers that fit the aforementioned profile, [partitioned
//! hashers](crate::partitioned::Hasher) take care of the necessary accounting behind
//! the scenes and allow the implementer to simply focus on a fixed-size interface for
//! hashing.
//!
//! ## Implementing a Partitioned Hasher
//!
//! Let's look at the core [`bufhash::partitioned::Hasher`](crate::partitioned::Hasher)
//! trait to ascertain how writing partitioned hashers is different than writing an
//! implementation for a [`std::hash::Hasher`].
//!
//! ```
//! pub trait Hasher<const S: usize> {
//!    fn write(&mut self, bytes: &[u8; S]);
//!    fn finish(&self, bytes: &[u8]) -> u64;
//! }
//! ```
//!
//! You'll find the same two methods as provided in [`std::hash::Hash`] with a few,
//! minor (but important!) differences:
//!
//! * The trait has a generic parameter `const S: usize`, which defines the number of
//!   bytes per partition. In the case of our example above, we would set this to four
//!   (4) bytes.
//! * The [`write()`](crate::partitioned::Hasher::write) method takes a fixed-size array
//!   of bytes (`&[u8; S]`). The `bytes` parameter is _guaranteed_ to be exactly the
//!   size of the generic parameter, allowing the implementer to forgo size checking.
//! * The [`finish()`](crate::partitioned::Hasher::finish) method also provides a
//!   `bytes` parameter as a byte slice. This parameter are the bytes left over in the
//!   internal buffer that need to be handled before the hasher is finished: it is
//!   guaranteed to have a size less than the generic parameter.
//!
//! As an example, let's implement a partitioned hasher that (a) interprets the byte
//! stream as little-endian `u64`s that must be added together and then (b) shifts the
//! result left by the number of bytes remaining in the buffer when `finish()` is called
//! (not a particularly good hashing algorithm, but it will work for our demonstration
//! purposes).
//!
//! ```
//! #[derive(Debug, Default)]
//! pub struct Simple(u64);
//!
//! impl bufhash::partitioned::Hasher<8> for Simple {
//!     fn write(&mut self, bytes: &[u8; 8]) {
//!         let data = u64::from_le_bytes(*bytes);
//!         self.0 = self.0.wrapping_add(data);
//!     }
//!
//!     fn finish(&self, bytes: &[u8]) -> u64 {
//!         self.0 << bytes.len()
//!     }
//! }
//! ```
//!
//! As you can see, this is much more straightforward to implement and maintain that
//! handling the accounting yourself.
//!
//! To use this new implementation with the [`std::hash::Hasher`] interface, you can use
//! the [`PartitionedHasher`] adapter like so:
//!
//! ```compile_fail
//! let mut hasher = bufhash::PartitionedHasher::new(Simple::default());
//! hasher.write(b"Hello, world!");
//! println!("Result: {:#X}", hasher.finish());
//! ```
//!
//! [MurmurHash3]: https://en.wikipedia.org/wiki/MurmurHash#MurmurHash3

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(rustdoc::broken_intra_doc_links)]

pub mod partitioned;

pub use partitioned::PartitionedHasher;
