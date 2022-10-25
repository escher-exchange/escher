use frame_support::pallet_prelude::*;
#[allow(unused_variables)]
// ----------------------------------------------------------------------------------------------------
//		Enums
// ----------------------------------------------------------------------------------------------------
/// Indicates the type of option: `Call` or `Put`
#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum OptionType {
    Call,
    Put,
}

/// Indicates the type of exercise of the option: `European` or `American`
#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum ExerciseType {
    European,
    American,
}

/// Indicates the type of phases of the option.
#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum Status {
    NotStarted,
    Deposit,
    Purchase,
    Exercise,
    End,
}

// ----------------------------------------------------------------------------------------------------
//		Structs and implementations
// ----------------------------------------------------------------------------------------------------

/// Stores the timestamps of an epoch.
/// An Epoch is divided into 4 phases: deposit, purchase, exercise.
#[derive(Clone, Copy, Encode, Decode, Debug, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct Epoch<Moment> {
    pub deposit: Moment,
    pub purchase: Moment,
    pub exercise: Moment,
    pub end: Moment,
}

/// Configuration for creating an option
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen, Debug)]
pub struct OptionConfig<AssetId, Balance, Moment> {
    pub base_asset_id: AssetId,
    pub quote_asset_id: AssetId,
    pub base_asset_strike_price: Balance,
    pub quote_asset_strike_price: Balance,
    pub option_type: OptionType,
    pub expiring_date: Moment,
    pub exercise_type: ExerciseType,
    pub epoch: Epoch<Moment>,
    pub status: Status,
    pub base_asset_amount_per_option: Balance,
    pub quote_asset_amount_per_option: Balance,
    pub total_issuance_seller: Balance,
    pub total_premium_paid: Balance,
    pub exercise_amount: Balance,
    pub base_asset_spot_price: Balance,
    pub total_issuance_buyer: Balance,
    pub total_shares_amount: Balance,
}

// ----------------------------------------------------------------------------------------------------
//		Trait
// ----------------------------------------------------------------------------------------------------
pub trait TokenizedOptions {
    type AccountId;
    type Balance;
    type Moment;
    type OptionId;
    type VaultId;
    type OptionConfig;
    type VaultConfig;

    fn create_asset_vault(config: Self::VaultConfig) -> Result<Self::VaultId, DispatchError>;

    fn create_option(option_config: Self::OptionConfig) -> Result<Self::OptionId, DispatchError>;

    fn sell_option(
        from: &Self::AccountId,
        option_amount: Self::Balance,
        option_id: Self::OptionId,
    ) -> Result<(), DispatchError>;

    fn delete_sell_option(
        from: &Self::AccountId,
        option_amount: Self::Balance,
        option_id: Self::OptionId,
    ) -> Result<(), DispatchError>;

    fn buy_option(
        from: &Self::AccountId,
        option_amount: Self::Balance,
        option_id: Self::OptionId,
    ) -> Result<(), DispatchError>;

    fn exercise_option(
        from: &Self::AccountId,
        option_amount: Self::Balance,
        option_id: Self::OptionId,
    ) -> Result<(), DispatchError>;

    fn withdraw_collateral(
        from: &Self::AccountId,
        option_id: Self::OptionId,
    ) -> Result<(), DispatchError>;
}
