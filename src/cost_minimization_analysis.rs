//! # Cost-Minimization Analysis (CMA)
//!
//! CMA compares only costs, and picks the cheapest option — legitimate
//! *only* when the outcomes of the alternatives have been demonstrated to be
//! equivalent.
//!
//! The rigor lives in *proving* equivalence first — typically via a
//! non-inferiority study with a pre-specified margin δ — which is exactly
//! the step buyers usually skip. If equivalence cannot be evidenced, CMA is
//! invalid: use CEA/CUA instead.
//!
//! ## Formula
//!
//! ```text
//! Given evidence that Effect_A ≈ Effect_B (within a pre-specified margin δ):
//! Choose min(Cost_A, Cost_B)
//!
//! Cost_A, Cost_B — total costs from the same perspective, over the same
//!                  horizon, including switching/transition costs
//! δ              — equivalence margin agreed BEFORE the pilot, in the
//!                  outcome's own units
//! ```
//!
//! ## Why it matters
//!
//! CMA is the simplest analysis and the most abused. The equivalence claim
//! is doing all the work: if outcomes genuinely don't differ (a biosimilar
//! vs its originator; two suppliers of the same service meeting the same
//! specification), then cost is the only question and CMA is correct.
//! Without the pilot, the equivalence claim rests on vendor brochures — and
//! a 1-point completion-rate difference (≈ thousands of failed
//! consultations/year) would dwarf a £50,000 price advantage.
//!
//! ## Example
//!
//! A trust chooses between two video-consultation platforms. A 3-month
//! parallel pilot shows completion rates 94.1% vs 93.8% and satisfaction
//! 4.4 vs 4.4 — inside the pre-agreed δ of 2 percentage points. Over 3
//! years Platform A totals £500,000 and Platform B £450,000: B wins by
//! £50,000, *including* its higher integration cost.
//!
//! ```rust
//! use health_economics::cost_minimization_analysis::{
//!     CostLines, Selection, cost_minimization, cost_saving, outcomes_equivalent,
//! };
//!
//! // Pilot: completion 94.1% vs 93.8%, satisfaction 4.4 vs 4.4, δ = 2pp.
//! let equivalent = outcomes_equivalent(94.1, 93.8, 2.0)
//!     && outcomes_equivalent(4.4, 4.4, 2.0);
//! assert!(equivalent);
//!
//! // 3-year costs, including switching costs.
//! let a = CostLines { licences: 360_000.0, integration: 80_000.0, training_support: 60_000.0 };
//! let b = CostLines { licences: 210_000.0, integration: 150_000.0, training_support: 90_000.0 };
//! assert!((a.total() - 500_000.0).abs() < 1e-6);
//! assert!((b.total() - 450_000.0).abs() < 1e-6);
//!
//! // Platform B wins by £50,000 — including its higher integration cost.
//! let choice = cost_minimization(a.total(), b.total(), equivalent);
//! assert_eq!(choice, Some(Selection::OptionB));
//! assert!((cost_saving(a.total(), b.total()) - 50_000.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - CMA is the formal shape of commodity procurement: two CI providers
//!   meeting identical SLOs, two object stores with the same durability
//!   spec.
//! - The lesson is the *order of operations*: first evidence equivalence
//!   (benchmark against your workload, pilot against your SLOs, with the
//!   margin agreed in advance), then compare total costs including
//!   migration.
//! - "They're basically the same, B is cheaper" without the first step is
//!   how orgs buy the tool that's 10% cheaper and 40% worse.
//! - Corollary: when a vendor argues price, make them stipulate equivalence
//!   — it's binding in the other direction too.
//!
//! ## Pitfalls
//!
//! - **Assumed equivalence** — the defining sin; absence of evidence of
//!   difference is not evidence of equivalence (underpowered pilots "show"
//!   equivalence for free).
//! - **Omitting switching costs** — migration, retraining, and parallel
//!   running belong in the cost side.
//! - **Equivalence on the wrong outcomes**: equivalent on the measured
//!   metric, different on one that matters (accessibility, tail latency,
//!   data egress).
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: cost-minimisation analysis.
//!   <https://yhec.co.uk/glossary/cost-minimisation-analysis/>
//! - Briggs AH, O'Brien BJ. "The death of cost-minimization analysis?"
//!   Health Economics 2001. <https://pubmed.ncbi.nlm.nih.gov/11288052/>
//!
//! Topic doc: health-economics-metrics/topics/cost-minimization-analysis.md

/// Which option a valid CMA selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Option A is cheapest (or the tie-break winner on equal cost).
    OptionA,
    /// Option B is strictly cheapest.
    OptionB,
}

/// True when two outcomes are equivalent within the pre-specified margin δ:
/// |effect_a − effect_b| ≤ margin.
///
/// The margin must be agreed before the pilot, not fitted afterwards, and
/// all three arguments share the outcome's own units (percentage points,
/// score points, ...). Absence of evidence of difference is not evidence of
/// equivalence — the pilot must be powered for the margin.
///
/// # Arguments
///
/// * `effect_a` — outcome of option A.
/// * `effect_b` — outcome of option B, same units.
/// * `margin` — the pre-specified equivalence margin δ, same units.
///
/// # Returns
///
/// `true` if the absolute difference is within the margin.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_minimization_analysis::outcomes_equivalent;
///
/// // Completion rates 94.1% vs 93.8% inside the pre-agreed δ of 2pp.
/// assert!(outcomes_equivalent(94.1, 93.8, 2.0));
///
/// // A 3-point gap would fail the same margin.
/// assert!(!outcomes_equivalent(94.1, 91.0, 2.0));
/// ```
pub fn outcomes_equivalent(effect_a: f64, effect_b: f64, margin: f64) -> bool {
    (effect_a - effect_b).abs() <= margin
}

/// Total cost of one option over the horizon, including switching costs
/// (licences + integration + training/support in the worked example).
///
/// All lines are in the same currency over the same horizon; omitting the
/// switching lines (integration, retraining, parallel running) is one of
/// the canonical CMA pitfalls.
#[derive(Debug, Clone, Copy)]
pub struct CostLines {
    /// Licence costs over the horizon.
    pub licences: f64,
    /// Integration/migration costs (switching costs belong in CMA).
    pub integration: f64,
    /// Training and support costs over the horizon.
    pub training_support: f64,
}

impl CostLines {
    /// Total cost = licences + integration + training/support.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cost_minimization_analysis::CostLines;
    ///
    /// // Platform B: £210k licences + £150k integration + £90k training = £450k.
    /// let b = CostLines { licences: 210_000.0, integration: 150_000.0, training_support: 90_000.0 };
    /// assert!((b.total() - 450_000.0).abs() < 1e-6);
    /// ```
    pub fn total(&self) -> f64 {
        self.licences + self.integration + self.training_support
    }
}

/// Run the CMA decision: pick the cheaper option, but only if equivalence
/// was evidenced first.
///
/// Encodes the order of operations that defines a valid CMA: the
/// equivalence evidence gates the cost comparison.
///
/// # Arguments
///
/// * `cost_a` — total cost of option A (including switching costs).
/// * `cost_b` — total cost of option B, same perspective and horizon.
/// * `equivalence_evidenced` — whether outcome equivalence was demonstrated
///   within the pre-specified margin (e.g. via a non-inferiority pilot).
///
/// # Returns
///
/// `None` if equivalence was not evidenced (CMA is then invalid — use
/// CEA/CUA instead); otherwise `Some` of the cheaper option, with A winning
/// ties.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_minimization_analysis::{
///     Selection, cost_minimization,
/// };
///
/// // Equivalence evidenced: B's £450k beats A's £500k.
/// assert_eq!(
///     cost_minimization(500_000.0, 450_000.0, true),
///     Some(Selection::OptionB)
/// );
///
/// // No evidenced equivalence → CMA invalid, no selection.
/// assert!(cost_minimization(500_000.0, 450_000.0, false).is_none());
/// ```
pub fn cost_minimization(
    cost_a: f64,
    cost_b: f64,
    equivalence_evidenced: bool,
) -> Option<Selection> {
    // Equivalence gates the whole analysis: without it there is no valid CMA.
    if !equivalence_evidenced {
        return None;
    }
    Some(if cost_b < cost_a { Selection::OptionB } else { Selection::OptionA })
}

/// Cost saving of choosing the cheaper option: |cost_a − cost_b|.
///
/// # Arguments
///
/// * `cost_a` — total cost of option A.
/// * `cost_b` — total cost of option B, same currency and horizon.
///
/// # Returns
///
/// The absolute cost difference (always non-negative).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_minimization_analysis::cost_saving;
///
/// // Platform B wins by £50,000 (£500k vs £450k).
/// assert!((cost_saving(500_000.0, 450_000.0) - 50_000.0).abs() < 1e-6);
/// ```
pub fn cost_saving(cost_a: f64, cost_b: f64) -> f64 {
    (cost_a - cost_b).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: two video-consultation platforms; 3-month parallel
    // pilot with a pre-agreed δ of 2 percentage points; 3-year costs.

    fn platform_a() -> CostLines {
        CostLines { licences: 360_000.0, integration: 80_000.0, training_support: 60_000.0 }
    }

    fn platform_b() -> CostLines {
        CostLines { licences: 210_000.0, integration: 150_000.0, training_support: 90_000.0 }
    }

    // Worked-example line: "completion rates 94.1% vs 93.8% ... inside the
    // pre-agreed δ of 2 percentage points".
    #[test]
    fn completion_rates_are_equivalent_within_2_points() {
        // 94.1% vs 93.8%, δ = 2pp.
        assert!(outcomes_equivalent(94.1, 93.8, 2.0));
    }

    // Worked-example line: "patient satisfaction 4.4 vs 4.4".
    #[test]
    fn satisfaction_is_equivalent() {
        // 4.4 vs 4.4.
        assert!(outcomes_equivalent(4.4, 4.4, 2.0));
    }

    // Worked-example table: Platform A total £500,000 (£360k + £80k + £60k).
    #[test]
    fn platform_a_total_is_500k() {
        assert!((platform_a().total() - 500_000.0).abs() < 1e-6);
    }

    // Worked-example table: Platform B total £450,000 (£210k + £150k + £90k).
    #[test]
    fn platform_b_total_is_450k() {
        assert!((platform_b().total() - 450_000.0).abs() < 1e-6);
    }

    // Worked-example line: "Platform B wins by £50,000 — *including* its
    // higher integration cost".
    #[test]
    fn platform_b_wins_by_50k_including_higher_integration_cost() {
        let equivalent = outcomes_equivalent(94.1, 93.8, 2.0)
            && outcomes_equivalent(4.4, 4.4, 2.0);
        let choice = cost_minimization(platform_a().total(), platform_b().total(), equivalent);
        assert_eq!(choice, Some(Selection::OptionB));
        assert!((cost_saving(platform_a().total(), platform_b().total()) - 50_000.0).abs() < 1e-6);
        // B wins despite integration costing £150k vs A's £80k.
        assert!(platform_b().integration > platform_a().integration);
    }

    // Doc rule: "If equivalence cannot be evidenced, CMA is invalid — use
    // CEA/CUA instead" (a 1-point completion-rate difference outside δ
    // would dwarf £50,000).
    #[test]
    fn cma_is_invalid_without_evidenced_equivalence() {
        // A 1-point completion-rate difference outside δ would invalidate CMA.
        assert!(cost_minimization(500_000.0, 450_000.0, false).is_none());
    }
}
