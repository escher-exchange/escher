#![allow(clippy::identity_op)]
use crate::twap::Twap;
use frame_support::assert_ok;
use plotters::prelude::*;
use polars::prelude::*;
use rstest::rstest;
use sp_runtime::FixedU128;

// -------------------------------------------------------------------------------------------------
//                                             Constants
// -------------------------------------------------------------------------------------------------

const SECOND: u64 = 1000; // 1 second, in millis
const MINUTE: u64 = 60 * SECOND;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

const PERIOD: u64 = 1 * DAY; // Default period for twap

// -------------------------------------------------------------------------------------------------
//                                         Helper Functions
// -------------------------------------------------------------------------------------------------

fn from_float(x: f64) -> FixedU128 {
    FixedU128::from_float(x)
}

// -------------------------------------------------------------------------------------------------
//                                            Unit Tests
// -------------------------------------------------------------------------------------------------

#[rstest]
#[case(u128::MIN, u64::MIN, PERIOD)]
#[case(u128::MAX, u64::MAX, PERIOD)]
#[case(u128::MIN, u64::MAX, PERIOD)]
#[case(u128::MAX, u64::MIN, PERIOD)]
#[case(0, 0, PERIOD)]
fn should_create_twap_struct_successfully(
    #[case] twap: u128,
    #[case] ts: u64,
    #[case] period: u64,
) {
    let twap = FixedU128::from_inner(twap);
    let t = Twap::new(twap, ts, period);
    assert_eq!(t.twap, twap);
    assert_eq!(t.ts, ts);
    assert_eq!(t.period, period);
}

#[test]
fn should_update_twap_to_correct_value() {
    // Initialize twap to 100,
    // Set timestamp to "Mon Aug  8 11:06:40 PM UTC 2022"
    let mut ts = 1660000000;
    let mut t = Twap::new(from_float(100.0), ts, PERIOD);

    // After half PERDIOD passes, we update the twap.
    ts += PERIOD / 2;
    t.accumulate(&from_float(200.0), ts);

    // The value should be half the previous price and half the new one.
    assert_eq!(t.twap, from_float(150.0));
}

#[test]
fn should_update_twap_on_accumulate_call() {
    let mut t = Twap::new(from_float(25.0), 0, PERIOD);
    assert_ok!(t.accumulate(&from_float(50.0), PERIOD / 2));
}

#[test]
fn should_succeed_setting_and_retrieving_values() {
    let mut t = Twap::new(from_float(0.0), 0, PERIOD);

    let price = from_float(25.0);
    let ts = 10;
    t.set_twap(price);
    t.set_timestamp(ts);

    assert_eq!(t.get_twap(), price);
    assert_eq!(t.get_timestamp(), ts);
    assert_eq!(t.get_period(), PERIOD);
}
