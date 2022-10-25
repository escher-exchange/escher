//! # Tokenized Options Pallet
//!
//! ## Overview
//! This pallet provides an implementation for creating, selling, buying and exercise options as
//! tokens. For all the possible actions available in an option's epoch, look at the `diagrams`
//! folder
//!
//! ### Terminology
//! - **Base asset**: the asset the user wants to buy/sell in future.
//! - **Quote asset**: the asset traded against the base asset (usually a stablecoin).
//! - **Option**: a financial instrument that gives you the right to buy/sell a base asset at a
//!   fixed price (denominated in quote asset)
//!  in the future. You can either buy (or long) an option (obtaining the right to buy/sell the base
//! asset) or sell (or short) an option (give another user the right to buy/sell the base asset you
//! provide as collateral).
//! - **Call / Put**: option type, used to choose if you want to buy (Call) the base asset in the
//!   future or sell it (Put).
//! - **Strike price**: the price at which the user has the right to buy/sell the base asset in the
//!   future denominated in quote asset.
//! - **Spot price**: the current price of the base asset denominated in quote asset.
//! - **Expiration date**: the date of maturity of the option, after which the user can exercise it
//!   if the option is in profit.
//! - **Premium**: the cost the user has to pay denominated in quote asset to buy the option from
//!   the seller.
//! - **Collateral**: base/quote asset backing the seller's position, used to pay the buyer if the
//!   option ends in profit.
//! For selling `Call` options, the user needs to provide the right amount of base asset as
//! collateral; for selling `Put` options, the user needs to provide the right amount of quote asset
//! as collateral.
//! - **Epoch**: the full lifecycle of an option. It's composed by the deposit phase, the purchase
//!   phase and the exercise phase.
//!
//! ### Actors
//! - Sellers: users that provide collateral for selling options and collect the corresponding
//!   premium.
//! - Buyers: users that pay the premium for buying (and later exercise if in profit) the options.
//!
//! ### Implementations
//! The Tokenized Option pallet provides implementations for the following traits:
//! - [`TokenizedOptions`](traits::tokenized_options::TokenizedOptions)
//!
//! ## Interface
//!
//! ### Extrinsics
//! - [`create_asset_vault`](Pallet::create_asset_vault): creates a vault that is responsible for
//!   collecting the specified asset and apply a particular strategy.
//!
//! - [`create_option`](Pallet::create_option): creates an option that can be sold or bought from
//!   users.
//!
//! - [`sell_option`](Pallet::sell_option): deposit collateral used for selling an option.
//!
//! - [`delete_sell_option`](Pallet::delete_sell_option): withdraw the deposited collateral used for
//!   selling an option.
//!
//! - [`buy_option`](Pallet::buy_option): pay the premium for minting the selected option token into
//!   the user's account.
//!
//! - [`exercise_option`](Pallet::exercise_option): burn the option tokens from user's account and
//!   transfer buyer's profit into
//! buyer's account.
//|
//! - [`withdraw_collateral`](Pallet::withdraw_collateral): withdraw seller's deposited collateral
//!   and its part of the premium.
//!
//! ### Runtime Storage Objects
//! - [`AssetToVault`]: maps an AssetId to its vault.
//! - [`OptionIdToOption`]: maps an OptionId to its option information.
//! - [`OptionHashToOptionId`]: maps a `H256` to its optionId. The hash is obtained from option's
//!   attributes.
//! - [`Sellers`]: maps an OptionId and an AccountId to its position as a seller.
//! - `Scheduler`: maps a [`Moment`](Config::Moment) to an OptionId [`OptionId`](OptionIdOf)
//!   identifying the timestamp
//! of the next phase of the epoch for the option.
//!
//! ### Example
//!
//! ## Related Modules
//! - [`Vault Pallet`](../pallet_vault/index.html)
//! - [`Oracle Pallet`](../oracle/index.html)
//! <!-- Original author: @nickkuk and @scoda95 -->

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    not(test),
    deny(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unwrap_used,
        clippy::panic
    )
)] // allow in tests
#![deny(
    dead_code,
    bad_style,
    bare_trait_objects,
    const_err,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates
)]

pub use crate::weights::WeightInfo;
mod types;
mod validation;
mod weights;

// #[allow(unused_imports)]
#[cfg(test)]
mod mocks;

#[cfg(test)]
#[allow(dead_code)]
mod tests;

// #[allow(dead_code)]
// #[allow(unused_imports)]
// #[cfg(any(feature = "runtime-benchmarks", test))]
// mod benchmarking;

pub use pallet::*;

#[frame_support::pallet]
#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
pub mod pallet {
    // ----------------------------------------------------------------------------------------------------
    //		Imports and Dependencies
    // ----------------------------------------------------------------------------------------------------
    use crate::{types::*, validation::*, weights::*};

    use codec::Codec;
    use composable_support::validation::Validated;
    use composable_traits::{
        currency::{CurrencyFactory, LocalAssets, RangeId},
        defi::DeFiComposableConfig,
        oracle::Oracle,
        vault::{CapabilityVault, Deposit as Duration, Vault, VaultConfig},
    };

    use traits::{
        options_pricing::*,
        swap_bytes::{SwapBytes, Swapped},
        tokenized_options::*,
    };

    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::Hash,
        storage::{bounded_btree_map::BoundedBTreeMap, bounded_btree_set::BoundedBTreeSet},
        traits::{
            fungibles::{Inspect, InspectHold, Mutate, MutateHold, Transfer},
            EnsureOrigin, Time,
        },
        transactional, PalletId,
    };

    use frame_system::{ensure_signed, pallet_prelude::*};
    use sp_arithmetic::Rounding;
    use sp_core::H256;
    use sp_runtime::{
        helpers_128bit::multiply_by_rational_with_rounding,
        traits::{
            AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned, BlakeTwo256, CheckedAdd,
            CheckedDiv, CheckedMul, CheckedSub, Convert, One, Saturating, Zero,
        },
        ArithmeticError, DispatchError, FixedPointNumber, FixedPointOperand, Perquintill,
    };
    use sp_std::cmp::min;

    use sp_std::{collections::btree_map::BTreeMap, fmt::Debug};

    // ----------------------------------------------------------------------------------------------------
    //		Declaration Of The Pallet Type
    // ----------------------------------------------------------------------------------------------------
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // ----------------------------------------------------------------------------------------------------
    //		Config Trait
    // ----------------------------------------------------------------------------------------------------
    #[pallet::config]
    pub trait Config: frame_system::Config + DeFiComposableConfig {
        #[allow(missing_docs)]
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type WeightInfo: WeightInfo;

        /// The id used as `AccountId` for the pallet.
        /// This should be unique across all pallets to avoid name collisions.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Oracle pallet to retrieve prices expressed in USDT.
        type Oracle: Oracle<AssetId = AssetIdOf<Self>, Balance = BalanceOf<Self>>;

        /// Type of time moment. We use [`SwapBytes`] trait to store this type in
        /// big endian format and take advantage of the fact that storage keys are
        /// stored in lexical order.
        type Moment: SwapBytes
            + AtLeast32Bit
            + Parameter
            + Copy
            + MaxEncodedLen
            + MaybeSerializeDeserialize;

        /// The Unix time provider.
        type Time: Time<Moment = MomentOf<Self>>;

        /// Trait used to convert from this pallet `Balance` type to `u128`.
        type Convert: Convert<BalanceOf<Self>, u128> + Convert<u128, BalanceOf<Self>>;

        /// Option IDs generator.
        type CurrencyFactory: CurrencyFactory<OptionIdOf<Self>, BalanceOf<Self>>;

        /// Stablecoin ID to use for cash operations.
        type StablecoinAssetId: Get<AssetIdOf<Self>>;

        /// General asset type to retrieve decimal information of the asset.
        type LocalAssets: LocalAssets<AssetIdOf<Self>>;

        /// Protocol Origin that can create vaults and options.
        type ProtocolOrigin: EnsureOrigin<Self::Origin>;

        /// Used for option tokens and other assets management.
        type Assets: Transfer<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
            + Mutate<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
            + MutateHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
            + Inspect<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>
            + InspectHold<AccountIdOf<Self>, Balance = BalanceOf<Self>, AssetId = AssetIdOf<Self>>;

        /// The [`VaultId`](Config::VaultId) used by the pallet. Corresponds to the id used by the
        /// Vault pallet.
        type VaultId: Clone + Copy + Codec + MaxEncodedLen + Debug + PartialEq + Default + Parameter;

        /// Vaults to collect collaterals.
        type Vault: CapabilityVault<
            AssetId = AssetIdOf<Self>,
            Balance = BalanceOf<Self>,
            AccountId = AccountIdOf<Self>,
            VaultId = VaultIdOf<Self>,
        >;

        // type OptionsPricing: OptionsPricing<
        // 	AssetId = AssetIdOf<Self>,
        // 	Balance = BalanceOf<Self>,
        // 	Moment = MomentOf<Self>,
        // 	OptionId = OptionIdOf<Self>,
        // >;
    }

    // ----------------------------------------------------------------------------------------------------
    //		Internal Pallet Types
    // ----------------------------------------------------------------------------------------------------
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetIdOf<T> = <T as DeFiComposableConfig>::MayBeAssetId;
    pub type BalanceOf<T> = <T as DeFiComposableConfig>::Balance;
    pub type AssetsOf<T> = <T as Config>::Assets;
    pub type MomentOf<T> = <T as Config>::Moment;
    pub type OracleOf<T> = <T as Config>::Oracle;
    pub type OptionConfigOf<T> = OptionConfig<AssetIdOf<T>, BalanceOf<T>, MomentOf<T>>;
    pub type OptionIdOf<T> = AssetIdOf<T>;
    pub type VaultIdOf<T> = <T as Config>::VaultId;
    pub type VaultOf<T> = <T as Config>::Vault;
    pub type VaultConfigOf<T> = VaultConfig<AccountIdOf<T>, AssetIdOf<T>>;
    // pub type OptionsPricingOf<T> = <T as Config>::OptionsPricing;

    // ----------------------------------------------------------------------------------------------------
    //		Storage
    // ----------------------------------------------------------------------------------------------------
    /// Maps [`AssetId`](AssetIdOf) to the corresponding
    /// [`VaultId`](Config::VaultId).
    #[pallet::storage]
    #[pallet::getter(fn asset_id_to_vault_id)]
    pub type AssetToVault<T: Config> = StorageMap<_, Blake2_128Concat, AssetIdOf<T>, VaultIdOf<T>>;

    /// Maps [`OptionId`](OptionIdOf) to the corresponding
    /// `OptionToken` struct.
    #[pallet::storage]
    #[pallet::getter(fn option_id_to_option)]
    pub type OptionIdToOption<T: Config> =
        StorageMap<_, Blake2_128Concat, OptionIdOf<T>, OptionToken<T>>;

    /// Maps option's hash [`H256`](H256) with the option id
    /// [`OptionId`](OptionIdOf). Used to quickly check if option exists
    /// and for all the other searching use cases.
    #[pallet::storage]
    #[pallet::getter(fn options_hash)]
    pub type OptionHashToOptionId<T: Config> = StorageMap<_, Blake2_128Concat, H256, OptionIdOf<T>>;

    /// Maps [`AccountId`](frame_system::Config::AccountId) and option id
    /// [`OptionId`](OptionIdOf) to the user's
    /// `SellerPosition`.
    #[pallet::storage]
    #[pallet::getter(fn sellers)]
    pub type Sellers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        OptionIdOf<T>,
        Blake2_128Concat,
        AccountIdOf<T>,
        SellerPosition<T>,
    >;

    /// Maps a timestamp [`Moment`](Config::Moment) and option id
    /// [`OptionId`](OptionIdOf) to its currently active window type.
    /// Scheduler is a timestamp-ordered list.
    #[pallet::storage]
    pub(crate) type Scheduler<T: Config> =
        StorageDoubleMap<_, Identity, Swapped<MomentOf<T>>, Identity, OptionIdOf<T>, Status>;

    // ----------------------------------------------------------------------------------------------------
    //		Events
    // ----------------------------------------------------------------------------------------------------
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted after a successful call to the
        /// [`create_asset_vault`](Pallet::create_asset_vault) extrinsic.
        CreatedAssetVault {
            vault_id: VaultIdOf<T>,
            asset_id: AssetIdOf<T>,
        },

        /// Emitted after a successful call to the [`create_option`](Pallet::create_option)
        /// extrinsic.
        CreatedOption {
            option_id: OptionIdOf<T>,
            option_config: OptionConfigOf<T>,
        },

        /// Emitted after a successful call to the [`sell_option`](Pallet::sell_option) extrinsic.
        SellOption {
            user: AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        },

        /// Emitted after a successful call to the
        /// [`delete_sell_option`](Pallet::delete_sell_option) extrinsic.
        DeleteSellOption {
            user: AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        },

        /// Emitted after a successful call to the [`buy_option`](Pallet::buy_option) extrinsic.
        BuyOption {
            user: AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        },

        /// Emitted after a successful call to the `do_settle_option`
        /// function.
        SettleOption { option_id: OptionIdOf<T> },

        /// Emitted after a successful call to the `exercise_option`
        /// extrinsic.
        ExerciseOption {
            user: AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        },

        /// Emitted after a successful call to the
        /// [`withdraw_collateral`](Pallet::withdraw_collateral) extrinsic.
        WithdrawCollateral {
            user: AccountIdOf<T>,
            option_id: OptionIdOf<T>,
        },

        /// Emitted when the deposit phase for the reported option starts.
        OptionDepositStart { option_id: OptionIdOf<T> },

        /// Emitted when the purchase phase for the reported option starts.
        OptionPurchaseStart { option_id: OptionIdOf<T> },

        /// Emitted when the exercise phase for the reported option starts.
        OptionExerciseStart { option_id: OptionIdOf<T> },

        /// Emitted when the reported option epoch ends.
        OptionEnd { option_id: OptionIdOf<T> },
    }

    // ----------------------------------------------------------------------------------------------------
    //		Errors
    // ----------------------------------------------------------------------------------------------------
    #[pallet::error]
    pub enum Error<T> {
        UnexpectedError,

        /// Raised when trying to create a new vault, but the asset is not supported by the Oracle.
        AssetIsNotSupported,

        /// Raised when trying to retrieve the vault associated to an asset, but it does not exist.
        AssetVaultDoesNotExists,

        /// Raised when trying to create a new vault, but it already exists.
        AssetVaultAlreadyExists,

        /// Raised when trying to retrieve the option corresponding to the given option id,
        /// but it does not exist.
        OptionDoesNotExists,

        /// Raised when trying to create a new option, but it already exists.
        OptionAlreadyExists,

        /// Raised when trying to create a new option, but at least one between base asset
        /// and quote asset vaults do not exist.
        OptionAssetVaultsDoNotExist,

        /// Raised when trying to create a new option, but at least one of the option's attributes
        /// has an invalid value.
        OptionAttributesAreInvalid,

        /// Raised when trying to sell an option, but the user does not own enough collateral to
        /// complete the operation.
        UserHasNotEnoughFundsToDeposit,

        /// Raised when trying to sell an option, but deposits into vaults are disabled.
        VaultDepositNotAllowed,

        /// Raised when trying to sell/delete/buy an option, but the option amount is zero.
        CannotPassZeroOptionAmount,

        /// Raised when trying to delete the sale of an option or withdraw collateral, but
        /// the user had never sold the indicated option before or has not collateral to withdraw.
        UserDoesNotHaveSellerPosition,

        /// Raised when trying to delete the sale of an option, but the user is trying to withdraw
        /// more collateral than provided.
        UserDoesNotHaveEnoughCollateralDeposited,

        /// Raised when trying to delete the sale of an option, but withdrawals from vaults are
        /// disabled.
        VaultWithdrawNotAllowed,

        /// Raised when trying to buy an option, but there are not enough options for sale.
        NotEnoughOptionsForSale,

        /// Raised when trying to exercise options, but the amount is greater than what user owns.
        UserHasNotEnoughOptionTokens,

        /// Raised when trying to get the price for a specific asset, but the asset is not found in
        /// the Oracle.
        AssetPriceNotFound,

        /// Raised when trying to sell an option, but it is not deposit phase for that option.
        NotIntoDepositWindow,

        /// Raised when trying to buy an option, but it is not purchase phase for that option.
        NotIntoPurchaseWindow,

        /// Raised when trying to exercise an option, but it is not exercise phase for that option.
        NotIntoExerciseWindow,
    }

    // ----------------------------------------------------------------------------------------------------
    //		Hooks
    // ----------------------------------------------------------------------------------------------------

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        // TODO: use on_post_inherent after https://github.com/paritytech/substrate/pull/10128 is merged.
        /// At each block we perform timestamp checks to update the Scheduler.
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            let mut used_weight = 0;
            let now = T::Time::now();

            while let Some((moment_swapped, option_id, moment_type)) = <Scheduler<T>>::iter().next()
            {
                used_weight = used_weight.saturating_add(T::DbWeight::get().reads(1));
                let moment = moment_swapped.into_value();

                if now < moment {
                    break
                }

                <Scheduler<T>>::remove(moment_swapped, &option_id);

                used_weight = used_weight
                    .saturating_add(T::DbWeight::get().writes(1))
                    .saturating_add(Self::option_status_change(option_id, moment_type));
            }
            let max_weight = <T as frame_system::Config>::BlockWeights::get().max_block;
            used_weight.min(max_weight)
        }
    }

    // ----------------------------------------------------------------------------------------------------
    //		Extrinsics
    // ----------------------------------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new vault for the given asset and save the vault id on storage.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `vault_config`: the configuration of the vault to create.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the protocol account.
        /// 2. The vault should not already exist.
        /// 3. The asset should be supported by the Oracle.
        ///
        /// ## Emits
        /// - [`Event::CreatedAssetVault`]
        ///
        /// ## State Changes
        /// - Updates the [`AssetToVault`] storage mapping the asset id with the new created vault
        ///   id.
        ///
        /// ## Errors
        /// - [`AssetIsNotSupported`](Error::AssetIsNotSupported): raised when trying to create a
        ///   new vault,
        ///  but the asset is not supported by the Oracle.
        /// - [`AssetVaultAlreadyExists`](Error::AssetVaultAlreadyExists): raised when trying to
        ///   create a new vault,
        /// but it already exists.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::create_asset_vault())]
        pub fn create_asset_vault(
            origin: OriginFor<T>,
            vault_config: VaultConfigOf<T>,
        ) -> DispatchResult {
            // Check if it's protocol to call the extrinsic
            T::ProtocolOrigin::ensure_origin(origin)?;

            <Self as TokenizedOptions>::create_asset_vault(vault_config)?;

            Ok(())
        }

        /// Create a new option and save the option's id, option's hash and option's epoch details
        /// on storage.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_config`: the configuration of the option to create.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the protocol account.
        /// 2. The option should not already exist.
        /// 3. Both the base asset and the quote asset vaults should exist.
        /// 4. The option attributes should all have valid values.
        ///
        /// ## Emits
        /// - [`Event::CreatedOption`]
        ///
        /// ## State Changes
        /// - Updates the [`OptionIdToOption`] storage mapping the option id with the created
        ///   option.
        /// - Updates the [`OptionHashToOptionId`] storage mapping the option hash with the
        ///   generated option id.
        /// - Updates the `Scheduler` storage inserting the timestamps when the option should change
        ///   phases.
        ///
        /// ## Errors
        /// - [`OptionAlreadyExists`](Error::OptionAlreadyExists): raised when trying to create a
        ///   new option,
        /// but it already exists.
        /// - [`OptionAssetVaultsDoNotExist`](Error::OptionAssetVaultsDoNotExist): raised when
        ///   trying to create a new option,
        /// but at least one between base asset and quote asset vaults do not exist.
        /// - [`OptionAttributesAreInvalid`](Error::OptionAttributesAreInvalid): raised when trying
        ///   to create a new option,
        /// but at least one of the option's attributes has an invalid value.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::create_option())]
        pub fn create_option(
            origin: OriginFor<T>,
            option_config: OptionConfigOf<T>,
        ) -> DispatchResult {
            // Check if it's protocol to call the extrinsic
            T::ProtocolOrigin::ensure_origin(origin)?;

            <Self as TokenizedOptions>::create_option(option_config)?;

            Ok(())
        }

        /// Sell the indicated option and save the seller's position.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_amount`: the amount of option the user wants to sell.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the user.
        /// 2. The option should exist.
        /// 3. The option should be in deposit phase.
        /// 4. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::SellOption`]
        ///
        /// ## State Changes
        /// - Updates the [`Sellers`] storage mapping the option id and the user's account id with
        ///   his position.
        /// - Updates the [`OptionIdToOption`] storage adding the amount of option to sell to the
        ///   total amount
        /// already for sale.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`AssetVaultDoesNotExists`](Error::AssetVaultDoesNotExists): raised when trying to
        ///   retrieve the vault
        /// associated to an asset, but it does not exist.
        /// - [`NotIntoDepositWindow`](Error::NotIntoDepositWindow): raised when trying to sell an
        ///   option,
        /// but it is not deposit phase for that option.
        /// - [`UserHasNotEnoughFundsToDeposit`](Error::UserHasNotEnoughFundsToDeposit): raised when
        ///   trying to sell an option,
        /// but the user does not own enough collateral to complete the operation.
        /// - [`VaultDepositNotAllowed`](Error::VaultDepositNotAllowed): raised when trying to sell
        ///   an option,
        /// but deposits into vaults are disabled.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to sell an option,
        /// but the option amount is zero.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::sell_option())]
        pub fn sell_option(
            origin: OriginFor<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            <Self as TokenizedOptions>::sell_option(&from, option_amount, option_id)?;

            Ok(())
        }

        /// Delete the sale of the indicated option and update the seller's position.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_amount`: the amount of option the user wants to delete the sale of.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the user.
        /// 2. The option should exist.
        /// 3. The option should be in deposit phase.
        /// 4. The user should already have a seller position for the chosen option.
        /// 5. The option amount to delete should not be zero.
        ///
        /// ## Emits
        /// - [`Event::DeleteSellOption`]
        ///
        /// ## State Changes
        /// - Updates the [`Sellers`] storage mapping the option id and the user's account id with
        ///   his position.
        /// - Updates the [`OptionIdToOption`] storage subtracting the amount of option to delete
        ///   the sale of from the total amount
        /// already for sale.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`AssetVaultDoesNotExists`](Error::AssetVaultDoesNotExists): raised when trying to
        ///   retrieve the vault
        /// associated to an asset, but it does not exist.
        /// - [`NotIntoDepositWindow`](Error::NotIntoDepositWindow): raised when trying to sell an
        ///   option,
        /// but it is not deposit phase for that option.
        /// - [`UserDoesNotHaveSellerPosition`](Error::UserDoesNotHaveSellerPosition): raised when
        ///   trying to delete the sale of an option,
        /// but the user had never sold the indicated option.
        /// - [`UserDoesNotHaveEnoughCollateralDeposited`](Error::UserDoesNotHaveEnoughCollateralDeposited): raised when trying
        /// to delete the sale of an option, but the user is trying to withdraw more collateral than
        /// provided.
        /// - [`VaultWithdrawNotAllowed`](Error::VaultWithdrawNotAllowed): raised when trying to
        ///   delete the sale of an option,
        /// but withdrawals from vaults are disabled.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to delete the sale of an option,
        /// but the option amount is zero.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::delete_sell_option())]
        pub fn delete_sell_option(
            origin: OriginFor<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            <Self as TokenizedOptions>::delete_sell_option(&from, option_amount, option_id)?;

            Ok(())
        }

        /// Buy the indicated option paying the corresponding premium.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_amount`: the amount of option the user wants to buy.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the user.
        /// 2. The option should exist.
        /// 3. The option should be in purchase phase.
        /// 4. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::BuyOption`]
        ///
        /// ## State Changes
        /// - Updates the [`OptionIdToOption`] storage adding the amount of option to buy to the
        ///   total amount
        /// already bought.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoPurchaseWindow`](Error::NotIntoPurchaseWindow): raised when trying to buy an
        ///   option,
        /// but it is not purchase phase for that option.
        /// - [`UserHasNotEnoughFundsToDeposit`](Error::UserHasNotEnoughFundsToDeposit): raised when
        ///   trying to buy an option,
        /// but the user does not own enough funds to complete the operation paying the premium.
        /// - [`NotEnoughOptionsForSale`](Error::NotEnoughOptionsForSale): raised when trying to buy
        ///   an option,
        /// but there are not enough option for sale to complete the purchase.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to buy an option,
        /// but the option amount is zero.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::buy_option())]
        pub fn buy_option(
            origin: OriginFor<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            <Self as TokenizedOptions>::buy_option(&from, option_amount, option_id)?;

            Ok(())
        }

        /// Exercise the indicated option.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_amount`: the amount of option the user wants to exercise.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the user.
        /// 2. The option should exist.
        /// 3. The option should be in exercise phase.
        /// 4. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::ExerciseOption`]
        ///
        /// ## State Changes
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoExerciseWindow`](Error::NotIntoExerciseWindow): raised when trying to
        ///   exercise an option,
        /// but it is not exercise phase for that option.
        /// - [`UserHasNotEnoughOptionTokens`](Error::UserHasNotEnoughOptionTokens): raised when
        ///   trying to exercise options,
        /// but the amount is greater than what user owns.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to exercise an option,
        /// but the option amount is zero.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::exercise_option())]
        pub fn exercise_option(
            origin: OriginFor<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            <Self as TokenizedOptions>::exercise_option(&from, option_amount, option_id)?;

            Ok(())
        }

        /// Withdraw the seller's collateral related to the indicated option.
        ///
        /// # Overview
        /// ## Parameters
        /// - `origin`: type representing the origin of this dispatch.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The call must have been signed by the user.
        /// 2. The option should exist.
        /// 3. The option should be in exercise phase.
        ///
        /// ## Emits
        /// - [`Event::WithdrawCollateral`]
        ///
        /// ## State Changes
        /// - Delete from [`Sellers`] storage the seller's position related to the option.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoExerciseWindow`](Error::NotIntoExerciseWindow): raised when trying to
        ///   withdraw collateral,
        /// but it is not exercise phase for that option.
        /// - [`UserDoesNotHaveSellerPosition`](Error::UserDoesNotHaveSellerPosition): raised when
        ///   trying to withdraw collateral,
        /// but the seller has not a seller position.
        /// - [`VaultWithdrawNotAllowed`](Error::VaultWithdrawNotAllowed): raised when trying to
        ///   withdraw collateral,
        /// but withdrawals from vaults are disabled.
        ///
        /// # Examples
        ///
        /// # Weight: O(TBD)
        #[pallet::weight(<T as Config>::WeightInfo::withdraw_collateral())]
        pub fn withdraw_collateral(
            origin: OriginFor<T>,
            option_id: OptionIdOf<T>,
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;

            <Self as TokenizedOptions>::withdraw_collateral(&from, option_id)?;

            Ok(())
        }
    }

    // ----------------------------------------------------------------------------------------------------
    //		TokenizedOptions Trait
    // ----------------------------------------------------------------------------------------------------
    impl<T: Config> TokenizedOptions for Pallet<T> {
        type AccountId = AccountIdOf<T>;
        type OptionId = OptionIdOf<T>;
        type Balance = BalanceOf<T>;
        type Moment = MomentOf<T>;
        type VaultId = VaultIdOf<T>;
        type OptionConfig = OptionConfigOf<T>;
        type VaultConfig = VaultConfigOf<T>;

        /// Create a new vault for the given asset and save the vault id on storage.
        ///
        /// # Overview
        /// ## Parameters
        /// - `vault_config`: the configuration of the vault to create.
        ///
        /// ## Requirements
        /// 1. The vault should not already exist.
        /// 2. The asset should be supported by the Oracle.
        ///
        /// ## Emits
        /// - [`Event::CreatedAssetVault`]
        ///
        /// ## State Changes
        /// - Updates the [`AssetToVault`] storage mapping the asset id with the new created vault
        ///   id.
        ///
        /// ## Errors
        /// - [`AssetIsNotSupported`](Error::AssetIsNotSupported): raised when trying to create a
        ///   new vault,
        ///  but the asset is not supported by the Oracle.
        /// - [`AssetVaultAlreadyExists`](Error::AssetVaultAlreadyExists): raised when trying to
        ///   create a new vault,
        /// but it already exists.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn create_asset_vault(
            vault_config: Self::VaultConfig,
        ) -> Result<Self::VaultId, DispatchError> {
            match Validated::new(vault_config) {
                Ok(validated_vault_config) => Self::do_create_asset_vault(validated_vault_config),
                Err(error) => match error {
                    "ValidateVaultDoesNotExist" => Err(Error::<T>::AssetVaultAlreadyExists.into()),
                    "ValidateAssetIsSupported" => Err(Error::<T>::AssetIsNotSupported.into()),
                    _ => Err(Error::<T>::UnexpectedError.into()),
                },
            }
        }

        /// Create a new option and save the option's id, option's hash and option's epoch on
        /// storage.
        ///
        /// # Overview
        /// ## Parameters
        /// - `option_config`: the configuration of the option to create.
        ///
        /// ## Requirements
        /// 1. The option should not already exist.
        /// 2. Both the base asset and the quote asset vaults should exist.
        /// 3. The option attributes should all have valid values.
        ///
        /// ## Emits
        /// - [`Event::CreatedOption`]
        ///
        /// ## State Changes
        /// - Updates the [`OptionIdToOption`] storage mapping the option id with the created
        ///   option.
        /// - Updates the [`OptionHashToOptionId`] storage mapping the option hash with the
        ///   generated option id.
        /// - Updates the `Scheduler` storage inserting the timestamps when the option should change
        ///   phases.
        ///
        /// ## Errors
        /// - [`OptionAlreadyExists`](Error::OptionAlreadyExists): raised when trying to create a
        ///   new option,
        /// but it already exists.
        /// - [`OptionAssetVaultsDoNotExist`](Error::OptionAssetVaultsDoNotExist): raised when
        ///   trying to create a new option,
        /// but at least one between base asset and quote asset vaults do not exist.
        /// - [`OptionAttributesAreInvalid`](Error::OptionAttributesAreInvalid): raised when trying
        ///   to create a new option,
        /// but at least one of the option's attributes has an invalid value.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn create_option(
            option_config: Self::OptionConfig,
        ) -> Result<Self::OptionId, DispatchError> {
            match Validated::new(option_config) {
                Ok(validated_option_config) => Self::do_create_option(validated_option_config),
                Err(error) => match error {
                    "ValidateOptionDoesNotExist" => Err(Error::<T>::OptionAlreadyExists.into()),
                    "ValidateOptionAssetVaultsExist" =>
                        Err(Error::<T>::OptionAssetVaultsDoNotExist.into()),
                    "ValidateOptionAttributes" =>
                        Err(Error::<T>::OptionAttributesAreInvalid.into()),
                    _ => Err(Error::<T>::UnexpectedError.into()),
                },
            }
        }

        /// Sell the indicated option and save the seller's position.
        ///
        /// # Overview
        /// ## Parameters
        /// - `from`: the user's account id.
        /// - `option_amount`: the amount of option the user wants to sell.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The option should exist.
        /// 2. The option should be in deposit phase.
        /// 3. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::SellOption`]
        ///
        /// ## State Changes
        /// - Updates the [`Sellers`] storage mapping the option id and the user's account id with
        ///   his position.
        /// - Updates the [`OptionIdToOption`] storage adding the amount of option to sell to the
        ///   total amount
        /// already for sale.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`AssetVaultDoesNotExists`](Error::AssetVaultDoesNotExists): raised when trying to
        ///   retrieve the vault
        /// associated to an asset, but it does not exist.
        /// - [`NotIntoDepositWindow`](Error::NotIntoDepositWindow): raised when trying to sell an
        ///   option,
        /// but it is not deposit phase for that option.
        /// - [`UserHasNotEnoughFundsToDeposit`](Error::UserHasNotEnoughFundsToDeposit): raised when
        ///   trying to sell an option,
        /// but the user does not own enough collateral to complete the operation.
        /// - [`VaultDepositNotAllowed`](Error::VaultDepositNotAllowed): raised when trying to sell
        ///   an option,
        /// but deposits into vaults are disabled.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to sell an option,
        /// but the option amount is zero.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn sell_option(
            from: &Self::AccountId,
            option_amount: Self::Balance,
            option_id: Self::OptionId,
        ) -> Result<(), DispatchError> {
            OptionIdToOption::<T>::try_mutate(option_id, |option| match option {
                Some(option) => Self::do_sell_option(from, option_amount, option_id, option),
                None => Err(Error::<T>::OptionDoesNotExists.into()),
            })
        }

        /// Delete the selling of the indicated option and update the seller's position.
        ///
        /// # Overview
        /// ## Parameters
        /// - `from`: the user's account id.
        /// - `option_amount`: the amount of option the user wants to delete the sale of.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The option should exist.
        /// 2. The option should be in deposit phase.
        /// 3. The user should already have a seller position for the chosen option.
        /// 4. The option amount to delete should not be zero.
        ///
        /// ## Emits
        /// - [`Event::DeleteSellOption`]
        ///
        /// ## State Changes
        /// - Updates the [`Sellers`] storage mapping the option id and the user's account id with
        ///   his position.
        /// - Updates the [`OptionIdToOption`] storage subtracting the amount of option to delete
        ///   the sale of from the total amount
        /// already for sale.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`AssetVaultDoesNotExists`](Error::AssetVaultDoesNotExists): raised when trying to
        ///   retrieve the vault
        /// associated to an asset, but it does not exist.
        /// - [`NotIntoDepositWindow`](Error::NotIntoDepositWindow): raised when trying to sell an
        ///   option,
        /// but it is not deposit phase for that option.
        /// - [`UserDoesNotHaveSellerPosition`](Error::UserDoesNotHaveSellerPosition): raised when
        ///   trying to delete the sale of an option,
        /// but the user had never sold the indicated option.
        /// - [`UserDoesNotHaveEnoughCollateralDeposited`](Error::UserDoesNotHaveEnoughCollateralDeposited): raised when trying
        /// to delete the sale of an option, but the user is trying to withdraw more collateral than
        /// provided.
        /// - [`VaultWithdrawNotAllowed`](Error::VaultWithdrawNotAllowed): raised when trying to
        ///   delete the sale of an option,
        /// but withdrawals from vaults are disabled.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to delete the sale of an option,
        /// but the option amount is zero.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn delete_sell_option(
            from: &Self::AccountId,
            option_amount: Self::Balance,
            option_id: Self::OptionId,
        ) -> Result<(), DispatchError> {
            OptionIdToOption::<T>::try_mutate(option_id, |option| match option {
                Some(option) => Sellers::<T>::try_mutate(option_id, from, |position| {
                    Self::do_delete_sell_option(from, option_amount, option_id, option, position)
                }),
                None => Err(Error::<T>::OptionDoesNotExists.into()),
            })
        }

        /// Buy the indicated option.
        ///
        /// # Overview
        /// ## Parameters
        /// - `from`: user's account id.
        /// - `option_amount`: the amount of option the user wants to buy.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The option should exist.
        /// 2. The option should be in purchase phase.
        /// 3. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::BuyOption`]
        ///
        /// ## State Changes
        /// - Updates the [`OptionIdToOption`] storage adding the amount of option to buy to the
        ///   total amount
        /// already bought.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoPurchaseWindow`](Error::NotIntoPurchaseWindow): raised when trying to buy an
        ///   option,
        /// but it is not purchase phase for that option.
        /// - [`UserHasNotEnoughFundsToDeposit`](Error::UserHasNotEnoughFundsToDeposit): raised when
        ///   trying to buy an option,
        /// but the user does not own enough funds to complete the operation paying the premium.
        /// - [`NotEnoughOptionsForSale`](Error::NotEnoughOptionsForSale): raised when trying to buy
        ///   an option,
        /// but there are not enough option for sale to complete the purchase.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to buy an option,
        /// but the option amount is zero.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn buy_option(
            from: &Self::AccountId,
            option_amount: Self::Balance,
            option_id: Self::OptionId,
        ) -> Result<(), DispatchError> {
            OptionIdToOption::<T>::try_mutate(option_id, |option| match option {
                Some(option) => Self::do_buy_option(from, option_amount, option_id, option),
                None => Err(Error::<T>::OptionDoesNotExists.into()),
            })
        }

        /// Exercise the indicated option.
        ///
        /// # Overview
        /// ## Parameters
        /// - `from`: user's account id.
        /// - `option_amount`: the amount of option the user wants to exercise.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The option should exist.
        /// 2. The option should be in exercise phase.
        /// 3. The option amount should not be zero.
        ///
        /// ## Emits
        /// - [`Event::ExerciseOption`]
        ///
        /// ## State Changes
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoExerciseWindow`](Error::NotIntoExerciseWindow): raised when trying to
        ///   exercise an option,
        /// but it is not exercise phase for that option.
        /// - [`UserHasNotEnoughOptionTokens`](Error::UserHasNotEnoughOptionTokens): raised when
        ///   trying to exercise options,
        /// but the amount is greater than what user owns.
        /// - [`CannotPassZeroOptionAmount`](Error::CannotPassZeroOptionAmount): raised when trying
        ///   to exercise an option,
        /// but the option amount is zero.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn exercise_option(
            from: &Self::AccountId,
            option_amount: Self::Balance,
            option_id: Self::OptionId,
        ) -> Result<(), DispatchError> {
            match Self::option_id_to_option(option_id) {
                Some(option) => Self::do_exercise_option(from, option_amount, option_id, &option),
                None => Err(Error::<T>::OptionDoesNotExists.into()),
            }
        }

        /// Withdraw the seller's collateral related to the indicated option.
        ///
        /// # Overview
        /// ## Parameters
        /// - `from`: user's account id.
        /// - `option_id`: the option id.
        ///
        /// ## Requirements
        /// 1. The option should exist.
        /// 2. The option should be in exercise phase.
        ///
        /// ## Emits
        /// - [`Event::WithdrawCollateral`]
        ///
        /// ## State Changes
        /// - Delete from [`Sellers`] storage the seller's position related to the option.
        ///
        /// ## Errors
        /// - [`OptionDoesNotExists`](Error::OptionDoesNotExists): raised when trying to retrieve
        ///   the option corresponding
        /// to the given option id, but it does not exist.
        /// - [`NotIntoExerciseWindow`](Error::NotIntoExerciseWindow): raised when trying to
        ///   withdraw collateral,
        /// but it is not exercise phase for that option.
        /// - [`UserDoesNotHaveSellerPosition`](Error::UserDoesNotHaveSellerPosition): raised when
        ///   trying to withdraw collateral,
        /// but the seller has not a seller position.
        /// - [`VaultWithdrawNotAllowed`](Error::VaultWithdrawNotAllowed): raised when trying to
        ///   withdraw collateral,
        /// but withdrawals from vaults are disabled.
        ///
        /// # Weight: O(TBD)
        #[transactional]
        fn withdraw_collateral(
            from: &Self::AccountId,
            option_id: Self::OptionId,
        ) -> Result<(), DispatchError> {
            match Self::option_id_to_option(option_id) {
                Some(option) => Sellers::<T>::try_mutate(option_id, from, |position| {
                    Self::do_withdraw_collateral(from, option_id, &option, position)
                }),
                None => Err(Error::<T>::OptionDoesNotExists.into()),
            }
        }
    }

    // ----------------------------------------------------------------------------------------------------
    //		Internal Pallet Functions
    // ----------------------------------------------------------------------------------------------------
    impl<T: Config> Pallet<T> {
        fn do_create_asset_vault(
            config: Validated<
                VaultConfigOf<T>,
                (ValidateVaultDoesNotExist<T>, ValidateAssetIsSupported<T>),
            >,
        ) -> Result<VaultIdOf<T>, DispatchError> {
            // Get pallet account for the asset
            let account_id = Self::account_id(config.asset_id);

            // Create new vault for the asset
            let asset_vault_id: VaultIdOf<T> = VaultOf::<T>::create(
                Duration::Existential,
                VaultConfig {
                    asset_id: config.asset_id,
                    manager: account_id,
                    reserved: config.reserved,
                    strategies: config.strategies.clone(),
                },
            )?;

            // Add asset to the corresponding asset vault
            AssetToVault::<T>::insert(config.asset_id, asset_vault_id);

            Self::deposit_event(Event::CreatedAssetVault {
                vault_id: asset_vault_id,
                asset_id: config.asset_id,
            });

            Ok(asset_vault_id)
        }

        fn do_create_option(
            option_config: Validated<
                OptionConfigOf<T>,
                (
                    ValidateOptionDoesNotExist<T>,
                    ValidateOptionAssetVaultsExist<T>,
                    ValidateOptionAttributes<T>,
                ),
            >,
        ) -> Result<OptionIdOf<T>, DispatchError> {
            // Generate new option_id for the option token
            let option_id =
                T::CurrencyFactory::create(RangeId::LP_TOKENS, BalanceOf::<T>::default())?;

            let option = OptionToken {
                base_asset_id: option_config.base_asset_id,
                quote_asset_id: option_config.quote_asset_id,
                base_asset_strike_price: option_config.base_asset_strike_price,
                quote_asset_strike_price: option_config.quote_asset_strike_price,
                option_type: option_config.option_type,
                exercise_type: option_config.exercise_type,
                expiring_date: option_config.expiring_date,
                epoch: option_config.epoch,
                status: Status::NotStarted,
                base_asset_amount_per_option: option_config.base_asset_amount_per_option,
                quote_asset_amount_per_option: option_config.quote_asset_amount_per_option,
                total_issuance_seller: option_config.total_issuance_seller,
                total_premium_paid: option_config.total_premium_paid,
                exercise_amount: option_config.exercise_amount,
                base_asset_spot_price: option_config.base_asset_spot_price,
                total_issuance_buyer: option_config.total_issuance_buyer,
                total_shares_amount: option_config.total_shares_amount,
            };

            let option_hash = option.generate_id();

            // Add option_id to corresponding option
            OptionHashToOptionId::<T>::insert(option_hash, option_id);
            OptionIdToOption::<T>::insert(option_id, option);
            Self::schedule_option(option_config.epoch, option_id);

            Self::deposit_event(Event::CreatedOption {
                option_id,
                option_config: option_config.value(),
            });

            Ok(option_id)
        }

        fn do_sell_option(
            from: &AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
            option: &mut OptionToken<T>,
        ) -> Result<(), DispatchError> {
            ensure!(
                option_amount != BalanceOf::<T>::zero(),
                Error::<T>::CannotPassZeroOptionAmount
            );

            // Check if we are in deposit window
            ensure!(
                option.status == Status::Deposit,
                Error::<T>::NotIntoDepositWindow
            );

            // Different behaviors based on Call or Put option
            let (asset_id, asset_amount) = match option.option_type {
                // For CALL options it should be `base_asset_amount_per_option` *
                // `quote_asset_strike_price`
                OptionType::Call => (option.base_asset_id, option.quote_asset_strike_price),
                // For PUT options it should be `quote_asset_amount_per_option` *
                // `base_asset_strike_price`
                OptionType::Put => (option.quote_asset_id, option.base_asset_strike_price),
            };

            let asset_amount = asset_amount
                .checked_mul(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            // Get vault_id for depositing collateral
            let vault_id =
                Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

            // Calculate the amount of shares the user should get and make checks
            let shares_amount = VaultOf::<T>::calculate_lp_tokens_to_mint(&vault_id, asset_amount)?;

            // Update position or create position
            Sellers::<T>::try_mutate(option_id, from, |position| -> Result<(), DispatchError> {
                match position {
                    Some(position) => {
                        // Add option amount to position
                        let new_option_amount = position
                            .option_amount
                            .checked_add(&option_amount)
                            .ok_or(ArithmeticError::Overflow)?;

                        // Add shares amount to position
                        let new_shares_amount = position
                            .shares_amount
                            .checked_add(&shares_amount)
                            .ok_or(ArithmeticError::Overflow)?;

                        position.option_amount = new_option_amount;
                        position.shares_amount = new_shares_amount;
                    },
                    None =>
                        *position = Some(SellerPosition {
                            option_amount,
                            shares_amount,
                        }),
                }
                Ok(())
            })?;

            // Transfer collateral to protocol account
            let protocol_account = Self::account_id(asset_id);
            AssetsOf::<T>::transfer(asset_id, from, &protocol_account, asset_amount, true)
                .map_err(|_| Error::<T>::UserHasNotEnoughFundsToDeposit)?;

            // Protocol account deposits into the vault and keep asset_amount
            VaultOf::<T>::deposit(&vault_id, &protocol_account, asset_amount)
                .map_err(|_| Error::<T>::VaultDepositNotAllowed)?;

            // Add option amount to total issuance
            let new_total_issuance_seller = option
                .total_issuance_seller
                .checked_add(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            option.total_issuance_seller = new_total_issuance_seller;

            Self::deposit_event(Event::SellOption {
                user: from.clone(),
                option_amount,
                option_id,
            });

            Ok(())
        }

        fn do_delete_sell_option(
            from: &AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
            option: &mut OptionToken<T>,
            position: &mut Option<SellerPosition<T>>,
        ) -> Result<(), DispatchError> {
            ensure!(
                option_amount != BalanceOf::<T>::zero(),
                Error::<T>::CannotPassZeroOptionAmount
            );

            // Check if we are in deposit window
            ensure!(
                option.status == Status::Deposit,
                Error::<T>::NotIntoDepositWindow
            );

            // Check if user has deposited any collateral before and retrieve position
            let seller_position = position
                .as_mut()
                .ok_or(Error::<T>::UserDoesNotHaveSellerPosition)?;

            // Different behaviors based on Call or Put option
            let asset_id = match option.option_type {
                OptionType::Call => option.base_asset_id,
                OptionType::Put => option.quote_asset_id,
            };

            // Get vault_id for withdrawing collateral
            let vault_id =
                Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

            // Calculate shares amount to withdraw
            let shares_amount = Self::convert_and_multiply_by_rational(
                seller_position.shares_amount,
                option_amount,
                seller_position.option_amount,
                Rounding::Down,
            )?;

            let asset_amount = VaultOf::<T>::lp_share_value(&vault_id, shares_amount)?;

            // Sanity checks
            // 1. Asset amount <= Max asset amount withdrawable by user
            // 2. Option amount <= Max option amount withdrawable by user
            ensure!(
                asset_amount <=
                    VaultOf::<T>::lp_share_value(&vault_id, seller_position.shares_amount)? &&
                    option_amount <= seller_position.option_amount,
                Error::<T>::UserDoesNotHaveEnoughCollateralDeposited
            );

            // Update position or delete position
            if shares_amount != seller_position.shares_amount {
                // Subtract option amount to position
                let new_option_amount = seller_position
                    .option_amount
                    .checked_sub(&option_amount)
                    .ok_or(ArithmeticError::Overflow)?;

                // Subtract shares amount to position
                let new_shares_amount = seller_position
                    .shares_amount
                    .checked_sub(&shares_amount)
                    .ok_or(ArithmeticError::Overflow)?;

                seller_position.option_amount = new_option_amount;
                seller_position.shares_amount = new_shares_amount;
            } else {
                *position = None;
            }
            // Protocol account withdraw from the vault and burn shares_amount
            let protocol_account = Self::account_id(asset_id);
            VaultOf::<T>::withdraw(&vault_id, &protocol_account, shares_amount)
                .map_err(|_| Error::<T>::VaultWithdrawNotAllowed)?;

            // Transfer collateral to user account
            AssetsOf::<T>::transfer(asset_id, &protocol_account, from, asset_amount, true)?;

            // Subtract option amount to total issuance
            let new_total_issuance_seller = option
                .total_issuance_seller
                .checked_sub(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            option.total_issuance_seller = new_total_issuance_seller;

            Self::deposit_event(Event::DeleteSellOption {
                user: from.clone(),
                option_amount,
                option_id,
            });
            Ok(())
        }

        fn do_buy_option(
            from: &AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
            option: &mut OptionToken<T>,
        ) -> Result<(), DispatchError> {
            ensure!(
                option_amount != BalanceOf::<T>::zero(),
                Error::<T>::CannotPassZeroOptionAmount
            );

            let stablecoin_id = T::StablecoinAssetId::get();

            // Check if we are in purchase window
            ensure!(
                option.status == Status::Purchase,
                Error::<T>::NotIntoPurchaseWindow
            );

            // Fake call to pricing pallet (to replace with actual call to pricing pallet)
            // let option_premium = Self::fake_option_price().expect("Error pricing option");

            let bs_params = BlackScholesParams {
                base_asset_id: option.base_asset_id,
                base_asset_strike_price: option.base_asset_strike_price,
                base_asset_spot_price: option.base_asset_spot_price,
                expiring_date: option.expiring_date,
                option_type: option.option_type,
                total_issuance_buyer: option.total_issuance_buyer,
                total_premium_paid: option.total_premium_paid,
            };

            // let option_premium =
            // 	OptionsPricingOf::<T>::calculate_option_price(option_id, bs_params)?;

            let option_premium = Self::fake_option_price();

            let option_premium = option_premium
                .checked_mul(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            // Check option availability
            let total_issuance_buyer = AssetsOf::<T>::total_issuance(option_id);

            let new_total_issuance_buyer = total_issuance_buyer
                .checked_add(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            // Check if there are enough options for sale
            if new_total_issuance_buyer > option.total_issuance_seller {
                return Err(DispatchError::from(Error::<T>::NotEnoughOptionsForSale))
            }

            let new_total_premium_paid = option
                .total_premium_paid
                .checked_add(&option_premium)
                .ok_or(ArithmeticError::Overflow)?;

            option.total_premium_paid = new_total_premium_paid;

            // Transfer premium to protocol account
            let protocol_account = Self::account_id(stablecoin_id);
            AssetsOf::<T>::transfer(stablecoin_id, from, &protocol_account, option_premium, true)
                .map_err(|_| Error::<T>::UserHasNotEnoughFundsToDeposit)?;

            // Mint option token into user's account
            AssetsOf::<T>::mint_into(option_id, from, option_amount)?;

            Self::deposit_event(Event::BuyOption {
                user: from.clone(),
                option_amount,
                option_id,
            });

            Ok(())
        }

        /// Settle the option specified by `option_id`.
        ///
        /// # Overview
        /// ## Parameters
        ///
        /// ## Requirements
        /// 1. The option to settle should exist.
        /// 2. The option to settle should be in exercise phase, which suppose expiration date is
        /// passed.
        ///
        /// ## Emits
        /// - [`Event::SettleOption`]
        ///
        /// ## State Changes
        /// - For each option, updates the [`OptionIdToOption`] storage calculating the exercise
        ///   amount for buyers and saving
        /// the info to calculate the remaining collateral for sellers and their share of premium.
        ///
        /// ## Errors
        /// - There should not be errors in any case.
        ///
        /// # Weight: O(TBD)
        pub(crate) fn do_settle_option(
            option_id: OptionIdOf<T>,
            option: &mut OptionToken<T>,
        ) -> Result<(), DispatchError> {
            // Get current asset's spot price
            let base_asset_spot_price = Self::get_price(option.base_asset_id)?;

            // Get total options bought
            let total_issuance_buyer = AssetsOf::<T>::total_issuance(option_id);

            // Different behaviors based on Call or Put option
            let (asset_id, collateral_for_option) = match option.option_type {
                OptionType::Call => (
                    option.base_asset_id,
                    Self::call_option_collateral_amount(base_asset_spot_price, option)?,
                ),
                OptionType::Put => (
                    option.quote_asset_id,
                    Self::put_option_collateral_amount(base_asset_spot_price, option)?,
                ),
            };

            // Calculate total_collateral to withdraw for buyers and corresponding amount of shares
            let protocol_account = Self::account_id(asset_id);

            let vault_id =
                Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

            let total_collateral = collateral_for_option
                .checked_mul(&total_issuance_buyer)
                .ok_or(ArithmeticError::Overflow)?;

            let total_shares_amount =
                VaultOf::<T>::amount_of_lp_token_for_added_liquidity(&vault_id, total_collateral)?;

            if total_shares_amount != BalanceOf::<T>::zero() {
                VaultOf::<T>::withdraw(&vault_id, &protocol_account, total_shares_amount)
                    .map_err(|_| Error::<T>::VaultWithdrawNotAllowed)?;
            };

            // Update option to calculate buyers and sellers positions
            // in exercise_option and withdraw_collateral functions
            option.exercise_amount = collateral_for_option;
            option.base_asset_spot_price = base_asset_spot_price;
            option.total_issuance_buyer = total_issuance_buyer;
            option.total_shares_amount = total_shares_amount;

            Self::deposit_event(Event::SettleOption { option_id });

            Ok(())
        }

        fn do_exercise_option(
            from: &AccountIdOf<T>,
            option_amount: BalanceOf<T>,
            option_id: OptionIdOf<T>,
            option: &OptionToken<T>,
        ) -> Result<(), DispatchError> {
            ensure!(
                option_amount != BalanceOf::<T>::zero(),
                Error::<T>::CannotPassZeroOptionAmount
            );

            // Check if we are in exercise window
            ensure!(
                option.status == Status::Exercise,
                Error::<T>::NotIntoExerciseWindow
            );

            // Different behaviors based on Call or Put option
            let asset_id = match option.option_type {
                OptionType::Call => option.base_asset_id,
                OptionType::Put => option.quote_asset_id,
            };

            let protocol_account = Self::account_id(asset_id);

            let total_amount_to_exercise = option
                .exercise_amount
                .checked_mul(&option_amount)
                .ok_or(ArithmeticError::Overflow)?;

            // Transfer buyer profit to buyer account if option is ITM
            if option.exercise_amount != BalanceOf::<T>::zero() {
                AssetsOf::<T>::transfer(
                    asset_id,
                    &protocol_account,
                    from,
                    total_amount_to_exercise,
                    true,
                )
                .map_err(|_| Error::<T>::UserHasNotEnoughOptionTokens)?;
            }

            // Burn option token from user's account
            AssetsOf::<T>::burn_from(option_id, from, option_amount)
                .map_err(|_| Error::<T>::UserHasNotEnoughOptionTokens)?;

            Self::deposit_event(Event::ExerciseOption {
                user: from.clone(),
                option_amount,
                option_id,
            });

            Ok(())
        }

        fn do_withdraw_collateral(
            from: &AccountIdOf<T>,
            option_id: OptionIdOf<T>,
            option: &OptionToken<T>,
            position: &mut Option<SellerPosition<T>>,
        ) -> Result<(), DispatchError> {
            // Check if we are in exercise window
            ensure!(
                option.status == Status::Exercise,
                Error::<T>::NotIntoExerciseWindow
            );

            // Check if user has any collateral and retrieve position
            let seller_position = position
                .as_mut()
                .ok_or(Error::<T>::UserDoesNotHaveSellerPosition)?;

            // ------ Shares calculations for user ------
            // shares_per_option = total_shares / total_option_bought
            // option_bought_ratio = total_option_bought / total_option_for_sale
            // user_shares_to_subtract = shares_per_option * option_bought_ratio *
            // user_option_amount
            let shares_for_buyers = Self::convert_and_multiply_by_rational(
                option.total_shares_amount,
                seller_position.option_amount,
                option.total_issuance_seller,
                Rounding::Down,
            )?;

            let user_shares_amount = seller_position
                .shares_amount
                .checked_sub(&shares_for_buyers)
                .ok_or(ArithmeticError::Overflow)?;

            // Do withdraw and transfer the collateral to user's account
            let asset_id = match option.option_type {
                OptionType::Call => option.base_asset_id,
                OptionType::Put => option.quote_asset_id,
            };

            let protocol_account = Self::account_id(asset_id);
            let vault_id =
                Self::asset_id_to_vault_id(asset_id).ok_or(Error::<T>::AssetVaultDoesNotExists)?;

            let lp_token_issuance =
                AssetsOf::<T>::balance(VaultOf::<T>::lp_asset_id(&vault_id)?, &protocol_account);

            let asset_amount = VaultOf::<T>::withdraw(
                &vault_id,
                &protocol_account,
                min(user_shares_amount, lp_token_issuance),
            )
            .map_err(|_| Error::<T>::VaultWithdrawNotAllowed)?;

            AssetsOf::<T>::transfer(asset_id, &protocol_account, from, asset_amount, true)?;

            // ------ Premium calculations for user ------
            // premium_per_option = total_premium_paid / total_option_bought
            // option_bought_ratio = total_option_bought / total_option_for_sale
            // user_premium = premium_per_option * option_bought_ratio * user_option_amount
            let user_premium_amount = Self::convert_and_multiply_by_rational(
                option.total_premium_paid,
                seller_position.option_amount,
                option.total_issuance_seller,
                Rounding::Down,
            )?;

            // Get info to transfer premium to seller
            let stablecoin_id = T::StablecoinAssetId::get();
            let stablecoin_protocol_account = Self::account_id(stablecoin_id);

            // Transfer premium to user account
            AssetsOf::<T>::transfer(
                stablecoin_id,
                &stablecoin_protocol_account,
                from,
                user_premium_amount,
                true,
            )?;

            // Delete position
            *position = None;

            Self::deposit_event(Event::WithdrawCollateral {
                user: from.clone(),
                option_id,
            });

            Ok(())
        }

        // ----------------------------------------------------------------------------------------------------
        //		Helper Functions
        // ----------------------------------------------------------------------------------------------------
        /// Protocol account for a particular asset.
        pub(crate) fn account_id(asset_id: AssetIdOf<T>) -> AccountIdOf<T> {
            T::PalletId::get().into_sub_account_truncating(asset_id)
        }

        /// Calculate the hash of an option providing the required attributes.
        pub(crate) fn generate_id(
            base_asset_id: AssetIdOf<T>,
            quote_asset_id: AssetIdOf<T>,
            base_asset_strike_price: BalanceOf<T>,
            quote_asset_strike_price: BalanceOf<T>,
            option_type: OptionType,
            expiring_date: MomentOf<T>,
            exercise_type: ExerciseType,
        ) -> H256 {
            BlakeTwo256::hash_of(&(
                base_asset_id,
                quote_asset_id,
                base_asset_strike_price,
                quote_asset_strike_price,
                option_type,
                expiring_date,
                exercise_type,
            ))
        }

        pub(crate) fn call_option_collateral_amount(
            base_asset_spot_price: BalanceOf<T>,
            option: &OptionToken<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            // Calculate amount of asset to reserve for buyers if the option is ITM
            let collateral_for_option = if base_asset_spot_price >= option.base_asset_strike_price {
                let diff = base_asset_spot_price
                    .checked_sub(&option.base_asset_strike_price)
                    .ok_or(ArithmeticError::Overflow)?;

                let unit = T::LocalAssets::unit::<BalanceOf<T>>(option.base_asset_id)?;

                Self::convert_and_multiply_by_rational(
                    diff,
                    unit,
                    base_asset_spot_price,
                    Rounding::NearestPrefDown,
                )?
            } else {
                BalanceOf::<T>::zero()
            };

            Ok(collateral_for_option)
        }

        pub(crate) fn put_option_collateral_amount(
            base_asset_spot_price: BalanceOf<T>,
            option: &OptionToken<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            // Calculate amount of asset to reserve for buyers if the option is ITM
            let collateral_for_option = if option.base_asset_strike_price >= base_asset_spot_price {
                option
                    .base_asset_strike_price
                    .checked_sub(&base_asset_spot_price)
                    .ok_or(ArithmeticError::Overflow)?
            } else {
                BalanceOf::<T>::zero()
            };

            Ok(collateral_for_option)
        }

        pub(crate) fn convert_and_multiply_by_rational(
            a: BalanceOf<T>,
            b: BalanceOf<T>,
            c: BalanceOf<T>,
            r: Rounding,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if c == BalanceOf::<T>::zero() {
                return Ok(BalanceOf::<T>::zero())
            };

            let a = <T::Convert as Convert<BalanceOf<T>, u128>>::convert(a);
            let b = <T::Convert as Convert<BalanceOf<T>, u128>>::convert(b);
            let c = <T::Convert as Convert<BalanceOf<T>, u128>>::convert(c);

            let res = match multiply_by_rational_with_rounding(a, b, c, r) {
                Some(res) => res,
                None => return Err(DispatchError::from(ArithmeticError::Overflow)),
            };

            let res = <T::Convert as Convert<u128, T::Balance>>::convert(res);
            Ok(res)
        }

        fn get_price(asset_id: AssetIdOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            let unit = T::LocalAssets::unit::<BalanceOf<T>>(asset_id)?;

            OracleOf::<T>::get_price(asset_id, unit)
                .map(|p| p.price)
                .map_err(|_| Error::<T>::AssetPriceNotFound.into())
        }

        pub fn fake_option_price() -> BalanceOf<T> {
            (1000u128 * 10u128.pow(12)).into()
        }

        fn schedule_option(epoch: Epoch<MomentOf<T>>, option_id: OptionIdOf<T>) {
            <Scheduler<T>>::insert(Swapped::from(epoch.deposit), option_id, Status::Deposit);
            <Scheduler<T>>::insert(Swapped::from(epoch.purchase), option_id, Status::Purchase);
            <Scheduler<T>>::insert(Swapped::from(epoch.exercise), option_id, Status::Exercise);
            <Scheduler<T>>::insert(Swapped::from(epoch.end), option_id, Status::End);
        }

        fn option_status_change(option_id: OptionIdOf<T>, moment_type: Status) -> Weight {
            OptionIdToOption::<T>::mutate(option_id, |option| match option {
                Some(option) => match moment_type {
                    // This variant shouldn't happen because we don't schedule it.
                    Status::NotStarted => 0,
                    Status::Deposit => Self::option_deposit_start(option_id, option),
                    Status::Purchase => Self::option_purchase_start(option_id, option),
                    Status::Exercise => Self::option_exercise_start(option_id, option),
                    Status::End => Self::option_end(option_id, option),
                },
                // This variant shouldn't happen because we don't delete options now;
                // and we'll delete them in the future only after they finish.
                None => 0,
            })
        }

        fn option_deposit_start(option_id: OptionIdOf<T>, option: &mut OptionToken<T>) -> Weight {
            option.status = Status::Deposit;
            Self::deposit_event(Event::OptionDepositStart { option_id });
            0
        }

        fn option_purchase_start(option_id: OptionIdOf<T>, option: &mut OptionToken<T>) -> Weight {
            option.status = Status::Purchase;
            Self::deposit_event(Event::OptionPurchaseStart { option_id });
            0
        }

        fn option_exercise_start(option_id: OptionIdOf<T>, option: &mut OptionToken<T>) -> Weight {
            // Check if option is expired is redundant if we trust the Scheduler behavior

            // TODO: Handle the result to address overflow errors or other types of errors.
            // `do_settle_option` should never return an error, but if happens, it should be
            // handled.
            Self::do_settle_option(option_id, option).expect("TODO ERROR ON SETTLE OPTION");

            option.status = Status::Exercise;
            Self::deposit_event(Event::OptionExerciseStart { option_id });
            0
        }

        fn option_end(option_id: OptionIdOf<T>, option: &mut OptionToken<T>) -> Weight {
            option.status = Status::End;
            Self::deposit_event(Event::OptionEnd { option_id });
            0
        }
    }
}
