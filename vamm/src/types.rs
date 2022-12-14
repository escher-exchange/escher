use frame_support::pallet_prelude::*;
use sp_core::U256;
use sp_std::cmp::Ordering::Greater;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Data relating to the state of a virtual market.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VammState<Balance, Moment, Twap> {
    /// The total amount of base asset present in the vamm.
    pub base_asset_reserves: Balance,

    /// The total amount of quote asset present in the vamm.
    pub quote_asset_reserves: Balance,

    /// The amount of base asset when the vamm has no open positions against it.
    pub terminal_base_asset_reserves: Balance,

    /// The amount of quote asset when the vamm has no open positions against it.
    pub terminal_quote_asset_reserves: Balance,

    /// The magnitude of the quote asset reserve.
    pub peg_multiplier: Balance,

    /// The invariant `K`.
    pub invariant: U256,

    /// Whether this market is closed or not.
    ///
    /// This variable function as a signal to allow pallets who uses the
    /// Vamm to set a market as "operating as normal" or "not to be used
    /// anymore".  If the value is `None` it means the market is operating
    /// as normal, but if the value is `Some(timestamp)` it means the market
    /// is flagged to be closed and the closing action will take (or took)
    /// effect at the time `timestamp`.
    pub closed: Option<Moment>,

    /// The time weighted average price of
    /// [`base`](traits::vamm::AssetType::Base) asset w.r.t.
    /// [`quote`](traits::vamm::AssetType::Quote) asset. If wanting to get
    /// `quote_asset_twap`, just call `base_asset_twap.get_twap().reciprocal()`
    /// as those values should always be reciprocal of each other. For more
    /// information about computing the reciprocal, please check
    /// [`reciprocal`](sp_runtime::FixedPointNumber::reciprocal).
    pub base_asset_twap: Twap,
}

/// Represents the closing state of the vamm.
pub enum ClosingState {
    /// The vamm is open. All functionalities are working without restriction.
    Open,
    /// The vamm is open, but in the closing period. In some time in the future
    /// it will not perform any operation. If the vamm is in this state, some
    /// functionalities are already limited.
    Closing,
    /// The vamm is closed, all functionalities are restricted.
    Closed,
}

impl<Balance, Moment, Twap> VammState<Balance, Moment, Twap>
where
    Moment: Ord,
{
    /// Checks if the vamm is [`Open`](ClosingState::Open), in the
    /// [`Closing`](ClosingState::Closing) period or if it's already
    /// [`Closed`](ClosingState::Closed).
    ///
    /// To know in which exact state the vamm is the function requires
    /// the parameter `reference_time` to perform it's calculations. Usually the
    /// `reference_time` is the current timestamp, but it can be used to asses
    /// what would be the state of the vamm in the future (assuming no changes
    /// regarding closing it) or any past state.
    pub fn closing_state(&self, reference_time: &Moment) -> ClosingState {
        match &self.closed {
            Some(closing_time) => match closing_time.cmp(reference_time) {
                Greater => ClosingState::Closing,
                _ => ClosingState::Closed,
            },
            None => ClosingState::Open,
        }
    }
}

/// Represents the direction a of a position.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum SwapDirection {
    /// Adding an asset to the vamm, receiving the other in return.
    Add,
    /// Removing an asset from the vamm, giving the other in return.
    Remove,
}
