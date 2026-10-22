use super::*;

fn fixed(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&Utc)
}

fn diagnose(
    labels: &[&str],
    market: TimingMarket,
    convention: MinuteLabelConvention,
) -> LthMinuteTimingView {
    diagnose_minute_timing(MinuteTimingInput {
        labels,
        source_rows: labels.len(),
        selected_rows: labels.len(),
        market,
        label_convention: convention,
        reference_as_of: fixed("2027-01-01T00:00:00Z"),
    })
}

fn us(labels: &[&str]) -> LthMinuteTimingView {
    diagnose(labels, TimingMarket::Us, MinuteLabelConvention::BarClose)
}

#[test]
fn spring_and_fall_dst_use_exchange_instant_not_fixed_kst_clock() {
    let view = us(&[
        "2026-03-06T14:31:00Z",
        "2026-03-09T13:31:00Z",
        "2026-10-30T13:31:00Z",
        "2026-11-02T14:31:00Z",
    ]);
    assert_eq!(view.regular_rows, 4);
    assert_eq!(view.market_utc_offsets_minutes, [-300, -240]);
    assert_eq!(
        view.days
            .iter()
            .map(|d| d.utc_offset_minutes)
            .collect::<Vec<_>>(),
        [-300, -240, -240, -300]
    );
    assert!(view
        .days
        .iter()
        .all(|d| d.first_close_market_local.contains("T09:31:00")));
    assert_eq!(view.exchange_timezone, "America/New_York");
}

#[test]
fn repeated_new_york_hour_has_distinct_instants_and_offsets() {
    let view = us(&["2026-11-01T01:30:00-04:00", "2026-11-01T01:30:00-05:00"]);
    assert_eq!(view.duplicate_instants, 0);
    assert_eq!(view.weekend_rows, 2);
    assert_eq!(view.market_utc_offsets_minutes, [-300, -240]);
    assert!(view.first_market_local.unwrap().ends_with("-04:00"));
    assert!(view.last_market_local.unwrap().ends_with("-05:00"));
}

#[test]
fn naive_local_dst_gap_and_daily_labels_are_rejected() {
    let view = us(&["2026-03-08T02:30:00", "20260308", "202602300931", "invalid"]);
    assert_eq!(view.invalid_rows, 4);
    assert_eq!(view.parsed_rows, 0);
    assert!(view.first_market_local.is_none());
}

#[test]
fn compact_kst_midnight_maps_to_previous_new_york_date() {
    let view = us(&["202603100000", "20260310000100"]);
    assert_eq!(view.assumed_kst_rows, 2);
    assert_eq!(view.regular_rows, 2);
    assert_eq!(view.days[0].market_date, "2026-03-09");
    assert_eq!(
        view.first_market_local.as_deref(),
        Some("2026-03-09T11:00:00-04:00")
    );
}

#[test]
fn us_close_labels_observe_first_0931_and_last_1600() {
    let view = us(&[
        "2026-03-09T09:30:00-04:00",
        "2026-03-09T09:31:00-04:00",
        "2026-03-09T16:00:00-04:00",
        "2026-03-09T16:01:00-04:00",
    ]);
    assert_eq!((view.regular_rows, view.off_session_rows), (2, 2));
    assert_eq!(view.days[0].ordinary_window_expected_slots, 390);
    assert_eq!(view.days[0].interior_unobserved_slots, 388);
    assert_eq!(view.days[0].leading_unobserved_window_slots, 0);
    assert_eq!(view.days[0].trailing_unobserved_window_slots, 0);
}

#[test]
fn kr_close_labels_observe_first_0901_and_last_1530() {
    let view = diagnose(
        &[
            "2026-03-09T09:00:00+09:00",
            "2026-03-09T09:01:00+09:00",
            "2026-03-09T15:30:00+09:00",
            "2026-03-09T15:31:00+09:00",
        ],
        TimingMarket::Kr,
        MinuteLabelConvention::BarClose,
    );
    assert_eq!((view.regular_rows, view.off_session_rows), (2, 2));
    assert_eq!(view.market_utc_offsets_minutes, [540]);
    assert_eq!(view.exchange_timezone, "Asia/Seoul");
    assert_eq!(view.days[0].ordinary_window_expected_slots, 390);
}

#[test]
fn bar_start_interpretation_has_one_minute_later_availability() {
    let view = diagnose(
        &[
            "2026-03-09T09:30:00-04:00",
            "2026-03-09T15:59:00-04:00",
            "2026-03-09T16:00:00-04:00",
        ],
        TimingMarket::Us,
        MinuteLabelConvention::BarStart,
    );
    assert_eq!((view.regular_rows, view.off_session_rows), (2, 1));
    assert_eq!(
        view.first_bar_start_utc.as_deref(),
        Some("2026-03-09T13:30:00Z")
    );
    assert_eq!(
        view.first_close_available_at_utc.as_deref(),
        Some("2026-03-09T13:31:00Z")
    );
}

#[test]
fn unknown_convention_only_converts_instants_without_gap_or_session_claims() {
    let view = diagnose(
        &["2026-03-08T02:30:00-05:00", "2026-03-09T13:31:00Z"],
        TimingMarket::Us,
        MinuteLabelConvention::Unknown,
    );
    assert_eq!(view.parsed_rows, 2);
    assert_eq!(
        (
            view.regular_rows,
            view.off_session_rows,
            view.weekend_rows,
            view.unfinished_rows
        ),
        (0, 0, 0, 0)
    );
    assert!(view.first_bar_start_utc.is_none());
    assert!(view.last_close_available_at_utc.is_none());
    assert!(view.first_market_local.is_some());
    assert!(view.days.is_empty());
}

#[test]
fn same_instant_offset_aliases_do_not_inflate_unique_grid() {
    let view = us(&[
        "2026-03-09T09:31:00-04:00",
        "2026-03-09T13:31:00Z",
        "20260309223100",
    ]);
    assert_eq!(view.duplicate_instants, 2);
    assert_eq!(view.regular_rows, 3);
    assert_eq!(view.days[0].regular_unique_bars, 1);
    assert_eq!(view.interior_unobserved_slots, 0);
}

#[test]
fn actual_instant_reverse_order_is_detected_without_reordering_input() {
    let labels = [
        "2026-03-09T13:33:00Z",
        "invalid",
        "2026-03-09T09:31:00-04:00",
        "2026-03-09T13:32:00Z",
    ];
    let view = us(&labels);
    assert_eq!(view.out_of_order_rows, 1);
    assert_eq!(view.invalid_rows, 1);
    assert_eq!(
        view.first_close_available_at_utc.as_deref(),
        Some("2026-03-09T13:31:00Z")
    );
    assert_eq!(view.days[0].regular_unique_bars, 3);
    assert_eq!(labels[0], "2026-03-09T13:33:00Z");
}

#[test]
fn second_and_fraction_labels_are_counted_but_not_inserted_into_minute_grid() {
    let view = us(&[
        "2026-03-09T13:31:00Z",
        "2026-03-09T13:32:01Z",
        "2026-03-09T13:32:00.5Z",
        "2026-03-09T13:33:00Z",
    ]);
    assert_eq!(view.non_minute_aligned_rows, 2);
    assert_eq!(view.parsed_rows, 4);
    assert_eq!(view.days[0].regular_unique_bars, 2);
    assert_eq!(view.interior_unobserved_slots, 1);
}

#[test]
fn fractional_close_after_window_is_off_session() {
    let view = us(&["2026-03-09T16:00:00.000000001-04:00"]);
    assert_eq!(view.off_session_rows, 1);
    assert_eq!(view.regular_rows, 0);
}

#[test]
fn internal_gaps_use_unique_slots_and_separate_window_edges() {
    let view = us(&[
        "2026-03-09T13:32:00Z",
        "2026-03-09T13:35:00Z",
        "2026-03-09T13:35:00Z",
        "2026-03-09T13:37:00Z",
    ]);
    let day = &view.days[0];
    assert_eq!(day.regular_unique_bars, 3);
    assert_eq!(day.interior_unobserved_slots, 3);
    assert_eq!(day.max_interior_gap_slots, 2);
    assert_eq!(day.leading_unobserved_window_slots, 1);
    assert_eq!(day.trailing_unobserved_window_slots, 383);
    assert_eq!(
        day.regular_unique_bars
            + day.interior_unobserved_slots
            + day.leading_unobserved_window_slots
            + day.trailing_unobserved_window_slots,
        390
    );
}

#[test]
fn nights_weekends_and_unobserved_dates_never_become_internal_gaps() {
    let view = us(&[
        "2026-03-06T16:00:00-05:00",
        "2026-03-07T09:31:00-05:00",
        "2026-03-09T09:31:00-04:00",
        "2026-03-09T20:00:00-04:00",
        "2026-03-12T09:31:00-04:00",
    ]);
    assert_eq!(view.observed_days_count, 3);
    assert_eq!(view.interior_unobserved_slots, 0);
    assert_eq!(view.weekend_rows, 1);
    assert_eq!(view.off_session_rows, 1);
}

#[test]
fn holiday_and_early_close_are_ordinary_window_examples_not_calendar_proofs() {
    let view = us(&["2026-12-25T09:31:00-05:00", "2026-11-27T13:00:00-05:00"]);
    assert_eq!(view.regular_rows, 2);
    assert!(!view.calendar_verified && !view.provider_coverage_verified);
    assert!(!view.requested_period_coverage_verified && !view.actual_publication_verified);
    assert_eq!(view.days[0].trailing_unobserved_window_slots, 180);
    assert_eq!(view.interior_unobserved_slots, 0);
    assert_eq!(view.legacy_session_policy, "legacyFixedKstUnverified");
}

#[test]
fn reference_as_of_is_fixed_and_close_equality_is_finished() {
    let labels = ["2026-03-09T13:31:00Z", "2026-03-09T13:32:00Z"];
    let view = diagnose_minute_timing(MinuteTimingInput {
        labels: &labels,
        source_rows: 2,
        selected_rows: 2,
        market: TimingMarket::Us,
        label_convention: MinuteLabelConvention::BarClose,
        reference_as_of: fixed(labels[0]),
    });
    assert_eq!(view.unfinished_rows, 1);
    assert_eq!(view.reference_as_of, "2026-03-09T13:31:00Z");
    let start = diagnose_minute_timing(MinuteTimingInput {
        labels: &labels[..1],
        source_rows: 1,
        selected_rows: 1,
        market: TimingMarket::Us,
        label_convention: MinuteLabelConvention::BarStart,
        reference_as_of: fixed(labels[0]),
    });
    assert_eq!(start.unfinished_rows, 1);
}

#[test]
fn empty_input_is_bounded_and_does_not_claim_coverage() {
    let view = us(&[]);
    assert_eq!(view.processed_rows, 0);
    assert_eq!(view.observed_days_count, 0);
    assert!(!view.input_truncated);
    assert!(view.days.is_empty() && view.examples.is_empty());
    assert!(view.first_market_local.is_none() && view.first_bar_start_utc.is_none());
}

#[test]
fn input_and_example_limits_report_actual_processed_count() {
    let too_long = "한".repeat(40);
    let labels = vec![too_long.as_str(); 600];
    let view = us(&labels);
    assert_eq!(view.used_rows, 600);
    assert_eq!(view.processed_rows, 512);
    assert_eq!(view.invalid_rows, 512);
    assert!(view.input_truncated);
    assert_eq!(view.examples.len(), 6);
    assert!(view.examples.iter().all(|e| e.label.len() <= 64));
    assert_eq!(view.examples[0].label.len(), 63);
}

#[test]
fn day_row_limit_does_not_drop_aggregate_gap_counts() {
    let start = fixed("2026-03-02T14:31:00Z");
    let labels = (0..25)
        .flat_map(|day| {
            let instant = start + Duration::days(day);
            [utc(instant), utc(instant + Duration::minutes(2))]
        })
        .collect::<Vec<_>>();
    let borrowed = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let view = us(&borrowed);
    assert_eq!(view.days.len(), 8);
    assert_eq!(view.observed_days_count, 19);
    assert_eq!(view.omitted_day_rows, 11);
    assert_eq!(view.interior_unobserved_slots, 19);
    assert_eq!(view.max_interior_gap_slots, 1);
}

#[test]
fn normalization_wrapper_preserves_legacy_nanosecond_bytes() {
    let parse = super::super::minute_time::parse_minute_instant;
    for (label, expected) in [
        ("202603092231", "2026-03-09T13:31:00.000000000Z"),
        ("20260309223101", "2026-03-09T13:31:01.000000000Z"),
        (
            "2026-03-09T09:31:00.123-04:00",
            "2026-03-09T13:31:00.123000000Z",
        ),
    ] {
        assert_eq!(
            parse(label)
                .unwrap()
                .to_rfc3339_opts(SecondsFormat::Nanos, true),
            expected
        );
    }
    for label in ["20260309", "2026-03-09T13:31:00", "20260230093100", ""] {
        assert!(parse(label).is_none());
    }
}
