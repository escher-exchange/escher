use crate::{
    mock::{Balance, ExtBuilder, MockRuntime, TestPallet},
    tests::{
        helpers::{as_balance, run_for_seconds},
        types::{TestSwapConfig, TestVammConfig},
    },
    Error,
};
use frame_support::{assert_noop, assert_ok};
use rstest::rstest;
use sp_runtime::traits::Zero;
use traits::vamm::{AssetType, Direction, Vamm};

// -------------------------------------------------------------------------------------------------
//                                            Helpers
// -------------------------------------------------------------------------------------------------

fn create_trade_and_close(long_size: Option<Balance>, short_size: Option<Balance>) {
    assert_ok!(TestPallet::create(&TestVammConfig::default().into()), 0);

    match long_size {
        None => (),
        Some(amount) => {
            assert_ok!(TestPallet::swap(
                &TestSwapConfig {
                    vamm_id: 0,
                    asset: AssetType::Base,
                    input_amount: amount,
                    direction: Direction::Add,
                    output_amount_limit: Zero::zero(),
                }
                .into()
            ));
        },
    };
    match short_size {
        None => (),
        Some(amount) => {
            assert_ok!(TestPallet::swap(
                &TestSwapConfig {
                    vamm_id: 0,
                    asset: AssetType::Base,
                    input_amount: amount,
                    direction: Direction::Remove,
                    output_amount_limit: Balance::MAX,
                }
                .into()
            ));
        },
    };

    assert_ok!(TestPallet::close(0, 10));
    run_for_seconds(10);
}

// -------------------------------------------------------------------------------------------------
//                                           Tests
// -------------------------------------------------------------------------------------------------

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
fn should_fail_if_vamm_is_open() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(TestPallet::create(&TestVammConfig::default().into()), 0);

        assert_noop!(
            TestPallet::get_settlement_price(0),
            Error::<MockRuntime>::VammIsNotClosed
        );
    })
}

#[test]
fn should_fail_if_vamm_is_closing() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(TestPallet::create(&TestVammConfig::default().into()), 0);

        assert_ok!(TestPallet::close(0, 10));

        assert_noop!(
            TestPallet::get_settlement_price(0),
            Error::<MockRuntime>::VammIsNotClosed
        );
    })
}

#[test]
fn should_succeed_if_vamm_is_closed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(TestPallet::create(&TestVammConfig::default().into()), 0);

        assert_ok!(TestPallet::close(0, 10));
        run_for_seconds(10);

        assert_ok!(TestPallet::get_settlement_price(0));
    })
}

#[rstest]
#[case(None, None)]
#[case(Some(as_balance(100)), Some(as_balance(100)))]
fn should_return_zero_if_at_terminal_reserves(
    #[case] long_size: Option<Balance>,
    #[case] short_size: Option<Balance>,
) {
    ExtBuilder::default().build().execute_with(|| {
        // Simulate equal long and shorts
        create_trade_and_close(long_size, short_size);

        assert_ok!(TestPallet::get_settlement_price(0), 0.into());
    })
}

#[rstest]
#[case(None, Some(100))]
#[case(Some(100), None)]
#[case(Some(50), Some(100))]
#[case(Some(100), Some(50))]
fn should_return_nonzero_if_not_at_terminal_reserves(
    #[case] long_size: Option<Balance>,
    #[case] short_size: Option<Balance>,
) {
    ExtBuilder::default().build().execute_with(|| {
        // Simulate unequal long and shorts
        create_trade_and_close(long_size, short_size);

        let price = TestPallet::get_settlement_price(0);
        assert_ok!(price);
        assert!(!price.unwrap().is_zero());
    })
}
