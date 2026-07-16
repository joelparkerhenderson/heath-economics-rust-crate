//! Comprehensive integration tests — group 8.
//!
//! Modules under test:
//! - value_generating_capacity_operational_turnaround
//! - waiting_list_impact
//! - wearable_validation
//! - willingness_to_pay_thresholds
//! - workforce_retention
//! - wsjf_and_cd3
//!
//! Sections:
//! 1. EDGE CASES — None conditions, zero/empty/boundary inputs, extremes.
//! 2. PROPERTIES / INVARIANTS — linearity, monotonicity, bounds, ordering.
//! 3. CROSS-MODULE CONSISTENCY — ICER vs NMB decisions, capacity vs slots.
//! 4. DOMAIN SCENARIOS — hand-computed end-to-end business cases.

use health_economics::value_generating_capacity_operational_turnaround::{
    annual_capacity_value, capacity_value, extra_activity_units,
};
use health_economics::waiting_list_impact::{
    activity_value, extra_slots, hours_released, list_reduction, patients_seen,
    wait_reduction_fraction, waiting_time_gain,
};
use health_economics::wearable_validation::{
    absolute_error_at, annual_false_alert_cost, bland_altman, classify_heart_rate_mape,
    concordance_correlation, data_completeness_percent, mape_percent,
    wear_time_compliance_percent, HeartRateMapeGrade,
};
use health_economics::willingness_to_pay_thresholds::{
    adopt_by_icer, adopt_by_nmb, icer, max_defensible_price, net_monetary_benefit,
};
use health_economics::workforce_retention::{
    annual_turnover_cost, retention_value, software_value, vacancy_cover_cost, CostPerLeaver,
};
use health_economics::wsjf_and_cd3::{
    cd3, sequence_by_cd3, sequencing_savings, total_delay_cost, wsjf, Feature,
};

const TOL: f64 = 1e-9;

fn assert_close(a: f64, b: f64, tol: f64) {
    assert!(
        (a - b).abs() < tol,
        "expected {b}, got {a} (|diff| = {})",
        (a - b).abs()
    );
}

// ===========================================================================
// SECTION 1 — EDGE CASES
// ===========================================================================

// ---- value_generating_capacity_operational_turnaround --------------------

/// Zero staff, zero units/day, or zero working days each collapse extra activity to zero.
#[test]
fn edge_capacity_zero_factor_yields_zero_units() {
    assert_close(extra_activity_units(0.0, 2.0, 250.0), 0.0, TOL);
    assert_close(extra_activity_units(25.0, 0.0, 250.0), 0.0, TOL);
    assert_close(extra_activity_units(25.0, 2.0, 0.0), 0.0, TOL);
}

/// Zero units or zero scheme value gives zero capacity value, end-to-end too.
#[test]
fn edge_capacity_value_zero_inputs() {
    assert_close(capacity_value(0.0, 120.0), 0.0, TOL);
    assert_close(capacity_value(12_500.0, 0.0), 0.0, TOL);
    assert_close(annual_capacity_value(25.0, 2.0, 250.0, 0.0), 0.0, TOL);
}

/// Extreme 1e12-scale inputs stay finite (no NaN/inf) in the capacity chain.
#[test]
fn edge_capacity_extreme_magnitudes_stay_finite() {
    let v = annual_capacity_value(1e12, 2.0, 250.0, 120.0);
    assert!(v.is_finite());
    assert_close(v, 1e12 * 2.0 * 250.0 * 120.0, 1e3);
}

// ---- waiting_list_impact --------------------------------------------------

/// extra_slots returns None exactly when slot_duration_hours is zero.
#[test]
fn edge_extra_slots_none_only_on_zero_slot_duration() {
    assert!(extra_slots(3_750.0, 0.0, 0.85).is_none());
    // Zero hours or zero utilization are defined (they just give zero slots).
    assert_close(extra_slots(0.0, 0.5, 0.85).unwrap(), 0.0, TOL);
    assert_close(extra_slots(3_750.0, 0.5, 0.0).unwrap(), 0.0, TOL);
}

/// Utilization boundaries: 1.0 converts every released hour; 0.0 converts none.
#[test]
fn edge_extra_slots_utilization_boundaries() {
    assert_close(extra_slots(100.0, 0.5, 1.0).unwrap(), 200.0, TOL);
    assert_close(extra_slots(100.0, 0.5, 0.0).unwrap(), 0.0, TOL);
}

/// DNA boundaries: rate 0.0 sees every slot; rate 1.0 sees no patients.
#[test]
fn edge_patients_seen_dna_boundaries() {
    assert_close(patients_seen(6_375.0, 0.0), 6_375.0, TOL);
    assert_close(patients_seen(6_375.0, 1.0), 0.0, TOL);
}

/// Induced demand above patients seen drives net list reduction negative (list grows).
#[test]
fn edge_list_reduction_can_go_negative() {
    let net = list_reduction(1_000.0, 1_500.0);
    assert_close(net, -500.0, TOL);
}

/// waiting_time_gain is None only at zero service rate; zero backlog gives zero gain.
#[test]
fn edge_waiting_time_gain_none_condition() {
    assert!(waiting_time_gain(100.0, 0.0).is_none());
    assert_close(waiting_time_gain(0.0, 24_000.0).unwrap(), 0.0, TOL);
}

/// wait_reduction_fraction is None only at zero annual capacity.
#[test]
fn edge_wait_reduction_fraction_none_condition() {
    assert!(wait_reduction_fraction(100.0, 0.0).is_none());
    assert_close(wait_reduction_fraction(0.0, 24_000.0).unwrap(), 0.0, TOL);
}

/// Zero staff releases zero hours; zero-valued attendance tariff yields zero activity value.
#[test]
fn edge_waiting_list_zero_inputs() {
    assert_close(hours_released(0.0, 0.75, 250.0), 0.0, TOL);
    assert_close(activity_value(5_929.0, 0.0), 0.0, TOL);
}

/// 1e12-scale hours flow through the slots chain without NaN/inf.
#[test]
fn edge_waiting_list_extreme_magnitudes_stay_finite() {
    let slots = extra_slots(1e12, 0.5, 0.85).unwrap();
    assert!(slots.is_finite());
    let seen = patients_seen(slots, 0.07);
    assert!(seen.is_finite());
    assert!(activity_value(seen, 160.0).is_finite());
}

// ---- wearable_validation ----------------------------------------------------

/// mape_percent is None for empty slices, length mismatch, or any zero reference.
#[test]
fn edge_mape_none_conditions() {
    assert!(mape_percent(&[], &[]).is_none());
    assert!(mape_percent(&[1.0, 2.0], &[1.0]).is_none());
    // A zero reference anywhere in the series (not just first) is fatal.
    assert!(mape_percent(&[1.0, 2.0, 3.0], &[1.0, 0.0, 3.0]).is_none());
}

/// A single-element pair is a valid MAPE input; perfect agreement gives 0%.
#[test]
fn edge_mape_single_element_and_perfect_agreement() {
    assert_close(mape_percent(&[100.0], &[80.0]).unwrap(), 25.0, TOL);
    assert_close(mape_percent(&[70.0, 80.0], &[70.0, 80.0]).unwrap(), 0.0, TOL);
}

/// CCC is None for empty slices, length mismatch, and constant series with equal means.
#[test]
fn edge_ccc_none_conditions() {
    assert!(concordance_correlation(&[], &[]).is_none());
    assert!(concordance_correlation(&[1.0], &[1.0, 2.0]).is_none());
    // Both series constant with equal means: denominator 0 → None.
    assert!(concordance_correlation(&[5.0, 5.0], &[5.0, 5.0]).is_none());
}

/// Constant series with *different* means have denom = bias² > 0 and CCC = 0 (no covariance).
#[test]
fn edge_ccc_constant_series_different_means_is_zero() {
    let ccc = concordance_correlation(&[5.0, 5.0], &[7.0, 7.0]).unwrap();
    assert_close(ccc, 0.0, TOL);
}

/// A single unequal pair has zero variances but a bias penalty, so CCC = 0 (defined).
#[test]
fn edge_ccc_single_unequal_pair_is_zero() {
    assert_close(concordance_correlation(&[10.0], &[20.0]).unwrap(), 0.0, TOL);
    // A single equal pair has a zero denominator → None.
    assert!(concordance_correlation(&[10.0], &[10.0]).is_none());
}

/// bland_altman needs n ≥ 2 matched pairs; one pair or mismatched lengths give None.
#[test]
fn edge_bland_altman_none_conditions() {
    assert!(bland_altman(&[1.0], &[1.0]).is_none());
    assert!(bland_altman(&[], &[]).is_none());
    assert!(bland_altman(&[1.0, 2.0, 3.0], &[1.0, 2.0]).is_none());
}

/// Two identical pairs: zero bias and zero-width limits of agreement.
#[test]
fn edge_bland_altman_two_perfect_pairs() {
    let ba = bland_altman(&[60.0, 70.0], &[60.0, 70.0]).unwrap();
    assert_close(ba.mean_bias, 0.0, TOL);
    assert_close(ba.lower_limit, 0.0, TOL);
    assert_close(ba.upper_limit, 0.0, TOL);
}

/// Wear-time compliance is None only at zero protocol time; zero wear is 0%.
#[test]
fn edge_wear_time_compliance_none_and_zero() {
    assert!(wear_time_compliance_percent(16.0, 0.0).is_none());
    assert_close(wear_time_compliance_percent(0.0, 30.0).unwrap(), 0.0, TOL);
}

/// Compliance is not clamped: wearing longer than protocol reports > 100%.
#[test]
fn edge_wear_time_compliance_exceeds_100_uncapped() {
    assert_close(wear_time_compliance_percent(45.0, 30.0).unwrap(), 150.0, TOL);
}

/// Data completeness is None only at zero expected points; zero observed is 0%.
#[test]
fn edge_data_completeness_none_and_zero() {
    assert!(data_completeness_percent(900.0, 0.0).is_none());
    assert_close(data_completeness_percent(0.0, 1_000.0).unwrap(), 0.0, TOL);
}

/// Grade boundaries are inclusive: exactly 5.0% is Strict and exactly 10.0% is Lenient.
#[test]
fn edge_mape_grade_boundary_equality() {
    assert_eq!(classify_heart_rate_mape(5.0), HeartRateMapeGrade::Strict);
    assert_eq!(classify_heart_rate_mape(10.0), HeartRateMapeGrade::Lenient);
    assert_eq!(classify_heart_rate_mape(5.000001), HeartRateMapeGrade::Lenient);
    assert_eq!(classify_heart_rate_mape(10.000001), HeartRateMapeGrade::Fail);
    assert_eq!(classify_heart_rate_mape(0.0), HeartRateMapeGrade::Strict);
}

/// Zero MAPE or zero true value implies zero absolute error.
#[test]
fn edge_absolute_error_zero_inputs() {
    assert_close(absolute_error_at(0.0, 100.0), 0.0, TOL);
    assert_close(absolute_error_at(11.4, 0.0), 0.0, TOL);
}

/// Zero patients, alerts, or cost each zero out the annual false-alert cost.
#[test]
fn edge_false_alert_cost_zero_factors() {
    assert_close(annual_false_alert_cost(0.0, 2.0, 40.0), 0.0, TOL);
    assert_close(annual_false_alert_cost(500.0, 0.0, 40.0), 0.0, TOL);
    assert_close(annual_false_alert_cost(500.0, 2.0, 0.0), 0.0, TOL);
}

/// 1e12-scale wearable readings keep MAPE/CCC/BA finite.
#[test]
fn edge_wearable_extreme_magnitudes_stay_finite() {
    let measured = [1.02e12, 0.97e12, 1.01e12];
    let reference = [1e12, 1e12, 1e12];
    let mape = mape_percent(&measured, &reference).unwrap();
    assert!(mape.is_finite());
    assert_close(mape, 2.0, 1e-6);
    let ccc = concordance_correlation(&measured, &reference).unwrap();
    assert!(ccc.is_finite());
    let ba = bland_altman(&measured, &reference).unwrap();
    assert!(ba.mean_bias.is_finite() && ba.lower_limit.is_finite() && ba.upper_limit.is_finite());
}

// ---- willingness_to_pay_thresholds -----------------------------------------

/// ICER is None exactly at ΔE = 0; adopt_by_icer inherits the same None condition.
#[test]
fn edge_icer_none_at_zero_effect() {
    assert!(icer(800.0, 0.0).is_none());
    assert_eq!(adopt_by_icer(800.0, 0.0, 20_000.0), None);
}

/// Negative ΔE is not rejected: ICER goes negative (a dominated/dominant quadrant artifact).
#[test]
fn edge_icer_negative_effect_gives_negative_ratio() {
    assert_close(icer(800.0, -0.05).unwrap(), -16_000.0, TOL);
    // Cost-saving intervention (negative ΔC, positive ΔE) also yields a negative ICER.
    assert_close(icer(-800.0, 0.05).unwrap(), -16_000.0, TOL);
}

/// Negative NMB signals reject; NMB is defined even with ΔE = 0 (unlike the ICER form).
#[test]
fn edge_nmb_negative_and_defined_at_zero_effect() {
    assert_close(net_monetary_benefit(20_000.0, 0.05, 2_000.0), -1_000.0, TOL);
    assert_close(net_monetary_benefit(20_000.0, 0.0, 800.0), -800.0, TOL);
    assert!(!adopt_by_nmb(20_000.0, 0.0, 800.0));
}

/// Boundary equality: ICER == λ is rejected (strict <), and NMB == 0 is rejected (strict >).
#[test]
fn edge_adopt_boundary_equality_rejects() {
    // ΔC = 1,000, ΔE = 0.05, λ = 20,000 → ICER exactly 20,000 and NMB exactly 0.
    assert_close(icer(1_000.0, 0.05).unwrap(), 20_000.0, TOL);
    assert_eq!(adopt_by_icer(1_000.0, 0.05, 20_000.0), Some(false));
    assert_close(net_monetary_benefit(20_000.0, 0.05, 1_000.0), 0.0, TOL);
    assert!(!adopt_by_nmb(20_000.0, 0.05, 1_000.0));
}

/// Zero QALYs gained caps the defensible price at the offsets alone; zero everything is zero.
#[test]
fn edge_max_price_zero_qalys_is_offsets_only() {
    assert_close(max_defensible_price(20_000.0, 0.0, 300.0), 300.0, TOL);
    assert_close(max_defensible_price(20_000.0, 0.0, 0.0), 0.0, TOL);
}

/// 1e12-scale thresholds stay finite through NMB and pricing.
#[test]
fn edge_wtp_extreme_magnitudes_stay_finite() {
    let nmb = net_monetary_benefit(1e12, 0.05, 800.0);
    assert!(nmb.is_finite());
    assert!(max_defensible_price(1e12, 0.05, 0.0).is_finite());
}

// ---- workforce_retention ----------------------------------------------------

/// An all-zero leaver costs nothing; each component contributes additively to total().
#[test]
fn edge_cost_per_leaver_zero_and_single_component() {
    let zero = CostPerLeaver { recruitment: 0.0, onboarding_ramp: 0.0, vacancy_cover: 0.0 };
    assert_close(zero.total(), 0.0, TOL);
    let only_recruitment =
        CostPerLeaver { recruitment: 4_500.0, onboarding_ramp: 0.0, vacancy_cover: 0.0 };
    assert_close(only_recruitment.total(), 4_500.0, TOL);
}

/// Zero vacancy months or zero WTE coverage means zero vacancy cover cost.
#[test]
fn edge_vacancy_cover_zero_factors() {
    assert_close(vacancy_cover_cost(3_333.0, 0.0, 0.6), 0.0, TOL);
    assert_close(vacancy_cover_cost(3_333.0, 4.0, 0.0), 0.0, TOL);
    // Full-WTE boundary: fraction 1.0 charges the whole premium.
    assert_close(vacancy_cover_cost(3_000.0, 4.0, 1.0), 12_000.0, TOL);
}

/// Turnover-rate boundaries: 0.0 costs nothing; 1.0 replaces the whole workforce.
#[test]
fn edge_turnover_rate_boundaries() {
    assert_close(annual_turnover_cost(1_200.0, 0.0, 18_500.0), 0.0, TOL);
    assert_close(annual_turnover_cost(1_200.0, 1.0, 18_500.0), 1_200.0 * 18_500.0, TOL);
}

/// Zero claimed turnover reduction is worth zero; software_value then rests on sickness only.
#[test]
fn edge_retention_value_zero_reduction() {
    assert_close(retention_value(1_200.0, 0.0, 18_500.0), 0.0, TOL);
    assert_close(software_value(1_200.0, 0.0, 18_500.0, 100.0, 250.0), 25_000.0, TOL);
}

/// 1e12-scale headcounts keep the turnover math finite.
#[test]
fn edge_workforce_extreme_magnitudes_stay_finite() {
    let cost = annual_turnover_cost(1e12, 0.11, 18_500.0);
    assert!(cost.is_finite());
    assert!(software_value(1e12, 0.01, 18_500.0, 1e12, 250.0).is_finite());
}

// ---- wsjf_and_cd3 -------------------------------------------------------------

/// CD3 is None exactly at zero duration — via both the free function and the method.
#[test]
fn edge_cd3_none_at_zero_duration() {
    assert!(cd3(10_000.0, 0.0).is_none());
    let f = Feature { cost_of_delay_per_week: 10_000.0, duration_weeks: 0.0 };
    assert!(f.cd3().is_none());
}

/// WSJF is None exactly at zero job size; an all-zero numerator is a valid zero score.
#[test]
fn edge_wsjf_none_and_zero_numerator() {
    assert!(wsjf(8.0, 5.0, 3.0, 0.0).is_none());
    assert_close(wsjf(0.0, 0.0, 0.0, 5.0).unwrap(), 0.0, TOL);
}

/// Empty and single-item schedules accrue zero delay cost (nothing waits).
#[test]
fn edge_total_delay_cost_empty_and_single() {
    assert_close(total_delay_cost(&[]), 0.0, TOL);
    let solo = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
    assert_close(total_delay_cost(&[solo]), 0.0, TOL);
}

/// Sequencing an empty or single-item backlog is a no-op with zero savings.
#[test]
fn edge_sequence_empty_and_single() {
    assert!(sequence_by_cd3(&[]).is_empty());
    let solo = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
    assert_eq!(sequence_by_cd3(&[solo]), vec![solo]);
    assert_close(sequencing_savings(&[solo]), 0.0, TOL);
}

/// Zero-duration items (undefined CD3) are scheduled first — they consume no capacity.
#[test]
fn edge_zero_duration_feature_sorts_first() {
    let instant = Feature { cost_of_delay_per_week: 1.0, duration_weeks: 0.0 };
    let big = Feature { cost_of_delay_per_week: 100_000.0, duration_weeks: 1.0 };
    let ordered = sequence_by_cd3(&[big, instant]);
    assert_eq!(ordered[0], instant);
    // Running the instant item first adds no delay cost to the big item.
    assert_close(total_delay_cost(&ordered), 0.0, TOL);
}

/// 1e12-scale costs of delay stay finite through scoring and scheduling.
#[test]
fn edge_wsjf_cd3_extreme_magnitudes_stay_finite() {
    assert!(cd3(1e12, 2.0).unwrap().is_finite());
    let f1 = Feature { cost_of_delay_per_week: 1e12, duration_weeks: 3.0 };
    let f2 = Feature { cost_of_delay_per_week: 1e11, duration_weeks: 1.0 };
    assert!(total_delay_cost(&[f1, f2]).is_finite());
    assert!(sequencing_savings(&[f1, f2]).is_finite());
}

// ===========================================================================
// SECTION 2 — PROPERTIES / INVARIANTS
// ===========================================================================

/// Linearity: doubling staff (or units/day, or days) doubles capacity units and value.
#[test]
fn prop_capacity_value_is_linear_in_each_factor() {
    for staff in [1.0, 5.0, 25.0, 400.0] {
        for units in [0.5, 1.0, 2.0, 4.0] {
            for days in [200.0, 250.0] {
                let base = annual_capacity_value(staff, units, days, 120.0);
                assert_close(annual_capacity_value(2.0 * staff, units, days, 120.0), 2.0 * base, 1e-6);
                assert_close(annual_capacity_value(staff, 2.0 * units, days, 120.0), 2.0 * base, 1e-6);
                assert_close(annual_capacity_value(staff, units, 2.0 * days, 120.0), 2.0 * base, 1e-6);
            }
        }
    }
}

/// Consistency: annual_capacity_value equals the manual two-step composition on a grid.
#[test]
fn prop_annual_capacity_value_matches_composition() {
    for staff in [1.0, 10.0, 25.0] {
        for value in [80.0, 120.0, 160.0] {
            let units = extra_activity_units(staff, 2.0, 250.0);
            assert_close(
                annual_capacity_value(staff, 2.0, 250.0, value),
                capacity_value(units, value),
                TOL,
            );
        }
    }
}

/// Monotonicity: a higher DNA rate never increases patients seen (grid over rates).
#[test]
fn prop_higher_dna_rate_sees_fewer_patients() {
    let rates = [0.0, 0.05, 0.07, 0.1, 0.2, 0.5, 0.9, 1.0];
    for slots in [100.0, 6_375.0, 50_000.0] {
        for pair in rates.windows(2) {
            let more = patients_seen(slots, pair[0]);
            let fewer = patients_seen(slots, pair[1]);
            assert!(fewer <= more + TOL, "DNA {} should see <= DNA {}", pair[1], pair[0]);
        }
    }
}

/// Monotonicity: extra slots increase with utilization and decrease with slot length.
#[test]
fn prop_extra_slots_monotone_in_utilization_and_duration() {
    for hours in [500.0, 3_750.0] {
        let mut prev = -1.0;
        for util in [0.0, 0.25, 0.5, 0.85, 1.0] {
            let slots = extra_slots(hours, 0.5, util).unwrap();
            assert!(slots >= prev);
            prev = slots;
        }
        // Longer slots from the same hours → fewer slots.
        let short = extra_slots(hours, 0.25, 0.85).unwrap();
        let long = extra_slots(hours, 1.0, 0.85).unwrap();
        assert!(long < short);
    }
}

/// Monotonicity: a faster service rate shrinks the waiting-time gain from the same ΔN.
#[test]
fn prop_waiting_time_gain_decreases_with_service_rate() {
    let mut prev = f64::INFINITY;
    for mu in [1_000.0, 6_000.0, 24_000.0, 100_000.0] {
        let gain = waiting_time_gain(5_929.0, mu).unwrap();
        assert!(gain < prev);
        prev = gain;
    }
}

/// Linearity: hours_released and activity_value both scale linearly with their inputs.
#[test]
fn prop_waiting_list_scaling_linearity() {
    for staff in [5.0, 20.0, 80.0] {
        let base = hours_released(staff, 0.75, 250.0);
        assert_close(hours_released(2.0 * staff, 0.75, 250.0), 2.0 * base, 1e-6);
    }
    for seen in [100.0, 5_929.0] {
        assert_close(activity_value(2.0 * seen, 160.0), 2.0 * activity_value(seen, 160.0), 1e-6);
    }
}

/// Bounds: MAPE is always ≥ 0 for any series over positive references.
#[test]
fn prop_mape_is_nonnegative() {
    let refs = [60.0, 75.0, 90.0, 110.0];
    for shift in [-20.0, -5.0, 0.0, 5.0, 20.0] {
        let measured: Vec<f64> = refs.iter().map(|r| r + shift).collect();
        let mape = mape_percent(&measured, &refs).unwrap();
        assert!(mape >= 0.0, "MAPE {mape} < 0 at shift {shift}");
    }
}

/// Bounds: Lin's CCC lies in [-1, 1] across varied bias/scale/inverted series.
#[test]
fn prop_ccc_bounded_in_minus_one_to_one() {
    let reference = [60.0, 70.0, 80.0, 90.0, 100.0];
    // A deterministic LCG supplies varied but reproducible perturbations.
    let mut seed: u64 = 42;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as f64 / (u32::MAX as f64)) * 20.0 - 10.0
    };
    for bias in [-30.0, -5.0, 0.0, 5.0, 30.0] {
        for scale in [-2.0, -1.0, 0.5, 1.0, 3.0] {
            let measured: Vec<f64> =
                reference.iter().map(|r| scale * r + bias + next()).collect();
            let ccc = concordance_correlation(&measured, &reference).unwrap();
            assert!(
                (-1.0 - 1e-12..=1.0 + 1e-12).contains(&ccc),
                "CCC {ccc} out of [-1,1] at bias {bias}, scale {scale}"
            );
        }
    }
}

/// CCC = 1 for identity data and any added bias strictly lowers it (monotone penalty).
#[test]
fn prop_ccc_bias_penalty_monotone() {
    let reference = [60.0, 70.0, 80.0, 90.0, 100.0];
    let mut prev = concordance_correlation(&reference, &reference).unwrap();
    assert_close(prev, 1.0, TOL);
    for bias in [1.0, 5.0, 10.0, 25.0, 50.0] {
        let measured: Vec<f64> = reference.iter().map(|r| r + bias).collect();
        let ccc = concordance_correlation(&measured, &reference).unwrap();
        assert!(ccc < prev, "bias {bias} should lower CCC below {prev}, got {ccc}");
        prev = ccc;
    }
}

/// Bounds: Bland–Altman limits always bracket the mean bias symmetrically.
#[test]
fn prop_bland_altman_limits_bracket_bias() {
    let reference = [58.0, 63.0, 77.0, 84.0, 96.0, 105.0];
    for bias in [-10.0, 0.0, 4.0, 15.0] {
        for wobble in [0.0, 1.0, 3.0, 8.0] {
            let measured: Vec<f64> = reference
                .iter()
                .enumerate()
                .map(|(i, r)| r + bias + if i % 2 == 0 { wobble } else { -wobble })
                .collect();
            let ba = bland_altman(&measured, &reference).unwrap();
            assert!(ba.lower_limit <= ba.mean_bias + TOL);
            assert!(ba.upper_limit >= ba.mean_bias - TOL);
            // Symmetry: bias is the midpoint of the limits.
            assert_close((ba.lower_limit + ba.upper_limit) / 2.0, ba.mean_bias, 1e-9);
        }
    }
}

/// Bounds: compliance and completeness percentages stay in [0, 100] for valid inputs.
#[test]
fn prop_percent_gates_in_0_to_100_for_valid_inputs() {
    for worn in [0.0, 4.0, 16.0, 29.0, 30.0] {
        let pct = wear_time_compliance_percent(worn, 30.0).unwrap();
        assert!((0.0..=100.0).contains(&pct), "compliance {pct} out of range");
    }
    for observed in [0.0, 250.0, 900.0, 1_000.0] {
        let pct = data_completeness_percent(observed, 1_000.0).unwrap();
        assert!((0.0..=100.0).contains(&pct), "completeness {pct} out of range");
    }
}

/// Monotonicity: more time worn never lowers the wear-time compliance metric.
#[test]
fn prop_compliance_monotone_in_time_worn() {
    let mut prev = -1.0;
    for worn in [0.0, 5.0, 10.0, 16.0, 22.0, 30.0] {
        let pct = wear_time_compliance_percent(worn, 30.0).unwrap();
        assert!(pct >= prev);
        prev = pct;
    }
}

/// Consistency: MAPE grades never improve as MAPE rises across the whole scale.
#[test]
fn prop_mape_grade_ordering_is_monotone() {
    let rank = |g: HeartRateMapeGrade| match g {
        HeartRateMapeGrade::Strict => 0,
        HeartRateMapeGrade::Lenient => 1,
        HeartRateMapeGrade::Fail => 2,
    };
    let mut prev = 0;
    for mape in [0.0, 1.67, 3.8, 5.0, 5.5, 6.9, 10.0, 11.4, 50.0] {
        let r = rank(classify_heart_rate_mape(mape));
        assert!(r >= prev, "grade worsened out of order at MAPE {mape}");
        prev = r;
    }
}

/// Linearity: absolute_error_at scales linearly in both MAPE and the true value.
#[test]
fn prop_absolute_error_linear() {
    for mape in [1.0, 5.0, 11.4] {
        for value in [40.0, 100.0, 180.0] {
            let base = absolute_error_at(mape, value);
            assert_close(absolute_error_at(2.0 * mape, value), 2.0 * base, 1e-9);
            assert_close(absolute_error_at(mape, 2.0 * value), 2.0 * base, 1e-9);
        }
    }
}

/// Linearity: the 52-week false-alert cost doubles when any single factor doubles.
#[test]
fn prop_false_alert_cost_linear_in_each_factor() {
    for patients in [50.0, 500.0] {
        for alerts in [0.5, 2.0] {
            let base = annual_false_alert_cost(patients, alerts, 40.0);
            assert_close(annual_false_alert_cost(2.0 * patients, alerts, 40.0), 2.0 * base, 1e-6);
            assert_close(annual_false_alert_cost(patients, 2.0 * alerts, 40.0), 2.0 * base, 1e-6);
            assert_close(annual_false_alert_cost(patients, alerts, 80.0), 2.0 * base, 1e-6);
        }
    }
}

/// Linearity: NMB and max defensible price scale linearly in the threshold λ (at ΔC = 0).
#[test]
fn prop_wtp_values_linear_in_threshold() {
    for lambda in [4_000.0, 13_000.0, 20_000.0, 150_000.0] {
        for qalys in [0.01, 0.05, 0.5] {
            assert_close(
                net_monetary_benefit(2.0 * lambda, qalys, 0.0),
                2.0 * net_monetary_benefit(lambda, qalys, 0.0),
                1e-6,
            );
            assert_close(
                max_defensible_price(2.0 * lambda, qalys, 0.0),
                2.0 * max_defensible_price(lambda, qalys, 0.0),
                1e-6,
            );
        }
    }
}

/// Monotonicity: raising λ never flips an adopt decision back to reject.
#[test]
fn prop_adoption_monotone_in_threshold() {
    let lambdas = [4_000.0, 13_000.0, 16_000.0, 20_000.0, 30_000.0, 150_000.0];
    for dc in [200.0, 800.0, 5_000.0] {
        for de in [0.01, 0.05, 0.4] {
            let mut adopted = false;
            for lambda in lambdas {
                let now = adopt_by_nmb(lambda, de, dc);
                assert!(!adopted || now, "adoption flipped off as λ rose to {lambda}");
                adopted = now;
            }
        }
    }
}

/// Monotonicity: turnover cost rises with churn cost per leaver and with the rate.
#[test]
fn prop_turnover_cost_monotone_in_cost_and_rate() {
    let mut prev = -1.0;
    for cost in [5_000.0, 12_000.0, 18_500.0, 40_000.0] {
        let total = annual_turnover_cost(1_200.0, 0.11, cost);
        assert!(total > prev);
        prev = total;
    }
    let mut prev = -1.0;
    for rate in [0.0, 0.05, 0.11, 0.25, 1.0] {
        let total = annual_turnover_cost(1_200.0, rate, 18_500.0);
        assert!(total >= prev);
        prev = total;
    }
}

/// Consistency: retention_value equals the turnover-cost delta of the rate improvement.
#[test]
fn prop_retention_value_equals_turnover_cost_delta() {
    for headcount in [200.0, 1_200.0] {
        for rate in [0.08, 0.11, 0.2] {
            for delta in [0.005, 0.01, 0.03] {
                let saving = retention_value(headcount, delta, 18_500.0);
                let before = annual_turnover_cost(headcount, rate, 18_500.0);
                let after = annual_turnover_cost(headcount, rate - delta, 18_500.0);
                assert_close(saving, before - after, 1e-6);
            }
        }
    }
}

/// Consistency: software_value decomposes exactly into retention + sickness terms.
#[test]
fn prop_software_value_decomposition() {
    for days in [0.0, 50.0, 100.0, 400.0] {
        for cover in [0.0, 250.0, 600.0] {
            let total = software_value(1_200.0, 0.01, 18_500.0, days, cover);
            let parts = retention_value(1_200.0, 0.01, 18_500.0) + days * cover;
            assert_close(total, parts, 1e-9);
        }
    }
}

/// Consistency: Feature::cd3() always agrees with the free cd3() function.
#[test]
fn prop_feature_cd3_agrees_with_free_function() {
    for cod in [0.0, 5_000.0, 12_000.0, 30_000.0, 1e9] {
        for dur in [0.0, 0.5, 1.0, 2.0, 10.0] {
            let f = Feature { cost_of_delay_per_week: cod, duration_weeks: dur };
            assert_eq!(f.cd3(), cd3(cod, dur));
        }
    }
}

/// Ordering: sequence_by_cd3 output is non-increasing in CD3 for a grid of backlogs.
#[test]
fn prop_sequence_by_cd3_is_sorted_descending() {
    let backlog = [
        Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 },
        Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 },
        Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 },
        Feature { cost_of_delay_per_week: 7_000.0, duration_weeks: 3.5 },
        Feature { cost_of_delay_per_week: 1_000.0, duration_weeks: 0.25 },
    ];
    let ordered = sequence_by_cd3(&backlog);
    assert_eq!(ordered.len(), backlog.len());
    for pair in ordered.windows(2) {
        let k0 = pair[0].cd3().unwrap_or(f64::INFINITY);
        let k1 = pair[1].cd3().unwrap_or(f64::INFINITY);
        assert!(k0 >= k1, "sequence not descending: {k0} before {k1}");
    }
}

/// Optimality: the CD3 order beats or ties ALL 6 permutations of a 3-feature set.
#[test]
fn prop_cd3_order_is_brute_force_optimal_over_all_permutations() {
    let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
    let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
    let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
    let perms: [[Feature; 3]; 6] =
        [[a, b, c], [a, c, b], [b, a, c], [b, c, a], [c, a, b], [c, b, a]];
    let cd3_cost = total_delay_cost(&sequence_by_cd3(&[a, b, c]));
    for perm in &perms {
        let cost = total_delay_cost(perm);
        assert!(
            cd3_cost <= cost + TOL,
            "CD3 order cost {cd3_cost} exceeds permutation cost {cost}"
        );
        // Savings vs any starting order are never negative.
        assert!(sequencing_savings(perm) >= -TOL);
    }
    // And the best permutation IS the CD3 cost.
    let best = perms.iter().map(|p| total_delay_cost(p)).fold(f64::INFINITY, f64::min);
    assert_close(cd3_cost, best, TOL);
}

/// Optimality holds on a second, irregular 3-feature set (guards against coincidence).
#[test]
fn prop_cd3_optimality_on_second_feature_set() {
    let x = Feature { cost_of_delay_per_week: 9_000.0, duration_weeks: 4.0 };
    let y = Feature { cost_of_delay_per_week: 2_500.0, duration_weeks: 0.5 };
    let z = Feature { cost_of_delay_per_week: 15_000.0, duration_weeks: 6.0 };
    let perms: [[Feature; 3]; 6] =
        [[x, y, z], [x, z, y], [y, x, z], [y, z, x], [z, x, y], [z, y, x]];
    let cd3_cost = total_delay_cost(&sequence_by_cd3(&[x, y, z]));
    for perm in &perms {
        assert!(cd3_cost <= total_delay_cost(perm) + TOL);
    }
}

/// Consistency: sequencing_savings is exactly given-order cost minus CD3-order cost.
#[test]
fn prop_sequencing_savings_identity() {
    let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
    let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
    let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
    for order in [[a, b, c], [c, a, b], [b, c, a]] {
        let expected = total_delay_cost(&order) - total_delay_cost(&sequence_by_cd3(&order));
        assert_close(sequencing_savings(&order), expected, TOL);
    }
    // A backlog already in CD3 order saves exactly zero.
    assert_close(sequencing_savings(&[b, c, a]), 0.0, TOL);
}

/// Monotonicity: a higher WSJF numerator raises the score; a bigger job lowers it.
#[test]
fn prop_wsjf_monotone_in_value_and_size() {
    let mut prev = -1.0;
    for value in [1.0, 3.0, 8.0, 20.0] {
        let score = wsjf(value, 5.0, 3.0, 5.0).unwrap();
        assert!(score > prev);
        prev = score;
    }
    let mut prev = f64::INFINITY;
    for size in [1.0, 2.0, 5.0, 13.0] {
        let score = wsjf(8.0, 5.0, 3.0, size).unwrap();
        assert!(score < prev);
        prev = score;
    }
}

// ===========================================================================
// SECTION 3 — CROSS-MODULE CONSISTENCY
// ===========================================================================

/// adopt_by_icer and adopt_by_nmb agree for every grid point with ΔE > 0 (incl. ICER == λ).
#[test]
fn cross_icer_and_nmb_decisions_agree_for_positive_effect() {
    let costs = [-500.0, 0.0, 100.0, 800.0, 1_000.0, 2_000.0, 50_000.0];
    let effects = [0.01, 0.05, 0.25, 1.0];
    let lambdas = [4_000.0, 13_000.0, 20_000.0, 30_000.0, 150_000.0];
    for &dc in &costs {
        for &de in &effects {
            for &lambda in &lambdas {
                let by_icer = adopt_by_icer(dc, de, lambda);
                let by_nmb = adopt_by_nmb(lambda, de, dc);
                assert_eq!(
                    by_icer,
                    Some(by_nmb),
                    "disagreement at ΔC={dc}, ΔE={de}, λ={lambda}"
                );
            }
        }
    }
}

/// Boundary agreement: at ICER == λ both rules reject (ICER uses strict <, NMB strict > 0).
#[test]
fn cross_icer_nmb_boundary_equality_both_reject() {
    for &(de, lambda) in &[(0.05, 20_000.0), (0.5, 13_000.0), (1.0, 150_000.0)] {
        let dc = lambda * de; // Constructs ICER exactly equal to λ.
        assert_close(icer(dc, de).unwrap(), lambda, 1e-6);
        assert_eq!(adopt_by_icer(dc, de, lambda), Some(false));
        assert!(!adopt_by_nmb(lambda, de, dc));
    }
}

/// Known divergence: with ΔE < 0 the ICER sign flips and the two rules disagree —
/// locking down why the equivalence is only claimed for ΔE > 0.
#[test]
fn cross_icer_nmb_diverge_for_negative_effect() {
    // Costs more AND harms: NMB correctly rejects, naive ICER rule "adopts".
    let dc = 100.0;
    let de = -0.05;
    let lambda = 20_000.0;
    assert_eq!(adopt_by_icer(dc, de, lambda), Some(true)); // −2,000 < 20,000
    assert!(!adopt_by_nmb(lambda, de, dc)); // −1,000 − 100 < 0
}

/// ICER inverts NMB: NMB is zero exactly when ΔC = ICER × ΔE at λ = ICER.
#[test]
fn cross_icer_recovers_cost_and_zeroes_nmb() {
    for &(dc, de) in &[(800.0, 0.05), (2_600.0, 0.2), (45.0, 0.003)] {
        let r = icer(dc, de).unwrap();
        assert_close(r * de, dc, 1e-9);
        assert_close(net_monetary_benefit(r, de, dc), 0.0, 1e-9);
    }
}

/// max_defensible_price is the break-even ΔC: pricing at it makes NMB exactly zero.
#[test]
fn cross_max_price_is_nmb_break_even() {
    for &lambda in &[4_000.0, 20_000.0, 150_000.0] {
        for &qalys in &[0.01, 0.05, 0.3] {
            let price = max_defensible_price(lambda, qalys, 0.0);
            assert_close(net_monetary_benefit(lambda, qalys, price), 0.0, 1e-9);
            // Strict rules therefore reject AT the max price and adopt just under it.
            assert!(!adopt_by_nmb(lambda, qalys, price));
            assert!(adopt_by_nmb(lambda, qalys, price - 0.01));
        }
    }
}

/// Capacity-module units agree with waiting-list slots when hours convert losslessly:
/// staff × (h/day ÷ slot-length) × days == extra_slots(hours_released, slot, util=1).
#[test]
fn cross_capacity_units_agree_with_waiting_list_slots() {
    for &(staff, hours_per_day, days) in &[(20.0, 1.0, 250.0), (8.0, 0.75, 220.0)] {
        let slot_len = 0.5;
        let hours = hours_released(staff, hours_per_day, days);
        let slots = extra_slots(hours, slot_len, 1.0).unwrap();
        let units = extra_activity_units(staff, hours_per_day / slot_len, days);
        assert_close(slots, units, 1e-6);
        // And valuing them at the same tariff yields the same £ figure
        // (waiting-list side with zero DNA so every slot is a patient seen).
        assert_close(
            activity_value(patients_seen(slots, 0.0), 160.0),
            capacity_value(units, 160.0),
            1e-6,
        );
    }
}

/// A MAPE computed from a constant-relative-error series converts back to the
/// exact per-reading absolute error via absolute_error_at.
#[test]
fn cross_mape_roundtrips_through_absolute_error() {
    let reference = [80.0, 95.0, 110.0];
    for rel in [0.02, 0.055, 0.114] {
        let measured: Vec<f64> = reference.iter().map(|r| r * (1.0 + rel)).collect();
        let mape = mape_percent(&measured, &reference).unwrap();
        assert_close(mape, rel * 100.0, 1e-9);
        for r in reference {
            assert_close(absolute_error_at(mape, r), rel * r, 1e-9);
        }
    }
}

/// Health-denominated cost of delay: CoD/week = λ × QALYs-at-stake/week
/// (via max_defensible_price) feeds CD3 sequencing coherently.
#[test]
fn cross_wtp_denominated_cost_of_delay_drives_cd3() {
    let lambda = 20_000.0;
    // Feature X: 0.5 QALYs/week at stake → CoD £10,000/wk, 5 weeks on the constraint.
    let x = Feature {
        cost_of_delay_per_week: max_defensible_price(lambda, 0.5, 0.0),
        duration_weeks: 5.0,
    };
    // Feature Y: 0.1 QALYs/week + £1,000/wk operational → CoD £3,000/wk, 1 week.
    let y = Feature {
        cost_of_delay_per_week: max_defensible_price(lambda, 0.1, 1_000.0),
        duration_weeks: 1.0,
    };
    assert_close(x.cost_of_delay_per_week, 10_000.0, TOL);
    assert_close(y.cost_of_delay_per_week, 3_000.0, TOL);
    // CD3: X = 2,000; Y = 3,000 → Y first despite X's larger raw CoD.
    assert_eq!(sequence_by_cd3(&[x, y]), vec![y, x]);
    // Cost given order (X,Y): Y waits 5 → £15,000. CD3 order: X waits 1 → £10,000.
    assert_close(total_delay_cost(&[x, y]), 15_000.0, TOL);
    assert_close(total_delay_cost(&[y, x]), 10_000.0, TOL);
    assert_close(sequencing_savings(&[x, y]), 5_000.0, TOL);
}

// ===========================================================================
// SECTION 4 — DOMAIN SCENARIOS (hand-computed end-to-end)
// ===========================================================================

/// Full waiting-list initiative: 30 nurses × 0.5 h/day × 240 days through slots,
/// DNA, induced demand, queueing gain, and tariff value — all hand-computed.
#[test]
fn scenario_waiting_list_initiative_end_to_end() {
    // Hours released: 30 × 0.5 × 240 = 3,600 h/year.
    let hours = hours_released(30.0, 0.5, 240.0);
    assert_close(hours, 3_600.0, TOL);

    // 20-minute slots (1/3 h) at 90% usable: 3,600 / (1/3) × 0.9 = 9,720 slots.
    let slots = extra_slots(hours, 1.0 / 3.0, 0.9).unwrap();
    assert_close(slots, 9_720.0, 1e-6);

    // 10% DNA: 9,720 × 0.9 = 8,748 patients actually seen.
    let seen = patients_seen(slots, 0.10);
    assert_close(seen, 8_748.0, 1e-6);

    // Visible capacity induces 748 extra referrals: net list cut = 8,000.
    let net = list_reduction(seen, 748.0);
    assert_close(net, 8_000.0, 1e-6);

    // Service rate 20,000 patients/year: everyone pulled forward 0.4 years.
    let gain = waiting_time_gain(net, 20_000.0).unwrap();
    assert_close(gain, 0.4, 1e-9);

    // Against 36,000 appointments/year of capacity: waits cut by 8,748/36,000 = 24.3%.
    let fraction = wait_reduction_fraction(seen, 36_000.0).unwrap();
    assert_close(fraction, 0.243, 1e-9);

    // Tariff framing (presented second, non-cash): 8,748 × £150 = £1,312,200/year.
    let value = activity_value(seen, 150.0);
    assert_close(value, 1_312_200.0, 1e-6);
}

/// Wearable validation verdict from a raw paired series: MAPE grade, CCC,
/// Bland–Altman, operational gates, and the rival's false-alert bill.
#[test]
fn scenario_wearable_validation_verdict_from_raw_series() {
    // Candidate device reads a constant +2 bpm high against ECG.
    let reference = [60.0, 70.0, 80.0, 90.0, 100.0];
    let measured = [62.0, 72.0, 82.0, 92.0, 102.0];

    // MAPE = mean(2/60, 2/70, 2/80, 2/90, 2/100) × 100 ≈ 2.5825%.
    let mape = mape_percent(&measured, &reference).unwrap();
    let expected_mape = (2.0 / 60.0 + 2.0 / 70.0 + 2.0 / 80.0 + 2.0 / 90.0 + 2.0 / 100.0)
        / 5.0
        * 100.0;
    assert_close(mape, expected_mape, 1e-12);
    assert!((mape - 2.582_539_682_539_682_6).abs() < 1e-9);

    // 2.58% ≤ 5% → clinical-grade strict pass.
    assert_eq!(classify_heart_rate_mape(mape), HeartRateMapeGrade::Strict);

    // CCC: var 200 each, cov 200, bias² 4 → 2×200 / (200+200+4) = 0.990099….
    let ccc = concordance_correlation(&measured, &reference).unwrap();
    assert_close(ccc, 400.0 / 404.0, 1e-12);

    // Bland–Altman on a pure constant bias: bias 2, zero-width limits.
    let ba = bland_altman(&measured, &reference).unwrap();
    assert_close(ba.mean_bias, 2.0, TOL);
    assert_close(ba.lower_limit, 2.0, TOL);
    assert_close(ba.upper_limit, 2.0, TOL);

    // At the 100-bpm alert threshold the implied error is a tolerable ~2.6 bpm.
    assert!(absolute_error_at(mape, 100.0) < 3.0);

    // Operational gates: 20/30 days worn = 66.67% (clears the 16-in-30 RPM gate),
    // 950/1,000 points = 95% completeness.
    let compliance = wear_time_compliance_percent(20.0, 30.0).unwrap();
    assert_close(compliance, 200.0 / 3.0, 1e-9);
    assert!(compliance > wear_time_compliance_percent(16.0, 30.0).unwrap() - 1e-9);
    assert_close(data_completeness_percent(950.0, 1_000.0).unwrap(), 95.0, TOL);

    // The rejected rival's inaccuracy bill: 300 patients × 1.5 extra false
    // alerts/week × £40 × 52 = £936,000/year avoided by this verdict.
    assert_close(annual_false_alert_cost(300.0, 1.5, 40.0), 936_000.0, TOL);
}

/// Workforce retention business case: leaver costing → baseline churn burn →
/// modest 1-point claim → full software-value line, hand-computed throughout.
#[test]
fn scenario_workforce_retention_business_case() {
    // Vacancy cover: £3,000/month premium × 5 months × 0.5 WTE covered = £7,500.
    let cover = vacancy_cover_cost(3_000.0, 5.0, 0.5);
    assert_close(cover, 7_500.0, TOL);

    // Cost per leaver: £5,000 recruitment + £7,000 onboarding + £7,500 cover = £19,500.
    let leaver = CostPerLeaver {
        recruitment: 5_000.0,
        onboarding_ramp: 7_000.0,
        vacancy_cover: cover,
    };
    assert_close(leaver.total(), 19_500.0, TOL);

    // Baseline churn burn: 800 nurses × 12% × £19,500 = £1,872,000/year.
    let baseline = annual_turnover_cost(800.0, 0.12, leaver.total());
    assert_close(baseline, 1_872_000.0, TOL);

    // Modest 1-point claim: 800 × 0.01 × £19,500 = £156,000/year cash-relevant.
    let retention = retention_value(800.0, 0.01, leaver.total());
    assert_close(retention, 156_000.0, TOL);

    // Full line with 200 sickness days avoided at £220/day cover: +£44,000 → £200,000.
    let total = software_value(800.0, 0.01, leaver.total(), 200.0, 220.0);
    assert_close(total, 200_000.0, TOL);

    // Sanity: the claimed value is a small fraction (~8.3%) of the baseline burn.
    assert!(retention / baseline < 0.1);
}

/// Portfolio sequencing scenario: a three-item healthcare backlog where CD3
/// re-ordering saves hand-computed delay cost and WSJF agrees on the leader.
#[test]
fn scenario_backlog_sequencing_saves_delay_cost() {
    // P: platform migration — CoD £9,000/wk, 4 weeks on the constrained team.
    let p = Feature { cost_of_delay_per_week: 9_000.0, duration_weeks: 4.0 };
    // Q: quick compliance fix — CoD £2,500/wk, 0.5 weeks.
    let q = Feature { cost_of_delay_per_week: 2_500.0, duration_weeks: 0.5 };
    // R: reporting feature — CoD £15,000/wk, 6 weeks.
    let r = Feature { cost_of_delay_per_week: 15_000.0, duration_weeks: 6.0 };

    // CD3 scores: P = 2,250; Q = 5,000; R = 2,500 → order Q, R, P.
    assert_close(p.cd3().unwrap(), 2_250.0, TOL);
    assert_close(q.cd3().unwrap(), 5_000.0, TOL);
    assert_close(r.cd3().unwrap(), 2_500.0, TOL);
    assert_eq!(sequence_by_cd3(&[p, q, r]), vec![q, r, p]);

    // Planned order (P,Q,R): Q waits 4, R waits 4.5 → 2.5k×4 + 15k×4.5 = £77,500.
    assert_close(total_delay_cost(&[p, q, r]), 77_500.0, TOL);
    // CD3 order (Q,R,P): R waits 0.5, P waits 6.5 → 15k×0.5 + 9k×6.5 = £66,000.
    assert_close(total_delay_cost(&[q, r, p]), 66_000.0, TOL);
    // Sequencing alone banks £11,500 of delay cost.
    assert_close(sequencing_savings(&[p, q, r]), 11_500.0, TOL);

    // A quick WSJF pass ranks the compliance fix first too: (3+13+5)/1 = 21
    // beats (8+5+8)/8 = 2.625 and (13+3+5)/13 ≈ 1.615.
    let wsjf_q = wsjf(3.0, 13.0, 5.0, 1.0).unwrap();
    let wsjf_p = wsjf(8.0, 5.0, 8.0, 8.0).unwrap();
    let wsjf_r = wsjf(13.0, 3.0, 5.0, 13.0).unwrap();
    assert_close(wsjf_q, 21.0, TOL);
    assert!(wsjf_q > wsjf_r && wsjf_q > wsjf_p);
}
