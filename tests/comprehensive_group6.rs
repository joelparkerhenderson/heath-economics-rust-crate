//! Comprehensive integration tests, group 6.
//!
//! Modules covered:
//! - opportunity_cost
//! - patient_reported_outcomes
//! - practitioner_time
//! - prevention_economics
//! - probabilistic_sensitivity_analysis
//! - qaly_shortfall_and_severity_modifiers
//! - quality_adjusted_life_year
//! - reach_and_equity
//! - readmission_rate
//! - referral_to_treatment
//!
//! Sections:
//! 1. EDGE CASES
//! 2. PROPERTIES / INVARIANTS
//! 3. CROSS-MODULE CONSISTENCY
//! 4. DOMAIN SCENARIOS

use health_economics::net_monetary_benefit as nmb;
use health_economics::opportunity_cost as oc;
use health_economics::patient_reported_outcomes as pro;
use health_economics::practitioner_time as pt;
use health_economics::prevention_economics as prev;
use health_economics::probabilistic_sensitivity_analysis as psa;
use health_economics::qaly_shortfall_and_severity_modifiers as sev;
use health_economics::quality_adjusted_life_year as qaly;
use health_economics::quality_adjusted_life_year::HealthState;
use health_economics::reach_and_equity as re;
use health_economics::readmission_rate as rr;
use health_economics::referral_to_treatment as rtt;

const TOL: f64 = 1e-9;

// ============================================================================
// 1. EDGE CASES
// ============================================================================

// Locks down: every Option-returning function returns None exactly on its documented undefined input.
#[test]
fn edge_exact_none_conditions_of_every_option_function() {
    // opportunity_cost: empty slice of alternatives is undefined, not zero.
    assert!(oc::opportunity_cost(&[]).is_none());
    // qalys_displaced: zero marginal cost per QALY is undefined.
    assert!(oc::qalys_displaced(1_000.0, 0.0).is_none());
    // number_needed_to_treat: ARR exactly zero is undefined.
    assert!(pro::number_needed_to_treat(0.0).is_none());
    // extra_appointments_per_day: zero-length appointment is undefined.
    assert!(pt::extra_appointments_per_day(60.0, 0.0).is_none());
    // cost_per_qaly: zero QALYs gained is undefined.
    assert!(prev::cost_per_qaly(1_000.0, 0.0).is_none());
    // proportional_shortfall: zero general-population QALYs is undefined.
    assert!(sev::proportional_shortfall(1.0, 0.0).is_none());
    // effective_icer: zero severity weight is undefined.
    assert!(sev::effective_icer(26_000.0, 0.0).is_none());
    // reach: zero eligible population is undefined.
    assert!(re::reach(10.0, 0.0).is_none());
    // impact_ratio: zero bottom-group impact is undefined.
    assert!(re::impact_ratio(0.0044, 0.0).is_none());
    // readmission_rate_percent: zero index discharges is undefined.
    assert!(rr::readmission_rate_percent(10.0, 0.0).is_none());
    // observed_vs_expected: zero expected count is undefined.
    assert!(rr::observed_vs_expected(10.0, 0.0).is_none());
    // rtt_performance_percent: nobody treated is undefined.
    assert!(rtt::rtt_performance_percent(0.0, 0.0).is_none());
    // longest_stage: empty pathway has no longest stage.
    assert!(rtt::longest_stage(&[]).is_none());
    // PSA summary statistics: empty draw sets are undefined.
    assert!(psa::mean(&[]).is_none());
    assert!(psa::probability_positive(&[]).is_none());
    assert!(psa::percentile(&[], 50.0).is_none());
    assert!(psa::ceac(&[]).is_none());
}

// Locks down: the None conditions are exact — the smallest nonzero denominators flip every function back to Some.
#[test]
fn edge_none_conditions_are_exact_not_approximate() {
    assert!(oc::qalys_displaced(1_000.0, 1e-300).is_some());
    assert!(pro::number_needed_to_treat(1e-300).is_some());
    assert!(pt::extra_appointments_per_day(60.0, 1e-300).is_some());
    assert!(prev::cost_per_qaly(1_000.0, 1e-300).is_some());
    assert!(sev::proportional_shortfall(1.0, 1e-300).is_some());
    assert!(sev::effective_icer(26_000.0, 1e-300).is_some());
    assert!(re::reach(10.0, 1e-300).is_some());
    assert!(re::impact_ratio(0.0044, 1e-300).is_some());
    assert!(rr::readmission_rate_percent(10.0, 1e-300).is_some());
    assert!(rr::observed_vs_expected(10.0, 1e-300).is_some());
    assert!(rtt::rtt_performance_percent(1.0, 1e-300).is_some());
    // Negative denominators are NOT None — only exact zero is undefined.
    assert!(oc::qalys_displaced(1_000.0, -13_000.0).is_some());
    assert!(pro::number_needed_to_treat(-0.1).is_some());
}

// Locks down: gamma_mean_sd is None exactly when mean <= 0 or sd <= 0 (both zero and negative).
#[test]
fn edge_gamma_mean_sd_none_condition() {
    let mut rng = psa::Lcg::new(11);
    assert!(rng.gamma_mean_sd(0.0, 200_000.0).is_none());
    assert!(rng.gamma_mean_sd(800_000.0, 0.0).is_none());
    assert!(rng.gamma_mean_sd(-1.0, 200_000.0).is_none());
    assert!(rng.gamma_mean_sd(800_000.0, -1.0).is_none());
    assert!(rng.gamma_mean_sd(0.0, 0.0).is_none());
    // Strictly positive parameters are defined and yield a positive draw.
    assert!(rng.gamma_mean_sd(800_000.0, 200_000.0).unwrap() > 0.0);
}

// Locks down: simulate_migration_net_benefits is None on non-positive Gamma cost
// parameters when at least one draw is requested — but n = 0 returns Some(empty)
// even with invalid parameters, because the sampler is never invoked (as coded).
#[test]
fn edge_simulate_migration_none_condition_and_n_zero() {
    let bad = psa::MigrationCase {
        cost_mean: 0.0,
        cost_sd: 200_000.0,
        benefit_mean: 350_000.0,
        benefit_sd: 150_000.0,
        duration_low: 3.0,
        duration_high: 6.0,
    };
    assert!(psa::simulate_migration_net_benefits(&bad, 1, 42).is_none());
    // n = 0 short-circuits: Some(vec![]) even with the invalid case.
    assert_eq!(psa::simulate_migration_net_benefits(&bad, 0, 42), Some(vec![]));
    let good = psa::MigrationCase {
        cost_mean: 800_000.0,
        cost_sd: 200_000.0,
        benefit_mean: 350_000.0,
        benefit_sd: 150_000.0,
        duration_low: 3.0,
        duration_high: 6.0,
    };
    assert_eq!(psa::simulate_migration_net_benefits(&good, 0, 42), Some(vec![]));
}

// Locks down: ceac is None for no options, for zero-length draws, and for mismatched draw counts.
#[test]
fn edge_ceac_none_conditions() {
    // No options at all.
    assert!(psa::ceac(&[]).is_none());
    // Options present but zero draws.
    let empty: [f64; 0] = [];
    assert!(psa::ceac(&[&empty, &empty]).is_none());
    // Mismatched draw counts across options.
    let a = [1.0, 2.0, 3.0];
    let b = [1.0, 2.0];
    assert!(psa::ceac(&[&a, &b]).is_none());
}

// Locks down: empty-slice behavior of the sum-style functions — empty means 0, not None.
#[test]
fn edge_empty_slices_that_sum_to_zero() {
    // An empty instrument has a sum score of 0.
    assert_eq!(pro::instrument_sum_score(&[]), 0.0);
    // An empty health-state stream is 0 QALYs.
    assert_eq!(qaly::qalys(&[]), 0.0);
    // An empty pathway has total duration 0 (but no longest stage — see above).
    assert_eq!(rtt::total_pathway_duration(&[]), 0.0);
}

// Locks down: opportunity_cost of a single alternative is that alternative; negatives are legal values.
#[test]
fn edge_opportunity_cost_single_and_negative_alternatives() {
    assert_eq!(oc::opportunity_cost(&[300_000.0]), Some(300_000.0));
    // All-negative alternatives: the "best" forgone value is the least bad one.
    assert_eq!(oc::opportunity_cost(&[-50.0, -10.0, -99.0]), Some(-10.0));
}

// Locks down: utility boundary values — 0 (dead), 1 (perfect health), and negative (worse than death).
#[test]
fn edge_utility_boundaries_in_qaly_math() {
    // Utility 0 contributes nothing regardless of duration.
    assert!((qaly::qalys(&[HealthState { duration_years: 10.0, utility: 0.0 }]) - 0.0).abs() < TOL);
    // Utility 1 makes QALYs equal calendar years.
    assert!((qaly::qalys(&[HealthState { duration_years: 10.0, utility: 1.0 }]) - 10.0).abs() < TOL);
    // A worse-than-death state subtracts QALYs.
    let q = qaly::qalys(&[
        HealthState { duration_years: 1.0, utility: -0.1 },
        HealthState { duration_years: 1.0, utility: 0.5 },
    ]);
    assert!((q - 0.4).abs() < TOL);
    // Waiting in a worse-than-death state makes the delay loss exceed the treated utility alone.
    let loss = qaly::qaly_loss_from_delay(1.0, -0.2, 0.8);
    assert!((loss - 1.0).abs() < TOL);
    // Same negative-utility handling in the RTT per-patient waiting cost.
    let w = rtt::waiting_health_cost_qalys(1.0, 0.8, -0.2);
    assert!((w - 1.0).abs() < TOL);
}

// Locks down: probability boundary values 0 and 1 flow through the responder math sensibly.
#[test]
fn edge_probability_boundaries_in_responder_math() {
    // Perfect separation: ARR = 1 → NNT = 1 (everyone treated responds, nobody untreated does).
    let arr = pro::absolute_risk_reduction(1.0, 0.0);
    assert!((arr - 1.0).abs() < TOL);
    assert!((pro::number_needed_to_treat(arr).unwrap() - 1.0).abs() < TOL);
    // Inverted arms: ARR = −1 → NNT = −1 (a number needed to harm).
    let harm = pro::absolute_risk_reduction(0.0, 1.0);
    assert!((pro::number_needed_to_treat(harm).unwrap() - (-1.0)).abs() < TOL);
    // Zero reach or full reach bracket population impact.
    assert_eq!(re::population_impact(0.0, 0.02), 0.0);
    assert!((re::population_impact(1.0, 0.02) - 0.02).abs() < TOL);
    // Readmission rates of 0% and 100%.
    assert_eq!(rr::readmission_rate_percent(0.0, 2_000.0), Some(0.0));
    assert_eq!(rr::readmission_rate_percent(2_000.0, 2_000.0), Some(100.0));
}

// Locks down: the severity-weight band edges are >= cutoffs on BOTH measures (12/18 absolute, 0.85/0.95 proportional).
#[test]
fn edge_severity_weight_band_boundaries() {
    // Absolute-shortfall edges: 12 is in the ×1.2 band, 18 is in the ×1.7 band (>=, not >).
    assert_eq!(sev::severity_weight(11.999_999, 0.0), 1.0);
    assert_eq!(sev::severity_weight(12.0, 0.0), 1.2);
    assert_eq!(sev::severity_weight(17.999_999, 0.0), 1.2);
    assert_eq!(sev::severity_weight(18.0, 0.0), 1.7);
    // Proportional-shortfall edges: 0.85 → ×1.2 and 0.95 → ×1.7 (>=, not >).
    assert_eq!(sev::severity_weight(0.0, 0.849_999_9), 1.0);
    assert_eq!(sev::severity_weight(0.0, 0.85), 1.2);
    assert_eq!(sev::severity_weight(0.0, 0.949_999_9), 1.2);
    assert_eq!(sev::severity_weight(0.0, 0.95), 1.7);
    // Whichever measure gives the higher weight applies.
    assert_eq!(sev::severity_weight(18.0, 0.10), 1.7);
    assert_eq!(sev::severity_weight(1.0, 0.95), 1.7);
    assert_eq!(sev::severity_weight(12.0, 0.10), 1.2);
    assert_eq!(sev::severity_weight(1.0, 0.85), 1.2);
}

// Locks down: clears_mcid uses |difference| >= mcid, so exactly the MCID clears, in either direction.
#[test]
fn edge_clears_mcid_boundary_at_exactly_the_mcid() {
    assert!(pro::clears_mcid(pro::PHQ9_MCID, pro::PHQ9_MCID)); // exactly 5 clears
    assert!(pro::clears_mcid(-pro::PHQ9_MCID, pro::PHQ9_MCID)); // sign is ignored
    assert!(!pro::clears_mcid(4.999_999, pro::PHQ9_MCID)); // just under does not
    assert!(pro::clears_mcid(-4.0, pro::GAD7_MCID)); // GAD-7 MCID is 4: exactly 4 clears
    assert!(!pro::clears_mcid(3.999_999, pro::GAD7_MCID));
}

// Locks down: meets_rtt_standard is >= 92.0, so exactly 92% meets and just below fails.
#[test]
fn edge_rtt_standard_boundary_at_exactly_92_percent() {
    assert!(rtt::meets_rtt_standard(rtt::RTT_STANDARD_PERCENT));
    assert!(rtt::meets_rtt_standard(92.0));
    assert!(!rtt::meets_rtt_standard(91.999_999));
    // 4,600 of 5,000 is exactly 92%.
    let p = rtt::rtt_performance_percent(4_600.0, 5_000.0).unwrap();
    assert!(rtt::meets_rtt_standard(p));
}

// Locks down: prevention boolean gates are strict inequalities — exactly zero net cost is not
// cost-saving, and an ICER exactly at the threshold is not cost-effective.
#[test]
fn edge_prevention_strict_inequality_boundaries() {
    assert!(!prev::is_cost_saving(0.0)); // exactly zero: not a saving
    assert!(prev::is_cost_saving(-1e-9));
    assert!(!prev::is_cost_effective(20_000.0, 20_000.0)); // strict <
    assert!(prev::is_cost_effective(19_999.999, 20_000.0));
    // Per-person condition is also strict: equality fails.
    assert!(!prev::per_person_cost_saving_condition(180.0, 0.004, 45_000.0, 1.0));
    assert!(prev::per_person_cost_saving_condition(179.999, 0.004, 45_000.0, 1.0));
}

// Locks down: probability_positive counts strictly positive draws only — exact zeros do not count.
#[test]
fn edge_probability_positive_excludes_exact_zeros() {
    assert_eq!(psa::probability_positive(&[0.0, 1.0]), Some(0.5));
    assert_eq!(psa::probability_positive(&[0.0, 0.0]), Some(0.0));
    assert_eq!(psa::probability_positive(&[-1.0, 1e-300]), Some(0.5));
}

// Locks down: percentile clamps p outside [0, 100] and handles a single-draw set.
#[test]
fn edge_percentile_clamping_and_single_draw() {
    let d = [30.0, 10.0, 50.0, 20.0, 40.0];
    assert_eq!(psa::percentile(&d, -25.0), psa::percentile(&d, 0.0));
    assert_eq!(psa::percentile(&d, 150.0), psa::percentile(&d, 100.0));
    assert_eq!(psa::percentile(&d, 0.0), Some(10.0));
    assert_eq!(psa::percentile(&d, 100.0), Some(50.0));
    // One draw: every percentile is that draw.
    assert_eq!(psa::percentile(&[7.5], 0.0), Some(7.5));
    assert_eq!(psa::percentile(&[7.5], 63.0), Some(7.5));
}

// Locks down: ceac ties credit the earliest-listed option, and a solo option always wins.
#[test]
fn edge_ceac_ties_and_single_option() {
    let a = [1.0, 1.0];
    let b = [1.0, 1.0];
    assert_eq!(psa::ceac(&[&a, &b]), Some(vec![1.0, 0.0])); // strict > → ties to first
    let solo = [-5.0, 3.0, 0.0];
    assert_eq!(psa::ceac(&[&solo]), Some(vec![1.0]));
}

// Locks down: extreme magnitudes stay finite (no NaN/inf) through the linear formulas.
#[test]
fn edge_extreme_magnitudes_stay_finite() {
    assert!(oc::bed_day_savings_value(1e9, 1e5).is_finite());
    assert!(oc::net_gain(1e15, -1e15).is_finite());
    assert!(qaly::monetized_value(1e12, 1e6).is_finite());
    assert!(pro::monetized_value(1e12, 1e6).is_finite());
    assert!(rtt::monetized_value(1e12, 1e6).is_finite());
    assert!(pt::bottleneck_basis_value(1e9, 1e6).is_finite());
    assert!(prev::program_cost(1e9, 1e3, 8.3).is_finite());
    assert!(rr::value_of_avoidance(1e9, 1e6, 1e6).is_finite());
    // Deep discounting underflows toward zero but never goes negative, NaN, or infinite.
    let df = qaly::discount_factor(0.035, 1_000.0);
    assert!(df.is_finite() && df > 0.0 && df < 1e-14);
    // opportunity_cost over huge values still returns the maximum, finitely.
    assert_eq!(oc::opportunity_cost(&[f64::MAX, 1.0]), Some(f64::MAX));
}

// Locks down: zero-input degenerate cases produce zeros, not errors, across the multiply-style functions.
#[test]
fn edge_zero_inputs_produce_zero_values() {
    assert_eq!(pt::daily_minutes_saved(0.0, 30.0), 0.0);
    assert_eq!(pt::annual_hours_saved(0.0, 220.0), 0.0);
    assert_eq!(pt::wage_basis_value(0.0, 80.0), 0.0);
    assert_eq!(pt::annual_extra_appointments(0.0, 220.0), 0.0);
    assert_eq!(pt::output_basis_value(0.0, 42.0), 0.0);
    assert_eq!(prev::downstream_offsets(0.0, 45_000.0), 0.0);
    assert_eq!(prev::qalys_gained(0.0, 3.0), 0.0);
    assert_eq!(pro::extra_responders(0.0, 0.26), 0.0);
    assert_eq!(pro::cohort_qalys(0.0, 0.03), 0.0);
    assert_eq!(pro::qalys_from_utility_gain(0.0, 0.5), 0.0);
    assert_eq!(qaly::population_qalys(0.0, 400.0), 0.0);
    assert_eq!(rr::avoided_readmissions(0.0, 0.18, 0.14), 0.0);
    assert_eq!(rr::program_cost(0.0, 60.0), 0.0);
    assert_eq!(re::equity_weighted_qalys(0.0, 1.5), 0.0);
    assert_eq!(rtt::qaly_gain_from_wait_reduction(0.0, 5.0, 0.80, 0.68), 0.0);
}

// Locks down: an instrument scored all-zero sums to zero, and the max PHQ-9 profile sums to 27.
#[test]
fn edge_instrument_sum_score_extremes() {
    assert_eq!(pro::instrument_sum_score(&[0.0; 9]), 0.0);
    assert_eq!(pro::instrument_sum_score(&[3.0; 9]), 27.0);
}

// ============================================================================
// 2. PROPERTIES / INVARIANTS
// ============================================================================

// ---- PSA determinism and sampler ranges ----

// Locks down: the same seed reproduces identical draws; different seeds produce different draws.
#[test]
fn prop_psa_determinism_same_seed_identical_different_seed_differs() {
    let case = psa::MigrationCase {
        cost_mean: 800_000.0,
        cost_sd: 200_000.0,
        benefit_mean: 350_000.0,
        benefit_sd: 150_000.0,
        duration_low: 3.0,
        duration_high: 6.0,
    };
    for seed in [0u64, 1, 42, 999, u64::MAX] {
        let a = psa::simulate_migration_net_benefits(&case, 200, seed).unwrap();
        let b = psa::simulate_migration_net_benefits(&case, 200, seed).unwrap();
        assert_eq!(a, b, "same seed {seed} must reproduce the run exactly");
    }
    let a = psa::simulate_migration_net_benefits(&case, 200, 1).unwrap();
    let b = psa::simulate_migration_net_benefits(&case, 200, 2).unwrap();
    assert_ne!(a, b, "different seeds must not reproduce the same run");
    // The raw generator is deterministic too.
    let mut r1 = psa::Lcg::new(7);
    let mut r2 = psa::Lcg::new(7);
    for _ in 0..100 {
        assert_eq!(r1.next_uniform(), r2.next_uniform());
    }
}

// Locks down: Lcg::uniform(a, b) always lands in [a, b), including negative and tight ranges.
#[test]
fn prop_lcg_uniform_always_in_half_open_range() {
    let ranges = [(0.0, 1.0), (3.0, 6.0), (-10.0, -2.0), (-1.0, 1.0), (5.0, 5.000001)];
    for (seed, &(lo, hi)) in ranges.iter().enumerate() {
        let mut rng = psa::Lcg::new(seed as u64 + 100);
        for _ in 0..2_000 {
            let u = rng.uniform(lo, hi);
            assert!(u >= lo && u < hi, "uniform({lo}, {hi}) produced {u}");
        }
    }
    // next_uniform itself is in [0, 1).
    let mut rng = psa::Lcg::new(3);
    for _ in 0..2_000 {
        let u = rng.next_uniform();
        assert!((0.0..1.0).contains(&u));
    }
}

// Locks down: normal(m, s) has sample mean ≈ m over many draws (wide Monte Carlo tolerance).
#[test]
fn prop_lcg_normal_sample_mean_matches_parameter() {
    let n = 20_000;
    for (seed, &(m, s)) in [(0.0, 1.0), (350_000.0, 150_000.0), (-5.0, 2.0)].iter().enumerate() {
        let mut rng = psa::Lcg::new(seed as u64 + 50);
        let sample_mean: f64 = (0..n).map(|_| rng.normal(m, s)).sum::<f64>() / n as f64;
        // Tolerance: ~7 standard errors — wide enough to be robust, tight enough to be meaningful.
        let tol = 7.0 * s / (n as f64).sqrt() + 1e-12;
        assert!(
            (sample_mean - m).abs() < tol,
            "normal({m}, {s}) sample mean {sample_mean} not within {tol}"
        );
    }
}

// Locks down: raw gamma(shape, scale) has mean ≈ shape × scale and stays positive, including shape < 1.
#[test]
fn prop_lcg_gamma_mean_and_positivity() {
    let n = 20_000;
    let mut rng = psa::Lcg::new(4);
    let m: f64 = (0..n).map(|_| rng.gamma(16.0, 50_000.0)).sum::<f64>() / n as f64;
    assert!((m - 800_000.0).abs() < 20_000.0);
    // Shape < 1 exercises the boost identity branch; draws must remain positive and finite.
    let mut rng = psa::Lcg::new(5);
    for _ in 0..2_000 {
        let g = rng.gamma(0.5, 2.0);
        assert!(g > 0.0 && g.is_finite());
    }
}

// Locks down: gamma_mean_sd reproduces its own mean/sd parameterization within Monte Carlo error.
#[test]
fn prop_gamma_mean_sd_reproduces_parameterization() {
    let n = 20_000;
    let mut rng = psa::Lcg::new(6);
    let (mut sum, mut sum_sq) = (0.0, 0.0);
    for _ in 0..n {
        let g = rng.gamma_mean_sd(800_000.0, 200_000.0).unwrap();
        assert!(g > 0.0);
        sum += g;
        sum_sq += g * g;
    }
    let m = sum / n as f64;
    let sd = (sum_sq / n as f64 - m * m).sqrt();
    assert!((m - 800_000.0).abs() < 10_000.0);
    assert!((sd - 200_000.0).abs() < 10_000.0);
}

// Locks down: percentile is monotone — p0 ≤ p50 ≤ p100 — on every draw set tried.
#[test]
fn prop_percentile_monotonicity() {
    let sets: [&[f64]; 4] = [
        &[10.0, 20.0, 30.0, 40.0, 50.0],
        &[-5.0, 3.0, 3.0, 100.0],
        &[42.0],
        &[0.0, 0.0, 0.0],
    ];
    for d in sets {
        let p0 = psa::percentile(d, 0.0).unwrap();
        let p50 = psa::percentile(d, 50.0).unwrap();
        let p100 = psa::percentile(d, 100.0).unwrap();
        assert!(p0 <= p50 && p50 <= p100, "percentiles not monotone on {d:?}");
    }
    // Also holds on a simulated set.
    let case = psa::MigrationCase {
        cost_mean: 800_000.0,
        cost_sd: 200_000.0,
        benefit_mean: 350_000.0,
        benefit_sd: 150_000.0,
        duration_low: 3.0,
        duration_high: 6.0,
    };
    let d = psa::simulate_migration_net_benefits(&case, 1_000, 9).unwrap();
    let p0 = psa::percentile(&d, 0.0).unwrap();
    let p50 = psa::percentile(&d, 50.0).unwrap();
    let p100 = psa::percentile(&d, 100.0).unwrap();
    assert!(p0 <= p50 && p50 <= p100);
}

// Locks down: ceac entries always sum to 1 and each lies in [0, 1], across a grid of thresholds.
#[test]
fn prop_ceac_entries_sum_to_one_and_lie_in_unit_interval() {
    // Three options with deterministic effect/cost draws.
    let n = 500;
    let mut rng = psa::Lcg::new(21);
    let mut effects = vec![vec![0.0; n]; 3];
    let mut costs = vec![vec![0.0; n]; 3];
    for i in 0..n {
        for j in 0..3 {
            effects[j][i] = rng.uniform(0.0, 3.0);
            costs[j][i] = rng.uniform(0.0, 60_000.0);
        }
    }
    for lambda in [0.0, 10_000.0, 20_000.0, 30_000.0, 100_000.0] {
        let nmb_draws: Vec<Vec<f64>> = (0..3)
            .map(|j| {
                (0..n)
                    .map(|i| psa::net_monetary_benefit(lambda, effects[j][i], costs[j][i]))
                    .collect()
            })
            .collect();
        let refs: Vec<&[f64]> = nmb_draws.iter().map(|v| v.as_slice()).collect();
        let c = psa::ceac(&refs).unwrap();
        assert_eq!(c.len(), 3);
        assert!((c.iter().sum::<f64>() - 1.0).abs() < TOL, "ceac at λ={lambda} must sum to 1");
        for &p in &c {
            assert!((0.0..=1.0).contains(&p), "ceac entry {p} outside [0,1] at λ={lambda}");
        }
    }
}

// ---- QALY invariants ----

// Locks down: qalys is additive — the QALYs of concatenated state lists equal the sum of the parts.
#[test]
fn prop_qalys_additive_over_concatenated_state_lists() {
    let grid = [(0.5, 0.6), (1.0, 0.85), (2.0, 1.0), (0.25, -0.1), (3.0, 0.0)];
    for &(d1, u1) in &grid {
        for &(d2, u2) in &grid {
            let part_a = qaly::qalys(&[HealthState { duration_years: d1, utility: u1 }]);
            let part_b = qaly::qalys(&[HealthState { duration_years: d2, utility: u2 }]);
            let whole = qaly::qalys(&[
                HealthState { duration_years: d1, utility: u1 },
                HealthState { duration_years: d2, utility: u2 },
            ]);
            assert!((whole - (part_a + part_b)).abs() < TOL);
        }
    }
}

// Locks down: qaly_gain is antisymmetric — gain(a, b) == −gain(b, a) across a grid.
#[test]
fn prop_qaly_gain_antisymmetric() {
    let vals = [0.0, 0.125, 0.725, 0.85, 50.0, -1.5];
    for &a in &vals {
        for &b in &vals {
            assert!((qaly::qaly_gain(a, b) + qaly::qaly_gain(b, a)).abs() < TOL);
        }
    }
}

// Locks down: monetized_value is linear in both arguments (scaling and additivity).
#[test]
fn prop_monetized_value_linear_in_both_args() {
    let qs = [0.125, 7.8, 50.0, 0.0];
    let ts = [13_000.0, 20_000.0, 30_000.0];
    for &q in &qs {
        for &t in &ts {
            let base = qaly::monetized_value(q, t);
            assert!((qaly::monetized_value(2.0 * q, t) - 2.0 * base).abs() < TOL);
            assert!((qaly::monetized_value(q, 2.0 * t) - 2.0 * base).abs() < TOL);
            assert!(
                (qaly::monetized_value(q + 1.0, t) - (base + qaly::monetized_value(1.0, t))).abs()
                    < 1e-6
            );
        }
    }
}

// Locks down: discount_factor(r, 0) == 1 for any rate, and the factor strictly decreases with year for r > 0.
#[test]
fn prop_discount_factor_starts_at_one_and_decreases() {
    for &r in &[0.015, 0.035, 0.05, 0.10] {
        assert!((qaly::discount_factor(r, 0.0) - 1.0).abs() < TOL);
        let mut prev = qaly::discount_factor(r, 0.0);
        for year in 1..=30 {
            let df = qaly::discount_factor(r, year as f64);
            assert!(df < prev, "discount factor must strictly decrease (r={r}, year={year})");
            assert!(df > 0.0);
            prev = df;
        }
    }
    // Zero rate never discounts.
    assert!((qaly::discount_factor(0.0, 25.0) - 1.0).abs() < TOL);
}

// ---- Severity-modifier invariants ----

// Locks down: severity_weight returns exactly 1.0, 1.2, or 1.7 and nothing else across a dense sweep.
#[test]
fn prop_severity_weight_only_three_values_across_sweep() {
    let mut a = 0.0;
    while a <= 30.0 {
        let mut p = 0.0;
        while p <= 1.2 {
            let w = sev::severity_weight(a, p);
            assert!(
                w == 1.0 || w == 1.2 || w == 1.7,
                "unexpected weight {w} at absolute={a}, proportional={p}"
            );
            p += 0.01;
        }
        a += 0.25;
    }
}

// Locks down: effective_icer is exactly icer/weight, and effective_threshold exactly multiplies —
// and the two views agree: effective_icer < threshold iff icer < effective_threshold.
#[test]
fn prop_effective_icer_and_threshold_are_dual_views() {
    let icers = [2_300.0, 21_700.0, 26_000.0, 51_000.0, 60_000.0];
    let thresholds = [20_000.0, 30_000.0];
    for &w in &[1.0, 1.2, 1.7] {
        for &icer in &icers {
            let e = sev::effective_icer(icer, w).unwrap();
            assert!((e - icer / w).abs() < TOL);
            for &t in &thresholds {
                let et = sev::effective_threshold(t, w);
                assert!((et - t * w).abs() < TOL);
                // Dividing the ICER or multiplying the threshold is the same decision.
                assert_eq!(e < t, icer < et, "dual views disagree at icer={icer}, w={w}, t={t}");
            }
        }
    }
}

// Locks down: shortfall pipeline consistency — proportional_shortfall(absolute_shortfall(g, c), g)
// equals 1 − c/g over a grid of populations.
#[test]
fn prop_shortfall_pipeline_consistency() {
    let grid = [(14.2, 2.1), (21.0, 2.0), (10.0, 9.0), (14.2, 0.0)];
    for &(g, c) in &grid {
        let a = sev::absolute_shortfall(g, c);
        assert!((a - (g - c)).abs() < TOL);
        let p = sev::proportional_shortfall(a, g).unwrap();
        assert!((p - (1.0 - c / g)).abs() < TOL);
    }
}

// ---- PRO invariants ----

// Locks down: NNT is exactly 1/ARR across a grid of nonzero ARRs.
#[test]
fn prop_nnt_is_reciprocal_of_arr() {
    for &arr in &[0.01, 0.05, 0.26, 0.5, 1.0, -0.25] {
        let nnt = pro::number_needed_to_treat(arr).unwrap();
        assert!((nnt - 1.0 / arr).abs() < TOL);
        // Round trip: 1/NNT recovers the ARR.
        assert!((1.0 / nnt - arr).abs() < TOL);
    }
}

// Locks down: distribution_based_mcid(2 × sd) == sd, i.e. the heuristic is exactly half the SD.
#[test]
fn prop_distribution_based_mcid_is_half_sd() {
    for &sd in &[0.0, 1.0, 4.0, 8.0, 13.7] {
        assert!((pro::distribution_based_mcid(2.0 * sd) - sd).abs() < TOL);
        assert!((pro::distribution_based_mcid(sd) - 0.5 * sd).abs() < TOL);
    }
}

// Locks down: adjusted_difference and absolute_risk_reduction are both plain differences (antisymmetric).
#[test]
fn prop_pro_differences_are_antisymmetric() {
    let pairs = [(-6.2, -2.1), (0.48, 0.22), (0.0, 0.0), (1.0, -1.0)];
    for &(t, c) in &pairs {
        assert!((pro::adjusted_difference(t, c) + pro::adjusted_difference(c, t)).abs() < TOL);
        assert!(
            (pro::absolute_risk_reduction(t, c) + pro::absolute_risk_reduction(c, t)).abs() < TOL
        );
    }
}

// Locks down: the PRO economic bridge is multiplicative all the way —
// cohort_qalys(extra_responders(n, arr), qalys_from_utility_gain(u, y)) == n·arr·u·y.
#[test]
fn prop_pro_economic_bridge_is_multiplicative() {
    for &(n, arr, u, y) in &[(1_000.0, 0.26, 0.06, 0.5), (500.0, 0.10, 0.03, 1.0)] {
        let r = pro::extra_responders(n, arr);
        let q_each = pro::qalys_from_utility_gain(u, y);
        let total = pro::cohort_qalys(r, q_each);
        assert!((total - n * arr * u * y).abs() < TOL);
        // Monetization is the same linear map as in the QALY module.
        assert!((pro::monetized_value(total, 20_000.0) - total * 20_000.0).abs() < 1e-6);
    }
}

// ---- Practitioner-time invariants ----

// Locks down: on the worked example, the three valuation levels are ordered wage ≤ output ≤ bottleneck.
#[test]
fn prop_practitioner_valuation_levels_ordered_on_worked_example() {
    let daily = pt::daily_minutes_saved(2.0, 30.0); // 60 min/day
    let hours = pt::annual_hours_saved(daily, 220.0); // 220 h/yr
    let wage = pt::wage_basis_value(hours, 80.0); // £17,600
    let per_day = pt::extra_appointments_per_day(daily, 12.0).unwrap(); // 5/day
    let per_year = pt::annual_extra_appointments(per_day, 220.0); // 1,100/yr
    let output = pt::output_basis_value(per_year, 42.0); // £46,200
    let bottleneck = pt::bottleneck_basis_value(hours, 500.0); // £110,000
    assert!((wage - 17_600.0).abs() < TOL);
    assert!((output - 46_200.0).abs() < TOL);
    assert!((bottleneck - 110_000.0).abs() < TOL);
    assert!(wage <= output && output <= bottleneck);
}

// Locks down: fragmentation_adjusted_value(v, 1) == v, (v, 0) == 0, and it scales linearly between.
#[test]
fn prop_fragmentation_adjustment_endpoints_and_linearity() {
    for &v in &[0.0, 17_600.0, 46_200.0, 110_000.0] {
        assert!((pt::fragmentation_adjusted_value(v, 1.0) - v).abs() < TOL);
        assert!((pt::fragmentation_adjusted_value(v, 0.0) - 0.0).abs() < TOL);
        assert!((pt::fragmentation_adjusted_value(v, 0.5) - 0.5 * v).abs() < TOL);
    }
}

// Locks down: annual_hours_saved converts minutes to hours (÷60) consistently with the daily figure.
#[test]
fn prop_annual_hours_saved_is_minutes_over_sixty() {
    for &(mins, days) in &[(60.0, 220.0), (30.0, 200.0), (90.0, 250.0)] {
        let h = pt::annual_hours_saved(mins, days);
        assert!((h - mins * days / 60.0).abs() < TOL);
    }
    // Cross-check via the per-consultation route: 3 min × 20 consultations = 60 min/day.
    let daily = pt::daily_minutes_saved(3.0, 20.0);
    assert!((pt::annual_hours_saved(daily, 100.0) - 100.0).abs() < TOL);
}

// ---- Reach & equity invariants ----

// Locks down: population_impact == reach × effectiveness, and Stratum::impact agrees with the free function.
#[test]
fn prop_stratum_impact_consistent_with_free_function() {
    let grid = [(0.12, 0.02), (0.22, 0.02), (0.04, 0.025), (0.0, 0.5), (1.0, 0.0)];
    for &(r, e) in &grid {
        let s = re::Stratum { reach: r, effectiveness: e };
        assert!((s.impact() - re::population_impact(r, e)).abs() < TOL);
        assert!((re::population_impact(r, e) - r * e).abs() < TOL);
    }
    // reach() feeds population_impact consistently: 12,000/100,000 → 0.12 → 0.0024.
    let r = re::reach(12_000.0, 100_000.0).unwrap();
    assert!((re::population_impact(r, 0.02) - 0.0024).abs() < TOL);
}

// Locks down: equity_gap is antisymmetric — gap(a, b) == −gap(b, a).
#[test]
fn prop_equity_gap_antisymmetric() {
    let vals = [0.0044, 0.0010, 0.0, 0.0030];
    for &a in &vals {
        for &b in &vals {
            assert!((re::equity_gap(a, b) + re::equity_gap(b, a)).abs() < TOL);
        }
    }
}

// Locks down: equity_weighted_qalys with all weights 1.0 equals the plain unweighted sum.
#[test]
fn prop_equity_weight_one_is_identity() {
    let groups = [85.8, 18.0, 27.0, 0.0];
    let unweighted: f64 = groups.iter().sum();
    let weighted: f64 = groups.iter().map(|&q| re::equity_weighted_qalys(q, 1.0)).sum();
    assert!((weighted - unweighted).abs() < TOL);
    // Any weight > 1 strictly increases a positive group's contribution.
    assert!(re::equity_weighted_qalys(10.0, 1.5) > re::equity_weighted_qalys(10.0, 1.0));
}

// Locks down: impact_ratio × bottom recovers top (ratio is exactly the quotient).
#[test]
fn prop_impact_ratio_is_exact_quotient() {
    for &(top, bottom) in &[(0.0044, 0.0010), (0.0030, 0.0010), (0.5, 0.5)] {
        let r = re::impact_ratio(top, bottom).unwrap();
        assert!((r * bottom - top).abs() < TOL);
    }
}

// ---- Readmission invariants ----

// Locks down: observed_vs_expected == 1 exactly when observed == expected, across a grid.
#[test]
fn prop_observed_vs_expected_is_one_when_equal() {
    for &x in &[1.0, 80.0, 360.0, 12_345.0] {
        assert!((rr::observed_vs_expected(x, x).unwrap() - 1.0).abs() < TOL);
    }
    // And deviates in the right direction otherwise.
    assert!(rr::observed_vs_expected(400.0, 360.0).unwrap() > 1.0);
    assert!(rr::observed_vs_expected(300.0, 360.0).unwrap() < 1.0);
}

// Locks down: net_benefit is exactly value − cost, and the full chain matches the hand-set identity.
#[test]
fn prop_readmission_net_benefit_identity() {
    for &(v, c) in &[(280_000.0, 120_000.0), (0.0, 0.0), (100.0, 250.0)] {
        assert!((rr::net_benefit(v, c) - (v - c)).abs() < TOL);
    }
    // Chain: avoided × (spell + penalty) − discharges × per-discharge cost.
    let avoided = rr::avoided_readmissions(2_000.0, 0.18, 0.14);
    let value = rr::value_of_avoidance(avoided, 3_500.0, 500.0);
    let cost = rr::program_cost(2_000.0, 60.0);
    assert!((rr::net_benefit(value, cost) - (80.0 * 4_000.0 - 120_000.0)).abs() < 1e-6);
}

// Locks down: avoided_readmissions is negative when the new rate is worse — no silent clamping.
#[test]
fn prop_avoided_readmissions_sign_follows_rate_direction() {
    assert!(rr::avoided_readmissions(2_000.0, 0.14, 0.18) < 0.0);
    assert_eq!(rr::avoided_readmissions(2_000.0, 0.18, 0.18), 0.0);
}

// ---- RTT invariants ----

// Locks down: total_pathway_duration == Σ stages and longest_stage == max stage, on several pathways.
#[test]
fn prop_pathway_sum_and_max() {
    let pathways: [&[f64]; 3] = [
        &[1.0, 6.0, 9.0, 2.0, 6.0],
        &[18.0],
        &[0.0, 0.0, 4.5],
    ];
    for stages in pathways {
        let sum: f64 = stages.iter().sum();
        let max = stages.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((rtt::total_pathway_duration(stages) - sum).abs() < TOL);
        assert!((rtt::longest_stage(stages).unwrap() - max).abs() < TOL);
        // The longest stage never exceeds the whole pathway (non-negative stages).
        assert!(rtt::longest_stage(stages).unwrap() <= rtt::total_pathway_duration(stages) + TOL);
    }
}

// Locks down: cohort QALY gain equals patients × the per-patient waiting cost of the removed weeks.
#[test]
fn prop_rtt_cohort_gain_matches_per_patient_cost() {
    for &(patients, weeks, ut, uw) in
        &[(5_000.0, 5.0, 0.80, 0.68), (400.0, 26.0, 0.85, 0.60), (1.0, 52.0, 1.0, 0.0)]
    {
        let per_patient = rtt::waiting_health_cost_qalys(weeks / 52.0, ut, uw);
        let cohort = rtt::qaly_gain_from_wait_reduction(patients, weeks, ut, uw);
        assert!((cohort - patients * per_patient).abs() < 1e-6);
    }
}

// ---- Opportunity-cost invariants ----

// Locks down: opportunity_cost returns the maximum regardless of order, and net_gain is antisymmetric.
#[test]
fn prop_opportunity_cost_is_order_invariant_max() {
    let perms: [&[f64]; 3] = [
        &[120_000.0, 300_000.0, 90_000.0],
        &[300_000.0, 90_000.0, 120_000.0],
        &[90_000.0, 120_000.0, 300_000.0],
    ];
    for p in perms {
        assert_eq!(oc::opportunity_cost(p), Some(300_000.0));
    }
    for &(a, b) in &[(400_000.0, 300_000.0), (0.0, 0.0), (-5.0, 5.0)] {
        assert!((oc::net_gain(a, b) + oc::net_gain(b, a)).abs() < TOL);
    }
}

// Locks down: qalys_displaced is linear in spend — displacement scales with the diverted budget.
#[test]
fn prop_qalys_displaced_linear_in_spend() {
    for &spend in &[13_000.0, 26_000.0, 130_000.0] {
        let q = oc::qalys_displaced(spend, oc::NHS_MARGINAL_COST_PER_QALY_GBP).unwrap();
        assert!((q - spend / 13_000.0).abs() < TOL);
        let q2 = oc::qalys_displaced(2.0 * spend, oc::NHS_MARGINAL_COST_PER_QALY_GBP).unwrap();
        assert!((q2 - 2.0 * q).abs() < TOL);
    }
}

// ============================================================================
// 3. CROSS-MODULE CONSISTENCY
// ============================================================================

// Locks down: monetizing X QALYs at the £13k margin, then asking how many QALYs that spend
// displaces at the same margin, round-trips back to exactly X (and in the reverse direction).
#[test]
fn cross_monetized_value_and_qalys_displaced_round_trip() {
    let margin = oc::NHS_MARGINAL_COST_PER_QALY_GBP; // £13,000
    for &x in &[0.125, 1.0, 7.8, 50.0, 57.7] {
        let spend = qaly::monetized_value(x, margin);
        let displaced = oc::qalys_displaced(spend, margin).unwrap();
        assert!((displaced - x).abs() < TOL, "QALY → £ → QALY round trip broke at {x}");
    }
    // Reverse: a spend displaces q QALYs whose monetized value is the original spend.
    for &spend in &[13_000.0, 500_000.0, 1_000_000.0] {
        let q = oc::qalys_displaced(spend, margin).unwrap();
        assert!((qaly::monetized_value(q, margin) - spend).abs() < 1e-6);
    }
}

// Locks down: psa::net_monetary_benefit(λ, effect, cost) equals
// net_monetary_benefit::net_monetary_benefit(effect, cost, λ) — NOTE the different argument
// orders: PSA takes (threshold, effect, cost); the NMB module takes (delta_effect, delta_cost, lambda).
#[test]
fn cross_nmb_agreement_despite_different_argument_orders() {
    let grid = [
        (30_000.0, 2.0, 24_000.0),
        (20_000.0, 0.125, 500.0),
        (13_000.0, 57.7, 750_000.0),
        (0.0, 5.0, 100.0),
        (30_000.0, -0.5, -1_000.0),
    ];
    for &(lambda, effect, cost) in &grid {
        let from_psa = psa::net_monetary_benefit(lambda, effect, cost);
        let from_nmb = nmb::net_monetary_benefit(effect, cost, lambda);
        assert!(
            (from_psa - from_nmb).abs() < TOL,
            "NMB modules disagree at λ={lambda}, e={effect}, c={cost}"
        );
        // Both equal the textbook formula λ·E − C.
        assert!((from_psa - (lambda * effect - cost)).abs() < TOL);
    }
}

// Locks down: the three monetized_value implementations (QALY, PRO, RTT modules) are the same map.
#[test]
fn cross_monetized_value_identical_across_modules() {
    for &(q, t) in &[(7.8, 20_000.0), (50.0, 30_000.0), (0.0, 25_000.0), (57.7, 13_000.0)] {
        let a = qaly::monetized_value(q, t);
        let b = pro::monetized_value(q, t);
        let c = rtt::monetized_value(q, t);
        assert!((a - b).abs() < TOL && (b - c).abs() < TOL);
    }
}

// ============================================================================
// 4. DOMAIN SCENARIOS
// ============================================================================

// Locks down: full RTT-acceleration case — wait cut → QALYs → monetized value vs the £13k
// opportunity-cost margin, with hand-computed expected values throughout.
#[test]
fn scenario_rtt_acceleration_vs_opportunity_cost_margin() {
    // Pathway: triage 1 wk, first appointment 6, diagnostics 9, decision 2, treatment 6 = 24 weeks.
    let stages = [1.0, 6.0, 9.0, 2.0, 6.0];
    assert!((rtt::total_pathway_duration(&stages) - 24.0).abs() < TOL);
    // Diagnostics is the constraint; straight-to-test cuts 5 weeks of pure queueing.
    assert_eq!(rtt::longest_stage(&stages), Some(9.0));

    // 5,000 patients/year, utilities 0.80 treated vs 0.68 waiting, 5 weeks removed:
    // QALY gain = 5,000 × (5/52) × 0.12 = 3,000/52 = 57.6923... QALYs/year.
    let q = rtt::qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
    assert!((q - 3_000.0 / 52.0).abs() < 1e-9);

    // Monetized at the £13k NHS margin: 57.6923 × 13,000 = £750,000 exactly.
    let health_value = rtt::monetized_value(q, oc::NHS_MARGINAL_COST_PER_QALY_GBP);
    assert!((health_value - 750_000.0).abs() < 1e-6);

    // The triage software costs £500,000/year: that spend displaces
    // 500,000 / 13,000 = 38.4615 QALYs elsewhere in the system.
    let displaced = oc::qalys_displaced(500_000.0, oc::NHS_MARGINAL_COST_PER_QALY_GBP).unwrap();
    assert!((displaced - 500_000.0 / 13_000.0).abs() < TOL);

    // Net health gain: 57.6923 − 38.4615 = 19.2308 QALYs/year > 0 — funding it makes
    // the system healthier at the margin.
    let net_health = qaly::qaly_gain(q, displaced);
    assert!((net_health - (3_000.0 / 52.0 - 500_000.0 / 13_000.0)).abs() < 1e-9);
    assert!(net_health > 0.0);

    // Same verdict in money: NMB at λ = £13k is 750,000 − 500,000 = £250,000.
    let money_verdict = nmb::net_monetary_benefit(q, 500_000.0, oc::NHS_MARGINAL_COST_PER_QALY_GBP);
    assert!((money_verdict - 250_000.0).abs() < 1e-6);
    // And the money verdict is exactly the net health gain monetized at the margin.
    assert!(
        (money_verdict - qaly::monetized_value(net_health, oc::NHS_MARGINAL_COST_PER_QALY_GBP))
            .abs()
            < 1e-6
    );

    // Governance side-effect: performance moves from 88% (breach) to 92% (meets).
    let before = rtt::rtt_performance_percent(4_400.0, 5_000.0).unwrap();
    let after = rtt::rtt_performance_percent(4_600.0, 5_000.0).unwrap();
    assert!(!rtt::meets_rtt_standard(before));
    assert!(rtt::meets_rtt_standard(after));
}

// Locks down: a PSA run gated by a severity-weighted threshold — the severity modifier moves the
// probabilistic decision, not just the point estimate.
#[test]
fn scenario_psa_gated_by_severity_weighted_threshold() {
    // Severe condition: general population expects 21 discounted QALYs, patients 2 →
    // absolute shortfall 19 (≥ 18) → top ×1.7 band.
    let a = sev::absolute_shortfall(21.0, 2.0);
    assert!((a - 19.0).abs() < TOL);
    let p = sev::proportional_shortfall(a, 21.0).unwrap();
    assert!((p - 19.0 / 21.0).abs() < TOL); // 0.9048: below 0.95 — absolute measure drives the band.
    let w = sev::severity_weight(a, p);
    assert_eq!(w, 1.7);
    // Effective threshold: £30,000 × 1.7 = £51,000/QALY.
    let lambda_base = 30_000.0;
    let lambda_eff = sev::effective_threshold(lambda_base, w);
    assert!((lambda_eff - 51_000.0).abs() < TOL);

    // PSA: effect ~ Normal(0.5 QALYs, 0.15), cost ~ Gamma(mean £20k, sd £4k), 5,000 draws.
    // Analytic mean NMB: at £30k → 0.5×30k − 20k = −£5,000 (likely rejected);
    //                    at £51k → 0.5×51k − 20k = +£5,500 (likely funded).
    let n = 5_000;
    let mut rng = psa::Lcg::new(2024);
    let mut effects = Vec::with_capacity(n);
    let mut costs = Vec::with_capacity(n);
    for _ in 0..n {
        effects.push(rng.normal(0.5, 0.15));
        costs.push(rng.gamma_mean_sd(20_000.0, 4_000.0).unwrap());
    }
    let nmb_base: Vec<f64> = effects
        .iter()
        .zip(&costs)
        .map(|(&e, &c)| psa::net_monetary_benefit(lambda_base, e, c))
        .collect();
    let nmb_eff: Vec<f64> = effects
        .iter()
        .zip(&costs)
        .map(|(&e, &c)| psa::net_monetary_benefit(lambda_eff, e, c))
        .collect();

    // Sample means agree with the analytic values (wide Monte Carlo tolerances).
    assert!((psa::mean(&nmb_base).unwrap() - (-5_000.0)).abs() < 500.0);
    assert!((psa::mean(&nmb_eff).unwrap() - 5_500.0).abs() < 800.0);

    // The severity weight flips the probabilistic verdict: under £30k the technology is
    // more likely rejected; under the ×1.7-weighted £51k it is more likely funded.
    let p_base = psa::probability_positive(&nmb_base).unwrap();
    let p_eff = psa::probability_positive(&nmb_eff).unwrap();
    assert!(p_base < 0.5, "at the base threshold P(fund) = {p_base}, expected < 0.5");
    assert!(p_eff > 0.5, "at the weighted threshold P(fund) = {p_eff}, expected > 0.5");
    assert!(p_eff > p_base);

    // CEAC framing: "treat" vs "standard care" (NMB 0 in every draw). Fractions are a
    // valid probability split at both thresholds.
    let zeros = vec![0.0; n];
    for draws in [&nmb_base, &nmb_eff] {
        let c = psa::ceac(&[draws.as_slice(), zeros.as_slice()]).unwrap();
        assert!((c[0] + c[1] - 1.0).abs() < TOL);
        assert!((0.0..=1.0).contains(&c[0]));
    }
    // Median NMB at the weighted threshold is positive too — the decision is not mean-driven.
    assert!(psa::percentile(&nmb_eff, 50.0).unwrap() > 0.0);

    // Consistency: the same verdicts expressed as effective ICERs. Point-estimate ICER =
    // £20,000 / 0.5 = £40,000/QALY: above £30k unweighted, below £51k after ÷1.7 (≈ £23,529).
    let point_icer = 20_000.0 / 0.5;
    let eff_icer = sev::effective_icer(point_icer, w).unwrap();
    assert!(point_icer > lambda_base);
    assert!((eff_icer - 40_000.0 / 1.7).abs() < TOL);
    assert!(eff_icer < lambda_base);
}

// Locks down: an equity-stratified rollout of a depression app — PRO responder math feeding
// RE-AIM impact strata, with an assisted-digital arm shrinking the equity gap.
#[test]
fn scenario_equity_stratified_depression_app_rollout() {
    // Trial evidence (aggregate): PHQ-9 mean difference −4.1 misses the MCID of 5 →
    // the claim must be made on responders, not means.
    let diff = pro::adjusted_difference(-6.2, -2.1);
    assert!(!pro::clears_mcid(diff, pro::PHQ9_MCID));

    // Per-responder health gain: EQ-5D +0.06 sustained 6 months = 0.03 QALYs.
    let q_per_responder = pro::qalys_from_utility_gain(0.06, 0.5);
    assert!((q_per_responder - 0.03).abs() < TOL);

    // Stratum Q1 (least deprived): 11,000 of 50,000 eligible participate → reach 0.22;
    // ARR 0.26 → per-participant effectiveness 0.26 × 0.03 = 0.0078 QALYs.
    let q1_reach = re::reach(11_000.0, 50_000.0).unwrap();
    assert!((q1_reach - 0.22).abs() < TOL);
    let q1_arr = pro::absolute_risk_reduction(0.48, 0.22);
    let q1_eff = q1_arr * q_per_responder;
    let q1 = re::Stratum { reach: q1_reach, effectiveness: q1_eff };
    assert!((q1.impact() - 0.22 * 0.0078).abs() < TOL); // 0.001716 QALYs/eligible person

    // Stratum Q5 (most deprived): 2,000 of 50,000 → reach 0.04; more headroom, ARR 0.30 →
    // effectiveness 0.009. Impact 0.04 × 0.009 = 0.00036 — nearly 5× less than Q1.
    let q5_reach = re::reach(2_000.0, 50_000.0).unwrap();
    assert!((q5_reach - 0.04).abs() < TOL);
    let q5_arr = pro::absolute_risk_reduction(0.52, 0.22);
    let q5_eff = q5_arr * q_per_responder;
    let q5 = re::Stratum { reach: q5_reach, effectiveness: q5_eff };
    assert!((q5.impact() - 0.000_36).abs() < TOL);

    // Equity gap and ratio, hand-computed: 0.001716 − 0.00036 = 0.001356; ratio 4.7667.
    let gap_before = re::equity_gap(q1.impact(), q5.impact());
    assert!((gap_before - 0.001_356).abs() < TOL);
    let ratio_before = re::impact_ratio(q1.impact(), q5.impact()).unwrap();
    assert!((ratio_before - 0.001_716 / 0.000_36).abs() < 1e-6);

    // Cohort QALYs: Q1 = 11,000 × 0.26 = 2,860 responders × 0.03 = 85.8;
    //               Q5 = 2,000 × 0.30 = 600 responders × 0.03 = 18.0.
    let q1_qalys = pro::cohort_qalys(pro::extra_responders(11_000.0, q1_arr), q_per_responder);
    let q5_qalys = pro::cohort_qalys(pro::extra_responders(2_000.0, q5_arr), q_per_responder);
    assert!((q1_qalys - 85.8).abs() < 1e-6);
    assert!((q5_qalys - 18.0).abs() < 1e-6);

    // Distributional view: weight Q5 at 1.5, Q1 at 1.0 → 85.8 + 27.0 = 112.8 weighted QALYs
    // versus 103.8 unweighted.
    let weighted = re::equity_weighted_qalys(q1_qalys, 1.0) + re::equity_weighted_qalys(q5_qalys, 1.5);
    assert!((weighted - 112.8).abs() < 1e-6);
    assert!(weighted > q1_qalys + q5_qalys);

    // Assisted-digital arm lifts Q5 reach to 0.12 (6,000 participants): impact triples to
    // 0.00108, the gap shrinks to 0.000636, and Q5 gains 54 − 18 = 36 extra QALYs.
    let q5_after_impact = re::population_impact(0.12, q5_eff);
    assert!((q5_after_impact - 0.001_08).abs() < TOL);
    let gap_after = re::equity_gap(q1.impact(), q5_after_impact);
    assert!((gap_after - 0.000_636).abs() < TOL);
    assert!(gap_after < gap_before);
    let q5_qalys_after = pro::cohort_qalys(pro::extra_responders(6_000.0, q5_arr), q_per_responder);
    assert!((q5_qalys_after - 54.0).abs() < 1e-6);

    // Economic close: the 36 extra QALYs are worth £720k–£1.08M at NICE thresholds, so an
    // assisted-digital arm costing £400k/year clears even the £20k valuation — the equity
    // investment IS the efficiency investment.
    let extra = qaly::qaly_gain(q5_qalys_after, q5_qalys);
    assert!((extra - 36.0).abs() < 1e-6);
    assert!((pro::monetized_value(extra, 20_000.0) - 720_000.0).abs() < 1e-3);
    assert!((pro::monetized_value(extra, 30_000.0) - 1_080_000.0).abs() < 1e-3);
    assert!(nmb::net_monetary_benefit(extra, 400_000.0, 20_000.0) > 0.0);
}

// Locks down: prevention-economics honesty scenario — the discharge-support app is cash-positive
// on readmissions alone, while the hypertension program is cost-effective but NOT cost-saving,
// and both verdicts survive a practitioner-time reality check.
#[test]
fn scenario_prevention_vs_cash_releasing_readmission_case() {
    // Readmission app: 2,000 discharges, 18% → 14%, £3,500/spell, £60/discharge.
    let baseline = rr::readmission_rate_percent(360.0, 2_000.0).unwrap();
    assert!((baseline - 18.0).abs() < TOL);
    let avoided = rr::avoided_readmissions(2_000.0, 0.18, 0.14);
    let value = rr::value_of_avoidance(avoided, 3_500.0, 0.0);
    let cost = rr::program_cost(2_000.0, 60.0);
    let net = rr::net_benefit(value, cost);
    assert!((net - 160_000.0).abs() < 1e-6); // genuinely cash-positive
    // The hospital was readmitting exactly as expected for its case mix — no risk-adjustment alibi.
    assert!((rr::observed_vs_expected(360.0, 360.0).unwrap() - 1.0).abs() < TOL);

    // Hypertension prevention program: £20.75M cost, £18M offsets → net cost £2.75M.
    let p_cost = prev::program_cost(100_000.0, 25.0, 8.3);
    let offsets = prev::downstream_offsets(400.0, 45_000.0);
    let p_net = prev::net_cost(p_cost, offsets);
    assert!((p_net - 2_750_000.0).abs() < 1e-6);
    // NOT cost-saving — the per-person inequality agrees (£207.50 spent vs £180 expected avoided).
    assert!(!prev::is_cost_saving(p_net));
    assert!(!prev::per_person_cost_saving_condition(25.0 * 8.3, 0.004, 45_000.0, 1.0));
    // But outstandingly cost-effective: £2.75M / 1,200 QALYs = £2,291.67/QALY < £20,000.
    let q = prev::qalys_gained(400.0, 3.0);
    let cpq = prev::cost_per_qaly(p_net, q).unwrap();
    assert!((cpq - 2_750_000.0 / 1_200.0).abs() < TOL);
    assert!(prev::is_cost_effective(cpq, 20_000.0));

    // Practitioner-time reality check on the readmission app's nurse-escalation claim:
    // 4 min/day of nurse time across 10 escalation reviews = 40 min/day; at 12-minute
    // review slots that is 3.333 extra reviews/day, but only 60% consolidates into
    // usable blocks — value the output basis at £30/review honestly.
    let daily = pt::daily_minutes_saved(4.0, 10.0);
    let per_day = pt::extra_appointments_per_day(daily, 12.0).unwrap();
    let per_year = pt::annual_extra_appointments(per_day, 220.0);
    let raw = pt::output_basis_value(per_year, 30.0);
    let honest = pt::fragmentation_adjusted_value(raw, 0.6);
    assert!((daily - 40.0).abs() < TOL);
    assert!((per_year - 40.0 / 12.0 * 220.0).abs() < 1e-9);
    assert!((honest - raw * 0.6).abs() < 1e-9);
    // The wage basis (£25/h loaded band-5 nurse hour) is the least honest and smallest figure here.
    let hours = pt::annual_hours_saved(daily, 220.0);
    let wage = pt::wage_basis_value(hours, 25.0);
    assert!(wage < raw);
}
