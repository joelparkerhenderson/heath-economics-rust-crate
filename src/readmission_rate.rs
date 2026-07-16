//! # Readmission Rate
//!
//! The 30-day readmission rate is the percentage of discharged patients who
//! return as an emergency within 30 days. It is the health system's
//! canonical *quality-of-discharge* metric — and it carries direct financial
//! penalties.
//!
//! A readmission means the first discharge didn't stick: premature
//! discharge, failed medication handoff, no follow-up, or missing social
//! support.
//!
//! ## Formula
//!
//! ```text
//! Readmission rate = emergency readmissions within 30 days / index discharges × 100
//!
//! Value of avoidance = avoided readmissions × (cost per readmission spell
//!                      + penalty exposure per readmission)
//! ```
//!
//! Legend:
//! - `index discharges` — the discharges being followed for 30 days.
//! - `emergency readmissions within 30 days` — unplanned returns among those
//!   discharges.
//! - `cost per readmission spell` — £ treatment cost of one readmission
//!   spell.
//! - `penalty exposure per readmission` — £ payment lost per readmission
//!   under penalty/non-payment rules.
//!
//! Risk-standardized comparisons adjust for case mix; penalty programs
//! compare observed vs expected for like hospitals.
//!
//! ## Why it matters
//!
//! Payers penalize readmissions explicitly — the US Hospital Readmissions
//! Reduction Program docks up to 3% of a hospital's Medicare payments; the
//! NHS has historically not paid for avoidable 30-day emergency
//! readmissions. So readmission avoidance is one of the few benefit
//! categories that is *directly* cash-relevant to a provider, not just
//! capacity.
//!
//! ## Example
//!
//! A heart-failure discharge-support app (symptom tracking, weight alerts,
//! medication reminders, nurse escalation): 2,000 discharges/year, baseline
//! readmission rate 18%, trial shows 14% with the app:
//!
//! ```rust
//! use health_economics::readmission_rate::{
//!     avoided_readmissions, net_benefit, program_cost, readmission_rate_percent,
//!     value_of_avoidance,
//! };
//!
//! // Baseline: 360 readmissions of 2,000 discharges = 18%.
//! let baseline = readmission_rate_percent(360.0, 2_000.0).unwrap();
//! assert_eq!(baseline, 18.0);
//!
//! // Avoided readmissions = 2,000 × (0.18 − 0.14) = 80/year.
//! let avoided = avoided_readmissions(2_000.0, 0.18, 0.14);
//! assert!((avoided - 80.0).abs() < 1e-9);
//!
//! // Cost per readmission spell ≈ £3,500 → £280,000/year avoided treatment cost.
//! let value = value_of_avoidance(avoided, 3_500.0, 0.0);
//! assert!((value - 280_000.0).abs() < 1e-3);
//!
//! // App cost: 2,000 × £60 = £120,000/year.
//! let cost = program_cost(2_000.0, 60.0);
//! assert_eq!(cost, 120_000.0);
//!
//! // Net ≈ +£160,000/year, before any QALY claim for avoided deterioration.
//! assert!((net_benefit(value, cost) - 160_000.0).abs() < 1e-3);
//! ```
//!
//! The number to defend is the 4-percentage-point effect: it must come from
//! a controlled comparison, because readmission rates swing with case mix
//! and season.
//!
//! ## Software engineering connection
//!
//! - Readmission is the health system's **change failure rate** (DORA): work
//!   that "shipped" and bounced back within 30 days.
//! - Reopened tickets and regression incidents indicate poor "discharge
//!   quality" — weak verification, premature closure, missing handoff docs.
//! - Penalty-style accounting (the fixing team pays, not the receiving team)
//!   changes behavior.
//! - Both fields learned the same lesson: pushing raw throughput (faster
//!   discharge, faster shipping) without investing in the handoff converts
//!   visible queues into invisible rework.
//! - A "30-day reopen rate" belongs on any team dashboard that celebrates
//!   cycle time.
//!
//! ## Pitfalls
//!
//! - **Gaming by re-labeling**: readmissions coded as observation stays or
//!   new conditions; audit the definition.
//! - **All-cause vs related-cause**: 30-day all-cause includes genuinely
//!   unrelated events; penalties usually use all-cause precisely because
//!   "related" is gameable.
//! - **Case-mix blindness**: a hospital serving sicker, poorer populations
//!   readmits more for reasons no app fixes — risk-adjust before comparing.
//!
//! ## Sources
//!
//! - CMS, Hospital Readmissions Reduction Program.
//!   <https://www.cms.gov/medicare/payment/prospective-payment-systems/acute-inpatient-pps/hospital-readmissions-reduction-program-hrrp>
//! - NHS Digital, emergency readmissions statistics.
//!   <https://digital.nhs.uk/data-and-information/publications/statistical/compendium-emergency-readmissions>
//!
//! Topic doc: health-economics-metrics/topics/readmission-rate.md

/// 30-day readmission rate as a percentage.
///
/// Emergency readmissions within 30 days ÷ index discharges × 100.
///
/// # Arguments
///
/// * `emergency_readmissions_within_30_days` — unplanned emergency returns
///   within 30 days of an index discharge.
/// * `index_discharges` — discharges being followed for 30 days.
///
/// # Returns
///
/// `Some(rate as a percentage, e.g. 18.0)`, or `None` when there are no
/// index discharges (the rate is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::readmission_rate_percent;
///
/// // 360 readmissions of 2,000 discharges → 18.0%.
/// assert_eq!(readmission_rate_percent(360.0, 2_000.0), Some(18.0));
///
/// // No discharges: undefined.
/// assert_eq!(readmission_rate_percent(10.0, 0.0), None);
/// ```
pub fn readmission_rate_percent(
    emergency_readmissions_within_30_days: f64,
    index_discharges: f64,
) -> Option<f64> {
    if index_discharges == 0.0 {
        None
    } else {
        Some(emergency_readmissions_within_30_days / index_discharges * 100.0)
    }
}

/// Observed-vs-expected ratio used by risk-standardized penalty comparisons.
///
/// Penalty programs (e.g. the US HRRP) compare a hospital's observed
/// readmissions against the count expected for its case mix; a ratio above 1
/// means more readmissions than expected for like hospitals.
///
/// # Arguments
///
/// * `observed` — actual readmission count.
/// * `expected` — risk-adjusted expected count for the hospital's case mix.
///
/// # Returns
///
/// `Some(observed / expected)`, or `None` when the expected count is zero
/// (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::observed_vs_expected;
///
/// // Readmitting exactly as often as expected → ratio 1.0.
/// assert_eq!(observed_vs_expected(360.0, 360.0), Some(1.0));
///
/// // Zero expected count: undefined.
/// assert_eq!(observed_vs_expected(10.0, 0.0), None);
/// ```
pub fn observed_vs_expected(observed: f64, expected: f64) -> Option<f64> {
    if expected == 0.0 {
        None
    } else {
        Some(observed / expected)
    }
}

/// Readmissions avoided per year: discharges × (baseline rate − new rate).
///
/// Rates are fractions (e.g. `0.18` for 18%), not percentages. The rate
/// difference must come from a controlled comparison — readmission rates
/// swing with case mix and season.
///
/// # Arguments
///
/// * `discharges` — index discharges per year covered by the intervention.
/// * `baseline_rate` — readmission rate without the intervention, as a
///   fraction.
/// * `new_rate` — readmission rate with the intervention, as a fraction.
///
/// # Returns
///
/// Expected readmissions avoided per year (negative if the new rate is
/// worse).
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::avoided_readmissions;
///
/// // 2,000 × (0.18 − 0.14) = 80 readmissions avoided per year.
/// assert!((avoided_readmissions(2_000.0, 0.18, 0.14) - 80.0).abs() < 1e-9);
/// ```
pub fn avoided_readmissions(
    discharges: f64,
    baseline_rate: f64,
    new_rate: f64,
) -> f64 {
    discharges * (baseline_rate - new_rate)
}

/// Value of avoided readmissions.
///
/// Avoided readmissions × (cost per readmission spell + penalty exposure per
/// readmission). Pass zero penalty exposure when no penalty/non-payment rule
/// applies.
///
/// # Arguments
///
/// * `avoided_readmissions` — readmissions avoided per year (see
///   [`avoided_readmissions`]).
/// * `cost_per_readmission_spell` — £ treatment cost per readmission spell
///   (worked example: ≈ £3,500).
/// * `penalty_exposure_per_readmission` — £ payment lost per readmission
///   under penalty rules (0 when not applicable).
///
/// # Returns
///
/// Annual value of avoidance in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::value_of_avoidance;
///
/// // 80 avoided spells × £3,500 = £280,000/year (no penalty term).
/// assert_eq!(value_of_avoidance(80.0, 3_500.0, 0.0), 280_000.0);
///
/// // With £500/readmission penalty exposure: £320,000/year.
/// assert_eq!(value_of_avoidance(80.0, 3_500.0, 500.0), 320_000.0);
/// ```
pub fn value_of_avoidance(
    avoided_readmissions: f64,
    cost_per_readmission_spell: f64,
    penalty_exposure_per_readmission: f64,
) -> f64 {
    avoided_readmissions * (cost_per_readmission_spell + penalty_exposure_per_readmission)
}

/// Annual program cost: discharges covered × cost per discharge.
///
/// E.g. an app licence per discharged patient.
///
/// # Arguments
///
/// * `discharges` — discharges covered per year.
/// * `cost_per_discharge` — £ per covered discharge (worked example: £60 app
///   licence).
///
/// # Returns
///
/// Annual program cost in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::program_cost;
///
/// // 2,000 discharges × £60 = £120,000/year.
/// assert_eq!(program_cost(2_000.0, 60.0), 120_000.0);
/// ```
pub fn program_cost(discharges: f64, cost_per_discharge: f64) -> f64 {
    discharges * cost_per_discharge
}

/// Net annual benefit: value of avoidance minus program cost.
///
/// Positive means the program pays for itself on cash terms alone, before
/// any QALY claim for avoided deterioration.
///
/// # Arguments
///
/// * `value_of_avoidance` — annual £ value of avoided readmissions (see
///   [`value_of_avoidance`]).
/// * `program_cost` — annual £ program cost (see [`program_cost`]).
///
/// # Returns
///
/// Net annual benefit in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::readmission_rate::net_benefit;
///
/// // £280,000 avoided − £120,000 app cost ≈ +£160,000/year.
/// assert_eq!(net_benefit(280_000.0, 120_000.0), 160_000.0);
/// ```
pub fn net_benefit(value_of_avoidance: f64, program_cost: f64) -> f64 {
    value_of_avoidance - program_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Baseline: 18% of 2,000 discharges readmit — 360 spells → 18.0%.
    #[test]
    fn baseline_rate_is_18_percent() {
        // Worked example: "2,000 discharges/year, baseline readmission rate 18%."
        let r = readmission_rate_percent(360.0, 2_000.0).unwrap();
        assert!((r - 18.0).abs() < TOL);
    }

    /// Avoided readmissions = 2,000 × (0.18 − 0.14) = 80/year.
    #[test]
    fn app_avoids_80_readmissions_per_year() {
        // Worked example: "Avoided readmissions = 2,000 × (0.18 − 0.14) = 80/year."
        let a = avoided_readmissions(2_000.0, 0.18, 0.14);
        assert!((a - 80.0).abs() < TOL);
    }

    /// 80 avoided spells × £3,500 ≈ £280,000/year avoided treatment cost.
    #[test]
    fn avoided_treatment_cost_is_280_000_per_year() {
        // Worked example: "Cost per readmission spell ≈ £3,500 →
        // £280,000/year avoided treatment cost."
        let v = value_of_avoidance(80.0, 3_500.0, 0.0);
        assert!((v - 280_000.0).abs() < TOL);
    }

    /// App cost: 2,000 discharges × £60 = £120,000/year.
    #[test]
    fn app_cost_is_120_000_per_year() {
        // Worked example: "App cost: 2,000 × £60 = £120,000/year."
        let c = program_cost(2_000.0, 60.0);
        assert!((c - 120_000.0).abs() < TOL);
    }

    /// Net ≈ +£160,000/year, before any QALY claim.
    #[test]
    fn net_benefit_is_160_000_per_year() {
        // Worked example: "Net ≈ +£160,000/year, before any QALY claim for
        // avoided deterioration."
        let n = net_benefit(280_000.0, 120_000.0);
        assert!((n - 160_000.0).abs() < TOL);
    }

    /// Penalty exposure adds to the per-spell value when present.
    #[test]
    fn penalty_exposure_adds_to_avoidance_value() {
        // Doc math: "Value of avoidance = avoided readmissions × (cost per
        // readmission spell + penalty exposure per readmission)."
        let v = value_of_avoidance(80.0, 3_500.0, 500.0);
        assert!((v - 320_000.0).abs() < TOL);
    }

    /// A hospital readmitting as often as expected has an O/E ratio of 1.
    #[test]
    fn observed_equal_to_expected_gives_ratio_one() {
        // Doc math: "penalty programs compare observed vs expected for like hospitals."
        let r = observed_vs_expected(360.0, 360.0).unwrap();
        assert!((r - 1.0).abs() < TOL);
    }

    // Edge case: no index discharges leaves the rate undefined.
    #[test]
    fn zero_discharges_has_no_defined_rate() {
        assert!(readmission_rate_percent(10.0, 0.0).is_none());
    }

    // Edge case: a zero expected count leaves the O/E ratio undefined.
    #[test]
    fn zero_expected_has_no_defined_ratio() {
        assert!(observed_vs_expected(10.0, 0.0).is_none());
    }
}
