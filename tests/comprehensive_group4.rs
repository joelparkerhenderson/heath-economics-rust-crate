//! Comprehensive integration tests, group 4.
//!
//! Covers: earlier_intervention, emergency_attendance_avoidance,
//! engagement_metrics, eq_5d, expected_value_of_perfect_information,
//! flow_metrics, gds_service_metrics,
//! hard_cash_releasing_savings_deficit_defense,
//! health_adjusted_life_expectancy, health_app_unit_economics.
//!
//! Sections:
//!   1. EDGE CASES
//!   2. PROPERTIES / INVARIANTS
//!   3. CROSS-MODULE CONSISTENCY
//!   4. DOMAIN SCENARIOS

use health_economics::earlier_intervention as ei;
use health_economics::emergency_attendance_avoidance as ea;
use health_economics::engagement_metrics as em;
use health_economics::eq_5d;
use health_economics::eq_5d::Eq5dProfile;
use health_economics::expected_value_of_perfect_information as evpi_mod;
use health_economics::expected_value_of_perfect_information::Scenario;
use health_economics::flow_metrics as fm;
use health_economics::gds_service_metrics as gds;
use health_economics::hard_cash_releasing_savings_deficit_defense as hc;
use health_economics::health_adjusted_life_expectancy as hale_mod;
use health_economics::health_adjusted_life_expectancy::ConditionBurden;
use health_economics::health_app_unit_economics as ue;
use health_economics::quality_adjusted_life_year as qaly_mod;
use health_economics::quality_adjusted_life_year::HealthState;
use health_economics::retention_and_churn as rc;

const TOL: f64 = 1e-9;

/// Deterministic pseudo-random f64 in [0, 1): a plain LCG, no external crates.
fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Top 53 bits as a fraction.
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

// =========================================================================
// 1. EDGE CASES
// =========================================================================

// Locks down: every Option-returning fn in engagement_metrics returns None
// exactly when its denominator is 0.0, and Some otherwise.
#[test]
fn edge_engagement_none_conditions_are_exactly_zero_denominators() {
    assert!(em::stickiness_percent(5.0, 0.0).is_none());
    assert!(em::stickiness_percent(0.0, 1.0).is_some());
    assert!(em::sessions_per_user(5.0, 0.0).is_none());
    assert!(em::sessions_per_user(0.0, 1.0).is_some());
    assert!(em::average_session_duration(5.0, 0.0).is_none());
    assert!(em::average_session_duration(0.0, 1.0).is_some());
    assert!(em::feature_engagement(5.0, 0.0).is_none());
    assert!(em::feature_engagement(0.0, 1.0).is_some());
    assert!(em::effective_dose_share(5.0, 0.0).is_none());
    assert!(em::effective_dose_share(0.0, 1.0).is_some());
    // overstatement_factor guards on effective-dose users (its denominator),
    // not registered users.
    assert!(em::overstatement_factor(5.0, 0.0).is_none());
    assert!(em::overstatement_factor(0.0, 1.0).is_some());
}

// Locks down: engagement boundary values — DAU == MAU gives 100%, DAU == 0
// gives 0%, and a numerator of zero is Some(0), never None.
#[test]
fn edge_engagement_boundary_values() {
    assert!((em::stickiness_percent(500.0, 500.0).unwrap() - 100.0).abs() < TOL);
    assert!((em::stickiness_percent(0.0, 500.0).unwrap() - 0.0).abs() < TOL);
    assert_eq!(em::sessions_per_user(0.0, 10.0), Some(0.0));
    assert_eq!(em::average_session_duration(0.0, 10.0), Some(0.0));
    assert_eq!(em::feature_engagement(0.0, 10.0), Some(0.0));
    // effective_dose_share is not clamped: more effective users than
    // registered yields a share > 1 (surprising but as coded).
    assert!((em::effective_dose_share(20.0, 10.0).unwrap() - 2.0).abs() < TOL);
    // overstatement_factor is not clamped either: it drops below 1 when
    // "effective" users exceed registered users.
    assert!((em::overstatement_factor(10.0, 20.0).unwrap() - 0.5).abs() < TOL);
}

// Locks down: population_effect boundary shares 0 and 1 (zero and full credit).
#[test]
fn edge_population_effect_share_boundaries() {
    assert!((em::population_effect(6.0, 0.0) - 0.0).abs() < TOL);
    assert!((em::population_effect(6.0, 1.0) - 6.0).abs() < TOL);
}

// Locks down: flow_metrics None conditions — zero period, zero throughput,
// and zero total elapsed time each return None; nothing else does.
#[test]
fn edge_flow_none_conditions() {
    assert!(fm::throughput(5.0, 0.0).is_none());
    assert!(fm::throughput(0.0, 1.0).is_some());
    assert!(fm::littles_law_cycle_time(40.0, 0.0).is_none());
    assert!(fm::littles_law_cycle_time(0.0, 1.0).is_some());
    assert!(fm::flow_efficiency_percent(0.0, 0.0).is_none());
    // All-active and all-wait boundaries: 100% and 0%.
    assert_eq!(fm::flow_efficiency_percent(3.0, 0.0), Some(100.0));
    assert_eq!(fm::flow_efficiency_percent(0.0, 3.0), Some(0.0));
}

// Locks down: zero inputs collapse the flow arithmetic to zero, not None.
#[test]
fn edge_flow_zero_inputs() {
    assert!((fm::cycle_time(5.0, 5.0) - 0.0).abs() < TOL);
    assert!((fm::lead_time(5.0, 5.0) - 0.0).abs() < TOL);
    assert!((fm::littles_law_wip(0.0, 6.0) - 0.0).abs() < TOL);
    assert!((fm::delay_cost_eliminated(0.0, 2.5, 3_000.0) - 0.0).abs() < TOL);
    assert!((fm::delay_cost_eliminated(10.0, 0.0, 3_000.0) - 0.0).abs() < TOL);
}

// Locks down: gds_service_metrics None conditions — each KPI ratio guards
// only its own denominator.
#[test]
fn edge_gds_none_conditions() {
    assert!(gds::cost_per_transaction(1_000.0, 0.0).is_none());
    assert!(gds::cost_per_transaction(0.0, 1.0).is_some());
    assert!(gds::completion_rate_percent(1.0, 0.0).is_none());
    assert!(gds::completion_rate_percent(0.0, 1.0).is_some());
    assert!(gds::digital_take_up_percent(1.0, 0.0).is_none());
    assert!(gds::digital_take_up_percent(0.0, 1.0).is_some());
    assert!(gds::user_satisfaction_percent(1.0, 0.0).is_none());
    assert!(gds::user_satisfaction_percent(0.0, 1.0).is_some());
}

// Locks down: GDS boundary rates — completion 1.0 kills failure demand,
// completion 0.0 makes every digital attempt fall back; shift 0 saves nothing.
#[test]
fn edge_gds_boundary_rates() {
    assert!((gds::failure_demand_cost(1_000.0, 0.5, 1.0, 3.20) - 0.0).abs() < TOL);
    let all_fail = gds::failure_demand_cost(1_000.0, 0.5, 0.0, 3.20);
    assert!((all_fail - 1_000.0 * 0.5 * 3.20).abs() < TOL);
    assert!((gds::channel_shift_saving(1_000.0, 0.0, 3.20, 0.25) - 0.0).abs() < TOL);
    // Full shift of the whole volume: volume × (old − digital).
    let full = gds::channel_shift_saving(1_000.0, 1.0, 3.20, 0.25);
    assert!((full - 1_000.0 * (3.20 - 0.25)).abs() < 1e-6);
}

// Locks down: EVPI None conditions — empty scenario slice, first scenario
// with an empty option list, empty PSA draw set, and a draw with no options.
#[test]
fn edge_evpi_none_conditions() {
    assert!(evpi_mod::evpi(&[]).is_none());
    assert!(evpi_mod::expected_nmb_of_best_option(&[]).is_none());
    assert!(evpi_mod::expected_nmb_with_perfect_information(&[]).is_none());
    let no_options = vec![Scenario { probability: 1.0, option_nmbs: vec![] }];
    assert!(evpi_mod::evpi(&no_options).is_none());
    assert!(evpi_mod::expected_nmb_of_best_option(&no_options).is_none());
    assert!(evpi_mod::expected_nmb_with_perfect_information(&no_options).is_none());
    assert!(evpi_mod::evpi_from_psa_draws(&[]).is_none());
    assert!(evpi_mod::evpi_from_psa_draws(&[vec![]]).is_none());
}

// Locks down: degenerate EVPI inputs — a single world (probability 1) or a
// single option always give EVPI == 0 (no uncertainty / no choice).
#[test]
fn edge_evpi_degenerate_worlds_and_options() {
    let one_world = vec![Scenario { probability: 1.0, option_nmbs: vec![5.0, -2.0] }];
    assert!((evpi_mod::evpi(&one_world).unwrap() - 0.0).abs() < TOL);
    let one_option = vec![
        Scenario { probability: 0.5, option_nmbs: vec![7.0] },
        Scenario { probability: 0.5, option_nmbs: vec![-7.0] },
    ];
    assert!((evpi_mod::evpi(&one_option).unwrap() - 0.0).abs() < TOL);
    // population_evpi with zero decisions is zero.
    assert!((evpi_mod::population_evpi(1.2, 0.0) - 0.0).abs() < TOL);
}

// Locks down: Eq5dProfile::new accepts exactly levels 1–5 in every dimension
// (0 and 6 rejected, 1 and 5 accepted), independently per dimension.
#[test]
fn edge_eq5d_profile_validation_per_dimension() {
    for dim in 0..5u8 {
        for level in 0..=6u8 {
            let mut levels = [1u8; 5];
            levels[dim as usize] = level;
            let p = Eq5dProfile::new(levels[0], levels[1], levels[2], levels[3], levels[4]);
            if (1..=5).contains(&level) {
                assert!(p.is_some(), "dim {dim} level {level} should be accepted");
            } else {
                assert!(p.is_none(), "dim {dim} level {level} should be rejected");
            }
        }
    }
    // Explicit corner profiles: all-1s and all-5s are both valid.
    assert!(Eq5dProfile::new(1, 1, 1, 1, 1).is_some());
    assert!(Eq5dProfile::new(5, 5, 5, 5, 5).is_some());
}

// Locks down: code() round-trips to the constructor levels, and
// is_full_health is true for exactly the "11111" profile — checked over all
// 5^5 = 3125 valid profiles.
#[test]
fn edge_eq5d_code_roundtrip_and_full_health_exhaustive() {
    for mo in 1..=5u8 {
        for sc in 1..=5u8 {
            for ua in 1..=5u8 {
                for pd in 1..=5u8 {
                    for ad in 1..=5u8 {
                        let p = Eq5dProfile::new(mo, sc, ua, pd, ad).unwrap();
                        let code = p.code();
                        assert_eq!(code, format!("{mo}{sc}{ua}{pd}{ad}"));
                        // Round-trip: rebuild the profile from the code digits.
                        let digits: Vec<u8> =
                            code.bytes().map(|b| b - b'0').collect();
                        let p2 = Eq5dProfile::new(
                            digits[0], digits[1], digits[2], digits[3], digits[4],
                        )
                        .unwrap();
                        assert_eq!(p, p2);
                        assert_eq!(p.is_full_health(), code == "11111");
                    }
                }
            }
        }
    }
}

// Locks down: eq_5d QALY arithmetic at boundary utilities 0, 1, and the UK
// 3L floor −0.594; zero duration always yields zero QALYs and zero gain.
#[test]
fn edge_eq5d_qaly_boundaries() {
    assert!((eq_5d::qalys(1.0, 1.0) - 1.0).abs() < TOL);
    assert!((eq_5d::qalys(10.0, 0.0) - 0.0).abs() < TOL);
    assert!((eq_5d::qalys(1.0, -0.594) - (-0.594)).abs() < TOL);
    assert!((eq_5d::qalys(0.0, 0.9) - 0.0).abs() < TOL);
    assert!((eq_5d::qaly_gain(0.62, 0.71, 0.0) - 0.0).abs() < TOL);
    // Declining utility gives a negative gain.
    assert!(eq_5d::qaly_gain(0.71, 0.62, 1.0) < 0.0);
    // Control beating intervention gives a negative attributable gain.
    assert!(eq_5d::attributable_qaly_gain(0.02, 0.05) < 0.0);
    assert!((eq_5d::monetized_value(0.0, 20_000.0) - 0.0).abs() < TOL);
}

// Locks down: HALE guards — zero survivors and mismatched slice lengths are
// None; empty parallel slices with survivors are Some(0.0), not None.
#[test]
fn edge_hale_none_conditions_and_empty_slices() {
    assert!(hale_mod::sullivan_hale(&[1.0], &[1.0], 0.0).is_none());
    assert!(hale_mod::sullivan_hale(&[1.0, 2.0], &[1.0], 100.0).is_none());
    assert!(hale_mod::sullivan_hale(&[1.0], &[1.0, 1.0], 100.0).is_none());
    // Empty life table but live survivors: zero healthy years, not None.
    assert_eq!(hale_mod::sullivan_hale(&[], &[], 100.0), Some(0.0));
    assert!(hale_mod::hale_contribution_per_person(1.0, 0.0).is_none());
    assert!(hale_mod::hale_contribution_per_person(0.0, 1.0).is_some());
    // Empty ConditionBurden slice: everyone in full health.
    assert!((hale_mod::proportion_in_full_health(&[]) - 1.0).abs() < TOL);
}

// Locks down: proportion_in_full_health boundary burdens — prevalence and
// weight at 0 and 1; total burden > 1 goes negative (documented, no clamp).
#[test]
fn edge_hale_proportion_boundaries() {
    let none = [ConditionBurden { prevalence: 0.0, disability_weight: 1.0 }];
    assert!((hale_mod::proportion_in_full_health(&none) - 1.0).abs() < TOL);
    let weightless = [ConditionBurden { prevalence: 1.0, disability_weight: 0.0 }];
    assert!((hale_mod::proportion_in_full_health(&weightless) - 1.0).abs() < TOL);
    let total = [ConditionBurden { prevalence: 1.0, disability_weight: 1.0 }];
    assert!((hale_mod::proportion_in_full_health(&total) - 0.0).abs() < TOL);
    // Two death-like universal conditions: additive model goes to −1.0.
    let double = [
        ConditionBurden { prevalence: 1.0, disability_weight: 1.0 },
        ConditionBurden { prevalence: 1.0, disability_weight: 1.0 },
    ];
    assert!((hale_mod::proportion_in_full_health(&double) - (-1.0)).abs() < TOL);
}

// Locks down: unit-economics None conditions — each guarded fn returns None
// exactly on its zero denominator, and is_viable_ltv_cac is false (not a
// panic, not true) when CAC is zero even with a huge LTV.
#[test]
fn edge_unit_economics_none_conditions() {
    assert!(ue::cac(1.0, 0.0).is_none());
    assert!(ue::cac(0.0, 1.0).is_some());
    assert!(ue::arpu(1.0, 0.0).is_none());
    assert!(ue::arpu(0.0, 1.0).is_some());
    assert!(ue::ltv(6.99, 0.0).is_none());
    assert!(ue::ltv(0.0, 0.5).is_some());
    assert!(ue::ltv_cac_ratio(38.0, 0.0).is_none());
    assert!(ue::ltv_cac_ratio(0.0, 38.0).is_some());
    assert!(ue::effective_cac_per_retained_user(5.0, 0.0).is_none());
    assert!(ue::pmpm_margin_fraction(0.0, 0.30).is_none());
    assert!(!ue::is_viable_ltv_cac(1.0e12, 0.0));
}

// Locks down: churn boundary 1.0 — everyone churns each period, so LTV is
// exactly one period of ARPU; retention boundary 1.0 leaves CAC unchanged.
#[test]
fn edge_unit_economics_churn_and_retention_boundaries() {
    assert!((ue::ltv(6.99, 1.0).unwrap() - 6.99).abs() < TOL);
    assert!((ue::effective_cac_per_retained_user(38.0, 1.0).unwrap() - 38.0).abs() < TOL);
    // Zero-rate PMPM contract: zero revenue and a negative margin if serving
    // still costs money.
    assert!((ue::pmpm_revenue(0.0, 40_000.0, 12.0) - 0.0).abs() < TOL);
    assert!(ue::pmpm_margin(0.0, 0.30) < 0.0);
    assert!((ue::health_value_per_acquired_user(0.0, 20_000.0) - 0.0).abs() < TOL);
}

// Locks down: earlier_intervention zero/boundary inputs — empty backlog,
// zero rate, probability 0 and 1.
#[test]
fn edge_earlier_intervention_boundaries() {
    assert!((ei::progression_events_avoided(0.0, 0.02, 0.5) - 0.0).abs() < TOL);
    assert!((ei::progression_events_avoided(4_000.0, 0.0, 0.5) - 0.0).abs() < TOL);
    // Certain progression over a full year: every patient is an event.
    assert!((ei::progression_events_avoided(100.0, 1.0, 1.0) - 100.0).abs() < TOL);
    // Probability 0: no expected value per patient.
    assert!((ei::value_per_patient(4_000.0, 0.0, 0.8, 0.0, 20_000.0, 0.0) - 0.0).abs() < TOL);
    // Probability 1: per-patient value equals the full per-progression value.
    let full = ei::value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0);
    let per = ei::value_per_patient(4_000.0, 0.0, 0.8, 0.0, 20_000.0, 1.0);
    assert!((per - full).abs() < TOL);
    assert!((ei::total_backlog_value(0.0, 20_000.0) - 0.0).abs() < TOL);
    // Late treatment cheaper AND worse outcomes reversed: value can go negative.
    assert!(ei::value_per_avoided_progression(0.0, 4_000.0, 0.0, 0.8, 20_000.0) < 0.0);
}

// Locks down: emergency_attendance_avoidance boundaries — no rate change
// avoids nothing, a rate increase goes negative, and net can be negative.
#[test]
fn edge_emergency_avoidance_boundaries() {
    assert!((ea::avoided_events(3_000.0, 0.9, 0.9) - 0.0).abs() < TOL);
    assert!(ea::avoided_events(3_000.0, 0.7, 0.9) < 0.0);
    assert!((ea::avoided_events(0.0, 0.9, 0.7) - 0.0).abs() < TOL);
    assert!((ea::gross_saving(0.0, 300.0) - 0.0).abs() < TOL);
    assert!((ea::gross_saving_attendances_and_admissions(0.0, 300.0, 0.0, 3_800.0) - 0.0).abs() < TOL);
    // Intervention costing more than it saves: negative net (QALYs separate).
    assert!(ea::net_saving(100_000.0, 90_000.0, 20_000.0) < 0.0);
}

// Locks down: hard-cash boundaries — zero volumes on every mechanism give
// zero, and a premium equal to the substantive rate releases nothing.
#[test]
fn edge_hard_cash_boundaries() {
    assert!((hc::premium_shift_saving(0.0, 180.0, 0.0) - 0.0).abs() < TOL);
    assert!((hc::premium_shift_saving(780.0, 150.0, 150.0) - 0.0).abs() < TOL);
    assert!((hc::overtime_saving(0.0, 8.0) - 0.0).abs() < TOL);
    assert!((hc::cancelled_contract_saving(0.0, 50_000.0) - 0.0).abs() < TOL);
    assert!((hc::hard_saving(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0) - 0.0).abs() < TOL);
    assert!((hc::annual_workforce_overtime_saving(0.0, 2.5, 8.0, 46.0) - 0.0).abs() < TOL);
    assert!((hc::annual_bank_agency_saving(15.0, 180.0, 0.0) - 0.0).abs() < TOL);
    // Licence exceeding the saving: negative net cash position.
    assert!(hc::net_of_licence(100_000.0, 150_000.0) < 0.0);
}

// Locks down: extreme magnitudes stay finite (no NaN/inf) across a sample of
// pure-arithmetic functions from several modules.
#[test]
fn edge_extreme_magnitudes_stay_finite() {
    assert!(ei::total_backlog_value(1.0e100, 1.0e100).is_finite());
    assert!(ea::net_saving(1.0e100, 1.0e99, 1.0e99).is_finite());
    assert!(eq_5d::monetized_value(1.0e100, 1.0e100).is_finite());
    assert!(fm::littles_law_wip(1.0e100, 1.0e100).is_finite());
    assert!(hc::hard_saving(1.0e50, 1.0e50, 0.0, 1.0e50, 1.0e50, 1.0e50, 1.0e50).is_finite());
    assert!(hale_mod::years_to_days(1.0e100).is_finite());
    assert!(ue::pmpm_revenue(1.0e50, 1.0e50, 1.0e50).is_finite());
    assert!(evpi_mod::population_evpi(1.0e100, 1.0e100).is_finite());
    // Tiny magnitudes survive too.
    let tiny = evpi_mod::evpi(&[
        Scenario { probability: 0.5, option_nmbs: vec![1.0e-12, 0.0] },
        Scenario { probability: 0.5, option_nmbs: vec![-1.0e-12, 0.0] },
    ])
    .unwrap();
    assert!(tiny.is_finite() && tiny >= 0.0);
}

// =========================================================================
// 2. PROPERTIES / INVARIANTS
// =========================================================================

// Locks down: EVPI ≥ 0 (E[max] ≥ max E[]) over 200 pseudo-random scenario
// sets, and evpi == expected_nmb_with_perfect_information − expected_nmb_of_best_option.
#[test]
fn prop_evpi_nonnegative_and_identity_over_random_scenarios() {
    let mut seed = 42u64;
    for _ in 0..200 {
        let n_worlds = 2 + (lcg(&mut seed) * 4.0) as usize; // 2–5 worlds
        let n_options = 2 + (lcg(&mut seed) * 3.0) as usize; // 2–4 options
        // Raw weights normalized so probabilities sum to 1.
        let raw: Vec<f64> = (0..n_worlds).map(|_| 0.1 + lcg(&mut seed)).collect();
        let total: f64 = raw.iter().sum();
        let scenarios: Vec<Scenario> = raw
            .iter()
            .map(|w| Scenario {
                probability: w / total,
                option_nmbs: (0..n_options).map(|_| lcg(&mut seed) * 20.0 - 10.0).collect(),
            })
            .collect();
        let pi = evpi_mod::expected_nmb_with_perfect_information(&scenarios).unwrap();
        let best = evpi_mod::expected_nmb_of_best_option(&scenarios).unwrap();
        let v = evpi_mod::evpi(&scenarios).unwrap();
        assert!(pi >= best - 1e-9, "E[max] {pi} < max E[] {best}");
        assert!(v >= -1e-9, "EVPI negative: {v}");
        assert!((v - (pi - best)).abs() < 1e-9, "identity broken: {v} vs {}", pi - best);
    }
}

// Locks down: EVPI == 0 whenever one option dominates in every world
// (information cannot change the decision).
#[test]
fn prop_evpi_zero_under_dominance() {
    let mut seed = 7u64;
    for _ in 0..50 {
        let n_worlds = 2 + (lcg(&mut seed) * 4.0) as usize;
        let raw: Vec<f64> = (0..n_worlds).map(|_| 0.1 + lcg(&mut seed)).collect();
        let total: f64 = raw.iter().sum();
        let scenarios: Vec<Scenario> = raw
            .iter()
            .map(|w| {
                let base = lcg(&mut seed) * 10.0 - 5.0;
                let gap = 0.5 + lcg(&mut seed); // option 0 strictly better everywhere
                Scenario { probability: w / total, option_nmbs: vec![base, base - gap] }
            })
            .collect();
        let v = evpi_mod::evpi(&scenarios).unwrap();
        assert!(v.abs() < 1e-9, "dominated option produced EVPI {v}");
    }
}

// Locks down: evpi_from_psa_draws equals evpi over explicit equal-probability
// scenarios built from the same draws.
#[test]
fn prop_evpi_psa_draws_match_equal_probability_scenarios() {
    let mut seed = 99u64;
    for _ in 0..30 {
        let n_draws = 3 + (lcg(&mut seed) * 8.0) as usize; // 3–10 draws
        let n_options = 2 + (lcg(&mut seed) * 2.0) as usize; // 2–3 options
        let draws: Vec<Vec<f64>> = (0..n_draws)
            .map(|_| (0..n_options).map(|_| lcg(&mut seed) * 12.0 - 6.0).collect())
            .collect();
        let from_draws = evpi_mod::evpi_from_psa_draws(&draws).unwrap();
        let p = 1.0 / n_draws as f64;
        let scenarios: Vec<Scenario> = draws
            .iter()
            .map(|d| Scenario { probability: p, option_nmbs: d.clone() })
            .collect();
        let from_scenarios = evpi_mod::evpi(&scenarios).unwrap();
        assert!((from_draws - from_scenarios).abs() < 1e-9);
    }
}

// Locks down: population_evpi is linear in both arguments (additive in
// decisions, scaling in per-decision EVPI).
#[test]
fn prop_population_evpi_linear() {
    for i in 1..=10 {
        let v = 0.3 * i as f64;
        let a = 4.0 * i as f64;
        let b = 7.0;
        let sum = evpi_mod::population_evpi(v, a) + evpi_mod::population_evpi(v, b);
        assert!((evpi_mod::population_evpi(v, a + b) - sum).abs() < 1e-9);
        let doubled = evpi_mod::population_evpi(2.0 * v, a);
        assert!((doubled - 2.0 * evpi_mod::population_evpi(v, a)).abs() < 1e-9);
    }
}

// Locks down: Little's Law round-trips — WIP → cycle time → WIP and
// cycle time → WIP → cycle time are identities over a grid.
#[test]
fn prop_littles_law_round_trips() {
    for wip_i in 1..=8 {
        for tp_i in 1..=8 {
            let wip = 3.5 * wip_i as f64;
            let tp = 1.25 * tp_i as f64;
            let ct = fm::littles_law_cycle_time(wip, tp).unwrap();
            assert!((fm::littles_law_wip(tp, ct) - wip).abs() < 1e-9);
            let wip2 = fm::littles_law_wip(tp, ct);
            let ct2 = fm::littles_law_cycle_time(wip2, tp).unwrap();
            assert!((ct2 - ct).abs() < 1e-9);
        }
    }
}

// Locks down: flow_efficiency_percent ∈ (0, 100] whenever active time is
// positive, over a grid of active/wait combinations.
#[test]
fn prop_flow_efficiency_range() {
    for a_i in 1..=10 {
        for w_i in 0..=10 {
            let active = 0.5 * a_i as f64;
            let wait = 2.0 * w_i as f64;
            let fe = fm::flow_efficiency_percent(active, wait).unwrap();
            assert!(fe > 0.0 && fe <= 100.0, "fe {fe} out of (0,100]");
        }
    }
}

// Locks down: cycle_time ≤ lead_time whenever start ≥ request and
// finished ≤ delivered (the in-progress clock nests inside the request clock).
#[test]
fn prop_cycle_time_bounded_by_lead_time() {
    for req_i in 0..5 {
        for queue_i in 0..5 {
            for work_i in 1..5 {
                let requested = req_i as f64;
                let started = requested + queue_i as f64; // start ≥ request
                let finished = started + work_i as f64;
                let delivered = finished + 0.5; // delivery lag after finish
                let ct = fm::cycle_time(finished, started);
                let lt = fm::lead_time(delivered, requested);
                assert!(ct <= lt + 1e-9, "cycle {ct} > lead {lt}");
            }
        }
    }
    // throughput sanity on the same grid scale: 20 items over 4 periods = 5.
    assert!((fm::throughput(20.0, 4.0).unwrap() - 5.0).abs() < TOL);
}

// Locks down: stickiness is 100% at DAU == MAU and strictly monotone
// increasing in DAU at fixed MAU.
#[test]
fn prop_stickiness_monotone_in_dau() {
    let mau = 10_000.0;
    assert!((em::stickiness_percent(mau, mau).unwrap() - 100.0).abs() < TOL);
    let mut prev = -1.0;
    for dau_i in 0..=20 {
        let dau = 500.0 * dau_i as f64;
        let s = em::stickiness_percent(dau, mau).unwrap();
        assert!(s > prev, "stickiness not increasing at DAU {dau}");
        prev = s;
    }
}

// Locks down: effective_dose_share × overstatement_factor == 1 — the two
// funnel views are exact reciprocals (share = e/r, factor = r/e).
#[test]
fn prop_funnel_share_and_overstatement_are_reciprocals() {
    for e_i in 1..=10 {
        for r_i in 1..=10 {
            let effective = 700.0 * e_i as f64;
            let registered = 5_000.0 * r_i as f64;
            let share = em::effective_dose_share(effective, registered).unwrap();
            let factor = em::overstatement_factor(registered, effective).unwrap();
            assert!((share * factor - 1.0).abs() < 1e-9);
            // population_effect at full share returns the trial effect intact.
            assert!((em::population_effect(6.0, 1.0) - 6.0).abs() < TOL);
        }
    }
}

// Locks down: proportion_in_full_health strictly decreases as each positive
// burden is added, and equals 1 with zero burdens.
#[test]
fn prop_hale_proportion_decreases_as_burdens_added() {
    let all = [
        ConditionBurden { prevalence: 0.10, disability_weight: 0.32 },
        ConditionBurden { prevalence: 0.20, disability_weight: 0.10 },
        ConditionBurden { prevalence: 0.05, disability_weight: 0.50 },
        ConditionBurden { prevalence: 0.15, disability_weight: 0.05 },
    ];
    let mut prev = hale_mod::proportion_in_full_health(&all[..0]);
    assert!((prev - 1.0).abs() < TOL);
    for n in 1..=all.len() {
        let p = hale_mod::proportion_in_full_health(&all[..n]);
        assert!(p < prev, "adding burden {n} did not reduce full-health share");
        prev = p;
    }
}

// Locks down: with zero burdens (all full-health proportions 1.0) Sullivan
// HALE equals plain life expectancy on the same life table, and
// hale_gap == LE − HALE exactly.
#[test]
fn prop_sullivan_hale_reduces_to_life_expectancy_and_gap_identity() {
    let person_years = [98_000.0, 96_500.0, 93_000.0, 85_000.0, 60_000.0];
    let survivors = 100_000.0;
    // Zero burdens per interval → proportion 1.0 everywhere.
    let full: Vec<f64> = person_years
        .iter()
        .map(|_| hale_mod::proportion_in_full_health(&[]))
        .collect();
    let le = hale_mod::sullivan_hale(&person_years, &full, survivors).unwrap();
    let expected_le: f64 = person_years.iter().sum::<f64>() / survivors;
    assert!((le - expected_le).abs() < 1e-9);
    // With real burdens, HALE < LE and the gap identity holds.
    let burdened: Vec<f64> = (0..person_years.len())
        .map(|i| {
            hale_mod::proportion_in_full_health(&[ConditionBurden {
                prevalence: 0.05 * (i + 1) as f64,
                disability_weight: 0.32,
            }])
        })
        .collect();
    let hale = hale_mod::sullivan_hale(&person_years, &burdened, survivors).unwrap();
    assert!(hale < le);
    let gap = hale_mod::hale_gap(le, hale);
    assert!((gap - (le - hale)).abs() < 1e-9);
    assert!(gap > 0.0);
}

// Locks down: ltv == arpu / churn over a grid, and effective CAC at full
// retention equals raw CAC.
#[test]
fn prop_ltv_is_arpu_over_churn() {
    for a_i in 1..=6 {
        for c_i in 1..=10 {
            let arpu = 2.5 * a_i as f64;
            let churn = 0.1 * c_i as f64; // 0.1 … 1.0 inclusive
            let v = ue::ltv(arpu, churn).unwrap();
            assert!((v - arpu / churn).abs() < 1e-9);
        }
    }
    for cac_i in 1..=5 {
        let cac = 10.0 * cac_i as f64;
        assert!((ue::effective_cac_per_retained_user(cac, 1.0).unwrap() - cac).abs() < TOL);
    }
}

// Locks down: is_viable_ltv_cac ⟺ ltv_cac_ratio ≥ 3, with the boundary at
// exactly 3.0 counted as viable (>= as coded).
#[test]
fn prop_ltv_cac_viability_boundary_at_exactly_3() {
    let cac = 100.0;
    // Exactly 3:1 — ratio is exactly 3.0 in floating point (300/100).
    assert!((ue::ltv_cac_ratio(300.0, cac).unwrap() - 3.0).abs() < TOL);
    assert!(ue::is_viable_ltv_cac(300.0, cac));
    // Just below and just above the bar.
    assert!(!ue::is_viable_ltv_cac(299.0, cac));
    assert!(ue::is_viable_ltv_cac(301.0, cac));
    // Equivalence over a grid: viable ⟺ ratio ≥ 3.
    for ltv_i in 1..=12 {
        let ltv = 50.0 * ltv_i as f64;
        let ratio = ue::ltv_cac_ratio(ltv, cac).unwrap();
        assert_eq!(ue::is_viable_ltv_cac(ltv, cac), ratio >= 3.0);
    }
}

// Locks down: channel_shift_saving == volume × shift × (cost_old − cost_digital)
// over a grid, including a negative saving when digital costs more.
#[test]
fn prop_channel_shift_saving_formula() {
    for v_i in 1..=5 {
        for s_i in 0..=4 {
            let volume = 100_000.0 * v_i as f64;
            let shift = 0.25 * s_i as f64; // 0 … 1
            let s = gds::channel_shift_saving(volume, shift, 3.20, 0.25);
            assert!((s - volume * shift * (3.20 - 0.25)).abs() < 1e-6);
        }
    }
    // Digital dearer than the old channel: saving goes negative.
    assert!(gds::channel_shift_saving(1_000.0, 0.5, 0.25, 3.20) < 0.0);
}

// Locks down: failure_demand_cost strictly falls as completion_rate rises
// (fixed volume, share, fallback cost).
#[test]
fn prop_failure_demand_falls_with_completion_rate() {
    let mut prev = f64::INFINITY;
    for c_i in 0..=10 {
        let completion = 0.1 * c_i as f64;
        let cost = gds::failure_demand_cost(2_000_000.0, 0.40, completion, 3.20);
        assert!(cost < prev, "failure demand did not fall at completion {completion}");
        prev = cost;
    }
    // And at completion 1.0 it is exactly zero.
    assert!((prev - 0.0).abs() < 1e-9);
}

// Locks down: hard_saving == premium_shift_saving + overtime_saving +
// cancelled_contract_saving on identical inputs, over a grid.
#[test]
fn prop_hard_saving_is_sum_of_three_mechanisms() {
    for i in 0..=5 {
        for j in 0..=3 {
            let shifts = 100.0 * i as f64;
            let hours = 5_000.0 * j as f64;
            let contracts = j as f64;
            let sum = hc::premium_shift_saving(shifts, 180.0, 30.0)
                + hc::overtime_saving(hours, 8.0)
                + hc::cancelled_contract_saving(contracts, 50_000.0);
            let total = hc::hard_saving(shifts, 180.0, 30.0, hours, 8.0, contracts, 50_000.0);
            assert!((total - sum).abs() < 1e-9);
        }
    }
    // The convenience annualizers agree with the base fns on the same volumes.
    let annual_ot = hc::annual_workforce_overtime_saving(300.0, 2.5, 8.0, 46.0);
    assert!((annual_ot - hc::overtime_saving(300.0 * 2.5 * 46.0, 8.0)).abs() < 1e-9);
    let annual_bank = hc::annual_bank_agency_saving(15.0, 180.0, 52.0);
    assert!((annual_bank - hc::premium_shift_saving(15.0 * 52.0, 180.0, 0.0)).abs() < 1e-9);
}

// Locks down: earlier-intervention composition — value_per_patient × cohort
// equals total_backlog_value(events, per-event value) on consistent inputs.
#[test]
fn prop_earlier_intervention_per_patient_matches_backlog_total() {
    for p_i in 1..=4 {
        let patients = 1_000.0 * p_i as f64;
        let rate = 0.02;
        let years = 0.25;
        let events = ei::progression_events_avoided(patients, rate, years);
        let per_event = ei::value_per_avoided_progression(4_000.0, 500.0, 0.8, 0.2, 20_000.0);
        let total = ei::total_backlog_value(events, per_event);
        let per_patient = ei::value_per_patient(4_000.0, 500.0, 0.8, 0.2, 20_000.0, rate * years);
        assert!((per_patient * patients - total).abs() < 1e-6);
    }
}

// Locks down: gross_saving_attendances_and_admissions equals the sum of two
// single-line gross_saving calls on the same inputs.
#[test]
fn prop_gross_saving_combined_equals_sum_of_lines() {
    for i in 1..=5 {
        let attendances = ea::avoided_events(1_000.0 * i as f64, 0.9, 0.7);
        let admissions = ea::avoided_events(1_000.0 * i as f64, 0.5, 0.42);
        let combined =
            ea::gross_saving_attendances_and_admissions(attendances, 300.0, admissions, 3_800.0);
        let split = ea::gross_saving(attendances, 300.0) + ea::gross_saving(admissions, 3_800.0);
        assert!((combined - split).abs() < 1e-9);
    }
}

// Locks down: years_to_days uses a 365.25-day year, linearly, and
// hale_contribution_per_person scales inversely with cohort size.
#[test]
fn prop_hale_contribution_and_day_conversion_scale() {
    assert!((hale_mod::years_to_days(1.0) - 365.25).abs() < TOL);
    assert!((hale_mod::years_to_days(2.0) - 2.0 * hale_mod::years_to_days(1.0)).abs() < TOL);
    let base = hale_mod::hale_contribution_per_person(15_000.0, 500_000.0).unwrap();
    let half_cohort = hale_mod::hale_contribution_per_person(15_000.0, 250_000.0).unwrap();
    assert!((half_cohort - 2.0 * base).abs() < 1e-12);
}

// =========================================================================
// 3. CROSS-MODULE CONSISTENCY
// =========================================================================

// Locks down: eq_5d::qalys(duration, utility) agrees with
// quality_adjusted_life_year::qalys over single-state streams on a grid.
#[test]
fn cross_eq5d_qalys_match_qaly_module_single_state() {
    for d_i in 0..=8 {
        for u_i in -2..=10 {
            let duration = 0.5 * d_i as f64;
            let utility = 0.1 * u_i as f64; // −0.2 … 1.0
            let via_eq5d = eq_5d::qalys(duration, utility);
            let via_qaly = qaly_mod::qalys(&[HealthState { duration_years: duration, utility }]);
            assert!((via_eq5d - via_qaly).abs() < 1e-12);
        }
    }
}

// Locks down: a multi-state QALY stream equals the sum of eq_5d per-state
// QALYs, and both modules' monetized_value agree on the result.
#[test]
fn cross_eq5d_stream_sum_and_monetization_agree() {
    let states = [
        HealthState { duration_years: 0.5, utility: 0.62 },
        HealthState { duration_years: 0.5, utility: 0.71 },
        HealthState { duration_years: 1.0, utility: 0.85 },
    ];
    let stream_total = qaly_mod::qalys(&states);
    let piecewise: f64 = states
        .iter()
        .map(|s| eq_5d::qalys(s.duration_years, s.utility))
        .sum();
    assert!((stream_total - piecewise).abs() < 1e-12);
    // 0.31 + 0.355 + 0.85 = 1.515 QALYs, hand-computed.
    assert!((stream_total - 1.515).abs() < 1e-9);
    let via_eq5d = eq_5d::monetized_value(stream_total, 20_000.0);
    let via_qaly = qaly_mod::monetized_value(stream_total, 20_000.0);
    assert!((via_eq5d - via_qaly).abs() < 1e-9);
    assert!((via_eq5d - 30_300.0).abs() < 1e-6);
}

// Locks down: eq_5d::qaly_gain equals the difference of two qaly_mod streams
// (after vs before) over the same duration.
#[test]
fn cross_eq5d_gain_equals_stream_difference() {
    for d_i in 1..=6 {
        let duration = 0.25 * d_i as f64;
        let before = qaly_mod::qalys(&[HealthState { duration_years: duration, utility: 0.62 }]);
        let after = qaly_mod::qalys(&[HealthState { duration_years: duration, utility: 0.71 }]);
        let gain = eq_5d::qaly_gain(0.62, 0.71, duration);
        assert!((gain - (after - before)).abs() < 1e-12);
        // attributable gain nets a control stream computed the same way.
        let control = eq_5d::qaly_gain(0.62, 0.65, duration);
        let attributable = eq_5d::attributable_qaly_gain(gain, control);
        assert!((attributable - (gain - control)).abs() < 1e-12);
    }
}

// Locks down: health_app_unit_economics::health_value_per_acquired_user × cohort
// equals the retention_and_churn composition completers → qalys_delivered →
// monetized_health_value on consistent inputs.
#[test]
fn cross_health_value_per_user_matches_retention_composition() {
    for c_i in 1..=4 {
        let cohort = 50_000.0 * c_i as f64;
        let completion_fraction = 0.04;
        let qalys_per_completer = 0.02;
        let threshold = 20_000.0;
        // Cohort route (retention_and_churn).
        let n_completers = rc::completers(cohort, completion_fraction);
        let total_qalys = rc::qalys_delivered(n_completers, qalys_per_completer);
        let total_value = rc::monetized_health_value(total_qalys, threshold);
        // Per-user route (health_app_unit_economics): retention-weighted
        // QALYs per acquired user = completion fraction × QALYs/completer.
        let per_user =
            ue::health_value_per_acquired_user(completion_fraction * qalys_per_completer, threshold);
        assert!((per_user * cohort - total_value).abs() < 1e-6);
    }
}

// Locks down: effective_cac_per_retained_user (unit economics) and
// cost_per_retained_user (retention_and_churn) are the same computation.
#[test]
fn cross_effective_cac_matches_cost_per_retained_user() {
    for r_i in 1..=10 {
        let retention = 0.02 * r_i as f64;
        let a = ue::effective_cac_per_retained_user(5.0, retention).unwrap();
        let b = rc::cost_per_retained_user(5.0, retention).unwrap();
        assert!((a - b).abs() < 1e-12);
    }
}

// Locks down: eq_5d monetization agrees with retention_and_churn's
// monetized_health_value — one QALY-to-£ convention across the crate.
#[test]
fn cross_monetization_conventions_agree() {
    for q_i in 0..=6 {
        let qalys = 0.05 * q_i as f64;
        let a = eq_5d::monetized_value(qalys, 30_000.0);
        let b = rc::monetized_health_value(qalys, 30_000.0);
        let c = ue::health_value_per_acquired_user(qalys, 30_000.0);
        assert!((a - b).abs() < 1e-12);
        assert!((a - c).abs() < 1e-12);
    }
}

// =========================================================================
// 4. DOMAIN SCENARIOS
// =========================================================================

// Locks down: end-to-end virtual-ward case — avoided ED events valued per
// line, netted against service and pathway costs, plus the hard-cash agency
// line the freed capacity releases. All values hand-computed.
#[test]
fn scenario_virtual_ward_nets_out_pathway_costs() {
    // 2,000 COPD patients on a virtual ward. Matched controls:
    // ED attendances 1.2 → 0.9 /py; emergency admissions 0.6 → 0.45 /py.
    let attendances = ea::avoided_events(2_000.0, 1.2, 0.9);
    let admissions = ea::avoided_events(2_000.0, 0.6, 0.45);
    assert!((attendances - 600.0).abs() < 1e-9); // 2,000 × 0.3
    assert!((admissions - 300.0).abs() < 1e-9); // 2,000 × 0.15
    // Gross: 600 × £280 + 300 × £3,500 = £168,000 + £1,050,000 = £1,218,000.
    let gross =
        ea::gross_saving_attendances_and_admissions(attendances, 280.0, admissions, 3_500.0);
    assert!((gross - 1_218_000.0).abs() < 1e-6);
    // Net of £700k virtual-ward service and £120k community-nurse pathway:
    // £1,218,000 − £700,000 − £120,000 = £398,000/year.
    let net = ea::net_saving(gross, 700_000.0, 120_000.0);
    assert!((net - 398_000.0).abs() < 1e-6);
    assert!(net > 0.0);
    // Hard-cash line: fewer escalation admissions end 4 agency shifts/week at
    // a £150 premium: 4 × £150 × 52 = £31,200/year, confirmed via hard_saving.
    let agency = hc::annual_bank_agency_saving(4.0, 150.0, 52.0);
    assert!((agency - 31_200.0).abs() < 1e-9);
    let hard = hc::hard_saving(4.0 * 52.0, 150.0, 0.0, 0.0, 8.0, 0.0, 0.0);
    assert!((hard - agency).abs() < 1e-9);
    // Whole package remains self-funding after a £50k licence.
    let cash_position = hc::net_of_licence(net + hard, 50_000.0);
    assert!((cash_position - 379_200.0).abs() < 1e-6);
}

// Locks down: end-to-end health-app funnel — downloads → engagement →
// effective dose → QALYs → £ — with the honest (dose-weighted) claim ~20×
// smaller than the naive one. All values hand-computed.
#[test]
fn scenario_health_app_funnel_from_downloads_to_monetized_qalys() {
    // 200,000 downloads; MAU 60,000; DAU 12,000; 10,000 users at trial dose.
    let stickiness = em::stickiness_percent(12_000.0, 60_000.0).unwrap();
    assert!((stickiness - 20.0).abs() < TOL); // the "healthy" benchmark
    let sessions = em::sessions_per_user(240_000.0, 60_000.0).unwrap();
    assert!((sessions - 4.0).abs() < TOL);
    let avg_dur = em::average_session_duration(720_000.0, 240_000.0).unwrap();
    assert!((avg_dur - 3.0).abs() < TOL); // minutes/session
    let fe = em::feature_engagement(10_000.0, 60_000.0).unwrap();
    assert!((fe - 1.0 / 6.0).abs() < 1e-9);
    // Effective dose: 10,000 / 200,000 = 5%; naive models overstate 20×.
    let share = em::effective_dose_share(10_000.0, 200_000.0).unwrap();
    assert!((share - 0.05).abs() < TOL);
    let factor = em::overstatement_factor(200_000.0, 10_000.0).unwrap();
    assert!((factor - 20.0).abs() < TOL);
    // Trial effect per at-dose user: EQ-5D 0.62 → 0.72 sustained 0.2 years
    // = 0.02 QALYs; per registered user = 0.02 × 0.05 = 0.001 QALYs.
    let per_completer = eq_5d::qaly_gain(0.62, 0.72, 0.2);
    assert!((per_completer - 0.02).abs() < 1e-9);
    let per_registered = em::population_effect(per_completer, share);
    assert!((per_registered - 0.001).abs() < 1e-12);
    // £20/user at £20,000/QALY; cohort total £4M via the retention route.
    let value_per_user = ue::health_value_per_acquired_user(per_registered, 20_000.0);
    assert!((value_per_user - 20.0).abs() < 1e-9);
    let total = rc::monetized_health_value(
        rc::qalys_delivered(rc::completers(200_000.0, share), per_completer),
        20_000.0,
    );
    assert!((total - 4_000_000.0).abs() < 1e-6);
    assert!((value_per_user * 200_000.0 - total).abs() < 1e-6);
    // Unit economics: £1M spend / 200,000 = £5 CAC; at 5% effective-dose
    // retention that is £100 per effective user — above the £20 health value,
    // so the health case alone does not fund acquisition.
    let cac = ue::cac(1_000_000.0, 200_000.0).unwrap();
    assert!((cac - 5.0).abs() < TOL);
    let effective_cac = ue::effective_cac_per_retained_user(cac, share).unwrap();
    assert!((effective_cac - 100.0).abs() < TOL);
    assert!(value_per_user < effective_cac);
    // Commercial side: £6.99 ARPU at 18% churn misses 3:1 against £38 CAC;
    // the PMPM pivot at £1.20 × 40,000 lives yields £48k/month at 75% margin.
    let arpu = ue::arpu(419_400.0, 60_000.0).unwrap();
    assert!((arpu - 6.99).abs() < 1e-9);
    let ltv = ue::ltv(arpu, 0.18).unwrap();
    assert!(!ue::is_viable_ltv_cac(ltv, 38.0));
    assert!(ue::ltv_cac_ratio(ltv, 38.0).unwrap() < 3.0);
    assert!((ue::pmpm_revenue(1.20, 40_000.0, 1.0) - 48_000.0).abs() < TOL);
    assert!((ue::pmpm_margin(1.20, 0.30) - 0.90).abs() < TOL);
    assert!((ue::pmpm_margin_fraction(1.20, 0.30).unwrap() - 0.75).abs() < TOL);
}

// Locks down: EVPI-gated pilot decision — NMBs built from the
// earlier-intervention model, EVPI bounding pilot spend, and the PSA-draw
// route agreeing with the discrete worlds. All values hand-computed.
#[test]
fn scenario_evpi_gated_pilot_for_ai_grading_rollout() {
    // Rollout of AI retinopathy grading. If it works (p = 0.6): 4,000-patient
    // backlog reviewed 4 months sooner at 2%/year progression, £20,000 per
    // avoided progression, £300,000 running cost.
    let events = ei::progression_events_avoided(4_000.0, 0.02, 4.0 / 12.0);
    let per_event = ei::value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0);
    assert!((per_event - 20_000.0).abs() < TOL);
    let benefit = ei::total_backlog_value(events, per_event);
    assert!((benefit - 533_333.333_333_333_3).abs() < 1e-6); // 80/3 × 20,000
    let nmb_works = benefit - 300_000.0; // ≈ £233,333
    // If it fails (p = 0.4): no clinical benefit, cost only → −£300,000.
    let nmb_fails = -300_000.0;
    let worlds = vec![
        Scenario { probability: 0.6, option_nmbs: vec![nmb_works, 0.0] },
        Scenario { probability: 0.4, option_nmbs: vec![nmb_fails, 0.0] },
    ];
    // Decide now: 0.6 × 233,333.33 − 0.4 × 300,000 = £20,000 → still roll out.
    let decide_now = evpi_mod::expected_nmb_of_best_option(&worlds).unwrap();
    assert!((decide_now - 20_000.0).abs() < 1e-6);
    // Perfect information: 0.6 × 233,333.33 + 0.4 × 0 = £140,000.
    let perfect = evpi_mod::expected_nmb_with_perfect_information(&worlds).unwrap();
    assert!((perfect - 140_000.0).abs() < 1e-6);
    // EVPI = £120,000: a £50k pilot is funded, a £150k pilot is theater.
    let v = evpi_mod::evpi(&worlds).unwrap();
    assert!((v - 120_000.0).abs() < 1e-6);
    assert!(50_000.0 < v && v < 150_000.0);
    // Same answer from 10 equally weighted PSA draws (6 work / 4 fail).
    let mut draws = vec![vec![nmb_works, 0.0]; 6];
    draws.extend(vec![vec![nmb_fails, 0.0]; 4]);
    let v_psa = evpi_mod::evpi_from_psa_draws(&draws).unwrap();
    assert!((v_psa - v).abs() < 1e-6);
    // The evidence transfers to 5 comparable trusts: population EVPI £600k.
    let pop = evpi_mod::population_evpi(v, 5.0);
    assert!((pop - 600_000.0).abs() < 1e-6);
}

// Locks down: end-to-end GDS channel-shift case with flow and HALE roll-up —
// a redesign's saving, its failure-demand protection, the queue economics of
// shipping it sooner, and the ministry-level HALE framing. Hand-computed.
#[test]
fn scenario_gds_redesign_with_flow_and_hale_rollup() {
    // NHS appointment service, 2M transactions/year: take-up 30% → 55%,
    // completion 84% → 93% (doc worked example).
    let take_up_before = gds::digital_take_up_percent(600_000.0, 2_000_000.0).unwrap();
    assert!((take_up_before - 30.0).abs() < TOL);
    let take_up_after = gds::digital_take_up_percent(1_100_000.0, 2_000_000.0).unwrap();
    assert!((take_up_after - 55.0).abs() < TOL);
    let completion_after = gds::completion_rate_percent(930.0, 1_000.0).unwrap();
    assert!((completion_after - 93.0).abs() < TOL);
    let satisfaction = gds::user_satisfaction_percent(820.0, 1_000.0).unwrap();
    assert!((satisfaction - 82.0).abs() < TOL);
    // Shift saving £1,475,000; failure demand falls £307,200 → £246,400.
    let shift = gds::channel_shift_saving(2_000_000.0, 0.25, 3.20, 0.25);
    assert!((shift - 1_475_000.0).abs() < 1e-6);
    let fd_before = gds::failure_demand_cost(2_000_000.0, 0.30, 0.84, 3.20);
    let fd_after = gds::failure_demand_cost(2_000_000.0, 0.55, 0.93, 3.20);
    assert!((fd_before - fd_after - 60_800.0).abs() < 1e-6);
    // Digital unit cost check: £500k service cost / 2M completions = £0.25.
    assert_eq!(gds::cost_per_transaction(500_000.0, 2_000_000.0), Some(0.25));
    // Delivery economics of the redesign team: WIP 12 at 3 items/week is a
    // 4-week cycle; WIP-limited to 6 it is 2 weeks, and at £8,000/week of
    // delay cost the queue policy alone banks 3 × 2 × £8,000 = £48,000/week.
    let before_ct = fm::littles_law_cycle_time(12.0, 3.0).unwrap();
    let after_ct = fm::littles_law_cycle_time(6.0, 3.0).unwrap();
    assert!((before_ct - 4.0).abs() < TOL);
    assert!((after_ct - 2.0).abs() < TOL);
    let saved = fm::delay_cost_eliminated(3.0, before_ct - after_ct, 8_000.0);
    assert!((saved - 48_000.0).abs() < 1e-9);
    // Ministry roll-up: better access averts 1,200 disability-weighted
    // healthy years across 400,000 users ≈ 0.003 years ≈ 1.1 days each.
    let per_person = hale_mod::hale_contribution_per_person(1_200.0, 400_000.0).unwrap();
    assert!((per_person - 0.003).abs() < 1e-12);
    let days = hale_mod::years_to_days(per_person);
    assert!((days - 1.09575).abs() < 1e-9);
    assert!((hale_mod::hale_gap(73.3, 61.9) - 11.4).abs() < TOL);
}
