use super::*;

#[test]
#[ignore = "Vamm::close not yet implemented"]
fn should_close_market_and_vamm_under_normal_conditions() {
    ExtBuilder {
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig::default();
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        let market_id = Zero::zero();
        let now = get_time_now();
        assert_noop!(
            TestPallet::close_market(Origin::signed(BOB), market_id, now + 10),
            BadOrigin
        );
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 10
        ));

        let market = get_market(&market_id);
        let vamm = get_vamm(&market.vamm_id);
        assert_eq!(vamm.closed, Some(now + 10));
    })
}

#[test]
fn should_block_closing_positions_only_after_market_close() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig::default();
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));
        assert_ok!(TestPallet::open_position(
            Origin::signed(BOB),
            market_id,
            Short,
            UNIT * 100,
            Balance::MAX
        ));

        let now = get_time_now();
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 12
        ));

        advance_blocks_by(1, 6);
        assert_ok!(TestPallet::close_position(Origin::signed(ALICE), market_id));

        advance_blocks_by(1, 6);
        assert_noop!(
            TestPallet::close_position(Origin::signed(BOB), market_id),
            Error::<Runtime>::MarketClosed
        );
    })
}

#[test]
fn should_not_allow_opening_positions_after_close_market_call() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig::default();
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        let market_id = Zero::zero();
        let now = get_time_now();
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 12
        ));

        advance_blocks_by(1, 6);
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));
        assert_noop!(
            TestPallet::open_position(Origin::signed(ALICE), market_id, Long, UNIT * 100, 0),
            Error::<Runtime>::MarketShuttingDown
        );

        advance_blocks_by(1, 6);
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            UNIT * 100
        ));
        assert_noop!(
            TestPallet::open_position(
                Origin::signed(BOB),
                market_id,
                Short,
                UNIT * 100,
                Balance::MAX
            ),
            Error::<Runtime>::MarketClosed
        );
    })
}
