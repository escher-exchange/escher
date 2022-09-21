//! The `TWAP` module provides a helper type and associated methods that are
//! generic over the underlying `twap` and `timestamp` types used to operate on
//! time weighted average price values.
//!
//! TODO(Cardosaum): Add high quality gif showing twap behaviour
//!
//! # Available methods
//!
//! - [`new`](struct.Twap.html#method.new): Creates a new [`Twap`] instance, returning it.
//! - [`accumulate`](struct.Twap.html#method.accumulate): Updates the Twap's value using the
//! default exponential moving average function.
//! - [`get_twap`](struct.Twap.html#method.get_twap): Return the Twap's value.
//! - [`set_twap`](struct.Twap.html#method.set_twap): Set the Twap's value.
//! - [`get_timestamp`](struct.Twap.html#method.get_timestamp): Return the Twap's timestamp.
//! - [`set_timestamp`](struct.Twap.html#method.set_timestamp): Set the Twap's timestamp.
//! - [`get_period`](struct.Twap.html#method.get_period): Return the Twap's period.
//!
//! # Examples
//!
//! ```
//! # use sp_runtime::FixedU128;
//! # use crate::helpers::twap::Twap;
//! // Initialize the twap value.
//! let price = FixedU128::from_float(42.0);
//! let timestamp: u64 = 1600000000;
//! let period: u64 = 3600;
//! let mut twap = Twap::new(price, timestamp, period);
//!
//! // Some time passes...
//! let timestamp: u64 = 1700000000;
//!
//! // Update twap values according to a exponential moving average function.
//! let price = FixedU128::from_float(1337.0);
//! assert_eq!(Ok(price), twap.accumulate(&price, timestamp));
//!
//! // It's also possible to update the twap value directly, as well as its timestamp.
//! let price = FixedU128::from_float(1337.0);
//! let timestamp: u64 = 1600000000;
//! twap.set_twap(price);
//! twap.set_timestamp(timestamp);
//! assert_eq!(price, twap.get_twap());
//! assert_eq!(timestamp, twap.get_timestamp());
//! ```
#![cfg_attr(
    not(test),
    deny(
        clippy::all,
        clippy::cargo,
        clippy::complexity,
        clippy::correctness,
        clippy::pedantic,
        clippy::perf,
        clippy::style,
        clippy::suspicious,
        missing_docs,
        rustdoc::missing_crate_level_docs,
        rustdoc::missing_doc_code_examples,
        warnings,
    )
)]
#![allow(clippy::wildcard_imports)]

use crate::numbers::{FixedPointMath, IntoU256, UnsignedMath};
use frame_support::pallet_prelude::*;
use num_traits::CheckedMul;
use sp_core::U256;
use sp_runtime::{
    traits::Saturating,
    ArithmeticError::{self, DivisionByZero, Overflow},
};
use sp_std::cmp::Ord;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The [`Twap`] value itself, storing both the underlying time weighted average
/// price and its most recent timestamp.
///
/// The [`Twap`] type implement some convinience methods to facilitate when
/// working with time weighted average prices. Here is a list of all possible
/// functions that could be used with this type and also some examples:
///
/// # Available methods
///
/// - [`new`](Self::new): Creates a new [`Twap`] instance, returning it.
/// - [`accumulate`](Self::accumulate): Updates the Twap's value using the
/// default exponential moving average function.
/// - [`get_twap`](Self::get_twap): Return the Twap's value.
/// - [`set_twap`](Self::set_twap): Set the Twap's value.
/// - [`get_timestamp`](Self::get_timestamp): Return the Twap's timestamp.
/// - [`set_timestamp`](Self::set_timestamp): Set the Twap's timestamp.
/// - [`get_period`](Self::get_period): Return the Twap's period.
///
/// # Examples
///
/// ```
/// # use sp_runtime::FixedU128;
/// # use crate::helpers::twap::Twap;
/// // Initialize the twap value.
/// let price = FixedU128::from_float(42.0);
/// let timestamp: u64 = 1600000000;
/// let period: u64 = 3600;
/// let mut twap = Twap::new(price, timestamp, period);
///
/// // Some time passes...
/// let timestamp: u64 = 1600500000;
///
/// // Update twap values according to a exponential moving average function.
/// let price = FixedU128::from_float(1337.0);
/// assert_eq!(Ok(price), twap.accumulate(&price, timestamp));
///
/// // It's also possible to update the twap value directly, as well as its timestamp.
/// let price = FixedU128::from_float(1337.0);
/// let timestamp: u64 = 1600000000;
/// twap.set_twap(price);
/// twap.set_timestamp(timestamp);
/// assert_eq!(price, twap.get_twap());
/// assert_eq!(timestamp, twap.get_timestamp());
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Twap<FixedPoint, Moment> {
    /// The "time weighted average price", represented by a decimal number.
    twap: FixedPoint,
    /// The last time when the [`twap`](Self::twap) value was updated.
    ts: Moment,
    period: Moment,
}

///  Default implementation for Twap.
#[allow(rustdoc::missing_doc_code_examples)]
impl<FixedPoint, Moment> Twap<FixedPoint, Moment>
where
    FixedPoint: FixedPointMath,
    FixedPoint::Inner: Into<u128> + From<u128>,
    Moment: Copy + From<u64> + Into<FixedPoint::Inner> + Ord + UnsignedMath + Saturating,
{
    /// Creates a new [`Twap`] instance, returning it.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// // Set the initial twap value to `42.0`, with an initial timestamp
    /// // representing the date `Sun Sep 13 12:26:40 AM 2020`, and a period of
    /// // one hour.
    /// let price = FixedU128::from_float(42.0);
    /// let timestamp: u64 = 1600000000;
    /// let period: u64 = 3600;
    ///
    /// let twap = Twap::new(price, timestamp, period);
    /// dbg!(twap);
    /// // Twap {
    /// //    twap: FixedU128(42.000000000000000000),
    /// //    ts: 1600000000,
    /// //    period: 3600,
    /// // };
    /// ```
    pub const fn new(twap: FixedPoint, ts: Moment, period: Moment) -> Self {
        Self { twap, ts, period }
    }

    /// Return the Twap's value.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// let price = FixedU128::from_float(42.0);
    /// # let timestamp: u64 = 1600000000;
    /// # let period: u64 = 3600;
    ///
    /// let twap = Twap::new(price, timestamp, period);
    ///
    /// assert_eq!(twap.get_twap(), price);
    /// ```
    pub const fn get_twap(&self) -> FixedPoint {
        self.twap
    }

    /// Set the Twap's value.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// let price = FixedU128::from_float(42.0);
    /// # let timestamp: u64 = 1600000000;
    /// # let period: u64 = 3600;
    /// let mut twap = Twap::new(price, timestamp, period);
    /// assert_eq!(twap.get_twap(), price);
    ///
    /// let new_price = FixedU128::from_float(1337.0);
    /// twap.set_twap(new_price);
    /// assert_eq!(twap.get_twap(), new_price);
    /// ```
    pub fn set_twap(&mut self, price: FixedPoint) -> FixedPoint {
        self.twap = price;
        price
    }

    /// Return the Twap's timestamp.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// # let price = FixedU128::from_float(42.0);
    /// let timestamp: u64 = 1600000000;
    /// # let period: u64 = 3600;
    /// let twap = Twap::new(price, timestamp, period);
    ///
    /// assert_eq!(twap.get_timestamp(), timestamp);
    /// ```
    pub const fn get_timestamp(&self) -> Moment {
        self.ts
    }

    /// Set the Twap's timestamp.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// # let price = FixedU128::from_float(42.0);
    /// let timestamp: u64 = 1600000000;
    /// # let period: u64 = 3600;
    /// let mut twap = Twap::new(price, timestamp, period);
    /// assert_eq!(twap.get_timestamp(), timestamp);
    ///
    /// let new_timestamp: u64 = 1700000000;
    /// twap.set_timestamp(new_timestamp);
    /// assert_eq!(twap.get_timestamp(), new_timestamp);
    /// ```
    pub fn set_timestamp(&mut self, timestamp: Moment) -> Moment {
        self.ts = timestamp;
        timestamp
    }

    /// Return the Twap's period.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// # let price = FixedU128::from_float(42.0);
    /// # let timestamp: u64 = 1600000000;
    /// let period: u64 = 3600;
    ///
    /// let twap = Twap::new(price, timestamp, period);
    ///
    /// assert_eq!(twap.get_period(), period);
    /// ```
    pub const fn get_period(&self) -> Moment {
        self.period
    }

    /// Updates the Twap's value using the default exponential moving average
    /// function.
    ///
    /// # Examples
    /// ```
    /// # use sp_runtime::FixedU128;
    /// # use crate::helpers::twap::Twap;
    /// # use frame_support::assert_ok;
    /// let mut price = FixedU128::from_float(42.0);
    /// let mut timestamp: u64 = 1600000000;
    /// let period: u64 = 3600;
    /// let mut twap = Twap::new(price, timestamp, period);
    ///
    /// // Assumes one hour has passed.
    /// timestamp += period;
    ///
    /// // Cut price by half.
    /// price = price / FixedU128::from_float(2.0);
    ///
    /// // Update twap value with a new price.
    /// let result = twap.accumulate(&price, timestamp);
    /// assert_ok!(result, price);
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ArithmeticError`]
    pub fn accumulate(
        &mut self,
        price: &FixedPoint,
        now: Moment,
    ) -> Result<FixedPoint, ArithmeticError> {
        let since_last = now.saturating_sub(self.ts).max(1.into());
        let from_start = self.period.saturating_sub(since_last);
        self.update_mut(price, from_start, since_last, now)?;
        Ok(self.twap)
    }

    /// This function *simulates* the [`twap`](Twap::twap) update, returning the
    /// value that would be used as the new [`twap`](Twap::twap), but **not**
    /// modifying the current value.
    ///
    /// # Errors
    ///
    /// - [`ArithmeticError`]
    fn update(
        &self,
        price: &FixedPoint,
        from_start: Moment,
        since_last: Moment,
    ) -> Result<FixedPoint, ArithmeticError> {
        self.fast_weighted_average(price, from_start, since_last)
            .or_else(|_| self.slow_weighted_average(price, from_start, since_last))
    }

    /// This function is similar to [`update`](Self::update), but it **does**
    /// change the current [`twap`](Twap::twap) value, and does not return
    /// anything in case of a successfull call.
    ///
    /// # Errors
    ///
    /// - [`ArithmeticError`]
    fn update_mut(
        &mut self,
        price: &FixedPoint,
        from_start: Moment,
        since_last: Moment,
        ts: Moment,
    ) -> Result<(), ArithmeticError> {
        self.twap = self.update(price, from_start, since_last)?;
        self.ts = ts;
        Ok(())
    }

    // -------------------------------------------------------------------------------------------------
    //                                         Helper Functions
    // -------------------------------------------------------------------------------------------------
    fn fast_weighted_average(
        &self,
        price: &FixedPoint,
        from_start: Moment,
        since_last: Moment,
    ) -> Result<FixedPoint, ArithmeticError> {
        let unit = FixedPoint::DIV;
        let denominator = FixedPoint::from_inner(
            unit.checked_mul(&since_last.try_add(&from_start)?.into())
                .ok_or(Overflow)?,
        );
        let twap_t0 = self.twap.try_mul(&FixedPoint::from_inner(
            unit.checked_mul(&from_start.into()).ok_or(Overflow)?,
        ))?;
        let twap_t1 = price.try_mul(&FixedPoint::from_inner(
            unit.checked_mul(&since_last.into()).ok_or(Overflow)?,
        ))?;
        twap_t0.try_add(&twap_t1)?.try_div(&denominator)
    }

    fn slow_weighted_average(
        &self,
        price: &FixedPoint,
        from_start: Moment,
        since_last: Moment,
    ) -> Result<FixedPoint, ArithmeticError> {
        let unit: U256 = FixedPoint::DIV.into_u256();
        let from_start: U256 = from_start.into().into_u256();
        let since_last: U256 = since_last.into().into_u256();
        let denominator: U256 = unit
            .checked_mul(since_last.checked_add(from_start).ok_or(Overflow)?)
            .ok_or(Overflow)?;
        let twap_t0: U256 = self
            .twap
            .into_inner()
            .into_u256()
            .checked_mul(unit.checked_mul(from_start).ok_or(Overflow)?)
            .ok_or(Overflow)?;
        let twap_t1: U256 = price
            .into_inner()
            .into_u256()
            .checked_mul(unit.checked_mul(since_last).ok_or(Overflow)?)
            .ok_or(Overflow)?;
        Ok(FixedPoint::from_inner(
            u128::try_from(
                twap_t0
                    .checked_add(twap_t1)
                    .ok_or(Overflow)?
                    .checked_div(denominator)
                    .ok_or(DivisionByZero)?,
            )
            .or(Err(Overflow))?
            .into(),
        ))
    }
}
