//! # Patient-Reported Outcomes (PROMs, PREMs, MCID)
//!
//! PROMs are standardized instruments where patients report their own health
//! status (symptoms, function, quality of life); PREMs capture the care
//! *experience*. The **MCID** — minimal clinically important difference — is
//! the smallest score change patients actually perceive as beneficial: the
//! bar any claimed improvement must clear.
//!
//! This module scores instruments, estimates MCIDs, tests whether a
//! between-arm difference clears the MCID, and bridges responder rates into
//! QALYs and money.
//!
//! ## Formula
//!
//! ```text
//! PROM scoring: instrument-specific sums (e.g., PHQ-9 = Σ 9 items × 0–3)
//!
//! MCID estimation:
//!   anchor-based:       score change among patients who report "somewhat better"
//!   distribution-based: ≈ 0.5 × SD of baseline scores
//!
//! responder = patient improving ≥ MCID
//! ARR = responder rate_treatment − responder rate_control
//! NNT = 1 / ARR
//! ```
//!
//! Legend:
//! - `SD` — standard deviation of baseline scores across the cohort.
//! - `MCID` — minimal clinically important difference (score points).
//! - `responder` — a patient whose score improves by at least the MCID.
//! - `ARR` — absolute risk reduction, the between-arm responder-rate gap
//!   (fraction).
//! - `NNT` — number needed to treat for one additional responder.
//!
//! ## Why it matters
//!
//! PROMs are the primary efficacy currency for digital health: apps rarely
//! move mortality, but they can credibly move validated symptom scores. The
//! instruments that matter are few and standardized — **PHQ-9** (depression,
//! 0–27; severity bands at 5/10/15/20), **GAD-7** (anxiety, 0–21; bands at
//! 5/10/15), **EQ-5D** (utility for QALYs) — and regulators, HTA bodies, and
//! payers accept them precisely because they are comparable across products
//! and trials. The MCID is the honesty gate: PHQ-9 MCID ≈ 5 points, GAD-7
//! ≈ 4, EQ-5D index commonly ~0.03–0.08 — a statistically significant
//! 1.5-point PHQ-9 change on a large sample is *real but clinically
//! meaningless*, and an evidence reviewer will say so.
//!
//! ## Example
//!
//! A depression-support app, RCT vs waiting list, 12 weeks:
//!
//! ```rust
//! use health_economics::patient_reported_outcomes::{
//!     absolute_risk_reduction, adjusted_difference, clears_mcid, cohort_qalys,
//!     extra_responders, monetized_value, number_needed_to_treat,
//!     qalys_from_utility_gain, PHQ9_MCID,
//! };
//!
//! // PHQ-9 change: app −6.2 points, control −2.1 → adjusted difference −4.1.
//! let diff = adjusted_difference(-6.2, -2.1);
//! assert!((diff - (-4.1)).abs() < 1e-9);
//!
//! // MCID check: 4.1 < 5 → below the PHQ-9 MCID; report responders instead.
//! assert!(!clears_mcid(diff, PHQ9_MCID));
//!
//! // Responders (≥5-point drop): app 48%, control 22% → ARR 26%.
//! let arr = absolute_risk_reduction(0.48, 0.22);
//! assert!((arr - 0.26).abs() < 1e-9);
//!
//! // NNT = 1/0.26 ≈ 4 — four users treated per additional clinical response.
//! let nnt = number_needed_to_treat(arr).unwrap();
//! assert!((nnt - 4.0).abs() < 0.2);
//!
//! // Economic bridge: responders' EQ-5D gain 0.06 sustained 6 months = 0.03 QALYs.
//! let q_per_responder = qalys_from_utility_gain(0.06, 0.5);
//! assert!((q_per_responder - 0.03).abs() < 1e-9);
//!
//! // Per 1,000 users: 260 extra responders × 0.03 = 7.8 QALYs
//! // ≈ £156,000–£234,000 of health value at NICE thresholds.
//! let responders = extra_responders(1_000.0, arr);
//! assert!((responders - 260.0).abs() < 1e-9);
//! let q = cohort_qalys(responders, q_per_responder);
//! assert!((q - 7.8).abs() < 1e-9);
//! assert!((monetized_value(q, 20_000.0) - 156_000.0).abs() < 1e-9);
//! assert!((monetized_value(q, 30_000.0) - 234_000.0).abs() < 1e-9);
//! ```
//!
//! The responder/NNT framing survives review where the sub-MCID mean
//! difference would have been dismissed.
//!
//! ## Software engineering connection
//!
//! - PROMs are a data-collection problem software is uniquely placed to
//!   solve: in-app instruments get completion rates and longitudinal density
//!   paper never achieved (EQ-5D is five screens).
//! - Use the validated instrument *verbatim* — rewording invalidates it, and
//!   licensing applies.
//! - Schedule measurement by protocol, not engagement convenience — measuring
//!   only active users is survivorship bias.
//! - Version-lock instrument data like any schema — a mid-study wording
//!   change is data corruption.
//! - PREMs map to CSAT/NPS-style instruments: standardized beats homegrown
//!   wherever the audience is a payer.
//!
//! ## Pitfalls
//!
//! - **Statistical significance below MCID** presented as clinical benefit —
//!   the field's most common inflation.
//! - **Regression to the mean**: users enroll at symptom peaks; single-arm
//!   before/after overstates hugely — comparators are non-negotiable.
//! - **Instrument shopping**: running PHQ-9, GAD-7, and WHO-5, then reporting
//!   whichever moved — pre-register the primary.
//! - **Digital-consent survey pressure**: nudging users toward favorable
//!   responses corrupts the instrument (and the reviewers know the base
//!   rates).
//!
//! ## Sources
//!
//! - MCID estimation review (EQ-5D).
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC10526144/>
//! - PROMs vs PREMs primer.
//!   <https://www.forcetherapeutics.com/blog/whats-the-difference-between-pros-proms-pro-pms-and-prems>
//! - Kroenke K, et al. PHQ-9 validation literature.
//!   <https://pubmed.ncbi.nlm.nih.gov/11556941/>
//!
//! Topic doc: health-economics-metrics/topics/patient-reported-outcomes.md

/// PHQ-9 (depression, score range 0–27) minimal clinically important difference.
///
/// Approximately 5 score points: the smallest PHQ-9 change patients actually
/// perceive as beneficial. Severity bands sit at 5/10/15/20.
pub const PHQ9_MCID: f64 = 5.0;

/// GAD-7 (anxiety, score range 0–21) minimal clinically important difference.
///
/// Approximately 4 score points. Severity bands sit at 5/10/15.
pub const GAD7_MCID: f64 = 4.0;

/// Instrument-specific sum score across item responses.
///
/// PROM totals are simple sums of item responses (e.g., PHQ-9 = Σ of 9 items
/// each scored 0–3). This performs no range validation — supply responses on
/// the instrument's own scale.
///
/// # Arguments
///
/// * `item_responses` — one response per item, on the instrument's item scale
///   (PHQ-9 items: 0–3).
///
/// # Returns
///
/// The total score (sum of items); `0.0` for an empty slice.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::instrument_sum_score;
///
/// // A PHQ-9 questionnaire: nine items each scored 0–3, summed to 15
/// // (moderately severe threshold sits at 15).
/// let items = [3.0, 3.0, 2.0, 2.0, 1.0, 1.0, 0.0, 0.0, 3.0];
/// assert_eq!(instrument_sum_score(&items), 15.0);
/// ```
pub fn instrument_sum_score(item_responses: &[f64]) -> f64 {
    item_responses.iter().sum()
}

/// Distribution-based MCID heuristic: roughly half the standard deviation of baseline scores.
///
/// A rough fallback when no anchor-based estimate exists. Prefer anchor-based
/// MCIDs (score change among patients who report feeling "somewhat better")
/// or the published instrument MCIDs ([`PHQ9_MCID`], [`GAD7_MCID`]).
///
/// # Arguments
///
/// * `baseline_score_sd` — standard deviation of baseline scores across the
///   cohort, in score points.
///
/// # Returns
///
/// `0.5 × baseline_score_sd`, in score points.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::distribution_based_mcid;
///
/// // Baseline SD of 8 points → distribution-based MCID of 4 points.
/// assert_eq!(distribution_based_mcid(8.0), 4.0);
/// ```
pub fn distribution_based_mcid(baseline_score_sd: f64) -> f64 {
    0.5 * baseline_score_sd
}

/// Adjusted between-arm difference in score change.
///
/// Treatment-arm change minus control-arm change. For symptom scales like
/// PHQ-9 where lower is better, changes are negative and a *negative*
/// difference means greater symptom reduction in the treatment arm.
///
/// # Arguments
///
/// * `treatment_change` — mean score change in the treatment arm (score
///   points; negative = improvement on symptom scales).
/// * `control_change` — mean score change in the control arm, same
///   convention.
///
/// # Returns
///
/// `treatment_change − control_change` in score points.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::adjusted_difference;
///
/// // PHQ-9 change: app −6.2 points, control −2.1 → adjusted difference −4.1.
/// let d = adjusted_difference(-6.2, -2.1);
/// assert!((d - (-4.1)).abs() < 1e-9);
/// ```
pub fn adjusted_difference(treatment_change: f64, control_change: f64) -> f64 {
    treatment_change - control_change
}

/// Whether a mean between-arm difference clears the MCID in magnitude.
///
/// Uses the absolute value, so it works for both "lower is better" symptom
/// scales and "higher is better" function scales. A statistically significant
/// sub-MCID difference is real but clinically meaningless — this is the
/// honesty gate any claimed improvement must pass.
///
/// # Arguments
///
/// * `mean_difference` — adjusted between-arm difference (score points; sign
///   ignored).
/// * `mcid` — minimal clinically important difference for the instrument
///   (score points).
///
/// # Returns
///
/// `true` when `|mean_difference| ≥ mcid`.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::{clears_mcid, PHQ9_MCID};
///
/// // Worked example: the −4.1-point mean difference is below the PHQ-9 MCID of 5.
/// assert!(!clears_mcid(-4.1, PHQ9_MCID));
/// // A −5.5-point difference would clear it.
/// assert!(clears_mcid(-5.5, PHQ9_MCID));
/// ```
pub fn clears_mcid(mean_difference: f64, mcid: f64) -> bool {
    mean_difference.abs() >= mcid
}

/// Absolute risk reduction: the between-arm gap in responder rates.
///
/// Responder rate in the treatment arm minus responder rate in the control
/// arm, both as fractions (e.g. `0.48` for 48%). A responder is a patient
/// improving by at least the MCID (or by ≥50% for the PHQ-9 convention).
///
/// # Arguments
///
/// * `responder_rate_treatment` — fraction of treatment-arm patients who
///   respond (0–1).
/// * `responder_rate_control` — fraction of control-arm patients who respond
///   (0–1).
///
/// # Returns
///
/// The ARR as a fraction; negative when the control arm does better.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::absolute_risk_reduction;
///
/// // Responders: app 48%, control 22% → ARR 26%.
/// let arr = absolute_risk_reduction(0.48, 0.22);
/// assert!((arr - 0.26).abs() < 1e-9);
/// ```
pub fn absolute_risk_reduction(
    responder_rate_treatment: f64,
    responder_rate_control: f64,
) -> f64 {
    responder_rate_treatment - responder_rate_control
}

/// Number needed to treat: how many patients must receive the intervention for one additional responder.
///
/// NNT = 1 / ARR. Smaller is better; NNT = 4 means four users treated per
/// additional clinical response.
///
/// # Arguments
///
/// * `absolute_risk_reduction` — between-arm responder-rate gap as a fraction
///   (e.g. `0.26`).
///
/// # Returns
///
/// `Some(1 / ARR)`, or `None` when the ARR is exactly zero (the arms do not
/// differ, so NNT is undefined/infinite). A negative ARR yields a negative
/// value (a number needed to harm).
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::number_needed_to_treat;
///
/// // NNT = 1/0.26 ≈ 4 — four users treated per additional clinical response.
/// let nnt = number_needed_to_treat(0.26).unwrap();
/// assert!((nnt - 4.0).abs() < 0.2);
///
/// // Identical arms: NNT undefined.
/// assert!(number_needed_to_treat(0.0).is_none());
/// ```
pub fn number_needed_to_treat(absolute_risk_reduction: f64) -> Option<f64> {
    if absolute_risk_reduction == 0.0 {
        None
    } else {
        Some(1.0 / absolute_risk_reduction)
    }
}

/// Extra responders produced in a cohort.
///
/// Cohort size × absolute risk reduction: the additional patients who respond
/// because the intervention exists, beyond what the control condition would
/// have produced.
///
/// # Arguments
///
/// * `cohort_size` — number of people receiving the intervention.
/// * `absolute_risk_reduction` — between-arm responder-rate gap as a fraction.
///
/// # Returns
///
/// The expected count of additional responders.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::extra_responders;
///
/// // Per 1,000 users at ARR 26%: 260 extra responders.
/// assert_eq!(extra_responders(1_000.0, 0.26), 260.0);
/// ```
pub fn extra_responders(cohort_size: f64, absolute_risk_reduction: f64) -> f64 {
    cohort_size * absolute_risk_reduction
}

/// QALYs per responder from a sustained utility gain.
///
/// Utility gain × duration in years. Duration is in *years*, so a gain
/// sustained for 6 months is `duration_years = 0.5`.
///
/// # Arguments
///
/// * `utility_gain` — EQ-5D-style utility improvement per responder (e.g.
///   `0.06`).
/// * `duration_years` — how long the gain is sustained, in years.
///
/// # Returns
///
/// QALYs per responder: `utility_gain × duration_years`.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::qalys_from_utility_gain;
///
/// // An EQ-5D gain of 0.06 sustained 6 months = 0.03 QALYs per responder.
/// assert!((qalys_from_utility_gain(0.06, 0.5) - 0.03).abs() < 1e-9);
/// ```
pub fn qalys_from_utility_gain(utility_gain: f64, duration_years: f64) -> f64 {
    utility_gain * duration_years
}

/// Cohort-level QALY total: extra responders × QALYs per responder.
///
/// # Arguments
///
/// * `extra_responders` — additional responders attributable to the
///   intervention (see [`extra_responders`]).
/// * `qalys_per_responder` — QALYs each responder gains (see
///   [`qalys_from_utility_gain`]).
///
/// # Returns
///
/// Total QALYs across the cohort.
///
/// # Examples
///
/// ```rust
/// use health_economics::patient_reported_outcomes::cohort_qalys;
///
/// // 260 extra responders × 0.03 QALYs = 7.8 QALYs.
/// assert!((cohort_qalys(260.0, 0.03) - 7.8).abs() < 1e-9);
/// ```
pub fn cohort_qalys(extra_responders: f64, qalys_per_responder: f64) -> f64 {
    extra_responders * qalys_per_responder
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
/// use health_economics::patient_reported_outcomes::monetized_value;
///
/// // 7.8 QALYs ≈ £156,000–£234,000 at NICE thresholds (£20k–£30k/QALY).
/// assert_eq!(monetized_value(7.8, 20_000.0), 156_000.0);
/// assert_eq!(monetized_value(7.8, 30_000.0), 234_000.0);
/// ```
pub fn monetized_value(qalys: f64, threshold_per_qaly: f64) -> f64 {
    qalys * threshold_per_qaly
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// PHQ-9 change: app −6.2, control −2.1 → adjusted difference −4.1.
    #[test]
    fn adjusted_phq9_difference_is_minus_4_point_1() {
        // Worked example: "PHQ-9 change: app −6.2 points, control −2.1 →
        // adjusted difference −4.1."
        let d = adjusted_difference(-6.2, -2.1);
        assert!((d - (-4.1)).abs() < TOL);
    }

    /// MCID check: 4.1 < 5 → mean difference below the PHQ-9 MCID.
    #[test]
    fn mean_difference_of_4_point_1_is_below_phq9_mcid() {
        // Worked example: "MCID check: 4.1 < 5 → mean difference below MCID."
        assert!(!clears_mcid(-4.1, PHQ9_MCID));
    }

    /// Responders: app 48%, control 22% → ARR 26%.
    #[test]
    fn absolute_risk_reduction_is_26_percent() {
        // Worked example: "responders (≥5-point drop): app 48%, control 22% → ARR 26%."
        let arr = absolute_risk_reduction(0.48, 0.22);
        assert!((arr - 0.26).abs() < TOL);
    }

    /// NNT = 1/0.26 ≈ 4 — four users treated per additional clinical response.
    #[test]
    fn nnt_is_approximately_4() {
        // Worked example: "NNT = 1/0.26 ≈ 4."
        let nnt = number_needed_to_treat(0.26).unwrap();
        assert!((nnt - 1.0 / 0.26).abs() < TOL);
        assert!((nnt - 4.0).abs() < 0.2);
    }

    /// EQ-5D gain 0.06 sustained 6 months = 0.03 QALYs per responder.
    #[test]
    fn responder_utility_gain_yields_0_03_qalys() {
        // Worked example: "responders' EQ-5D gain 0.06 sustained 6 months = 0.03 QALYs."
        let q = qalys_from_utility_gain(0.06, 0.5);
        assert!((q - 0.03).abs() < TOL);
    }

    /// Per 1,000 users: 260 extra responders.
    #[test]
    fn cohort_of_1000_yields_260_extra_responders() {
        // Worked example: "per 1,000 users: 260 extra responders."
        let r = extra_responders(1_000.0, 0.26);
        assert!((r - 260.0).abs() < TOL);
    }

    /// 260 extra responders × 0.03 QALYs = 7.8 QALYs.
    #[test]
    fn cohort_qalys_are_7_point_8() {
        // Worked example: "260 extra responders × 0.03 = 7.8 QALYs."
        let q = cohort_qalys(260.0, 0.03);
        assert!((q - 7.8).abs() < TOL);
    }

    /// 7.8 QALYs ≈ £156,000–£234,000 at NICE thresholds (£20k–£30k/QALY).
    #[test]
    fn monetized_value_is_156k_to_234k_at_nice_thresholds() {
        // Worked example: "≈ £156,000–£234,000 of health value at NICE thresholds."
        let low = monetized_value(7.8, 20_000.0);
        let high = monetized_value(7.8, 30_000.0);
        assert!((low - 156_000.0).abs() < TOL);
        assert!((high - 234_000.0).abs() < TOL);
    }

    /// PHQ-9 is the sum of its nine 0–3 items.
    #[test]
    fn phq9_score_is_sum_of_items() {
        // Doc math: "PROM scoring: instrument-specific sums (e.g., PHQ-9 = Σ 9 items × 0–3)."
        let items = [3.0, 3.0, 2.0, 2.0, 1.0, 1.0, 0.0, 0.0, 3.0];
        assert!((instrument_sum_score(&items) - 15.0).abs() < TOL);
    }

    /// Distribution-based MCID is half the baseline SD.
    #[test]
    fn distribution_based_mcid_is_half_baseline_sd() {
        // Doc math: "distribution-based: ≈ 0.5 × SD of baseline scores."
        assert!((distribution_based_mcid(8.0) - 4.0).abs() < TOL);
    }

    // Edge case: identical arms (ARR = 0) leave NNT undefined.
    #[test]
    fn zero_arr_has_no_defined_nnt() {
        assert!(number_needed_to_treat(0.0).is_none());
    }
}
