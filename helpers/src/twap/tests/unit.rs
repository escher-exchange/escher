#![allow(clippy::identity_op)]
use crate::twap::Twap;
use frame_support::assert_ok;
use plotters::prelude::*;
use polars::prelude::*;
use rstest::rstest;
use sp_runtime::FixedU128;

// -------------------------------------------------------------------------------------------------
//                                             Constants
// -------------------------------------------------------------------------------------------------

const SECOND: u64 = 1000; // 1 second, in millis
const MINUTE: u64 = 60 * SECOND;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

const PERIOD: u64 = 1 * DAY; // Default period for twap

// -------------------------------------------------------------------------------------------------
//                                         Helper Functions
// -------------------------------------------------------------------------------------------------

fn from_float(x: f64) -> FixedU128 {
    FixedU128::from_float(x)
}

// -------------------------------------------------------------------------------------------------
//                                            Unit Tests
// -------------------------------------------------------------------------------------------------

#[rstest]
#[case(u128::MIN, u64::MIN, PERIOD)]
#[case(u128::MAX, u64::MAX, PERIOD)]
#[case(u128::MIN, u64::MAX, PERIOD)]
#[case(u128::MAX, u64::MIN, PERIOD)]
#[case(0, 0, PERIOD)]
fn should_create_twap_struct_successfully(
    #[case] twap: u128,
    #[case] ts: u64,
    #[case] period: u64,
) {
    let twap = FixedU128::from_inner(twap);
    let t = Twap::new(twap, ts, period);
    assert_eq!(t.twap, twap);
    assert_eq!(t.ts, ts);
    assert_eq!(t.period, period);
}

#[test]
fn should_update_twap_to_correct_value() {
    // Initialize twap to 100,
    // Set timestamp to "Mon Aug  8 11:06:40 PM UTC 2022"
    let mut ts = 1660000000;
    let mut t = Twap::new(from_float(100.0), ts, PERIOD);

    // After half PERDIOD passes, we update the twap.
    ts += PERIOD / 2;
    t.accumulate(&from_float(200.0), ts);

    // The value should be half the previous price and half the new one.
    assert_eq!(t.twap, from_float(150.0));
}

#[test]
fn should_update_twap_on_accumulate_call() {
    let mut t = Twap::new(from_float(25.0), 0, PERIOD);
    assert_ok!(t.accumulate(&from_float(50.0), PERIOD / 2));
}

#[test]
fn should_succeed_setting_and_retrieving_values() {
    let mut t = Twap::new(from_float(0.0), 0, PERIOD);

    let price = from_float(25.0);
    let ts = 10;
    t.set_twap(price);
    t.set_timestamp(ts);

    assert_eq!(t.get_twap(), price);
    assert_eq!(t.get_timestamp(), ts);
    assert_eq!(t.get_period(), PERIOD);
}

#[rstest]
#[case(1 * HOUR, 0)]
#[case(2 * HOUR, 0)]
#[case(3 * HOUR, 0)]
#[case(4 * HOUR, 0)]
#[case(5 * HOUR, 0)]
#[case(6 * HOUR, 0)]
#[case(7 * HOUR, 0)]
#[case(8 * HOUR, 0)]
#[case(9 * HOUR, 0)]
#[case(10 * HOUR, 0)]
#[case(11 * HOUR, 0)]
#[case(12 * HOUR, 0)]
#[case(1 * DAY, 0)]
#[case(2 * DAY, 0)]
#[case(3 * DAY, 0)]
#[case(4 * DAY, 0)]
#[case(5 * DAY, 0)]
#[case(6 * DAY, 0)]
#[case(7 * DAY, 0)]
#[case(1 * HOUR, 1)]
#[case(2 * HOUR, 1)]
#[case(3 * HOUR, 1)]
#[case(4 * HOUR, 1)]
#[case(5 * HOUR, 1)]
#[case(6 * HOUR, 1)]
#[case(7 * HOUR, 1)]
#[case(8 * HOUR, 1)]
#[case(9 * HOUR, 1)]
#[case(10 * HOUR, 1)]
#[case(11 * HOUR, 1)]
#[case(12 * HOUR, 1)]
#[case(1 * DAY, 1)]
#[case(2 * DAY, 1)]
#[case(3 * DAY, 1)]
#[case(4 * DAY, 1)]
#[case(5 * DAY, 1)]
#[case(6 * DAY, 1)]
#[case(7 * DAY, 1)]
#[case(1 * HOUR, 2)]
#[case(2 * HOUR, 2)]
#[case(3 * HOUR, 2)]
#[case(4 * HOUR, 2)]
#[case(5 * HOUR, 2)]
#[case(6 * HOUR, 2)]
#[case(7 * HOUR, 2)]
#[case(8 * HOUR, 2)]
#[case(9 * HOUR, 2)]
#[case(10 * HOUR, 2)]
#[case(11 * HOUR, 2)]
#[case(12 * HOUR, 2)]
#[case(1 * DAY, 2)]
#[case(2 * DAY, 2)]
#[case(3 * DAY, 2)]
#[case(4 * DAY, 2)]
#[case(5 * DAY, 2)]
#[case(6 * DAY, 2)]
#[case(7 * DAY, 2)]
#[case(1 * HOUR, 3)]
#[case(2 * HOUR, 3)]
#[case(3 * HOUR, 3)]
#[case(4 * HOUR, 3)]
#[case(5 * HOUR, 3)]
#[case(6 * HOUR, 3)]
#[case(7 * HOUR, 3)]
#[case(8 * HOUR, 3)]
#[case(9 * HOUR, 3)]
#[case(10 * HOUR, 3)]
#[case(11 * HOUR, 3)]
#[case(12 * HOUR, 3)]
#[case(1 * DAY, 3)]
#[case(2 * DAY, 3)]
#[case(3 * DAY, 3)]
#[case(4 * DAY, 3)]
#[case(5 * DAY, 3)]
#[case(6 * DAY, 3)]
#[case(7 * DAY, 3)]
#[case(1 * HOUR, 4)]
#[case(2 * HOUR, 4)]
#[case(3 * HOUR, 4)]
#[case(4 * HOUR, 4)]
#[case(5 * HOUR, 4)]
#[case(6 * HOUR, 4)]
#[case(7 * HOUR, 4)]
#[case(8 * HOUR, 4)]
#[case(9 * HOUR, 4)]
#[case(10 * HOUR, 4)]
#[case(11 * HOUR, 4)]
#[case(12 * HOUR, 4)]
#[case(1 * DAY, 4)]
#[case(2 * DAY, 4)]
#[case(3 * DAY, 4)]
#[case(4 * DAY, 4)]
#[case(5 * DAY, 4)]
#[case(6 * DAY, 4)]
#[case(7 * DAY, 4)]
#[case(1 * HOUR, 5)]
#[case(2 * HOUR, 5)]
#[case(3 * HOUR, 5)]
#[case(4 * HOUR, 5)]
#[case(5 * HOUR, 5)]
#[case(6 * HOUR, 5)]
#[case(7 * HOUR, 5)]
#[case(8 * HOUR, 5)]
#[case(9 * HOUR, 5)]
#[case(10 * HOUR, 5)]
#[case(11 * HOUR, 5)]
#[case(12 * HOUR, 5)]
#[case(1 * DAY, 5)]
#[case(2 * DAY, 5)]
#[case(3 * DAY, 5)]
#[case(4 * DAY, 5)]
#[case(5 * DAY, 5)]
#[case(6 * DAY, 5)]
#[case(7 * DAY, 5)]
#[case(1 * HOUR, 6)]
#[case(2 * HOUR, 6)]
#[case(3 * HOUR, 6)]
#[case(4 * HOUR, 6)]
#[case(5 * HOUR, 6)]
#[case(6 * HOUR, 6)]
#[case(7 * HOUR, 6)]
#[case(8 * HOUR, 6)]
#[case(9 * HOUR, 6)]
#[case(10 * HOUR, 6)]
#[case(11 * HOUR, 6)]
#[case(12 * HOUR, 6)]
#[case(1 * DAY, 6)]
#[case(2 * DAY, 6)]
#[case(3 * DAY, 6)]
#[case(4 * DAY, 6)]
#[case(5 * DAY, 6)]
#[case(6 * DAY, 6)]
#[case(7 * DAY, 6)]
#[case(1 * HOUR, 7)]
#[case(2 * HOUR, 7)]
#[case(3 * HOUR, 7)]
#[case(4 * HOUR, 7)]
#[case(5 * HOUR, 7)]
#[case(6 * HOUR, 7)]
#[case(7 * HOUR, 7)]
#[case(8 * HOUR, 7)]
#[case(9 * HOUR, 7)]
#[case(10 * HOUR, 7)]
#[case(11 * HOUR, 7)]
#[case(12 * HOUR, 7)]
#[case(1 * DAY, 7)]
#[case(2 * DAY, 7)]
#[case(3 * DAY, 7)]
#[case(4 * DAY, 7)]
#[case(5 * DAY, 7)]
#[case(6 * DAY, 7)]
#[case(7 * DAY, 7)]
#[case(1 * HOUR, 8)]
#[case(2 * HOUR, 8)]
#[case(3 * HOUR, 8)]
#[case(4 * HOUR, 8)]
#[case(5 * HOUR, 8)]
#[case(6 * HOUR, 8)]
#[case(7 * HOUR, 8)]
#[case(8 * HOUR, 8)]
#[case(9 * HOUR, 8)]
#[case(10 * HOUR, 8)]
#[case(11 * HOUR, 8)]
#[case(12 * HOUR, 8)]
#[case(1 * DAY, 8)]
#[case(2 * DAY, 8)]
#[case(3 * DAY, 8)]
#[case(4 * DAY, 8)]
#[case(5 * DAY, 8)]
#[case(6 * DAY, 8)]
#[case(7 * DAY, 8)]
#[case(1 * HOUR, 9)]
#[case(2 * HOUR, 9)]
#[case(3 * HOUR, 9)]
#[case(4 * HOUR, 9)]
#[case(5 * HOUR, 9)]
#[case(6 * HOUR, 9)]
#[case(7 * HOUR, 9)]
#[case(8 * HOUR, 9)]
#[case(9 * HOUR, 9)]
#[case(10 * HOUR, 9)]
#[case(11 * HOUR, 9)]
#[case(12 * HOUR, 9)]
#[case(1 * DAY, 9)]
#[case(2 * DAY, 9)]
#[case(3 * DAY, 9)]
#[case(4 * DAY, 9)]
#[case(5 * DAY, 9)]
#[case(6 * DAY, 9)]
#[case(7 * DAY, 9)]
#[case(1 * HOUR, 10)]
#[case(2 * HOUR, 10)]
#[case(3 * HOUR, 10)]
#[case(4 * HOUR, 10)]
#[case(5 * HOUR, 10)]
#[case(6 * HOUR, 10)]
#[case(7 * HOUR, 10)]
#[case(8 * HOUR, 10)]
#[case(9 * HOUR, 10)]
#[case(10 * HOUR, 10)]
#[case(11 * HOUR, 10)]
#[case(12 * HOUR, 10)]
#[case(1 * DAY, 10)]
#[case(2 * DAY, 10)]
#[case(3 * DAY, 10)]
#[case(4 * DAY, 10)]
#[case(5 * DAY, 10)]
#[case(6 * DAY, 10)]
#[case(7 * DAY, 10)]
#[case(1 * HOUR, 11)]
#[case(2 * HOUR, 11)]
#[case(3 * HOUR, 11)]
#[case(4 * HOUR, 11)]
#[case(5 * HOUR, 11)]
#[case(6 * HOUR, 11)]
#[case(7 * HOUR, 11)]
#[case(8 * HOUR, 11)]
#[case(9 * HOUR, 11)]
#[case(10 * HOUR, 11)]
#[case(11 * HOUR, 11)]
#[case(12 * HOUR, 11)]
#[case(1 * DAY, 11)]
#[case(2 * DAY, 11)]
#[case(3 * DAY, 11)]
#[case(4 * DAY, 11)]
#[case(5 * DAY, 11)]
#[case(6 * DAY, 11)]
#[case(7 * DAY, 11)]
#[case(1 * HOUR, 12)]
#[case(2 * HOUR, 12)]
#[case(3 * HOUR, 12)]
#[case(4 * HOUR, 12)]
#[case(5 * HOUR, 12)]
#[case(6 * HOUR, 12)]
#[case(7 * HOUR, 12)]
#[case(8 * HOUR, 12)]
#[case(9 * HOUR, 12)]
#[case(10 * HOUR, 12)]
#[case(11 * HOUR, 12)]
#[case(12 * HOUR, 12)]
#[case(1 * DAY, 12)]
#[case(2 * DAY, 12)]
#[case(3 * DAY, 12)]
#[case(4 * DAY, 12)]
#[case(5 * DAY, 12)]
#[case(6 * DAY, 12)]
#[case(7 * DAY, 12)]
#[ignore = "TODO: Still need to configure flag to run this test as this \
            one takes quite a while and is not meant to be run every time"]
fn test_polars(#[case] period: u64, #[case] dataset: i32) {
    let mut twap: Option<Twap<FixedU128, u64>> = None;
    let df = LazyCsvReader::new("eda/raw_data/final.csv".into())
        .has_header(true)
        .finish()
        .unwrap()
        .filter(col("market_index").eq(lit(dataset)))
        .select(&[col("ts"), col("mark_price_before")])
        .with_column(
            col("ts")
                .str()
                .strptime(StrpTimeOptions {
                    date_dtype: DataType::Datetime(TimeUnit::Milliseconds, None),
                    fmt: Some("%Y-%m-%d %T%z".to_string()),
                    strict: true,
                    exact: true,
                })
                .dt()
                .timestamp(TimeUnit::Milliseconds)
                .alias("timestamp"),
        )
        .sort(
            "timestamp",
            SortOptions {
                descending: false,
                nulls_last: true,
            },
        )
        .with_column(
            as_struct(&[cols(vec!["timestamp", "mark_price_before"])])
                .map(
                    move |data| {
                        Ok(Series::from_vec(
                            "parsed_twap",
                            data.iter()
                                .map(move |i| match i {
                                    AnyValue::Struct(v, _) => {
                                        let (now, price) = match v[..] {
                                            [AnyValue::Int64(now), AnyValue::Int64(price)] =>
                                                (now as u64, {
                                                    FixedU128::from_inner(
                                                        (price as u128)
                                                            .saturating_mul(10_u128.pow(12)),
                                                    )
                                                }),
                                            _ => panic!(
                                                "Could not extranct `now` and `price` values"
                                            ),
                                        };
                                        match twap {
                                            Some(ref mut t) => t
                                                .accumulate(&price, now)
                                                .unwrap_or_else(|_| {
                                                    panic!(
                                                        "Failed to accumulate twap, {now} {price}"
                                                    )
                                                })
                                                .to_float(),
                                            None => {
                                                twap = Some(Twap::new(price, now, period));
                                                twap.unwrap().get_twap().to_float()
                                            },
                                        }
                                    },
                                    _ => panic!("Failed to parse a struct field"),
                                })
                                .collect::<Vec<f64>>(),
                        ))
                    },
                    GetOutput::from_type(DataType::Float64),
                )
                .alias("twap"),
        )
        .with_column(
            col("mark_price_before")
                .map(
                    |p| {
                        Ok(Series::from_vec(
                            "price",
                            p.iter()
                                .map(|x| match x {
                                    AnyValue::Int64(i) => i as f64 / 10.0_f64.powf(6.),
                                    err => panic!("Failed to parse int: {err}"),
                                })
                                .collect::<Vec<f64>>(),
                        ))
                    },
                    GetOutput::from_type(DataType::Int64),
                )
                .alias("price"),
        )
        .collect()
        .unwrap();

    let x_lim_0 = df["timestamp"].min::<i64>().unwrap();
    let x_lim_1 = df["timestamp"].max::<i64>().unwrap();
    let y_lim_0 = df["price"]
        .min::<f64>()
        .unwrap()
        .min(df["twap"].min::<f64>().unwrap());
    let y_lim_1 = df["price"]
        .max::<f64>()
        .unwrap()
        .max(df["twap"].max::<f64>().unwrap());

    let file_name = format!("eda/imgs/test_{dataset:02}_{period:010}.png");
    dbg!(&file_name);
    let root = BitMapBackend::new(&file_name, (3840, 2160)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(
            format!("Dataset = {dataset:02}, Period = {period:010}"),
            ("sans-serif", 40),
        )
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(x_lim_0..x_lim_1, y_lim_0..y_lim_1)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .max_light_lines(4)
        .y_desc("price/twap")
        .draw()
        .unwrap();

    let price_plot = df["timestamp"]
        .iter()
        .zip(df["price"].iter())
        .map(|(ts, price)| {
            (
                match ts {
                    AnyValue::Int64(ts) => ts,
                    _ => panic!(),
                },
                match price {
                    AnyValue::Float64(price) => price,
                    _ => panic!(),
                },
            )
        });
    let twap_plot = df["timestamp"]
        .iter()
        .zip(df["twap"].iter())
        .map(|(ts, price)| {
            (
                match ts {
                    AnyValue::Int64(ts) => ts,
                    _ => panic!(),
                },
                match price {
                    AnyValue::Float64(price) => price,
                    _ => panic!(),
                },
            )
        });
    chart
        .draw_series(LineSeries::new(price_plot, &BLUE))
        .unwrap();
    chart.draw_series(LineSeries::new(twap_plot, &RED)).unwrap();
    root.present().unwrap();
}
