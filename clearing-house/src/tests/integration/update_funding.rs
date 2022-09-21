use super::*;

#[test]
fn should_update_oracle_twap() {
    ExtBuilder {
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        set_oracle_for(asset_id, 1_000); // Index price = 10.0

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
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        let market_id = Zero::zero();
        let market = get_market(&market_id);
        let vamm = get_vamm(&market.vamm_id);

        assert_eq!(market.last_oracle_price, 10.into());
        assert_eq!(market.last_oracle_twap, 10.into());
        assert_eq!(get_vamm_twap_value(&vamm), 10.into());

        update_oracle_for(asset_id, 1_100); //  Index price = 11.0
        run_to_time(market.last_oracle_ts + config.twap_period);
        assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
        let market = get_market(&market_id);
        // Oracle price updates are clipped at 10bps from the previous recorded price
        assert_eq!(market.last_oracle_price, (1001, 100).into());
        assert!(market.last_oracle_twap > 10.into());
    })
}

#[test]
fn should_update_vamm_twap() {
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
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        let market = get_market(&market_id);
        let vamm_before = get_vamm(&market.vamm_id);

        assert_eq!(market.last_oracle_price, 10.into());
        assert_eq!(market.last_oracle_twap, 10.into());
        assert_eq!(get_vamm_twap_value(&vamm_before), 10.into());

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

        run_to_time(market.last_oracle_ts + config.twap_period);
        assert_ok!(TestPallet::update_funding(Origin::signed(ALICE), market_id));
        let vamm_after = get_vamm(&market.vamm_id);
        assert!(get_vamm_twap_value(&vamm_before) < get_vamm_twap_value(&vamm_after));
    })
}

#[test]
fn should_block_update_if_mark_index_too_divergent() {
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
                // Mark price = 111.0
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 1_110_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        let market_id = Zero::zero();
        let market = get_market(&market_id);
        let vamm = get_vamm(&market.vamm_id);
        assert_eq!(market.last_oracle_twap, 100.into());
        assert_eq!(
            <Vamm as VammTrait>::get_price(market.vamm_id, AssetType::Base).unwrap(),
            111.into()
        );
        assert_eq!(get_vamm_twap_value(&vamm), 111.into());

        set_maximum_oracle_mark_divergence((1, 10).into());

        run_to_time(market.last_oracle_ts + config.twap_period);
        assert_noop!(
            TestPallet::update_funding(Origin::signed(ALICE), market_id),
            Error::<Runtime>::OracleMarkTooDivergent
        );
    })
}

#[test]
fn clearing_house_should_receive_funding() {
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
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        assert_eq!(get_market_fee_pool(market_id), 0);
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));

        let market = get_market(&market_id);
        run_to_time(market.last_oracle_ts + config.twap_period);
        // update_funding updates the vAMM TWAP, which increases since the previous trade pushed
        // the price upwards
        assert_ok!(TestPallet::update_funding(Origin::signed(BOB), market_id));
        assert!(get_market_fee_pool(market_id) > 0);
    })
}

#[test]
fn clearing_house_should_pay_funding() {
    ExtBuilder {
        balances: vec![(ALICE, USDC, UNIT * 100), (BOB, USDC, UNIT * 1_000_000)],
        ..Default::default()
    }
    .build()
    .execute_with(|| {
        let asset_id = DOT;
        // Oracle price (and TWAP) start at 20.0
        set_oracle_for(asset_id, 2_000);

        // vAMM price (and TWAP start at 10.0)
        let config = MarketConfig {
            asset: asset_id,
            vamm_config: VammConfig {
                base_asset_reserves: UNIT * 10_000,
                quote_asset_reserves: UNIT * 100_000,
                peg_multiplier: 1,
                twap_period: ONE_HOUR,
            },
            ..Default::default()
        };
        assert_ok!(TestPallet::create_market(
            Origin::signed(ALICE),
            config.clone()
        ));

        assert_ok!(TestPallet::deposit_collateral(
            Origin::signed(ALICE),
            USDC,
            UNIT * 100
        ));

        let market_id = Zero::zero();
        assert_eq!(get_market_fee_pool(market_id), 0);

        // Alice goes long, but not enough to bring mark price to index
        assert_ok!(TestPallet::open_position(
            Origin::signed(ALICE),
            market_id,
            Long,
            UNIT * 100,
            0
        ));

        // Populate Fee Pool with funds
        let fee_pool_before = UNIT * 1_000_000;
        <Assets as Transfer<AccountId>>::transfer(
            USDC,
            &BOB,
            &TestPallet::get_fee_pool_account(market_id),
            fee_pool_before,
            false,
        )
        .unwrap();

        let market = get_market(&market_id);
        run_to_time(market.last_oracle_ts + config.twap_period);
        assert_ok!(TestPallet::update_funding(Origin::signed(BOB), market_id));
        assert!(get_market_fee_pool(market_id) < fee_pool_before);
    })
}
