//! Comprehensive integration tests, group 3.
//!
//! Modules under test:
//! - cost_of_delay
//! - cost_utility_analysis
//! - did_not_attend_rate
//! - diga_fast_track
//! - digital_endpoints_and_biomarkers
//! - disability_adjusted_life_year
//! - discounting_and_time_preference
//! - dominance_and_efficiency_frontier
//! - dora_metrics
//! - downstream_resource_optimization
//!
//! Sections:
//!   1. EDGE CASES
//!   2. PROPERTIES / INVARIANTS
//!   3. CROSS-MODULE CONSISTENCY
//!   4. DOMAIN SCENARIOS

use health_economics::cost_of_delay as cod;
use health_economics::cost_utility_analysis as cua;
use health_economics::did_not_attend_rate as dna;
use health_economics::diga_fast_track as diga;
use health_economics::digital_endpoints_and_biomarkers as deb;
use health_economics::disability_adjusted_life_year as daly;
use health_economics::discounting_and_time_preference as disc;
use health_economics::dominance_and_efficiency_frontier as dom;
use health_economics::dora_metrics as dora;
use health_economics::downstream_resource_optimization as dro;

// Sibling modules used only for cross-module consistency checks.
use health_economics::avoided_downstream_costs as adc;
use health_economics::cost_benefit_analysis as cba;
use health_economics::quality_adjusted_life_year as qalym;

use cua::HealthState;
use daly::WhoChoiceBand;
use dom::Alternative;
use dro::DownstreamRelease;

/// Absolute-difference float comparison used throughout.
fn approx(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

// =========================================================================
// 1. EDGE CASES
// =========================================================================

// ---- cost_of_delay ------------------------------------------------------

// Edge: zero saving or zero throughput yields zero CoD, and 1e12 magnitudes stay finite.
#[test]
fn edge_cod_operational_zero_and_extreme() {
    assert_eq!(cod::operational_cost_of_delay(0.0, 50.0), 0.0);
    assert_eq!(cod::operational_cost_of_delay(200.0, 0.0), 0.0);
    let huge = cod::operational_cost_of_delay(1e12, 1e12);
    assert!(huge.is_finite() && huge > 0.0);
}

// Edge: zero delay loses nothing; negative delay (early delivery) flips the sign of the loss.
#[test]
fn edge_cod_total_delay_loss_zero_and_negative_delay() {
    assert_eq!(cod::total_delay_loss(10_000.0, 0.0), 0.0);
    assert!(approx(cod::total_delay_loss(10_000.0, -2.0), -20_000.0, 1e-9));
    assert!(cod::total_delay_loss(1e12, 1e12).is_finite());
}

// Edge: zero waiting removed or zero utility gain gives zero QALY gain; a negative
// utility gain (new state worse) gives a negative QALY gain.
#[test]
fn edge_cod_qaly_gain_zero_and_negative_utility_gain() {
    assert_eq!(cod::qaly_gain_per_patient(0.0, 0.12), 0.0);
    assert_eq!(cod::qaly_gain_per_patient(5.0, 0.0), 0.0);
    assert!(cod::qaly_gain_per_patient(5.0, -0.12) < 0.0);
}

// Edge: health CoD is zero with no patients or no per-patient gain.
#[test]
fn edge_cod_health_zero_inputs() {
    assert_eq!(cod::cost_of_delay_health(0.0, 0.0115), 0.0);
    assert_eq!(cod::cost_of_delay_health(100.0, 0.0), 0.0);
}

// Edge: money CoD with all-zero terms is zero, and the operational term alone passes through.
#[test]
fn edge_cod_money_zero_and_operational_only() {
    assert_eq!(cod::cost_of_delay_money(0.0, 20_000.0, 0.0), 0.0);
    assert_eq!(cod::cost_of_delay_money(0.0, 20_000.0, 5_000.0), 5_000.0);
    assert!(cod::cost_of_delay_money(1e12, 1e12, 1e12).is_finite());
}

// ---- cost_utility_analysis ----------------------------------------------

// Edge: an empty pathway accrues zero QALYs; a single state accrues duration × utility.
#[test]
fn edge_cua_total_qalys_empty_and_single() {
    assert_eq!(cua::total_qalys(&[]), 0.0);
    let one = [HealthState { duration_years: 2.0, utility: 0.5 }];
    assert!(approx(cua::total_qalys(&one), 1.0, 1e-12));
}

// Edge: utility boundaries — 0 (dead-equivalent) accrues nothing, 1 (full health)
// accrues the full duration, and a below-zero (worse-than-dead) utility accrues negative QALYs.
#[test]
fn edge_cua_total_qalys_utility_boundaries_and_negative() {
    let dead = [HealthState { duration_years: 3.0, utility: 0.0 }];
    let full = [HealthState { duration_years: 3.0, utility: 1.0 }];
    let worse = [HealthState { duration_years: 1.0, utility: -0.2 }];
    assert_eq!(cua::total_qalys(&dead), 0.0);
    assert!(approx(cua::total_qalys(&full), 3.0, 1e-12));
    assert!(approx(cua::total_qalys(&worse), -0.2, 1e-12));
}

// Edge: two empty pathways have zero QALY difference.
#[test]
fn edge_cua_delta_qalys_empty_vs_empty() {
    assert_eq!(cua::delta_qalys(&[], &[]), 0.0);
}

// Edge: displacement fraction boundaries 0.0 (no saving) and 1.0 (full comparator cost).
#[test]
fn edge_cua_displaced_care_saving_fraction_boundaries() {
    assert_eq!(cua::displaced_care_saving(0.0, 1_700.0), 0.0);
    assert_eq!(cua::displaced_care_saving(1.0, 1_700.0), 1_700.0);
}

// Edge: icur is None exactly when ΔQALYs == 0.0 (including -0.0); nonzero ΔE gives Some.
#[test]
fn edge_cua_icur_none_only_on_zero_delta_qalys() {
    assert_eq!(cua::icur(100.0, 0.0), None);
    assert_eq!(cua::icur(100.0, -0.0), None); // -0.0 == 0.0 in f64
    assert_eq!(cua::icur(0.0, 0.04), Some(0.0));
    assert_eq!(cua::icur(-80.0, -0.04), Some(2_000.0)); // dominated quadrant still a ratio
}

// Edge: dominance boundaries — ΔC = 0 or ΔE = 0 is NOT dominance (both strict).
#[test]
fn edge_cua_is_dominant_boundaries() {
    assert!(!cua::is_dominant(0.0, 0.04));
    assert!(!cua::is_dominant(-430.0, 0.0));
    assert!(cua::is_dominant(-1e-12, 1e-12));
}

// ---- did_not_attend_rate --------------------------------------------------

// Edge: DNA rate is None only when booked appointments are zero; 0 and 100% bounds work.
#[test]
fn edge_dna_rate_none_and_bounds() {
    assert_eq!(dna::dna_rate_percent(5.0, 0.0), None);
    assert_eq!(dna::dna_rate_percent(0.0, 100.0), Some(0.0));
    assert_eq!(dna::dna_rate_percent(100.0, 100.0), Some(100.0));
}

// Edge: zero-point reduction recovers no slots; a worsening (negative reduction) is negative.
#[test]
fn edge_dna_recovered_slots_zero_and_negative() {
    assert_eq!(dna::recovered_slots(200_000.0, 0.0), 0.0);
    assert!(dna::recovered_slots(200_000.0, -1.0) < 0.0);
    assert!(dna::recovered_slots(1e12, 100.0).is_finite());
}

// Edge: zero slots or zero per-slot value recovers zero value.
#[test]
fn edge_dna_value_of_reduction_zero_inputs() {
    assert_eq!(dna::value_of_reduction(0.0, 160.0), 0.0);
    assert_eq!(dna::value_of_reduction(5_000.0, 0.0), 0.0);
}

// Edge: a free service or no appointments costs nothing.
#[test]
fn edge_dna_service_cost_zero_inputs() {
    assert_eq!(dna::service_cost(0.0, 0.40), 0.0);
    assert_eq!(dna::service_cost(200_000.0, 0.0), 0.0);
}

// Edge: return ratio is None only when the service cost is zero; zero value gives 0.
#[test]
fn edge_dna_return_ratio_none_only_on_zero_cost() {
    assert_eq!(dna::return_ratio(800_000.0, 0.0), None);
    assert_eq!(dna::return_ratio(0.0, 80_000.0), Some(0.0));
}

// Edge: relative reduction is None only for a zero baseline; a worsening rate is negative.
#[test]
fn edge_dna_relative_reduction_none_and_negative() {
    assert_eq!(dna::relative_reduction(0.0, 5.0), None);
    let worse = dna::relative_reduction(5.0, 8.0).unwrap();
    assert!(worse < 0.0 && approx(worse, -0.6, 1e-12));
}

// ---- diga_fast_track -------------------------------------------------------

// Edge: activation-rate boundaries 0.0 (no revenue) and 1.0 (every script reimbursed).
#[test]
fn edge_diga_revenue_activation_boundaries() {
    assert_eq!(diga::revenue(20_000.0, 0.0, 450.0), 0.0);
    assert_eq!(diga::revenue(20_000.0, 1.0, 450.0), 9_000_000.0);
    assert!(diga::revenue(1e12, 1.0, 1e12).is_finite());
}

// Edge: zero prescriptions activate to zero; rate boundaries pass through.
#[test]
fn edge_diga_activated_prescriptions_boundaries() {
    assert_eq!(diga::activated_prescriptions(0.0, 0.81), 0.0);
    assert_eq!(diga::activated_prescriptions(20_000.0, 0.0), 0.0);
    assert_eq!(diga::activated_prescriptions(20_000.0, 1.0), 20_000.0);
}

// Edge: probability boundaries — P = 0 loses exactly the RCT cost, P = 1 nets revenue − cost.
#[test]
fn edge_diga_expected_value_probability_boundaries() {
    assert_eq!(diga::expected_value(0.0, 18_468_000.0, 2_000_000.0), -2_000_000.0);
    assert_eq!(diga::expected_value(1.0, 18_468_000.0, 2_000_000.0), 16_468_000.0);
}

// Edge: the financing test is inclusive — revenue exactly equal to the RCT cost counts (>=).
#[test]
fn edge_diga_financing_boundary_is_inclusive() {
    assert!(diga::provisional_year_finances_evidence(2_000_000.0, 2_000_000.0));
    assert!(!diga::provisional_year_finances_evidence(1_999_999.99, 2_000_000.0));
    assert!(diga::provisional_year_finances_evidence(0.0, 0.0)); // zero-cost evidence
}

// ---- digital_endpoints_and_biomarkers ---------------------------------------

// Edge: sample size is None only when the detectable effect is zero; zero variance needs 0 patients.
#[test]
fn edge_deb_required_sample_size_none_and_zero_variance() {
    assert_eq!(deb::required_sample_size(1.0, 0.0, 16.0), None);
    assert_eq!(deb::required_sample_size(0.0, 1.0, 16.0), Some(0.0));
    assert!(deb::required_sample_size(1e12, 1.0, 16.0).unwrap().is_finite());
}

// Edge: density ratio is None only when the clinic count is zero; zero digital sampling gives 0.
#[test]
fn edge_deb_sampling_density_ratio_none_and_zero() {
    assert_eq!(deb::sampling_density_ratio(200.0, 0.0), None);
    assert_eq!(deb::sampling_density_ratio(0.0, 4.0), Some(0.0));
}

// Edge: a 1× (no) variance fall improves nothing; 0× collapses to 0; 1e12 stays finite.
#[test]
fn edge_deb_detectable_effect_improvement_boundaries() {
    assert_eq!(deb::detectable_effect_improvement(1.0), 1.0);
    assert_eq!(deb::detectable_effect_improvement(0.0), 0.0);
    assert!(approx(deb::detectable_effect_improvement(1e12), 1e6, 1e-3));
}

// Edge: sample size ratio is None only when the old variance is zero; zero new variance gives 0.
#[test]
fn edge_deb_sample_size_ratio_none_and_zero() {
    assert_eq!(deb::sample_size_ratio(1.0, 0.0), None);
    assert_eq!(deb::sample_size_ratio(0.0, 5.0), Some(0.0));
}

// Edge: cutting zero patients (or free patients) saves nothing.
#[test]
fn edge_deb_trial_cost_saving_zero_inputs() {
    assert_eq!(deb::trial_cost_saving(0.0, 25_000.0), 0.0);
    assert_eq!(deb::trial_cost_saving(200.0, 0.0), 0.0);
    assert!(deb::trial_cost_saving(1e12, 1e12).is_finite());
}

// ---- disability_adjusted_life_year ------------------------------------------

// Edge: zero deaths or zero remaining life expectancy loses zero life years.
#[test]
fn edge_daly_yll_zero_inputs() {
    assert_eq!(daly::years_of_life_lost(0.0, 20.0), 0.0);
    assert_eq!(daly::years_of_life_lost(10.0, 0.0), 0.0);
}

// Edge: disability-weight boundaries — 0 (full health) yields no YLD, 1 (death-equivalent)
// yields the full prevalence.
#[test]
fn edge_daly_yld_weight_boundaries() {
    assert_eq!(daly::years_lived_with_disability(200.0, 0.0), 0.0);
    assert_eq!(daly::years_lived_with_disability(200.0, 1.0), 200.0);
}

// Edge: zero burden on both components gives zero DALYs; large components stay finite.
#[test]
fn edge_daly_dalys_zero_and_extreme() {
    assert_eq!(daly::dalys(0.0, 0.0), 0.0);
    assert!(daly::dalys(1e12, 1e12).is_finite());
}

// Edge: cost per DALY averted is None only when zero DALYs are averted.
#[test]
fn edge_daly_cost_per_daly_none_and_free() {
    assert_eq!(daly::cost_per_daly_averted(600_000.0, 0.0), None);
    assert_eq!(daly::cost_per_daly_averted(0.0, 240.0), Some(0.0));
}

// Edge: WHO-CHOICE boundary equality — exactly 1× GDP is CostEffective (the < is strict),
// exactly 3× GDP is CostEffective (the <= is inclusive).
#[test]
fn edge_daly_who_choice_exact_boundaries() {
    let gdp = 8_000.0;
    assert_eq!(daly::who_choice_band(gdp, gdp), WhoChoiceBand::CostEffective);
    assert_eq!(daly::who_choice_band(3.0 * gdp, gdp), WhoChoiceBand::CostEffective);
    assert_eq!(daly::who_choice_band(0.0, gdp), WhoChoiceBand::HighlyCostEffective);
    assert_eq!(
        daly::who_choice_band(3.0 * gdp + 1e-6, gdp),
        WhoChoiceBand::NotCostEffective
    );
}

// ---- discounting_and_time_preference ----------------------------------------

// Edge: year 0 leaves any amount undiscounted (factor exactly 1.0).
#[test]
fn edge_disc_present_value_year_zero_is_identity() {
    assert_eq!(disc::present_value(100_000.0, disc::NICE_REFERENCE_RATE, 0.0), 100_000.0);
    assert_eq!(disc::present_value(1.0, 0.5, 0.0), 1.0);
}

// Edge: a 0.0 rate never discounts, at any horizon.
#[test]
fn edge_disc_present_value_zero_rate_is_identity() {
    for t in [0.0, 1.0, 10.0, 100.0] {
        assert!(approx(disc::present_value(42.0, 0.0, t), 42.0, 1e-12));
    }
}

// Edge: annuity r = 0 limit is exactly B × n, and a 0-year annuity is worth 0 at any rate.
#[test]
fn edge_disc_annuity_rate_zero_limit_and_zero_years() {
    assert_eq!(disc::annuity_present_value(100_000.0, 0.0, 5.0), 500_000.0);
    assert_eq!(disc::annuity_present_value(100_000.0, 0.0, 0.0), 0.0);
    assert!(approx(disc::annuity_present_value(100_000.0, 0.035, 0.0), 0.0, 1e-9));
}

// Edge: a zero-year delay leaves the PV unchanged; extreme horizons stay finite and positive.
#[test]
fn edge_disc_delayed_pv_zero_delay_and_extreme_horizon() {
    assert_eq!(disc::delayed_present_value(451_505.0, 0.035, 0.0), 451_505.0);
    let far = disc::present_value(1e12, 0.035, 100.0);
    assert!(far.is_finite() && far > 0.0);
}

// ---- dominance_and_efficiency_frontier ----------------------------------------

// Edge: identical options do not dominate each other (a strict inequality is required).
#[test]
fn edge_dom_identical_options_do_not_dominate() {
    let a = Alternative::new("A", 100.0, 10.0);
    let b = Alternative::new("B", 100.0, 10.0);
    assert!(!dom::strictly_dominates(&a, &b));
    assert!(!dom::strictly_dominates(&b, &a));
}

// Edge: a tie on one axis with a strict win on the other IS dominance.
#[test]
fn edge_dom_tie_plus_strict_win_dominates() {
    let cheap = Alternative::new("cheap", 50.0, 10.0);
    let dear = Alternative::new("dear", 80.0, 10.0);
    let strong = Alternative::new("strong", 50.0, 12.0);
    assert!(dom::strictly_dominates(&cheap, &dear)); // same effect, cheaper
    assert!(dom::strictly_dominates(&strong, &cheap)); // same cost, more effect
}

// Edge: icer is None exactly when the two effects are equal (ΔE = 0).
#[test]
fn edge_dom_icer_none_on_equal_effects() {
    let a = Alternative::new("A", 10.0, 5.0);
    let b = Alternative::new("B", 90.0, 5.0);
    assert_eq!(dom::icer(&b, &a), None);
    assert_eq!(dom::icer(&a, &a), None);
}

// Edge: empty input yields an empty frontier; a single option is its own frontier.
#[test]
fn edge_dom_frontier_empty_and_single() {
    assert!(dom::efficiency_frontier(&[]).is_empty());
    let solo = vec![Alternative::new("only", 5.0, 1.0)];
    assert_eq!(dom::efficiency_frontier(&solo), solo);
    assert!(dom::frontier_icers(&solo).is_empty());
    assert!(dom::frontier_icers(&[]).is_empty());
}

// Edge: exact duplicates both survive the frontier (neither strictly dominates),
// and their pairwise ICER is None.
#[test]
fn edge_dom_frontier_keeps_exact_duplicates() {
    let options = vec![
        Alternative::new("dup1", 10.0, 2.0),
        Alternative::new("dup2", 10.0, 2.0),
    ];
    let frontier = dom::efficiency_frontier(&options);
    assert_eq!(frontier.len(), 2);
    assert_eq!(dom::frontier_icers(&frontier), vec![None]);
}

// Edge: of two equal-effect options, only the cheaper survives.
#[test]
fn edge_dom_frontier_drops_dearer_equal_effect_option() {
    let options = vec![
        Alternative::new("base", 0.0, 0.0),
        Alternative::new("cheap", 50.0, 5.0),
        Alternative::new("dear", 80.0, 5.0),
    ];
    let frontier = dom::efficiency_frontier(&options);
    let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, vec!["base", "cheap"]);
}

// ---- dora_metrics --------------------------------------------------------------

// Edge: deployment frequency is None only for a zero-length period.
#[test]
fn edge_dora_deployment_frequency_none_and_zero_deploys() {
    assert_eq!(dora::deployment_frequency(12.0, 0.0), None);
    assert_eq!(dora::deployment_frequency(0.0, 1.0), Some(0.0));
}

// Edge: CFR is None only with zero total changes; 0% and 100% bounds hold.
#[test]
fn edge_dora_cfr_none_and_bounds() {
    assert_eq!(dora::change_failure_rate_percent(1.0, 0.0), None);
    assert_eq!(dora::change_failure_rate_percent(0.0, 40.0), Some(0.0));
    assert_eq!(dora::change_failure_rate_percent(40.0, 40.0), Some(100.0));
}

// Edge: 0 days is 0 weeks and 7 days is exactly 1 week.
#[test]
fn edge_dora_days_to_weeks_boundaries() {
    assert_eq!(dora::days_to_weeks(0.0), 0.0);
    assert_eq!(dora::days_to_weeks(7.0), 1.0);
}

// Edge: a worsened lead time yields a negative reduction.
#[test]
fn edge_dora_lead_time_reduction_negative_when_worse() {
    assert!(approx(dora::lead_time_reduction_weeks(2.0, 6.0), -4.0, 1e-12));
    assert_eq!(dora::lead_time_reduction_weeks(6.0, 6.0), 0.0);
}

// Edge: value pulled forward is zero whenever any factor is zero, finite at 1e12.
#[test]
fn edge_dora_value_pulled_forward_zeros_and_extreme() {
    assert_eq!(dora::value_pulled_forward(0.0, 5.4, 4_000.0), 0.0);
    assert_eq!(dora::value_pulled_forward(30.0, 0.0, 4_000.0), 0.0);
    assert_eq!(dora::value_pulled_forward(30.0, 5.4, 0.0), 0.0);
    assert!(dora::value_pulled_forward(1e12, 1.0, 1e12).is_finite());
}

// Edge: a worsening CFR gives NEGATIVE failed changes avoided (and negative cost avoided).
#[test]
fn edge_dora_failed_changes_avoided_negative_when_cfr_worsens() {
    assert!(approx(dora::failed_changes_avoided(30.0, 0.05, 0.25), -6.0, 1e-12));
    assert!(dora::failure_cost_avoided(30.0, 0.05, 0.25, 15_000.0) < 0.0);
    assert_eq!(dora::failed_changes_avoided(30.0, 0.25, 0.25), 0.0);
}

// Edge: instant recovery or a harmless outage costs nothing.
#[test]
fn edge_dora_downtime_harm_zero_inputs() {
    assert_eq!(dora::downtime_harm(0.0, 1_000.0), 0.0);
    assert_eq!(dora::downtime_harm(48.0, 0.0), 0.0);
}

// Edge: SLO attainment boundaries — 0.0 delivers nothing, 1.0 delivers the full modeled benefit.
#[test]
fn edge_dora_reliability_adjusted_benefit_boundaries() {
    assert_eq!(dora::reliability_adjusted_benefit(500_000.0, 0.0), 0.0);
    assert_eq!(dora::reliability_adjusted_benefit(500_000.0, 1.0), 500_000.0);
}

// ---- downstream_resource_optimization -------------------------------------------

// Edge: no downstream releases and no throughput gain is worth exactly zero
// (the non-gating-role contrast), and empty releases with a gain keep the pathway term.
#[test]
fn edge_dro_value_of_unblocking_empty_and_pathway_only() {
    assert_eq!(dro::value_of_unblocking(&[], 0.0, 300.0), 0.0);
    assert_eq!(dro::value_of_unblocking(&[], 10.0, 300.0), 3_000.0);
    let free_role = [DownstreamRelease { blocked_hours_released: 100.0, unit_cost_per_hour: 0.0 }];
    assert_eq!(dro::value_of_unblocking(&free_role, 0.0, 0.0), 0.0);
}

// Edge: a gating task that got slower saves negative minutes.
#[test]
fn edge_dro_gating_task_negative_when_slower() {
    assert!(approx(dro::gating_task_minutes_saved(20.0, 90.0), -70.0, 1e-12));
    assert_eq!(dro::gating_task_minutes_saved(90.0, 90.0), 0.0);
}

// Edge: annualizing zero per-day or zero operating days yields zero; 1e12 stays finite.
#[test]
fn edge_dro_annualize_zero_and_extreme() {
    assert_eq!(dro::annualize(0.0, 365.0), 0.0);
    assert_eq!(dro::annualize(3.0, 0.0), 0.0);
    assert!(dro::annualize(1e12, 365.0).is_finite());
}

// Edge: recovering zero late discharges avoids zero bed days.
#[test]
fn edge_dro_bed_days_zero_recovered() {
    assert_eq!(dro::bed_days_avoided_per_year(0.0, 365.0), 0.0);
}

// =========================================================================
// 2. PROPERTIES / INVARIANTS
// =========================================================================

// Property: the discount factor PV(1, r, t) lies in (0, 1] for r >= 0 and is strictly
// decreasing in years for r > 0.
#[test]
fn prop_disc_discount_factor_in_unit_interval_and_decreasing() {
    for &r in &[0.0, 0.015, 0.035, 0.10, 0.50] {
        let mut prev = f64::INFINITY;
        for t in 0..=30 {
            let f = disc::present_value(1.0, r, t as f64);
            assert!(f > 0.0 && f <= 1.0, "factor out of (0,1]: r={r} t={t} f={f}");
            if r > 0.0 {
                assert!(f < prev, "not strictly decreasing: r={r} t={t}");
            } else {
                assert!(approx(f, 1.0, 1e-12));
            }
            prev = f;
        }
    }
}

// Property: the present value of a delayed amount/stream is strictly below the undelayed one
// for any positive rate and delay.
#[test]
fn prop_disc_delay_strictly_reduces_present_value() {
    for &r in &[0.015, 0.035, 0.10] {
        for &delay in &[0.5, 1.0, 3.0, 10.0] {
            let pv = disc::annuity_present_value(100_000.0, r, 5.0);
            let slipped = disc::delayed_present_value(pv, r, delay);
            assert!(slipped < pv, "delay did not reduce PV: r={r} delay={delay}");
            // The slipped stream equals each future term discounted once more.
            assert!(approx(slipped, pv * disc::present_value(1.0, r, delay), 1e-6));
        }
    }
}

// Property: the annuity closed form equals the year-by-year sum of present values over a grid.
#[test]
fn prop_disc_annuity_equals_year_by_year_sum() {
    for &r in &[0.015, 0.035, 0.08, 0.20] {
        for &n in &[1u32, 2, 5, 10, 30] {
            let sum: f64 = (1..=n).map(|t| disc::present_value(1_000.0, r, t as f64)).sum();
            let annuity = disc::annuity_present_value(1_000.0, r, n as f64);
            assert!(approx(annuity, sum, 1e-6), "annuity != sum at r={r} n={n}");
        }
    }
}

// Property: QALY totals are additive — total of a concatenated state list equals the sum
// of the parts' totals, over a grid of durations and utilities.
#[test]
fn prop_cua_total_qalys_additive_over_concatenation() {
    let utilities = [0.0, 0.25, 0.5, 0.76, 1.0];
    let durations = [0.1, 0.5, 1.0, 2.0];
    let mut part_a: Vec<HealthState> = Vec::new();
    let mut part_b: Vec<HealthState> = Vec::new();
    for (i, &d) in durations.iter().enumerate() {
        for (j, &u) in utilities.iter().enumerate() {
            let s = HealthState { duration_years: d, utility: u };
            if (i + j) % 2 == 0 { part_a.push(s) } else { part_b.push(s) }
        }
    }
    let mut concat = part_a.clone();
    concat.extend_from_slice(&part_b);
    assert!(approx(
        cua::total_qalys(&concat),
        cua::total_qalys(&part_a) + cua::total_qalys(&part_b),
        1e-12
    ));
}

// Property: delta_qalys is antisymmetric — delta(new, old) = −delta(old, new) over a grid.
#[test]
fn prop_cua_delta_qalys_antisymmetric() {
    let pathways = [
        vec![HealthState { duration_years: 0.5, utility: 0.76 }],
        vec![HealthState { duration_years: 0.5, utility: 0.68 }],
        vec![
            HealthState { duration_years: 1.0, utility: 0.9 },
            HealthState { duration_years: 2.0, utility: 0.4 },
        ],
        vec![],
    ];
    for a in &pathways {
        for b in &pathways {
            assert!(approx(cua::delta_qalys(a, b), -cua::delta_qalys(b, a), 1e-12));
        }
    }
}

// Property: is_dominant is true exactly in the ΔC < 0, ΔE > 0 quadrant.
#[test]
fn prop_cua_is_dominant_quadrant() {
    for &dc in &[-10.0, 0.0, 10.0] {
        for &de in &[-1.0, 0.0, 1.0] {
            assert_eq!(cua::is_dominant(dc, de), dc < 0.0 && de > 0.0);
        }
    }
}

// Property: displaced care saving is linear in both the fraction and the comparator cost.
#[test]
fn prop_cua_displaced_care_saving_linear() {
    for &f in &[0.0, 0.1, 0.4, 1.0] {
        for &c in &[0.0, 500.0, 1_700.0] {
            assert!(approx(cua::displaced_care_saving(2.0 * f, c), 2.0 * cua::displaced_care_saving(f, c), 1e-9));
            assert!(approx(cua::displaced_care_saving(f, 3.0 * c), 3.0 * cua::displaced_care_saving(f, c), 1e-9));
        }
    }
}

// Property: WHO-CHOICE banding over a grid of GDP multiples — < 1× is HighlyCostEffective,
// 1×..=3× (inclusive both ends) is CostEffective, above 3× is NotCostEffective.
#[test]
fn prop_daly_who_choice_banding_grid() {
    for &gdp in &[800.0, 8_000.0, 50_000.0] {
        for &m in &[0.0, 0.25, 0.5, 0.999, 1.0, 1.5, 2.0, 2.999, 3.0, 3.5, 10.0] {
            let cost = m * gdp;
            let expected = if cost < gdp {
                WhoChoiceBand::HighlyCostEffective
            } else if cost <= 3.0 * gdp {
                WhoChoiceBand::CostEffective
            } else {
                WhoChoiceBand::NotCostEffective
            };
            assert_eq!(daly::who_choice_band(cost, gdp), expected, "gdp={gdp} multiple={m}");
        }
    }
}

// Property: the DALY chain is linear — doubling deaths doubles YLL, doubling prevalence
// doubles YLD, and dalys() adds the components exactly.
#[test]
fn prop_daly_chain_linearity_and_additivity() {
    for &deaths in &[0.0, 1.0, 10.0, 500.0] {
        for &prev in &[0.0, 50.0, 200.0] {
            let yll = daly::years_of_life_lost(deaths, 20.0);
            let yld = daly::years_lived_with_disability(prev, 0.2);
            assert!(approx(daly::years_of_life_lost(2.0 * deaths, 20.0), 2.0 * yll, 1e-9));
            assert!(approx(daly::years_lived_with_disability(2.0 * prev, 0.2), 2.0 * yld, 1e-9));
            assert!(approx(daly::dalys(yll, yld), yll + yld, 1e-12));
        }
    }
}

// Property: cost per DALY averted scales inversely with DALYs averted (double the DALYs,
// halve the cost per DALY).
#[test]
fn prop_daly_cost_per_daly_inverse_scaling() {
    for &averted in &[1.0, 24.0, 240.0, 1e6] {
        let one = daly::cost_per_daly_averted(600_000.0, averted).unwrap();
        let two = daly::cost_per_daly_averted(600_000.0, 2.0 * averted).unwrap();
        assert!(approx(two, one / 2.0, 1e-9 * one.max(1.0)));
    }
}

// Property (frontier invariants): for varied input sets the frontier is a subset of the
// inputs, strictly sorted by effect and cost, no member is strictly dominated by ANY input,
// and frontier ICERs are strictly increasing where Some.
#[test]
fn prop_dom_frontier_invariants_over_input_sets() {
    let input_sets: Vec<Vec<Alternative>> = vec![
        // Worked example (one strictly dominated option).
        vec![
            Alternative::new("Do nothing", 0.0, 0.0),
            Alternative::new("SMS reminders", 20_000.0, 2_000.0),
            Alternative::new("Phone calls", 120_000.0, 2_200.0),
            Alternative::new("SMS + AI triage", 90_000.0, 3_500.0),
        ],
        // Extended-dominance case.
        vec![
            Alternative::new("A", 0.0, 0.0),
            Alternative::new("B", 100.0, 1.0),
            Alternative::new("C", 120.0, 2.0),
        ],
        // All options survive (increasing ICERs).
        vec![
            Alternative::new("W", 0.0, 0.0),
            Alternative::new("X", 10.0, 1.0),
            Alternative::new("Y", 30.0, 2.0),
            Alternative::new("Z", 60.0, 3.0),
        ],
        // Unsorted input with a mix of strict and extended dominance.
        vec![
            Alternative::new("p", 500.0, 40.0),
            Alternative::new("q", 50.0, 10.0),
            Alternative::new("r", 400.0, 12.0),  // strictly dominated by p? no: cheaper than p, but s dominates
            Alternative::new("s", 60.0, 30.0),
            Alternative::new("t", 0.0, 0.0),
        ],
    ];
    for options in &input_sets {
        let frontier = dom::efficiency_frontier(options);
        // Subset of the inputs.
        for member in &frontier {
            assert!(options.contains(member), "frontier member not from inputs: {member:?}");
        }
        // Strictly sorted by effect and by cost.
        for w in frontier.windows(2) {
            assert!(w[0].effect < w[1].effect, "effects not increasing");
            assert!(w[0].cost < w[1].cost, "costs not increasing");
        }
        // No frontier member is strictly dominated by any input option.
        for member in &frontier {
            for other in options {
                assert!(
                    !dom::strictly_dominates(other, member),
                    "{:?} dominates frontier member {:?}",
                    other.name,
                    member.name
                );
            }
        }
        // ICERs strictly increasing where defined.
        let icers = dom::frontier_icers(&frontier);
        let defined: Vec<f64> = icers.into_iter().flatten().collect();
        for w in defined.windows(2) {
            assert!(w[0] < w[1], "frontier ICERs not strictly increasing");
        }
    }
}

// Property (extended dominance): the middle option is removed even though nothing strictly
// dominates it, because a mix of its neighbors buys effect more cheaply.
#[test]
fn prop_dom_extended_dominance_removes_undominated_middle() {
    let a = Alternative::new("A", 0.0, 0.0);
    let b = Alternative::new("B", 100.0, 1.0);
    let c = Alternative::new("C", 120.0, 2.0);
    // B is NOT strictly dominated by either neighbor...
    assert!(!dom::strictly_dominates(&a, &b));
    assert!(!dom::strictly_dominates(&c, &b));
    // ...yet leaves the frontier (ICER to B = 100 > ICER from B = 20).
    let frontier = dom::efficiency_frontier(&[a, b, c]);
    let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, vec!["A", "C"]);
}

// Property (all survive): with monotonically increasing ICERs no option is removed.
#[test]
fn prop_dom_all_options_survive_when_icers_increase() {
    let options = vec![
        Alternative::new("W", 0.0, 0.0),
        Alternative::new("X", 10.0, 1.0),
        Alternative::new("Y", 30.0, 2.0),
        Alternative::new("Z", 60.0, 3.0),
    ];
    let frontier = dom::efficiency_frontier(&options);
    assert_eq!(frontier.len(), 4);
    let icers: Vec<f64> = dom::frontier_icers(&frontier).into_iter().flatten().collect();
    assert_eq!(icers.len(), 3);
    assert!(approx(icers[0], 10.0, 1e-9));
    assert!(approx(icers[1], 20.0, 1e-9));
    assert!(approx(icers[2], 30.0, 1e-9));
}

// Property: value_pulled_forward is linear in each of its three arguments separately.
#[test]
fn prop_dora_value_pulled_forward_linear_in_each_argument() {
    let base = dora::value_pulled_forward(30.0, 5.4, 4_000.0);
    for &k in &[0.0, 0.5, 2.0, 10.0] {
        assert!(approx(dora::value_pulled_forward(30.0 * k, 5.4, 4_000.0), k * base, 1e-6));
        assert!(approx(dora::value_pulled_forward(30.0, 5.4 * k, 4_000.0), k * base, 1e-6));
        assert!(approx(dora::value_pulled_forward(30.0, 5.4, 4_000.0 * k), k * base, 1e-6));
    }
}

// Property: failed_changes_avoided = changes × (cfr_before − cfr_after) over a grid,
// including the negative (worsening) region, and failure_cost_avoided is that × cost.
#[test]
fn prop_dora_failed_changes_avoided_formula_grid() {
    for &changes in &[0.0, 10.0, 30.0, 250.0] {
        for &before in &[0.0, 0.05, 0.25, 0.40] {
            for &after in &[0.0, 0.05, 0.25, 0.40] {
                let avoided = dora::failed_changes_avoided(changes, before, after);
                assert!(approx(avoided, changes * (before - after), 1e-9));
                let cost = dora::failure_cost_avoided(changes, before, after, 15_000.0);
                assert!(approx(cost, avoided * 15_000.0, 1e-6));
                if after > before && changes > 0.0 {
                    assert!(avoided < 0.0, "worsening CFR must be negative");
                }
            }
        }
    }
}

// Property: downtime harm and reliability-adjusted benefit both scale linearly.
#[test]
fn prop_dora_downtime_and_reliability_linear() {
    for &k in &[0.0, 0.5, 2.0, 24.0] {
        assert!(approx(dora::downtime_harm(k * 2.0, 1_000.0), k * dora::downtime_harm(2.0, 1_000.0), 1e-9));
        assert!(approx(
            dora::reliability_adjusted_benefit(k * 100.0, 0.99),
            k * dora::reliability_adjusted_benefit(100.0, 0.99),
            1e-9
        ));
    }
}

// Property: DNA recovered_slots and value_of_reduction scale linearly in every argument.
#[test]
fn prop_dna_recovered_slots_and_value_scale_linearly() {
    let slots = dna::recovered_slots(200_000.0, 2.5);
    let value = dna::value_of_reduction(slots, 160.0);
    for &k in &[0.0, 0.5, 2.0, 4.0] {
        assert!(approx(dna::recovered_slots(200_000.0 * k, 2.5), k * slots, 1e-6));
        assert!(approx(dna::recovered_slots(200_000.0, 2.5 * k), k * slots, 1e-6));
        assert!(approx(dna::value_of_reduction(slots * k, 160.0), k * value, 1e-6));
        assert!(approx(dna::value_of_reduction(slots, 160.0 * k), k * value, 1e-6));
    }
}

// Property: relative_reduction of identical (nonzero) rates is exactly 0.
#[test]
fn prop_dna_relative_reduction_identical_rates_is_zero() {
    for &r in &[0.5, 5.5, 8.0, 100.0] {
        assert_eq!(dna::relative_reduction(r, r), Some(0.0));
    }
}

// Property: DiGA expected value is linear in the success probability — the midpoint
// probability gives the midpoint EV.
#[test]
fn prop_diga_expected_value_linear_in_probability() {
    let rev = diga::revenue(60_000.0, 0.81, 380.0);
    let lo = diga::expected_value(0.0, rev, 2_000_000.0);
    let hi = diga::expected_value(1.0, rev, 2_000_000.0);
    let mid = diga::expected_value(0.5, rev, 2_000_000.0);
    assert!(approx(mid, (lo + hi) / 2.0, 1e-6));
}

// Property: DiGA revenue equals activated prescriptions × price, over a grid.
#[test]
fn prop_diga_revenue_equals_activated_times_price() {
    for &scripts in &[0.0, 1_000.0, 20_000.0] {
        for &rate in &[0.0, 0.5, 0.81, 1.0] {
            for &price in &[0.0, 380.0, 500.0] {
                assert!(approx(
                    diga::revenue(scripts, rate, price),
                    diga::activated_prescriptions(scripts, rate) * price,
                    1e-9
                ));
            }
        }
    }
}

// Property: N ∝ σ²/Δ² — required_sample_size, sample_size_ratio and
// detectable_effect_improvement are mutually consistent over a grid.
#[test]
fn prop_deb_power_relations_consistent() {
    for &var_old in &[1.0, 5.0, 25.0] {
        for &var_new in &[0.5, 1.0, 5.0] {
            let ratio = deb::sample_size_ratio(var_new, var_old).unwrap();
            let n_old = deb::required_sample_size(var_old, 1.0, 16.0).unwrap();
            let n_new = deb::required_sample_size(var_new, 1.0, 16.0).unwrap();
            // Ratio of required Ns at fixed Δ equals the variance ratio.
            assert!(approx(n_new / n_old, ratio, 1e-9));
            // The detectable-effect improvement squared recovers the variance fold.
            let fold = var_old / var_new;
            let improvement = deb::detectable_effect_improvement(fold);
            assert!(approx(improvement * improvement, fold, 1e-9));
        }
    }
}

// Property: required sample size is symmetric in the sign of Δ (Δ enters squared)
// and linear in both variance and k.
#[test]
fn prop_deb_sample_size_sign_symmetric_and_linear() {
    for &effect in &[0.1, 0.5, 1.0, 2.0] {
        let plus = deb::required_sample_size(5.0, effect, 16.0).unwrap();
        let minus = deb::required_sample_size(5.0, -effect, 16.0).unwrap();
        assert!(approx(plus, minus, 1e-9));
        assert!(approx(deb::required_sample_size(10.0, effect, 16.0).unwrap(), 2.0 * plus, 1e-9));
        assert!(approx(deb::required_sample_size(5.0, effect, 32.0).unwrap(), 2.0 * plus, 1e-9));
    }
}

// Property: trial cost saving and sampling density ratio scale linearly.
#[test]
fn prop_deb_trial_saving_and_density_linear() {
    for &k in &[0.0, 0.5, 2.0, 10.0] {
        assert!(approx(deb::trial_cost_saving(200.0 * k, 25_000.0), k * 5_000_000.0, 1e-6));
        assert!(approx(
            deb::sampling_density_ratio(200.0 * k, 4.0).unwrap(),
            k * 50.0,
            1e-9
        ));
    }
}

// Property: cost-of-delay chain is multiplicative — health CoD × λ monetization matches
// building the money CoD directly, over a grid.
#[test]
fn prop_cod_chain_multiplicative_consistency() {
    for &patients in &[0.0, 50.0, 100.0] {
        for &weeks_removed in &[0.0, 5.0, 26.0] {
            let gain = cod::qaly_gain_per_patient(weeks_removed, 0.12);
            let health = cod::cost_of_delay_health(patients, gain);
            // Direct product identity.
            assert!(approx(health, patients * weeks_removed / 52.0 * 0.12, 1e-9));
            // Monetizing with zero operational term is exactly health × λ.
            assert!(approx(cod::cost_of_delay_money(health, 20_000.0, 0.0), health * 20_000.0, 1e-9));
            // Total loss is linear in the delay duration.
            assert!(approx(cod::total_delay_loss(health, 10.0), 10.0 * cod::total_delay_loss(health, 1.0), 1e-9));
        }
    }
}

// Property: value_of_unblocking is additive over downstream releases, and
// bed_days_avoided_per_year is exactly annualize.
#[test]
fn prop_dro_value_additive_and_bed_days_is_annualize() {
    let r1 = DownstreamRelease { blocked_hours_released: 500.0, unit_cost_per_hour: 30.0 };
    let r2 = DownstreamRelease { blocked_hours_released: 595.0, unit_cost_per_hour: 45.0 };
    let split = dro::value_of_unblocking(&[r1], 0.0, 0.0) + dro::value_of_unblocking(&[r2], 1_460.0, 300.0);
    let joint = dro::value_of_unblocking(&[r1, r2], 1_460.0, 300.0);
    assert!(approx(joint, split, 1e-9));
    for &per_day in &[0.0, 1.0, 4.0, 6.0] {
        assert!(approx(
            dro::bed_days_avoided_per_year(per_day, 365.0),
            dro::annualize(per_day, 365.0),
            1e-12
        ));
    }
}

// Property: gating_task_minutes_saved is antisymmetric in its arguments.
#[test]
fn prop_dro_gating_task_antisymmetric() {
    for &(before, after) in &[(90.0, 20.0), (20.0, 90.0), (45.0, 45.0)] {
        assert!(approx(
            dro::gating_task_minutes_saved(before, after),
            -dro::gating_task_minutes_saved(after, before),
            1e-12
        ));
    }
}

// =========================================================================
// 3. CROSS-MODULE CONSISTENCY
// =========================================================================

// Cross-module: cost_utility_analysis::icur agrees exactly with
// dominance_and_efficiency_frontier::icer on the same ΔC/ΔE pairs.
#[test]
fn cross_icur_agrees_with_frontier_icer() {
    let base = Alternative::new("base", 1_000.0, 10.0);
    for &dc in &[-500.0, -1.0, 0.0, 0.5, 80.0, 1e6] {
        for &de in &[-0.5, 0.01, 0.04, 1.0, 250.0] {
            let next = Alternative::new("next", base.cost + dc, base.effect + de);
            let via_icer = dom::icer(&next, &base).unwrap();
            let via_icur = cua::icur(dc, de).unwrap();
            let tol = 1e-9 * via_icur.abs().max(1.0);
            assert!(approx(via_icer, via_icur, tol), "dc={dc} de={de}");
        }
    }
    // Both agree that ΔE = 0 has no ratio.
    let flat = Alternative::new("flat", base.cost + 50.0, base.effect);
    assert_eq!(dom::icer(&flat, &base), None);
    assert_eq!(cua::icur(50.0, 0.0), None);
}

// Cross-module: discounting present_value(fv, r, t) == fv × discount_factor(r, t)
// from all three sibling modules exposing a discount factor, over a grid.
#[test]
fn cross_present_value_matches_all_discount_factors() {
    let fv = 100_000.0;
    for &r in &[0.0, 0.015, disc::NICE_REFERENCE_RATE, 0.10] {
        for t in 0..=20 {
            let t = t as f64;
            let pv = disc::present_value(fv, r, t);
            assert!(approx(pv, fv * cba::discount_factor(r, t), 1e-6));
            assert!(approx(pv, fv * qalym::discount_factor(r, t), 1e-6));
            assert!(approx(pv, fv * adc::discount_factor(r, t), 1e-6));
        }
    }
}

// Cross-module: the three sibling discount_factor implementations agree with each other
// and with delayed_present_value's implied factor.
#[test]
fn cross_discount_factors_agree_and_match_delayed_pv() {
    for &r in &[0.0, 0.035, 0.10] {
        for &t in &[0.0, 0.5, 1.0, 7.0, 30.0] {
            let f = cba::discount_factor(r, t);
            assert!(approx(f, qalym::discount_factor(r, t), 1e-12));
            assert!(approx(f, adc::discount_factor(r, t), 1e-12));
            // delayed_present_value(pv, r, t) divides by (1+r)^t, i.e. multiplies by f.
            assert!(approx(disc::delayed_present_value(1_000.0, r, t), 1_000.0 * f, 1e-9));
        }
    }
}

// Cross-module: dna_rate_percent and change_failure_rate_percent share the x/y×100 formula
// and the same None convention (zero denominator).
#[test]
fn cross_dna_rate_and_cfr_percent_share_formula() {
    for &(num, den) in &[(0.0, 100.0), (8.0, 100.0), (16_000.0, 200_000.0), (40.0, 40.0)] {
        assert_eq!(
            dna::dna_rate_percent(num, den),
            dora::change_failure_rate_percent(num, den)
        );
    }
    assert_eq!(dna::dna_rate_percent(1.0, 0.0), None);
    assert_eq!(dora::change_failure_rate_percent(1.0, 0.0), None);
}

// Cross-module: DORA's value_pulled_forward for one improvement equals cost-of-delay's
// total_delay_loss for the same weeks and £/week.
#[test]
fn cross_value_pulled_forward_matches_total_delay_loss() {
    for &weeks in &[0.0, 1.0, 5.4, 26.0] {
        for &per_week in &[0.0, 2_000.0, 4_000.0] {
            assert!(approx(
                dora::value_pulled_forward(1.0, weeks, per_week),
                cod::total_delay_loss(per_week, weeks),
                1e-9
            ));
        }
    }
}

// Cross-module: a QALY-denominated cost of delay divided into a threshold matches icur —
// pricing ΔC = CoD money and ΔE = CoD health recovers λ exactly.
#[test]
fn cross_cod_monetization_round_trips_through_icur() {
    let gain = cod::qaly_gain_per_patient(5.0, 0.12);
    let health = cod::cost_of_delay_health(100.0, gain); // QALYs/week
    let money = cod::cost_of_delay_money(health, 20_000.0, 0.0); // £/week
    // £ per QALY implied by the pair is the willingness-to-pay threshold.
    assert!(approx(cua::icur(money, health).unwrap(), 20_000.0, 1e-6));
}

// =========================================================================
// 4. DOMAIN SCENARIOS
// =========================================================================

// Scenario: a DiGA launch P&L — provisional-year revenue finances the RCT, expected value
// is positive at an honest success probability, and the 3-year steady-state PV discounts
// exactly as the annuity formula says.
#[test]
fn scenario_diga_launch_profit_and_loss() {
    // Provisional year: 10,000 scripts × 80% activation × €500 = €4,000,000.
    let year_one = diga::revenue(10_000.0, 0.80, 500.0);
    assert!(approx(year_one, 4_000_000.0, 1e-6));
    assert!(approx(diga::activated_prescriptions(10_000.0, 0.80), 8_000.0, 1e-9));

    // The €1.5M pivotal RCT is financed by the provisional year.
    let rct = 1_500_000.0;
    assert!(diga::provisional_year_finances_evidence(year_one, rct));

    // Steady state after conversion: 40,000 scripts × 80% × €400 = €12,800,000/yr.
    let steady = diga::revenue(40_000.0, 0.80, 400.0);
    assert!(approx(steady, 12_800_000.0, 1e-6));

    // Expected value at P = 0.6: 0.6 × 12.8M − 1.5M = €6,180,000.
    let ev = diga::expected_value(0.6, steady, rct);
    assert!(approx(ev, 6_180_000.0, 1e-6));

    // Three years of steady state discounted at 3.5%: hand value €35,860,953.36.
    let pv3 = disc::annuity_present_value(steady, disc::NICE_REFERENCE_RATE, 3.0);
    assert!(approx(pv3, 35_860_953.36, 1.0));
    let year_by_year: f64 = (1..=3)
        .map(|t| disc::present_value(steady, disc::NICE_REFERENCE_RATE, t as f64))
        .sum();
    assert!(approx(pv3, year_by_year, 1e-6));

    // A failed conversion is strictly worse than not running the RCT at all.
    assert!(diga::expected_value(0.0, steady, rct) < 0.0);
}

// Scenario: a DNA-reduction program — four candidate services drawn on the frontier,
// the threshold pick at £160/slot, and the winning option's full return arithmetic.
#[test]
fn scenario_dna_reduction_frontier_and_return() {
    let appointments = 200_000.0;

    // Baseline: 16,000 DNAs of 200,000 booked = 8%.
    assert_eq!(dna::dna_rate_percent(16_000.0, appointments), Some(8.0));

    // Candidate options (cost/yr, slots recovered/yr).
    let sms_cost = dna::service_cost(appointments, 0.125); // £25,000
    assert!(approx(sms_cost, 25_000.0, 1e-9));
    let sms_slots = dna::recovered_slots(appointments, 1.0); // 8% → 7%: 2,000 slots
    assert!(approx(sms_slots, 2_000.0, 1e-9));
    let combo_slots = dna::recovered_slots(appointments, 2.0); // 8% → 6%: 4,000 slots
    assert!(approx(combo_slots, 4_000.0, 1e-9));

    let options = vec![
        Alternative::new("Do nothing", 0.0, 0.0),
        Alternative::new("SMS reminders", sms_cost, sms_slots),
        Alternative::new("Phone bank", 150_000.0, 2_100.0), // strictly dominated by the combo
        Alternative::new("SMS + backfill", 100_000.0, combo_slots),
    ];

    // The phone bank never reaches the committee.
    assert!(dom::strictly_dominates(&options[3], &options[2]));
    let frontier = dom::efficiency_frontier(&options);
    let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, vec!["Do nothing", "SMS reminders", "SMS + backfill"]);

    // ICERs: 25,000/2,000 = £12.50, then 75,000/2,000 = £37.50 per slot — both below
    // the £160 refilled-slot value, so both steps are worth taking.
    let icers: Vec<f64> = dom::frontier_icers(&frontier).into_iter().flatten().collect();
    assert!(approx(icers[0], 12.50, 1e-9));
    assert!(approx(icers[1], 37.50, 1e-9));
    assert!(icers.iter().all(|&i| i < 160.0));

    // Pick "SMS + backfill": £640,000 recovered vs £100,000 cost = 6.4:1.
    let value = dna::value_of_reduction(combo_slots, 160.0);
    assert!(approx(value, 640_000.0, 1e-6));
    assert_eq!(dna::return_ratio(value, 100_000.0), Some(6.4));

    // Effect size sanity: 8% → 6% is a 25% relative reduction, at the bottom of the RCT band.
    let rel = dna::relative_reduction(8.0, 6.0).unwrap();
    assert!(approx(rel, 0.25, 1e-12));

    // A 12-week procurement delay forgoes 12/52 of the £540,000/yr net value = £124,615.38.
    let net_per_week = (value - 100_000.0) / 52.0;
    let delay_loss = cod::total_delay_loss(net_per_week, 12.0);
    assert!(approx(delay_loss, 124_615.384_6, 0.01));
}

// Scenario: monetizing a DORA improvement — lead-time value pulled forward plus incident
// costs avoided, discounted over five years, with a reliability haircut and a slip check.
#[test]
fn scenario_dora_improvement_monetization() {
    // Delivery cadence: monthly → weekly.
    assert_eq!(dora::deployment_frequency(12.0, 1.0), Some(12.0));
    assert_eq!(dora::deployment_frequency(52.0, 1.0), Some(52.0));

    // CFR measured from raw counts: 5/25 = 20% before, 2/40 = 5% after.
    assert_eq!(dora::change_failure_rate_percent(5.0, 25.0), Some(20.0));
    assert_eq!(dora::change_failure_rate_percent(2.0, 40.0), Some(5.0));

    // Lead time: 8 weeks → 14 days = 2 weeks, a 6-week reduction.
    let reduction = dora::lead_time_reduction_weeks(8.0, dora::days_to_weeks(14.0));
    assert!(approx(reduction, 6.0, 1e-12));

    // Each improvement's CoD: £50 saved/patient × 40 patients/week = £2,000/week.
    let cod_per_week = cod::operational_cost_of_delay(50.0, 40.0);
    assert!(approx(cod_per_week, 2_000.0, 1e-9));

    // 20 improvements × 6 weeks × £2,000/week = £240,000/yr pulled forward.
    let pulled = dora::value_pulled_forward(20.0, reduction, cod_per_week);
    assert!(approx(pulled, 240_000.0, 1e-6));

    // CFR 20% → 5% on 20 changes/yr avoids 3 failures × £10,000 = £30,000/yr.
    let avoided = dora::failed_changes_avoided(20.0, 0.20, 0.05);
    assert!(approx(avoided, 3.0, 1e-9));
    let incident_savings = dora::failure_cost_avoided(20.0, 0.20, 0.05, 10_000.0);
    assert!(approx(incident_savings, 30_000.0, 1e-6));

    // MTTR 24h → 1h at £2,000/hr harm: per-outage harm falls £48,000 → £2,000.
    assert!(approx(dora::downtime_harm(24.0, 2_000.0), 48_000.0, 1e-9));
    assert!(approx(dora::downtime_harm(1.0, 2_000.0), 2_000.0, 1e-9));

    // Annual benefit £270,000, discounted for the 99% SLO actually attained: £267,300.
    let annual = pulled + incident_savings;
    let realistic = dora::reliability_adjusted_benefit(annual, 0.99);
    assert!(approx(realistic, 267_300.0, 1e-6));

    // Five-year PV at 3.5%: 267,300 × 4.5150524 ≈ £1,206,873.5.
    let pv = disc::annuity_present_value(realistic, disc::NICE_REFERENCE_RATE, 5.0);
    assert!(approx(pv, 1_206_873.5, 5.0));

    // A one-year program slip discounts the whole stream once more — strictly worse.
    let slipped = disc::delayed_present_value(pv, disc::NICE_REFERENCE_RATE, 1.0);
    assert!(approx(slipped, pv / 1.035, 1e-6));
    assert!(slipped < pv);
}

// Scenario: a global-health screening platform — DALY burden averted, cost per DALY,
// WHO-CHOICE verdict, and the downstream ward unblocking the platform also delivers.
#[test]
fn scenario_global_health_platform_dalys_and_downstream() {
    // Burden averted: 8 deaths × 25 years + 150 person-years at weight 0.3 = 245 DALYs/yr.
    let yll = daly::years_of_life_lost(8.0, 25.0);
    let yld = daly::years_lived_with_disability(150.0, 0.3);
    assert!(approx(yll, 200.0, 1e-9));
    assert!(approx(yld, 45.0, 1e-9));
    let averted = daly::dalys(yll, yld);
    assert!(approx(averted, 245.0, 1e-9));

    // $490,000/yr running cost → $2,000 per DALY averted.
    let cpd = daly::cost_per_daly_averted(490_000.0, averted).unwrap();
    assert!(approx(cpd, 2_000.0, 1e-9));

    // At $3,000 GDP per capita that is under 1× GDP: highly cost-effective;
    // the same platform in a $1,000-GDP country lands in the 1–3× band.
    assert_eq!(daly::who_choice_band(cpd, 3_000.0), WhoChoiceBand::HighlyCostEffective);
    assert_eq!(daly::who_choice_band(cpd, 1_000.0), WhoChoiceBand::CostEffective);

    // The platform also unblocks the ward's gating sign-off: 60 → 15 minutes,
    // 2 recovered discharges/day (730 bed days/yr) and 2 staff-hrs/day released (730 hrs/yr).
    assert!(approx(dro::gating_task_minutes_saved(60.0, 15.0), 45.0, 1e-12));
    let bed_days = dro::bed_days_avoided_per_year(2.0, 365.0);
    assert!(approx(bed_days, 730.0, 1e-9));
    let released = dro::annualize(2.0, 365.0);
    let releases = [DownstreamRelease { blocked_hours_released: released, unit_cost_per_hour: 25.0 }];
    // 730 hrs × $25 + 730 bed days × $200 = $18,250 + $146,000 = $164,250.
    let unblock_value = dro::value_of_unblocking(&releases, bed_days, 200.0);
    assert!(approx(unblock_value, 164_250.0, 1e-6));
}

// Scenario: a digital-endpoint trial redesign — dense passive sampling shrinks the trial,
// the saving dwarfs validation, and the CUA of the resulting therapy is dominant.
#[test]
fn scenario_digital_endpoint_trial_and_cua() {
    // 365 passive vs 4 clinic measurements per patient-year.
    let density = deb::sampling_density_ratio(365.0, 4.0).unwrap();
    assert!(approx(density, 91.25, 1e-9));

    // Variance falls 4×: detectable effect improves 2×, N falls to 1/4.
    assert!(approx(deb::detectable_effect_improvement(4.0), 2.0, 1e-12));
    assert_eq!(deb::sample_size_ratio(1.0, 4.0), Some(0.25));
    let n_old = deb::required_sample_size(4.0, 0.5, 16.0).unwrap(); // 16 × 4 / 0.25 = 256
    let n_new = deb::required_sample_size(1.0, 0.5, 16.0).unwrap(); // 64
    assert!(approx(n_old, 256.0, 1e-9));
    assert!(approx(n_new, 64.0, 1e-9));

    // Cutting 192 patients at £25,000 saves £4.8M — well above a £1.5M validation spend.
    let saving = deb::trial_cost_saving(n_old - n_new, 25_000.0);
    assert!(approx(saving, 4_800_000.0, 1e-6));
    assert!(saving > 1_500_000.0);

    // The therapy the trial supports: £300 app, 30% displacement of £2,000 care,
    // 1 year at utility 0.75 vs 0.70 → ΔC = −£300, ΔE = +0.05: dominant.
    let delta_cost = 300.0 - cua::displaced_care_saving(0.30, 2_000.0);
    assert!(approx(delta_cost, -300.0, 1e-9));
    let new = [HealthState { duration_years: 1.0, utility: 0.75 }];
    let old = [HealthState { duration_years: 1.0, utility: 0.70 }];
    let de = cua::delta_qalys(&new, &old);
    assert!(approx(de, 0.05, 1e-12));
    assert!(cua::is_dominant(delta_cost, de));
    // And the (dominant-quadrant, negative) ratio is −£6,000/QALY — reported only with care.
    assert!(approx(cua::icur(delta_cost, de).unwrap(), -6_000.0, 1e-6));
}
