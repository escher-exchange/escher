//! The `Tests` module provides helper functions meant to be used in the test
//! suite.
#![cfg_attr(
    not(test),
    deny(
        clippy::all,
        clippy::cargo,
        clippy::complexity,
        clippy::correctness,
        clippy::nursery,
        clippy::pedantic,
        clippy::perf,
        clippy::style,
        clippy::suspicious,
        missing_docs,
        rustdoc::missing_crate_level_docs,
        warnings,
    )
)]

use sp_runtime::{FixedPointNumber, FixedU128};

/// Default is percent
pub const DEFAULT_PRECISION: u128 = 1000;

/// Per mill
pub const DEFAULT_EPSILON: u128 = 1;

/// This function should be used in context of approximation.
/// It is extensively used in conjunction with proptest because of random input
/// generation.
///
/// # Errors
///
/// Will return an error if the computation error is greater than the required
/// precision.
pub fn acceptable_computation_error(
    x: u128,
    y: u128,
    precision: u128,
    epsilon: u128,
) -> Result<(), FixedU128> {
    if x.max(y).saturating_sub(x.min(y)) <= 1 {
        return Ok(())
    }
    let lower = FixedU128::saturating_from_rational(precision, precision.saturating_add(epsilon));
    let upper = FixedU128::saturating_from_rational(precision, precision.saturating_sub(epsilon));
    match FixedU128::checked_from_rational(x, y) {
        Some(q) =>
            if lower <= q && q <= upper {
                Ok(())
            } else {
                Err(q)
            },
        None => Err(FixedU128::default()),
    }
}

/// This function should be used in context of approximation.
/// It makes use of [`acceptable_computation_error`], but using sane default
/// values for `precision` and `epsilon`.
///
/// # Errors
///
/// Will return an error if the computation error is greater than the default
/// precision.
pub fn default_acceptable_computation_error(x: u128, y: u128) -> Result<(), FixedU128> {
    acceptable_computation_error(x, y, DEFAULT_PRECISION, DEFAULT_EPSILON)
}
