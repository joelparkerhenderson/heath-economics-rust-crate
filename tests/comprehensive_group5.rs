//! Comprehensive integration tests — group 5.
//!
//! Modules under test:
//! - health_technology_assessment
//! - incremental_cost_effectiveness_ratio
//! - inference_unit_economics
//! - length_of_stay
//! - life_years_gained
//! - marginal_vs_average_cost
//! - national_tariff_and_unit_costs
//! - net_monetary_benefit
//! - nice_evidence_standards_framework
//! - number_needed_to_treat
//!
//! Sections: 1. EDGE CASES, 2. PROPERTIES / INVARIANTS,
//! 3. CROSS-MODULE CONSISTENCY, 4. DOMAIN SCENARIOS.
//! All tests are deterministic; floats compared with an absolute tolerance.

use health_economics::health_technology_assessment as hta;
use health_economics::incremental_cost_effectiveness_ratio as icer_mod;
use health_economics::incremental_cost_effectiveness_ratio::CostEffectivenessQuadrant;
use health_economics::inference_unit_economics as infer;
use health_economics::inference_unit_economics::LlmCall;
use health_economics::length_of_stay as los;
use health_economics::life_years_gained as lyg;
use health_economics::marginal_vs_average_cost as mva;
use health_economics::national_tariff_and_unit_costs as tariff;
use health_economics::net_monetary_benefit as nmb_mod;
use health_economics::net_monetary_benefit::EvaluatedOption;
use health_economics::nice_evidence_standards_framework as esf;
use health_economics::nice_evidence_standards_framework::{ClinicalFunction, EsfTier};
use health_economics::number_needed_to_treat as nnt_mod;

const TOL: f64 = 1e-9;

// =========================================================================
// 1. EDGE CASES
// =========================================================================

/// Locks down: both icer() functions return None exactly when ΔE == 0, and only then.
#[test]
fn edge_icer_none_exactly_at_zero_delta_effect() {
    assert!(hta::icer(450.0, 0.0).is_none());
    assert!(icer_mod::icer(300_000.0, 0.0).is_none());
    assert!(hta::icer(0.0, 0.0).is_none()); // ΔC = 0 does not rescue a zero ΔE
    assert!(icer_mod::icer(-1.0, 0.0).is_none());
    // Tiny but nonzero ΔE is defined (only exact zero is the None condition).
    assert!(hta::icer(450.0, 1e-300).is_some());
    assert!(icer_mod::icer(450.0, -1e-300).is_some());
    // Zero ΔC with nonzero ΔE is a defined zero-cost ratio, not None.
    assert!((hta::icer(0.0, 0.5).unwrap() - 0.0).abs() < TOL);
}

/// Locks down: negative ΔC (cost-saving intervention) yields a negative ICER, not None.
#[test]
fn edge_icer_negative_delta_cost_is_cost_saving_negative_ratio() {
    let r = icer_mod::icer(-56.0, 8e-5).unwrap();
    assert!((r - -700_000.0).abs() < 1e-6);
    let r2 = hta::icer(-450.0, 0.03).unwrap();
    assert!((r2 - -15_000.0).abs() < TOL);
}

/// Locks down: extreme magnitudes (1e12) keep the ICER finite — no NaN/inf.
#[test]
fn edge_icer_extreme_magnitudes_stay_finite() {
    let big = icer_mod::icer(1e12, 1e-6).unwrap();
    assert!(big.is_finite());
    assert!((big - 1e18).abs() < 1e6); // relative error negligible
    let small = hta::icer(1e-12, 1e12).unwrap();
    assert!(small.is_finite());
    assert!((small - 1e-24).abs() < 1e-30);
}

/// Locks down: meets_threshold / adopt_at_threshold are strict — equality does not clear.
#[test]
fn edge_threshold_comparisons_are_strict_at_equality() {
    assert!(!hta::meets_threshold(20_000.0, 20_000.0));
    assert!(!icer_mod::adopt_at_threshold(20_000.0, 20_000.0));
    assert!(hta::meets_threshold(19_999.999, 20_000.0));
    assert!(icer_mod::adopt_at_threshold(-5.0, 0.0)); // negative ICER clears a zero threshold
}

/// Locks down: probability_cost_effective is None only for an empty draw set; 0 and 1 are reachable.
#[test]
fn edge_probability_cost_effective_empty_and_saturated() {
    assert!(hta::probability_cost_effective(&[], 20_000.0).is_none());
    let all_good = [(100.0, 0.03); 4];
    assert!((hta::probability_cost_effective(&all_good, 20_000.0).unwrap() - 1.0).abs() < TOL);
    let all_bad = [(900.0, 0.01); 4];
    assert!((hta::probability_cost_effective(&all_bad, 20_000.0).unwrap() - 0.0).abs() < TOL);
}

/// Locks down: a draw with NMB exactly zero counts as NOT cost-effective (strict > 0).
#[test]
fn edge_probability_cost_effective_boundary_draw_is_unfavorable() {
    // ΔE × λ − ΔC = 0.5 × 20,000 − 10,000 = 0 exactly → unfavorable.
    let draws = [(10_000.0, 0.5)];
    let p = hta::probability_cost_effective(&draws, 20_000.0).unwrap();
    assert!((p - 0.0).abs() < TOL);
}

/// Locks down: ReferenceCaseChecklist fails on any single false; recommend requires pass AND strict threshold.
#[test]
fn edge_checklist_single_failure_and_boundary_icer_reject() {
    let all_true = hta::ReferenceCaseChecklist {
        utilities_from_mandated_instrument: true,
        comparator_is_current_care_pathway: true,
        psa_reported: true,
    };
    assert!(all_true.passes());
    for i in 0..3 {
        let cl = hta::ReferenceCaseChecklist {
            utilities_from_mandated_instrument: i != 0,
            comparator_is_current_care_pathway: i != 1,
            psa_reported: i != 2,
        };
        assert!(!cl.passes());
        assert!(!hta::recommend_routine_commissioning(&cl, 1_000.0, 20_000.0));
    }
    // Conformant but ICER exactly at threshold → rejected (strict comparison).
    assert!(!hta::recommend_routine_commissioning(&all_true, 20_000.0, 20_000.0));
}

/// Locks down: classify_quadrant's ΔE = 0 axis is OnAxis for every sign of ΔC.
#[test]
fn edge_classify_quadrant_zero_effect_axis() {
    for dc in [-5.0, 0.0, 5.0, 1e12] {
        assert_eq!(icer_mod::classify_quadrant(dc, 0.0), CostEffectivenessQuadrant::OnAxis);
    }
}

/// Locks down: ΔC = 0 boundary — free-and-better is Dominant, free-and-worse is SavingsForLoss.
#[test]
fn edge_classify_quadrant_zero_cost_boundary() {
    assert_eq!(icer_mod::classify_quadrant(0.0, 1.0), CostEffectivenessQuadrant::Dominant);
    assert_eq!(icer_mod::classify_quadrant(0.0, -1.0), CostEffectivenessQuadrant::SavingsForLoss);
}

/// Locks down: net_incremental_cost goes negative when offsets exceed gross (dominance candidate).
#[test]
fn edge_net_incremental_cost_offsets_exceed_gross() {
    let dc = icer_mod::net_incremental_cost(500_000.0, 900_000.0);
    assert!((dc - -400_000.0).abs() < TOL);
    assert!((icer_mod::net_incremental_cost(0.0, 0.0) - 0.0).abs() < TOL);
}

/// Locks down: zero-token calls cost 0, empty call slice costs 0, and 1e12 tokens stay finite.
#[test]
fn edge_inference_zero_and_extreme_tokens() {
    let zero = LlmCall { input_tokens: 0.0, output_tokens: 0.0 };
    assert!((infer::cost_per_call(&zero, 3.0, 15.0) - 0.0).abs() < TOL);
    assert!((infer::cost_per_unit(&[], 3.0, 15.0) - 0.0).abs() < TOL);
    let huge = LlmCall { input_tokens: 1e12, output_tokens: 1e12 };
    let c = infer::cost_per_call(&huge, 3.0, 15.0);
    assert!(c.is_finite());
    assert!((c - 18e6).abs() < 1e-3); // 1e12 × (3+15)/1e6 = 18,000,000
}

/// Locks down: cost_share_of_value is None exactly at zero value; annual_cost at zero volume is 0.
#[test]
fn edge_inference_zero_value_and_zero_volume() {
    assert!(infer::cost_share_of_value(0.0765, 0.0).is_none());
    assert!(infer::cost_share_of_value(0.0, 31.25).is_some()); // zero cost is a defined 0 share
    assert!((infer::annual_cost(0.0765, 0.0) - 0.0).abs() < TOL);
    assert!((infer::annual_cost(0.0, 1e12) - 0.0).abs() < TOL);
}

/// Locks down: projected_cost at t = 0 is the base cost, and d = 1 holds prices flat forever.
#[test]
fn edge_projected_cost_zero_years_and_flat_ratio() {
    assert!((infer::projected_cost(1_000.0, 0.3, 0.0) - 1_000.0).abs() < TOL);
    assert!((infer::projected_cost(1_000.0, 1.0, 50.0) - 1_000.0).abs() < TOL);
    assert!(infer::projected_cost(1e12, 0.5, 30.0).is_finite());
}

/// Locks down: average_length_of_stay is None exactly at zero discharges.
#[test]
fn edge_average_los_none_at_zero_discharges() {
    assert!(los::average_length_of_stay(240.0, 0.0).is_none());
    assert!(los::average_length_of_stay(0.0, 40.0).is_some()); // zero bed days is a defined 0-day mean
    assert!((los::average_length_of_stay(0.0, 40.0).unwrap() - 0.0).abs() < TOL);
}

/// Locks down: mean/median are None only for empty sets; a single spell is its own mean and median.
#[test]
fn edge_mean_median_empty_and_singleton() {
    assert!(los::mean_length_of_stay(&[]).is_none());
    assert!(los::median_length_of_stay(&[]).is_none());
    assert!((los::mean_length_of_stay(&[7.5]).unwrap() - 7.5).abs() < TOL);
    assert!((los::median_length_of_stay(&[7.5]).unwrap() - 7.5).abs() < TOL);
    // Even count: midpoint of the two central order statistics, order-independent.
    assert!((los::median_length_of_stay(&[8.0, 2.0, 4.0, 6.0]).unwrap() - 5.0).abs() < TOL);
}

/// Locks down: same-day discharge is a 0-day spell; reversed day numbers go negative (not clamped).
#[test]
fn edge_length_of_stay_zero_and_negative() {
    assert!((los::length_of_stay_days(10.0, 10.0) - 0.0).abs() < TOL);
    assert!((los::length_of_stay_days(16.0, 10.0) - -6.0).abs() < TOL);
}

/// Locks down: beds_occupied at zero rate or zero LOS is 0; beds_freed is negative when LOS rises.
#[test]
fn edge_beds_occupied_zero_and_beds_freed_negative() {
    assert!((los::beds_occupied(0.0, 6.0) - 0.0).abs() < TOL);
    assert!((los::beds_occupied(40.0, 0.0) - 0.0).abs() < TOL);
    assert!((los::beds_freed(40.0, 5.6, 6.0) - -16.0).abs() < TOL);
    assert!((los::annual_bed_days_freed(0.0) - 0.0).abs() < TOL);
}

/// Locks down: area_between_survival_curves rejects mismatched lengths, empty, and single points.
#[test]
fn edge_area_between_curves_malformed_inputs() {
    assert!(lyg::area_between_survival_curves(&[], &[], &[]).is_none());
    assert!(lyg::area_between_survival_curves(&[0.0], &[1.0], &[1.0]).is_none());
    assert!(lyg::area_between_survival_curves(&[0.0, 1.0], &[1.0], &[1.0, 0.9]).is_none());
    assert!(lyg::area_between_survival_curves(&[0.0, 1.0], &[1.0, 0.9], &[1.0]).is_none());
    assert!(lyg::area_between_survival_curves(&[0.0, 1.0, 2.0], &[1.0, 0.9], &[1.0, 0.9]).is_none());
}

/// Locks down: crossing curves — the negative region after the cross subtracts from the area.
#[test]
fn edge_area_between_curves_crossing_regions_subtract() {
    // Gap is +0.2 at t=0, 0 at t=1, −0.2 at t=2: trapezoids +0.1 then −0.1 → net 0.
    let times = [0.0, 1.0, 2.0];
    let s_new = [0.9, 0.5, 0.1];
    let s_comp = [0.7, 0.5, 0.3];
    let area = lyg::area_between_survival_curves(&times, &s_new, &s_comp).unwrap();
    assert!((area - 0.0).abs() < TOL);
    // Asymmetric cross: +0.1 over [0,1] then −0.2 average… hand-computed net −0.05.
    let s_new2 = [0.8, 0.5, 0.1];
    let s_comp2 = [0.6, 0.5, 0.4];
    // Gaps: +0.2, 0.0, −0.3 → 0.5×(0.2+0) + 0.5×(0−0.3) = 0.1 − 0.15 = −0.05.
    let area2 = lyg::area_between_survival_curves(&times, &s_new2, &s_comp2).unwrap();
    assert!((area2 - -0.05).abs() < TOL);
}

/// Locks down: utility 0 zeroes the QALY view; monetary_value at zero threshold is 0 (not None).
#[test]
fn edge_lyg_zero_utility_and_zero_threshold() {
    assert!((lyg::qalys_from_life_extension(96.0, 0.0) - 0.0).abs() < TOL);
    assert!((lyg::evlyg_from_life_extension(0.0, lyg::EVLYG_FIXED_UTILITY) - 0.0).abs() < TOL);
    assert!((lyg::monetary_value(67.2, 0.0) - 0.0).abs() < TOL);
    // Harm direction: worse mean survival gives negative LYG.
    assert!((lyg::life_years_gained_from_mean_survival(3.0, 5.0) - -2.0).abs() < TOL);
    assert!((lyg::life_years_gained_from_deaths_prevented(0.0, 8.0) - 0.0).abs() < TOL);
}

/// Locks down: average_cost is None exactly at zero quantity; huge quantities stay finite.
#[test]
fn edge_average_cost_zero_quantity_and_extremes() {
    assert!(mva::average_cost(400_000.0, 0.0).is_none());
    let tiny = mva::average_cost(1.0, 1e12).unwrap();
    assert!(tiny.is_finite());
    assert!((tiny - 1e-12).abs() < 1e-18);
    assert!((mva::naive_average_cost_saving(0.0, 400.0) - 0.0).abs() < TOL);
    assert!((mva::marginal_saving(0.0, 120.0) - 0.0).abs() < TOL);
}

/// Locks down: crosses_capacity_step is inclusive (>=) at the exact step boundary.
#[test]
fn edge_crosses_capacity_step_boundary_inclusive() {
    let step = mva::ward_bed_days_per_year(20.0);
    assert!(mva::crosses_capacity_step(step, step)); // exactly at the step counts
    assert!(!mva::crosses_capacity_step(step - 1e-9, step));
    assert!((mva::ward_bed_days_per_year(0.0) - 0.0).abs() < TOL);
}

/// Locks down: step_change_saving is negative when total cost rises (no clamping).
#[test]
fn edge_step_change_saving_negative_when_cost_rises() {
    assert!((mva::step_change_saving(8_500_000.0, 10_000_000.0) - -1_500_000.0).abs() < TOL);
    assert!((mva::step_change_saving(1e12, 0.0) - 1e12).abs() < 1e-3);
}

/// Locks down: ncc_unit_cost and valuation_ratio are None exactly at zero denominators.
#[test]
fn edge_tariff_zero_denominators() {
    assert!(tariff::ncc_unit_cost(16_000_000.0, 0.0).is_none());
    assert!(tariff::valuation_ratio(80_000.0, 0.0).is_none());
    assert!(tariff::ncc_unit_cost(0.0, 100.0).is_some()); // zero cost is a defined £0 unit cost
    assert!(tariff::valuation_ratio(0.0, 7_750.0).is_some());
}

/// Locks down: blended_payment at zero activity is the fixed element alone; MFF 1.0 is identity.
#[test]
fn edge_tariff_degenerate_blend_and_identity_mff() {
    assert!((tariff::blended_payment(1_000_000.0, 160.0, 0.0) - 1_000_000.0).abs() < TOL);
    assert!((tariff::blended_payment(0.0, 160.0, 500.0) - 80_000.0).abs() < TOL);
    assert!((tariff::tariff_price(160.0, 1.0) - 160.0).abs() < TOL);
    assert!((tariff::staff_capacity_value(0.0, 250.0, 31.0) - 0.0).abs() < TOL);
    assert!((tariff::redeployed_activity_value(0.0, 250.0, 160.0) - 0.0).abs() < TOL);
}

/// Locks down: net_health_benefit is None exactly at λ = 0; NMB itself is total at any λ.
#[test]
fn edge_nhb_none_at_zero_lambda() {
    assert!(nmb_mod::net_health_benefit(30.0, 400_000.0, 0.0).is_none());
    assert!(nmb_mod::net_health_benefit(30.0, 400_000.0, -1.0).is_some()); // only exact 0 is undefined
    // NMB at λ = 0 collapses to −ΔC.
    assert!((nmb_mod::net_monetary_benefit(30.0, 400_000.0, 0.0) - -400_000.0).abs() < TOL);
}

/// Locks down: adopt is strict — NMB of exactly zero is indifference, not adoption.
#[test]
fn edge_adopt_zero_nmb_is_not_adoption() {
    assert!(!nmb_mod::adopt(0.0));
    assert!(nmb_mod::adopt(1e-300));
    assert!(!nmb_mod::adopt(-1e-300));
}

/// Locks down: best_option_index is None for empty input, Some(0) for a single option.
#[test]
fn edge_best_option_index_empty_and_singleton() {
    assert!(nmb_mod::best_option_index(&[], 20_000.0).is_none());
    let one = [EvaluatedOption { name: "only", delta_cost: 1e9, delta_effect: -5.0 }];
    assert_eq!(nmb_mod::best_option_index(&one, 20_000.0), Some(0)); // even a value-destroying option is argmax of itself
}

/// Locks down: years_to_recoup_evidence_cost is None exactly at zero incremental revenue.
#[test]
fn edge_years_to_recoup_none_at_zero_revenue() {
    assert!(esf::years_to_recoup_evidence_cost(600_000.0, 0.0).is_none());
    let instant = esf::years_to_recoup_evidence_cost(0.0, 200_000.0).unwrap();
    assert!((instant - 0.0).abs() < TOL); // zero evidence cost breaks even immediately
    assert!(esf::years_to_recoup_evidence_cost(1e12, 1.0).unwrap().is_finite());
}

/// Locks down: evidence_investment_range endpoints for every tier as coded.
#[test]
fn edge_evidence_ranges_exact_endpoints() {
    assert_eq!(esf::evidence_investment_range(EsfTier::A), (10_000.0, 50_000.0));
    assert_eq!(esf::evidence_investment_range(EsfTier::B), (50_000.0, 250_000.0));
    assert_eq!(esf::evidence_investment_range(EsfTier::C), (250_000.0, 2_000_000.0));
}

/// Locks down: NNT is None exactly at zero ARR; a negative ARR (harm) yields a negative NNT.
#[test]
fn edge_nnt_zero_and_negative_arr() {
    assert!(nnt_mod::number_needed_to_treat(0.0).is_none());
    // Treatment makes things worse: ARR = 0.024 − 0.032 = −0.008 → NNT = −125.
    let arr = nnt_mod::absolute_risk_reduction(0.024, 0.032);
    assert!((arr - -0.008).abs() < TOL);
    let nnt = nnt_mod::number_needed_to_treat(arr).unwrap();
    assert!((nnt - -125.0).abs() < 1e-6);
}

/// Locks down: RRR None at zero control rate; NNH None only at exactly equal harm rates.
#[test]
fn edge_rrr_and_nnh_none_conditions() {
    assert!(nnt_mod::relative_risk_reduction(0.0, 0.0).is_none());
    assert!(nnt_mod::relative_risk_reduction(0.0, 0.05).is_none()); // control = 0 undefined even with treatment events
    assert!(nnt_mod::number_needed_to_harm(0.02, 0.02).is_none());
    // Protective on the harm axis: negative NNH (fewer harms on treatment).
    let nnh = nnt_mod::number_needed_to_harm(0.01, 0.03).unwrap();
    assert!((nnh - -50.0).abs() < 1e-6);
}

/// Locks down: prevention_payoff_ratio None at zero prevention cost; extreme NNT stays finite.
#[test]
fn edge_prevention_payoff_and_extreme_nnt() {
    assert!(nnt_mod::prevention_payoff_ratio(12_000.0, 0.0).is_none());
    let arr = 1e-12;
    let nnt = nnt_mod::number_needed_to_treat(arr).unwrap();
    assert!(nnt.is_finite());
    let cost = nnt_mod::cost_per_event_prevented(nnt, 40.0);
    assert!(cost.is_finite());
    assert!((cost - 4e13).abs() < 1.0);
}

// =========================================================================
// 2. PROPERTIES / INVARIANTS
// =========================================================================

/// Locks down: ICER/NMB duality — adopt_at_threshold(ICER, λ) ⇔ NMB > 0 for all ΔC, ΔE > 0, λ on a grid.
#[test]
fn prop_icer_nmb_duality_on_grid() {
    let dcs = [100.0, 450.0, 10_000.0, 300_000.0];
    let des = [0.01, 0.5, 2.0, 25.0];
    let lambdas = [1_000.0, 20_000.0, 30_000.0, 100_000.0];
    for &dc in &dcs {
        for &de in &des {
            for &lambda in &lambdas {
                let ratio = icer_mod::icer(dc, de).unwrap();
                let nmb = nmb_mod::net_monetary_benefit(de, dc, lambda);
                assert_eq!(
                    icer_mod::adopt_at_threshold(ratio, lambda),
                    nmb_mod::adopt(nmb),
                    "duality broke at dc={dc} de={de} lambda={lambda}"
                );
            }
        }
    }
}

/// Locks down: at exact ICER == λ both rules reject (both strict) — no disagreement at equality.
#[test]
fn prop_icer_nmb_agree_at_exact_equality() {
    // 10,000 / 0.5 = 20,000 exactly in f64; NMB = 0.5 × 20,000 − 10,000 = 0 exactly.
    let ratio = icer_mod::icer(10_000.0, 0.5).unwrap();
    assert!((ratio - 20_000.0).abs() < TOL);
    let nmb = nmb_mod::net_monetary_benefit(0.5, 10_000.0, 20_000.0);
    assert!((nmb - 0.0).abs() < TOL);
    assert!(!icer_mod::adopt_at_threshold(ratio, 20_000.0));
    assert!(!nmb_mod::adopt(nmb));
}

/// Locks down: classify_quadrant maps all four strict sign combinations to the right variant.
#[test]
fn prop_classify_quadrant_four_sign_combinations() {
    for &(dc, de, want) in &[
        (5.0, 3.0, CostEffectivenessQuadrant::TradeOff),
        (-5.0, 3.0, CostEffectivenessQuadrant::Dominant),
        (5.0, -3.0, CostEffectivenessQuadrant::Dominated),
        (-5.0, -3.0, CostEffectivenessQuadrant::SavingsForLoss),
    ] {
        assert_eq!(icer_mod::classify_quadrant(dc, de), want);
        // Scale invariance: quadrant depends only on signs.
        assert_eq!(icer_mod::classify_quadrant(dc * 1e6, de * 1e-6), want);
    }
}

/// Locks down: NMB is linear in λ — the difference of NMBs at two thresholds is ΔE × Δλ.
#[test]
fn prop_nmb_linearity_in_lambda() {
    let cases = [(30.0, 400_000.0), (12.0, 150_000.0), (-2.0, -1_000.0), (0.0, 500.0)];
    let lambdas = [0.0, 10_000.0, 20_000.0, 30_000.0];
    for &(de, dc) in &cases {
        for &l1 in &lambdas {
            for &l2 in &lambdas {
                let lhs = nmb_mod::net_monetary_benefit(de, dc, l2)
                    - nmb_mod::net_monetary_benefit(de, dc, l1);
                assert!((lhs - de * (l2 - l1)).abs() < 1e-6);
            }
        }
    }
}

/// Locks down: best_option_index picks the argmax NMB and keeps the earliest option on ties.
#[test]
fn prop_best_option_index_is_argmax_and_tie_stable() {
    let options = [
        EvaluatedOption { name: "a", delta_cost: 400_000.0, delta_effect: 30.0 },
        EvaluatedOption { name: "b", delta_cost: 150_000.0, delta_effect: 12.0 },
        EvaluatedOption { name: "c", delta_cost: 700_000.0, delta_effect: 32.0 },
    ];
    for lambda in [5_000.0, 20_000.0, 50_000.0] {
        let winner = nmb_mod::best_option_index(&options, lambda).unwrap();
        let winner_nmb = nmb_mod::net_monetary_benefit(options[winner].delta_effect, options[winner].delta_cost, lambda);
        for opt in &options {
            assert!(nmb_mod::net_monetary_benefit(opt.delta_effect, opt.delta_cost, lambda) <= winner_nmb + TOL);
        }
    }
    // At λ = 5,000: a: −250k, b: −90k, c: −540k → b wins even though all are negative.
    assert_eq!(nmb_mod::best_option_index(&options, 5_000.0), Some(1));
    // Exact tie: identical options — the earliest index wins.
    let tied = [
        EvaluatedOption { name: "first", delta_cost: 100.0, delta_effect: 0.01 },
        EvaluatedOption { name: "second", delta_cost: 100.0, delta_effect: 0.01 },
    ];
    assert_eq!(nmb_mod::best_option_index(&tied, 20_000.0), Some(0));
}

/// Locks down: probability_cost_effective stays in [0,1] and equals the hand-counted fraction.
#[test]
fn prop_probability_cost_effective_matches_hand_count() {
    // Hand count at λ = 20,000 (NMB = ΔE×λ − ΔC, strict > 0):
    //   (100, 0.02)  → 400 − 100 = 300  > 0 ✓
    //   (500, 0.01)  → 200 − 500 = −300 ✗
    //   (300, 0.03)  → 600 − 300 = 300  > 0 ✓
    //   (−50, 0.0)   → 0 + 50 = 50      > 0 ✓  (cost-saving, zero effect)
    //   (200, 0.01)  → 200 − 200 = 0    ✗  (boundary is unfavorable)
    let draws = [(100.0, 0.02), (500.0, 0.01), (300.0, 0.03), (-50.0, 0.0), (200.0, 0.01)];
    let p = hta::probability_cost_effective(&draws, 20_000.0).unwrap();
    assert!((p - 0.6).abs() < TOL);
    for lambda in [0.0, 1_000.0, 20_000.0, 1e6] {
        let p = hta::probability_cost_effective(&draws, lambda).unwrap();
        assert!((0.0..=1.0).contains(&p), "p out of [0,1] at lambda={lambda}");
    }
}

/// Locks down: Little's Law — beds_occupied is linear in both rate and LOS on a grid.
#[test]
fn prop_beds_occupied_bilinear() {
    for rate in [1.0, 7.0, 40.0] {
        for l in [0.5, 4.0, 6.0] {
            assert!((los::beds_occupied(2.0 * rate, l) - 2.0 * los::beds_occupied(rate, l)).abs() < TOL);
            assert!((los::beds_occupied(rate, 3.0 * l) - 3.0 * los::beds_occupied(rate, l)).abs() < TOL);
            assert!((los::beds_occupied(rate, l) - rate * l).abs() < TOL);
        }
    }
}

/// Locks down: beds_freed equals the difference of the two Little's-Law occupancies; annual = beds × 365.
#[test]
fn prop_beds_freed_is_occupancy_difference_and_annualizes_at_365() {
    for rate in [10.0, 40.0] {
        for (before, after) in [(6.0, 5.6), (4.0, 4.0), (3.0, 5.0)] {
            let freed = los::beds_freed(rate, before, after);
            let diff = los::beds_occupied(rate, before) - los::beds_occupied(rate, after);
            assert!((freed - diff).abs() < TOL);
            assert!((los::annual_bed_days_freed(freed) - freed * 365.0).abs() < TOL);
        }
    }
}

/// Locks down: mean of identical spells equals the spell; median never exceeds the max spell.
#[test]
fn prop_mean_median_sanity_on_uniform_and_skewed_sets() {
    let uniform = [4.0; 6];
    assert!((los::mean_length_of_stay(&uniform).unwrap() - 4.0).abs() < TOL);
    assert!((los::median_length_of_stay(&uniform).unwrap() - 4.0).abs() < TOL);
    let skewed = [2.0, 3.0, 3.0, 4.0, 5.0, 6.0, 61.0];
    let mean = los::mean_length_of_stay(&skewed).unwrap();
    let median = los::median_length_of_stay(&skewed).unwrap();
    assert!(mean > median); // right-skew: long-stay tail drags the mean above the median
    assert!((mean - 12.0).abs() < TOL);
    assert!((median - 4.0).abs() < TOL);
}

/// Locks down: area between identical survival curves is exactly 0.
#[test]
fn prop_area_identical_curves_is_zero() {
    let times = [0.0, 1.0, 2.5, 4.0, 10.0];
    let s = [1.0, 0.8, 0.55, 0.3, 0.0];
    let area = lyg::area_between_survival_curves(&times, &s, &s).unwrap();
    assert!((area - 0.0).abs() < TOL);
}

/// Locks down: a constant gap g over horizon T integrates to exactly g × T.
#[test]
fn prop_area_constant_gap_is_gap_times_horizon() {
    for g in [0.05, 0.2, 0.5] {
        let times = [0.0, 2.0, 7.0, 10.0];
        let s_comp = [0.5, 0.4, 0.3, 0.2];
        let s_new: Vec<f64> = s_comp.iter().map(|v| v + g).collect();
        let area = lyg::area_between_survival_curves(&times, &s_new, &s_comp).unwrap();
        assert!((area - g * 10.0).abs() < TOL, "gap {g} broke");
    }
}

/// Locks down: adding a collinear midpoint does not change the trapezoid result.
#[test]
fn prop_area_invariant_under_collinear_midpoint() {
    let coarse = lyg::area_between_survival_curves(&[0.0, 10.0], &[1.0, 0.0], &[0.8, 0.0]).unwrap();
    // Midpoint at t = 5 on the straight lines: s_new = 0.5, s_comp = 0.4.
    let fine = lyg::area_between_survival_curves(&[0.0, 5.0, 10.0], &[1.0, 0.5, 0.0], &[0.8, 0.4, 0.0]).unwrap();
    assert!((coarse - fine).abs() < TOL);
    assert!((coarse - 1.0).abs() < TOL); // hand value: (5.0 − 4.0) mean survival difference
}

/// Locks down: area between curves equals the difference of the two mean-survival areas.
#[test]
fn prop_area_equals_mean_survival_difference() {
    let times = [0.0, 6.0, 10.0];
    let s_new = [1.0, 0.4, 0.0]; // mean survival 5.0 (triangle area over 10y)
    let s_comp = [1.0, 0.0, 0.0]; // mean survival 3.0 (triangle over 6y)
    let area = lyg::area_between_survival_curves(&times, &s_new, &s_comp).unwrap();
    let diff = lyg::life_years_gained_from_mean_survival(5.0, 3.0);
    assert!((area - diff).abs() < TOL);
}

/// Locks down: NNT × ARR == 1, and halving the ARR exactly doubles the NNT.
#[test]
fn prop_nnt_inverse_and_halving_doubles() {
    for arr in [0.5, 0.1, 0.008, 0.001, 1e-6] {
        let nnt = nnt_mod::number_needed_to_treat(arr).unwrap();
        assert!((nnt * arr - 1.0).abs() < TOL);
        let nnt_half = nnt_mod::number_needed_to_treat(arr / 2.0).unwrap();
        assert!((nnt_half - 2.0 * nnt).abs() < TOL * nnt.abs().max(1.0));
    }
}

/// Locks down: cost_per_event_prevented is exactly NNT × course cost on a grid.
#[test]
fn prop_cost_per_event_prevented_is_product() {
    for nnt in [10.0, 125.0, 400.0] {
        for course in [0.0, 40.0, 1_000.0] {
            assert!((nnt_mod::cost_per_event_prevented(nnt, course) - nnt * course).abs() < TOL);
        }
    }
}

/// Locks down: RRR × control rate == ARR (the two risk views describe the same trial).
#[test]
fn prop_rrr_times_baseline_is_arr() {
    for &(control, treat) in &[(0.032, 0.024), (0.04, 0.03), (0.5, 0.1), (0.1, 0.15)] {
        let arr = nnt_mod::absolute_risk_reduction(control, treat);
        let rrr = nnt_mod::relative_risk_reduction(control, treat).unwrap();
        assert!((rrr * control - arr).abs() < TOL);
    }
}

/// Locks down: naive average-cost saving ≥ marginal saving whenever AC ≥ MC (the bed-day trap).
#[test]
fn prop_naive_saving_dominates_marginal_when_fixed_costs_exist() {
    // Constructed cost structure: TC(q) = F + v·q with F = 280,000, v = 120 → AC(1,000) = 400.
    let fixed = 280_000.0;
    let variable = 120.0;
    let q = 1_000.0;
    let total = fixed + variable * q;
    let ac = mva::average_cost(total, q).unwrap();
    assert!((ac - 400.0).abs() < TOL);
    for freed in [1.0, 100.0, 1_000.0] {
        let naive = mva::naive_average_cost_saving(freed, ac);
        let marginal = mva::marginal_saving(freed, variable);
        assert!(naive >= marginal); // the naive claim always overstates while fixed costs continue
        assert!((naive - marginal - freed * (ac - variable)).abs() < 1e-6);
    }
}

/// Locks down: average_cost × quantity round-trips to total cost; step saving equals removed spend.
#[test]
fn prop_average_cost_roundtrip_and_step_change() {
    for &(tc, q) in &[(400_000.0, 1_000.0), (16_000_000.0, 100_000.0), (7.0, 3.0)] {
        let ac = mva::average_cost(tc, q).unwrap();
        assert!((ac * q - tc).abs() < 1e-6 * tc.abs().max(1.0));
    }
    // Closing a ward removes its whole cost block: step saving is exactly before − after.
    let before = 10_000_000.0;
    let after = 8_500_000.0;
    assert!((mva::step_change_saving(before, after) - 1_500_000.0).abs() < TOL);
    // Only the ward-sized freed volume unlocks step accounting.
    let step = mva::ward_bed_days_per_year(20.0);
    assert!(mva::crosses_capacity_step(7_300.0, step));
    assert!(!mva::crosses_capacity_step(5_840.0, step));
}

/// Locks down: EsfTier Ord is A < B < C, and classify_tier covers every ClinicalFunction variant.
#[test]
fn prop_esf_tier_ordering_and_total_classification() {
    assert!(EsfTier::A < EsfTier::B);
    assert!(EsfTier::B < EsfTier::C);
    assert!(EsfTier::A < EsfTier::C);
    assert_eq!(esf::classify_tier(ClinicalFunction::SystemService), EsfTier::A);
    assert_eq!(esf::classify_tier(ClinicalFunction::InformOrMonitor), EsfTier::B);
    assert_eq!(esf::classify_tier(ClinicalFunction::TreatDiagnoseOrGuide), EsfTier::C);
}

/// Locks down: evidence ranges escalate with tier and are contiguous (each high == next low).
#[test]
fn prop_esf_evidence_ranges_escalate_and_touch() {
    let (a_lo, a_hi) = esf::evidence_investment_range(EsfTier::A);
    let (b_lo, b_hi) = esf::evidence_investment_range(EsfTier::B);
    let (c_lo, c_hi) = esf::evidence_investment_range(EsfTier::C);
    assert!(a_lo < a_hi && b_lo < b_hi && c_lo < c_hi);
    assert!((a_hi - b_lo).abs() < TOL); // 50k boundary shared
    assert!((b_hi - c_lo).abs() < TOL); // 250k boundary shared
    assert!(c_lo / b_lo >= 5.0); // "moving a tier can 10× the bill" — at least 5× at the low end
}

/// Locks down: cost_per_unit is exactly the sum of the constituent cost_per_call values.
#[test]
fn prop_cost_per_unit_is_sum_of_calls() {
    let calls = [
        LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 },
        LlmCall { input_tokens: 6_000.0, output_tokens: 300.0 },
        LlmCall { input_tokens: 0.0, output_tokens: 50.0 },
    ];
    for &(inr, outr) in &[(3.0, 15.0), (0.4, 1.6), (75.0, 150.0)] {
        let sum: f64 = calls.iter().map(|c| infer::cost_per_call(c, inr, outr)).sum();
        assert!((infer::cost_per_unit(&calls, inr, outr) - sum).abs() < TOL);
    }
}

/// Locks down: projected_cost is identity at d = 1, strictly decreasing in t for d < 1, and compounds.
#[test]
fn prop_projected_cost_identity_monotone_and_compounding() {
    for t in [0.0, 1.0, 2.5, 10.0] {
        assert!((infer::projected_cost(7_650.0, 1.0, t) - 7_650.0).abs() < TOL);
    }
    for d in [0.3, 0.5, 0.7] {
        let mut prev = infer::projected_cost(1_000.0, d, 0.0);
        for t in [1.0, 2.0, 3.0, 5.0] {
            let now = infer::projected_cost(1_000.0, d, t);
            assert!(now < prev, "not decreasing at d={d}, t={t}");
            prev = now;
        }
        // Compounding: projecting 3 years equals projecting 1 then 2 more.
        let direct = infer::projected_cost(1_000.0, d, 3.0);
        let staged = infer::projected_cost(infer::projected_cost(1_000.0, d, 1.0), d, 2.0);
        assert!((direct - staged).abs() < 1e-9);
    }
}

// =========================================================================
// 3. CROSS-MODULE CONSISTENCY
// =========================================================================

/// Locks down: hta::icer and incremental_cost_effectiveness_ratio::icer agree on values and None conditions.
#[test]
fn cross_icer_implementations_agree() {
    let cases = [(450.0, 0.03), (300_000.0, 25.0), (-56.0, 8e-5), (900_000.0, 25.0), (0.0, 1.0), (5.0, -2.0)];
    for &(dc, de) in &cases {
        let a = hta::icer(dc, de).unwrap();
        let b = icer_mod::icer(dc, de).unwrap();
        assert!((a - b).abs() < TOL, "icer mismatch at dc={dc} de={de}");
    }
    assert_eq!(hta::icer(450.0, 0.0).is_none(), icer_mod::icer(450.0, 0.0).is_none());
}

/// Locks down: hta::meets_threshold and icer_mod::adopt_at_threshold implement the same strict rule.
#[test]
fn cross_threshold_rules_agree_including_equality() {
    for icer_value in [11_999.9, 12_000.0, 20_000.0, 20_000.000001, 36_000.0, -700_000.0] {
        for threshold in [12_000.0, 20_000.0, 30_000.0] {
            assert_eq!(
                hta::meets_threshold(icer_value, threshold),
                icer_mod::adopt_at_threshold(icer_value, threshold),
                "rules diverge at icer={icer_value} thr={threshold}"
            );
        }
    }
}

/// Locks down: a single PSA draw's probability is 1.0 or 0.0 exactly as adopt(NMB) decides.
#[test]
fn cross_single_draw_probability_matches_nmb_adopt() {
    let draws = [
        (450.0, 0.03),   // NMB = 150 > 0
        (450.0, 0.01),   // NMB = −250 < 0
        (10_000.0, 0.5), // NMB = 0 boundary
        (-50.0, 0.0),    // cost-saving with zero effect: NMB = 50 > 0
    ];
    for &(dc, de) in &draws {
        let p = hta::probability_cost_effective(&[(dc, de)], 20_000.0).unwrap();
        let adopt = nmb_mod::adopt(nmb_mod::net_monetary_benefit(de, dc, 20_000.0));
        assert!((p - if adopt { 1.0 } else { 0.0 }).abs() < TOL, "mismatch at dc={dc} de={de}");
    }
}

/// Locks down: in the TradeOff quadrant, NMB adoption, ICER threshold, and HTA recommendation align.
#[test]
fn cross_quadrant_icer_nmb_and_recommendation_align() {
    let checklist = hta::ReferenceCaseChecklist {
        utilities_from_mandated_instrument: true,
        comparator_is_current_care_pathway: true,
        psa_reported: true,
    };
    for &(dc, de) in &[(300_000.0, 25.0), (700_000.0, 32.0), (150_000.0, 12.0)] {
        assert_eq!(icer_mod::classify_quadrant(dc, de), CostEffectivenessQuadrant::TradeOff);
        let ratio = icer_mod::icer(dc, de).unwrap();
        let adopt = nmb_mod::adopt(nmb_mod::net_monetary_benefit(de, dc, 20_000.0));
        assert_eq!(adopt, icer_mod::adopt_at_threshold(ratio, 20_000.0));
        assert_eq!(adopt, hta::recommend_routine_commissioning(&checklist, ratio, 20_000.0));
    }
}

/// Locks down: LOS and marginal-cost modules share the ×365 annualization constant.
#[test]
fn cross_annualization_constants_agree() {
    // 16 beds freed for a year and a 16-bed ward's annual bed days are the same quantity.
    assert!((los::annual_bed_days_freed(16.0) - mva::ward_bed_days_per_year(16.0)).abs() < TOL);
    assert!((los::annual_bed_days_freed(1.0) - 365.0).abs() < TOL);
}

/// Locks down: NHB × λ == NMB (the two decision forms are the same rule in different units).
#[test]
fn cross_nhb_scales_to_nmb() {
    for &(de, dc) in &[(30.0, 400_000.0), (12.0, 150_000.0), (32.0, 700_000.0), (-1.0, -30_000.0)] {
        for lambda in [10_000.0, 20_000.0, 30_000.0] {
            let nhb = nmb_mod::net_health_benefit(de, dc, lambda).unwrap();
            let nmb = nmb_mod::net_monetary_benefit(de, dc, lambda);
            assert!((nhb * lambda - nmb).abs() < 1e-6);
        }
    }
}

// =========================================================================
// 4. DOMAIN SCENARIOS
// =========================================================================

/// Locks down: full HTA submission chain — net ΔC → ICER → threshold → NMB/NHB → PSA → recommendation.
#[test]
fn scenario_hta_submission_end_to_end() {
    // Digital therapeutic: gross £700/patient, £250 of displaced spend → ΔC = £450; ΔE = 0.03 QALYs.
    let dc = icer_mod::net_incremental_cost(700.0, 250.0);
    assert!((dc - 450.0).abs() < TOL);
    assert_eq!(icer_mod::classify_quadrant(dc, 0.03), CostEffectivenessQuadrant::TradeOff);

    // ICER = 450 / 0.03 = £15,000/QALY; both modules agree and it clears £20k.
    let ratio = hta::icer(dc, 0.03).unwrap();
    assert!((ratio - 15_000.0).abs() < TOL);
    assert!((ratio - icer_mod::icer(dc, 0.03).unwrap()).abs() < TOL);
    assert!(hta::meets_threshold(ratio, 20_000.0));

    // NMB = 0.03 × 20,000 − 450 = £150/patient; NHB = 0.03 − 0.0225 = 0.0075 QALYs.
    let nmb = nmb_mod::net_monetary_benefit(0.03, dc, 20_000.0);
    assert!((nmb - 150.0).abs() < TOL);
    assert!(nmb_mod::adopt(nmb));
    let nhb = nmb_mod::net_health_benefit(0.03, dc, 20_000.0).unwrap();
    assert!((nhb - 0.0075).abs() < TOL);

    // PSA: 71 favorable + 29 unfavorable draws → 71% cost-effective at £20k.
    let mut draws = vec![(450.0, 0.03); 71];
    draws.extend(vec![(450.0, 0.01); 29]);
    let p = hta::probability_cost_effective(&draws, 20_000.0).unwrap();
    assert!((p - 0.71).abs() < TOL);

    // Conformant reference case + ICER under threshold → routine commissioning.
    let checklist = hta::ReferenceCaseChecklist {
        utilities_from_mandated_instrument: true,
        comparator_is_current_care_pathway: true,
        psa_reported: true,
    };
    assert!(hta::recommend_routine_commissioning(&checklist, ratio, 20_000.0));
    // The identical numbers fail at appraisal if the comparator was a strawman.
    let strawman = hta::ReferenceCaseChecklist {
        utilities_from_mandated_instrument: true,
        comparator_is_current_care_pathway: false,
        psa_reported: true,
    };
    assert!(!hta::recommend_routine_commissioning(&strawman, ratio, 20_000.0));
}

/// Locks down: ward-flow business case — LOS cut → beds freed → honest marginal vs naive vs tariff claims.
#[test]
fn scenario_ward_flow_business_case() {
    // Discharge software cuts mean LOS 6.0 → 5.6 days at 40 admissions/day.
    let spells_after = [5.0, 5.2, 5.6, 5.8, 6.4]; // mean 5.6, hand-computed
    assert!((los::mean_length_of_stay(&spells_after).unwrap() - 5.6).abs() < TOL);
    assert!((los::median_length_of_stay(&spells_after).unwrap() - 5.6).abs() < TOL);
    assert!((los::length_of_stay_days(100.0, 105.6) - 5.6).abs() < TOL);
    assert!((los::average_length_of_stay(224.0, 40.0).unwrap() - 5.6).abs() < TOL);

    let freed = los::beds_freed(40.0, 6.0, 5.6);
    assert!((freed - 16.0).abs() < TOL);
    let bed_days = los::annual_bed_days_freed(freed);
    assert!((bed_days - 5_840.0).abs() < TOL);

    // Naive claim: 5,840 × £400 = £2,336,000. Marginal truth: 5,840 × £120 = £700,800.
    let ac = mva::average_cost(400_000.0, 1_000.0).unwrap();
    let naive = mva::naive_average_cost_saving(bed_days, ac);
    assert!((naive - 2_336_000.0).abs() < TOL);
    let marginal = mva::marginal_saving(bed_days, 120.0);
    assert!((marginal - 700_800.0).abs() < TOL);
    assert!(naive > marginal);

    // 5,840 bed days do not reach the 20-bed-ward step (7,300), so no step-change claim.
    let step = mva::ward_bed_days_per_year(20.0);
    assert!(!mva::crosses_capacity_step(bed_days, step));
    assert!((mva::step_change_saving(10_000_000.0, 8_500_000.0) - 1_500_000.0).abs() < TOL);

    // Tariff side: nurse hour freed is £7,750/year capacity; redeployed clinics £80,000/year — ~10.3×.
    let unit_cost = tariff::ncc_unit_cost(16_000_000.0, 100_000.0).unwrap();
    assert!((unit_cost - 160.0).abs() < TOL);
    let local_price = tariff::tariff_price(unit_cost, 1.15);
    assert!((local_price - 184.0).abs() < TOL);
    let capacity = tariff::staff_capacity_value(1.0, 250.0, 31.0);
    assert!((capacity - 7_750.0).abs() < TOL);
    let funded = tariff::redeployed_activity_value(2.0, 250.0, 160.0);
    assert!((funded - 80_000.0).abs() < TOL);
    let ratio = tariff::valuation_ratio(funded, capacity).unwrap();
    assert!((ratio - 80_000.0 / 7_750.0).abs() < TOL);
    // Commissioner pays a blended contract: £1M fixed + 500 extra attendances × £160.
    assert!((tariff::blended_payment(1_000_000.0, 160.0, 500.0) - 1_080_000.0).abs() < TOL);
}

/// Locks down: LLM feature unit economics — tokens → per-call → per-unit → annual → value share → ESF pricing.
#[test]
fn scenario_llm_feature_unit_economics_and_esf() {
    // Discharge-summary drafts: 12k in / 1.2k out plus a 6k/300 verify pass at $3/M in, $15/M out.
    let draft = LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 };
    let verify = LlmCall { input_tokens: 6_000.0, output_tokens: 300.0 };
    assert!((infer::cost_per_call(&draft, 3.0, 15.0) - 0.054).abs() < TOL);
    assert!((infer::cost_per_call(&verify, 3.0, 15.0) - 0.0225).abs() < TOL);
    let per_unit = infer::cost_per_unit(&[draft, verify], 3.0, 15.0);
    assert!((per_unit - 0.0765).abs() < TOL);

    // 100,000 summaries/year → $7,650; ~0.245% of the $31.25 clinician-time value per summary.
    let annual = infer::annual_cost(per_unit, 100_000.0);
    assert!((annual - 7_650.0).abs() < TOL);
    let share = infer::cost_share_of_value(per_unit, 31.25).unwrap();
    assert!((share - 0.002448).abs() < 1e-6);

    // Price-decline scenario d = 0.5: year-2 annual cost is a quarter — $1,912.50.
    let annual_y2 = infer::projected_cost(annual, 0.5, 2.0);
    assert!((annual_y2 - 1_912.5).abs() < TOL);

    // ESF pricing of the claim: summarization/drafting that informs clinicians is Tier B;
    // auto-adjusting treatment would be Tier C at up to 10× the evidence bill.
    let tier = esf::classify_tier(ClinicalFunction::InformOrMonitor);
    assert_eq!(tier, EsfTier::B);
    let (b_lo, b_hi) = esf::evidence_investment_range(tier);
    let study_cost = 150_000.0;
    assert!(b_lo <= study_cost && study_cost <= b_hi);
    assert!(esf::classify_tier(ClinicalFunction::TreatDiagnoseOrGuide) > tier);
    // £150k study against £75k/year incremental revenue → 2 years to recoup.
    let years = esf::years_to_recoup_evidence_cost(study_cost, 75_000.0).unwrap();
    assert!((years - 2.0).abs() < TOL);
}

/// Locks down: falls-prevention chain — NNT economics feed a per-patient NMB and quadrant call.
#[test]
fn scenario_falls_prevention_nnt_to_nmb() {
    // Trial: injurious falls 3.2% → 2.4%; bundle costs £40/patient; a fall costs £12,000.
    let arr = nnt_mod::absolute_risk_reduction(0.032, 0.024);
    assert!((arr - 0.008).abs() < TOL);
    let rrr = nnt_mod::relative_risk_reduction(0.032, 0.024).unwrap();
    assert!((rrr - 0.25).abs() < TOL); // the "reduces falls 25%" marketing view of the same data
    let nnt = nnt_mod::number_needed_to_treat(arr).unwrap();
    assert!((nnt - 125.0).abs() < TOL);
    let cost_per_prevented = nnt_mod::cost_per_event_prevented(nnt, 40.0);
    assert!((cost_per_prevented - 5_000.0).abs() < TOL);
    let payoff = nnt_mod::prevention_payoff_ratio(12_000.0, cost_per_prevented).unwrap();
    assert!((payoff - 2.4).abs() < TOL);
    // Harm side: alarm-fatigue pressure injuries 0.5% vs 0.3% → NNH = 500 (>> NNT, acceptable).
    let nnh = nnt_mod::number_needed_to_harm(0.005, 0.003).unwrap();
    assert!((nnh - 500.0).abs() < 1e-6);

    // Per-patient economics: ΔC = £40 − 0.008 × £12,000 = −£56 (cost-saving);
    // ΔE = 0.008 falls avoided × 0.01 QALY per fall = 8e-5 QALYs.
    let dc = icer_mod::net_incremental_cost(40.0, arr * 12_000.0);
    assert!((dc - -56.0).abs() < TOL);
    let de = arr * 0.01;
    assert_eq!(icer_mod::classify_quadrant(dc, de), CostEffectivenessQuadrant::Dominant);
    // Dominant: NMB positive at any non-negative λ — even λ = 0 keeps the £56 saving.
    let nmb0 = nmb_mod::net_monetary_benefit(de, dc, 0.0);
    assert!((nmb0 - 56.0).abs() < TOL);
    assert!(nmb_mod::adopt(nmb0));
    let nmb20k = nmb_mod::net_monetary_benefit(de, dc, 20_000.0);
    assert!((nmb20k - 57.6).abs() < TOL); // 8e-5 × 20,000 + 56 = 1.6 + 56
    assert!(nmb_mod::adopt(nmb20k));

    // Survival framing: 0.4 deaths/year prevented × 8 remaining years = 3.2 LYG;
    // QALY view 3.2 × 0.7 = 2.24 vs evLYG view 3.2 × 0.851 = 2.7232; both priced at £20k.
    let lyg_total = lyg::life_years_gained_from_deaths_prevented(0.4, 8.0);
    assert!((lyg_total - 3.2).abs() < TOL);
    let qalys = lyg::qalys_from_life_extension(lyg_total, 0.7);
    assert!((qalys - 2.24).abs() < TOL);
    let evlyg = lyg::evlyg_from_life_extension(lyg_total, lyg::EVLYG_FIXED_UTILITY);
    assert!((evlyg - 2.7232).abs() < TOL);
    assert!((lyg::monetary_value(qalys, 20_000.0) - 44_800.0).abs() < TOL);
    assert!((lyg::monetary_value(evlyg, 20_000.0) - 54_464.0).abs() < TOL);

    // Ranking the bundle against a cheaper leaflet-only option: the bundle's NMB wins at £20k.
    let options = [
        EvaluatedOption { name: "sensor bundle", delta_cost: -56.0, delta_effect: 8e-5 },
        EvaluatedOption { name: "leaflets only", delta_cost: -1.0, delta_effect: 0.0 },
    ];
    assert_eq!(nmb_mod::best_option_index(&options, 20_000.0), Some(0));
}
