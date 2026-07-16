//! # Referral to Treatment (RTT)
//!
//! Referral to treatment is the elapsed time from a GP's referral to the
//! start of consultant-led treatment. The NHS Constitution sets the
//! standard: **92% of patients should start treatment within 18 weeks**. RTT
//! is the single most politically visible operational metric in the English
//! NHS.
//!
//! Every week a patient waits is health lost — the wait is spent in a worse
//! health state — and often cost gained as conditions deteriorate.
//!
//! ## Formula
//!
//! ```text
//! RTT performance = patients treated within 18 weeks / total treated × 100
//! Waiting-time health cost per patient = wait duration × (utility_treated − utility_waiting)
//!
//! Pathway view: RTT = Σ stage durations
//! ```
//!
//! Legend:
//! - `RTT performance` — percentage of treated patients who started within
//!   18 weeks (standard: ≥ 92%).
//! - `wait duration` — time spent waiting, in years for QALY math.
//! - `utility_waiting` / `utility_treated` — health-state quality weights
//!   while waiting vs once treated (1 = perfect health).
//! - stage durations — referral triage → first appointment → diagnostics →
//!   decision → treatment; improve the longest queue, not the busiest stage.
//!
//! ## Why it matters
//!
//! Trusts that miss RTT targets face regulatory scrutiny, intervention, and
//! reputational damage; the national waiting list is a front-page number.
//! Software that saves time anywhere in the referral-to-treatment pathway —
//! triage, diagnostics turnaround, clinic capacity, scheduling — directly
//! mitigates the operational and financial consequences of failing the
//! standard, which is why RTT impact is a first-class benefit line in NHS
//! digital business cases.
//!
//! ## Example
//!
//! A specialty treats 5,000 pathway patients/year; mean wait 24 weeks;
//! waiting utility 0.68 vs treated 0.80. Digital triage plus
//! straight-to-test protocols remove 5 weeks of pure queueing:
//!
//! ```rust
//! use health_economics::referral_to_treatment::{
//!     meets_rtt_standard, monetized_value, qaly_gain_from_wait_reduction,
//!     rtt_performance_percent,
//! };
//!
//! // QALY gain = 5,000 × (5/52) × (0.80 − 0.68) = 57.7 QALYs/year.
//! let q = qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
//! assert!((q - 57.7).abs() < 0.05);
//!
//! // Monetized at £20,000–£30,000/QALY ≈ £1.15M–£1.73M/year of health value.
//! assert!((monetized_value(q, 20_000.0) - 1_150_000.0).abs() < 10_000.0);
//! assert!((monetized_value(q, 30_000.0) - 1_730_000.0).abs() < 10_000.0);
//!
//! // And the trust moves from breaching to meeting the 18-week standard:
//! // 4,600 of 5,000 treated within 18 weeks is exactly 92%.
//! let p = rtt_performance_percent(4_600.0, 5_000.0).unwrap();
//! assert_eq!(p, 92.0);
//! assert!(meets_rtt_standard(p));
//! ```
//!
//! — plus governance value from meeting the standard that no spreadsheet
//! fully captures.
//!
//! ## Software engineering connection
//!
//! - RTT is a **lead-time metric over a multi-stage queue** — the hospital's
//!   version of commit-to-production lead time (DORA).
//! - The improvement method is identical: instrument every stage, find where
//!   calendar time pools (nearly always handoffs and queues, not clinical
//!   work), and remove wait states.
//! - Typical software wins: e-triage that routes referrals in hours instead
//!   of weekly batches, diagnostic-results push instead of follow-up
//!   appointments, automated straight-to-test criteria.
//! - Value the improvement with cost of delay denominated in QALYs/week.
//!
//! ## Pitfalls
//!
//! - **Improving a stage that isn't the constraint** — cutting
//!   first-appointment waits while diagnostics queues grow just moves the
//!   pool.
//! - **Gaming**: pathway resets and clock pauses can improve reported RTT
//!   without treating anyone sooner; audit the underlying distribution.
//! - **Claiming the whole pathway improvement** for one tool when several
//!   changes landed together — attribution needs a comparator.
//!
//! ## Sources
//!
//! - NHS England, RTT waiting times statistics.
//!   <https://www.england.nhs.uk/statistics/statistical-work-areas/rtt-waiting-times/>
//! - NHS England, elective care recovery plan.
//!   <https://www.england.nhs.uk/coronavirus/publication/delivery-plan-for-tackling-the-covid-19-backlog-of-elective-care/>
//!
//! Topic doc: health-economics-metrics/topics/referral-to-treatment.md

/// The NHS Constitution RTT standard: 92% of patients should start treatment within 18 weeks.
///
/// A percentage (92.0). Performance at or above this meets the standard.
pub const RTT_STANDARD_PERCENT: f64 = 92.0;

/// RTT performance as a percentage.
///
/// Patients treated within 18 weeks ÷ total treated × 100.
///
/// # Arguments
///
/// * `treated_within_18_weeks` — patients who started treatment within 18
///   weeks of referral.
/// * `total_treated` — all patients who started treatment in the period.
///
/// # Returns
///
/// `Some(percentage)`, or `None` when no patients were treated (the
/// percentage is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::rtt_performance_percent;
///
/// // 4,600 of 5,000 treated within 18 weeks → 92.0%.
/// assert_eq!(rtt_performance_percent(4_600.0, 5_000.0), Some(92.0));
///
/// // Nobody treated: undefined.
/// assert_eq!(rtt_performance_percent(0.0, 0.0), None);
/// ```
pub fn rtt_performance_percent(
    treated_within_18_weeks: f64,
    total_treated: f64,
) -> Option<f64> {
    if total_treated == 0.0 {
        None
    } else {
        Some(treated_within_18_weeks / total_treated * 100.0)
    }
}

/// Whether an RTT performance percentage meets the 92% standard.
///
/// # Arguments
///
/// * `performance_percent` — RTT performance as a percentage (see
///   [`rtt_performance_percent`]).
///
/// # Returns
///
/// `true` when `performance_percent ≥ 92.0`
/// ([`RTT_STANDARD_PERCENT`]); exactly 92% meets the standard.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::meets_rtt_standard;
///
/// assert!(meets_rtt_standard(92.0));
/// assert!(!meets_rtt_standard(91.9));
/// ```
pub fn meets_rtt_standard(performance_percent: f64) -> bool {
    performance_percent >= RTT_STANDARD_PERCENT
}

/// Waiting-time health cost per patient, in QALYs.
///
/// Wait duration (years) × (utility treated − utility waiting): the health
/// forgone by spending the wait in the worse state.
///
/// # Arguments
///
/// * `wait_duration_years` — wait length in years (5 weeks = `5.0 / 52.0`).
/// * `utility_treated` — health-state utility once treated (worked example:
///   0.80).
/// * `utility_waiting` — health-state utility while waiting (worked example:
///   0.68).
///
/// # Returns
///
/// QALYs lost per patient over the wait.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::waiting_health_cost_qalys;
///
/// // A 5-week wait at 0.68 instead of 0.80: (5/52) × 0.12 ≈ 0.0115 QALYs.
/// let q = waiting_health_cost_qalys(5.0 / 52.0, 0.80, 0.68);
/// assert!((q - 5.0 / 52.0 * 0.12).abs() < 1e-9);
/// ```
pub fn waiting_health_cost_qalys(
    wait_duration_years: f64,
    utility_treated: f64,
    utility_waiting: f64,
) -> f64 {
    wait_duration_years * (utility_treated - utility_waiting)
}

/// Annual QALY gain from removing weeks of pure queueing across a cohort.
///
/// Patients × (weeks removed / 52) × (utility treated − utility waiting).
/// The division by 52 converts weeks to years so the result is in QALYs.
///
/// # Arguments
///
/// * `patients_per_year` — pathway patients per year who benefit (worked
///   example: 5,000).
/// * `weeks_removed` — weeks of queueing removed per patient (worked
///   example: 5).
/// * `utility_treated` — utility once treated (worked example: 0.80).
/// * `utility_waiting` — utility while waiting (worked example: 0.68).
///
/// # Returns
///
/// QALYs gained per year across the cohort.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::qaly_gain_from_wait_reduction;
///
/// // 5,000 × (5/52) × (0.80 − 0.68) = 57.7 QALYs/year.
/// let q = qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
/// assert!((q - 57.7).abs() < 0.05);
/// ```
pub fn qaly_gain_from_wait_reduction(
    patients_per_year: f64,
    weeks_removed: f64,
    utility_treated: f64,
    utility_waiting: f64,
) -> f64 {
    patients_per_year * (weeks_removed / 52.0) * (utility_treated - utility_waiting)
}

/// Monetized health value of QALYs at a willingness-to-pay threshold.
///
/// # Arguments
///
/// * `qalys` — health gain in QALYs.
/// * `threshold_per_qaly` — willingness-to-pay per QALY in £ (NICE:
///   £20,000–£30,000).
///
/// # Returns
///
/// Health value in £: `qalys × threshold_per_qaly`.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::monetized_value;
///
/// // 57.7 QALYs/year at £20k–£30k/QALY ≈ £1.15M–£1.73M/year.
/// assert!((monetized_value(57.7, 20_000.0) - 1_154_000.0).abs() < 1e-6);
/// assert!((monetized_value(57.7, 30_000.0) - 1_731_000.0).abs() < 1e-6);
/// ```
pub fn monetized_value(qalys: f64, threshold_per_qaly: f64) -> f64 {
    qalys * threshold_per_qaly
}

/// Pathway view: total RTT is the sum of stage durations.
///
/// Stages run referral triage → first appointment → diagnostics → decision →
/// treatment. Use consistent units (typically weeks).
///
/// # Arguments
///
/// * `stage_durations` — duration of each pathway stage (e.g. weeks).
///
/// # Returns
///
/// The total pathway duration; `0.0` for an empty pathway.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::total_pathway_duration;
///
/// // Five stages totalling 24 weeks — matching the worked example's mean wait.
/// let stages = [1.0, 6.0, 9.0, 2.0, 6.0];
/// assert_eq!(total_pathway_duration(&stages), 24.0);
/// ```
pub fn total_pathway_duration(stage_durations: &[f64]) -> f64 {
    stage_durations.iter().sum()
}

/// The longest stage — improve the longest queue, not the busiest stage.
///
/// Calendar time pools in handoffs and queues, not clinical work; the
/// constraint is the stage where the most time sits.
///
/// # Arguments
///
/// * `stage_durations` — duration of each pathway stage (e.g. weeks).
///
/// # Returns
///
/// `Some(longest duration)`, or `None` for an empty pathway.
///
/// # Examples
///
/// ```rust
/// use health_economics::referral_to_treatment::longest_stage;
///
/// // The 9-week diagnostics queue is the stage to improve.
/// let stages = [1.0, 6.0, 9.0, 2.0, 6.0];
/// assert_eq!(longest_stage(&stages), Some(9.0));
///
/// // An empty pathway has no longest stage.
/// assert_eq!(longest_stage(&[]), None);
/// ```
pub fn longest_stage(stage_durations: &[f64]) -> Option<f64> {
    stage_durations.iter().copied().fold(None, |m, d| match m {
        None => Some(d),
        Some(b) => Some(if d > b { d } else { b }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Removing 5 weeks of queueing for 5,000 patients/year at utilities
    /// 0.80 treated vs 0.68 waiting: 5,000 × (5/52) × 0.12 = 57.7 QALYs/year.
    #[test]
    fn wait_reduction_yields_57_7_qalys_per_year() {
        // Worked example: "QALY gain = 5,000 × (5/52) × (0.80 − 0.68) = 57.7 QALYs/year."
        let q = qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
        assert!((q - 57.7).abs() < 0.05);
    }

    /// Monetized at £20,000–£30,000/QALY ≈ £1.15M–£1.73M/year.
    #[test]
    fn health_value_is_1_15m_to_1_73m_per_year() {
        // Worked example: "Monetized at £20,000–£30,000/QALY ≈ £1.15M–£1.73M/year."
        let q = qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
        let low = monetized_value(q, 20_000.0);
        let high = monetized_value(q, 30_000.0);
        assert!((low - 1_150_000.0).abs() < 10_000.0);
        assert!((high - 1_730_000.0).abs() < 10_000.0);
    }

    /// Per-patient waiting cost over the 5-week wait matches the cohort math:
    /// (5/52) × 0.12 QALYs each.
    #[test]
    fn per_patient_waiting_cost_matches_cohort() {
        // Doc math: "Waiting-time health cost per patient = wait duration ×
        // (utility_treated − utility_waiting)."
        let per_patient = waiting_health_cost_qalys(5.0 / 52.0, 0.80, 0.68);
        let cohort = qaly_gain_from_wait_reduction(5_000.0, 5.0, 0.80, 0.68);
        assert!((per_patient * 5_000.0 - cohort).abs() < TOL);
    }

    /// 4,600 of 5,000 treated within 18 weeks is exactly the 92% standard.
    #[test]
    fn ninety_two_percent_meets_the_standard() {
        // Doc standard: "92% of patients should start treatment within 18 weeks."
        let p = rtt_performance_percent(4_600.0, 5_000.0).unwrap();
        assert!((p - 92.0).abs() < TOL);
        assert!(meets_rtt_standard(p));
        assert!(!meets_rtt_standard(91.9));
    }

    /// RTT is the sum of stage durations, and the longest queue is the one
    /// to improve.
    #[test]
    fn pathway_sums_and_longest_stage() {
        // Doc math: "Pathway view: RTT = Σ stage durations ... improve the
        // longest queue, not the busiest stage." (24 weeks matches the worked
        // example's mean wait.)
        let stages = [1.0, 6.0, 9.0, 2.0, 6.0]; // weeks per stage
        assert!((total_pathway_duration(&stages) - 24.0).abs() < TOL);
        assert!((longest_stage(&stages).unwrap() - 9.0).abs() < TOL);
    }

    // Edge case: no treated patients leaves performance undefined.
    #[test]
    fn zero_treated_has_no_defined_performance() {
        assert!(rtt_performance_percent(0.0, 0.0).is_none());
    }

    // Edge case: an empty pathway has no longest stage.
    #[test]
    fn empty_pathway_has_no_longest_stage() {
        assert!(longest_stage(&[]).is_none());
    }
}
