use crate::{
    mock::{Event, ExtBuilder, MockRuntime, System, TestPallet, Twap},
    pallet::{self, Error},
    tests::{
        constants::{RUN_CASES, TWAP_PERIOD},
        helpers::{
            any_sane_asset_amount, as_decimal, get_twap_value, run_for_seconds, twap_update_delay,
            with_existing_vamm_context,
        },
        helpers_propcompose::any_valid_vammconfig,
        types::{Decimal, Timestamp},
    },
    types::VammState,
};
use frame_support::{assert_noop, assert_ok, assert_storage_noop};
use helpers::tests::default_acceptable_computation_error;
use proptest::prelude::*;
use sp_runtime::{traits::One, FixedPointNumber, FixedU128};
use traits::vamm::{Vamm as VammTrait, VammConfig};

// ----------------------------------------------------------------------------------------------------
//                                           Prop Compose
// ----------------------------------------------------------------------------------------------------

prop_compose! {
    fn any_new_twap()(
        twap in any_sane_asset_amount(),
    ) (
        twap in Just(Decimal::from_inner(twap))
    ) -> Decimal {
        twap
    }
}

// -------------------------------------------------------------------------------------------------
//                                           Unit Tests
// -------------------------------------------------------------------------------------------------

#[test]
fn should_succeed_computing_correct_reciprocal_twap() {
    assert_eq!(
        as_decimal(2).reciprocal().unwrap(),
        FixedU128::saturating_from_rational(50, 100)
    );
    assert_eq!(
        as_decimal(50).reciprocal().unwrap(),
        FixedU128::saturating_from_rational(2, 100)
    );
}

#[test]
fn update_twap_fails_if_vamm_does_not_exist() {
    let vamm_state = VammState::default();
    let base_twap = Some(as_decimal(10));
    ExtBuilder {
        vamm_count: 1,
        vamms: vec![(0, vamm_state)],
    }
    .build()
    .execute_with(|| {
        assert_noop!(
            TestPallet::update_twap(1, None),
            Error::<MockRuntime>::VammDoesNotExist
        );

        assert_noop!(
            TestPallet::update_twap(1, base_twap),
            Error::<MockRuntime>::VammDoesNotExist
        );
    });
}

#[test]
fn update_twap_fails_if_vamm_is_closed() {
    let vamm_state = VammState {
        closed: Some(Timestamp::MIN),
        base_asset_reserves: as_decimal(42).into_inner(),
        quote_asset_reserves: as_decimal(1337).into_inner(),
        base_asset_twap: Twap::new(as_decimal(42), Default::default(), TWAP_PERIOD),
        peg_multiplier: One::one(),
        ..Default::default()
    };
    let base_twap = Some(as_decimal(10));
    ExtBuilder {
        vamm_count: 1,
        vamms: vec![(0, vamm_state)],
    }
    .build()
    .execute_with(|| {
        // For event emission
        run_for_seconds(vamm_state.closed.unwrap() + 1);

        assert_noop!(
            TestPallet::update_twap(0, base_twap),
            Error::<MockRuntime>::VammIsClosed
        );

        assert_noop!(
            TestPallet::update_twap(0, None),
            Error::<MockRuntime>::VammIsClosed
        );
    });
}

#[test]
fn update_twap_fails_if_new_twap_is_zero() {
    let vamm_state = VammState::default();
    let base_twap = Some(as_decimal(0));
    ExtBuilder {
        vamm_count: 1,
        vamms: vec![(0, vamm_state)],
    }
    .build()
    .execute_with(|| {
        run_for_seconds(1);
        assert_storage_noop!(TestPallet::update_twap(0, base_twap));
    });
}

#[test]
fn update_twap_fails_if_twap_timestamp_is_more_recent() {
    let timestamp = Timestamp::MIN;
    let timestamp_greater = Timestamp::MIN + 1;
    let vamm_state = VammState {
        base_asset_reserves: as_decimal(42).into_inner(),
        quote_asset_reserves: as_decimal(1337).into_inner(),
        base_asset_twap: Twap::new(as_decimal(42), timestamp_greater, TWAP_PERIOD),
        peg_multiplier: One::one(),
        ..Default::default()
    };
    let new_twap = Some(as_decimal(10));
    ExtBuilder {
        vamm_count: 1,
        vamms: vec![(0, vamm_state)],
    }
    .build()
    .execute_with(|| {
        // For event emission
        run_for_seconds(timestamp);
        assert_noop!(
            TestPallet::update_twap(0, new_twap),
            Error::<MockRuntime>::AssetTwapTimestampIsMoreRecent
        );
        assert_noop!(
            TestPallet::update_twap(0, None),
            Error::<MockRuntime>::AssetTwapTimestampIsMoreRecent
        );
    });
}

#[test]
fn should_succeed_updating_twap_correctly() {
    let timestamp = Timestamp::MIN;
    let twap = as_decimal(1).into_inner();
    let new_twap = Some(as_decimal(5));
    let vamm_state = VammState {
        base_asset_twap: Twap::new(twap.into(), timestamp, TWAP_PERIOD),
        base_asset_reserves: twap,
        quote_asset_reserves: twap,
        peg_multiplier: 1,
        ..Default::default()
    };
    ExtBuilder {
        vamm_count: 1,
        vamms: vec![(0, vamm_state)],
    }
    .build()
    .execute_with(|| {
        run_for_seconds(twap_update_delay(0));
        assert_ok!(TestPallet::update_twap(0, new_twap), new_twap.unwrap());
        assert_eq!(
            get_twap_value(&TestPallet::get_vamm(0).unwrap()),
            new_twap.unwrap()
        );

        run_for_seconds(twap_update_delay(0));
        assert_ok!(TestPallet::update_twap(0, None));
        assert_ne!(
            get_twap_value(&TestPallet::get_vamm(0).unwrap()),
            new_twap.unwrap()
        );
    });
}

#[test]
fn should_update_twap_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let vamm_creation = TestPallet::create(&VammConfig {
            base_asset_reserves: as_decimal(2).into_inner(),
            quote_asset_reserves: as_decimal(50).into_inner(),
            peg_multiplier: 1,
            twap_period: TWAP_PERIOD,
        });
        let vamm_id = vamm_creation.unwrap();
        let original_base_twap = get_twap_value(&TestPallet::get_vamm(vamm_id).unwrap());
        assert_ok!(vamm_creation);

        // For event emission & twap update
        run_for_seconds(twap_update_delay(vamm_id));
        let new_base_price = Some(as_decimal(100));
        let new_base_twap_value = TestPallet::update_twap(vamm_id, new_base_price).unwrap();
        let vamm_state = TestPallet::get_vamm(vamm_id).unwrap();
        assert_ok!(default_acceptable_computation_error(
            get_twap_value(&vamm_state).into_inner(),
            new_base_price.unwrap().into_inner()
        ));
        System::assert_last_event(Event::TestPallet(pallet::Event::UpdatedTwap {
            vamm_id,
            base_twap: new_base_twap_value,
        }));

        // Run for long enough in order to approximate to the original twap value.
        run_for_seconds(twap_update_delay(vamm_id).saturating_pow(2));
        assert_ok!(TestPallet::update_twap(vamm_id, None));
        let vamm_state = TestPallet::get_vamm(vamm_id).unwrap();

        assert_ok!(default_acceptable_computation_error(
            get_twap_value(&vamm_state).into_inner(),
            original_base_twap.into_inner(),
        ));

        System::assert_last_event(Event::TestPallet(pallet::Event::UpdatedTwap {
            vamm_id,
            base_twap: get_twap_value(&vamm_state),
        }));
    });
}

// -------------------------------------------------------------------------------------------------
//                                           Proptests
// -------------------------------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(RUN_CASES))]
    #[test]
    fn update_twap_proptest_succeeds(
        vamm_config in any_valid_vammconfig(),
        base_twap in any_new_twap()
    ) {
        with_existing_vamm_context(vamm_config.into(), || {
            run_for_seconds(twap_update_delay(0)/1000);
            assert_ok!(
                TestPallet::update_twap(0, None)
            );

            run_for_seconds(twap_update_delay(0)/1000);
            assert_ok!(
                TestPallet::update_twap(0, Some(base_twap)),
                base_twap
            );
        })
    }
}
