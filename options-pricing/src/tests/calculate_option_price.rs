use crate::{
    mocks::{
        accounts::*,
        assets::*,
        runtime::{ExtBuilder, MockRuntime, OptionsPricing, Origin},
    },
    pallet::{self, Error, InterestRate, LatestSnapshots},
    tests::*,
};

use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use greeks::*;
use sp_runtime::FixedPointNumber;
use std;
// ----------------------------------------------------------------------------------------------------
//		Options Price Tests
// ----------------------------------------------------------------------------------------------------

#[test]
fn test_calculate_option_price() {
    ExtBuilder::default()
        .build()
        .initialize_oracle_prices()
        .execute_with(|| {
            let bs_params = BlackScholesParamsBuilder::default().build();

            let option_id = 1_u128;

            let interest_rate = InterestRate::<MockRuntime>::get();

            let correct_price = greeks::euro_call(
                f64::from(bs_params.base_asset_spot_price as u32),
                f64::from(bs_params.base_asset_strike_price as u32),
                f64::from(bs_params.expiring_date as u32),
                f64::from(interest_rate.into_inner() as u32),
                f64::from(0),
                0.54,
            );

            println!("{:?}", correct_price);

            assert_ok!(OptionsPricing::calculate_option_price(
                Origin::signed(ADMIN),
                option_id,
                bs_params
            ));
        });
}

#[test]
fn test_create_vault_error_not_protocol_origin_ext() {
    ExtBuilder::default()
        .build()
        .initialize_oracle_prices()
        .execute_with(|| {
            let bs_params = BlackScholesParamsBuilder::default().build();
            let option_id = 1_u128;

            // Check no changes have been performed with ALICE caller
            assert_noop!(
                OptionsPricing::calculate_option_price(Origin::signed(ALICE), option_id, bs_params),
                BadOrigin
            );
        });
}
