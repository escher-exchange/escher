use super::*;

proptest! {
    #[test]
    fn should_succeed_in_creating_first_market(
        asset_id in prop_oneof![Just(DOT), Just(PICA)]
    ) {
        ExtBuilder {
            balances: vec![(ALICE, PICA, UNIT), (BOB, PICA, UNIT)],
            ..Default::default()
        }
        .build()
        .execute_with(|| {
            set_oracle_for(asset_id, 1_000); // 10 in cents
            let config = MarketConfig { asset: asset_id, ..Default::default() };
            assert_ok!(TestPallet::create_market(Origin::signed(ALICE), config));

            let market_id = MarketId::zero();
            let market = TestPallet::get_market(&market_id).unwrap();
            assert_eq!(market.asset_id, asset_id);
            assert_eq!(market.last_oracle_price, Decimal::from(10));
            assert_eq!(market.last_oracle_twap, Decimal::from(10));
        })
    }
}
