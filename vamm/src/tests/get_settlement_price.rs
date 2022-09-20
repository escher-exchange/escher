use crate::{
    mock::{ExtBuilder, MockRuntime, TestPallet},
    Error, tests::types::TestVammConfig,
};
use frame_support::{assert_noop, assert_ok};
use traits::vamm::Vamm;


#[test]
fn should_fail_if_vamm_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            TestPallet::get_settlement_price(0),
            Error::<MockRuntime>::VammDoesNotExist
        );
    })
}

#[test]
fn should_fail_if_vamm_is_not_closed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(TestPallet::create(&TestVammConfig::default().into()), 0);

        assert_noop!(
            TestPallet::get_settlement_price(0),
            Error::<MockRuntime>::VammIsOpen
        );
    })
}
