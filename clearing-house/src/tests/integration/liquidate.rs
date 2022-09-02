use super::*;

#[test]
fn should_not_liquidate_if_above_partial_margin_ratio() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 10), (BOB, USDC, UNIT * 1_000_000)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        set_partial_liquidation_close((25, 100).into());
        set_partial_liquidation_penalty((25, 1000).into());
        set_liquidator_share_partial((50, 100).into());

        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: (100, 1000).into(), // 10x max leverage
            margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
            margin_ratio_maintenance: (80, 1000).into(),
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 10
        ));

        let market_id = Zero::zero();

        // Alice goes long with maximum leverage
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));

        assert_noop!(
            TestPallet::liquidate(Origin::signed(BOB), ALICE),
            Error::<Runtime>::SufficientCollateral
        );
    })
}

#[test]
fn should_liquidate_if_below_partial_margin_ratio_by_pnl() {
    let alice_col = UNIT * 10;
    let bob_col = UNIT * 1_000;

    ExtBuilder {
        balances: vec![(ALICE, USDC, alice_col), (BOB, USDC, bob_col)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        set_partial_liquidation_close((25, 100).into());
        set_partial_liquidation_penalty((25, 1000).into());
        set_liquidator_share_partial((50, 100).into());

        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: (100, 1000).into(), // 10x max leverage
            margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
            margin_ratio_maintenance: (80, 1000).into(),
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            alice_col
        ));
        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(BOB),
            USDC,
            bob_col
        ));

        let market_id = Zero::zero();

        // Alice goes long with maximum leverage
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            alice_col * 10,
            0
        ));

        // Bob goes short with size, pushing the price below Alice's partial liquidation
        // threshold
        assert_ok!(TestPallet::open_position(
            Origin::signed(BOB),
            market_id,
            Short,
            bob_col,
            Balance::MAX
        ));

        let unrealized_pnl = get_unrealized_pnl(&ALICE, &market_id);
        assert!(unrealized_pnl < Zero::zero());

        assert_ok!(TestPallet::liquidate(Origin::signed(BOB), ALICE));
        assert!(get_collateral(&BOB) > bob_col);
        assert!(get_insurance_acc_balance() > Zero::zero());
    })
}

#[test]
fn should_liquidate_if_below_partial_margin_ratio_by_funding() {
    let collateral = UNIT * 10;
    ExtBuilder {
        balances: vec![(ALICE, USDC, collateral)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        set_partial_liquidation_close((25, 100).into());
        set_partial_liquidation_penalty((25, 1000).into());
        set_liquidator_share_partial((50, 100).into());

        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000);

        let config = MarketConfig {
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            margin_ratio_initial: (100, 1000).into(), // 10x max leverage
            margin_ratio_partial: (99, 1000).into(),  // ~10.1x max leverage
            margin_ratio_maintenance: (80, 1000).into(),
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            collateral
        ));

        let market_id = Zero::zero();

        // Alice goes long with maximum leverage
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            collateral * 10,
            0
        ));

        // Index price moves against Alice's position
        update_oracle_for(asset_id, 900);
        let market = get_market(&market_id);
        run_to_time(market.last_oracle_ts + config.twap_period);
        assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));

        // Give time for TWAP to catch up to index
        for _ in 0..10 {
            advance_blocks_by(1, config.twap_period);
            assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
            dbg!(get_market(&market_id).last_oracle_twap);
        }

        assert_ok!(TestPallet::liquidate(Origin::signed(BOB), ALICE));
        System::assert_last_event(Event::PartialLiquidation { user: ALICE }.into());
    })
}
