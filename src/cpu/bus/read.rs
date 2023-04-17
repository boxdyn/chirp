// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE.txt for details)

//! Trait for getting a generic integer for a structure.

#[allow(unused_imports)]
use core::mem::size_of;
use std::{fmt::Debug, slice::SliceIndex};

/// Gets a `&[T]` at [SliceIndex] `I`.
///
/// This is similar to the [SliceIndex] method `.get(...)`, however implementing this trait
/// for [u8] will auto-impl [Read]<(i8, u8, i16, u16 ... i128, u128)>
pub trait Get<T> {
    /// Gets the slice of Self at [SliceIndex] I
    fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>;

    /// Gets a mutable slice of Self at [SliceIndex] I
    fn get_mut<I>(&mut self, index: I) -> Option<&mut <I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>;
}

/// Read a T from address `addr`
pub trait ReadWrite<T>: FallibleReadWrite<T> {
    /// Reads a T from address `addr`
    ///
    /// # May Panic
    ///
    /// If the type is not [Default], this will panic on error.
    fn read(&self, addr: impl Into<usize>) -> T {
        self.read_fallible(addr).unwrap_or_else(|e| panic!("{e:?}"))
    }
    /// Write a T to address `addr`
    fn write(&mut self, addr: impl Into<usize>, data: T) {
        self.write_fallible(addr, data)
            .unwrap_or_else(|e| panic!("{e:?}"));
    }
}

/// Read a T from address `addr`, and return the value as a [Result]
pub trait FallibleReadWrite<T>: Get<u8> {
    /// The [Err] type returned by [read_fallible]
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
macro_rules! impl_rw {
    ($($t:ty) ,* $(,)?) => {
        $(
            impl<T: Get<u8> + FallibleReadWrite<$t>> ReadWrite<$t> for T {
                #[inline(always)]
                fn read(&self, addr: impl Into<usize>) -> $t {
                    self.read_fallible(addr).ok().unwrap_or_default()
                }
                #[inline(always)]
                fn write(&mut self, addr: impl Into<usize>, data: $t) {
                    self.write_fallible(addr, data).ok();
                }
            }
            impl<T: Get<u8>> FallibleReadWrite<$t> for T {
                type Error = $crate::error::Error;
                #[inline(always)]
                fn read_fallible(&self, addr: impl Into<usize>) -> $crate::error::Result<$t> {
                    let addr: usize = addr.into();
                    let range = addr..addr + core::mem::size_of::<$t>();
                    if let Some(bytes) = self.get(range.clone()) {
                        // Chip-8 is a big-endian system
                        Ok(<$t>::from_be_bytes(bytes.try_into()?))
                    } else {
                        Err($crate::error::Error::InvalidAddressRange{range})
                    }
                }
                #[inline(always)]
                fn write_fallible(&mut self, addr: impl Into<usize>, data: $t) -> std::result::Result<(), Self::Error> {
                    let addr: usize = addr.into();
                    if let Some(slice) = self.get_mut(addr..addr + core::mem::size_of::<$t>()) {
                        // Chip-8 is a big-endian system
                        data.to_be_bytes().as_mut().swap_with_slice(slice);
                    }
                    Ok(())
                }
            }
        )*
    };
}

impl_rw!(i8, i16, i32, i64, i128);
impl_rw!(u8, u16, u32, u64, u128);
impl_rw!(f32, f64);
