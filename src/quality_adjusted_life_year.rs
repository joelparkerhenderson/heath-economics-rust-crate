//! # Quality-Adjusted Life Year (QALY)
//!
//! A QALY is one year of life lived in perfect health. It combines *how
//! long* people live with *how well* they live, so that a year in poor
//! health counts as less than one QALY — making utterly different health
//! interventions comparable on a single scale.
//!
//! Utility weights come from validated instruments, most commonly EQ-5D.
//! QALY *gain* from an intervention is the difference between the QALY
//! streams with and without it, discounted at 3.5%/year in the NICE
//! reference case.
//!
//! ## Formula
//!
//! ```text
//! QALYs = Σ_i (duration_i × utility_i)
//! ```
//!
//! Legend:
//! - `duration_i` — years spent in health state `i`.
//! - `utility_i` — quality weight of state `i`, anchored at 1 = perfect
//!   health, 0 = dead (negative values allowed for states worse than death).
//!
//! ## Why it matters
//!
//! The QALY is the common currency of health technology assessment. NICE
//! (England) values health gains at **£20,000–£30,000 per QALY**: an
//! intervention that buys QALYs cheaper than that threshold is normally
//! recommended; one that buys them dearer is normally rejected. This one
//! number is how a national health service compares a cancer drug, a hip
//! replacement, and a triage app on the same axis. If your software can
//! credibly claim QALYs — by preventing deterioration, accelerating
//! treatment, or improving safety — you can price its health value in the
//! same currency as medicine itself.
//!
//! ## Example
//!
//! A patient waits for cardiac treatment in a state with utility 0.6.
//! Treatment restores them to utility 0.85:
//!
//! ```rust
//! use health_economics::quality_adjusted_life_year::{
//!     monetized_value, population_qalys, qaly_gain, qalys, HealthState,
//! };
//!
//! // Treated now: 1 year at 0.85 = 0.85 QALYs this year.
//! let now = qalys(&[HealthState { duration_years: 1.0, utility: 0.85 }]);
//! assert!((now - 0.85).abs() < 1e-9);
//!
//! // Treated after a 6-month delay: 0.5 × 0.6 + 0.5 × 0.85 = 0.725 QALYs.
//! let delayed = qalys(&[
//!     HealthState { duration_years: 0.5, utility: 0.6 },
//!     HealthState { duration_years: 0.5, utility: 0.85 },
//! ]);
//! assert!((delayed - 0.725).abs() < 1e-9);
//!
//! // QALY loss per patient from the delay: 0.85 − 0.725 = 0.125 QALYs.
//! let loss = qaly_gain(now, delayed);
//! assert!((loss - 0.125).abs() < 1e-9);
//!
//! // Monetized: 0.125 × £20,000–£30,000 = £2,500–£3,750 per patient per delay.
//! assert!((monetized_value(loss, 20_000.0) - 2_500.0).abs() < 1e-9);
//! assert!((monetized_value(loss, 30_000.0) - 3_750.0).abs() < 1e-9);
//!
//! // Removing the delay for 400 patients/year: 50 QALYs ≈ £1.0–£1.5M/year.
//! let cohort = population_qalys(loss, 400.0);
//! assert!((cohort - 50.0).abs() < 1e-9);
//! assert!((monetized_value(cohort, 20_000.0) - 1_000_000.0).abs() < 1e-9);
//! assert!((monetized_value(cohort, 30_000.0) - 1_500_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Faster pathways = earlier QALYs.** Anything that shortens referral to
//!   treatment converts waiting-time disutility into health gain, valued as
//!   above.
//! - **Safety = QALYs preserved.** Prevented medication errors and missed
//!   diagnoses are QALY losses avoided.
//! - **The QALY is a metric-design template**: a composite of quantity ×
//!   quality, with quality weights elicited from a standardized instrument —
//!   a "quality-adjusted engineer year" (time × DevEx-survey weight) is the
//!   same construction.
//! - To turn QALYs into money for a business case, use net monetary benefit;
//!   to turn them into a decision, use willingness-to-pay thresholds.
//!
//! ## Pitfalls
//!
//! - **Inventing utility weights.** Weights must come from validated
//!   instruments (EQ-5D) and published value sets, not intuition.
//! - **Claiming QALYs without a causal pathway.** "Our app improves
//!   wellbeing" is not a QALY claim; "removes X weeks of waiting in state
//!   utility 0.6" is.
//! - **Double counting**: claiming both the QALY gain and the cost savings of
//!   the same avoided deterioration requires care that they are genuinely
//!   separate.
//! - **Equity blind spots**: QALYs value a year of life extension by baseline
//!   utility, which can disadvantage people with disabilities — the reason
//!   ICER (US) also reports the evLYG.
//!
//! ## Sources
//!
//! - NICE glossary: QALY. <https://www.nice.org.uk/glossary?letter=q>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//! - ICER, "Cost-Effectiveness, the QALY, and the evLYG."
//!   <https://icer.org/our-approach/methods-process/cost-effectiveness-the-qaly-and-the-evlyg/>
//!
//! Topic doc: health-economics-metrics/topics/quality-adjusted-life-year.md

/// A health state: a duration in years spent at a given utility weight.
///
/// Utilities are anchored at 1 = perfect health and 0 = dead; negative
/// values are allowed for states considered worse than death. Weights should
/// come from validated instruments (EQ-5D) and published value sets, not
/// intuition.
pub struct HealthState {
    /// Years spent in this state (e.g. `0.5` for six months).
    pub duration_years: f64,
    /// Quality weight of this state (1 = perfect health, 0 = dead,
    /// negative = worse than death).
    pub utility: f64,
}

/// Total QALYs across a stream of health states: Σ duration × utility.
///
/// # Arguments
///
/// * `states` — the sequence of health states lived through; order does not
///   affect the (undiscounted) total.
///
/// # Returns
///
/// Total QALYs; `0.0` for an empty stream.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::{qalys, HealthState};
///
/// // Six months waiting at 0.6, then six months treated at 0.85:
/// // 0.5 × 0.6 + 0.5 × 0.85 = 0.725 QALYs.
/// let q = qalys(&[
///     HealthState { duration_years: 0.5, utility: 0.6 },
///     HealthState { duration_years: 0.5, utility: 0.85 },
/// ]);
/// assert!((q - 0.725).abs() < 1e-9);
/// ```
pub fn qalys(states: &[HealthState]) -> f64 {
    states.iter().map(|s| s.duration_years * s.utility).sum()
}

/// QALY loss per patient from delaying treatment.
///
/// The delay is spent at the waiting utility instead of the treated utility,
/// so the loss is `delay × (utility_treated − utility_waiting)`. This is the
/// shortcut form of comparing the two full QALY streams.
///
/// # Arguments
///
/// * `delay_years` — length of the delay in years (e.g. `0.5` for six
///   months).
/// * `utility_waiting` — utility while waiting (worked example: 0.6).
/// * `utility_treated` — utility once treated (worked example: 0.85).
///
/// # Returns
///
/// QALYs lost per patient over the delay.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::qaly_loss_from_delay;
///
/// // A 6-month delay at 0.6 instead of 0.85: 0.5 × 0.25 = 0.125 QALYs lost.
/// assert!((qaly_loss_from_delay(0.5, 0.6, 0.85) - 0.125).abs() < 1e-9);
/// ```
pub fn qaly_loss_from_delay(
    delay_years: f64,
    utility_waiting: f64,
    utility_treated: f64,
) -> f64 {
    delay_years * (utility_treated - utility_waiting)
}

/// QALY gain (or loss) of one stream relative to another.
///
/// The difference between QALYs with and without the intervention — the
/// quantity every QALY claim reduces to. Negative means the intervention
/// stream is worse.
///
/// # Arguments
///
/// * `qalys_with_intervention` — QALYs in the intervention scenario.
/// * `qalys_without_intervention` — QALYs in the comparator scenario.
///
/// # Returns
///
/// `with − without`, in QALYs.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::qaly_gain;
///
/// // Treated now (0.85) vs after a 6-month delay (0.725): gain 0.125 QALYs.
/// assert!((qaly_gain(0.85, 0.725) - 0.125).abs() < 1e-9);
/// ```
pub fn qaly_gain(qalys_with_intervention: f64, qalys_without_intervention: f64) -> f64 {
    qalys_with_intervention - qalys_without_intervention
}

/// Population-level QALYs: per-patient QALYs × number of patients.
///
/// # Arguments
///
/// * `qalys_per_patient` — QALYs gained (or lost) per patient.
/// * `patients` — number of patients affected per period.
///
/// # Returns
///
/// Total QALYs across the population.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::population_qalys;
///
/// // Removing the delay for 400 patients/year at 0.125 QALYs each = 50 QALYs.
/// assert!((population_qalys(0.125, 400.0) - 50.0).abs() < 1e-9);
/// ```
pub fn population_qalys(qalys_per_patient: f64, patients: f64) -> f64 {
    qalys_per_patient * patients
}

/// Monetized health value of QALYs at a willingness-to-pay threshold.
///
/// NICE values health gains at £20,000–£30,000 per QALY.
///
/// # Arguments
///
/// * `qalys` — health gain in QALYs.
/// * `threshold_per_qaly` — willingness-to-pay per QALY in £.
///
/// # Returns
///
/// Health value in £: `qalys × threshold_per_qaly`.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::monetized_value;
///
/// // 50 QALYs ≈ £1.0–£1.5 million/year at NICE thresholds.
/// assert_eq!(monetized_value(50.0, 20_000.0), 1_000_000.0);
/// assert_eq!(monetized_value(50.0, 30_000.0), 1_500_000.0);
/// ```
pub fn monetized_value(qalys: f64, threshold_per_qaly: f64) -> f64 {
    qalys * threshold_per_qaly
}

/// Discount factor for a value accruing `year` years in the future.
///
/// `1 / (1 + rate)^year`. The NICE reference case discounts both costs and
/// QALYs at 3.5%/year. Year 0 is undiscounted (factor 1.0).
///
/// # Arguments
///
/// * `rate` — annual discount rate as a fraction (NICE reference case:
///   `0.035`).
/// * `year` — years into the future the value accrues (fractional years
///   allowed).
///
/// # Returns
///
/// The multiplicative present-value factor in (0, 1] for non-negative
/// rate and year.
///
/// # Examples
///
/// ```rust
/// use health_economics::quality_adjusted_life_year::discount_factor;
///
/// // NICE reference case: 3.5%/year. Year 0 undiscounted; year 1 = 1/1.035.
/// assert_eq!(discount_factor(0.035, 0.0), 1.0);
/// assert!((discount_factor(0.035, 1.0) - 1.0 / 1.035).abs() < 1e-9);
/// ```
pub fn discount_factor(rate: f64, year: f64) -> f64 {
    1.0 / (1.0 + rate).powf(year)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Treated now: 1 year at utility 0.85 = 0.85 QALYs this year.
    #[test]
    fn treated_now_yields_0_85_qalys() {
        // Worked example: "Treated now: 1 year at 0.85 = 0.85 QALYs this year."
        let q = qalys(&[HealthState { duration_years: 1.0, utility: 0.85 }]);
        assert!((q - 0.85).abs() < TOL);
    }

    /// Treated after a 6-month delay: 0.5 × 0.6 + 0.5 × 0.85 = 0.725 QALYs.
    #[test]
    fn six_month_delay_yields_0_725_qalys() {
        // Worked example: "Treated after a 6-month delay: 0.5 × 0.6 + 0.5 ×
        // 0.85 = 0.725 QALYs."
        let q = qalys(&[
            HealthState { duration_years: 0.5, utility: 0.6 },
            HealthState { duration_years: 0.5, utility: 0.85 },
        ]);
        assert!((q - 0.725).abs() < TOL);
    }

    /// QALY loss per patient from the delay: 0.85 − 0.725 = 0.125 QALYs.
    #[test]
    fn delay_costs_0_125_qalys_per_patient() {
        // Worked example: "QALY loss per patient from the delay: 0.85 − 0.725
        // = 0.125 QALYs."
        let now = qalys(&[HealthState { duration_years: 1.0, utility: 0.85 }]);
        let delayed = qalys(&[
            HealthState { duration_years: 0.5, utility: 0.6 },
            HealthState { duration_years: 0.5, utility: 0.85 },
        ]);
        assert!((qaly_gain(now, delayed) - 0.125).abs() < TOL);
        // The shortcut formula agrees.
        assert!((qaly_loss_from_delay(0.5, 0.6, 0.85) - 0.125).abs() < TOL);
    }

    /// Monetized: 0.125 × £20,000–£30,000 = £2,500–£3,750 per patient per
    /// 6-month delay.
    #[test]
    fn delay_loss_is_worth_2_500_to_3_750_per_patient() {
        // Worked example: "0.125 × £20,000–£30,000 = £2,500–£3,750 of health
        // value lost per patient per 6-month delay."
        assert!((monetized_value(0.125, 20_000.0) - 2_500.0).abs() < TOL);
        assert!((monetized_value(0.125, 30_000.0) - 3_750.0).abs() < TOL);
    }

    /// Removing the delay for 400 patients/year is 50 QALYs.
    #[test]
    fn four_hundred_patients_yield_50_qalys() {
        // Worked example: "removes that delay for 400 patients/year, the
        // health value is 50 QALYs."
        let q = population_qalys(0.125, 400.0);
        assert!((q - 50.0).abs() < TOL);
    }

    /// 50 QALYs ≈ £1.0–£1.5 million/year at NICE thresholds.
    #[test]
    fn fifty_qalys_are_worth_1_to_1_5_million() {
        // Worked example: "50 QALYs ≈ £1.0–£1.5 million/year."
        assert!((monetized_value(50.0, 20_000.0) - 1_000_000.0).abs() < TOL);
        assert!((monetized_value(50.0, 30_000.0) - 1_500_000.0).abs() < TOL);
    }

    /// The NICE reference-case discount rate is 3.5%/year; year 0 is
    /// undiscounted and year 1 is 1/1.035.
    #[test]
    fn discount_factor_matches_reference_case() {
        // Doc math: "discounted at 3.5%/year in the NICE reference case."
        assert!((discount_factor(0.035, 0.0) - 1.0).abs() < TOL);
        assert!((discount_factor(0.035, 1.0) - 1.0 / 1.035).abs() < TOL);
    }
}
