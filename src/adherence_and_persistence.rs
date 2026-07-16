//! # Adherence and Persistence
//!
//! Adherence is how closely actual use matches prescribed use (intensity);
//! persistence is how long use continues before discontinuation (duration).
//! Pharmacy has standardized measures — MPR and PDC, with ≥80% the
//! conventional "adherent" bar — and digital therapeutics inherit both the
//! concepts and the problem: adherence is the multiplier between efficacy and
//! realized value.
//!
//! ## Formula
//!
//! ```text
//! MPR = Σ days' supply dispensed / days in period × 100   (can exceed 100%;
//!       overestimates via early refills)
//! PDC = days covered by supply / days in period × 100     (capped at 100%;
//!       the conservative, CMS-preferred estimator)
//! Digital adherence = actual usage events / prescribed usage events × 100
//! Persistence       = days from initiation to discontinuation
//!                     (report % persistent at N months; survival methods)
//!
//! Value gating: realized outcome ≈ efficacy × g(adherence)
//!   where g is the dose-response function; below the minimum effective
//!   dose, g ≈ 0 — cost incurred, benefit forfeited
//! ```
//!
//! Legend:
//! - `MPR` — medication possession ratio (percent).
//! - `PDC` — proportion of days covered (percent, capped at 100).
//! - `days' supply dispensed` / `days covered` — days of medication supplied
//!   or covered in the measurement period (days).
//! - `usage events` — protocol units for digital therapeutics (sessions,
//!   modules, doses).
//! - `g(adherence)` — the empirically established dose-response function.
//!
//! ## Why it matters
//!
//! Payers already run on these numbers: PDC ≥80% feeds US Medicare Star
//! Ratings, which move real payer revenue — adherence is financially
//! load-bearing infrastructure, not a soft metric. For digital therapeutics
//! the pattern repeats: DiGA data shows strong prescription volumes with weak
//! sustained adherence, and outcomes-based DTx pricing (arriving in Germany
//! from 2026) will pay on adherence-gated results. The conceptual upgrade
//! from digital health research: **effective engagement** — *sufficient*
//! engagement to achieve the intended outcome — and its corollary, the
//! **minimum effective dose**, established empirically per intervention
//! rather than assumed to be "more."
//!
//! ## Example
//!
//! A digital CBT-for-insomnia product, prescribed as 6 modules over 6 weeks;
//! trial efficacy 0.025 QALYs among those completing ≥4 modules (the
//! empirically established minimum effective dose). 1,000 prescriptions at
//! £250 → £250,000 payer spend; 38% reach ≥4 modules → 9.5 QALYs at
//! ≈£26,300/QALY; adherence engineering lifting completion to 50% yields
//! 12.5 QALYs at £20,000/QALY — crossing the funding threshold without
//! touching the therapy content.
//!
//! ```rust
//! use health_economics::adherence_and_persistence::{
//!     cost_per_qaly, payer_spend, qalys_realized,
//! };
//!
//! // 1,000 prescriptions at £250 → £250,000 payer spend.
//! let spend = payer_spend(1_000.0, 250.0);
//! assert!((spend - 250_000.0).abs() < 1e-9);
//!
//! // QALYs realized = 1,000 × 0.38 × 0.025 = 9.5.
//! let qalys = qalys_realized(1_000.0, 0.38, 0.025);
//! assert!((qalys - 9.5).abs() < 1e-9);
//!
//! // Cost per QALY = 250,000 / 9.5 ≈ £26,300 — marginal at NICE thresholds.
//! let icer = cost_per_qaly(spend, qalys).unwrap();
//! assert!((icer - 26_300.0).abs() < 50.0);
//!
//! // Adherence engineering lifts ≥4-module completion to 50%:
//! // 12.5 QALYs → £20,000/QALY.
//! let lifted_qalys = qalys_realized(1_000.0, 0.50, 0.025);
//! assert!((lifted_qalys - 12.5).abs() < 1e-9);
//! assert!((cost_per_qaly(spend, lifted_qalys).unwrap() - 20_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Two vocabularies converge on one concept: software analytics
//!   (activation, stickiness, retention) and clinical pharmacy (MPR, PDC,
//!   persistence) both measure exposure to an intervention — map product
//!   events onto the clinical vocabulary and payers can read your dashboards.
//! - Engineering owns the adherence levers: reminder logic (dumb daily pings
//!   train dismissal; adaptive timing doesn't).
//! - Session cost matters: a 20-minute module completes less than
//!   3 × 7-minute ones.
//! - Friction telemetry locates *where* in the protocol users fall off.
//! - Instrument dose-response from day one — the minimum-effective-dose
//!   analysis that gates the whole economic model needs
//!   usage-linked-to-outcome data only the product can collect.
//!
//! ## Pitfalls
//!
//! - **MPR/PDC conflation**: MPR inflates; state which estimator and use PDC
//!   for anything payer-facing.
//! - **Adherence to the metric, not the therapy**: opens counted as doses.
//! - **"More is better" engagement targets** where the intervention has a
//!   finite dose — graduation is success, perpetual use is not.
//! - **Survivor-based efficacy claims**: outcomes among the adherent include
//!   selection effects (adherent people differ); the honest causal estimate
//!   needs randomization or careful adjustment.
//!
//! ## Sources
//!
//! - MPR vs PDC. <https://phslrx.com/medication-adherence-metrics/>
//! - Yardley L, et al., effective engagement.
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC8726056/>
//! - DiGA adherence findings, npj Digital Medicine 2024.
//!   <https://www.nature.com/articles/s41746-024-01137-1>
//!
//! Topic doc: health-economics-metrics/topics/adherence-and-persistence.md

/// Medication possession ratio (MPR) as a percentage.
///
/// Total days' supply dispensed divided by days in the period, times 100.
/// MPR can exceed 100% and overestimates adherence via early refills — use
/// [`pdc_percent`] for anything payer-facing.
///
/// # Arguments
///
/// * `total_days_supply_dispensed` — sum of days' supply across all fills in
///   the period (days).
/// * `days_in_period` — length of the measurement period (days).
///
/// # Returns
///
/// The MPR in percent (uncapped, may exceed 100), or `None` if
/// `days_in_period` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::mpr_percent;
///
/// // 400 days of supply dispensed over a 365-day period: MPR > 100%.
/// let mpr = mpr_percent(400.0, 365.0).unwrap();
/// assert!(mpr > 100.0);
///
/// assert!(mpr_percent(400.0, 0.0).is_none());
/// ```
pub fn mpr_percent(total_days_supply_dispensed: f64, days_in_period: f64) -> Option<f64> {
    if days_in_period == 0.0 {
        None
    } else {
        Some(total_days_supply_dispensed / days_in_period * 100.0)
    }
}

/// Proportion of days covered (PDC) as a percentage, capped at 100%.
///
/// Days covered by supply divided by days in the period, times 100, capped at
/// 100%. The conservative, CMS-preferred estimator; PDC ≥80% is the
/// conventional "adherent" bar that feeds US Medicare Star Ratings.
///
/// # Arguments
///
/// * `days_covered` — days in the period on which the patient had supply
///   available (days).
/// * `days_in_period` — length of the measurement period (days).
///
/// # Returns
///
/// The PDC in percent (0–100), or `None` if `days_in_period` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::pdc_percent;
///
/// // Even with 400 days of nominal coverage in a 365-day period, PDC caps at 100%.
/// let pdc = pdc_percent(400.0, 365.0).unwrap();
/// assert!((pdc - 100.0).abs() < 1e-9);
///
/// // 292 covered days of 365 → 80%: exactly the conventional adherent bar.
/// let pdc = pdc_percent(292.0, 365.0).unwrap();
/// assert!((pdc - 80.0).abs() < 1e-9);
/// ```
pub fn pdc_percent(days_covered: f64, days_in_period: f64) -> Option<f64> {
    if days_in_period == 0.0 {
        None
    } else {
        // Cap at 100%: overlapping fills cannot cover a day twice.
        Some((days_covered / days_in_period * 100.0).min(100.0))
    }
}

/// Digital adherence as a percentage of the prescribed protocol.
///
/// Actual usage events divided by prescribed usage events, times 100. Usage
/// events are protocol units (sessions, modules, doses) — count therapeutic
/// actions, not app opens.
///
/// # Arguments
///
/// * `actual_usage_events` — protocol units actually completed (count).
/// * `prescribed_usage_events` — protocol units prescribed (count).
///
/// # Returns
///
/// The digital adherence in percent, or `None` if `prescribed_usage_events`
/// is zero (nothing was prescribed).
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::digital_adherence_percent;
///
/// // Worked example protocol: 4 of 6 prescribed CBT modules completed ≈ 66.7%.
/// let adherence = digital_adherence_percent(4.0, 6.0).unwrap();
/// assert!((adherence - 400.0 / 6.0).abs() < 1e-9);
///
/// assert!(digital_adherence_percent(4.0, 0.0).is_none());
/// ```
pub fn digital_adherence_percent(
    actual_usage_events: f64,
    prescribed_usage_events: f64,
) -> Option<f64> {
    if prescribed_usage_events == 0.0 {
        None
    } else {
        Some(actual_usage_events / prescribed_usage_events * 100.0)
    }
}

/// Persistence: days from initiation to discontinuation.
///
/// Duration of continued use. Report the cohort form as % persistent at
/// N months (see [`percent_persistent`]) using survival methods. Both
/// arguments are day indices on the same timeline.
///
/// # Arguments
///
/// * `initiation_day` — day index of therapy initiation.
/// * `discontinuation_day` — day index of discontinuation.
///
/// # Returns
///
/// Days persisted (negative if the arguments are reversed — callers should
/// pass `discontinuation_day >= initiation_day`).
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::persistence_days;
///
/// // A 6-week (42-day) programme used from day 0 to day 42.
/// assert!((persistence_days(0.0, 42.0) - 42.0).abs() < 1e-9);
/// ```
pub fn persistence_days(initiation_day: f64, discontinuation_day: f64) -> f64 {
    discontinuation_day - initiation_day
}

/// Share of a cohort still persistent at a follow-up point, as a percentage.
///
/// The cohort-level persistence summary ("% persistent at N months").
///
/// # Arguments
///
/// * `still_persistent` — cohort members who have not discontinued at the
///   follow-up point (count).
/// * `cohort_size` — total cohort at initiation (count).
///
/// # Returns
///
/// Percent persistent, or `None` for an empty cohort (`cohort_size` zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::percent_persistent;
///
/// // 380 of 1,000 prescriptions still in use at 6 weeks → 38% persistent.
/// let pct = percent_persistent(380.0, 1_000.0).unwrap();
/// assert!((pct - 38.0).abs() < 1e-9);
///
/// assert!(percent_persistent(380.0, 0.0).is_none());
/// ```
pub fn percent_persistent(still_persistent: f64, cohort_size: f64) -> Option<f64> {
    if cohort_size == 0.0 {
        None
    } else {
        Some(still_persistent / cohort_size * 100.0)
    }
}

/// Whether an adherence percentage meets the conventional ≥80% "adherent" bar.
///
/// Applies to PDC (or MPR) percentages; PDC ≥80% is the threshold that feeds
/// US Medicare Star Ratings. The comparison is inclusive: exactly 80.0 counts
/// as adherent.
///
/// # Arguments
///
/// * `adherence_percent` — a PDC or MPR value (percent).
///
/// # Returns
///
/// `true` if `adherence_percent >= 80.0`.
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::is_adherent;
///
/// assert!(is_adherent(80.0));
/// assert!(!is_adherent(79.9));
/// ```
pub fn is_adherent(adherence_percent: f64) -> bool {
    adherence_percent >= 80.0
}

/// Total payer spend: prescriptions times price per prescription.
///
/// # Arguments
///
/// * `prescriptions` — prescriptions issued and paid (count).
/// * `price_per_prescription` — payer price per prescription (currency).
///
/// # Returns
///
/// Total payer spend (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::payer_spend;
///
/// // Worked example: 1,000 prescriptions at £250 → £250,000 payer spend.
/// let spend = payer_spend(1_000.0, 250.0);
/// assert!((spend - 250_000.0).abs() < 1e-9);
/// ```
pub fn payer_spend(prescriptions: f64, price_per_prescription: f64) -> f64 {
    prescriptions * price_per_prescription
}

/// QALYs realized under value gating by the minimum effective dose.
///
/// Only the fraction of the prescribed population reaching the minimum
/// effective dose realizes the trial efficacy — below the dose, the
/// dose-response function g ≈ 0: cost incurred, benefit forfeited.
///
/// # Arguments
///
/// * `prescriptions` — prescriptions issued (count).
/// * `fraction_reaching_minimum_effective_dose` — share of prescribed
///   patients reaching the empirically established minimum effective dose
///   (0.0–1.0).
/// * `qalys_per_effectively_dosed_patient` — trial efficacy among those at or
///   above the dose (QALYs per patient).
///
/// # Returns
///
/// Total QALYs realized across the prescribed population.
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::qalys_realized;
///
/// // Worked example: 1,000 × 0.38 × 0.025 = 9.5 QALYs.
/// let qalys = qalys_realized(1_000.0, 0.38, 0.025);
/// assert!((qalys - 9.5).abs() < 1e-9);
/// ```
pub fn qalys_realized(
    prescriptions: f64,
    fraction_reaching_minimum_effective_dose: f64,
    qalys_per_effectively_dosed_patient: f64,
) -> f64 {
    // Value gating: efficacy accrues only to the effectively dosed fraction.
    prescriptions * fraction_reaching_minimum_effective_dose * qalys_per_effectively_dosed_patient
}

/// Cost per QALY: total spend divided by QALYs realized.
///
/// The adherence-gated cost-effectiveness ratio; compare against
/// willingness-to-pay thresholds (e.g. NICE's £20,000–£30,000/QALY).
///
/// # Arguments
///
/// * `total_spend` — total payer spend (currency).
/// * `qalys` — QALYs realized (QALYs).
///
/// # Returns
///
/// Cost per QALY (currency/QALY), or `None` if `qalys` is zero (no benefit
/// realized — the ratio is undefined/infinite).
///
/// # Examples
///
/// ```rust
/// use health_economics::adherence_and_persistence::cost_per_qaly;
///
/// // Worked example: 250,000 / 9.5 ≈ £26,300 — marginal at NICE thresholds.
/// let icer = cost_per_qaly(250_000.0, 9.5).unwrap();
/// assert!((icer - 26_300.0).abs() < 50.0);
///
/// assert!(cost_per_qaly(250_000.0, 0.0).is_none());
/// ```
pub fn cost_per_qaly(total_spend: f64, qalys: f64) -> Option<f64> {
    if qalys == 0.0 {
        None
    } else {
        Some(total_spend / qalys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: digital CBT-for-insomnia, 6 modules over 6 weeks,
    // trial efficacy 0.025 QALYs among those completing ≥4 modules.

    #[test]
    fn payer_spend_is_250_000() {
        // 1,000 prescriptions at £250 → £250,000.
        let got = payer_spend(1_000.0, 250.0);
        assert!((got - 250_000.0).abs() < 1e-9);
    }

    #[test]
    fn qalys_realized_at_38_percent_completion_is_9_5() {
        // 1,000 × 0.38 × 0.025 = 9.5 QALYs.
        let got = qalys_realized(1_000.0, 0.38, 0.025);
        assert!((got - 9.5).abs() < 1e-9);
    }

    #[test]
    fn cost_per_qaly_at_38_percent_is_about_26_300() {
        // 250,000 / 9.5 ≈ £26,300.
        let got = cost_per_qaly(250_000.0, 9.5).unwrap();
        assert!((got - 26_300.0).abs() < 50.0);
    }

    #[test]
    fn adherence_engineering_lifts_qalys_to_12_5() {
        // ≥4-module completion lifted to 50%: 12.5 QALYs.
        let got = qalys_realized(1_000.0, 0.50, 0.025);
        assert!((got - 12.5).abs() < 1e-9);
    }

    #[test]
    fn cost_per_qaly_at_50_percent_is_20_000() {
        // 250,000 / 12.5 = £20,000/QALY.
        let got = cost_per_qaly(250_000.0, 12.5).unwrap();
        assert!((got - 20_000.0).abs() < 1e-9);
    }

    #[test]
    fn pdc_is_capped_at_100_percent_but_mpr_is_not() {
        // MPR can exceed 100% via early refills; PDC cannot (doc: "The math").
        let mpr = mpr_percent(400.0, 365.0).unwrap();
        let pdc = pdc_percent(400.0, 365.0).unwrap();
        assert!(mpr > 100.0);
        assert!((pdc - 100.0).abs() < 1e-9);
    }

    #[test]
    fn eighty_percent_bar_marks_adherent() {
        // Doc: "≥80% the conventional 'adherent' bar" — inclusive at 80.0.
        assert!(is_adherent(80.0));
        assert!(!is_adherent(79.9));
    }

    #[test]
    fn digital_adherence_two_thirds_of_modules() {
        // 4 of 6 prescribed modules completed (the ≥4-module minimum
        // effective dose against the 6-module prescription).
        let got = digital_adherence_percent(4.0, 6.0).unwrap();
        assert!((got - 400.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn persistence_is_days_from_initiation_to_discontinuation() {
        // The 6-week (42-day) prescribed course, persisted in full.
        assert!((persistence_days(0.0, 42.0) - 42.0).abs() < 1e-9);
    }

    #[test]
    fn zero_denominators_return_none() {
        // Edge-case semantics: every ratio is undefined at a zero denominator.
        assert!(mpr_percent(30.0, 0.0).is_none());
        assert!(pdc_percent(30.0, 0.0).is_none());
        assert!(digital_adherence_percent(1.0, 0.0).is_none());
        assert!(percent_persistent(1.0, 0.0).is_none());
        assert!(cost_per_qaly(1.0, 0.0).is_none());
    }
}
