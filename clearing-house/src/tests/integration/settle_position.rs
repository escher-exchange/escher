use super::*;

#[test]
fn should_handle_one_long() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            MarketConfig::default()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
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

        let now = get_time_now();
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 12
        ));

        advance_blocks_by(1, 12);

        assert_ok!(TestPallet::settle_position(
            Origin::signed(ALICE),
            market_id
        ));

        // Alice's collateral remains unchanged
        assert_eq!(get_collateral(&ALICE), UNIT * 100);
        assert_eq!(get_outstanding_profits(&ALICE), 0);
        assert!(get_position(&ALICE, &market_id).is_none());
    })
}

#[test]
fn should_handle_both_longs() {
    let (alice_col0, bob_col0) = (UNIT * 100, UNIT * 100);
    ExtBuilder {
        balances: vec![(ALICE, USDC, alice_col0), (BOB, USDC, bob_col0)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            MarketConfig::default()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            alice_col0
        ));
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            bob_col0
        ));

        let market_id = Zero::zero();
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            alice_col0,
            0
        ));
        assert_ok!(TestPallet::open_position(
            Origin::signed(BOB),
            market_id,
            Long,
            bob_col0,
            0
        ));

        let now = get_time_now();
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 12
        ));

        advance_blocks_by(1, 12);

        assert_ok!(TestPallet::settle_position(
            Origin::signed(ALICE),
            market_id
        ));
        assert_ok!(TestPallet::settle_position(Origin::signed(BOB), market_id));

        let (alice_col, bob_col) = (get_collateral(&ALICE), get_collateral(&BOB));
        assert!(alice_col > alice_col0);
        assert!(bob_col < bob_col0);
        assert_eq!(alice_col + bob_col, alice_col0 + bob_col0);
    })
}

#[test]
fn should_handle_equivalent_long_and_short() {
    let (alice_col0, bob_col0) = (UNIT * 100, UNIT * 100);
    ExtBuilder {
        balances: vec![(ALICE, USDC, alice_col0), (BOB, USDC, bob_col0)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            MarketConfig::default()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            alice_col0
        ));
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            bob_col0
        ));

        let market_id = Zero::zero();
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            alice_col0,
            0
        ));
        assert_ok!(TestPallet::open_position(
            Origin::signed(BOB),
            market_id,
            Short,
            bob_col0,
            0
        ));

        let now = get_time_now();
        assert_ok!(TestPallet::close_market(
            Origin::root(),
            market_id,
            now + 12
        ));

        advance_blocks_by(1, 12);

        assert_ok!(TestPallet::settle_position(
            Origin::signed(ALICE),
            market_id
        ));
        assert_ok!(TestPallet::settle_position(Origin::signed(BOB), market_id));

        let (alice_col, bob_col) = (get_collateral(&ALICE), get_collateral(&BOB));
        assert_eq!(alice_col, alice_col0);
        assert_eq!(bob_col, bob_col0);
    })
}
