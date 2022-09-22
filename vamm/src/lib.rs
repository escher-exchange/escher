//! # VAMM Pallet
//!
//! The VAMM Pallet provides functionality to manage virtual automated market makers.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The VAMM Pallet allows other Pallets to leverage it's functions in order to
//! manage virtual automated market makers, abstracting away complexity. It's
//! important to note that currently just one type of constant function market
//! maker is supported, namely the `x * y = k`.
//!
//! Below is a diagram showing how the trait and runtime storage looks like and
//! interact with each other:
//!
//! ![](https://www.plantuml.com/plantuml/svg/ZLJDZjCm4BxdAKnFYzJk0qGXscMvS408kk8Q3SbiQX7_u9cqGgWyEx4JR6zJfFgGsZFV_7_J1s9mFAgXUCC7L2WKE2eA2-qFw55iB0m3yku8Ict4xq9CHsf6zm8jYc-JL5GLEv1Srrwzd3-YTGYCTwtHBx8l0_8ftD_ceC4GtddVC-9ZjnMd0-fIF4k5nA1i3k-H6-jaEviqiajMG8HSYaV_y_pBugKPdy2-2fG3Q5B6JFVJOvsfCaVCOgV0tu6m2T4RK6RKN8htC81kSIj-ZeR_erpvPdFLFOEBLLyOdyEt0mQVWzY4OUpPEEXnayr2WGtkQ9hKelu4DX-NFqj4yQwEqdEyjGCG1SIUWN5oHEp6bbTEbWJphZWaT4UagpZVePk05lj6ZGDBEqXqho2VBKkZgyYOUgPLbzSHlkT8wwLPJoEnKSBpXNp7Kgc9hgjQRwZpXXflgEzSf8GIAzS9vTDRzYAAupxC2x8AAxKT5sucvGVfiFKz5Ts_syhGZ9micq4goNdIg4UL1QygBxZe865yVF4jMjcdF2xi7xjk6ovVqUzE6cyHnhhhp4dlweNqfJWvoLZCh_jx9_i3rncPIxyXL3oWxlpVu5y0)
//!
//! ### Terminology
//!
//! * **VAMM:** Acronym for Virtual Automated Market Maker.
//! * **CFMM:** Acronym for Constant Function Market Maker.
//! * **TWAP:** Acronym for Time Weighted Average Price.
//!
//! ### Goals
//!
//! ### Actors
//!
//! ### Implementations
//!
//! The VAMM Pallet provides implementations for the following traits:
//!
//! - [`Vamm`](traits::vamm::Vamm): Exposes functionality for
//! creating, managing and closing virtual automated market makers.
//!
//! ## Interface
//!
//! ### Extrinsics
//!
//! The current implementation doesn't deal with external calls to the pallet,
//! so there is no extrinsic defined.
//!
//! ### Public Functions
//!
//! * [`create`](pallet/struct.Pallet.html#method.create): Creates a new vamm,
//! returning it's Id.
//! * [`get_price`](pallet/struct.Pallet.html#method.get_price): Gets the
//! current price of the
//! [`base`](types/struct.VammState.html#structfield.base_asset_reserves) or
//! [`quote`](types/struct.VammState.html#structfield.quote_asset_reserves)
//! asset in a vamm.
//! * [`get_twap`](pallet/struct.Pallet.html#method.get_twap): Gets the time
//! weighted average price of the desired asset.
//! * [`move_price`](pallet/struct.Pallet.html#method.move_price): Changes
//! amount of
//! [`base`](types/struct.VammState.html#structfield.base_asset_reserves) and
//! [`quote`](types/struct.VammState.html#structfield.quote_asset_reserves)
//! assets in reserve, essentially changing the invariant.
//! * [`swap`](pallet/struct.Pallet.html#method.swap): Performs the swap of the
//! desired asset against the vamm.
//! * [`swap_simulation`](pallet/struct.Pallet.html#method.swap_simulation):
//! Performs the *simulation* of the swap operation for the desired asset
//! against the vamm, returning the expected amount such a trade would result if
//! the swap were in fact executed.
//! * [`update_twap`](pallet/struct.Pallet.html#method.update_twap): Updates the
//! time weighted average price of the desired asset.
//! * [`close`](pallet/struct.Pallet.html#method.close): Schedules a closing
//! date for the desired vamm, after which the vamm will be considered closed
//! and all operations in it will be halted.
//!
//! ### Runtime Storage Objects
//!
//! - [`VammCounter`](VammCounter): The number of created vamms.
//! - [`VammMap`](VammMap): Mapping of a [`VammId`](Config::VammId) to it's
//! corresponding [`VammState`](types/struct.VammState.html#).
//!
//! ## Usage
//!
//! ### Example
//!
//! ## Related Modules
//!
//! - [`Clearing House Pallet`](../clearing_house/index.html)
//!
//! <!-- Original author: @Cardosaum -->

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// Allow some linters for tests.
#![cfg_attr(
    not(test),
    warn(
        clippy::all,
        clippy::cargo,
        clippy::complexity,
        clippy::correctness,
        clippy::disallowed_methods,
        clippy::disallowed_types,
        clippy::doc_markdown,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::pedantic,
        clippy::perf,
        clippy::style,
        clippy::suspicious,
        clippy::todo,
        clippy::unwrap_used,
        missing_docs,
        rustdoc::missing_crate_level_docs,
        // TODO(Cardosaum): Write examples to all sections required
        // rustdoc::missing_doc_code_examples,
        warnings,
    )
)]
// Specify linters to VAMM Pallet.
#![warn(clippy::unseparated_literal_suffix)]
#![deny(
    bad_style,
    bare_trait_objects,
    const_err,
    dead_code,
    improper_ctypes,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_parens,
    while_true,
    rustdoc::broken_intra_doc_links
)]
// TODO(Cardosaum): Assess if it's possible to remove some of these allowed
// linters in the future.
#![allow(
    clippy::wildcard_imports,
    clippy::used_underscore_binding,
    clippy::cargo_common_metadata,
    clippy::default_trait_access,
    clippy::must_use_candidate,
    clippy::missing_doc_code_examples
)]

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

/// Specific types useful for the Vamm Pallet, such as
/// [`VammState`](types/struct.VammState.html).
pub mod types;

/// Helper functions and types for low-level functionalities of the Vamm Pallet.
pub mod helpers;

pub use pallet::*;

#[allow(clippy::too_many_lines, clippy::let_underscore_drop)]
#[frame_support::pallet]
pub mod pallet {
    // ----------------------------------------------------------------------------------------------------
    //                                       Imports and Dependencies
    // ----------------------------------------------------------------------------------------------------

    use crate::types::VammState;
    use codec::{Codec, FullCodec};
    use frame_support::{
        pallet_prelude::*, sp_std::fmt::Debug, traits::UnixTime, transactional, Blake2_128Concat,
    };
    use helpers::{
        numbers::{FixedPointMath, TryReciprocal, UnsignedMath},
        twap::Twap,
    };
    use num_integer::Integer;
    use sp_arithmetic::traits::Unsigned;
    use sp_core::U256;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero},
        ArithmeticError, FixedPointNumber, FixedU128,
    };
    use traits::vamm::{
        AssetType, Direction, MovePriceConfig, SwapConfig, SwapOutput, Vamm, VammConfig,
        MINIMUM_TWAP_PERIOD,
    };

    // ----------------------------------------------------------------------------------------------------
    //                                    Declaration Of The Pallet Type
    // ----------------------------------------------------------------------------------------------------

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // ----------------------------------------------------------------------------------------------------
    //                                             Config Trait
    // ----------------------------------------------------------------------------------------------------

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event type emitted by this pallet. Depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Ids used by the pallet to index each virtual automated market maker created.
        type VammId: Default
            + CheckedAdd
            + Clone
            + Copy
            + Debug
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + One
            + Parameter
            + PartialEq
            + TypeInfo
            + Unsigned
            + Zero;

        /// The Balance type used by the pallet for bookkeeping.
        type Balance: Default
            + AtLeast32BitUnsigned
            + CheckedAdd
            + CheckedDiv
            + CheckedMul
            + CheckedSub
            + Codec
            + Copy
            + From<u64>
            + From<u128>
            + Into<u128>
            + From<Self::Moment>
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + Ord
            + Parameter
            + Unsigned
            + Zero;

        /// Signed decimal fixed point number.
        type Decimal: Default
            + FixedPointNumber<Inner = Self::Balance>
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + One
            + TryReciprocal
            + TypeInfo
            + Into<FixedU128>
            + From<FixedU128>
            + Zero;

        /// The Integer type used by the pallet for computing swaps.
        type Integer: Integer;

        /// Type representing the current time.
        type Moment: Default
            + AtLeast32BitUnsigned
            + Clone
            + Codec
            + Copy
            + Debug
            + From<u64>
            + Into<u64>
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        /// Implementation for querying the current Unix timestamp.
        type TimeProvider: UnixTime;
    }

    // ----------------------------------------------------------------------------------------------------
    //                                             Pallet Types
    // ----------------------------------------------------------------------------------------------------

    /// Type alias for the [`SwapOutput`] value of the Vamm Pallet.
    pub type SwapOutputOf<T> = SwapOutput<<T as Config>::Balance>;

    /// Type alias for the [`SwapConfig`] value of the Vamm Pallet.
    pub type SwapConfigOf<T> = SwapConfig<<T as Config>::VammId, <T as Config>::Balance>;

    /// Type alias for the [`Twap`] value of the Vamm Pallet.
    pub type TwapOf<T> = Twap<<T as Config>::Decimal, <T as Config>::Moment>;

    /// Type alias for the [`VammState`] value of the Vamm Pallet.
    pub type VammStateOf<T> = VammState<<T as Config>::Balance, <T as Config>::Moment, TwapOf<T>>;

    // ----------------------------------------------------------------------------------------------------
    //                                           Runtime  Storage
    // ----------------------------------------------------------------------------------------------------

    /// The number of created vamms, also used to generate the next market
    /// identifier.
    ///
    /// # Note
    ///
    /// Frozen markets do not decrement the counter.
    #[pallet::storage]
    #[pallet::getter(fn vamm_count)]
    #[allow(clippy::disallowed_types)]
    pub type VammCounter<T: Config> = StorageValue<_, T::VammId, ValueQuery>;

    /// Maps [VammId](Config::VammId) to the corresponding virtual
    /// [VammState] specs.
    #[pallet::storage]
    #[pallet::getter(fn get_vamm)]
    pub type VammMap<T: Config> = StorageMap<_, Blake2_128Concat, T::VammId, VammStateOf<T>>;

    // ----------------------------------------------------------------------------------------------------
    //                                            Runtime Events
    // ----------------------------------------------------------------------------------------------------

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emitted after a successful call to the [`create`](Pallet::create)
        /// function.
        Created {
            /// The identifier for the Vamm where the operation took place.
            vamm_id: T::VammId,
            /// The updated state for the Vamm.
            state: VammStateOf<T>,
        },
        /// Emitted after a successful call to the [`swap`](Pallet::swap)
        /// function.
        Swapped {
            /// The identifier for the Vamm where the operation took place.
            vamm_id: T::VammId,
            /// The asset amount the caller used (or requested, depending on
            /// `direction`) to make this swap operation happen.
            input_amount: T::Balance,
            /// The asset amount the caller received (or paid, depending on
            /// `direction`) after the swap operation happen.
            output_amount: SwapOutputOf<T>,
            /// The [`asset type `](AssetType) the caller used (or requested,
            /// depending on `direction`) for this operation.
            input_asset_type: AssetType,
            /// The [`direction`](Direction) the caller wanted for this
            /// operation to happen, either adding or removing asset from the
            /// Vamm.
            direction: Direction,
        },
        /// Emitted after a successful call to the
        /// [`move_price`](Pallet::move_price) function.
        PriceMoved {
            /// The identifier for the Vamm where the operation took place.
            vamm_id: T::VammId,
            /// The new value for the amount of [`base assets in
            /// reserve`](VammState::base_asset_reserves) for the specified
            /// Vamm.
            base_asset_reserves: T::Balance,
            /// The new value for the amount of [`quote assets in
            /// reserve`](VammState::quote_asset_reserves) for the specified
            /// Vamm.
            quote_asset_reserves: T::Balance,
            /// The new invariant (aka. the constant `K`) for the specified
            /// Vamm, obtained by multiplying the amount of
            /// [`base`](VammState::base_asset_reserves) and
            /// [`quote`](VammState::quote_asset_reserves) reserves present in
            /// the Vamm.
            invariant: U256,
        },
        /// Emitted after a successful call to the
        /// [`update_twap`](Pallet::update_twap) function.
        UpdatedTwap {
            /// The identifier for the Vamm where the operation took place.
            vamm_id: T::VammId,
            /// The new time weighted average price for the [`base
            /// asset`](VammState::base_asset_reserves) in the specified Vamm.
            base_twap: T::Decimal,
        },
        /// Emitted after a successful call to the [`close`](Pallet::close)
        /// function.
        Closed {
            /// The identifier for the Vamm where the operation took place.
            vamm_id: T::VammId,
            /// The timestamp where the closing process will take place. After
            /// reaching the specified time the vamm will be considered *closed*.
            closing_time: T::Moment,
        },
    }

    // ----------------------------------------------------------------------------------------------------
    //                                           Runtime  Errors
    // ----------------------------------------------------------------------------------------------------

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Tried to set [`base_asset_reserves`](VammState::base_asset_reserves)
        /// to zero.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::create`]
        /// * [`Pallet::move_price`]
        /// * [`Pallet::compute_invariant`]
        BaseAssetReserveIsZero,
        /// Tried to set
        /// [`quote_asset_reserves`](VammState::quote_asset_reserves) to zero.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::create`]
        /// * [`Pallet::move_price`]
        /// * [`Pallet::compute_invariant`]
        QuoteAssetReserveIsZero,
        /// Computed Invariant is zero.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::create`]
        /// * [`Pallet::move_price`]
        /// * [`Pallet::compute_invariant`]
        InvariantIsZero,
        /// Tried to set [`peg_multiplier`](VammState) to zero.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::create`]
        PegMultiplierIsZero,
        /// Tried to access an invalid [`VammId`](Config::VammId).
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::get_price`]
        /// * [`Pallet::get_twap`]
        /// * [`Pallet::update_twap`]
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::move_price`]
        /// * [`Pallet::close`]
        /// * [`Pallet::get_vamm_state`]
        VammDoesNotExist,
        /// Tried to execute a trade but the Vamm didn't have enough funds to
        /// fulfill it.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_before_swap`]
        InsufficientFundsForTrade,
        /// Tried to add some amount of asset to Vamm but it would exceeds the
        /// supported maximum value.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_before_swap`]
        TradeExtrapolatesMaximumSupportedAmount,
        /// Tried to perform operation against a closed Vamm.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::get_price`]
        /// * [`Pallet::get_twap`]
        /// * [`Pallet::update_twap`]
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::move_price`]
        /// * [`Pallet::close`]
        /// * [`Pallet::sanity_check_before_swap`]
        /// * [`Pallet::sanity_check_before_update_twap`]
        /// * [`Pallet::sanity_check_before_close`]
        VammIsClosed,
        /// Tried to perform operation against a closing Vamm, but this specific
        /// operation is not allowed.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::close`]
        /// * [`Pallet::sanity_check_before_close`]
        VammIsClosing,
        /// Tried to perform an operation against an open/closing Vamm, whereas it should be
        /// closed.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::get_settlement_price`]
        VammIsNotClosed,
        /// Tried to swap assets but the amount returned was less than the minimum expected.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_after_swap`]
        SwappedAmountLessThanMinimumLimit,
        /// Tried to swap assets but the amount returned was more than the maximum expected.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_after_swap`]
        SwappedAmountMoreThanMaximumLimit,
        /// Tried to perform swap operation but it would drain all
        /// [`base`](VammState::base_asset_reserves) asset reserves.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_after_swap`]
        BaseAssetReservesWouldBeCompletelyDrained,
        /// Tried to perform swap operation but it would drain all
        /// [`quote`](VammState::quote_asset_reserves) asset reserves.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::swap`]
        /// * [`Pallet::swap_simulation`]
        /// * [`Pallet::do_swap`]
        /// * [`Pallet::compute_swap`]
        /// * [`Pallet::sanity_check_after_swap`]
        QuoteAssetReservesWouldBeCompletelyDrained,
        /// Tried to update twap for an asset, but its last twap update was
        /// more recent than the current time.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::update_twap`]
        /// * [`Pallet::do_update_twap`]
        /// * [`Pallet::sanity_check_before_update_twap`]
        AssetTwapTimestampIsMoreRecent,
        /// Tried to update twap for an asset, but the desired new twap value is
        /// zero.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::update_twap`]
        /// * [`Pallet::do_update_twap`]
        /// * [`Pallet::sanity_check_before_update_twap`]
        NewTwapValueIsZero,
        /// Tried to update twap value, but a function call responsible for
        /// returning a new twap value didn't do so. As the called function
        /// should return a value always, not doing so must be an error.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::update_twap`]
        /// * [`Pallet::do_update_twap`]
        InternalUpdateTwapDidNotReturnValue,
        /// Tried to create a vamm with a
        /// [`twap_period`](VammState::base_asset_twap) smaller than the
        /// minimum allowed value specified by
        /// [`MINIMUM_TWAP_PERIOD`](traits::vamm::MINIMUM_TWAP_PERIOD).
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::create`]
        FundingPeriodTooSmall,
        /// Tried to close a vamm with a timestamp that is in the past. To close
        /// a vamm successfully it's required to specify a time in the *future*.
        ///
        /// ## Occurrences
        ///
        /// * [`Pallet::close`]
        /// * [`Pallet::sanity_check_before_close`]
        ClosingDateIsInThePast,
    }

    // ----------------------------------------------------------------------------------------------------
    //                                                Hooks
    // ----------------------------------------------------------------------------------------------------

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    // ----------------------------------------------------------------------------------------------------
    //                                         Genesis Configuration
    // ----------------------------------------------------------------------------------------------------

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// The number of Vamms with which the initial storage state will start.
        pub vamm_count: T::VammId,
        /// The [`VammState`] of each Vamm with which the initial storage state will start.
        pub vamms: Vec<(T::VammId, VammStateOf<T>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                vamm_count: Default::default(),
                vamms: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            VammCounter::<T>::put(self.vamm_count);
            self.vamms.iter().for_each(|(vamm_id, vamm_state)| {
                VammMap::<T>::insert(vamm_id, vamm_state);
            });
        }
    }

    // ----------------------------------------------------------------------------------------------------
    //                                           Vamm Trait
    // ----------------------------------------------------------------------------------------------------

    impl<T: Config> Vamm for Pallet<T> {
        type Balance = T::Balance;
        type Decimal = T::Decimal;
        type Moment = T::Moment;
        type MovePriceConfig = MovePriceConfig<T::VammId, T::Balance>;
        type SwapConfig = SwapConfigOf<T>;
        type VammConfig = VammConfig<T::Balance, T::Moment>;
        type VammId = T::VammId;

        /// Creates a new virtual automated market maker.
        ///
        /// # Overview
        /// In order for the caller to create new vamms, it has to request it to
        /// the Vamm Pallet, which is responsible to keep track of and update
        /// when requested all active virtual automated market makers. The Vamm
        /// Pallet creates a new vamm, inserts it into storage, deposits a
        /// [`Created`](Event::<T>::Created) event on the blockchain and returns
        /// the new [`VammId`](Config::VammId) to the caller.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/NP2nJiCm48PtFyNH1L2L5yXGbROB0on8x5VdLsibjiFT9NbzQaE4odRIz-dp-VPgB3R5mMaVqiZ2aGGwvgHuWofVSC2GbnUHl93916V11j0dnqXUm1PoSeyyMMPlOMO3vUGUx8e8YYpgtCXYmOUHaz7cE0Gasn0h-JhUuzAjSBuDhcFZCojeys5P-09wAi9pDVIVSXYox_sLGwhux9txUO6QNSrjjoqToyfriHv6Wgy9QgxGOjNalRJ2PfTloPPE6BC68r-TRYrXHlfJVx_MD2szOrcTrvFR8tNbsjy0)
        ///
        /// ## Parameters:
        /// - `base_asset_reserves`: The amount of
        /// [`base`](VammState::base_asset_reserves) asset.
        /// - `quote_asset_reserves`: The amount of
        /// [`quote`](VammState::quote_asset_reserves) asset.
        /// - `peg_multiplier`: The constant multiplier responsible to balance
        /// [`quote`](VammState::quote_asset_reserves) and
        /// [`base`](VammState::base_asset_reserves)
        /// asset.
        ///
        /// ## Returns
        /// The new vamm's id, if successful.
        ///
        /// ## Assumptions or Requirements
        /// In order to create a valid vamm, we need to ensure that both
        /// [`base`](VammState::base_asset_reserves) and
        /// [`quote`](VammState::quote_asset_reserves) asset reserves, as well
        /// as the peg_multiplier, are non-zero. Every parameter must be greater
        /// than zero.
        ///
        /// ## Emits
        /// * [`Created`](Event::<T>::Created)
        ///
        /// ## State Changes
        /// Updates [`VammMap`] storage map and [`VammCounter`] storage value.
        ///
        /// ## Errors
        /// * [`BaseAssetReserveIsZero`](Error::<T>::BaseAssetReserveIsZero)
        /// * [`QuoteAssetReserveIsZero`](Error::<T>::QuoteAssetReserveIsZero)
        /// * [`InvariantIsZero`](Error::<T>::InvariantIsZero)
        /// * [`PegMultiplierIsZero`](Error::<T>::PegMultiplierIsZero)
        /// * [`FundingPeriodTooSmall`](Error::<T>::FundingPeriodTooSmall)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn create(config: &Self::VammConfig) -> Result<T::VammId, DispatchError> {
            ensure!(
                !config.peg_multiplier.is_zero(),
                Error::<T>::PegMultiplierIsZero
            );
            ensure!(
                config.twap_period >= MINIMUM_TWAP_PERIOD.into(),
                Error::<T>::FundingPeriodTooSmall
            );

            let invariant =
                Self::compute_invariant(config.base_asset_reserves, config.quote_asset_reserves)?;
            let now = Self::now(&None);
            let tmp_vamm_state = VammStateOf::<T> {
                base_asset_reserves: config.base_asset_reserves,
                quote_asset_reserves: config.quote_asset_reserves,
                peg_multiplier: config.peg_multiplier,
                ..Default::default()
            };

            VammCounter::<T>::try_mutate(|next_id| {
                let id = *next_id;
                let vamm_state = VammStateOf::<T> {
                    base_asset_reserves: config.base_asset_reserves,
                    quote_asset_reserves: config.quote_asset_reserves,
                    base_asset_twap: Twap::new(
                        Self::do_get_price(&tmp_vamm_state, AssetType::Base)?,
                        now,
                        config.twap_period,
                    ),
                    terminal_base_asset_reserves: config.base_asset_reserves,
                    terminal_quote_asset_reserves: config.quote_asset_reserves,
                    peg_multiplier: config.peg_multiplier,
                    invariant,
                    closed: None,
                };

                VammMap::<T>::insert(&id, vamm_state);
                *next_id = id
                    .checked_add(&One::one())
                    .ok_or(ArithmeticError::Overflow)?;

                Self::deposit_event(Event::<T>::Created {
                    vamm_id: id,
                    state: vamm_state,
                });

                Ok(id)
            })
        }

        /// Gets the current price of the
        /// [`base`](VammState::base_asset_reserves) or
        /// [`quote`](VammState::quote_asset_reserves) asset in a vamm.
        ///
        /// # Overview
        /// In order for the caller to know what the current price of an asset
        /// in a specific vamm is, it has to request it to the Vamm Pallet. The
        /// Vamm Pallet consults the runtime storage for the desired vamm,
        /// computes the current price and returns it to the caller.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/PP0zJWCn44PxdsBO1b2q5qY14b9GKI7H3vkFOB7-OURRvFfWhm0XEillpHlBEwSQbpG7Vu-vgcaIWzUI7OzmrnFkCPVBtgnSXBOWC7A6F82Yxg1KYnFajPYeF6jAuLeN5fqOpqf8oU6ARqYGfEOXL3N6ALRDbE4mHsGEeYvJF_x5BTVXkNMFIdrHXmnFBAOdo4qJRhlXNGbhHSQxFhBPRFyzrF2nm1aQRruVNBL-vLJYXwxmK59TY5xuPbzmNJQEMzd_BWWxv6Fxq4y0)
        ///
        /// ## Parameters
        ///  - `vamm_id`: The ID of the desired vamm to query.
        ///  - `asset_type`: The desired asset type to get info about. (either
        ///  [`base`](VammState::base_asset_reserves) or
        ///  [`quote`](VammState::quote_asset_reserves)).
        ///
        /// ## Returns
        /// The price of [`base`](VammState::base_asset_reserves) asset in
        /// relation to [`quote`](VammState::quote_asset_reserves) (or
        /// vice-versa).
        ///
        /// ## Assumptions or Requirements
        /// In order to consult the current price for an asset, we need to
        /// ensure that the desired vamm_id exists.
        ///
        /// ## Emits
        /// No event is emitted for this function.
        ///
        /// ## State Changes
        /// This function does not mutate runtime storage.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn get_price(
            vamm_id: T::VammId,
            asset_type: AssetType,
        ) -> Result<T::Decimal, DispatchError> {
            // Get Vamm state.
            let vamm_state = Self::get_vamm_state(&vamm_id)?;

            // Vamm must be open
            ensure!(
                !Self::is_vamm_closed(&vamm_state, &None),
                Error::<T>::VammIsClosed
            );

            Self::do_get_price(&vamm_state, asset_type)
        }

        /// Returns the time weighted average price of the desired asset.
        ///
        /// # Overview
        /// In order for the caller to know which is the time weighted average
        /// price of the desired asset, it has to request it to the Vamm Pallet.
        /// The pallet will query the runtime storage and return the desired
        /// twap.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/FSqz3i8m343XdLF01UgTgH8IrwXSrsqYnKxa7tfzAWQcfszwimTQfBJReogrt3YjtKl4y2U0uJaTDKgkwMpKDLXZeYxmwZAwuzhuNO7-07OgRB0R2iC7HM2hU5nos5CfQjVbu5ZYn36DXlfxpwpRrIy0)
        ///
        /// ## Parameters
        ///  - [`vamm_id`](Config::VammId): The ID of the desired vamm to query.
        ///  - [`asset_type`](traits::vamm::AssetType): The desired
        ///  asset type to get info about.
        ///
        /// ## Returns
        /// The twap for the specified asset.
        ///
        /// ## Assumptions or Requirements
        /// * The requested [`VammId`](Config::VammId) must exist.
        /// * The requested Vamm must be open.
        ///
        /// For more information about how to know if a Vamm is open or not,
        /// please have a look in the variable [`closed`](VammState::closed).
        ///
        /// ## Emits
        /// No event is emitted for this function.
        ///
        /// ## State Changes
        /// This function does not mutate runtime storage.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn get_twap(
            vamm_id: T::VammId,
            asset_type: AssetType,
        ) -> Result<T::Decimal, DispatchError> {
            // Sanity Checks
            // 1) Vamm must exist
            let vamm_state = Self::get_vamm_state(&vamm_id)?;

            // 2) Vamm must be open
            ensure!(
                !Self::is_vamm_closed(&vamm_state, &None),
                Error::<T>::VammIsClosed
            );

            match asset_type {
                AssetType::Base => Ok(vamm_state.base_asset_twap.get_twap()),
                AssetType::Quote => Ok(vamm_state.base_asset_twap.get_twap().try_reciprocal()?),
            }
        }

        /// Updates the time weighted average price of the [base
        /// asset](VammState::base_asset_twap).
        ///
        /// # Overview
        /// In order for the caller to update the time weighted average price of
        /// the base asset, it has to send the request to the Vamm Pallet. The
        /// pallet will perform the needed sanity checks and update the runtime
        /// storage with the desired twap value, returning it in case of
        /// success.
        ///
        /// This function can also compute the new twap value using an
        /// Exponential Moving Average algorithm rather than blindly seting it
        /// to the value passed by the caller. In that case, the following
        /// algorithm will be used:
        ///
        /// $$
        /// twap_t = \frac{(x_t \cdot w_t) + (twap_{t-1} \cdot w_{t-1})}{w_t + w_{t-1}}
        /// $$
        ///
        /// Where:
        /// * $x_t$: Is the current price of the asset.
        /// * $twap_t$: Is the new calculated twap.
        /// * $twap_{t-1}$: Is the last twap of the asset.
        /// * $w_t$: $max(1, T_{now} - T_{last\\_update})$.
        /// * $w_{t-1}$: $max(1, $[`twap_period`](VammState::base_asset_twap)$ - w_t)$.
        /// * $T_{now}$: current unix timestamp (ie. seconds since the Unix epoch).
        /// * $T_{last\\_update}$: timestamp from last twap update.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/FSqz3i8m343XdLF01UgTgH8IrwXSnsqZnKxa7tfzAWQcfszwimTQfBJReogrB9pMxaV4y2U0uJdjDOvSqzceQx36H5tWrMLqnxNnkmBz0UnqiC5cA0mV585ISR_aiALIrAvBZeB1Ivmufj5GV_kPjLpz0W00)
        ///
        /// ## Parameters
        ///  - [`vamm_id`](Config::VammId): The ID of the desired vamm to update.
        ///  - [`base_twap`](VammState::base_asset_twap): The optional desired
        ///  value for the base asset's twap.  If the value is `None`, than the
        ///  Vamm will update the twap using an exponential moving average
        ///  algorithm.
        ///
        /// ## Returns
        /// The new twap value for [`base_twap`](VammState::base_asset_twap).
        ///
        /// ## Assumptions or Requirements
        /// * The requested [`VammId`](Config::VammId) must exists.
        /// * The requested Vamm must be open.
        /// * The `base_twap` value can't be zero.
        ///
        /// For more information about how to know if a Vamm is open or not,
        /// please have a look in the variable [`closed`](VammState::closed).
        ///
        /// ## Emits
        /// * [`UpdatedTwap`](Event::<T>::UpdatedTwap)
        ///
        /// ## State Changes
        /// Updates [`VammMap`] storage map.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`NewTwapValueIsZero`](Error::<T>::NewTwapValueIsZero)
        /// * [`AssetTwapTimestampIsMoreRecent`](Error::<T>::AssetTwapTimestampIsMoreRecent)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn update_twap(
            vamm_id: T::VammId,
            base_twap: Option<T::Decimal>,
        ) -> Result<T::Decimal, DispatchError> {
            // Retrieve vamm state.
            let mut vamm_state = Self::get_vamm_state(&vamm_id)?;

            // Delegate update twap to internal functions.
            let output = Self::do_update_twap(vamm_id, &mut vamm_state, base_twap, &None)?;

            // Deposit updated twap event into blockchain.
            Self::deposit_event(Event::<T>::UpdatedTwap {
                vamm_id,
                base_twap: output,
            });

            Ok(output)
        }

        /// Performs the swap of the desired asset against the vamm.
        ///
        /// # Overview
        /// In order for the caller be able to swap assets in the vamm, it has
        /// to request it to the Vamm Pallet. The pallet will perform all needed
        /// checks to ensure the swap is a valid one and then, using the
        /// corresponding function it was configured to, will compute the amount
        /// of assets the caller will receive.
        ///
        /// In the current state the only function available to perform these
        /// computations is the CFMM `x * y = k`.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/FSq_giCm383n_PtYzGBHtYbGw3MA8YknmPAD_ZJNR-ZGwUCtVQi7MgJqlrjJwbauhV_NYEbt0CDpELhKtDBPQ6Ymna93u35a3iUjyxC1_G3iLDbWDnI6Duf0QNXSSjXJAThGbvyubzbHlz-LjLpz0000)
        ///
        /// ## Parameters
        ///  - `config`: Specification for swaps.
        ///
        /// ## Returns
        /// The amount of the other asset the caller will receive as a
        /// result of the swap.
        ///
        /// E.g. If the caller swaps [`quote`](VammState::quote_asset_reserves)
        /// asset, it will receive some amount of
        /// [`base`](VammState::base_asset_reserves) asset (and vice-versa).
        ///
        /// ## Assumptions or Requirements
        /// * The requested [`VammId`](Config::VammId) must exists.
        /// * The desired swap amount can not exceed the maximum supported value
        /// for the Vamm.
        /// * The desired swap amount must result in at least
        /// [`output_amount_limit`](traits::vamm::SwapConfig).
        ///
        /// ## Emits
        /// * [`Swapped`](Event::<T>::Swapped)
        ///
        /// ## State Changes
        /// Updates [`VammMap`] storage map.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`InsufficientFundsForTrade`](Error::<T>::InsufficientFundsForTrade)
        /// * [`TradeExtrapolatesMaximumSupportedAmount`](Error::<T>::TradeExtrapolatesMaximumSupportedAmount)
        /// * [`BaseAssetReservesWouldBeCompletelyDrained`](Error::<T>::BaseAssetReservesWouldBeCompletelyDrained)
        /// * [`QuoteAssetReservesWouldBeCompletelyDrained`](Error::<T>::QuoteAssetReservesWouldBeCompletelyDrained)
        /// * [`SwappedAmountLessThanMinimumLimit`](Error::<T>::SwappedAmountLessThanMinimumLimit)
        /// * [`SwappedAmountMoreThanMaximumLimit`](Error::<T>::SwappedAmountMoreThanMaximumLimit)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn swap(config: &SwapConfigOf<T>) -> Result<SwapOutputOf<T>, DispatchError> {
            // Get Vamm state.
            let mut vamm_state = Self::get_vamm_state(&config.vamm_id)?;

            // Tries to update twap before swapping assets.
            Self::try_update_twap(config.vamm_id, &mut vamm_state, None, &None)?;

            // Delegate swap to helper function.
            let amount_swapped = Self::do_swap(config, &mut vamm_state)?;

            // Return total swapped asset.
            Ok(amount_swapped)
        }

        /// Performs the *simulation* of the swap operation for the desired
        /// asset against the vamm, returning the expected amount such a trade
        /// would result if the swap were in fact executed.
        ///
        /// # Overview
        /// This function essentially does the same as [`swap`](Self::swap),
        /// except for the fact that the runtime storage is not mutated.
        ///
        /// ![](http://www.plantuml.com/plantuml/svg/FSuzZi90343XVa-nN23kgI8XSOt8cJYPaMpFo3_a-WGAggUlUxC7MgJmtwrfuTmeZVzhnF0xWE4v7IrghkbafMkGnbIwmAFBw8uhqxD1-G78IoM3tL08NYW2MyFZaiEUMg9rNVp4iNYJPFnu6epwNPX9jwjl)
        ///
        /// ## Parameters
        ///  - `config`: Specification for swaps.
        ///
        /// ## Returns
        /// The *expected* asset amount returned after a swap, taking into
        /// account slippage due to trade size.
        ///
        /// ## Assumptions or Requirements
        /// * The requested [`VammId`](Config::VammId) must exist.
        /// * The requested Vamm must be open.
        /// * The desired swap amount can not exceed the maximum supported value
        /// for the Vamm.
        ///
        /// ## Emits
        /// No event is emitted for this function.
        ///
        /// ## State Changes
        /// This function does not mutate runtime storage.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`InsufficientFundsForTrade`](Error::<T>::InsufficientFundsForTrade)
        /// * [`BaseAssetReservesWouldBeCompletelyDrained`](Error::<T>::BaseAssetReservesWouldBeCompletelyDrained)
        /// * [`QuoteAssetReservesWouldBeCompletelyDrained`](Error::<T>::QuoteAssetReservesWouldBeCompletelyDrained)
        /// * [`TradeExtrapolatesMaximumSupportedAmount`](Error::<T>::TradeExtrapolatesMaximumSupportedAmount)
        /// * [`SwappedAmountLessThanMinimumLimit`](Error::<T>::SwappedAmountLessThanMinimumLimit)
        /// * [`SwappedAmountMoreThanMaximumLimit`](Error::<T>::SwappedAmountMoreThanMaximumLimit)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn swap_simulation(config: &SwapConfigOf<T>) -> Result<SwapOutputOf<T>, DispatchError> {
            // Get Vamm state.
            let vamm_state = Self::get_vamm_state(&config.vamm_id)?;

            // Delegate swap to helper function.
            let swap = Self::compute_swap(config, &vamm_state)?;

            // Return swap result.
            Ok(swap.swap_output)
        }

        /// Moves the price of a vamm to the desired values of
        /// [`base`](VammState::base_asset_reserves) and
        /// [`quote`](VammState::quote_asset_reserves) asset reserves.
        ///
        /// # Overview
        /// In order for the caller to modify the
        /// [`base`](VammState::base_asset_reserves) and
        /// [`quote`](VammState::quote_asset_reserves) asset reserves,
        /// essentially modifying the invariant `k` of the function `x * y = k`,
        /// it has to request it to the Vamm Pallet. The pallet will perform the
        /// needed validity checks and, if everything succeeds, a
        /// [`PriceMoved`](Event::<T>::PriceMoved) event will be deposited on
        /// the blockchain warning the state change for the vamm and the asset
        /// reserves of the vamm and it's invariant will change accordingly.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/FSqz3i8m343XdLF01UgTgH8IrwXSrsqYnKxadt9zAWQcfszwimTQfBJReogrt3YjtKl4y2U0uMSwQfHSqzceQx36H5tWrMLqnxNnkmBz0UnKs60t58OJHM2hU5nos5CfQjT5-idBi4eyZORwky-iszKl)
        ///
        /// ## Parameters:
        /// * [`config`](traits::vamm::MovePriceConfig):
        /// Specification for moving the price of the vamm.
        ///
        /// ## Returns
        /// This function returns the calculated invariant `K` if successful.
        ///
        /// ## Assumptions or Requirements
        /// In order to move the price of a vamm we need to ensure that some properties hold:
        /// * The passed [`VammId`](Config::VammId) must be valid.
        /// * The desired vamm must be open. (See the [`closed`](VammState)
        /// field for more information).
        /// * Both [`base`](VammState::base_asset_reserves) and
        /// [`quote`](VammState::quote_asset_reserves) must be greater than
        /// zero.
        ///
        /// ## Emits
        /// * [`PriceMoved`](Event::<T>::PriceMoved)
        ///
        /// ## State Changes
        /// Updates:
        /// * [`VammMap`], modifying both
        /// [`base`](VammState::base_asset_reserves) and
        /// [`quote`](VammState::quote_asset_reserves) asset reserves as well as
        /// the invariant.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`BaseAssetReserveIsZero`](Error::<T>::BaseAssetReserveIsZero)
        /// * [`QuoteAssetReserveIsZero`](Error::<T>::QuoteAssetReserveIsZero)
        /// * [`InvariantIsZero`](Error::<T>::InvariantIsZero)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn move_price(config: &Self::MovePriceConfig) -> Result<U256, DispatchError> {
            // Get Vamm state.
            let mut vamm_state = Self::get_vamm_state(&config.vamm_id)?;

            // TODO(Cardosaum): Try to move from using function
            // Self::is_vamm_closed to Vamm.is_closed method
            ensure!(
                !Self::is_vamm_closed(&vamm_state, &None),
                Error::<T>::VammIsClosed
            );

            let invariant =
                Self::compute_invariant(config.base_asset_reserves, config.quote_asset_reserves)?;

            vamm_state.base_asset_reserves = config.base_asset_reserves;
            vamm_state.quote_asset_reserves = config.quote_asset_reserves;
            vamm_state.invariant = invariant;

            // Update runtime storage.
            VammMap::<T>::insert(&config.vamm_id, vamm_state);

            // Deposit price moved event into blockchain.
            Self::deposit_event(Event::<T>::PriceMoved {
                vamm_id: config.vamm_id,
                base_asset_reserves: config.base_asset_reserves,
                quote_asset_reserves: config.quote_asset_reserves,
                invariant,
            });

            // Return new invariant.
            Ok(invariant)
        }

        /// Computes the price to settle positions against after a vAMM has been closed.
        ///
        /// # Overview
        /// After trading against a vAMM is halted, users can only close their positions at a
        /// pre-computed settlement price. This function computes that settlement price after a vAMM
        /// is closed. It takes into account the net position of all traders left after vAMM
        /// closure. The settlement price is higher the more distant the reserves are from
        /// their terminal values. This price is the average execution price if a single
        /// trade took the vAMM from its terminal reserves to their current values. Traders
        /// who have a higher average execution price lose money and those who have a lower
        /// one, win. The settlement price is 0 if the vAMM ended up closing at terminal
        /// reserve values.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/BSqn3i8m34RXdLF00QXtfjwaCkvF4Ybs8yS6ZWz2J4zl-jOPx97QJvTcqdD7UZ_NY35lHCwlfRIeUSy9byC25eiSIfXIuLUyfR8L_9-Kcz6JLMblN9nrqYDDeXss5SGs4T6XiDY6Dy4oEjkFNs7xjny0)
        ///
        /// ## Parameters:
        /// * `vamm_id`: identifier for the closed vAMM
        ///
        /// ## Returns
        /// The settlement price as an unsigned decimal.
        ///
        /// ## Assumptions or Requirements
        /// * The vAMM with id `vamm_id` must exist
        /// * The vAMM with id `vamm_id` must be closed
        ///
        /// ## Emits
        /// No event is emitted for this function.
        ///
        /// ## State Changes
        /// No runtime storage item is updated by this function.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsNotClosed`](Error::<T>::VammIsNotClosed)
        /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
        ///
        /// # Runtime
        /// `O(1)`
        fn get_settlement_price(vamm_id: Self::VammId) -> Result<Self::Decimal, DispatchError> {
            let vamm_state = Self::get_vamm_state(&vamm_id)?;
            ensure!(
                Self::is_vamm_closed(&vamm_state, &None),
                Error::<T>::VammIsNotClosed
            );

            let abs_base_diff = Self::abs_balance_diff(
                vamm_state.base_asset_reserves,
                vamm_state.terminal_base_asset_reserves,
            );
            let abs_quote_diff = Self::abs_balance_diff(
                vamm_state
                    .peg_multiplier
                    .try_mul(&vamm_state.quote_asset_reserves)?,
                vamm_state
                    .peg_multiplier
                    .try_mul(&vamm_state.terminal_quote_asset_reserves)?,
            );

            if abs_base_diff.is_zero() {
                return Ok(Zero::zero())
            }

            let net_base_decimal = T::Decimal::from_inner(abs_base_diff);
            let net_quote_decimal = T::Decimal::from_inner(abs_quote_diff);
            Ok(net_quote_decimal.try_div(&net_base_decimal)?)
        }

        /// Schedules a closing date for the desired vamm, after which the vamm
        /// will be considered closed and all operations in it will be halted.
        ///
        /// # Overview
        /// In order for the caller to close a vamm it has to send the request
        /// to the Vamm Pallet, which will perform the necessary checks and, if
        /// all the requirements are satisfied, a closing date will be set to
        /// the specified vamm. The vamm will be considered closed when the
        /// current time reaches the specified time in this function call.
        ///
        /// ![](https://www.plantuml.com/plantuml/svg/BSsz3G8n343XdYbW0EAUwZP1nZ59fDWv-GSO7uIkUdhLjtcWHSeyNORIpCffyzmZThy16BvB6z7paSv6IuCr2Yq1TkfiL_vGHsryF0WEXHUAG1tO3CNXcKenbjvfBkUoJzI_jx7MNxy0)
        ///
        /// ## Parameters:
        /// * [`vamm_id`](Config::VammId): The ID of the desired vamm to close.
        /// * [`closing_time`](Config::Moment): The timestamp after which the
        /// vamm will be considered closed.
        ///
        /// ## Returns
        /// This function returns an empty `Ok(())` on success.
        ///
        /// ## Assumptions or Requirements
        /// In order to close a vamm we need to ensure that some properties hold:
        ///
        /// * The requested [`VammId`](Config::VammId) must exist.
        /// * The requested vamm must be open. (See the [`closed`](VammState)
        /// field for more information).
        /// * The requested [`closing_time`](Config::Moment) must be *strictly*
        /// in the future.
        ///
        /// ## Emits
        /// * [`Closed`](Event::<T>::Closed)
        ///
        /// ## State Changes
        /// Updates:
        ///
        /// * [`VammMap`], modifying the [`closed`](VammState::closed) field.
        ///
        /// ## Errors
        /// * [`VammDoesNotExist`](Error::<T>::VammDoesNotExist)
        /// * [`VammIsClosed`](Error::<T>::VammIsClosed)
        /// * [`VammIsClosing`](Error::<T>::VammIsClosing)
        /// * [`ClosingDateIsInThePast`](Error::<T>::ClosingDateIsInThePast)
        ///
        /// # Runtime
        /// `O(1)`
        #[transactional]
        fn close(vamm_id: T::VammId, closing_time: T::Moment) -> Result<(), DispatchError> {
            // Get Vamm state.
            let vamm_state = Self::get_vamm_state(&vamm_id)?;

            // Sanity checks.
            Self::sanity_check_before_close(&vamm_state, &closing_time)?;

            // Update runtime storage.
            VammMap::<T>::try_mutate(&vamm_id, |vamm| match vamm {
                Some(v) => {
                    v.closed = Some(closing_time);
                    Ok(())
                },
                None => Err(Error::<T>::VammDoesNotExist),
            })?;

            // Emit event.
            Self::deposit_event(Event::<T>::Closed {
                vamm_id,
                closing_time,
            });

            Ok(())
        }
    }
}
