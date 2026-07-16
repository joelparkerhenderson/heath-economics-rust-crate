//! # QALY Shortfall and Severity Modifiers
//!
//! QALY shortfall measures how much future health a disease takes from
//! patients compared to the general population. NICE uses it to apply
//! **severity modifiers**: the sicker the population, the more each QALY
//! gained is worth — up to 1.7× the standard threshold.
//!
//! Both shortfall measures are computed over remaining lifetime with
//! *current standard of care* (not untreated natural history), against the
//! age- and sex-matched general population's remaining discounted QALYs.
//!
//! ## Formula
//!
//! ```text
//! Absolute shortfall     = QALYs_general_population − QALYs_with_condition
//! Proportional shortfall = Absolute shortfall / QALYs_general_population
//!
//! Weight ×1.0: absolute < 12 and proportional < 0.85
//! Weight ×1.2: absolute ≥ 12 or  proportional ≥ 0.85
//! Weight ×1.7: absolute ≥ 18 or  proportional ≥ 0.95
//! (whichever measure gives the higher weight applies)
//! ```
//!
//! Legend:
//! - `QALYs_general_population` — remaining discounted QALYs an age/sex-matched
//!   member of the general population expects.
//! - `QALYs_with_condition` — remaining discounted QALYs expected with the
//!   condition under current standard of care.
//! - Weight — the NICE 2022 severity multiplier applied to the health gain
//!   (or equivalently to the threshold).
//!
//! ## Why it matters
//!
//! Since NICE's 2022 manual, severity is an explicit multiplier on the value
//! of health gains, replacing the old end-of-life premium. A technology for a
//! severe condition is judged against an effective threshold of up to
//! ~£51,000/QALY instead of £30,000 (the weight scales the £20k–£30k band to
//! £24k–£36k at ×1.2 and £34k–£51k at ×1.7). If your software serves a
//! severely affected population (advanced heart failure, severe mental
//! illness), the severity modifier can be the difference between a fundable
//! and unfundable economic case — and you need shortfall math to claim it.
//!
//! ## Example
//!
//! Patients with an aggressive condition, average age 60. The general
//! population at 60 expects 14.2 discounted QALYs; with the condition under
//! current care, 2.1:
//!
//! ```rust
//! use health_economics::qaly_shortfall_and_severity_modifiers::{
//!     absolute_shortfall, effective_icer, proportional_shortfall, severity_weight,
//! };
//!
//! // Absolute shortfall = 14.2 − 2.1 = 12.1 (≥ 12 → qualifies for ×1.2).
//! let a = absolute_shortfall(14.2, 2.1);
//! assert!((a - 12.1).abs() < 1e-9);
//!
//! // Proportional shortfall = 12.1 / 14.2 = 0.852 (≥ 0.85 → also ×1.2).
//! let p = proportional_shortfall(a, 14.2).unwrap();
//! assert!((p - 0.852).abs() < 5e-4);
//!
//! // Both measures land in the ×1.2 band.
//! let w = severity_weight(a, p);
//! assert_eq!(w, 1.2);
//!
//! // A £26,000/QALY ICER becomes effectively ≈ £21,700/QALY — comfortably fundable.
//! let e = effective_icer(26_000.0, w).unwrap();
//! assert!((e - 21_700.0).abs() < 100.0);
//! ```
//!
//! The shortfall calculation just moved the decision.
//!
//! ## Software engineering connection
//!
//! - Severity weighting is a formal version of something engineering orgs do
//!   by instinct: spend more per unit improvement on the worst-off systems.
//! - The transferable pattern: compute each service's "SLO shortfall" (how
//!   far below its expected healthy baseline it runs, absolutely and
//!   proportionally), and weight remediation value accordingly.
//! - This justifies, with arithmetic instead of arguments, why the burning
//!   legacy system gets more investment per hour saved than a healthy one.
//! - Same governance lesson: publish the weights *before* the prioritization
//!   meeting, or every team claims severity.
//!
//! ## Pitfalls
//!
//! - **Computing shortfall against the wrong baseline**: it is measured under
//!   *current standard of care*, not untreated natural history.
//! - **Age sensitivity**: shortfall depends heavily on population age
//!   (younger patients have more QALYs to lose → higher absolute shortfall);
//!   use the actual treated population's age distribution.
//! - **Assuming the modifier applies elsewhere** — it is a NICE (England)
//!   mechanism; other HTA bodies handle severity differently (or not at all).
//!
//! ## Sources
//!
//! - Analysis of NICE severity modifier decisions, Value in Health 2024.
//!   <https://www.sciencedirect.com/science/article/pii/S1098301524000858>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//! - Mtech Access, NICE HTA decision modifiers explainer.
//!   <https://mtechaccess.co.uk/nice-hta-decision-modifier/>
//!
//! Topic doc: health-economics-metrics/topics/qaly-shortfall-and-severity-modifiers.md

/// Absolute QALY shortfall: how many discounted QALYs the condition takes from patients.
///
/// Remaining discounted QALYs expected by the age/sex-matched general
/// population, minus those expected with the condition under current
/// standard of care. Units: QALYs.
///
/// # Arguments
///
/// * `general_population_qalys` — remaining discounted QALYs for the matched
///   general population (worked example: 14.2 at age 60).
/// * `qalys_with_condition` — remaining discounted QALYs with the condition
///   under current care (worked example: 2.1).
///
/// # Returns
///
/// The absolute shortfall in QALYs: `general − with_condition`.
///
/// # Examples
///
/// ```rust
/// use health_economics::qaly_shortfall_and_severity_modifiers::absolute_shortfall;
///
/// // 14.2 − 2.1 = 12.1 QALYs — at or above the ≥12 cut-off for the ×1.2 weight.
/// let a = absolute_shortfall(14.2, 2.1);
/// assert!((a - 12.1).abs() < 1e-9);
/// ```
pub fn absolute_shortfall(
    general_population_qalys: f64,
    qalys_with_condition: f64,
) -> f64 {
    general_population_qalys - qalys_with_condition
}

/// Proportional QALY shortfall: the fraction of remaining health the condition destroys.
///
/// Absolute shortfall divided by the general population's remaining QALYs; a
/// dimensionless fraction in [0, 1] for typical inputs (1.0 = the condition
/// takes everything).
///
/// # Arguments
///
/// * `absolute_shortfall` — QALYs lost to the condition (see
///   [`absolute_shortfall`]).
/// * `general_population_qalys` — remaining discounted QALYs for the matched
///   general population.
///
/// # Returns
///
/// `Some(absolute_shortfall / general_population_qalys)`, or `None` when the
/// general-population expectation is zero (the fraction is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::qaly_shortfall_and_severity_modifiers::proportional_shortfall;
///
/// // 12.1 / 14.2 = 0.852 — at or above the ≥0.85 cut-off for the ×1.2 weight.
/// let p = proportional_shortfall(12.1, 14.2).unwrap();
/// assert!((p - 0.852).abs() < 5e-4);
///
/// // Zero general-population QALYs: undefined.
/// assert!(proportional_shortfall(1.0, 0.0).is_none());
/// ```
pub fn proportional_shortfall(
    absolute_shortfall: f64,
    general_population_qalys: f64,
) -> Option<f64> {
    if general_population_qalys == 0.0 {
        None
    } else {
        Some(absolute_shortfall / general_population_qalys)
    }
}

/// NICE 2022 severity weight from the two shortfall measures.
///
/// Whichever measure gives the higher weight applies:
/// ×1.7 when absolute ≥ 18 or proportional ≥ 0.95;
/// ×1.2 when absolute ≥ 12 or proportional ≥ 0.85;
/// otherwise ×1.0.
///
/// # Arguments
///
/// * `absolute_shortfall` — QALYs lost to the condition.
/// * `proportional_shortfall` — fraction of remaining health lost (0–1).
///
/// # Returns
///
/// The severity multiplier: `1.0`, `1.2`, or `1.7`.
///
/// # Examples
///
/// ```rust
/// use health_economics::qaly_shortfall_and_severity_modifiers::severity_weight;
///
/// // Worked example: absolute 12.1 and proportional 0.852 both land in the ×1.2 band.
/// assert_eq!(severity_weight(12.1, 0.852), 1.2);
///
/// // Either measure alone can trigger the top band.
/// assert_eq!(severity_weight(18.0, 0.50), 1.7);
/// assert_eq!(severity_weight(5.0, 0.95), 1.7);
///
/// // Below both cut-offs: no modifier.
/// assert_eq!(severity_weight(5.0, 0.40), 1.0);
/// ```
pub fn severity_weight(absolute_shortfall: f64, proportional_shortfall: f64) -> f64 {
    if absolute_shortfall >= 18.0 || proportional_shortfall >= 0.95 {
        1.7
    } else if absolute_shortfall >= 12.0 || proportional_shortfall >= 0.85 {
        1.2
    } else {
        1.0
    }
}

/// Effective ICER after the severity weight multiplies the health gain.
///
/// Weighting the QALY gain by `w` is equivalent to dividing the ICER by `w`:
/// the same technology looks `w` times cheaper per (weighted) QALY.
///
/// # Arguments
///
/// * `icer` — the unweighted ICER, £ per QALY (worked example: £26,000).
/// * `severity_weight` — the multiplier from [`severity_weight`] (1.0, 1.2,
///   or 1.7).
///
/// # Returns
///
/// `Some(icer / severity_weight)`, or `None` for a zero weight (division
/// undefined; valid NICE weights are never zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::qaly_shortfall_and_severity_modifiers::effective_icer;
///
/// // £26,000/QALY at ×1.2 → effectively ≈ £21,700/QALY — comfortably fundable.
/// let e = effective_icer(26_000.0, 1.2).unwrap();
/// assert!((e - 21_700.0).abs() < 100.0);
///
/// // A zero weight is undefined.
/// assert!(effective_icer(26_000.0, 0.0).is_none());
/// ```
pub fn effective_icer(icer: f64, severity_weight: f64) -> Option<f64> {
    if severity_weight == 0.0 {
        None
    } else {
        Some(icer / severity_weight)
    }
}

/// Equivalent view: the severity weight multiplies the threshold instead.
///
/// Effective λ = threshold × weight. This scales NICE's £20k–£30k band to
/// £24k–£36k at ×1.2 and £34k–£51k at ×1.7 (e.g. £30k × 1.7 = £51k).
///
/// # Arguments
///
/// * `threshold` — the unweighted willingness-to-pay threshold, £ per QALY.
/// * `severity_weight` — the multiplier from [`severity_weight`].
///
/// # Returns
///
/// The effective threshold in £ per QALY: `threshold × severity_weight`.
///
/// # Examples
///
/// ```rust
/// use health_economics::qaly_shortfall_and_severity_modifiers::effective_threshold;
///
/// // ×1.2 scales £20k–£30k to £24k–£36k; ×1.7 scales it to £34k–£51k.
/// assert_eq!(effective_threshold(20_000.0, 1.2), 24_000.0);
/// assert_eq!(effective_threshold(30_000.0, 1.2), 36_000.0);
/// assert_eq!(effective_threshold(20_000.0, 1.7), 34_000.0);
/// assert_eq!(effective_threshold(30_000.0, 1.7), 51_000.0);
/// ```
pub fn effective_threshold(threshold: f64, severity_weight: f64) -> f64 {
    threshold * severity_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Absolute shortfall = 14.2 − 2.1 = 12.1 (≥ 12 → qualifies for ×1.2).
    #[test]
    fn absolute_shortfall_is_12_point_1() {
        // Worked example: "Absolute shortfall = 14.2 − 2.1 = 12.1 (≥ 12 →
        // qualifies for ×1.2)."
        let a = absolute_shortfall(14.2, 2.1);
        assert!((a - 12.1).abs() < TOL);
        assert!(a >= 12.0);
    }

    /// Proportional shortfall = 12.1 / 14.2 = 0.852 (≥ 0.85 → also ×1.2).
    #[test]
    fn proportional_shortfall_is_0_852() {
        // Worked example: "Proportional shortfall = 12.1 / 14.2 = 0.852
        // (≥ 0.85 → also ×1.2)."
        let a = absolute_shortfall(14.2, 2.1);
        let p = proportional_shortfall(a, 14.2).unwrap();
        assert!((p - 0.852).abs() < 5e-4);
        assert!(p >= 0.85);
    }

    /// Both measures land in the ×1.2 band for the worked example.
    #[test]
    fn worked_example_qualifies_for_1_2_weight() {
        // Worked example: both shortfall measures qualify for the ×1.2 weight.
        let a = absolute_shortfall(14.2, 2.1);
        let p = proportional_shortfall(a, 14.2).unwrap();
        assert!((severity_weight(a, p) - 1.2).abs() < TOL);
    }

    /// With the ×1.2 weight, a £26,000/QALY ICER becomes effectively
    /// ≈ £21,700/QALY — comfortably fundable.
    #[test]
    fn effective_icer_is_about_21_700() {
        // Worked example: "With the ×1.2 weight: effective ICER =
        // 26,000 / 1.2 ≈ £21,700/QALY — comfortably fundable."
        let e = effective_icer(26_000.0, 1.2).unwrap();
        assert!((e - 26_000.0 / 1.2).abs() < TOL);
        assert!((e - 21_700.0).abs() < 100.0);
    }

    /// Effective λ becomes £24k–£36k at ×1.2 and £34k–£51k at ×1.7.
    #[test]
    fn effective_thresholds_match_nice_bands() {
        // Doc math: "effective λ becomes £24k–£36k at ×1.2 and £34k–£51k at ×1.7."
        assert!((effective_threshold(20_000.0, 1.2) - 24_000.0).abs() < TOL);
        assert!((effective_threshold(30_000.0, 1.2) - 36_000.0).abs() < TOL);
        assert!((effective_threshold(20_000.0, 1.7) - 34_000.0).abs() < TOL);
        assert!((effective_threshold(30_000.0, 1.7) - 51_000.0).abs() < TOL);
    }

    /// Weight bands: below both cut-offs ×1.0; at the top band ×1.7.
    #[test]
    fn weight_bands_cover_all_three_levels() {
        // Doc math: "Weight ×1.0: absolute < 12 and proportional < 0.85;
        // ×1.2: absolute ≥ 12 or proportional ≥ 0.85; ×1.7: absolute ≥ 18 or
        // proportional ≥ 0.95."
        assert!((severity_weight(5.0, 0.40) - 1.0).abs() < TOL);
        assert!((severity_weight(12.0, 0.50) - 1.2).abs() < TOL);
        assert!((severity_weight(5.0, 0.85) - 1.2).abs() < TOL);
        assert!((severity_weight(18.0, 0.50) - 1.7).abs() < TOL);
        assert!((severity_weight(5.0, 0.95) - 1.7).abs() < TOL);
    }

    // Edge case: zero general-population QALYs leaves the fraction undefined.
    #[test]
    fn zero_general_population_qalys_is_undefined() {
        assert!(proportional_shortfall(1.0, 0.0).is_none());
    }
}
