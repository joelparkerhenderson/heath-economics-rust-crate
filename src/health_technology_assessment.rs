//! # Health Technology Assessment (HTA)
//!
//! HTA is the formal, institutionalized process by which health systems decide
//! whether a technology — drug, device, or software — is worth paying for. It
//! combines clinical-effectiveness evidence with economic evaluation under a
//! published, mandatory methodology.
//!
//! HTA's power is not a formula but a *mandated method*: every submission
//! computes the same ICER under the same reference-case rules (outcome
//! measure, utility instrument, perspective, comparator selection, discount
//! rate, time horizon, uncertainty analysis), so results are comparable
//! across products and years, and gaming is visible.
//!
//! ## Formula
//!
//! ```text
//! ICER = ΔC / ΔE            (computed under the reference case)
//! Adopt (routine commissioning) if ICER < threshold λ
//! PSA: probability cost-effective = fraction of draws with NMB > 0 at λ
//!
//! ΔC  = incremental cost versus the reference-case comparator
//! ΔE  = incremental effect (e.g. QALYs) versus that comparator
//! λ   = willingness-to-pay threshold (e.g. NICE's £20,000–£30,000/QALY)
//! NMB = net monetary benefit, ΔE × λ − ΔC
//! ```
//!
//! ## Why it matters
//!
//! If you sell into a national health service, an HTA body may literally
//! decide your market access — it is your real regulator-of-value. NICE
//! (England) runs statutory appraisals under a defined reference case (QALYs
//! from EQ-5D, NHS+PSS perspective, 3.5% discounting, PSA required) judged
//! against £20k–£30k/QALY, with severity modifiers and up to £100k+ with
//! weighting for highly specialised technologies. The US ICER institute
//! publishes health-benefit price benchmarks at $100k–$150k per QALY/evLYG;
//! Canada (CADTH → CDA-AMC) reviews at ≈CAD$50k/QALY and historically
//! requested price cuts in ~95% of submissions.
//!
//! ## Example
//!
//! A digital therapeutic submits to NICE-style evaluation: ΔC = +£450/patient,
//! ΔE = +0.03 QALYs → ICER = £15,000/QALY, under the £20k threshold, with a
//! conformant reference case and 71% probability cost-effective at £20k.
//!
//! ```rust
//! use health_economics::health_technology_assessment::{
//!     icer, meets_threshold, probability_cost_effective,
//!     recommend_routine_commissioning, ReferenceCaseChecklist,
//! };
//!
//! // ΔC = +£450/patient, ΔE = +0.03 QALYs → ICER = £15,000/QALY.
//! let ratio = icer(450.0, 0.03).unwrap();
//! assert!((ratio - 15_000.0).abs() < 1e-9);
//! assert!(meets_threshold(ratio, 20_000.0));
//!
//! // PSA: 71 of 100 draws favorable at λ = £20k → 71% cost-effective.
//! let mut draws = vec![(450.0, 0.03); 71];
//! draws.extend(vec![(450.0, 0.01); 29]);
//! let p = probability_cost_effective(&draws, 20_000.0).unwrap();
//! assert!((p - 0.71).abs() < 1e-9);
//!
//! // Reference-case checks all pass → routine commissioning recommended.
//! let checklist = ReferenceCaseChecklist {
//!     utilities_from_mandated_instrument: true,
//!     comparator_is_current_care_pathway: true,
//!     psa_reported: true,
//! };
//! assert!(recommend_routine_commissioning(&checklist, ratio, 20_000.0));
//! ```
//!
//! ## Software engineering connection
//!
//! - The transferable artifact is the **internal reference case**: one
//!   mandated method for all tooling/platform business cases.
//! - Mandate a declared comparator, standard unit costs, a fixed discount
//!   rate, required sensitivity analysis, and a standard template.
//! - An "AMCP-dossier for tools" submitted to a platform council makes
//!   proposals comparable and gaming visible, exactly as HTA does for
//!   medicine.
//! - Start smaller than NICE did: a two-page template plus a published price
//!   book beats no standard at all.
//!
//! ## Pitfalls
//!
//! - **Treating HTA as a formality after regulatory clearance** — CE/UKCA/FDA
//!   clearance says a product is safe; HTA decides if it's *worth buying*.
//!   Different hurdle, different evidence.
//! - **Building the economic model after the trial** — evidence generation
//!   should be designed backwards from the reference case's requirements.
//! - **Ignoring jurisdiction differences**: an ICER fundable in the US at
//!   $120k/QALY fails NICE at £30k; plan evidence and pricing per market.
//!
//! ## Sources
//!
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//! - ICER 2023 Value Assessment Framework.
//!   <https://icer.org/our-approach/methods-process/value-assessment-framework/>
//! - Canada's Drug Agency (CDA-AMC). <https://www.cda-amc.ca/>
//!
//! Topic doc: health-economics-metrics/topics/health-technology-assessment.md

/// Incremental cost-effectiveness ratio ΔC / ΔE — the headline number of every HTA submission.
///
/// Both inputs are incremental versus the reference-case comparator: cost in
/// currency units (e.g. £ per patient), effect in outcome units (e.g. QALYs
/// per patient). The ratio's units are currency per outcome unit (£/QALY).
///
/// # Arguments
///
/// * `delta_cost` — incremental cost ΔC versus the comparator (currency, e.g. £/patient).
/// * `delta_effect` — incremental effect ΔE versus the comparator (e.g. QALYs/patient).
///
/// # Returns
///
/// `Some(ΔC / ΔE)`, or `None` when `delta_effect` is exactly zero (the ratio
/// is undefined at zero incremental effect).
///
/// # Examples
///
/// ```rust
/// use health_economics::health_technology_assessment::icer;
///
/// // ΔC = +£450/patient, ΔE = +0.03 QALYs → ICER = £15,000/QALY.
/// let ratio = icer(450.0, 0.03).unwrap();
/// assert!((ratio - 15_000.0).abs() < 1e-9);
///
/// // Undefined at zero incremental effect.
/// assert!(icer(450.0, 0.0).is_none());
/// ```
pub fn icer(delta_cost: f64, delta_effect: f64) -> Option<f64> {
    if delta_effect == 0.0 {
        None
    } else {
        Some(delta_cost / delta_effect)
    }
}

/// Threshold decision: does the ICER clear the willingness-to-pay threshold?
///
/// Both arguments share the same units (currency per outcome unit, e.g.
/// £/QALY). The comparison is strict: an ICER exactly equal to the threshold
/// does not clear it.
///
/// # Arguments
///
/// * `icer_value` — the computed ICER (e.g. £/QALY).
/// * `threshold` — the willingness-to-pay threshold λ (e.g. NICE's
///   £20,000–£30,000 per QALY).
///
/// # Returns
///
/// `true` when `icer_value < threshold`.
///
/// # Examples
///
/// ```rust
/// use health_economics::health_technology_assessment::meets_threshold;
///
/// // £15,000/QALY is under the £20k threshold.
/// assert!(meets_threshold(15_000.0, 20_000.0));
/// assert!(!meets_threshold(36_000.0, 20_000.0));
/// ```
pub fn meets_threshold(icer_value: f64, threshold: f64) -> bool {
    icer_value < threshold
}

/// Probability the technology is cost-effective at threshold `lambda`, from PSA draws.
///
/// Takes pre-drawn probabilistic-sensitivity-analysis samples of (ΔC, ΔE) and
/// returns the fraction of draws whose net monetary benefit `ΔE × λ − ΔC` is
/// strictly positive — the value a cost-effectiveness acceptability curve
/// plots at one λ.
///
/// # Arguments
///
/// * `psa_draws` — sampled (ΔC, ΔE) pairs, one per Monte Carlo draw
///   (cost in currency, effect in outcome units such as QALYs).
/// * `lambda` — willingness-to-pay threshold λ (currency per outcome unit).
///
/// # Returns
///
/// `Some(fraction in [0, 1])`, or `None` for an empty sample set.
///
/// # Examples
///
/// ```rust
/// use health_economics::health_technology_assessment::probability_cost_effective;
///
/// // 71 of 100 draws favorable at £20k → 71% probability cost-effective.
/// let mut draws = vec![(450.0, 0.03); 71]; // NMB = 600 − 450 > 0
/// draws.extend(vec![(450.0, 0.01); 29]);   // NMB = 200 − 450 < 0
/// let p = probability_cost_effective(&draws, 20_000.0).unwrap();
/// assert!((p - 0.71).abs() < 1e-9);
///
/// assert!(probability_cost_effective(&[], 20_000.0).is_none());
/// ```
pub fn probability_cost_effective(psa_draws: &[(f64, f64)], lambda: f64) -> Option<f64> {
    if psa_draws.is_empty() {
        return None;
    }
    // A draw is favorable when its net monetary benefit ΔE × λ − ΔC > 0,
    // which is the ICER < λ rule rewritten so it stays linear (no division,
    // so draws with ΔE near zero cannot blow up the statistic).
    let favorable = psa_draws
        .iter()
        .filter(|(dc, de)| de * lambda - dc > 0.0)
        .count();
    Some(favorable as f64 / psa_draws.len() as f64)
}

/// Reference-case conformance checklist for a NICE-style submission.
///
/// Each field is one of the mandated method choices the reference case
/// removes from the sponsor's discretion. A `false` anywhere means the
/// submission's ICER is not comparable with other reference-case results.
pub struct ReferenceCaseChecklist {
    /// Utilities measured with the mandated instrument (e.g. EQ-5D-5L with
    /// the UK value set), not a sponsor-chosen alternative.
    pub utilities_from_mandated_instrument: bool,
    /// Comparator is the current care pathway — the next-best real
    /// alternative — not "no treatment" or a strawman baseline.
    pub comparator_is_current_care_pathway: bool,
    /// Probabilistic sensitivity analysis performed and reported (e.g. as a
    /// probability cost-effective at the threshold).
    pub psa_reported: bool,
}

impl ReferenceCaseChecklist {
    /// Whether every mandated reference-case check passes.
    ///
    /// # Returns
    ///
    /// `true` only when all three checks are `true`; any single failure
    /// fails the whole checklist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::health_technology_assessment::ReferenceCaseChecklist;
    ///
    /// let conformant = ReferenceCaseChecklist {
    ///     utilities_from_mandated_instrument: true,
    ///     comparator_is_current_care_pathway: true,
    ///     psa_reported: true,
    /// };
    /// assert!(conformant.passes());
    ///
    /// // A strawman comparator fails the reference case.
    /// let strawman = ReferenceCaseChecklist {
    ///     utilities_from_mandated_instrument: true,
    ///     comparator_is_current_care_pathway: false,
    ///     psa_reported: true,
    /// };
    /// assert!(!strawman.passes());
    /// ```
    pub fn passes(&self) -> bool {
        self.utilities_from_mandated_instrument
            && self.comparator_is_current_care_pathway
            && self.psa_reported
    }
}

/// Appraisal outcome: recommend routine commissioning only when the submission
/// conforms to the reference case *and* its ICER clears the threshold.
///
/// A flattering ICER computed off the reference case (e.g. against a strawman
/// comparator) is rejected regardless of its value — that is the point of a
/// mandated method.
///
/// # Arguments
///
/// * `checklist` — the reference-case conformance checks for the submission.
/// * `icer_value` — the ICER computed under the reference case (£/QALY).
/// * `threshold` — the willingness-to-pay threshold λ (£/QALY).
///
/// # Returns
///
/// `true` when `checklist.passes()` and `icer_value < threshold`.
///
/// # Examples
///
/// ```rust
/// use health_economics::health_technology_assessment::{
///     recommend_routine_commissioning, ReferenceCaseChecklist,
/// };
///
/// let checklist = ReferenceCaseChecklist {
///     utilities_from_mandated_instrument: true,
///     comparator_is_current_care_pathway: true,
///     psa_reported: true,
/// };
/// // Reference-case ICER £15,000/QALY under the £20k threshold → recommended.
/// assert!(recommend_routine_commissioning(&checklist, 15_000.0, 20_000.0));
///
/// // The sponsor's flattering £9,000/QALY against a strawman comparator is
/// // rejected because the reference case fails.
/// let strawman = ReferenceCaseChecklist {
///     utilities_from_mandated_instrument: true,
///     comparator_is_current_care_pathway: false,
///     psa_reported: true,
/// };
/// assert!(!recommend_routine_commissioning(&strawman, 9_000.0, 20_000.0));
/// ```
pub fn recommend_routine_commissioning(
    checklist: &ReferenceCaseChecklist,
    icer_value: f64,
    threshold: f64,
) -> bool {
    checklist.passes() && meets_threshold(icer_value, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ΔC = +£450/patient, ΔE = +0.03 QALYs → ICER = £15,000/QALY.
    #[test]
    fn digital_therapeutic_icer_is_15000_per_qaly() {
        // Worked example: "ΔC = +£450/patient, ΔE = +0.03 QALYs → ICER = £15,000/QALY".
        let got = icer(450.0, 0.03).unwrap();
        assert!((got - 15_000.0).abs() < 1e-9);
    }

    /// £15,000/QALY is under the £20k threshold.
    #[test]
    fn icer_15000_clears_20k_threshold() {
        // Worked example: "£15,000/QALY ✓ under £20k".
        assert!(meets_threshold(15_000.0, 20_000.0));
    }

    /// Sponsor's own preferred analysis showed £9,000/QALY (same ΔE, lower
    /// net cost of £270/patient against the strawman comparator); the
    /// reference case's honest comparator pushed it to £15,000.
    #[test]
    fn sponsor_preferred_analysis_shows_9000_per_qaly() {
        // Worked example: "The sponsor's own preferred analysis showed
        // £9,000/QALY; the reference case pushed it to £15,000".
        let sponsor = icer(270.0, 0.03).unwrap();
        assert!((sponsor - 9_000.0).abs() < 1e-9);
        let reference_case = icer(450.0, 0.03).unwrap();
        assert!(reference_case > sponsor);
    }

    /// PSA: 71% probability cost-effective at £20k.
    #[test]
    fn psa_shows_71_percent_probability_cost_effective_at_20k() {
        // Worked example: "PSA: 71% probability cost-effective at £20k".
        // 100 pre-drawn (ΔC, ΔE) samples: 71 favorable at λ = £20k, 29 not.
        let mut draws = Vec::new();
        for _ in 0..71 {
            draws.push((450.0, 0.03)); // NMB = 600 − 450 > 0
        }
        for _ in 0..29 {
            draws.push((450.0, 0.01)); // NMB = 200 − 450 < 0
        }
        let p = probability_cost_effective(&draws, 20_000.0).unwrap();
        assert!((p - 0.71).abs() < 1e-9);
    }

    /// All reference-case checks pass → routine commissioning recommended.
    #[test]
    fn conformant_submission_under_threshold_is_recommended() {
        // Worked example: all reference-case checks ✓ → "Recommendation:
        // routine commissioning, with real-world data collection".
        let checklist = ReferenceCaseChecklist {
            utilities_from_mandated_instrument: true,
            comparator_is_current_care_pathway: true,
            psa_reported: true,
        };
        assert!(checklist.passes());
        assert!(recommend_routine_commissioning(&checklist, 15_000.0, 20_000.0));
    }

    /// A submission using the strawman comparator fails the reference case
    /// even with a flattering ICER.
    #[test]
    fn strawman_comparator_fails_reference_case() {
        // Worked example: "comparator = current care pathway (not 'no
        // treatment')" is mandated — the flattering £9,000/QALY is rejected.
        let checklist = ReferenceCaseChecklist {
            utilities_from_mandated_instrument: true,
            comparator_is_current_care_pathway: false,
            psa_reported: true,
        };
        assert!(!recommend_routine_commissioning(&checklist, 9_000.0, 20_000.0));
    }
}
