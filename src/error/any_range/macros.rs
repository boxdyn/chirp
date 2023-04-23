//! Macros for AnyRange

/// Generate From and TryFrom impls for each variant
macro_rules! variant_from {(
    match impl<$T:ident> $_:tt for $enum:path {
        type Error = $error:path;
        $($src:path => $variant:path),+$(,)?
    }
) => {
    // The forward direction is infallible
    $(impl<$T> From<$src> for $enum {
        fn from(value: $src) -> Self { $variant(value) }
    })+
    // The reverse direction could fail if the $variant doesn't hold $src
    $(impl<$T> TryFrom<$enum> for $src {
        type Error = $error;
        fn try_from(value: $enum) -> Result<Self, Self::Error> {
            if let $variant(r) = value { Ok(r) } else { Err(value.into()) }
        }
    })+
}}

/// Turns a list of function prototypes into functions which use [TryInto], returning [Option]
///
/// # Examples
/// ```rust,ignore
/// try_into_fn! { fn range(self) -> Option<Other>; }
/// ```
macro_rules! try_into_fn {
    ($($(#[$doc:meta])? $pub:vis fn $name:ident $args:tt -> $ret:ty);+ $(;)?) => {
        $($(#[$doc])? $pub fn $name $args -> $ret { $args.try_into().ok() })+
    }
}
