use crate::twap::Twap;
use frame_support::assert_ok;
use proptest::prelude::*;
use sp_runtime::FixedU128;
use std::ops::RangeInclusive;

// -------------------------------------------------------------------------------------------------
//                                         Helper Functions
// -------------------------------------------------------------------------------------------------

fn from_float(x: f64) -> FixedU128 {
    FixedU128::from_float(x)
}

fn price_range() -> RangeInclusive<u128> {
    u128::MIN..=u128::MAX
}

fn any_time() -> RangeInclusive<u64> {
    u64::MIN..=u64::MAX
}

prop_compose! {
    fn any_price()(
        twap in price_range(),
    ) -> FixedU128 {
        FixedU128::from_inner(twap)
    }
}

prop_compose! {
    fn any_twap()(
        price in any_price(),
        ts in any_time(),
        period in any_time(),
    ) -> Twap<FixedU128, u64> {
        Twap::new(price, ts, period)
    }
}

// -------------------------------------------------------------------------------------------------
//                                          Property Tests
// -------------------------------------------------------------------------------------------------

proptest! {
    #[test]
    fn should_update_price_and_retrive_it_correctly(
        mut twap in any_twap(),
        price in any_price()
    ) {
        twap.set_twap(price);
        assert_eq!(price, twap.get_twap());
    }

    #[test]
    fn should_update_timestamp_and_retrive_it_correctly(
        mut twap in any_twap(),
        timestamp in any_time()
    ) {
        twap.set_timestamp(timestamp);
        assert_eq!(timestamp, twap.get_timestamp());
    }

    #[test]
    fn should_accumulate_price_correctly(
        mut twap in any_twap(),
        price in any_price(),
        timestamp in any_time()
    ) {
        assert_ok!(twap.accumulate(&price, timestamp));
    }
}
