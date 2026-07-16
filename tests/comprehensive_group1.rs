//! Comprehensive integration tests, group 1.
//!
//! Covers:
//! - activation_and_uptake
//! - adherence_and_persistence
//! - ai_developer_productivity
//! - ai_quality_metrics
//! - ai_regulatory_evaluation
//! - ai_return_on_investment
//! - analysis_perspective
//! - avoidable_outsourcing_costs
//! - avoided_downstream_costs
//! - bed_days_saved
//!
//! Sections: 1. EDGE CASES, 2. PROPERTIES / INVARIANTS,
//! 3. CROSS-MODULE CONSISTENCY, 4. DOMAIN SCENARIOS.
//!
//! Domain-boundary choices are grounded in the topic docs at
//! health-economics-metrics/topics/*.md (kebab-case names matching modules).

use health_economics::activation_and_uptake::{
    activation_rate_percent, dtx_fill_rate_percent, funnel_population_value,
    per_eligible_person_value, population_value, uptake_rate_percent, value_per_completer,
};
use health_economics::adherence_and_persistence::{
    cost_per_qaly, digital_adherence_percent, is_adherent, mpr_percent, payer_spend, pdc_percent,
    percent_persistent, persistence_days, qalys_realized,
};
use health_economics::ai_developer_productivity::{
    acceptance_rate, annual_capacity_value, annual_tool_cost, net_capacity_ratio,
    perception_gap_ratio, retention_rate, speedup, throughput_delta,
};
use health_economics::ai_quality_metrics::{
    errors_reaching_submission, expected_error_cost, expected_harm_cost_per_output, faithfulness,
    hallucination_rate, review_cost, ErrorType,
};
use health_economics::ai_regulatory_evaluation::{
    benefit_months_gained, pccp_lifetime_cost, pccp_saving, traditional_lifetime_cost,
    traditional_update_cost,
};
use health_economics::ai_return_on_investment::{
    ai_roi, attributable_benefit, AiCostStack,
};
use health_economics::analysis_perspective::{
    false_reassurance_harm, net_value_from_perspective, patient_time_value, payer_savings,
    ImpactItem, Perspective,
};
use health_economics::avoidable_outsourcing_costs::{
    avoidable_outsourcing_saving, net_benefit, outsourcing_premium_ratio, outsourcing_spend,
};
use health_economics::avoided_downstream_costs::{
    attributable_events_avoided, discount_factor, intervention_cost, net_cost, offset_value,
    probability_weighted_offset,
};
use health_economics::bed_days_saved::{
    additional_elective_spells, bed_days_saved_from_avoided_admissions,
    bed_days_saved_from_earlier_discharge, freed_capacity_value, naive_bed_day_value,
    FreedCapacityUse,
};

/// Float-comparison helper: asserts |a − b| < tol with a readable panic message.
fn assert_close(a: f64, b: f64, tol: f64) {
    assert!(
        (a - b).abs() < tol,
        "expected {a} ≈ {b} (tol {tol}, diff {})",
        (a - b).abs()
    );
}

// =========================================================================
// 1. EDGE CASES
// =========================================================================

// ----- activation_and_uptake -----

#[test]
fn edge_activation_and_uptake_zero_denominators_are_none() {
    // Locks down: each Option-returning ratio's exact None condition is a zero denominator.
    assert!(activation_rate_percent(5.0, 0.0).is_none());
    assert!(uptake_rate_percent(5.0, 0.0).is_none());
    assert!(dtx_fill_rate_percent(5.0, 0.0).is_none());
    assert!(per_eligible_person_value(1_000.0, 0.0).is_none());
    // Nonzero denominator with zero numerator is Some(0.0), not None.
    assert_close(activation_rate_percent(0.0, 10.0).unwrap(), 0.0, 1e-12);
    assert_close(uptake_rate_percent(0.0, 10.0).unwrap(), 0.0, 1e-12);
    assert_close(dtx_fill_rate_percent(0.0, 10.0).unwrap(), 0.0, 1e-12);
    assert_close(per_eligible_person_value(0.0, 10.0).unwrap(), 0.0, 1e-12);
}

#[test]
fn edge_funnel_value_zero_at_any_zero_stage() {
    // Locks down: any zero funnel stage collapses population value to exactly zero.
    assert_close(funnel_population_value(0.0, 0.5, 0.5, 0.5, 780.0), 0.0, 1e-12);
    assert_close(funnel_population_value(80_000.0, 0.0, 0.5, 0.5, 780.0), 0.0, 1e-12);
    assert_close(funnel_population_value(80_000.0, 0.5, 0.0, 0.5, 780.0), 0.0, 1e-12);
    assert_close(funnel_population_value(80_000.0, 0.5, 0.5, 0.0, 780.0), 0.0, 1e-12);
    assert_close(funnel_population_value(80_000.0, 0.5, 0.5, 0.5, 0.0), 0.0, 1e-12);
}

#[test]
fn edge_funnel_stage_boundaries_zero_and_one() {
    // Locks down: stage fractions at their 0.0/1.0 boundaries behave as pure pass-through.
    let all_through = funnel_population_value(1_000.0, 1.0, 1.0, 1.0, 500.0);
    assert_close(all_through, population_value(1_000.0, 500.0), 1e-9);
    let none_through = funnel_population_value(1_000.0, 1.0, 1.0, 0.0, 500.0);
    assert_close(none_through, 0.0, 1e-12);
}

#[test]
fn edge_value_per_completer_zero_and_negative_components() {
    // Locks down: zero QALYs leave only avoided costs; a negative avoided-cost term
    // (the intervention adds downstream cost) can push completer value negative.
    assert_close(value_per_completer(0.0, 20_000.0, 180.0), 180.0, 1e-12);
    assert_close(value_per_completer(0.0, 20_000.0, 0.0), 0.0, 1e-12);
    assert!(value_per_completer(0.001, 20_000.0, -100.0) < 0.0);
}

#[test]
fn edge_activation_extreme_magnitudes_stay_finite() {
    // Locks down: 1e12-scale populations produce finite, non-NaN values.
    let v = funnel_population_value(1e12, 0.5, 0.5, 0.5, 1e6);
    assert!(v.is_finite() && !v.is_nan());
    let per_head = per_eligible_person_value(v, 1e12).unwrap();
    assert!(per_head.is_finite());
    assert_close(per_head, 0.5 * 0.5 * 0.5 * 1e6, 1e-3);
}

// ----- adherence_and_persistence -----

#[test]
fn edge_adherence_zero_denominators_are_none() {
    // Locks down: every ratio in the module is None exactly when its denominator is zero.
    assert!(mpr_percent(30.0, 0.0).is_none());
    assert!(pdc_percent(30.0, 0.0).is_none());
    assert!(digital_adherence_percent(3.0, 0.0).is_none());
    assert!(percent_persistent(3.0, 0.0).is_none());
    assert!(cost_per_qaly(1_000.0, 0.0).is_none());
}

#[test]
fn edge_pdc_caps_at_100_but_mpr_does_not_on_oversupply() {
    // Locks down: with oversupply (early refills), MPR exceeds 100 while PDC caps at 100.
    let mpr = mpr_percent(400.0, 365.0).unwrap();
    let pdc = pdc_percent(400.0, 365.0).unwrap();
    assert!(mpr > 100.0);
    assert_close(mpr, 400.0 / 365.0 * 100.0, 1e-9);
    assert_close(pdc, 100.0, 1e-12);
    // Massive oversupply: MPR grows unboundedly, PDC still pinned to 100.
    let mpr_extreme = mpr_percent(1e12, 365.0).unwrap();
    let pdc_extreme = pdc_percent(1e12, 365.0).unwrap();
    assert!(mpr_extreme.is_finite() && mpr_extreme > 1e9);
    assert_close(pdc_extreme, 100.0, 1e-12);
}

#[test]
fn edge_is_adherent_boundary_is_inclusive_at_80() {
    // Locks down: the ≥80% bar is inclusive (>=), so exactly 80.0 is adherent.
    assert!(is_adherent(80.0));
    assert!(!is_adherent(79.999999));
    assert!(is_adherent(80.000001));
    assert!(is_adherent(100.0));
    assert!(!is_adherent(0.0));
}

#[test]
fn edge_persistence_days_negative_when_reversed() {
    // Locks down: reversed day indices yield a negative duration (documented behavior).
    assert_close(persistence_days(42.0, 0.0), -42.0, 1e-12);
    assert_close(persistence_days(10.0, 10.0), 0.0, 1e-12);
}

#[test]
fn edge_qalys_and_spend_zero_inputs() {
    // Locks down: zero prescriptions or zero dose-reaching fraction realize zero value/spend.
    assert_close(qalys_realized(0.0, 0.38, 0.025), 0.0, 1e-12);
    assert_close(qalys_realized(1_000.0, 0.0, 0.025), 0.0, 1e-12);
    assert_close(payer_spend(0.0, 250.0), 0.0, 1e-12);
    // A fully non-adherent cohort makes cost/QALY undefined, not infinite.
    assert!(cost_per_qaly(payer_spend(1_000.0, 250.0), qalys_realized(1_000.0, 0.0, 0.025)).is_none());
}

// ----- ai_developer_productivity -----

#[test]
fn edge_ai_dev_zero_denominators_are_none() {
    // Locks down: exact None conditions — shown, accepted, t_control, before, cost, measured.
    assert!(acceptance_rate(1.0, 0.0).is_none());
    assert!(retention_rate(1.0, 0.0).is_none());
    assert!(speedup(0.0, 1.0).is_none());
    assert!(throughput_delta(0.0, 1.0).is_none());
    assert!(net_capacity_ratio(1.0, 0.0).is_none());
    assert!(perception_gap_ratio(1.0, 0.0).is_none());
}

#[test]
fn edge_speedup_negative_for_metr_style_slowdown() {
    // Locks down: t_AI > t_control gives a negative speedup (METR 2025: −19%).
    let s = speedup(100.0, 119.0).unwrap();
    assert_close(s, -0.19, 1e-12);
    // t_AI == 0 is a valid input: 100% speedup.
    assert_close(speedup(100.0, 0.0).unwrap(), 1.0, 1e-12);
}

#[test]
fn edge_throughput_delta_can_be_negative() {
    // Locks down: a throughput regression yields a negative delta, not None.
    assert_close(throughput_delta(100.0, 90.0).unwrap(), -0.10, 1e-12);
    assert_close(throughput_delta(100.0, 100.0).unwrap(), 0.0, 1e-12);
}

#[test]
fn edge_capacity_value_utilization_boundaries() {
    // Locks down: utilization 0.0 zeroes the value; 1.0 gives the undiscounted product.
    assert_close(annual_capacity_value(500.0, 0.25, 220.0, 60.0, 0.0), 0.0, 1e-12);
    assert_close(
        annual_capacity_value(500.0, 0.25, 220.0, 60.0, 1.0),
        500.0 * 0.25 * 220.0 * 60.0,
        1e-6,
    );
    assert_close(annual_tool_cost(0.0, 39.0), 0.0, 1e-12);
}

#[test]
fn edge_ai_dev_extreme_magnitudes_stay_finite() {
    // Locks down: 1e12-scale inputs do not produce NaN/inf in the value model.
    let v = annual_capacity_value(1e12, 1.0, 220.0, 60.0, 0.5);
    assert!(v.is_finite() && !v.is_nan());
    let ratio = net_capacity_ratio(v, annual_tool_cost(1e12, 39.0)).unwrap();
    assert!(ratio.is_finite());
}

// ----- ai_quality_metrics -----

#[test]
fn edge_ai_quality_zero_denominators_are_none() {
    // Locks down: rates are None with nothing evaluated / no claims made.
    assert!(hallucination_rate(1.0, 0.0).is_none());
    assert!(faithfulness(1.0, 0.0).is_none());
    // Zero numerators are Some(0.0).
    assert_close(hallucination_rate(0.0, 100.0).unwrap(), 0.0, 1e-12);
    assert_close(faithfulness(0.0, 100.0).unwrap(), 0.0, 1e-12);
}

#[test]
fn edge_expected_harm_empty_slice_is_zero() {
    // Locks down: an empty error-type slice sums to exactly 0.0 harm per output.
    assert_close(expected_harm_cost_per_output(&[]), 0.0, 1e-12);
}

#[test]
fn edge_errors_reaching_submission_catch_rate_boundaries() {
    // Locks down: catch rate 1.0 means zero uncaught errors; 0.0 means all errors survive.
    assert_close(errors_reaching_submission(200_000.0, 0.02, 1.0), 0.0, 1e-9);
    assert_close(errors_reaching_submission(200_000.0, 0.02, 0.0), 4_000.0, 1e-9);
    assert_close(errors_reaching_submission(0.0, 0.02, 0.5), 0.0, 1e-12);
    assert_close(expected_error_cost(0.0, 250.0), 0.0, 1e-12);
    assert_close(review_cost(2.0, 0.0, 0.5), 0.0, 1e-12);
}

#[test]
fn edge_harm_cost_extreme_magnitudes_stay_finite() {
    // Locks down: 1e12-scale volumes keep the harm arithmetic finite and exact.
    let uncaught = errors_reaching_submission(1e12, 0.02, 0.85);
    assert!(uncaught.is_finite());
    let cost = expected_error_cost(uncaught, 250.0);
    assert!(cost.is_finite() && !cost.is_nan());
}

// ----- ai_regulatory_evaluation -----

#[test]
fn edge_regulatory_zero_updates_isolate_fixed_costs() {
    // Locks down: with zero updates, traditional cost is 0 but PCCP still pays authoring.
    assert_close(traditional_lifetime_cost(0.0, 80_000.0, 4.0, 50_000.0), 0.0, 1e-12);
    assert_close(pccp_lifetime_cost(250_000.0, 0.0, 30_000.0), 250_000.0, 1e-9);
    // So at zero updates the PCCP route is strictly worse: negative saving.
    assert!(pccp_saving(0.0, 250_000.0) < 0.0);
    assert_close(benefit_months_gained(0.0, 4.0), 0.0, 1e-12);
}

#[test]
fn edge_traditional_update_cost_zero_delay_is_submission_only() {
    // Locks down: with no review delay the cost-of-delay term vanishes entirely.
    assert_close(traditional_update_cost(80_000.0, 0.0, 50_000.0), 80_000.0, 1e-9);
    assert_close(traditional_update_cost(0.0, 0.0, 50_000.0), 0.0, 1e-12);
}

// ----- ai_return_on_investment -----

#[test]
fn edge_ai_roi_zero_cost_is_none_and_zero_benefit_is_total_loss() {
    // Locks down: None exactly at zero investment; zero measured benefit is ROI −1.0.
    assert!(ai_roi(100.0, 0.0).is_none());
    assert_close(ai_roi(0.0, 120_000.0).unwrap(), -1.0, 1e-12);
}

#[test]
fn edge_attributable_benefit_goes_negative_when_new_costs_dominate() {
    // Locks down: new costs above stopped spend give a negative (net-loss) benefit.
    assert!(attributable_benefit(50_000.0, 80_000.0) < 0.0);
    assert_close(attributable_benefit(50_000.0, 80_000.0), -30_000.0, 1e-9);
    assert_close(attributable_benefit(0.0, 0.0), 0.0, 1e-12);
}

#[test]
fn edge_cost_stack_all_zero_lines_total_zero() {
    // Locks down: an empty (all-zero) cost stack totals exactly 0.0.
    let stack = AiCostStack {
        licences_and_inference: 0.0,
        integration: 0.0,
        data_readiness: 0.0,
        evaluation: 0.0,
        workflow_redesign: 0.0,
        governance_and_assurance: 0.0,
    };
    assert_close(stack.total(), 0.0, 1e-12);
    // A zero-total stack makes ROI undefined — consistent None convention.
    assert!(ai_roi(1.0, stack.total()).is_none());
}

// ----- analysis_perspective -----

#[test]
fn edge_perspective_empty_inventory_is_zero_everywhere() {
    // Locks down: an empty impact inventory nets to 0.0 from every perspective.
    let items: Vec<ImpactItem> = vec![];
    assert_close(net_value_from_perspective(&items, Perspective::Payer), 0.0, 1e-12);
    assert_close(net_value_from_perspective(&items, Perspective::Provider), 0.0, 1e-12);
    assert_close(net_value_from_perspective(&items, Perspective::Societal), 0.0, 1e-12);
}

#[test]
fn edge_perspective_helpers_zero_inputs() {
    // Locks down: zero visits/hours/rates give zero currency lines, never NaN.
    assert_close(payer_savings(0.0, 42.0), 0.0, 1e-12);
    assert_close(patient_time_value(10_000.0, 0.0, 15.0), 0.0, 1e-12);
    assert_close(false_reassurance_harm(10_000.0, 0.0, 3_000.0), 0.0, 1e-12);
    // Rate boundary 1.0: every diverted visit harms.
    assert_close(false_reassurance_harm(100.0, 1.0, 3_000.0), 300_000.0, 1e-9);
}

// ----- avoidable_outsourcing_costs -----

#[test]
fn edge_outsourcing_premium_ratio_none_at_zero_scheme_price() {
    // Locks down: the premium ratio is None exactly when the internal reference price is zero.
    assert!(outsourcing_premium_ratio(900.0, 0.0).is_none());
    assert_close(outsourcing_premium_ratio(900.0, 900.0).unwrap(), 1.0, 1e-12);
}

#[test]
fn edge_outsourcing_saving_negative_when_internal_costs_more() {
    // Locks down: internal marginal cost above the external price flips the saving negative.
    let saving = avoidable_outsourcing_saving(500.0, 900.0, 1_100.0);
    assert_close(saving, -100_000.0, 1e-9);
    assert!(net_benefit(saving, 90_000.0) < 0.0);
    assert_close(outsourcing_spend(0.0, 900.0), 0.0, 1e-12);
}

// ----- avoided_downstream_costs -----

#[test]
fn edge_events_avoided_negative_when_intervention_arm_worse() {
    // Locks down: an intervention with more events than baseline yields negative avoided events.
    let events = attributable_events_avoided(5_000.0, 0.031, 0.040);
    assert_close(events, -45.0, 1e-9);
    // Which flows through to a negative offset and a higher (positive) net cost.
    let offset = offset_value(events, 3_200.0);
    assert!(offset < 0.0);
    assert!(net_cost(intervention_cost(5_000.0, 20.0), &[offset]) > 100_000.0);
}

#[test]
fn edge_net_cost_empty_offsets_is_full_intervention_cost() {
    // Locks down: an empty offset slice subtracts nothing.
    assert_close(net_cost(100_000.0, &[]), 100_000.0, 1e-9);
    assert_close(net_cost(0.0, &[]), 0.0, 1e-12);
}

#[test]
fn edge_discount_factor_boundaries() {
    // Locks down: factor is exactly 1.0 at year 0 or at a 0% rate, and stays finite far out.
    assert_close(discount_factor(0.035, 0.0), 1.0, 1e-12);
    assert_close(discount_factor(0.0, 40.0), 1.0, 1e-12);
    let far = discount_factor(0.035, 1_000.0);
    assert!(far.is_finite() && far > 0.0 && far < 1e-9);
}

#[test]
fn edge_probability_weighted_offset_zero_probability() {
    // Locks down: probability 0.0 kills the offset; probability 1.0 is the full discounted cost.
    let df = discount_factor(0.035, 4.0);
    assert_close(probability_weighted_offset(0.0, 500_000.0, df), 0.0, 1e-12);
    assert_close(probability_weighted_offset(1.0, 500_000.0, df), 500_000.0 * df, 1e-9);
}

// ----- bed_days_saved -----

#[test]
fn edge_bed_days_zero_inputs() {
    // Locks down: zero patients or zero LOS reduction save zero bed days on both routes.
    assert_close(bed_days_saved_from_earlier_discharge(0.0, 2.0), 0.0, 1e-12);
    assert_close(bed_days_saved_from_earlier_discharge(600.0, 0.0), 0.0, 1e-12);
    assert_close(bed_days_saved_from_avoided_admissions(0.0, 3.0), 0.0, 1e-12);
    assert_close(naive_bed_day_value(0.0, 400.0), 0.0, 1e-12);
}

#[test]
fn edge_refill_zero_elective_los_is_none_other_mechanisms_never_none() {
    // Locks down: freed_capacity_value is None ONLY for RefilledWithElective with LOS 0.
    let refill_zero = FreedCapacityUse::RefilledWithElective {
        average_elective_stay_days: 0.0,
        income_per_spell: 6_000.0,
    };
    assert!(freed_capacity_value(1_200.0, &refill_zero).is_none());
    assert!(additional_elective_spells(1_200.0, 0.0).is_none());
    // Ward closure and slack always return Some, even for zero bed days.
    let closed = FreedCapacityUse::WardClosedOrFlexedDown { cost_released_per_bed_day: 400.0 };
    let slack = FreedCapacityUse::AbsorbedAsSlack { marginal_hotel_cost_per_bed_day: 100.0 };
    assert_close(freed_capacity_value(0.0, &closed).unwrap(), 0.0, 1e-12);
    assert_close(freed_capacity_value(0.0, &slack).unwrap(), 0.0, 1e-12);
}

#[test]
fn edge_bed_days_extreme_magnitudes_stay_finite() {
    // Locks down: 1e12 bed days keep every mechanism's valuation finite and non-NaN.
    let refill = FreedCapacityUse::RefilledWithElective {
        average_elective_stay_days: 3.0,
        income_per_spell: 6_000.0,
    };
    let v = freed_capacity_value(1e12, &refill).unwrap();
    assert!(v.is_finite() && !v.is_nan());
    assert!(naive_bed_day_value(1e12, 400.0).is_finite());
}

// =========================================================================
// 2. PROPERTIES / INVARIANTS
// =========================================================================

#[test]
fn prop_funnel_equals_population_value_of_equivalent_completers() {
    // Locks down: funnel_population_value == population_value(eligible×stages, vpc) over a grid.
    let fractions = [0.0, 0.1, 0.25, 0.5, 0.8, 1.0];
    for &uptake in &fractions {
        for &activation in &fractions {
            for &completion in &fractions {
                let eligible = 80_000.0;
                let vpc = 780.0;
                let funnel = funnel_population_value(eligible, uptake, activation, completion, vpc);
                let completers = eligible * uptake * activation * completion;
                let direct = population_value(completers, vpc);
                assert_close(funnel, direct, 1e-6);
            }
        }
    }
}

#[test]
fn prop_per_eligible_value_times_eligible_recovers_population_value() {
    // Locks down: per-eligible value × eligible population == population value over a grid.
    for &eligible in &[1.0, 100.0, 80_000.0, 1e9] {
        for &completers in &[0.0, 10.0, 1_600.0] {
            let pv = population_value(completers, 780.0);
            let per_head = per_eligible_person_value(pv, eligible).unwrap();
            assert_close(per_head * eligible, pv, 1e-6 * (1.0 + pv.abs()));
        }
    }
}

#[test]
fn prop_rates_in_0_to_100_for_valid_inputs() {
    // Locks down: percent functions stay in [0,100] whenever numerator ≤ denominator.
    let numerators = [0.0, 1.0, 37.0, 100.0];
    let denominators = [100.0, 250.0, 1e6];
    for &n in &numerators {
        for &d in &denominators {
            assert!(n <= d);
            for rate in [
                activation_rate_percent(n, d).unwrap(),
                uptake_rate_percent(n, d).unwrap(),
                dtx_fill_rate_percent(n, d).unwrap(),
                digital_adherence_percent(n, d).unwrap(),
                percent_persistent(n, d).unwrap(),
                mpr_percent(n, d).unwrap(),
                pdc_percent(n, d).unwrap(),
            ] {
                assert!(
                    (0.0..=100.0).contains(&rate),
                    "rate {rate} out of [0,100] for {n}/{d}"
                );
            }
        }
    }
}

#[test]
fn prop_pdc_equals_mpr_when_no_oversupply_diverges_above() {
    // Locks down: PDC == MPR below 100% coverage; above it only PDC saturates.
    for &covered in &[0.0, 100.0, 292.0, 365.0] {
        let mpr = mpr_percent(covered, 365.0).unwrap();
        let pdc = pdc_percent(covered, 365.0).unwrap();
        assert_close(mpr, pdc, 1e-9); // ≤100%: identical estimators.
    }
    for &covered in &[366.0, 400.0, 730.0] {
        let mpr = mpr_percent(covered, 365.0).unwrap();
        let pdc = pdc_percent(covered, 365.0).unwrap();
        assert!(mpr > 100.0);
        assert_close(pdc, 100.0, 1e-12);
        assert!(mpr > pdc);
    }
}

#[test]
fn prop_is_adherent_matches_ge_80_over_grid() {
    // Locks down: is_adherent is exactly the predicate (percent >= 80.0), nothing subtler.
    for &pct in &[0.0, 50.0, 79.0, 79.99, 80.0, 80.01, 99.0, 100.0, 120.0] {
        assert_eq!(is_adherent(pct), pct >= 80.0, "boundary mismatch at {pct}");
    }
}

#[test]
fn prop_expected_harm_additive_across_error_types() {
    // Locks down: harm over a combined slice == sum of harms over singleton slices.
    let make = |rate: f64, undetected: f64, acted: f64, cost: f64| ErrorType {
        rate,
        probability_undetected: undetected,
        probability_acted_upon: acted,
        cost_per_acted_upon_error: cost,
    };
    let a = make(0.02, 0.15, 1.0, 250.0);
    let b = make(0.001, 0.2, 0.5, 10_000.0);
    let c = make(0.05, 0.9, 0.1, 40.0);
    let combined = expected_harm_cost_per_output(&[
        make(0.02, 0.15, 1.0, 250.0),
        make(0.001, 0.2, 0.5, 10_000.0),
        make(0.05, 0.9, 0.1, 40.0),
    ]);
    let separate = expected_harm_cost_per_output(&[a])
        + expected_harm_cost_per_output(&[b])
        + expected_harm_cost_per_output(&[c]);
    assert_close(combined, separate, 1e-12);
}

#[test]
fn prop_zero_rate_error_type_contributes_nothing() {
    // Locks down: an ErrorType with rate 0.0 adds exactly zero to expected harm.
    let live = ErrorType {
        rate: 0.02,
        probability_undetected: 0.15,
        probability_acted_upon: 1.0,
        cost_per_acted_upon_error: 250.0,
    };
    let dead = ErrorType {
        rate: 0.0,
        probability_undetected: 1.0,
        probability_acted_upon: 1.0,
        cost_per_acted_upon_error: 1e9,
    };
    let with_dead = expected_harm_cost_per_output(&[
        ErrorType { ..live },
        dead,
    ]);
    let without = expected_harm_cost_per_output(&[live]);
    assert_close(with_dead, without, 1e-12);
}

#[test]
fn prop_faithfulness_and_hallucination_complementary_construction() {
    // Locks down: rates built from complementary counts sum to 1 (n/d + (d−n)/d == 1).
    for &(n, d) in &[(0.0, 10.0), (3.0, 10.0), (97.0, 100.0), (100.0, 100.0)] {
        let f = faithfulness(n, d).unwrap();
        let h = hallucination_rate(d - n, d).unwrap();
        assert_close(f + h, 1.0, 1e-12);
    }
}

#[test]
fn prop_societal_net_decomposes_into_payer_plus_time_minus_harm() {
    // Locks down: societal net == payer savings + patient time − harm when built
    // from the module's own ImpactItems over a grid of scenarios.
    for &visits in &[0.0, 1_000.0, 10_000.0] {
        for &harm_rate in &[0.0, 0.02, 0.10] {
            let payer_line = payer_savings(visits, 42.0);
            let time_line = patient_time_value(visits, 2.0, 15.0);
            let harm_line = false_reassurance_harm(visits, harm_rate, 3_000.0);
            let items = vec![
                ImpactItem {
                    amount: payer_line,
                    counts_for_payer: true,
                    counts_for_provider: false,
                    counts_for_societal: true,
                },
                ImpactItem {
                    amount: time_line,
                    counts_for_payer: false,
                    counts_for_provider: false,
                    counts_for_societal: true,
                },
                ImpactItem {
                    amount: -harm_line,
                    counts_for_payer: false,
                    counts_for_provider: false,
                    counts_for_societal: true,
                },
            ];
            let payer = net_value_from_perspective(&items, Perspective::Payer);
            let societal = net_value_from_perspective(&items, Perspective::Societal);
            assert_close(payer, payer_line, 1e-9);
            assert_close(societal, payer_line + time_line - harm_line, 1e-9);
        }
    }
}

#[test]
fn prop_item_in_no_perspective_contributes_to_none() {
    // Locks down: an item excluded from all perspectives changes no perspective's net.
    let phantom = ImpactItem {
        amount: 1e9,
        counts_for_payer: false,
        counts_for_provider: false,
        counts_for_societal: false,
    };
    assert!(!phantom.included_in(Perspective::Payer));
    assert!(!phantom.included_in(Perspective::Provider));
    assert!(!phantom.included_in(Perspective::Societal));
    let base = ImpactItem {
        amount: 100.0,
        counts_for_payer: true,
        counts_for_provider: true,
        counts_for_societal: true,
    };
    assert!(base.included_in(Perspective::Provider));
    let items = vec![base, phantom];
    for p in [Perspective::Payer, Perspective::Provider, Perspective::Societal] {
        assert_close(net_value_from_perspective(&items, p), 100.0, 1e-9);
    }
}

#[test]
fn prop_refill_value_is_spells_times_income_slack_matches_naive_form() {
    // Locks down: refill == additional_elective_spells × income (divides by elective LOS);
    // slack == bed_days × marginal hotel cost, i.e. the naive formula at the marginal rate.
    for &bed_days in &[0.0, 300.0, 1_200.0, 50_000.0] {
        for &los in &[1.0, 3.0, 7.5] {
            let refill = FreedCapacityUse::RefilledWithElective {
                average_elective_stay_days: los,
                income_per_spell: 6_000.0,
            };
            let value = freed_capacity_value(bed_days, &refill).unwrap();
            let spells = additional_elective_spells(bed_days, los).unwrap();
            assert_close(value, spells * 6_000.0, 1e-6);
        }
        let slack = FreedCapacityUse::AbsorbedAsSlack { marginal_hotel_cost_per_bed_day: 100.0 };
        // Slack valuation is exactly the naive bed-day formula priced at marginal hotel cost.
        assert_close(
            freed_capacity_value(bed_days, &slack).unwrap(),
            naive_bed_day_value(bed_days, 100.0),
            1e-9,
        );
        let closed = FreedCapacityUse::WardClosedOrFlexedDown { cost_released_per_bed_day: 400.0 };
        // Ward closure is the only mechanism where the naive average-cost claim is honest.
        assert_close(
            freed_capacity_value(bed_days, &closed).unwrap(),
            naive_bed_day_value(bed_days, 400.0),
            1e-9,
        );
    }
}

#[test]
fn prop_ai_roi_zero_at_breakeven_and_symmetric_scaling() {
    // Locks down: benefit == cost gives ROI exactly 0; ROI is scale-invariant in currency.
    for &cost in &[1.0, 120_000.0, 1e9] {
        assert_close(ai_roi(cost, cost).unwrap(), 0.0, 1e-12);
        // Scaling both by 1,000 leaves the ROI fraction unchanged.
        let roi = ai_roi(320_000.0, cost).unwrap();
        let scaled = ai_roi(320_000.0 * 1_000.0, cost * 1_000.0).unwrap();
        assert_close(roi, scaled, 1e-9);
    }
}

#[test]
fn prop_attributable_benefit_subtracts_new_costs() {
    // Locks down: benefit == stopped spend − new costs, exactly, over a grid.
    for &stopped in &[0.0, 100.0, 380_000.0] {
        for &new_cost in &[0.0, 60_000.0, 500_000.0] {
            assert_close(attributable_benefit(stopped, new_cost), stopped - new_cost, 1e-9);
        }
    }
}

#[test]
fn prop_cost_stack_total_is_sum_of_lines() {
    // Locks down: AiCostStack::total is the plain sum of its six lines.
    let stack = AiCostStack {
        licences_and_inference: 30_000.0,
        integration: 40_000.0,
        data_readiness: 20_000.0,
        evaluation: 10_000.0,
        workflow_redesign: 15_000.0,
        governance_and_assurance: 5_000.0,
    };
    assert_close(
        stack.total(),
        30_000.0 + 40_000.0 + 20_000.0 + 10_000.0 + 15_000.0 + 5_000.0,
        1e-9,
    );
    assert_close(stack.total(), 120_000.0, 1e-9);
}

#[test]
fn prop_traditional_lifetime_is_n_times_update_cost() {
    // Locks down: lifetime cost == n × per-update cost over a grid (linearity in n).
    for &n in &[0.0, 1.0, 4.0, 12.0, 100.0] {
        let per_update = traditional_update_cost(80_000.0, 4.0, 50_000.0);
        let lifetime = traditional_lifetime_cost(n, 80_000.0, 4.0, 50_000.0);
        assert_close(lifetime, n * per_update, 1e-6);
        // benefit_months_gained is likewise linear in n.
        assert_close(benefit_months_gained(n, 4.0), n * 4.0, 1e-9);
    }
}

#[test]
fn prop_pccp_saving_antisymmetric() {
    // Locks down: pccp_saving(a, b) == −pccp_saving(b, a); zero when routes cost the same.
    let a = traditional_lifetime_cost(12.0, 80_000.0, 4.0, 50_000.0);
    let b = pccp_lifetime_cost(250_000.0, 12.0, 30_000.0);
    assert_close(pccp_saving(a, b), -pccp_saving(b, a), 1e-9);
    assert_close(pccp_saving(a, a), 0.0, 1e-12);
}

#[test]
fn prop_qalys_and_cost_per_qaly_roundtrip() {
    // Locks down: cost_per_qaly(spend, qalys) × qalys recovers spend; better adherence
    // strictly lowers cost per QALY.
    let spend = payer_spend(1_000.0, 250.0);
    let mut prev_icer = f64::INFINITY;
    for &fraction in &[0.1, 0.38, 0.5, 0.9, 1.0] {
        let qalys = qalys_realized(1_000.0, fraction, 0.025);
        let icer = cost_per_qaly(spend, qalys).unwrap();
        assert_close(icer * qalys, spend, 1e-6);
        assert!(icer < prev_icer, "cost/QALY must fall as adherence rises");
        prev_icer = icer;
    }
}

#[test]
fn prop_speedup_and_perception_gap_identities() {
    // Locks down: speedup(t, t) == 0 (no change) and perception_gap_ratio(x, x) == 1
    // (self-report matching measurement) over a grid.
    for &t in &[1.0, 60.0, 161.0, 1e6] {
        assert_close(speedup(t, t).unwrap(), 0.0, 1e-12);
        assert_close(perception_gap_ratio(t, t).unwrap(), 1.0, 1e-12);
    }
    // Acceptance/retention of everything is exactly 1.0.
    assert_close(acceptance_rate(250.0, 250.0).unwrap(), 1.0, 1e-12);
    assert_close(retention_rate(250.0, 250.0).unwrap(), 1.0, 1e-12);
}

#[test]
fn prop_net_cost_linear_in_offsets() {
    // Locks down: net_cost subtracts the sum of the offset slice — order/grouping irrelevant.
    let cost = intervention_cost(5_000.0, 20.0);
    let single = net_cost(cost, &[144_000.0]);
    let split = net_cost(cost, &[100_000.0, 44_000.0]);
    let reordered = net_cost(cost, &[44_000.0, 100_000.0]);
    assert_close(single, split, 1e-9);
    assert_close(split, reordered, 1e-9);
    assert_close(single, cost - 144_000.0, 1e-9);
}

#[test]
fn prop_discount_factor_monotone_decreasing_in_rate_and_years() {
    // Locks down: the factor strictly falls as rate or horizon grows, and stays in (0, 1].
    let rates = [0.0, 0.015, 0.035, 0.10];
    let years = [0.0, 1.0, 4.0, 10.0, 40.0];
    for &y in &years {
        let mut prev = f64::INFINITY;
        for &r in &rates {
            let df = discount_factor(r, y);
            assert!(df > 0.0 && df <= 1.0);
            if y > 0.0 {
                assert!(df < prev || r == 0.0);
            }
            prev = df;
        }
    }
    for &r in &[0.035, 0.10] {
        let mut prev = 2.0;
        for &y in &years {
            let df = discount_factor(r, y);
            assert!(df < prev, "factor must fall with years at rate {r}");
            prev = df;
        }
    }
}

// =========================================================================
// 3. CROSS-MODULE CONSISTENCY
// =========================================================================

#[test]
fn cross_net_benefit_and_ai_roi_tell_same_adopt_reject_story() {
    // Locks down: for the same benefit/cost pair, roi > 0 ⟺ net benefit > 0 (and same for
    // < 0 and == 0) — the outsourcing and AI-ROI modules agree on the decision.
    let benefits = [0.0, 90_000.0, 185_000.0, 275_000.0, 1e9];
    let costs = [1.0, 90_000.0, 275_000.0, 5e8];
    for &b in &benefits {
        for &c in &costs {
            let net = net_benefit(b, c);
            let roi = ai_roi(b, c).unwrap();
            assert_eq!(roi > 0.0, net > 0.0, "adopt signal mismatch at b={b} c={c}");
            assert_eq!(roi < 0.0, net < 0.0, "reject signal mismatch at b={b} c={c}");
            // Exact identity: roi == net / cost.
            assert_close(roi, net / c, 1e-9);
        }
    }
}

#[test]
fn cross_net_capacity_ratio_is_ai_roi_plus_one() {
    // Locks down: value/cost ratio == ROI + 1 for identical inputs across the two AI modules.
    for &value in &[0.0, 234_000.0, 990_000.0, 3e6] {
        for &cost in &[1.0, 234_000.0, 1e6] {
            let ratio = net_capacity_ratio(value, cost).unwrap();
            let roi = ai_roi(value, cost).unwrap();
            assert_close(ratio, roi + 1.0, 1e-9);
        }
    }
}

#[test]
fn cross_discount_factor_agrees_with_closed_form_and_pw_offset() {
    // Locks down: discount_factor matches 1/(1+r)^t on a grid, and
    // probability_weighted_offset uses it as a plain multiplier.
    for &r in &[0.0, 0.015, 0.035, 0.10] {
        for &t in &[0.0, 1.0, 2.0, 4.0, 10.0, 25.0] {
            let df = discount_factor(r, t);
            let closed_form = 1.0 / (1.0 + r).powf(t);
            assert_close(df, closed_form, 1e-12);
            assert_close(
                probability_weighted_offset(0.6, 500_000.0, df),
                0.6 * 500_000.0 * closed_form,
                1e-6,
            );
        }
    }
}

#[test]
fn cross_bed_day_routes_and_spells_are_mutually_consistent() {
    // Locks down: earlier-discharge and avoided-admission routes produce identical bed days
    // when parameterized consistently, and the refill spell count inverts avoided admissions.
    for &(patients, delta_los) in &[(600.0, 2.0), (100.0, 4.5), (1.0, 1.0)] {
        let via_discharge = bed_days_saved_from_earlier_discharge(patients, delta_los);
        for &avg_los in &[1.5, 3.0, 6.0] {
            let admissions = via_discharge / avg_los;
            let via_avoidance = bed_days_saved_from_avoided_admissions(admissions, avg_los);
            assert_close(via_discharge, via_avoidance, 1e-9);
            // Spell arithmetic round-trips: bed days ÷ LOS == the admissions that made them.
            let spells = additional_elective_spells(via_discharge, avg_los).unwrap();
            assert_close(spells, admissions, 1e-9);
        }
    }
}

#[test]
fn cross_offset_pipeline_matches_perspective_inventory() {
    // Locks down: avoided_downstream_costs' net cost equals the (negated) payer-perspective
    // net of an inventory holding the same two lines — two modules, one arithmetic.
    let events = attributable_events_avoided(5_000.0, 0.040, 0.031);
    let offset = offset_value(events, 3_200.0);
    let cost = intervention_cost(5_000.0, 20.0);
    let net = net_cost(cost, &[offset]);
    let items = vec![
        // Offset is a payer benefit (+), intervention cost a payer cost (−).
        ImpactItem {
            amount: offset,
            counts_for_payer: true,
            counts_for_provider: false,
            counts_for_societal: true,
        },
        ImpactItem {
            amount: -cost,
            counts_for_payer: true,
            counts_for_provider: false,
            counts_for_societal: true,
        },
    ];
    let payer_net = net_value_from_perspective(&items, Perspective::Payer);
    assert_close(payer_net, -net, 1e-9); // net *cost* is the negative of net *value*.
    assert!(payer_net > 0.0 && net < 0.0); // both say: cost-saving.
}

#[test]
fn cross_funnel_completion_matches_adherence_value_gating() {
    // Locks down: activation_and_uptake's funnel and adherence_and_persistence's value
    // gating are the same multiplication — completers × per-completer value.
    let prescriptions = 1_000.0;
    let fraction_effective = 0.38;
    let qalys_each = 0.025;
    let wtp = 20_000.0;
    // Adherence route: gated QALYs monetized at WTP.
    let qalys = qalys_realized(prescriptions, fraction_effective, qalys_each);
    let monetized = qalys * wtp;
    // Funnel route: prescriptions as "eligible", one real gate, £/completer from WTP.
    let vpc = value_per_completer(qalys_each, wtp, 0.0);
    let funnel = funnel_population_value(prescriptions, 1.0, 1.0, fraction_effective, vpc);
    assert_close(monetized, funnel, 1e-6);
}

#[test]
fn cross_regulatory_saving_consistent_with_roi_sign() {
    // Locks down: pccp_saving > 0 ⟺ ai_roi(traditional, pccp) > 0 — treating the avoided
    // traditional spend as benefit and the PCCP route as the cost.
    for &(n, authoring) in &[(12.0, 250_000.0), (1.0, 250_000.0), (0.5, 1e7)] {
        let traditional = traditional_lifetime_cost(n, 80_000.0, 4.0, 50_000.0);
        let pccp = pccp_lifetime_cost(authoring, n, 30_000.0);
        let saving = pccp_saving(traditional, pccp);
        let roi = ai_roi(traditional, pccp).unwrap();
        assert_eq!(saving > 0.0, roi > 0.0, "n={n} authoring={authoring}");
    }
}

// =========================================================================
// 4. DOMAIN SCENARIOS
// =========================================================================

#[test]
fn scenario_digital_therapeutic_funnel_to_monetized_health_value() {
    // Scenario: a prescribed digital therapeutic runs the full funnel — eligible
    // population → prescriptions → activation → adherence-gated completion →
    // monetized QALYs — with every stage hand-computed.
    //
    // 50,000 eligible; 10,000 prescriptions issued; 8,100 codes activated (81% DiGA
    // fill rate); 4,050 activate clinically (50%); 1,620 reach the minimum effective
    // dose (40% of activated); 0.02 QALYs + £150 avoided costs per completer at
    // £20,000/QALY.
    let uptake = uptake_rate_percent(10_000.0, 50_000.0).unwrap();
    assert_close(uptake, 20.0, 1e-9);
    let fill = dtx_fill_rate_percent(8_100.0, 10_000.0).unwrap();
    assert_close(fill, 81.0, 1e-9);
    let activation = activation_rate_percent(4_050.0, 8_100.0).unwrap();
    assert_close(activation, 50.0, 1e-9);
    let completion_pct = percent_persistent(1_620.0, 4_050.0).unwrap();
    assert_close(completion_pct, 40.0, 1e-9);
    // Per-completer value: 0.02 × £20,000 + £150 = £550.
    let vpc = value_per_completer(0.02, 20_000.0, 150.0);
    assert_close(vpc, 550.0, 1e-9);
    // Funnel value: 50,000 × 0.2 × 0.81 × 0.5 × 0.4 × 550 — computed both ways.
    let completers = 50_000.0 * 0.2 * 0.81 * 0.5 * 0.4;
    assert_close(completers, 1_620.0, 1e-6);
    let pop_value = population_value(completers, vpc);
    assert_close(pop_value, 891_000.0, 1e-6); // 1,620 × £550 hand-computed.
    let funnel = funnel_population_value(50_000.0, 0.2, 0.81 * 0.5 * 0.4, 1.0, vpc);
    assert_close(funnel, pop_value, 1e-6);
    // Per eligible head: £891,000 / 50,000 = £17.82.
    let per_head = per_eligible_person_value(pop_value, 50_000.0).unwrap();
    assert_close(per_head, 17.82, 1e-9);
    // Payer economics: 10,000 prescriptions × £120 = £1.2M spend; QALYs gated at
    // the effective dose: 10,000 × 0.162 × 0.02 = 32.4 → £37,037/QALY (hand: 1.2e6/32.4).
    let spend = payer_spend(10_000.0, 120.0);
    assert_close(spend, 1_200_000.0, 1e-6);
    let qalys = qalys_realized(10_000.0, 0.162, 0.02);
    assert_close(qalys, 32.4, 1e-9);
    let icer = cost_per_qaly(spend, qalys).unwrap();
    assert_close(icer, 1_200_000.0 / 32.4, 1e-6);
    assert!(icer > 30_000.0); // above the NICE £20k–£30k band: adherence work needed.
    // Adherence lens on the protocol: 4 of 6 modules ≈ 66.7% — below the 80% bar.
    let adherence = digital_adherence_percent(4.0, 6.0).unwrap();
    assert!(!is_adherent(adherence));
    // Persistence through the 6-week course: 42 days.
    assert_close(persistence_days(0.0, 42.0), 42.0, 1e-9);
}

#[test]
fn scenario_ai_coding_tool_roi_with_perception_gap_and_quality_harms() {
    // Scenario: 200 developers pilot an AI assistant. Self-report says 60 min/day;
    // the controlled measurement says 12 min/day (0.2 h) — a 5× perception gap.
    // Fund the measured number, price the full cost stack, and net off a quality harm.
    let gap = perception_gap_ratio(60.0, 12.0).unwrap();
    assert_close(gap, 5.0, 1e-9);
    // Controlled speedup: 50-minute control task done in 45 minutes → 10%.
    let s = speedup(50.0, 45.0).unwrap();
    assert_close(s, 0.10, 1e-9);
    // Telemetry: 4,200 of 12,000 suggestions accepted (35%), 3,780 surviving to merge (90%).
    assert_close(acceptance_rate(4_200.0, 12_000.0).unwrap(), 0.35, 1e-9);
    assert_close(retention_rate(3_780.0, 4_200.0).unwrap(), 0.90, 1e-9);
    // Throughput: 4.0 → 4.3 merged PRs/dev/week = +7.5%.
    assert_close(throughput_delta(4.0, 4.3).unwrap(), 0.075, 1e-9);
    // Capacity value: 200 × 0.2h × 220d × £70 × 0.5 = £308,000/year (hand-computed).
    let value = annual_capacity_value(200.0, 0.2, 220.0, 70.0, 0.5);
    assert_close(value, 308_000.0, 1e-6);
    // Full cost stack (licence a minority line): totals £150,000/year.
    let stack = AiCostStack {
        licences_and_inference: 200.0 * 30.0 * 12.0 / 2.0, // £36,000 licence+inference
        integration: 40_000.0,
        data_readiness: 14_000.0,
        evaluation: 25_000.0,
        workflow_redesign: 20_000.0,
        governance_and_assurance: 15_000.0,
    };
    assert_close(stack.total(), 150_000.0, 1e-6);
    // Licence-only view would be 200 × £30 × 12 = £72,000 — under half the true stack.
    let licence_only = annual_tool_cost(200.0, 30.0);
    assert_close(licence_only, 72_000.0, 1e-6);
    assert!(licence_only < stack.total());
    // Quality harm arm: 1% error rate on 50,000 AI-touched changes/yr, reviewers catch 90%,
    // £400 per escaped defect → 50 escapes, £20,000/year.
    let escapes = errors_reaching_submission(50_000.0, 0.01, 0.90);
    assert_close(escapes, 50.0, 1e-6);
    let harm = expected_error_cost(escapes, 400.0);
    assert_close(harm, 20_000.0, 1e-6);
    // Same harm via the per-output weighting: 0.01 × 0.10 × 1.0 × £400 × 50,000.
    let per_output = expected_harm_cost_per_output(&[ErrorType {
        rate: 0.01,
        probability_undetected: 0.10,
        probability_acted_upon: 1.0,
        cost_per_acted_upon_error: 400.0,
    }]);
    assert_close(per_output * 50_000.0, harm, 1e-6);
    // Review layer: 1 min per change × 50,000 × £1/min = £50,000/year, costed not free.
    let review = review_cost(1.0, 50_000.0, 1.0);
    assert_close(review, 50_000.0, 1e-6);
    // Attributable benefit nets the harm and review lines off the capacity value:
    // 308,000 − 20,000 − 50,000 = £238,000.
    let benefit = attributable_benefit(value, harm + review);
    assert_close(benefit, 238_000.0, 1e-6);
    // ROI on the full stack: (238,000 − 150,000) / 150,000 ≈ 58.7% — adopt.
    let roi = ai_roi(benefit, stack.total()).unwrap();
    assert_close(roi, 88_000.0 / 150_000.0, 1e-9);
    assert!(roi > 0.0);
    // The capacity ratio view agrees: 238/150 ≈ 1.59 > 1.
    let ratio = net_capacity_ratio(benefit, stack.total()).unwrap();
    assert_close(ratio, roi + 1.0, 1e-9);
    // Had we budgeted on self-report (5× the measured saving), the benefit line would
    // have been 5× the capacity value — the exact bias the METR study exposed.
    let self_report_value = annual_capacity_value(200.0, 1.0, 220.0, 70.0, 0.5);
    assert_close(self_report_value, value * gap, 1e-6);
}

#[test]
fn scenario_insourcing_decision_with_bed_days_and_discounted_offsets() {
    // Scenario: a trust decides whether scheduling software that frees beds and
    // repatriates outsourced surgery is worth £150,000/year, from payer and societal
    // perspectives, with a discounted future offset — all hand-computed.
    //
    // Bed capacity: 500 patients discharged 1.5 days earlier = 750 bed days.
    let bed_days = bed_days_saved_from_earlier_discharge(500.0, 1.5);
    assert_close(bed_days, 750.0, 1e-9);
    // Naive claim: 750 × £400 = £300,000 — rejected (no ward closes).
    let naive = naive_bed_day_value(bed_days, 400.0);
    assert_close(naive, 300_000.0, 1e-6);
    // Honest mechanism: refill with electives (2.5-day LOS, £5,000/spell) → 300 spells,
    // £1.5M of funded activity (non-cash-releasing).
    let spells = additional_elective_spells(bed_days, 2.5).unwrap();
    assert_close(spells, 300.0, 1e-9);
    let refill_value = freed_capacity_value(
        bed_days,
        &FreedCapacityUse::RefilledWithElective {
            average_elective_stay_days: 2.5,
            income_per_spell: 5_000.0,
        },
    )
    .unwrap();
    assert_close(refill_value, 1_500_000.0, 1e-6);
    // Insourcing: baseline 600 cases outsourced at £1,000 (spend £600,000, premium 1.25×
    // over the £800 scheme price); 400 repatriated at £400 marginal cost.
    let baseline_spend = outsourcing_spend(600.0, 1_000.0);
    assert_close(baseline_spend, 600_000.0, 1e-6);
    let premium = outsourcing_premium_ratio(1_000.0, 800.0).unwrap();
    assert_close(premium, 1.25, 1e-9);
    let saving = avoidable_outsourcing_saving(400.0, 1_000.0, 400.0);
    assert_close(saving, 240_000.0, 1e-6); // 400 × (1,000 − 400), cash-releasing.
    let net = net_benefit(saving, 150_000.0);
    assert_close(net, 90_000.0, 1e-6);
    assert_eq!(net > 0.0, ai_roi(saving, 150_000.0).unwrap() > 0.0); // same adopt story.
    // Downstream offset: fewer avoidable readmissions among 2,000 monitored patients
    // (5% → 4.2%) at £2,500 marginal cost = 16 events × £2,500 = £40,000/year.
    let events = attributable_events_avoided(2_000.0, 0.050, 0.042);
    assert_close(events, 16.0, 1e-9);
    let offset = offset_value(events, 2_500.0);
    assert_close(offset, 40_000.0, 1e-6);
    // Plus a 50%-likely £200,000 system replacement avoided 3 years out at 3.5%:
    // 0.5 × 200,000 × 1/1.035³ ≈ £90,194.27 present value.
    let df = discount_factor(0.035, 3.0);
    let future_offset = probability_weighted_offset(0.5, 200_000.0, df);
    assert_close(future_offset, 100_000.0 / 1.035_f64.powi(3), 1e-6);
    // Net cost of the software year-1: 150,000 − 240,000 − 40,000 < 0 → dominant.
    let year1_net_cost = net_cost(150_000.0, &[saving, offset]);
    assert_close(year1_net_cost, -130_000.0, 1e-6);
    assert!(year1_net_cost < 0.0);
    // Perspective check: patients also save 300 spells × 4 h × £15 = £18,000 of time,
    // visible only societally; regulatory line: the vendor's PCCP saves review delay.
    let items = vec![
        ImpactItem {
            amount: saving,
            counts_for_payer: true,
            counts_for_provider: true,
            counts_for_societal: true,
        },
        ImpactItem {
            amount: patient_time_value(300.0, 4.0, 15.0),
            counts_for_payer: false,
            counts_for_provider: false,
            counts_for_societal: true,
        },
        ImpactItem {
            amount: -150_000.0,
            counts_for_payer: true,
            counts_for_provider: true,
            counts_for_societal: true,
        },
    ];
    let payer_view = net_value_from_perspective(&items, Perspective::Payer);
    let societal_view = net_value_from_perspective(&items, Perspective::Societal);
    assert_close(payer_view, 90_000.0, 1e-6);
    assert_close(societal_view, 90_000.0 + 18_000.0, 1e-6);
    assert!(societal_view > payer_view);
    // Vendor side: 4 updates under a PCCP (£100k authoring, £20k/protocol) vs
    // traditional 4 × (£50k + 2 × £40k) = £520k → saving £340k and 8 update-months.
    let trad = traditional_lifetime_cost(4.0, 50_000.0, 2.0, 40_000.0);
    assert_close(trad, 520_000.0, 1e-6);
    let pccp = pccp_lifetime_cost(100_000.0, 4.0, 20_000.0);
    assert_close(pccp, 180_000.0, 1e-6);
    assert_close(pccp_saving(trad, pccp), 340_000.0, 1e-6);
    assert_close(benefit_months_gained(4.0, 2.0), 8.0, 1e-9);
}
