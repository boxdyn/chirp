// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Traits for automatically serializing and deserializing Rust primitive types.
//!
//! Users of this module should impl [Grab]`<u8>` for their type, which notably returns `&[u8]` and `&mut [u8]`

#[allow(unused_imports)]
use core::mem::size_of;
use std::{fmt::Debug, slice::SliceIndex};

/// Get Raw Bytes at [SliceIndex] `I`.
///
/// This is similar to the [SliceIndex] method `.get(...)`, however implementing this
/// trait will auto-impl [AutoCast]<([i8], [u8], [i16], [u16] ... [i128], [u128])>
pub trait Grab {
    /// Gets the slice of Self at [SliceIndex] I
    fn grab<I>(&self, index: I) -> Option<&<I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>;

    /// Gets a mutable slice of Self at [SliceIndex] I
    fn grab_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[u8]>>::Output>
    where
        I: SliceIndex<[u8]>;
}

/// Read or Write a T at address `addr`
pub trait AutoCast<T>: FallibleAutoCast<T> {
    /// Reads a T from address `addr`
    ///
    /// # May Panic
    ///
    /// This will panic on error. For a non-panicking implementation, do it yourself.
    fn read(&self, addr: impl Into<usize>) -> T {
        self.read_fallible(addr).unwrap_or_else(|e| panic!("{e:?}"))
    }
    /// Writes a T to address `addr`
    ///
    /// # Will Panic
    ///
    /// This will panic on error. For a non-panicking implementation, do it yourself.
    fn write(&mut self, addr: impl Into<usize>, data: T) {
        self.write_fallible(addr, data)
            .unwrap_or_else(|e| panic!("{e:?}"));
    }
}

/// Read a T from address `addr`, and return the value as a [Result]
pub trait FallibleAutoCast<T>: Grab {
    /// The [Err] type
    type Error: Debug;
    /// Read a T from address `addr`, returning the value as a [Result]
    fn read_fallible(&self, addr: impl Into<usize>) -> Result<T, Self::Error>;
    /// Write a T to address `addr`, returning the value as a [Result]
    fn write_fallible(&mut self, addr: impl Into<usize>, data: T) -> Result<(), Self::Error>;
}

/// Implements Read and Write for the provided types
///
/// Relies on inherent methods of Rust numeric types:
/// - `Self::from_be_bytes`
/// - `Self::to_be_bytes`
macro_rules! impl_rw {($($t:ty) ,* $(,)?) =>{
    $(
        #[doc = concat!("Read or Write [`", stringify!($t), "`] at address `addr`, *discarding errors*.\n\nThis will never panic.")]
        impl<T: Grab + FallibleAutoCast<$t>> AutoCast<$t> for T {
            #[inline(always)]
            fn read(&self, addr: impl Into<usize>) -> $t {
                self.read_fallible(addr).ok().unwrap_or_default()
            }
            #[inline(always)]
            fn write(&mut self, addr: impl Into<usize>, data: $t) {
                self.write_fallible(addr, data).ok();
            }
        }
        impl<T: Grab> FallibleAutoCast<$t> for T {
            type Error = $crate::error::Error;
            #[inline(always)]
            fn read_fallible(&self, addr: impl Into<usize>) -> $crate::error::Result<$t> {
                let addr: usize = addr.into();
                let top = addr + core::mem::size_of::<$t>();
                if let Some(bytes) = self.grab(addr..top) {
                    // Chip-8 is a big-endian system
                    Ok(<$t>::from_be_bytes(bytes.try_into()?))
                } else {
                    Err($crate::error::Error::InvalidAddressRange{range: (addr..top).into()})
                }
            }
            #[inline(always)]
            fn write_fallible(&mut self, addr: impl Into<usize>, data: $t) -> std::result::Result<(), Self::Error> {
                let addr: usize = addr.into();
                if let Some(slice) = self.grab_mut(addr..addr + core::mem::size_of::<$t>()) {
                    // Chip-8 is a big-endian system
                    data.to_be_bytes().as_mut().swap_with_slice(slice);
                }
                Ok(())
            }
        }
    )*
}}

// Using macro to be "generic" over types without traits in common
impl_rw!(i8, i16, i32, i64, i128);
impl_rw!(u8, u16, u32, u64, u128);
impl_rw!(f32, f64);
