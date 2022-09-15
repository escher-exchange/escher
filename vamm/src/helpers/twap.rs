use crate::{helpers::checks::SanityCheckUpdateTwap, Config, Error, Pallet, VammMap, VammStateOf};
use frame_support::{pallet_prelude::*, transactional};
use sp_runtime::traits::Saturating;
use traits::vamm::AssetType;

impl<T: Config> Pallet<T> {
    /// Performs runtime storage changes, effectively updating the asset twap.
    /// This `update_twap` variation can't accept any error to occur, expecting
    /// all properties described in
    /// [`update_twap`](struct.Pallet.html#method.update_twap) to hold.
    ///
    /// # Errors
    ///
    /// * All errors returned by
    /// [`sanity_check_before_update_twap`](
    /// struct.Pallet.html#method.sanity_check_before_update_twap).
    #[transactional]
    pub fn do_update_twap(
        vamm_id: T::VammId,
        vamm_state: &mut VammStateOf<T>,
        current_price: Option<T::Decimal>,
        now: &Option<T::Moment>,
    ) -> Result<T::Decimal, DispatchError> {
        match Self::internal_update_twap(vamm_id, vamm_state, current_price, now, false)? {
            Some(twap) => Ok(twap),
            None => Err(Error::<T>::InternalUpdateTwapDidNotReturnValue.into()),
        }
    }

    /// *Tries to* perform runtime storage changes, effectively updating the
    /// asset twap. Contrary to [`do_update_twap`](Self::do_update_twap), this
    /// variation does *not* throw an error if the current twap timestamp is
    /// more recent than the current time, returning from the call without
    /// modifying storage if this condition is true.
    ///
    /// # Errors
    ///
    /// * All errors returned by
    /// [`sanity_check_before_update_twap`](
    /// struct.Pallet.html#method.sanity_check_before_update_twap),
    /// except
    /// [`Error::<T>::AssetTwapTimestampIsMoreRecent`](
    /// ../pallet/enum.Error.html#variant.AssetTwapTimestampIsMoreRecent).
    #[transactional]
    pub fn try_update_twap(
        vamm_id: T::VammId,
        vamm_state: &mut VammStateOf<T>,
        current_price: Option<T::Decimal>,
        now: &Option<T::Moment>,
    ) -> Result<Option<T::Decimal>, DispatchError> {
        Self::internal_update_twap(vamm_id, vamm_state, current_price, now, true)
    }

    // TODO(Cardosaum): Update documentattion
    /// Handles the optional value for `base_twap` parameter in function
    /// [`update_twap`](struct.Pallet.html#method.update_twap), computing a new
    /// twap value if necessary.
    ///
    /// # Errors
    ///
    /// * [`ArithmeticError`](sp_runtime::ArithmeticError)
    fn handle_current_price(
        current_price: Option<T::Decimal>,
        vamm_state: &VammStateOf<T>,
    ) -> Result<T::Decimal, DispatchError> {
        match current_price {
            Some(current_price) => Ok(current_price),
            None => Self::do_get_price(vamm_state, AssetType::Base),
        }
    }

    /// Effectively mutate runtime storage and
    /// [`VammState`](../types/struct.VammState.html#structfield.base_asset_reserves).
    fn internal_update_twap(
        vamm_id: T::VammId,
        vamm_state: &mut VammStateOf<T>,
        current_price: Option<T::Decimal>,
        now: &Option<T::Moment>,
        try_update: bool,
    ) -> Result<Option<T::Decimal>, DispatchError> {
        // Handle optional value.
        let current_price = Self::handle_current_price(current_price, vamm_state)?;

        // Sanity checks must pass before updating runtime storage.
        match Self::sanity_check_before_update_twap(vamm_state, current_price, now, try_update)? {
            SanityCheckUpdateTwap::Abort => Ok(None),
            SanityCheckUpdateTwap::Proceed => {
                // We can safely update runtime storage.
                // Update VammState.
                let now = Self::now(now);
                let current_twap = vamm_state.base_asset_twap.accumulate(&current_price, now)?;

                // Update runtime storage.
                VammMap::<T>::insert(&vamm_id, vamm_state);

                Ok(Some(current_twap))
            },
        }
    }
}
