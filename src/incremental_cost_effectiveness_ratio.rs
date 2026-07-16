//! # Incremental Cost-Effectiveness Ratio (ICER)
//!
//! The ICER is the extra cost per extra unit of health effect when you choose
//! one option over the next-best alternative. It is the headline number of
//! health technology assessment. (When the effect unit is QALYs, it is also
//! called the incremental cost-utility ratio, ICUR.)
//!
//! The comparator must be the *next-best non-dominated option*, not
//! "do nothing" — health systems never evaluate a technology in isolation,
//! always incrementally, against what would otherwise be done.
//!
//! ## Formula
//!
//! ```text
//! ICER = (Cost_new − Cost_comparator) / (Effect_new − Effect_comparator)
//!      = ΔC / ΔE
//!
//! ΔC = incremental cost of the new option versus the comparator
//! ΔE = incremental effect (e.g. QALYs) versus the comparator
//! λ  = willingness-to-pay threshold
//!
//! Rules of interpretation:
//!   ΔC < 0, ΔE > 0: new option dominates (cheaper and better; no ratio needed)
//!   ΔC > 0, ΔE > 0: compute ICER; adopt if ICER < λ
//!   ΔC > 0, ΔE < 0: new option is dominated — reject
//! ```
//!
//! ## Why it matters
//!
//! Whether your product is "worth it" to a national health service is,
//! formally, whether its ICER clears the local threshold. NICE compares a
//! technology's ICER to its **£20,000–£30,000 per QALY** threshold; the US
//! ICER institute reports across $50,000–$200,000/QALY; Canada works to
//! roughly CAD$50,000/QALY.
//!
//! ## Example
//!
//! A remote-monitoring service for heart-failure patients, per 1,000
//! patients/year, versus usual care: service costs £900,000, avoided
//! admissions save £600,000, and earlier intervention gains 25 QALYs.
//!
//! ```rust
//! use health_economics::incremental_cost_effectiveness_ratio::{
//!     adopt_at_threshold, classify_quadrant, icer, net_incremental_cost,
//!     CostEffectivenessQuadrant,
//! };
//!
//! // ΔC = 900,000 − 600,000 = £300,000.
//! let dc = net_incremental_cost(900_000.0, 600_000.0);
//! assert!((dc - 300_000.0).abs() < 1e-9);
//!
//! // ICER = 300,000 / 25 = £12,000 per QALY — below NICE's £20,000.
//! let ratio = icer(dc, 25.0).unwrap();
//! assert!((ratio - 12_000.0).abs() < 1e-9);
//! assert!(adopt_at_threshold(ratio, 20_000.0));
//! assert_eq!(classify_quadrant(dc, 25.0), CostEffectivenessQuadrant::TradeOff);
//!
//! // Without the £600,000 offset the ICER would be £36,000/QALY and fail.
//! let gross = icer(900_000.0, 25.0).unwrap();
//! assert!((gross - 36_000.0).abs() < 1e-9);
//! assert!(!adopt_at_threshold(gross, 20_000.0));
//! ```
//!
//! ## Software engineering connection
//!
//! - The discipline transfers wholesale:
//!   `(cost of option B − cost of option A) / (outcome B − outcome A)` —
//!   incremental cost per additional deploy, per engineer-hour saved, per
//!   incident avoided.
//! - Always compare against the next-best alternative, not against doing
//!   nothing.
//! - *Name the comparator explicitly* — most tool ROI claims secretly
//!   compare against a strawman.
//! - *Net the costs first* — a tool that costs £100k but displaces £80k of
//!   existing spend has ΔC = £20k.
//!
//! ## Pitfalls
//!
//! - **Comparator gaming**: comparing against an obsolete or artificially bad
//!   baseline inflates ΔE and flatters the ICER.
//! - **Averages instead of increments**: cost per QALY of a whole program is
//!   not the ICER of expanding or adopting it.
//! - **Point-estimate worship**: ICERs are ratios of two uncertain
//!   differences; report uncertainty via PSA and CEACs.
//! - **Negative ICERs are ambiguous** (cheaper-and-better vs
//!   costlier-and-worse give the same sign) — never report a negative ICER
//!   without saying which quadrant it is.
//!
//! ## Sources
//!
//! - NICE: cost-effectiveness thresholds FAQ.
//!   <https://www.nice.org.uk/what-nice-does/faqs/changes-to-nice-s-cost-effectiveness-thresholds>
//! - ICER 2023 Value Assessment Framework.
//!   <https://icer.org/wp-content/uploads/2023/09/ICER_2023_VAF_For-Publication_092523.pdf>
//! - York Health Economics Consortium glossary: ICER.
//!   <https://yhec.co.uk/glossary/incremental-cost-effectiveness-ratio-icer/>
//!
//! Topic doc: health-economics-metrics/topics/incremental-cost-effectiveness-ratio.md

/// Where an option lands on the cost-effectiveness plane relative to its comparator.
///
/// The plane's x-axis is ΔE (incremental effect) and y-axis is ΔC
/// (incremental cost). Negative ICERs are ambiguous — cheaper-and-better and
/// costlier-and-worse give the same sign — so the quadrant, not the ratio,
/// carries the decision.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CostEffectivenessQuadrant {
    /// Cheaper (ΔC ≤ 0) and more effective (ΔE > 0): adopt without computing
    /// a ratio — the new option dominates its comparator.
    Dominant,
    /// Costlier (ΔC > 0) and more effective (ΔE > 0): the only quadrant where
    /// the ICER is meaningful — compute it and compare to the threshold λ.
    TradeOff,
    /// Costlier (ΔC > 0) and less effective (ΔE < 0): reject without
    /// computing a ratio — the new option is dominated.
    Dominated,
    /// Cheaper (ΔC ≤ 0) and less effective (ΔE < 0): a disinvestment
    /// trade-off — is the saving worth the health forgone?
    SavingsForLoss,
    /// ΔE = 0 (or ΔC = 0 with ΔE = 0): the ratio is undefined or degenerate;
    /// decide on cost alone (cost-minimization) or use net monetary benefit.
    OnAxis,
}

/// Classify the (ΔC, ΔE) pair into its cost-effectiveness quadrant.
///
/// Conventions: ΔE = 0 is always [`CostEffectivenessQuadrant::OnAxis`]
/// (the ratio is undefined there); ΔC = 0 with ΔE > 0 counts as
/// [`CostEffectivenessQuadrant::Dominant`] (better at no extra cost).
///
/// # Arguments
///
/// * `delta_cost` — incremental cost ΔC (currency).
/// * `delta_effect` — incremental effect ΔE (e.g. QALYs).
///
/// # Returns
///
/// The [`CostEffectivenessQuadrant`] the pair falls into.
///
/// # Examples
///
/// ```rust
/// use health_economics::incremental_cost_effectiveness_ratio::{
///     classify_quadrant, CostEffectivenessQuadrant,
/// };
///
/// // Remote monitoring: ΔC = £300,000, ΔE = 25 QALYs — a trade-off.
/// assert_eq!(classify_quadrant(300_000.0, 25.0), CostEffectivenessQuadrant::TradeOff);
/// // Cheaper and better dominates; costlier and worse is dominated.
/// assert_eq!(classify_quadrant(-50_000.0, 10.0), CostEffectivenessQuadrant::Dominant);
/// assert_eq!(classify_quadrant(50_000.0, -5.0), CostEffectivenessQuadrant::Dominated);
/// // Zero effect puts the pair on the axis: no meaningful ratio.
/// assert_eq!(classify_quadrant(300_000.0, 0.0), CostEffectivenessQuadrant::OnAxis);
/// ```
pub fn classify_quadrant(delta_cost: f64, delta_effect: f64) -> CostEffectivenessQuadrant {
    // Order matters: the ΔE = 0 axis is carved out first because the ratio
    // is undefined there regardless of ΔC's sign.
    if delta_effect == 0.0 {
        CostEffectivenessQuadrant::OnAxis
    } else if delta_cost <= 0.0 && delta_effect > 0.0 {
        // ΔC ≤ 0, ΔE > 0: at-worst-free and better — dominant.
        CostEffectivenessQuadrant::Dominant
    } else if delta_cost > 0.0 && delta_effect > 0.0 {
        // ΔC > 0, ΔE > 0: the only quadrant where computing ΔC/ΔE decides.
        CostEffectivenessQuadrant::TradeOff
    } else if delta_cost > 0.0 {
        // Remaining ΔE < 0 cases: costlier and worse — dominated.
        CostEffectivenessQuadrant::Dominated
    } else {
        // ΔC ≤ 0, ΔE < 0: cheaper but worse — a disinvestment trade-off.
        CostEffectivenessQuadrant::SavingsForLoss
    }
}

/// ICER = ΔC / ΔE, the incremental cost per incremental unit of effect.
///
/// Units are currency per effect unit (e.g. £/QALY). Ratios behave badly near
/// zero effect — prefer net monetary benefit for ranking — and a negative
/// result is ambiguous without the quadrant (see [`classify_quadrant`]).
///
/// # Arguments
///
/// * `delta_cost` — incremental cost ΔC versus the next-best comparator (currency).
/// * `delta_effect` — incremental effect ΔE versus that comparator (e.g. QALYs).
///
/// # Returns
///
/// `Some(ΔC / ΔE)`, or `None` when `delta_effect` is exactly zero (the ratio
/// is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::incremental_cost_effectiveness_ratio::icer;
///
/// // ICER = 300,000 / 25 = £12,000 per QALY.
/// let ratio = icer(300_000.0, 25.0).unwrap();
/// assert!((ratio - 12_000.0).abs() < 1e-9);
///
/// // Undefined at ΔE = 0.
/// assert!(icer(300_000.0, 0.0).is_none());
/// ```
pub fn icer(delta_cost: f64, delta_effect: f64) -> Option<f64> {
    if delta_effect == 0.0 {
        None
    } else {
        Some(delta_cost / delta_effect)
    }
}

/// Incremental cost ΔC: gross cost of the new option minus the costs it offsets.
///
/// Offsets are real displaced spend — avoided admissions, retired systems —
/// evidenced, not assumed. Netting the costs first is where these analyses
/// are won and lost: without the offset the same intervention can fail the
/// threshold.
///
/// # Arguments
///
/// * `gross_cost` — full cost of the new option (currency).
/// * `cost_offsets` — costs the option avoids or displaces (currency, same
///   period and population as `gross_cost`).
///
/// # Returns
///
/// The net incremental cost `gross_cost − cost_offsets`; negative when
/// offsets exceed the gross cost (a candidate for dominance).
///
/// # Examples
///
/// ```rust
/// use health_economics::incremental_cost_effectiveness_ratio::net_incremental_cost;
///
/// // Service £900,000; admissions avoided save £600,000 → ΔC = £300,000.
/// let dc = net_incremental_cost(900_000.0, 600_000.0);
/// assert!((dc - 300_000.0).abs() < 1e-9);
/// ```
pub fn net_incremental_cost(gross_cost: f64, cost_offsets: f64) -> f64 {
    gross_cost - cost_offsets
}

/// Decision rule in the trade-off quadrant: adopt if ICER < λ.
///
/// Only meaningful when the option is in
/// [`CostEffectivenessQuadrant::TradeOff`] (ΔC > 0, ΔE > 0); dominant and
/// dominated options are decided by their quadrant alone. The comparison is
/// strict.
///
/// # Arguments
///
/// * `icer_value` — the computed ICER (e.g. £/QALY).
/// * `lambda` — willingness-to-pay threshold λ (e.g. £20,000/QALY at NICE).
///
/// # Returns
///
/// `true` when `icer_value < lambda`.
///
/// # Examples
///
/// ```rust
/// use health_economics::incremental_cost_effectiveness_ratio::adopt_at_threshold;
///
/// // £12,000/QALY is comfortably below NICE's £20,000 threshold.
/// assert!(adopt_at_threshold(12_000.0, 20_000.0));
/// // £36,000/QALY (the un-netted version of the same case) fails.
/// assert!(!adopt_at_threshold(36_000.0, 20_000.0));
/// ```
pub fn adopt_at_threshold(icer_value: f64, lambda: f64) -> bool {
    icer_value < lambda
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ΔC = 900,000 − 600,000 = £300,000.
    #[test]
    fn remote_monitoring_net_incremental_cost_is_300k() {
        // Worked example: "ΔC = 900,000 − 600,000 = £300,000".
        let dc = net_incremental_cost(900_000.0, 600_000.0);
        assert!((dc - 300_000.0).abs() < 1e-9);
    }

    /// ICER = 300,000 / 25 = £12,000 per QALY.
    #[test]
    fn remote_monitoring_icer_is_12000_per_qaly() {
        // Worked example: "ICER = 300,000 / 25 = £12,000 per QALY".
        let dc = net_incremental_cost(900_000.0, 600_000.0);
        let got = icer(dc, 25.0).unwrap();
        assert!((got - 12_000.0).abs() < 1e-9);
    }

    /// £12,000/QALY is comfortably below NICE's £20,000 threshold.
    #[test]
    fn icer_12000_is_below_nice_20k_threshold() {
        // Worked example: "£12,000/QALY is comfortably below NICE's £20,000
        // threshold — a strong case".
        assert!(adopt_at_threshold(12_000.0, 20_000.0));
    }

    /// Without the £600,000 offset the ICER would be £36,000/QALY and the
    /// case would likely fail at £20k.
    #[test]
    fn without_offset_icer_is_36000_and_fails() {
        // Worked example: "without the £600,000 offset the ICER would be
        // £36,000/QALY and the case would likely fail".
        let got = icer(900_000.0, 25.0).unwrap();
        assert!((got - 36_000.0).abs() < 1e-9);
        assert!(!adopt_at_threshold(got, 20_000.0));
    }

    /// The costlier-and-better case is a trade-off; cheaper-and-better
    /// dominates; costlier-and-worse is dominated.
    #[test]
    fn quadrant_rules_of_interpretation() {
        // Doc's rules of interpretation: ΔC>0,ΔE>0 → compute ICER;
        // ΔC<0,ΔE>0 → dominates; ΔC>0,ΔE<0 → dominated.
        assert_eq!(
            classify_quadrant(300_000.0, 25.0),
            CostEffectivenessQuadrant::TradeOff
        );
        assert_eq!(
            classify_quadrant(-50_000.0, 10.0),
            CostEffectivenessQuadrant::Dominant
        );
        assert_eq!(
            classify_quadrant(50_000.0, -5.0),
            CostEffectivenessQuadrant::Dominated
        );
    }

    /// The ratio is undefined at ΔE = 0.
    #[test]
    fn icer_is_undefined_at_zero_effect() {
        // Doc: "Ratios behave badly near ΔE = 0 — prefer net monetary
        // benefit for ranking".
        assert!(icer(300_000.0, 0.0).is_none());
        assert_eq!(
            classify_quadrant(300_000.0, 0.0),
            CostEffectivenessQuadrant::OnAxis
        );
    }
}
