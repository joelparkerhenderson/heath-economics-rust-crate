//! Comprehensive integration tests, group 7.
//!
//! Modules under test:
//! - remote_patient_monitoring_economics
//! - retention_and_churn
//! - return_on_investment
//! - screening_economics
//! - sensitivity_analysis
//! - social_return_on_investment
//! - space_and_devex
//! - technical_debt
//! - time_horizon
//! - total_cost_of_ownership
//!
//! Sections: 1. EDGE CASES, 2. PROPERTIES / INVARIANTS,
//! 3. CROSS-MODULE CONSISTENCY, 4. DOMAIN SCENARIOS.

use health_economics::cost_benefit_analysis;
use health_economics::remote_patient_monitoring_economics::{
    annual_margin, annual_revenue, compliance_lever_annual_gain, monthly_revenue,
    nhs_style_net_value, revenue_per_member_per_month, virtual_ward_gross_annual_value,
    CPT_99453_SETUP, CPT_99454_DEVICE_SUPPLY, CPT_99457_FIRST_20_MIN,
    CPT_99458_ADDITIONAL_20_MIN,
};
use health_economics::retention_and_churn::{
    churn_rate_percent, completers, cost_per_retained_user, expected_benefit_per_acquired_user,
    health_value_per_download, monetized_health_value, qalys_delivered, retention_improvement_value,
    retention_percent,
};
use health_economics::return_on_investment::{
    economic_roi, roi, strict_financial_roi, total_benefits, BenefitClass, BenefitLine,
};
use health_economics::return_on_investment;
use health_economics::screening_economics::{
    cost_per_true_case, false_positives, net_value_per_case_found, positive_predictive_value,
    total_programme_cost, true_positives,
};
use health_economics::sensitivity_analysis::{
    rank_by_swing, CodingAssistantCase, OneWayResult,
};
use health_economics::social_return_on_investment::{
    proxy_valued_share, sroi_ratio, total_outcome_value, value_after_drop_off, SocialOutcome,
};
use health_economics::space_and_devex::{
    capacity_value_per_year, space_rule_satisfied, time_reclaimed_minutes_per_day,
    vendor_index_minutes_per_week, MetricSource, SpaceDimension, SpaceMetric,
};
use health_economics::technical_debt::{
    annual_interest, interest_avoided_per_year, paydown_net_value, pv_of_interest_avoided,
    sqale_grade, sqale_principal, technical_debt_ratio_percent, SqaleGrade,
};
use health_economics::technical_debt;
use health_economics::time_horizon::{
    break_even_horizon_years, net_benefit_at_horizon, net_benefit_by_horizons, net_present_value,
};
use health_economics::total_cost_of_ownership::{
    annual_maintenance_benchmark, tco_advantage, TcoProfile,
};

/// Absolute-with-relative-fallback float comparison for grid tests where
/// magnitudes vary.
fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol * a.abs().max(b.abs()).max(1.0)
}

fn metric(dimension: SpaceDimension, source: MetricSource) -> SpaceMetric {
    SpaceMetric { dimension, source }
}

// =====================================================================
// SECTION 1: EDGE CASES
// =====================================================================

// Locks down: roi and both class-filtered ROIs return None exactly when costs == 0.
#[test]
fn edge_roi_family_none_when_costs_zero() {
    assert!(roi(1_000.0, 0.0).is_none());
    assert!(roi(0.0, 0.0).is_none());
    assert!(roi(-500.0, 0.0).is_none());
    let lines = [BenefitLine { class: BenefitClass::CashReleasing, amount: 100.0 }];
    assert!(strict_financial_roi(&lines, 0.0).is_none());
    assert!(strict_financial_roi(&[], 0.0).is_none());
    assert!(economic_roi(&lines, 0.0).is_none());
    assert!(economic_roi(&[], 0.0).is_none());
    // Non-zero costs always yield Some, even for negative benefits.
    assert!(roi(-1.0, 1.0).is_some());
}

// Locks down: both payback functions return None exactly when the annual saving is 0.
#[test]
fn edge_payback_none_when_annual_net_zero() {
    assert!(return_on_investment::payback_period_years(500_000.0, 0.0).is_none());
    assert!(technical_debt::payback_period_years(85_500.0, 0.0).is_none());
    // Negative annual net is Some (a negative, meaningless period), as documented.
    let p = return_on_investment::payback_period_years(100.0, -50.0).unwrap();
    assert!((p - (-2.0)).abs() < 1e-9);
    let q = technical_debt::payback_period_years(100.0, -50.0).unwrap();
    assert!((q - (-2.0)).abs() < 1e-9);
}

// Locks down: cost_per_retained_user (the CAC/retention "LTV-style" divide) is None at retention 0.
#[test]
fn edge_cost_per_retained_user_none_at_zero_retention() {
    assert!(cost_per_retained_user(5.0, 0.0).is_none());
    assert!(cost_per_retained_user(0.0, 0.0).is_none());
    // Any nonzero retention is Some.
    assert!(cost_per_retained_user(5.0, 1e-9).is_some());
}

// Locks down: retention/churn/per-download ratios are None exactly on zero denominators.
#[test]
fn edge_retention_ratios_none_on_zero_denominators() {
    assert!(retention_percent(10.0, 0.0).is_none());
    assert!(churn_rate_percent(10.0, 0.0).is_none());
    assert!(health_value_per_download(1_000.0, 0.0).is_none());
    assert!(retention_percent(0.0, 1.0).is_some());
    assert!(churn_rate_percent(0.0, 1.0).is_some());
    assert!(health_value_per_download(0.0, 1.0).is_some());
}

// Locks down: cost_per_true_case is None with zero true positives, Some otherwise.
#[test]
fn edge_cost_per_true_case_none_with_zero_tp() {
    assert!(cost_per_true_case(3_670_000.0, 0.0).is_none());
    assert!((cost_per_true_case(1_000.0, 4.0).unwrap() - 250.0).abs() < 1e-9);
}

// Locks down: PPV is None only when the test can never return a positive (denominator 0).
#[test]
fn edge_ppv_none_when_no_positive_possible() {
    // sens 0 and spec 1: neither true nor false positives are possible.
    assert!(positive_predictive_value(0.0, 1.0, 0.0).is_none());
    assert!(positive_predictive_value(0.0, 1.0, 1.0).is_none());
    // Prevalence 0 with imperfect specificity: PPV is Some(0.0) — all positives false.
    let ppv = positive_predictive_value(0.9, 0.95, 0.0).unwrap();
    assert!(ppv.abs() < 1e-12);
    // Prevalence 1 with any sensitivity > 0: every positive is real, PPV = 1.
    let ppv1 = positive_predictive_value(0.6, 0.5, 1.0).unwrap();
    assert!((ppv1 - 1.0).abs() < 1e-12);
    // Perfect specificity: no false positives, so PPV = 1 whenever sens × prev > 0.
    let ppv2 = positive_predictive_value(0.7, 1.0, 0.001).unwrap();
    assert!((ppv2 - 1.0).abs() < 1e-12);
}

// Locks down: break_even_horizon_years is None exactly when benefit == running cost.
#[test]
fn edge_break_even_none_when_annual_net_zero() {
    assert!(break_even_horizon_years(2_000_000.0, 400_000.0, 400_000.0).is_none());
    assert!(break_even_horizon_years(0.0, 100.0, 100.0).is_none());
    // Negative net is Some (negative horizon — never pays back), as documented.
    let t = break_even_horizon_years(100.0, 10.0, 60.0).unwrap();
    assert!((t - (-2.0)).abs() < 1e-9);
}

// Locks down: technical_debt_ratio_percent is None with zero redevelopment cost.
#[test]
fn edge_tdr_none_with_zero_redevelopment() {
    assert!(technical_debt_ratio_percent(285_000.0, 0.0).is_none());
    assert!(technical_debt_ratio_percent(0.0, 0.0).is_none());
    // Zero remediation on nonzero rebuild is a defined 0% (grade A).
    let tdr = technical_debt_ratio_percent(0.0, 1_000_000.0).unwrap();
    assert!(tdr.abs() < 1e-12);
    assert_eq!(sqale_grade(tdr), SqaleGrade::A);
}

// Locks down: sroi_ratio None at zero investment; proxy_valued_share None at zero total.
#[test]
fn edge_sroi_ratios_none_on_zero_denominators() {
    assert!(sroi_ratio(1_000.0, 0.0).is_none());
    assert!(sroi_ratio(0.0, 0.0).is_none());
    assert!(proxy_valued_share(0.0, 0.0).is_none());
    // All-proxy total: share is exactly 1.0.
    assert!((proxy_valued_share(500.0, 0.0).unwrap() - 1.0).abs() < 1e-12);
    // All-cash total: share is exactly 0.0.
    assert!(proxy_valued_share(0.0, 500.0).unwrap().abs() < 1e-12);
}

// Locks down: threshold_hours_saved_per_day is None when any benefit-rate factor is zero.
#[test]
fn edge_threshold_none_when_benefit_rate_is_zero() {
    let base = CodingAssistantCase {
        developers: 200.0,
        hours_saved_per_day: 0.5,
        loaded_cost_per_hour: 60.0,
        working_days_per_year: 220.0,
        license_per_dev_per_month: 39.0,
    };
    for (devs, days, rate) in [(0.0, 220.0, 60.0), (200.0, 0.0, 60.0), (200.0, 220.0, 0.0)] {
        let mut c = base;
        c.developers = devs;
        c.working_days_per_year = days;
        c.loaded_cost_per_hour = rate;
        assert!(c.threshold_hours_saved_per_day().is_none());
    }
    assert!(base.threshold_hours_saved_per_day().is_some());
}

// Locks down: initial_cost_share is None only for an all-zero (zero-TCO) profile.
#[test]
fn edge_initial_cost_share_none_for_zero_tco() {
    let zero = TcoProfile {
        initial_cost: 0.0,
        integration_and_training: 0.0,
        annual_run_cost: 0.0,
        horizon_years: 5,
        decommission_cost: 0.0,
    };
    assert!(zero.initial_cost_share().is_none());
    assert!(zero.undiscounted_tco().abs() < 1e-12);
    // A profile with only post-launch cost has a defined share of exactly 0.
    let run_only = TcoProfile { annual_run_cost: 10_000.0, ..zero };
    assert!(run_only.initial_cost_share().unwrap().abs() < 1e-12);
}

// Locks down: empty-slice behavior — all aggregations return 0 / empty / false, never panic.
#[test]
fn edge_empty_slices_are_neutral() {
    // Empty benefit-line list sums to zero, and its strict ROI is a defined −100%.
    assert!(total_benefits(&[], &[BenefitClass::CashReleasing]).abs() < 1e-12);
    assert!((strict_financial_roi(&[], 500.0).unwrap() - (-1.0)).abs() < 1e-12);
    assert!((economic_roi(&[], 500.0).unwrap() - (-1.0)).abs() < 1e-12);
    // Empty include-list also sums to zero even with lines present.
    let lines = [BenefitLine { class: BenefitClass::Capacity, amount: 7.0 }];
    assert!(total_benefits(&lines, &[]).abs() < 1e-12);
    // Empty SROI outcome list.
    assert!(total_outcome_value(&[]).abs() < 1e-12);
    // Empty SPACE metric list fails the rule (0 dimensions, no sources).
    assert!(!space_rule_satisfied(&[]));
    // Empty retention/benefit slices contribute nothing.
    assert!(expected_benefit_per_acquired_user(&[], &[]).abs() < 1e-12);
    assert!(expected_benefit_per_acquired_user(&[], &[4.0, 4.0]).abs() < 1e-12);
    assert!(expected_benefit_per_acquired_user(&[1.0, 1.0], &[]).abs() < 1e-12);
    // Empty flow vectors: NPV/PV are 0.
    assert!(net_present_value(&[], 0.035).abs() < 1e-12);
    assert!(cost_benefit_analysis::present_value(&[], 0.035).abs() < 1e-12);
    // Empty horizon list yields an empty report.
    assert!(net_benefit_by_horizons(1.0, 2.0, 1.0, &[]).is_empty());
    // Empty swing list sorts without panicking.
    let mut empty: [(&str, f64); 0] = [];
    rank_by_swing(&mut empty);
    assert!(empty.is_empty());
}

// Locks down: SocialOutcome adjustment-factor boundaries at 0 and 1.
#[test]
fn edge_social_outcome_factor_boundaries() {
    let base = SocialOutcome {
        quantity: 100.0,
        financial_proxy: 50.0,
        attribution: 1.0,
        deadweight: 0.0,
        displacement: 0.0,
    };
    // No adjustments: full gross claim.
    assert!((base.value() - 5_000.0).abs() < 1e-9);
    // deadweight 1: everything would have happened anyway → value 0.
    assert!(SocialOutcome { deadweight: 1.0, ..base }.value().abs() < 1e-12);
    // attribution 0: nothing caused by this intervention → value 0.
    assert!(SocialOutcome { attribution: 0.0, ..base }.value().abs() < 1e-12);
    // displacement 1: everything merely moved → value 0.
    assert!(SocialOutcome { displacement: 1.0, ..base }.value().abs() < 1e-12);
    // drop-off 0 leaves any value unchanged for any year count.
    assert!((value_after_drop_off(base.value(), 0.0, 10) - 5_000.0).abs() < 1e-9);
    // drop-off 1 kills the value from year 1 onward.
    assert!(value_after_drop_off(5_000.0, 1.0, 1).abs() < 1e-12);
}

// Locks down: retention/churn boundary values — churn 1.0 (100%), retention 0 and 1.
#[test]
fn edge_retention_boundaries() {
    // Total churn: 100% of the cohort lost.
    assert!((churn_rate_percent(1_000.0, 1_000.0).unwrap() - 100.0).abs() < 1e-9);
    // Zero retention: nobody left.
    assert!(retention_percent(0.0, 1_000.0).unwrap().abs() < 1e-12);
    // Full retention: 100%.
    assert!((retention_percent(1_000.0, 1_000.0).unwrap() - 100.0).abs() < 1e-9);
    // completers at fraction 0 and 1.
    assert!(completers(50_000.0, 0.0).abs() < 1e-12);
    assert!((completers(50_000.0, 1.0) - 50_000.0).abs() < 1e-9);
}

// Locks down: extreme magnitudes stay finite (no NaN/inf) across the numeric kernels.
#[test]
fn edge_extreme_magnitudes_stay_finite() {
    let big = 1e15;
    assert!(roi(big, 1e3).unwrap().is_finite());
    assert!(net_present_value(&[big, big, -big], 0.035).is_finite());
    assert!(pv_of_interest_avoided(1e12, 0.035, 500).is_finite());
    assert!(capacity_value_per_year(1e6, 8.0, 260.0, 1e4).is_finite());
    assert!(virtual_ward_gross_annual_value(1e6, 1.0, 1e6).is_finite());
    // Deep geometric decay underflows toward 0.0, never NaN.
    let decayed = value_after_drop_off(big, 0.5, 1_000);
    assert!(decayed.is_finite() && decayed >= 0.0);
    // A grotesque TDR still grades E rather than misbehaving.
    assert_eq!(sqale_grade(1e300), SqaleGrade::E);
    assert!(annual_maintenance_benchmark(big, 0.2).is_finite());
    assert!(monetized_health_value(1e12, 1e6).is_finite());
}

// =====================================================================
// SECTION 2: PROPERTIES / INVARIANTS
// =====================================================================

// ---- ROI classes ----

// Locks down: strict_financial_roi ≤ economic_roi whenever capacity amounts are ≥ 0.
#[test]
fn prop_strict_roi_never_exceeds_economic_roi() {
    for cash in [0.0, 10_000.0, 450_000.0, 2_000_000.0] {
        for capacity in [0.0, 1_000.0, 600_000.0] {
            for costs in [1.0, 93_600.0, 500_000.0] {
                let lines = [
                    BenefitLine { class: BenefitClass::CashReleasing, amount: cash },
                    BenefitLine { class: BenefitClass::Capacity, amount: capacity },
                    BenefitLine { class: BenefitClass::Qualitative, amount: 0.0 },
                ];
                let strict = strict_financial_roi(&lines, costs).unwrap();
                let economic = economic_roi(&lines, costs).unwrap();
                assert!(strict <= economic + 1e-12, "strict {strict} > economic {economic}");
            }
        }
    }
}

// Locks down: total_benefits over all three classes equals the plain sum of amounts.
#[test]
fn prop_total_benefits_all_classes_is_plain_sum() {
    let amounts = [450_000.0, 600_000.0, 12_345.0, 0.0, 77.7];
    let classes = [
        BenefitClass::CashReleasing,
        BenefitClass::Capacity,
        BenefitClass::Qualitative,
        BenefitClass::CashReleasing,
        BenefitClass::Capacity,
    ];
    let lines: Vec<BenefitLine> = classes
        .iter()
        .zip(amounts.iter())
        .map(|(&class, &amount)| BenefitLine { class, amount })
        .collect();
    let all = [BenefitClass::CashReleasing, BenefitClass::Capacity, BenefitClass::Qualitative];
    let total = total_benefits(&lines, &all);
    let sum: f64 = amounts.iter().sum();
    assert!((total - sum).abs() < 1e-9);
}

// Locks down actual behavior: Qualitative lines are excluded from BOTH ROIs even with a
// nonzero amount, but total_benefits does include them when asked to.
#[test]
fn prop_qualitative_lines_contribute_zero_to_both_rois() {
    let without = [
        BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
        BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
    ];
    let with_q = [
        BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
        BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
        BenefitLine { class: BenefitClass::Qualitative, amount: 123_456.0 }, // deliberately nonzero
    ];
    let costs = 500_000.0;
    assert!(
        (strict_financial_roi(&with_q, costs).unwrap()
            - strict_financial_roi(&without, costs).unwrap())
        .abs()
            < 1e-12
    );
    assert!(
        (economic_roi(&with_q, costs).unwrap() - economic_roi(&without, costs).unwrap()).abs()
            < 1e-12
    );
    // total_benefits itself is class-agnostic: a nonzero qualitative amount IS summed
    // when Qualitative is in the include list.
    let q_only = total_benefits(&with_q, &[BenefitClass::Qualitative]);
    assert!((q_only - 123_456.0).abs() < 1e-9);
}

// Locks down: roi(b, c) > 0 if and only if b > c (for positive costs).
#[test]
fn prop_roi_positive_iff_benefits_exceed_costs() {
    for benefits in [0.0, 50.0, 100.0, 100.0 + 1e-6, 1e6] {
        for costs in [1e-3, 50.0, 100.0, 1e6] {
            let r = roi(benefits, costs).unwrap();
            assert_eq!(r > 0.0, benefits > costs, "b={benefits} c={costs} r={r}");
            // And roi == 0 exactly at break-even.
            if benefits == costs {
                assert!(r.abs() < 1e-12);
            }
        }
    }
}

// ---- SROI ----

// Locks down: SocialOutcome::value is linear in quantity and in financial_proxy.
#[test]
fn prop_social_outcome_value_linear_in_quantity_and_proxy() {
    let base = SocialOutcome {
        quantity: 1_500.0,
        financial_proxy: 1_800.0,
        attribution: 0.8,
        deadweight: 0.25,
        displacement: 0.1,
    };
    let v = base.value();
    for k in [0.0, 0.5, 2.0, 10.0] {
        let scaled_q = SocialOutcome { quantity: base.quantity * k, ..base };
        assert!(close(scaled_q.value(), k * v, 1e-12), "quantity scale {k}");
        let scaled_p = SocialOutcome { financial_proxy: base.financial_proxy * k, ..base };
        assert!(close(scaled_p.value(), k * v, 1e-12), "proxy scale {k}");
    }
}

// Locks down: value == 0 when deadweight == 1 or attribution == 0, over a grid.
#[test]
fn prop_social_outcome_zeroed_by_full_deadweight_or_no_attribution() {
    for quantity in [1.0, 500.0, 1e6] {
        for proxy in [1.0, 42.0, 1_800.0] {
            let dead = SocialOutcome {
                quantity,
                financial_proxy: proxy,
                attribution: 0.9,
                deadweight: 1.0,
                displacement: 0.0,
            };
            assert!(dead.value().abs() < 1e-12);
            let unattributed = SocialOutcome {
                quantity,
                financial_proxy: proxy,
                attribution: 0.0,
                deadweight: 0.1,
                displacement: 0.2,
            };
            assert!(unattributed.value().abs() < 1e-12);
        }
    }
}

// Locks down: sroi_ratio(total_outcome_value(outcomes), inv) == total / investment.
#[test]
fn prop_sroi_ratio_is_total_value_over_investment() {
    let outcomes = [
        SocialOutcome {
            quantity: 1_500.0,
            financial_proxy: 1_800.0,
            attribution: 0.8,
            deadweight: 0.25,
            displacement: 0.0,
        },
        SocialOutcome {
            quantity: 1_800.0,
            financial_proxy: 42.0,
            attribution: 1.0,
            deadweight: 0.0,
            displacement: 0.0,
        },
    ];
    let total = total_outcome_value(&outcomes);
    // Sum of individual values matches the total.
    let by_hand: f64 = outcomes.iter().map(SocialOutcome::value).sum();
    assert!((total - by_hand).abs() < 1e-9);
    for inv in [1.0, 200_000.0, 5e6] {
        let ratio = sroi_ratio(total, inv).unwrap();
        assert!(close(ratio, total / inv, 1e-12));
    }
}

// Locks down: value_after_drop_off(v, 0, n) == v for every n; one year at rate d scales by (1−d).
#[test]
fn prop_drop_off_identity_and_single_year() {
    for v in [0.0, 810.0, 1_620_000.0] {
        for n in [0u32, 1, 7, 50] {
            assert!(close(value_after_drop_off(v, 0.0, n), v, 1e-12));
        }
        for d in [0.05, 0.10, 0.50] {
            assert!(close(value_after_drop_off(v, d, 1), v * (1.0 - d), 1e-12));
        }
    }
}

// ---- SPACE ----

// Locks down: the SPACE rule needs ≥3 distinct dimensions AND ≥1 perceptual AND ≥1 system —
// all four few/many × single/mixed combinations.
#[test]
fn prop_space_rule_requires_three_dimensions_and_mixed_sources() {
    // Few dims (2) + one source: fails.
    let few_single = [
        metric(SpaceDimension::Activity, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::System),
    ];
    assert!(!space_rule_satisfied(&few_single));
    // Few dims (2) + mixed sources: still fails (dimension count binds).
    let few_mixed = [
        metric(SpaceDimension::Activity, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::Perceptual),
    ];
    assert!(!space_rule_satisfied(&few_mixed));
    // Many dims (3+) + one source: fails both ways (source mix binds).
    let many_system = [
        metric(SpaceDimension::Activity, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Performance, MetricSource::System),
    ];
    assert!(!space_rule_satisfied(&many_system));
    let many_perceptual = [
        metric(SpaceDimension::Satisfaction, MetricSource::Perceptual),
        metric(SpaceDimension::Communication, MetricSource::Perceptual),
        metric(SpaceDimension::Performance, MetricSource::Perceptual),
    ];
    assert!(!space_rule_satisfied(&many_perceptual));
    // Many dims + mixed sources: passes.
    let many_mixed = [
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Satisfaction, MetricSource::Perceptual),
        metric(SpaceDimension::Performance, MetricSource::System),
    ];
    assert!(space_rule_satisfied(&many_mixed));
    // All five dimensions, mixed: also passes.
    let all_five = [
        metric(SpaceDimension::Satisfaction, MetricSource::Perceptual),
        metric(SpaceDimension::Performance, MetricSource::System),
        metric(SpaceDimension::Activity, MetricSource::System),
        metric(SpaceDimension::Communication, MetricSource::Perceptual),
        metric(SpaceDimension::Efficiency, MetricSource::System),
    ];
    assert!(space_rule_satisfied(&all_five));
}

// Locks down: repeated metrics in one dimension count as ONE distinct dimension.
#[test]
fn prop_space_rule_counts_distinct_dimensions_only() {
    // Four metrics, mixed sources, but a single dimension: fails.
    let one_dim = [
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::Perceptual),
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::Perceptual),
    ];
    assert!(!space_rule_satisfied(&one_dim));
    // Duplicates on top of a compliant core do not break compliance.
    let compliant_with_dupes = [
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Efficiency, MetricSource::System),
        metric(SpaceDimension::Satisfaction, MetricSource::Perceptual),
        metric(SpaceDimension::Performance, MetricSource::System),
    ];
    assert!(space_rule_satisfied(&compliant_with_dupes));
}

// Locks down: time reclaimed and vendor-index conversions are simple products (zero annihilates).
#[test]
fn prop_space_time_conversions_are_products() {
    // Zero usable fraction kills the reclaimed time entirely.
    assert!(time_reclaimed_minutes_per_day(6.0, 19.0, 0.0).abs() < 1e-12);
    // Full usability is the raw product.
    assert!((time_reclaimed_minutes_per_day(6.0, 19.0, 1.0) - 114.0).abs() < 1e-9);
    // Vendor index claim is linear in points gained.
    assert!(vendor_index_minutes_per_week(0.0, 13.0).abs() < 1e-12);
    assert!((vendor_index_minutes_per_week(3.0, 13.0) - 39.0).abs() < 1e-9);
    // Capacity value is linear in each factor: doubling developers doubles value.
    let v1 = capacity_value_per_year(300.0, 0.75, 220.0, 60.0);
    let v2 = capacity_value_per_year(600.0, 0.75, 220.0, 60.0);
    assert!(close(v2, 2.0 * v1, 1e-12));
}

// ---- Technical debt ----

// Locks down: sqale_grade band edges use <= at every boundary (5, 10, 20, 50).
#[test]
fn prop_sqale_grade_band_edges() {
    let cases = [
        (0.0, SqaleGrade::A),
        (4.999, SqaleGrade::A),
        (5.0, SqaleGrade::A),   // inclusive upper bound
        (5.001, SqaleGrade::B),
        (9.999, SqaleGrade::B),
        (10.0, SqaleGrade::B),  // inclusive upper bound
        (10.001, SqaleGrade::C),
        (20.0, SqaleGrade::C),  // inclusive upper bound
        (20.001, SqaleGrade::D),
        (50.0, SqaleGrade::D),  // inclusive upper bound
        (50.001, SqaleGrade::E),
        (100.0, SqaleGrade::E),
    ];
    for (tdr, want) in cases {
        assert_eq!(sqale_grade(tdr), want, "tdr={tdr}");
    }
    // And the grade of a computed ratio agrees with grading the hand ratio.
    let tdr = technical_debt_ratio_percent(50_000.0, 1_000_000.0).unwrap();
    assert!((tdr - 5.0).abs() < 1e-9);
    assert_eq!(sqale_grade(tdr), SqaleGrade::A);
}

// Locks down: annual_interest is additive in its velocity-drag and failure components.
#[test]
fn prop_annual_interest_additive_in_components() {
    for hours in [0.0, 2_000.0, 6_000.0] {
        for drag in [0.0, 0.25, 0.40] {
            for failures in [0.0, 5.0, 12.0] {
                let whole = annual_interest(hours, drag, 75.0, failures, 8_000.0);
                let drag_only = annual_interest(hours, drag, 75.0, 0.0, 8_000.0);
                let failures_only = annual_interest(0.0, 0.0, 75.0, failures, 8_000.0);
                assert!(close(whole, drag_only + failures_only, 1e-12));
            }
        }
    }
}

// Locks down: technical_debt payback == remediation / avoided, over a grid.
#[test]
fn prop_td_payback_is_cost_over_avoided() {
    for cost in [1.0, 85_500.0, 500_000.0] {
        for avoided in [40_000.0, 165_600.0, 400_000.0] {
            let p = technical_debt::payback_period_years(cost, avoided).unwrap();
            assert!(close(p, cost / avoided, 1e-12));
        }
    }
    // interest_avoided_per_year is the linear reduction it claims to be.
    assert!((interest_avoided_per_year(276_000.0, 0.0)).abs() < 1e-12);
    assert!((interest_avoided_per_year(276_000.0, 1.0) - 276_000.0).abs() < 1e-9);
}

// Locks down: pv_of_interest_avoided < undiscounted interest × years for every r > 0,
// and equals it exactly at r == 0.
#[test]
fn prop_pv_of_interest_below_undiscounted_for_positive_rates() {
    let annual = 165_600.0;
    for years in [1u32, 3, 10, 30] {
        let undiscounted = annual * years as f64;
        assert!(close(pv_of_interest_avoided(annual, 0.0, years), undiscounted, 1e-12));
        for r in [0.01, 0.035, 0.10, 0.50] {
            let pv = pv_of_interest_avoided(annual, r, years);
            assert!(pv < undiscounted, "r={r} years={years}");
            assert!(pv > 0.0);
        }
    }
    // paydown_net_value sign flips with the PV/cost ordering.
    assert!(paydown_net_value(100.0, 200.0) < 0.0);
    assert!(paydown_net_value(200.0, 100.0) > 0.0);
}

// ---- Tornado / sensitivity ----

// Locks down actual behavior: rank_by_swing sorts by RAW swing value descending — a
// negative entry sorts last even when its magnitude is largest.
#[test]
fn prop_rank_by_swing_sorts_raw_values_descending() {
    let mut swings = [
        ("negative-huge", -5_000_000.0),
        ("small", 48_000.0),
        ("large", 2_376_000.0),
        ("medium", 880_000.0),
        ("zero", 0.0),
    ];
    rank_by_swing(&mut swings);
    assert_eq!(swings[0].0, "large");
    assert_eq!(swings[1].0, "medium");
    assert_eq!(swings[2].0, "small");
    assert_eq!(swings[3].0, "zero");
    // Raw ordering, not |swing|: the −5M entry is LAST despite the largest magnitude.
    assert_eq!(swings[4].0, "negative-huge");
    // Every adjacent pair is non-increasing.
    for pair in swings.windows(2) {
        assert!(pair[0].1 >= pair[1].1);
    }
}

// Locks down: OneWayResult::swing is |high − low| — symmetric under swapping ends.
#[test]
fn prop_swing_is_symmetric_absolute_difference() {
    let grid = [(170_400.0, 2_546_400.0), (-100.0, 100.0), (5.0, 5.0), (1_248_000.0, 1_200_000.0)];
    for (low, high) in grid {
        let forward = OneWayResult { result_at_low: low, result_at_high: high };
        let reversed = OneWayResult { result_at_low: high, result_at_high: low };
        assert!(close(forward.swing(), (high - low).abs(), 1e-12));
        assert!(close(forward.swing(), reversed.swing(), 1e-12));
        assert!(forward.swing() >= 0.0);
    }
}

// Locks down: net_benefit == annual_benefit − annual_cost across a parameter grid.
#[test]
fn prop_case_net_benefit_is_benefit_minus_cost() {
    for developers in [1.0, 50.0, 200.0] {
        for hours in [0.0, 0.1, 0.5, 1.0] {
            for license in [0.0, 39.0, 100.0] {
                let case = CodingAssistantCase {
                    developers,
                    hours_saved_per_day: hours,
                    loaded_cost_per_hour: 60.0,
                    working_days_per_year: 220.0,
                    license_per_dev_per_month: license,
                };
                assert!(close(case.net_benefit(), case.annual_benefit() - case.annual_cost(), 1e-12));
            }
        }
    }
}

// Locks down: at exactly the threshold hours, net benefit is zero (within 1e-9).
#[test]
fn prop_threshold_hours_zeroes_net_benefit() {
    for developers in [50.0, 200.0] {
        for loaded in [40.0, 60.0] {
            for license in [10.0, 39.0] {
                let mut case = CodingAssistantCase {
                    developers,
                    hours_saved_per_day: 0.5,
                    loaded_cost_per_hour: loaded,
                    working_days_per_year: 220.0,
                    license_per_dev_per_month: license,
                };
                let threshold = case.threshold_hours_saved_per_day().unwrap();
                case.hours_saved_per_day = threshold;
                assert!(case.net_benefit().abs() < 1e-9, "residual {}", case.net_benefit());
                // Just above the threshold the case is positive; just below, negative.
                case.hours_saved_per_day = threshold + 1e-6;
                assert!(case.net_benefit() > 0.0);
                case.hours_saved_per_day = threshold - 1e-6;
                assert!(case.net_benefit() < 0.0);
            }
        }
    }
}

// ---- Time horizon ----

// Locks down: net_benefit_at_horizon is affine-linear in years with slope (benefit − cost).
#[test]
fn prop_net_benefit_linear_in_horizon() {
    let (implementation, benefit, run) = (2_000_000.0, 600_000.0, 200_000.0);
    let slope = benefit - run;
    for (y1, y2) in [(0.0, 1.0), (1.0, 3.0), (2.5, 10.0), (5.0, 5.0)] {
        let n1 = net_benefit_at_horizon(implementation, benefit, run, y1);
        let n2 = net_benefit_at_horizon(implementation, benefit, run, y2);
        assert!(close(n2 - n1, (y2 - y1) * slope, 1e-12));
    }
    // Horizon 0 is exactly −implementation cost.
    let at_zero = net_benefit_at_horizon(implementation, benefit, run, 0.0);
    assert!(close(at_zero, -implementation, 1e-12));
}

// Locks down: break-even horizon × annual net benefit reproduces the implementation cost.
#[test]
fn prop_break_even_times_annual_net_is_implementation_cost() {
    for implementation in [1.0, 500_000.0, 2_000_000.0] {
        for (benefit, run) in [(600_000.0, 200_000.0), (450_000.0, 50_000.0), (100.0, 25.0)] {
            let t = break_even_horizon_years(implementation, benefit, run).unwrap();
            assert!(close(t * (benefit - run), implementation, 1e-9));
            // And net benefit AT the break-even horizon is zero.
            let net = net_benefit_at_horizon(implementation, benefit, run, t);
            assert!(net.abs() < 1e-6 * implementation.max(1.0), "net {net}");
        }
    }
}

// Locks down: net_benefit_by_horizons agrees pointwise with net_benefit_at_horizon.
#[test]
fn prop_by_horizons_matches_pointwise() {
    let horizons = [0.0, 1.0, 2.5, 5.0, 7.75, 10.0];
    let report = net_benefit_by_horizons(2_000_000.0, 600_000.0, 200_000.0, &horizons);
    assert_eq!(report.len(), horizons.len());
    for (i, &(h, net)) in report.iter().enumerate() {
        assert!((h - horizons[i]).abs() < 1e-12); // order preserved
        let pointwise = net_benefit_at_horizon(2_000_000.0, 600_000.0, 200_000.0, h);
        assert!(close(net, pointwise, 1e-12));
    }
}

// Locks down: time_horizon NPV at rate 0 equals the plain sum of flows.
#[test]
fn prop_npv_rate_zero_is_sum_of_flows() {
    let flow_sets: [&[f64]; 3] = [
        &[-2_000_000.0, 400_000.0, 400_000.0, 400_000.0],
        &[100.0, -50.0, 25.0, -12.5, 6.25],
        &[0.0, 0.0, 0.0],
    ];
    for flows in flow_sets {
        let sum: f64 = flows.iter().sum();
        assert!(close(net_present_value(flows, 0.0), sum, 1e-12));
        // Any positive rate strictly reduces a stream of positive future flows.
        if flows.iter().skip(1).all(|&f| f > 0.0) {
            assert!(net_present_value(flows, 0.035) < sum);
        }
    }
}

// ---- TCO ----

// Locks down: undiscounted TCO ≥ discounted TCO for positive rates; equality at rate 0.
#[test]
fn prop_undiscounted_tco_dominates_discounted() {
    for annual in [0.0, 50_000.0, 190_000.0] {
        for horizon in [1u32, 5, 10] {
            for decommission in [0.0, 30_000.0] {
                let p = TcoProfile {
                    initial_cost: 900_000.0,
                    integration_and_training: 150_000.0,
                    annual_run_cost: annual,
                    horizon_years: horizon,
                    decommission_cost: decommission,
                };
                assert!(close(p.discounted_tco(0.0), p.undiscounted_tco(), 1e-12));
                for rate in [0.035, 0.08, 0.12] {
                    let disc = p.discounted_tco(rate);
                    assert!(disc <= p.undiscounted_tco() + 1e-9);
                    if annual > 0.0 || decommission > 0.0 {
                        // Strictly cheaper in PV terms when any cost is in the future.
                        assert!(disc < p.undiscounted_tco());
                    }
                }
            }
        }
    }
}

// Locks down: initial_cost_share is in (0, 1] whenever initial cost is positive.
#[test]
fn prop_initial_cost_share_bounded() {
    for initial in [1.0, 250_000.0, 900_000.0] {
        for annual in [0.0, 120_000.0] {
            for integration in [0.0, 180_000.0] {
                let p = TcoProfile {
                    initial_cost: initial,
                    integration_and_training: integration,
                    annual_run_cost: annual,
                    horizon_years: 5,
                    decommission_cost: 0.0,
                };
                let share = p.initial_cost_share().unwrap();
                assert!(share > 0.0 && share <= 1.0, "share {share}");
                // Share is exactly 1 only when there are no other costs at all.
                if annual == 0.0 && integration == 0.0 {
                    assert!(close(share, 1.0, 1e-12));
                } else {
                    assert!(share < 1.0);
                }
            }
        }
    }
}

// Locks down: tco_advantage is antisymmetric and zero against itself.
#[test]
fn prop_tco_advantage_antisymmetric() {
    let saas = TcoProfile {
        initial_cost: 250_000.0,
        integration_and_training: 180_000.0,
        annual_run_cost: 120_000.0,
        horizon_years: 5,
        decommission_cost: 60_000.0,
    };
    let build = TcoProfile {
        initial_cost: 900_000.0,
        integration_and_training: 150_000.0,
        annual_run_cost: 190_000.0,
        horizon_years: 5,
        decommission_cost: 30_000.0,
    };
    assert!(close(tco_advantage(&saas, &build), -tco_advantage(&build, &saas), 1e-12));
    assert!(tco_advantage(&saas, &saas).abs() < 1e-12);
    // Positive means the first argument is cheaper.
    assert!(tco_advantage(&saas, &build) > 0.0);
    // The advantage equals the difference of undiscounted TCOs.
    let diff = build.undiscounted_tco() - saas.undiscounted_tco();
    assert!(close(tco_advantage(&saas, &build), diff, 1e-12));
}

// Locks down: maintenance benchmark is a plain product, linear in build cost.
#[test]
fn prop_maintenance_benchmark_linear() {
    assert!((annual_maintenance_benchmark(1_000_000.0, 0.15) - 150_000.0).abs() < 1e-9);
    assert!(close(
        annual_maintenance_benchmark(2_000_000.0, 0.15),
        2.0 * annual_maintenance_benchmark(1_000_000.0, 0.15),
        1e-12
    ));
}

// ---- RPM ----

// Locks down: PMPM at compliance (0,0) and (1,1) brackets every intermediate compliance mix.
#[test]
fn prop_rpm_pmpm_bracketed_by_compliance_extremes() {
    let floor = revenue_per_member_per_month(0.0, CPT_99454_DEVICE_SUPPLY, 0.0, CPT_99457_FIRST_20_MIN);
    let ceiling = revenue_per_member_per_month(1.0, CPT_99454_DEVICE_SUPPLY, 1.0, CPT_99457_FIRST_20_MIN);
    assert!(floor.abs() < 1e-12);
    assert!((ceiling - (CPT_99454_DEVICE_SUPPLY + CPT_99457_FIRST_20_MIN)).abs() < 1e-9);
    for device in [0.0, 0.25, 0.5, 0.7, 1.0] {
        for management in [0.0, 0.3, 0.6, 1.0] {
            let pmpm = revenue_per_member_per_month(
                device,
                CPT_99454_DEVICE_SUPPLY,
                management,
                CPT_99457_FIRST_20_MIN,
            );
            assert!(pmpm >= floor - 1e-12 && pmpm <= ceiling + 1e-12, "pmpm {pmpm}");
        }
    }
}

// Locks down: annual revenue is exactly 12 × monthly, and the chain equals 12 × enrolled × PMPM.
#[test]
fn prop_rpm_annual_is_twelve_times_monthly() {
    for enrolled in [1.0, 400.0, 5_000.0] {
        for pmpm in [0.0, 58.84, 130.0] {
            let monthly = monthly_revenue(enrolled, pmpm);
            let annual = annual_revenue(monthly);
            assert!(close(annual, 12.0 * monthly, 1e-12));
            assert!(close(annual, 12.0 * enrolled * pmpm, 1e-12));
        }
    }
}

// Locks down: no compliance change → no compliance-lever gain; and the lever is linear.
#[test]
fn prop_rpm_compliance_lever_zero_when_unchanged() {
    for enrolled in [1.0, 400.0, 10_000.0] {
        assert!(compliance_lever_annual_gain(enrolled, 0.0, CPT_99454_DEVICE_SUPPLY).abs() < 1e-12);
    }
    // A 15-point move is exactly 3× a 5-point move.
    let small = compliance_lever_annual_gain(400.0, 0.05, CPT_99454_DEVICE_SUPPLY);
    let large = compliance_lever_annual_gain(400.0, 0.15, CPT_99454_DEVICE_SUPPLY);
    assert!(close(large, 3.0 * small, 1e-12));
}

// Locks down: the general NHS-style formula reduces to the virtual-ward gross value
// when only the bed-day term is active, over a grid.
#[test]
fn prop_nhs_style_reduces_to_virtual_ward_gross() {
    for beds in [10.0, 50.0] {
        for occupancy in [0.5, 0.8, 1.0] {
            for saving in [50.0, 150.0] {
                let gross = virtual_ward_gross_annual_value(beds, occupancy, saving);
                let bed_days = beds * occupancy * 365.0;
                // inpatient − virtual-ward day cost == net saving per day.
                let general = nhs_style_net_value(0.0, 0.0, bed_days, saving + 250.0, 250.0, 0.0);
                assert!(close(gross, general, 1e-9));
            }
        }
    }
    // Margin identities: zero-cost margin is the revenue; equal cost zeroes it.
    assert!(close(annual_margin(282_446.40, 0.0), 282_446.40, 1e-12));
    assert!(annual_margin(180_000.0, 180_000.0).abs() < 1e-12);
}

// ---- Retention ----

// Locks down: with full retention at every period, expected benefit == sum of benefit rates.
#[test]
fn prop_expected_benefit_full_retention_is_rate_sum() {
    let rates = [4.0, 3.0, 2.5, 1.0];
    let full = [1.0; 4];
    let expected = expected_benefit_per_acquired_user(&full, &rates);
    let sum: f64 = rates.iter().sum();
    assert!(close(expected, sum, 1e-12));
    // Any sub-1.0 retention strictly reduces it (for positive rates).
    let partial = [1.0, 0.25, 0.08, 0.04];
    assert!(expected_benefit_per_acquired_user(&partial, &rates) < sum);
}

// Locks down as coded: zip truncation — extra elements of the LONGER slice are ignored.
#[test]
fn prop_expected_benefit_zip_truncates_to_shorter_slice() {
    // Retention longer than benefits: third retention entry ignored.
    let longer_retention = expected_benefit_per_acquired_user(&[1.0, 0.5, 0.9], &[10.0, 10.0]);
    assert!(close(longer_retention, 15.0, 1e-12));
    // Benefits longer than retention: second and third benefit entries ignored.
    let longer_benefits = expected_benefit_per_acquired_user(&[1.0], &[10.0, 999.0, 999.0]);
    assert!(close(longer_benefits, 10.0, 1e-12));
    // Both truncations agree with the explicitly-trimmed computation.
    let trimmed = expected_benefit_per_acquired_user(&[1.0, 0.5], &[10.0, 10.0]);
    assert!(close(longer_retention, trimmed, 1e-12));
}

// Locks down: cost_per_retained_user(cac, 1.0) == cac — full retention costs exactly CAC.
#[test]
fn prop_cost_per_retained_user_identity_at_full_retention() {
    for cac in [0.0, 5.0, 125.0, 1e6] {
        let cost = cost_per_retained_user(cac, 1.0).unwrap();
        assert!(close(cost, cac, 1e-12));
    }
    // Halving retention doubles the cost per retained user.
    let at_half = cost_per_retained_user(5.0, 0.5).unwrap();
    let at_quarter = cost_per_retained_user(5.0, 0.25).unwrap();
    assert!(close(at_quarter, 2.0 * at_half, 1e-12));
}

// Locks down: retention_improvement_value equals the monetized QALY difference computed
// through the component functions, and is zero when nothing changes.
#[test]
fn prop_retention_improvement_matches_component_chain() {
    for cohort in [10_000.0, 100_000.0] {
        for (from, to) in [(0.04, 0.06), (0.04, 0.04), (0.10, 0.02)] {
            let direct = retention_improvement_value(cohort, from, to, 0.02, 20_000.0);
            let via_chain = monetized_health_value(
                qalys_delivered(completers(cohort, to), 0.02),
                20_000.0,
            ) - monetized_health_value(
                qalys_delivered(completers(cohort, from), 0.02),
                20_000.0,
            );
            assert!(close(direct, via_chain, 1e-9), "from {from} to {to}");
            if from == to {
                assert!(direct.abs() < 1e-9);
            }
            if to < from {
                assert!(direct < 0.0); // retention loss destroys value
            }
        }
    }
}

// Locks down: retention% and churn% are complements when lost = cohort − active.
#[test]
fn prop_retention_and_churn_are_complements() {
    for cohort in [100.0, 100_000.0] {
        for active in [0.0, cohort * 0.04, cohort * 0.5, cohort] {
            let retention = retention_percent(active, cohort).unwrap();
            let churn = churn_rate_percent(cohort - active, cohort).unwrap();
            assert!(close(retention + churn, 100.0, 1e-12));
        }
    }
}

// ---- Screening ----

// Locks down: TP/FP accounting — boundaries and the population budget.
#[test]
fn prop_screening_counts_within_population() {
    for population in [1_000.0, 100_000.0] {
        for prevalence in [0.0, 0.005, 0.5, 1.0] {
            for accuracy in [0.5, 0.9, 1.0] {
                let tp = true_positives(population, prevalence, accuracy);
                let fp = false_positives(population, prevalence, accuracy);
                assert!(tp >= 0.0 && fp >= 0.0);
                // Positives can never exceed the population screened.
                assert!(tp + fp <= population + 1e-9);
            }
        }
    }
    // Perfect sensitivity finds all cases; perfect specificity clears all healthy.
    assert!(close(true_positives(100_000.0, 0.005, 1.0), 500.0, 1e-12));
    assert!(false_positives(100_000.0, 0.005, 1.0).abs() < 1e-12);
    // Zero workup cost collapses total cost to the screening line alone.
    let screen_only = total_programme_cost(100_000.0, 15.0, 0.0, 450.0, 4_975.0);
    assert!(close(screen_only, 1_500_000.0, 1e-12));
}

// Locks down: PPV rises monotonically with prevalence (fixed test accuracy).
#[test]
fn prop_ppv_monotone_in_prevalence() {
    let mut previous = -1.0;
    for prevalence in [0.001, 0.005, 0.01, 0.05, 0.20, 0.50, 0.90] {
        let ppv = positive_predictive_value(0.90, 0.95, prevalence).unwrap();
        assert!(ppv > previous, "ppv not increasing at prev {prevalence}");
        assert!(ppv > 0.0 && ppv <= 1.0);
        previous = ppv;
    }
    // Overdiagnosis harm can flip the per-case value negative.
    assert!(net_value_per_case_found(1_000.0, 5_000.0) < 0.0);
    assert!(net_value_per_case_found(5_000.0, 1_000.0) > 0.0);
}

// =====================================================================
// SECTION 3: CROSS-MODULE CONSISTENCY
// =====================================================================

// Locks down: time_horizon::net_present_value(net flows) == present_value(benefits) −
// present_value(costs) from cost_benefit_analysis, at every rate tried.
#[test]
fn cross_npv_of_net_flows_equals_pv_benefits_minus_pv_costs() {
    let benefits = [0.0, 600_000.0, 600_000.0, 600_000.0, 600_000.0, 600_000.0];
    let costs = [2_000_000.0, 200_000.0, 200_000.0, 200_000.0, 200_000.0, 200_000.0];
    let net: Vec<f64> = benefits.iter().zip(costs.iter()).map(|(b, c)| b - c).collect();
    for rate in [0.0, 0.015, 0.035, 0.10] {
        let via_net = net_present_value(&net, rate);
        let via_gross = cost_benefit_analysis::present_value(&benefits, rate)
            - cost_benefit_analysis::present_value(&costs, rate);
        assert!(close(via_net, via_gross, 1e-9), "rate {rate}: {via_net} vs {via_gross}");
    }
}

// Locks down: on the SAME flow vector the two discounting kernels are numerically identical.
#[test]
fn cross_npv_and_present_value_agree_on_identical_flows() {
    let flows = [-2_000_000.0, 400_000.0, 400_000.0, 400_000.0, 400_000.0, 400_000.0];
    for rate in [0.0, 0.035, 0.12] {
        let th = net_present_value(&flows, rate);
        let cba = cost_benefit_analysis::present_value(&flows, rate);
        assert!(close(th, cba, 1e-9), "rate {rate}");
    }
}

// Locks down: technical_debt and return_on_investment payback functions agree on
// equivalent inputs (cost repaid by a constant annual saving).
#[test]
fn cross_payback_functions_agree() {
    for cost in [85_500.0, 500_000.0, 12.0] {
        for annual in [40_000.0, 165_600.0, 400_000.0] {
            let td = technical_debt::payback_period_years(cost, annual).unwrap();
            let roi_pb = return_on_investment::payback_period_years(cost, annual).unwrap();
            assert!(close(td, roi_pb, 1e-12));
        }
    }
    // Both agree on the undefined case too.
    assert!(technical_debt::payback_period_years(1.0, 0.0).is_none());
    assert!(return_on_investment::payback_period_years(1.0, 0.0).is_none());
}

// Locks down: pv_of_interest_avoided (years 1..=n annuity) matches the generic NPV of the
// same stream with a zero year-0 flow.
#[test]
fn cross_interest_annuity_matches_generic_npv() {
    let annual = 55_000.0;
    for (rate, years) in [(0.035, 4u32), (0.10, 7), (0.0, 3)] {
        let annuity = pv_of_interest_avoided(annual, rate, years);
        let mut flows = vec![0.0]; // year 0: nothing accrues
        flows.extend(std::iter::repeat_n(annual, years as usize));
        let generic = net_present_value(&flows, rate);
        assert!(close(annuity, generic, 1e-9), "rate {rate} years {years}");
    }
}

// =====================================================================
// SECTION 4: DOMAIN SCENARIOS
// =====================================================================

// Scenario: US RPM panel P&L — CPT stack to annual margin, plus the compliance lever
// and an NHS virtual-ward mirror. All expected values hand-computed.
#[test]
fn scenario_rpm_panel_profit_and_loss() {
    // 250 hypertension patients; 80% meet the 16-day rule, 65% get 20 logged minutes.
    let pmpm = revenue_per_member_per_month(
        0.80,
        CPT_99454_DEVICE_SUPPLY,   // $43.03
        0.65,
        CPT_99457_FIRST_20_MIN,    // $47.87
    );
    // 0.80 × 43.03 + 0.65 × 47.87 = 34.424 + 31.1155 = 65.5395
    assert!((pmpm - 65.5395).abs() < 1e-9);

    let monthly = monthly_revenue(250.0, pmpm);
    assert!((monthly - 16_384.875).abs() < 1e-6); // 250 × 65.5395

    let annual = annual_revenue(monthly);
    assert!((annual - 196_618.5).abs() < 1e-6); // 12 × 16,384.875

    // One-time setup billing in year 1: 250 × $19.73 = $4,932.50.
    let setup = 250.0 * CPT_99453_SETUP;
    assert!((setup - 4_932.5).abs() < 1e-9);

    // Steady-state margin against a $120k service cost.
    let margin = annual_margin(annual, 120_000.0);
    assert!((margin - 76_618.5).abs() < 1e-6);

    // ROI framing of the steady-state year: (196,618.5 − 120,000) / 120,000 = 63.85%.
    let service_roi = roi(annual, 120_000.0).unwrap();
    assert!((service_roi - 0.6384875).abs() < 1e-9);

    // Engineering lever: sync reliability moves 16-day compliance 0.80 → 0.90:
    // 250 × 0.10 × 43.03 × 12 = $12,909/year.
    let gain = compliance_lever_annual_gain(250.0, 0.90 - 0.80, CPT_99454_DEVICE_SUPPLY);
    assert!((gain - 12_909.0).abs() < 1e-6);

    // Upsell: 30% of the panel logs a second 20-minute block (CPT 99458):
    // extra PMPM = 0.30 × 38.49 = 11.547 → extra annual = 250 × 11.547 × 12 = $34,641.
    let extra_pmpm =
        revenue_per_member_per_month(0.30, CPT_99458_ADDITIONAL_20_MIN, 0.0, CPT_99457_FIRST_20_MIN);
    assert!((extra_pmpm - 11.547).abs() < 1e-9);
    let extra_annual = annual_revenue(monthly_revenue(250.0, extra_pmpm));
    assert!((extra_annual - 34_641.0).abs() < 1e-6);

    // NHS mirror: 20 virtual-ward beds, 75% occupancy, £120 net/day, 120 admissions
    // avoided at £1,500 marginal cost, £400k service cost.
    let gross = virtual_ward_gross_annual_value(20.0, 0.75, 120.0);
    assert!((gross - 657_000.0).abs() < 1e-6); // 20 × 0.75 × 365 × 120
    let bed_days = 20.0 * 0.75 * 365.0; // 5,475
    let net = nhs_style_net_value(120.0, 1_500.0, bed_days, 350.0, 230.0, 400_000.0);
    // 180,000 + 5,475 × 120 − 400,000 = 437,000
    assert!((net - 437_000.0).abs() < 1e-6);
}

// Scenario: technical-debt paydown investment case — principal, grade, interest, PV at
// Green Book 3.5%, payback, and the cash-vs-capacity ROI framing.
#[test]
fn scenario_tech_debt_paydown_investment_case() {
    // Integration layer: 2,000 remediation hours at £80/h → £160k principal.
    let principal = sqale_principal(2_000.0, 80.0);
    assert!((principal - 160_000.0).abs() < 1e-9);

    // Rebuild estimate £1.6M → TDR exactly 10% → boundary grade B (inclusive band edge).
    let tdr = technical_debt_ratio_percent(principal, 1_600_000.0).unwrap();
    assert!((tdr - 10.0).abs() < 1e-9);
    assert_eq!(sqale_grade(tdr), SqaleGrade::B);

    // Interest: 4,000 dev-h/year at 25% drag × £80 + 5 failures × £6,000
    // = 80,000 + 30,000 = £110,000/year.
    let interest = annual_interest(4_000.0, 0.25, 80.0, 5.0, 6_000.0);
    assert!((interest - 110_000.0).abs() < 1e-9);

    // Remediate the worst 40% of principal (£64k), modeled 50% interest reduction.
    let remediation = 0.40 * principal;
    let avoided = interest_avoided_per_year(interest, 0.50);
    assert!((avoided - 55_000.0).abs() < 1e-9);

    // Payback = 64,000 / 55,000 ≈ 1.1636 years ≈ 14 months.
    let payback = technical_debt::payback_period_years(remediation, avoided).unwrap();
    assert!((payback - 64_000.0 / 55_000.0).abs() < 1e-12);
    assert!((payback * 12.0 - 13.9636).abs() < 0.001);

    // PV of 4 years of avoided interest at 3.5%:
    // 55,000 × (1.035⁻¹ + 1.035⁻² + 1.035⁻³ + 1.035⁻⁴) ≈ £202,019.36.
    let pv = pv_of_interest_avoided(avoided, 0.035, 4);
    assert!((pv - 202_019.36).abs() < 0.01);

    // Paydown net value ≈ £138,019 — and it must equal the generic NPV of the
    // same decision modeled as flows [−64k, +55k × 4].
    let net = paydown_net_value(pv, remediation);
    assert!((net - 138_019.36).abs() < 0.01);
    let flows = [-remediation, avoided, avoided, avoided, avoided];
    assert!(close(net, net_present_value(&flows, 0.035), 1e-9));

    // Honest ROI framing: avoided interest is CAPACITY, not cash. Strict financial
    // ROI is −100% (no cash line); economic ROI on the undiscounted 4-year benefit
    // is (220,000 − 64,000) / 64,000 = +243.75%.
    let lines = [
        BenefitLine { class: BenefitClass::Capacity, amount: avoided * 4.0 },
        BenefitLine { class: BenefitClass::Qualitative, amount: 0.0 }, // morale, not monetized
    ];
    let strict = strict_financial_roi(&lines, remediation).unwrap();
    assert!((strict - (-1.0)).abs() < 1e-12);
    let economic = economic_roi(&lines, remediation).unwrap();
    assert!((economic - 2.4375).abs() < 1e-9);
}

// Scenario: screening programme feeding an SROI report — PPV, cost per case,
// adjusted outcome values, ratio, proxy-share honesty check, and drop-off.
#[test]
fn scenario_screening_programme_feeds_sroi_report() {
    // 50,000 screened; prevalence 1%; sensitivity 85%; specificity 97%;
    // scan £10; confirmatory workup £300.
    let ppv = positive_predictive_value(0.85, 0.97, 0.01).unwrap();
    // 0.0085 / (0.0085 + 0.03 × 0.99) = 0.0085 / 0.0382 ≈ 0.22251
    assert!((ppv - 0.0085 / 0.0382).abs() < 1e-12);
    assert!((ppv - 0.2225).abs() < 0.0002);

    let tp = true_positives(50_000.0, 0.01, 0.85);
    assert!((tp - 425.0).abs() < 1e-9);
    let fp = false_positives(50_000.0, 0.01, 0.97);
    assert!((fp - 1_485.0).abs() < 1e-9);

    // Cost = 50,000 × 10 + (425 + 1,485) × 300 = 500,000 + 573,000 = £1,073,000.
    let cost = total_programme_cost(50_000.0, 10.0, 300.0, tp, fp);
    assert!((cost - 1_073_000.0).abs() < 1e-6);
    let per_case = cost_per_true_case(cost, tp).unwrap();
    assert!((per_case - 1_073_000.0 / 425.0).abs() < 1e-9); // ≈ £2,524.71

    // Wilson–Jungner economics test: value per case (£9,000 saving − £1,500
    // overdiagnosis harm = £7,500) clears the £2,524.71 cost per case.
    let case_value = net_value_per_case_found(9_000.0, 1_500.0);
    assert!((case_value - 7_500.0).abs() < 1e-9);
    assert!(case_value > per_case);

    // SROI report on the programme year (investment = programme cost £1,073,000).
    // Outcome 1 (payer-real, adjusted): early treatment for 425 cases at £7,500,
    // attribution 90%, deadweight 20%, displacement 5%:
    // 425 × 7,500 × 0.9 × 0.8 × 0.95 = £2,180,250.
    let treatment = SocialOutcome {
        quantity: tp,
        financial_proxy: case_value,
        attribution: 0.90,
        deadweight: 0.20,
        displacement: 0.05,
    };
    assert!((treatment.value() - 2_180_250.0).abs() < 1e-6);
    // Outcome 2 (proxy-valued wellbeing): reassurance/earlier certainty for the 425
    // found, proxied at £2,000, attribution 80%, deadweight 25%:
    // 425 × 2,000 × 0.8 × 0.75 = £510,000.
    let wellbeing = SocialOutcome {
        quantity: tp,
        financial_proxy: 2_000.0,
        attribution: 0.80,
        deadweight: 0.25,
        displacement: 0.0,
    };
    assert!((wellbeing.value() - 510_000.0).abs() < 1e-6);

    let total = total_outcome_value(&[treatment, wellbeing]);
    assert!((total - 2_690_250.0).abs() < 1e-6);
    let ratio = sroi_ratio(total, cost).unwrap();
    assert!((ratio - 2_690_250.0 / 1_073_000.0).abs() < 1e-12); // ≈ 2.51 : 1
    assert!((ratio - 2.507).abs() < 0.001);

    // Honesty check: only ~19% of the claimed value is proxy-valued wellbeing.
    let share = proxy_valued_share(wellbeing.value(), treatment.value()).unwrap();
    assert!((share - 510_000.0 / 2_690_250.0).abs() < 1e-12);
    assert!((share - 0.1896).abs() < 0.001);

    // The wellbeing claim decays at 15%/year: after 2 years, 510,000 × 0.85² = £368,475.
    let decayed = value_after_drop_off(wellbeing.value(), 0.15, 2);
    assert!((decayed - 368_475.0).abs() < 1e-6);
}

// Scenario: DevEx platform investment — SPACE-compliant measurement, capacity value,
// tornado ranking, TCO of the platform, and the multi-horizon break-even story.
#[test]
fn scenario_devex_platform_investment_case() {
    // Measurement design passes the SPACE rule (3 dims, telemetry + survey).
    let design = [
        metric(SpaceDimension::Efficiency, MetricSource::System),      // CI p75
        metric(SpaceDimension::Satisfaction, MetricSource::Perceptual), // focus survey
        metric(SpaceDimension::Performance, MetricSource::System),     // change-failure rate
    ];
    assert!(space_rule_satisfied(&design));

    // CI speedup: 5 builds/day × 12 min saved × 0.4 usable = 24 min/day/dev.
    let minutes = time_reclaimed_minutes_per_day(5.0, 12.0, 0.4);
    assert!((minutes - 24.0).abs() < 1e-9);

    // 150 devs × 0.4 h/day × 220 days × £70/h = £924,000/year capacity value.
    let capacity = capacity_value_per_year(150.0, minutes / 60.0, 220.0, 70.0);
    assert!((capacity - 924_000.0).abs() < 1e-6);

    // Cross-check against the vendor's index pitch: 4 index points × 13 min/week
    // = 52 min/dev/week ≈ 10.4 min/day (5-day week) — under half the telemetry
    // estimate; the local measurement, not the vendor claim, feeds the case.
    let vendor_minutes_per_week = vendor_index_minutes_per_week(4.0, 13.0);
    assert!((vendor_minutes_per_week - 52.0).abs() < 1e-9);
    assert!(vendor_minutes_per_week / 5.0 < minutes);

    // Tornado on the case: usable-fraction range 0.2–0.8 dominates.
    let usable = OneWayResult {
        result_at_low: capacity_value_per_year(150.0, time_reclaimed_minutes_per_day(5.0, 12.0, 0.2) / 60.0, 220.0, 70.0),
        result_at_high: capacity_value_per_year(150.0, time_reclaimed_minutes_per_day(5.0, 12.0, 0.8) / 60.0, 220.0, 70.0),
    };
    // low = 462,000; high = 1,848,000; swing = 1,386,000.
    assert!((usable.swing() - 1_386_000.0).abs() < 1e-6);
    let rate = OneWayResult { result_at_low: 792_000.0, result_at_high: 1_056_000.0 }; // £60–£80/h
    let mut tornado = [("loaded rate", rate.swing()), ("usable fraction", usable.swing())];
    rank_by_swing(&mut tornado);
    assert_eq!(tornado[0].0, "usable fraction");
    assert!((tornado[1].1 - 264_000.0).abs() < 1e-6);

    // Platform TCO over 5 years: £400k build + £80k integration, maintenance at the
    // 18% benchmark (£72k/year), £20k decommission → £860k undiscounted.
    let maintenance = annual_maintenance_benchmark(400_000.0, 0.18);
    assert!((maintenance - 72_000.0).abs() < 1e-9);
    let platform = TcoProfile {
        initial_cost: 400_000.0,
        integration_and_training: 80_000.0,
        annual_run_cost: maintenance,
        horizon_years: 5,
        decommission_cost: 20_000.0,
    };
    assert!((platform.undiscounted_tco() - 860_000.0).abs() < 1e-6);
    assert!(platform.discounted_tco(0.035) < platform.undiscounted_tco());
    // Launch-cost anchoring check: build price is only ~47% of true TCO.
    let share = platform.initial_cost_share().unwrap();
    assert!((share - 400_000.0 / 860_000.0).abs() < 1e-12);

    // Horizon story: £480k up-front (build + integration), £924k/year benefit,
    // £72k/year running cost → break-even at 480/852 ≈ 0.563 years.
    let up_front = 480_000.0;
    let break_even = break_even_horizon_years(up_front, capacity, maintenance).unwrap();
    assert!((break_even - 480_000.0 / 852_000.0).abs() < 1e-12);
    let report = net_benefit_by_horizons(up_front, capacity, maintenance, &[1.0, 3.0, 5.0]);
    // Year 1: −480,000 + 852,000 = 372,000; year 3: 2,076,000; year 5: 3,780,000.
    assert!((report[0].1 - 372_000.0).abs() < 1e-6);
    assert!((report[1].1 - 2_076_000.0).abs() < 1e-6);
    assert!((report[2].1 - 3_780_000.0).abs() < 1e-6);
    // And the same story discounted: NPV at 3.5% over 5 years is positive but
    // below the undiscounted figure.
    let mut flows = vec![-up_front];
    flows.extend(std::iter::repeat_n(capacity - maintenance, 5));
    let npv = net_present_value(&flows, 0.035);
    assert!(npv > 0.0 && npv < 3_780_000.0);
}
