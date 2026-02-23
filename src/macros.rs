//! Cross-cutting utility macros.

/// Panic in debug/test builds, log an error in release builds.
///
/// Use for invariant violations that indicate bugs but should not crash
/// users in production:
/// - Unexpected enum variants in match arms
/// - Missing resources that should have been initialized
/// - State machine transitions that should be unreachable
///
/// Inspired by Zed's `debug_panic!` pattern.
///
/// # Examples
///
/// ```ignore
/// match state {
///     State::Active => { /* normal path */ }
///     State::Invalid => debug_panic!("entered invalid state"),
/// }
/// ```
#[allow(unused_macros)]
macro_rules! debug_panic {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            panic!($($arg)*);
        } else {
            bevy::log::error!($($arg)*);
        }
    };
}

#[allow(unused_imports)]
pub(crate) use debug_panic;

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "test invariant violated")]
    fn debug_panic_panics_in_debug_mode() {
        debug_panic!("test invariant violated");
    }

    #[test]
    #[should_panic(expected = "value was 42")]
    fn debug_panic_supports_format_args() {
        debug_panic!("value was {}", 42);
    }
}
