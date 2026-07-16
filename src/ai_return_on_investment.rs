//! # AI Return on Investment
//!
//! AI ROI is the measurable P&L return attributable to AI initiatives. The
//! sobering benchmark: MIT's 2025 "GenAI Divide" research found that despite
//! $30–40B of enterprise GenAI investment, ~95% of pilots showed no
//! measurable P&L return — and the successful 5% shared identifiable habits:
//! a baseline, an owner, and a budget line.
//!
//! The denominator is the full cost stack — licences/inference plus
//! integration, data readiness, evaluation, workflow redesign, and
//! governance/assurance — in which the licence is typically the minority.
//!
//! ## Formula
//!
//! ```text
//! AI ROI = (attributable benefit − total AI cost) / total AI cost
//!
//! Total AI cost = licences/inference + integration + data readiness
//!               + evaluation + workflow redesign + governance/assurance
//!               (the licence is typically the minority of the denominator)
//!
//! Attributable benefit: measured against a baseline or control, classed
//! cash / capacity / quality
//! ```
//!
//! Legend:
//! - `attributable benefit` — benefit demonstrably caused by the AI, measured
//!   against a baseline or control (currency).
//! - `total AI cost` — the six-line cost stack above (currency).
//! - ROI is a fraction: 1.67 ≈ 167%; −1.0 = total loss.
//!
//! ## Why it matters
//!
//! Health systems have a name for the AI-pilot pattern: **pilotitis** — the
//! NHS graveyard of promising apps piloted forever and scaled never. The MIT
//! findings map cleanly onto what health technology assessment already knows:
//! value claims need pre-specified endpoints, attribution needs comparators,
//! and "everyone feels it's helping" is not a benefit line. The successful
//! minority in the MIT data concentrated in back-office automation with
//! trackable cost baselines, and purchased tools succeeded ~67% of the time
//! versus internal builds at roughly a third of that — priors that belong in
//! every AI investment case.
//!
//! ## Example
//!
//! A hospital group deploys AI for two use cases. Use case A
//! (clinical-letter drafting, back office, trackable): a £380k/yr outsourced
//! transcription contract is cancelled, clinician review adds £60k, AI costs
//! £120k/yr all-in → ROI ≈ 167%, cash-releasing and auditable. Use case B
//! ("AI copilot for clinicians", broad, untracked): no baseline captured, no
//! demonstrable P&L effect — the 95% bucket, regardless of whether it
//! actually helps.
//!
//! ```rust
//! use health_economics::ai_return_on_investment::{
//!     ai_roi, attributable_benefit,
//! };
//!
//! // Use case A: £380k baseline stopped − £60k new review cost = £320k benefit.
//! let benefit = attributable_benefit(380_000.0, 60_000.0);
//! assert!((benefit - 320_000.0).abs() < 1e-9);
//!
//! // ROI = (380k − 60k − 120k) / 120k ≈ 167%.
//! let roi = ai_roi(benefit, 120_000.0).unwrap();
//! assert!((roi - 1.67).abs() < 0.01);
//!
//! // Use case B: no measured benefit → ROI = −100%.
//! let roi_b = ai_roi(0.0, 120_000.0).unwrap();
//! assert!((roi_b - (-1.0)).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Stage the evidence like NICE ESF tiers**: demo-grade evidence for
//!   low-stakes tools, controlled pilots before org-wide spend, rollout gates
//!   pre-registered (DiGA's provisional-listing-with-deadline pattern).
//! - **Count cost avoidance the way health economics counts demand
//!   avoidance** — real only when a specific budget line moves.
//! - **Price the pilot itself with EVPI** — a pilot that can't change the
//!   rollout decision is worth £0.
//! - For the developer-tools slice specifically, see AI developer
//!   productivity.
//!
//! ## Pitfalls
//!
//! - **Benefit diffusion**: value spread thin across thousands of users is
//!   unmeasurable by construction; pick use cases with concentrated,
//!   trackable baselines.
//! - **Licence-only costing**: integration, evaluation, and workflow redesign
//!   usually dominate the true denominator.
//! - **Attribution theft**: AI deployed alongside process redesign claims the
//!   whole delta.
//! - **Sunk-pilot escalation**: extending failed pilots because stopping
//!   admits failure — the sunset date must be pre-agreed.
//!
//! ## Sources
//!
//! - MIT Project NANDA "GenAI Divide" coverage.
//!   <https://fortune.com/2025/08/18/mit-report-95-percent-generative-ai-pilots-at-companies-failing-cfo/>
//! - MIT GenAI ROI findings summary.
//!   <https://blueflame.ai/blog/achieving-ai-roi-key-findings-from-mits-genai-report>
//! - MIT Technology Review, finding ROI on AI.
//!   <https://www.technologyreview.com/2025/10/28/1126693/finding-return-on-ai-investments-across-industries/>
//!
//! Topic doc: health-economics-metrics/topics/ai-return-on-investment.md

/// The full cost stack of an AI initiative, in currency units per period.
///
/// The true ROI denominator. The licence is typically the minority of the
/// total — integration, evaluation, and workflow redesign usually dominate.
/// All six lines share the same currency and period (e.g. £/year).
pub struct AiCostStack {
    /// Licences and inference spend (currency).
    pub licences_and_inference: f64,
    /// Integration cost: connecting the AI into existing systems (currency).
    pub integration: f64,
    /// Data-readiness cost: cleaning, access, pipelines (currency).
    pub data_readiness: f64,
    /// Evaluation cost: test sets, pilots, measurement (currency).
    pub evaluation: f64,
    /// Workflow-redesign cost: changing how people work around the AI
    /// (currency).
    pub workflow_redesign: f64,
    /// Governance and assurance cost: safety cases, review boards, audit
    /// (currency).
    pub governance_and_assurance: f64,
}

impl AiCostStack {
    /// Total AI cost: the sum of every line in the stack.
    ///
    /// # Returns
    ///
    /// The full-denominator cost (currency units).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::ai_return_on_investment::AiCostStack;
    ///
    /// // A £120k/yr all-in cost (use case A) where the licence line is
    /// // the minority of the denominator.
    /// let stack = AiCostStack {
    ///     licences_and_inference: 30_000.0,
    ///     integration: 40_000.0,
    ///     data_readiness: 20_000.0,
    ///     evaluation: 10_000.0,
    ///     workflow_redesign: 15_000.0,
    ///     governance_and_assurance: 5_000.0,
    /// };
    /// assert!((stack.total() - 120_000.0).abs() < 1e-9);
    /// assert!(stack.licences_and_inference < stack.total() / 2.0);
    /// ```
    pub fn total(&self) -> f64 {
        self.licences_and_inference
            + self.integration
            + self.data_readiness
            + self.evaluation
            + self.workflow_redesign
            + self.governance_and_assurance
    }
}

/// AI ROI as a fraction: (attributable benefit − total AI cost) / total AI cost.
///
/// The benefit must be measured against a baseline or control — "everyone
/// feels it's helping" is not a benefit line. A result of 1.67 reads as
/// ≈167% ROI; 0.0 is break-even; −1.0 is total loss (no measured benefit).
///
/// # Arguments
///
/// * `attributable_benefit` — benefit demonstrably caused by the AI,
///   measured against a baseline or control (currency).
/// * `total_ai_cost` — the full cost stack, not the licence alone (currency;
///   see [`AiCostStack::total`]).
///
/// # Returns
///
/// The ROI fraction, or `None` if `total_ai_cost` is zero (ROI is undefined
/// with no investment).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_return_on_investment::ai_roi;
///
/// // Use case A: (£320k benefit − £120k cost) / £120k ≈ 167%.
/// let roi = ai_roi(320_000.0, 120_000.0).unwrap();
/// assert!((roi - 1.67).abs() < 0.01);
///
/// // Use case B: no measured benefit → −100% (the 95% bucket).
/// assert!((ai_roi(0.0, 120_000.0).unwrap() - (-1.0)).abs() < 1e-9);
///
/// assert!(ai_roi(320_000.0, 0.0).is_none());
/// ```
pub fn ai_roi(attributable_benefit: f64, total_ai_cost: f64) -> Option<f64> {
    if total_ai_cost == 0.0 {
        None
    } else {
        Some((attributable_benefit - total_ai_cost) / total_ai_cost)
    }
}

/// Attributable benefit for a use case with a trackable baseline.
///
/// The baseline spend that genuinely stops (the budget line that moves),
/// minus any new cost the AI workflow introduces (e.g. added human review
/// time). This is the auditable, cash-releasing form of the benefit.
///
/// # Arguments
///
/// * `baseline_spend_stopped` — spend on the cancelled baseline activity
///   (currency), e.g. an outsourced transcription contract.
/// * `new_costs_introduced` — new costs the AI workflow adds (currency),
///   e.g. clinician review time.
///
/// # Returns
///
/// The net attributable benefit (currency units); negative if the new costs
/// exceed the stopped spend.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_return_on_investment::attributable_benefit;
///
/// // Use case A: £380k transcription contract cancelled; clinician review
/// // time +£60k → £320k attributable benefit.
/// let benefit = attributable_benefit(380_000.0, 60_000.0);
/// assert!((benefit - 320_000.0).abs() < 1e-9);
/// ```
pub fn attributable_benefit(baseline_spend_stopped: f64, new_costs_introduced: f64) -> f64 {
    baseline_spend_stopped - new_costs_introduced
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a hospital group deploys AI for two use cases.

    #[test]
    fn use_case_a_benefit_is_320k() {
        // Outsourced transcription £380k/yr cancelled; clinician review +£60k.
        let got = attributable_benefit(380_000.0, 60_000.0);
        assert!((got - 320_000.0).abs() < 1e-9);
    }

    #[test]
    fn use_case_a_roi_is_about_167_percent() {
        // ROI = (380k − 60k − 120k) / 120k ≈ 167% — cash-releasing, auditable.
        let benefit = attributable_benefit(380_000.0, 60_000.0);
        let got = ai_roi(benefit, 120_000.0).unwrap();
        assert!((got - 1.67).abs() < 0.01);
    }

    #[test]
    fn use_case_b_with_no_measured_benefit_is_pure_loss() {
        // "AI copilot for clinicians": no baseline captured, no demonstrable
        // P&L effect — the 95% bucket. Measured benefit 0 → ROI −100%.
        let got = ai_roi(0.0, 120_000.0).unwrap();
        assert!((got - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn cost_stack_totals_all_six_lines() {
        // Doc: total AI cost sums six lines, and "the licence is typically
        // the minority of the denominator".
        let stack = AiCostStack {
            licences_and_inference: 30_000.0,
            integration: 40_000.0,
            data_readiness: 20_000.0,
            evaluation: 10_000.0,
            workflow_redesign: 15_000.0,
            governance_and_assurance: 5_000.0,
        };
        assert!((stack.total() - 120_000.0).abs() < 1e-9);
        // The licence line is the minority of the denominator.
        assert!(stack.licences_and_inference < stack.total() / 2.0);
    }

    #[test]
    fn zero_cost_returns_none() {
        // Edge-case semantics: ROI is undefined with no investment.
        assert!(ai_roi(100.0, 0.0).is_none());
    }
}
