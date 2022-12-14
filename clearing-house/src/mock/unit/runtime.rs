use super::{
    accounts::{AccountId, ALICE},
    oracle as mock_oracle, vamm as mock_vamm,
};
use crate as clearing_house;
use crate::mock::assets::{AssetId, PICA};

use composable_traits::{defi::DeFiComposableConfig, time::DurationSeconds};
use frame_support::{
    ord_parameter_types, parameter_types,
    traits::{ConstU16, ConstU32, ConstU64, Everything, GenesisBuild},
    PalletId,
};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_traits::parameter_type_with_key;
use primitives::currency::ValidateCurrencyId;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedI128, FixedU128,
};

// ----------------------------------------------------------------------------------------------------
//                                             Construct Runtime
// ----------------------------------------------------------------------------------------------------

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        GovernanceRegistry: governance_registry,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
        LpTokenFactory: pallet_currency_factory,
        Assets: pallet_assets,
        Vamm: mock_vamm,
        Oracle: mock_oracle,
        TestPallet: clearing_house,
    }
);

pub type Amount = i64;
pub type Balance = u128;
pub type Decimal = FixedI128;
pub type Integer = i128;
pub type MarketId = u64;
pub type ReserveIdentifier = [u8; 8]; // copied from 'frame/assets/src/mocks.rs'
pub type UnsignedDecimal = FixedU128;
pub type VammId = u64;
pub type Moment = DurationSeconds;

// ----------------------------------------------------------------------------------------------------
//                                                FRAME System
// ----------------------------------------------------------------------------------------------------

impl system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

// ----------------------------------------------------------------------------------------------------
//                                                 Balances
// ----------------------------------------------------------------------------------------------------

parameter_types! {
    pub const NativeExistentialDeposit: Balance = 0;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = NativeExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = ReserveIdentifier;
}

// ----------------------------------------------------------------------------------------------------
//                                             Governance Registry
// ----------------------------------------------------------------------------------------------------

impl governance_registry::Config for Runtime {
    type AssetId = AssetId;
    type WeightInfo = ();
    type Event = Event;
}

// ----------------------------------------------------------------------------------------------------
//                                                 Timestamp
// ----------------------------------------------------------------------------------------------------

pub const MINIMUM_PERIOD_SECONDS: Moment = 5;

parameter_types! {
    pub const MinimumPeriod: u64 = MINIMUM_PERIOD_SECONDS;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// ----------------------------------------------------------------------------------------------------
//                                                 ORML Tokens
// ----------------------------------------------------------------------------------------------------

parameter_type_with_key! {
    pub TokensExistentialDeposit: |_currency_id: AssetId| -> Balance {
        0
    };
}

impl orml_tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = AssetId;
    type WeightInfo = ();
    type ExistentialDeposits = TokensExistentialDeposit;
    type OnDust = ();
    type MaxLocks = ();
    type DustRemovalWhitelist = Everything;
    type MaxReserves = frame_support::traits::ConstU32<2>; // copied from 'frame/assets/src/mocks.rs'
    type ReserveIdentifier = ReserveIdentifier;
    type OnNewTokenAccount = ();
    type OnKilledTokenAccount = ();
}

// ----------------------------------------------------------------------------------------------------
//                                               Currency Factory
// ----------------------------------------------------------------------------------------------------

impl pallet_currency_factory::Config for Runtime {
    type Event = Event;
    type AssetId = AssetId;
    type Balance = Balance;
    type AddOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

// ----------------------------------------------------------------------------------------------------
//                                                   Assets
// ----------------------------------------------------------------------------------------------------

parameter_types! {
    pub const NativeAssetId: AssetId = PICA;
}

ord_parameter_types! {
    pub const RootAccount: AccountId = ALICE;
}

impl pallet_assets::Config for Runtime {
    type NativeAssetId = NativeAssetId;
    type GenerateCurrencyId = LpTokenFactory;
    type AssetId = AssetId;
    type Balance = Balance;
    type NativeCurrency = Balances;
    type MultiCurrency = Tokens;
    type WeightInfo = ();
    type AdminOrigin = EnsureSignedBy<RootAccount, AccountId>;
    type GovernanceRegistry = GovernanceRegistry;
    type CurrencyValidator = ValidateCurrencyId;
}

// ----------------------------------------------------------------------------------------------------
//                                                   VAMM
// ----------------------------------------------------------------------------------------------------

impl mock_vamm::Config for Runtime {
    type VammId = VammId;
    type Decimal = UnsignedDecimal;
    type Moment = Moment;
}

// ----------------------------------------------------------------------------------------------------
//                                                   Oracle
// ----------------------------------------------------------------------------------------------------

parameter_types! {
    pub const MaxAnswerBound: u32 = 5;
    pub const TwapWindow: u16 = 3;
}

impl mock_oracle::Config for Runtime {
    type AssetId = AssetId;
    type Balance = Balance;
    type Timestamp = u64;
    type LocalAssets = ();
    type MaxAnswerBound = MaxAnswerBound;
    type TwapWindow = TwapWindow;
}

// ----------------------------------------------------------------------------------------------------
//                                               Clearing House
// ----------------------------------------------------------------------------------------------------

impl DeFiComposableConfig for Runtime {
    type Balance = Balance;
    type MayBeAssetId = AssetId;
}

parameter_types! {
    pub const MaxPositions: u32 = 5;
    pub const TestPalletId: PalletId = PalletId(*b"test_pid");
}

impl clearing_house::Config for Runtime {
    type Assets = Assets;
    type Decimal = Decimal;
    type Event = Event;
    type Integer = Integer;
    type MarketId = MarketId;
    type MaxPositions = MaxPositions;
    type Moment = Moment;
    type Oracle = Oracle;
    type PalletId = TestPalletId;
    type UnixTime = Timestamp;
    type Vamm = Vamm;
    type VammConfig = mock_vamm::VammConfig;
    type VammId = VammId;
    type WeightInfo = ();
}

// ----------------------------------------------------------------------------------------------------
//                                             Externalities Builder
// ----------------------------------------------------------------------------------------------------

pub struct ExtBuilder {
    pub native_balances: Vec<(AccountId, Balance)>,
    pub balances: Vec<(AccountId, AssetId, Balance)>,
    pub collateral_type: Option<AssetId>,
    pub vamm_id: Option<VammId>,
    pub vamm_twap: Option<UnsignedDecimal>,
    pub oracle_asset_support: Option<bool>,
    pub oracle_price: Option<Balance>,
    pub oracle_twap: Option<Balance>,
    pub max_price_divergence: Decimal,
}

impl ExtBuilder {
    #[allow(clippy::disallowed_methods)]
    pub fn build(self) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self.native_balances,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        orml_tokens::GenesisConfig::<Runtime> {
            balances: self.balances,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        clearing_house::GenesisConfig::<Runtime> {
            collateral_type: self.collateral_type,
            max_price_divergence: self.max_price_divergence,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        mock_vamm::GenesisConfig::<Runtime> {
            vamm_id: self.vamm_id,
            twap: self.vamm_twap,
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        let oracle_genesis = mock_oracle::GenesisConfig {
            price: self.oracle_price,
            supports_assets: self.oracle_asset_support,
            twap: self.oracle_twap,
        };
        GenesisBuild::<Runtime>::assimilate_storage(&oracle_genesis, &mut storage).unwrap();

        storage.into()
    }
}
