use super::*;

#[test]
fn should_succeed_in_opening_first_position() {
    ExtBuilder {
        balances: vec![
            (ALICE, PICA, UNIT),
            (BOB, PICA, UNIT),
            (BOB, USDC, UNIT * 100),
        ],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        set_oracle_for(DOT, 1_000);
        let config = MarketConfig {
            asset: DOT,
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 100,
                quote_asset_reserves: UNIT * 10_000,
                peg_multiplier: 10,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: Decimal::from_float(0.1),
            margin_ratio_maintenance: Decimal::from_float(0.02),
            margin_ratio_partial: Decimal::from_float(0.04),
            minimum_trade_size: 0.into(),
            funding_frequency: ONE_HOUR,
            funding_period: ONE_HOUR * 24,
            taker_fee: 0,
            twap_period: ONE_HOUR,
        };
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            UNIT * 100
        ));

        let market = get_market(&MarketId::zero());
        let vamm_state = get_vamm(&market.vamm_id);

        assert_ok!(TestPallet::open_position(
            Origin::signed(BOB),
            Zero::zero(),
            Long,
            UNIT * 100,
            0
        ));

        assert_ne!(get_market(&MarketId::zero()), market);
        assert_ne!(get_vamm(&market.vamm_id), vamm_state);
    })
}

#[test]
fn should_enforce_slippage_controls() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 10_000);

        let config = MarketConfig {
            asset: asset_id,
            vamm_config: VammConfig {
                // Start with a mark price of 100
                base_asset_reserves: UNIT * 100,
                quote_asset_reserves: UNIT * 10_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(Origin::signed(BOB), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        assert_noop!(
            TestPallet::open_position(Origin::signed(ALICE), market_id, Long, UNIT * 100, UNIT),
            pallet_vamm::Error::<Runtime>::SwappedAmountLessThanMinimumLimit
        );
    })
}

#[test]
fn should_succeed_with_two_traders_in_a_market() {
    ExtBuilder {
        balances: vec![
            (ALICE, PICA, UNIT),
            (BOB, PICA, UNIT),
            (ALICE, USDC, UNIT * 100),
            (BOB, USDC, UNIT * 100),
        ],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = PICA;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            asset: asset_id,
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 100,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
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
        let market = get_market(&market_id);
        let vamm_state_before = get_vamm(&market.vamm_id);

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
            Long,
            UNIT * 100,
            0
        ));

        assert_ok!(TestPallet::close_position(Origin::signed(ALICE), market_id));
        assert_ok!(TestPallet::close_position(Origin::signed(BOB), market_id));

        // Alice closes her position in profit, Bob closes his position in loss
        // However, since Alice closes her position first, there are no realized losses in the
        // market yet, so her profits are outstanding
        let alice_col = get_collateral(&ALICE);
        let alice_outstanding_profits = get_outstanding_profits(&ALICE);
        let bob_col = get_collateral(&BOB);
        assert!(alice_col + alice_outstanding_profits > bob_col);
        assert_eq!(alice_col + alice_outstanding_profits + bob_col, UNIT * 200);

        assert_ok!(TestPallet::withdraw_collateral(
            Origin::signed(ALICE),
            alice_col + alice_outstanding_profits
        ));

        // vAMM is back to its initial state due to path independence
        let vamm_state_after = get_vamm(&market.vamm_id);
        assert_eq!(
            vamm_state_before.base_asset_reserves,
            vamm_state_after.base_asset_reserves
        );
        assert_eq!(
            vamm_state_before.quote_asset_reserves,
            vamm_state_after.quote_asset_reserves
        );
    })
}

#[test]
#[ignore = "FIXME: vAMM TWAP isn't updated if last twap timestamp is equal to the current \
block's timestamp"]
fn should_update_vamm_twap_in_the_same_block() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            asset: asset_id,
            vamm_config: VammConfig {
                // Mark price = 10.0
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        let market = get_market(&market_id);
        let vamm_before = get_vamm(&market.vamm_id);

        assert_eq!(vamm_before.base_asset_twap, 10.into());

        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));
        let vamm_after = get_vamm(&market.vamm_id);
        // open_position should update TWAP before swapping, therefore not changing the mark
        // TWAP
        assert_eq!(vamm_before.base_asset_twap, vamm_after.base_asset_twap);
        let vamm_before = vamm_after;

        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));
        let vamm_after = get_vamm(&market.vamm_id);
        // now the vAMM picks up the change caused by the previous swap
        assert!(vamm_before.base_asset_twap < vamm_after.base_asset_twap);
    })
}

#[test]
fn should_update_vamm_twap_across_blocks() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            asset: asset_id,
            vamm_config: VammConfig {
                // Mark price = 10.0
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        let market = get_market(&market_id);
        let vamm_before = get_vamm(&market.vamm_id);

        assert_eq!(vamm_before.base_asset_twap, 10.into());

        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));
        let vamm_after = get_vamm(&market.vamm_id);
        // open_position should update TWAP before swapping, therefore not changing the mark
        // TWAP
        assert_eq!(vamm_before.base_asset_twap, vamm_after.base_asset_twap);
        let vamm_before = vamm_after;

        advance_blocks_by(1, 1);

        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));
        let vamm_after = get_vamm(&market.vamm_id);
        // now the vAMM picks up the change caused by the previous swap
        assert!(vamm_before.base_asset_twap < vamm_after.base_asset_twap);
    })
}
