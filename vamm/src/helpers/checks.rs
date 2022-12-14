use crate::{
    types::ClosingState::{Closed, Closing, Open},
    Config, Error, Pallet, SwapConfigOf, VammStateOf,
};
use frame_support::pallet_prelude::*;
use sp_runtime::traits::{CheckedAdd, Zero};
use sp_std::cmp::Ordering::Less;
use traits::vamm::{AssetType, Direction, SwapOutput};

#[derive(Debug)]
pub enum SanityCheckUpdateTwap {
    Proceed,
    Abort,
}

impl<T: Config> Pallet<T> {
    /// Checks if the following properties hold before performing a swap:
    ///
    /// * Vamm is open.
    /// * There is a sufficient amount of assets in the reserves to give to the
    /// caller if the swap is a [`Remove`](Direction::Remove) operation.
    /// * The total amount of assets in the reserve will not overflow if the
    /// swap is a [`Add`](Direction::Add) operation.
    ///
    /// # Errors
    ///
    /// * [`Error::<T>::VammIsClosed`]
    /// * [`Error::<T>::InsufficientFundsForTrade`]
    /// * [`Error::<T>::TradeExtrapolatesMaximumSupportedAmount`]
    pub fn sanity_check_before_swap(
        config: &SwapConfigOf<T>,
        vamm_state: &VammStateOf<T>,
    ) -> Result<(), DispatchError> {
        // We must ensure that the vamm is not closed before performing any swap.
        ensure!(
            !Self::is_vamm_closed(vamm_state, &None),
            Error::<T>::VammIsClosed
        );

        match config.direction {
            // If we intend to remove some asset amount from vamm, we must
            // have sufficient funds for it.
            Direction::Remove => match config.asset {
                AssetType::Base => ensure!(
                    config.input_amount < vamm_state.base_asset_reserves,
                    Error::<T>::InsufficientFundsForTrade
                ),
                AssetType::Quote => ensure!(
                    config.input_amount < vamm_state.quote_asset_reserves,
                    Error::<T>::InsufficientFundsForTrade
                ),
            },

            // If we intend to add some asset amount to the vamm, the
            // final amount must not overflow.
            Direction::Add => match config.asset {
                AssetType::Base => ensure!(
                    config
                        .input_amount
                        .checked_add(&vamm_state.base_asset_reserves)
                        .is_some(),
                    Error::<T>::TradeExtrapolatesMaximumSupportedAmount
                ),
                AssetType::Quote => ensure!(
                    config
                        .input_amount
                        .checked_add(&vamm_state.quote_asset_reserves)
                        .is_some(),
                    Error::<T>::TradeExtrapolatesMaximumSupportedAmount
                ),
            },
        };

        Ok(())
    }

    /// Checks if the following properties hold after performing a swap:
    ///
    /// * Swapped amount respects the limit specified in
    /// [`SwapConfig::output_amount_limit`](
    /// ../../traits/vamm/struct.SwapConfig.html#structfield.output_amount_limit).
    /// * Base assets was not completely drained.
    /// * Quote assets was not completely drained.
    ///
    /// # Errors
    /// * [`Error::<T>::SwappedAmountLessThanMinimumLimit`]
    /// * [`Error::<T>::SwappedAmountMoreThanMaximumLimit`]
    /// * [`Error::<T>::BaseAssetReservesWouldBeCompletelyDrained`]
    /// * [`Error::<T>::QuoteAssetReservesWouldBeCompletelyDrained`]
    pub fn sanity_check_after_swap(
        vamm_state: &VammStateOf<T>,
        config: &SwapConfigOf<T>,
        amount_swapped: &SwapOutput<T::Balance>,
    ) -> Result<(), DispatchError> {
        // Ensure swapped amount is valid.
        if let Some(limit) = config.output_amount_limit {
            match config.direction {
                Direction::Add => ensure!(
                    amount_swapped.output >= limit,
                    Error::<T>::SwappedAmountLessThanMinimumLimit
                ),
                Direction::Remove => ensure!(
                    amount_swapped.output <= limit,
                    Error::<T>::SwappedAmountMoreThanMaximumLimit
                ),
            }
        }

        // Ensure both quote and base assets weren't completely drained from vamm.
        ensure!(
            !vamm_state.base_asset_reserves.is_zero(),
            Error::<T>::BaseAssetReservesWouldBeCompletelyDrained
        );
        ensure!(
            !vamm_state.quote_asset_reserves.is_zero(),
            Error::<T>::QuoteAssetReservesWouldBeCompletelyDrained
        );

        // TODO(Cardosaum): Write one more `ensure!` block regarding
        // amount_swapped negative or positive?

        Ok(())
    }

    /// Checks if the following properties hold before updating twap:
    ///
    /// * Vamm is open.
    /// * New twap value is not zero.
    /// * Current time is greater than or equal to the last twap timestamp.
    ///
    /// # Errors
    ///
    /// * [`Error::<T>::NewTwapValueIsZero`]
    /// * [`Error::<T>::VammIsClosed`]
    /// * [`Error::<T>::AssetTwapTimestampIsMoreRecent`]
    pub fn sanity_check_before_update_twap(
        vamm_state: &VammStateOf<T>,
        current_price: T::Decimal,
        now: &Option<T::Moment>,
        try_update: bool,
    ) -> Result<SanityCheckUpdateTwap, DispatchError> {
        // New desired twap value can't be zero.
        ensure!(!current_price.is_zero(), Error::<T>::NewTwapValueIsZero);

        // Vamm must be open.
        ensure!(
            !Self::is_vamm_closed(vamm_state, now),
            Error::<T>::VammIsClosed
        );

        match Self::now(now).cmp(&vamm_state.base_asset_twap.get_timestamp()) {
            Less => {
                if try_update {
                    // Abort runtime storage update operation.
                    Ok(SanityCheckUpdateTwap::Abort)
                } else {
                    // We need to throw an error warning caller that one
                    // property of the swap operation was violated.
                    Err(Error::<T>::AssetTwapTimestampIsMoreRecent.into())
                }
            },
            _ => Ok(SanityCheckUpdateTwap::Proceed),
        }
    }

    /// Checks if the following properties hold before closing a vamm:
    ///
    /// * Vamm must be open without a scheduled time to close in the future.
    /// * The target closing time must be in the future.
    ///
    /// # Errors
    ///
    /// * [`Error::<T>::VammIsClosed`]
    /// * [`Error::<T>::VammIsClosing`]
    /// * [`Error::<T>::ClosingDateIsInThePast`]
    pub fn sanity_check_before_close(
        vamm_state: &VammStateOf<T>,
        closing_time: &T::Moment,
    ) -> Result<(), DispatchError> {
        // Vamm must be open
        let now = Self::now(&None);
        match vamm_state.closing_state(&now) {
            Closed => Err(Error::<T>::VammIsClosed),
            Closing => Err(Error::<T>::VammIsClosing),
            Open => Ok(()),
        }?;

        // Target closing time must be in the future
        ensure!(closing_time.gt(&now), Error::<T>::ClosingDateIsInThePast);

        Ok(())
    }
}
