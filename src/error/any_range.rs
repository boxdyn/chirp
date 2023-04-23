// (c) 2023 John A. Breaux
// This code is licensed under MIT license (see LICENSE for details)

//! Holder for any [Range]

// This is super over-engineered, considering it was originally meant to help with only one LOC
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use thiserror::Error;

#[macro_use]
mod macros;

#[derive(Clone, Debug, Error, PartialEq, Eq, Hash)]
#[error("Failed to convert variant {0} back into range.")]
/// Emitted when conversion back into a [std::ops]::Range\* fails.
pub struct AnyRangeError<Idx>(AnyRange<Idx>);

/// Holder for any [Range]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnyRange<Idx> {
    /// Bounded exclusive [Range] i.e. `0..10`
    Range(Range<Idx>),
    /// Unbounded [RangeFrom], i.e. `0..`
    RangeFrom(RangeFrom<Idx>),
    /// Unbounded [RangeFull], i.e. `..`
    RangeFull(RangeFull),
    /// Bounded inclusive [RangeInclusive], i.e. `0..=10`
    RangeInclusive(RangeInclusive<Idx>),
    /// Unbounded [RangeTo], i.e. `..10`
    RangeTo(RangeTo<Idx>),
    /// Unbounded inclusive [RangeToInclusive], i.e. `..=10`
    RangeToInclusive(RangeToInclusive<Idx>),
}

variant_from! {
    match impl<Idx> (From, TryInto) for AnyRange<Idx> {
        type Error = AnyRangeError<Idx>;
        Range<Idx> => AnyRange::Range,
        RangeFrom<Idx> => AnyRange::RangeFrom,
        RangeFull => AnyRange::RangeFull,
        RangeInclusive<Idx> => AnyRange::RangeInclusive,
        RangeTo<Idx> => AnyRange::RangeTo,
        RangeToInclusive<Idx> => AnyRange::RangeToInclusive,
    }
}

/// Convenient conversion functions from [AnyRange] to the inner type
impl<Idx> AnyRange<Idx> {
    try_into_fn! {
        /// Converts from [AnyRange::Range] into a [Range], else [None]
        pub fn range(self) -> Option<Range<Idx>>;
        /// Converts from [AnyRange::RangeFrom] into a [RangeFrom], else [None]
        pub fn range_from(self) -> Option<RangeFrom<Idx>>;
        /// Converts from [AnyRange::RangeFull] into a [RangeFull], else [None]
        pub fn range_full(self) -> Option<RangeFull>;
        /// Converts from [AnyRange::RangeInclusive] into a [RangeInclusive], else [None]
        pub fn range_inclusive(self) -> Option<RangeInclusive<Idx>>;
        /// Converts from [AnyRange::RangeTo] into a [RangeTo], else [None]
        pub fn range_to(self) -> Option<RangeTo<Idx>>;
        /// Converts from [AnyRange::RangeToInclusive] into a [RangeToInclusive], else [None]
        pub fn range_to_inclusive(self) -> Option<RangeToInclusive<Idx>>;
    }
}

impl<Idx> From<AnyRange<Idx>> for AnyRangeError<Idx> {
    fn from(value: AnyRange<Idx>) -> Self {
        AnyRangeError(value)
    }
}
