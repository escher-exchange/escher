use crate::{
    mock::assets::{AssetId, USDC},
    Config, FullLiquidationPenalty, FullLiquidationPenaltyLiquidatorShare, Market,
    MaxPriceDivergence, Pallet, PartialLiquidationCloseRatio, PartialLiquidationPenalty,
    PartialLiquidationPenaltyLiquidatorShare, Position,
};
use frame_support::traits::{fungible::Inspect as NativeInspect, fungibles::Inspect};
use sp_runtime::traits::Zero;

pub fn get_collateral<T: Config>(account_id: &T::AccountId) -> T::Balance {
    Pallet::<T>::get_collateral(account_id).unwrap()
}

pub fn get_outstanding_profits<T: Config>(account_id: &T::AccountId) -> T::Balance {
    Pallet::<T>::outstanding_profits(account_id).unwrap_or_else(Zero::zero)
}

pub fn get_market<T: Config>(market_id: &T::MarketId) -> Market<T> {
    Pallet::<T>::get_market(market_id).unwrap()
}

pub fn get_position<T: Config>(
    account_id: &T::AccountId,
    market_id: &T::MarketId,
) -> Option<Position<T>> {
    let positions = Pallet::<T>::get_positions(account_id);
    positions.into_iter().find(|p| p.market_id == *market_id)
}

pub fn get_market_fee_pool<T>(market_id: T::MarketId) -> <T as pallet_assets::Config>::Balance
where
    T: Config + pallet_assets::Config<AssetId = AssetId>,
    <T as pallet_assets::Config>::NativeCurrency:
        NativeInspect<T::AccountId, Balance = <T as pallet_assets::Config>::Balance>,
    <T as pallet_assets::Config>::MultiCurrency:
        Inspect<T::AccountId, AssetId = AssetId, Balance = <T as pallet_assets::Config>::Balance>,
{
    <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
        USDC,
        &Pallet::<T>::get_fee_pool_account(market_id),
    )
}

pub fn get_insurance_acc_balance<T>() -> <T as pallet_assets::Config>::Balance
where
    T: Config + pallet_assets::Config<AssetId = AssetId>,
    <T as pallet_assets::Config>::NativeCurrency:
        NativeInspect<T::AccountId, Balance = <T as pallet_assets::Config>::Balance>,
    <T as pallet_assets::Config>::MultiCurrency:
        Inspect<T::AccountId, AssetId = AssetId, Balance = <T as pallet_assets::Config>::Balance>,
{
    <pallet_assets::Pallet<T> as Inspect<T::AccountId>>::balance(
        USDC,
        &Pallet::<T>::get_insurance_account(),
    )
}

pub fn set_maximum_oracle_mark_divergence<T: Config>(fraction: T::Decimal) {
    MaxPriceDivergence::<T>::set(fraction);
}

pub fn set_full_liquidation_penalty<T: Config>(decimal: T::Decimal) {
    FullLiquidationPenalty::<T>::set(decimal);
}

pub fn set_liquidator_share_full<T: Config>(decimal: T::Decimal) {
    FullLiquidationPenaltyLiquidatorShare::<T>::set(decimal);
}

pub fn set_partial_liquidation_penalty<T: Config>(decimal: T::Decimal) {
    PartialLiquidationPenalty::<T>::set(decimal);
}

pub fn set_partial_liquidation_close<T: Config>(decimal: T::Decimal) {
    PartialLiquidationCloseRatio::<T>::set(decimal);
}

pub fn set_liquidator_share_partial<T: Config>(decimal: T::Decimal) {
    PartialLiquidationPenaltyLiquidatorShare::<T>::set(decimal);
}
