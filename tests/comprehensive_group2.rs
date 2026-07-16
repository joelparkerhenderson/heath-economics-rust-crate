//! Comprehensive integration tests, group 2.
//!
//! Covers: benefits_realization, budget_impact_analysis, build_vs_buy,
//! cash_releasing_vs_non_cash_releasing, clinical_ai_evaluation,
//! cloud_unit_economics, cost_benefit_analysis, cost_consequence_analysis,
//! cost_effectiveness_analysis, cost_minimization_analysis — plus
//! cross-module consistency with incremental_cost_effectiveness_ratio and
//! discounting_and_time_preference.
//!
//! Sections:
//!   1. EDGE CASES
//!   2. PROPERTIES / INVARIANTS (grid loops, deterministic)
//!   3. CROSS-MODULE CONSISTENCY
//!   4. DOMAIN SCENARIOS (end-to-end, hand-computed)

use health_economics::benefits_realization::{
    Benefit, BenefitClass, optimism_adjusted_forecast, optimism_error, realization_rate,
};
use health_economics::budget_impact_analysis::{
    PatientGroup, budget_impact, net_cost_per_patient, scenario_cost,
};
use health_economics::build_vs_buy::{
    cost_of_delay, effective_cost, risk_adjusted_build_cost, risk_adjusted_time_to_value,
    total_cost_of_ownership,
};
use health_economics::cash_releasing_vs_non_cash_releasing::{
    SavingCategory, TimeAllocation, annual_hours_saved, cash_releasing_saving, category_value,
    non_cash_releasing_value,
};
use health_economics::clinical_ai_evaluation::{
    auroc, cost_per_true_case, cost_per_true_case_from_counts, npv_from_counts, npv_from_rates,
    number_needed_to_screen, positive_rate, ppv_from_counts, ppv_from_rates, sensitivity,
    specificity,
};
use health_economics::cloud_unit_economics::{
    CloudSpend, unit_cost, unit_cost_change, unit_cost_ratio,
};
use health_economics::cost_benefit_analysis::{
    GREEN_BOOK_DISCOUNT_RATE, annuity_factor, benefit_cost_ratio, discount_factor,
    net_present_value, optimism_bias_benefit_haircut, optimism_bias_cost_uplift, present_value,
};
use health_economics::cost_consequence_analysis::{
    ConsequenceRow, CostConsequenceTable, cost_per_unit_gained, value_of_avoided_events,
};
use health_economics::cost_effectiveness_analysis::{
    InterventionOption, average_cost_effectiveness_ratio, icer, incremental_icers,
};
use health_economics::cost_minimization_analysis::{
    CostLines, Selection, cost_minimization, cost_saving, outcomes_equivalent,
};
// Cross-module partners (section 3).
use health_economics::discounting_and_time_preference as dtp;
use health_economics::incremental_cost_effectiveness_ratio as icer_mod;

/// Absolute-tolerance float comparison used throughout.
fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

// =========================================================================
// 1. EDGE CASES
// =========================================================================

// Locks down: realization_rate (fn and method) and optimism_error return None
// exactly when forecast == 0, and nowhere else nearby.
#[test]
fn edge_benefits_realization_zero_forecast_is_none() {
    assert!(realization_rate(5.0, 0.0).is_none());
    assert!(realization_rate(0.0, 0.0).is_none());
    assert!(optimism_error(0.0, 5.0).is_none());
    // Zero realized with nonzero forecast is defined (rate 0).
    assert!(close(realization_rate(0.0, 100.0).unwrap(), 0.0, 1e-12));
    assert!(close(optimism_error(100.0, 0.0).unwrap(), 1.0, 1e-12));
    let b = Benefit {
        name: "orphan".to_string(),
        class: BenefitClass::Qualitative,
        forecast: 0.0,
        realized: 7.0,
    };
    assert!(b.realization_rate().is_none());
}

// Locks down: negative and over-delivering benefit lines produce meaningful
// (not clamped) rates, and 1e12 magnitudes stay finite.
#[test]
fn edge_benefits_realization_negatives_and_extremes() {
    // Benefit went backwards vs baseline: negative rate, error above 1.
    assert!(close(realization_rate(-50.0, 100.0).unwrap(), -0.5, 1e-12));
    assert!(close(optimism_error(100.0, -50.0).unwrap(), 1.5, 1e-12));
    // Error of 0 leaves the forecast alone; error of 1 zeroes it.
    assert!(close(optimism_adjusted_forecast(123.0, 0.0), 123.0, 1e-12));
    assert!(close(optimism_adjusted_forecast(123.0, 1.0), 0.0, 1e-12));
    // Extreme magnitude stays finite.
    let r = realization_rate(1e12, 2e12).unwrap();
    assert!(r.is_finite() && close(r, 0.5, 1e-12));
    assert!(optimism_adjusted_forecast(1e12, 0.3).is_finite());
}

// Locks down: empty PatientGroup slice costs 0 and a null budget impact is 0.
#[test]
fn edge_budget_impact_empty_and_zero() {
    assert!(close(scenario_cost(&[]), 0.0, 1e-12));
    assert!(close(budget_impact(0.0, 0.0), 0.0, 1e-12));
    // Zero-uptake and zero-population groups contribute nothing.
    let dead = PatientGroup { eligible_population: 0.0, uptake: 0.5, net_cost_per_patient: 99.0 };
    let idle = PatientGroup { eligible_population: 500.0, uptake: 0.0, net_cost_per_patient: 99.0 };
    assert!(close(dead.cost(), 0.0, 1e-12));
    assert!(close(scenario_cost(&[dead, idle]), 0.0, 1e-12));
}

// Locks down: uptake boundaries 0.0/1.0, cost-saving (negative) net cost, and
// extreme magnitudes without NaN/inf.
#[test]
fn edge_budget_impact_boundaries_and_extremes() {
    let full = PatientGroup { eligible_population: 1_000.0, uptake: 1.0, net_cost_per_patient: 180.0 };
    assert!(close(full.cost(), 180_000.0, 1e-9));
    // Displacement exceeding price makes the net cost negative (a saving).
    let net = net_cost_per_patient(100.0, 250.0, 0.0);
    assert!(close(net, -150.0, 1e-12));
    let saver = PatientGroup { eligible_population: 1_000.0, uptake: 1.0, net_cost_per_patient: net };
    assert!(budget_impact(scenario_cost(&[saver]), 0.0) < 0.0);
    // Extremes stay finite.
    let big = PatientGroup { eligible_population: 1e12, uptake: 1.0, net_cost_per_patient: 1e12 };
    assert!(big.cost().is_finite());
    assert!(net_cost_per_patient(1e12, 1e12, 1e12).is_finite());
}

// Locks down: build-vs-buy identities at zero (no delay, no running cost,
// factor 1.0 / uplift 0.0 are identity adjustments).
#[test]
fn edge_build_vs_buy_zeros_and_identities() {
    assert!(close(total_cost_of_ownership(0.0, 0.0, 0.0), 0.0, 1e-12));
    assert!(close(total_cost_of_ownership(500.0, 100.0, 0.0), 500.0, 1e-12));
    assert!(close(cost_of_delay(25_000.0, 0.0), 0.0, 1e-12));
    assert!(close(risk_adjusted_build_cost(600_000.0, 1.0), 600_000.0, 1e-9));
    assert!(close(risk_adjusted_time_to_value(12.0, 0.0), 12.0, 1e-12));
    assert!(close(effective_cost(0.0, 0.0), 0.0, 1e-12));
}

// Locks down: negative months (option arrives EARLIER) yields a negative
// delay cost, and 1e12 magnitudes stay finite.
#[test]
fn edge_build_vs_buy_negative_delay_and_extremes() {
    assert!(close(cost_of_delay(10_000.0, -2.0), -20_000.0, 1e-9));
    assert!(total_cost_of_ownership(1e12, 1e12, 100.0).is_finite());
    assert!(risk_adjusted_build_cost(1e12, 1.4).is_finite());
    assert!(effective_cost(1e12, 1e12).is_finite());
    assert!(risk_adjusted_time_to_value(1e12, 0.6).is_finite());
}

// Locks down: cash-releasing edge behavior — empty splits are worth 0, a
// NoBenefit slice is worth exactly 0 despite its rate, budget growth gives a
// negative "saving".
#[test]
fn edge_cash_releasing_zero_empty_and_negative() {
    assert!(close(category_value(11_500.0, &[], SavingCategory::CashReleasing), 0.0, 1e-12));
    let slack = TimeAllocation { category: SavingCategory::NoBenefit, fraction: 1.0, hourly_rate: 1e6 };
    assert!(close(slack.value(10_000.0), 0.0, 1e-12));
    assert!(close(slack.hours(10_000.0), 10_000.0, 1e-9)); // hours still counted
    // A budget line that grew is a negative saving, not clamped to zero.
    assert!(close(cash_releasing_saving(100_000.0, 130_000.0), -30_000.0, 1e-9));
    // Zero staff or zero weeks saves zero hours.
    assert!(close(annual_hours_saved(0.0, 0.5, 5.0, 46.0), 0.0, 1e-12));
    assert!(close(annual_hours_saved(100.0, 0.5, 5.0, 0.0), 0.0, 1e-12));
    assert!(close(non_cash_releasing_value(0.0, 25.0), 0.0, 1e-12));
}

// Locks down: fraction boundaries 0.0/1.0 in TimeAllocation and extreme
// magnitudes staying finite.
#[test]
fn edge_cash_releasing_fraction_boundaries_and_extremes() {
    let none = TimeAllocation { category: SavingCategory::CashReleasing, fraction: 0.0, hourly_rate: 35.0 };
    let all = TimeAllocation { category: SavingCategory::CashReleasing, fraction: 1.0, hourly_rate: 35.0 };
    assert!(close(none.value(11_500.0), 0.0, 1e-12));
    assert!(close(all.value(11_500.0), 11_500.0 * 35.0, 1e-6));
    assert!(non_cash_releasing_value(1e12, 1e6).is_finite());
    assert!(annual_hours_saved(1e6, 1e3, 1e2, 1e1).is_finite());
}

// Locks down: every count-based diagnostic ratio is None exactly when its
// denominator (the pair of counts) is zero.
#[test]
fn edge_clinical_ai_count_ratios_none_conditions() {
    assert!(sensitivity(0.0, 0.0).is_none());
    assert!(specificity(0.0, 0.0).is_none());
    assert!(ppv_from_counts(0.0, 0.0).is_none());
    assert!(npv_from_counts(0.0, 0.0).is_none());
    assert!(cost_per_true_case_from_counts(1_000.0, 0.0).is_none());
    // One-sided zeros are fine and hit the 0.0/1.0 boundaries.
    assert!(close(sensitivity(10.0, 0.0).unwrap(), 1.0, 1e-12));
    assert!(close(sensitivity(0.0, 10.0).unwrap(), 0.0, 1e-12));
    assert!(close(specificity(0.0, 10.0).unwrap(), 0.0, 1e-12));
    assert!(close(ppv_from_counts(10.0, 0.0).unwrap(), 1.0, 1e-12));
    assert!(close(npv_from_counts(0.0, 10.0).unwrap(), 0.0, 1e-12));
}

// Locks down: rate-based diagnostics' exact None conditions — ppv_from_rates
// when the positive rate is 0, npv_from_rates when the negative rate is 0,
// NNS and cost_per_true_case when sens × prev == 0.
#[test]
fn edge_clinical_ai_rate_ratios_none_conditions() {
    // Nothing is ever flagged positive: sens 0 at prev 1, or spec 1 at prev 0.
    assert!(close(positive_rate(0.0, 1.0, 1.0), 0.0, 1e-12));
    assert!(ppv_from_rates(0.0, 1.0, 1.0).is_none());
    assert!(ppv_from_rates(0.9, 1.0, 0.0).is_none());
    // Nothing is ever cleared negative: spec 0 and sens 1.
    assert!(npv_from_rates(1.0, 0.0, 0.5).is_none());
    // No true case can ever be found.
    assert!(number_needed_to_screen(0.0, 0.9).is_none());
    assert!(number_needed_to_screen(0.5, 0.0).is_none());
    assert!(cost_per_true_case(0.9, 0.93, 0.0, 350.0).is_none());
    assert!(cost_per_true_case(0.0, 0.93, 0.01, 350.0).is_none());
}

// Locks down: prevalence boundaries 0 and 1 — positive_rate collapses to
// (1 − spec) and sens respectively; PPV is 1 at prevalence 1; NPV is 0 there.
#[test]
fn edge_clinical_ai_prevalence_boundaries() {
    assert!(close(positive_rate(0.90, 0.93, 0.0), 0.07, 1e-12));
    assert!(close(positive_rate(0.90, 0.93, 1.0), 0.90, 1e-12));
    // At prevalence 1 every positive call is right and every negative wrong.
    assert!(close(ppv_from_rates(0.90, 0.93, 1.0).unwrap(), 1.0, 1e-12));
    assert!(close(npv_from_rates(0.90, 0.93, 1.0).unwrap(), 0.0, 1e-12));
    // At prevalence 0 every alert is false.
    assert!(close(ppv_from_rates(0.90, 0.93, 0.0).unwrap(), 0.0, 1e-12));
    assert!(close(npv_from_rates(0.90, 0.93, 0.0).unwrap(), 1.0, 1e-12));
    // A perfect test at 50% prevalence flags exactly half.
    assert!(close(positive_rate(1.0, 1.0, 0.5), 0.5, 1e-12));
}

// Locks down: auroc is None iff either score slice is empty.
#[test]
fn edge_auroc_empty_inputs() {
    assert!(auroc(&[], &[0.1]).is_none());
    assert!(auroc(&[0.9], &[]).is_none());
    assert!(auroc(&[], &[]).is_none());
    // A single pair is enough for a defined value.
    assert!(close(auroc(&[0.9], &[0.1]).unwrap(), 1.0, 1e-12));
}

// Locks down: cloud unit-economics None conditions (zero units, zero
// reference cost, zero previous cost) and zero-spend total.
#[test]
fn edge_cloud_zero_denominators() {
    assert!(unit_cost(62_000.0, 0.0).is_none());
    assert!(unit_cost_ratio(1.0, 0.0).is_none());
    assert!(unit_cost_change(0.0, 1.0).is_none());
    let nil = CloudSpend { compute: 0.0, data: 0.0, shared_platform: 0.0 };
    assert!(close(nil.total(), 0.0, 1e-12));
    // Zero cost over positive units is a defined £0 unit cost.
    assert!(close(unit_cost(0.0, 1_000.0).unwrap(), 0.0, 1e-12));
}

// Locks down: cloud extreme magnitudes stay finite and sane.
#[test]
fn edge_cloud_extremes() {
    let big = CloudSpend { compute: 1e12, data: 1e12, shared_platform: 1e12 };
    assert!(close(big.total(), 3e12, 1e-3));
    let uc = unit_cost(big.total(), 1e12).unwrap();
    assert!(uc.is_finite() && close(uc, 3.0, 1e-12));
    assert!(unit_cost_ratio(1e12, 1e-12).unwrap().is_finite());
    assert!(unit_cost_change(1e-12, 1e12).unwrap().is_finite());
}

// Locks down: CBA edge identities — discount_factor(r, 0) == 1, rate-0
// factor is 1 forever, annuity_factor(r, 0) == 0, empty stream PV is 0,
// BCR undefined at zero PV costs.
#[test]
fn edge_cba_zero_identities() {
    assert!(close(discount_factor(GREEN_BOOK_DISCOUNT_RATE, 0.0), 1.0, 1e-12));
    assert!(close(discount_factor(0.0, 50.0), 1.0, 1e-12));
    assert!(close(annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 0), 0.0, 1e-12));
    assert!(close(present_value(&[], 0.035), 0.0, 1e-12));
    assert!(benefit_cost_ratio(1.0, 0.0).is_none());
    assert!(close(net_present_value(0.0, 0.0), 0.0, 1e-12));
    // Zero-percent bias adjustments are identities; a 100% haircut zeroes.
    assert!(close(optimism_bias_cost_uplift(1_200_000.0, 0.0), 1_200_000.0, 1e-9));
    assert!(close(optimism_bias_benefit_haircut(5_102_000.0, 0.0), 5_102_000.0, 1e-9));
    assert!(close(optimism_bias_benefit_haircut(5_102_000.0, 1.0), 0.0, 1e-9));
}

// Locks down: CBA with negative flows (costs as negative benefits) and
// extreme magnitudes staying finite.
#[test]
fn edge_cba_negatives_and_extremes() {
    // A pure cost stream has a negative PV.
    let pv = present_value(&[-100.0, -100.0], 0.035);
    assert!(pv < 0.0 && close(pv, -100.0 - 100.0 / 1.035, 1e-9));
    assert!(net_present_value(0.0, 1e12).is_finite());
    assert!(present_value(&[1e12, 1e12, 1e12], 0.035).is_finite());
    assert!(optimism_bias_cost_uplift(1e12, 0.4).is_finite());
    let df = discount_factor(0.035, 1_000.0);
    assert!(df.is_finite() && df > 0.0); // deep-future factor underflows toward 0, never NaN
}

// Locks down: CCA edge behavior — zero-gain rows are None, identical
// intervention/comparator rows difference to 0, empty consequence lists are
// legal.
#[test]
fn edge_cca_zero_and_empty() {
    assert!(cost_per_unit_gained(85_000.0, 0.0).is_none());
    let flat = ConsequenceRow { name: "No change".to_string(), intervention: 42.0, comparator: 42.0 };
    assert!(close(flat.difference(), 0.0, 1e-12));
    let table = CostConsequenceTable {
        cost: ConsequenceRow { name: "Cost".to_string(), intervention: 0.0, comparator: 0.0 },
        consequences: vec![],
    };
    assert!(close(table.incremental_cost(), 0.0, 1e-12));
    assert!(table.consequences.is_empty());
    assert!(close(value_of_avoided_events(0.0, 1_200.0), 0.0, 1e-12));
    assert!(close(value_of_avoided_events(82.0, 0.0), 0.0, 1e-12));
}

// Locks down: CCA cost-saving interventions give negative incremental cost
// and negative cost-per-unit (money saved per unit gained); extremes finite.
#[test]
fn edge_cca_negative_and_extremes() {
    let cheaper = CostConsequenceTable {
        cost: ConsequenceRow { name: "Cost".to_string(), intervention: 50_000.0, comparator: 95_000.0 },
        consequences: vec![ConsequenceRow {
            name: "Hours".to_string(),
            intervention: 6_200.0,
            comparator: 11_800.0,
        }],
    };
    assert!(close(cheaper.incremental_cost(), -45_000.0, 1e-9));
    let per_unit = cost_per_unit_gained(cheaper.incremental_cost(), 5_600.0).unwrap();
    assert!(per_unit < 0.0);
    assert!(value_of_avoided_events(1e12, 1e6).is_finite());
    assert!(cost_per_unit_gained(1e12, 1e-6).unwrap().is_finite());
}

// Locks down: CEA None conditions — icer undefined at equal effects, average
// ratio undefined at zero effect, incremental_icers empty below two options.
#[test]
fn edge_cea_none_conditions() {
    assert!(icer(400_000.0, 300.0, 150_000.0, 300.0).is_none());
    assert!(average_cost_effectiveness_ratio(100.0, 0.0).is_none());
    assert!(incremental_icers(&[]).is_empty());
    let solo = vec![InterventionOption { name: "Only".into(), cost: 1.0, effect: 1.0 }];
    assert!(incremental_icers(&solo).is_empty());
    // An equal-effect adjacent pair produces a None slot, not a panic.
    let pair = vec![
        InterventionOption { name: "A".into(), cost: 100.0, effect: 10.0 },
        InterventionOption { name: "B".into(), cost: 200.0, effect: 10.0 },
    ];
    let slots = incremental_icers(&pair);
    assert_eq!(slots.len(), 1);
    assert!(slots[0].is_none());
}

// Locks down: CEA sign behavior (cheaper-and-better gives a negative ICER —
// ambiguous without a quadrant) and extreme magnitudes.
#[test]
fn edge_cea_signs_and_extremes() {
    // A cheaper, more effective option: negative ICER as coded.
    let dominant = icer(100_000.0, 500.0, 150_000.0, 300.0).unwrap();
    assert!(dominant < 0.0);
    assert!(close(dominant, -50_000.0 / 200.0, 1e-9));
    assert!(icer(1e12, 1e12, 0.0, 1.0).unwrap().is_finite());
    assert!(average_cost_effectiveness_ratio(1e12, 1e-6).unwrap().is_finite());
}

// Locks down: CMA gate — without evidenced equivalence the analysis returns
// None no matter how lopsided the costs are.
#[test]
fn edge_cma_gate_and_boundaries() {
    assert!(cost_minimization(1e9, 1.0, false).is_none());
    assert!(cost_minimization(1.0, 1e9, false).is_none());
    // Equivalence margin boundary: |diff| == margin counts as equivalent
    // (<= as coded), and a zero margin accepts only exact equality.
    assert!(outcomes_equivalent(94.0, 92.0, 2.0));
    assert!(!outcomes_equivalent(94.0, 91.9, 2.0));
    assert!(outcomes_equivalent(4.4, 4.4, 0.0));
    assert!(!outcomes_equivalent(4.4, 4.5, 0.0));
    // cost_saving is |a − b|, never negative.
    assert!(close(cost_saving(450_000.0, 500_000.0), 50_000.0, 1e-9));
    assert!(close(cost_saving(0.0, 0.0), 0.0, 1e-12));
    let free = CostLines { licences: 0.0, integration: 0.0, training_support: 0.0 };
    assert!(close(free.total(), 0.0, 1e-12));
}

// =========================================================================
// 2. PROPERTIES / INVARIANTS
// =========================================================================

// Locks down: counts built from (sens, spec, prev) reproduce the same
// sens/spec/PPV/NPV as the rate formulas, and positive_rate == (TP + FP)/N.
#[test]
fn prop_diagnostics_counts_agree_with_rates() {
    let n = 1_000_000.0;
    for &sens in &[0.6, 0.75, 0.9, 0.99] {
        for &spec in &[0.7, 0.85, 0.95] {
            for &prev in &[0.01, 0.1, 0.3, 0.5] {
                let tp = n * prev * sens;
                let fnn = n * prev * (1.0 - sens);
                let fp = n * (1.0 - prev) * (1.0 - spec);
                let tn = n * (1.0 - prev) * spec;
                assert!(close(sensitivity(tp, fnn).unwrap(), sens, 1e-12));
                assert!(close(specificity(tn, fp).unwrap(), spec, 1e-12));
                assert!(close(
                    ppv_from_counts(tp, fp).unwrap(),
                    ppv_from_rates(sens, spec, prev).unwrap(),
                    1e-12
                ));
                assert!(close(
                    npv_from_counts(tn, fnn).unwrap(),
                    npv_from_rates(sens, spec, prev).unwrap(),
                    1e-12
                ));
                assert!(close(positive_rate(sens, spec, prev), (tp + fp) / n, 1e-12));
            }
        }
    }
}

// Locks down: PPV is strictly increasing in prevalence for a fixed imperfect
// operating point, and NNS shrinks as prevalence rises.
#[test]
fn prop_ppv_increases_with_prevalence() {
    let prevs = [0.001, 0.01, 0.05, 0.1, 0.2, 0.4, 0.6, 0.8];
    let mut last_ppv = -1.0;
    let mut last_nns = f64::INFINITY;
    for &prev in &prevs {
        let ppv = ppv_from_rates(0.90, 0.93, prev).unwrap();
        assert!(ppv > last_ppv, "PPV must rise with prevalence: {ppv} at {prev}");
        last_ppv = ppv;
        let nns = number_needed_to_screen(prev, 0.90).unwrap();
        assert!(nns < last_nns, "NNS must fall with prevalence: {nns} at {prev}");
        last_nns = nns;
    }
}

// Locks down: auroc == 1.0 for perfectly separated scores, 0.5 for identical
// distributions, ties scoring exactly 1/2 per pair.
#[test]
fn prop_auroc_separation_identity_and_ties() {
    // Perfect separation across grid sizes.
    for k in 1..=4usize {
        let pos: Vec<f64> = (0..k).map(|i| 0.6 + 0.1 * i as f64).collect();
        let neg: Vec<f64> = (0..k).map(|i| 0.1 + 0.1 * i as f64).collect();
        assert!(close(auroc(&pos, &neg).unwrap(), 1.0, 1e-12));
        // Identical distributions are chance.
        assert!(close(auroc(&pos, &pos).unwrap(), 0.5, 1e-12));
    }
    // One tied pair scores exactly 0.5.
    assert!(close(auroc(&[0.5], &[0.5]).unwrap(), 0.5, 1e-12));
    // Hand-counted mixed case: pairs (1.0,0.5)>, (1.0,0.0)>, (0.5,0.5)=,
    // (0.5,0.0)> → (1 + 1 + 0.5 + 1)/4 = 0.875.
    assert!(close(auroc(&[1.0, 0.5], &[0.5, 0.0]).unwrap(), 0.875, 1e-12));
    // Fully reversed ranking is 0.0.
    assert!(close(auroc(&[0.1, 0.2], &[0.8, 0.9]).unwrap(), 0.0, 1e-12));
}

// Locks down: auroc(pos, neg) + auroc(neg, pos) == 1 exactly, ties included.
#[test]
fn prop_auroc_complement_identity() {
    let score_sets: [(&[f64], &[f64]); 4] = [
        (&[0.9, 0.8, 0.7], &[0.2, 0.1]),
        (&[0.5, 0.5], &[0.5, 0.4]),
        (&[0.3], &[0.6, 0.3, 0.1]),
        (&[0.42, 0.42, 0.9], &[0.42, 0.5]),
    ];
    for (pos, neg) in score_sets {
        let fwd = auroc(pos, neg).unwrap();
        let rev = auroc(neg, pos).unwrap();
        assert!(close(fwd + rev, 1.0, 1e-12), "complement failed: {fwd} + {rev}");
    }
}

// Locks down: present_value at rate 0 equals the plain sum of flows.
#[test]
fn prop_pv_rate_zero_is_plain_sum() {
    let streams: [&[f64]; 4] = [
        &[100.0],
        &[1_200_000.0, 300_000.0, 300_000.0],
        &[-50.0, 25.0, 25.0, 25.0],
        &[0.0, 0.0, 1e9],
    ];
    for flows in streams {
        let total: f64 = flows.iter().sum();
        assert!(close(present_value(flows, 0.0), total, 1e-6));
    }
}

// Locks down: annuity_factor(r, n) == Σ discount_factor(r, t) for t in 1..=n,
// and discount_factor(r, 0) == 1, over a rate × horizon grid.
#[test]
fn prop_annuity_factor_is_sum_of_discount_factors() {
    for &rate in &[0.0, 0.01, GREEN_BOOK_DISCOUNT_RATE, 0.10, 0.25] {
        assert!(close(discount_factor(rate, 0.0), 1.0, 1e-12));
        for &years in &[0u32, 1, 3, 5, 10, 30] {
            let summed: f64 = (1..=years).map(|t| discount_factor(rate, t as f64)).sum();
            assert!(
                close(annuity_factor(rate, years), summed, 1e-9),
                "annuity mismatch at r={rate}, n={years}"
            );
        }
    }
}

// Locks down: NPV antisymmetry — net_present_value(a, b) == −net_present_value(b, a).
#[test]
fn prop_npv_antisymmetric() {
    for &a in &[0.0, 100.0, 5_102_000.0, 1e12] {
        for &b in &[0.0, 2_555_000.0, 750.0] {
            assert!(close(net_present_value(a, b), -net_present_value(b, a), 1e-3));
        }
    }
}

// Locks down: optimism-bias monotonicity — uplifting costs (haircutting
// benefits) can never raise the NPV; zero adjustments are exact identities.
#[test]
fn prop_optimism_bias_never_flatters() {
    let benefits = 5_102_000.0;
    let costs = 2_555_000.0;
    let raw = net_present_value(benefits, costs);
    for &adj in &[0.0, 0.1, 0.2, 0.4, 1.0] {
        let uplifted = net_present_value(benefits, optimism_bias_cost_uplift(costs, adj));
        let haircut = net_present_value(optimism_bias_benefit_haircut(benefits, adj), costs);
        assert!(uplifted <= raw + 1e-9, "cost uplift {adj} raised NPV");
        assert!(haircut <= raw + 1e-9, "benefit haircut {adj} raised NPV");
    }
    for &x in &[0.0, 42.0, 1_200_000.0, 1e12] {
        assert!(close(optimism_bias_cost_uplift(x, 0.0), x, 1e-3));
        assert!(close(optimism_bias_benefit_haircut(x, 0.0), x, 1e-3));
    }
}

// Locks down: scenario_cost is additive over groups and budget_impact of a
// scenario against itself is exactly zero.
#[test]
fn prop_budget_impact_additive_and_null() {
    let g1 = PatientGroup { eligible_population: 30_000.0, uptake: 0.2, net_cost_per_patient: 180.0 };
    let g2 = PatientGroup { eligible_population: 5_000.0, uptake: 0.9, net_cost_per_patient: -40.0 };
    let g3 = PatientGroup { eligible_population: 750.0, uptake: 1.0, net_cost_per_patient: 300.0 };
    let all = [g1, g2, g3];
    let summed = g1.cost() + g2.cost() + g3.cost();
    assert!(close(scenario_cost(&all), summed, 1e-6));
    // Splitting the slice preserves the total.
    assert!(close(scenario_cost(&[g1]) + scenario_cost(&[g2, g3]), scenario_cost(&all), 1e-6));
    for &c in &[0.0, 1_080_000.0, -55_000.0, 1e12] {
        assert!(close(budget_impact(c, c), 0.0, 1e-3));
    }
}

// Locks down: PatientGroup::cost is linear in each of its three fields.
#[test]
fn prop_patient_group_cost_linear_in_each_field() {
    let base = PatientGroup { eligible_population: 10_000.0, uptake: 0.35, net_cost_per_patient: 220.0 };
    for &k in &[0.0, 0.5, 2.0, 10.0] {
        let pop = PatientGroup { eligible_population: base.eligible_population * k, ..base };
        let upt = PatientGroup { uptake: base.uptake * k, ..base };
        let net = PatientGroup { net_cost_per_patient: base.net_cost_per_patient * k, ..base };
        assert!(close(pop.cost(), base.cost() * k, 1e-6));
        assert!(close(upt.cost(), base.cost() * k, 1e-6));
        assert!(close(net.cost(), base.cost() * k, 1e-6));
    }
}

// Locks down: incremental_icers on an effect-sorted frontier equals the
// hand-built adjacent-pair icer calls, including the None slot at equal effects.
#[test]
fn prop_incremental_icers_match_adjacent_pairs() {
    let frontiers: Vec<Vec<(f64, f64)>> = vec![
        vec![(150_000.0, 300.0), (400_000.0, 520.0), (900_000.0, 610.0)],
        vec![(10.0, 1.0), (20.0, 2.0), (40.0, 4.0), (100.0, 5.0)],
        vec![(5.0, 3.0), (9.0, 3.0), (12.0, 8.0)], // equal-effect pair inside
    ];
    for pts in frontiers {
        let options: Vec<InterventionOption> = pts
            .iter()
            .enumerate()
            .map(|(i, &(cost, effect))| InterventionOption { name: format!("opt{i}"), cost, effect })
            .collect();
        let got = incremental_icers(&options);
        assert_eq!(got.len(), options.len() - 1);
        for (i, w) in pts.windows(2).enumerate() {
            let expected = icer(w[1].0, w[1].1, w[0].0, w[0].1);
            match (got[i], expected) {
                (Some(g), Some(e)) => assert!(close(g, e, 1e-9)),
                (None, None) => {}
                other => panic!("slot {i} mismatch: {other:?}"),
            }
        }
    }
}

// Locks down: the average ratio equals cost/effect and the ICER of an option
// against a zero-cost, zero-effect comparator reduces to the average ratio.
#[test]
fn prop_average_ratio_is_icer_against_nothing() {
    for &(cost, effect) in &[(900_000.0, 610.0), (150_000.0, 300.0), (42.0, 7.0)] {
        let avg = average_cost_effectiveness_ratio(cost, effect).unwrap();
        assert!(close(avg, cost / effect, 1e-12));
        assert!(close(icer(cost, effect, 0.0, 0.0).unwrap(), avg, 1e-12));
    }
}

// Locks down: CMA returns None whenever equivalence is not evidenced (any
// costs), picks the strictly cheaper option, and ties go to OptionA as coded.
#[test]
fn prop_cma_gate_cheaper_and_tie_behavior() {
    let costs = [0.0, 1.0, 450_000.0, 500_000.0, 1e9];
    for &a in &costs {
        for &b in &costs {
            // Gate: no evidence, no selection, regardless of costs.
            assert!(cost_minimization(a, b, false).is_none());
            let pick = cost_minimization(a, b, true).unwrap();
            if b < a {
                assert_eq!(pick, Selection::OptionB);
            } else {
                // Ties and a-cheaper both select A (b < a is the only B path).
                assert_eq!(pick, Selection::OptionA);
            }
            // Saving is symmetric.
            assert!(close(cost_saving(a, b), cost_saving(b, a), 1e-6));
        }
    }
}

// Locks down: realization_rate × forecast reconstructs realized, and the
// optimism_error → optimism_adjusted_forecast loop round-trips exactly.
#[test]
fn prop_benefits_realization_roundtrip() {
    let cases = [(450_000.0, 287_000.0), (8_000.0, 5_100.0), (10.0, 12.0), (100.0, -25.0)];
    for &(forecast, realized) in &cases {
        let rate = realization_rate(realized, forecast).unwrap();
        assert!(close(rate * forecast, realized, 1e-6));
        // Applying the observed error to the same raw forecast recovers realized.
        let err = optimism_error(forecast, realized).unwrap();
        assert!(close(optimism_adjusted_forecast(forecast, err), realized, 1e-6));
        // rate + error == 1 by construction.
        assert!(close(rate + err, 1.0, 1e-12));
    }
    // Zero logged error is the identity on any forecast.
    for &f in &[0.0, 100_000.0, 1e12] {
        assert!(close(optimism_adjusted_forecast(f, 0.0), f, 1e-3));
    }
}

// Locks down: Benefit::realization_rate delegates exactly to the free function.
#[test]
fn prop_benefit_method_matches_free_function() {
    let pairs = [(450_000.0, 287_000.0), (8_000.0, 9_600.0)];
    for &(forecast, realized) in &pairs {
        let b = Benefit {
            name: "line".to_string(),
            class: BenefitClass::NonCashReleasing,
            forecast,
            realized,
        };
        assert!(close(
            b.realization_rate().unwrap(),
            realization_rate(realized, forecast).unwrap(),
            1e-12
        ));
    }
}

// Locks down: unit_cost_ratio(a, b) == 1 / unit_cost_ratio(b, a) and
// unit_cost_change(x, x) == 0 across a grid.
#[test]
fn prop_cloud_ratio_inverse_and_null_change() {
    let costs = [0.163, 1.0, 8.0, 42.0, 1e6];
    for &a in &costs {
        for &b in &costs {
            let fwd = unit_cost_ratio(a, b).unwrap();
            let rev = unit_cost_ratio(b, a).unwrap();
            assert!(close(fwd, 1.0 / rev, 1e-9), "ratio inverse failed at {a}/{b}");
        }
        assert!(close(unit_cost_change(a, a).unwrap(), 0.0, 1e-12));
    }
}

// Locks down: unit_cost recomposes — unit_cost × units == total — and the
// CloudSpend total is the plain sum of its three lines.
#[test]
fn prop_cloud_unit_cost_recomposes_total() {
    let spends = [
        CloudSpend { compute: 30_000.0, data: 18_000.0, shared_platform: 14_000.0 },
        CloudSpend { compute: 1.0, data: 2.0, shared_platform: 3.0 },
    ];
    for spend in spends {
        assert!(close(spend.total(), spend.compute + spend.data + spend.shared_platform, 1e-9));
        for &units in &[1.0, 380_000.0, 1e9] {
            let uc = unit_cost(spend.total(), units).unwrap();
            assert!(close(uc * units, spend.total(), 1e-6));
        }
    }
}

// Locks down: effective_cost == TCO + delay cost componentwise, and TCO is
// upfront + annual × years, across a grid.
#[test]
fn prop_build_vs_buy_effective_cost_decomposition() {
    for &upfront in &[0.0, 810_000.0] {
        for &annual in &[0.0, 120_000.0, 150_000.0] {
            for &years in &[1.0, 3.0, 5.0] {
                let tco = total_cost_of_ownership(upfront, annual, years);
                assert!(close(tco, upfront + annual * years, 1e-6));
                for &(v, m) in &[(0.0, 0.0), (25_000.0, 15.0), (10_000.0, 2.5)] {
                    let cod = cost_of_delay(v, m);
                    assert!(close(cod, v * m, 1e-6));
                    assert!(close(effective_cost(tco, cod), tco + cod, 1e-6));
                }
            }
        }
    }
}

// Locks down: risk adjustments are multiplicative as coded — scaling the
// estimate scales the output, and stacked overrun factors compose by product.
#[test]
fn prop_build_vs_buy_risk_adjustments_multiplicative() {
    for &est in &[100.0, 600_000.0] {
        for &f in &[1.0, 1.3, 1.4] {
            // Linear in the estimate.
            assert!(close(
                risk_adjusted_build_cost(2.0 * est, f),
                2.0 * risk_adjusted_build_cost(est, f),
                1e-6
            ));
            // Factors compose multiplicatively.
            assert!(close(
                risk_adjusted_build_cost(risk_adjusted_build_cost(est, f), 1.1),
                risk_adjusted_build_cost(est, f * 1.1),
                1e-6
            ));
        }
        // Time uplift is (1 + u) multiplicative, so u = 0 is identity and
        // doubling the estimate doubles the adjusted time.
        for &u in &[0.0, 0.4, 0.6] {
            assert!(close(risk_adjusted_time_to_value(est, u), est * (1.0 + u), 1e-6));
            assert!(close(
                risk_adjusted_time_to_value(2.0 * est, u),
                2.0 * risk_adjusted_time_to_value(est, u),
                1e-6
            ));
        }
    }
}

// Locks down: an honest cash/capacity/no-benefit split conserves hours, its
// category values match hours × rate by hand, and the NoBenefit rate never
// leaks into value.
#[test]
fn prop_cash_releasing_split_accounting() {
    let total_hours = annual_hours_saved(100.0, 0.5, 5.0, 46.0);
    for &(cash_frac, cap_frac) in &[(0.2, 0.6), (0.0, 1.0), (0.5, 0.0), (0.3, 0.3)] {
        let slack_frac = 1.0 - cash_frac - cap_frac;
        let split = [
            TimeAllocation { category: SavingCategory::CashReleasing, fraction: cash_frac, hourly_rate: 35.0 },
            TimeAllocation { category: SavingCategory::NonCashReleasing, fraction: cap_frac, hourly_rate: 25.0 },
            TimeAllocation { category: SavingCategory::NoBenefit, fraction: slack_frac, hourly_rate: 999.0 },
        ];
        // Hours conserve across the split.
        let hours_sum: f64 = split.iter().map(|s| s.hours(total_hours)).sum();
        assert!(close(hours_sum, total_hours, 1e-6));
        // Category totals match hand math.
        let cash = category_value(total_hours, &split, SavingCategory::CashReleasing);
        let cap = category_value(total_hours, &split, SavingCategory::NonCashReleasing);
        let none = category_value(total_hours, &split, SavingCategory::NoBenefit);
        assert!(close(cash, total_hours * cash_frac * 35.0, 1e-6));
        assert!(close(cap, total_hours * cap_frac * 25.0, 1e-6));
        assert!(close(none, 0.0, 1e-12), "NoBenefit rate must be ignored");
        // The credible split never exceeds the naive headline at the top rate.
        let headline = non_cash_releasing_value(total_hours, 35.0);
        assert!(cash + cap <= headline + 1e-9);
    }
}

// Locks down: cash_releasing_saving is a plain budget delta — antisymmetric
// and zero on an unchanged line.
#[test]
fn prop_cash_releasing_saving_delta_behavior() {
    for &(before, after) in &[(500_000.0, 419_500.0), (100.0, 100.0), (0.0, 250.0)] {
        assert!(close(
            cash_releasing_saving(before, after),
            -cash_releasing_saving(after, before),
            1e-9
        ));
    }
    assert!(close(cash_releasing_saving(77.0, 77.0), 0.0, 1e-12));
}

// =========================================================================
// 3. CROSS-MODULE CONSISTENCY
// =========================================================================

// Locks down: cost_effectiveness_analysis::icer(cA, eA, cB, eB) agrees with
// incremental_cost_effectiveness_ratio::icer(ΔC, ΔE) on the same deltas —
// including agreeing on the None condition at ΔE == 0.
#[test]
fn cross_two_icer_functions_agree() {
    let cases = [
        (400_000.0, 520.0, 150_000.0, 300.0), // pharmacy vs pulse: 250k/220
        (900_000.0, 610.0, 400_000.0, 520.0), // wearable vs pharmacy: 500k/90
        (300_000.0, 25.0, 0.0, 0.0),          // remote monitoring: £12k/QALY
        (100.0, 5.0, 400.0, 2.0),             // cheaper and better: negative
    ];
    for &(ca, ea, cb, eb) in &cases {
        let via_cea = icer(ca, ea, cb, eb).unwrap();
        let via_icer_mod = icer_mod::icer(ca - cb, ea - eb).unwrap();
        assert!(close(via_cea, via_icer_mod, 1e-9), "ICER modules disagree");
    }
    // Both agree the ratio is undefined at zero effect difference.
    assert!(icer(400_000.0, 300.0, 150_000.0, 300.0).is_none());
    assert!(icer_mod::icer(250_000.0, 0.0).is_none());
}

// Locks down: the remote-monitoring chain across modules — net ΔC through
// icer_mod equals the CEA icer on gross totals with the offset folded in.
#[test]
fn cross_net_incremental_cost_feeds_both_icers() {
    let dc = icer_mod::net_incremental_cost(900_000.0, 600_000.0);
    assert!(close(dc, 300_000.0, 1e-9));
    let ratio = icer_mod::icer(dc, 25.0).unwrap();
    // Same figure as CEA icer with the offset expressed in the comparator arm.
    let via_cea = icer(900_000.0, 25.0, 600_000.0, 0.0).unwrap();
    assert!(close(ratio, via_cea, 1e-9));
    assert!(close(ratio, 12_000.0, 1e-9));
    assert!(icer_mod::adopt_at_threshold(ratio, 20_000.0));
    assert!(!icer_mod::adopt_at_threshold(icer(900_000.0, 25.0, 0.0, 0.0).unwrap(), 20_000.0));
}

// Locks down: cost_benefit_analysis::discount_factor and
// discounting_and_time_preference::present_value are the same math —
// fv × factor(r, t) == present_value(fv, r, t) over a grid.
#[test]
fn cross_discount_factor_matches_dtp_present_value() {
    for &rate in &[0.0, 0.015, GREEN_BOOK_DISCOUNT_RATE, dtp::NICE_REFERENCE_RATE, 0.1] {
        for &year in &[0.0, 1.0, 2.5, 5.0, 30.0] {
            for &fv in &[1.0, 100_000.0] {
                assert!(close(
                    fv * discount_factor(rate, year),
                    dtp::present_value(fv, rate, year),
                    1e-6
                ));
            }
        }
    }
    // The two crates' reference rates are the same 3.5% constant.
    assert!(close(GREEN_BOOK_DISCOUNT_RATE, dtp::NICE_REFERENCE_RATE, 1e-15));
}

// Locks down: cba::annuity_factor equals dtp::annuity_present_value of £1,
// and a level cba::present_value stream equals both.
#[test]
fn cross_annuity_representations_agree() {
    for &rate in &[0.0, GREEN_BOOK_DISCOUNT_RATE, 0.07] {
        for &years in &[1u32, 5, 12] {
            let af = annuity_factor(rate, years);
            let apv = dtp::annuity_present_value(1.0, rate, years as f64);
            assert!(close(af, apv, 1e-9), "annuity mismatch at r={rate}, n={years}");
            // Level £300k stream, year 0 empty, years 1..=n.
            let mut flows = vec![0.0];
            flows.extend(std::iter::repeat_n(300_000.0, years as usize));
            assert!(close(present_value(&flows, rate), 300_000.0 * af, 1e-6));
        }
    }
}

// Locks down: a whole-stream delay via dtp::delayed_present_value equals
// re-discounting with cba::discount_factor.
#[test]
fn cross_delay_is_one_more_discount_factor() {
    let pv = dtp::annuity_present_value(100_000.0, dtp::NICE_REFERENCE_RATE, 5.0);
    for &delay in &[0.0, 1.0, 2.0] {
        let slipped = dtp::delayed_present_value(pv, dtp::NICE_REFERENCE_RATE, delay);
        assert!(close(slipped, pv * discount_factor(GREEN_BOOK_DISCOUNT_RATE, delay), 1e-6));
    }
}

// =========================================================================
// 4. DOMAIN SCENARIOS
// =========================================================================

// Locks down: a full 5-year Green Book appraisal — discounted CBA, optimism
// bias survival test, and the post-go-live benefits-realization audit
// feeding the next forecast, with hand-computed anchors throughout.
#[test]
fn scenario_green_book_appraisal_with_benefits_audit() {
    // e-referral system: build £1.2M (year 0), run £300k/yr, benefits £1,130k/yr.
    let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
    assert!(close(af, 4.515, 5e-4)); // hand anchor: 5-yr annuity at 3.5%

    let cost_flows = [1_200_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0];
    let benefit_flows = [0.0, 1_130_000.0, 1_130_000.0, 1_130_000.0, 1_130_000.0, 1_130_000.0];
    let pv_costs = present_value(&cost_flows, GREEN_BOOK_DISCOUNT_RATE);
    let pv_benefits = present_value(&benefit_flows, GREEN_BOOK_DISCOUNT_RATE);
    assert!(close(pv_costs, 1_200_000.0 + 300_000.0 * af, 1e-6));
    assert!(close(pv_benefits, 1_130_000.0 * af, 1e-6));
    assert!(close(pv_costs, 2_555_000.0, 1_000.0)); // hand anchor ≈ £2,555k
    assert!(close(pv_benefits, 5_102_000.0, 1_000.0)); // hand anchor ≈ £5,102k

    let npv = net_present_value(pv_benefits, pv_costs);
    assert!(close(npv, 2_547_000.0, 1_500.0)); // hand anchor ≈ +£2,547k
    let bcr = benefit_cost_ratio(pv_benefits, pv_costs).unwrap();
    assert!(close(bcr, 2.0, 0.01));

    // Green Book stress: +40% build, −20% benefits — case must still clear.
    let adj_costs = optimism_bias_cost_uplift(1_200_000.0, 0.40) + 300_000.0 * af;
    let adj_benefits = optimism_bias_benefit_haircut(pv_benefits, 0.20);
    let adj_npv = net_present_value(adj_benefits, adj_costs);
    assert!(close(adj_npv, 1_047_000.0, 1_500.0)); // hand anchor ≈ +£1,047k
    assert!(adj_npv > 0.0 && adj_npv < npv);
    assert!(benefit_cost_ratio(adj_benefits, adj_costs).unwrap() > 1.0);

    // T+12 months audit: only £850k of the £1,130k annual benefit evidenced.
    let line = Benefit {
        name: "Annual e-referral benefits".to_string(),
        class: BenefitClass::CashReleasing,
        forecast: 1_130_000.0,
        realized: 850_000.0,
    };
    let rate = line.realization_rate().unwrap();
    assert!(close(rate, 850.0 / 1_130.0, 1e-9)); // ≈ 75.2%
    let err = optimism_error(1_130_000.0, 850_000.0).unwrap();
    assert!(close(err, 280.0 / 1_130.0, 1e-9)); // ≈ 24.8% overshoot
    // Next year's raw £1,130k forecast is presented haircut to the evidence.
    let next = optimism_adjusted_forecast(1_130_000.0, err);
    assert!(close(next, 850_000.0, 1e-3));
    // The realized benefit stream would still clear the adjusted cost base:
    // 850k × 4.515 ≈ £3,838k PV > £3,035k adjusted PV costs.
    assert!(net_present_value(850_000.0 * af, adj_costs) > 0.0);
}

// Locks down: a clinical AI triage rollout — operating-point rates to a
// 10,000-person confusion matrix, cost per true case both ways, NNS, a
// payer budget-impact ramp, and the CCA table the committee actually reads.
#[test]
fn scenario_clinical_ai_triage_end_to_end() {
    // Locked operating point: sens 90%, spec 93%; primary care prevalence 1%.
    let (sens, spec, prev) = (0.90, 0.93, 0.01);
    let n = 10_000.0;
    // Hand-built confusion matrix: TP 90, FN 10, FP 693, TN 9,207.
    let (tp, fnn) = (n * prev * sens, n * prev * (1.0 - sens));
    let (fp, tn) = (n * (1.0 - prev) * (1.0 - spec), n * (1.0 - prev) * spec);
    assert!(close(tp, 90.0, 1e-9) && close(fnn, 10.0, 1e-9));
    assert!(close(fp, 693.0, 1e-9) && close(tn, 9_207.0, 1e-9));
    assert!(close(sensitivity(tp, fnn).unwrap(), sens, 1e-12));
    assert!(close(specificity(tn, fp).unwrap(), spec, 1e-12));

    // 783 positives per 10,000 screened; PPV ≈ 11.5%, NPV ≈ 99.9%.
    assert!(close(positive_rate(sens, spec, prev) * n, tp + fp, 1e-9));
    assert!(close(ppv_from_counts(tp, fp).unwrap(), 90.0 / 783.0, 1e-12));
    assert!(npv_from_counts(tn, fnn).unwrap() > 0.998);

    // Economics: £350 workup per positive → £3,045 per true case, both views.
    let per_case_rates = cost_per_true_case(sens, spec, prev, 350.0).unwrap();
    let per_case_counts = cost_per_true_case_from_counts((tp + fp) * 350.0, tp).unwrap();
    assert!(close(per_case_rates, per_case_counts, 1e-6));
    assert!(close(per_case_rates, 3_045.0, 0.5)); // hand anchor: 783×350/90
    assert!(close(number_needed_to_screen(prev, sens).unwrap(), 1.0 / 0.009, 1e-9));

    // Model quality gate: clear separation on the pilot's scored cases.
    let roc = auroc(&[0.91, 0.84, 0.77], &[0.40, 0.31, 0.12]).unwrap();
    assert!(close(roc, 1.0, 1e-12));

    // Payer affordability: 30,000 eligible, net £180/user, uptake 20/40/60%.
    let net = net_cost_per_patient(300.0, 120.0, 0.0);
    assert!(close(net, 180.0, 1e-12));
    let year = |uptake: f64| PatientGroup {
        eligible_population: 30_000.0,
        uptake,
        net_cost_per_patient: net,
    };
    let bi: Vec<f64> = [0.20, 0.40, 0.60]
        .iter()
        .map(|&u| budget_impact(scenario_cost(&[year(u)]), 0.0))
        .collect();
    assert!(close(bi[0], 1_080_000.0, 1e-6)); // hand: 30,000×0.20×180
    assert!(close(bi[1], 2_160_000.0, 1e-6)); // hand: 30,000×0.40×180
    assert!(close(bi[2], 3_240_000.0, 1e-6)); // hand: 30,000×0.60×180

    // The committee's CCA table: cost row plus disaggregated consequences.
    let table = CostConsequenceTable {
        cost: ConsequenceRow {
            name: "Annual programme cost".to_string(),
            intervention: 274_050.0, // 783 workups × £350 per 10,000 screened
            comparator: 200_000.0,
        },
        consequences: vec![
            ConsequenceRow { name: "True cases found".to_string(), intervention: 90.0, comparator: 60.0 },
            ConsequenceRow { name: "False alarms".to_string(), intervention: 693.0, comparator: 250.0 },
        ],
    };
    assert!(close(table.incremental_cost(), 74_050.0, 1e-6));
    let extra_cases = table.consequences[0].difference();
    assert!(close(extra_cases, 30.0, 1e-9));
    // £74,050 buys 30 extra cases ≈ £2,468 per additional case found.
    let per_extra = cost_per_unit_gained(table.incremental_cost(), extra_cases).unwrap();
    assert!(close(per_extra, 74_050.0 / 30.0, 1e-9));
    // The unfavorable row is shown too: +443 false alarms.
    assert!(close(table.consequences[1].difference(), 443.0, 1e-9));
    // Each true case caught early avoids a £2,800 late presentation.
    assert!(close(value_of_avoided_events(30.0, 2_800.0), 84_000.0, 1e-6));
}

// Locks down: a build-vs-buy shootout — risk-adjusted TCO plus cost of
// delay, gated through CMA once equivalence is evidenced, with cloud unit
// costs as the tie-breaking evidence, all hand-computed.
#[test]
fn scenario_build_vs_buy_shootout_with_cma_gate() {
    // Buy: £150k/yr SaaS, live in 3 months. Build: £600k + £120k/yr, est. 12 months.
    let build_cost = risk_adjusted_build_cost(600_000.0, 1.35);
    assert!(close(build_cost, 810_000.0, 1e-6)); // hand: 600k × 1.35
    let build_ttv = risk_adjusted_time_to_value(12.0, 0.5);
    assert!(close(build_ttv, 18.0, 1e-9)); // hand: 12 × 1.5
    let buy_ttv = 3.0; // vendor is live in 3 months; no delay prior applied

    let buy_tco = total_cost_of_ownership(0.0, 150_000.0, 5.0);
    let build_tco = total_cost_of_ownership(build_cost, 120_000.0, 5.0);
    assert!(close(buy_tco, 750_000.0, 1e-6)); // hand: 150k × 5
    assert!(close(build_tco, 1_410_000.0, 1e-6)); // hand: 810k + 600k

    // The slower option pays cost of delay at £25k/month of forgone value.
    let cod = cost_of_delay(25_000.0, build_ttv - buy_ttv);
    assert!(close(cod, 375_000.0, 1e-6)); // hand: 25k × 15
    let buy_eff = effective_cost(buy_tco, 0.0);
    let build_eff = effective_cost(build_tco, cod);
    assert!(close(build_eff, 1_785_000.0, 1e-6)); // hand: 1,410k + 375k

    // Equivalence pilot: both meet the spec inside the pre-agreed 2pp margin.
    let equivalent =
        outcomes_equivalent(94.1, 93.8, 2.0) && outcomes_equivalent(4.4, 4.4, 2.0);
    assert!(equivalent);
    // CMA over effective costs: A = build, B = buy → buy (OptionB) wins.
    let pick = cost_minimization(build_eff, buy_eff, equivalent);
    assert_eq!(pick, Some(Selection::OptionB));
    assert!(close(cost_saving(build_eff, buy_eff), 1_035_000.0, 1e-6)); // hand: 1,785k − 750k
    // Sanity: the same decision would be blocked without the pilot.
    assert!(cost_minimization(build_eff, buy_eff, false).is_none());

    // CostLines view of the buy option over 5 years matches its TCO.
    let buy_lines = CostLines { licences: 750_000.0, integration: 0.0, training_support: 0.0 };
    assert!(close(buy_lines.total(), buy_tco, 1e-6));

    // Unit-economics evidence for the QBR: 20,000 consults/year each.
    let buy_unit = unit_cost(buy_tco / 5.0, 20_000.0).unwrap();
    let build_unit = unit_cost(build_tco / 5.0, 20_000.0).unwrap();
    assert!(close(buy_unit, 7.50, 1e-9)); // hand: 150,000 / 20,000
    assert!(close(build_unit, 14.10, 1e-9)); // hand: 282,000 / 20,000
    let ratio = unit_cost_ratio(buy_unit, build_unit).unwrap();
    assert!(ratio < 1.0 && close(ratio, 7.5 / 14.1, 1e-9));
    // Moving from build-plan to buy is a 46.8% unit-cost reduction.
    let change = unit_cost_change(build_unit, buy_unit).unwrap();
    assert!(close(change, (7.5 - 14.1) / 14.1, 1e-9));
    assert!(change < 0.0);
}
