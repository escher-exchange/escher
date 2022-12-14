use helpers::numbers::TryIntoDecimal;
use sp_runtime::{traits::Zero, FixedI128, FixedPointNumber};

/// Returns the default tolerance for numeric approximations.
pub fn get_eps() -> FixedI128 {
    FixedI128::saturating_from_rational(1, 10_i32.pow(8))
}

/// Returns whether `a` isn't larger than `b` and at most a tiny amount smaller
pub fn approx_eq_lower<T: TryIntoDecimal<FixedI128>>(a: T, b: T) -> bool {
    let diff = get_diff(a, b);
    FixedI128::zero() <= diff && diff < get_eps()
}

/// Returns whether `a` isn't larger or smaller than `b` by more than `10^{-8}`.
pub fn approx_eq<T: TryIntoDecimal<FixedI128>>(a: T, b: T) -> bool {
    let diff = get_diff(a, b);
    let eps = get_eps();
    eps.neg() < diff && diff < eps
}

/// Returns `(b - a)` as decimals.
fn get_diff<T: TryIntoDecimal<FixedI128>>(a: T, b: T) -> FixedI128 {
    let a_dec: FixedI128 = a.try_into_decimal().unwrap();
    let b_dec: FixedI128 = b.try_into_decimal().unwrap();
    b_dec - a_dec
}
