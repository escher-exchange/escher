#![allow(clippy::disallowed_methods, clippy::identity_op)]

use crate::mocks::{accounts::*, assets::*, runtime::*};
use traits::{options_pricing::*, tokenized_options::*};

use frame_support::traits::Hooks;

pub mod calculate_option_price;

pub const UNIT: u128 = 10u128.pow(12);

// ----------------------------------------------------------------------------------------------------
//		OracleInitializer
// ----------------------------------------------------------------------------------------------------
pub trait OracleInitializer {
    fn initialize_oracle_prices(self) -> Self;
}

impl OracleInitializer for sp_io::TestExternalities {
    fn initialize_oracle_prices(mut self) -> Self {
        let assets_prices: Vec<(AssetId, Balance)> = Vec::from([
            (PICA, 1 * 10u128.pow(9)), // 0.001
            (USDC, 1 * UNIT),
            (BTC, 50_000 * UNIT),
        ]);

        self.execute_with(|| {
            assets_prices.iter().for_each(|&(asset, price)| {
                set_oracle_price(asset, price);
            });
        });

        self
    }
}

// ----------------------------------------------------------------------------------------------------
//		OptionsConfigBuilder
// ----------------------------------------------------------------------------------------------------
struct BlackScholesParamsBuilder {
    pub base_asset_id: AssetId,
    pub base_asset_strike_price: Balance,
    pub base_asset_spot_price: Balance,
    pub expiring_date: Moment,
    pub option_type: OptionType,
    pub total_issuance_buyer: Balance,
    pub total_premium_paid: Balance,
}

impl Default for BlackScholesParamsBuilder {
    fn default() -> Self {
        BlackScholesParamsBuilder {
            base_asset_id: BTC,
            base_asset_strike_price: 50000u128 * UNIT,
            option_type: OptionType::Call,
            expiring_date: 6000u64,
            total_premium_paid: 0u128,
            base_asset_spot_price: 40000u128,
            total_issuance_buyer: 0u128,
        }
    }
}

impl BlackScholesParamsBuilder {
    fn build(self) -> BlackScholesParams<AssetId, Balance, Moment> {
        BlackScholesParams {
            base_asset_id: self.base_asset_id,
            base_asset_strike_price: self.base_asset_strike_price,
            option_type: self.option_type,
            expiring_date: self.expiring_date,
            total_premium_paid: self.total_premium_paid,
            base_asset_spot_price: self.base_asset_spot_price,
            total_issuance_buyer: self.total_issuance_buyer,
        }
    }

    fn base_asset_id(mut self, base_asset_id: AssetId) -> Self {
        self.base_asset_id = base_asset_id;
        self
    }

    fn base_asset_strike_price(mut self, base_asset_strike_price: Balance) -> Self {
        self.base_asset_strike_price = base_asset_strike_price;
        self
    }

    fn option_type(mut self, option_type: OptionType) -> Self {
        self.option_type = option_type;
        self
    }

    fn expiring_date(mut self, expiring_date: Moment) -> Self {
        self.expiring_date = expiring_date;
        self
    }

    fn total_issuance_buyer(mut self, total_issuance_buyer: Balance) -> Self {
        self.total_issuance_buyer = total_issuance_buyer;
        self
    }

    fn total_premium_paid(mut self, total_premium_paid: Balance) -> Self {
        self.total_premium_paid = total_premium_paid;
        self
    }
}

// ----------------------------------------------------------------------------------------------------
//		Helper functions
// ----------------------------------------------------------------------------------------------------
// Move the block number to `n` calling the desired hooks
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 0 {
            Timestamp::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        // Assuming millisecond timestamps, one second for each block
        System::on_initialize(System::block_number());
        Timestamp::on_initialize(System::block_number());
        Timestamp::set(Origin::none(), System::block_number() * 1000).unwrap();
    }
}

// Move the block number by 1 and the timestamp by `n` seconds
pub fn run_for_seconds(n: u64) {
    if System::block_number() > 0 {
        Timestamp::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
    }
    System::set_block_number(System::block_number() + 1);
    System::on_initialize(System::block_number());
    Timestamp::on_initialize(System::block_number());
    Timestamp::set(Origin::none(), n * 1000).unwrap();
}
