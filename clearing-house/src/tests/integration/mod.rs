use crate::{
    mock::integration::{
        AccountId, AssetId, Assets, Balance, BlockNumber, Decimal, ExtBuilder, MarketId, Moment,
        Oracle, Origin, Runtime, StalePrice, System, TestPallet, Timestamp, UnsignedDecimal, Vamm,
        VammId, ALICE, BOB, DOT, PICA, USDC,
    },
    tests::helpers,
    Direction::{Long, Short},
    Error, Event, Market, MarketConfig as MarketConfigGeneric, Position,
};
use composable_support::validation::Validated;
use composable_traits::time::{DurationSeconds, ONE_HOUR};
use frame_support::{
    assert_noop, assert_ok,
    error::BadOrigin,
    pallet_prelude::Hooks,
    traits::{fungibles::Transfer, UnixTime},
};
use pallet_vamm::VammStateOf;
use proptest::prelude::*;
use sp_runtime::{traits::Zero, FixedPointNumber, Percent};
use traits::vamm::{AssetType, Vamm as VammTrait, VammConfig as VammConfigGeneric};

mod close_market;
mod create_market;
mod liquidate;
mod open_position;
mod update_funding;

// -------------------------------------------------------------------------------------------------
//                                  Helper Functions and Traits
// -------------------------------------------------------------------------------------------------

fn advance_blocks_by(blocks: BlockNumber, secs_per_block: DurationSeconds) {
    let mut curr_block = System::block_number();
    let mut time = Timestamp::get();
    for _ in 0..blocks {
        if curr_block > 0 {
            Timestamp::on_finalize(curr_block);
            Oracle::on_finalize(curr_block);
            System::on_finalize(curr_block);
        }
        curr_block += 1;
        System::set_block_number(curr_block);
        // Time is set in milliseconds
        time += 1000 * secs_per_block;
        let _ = Timestamp::set(Origin::none(), time);
        System::on_initialize(curr_block);
        Timestamp::on_initialize(curr_block);
        Oracle::on_initialize(curr_block);
    }
}

fn run_to_time(seconds: DurationSeconds) {
    let curr_block = System::block_number();
    if curr_block > 0 {
        Timestamp::on_finalize(curr_block);
        Oracle::on_finalize(curr_block);
        System::on_finalize(curr_block);
    }

    let next_block = curr_block + 1;
    System::set_block_number(next_block);
    // Time is set in milliseconds, so we multiply the seconds by 1000
    // Should fail if the current time is greater than or equal to the argument
    let _ = Timestamp::set(Origin::none(), 1_000 * seconds);
    System::on_initialize(next_block);
    Timestamp::on_initialize(next_block);
    Oracle::on_initialize(next_block);
}

fn set_oracle_for(asset_id: AssetId, price: Balance) {
    assert_ok!(Oracle::add_asset_and_info(
        Origin::signed(ALICE),
        asset_id,
        Validated::new(Percent::from_percent(80)).unwrap(), // threshold
        Validated::new(1).unwrap(),                         // min_answers
        Validated::new(3).unwrap(),                         // max_answers
        Validated::new(ORACLE_BLOCK_INTERVAL).unwrap(),     // block_interval
        5,                                                  // reward
        5,                                                  // slash
        false                                               // emit_price_changes
    ));

    assert_ok!(Oracle::set_signer(Origin::signed(ALICE), BOB));
    assert_ok!(Oracle::set_signer(Origin::signed(BOB), ALICE));

    assert_ok!(Oracle::add_stake(Origin::signed(ALICE), 50));
    assert_ok!(Oracle::add_stake(Origin::signed(BOB), 50));

    update_oracle_for(asset_id, price);
}

fn update_oracle_for(asset_id: AssetId, price: Balance) {
    // Must be strictly greater than block interval for price to be considered 'requested'
    advance_blocks_by(ORACLE_BLOCK_INTERVAL + 1, 1);

    assert_ok!(Oracle::submit_price(Origin::signed(BOB), price, asset_id));

    // Advance block so that Oracle block finalization hook is called
    advance_blocks_by(1, 1);
}

fn get_collateral(account_id: &AccountId) -> Balance {
    helpers::get_collateral::<Runtime>(account_id)
}

fn get_outstanding_profits(account_id: &AccountId) -> Balance {
    helpers::get_outstanding_profits::<Runtime>(account_id)
}

fn get_market(market_id: &MarketId) -> Market<Runtime> {
    helpers::get_market::<Runtime>(market_id)
}

fn get_market_fee_pool(market_id: MarketId) -> Balance {
    helpers::get_market_fee_pool::<Runtime>(market_id)
}

fn get_vamm(vamm_id: &VammId) -> VammStateOf<Runtime> {
    Vamm::get_vamm(vamm_id).unwrap()
}

fn get_insurance_acc_balance() -> Balance {
    helpers::get_insurance_acc_balance::<Runtime>()
}

fn get_position(account_id: &AccountId, market_id: &MarketId) -> Option<Position<Runtime>> {
    helpers::get_position::<Runtime>(account_id, market_id)
}

fn get_unrealized_pnl(account_id: &AccountId, market_id: &MarketId) -> Decimal {
    let market = get_market(market_id);
    let position = get_position(account_id, market_id).unwrap();
    let (_, pnl) = TestPallet::abs_position_notional_and_pnl(
        &market,
        &position,
        position.direction().unwrap(),
    )
    .unwrap();
    pnl
}

fn get_time_now() -> DurationSeconds {
    <Timestamp as UnixTime>::now().as_secs()
}

fn set_maximum_oracle_mark_divergence(fraction: Decimal) {
    helpers::set_maximum_oracle_mark_divergence::<Runtime>(fraction)
}

fn set_partial_liquidation_penalty(decimal: Decimal) {
    helpers::set_partial_liquidation_penalty::<Runtime>(decimal)
}

fn set_partial_liquidation_close(decimal: Decimal) {
    helpers::set_partial_liquidation_close::<Runtime>(decimal)
}

fn set_liquidator_share_partial(decimal: Decimal) {
    helpers::set_liquidator_share_partial::<Runtime>(decimal)
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            // For setting up and updating the oracle
            native_balances: vec![(ALICE, UNIT), (BOB, UNIT)],
            balances: vec![],
            collateral_type: Some(USDC),
            max_price_divergence: Decimal::from_inner(i128::MAX),
        }
    }
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            asset: DOT,
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 100,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: Decimal::from_float(0.1),
            margin_ratio_maintenance: Decimal::from_float(0.02),
            margin_ratio_partial: Decimal::from_float(0.04),
            minimum_trade_size: 0.into(),
            funding_frequency: ONE_HOUR,
            funding_period: ONE_HOUR * 24,
            taker_fee: 0,
            twap_period: ONE_HOUR,
        }
    }
}

// -------------------------------------------------------------------------------------------------
//                                      Types & Constants
// -------------------------------------------------------------------------------------------------

pub type MarketConfig = MarketConfigGeneric<AssetId, Balance, Decimal, Moment, VammConfig>;
pub type VammConfig = VammConfigGeneric<Balance, Moment>;

// Must be strictly greater than StalePrice
pub const ORACLE_BLOCK_INTERVAL: u64 = StalePrice::get() + 1;
pub const UNIT: Balance = UnsignedDecimal::DIV;

// -------------------------------------------------------------------------------------------------
//                                          Sanity Check
// -------------------------------------------------------------------------------------------------

#[test]
fn externalities_builder_works() {
    ExtBuilder::default().build().execute_with(|| {});
}
