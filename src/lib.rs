//! # Health Economics
//!
//! Rust implementations of 76 health-economics metrics and their software
//! engineering analogues — one module per topic. Each module implements the
//! formulas from its source topic document (`health-economics-metrics/topics/*.md`),
//! documents them with runnable examples, and reproduces the document's
//! worked example in its unit tests.
//!
//! The crate is `std`-only: no external dependencies, all quantities are
//! `f64`, and functions return `Option<f64>` wherever a denominator can be
//! zero. Randomness (for probabilistic sensitivity analysis) uses a seeded,
//! deterministic generator so results are reproducible.
//!
//! ## Quickstart
//!
//! The canonical chain — health gain, monetized, compared with cost, decided:
//!
//! ```
//! use health_economics::quality_adjusted_life_year as qaly;
//! use health_economics::incremental_cost_effectiveness_ratio as icer;
//! use health_economics::net_monetary_benefit as nmb;
//!
//! // Software removes a 6-month wait (utility 0.6 → 0.85) for 400 patients.
//! let per_patient = qaly::qaly_loss_from_delay(0.5, 0.6, 0.85);
//! let total = qaly::population_qalys(per_patient, 400.0);
//! assert_eq!(total, 50.0);
//!
//! // At £500k/year the software buys QALYs at £10k — under the £20k threshold.
//! let ratio = icer::icer(500_000.0, total).unwrap();
//! assert!(icer::adopt_at_threshold(ratio, 20_000.0));
//!
//! // Net monetary benefit says the same thing in money: +£500k/year.
//! let benefit = nmb::net_monetary_benefit(total, 500_000.0, 20_000.0);
//! assert!(nmb::adopt(benefit));
//! ```
//!
//! ## Learning path
//!
//! Long-form guides live in the [`tutorials`] module, and runnable programs
//! in `examples/` (`cargo run --example qaly_to_decision`).
//!
//! ## Module index by theme
//!
//! **Health outcome measures** —
//! [`quality_adjusted_life_year`], [`eq_5d`], [`disability_adjusted_life_year`],
//! [`life_years_gained`], [`health_adjusted_life_expectancy`],
//! [`patient_reported_outcomes`], [`number_needed_to_treat`]
//!
//! **Economic evaluation frameworks** —
//! [`cost_effectiveness_analysis`], [`cost_utility_analysis`],
//! [`cost_benefit_analysis`], [`cost_minimization_analysis`],
//! [`cost_consequence_analysis`], [`budget_impact_analysis`],
//! [`social_return_on_investment`], [`health_technology_assessment`]
//!
//! **Decision rules and thresholds** —
//! [`incremental_cost_effectiveness_ratio`], [`net_monetary_benefit`],
//! [`willingness_to_pay_thresholds`], [`dominance_and_efficiency_frontier`],
//! [`qaly_shortfall_and_severity_modifiers`], [`opportunity_cost`],
//! [`analysis_perspective`], [`time_horizon`], [`discounting_and_time_preference`],
//! [`marginal_vs_average_cost`]
//!
//! **Uncertainty and evidence** —
//! [`sensitivity_analysis`], [`probabilistic_sensitivity_analysis`],
//! [`expected_value_of_perfect_information`], [`benefits_realization`]
//!
//! **Healthcare operations** —
//! [`bed_days_saved`], [`length_of_stay`], [`readmission_rate`],
//! [`referral_to_treatment`], [`waiting_list_impact`], [`did_not_attend_rate`],
//! [`emergency_attendance_avoidance`], [`practitioner_time`],
//! [`national_tariff_and_unit_costs`], [`avoidable_outsourcing_costs`],
//! [`avoided_downstream_costs`], [`downstream_resource_optimization`],
//! [`earlier_intervention`], [`prevention_economics`], [`screening_economics`],
//! [`workforce_retention`], [`cash_releasing_vs_non_cash_releasing`],
//! [`hard_cash_releasing_savings_deficit_defense`],
//! [`value_generating_capacity_operational_turnaround`]
//!
//! **Digital health products** —
//! [`activation_and_uptake`], [`adherence_and_persistence`],
//! [`engagement_metrics`], [`retention_and_churn`], [`reach_and_equity`],
//! [`health_app_unit_economics`], [`remote_patient_monitoring_economics`],
//! [`digital_endpoints_and_biomarkers`], [`wearable_validation`],
//! [`diga_fast_track`], [`nice_evidence_standards_framework`],
//! [`gds_service_metrics`]
//!
//! **AI evaluation and economics** —
//! [`clinical_ai_evaluation`], [`ai_quality_metrics`],
//! [`ai_regulatory_evaluation`], [`ai_return_on_investment`],
//! [`ai_developer_productivity`], [`inference_unit_economics`]
//!
//! **Software engineering economics** —
//! [`dora_metrics`], [`flow_metrics`], [`space_and_devex`], [`technical_debt`],
//! [`cost_of_delay`], [`wsjf_and_cd3`], [`return_on_investment`],
//! [`total_cost_of_ownership`], [`build_vs_buy`], [`cloud_unit_economics`]

/// Long-form tutorials rendered into rustdoc. Each walks a complete
/// analysis with runnable, doctested code.
pub mod tutorials {
    #![doc = include_str!("../docs/tutorials/README.md")]

    #[doc = include_str!("../docs/tutorials/01-from-waiting-list-to-business-case.md")]
    pub mod from_waiting_list_to_business_case {}

    #[doc = include_str!("../docs/tutorials/02-building-the-financial-case.md")]
    pub mod building_the_financial_case {}

    #[doc = include_str!("../docs/tutorials/03-quantifying-uncertainty.md")]
    pub mod quantifying_uncertainty {}

    #[doc = include_str!("../docs/tutorials/04-the-engineering-mirror.md")]
    pub mod the_engineering_mirror {}
}

pub mod activation_and_uptake;
pub mod adherence_and_persistence;
pub mod ai_developer_productivity;
pub mod ai_quality_metrics;
pub mod ai_regulatory_evaluation;
pub mod ai_return_on_investment;
pub mod analysis_perspective;
pub mod avoidable_outsourcing_costs;
pub mod avoided_downstream_costs;
pub mod bed_days_saved;
pub mod benefits_realization;
pub mod budget_impact_analysis;
pub mod build_vs_buy;
pub mod cash_releasing_vs_non_cash_releasing;
pub mod clinical_ai_evaluation;
pub mod cloud_unit_economics;
pub mod cost_benefit_analysis;
pub mod cost_consequence_analysis;
pub mod cost_effectiveness_analysis;
pub mod cost_minimization_analysis;
pub mod cost_of_delay;
pub mod cost_utility_analysis;
pub mod did_not_attend_rate;
pub mod diga_fast_track;
pub mod digital_endpoints_and_biomarkers;
pub mod disability_adjusted_life_year;
pub mod discounting_and_time_preference;
pub mod dominance_and_efficiency_frontier;
pub mod dora_metrics;
pub mod downstream_resource_optimization;
pub mod earlier_intervention;
pub mod emergency_attendance_avoidance;
pub mod engagement_metrics;
pub mod eq_5d;
pub mod expected_value_of_perfect_information;
pub mod flow_metrics;
pub mod gds_service_metrics;
pub mod hard_cash_releasing_savings_deficit_defense;
pub mod health_adjusted_life_expectancy;
pub mod health_app_unit_economics;
pub mod health_technology_assessment;
pub mod incremental_cost_effectiveness_ratio;
pub mod inference_unit_economics;
pub mod length_of_stay;
pub mod life_years_gained;
pub mod marginal_vs_average_cost;
pub mod national_tariff_and_unit_costs;
pub mod net_monetary_benefit;
pub mod nice_evidence_standards_framework;
pub mod number_needed_to_treat;
pub mod opportunity_cost;
pub mod patient_reported_outcomes;
pub mod practitioner_time;
pub mod prevention_economics;
pub mod probabilistic_sensitivity_analysis;
pub mod qaly_shortfall_and_severity_modifiers;
pub mod quality_adjusted_life_year;
pub mod reach_and_equity;
pub mod readmission_rate;
pub mod referral_to_treatment;
pub mod remote_patient_monitoring_economics;
pub mod retention_and_churn;
pub mod return_on_investment;
pub mod screening_economics;
pub mod sensitivity_analysis;
pub mod social_return_on_investment;
pub mod space_and_devex;
pub mod technical_debt;
pub mod time_horizon;
pub mod total_cost_of_ownership;
pub mod value_generating_capacity_operational_turnaround;
pub mod waiting_list_impact;
pub mod wearable_validation;
pub mod willingness_to_pay_thresholds;
pub mod workforce_retention;
pub mod wsjf_and_cd3;
