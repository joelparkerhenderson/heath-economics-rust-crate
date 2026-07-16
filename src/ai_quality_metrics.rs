//! # AI Quality Metrics
//!
//! Metrics for the correctness of AI-generated output: accuracy against
//! ground truth, faithfulness/groundedness (is every claim supported by the
//! provided context?), and hallucination rate (what fraction of outputs
//! contain unsupported or false content?). In health settings these are not
//! quality niceties — they are harm rates.
//!
//! The economic weighting turns error rates into expected harm cost: a
//! hallucinated dosage or fabricated citation in a clinical workflow is a
//! false-information event with a harm pathway, priced like the false
//! positives of screening economics.
//!
//! ## Formula
//!
//! ```text
//! Hallucination rate = outputs containing unsupported/false content / total outputs
//!   intrinsic:  contradicts the provided context
//!   extrinsic:  unverifiable fabrication beyond the context
//!
//! Faithfulness (RAGAS-style) = supported claims in answer / total claims in answer
//!
//! Economic weighting — not all hallucinations cost alike:
//!   expected harm cost = Σ over error types (rate × P(undetected) ×
//!                        P(acted upon) × cost per acted-upon error)
//! ```
//!
//! Legend:
//! - `outputs containing unsupported/false content` / `total outputs` —
//!   counts of model outputs.
//! - `supported claims` / `total claims` — claims in one answer verifiable
//!   against the provided context (counts).
//! - `rate` — fraction of outputs containing a given error type (0.0–1.0).
//! - `P(undetected)` — probability the human-review layer misses the error.
//! - `P(acted upon)` — probability an undetected error is acted upon.
//! - `cost per acted-upon error` — downstream cost when it is (currency).
//!
//! ## Why it matters
//!
//! Medical-domain benchmarks have measured hallucination rates above 60% for
//! ungrounded LLMs on medical tasks (some open models >80%), while grounding,
//! retrieval, and reasoning modes cut rates dramatically (e.g., GPT-5's
//! thinking mode reduced HealthBench hallucinations 3.6% → 1.6% on one
//! benchmark). Every hallucination that survives review triggers downstream
//! cost: acting on wrong information, verification labor, medico-legal
//! exposure, eroded trust — so it belongs in the harms arm of any economic
//! model. The human-review layer sets P(undetected), and its cost (reviewer
//! minutes × volume) belongs in the model too.
//!
//! ## Example
//!
//! An AI clinical-coding assistant processes 200,000 episodes/year; audit
//! shows 2% of outputs contain a material coding error; human coders catch
//! 85% of those. Errors reaching submission = 600/year at ≈£250 each →
//! £150,000/year expected error cost, alongside a £200,000/year review cost.
//! Retrieval grounding cutting the error rate to 0.8% drops uncaught errors
//! to 240 and error cost to £60,000 (−£90k/yr).
//!
//! ```rust
//! use health_economics::ai_quality_metrics::{
//!     errors_reaching_submission, expected_error_cost, review_cost,
//! };
//!
//! // Errors reaching submission = 200,000 × 0.02 × 0.15 = 600/year.
//! let uncaught = errors_reaching_submission(200_000.0, 0.02, 0.85);
//! assert!((uncaught - 600.0).abs() < 1e-9);
//!
//! // Expected error cost = 600 × £250 = £150,000/year.
//! let error_cost = expected_error_cost(uncaught, 250.0);
//! assert!((error_cost - 150_000.0).abs() < 1e-9);
//!
//! // Review cost (2 min × 200k × £0.50/min) = £200,000/year.
//! let review = review_cost(2.0, 200_000.0, 0.50);
//! assert!((review - 200_000.0).abs() < 1e-9);
//!
//! // Improvement case: grounding cuts the error rate to 0.8% →
//! // 240 uncaught errors, £60,000 error cost, a −£90k/yr saving.
//! let improved = errors_reaching_submission(200_000.0, 0.008, 0.85);
//! assert!((improved - 240.0).abs() < 1e-9);
//! assert!((error_cost - expected_error_cost(improved, 250.0) - 90_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Treat model quality like test coverage economics, with health-grade
//!   discipline: **evaluation sets are your clinical trial** —
//!   pre-registered, representative of *your* case mix, refreshed against
//!   drift.
//! - **Grounding beats scale for factual tasks**: retrieval +
//!   citation-required prompting is usually the cheapest hallucination
//!   reduction available (mind its token overhead).
//! - **Publish the operating point**: like sensitivity/specificity, "97%
//!   faithful" means nothing without the task distribution and the detection
//!   threshold.
//! - The review-layer math is the same NNT/NNH arithmetic as any screening
//!   gate.
//!
//! ## Pitfalls
//!
//! - **Benchmark-to-production transplantation**: hallucination rates are
//!   wildly task-dependent; your case mix is the only benchmark that counts.
//! - **Uncosted human review**: "a clinician checks everything" halves the
//!   benefit and must appear in the cost line — and vigilance decays
//!   (automation complacency), so P(undetected) rises with trust.
//! - **Optimizing average quality while tail risk carries the harm**: one
//!   fabricated allergy note outweighs a thousand awkward phrasings; weight
//!   errors by consequence, per the expected-harm formula.
//!
//! ## Sources
//!
//! - Hallucination evaluation methods and metrics.
//!   <https://www.braintrust.dev/articles/ai-hallucination-evaluations-metrics-methods-2026>
//! - Medical LLM hallucination statistics.
//!   <https://sqmagazine.co.uk/llm-hallucination-statistics/>
//! - RAG faithfulness metrics.
//!   <https://www.getmaxim.ai/articles/measuring-llm-hallucinations-the-metrics-that-actually-matter-for-reliable-ai-apps/>
//!
//! Topic doc: health-economics-metrics/topics/ai-quality-metrics.md

/// Hallucination rate: outputs containing unsupported or false content over total outputs.
///
/// Covers both intrinsic errors (contradicting the provided context) and
/// extrinsic ones (unverifiable fabrication beyond it). Result is a fraction
/// (0.0–1.0), not a percentage. Rates are wildly task-dependent — measure on
/// your own case mix.
///
/// # Arguments
///
/// * `outputs_with_unsupported_content` — outputs containing unsupported or
///   false content (count).
/// * `total_outputs` — outputs evaluated (count).
///
/// # Returns
///
/// The hallucination rate as a fraction, or `None` if `total_outputs` is
/// zero (nothing was evaluated).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::hallucination_rate;
///
/// // GPT-5 thinking mode on HealthBench: 3.6% → 1.6% hallucinations.
/// let before = hallucination_rate(3.6, 100.0).unwrap();
/// let after = hallucination_rate(1.6, 100.0).unwrap();
/// assert!((before - 0.036).abs() < 1e-9);
/// assert!((after - 0.016).abs() < 1e-9);
///
/// assert!(hallucination_rate(1.0, 0.0).is_none());
/// ```
pub fn hallucination_rate(outputs_with_unsupported_content: f64, total_outputs: f64) -> Option<f64> {
    if total_outputs == 0.0 {
        None
    } else {
        Some(outputs_with_unsupported_content / total_outputs)
    }
}

/// RAGAS-style faithfulness: supported claims over total claims in an answer.
///
/// A claim is "supported" when it is verifiable against the provided
/// (retrieved) context. Result is a fraction (0.0–1.0); publish it with the
/// task distribution and detection threshold, or it means nothing.
///
/// # Arguments
///
/// * `supported_claims` — claims in the answer supported by the context
///   (count).
/// * `total_claims` — claims made in the answer (count).
///
/// # Returns
///
/// Faithfulness as a fraction, or `None` if `total_claims` is zero (the
/// answer makes no claims).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::faithfulness;
///
/// // 97 of 100 claims supported by the retrieved context → 0.97 faithful.
/// let f = faithfulness(97.0, 100.0).unwrap();
/// assert!((f - 0.97).abs() < 1e-9);
///
/// assert!(faithfulness(0.0, 0.0).is_none());
/// ```
pub fn faithfulness(supported_claims: f64, total_claims: f64) -> Option<f64> {
    if total_claims == 0.0 {
        None
    } else {
        Some(supported_claims / total_claims)
    }
}

/// One error type in the economic weighting of hallucinations.
///
/// Not all hallucinations cost alike: one fabricated allergy note outweighs a
/// thousand awkward phrasings, so errors are weighted by consequence. Each
/// error type contributes rate × P(undetected) × P(acted upon) × cost to the
/// expected harm cost per output (see [`expected_harm_cost_per_output`]).
pub struct ErrorType {
    /// Fraction of outputs containing this error type (0.0–1.0).
    pub rate: f64,
    /// Probability the human-review layer fails to catch the error (0.0–1.0).
    /// Rises with trust as vigilance decays (automation complacency).
    pub probability_undetected: f64,
    /// Probability an undetected error is acted upon (0.0–1.0).
    pub probability_acted_upon: f64,
    /// Cost per acted-upon error (currency units), e.g. mis-billing average
    /// plus audit exposure.
    pub cost_per_acted_upon_error: f64,
}

/// Expected harm cost per output, summed over error types.
///
/// Σ over error types of rate × P(undetected) × P(acted upon) × cost per
/// acted-upon error. Multiply by annual volume for the yearly harms line.
///
/// # Arguments
///
/// * `error_types` — the error types in the harm model (see [`ErrorType`]).
///
/// # Returns
///
/// The expected harm cost per output (currency units); 0.0 for an empty
/// slice.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::{
///     expected_harm_cost_per_output, ErrorType,
/// };
///
/// // Worked example as one error type: rate 2%, P(undetected) 15%,
/// // every uncaught error acted upon, £250 each.
/// let per_output = expected_harm_cost_per_output(&[ErrorType {
///     rate: 0.02,
///     probability_undetected: 0.15,
///     probability_acted_upon: 1.0,
///     cost_per_acted_upon_error: 250.0,
/// }]);
/// // × 200,000 episodes/year = the £150,000/year expected error cost.
/// assert!((per_output * 200_000.0 - 150_000.0).abs() < 1e-9);
/// ```
pub fn expected_harm_cost_per_output(error_types: &[ErrorType]) -> f64 {
    error_types
        .iter()
        .map(|e| {
            // Per error type: rate × P(undetected) × P(acted upon) × cost.
            e.rate * e.probability_undetected * e.probability_acted_upon
                * e.cost_per_acted_upon_error
        })
        .sum()
}

/// Errors reaching submission per year, after the human-review layer.
///
/// volume × error rate × (1 − catch rate): only the errors the reviewers
/// miss survive to submission.
///
/// # Arguments
///
/// * `annual_volume` — items processed per year (count).
/// * `material_error_rate` — fraction of outputs containing a material error
///   (0.0–1.0).
/// * `human_catch_rate` — fraction of those errors caught by human review
///   (0.0–1.0).
///
/// # Returns
///
/// Expected uncaught errors per year (count).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::errors_reaching_submission;
///
/// // Worked example: 200,000 × 0.02 × 0.15 = 600 errors/year.
/// let uncaught = errors_reaching_submission(200_000.0, 0.02, 0.85);
/// assert!((uncaught - 600.0).abs() < 1e-9);
/// ```
pub fn errors_reaching_submission(
    annual_volume: f64,
    material_error_rate: f64,
    human_catch_rate: f64,
) -> f64 {
    // (1 − catch rate) is P(undetected): the review layer's miss fraction.
    annual_volume * material_error_rate * (1.0 - human_catch_rate)
}

/// Expected annual error cost: uncaught errors × cost per uncaught error.
///
/// # Arguments
///
/// * `uncaught_errors` — errors reaching submission per year (count; see
///   [`errors_reaching_submission`]).
/// * `cost_per_uncaught_error` — downstream cost per uncaught error
///   (currency), e.g. mis-billing average plus audit exposure.
///
/// # Returns
///
/// Expected annual error cost (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::expected_error_cost;
///
/// // Worked example: 600 × £250 = £150,000/year.
/// let cost = expected_error_cost(600.0, 250.0);
/// assert!((cost - 150_000.0).abs() < 1e-9);
/// ```
pub fn expected_error_cost(uncaught_errors: f64, cost_per_uncaught_error: f64) -> f64 {
    uncaught_errors * cost_per_uncaught_error
}

/// Annual cost of the human-review layer.
///
/// reviewer minutes per item × volume × cost per reviewer minute. "A
/// clinician checks everything" is not free: this line belongs in the model
/// alongside the harm cost it suppresses.
///
/// # Arguments
///
/// * `minutes_per_item` — reviewer minutes spent per item (minutes).
/// * `annual_volume` — items reviewed per year (count).
/// * `cost_per_reviewer_minute` — loaded reviewer cost per minute (currency).
///
/// # Returns
///
/// Annual review cost (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_quality_metrics::review_cost;
///
/// // Worked example: 2 min × 200,000 × £0.50/min = £200,000/year.
/// let cost = review_cost(2.0, 200_000.0, 0.50);
/// assert!((cost - 200_000.0).abs() < 1e-9);
/// ```
pub fn review_cost(
    minutes_per_item: f64,
    annual_volume: f64,
    cost_per_reviewer_minute: f64,
) -> f64 {
    minutes_per_item * annual_volume * cost_per_reviewer_minute
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: an AI clinical-coding assistant processes 200,000
    // episodes/year; 2% of outputs contain a material error; human coders
    // catch 85% of those.

    #[test]
    fn errors_reaching_submission_are_600_per_year() {
        // 200,000 × 0.02 × 0.15 = 600/year.
        let got = errors_reaching_submission(200_000.0, 0.02, 0.85);
        assert!((got - 600.0).abs() < 1e-9);
    }

    #[test]
    fn expected_error_cost_is_150_000_per_year() {
        // 600 × £250 = £150,000/year.
        let got = expected_error_cost(600.0, 250.0);
        assert!((got - 150_000.0).abs() < 1e-9);
    }

    #[test]
    fn review_cost_is_200_000_per_year() {
        // 2 min × 200k × £0.50/min = £200,000/year.
        let got = review_cost(2.0, 200_000.0, 0.50);
        assert!((got - 200_000.0).abs() < 1e-9);
    }

    #[test]
    fn grounding_cuts_uncaught_errors_to_240() {
        // Retrieval grounding cuts error rate to 0.8% → 240 uncaught errors.
        let got = errors_reaching_submission(200_000.0, 0.008, 0.85);
        assert!((got - 240.0).abs() < 1e-9);
    }

    #[test]
    fn grounding_cuts_error_cost_to_60_000_saving_90_000() {
        // 240 × £250 = £60,000/year, a −£90k/yr improvement.
        let improved = expected_error_cost(240.0, 250.0);
        assert!((improved - 60_000.0).abs() < 1e-9);
        let baseline = expected_error_cost(600.0, 250.0);
        assert!((baseline - improved - 90_000.0).abs() < 1e-9);
    }

    #[test]
    fn expected_harm_formula_matches_worked_example() {
        // The worked example as a single error type: rate 2%, P(undetected)
        // 15%, P(acted upon) 1 (every uncaught error carries cost), £250.
        let per_output = expected_harm_cost_per_output(&[ErrorType {
            rate: 0.02,
            probability_undetected: 0.15,
            probability_acted_upon: 1.0,
            cost_per_acted_upon_error: 250.0,
        }]);
        let annual = per_output * 200_000.0;
        assert!((annual - 150_000.0).abs() < 1e-9);
    }

    #[test]
    fn harm_cost_sums_over_error_types() {
        // "Σ over error types" in the expected-harm formula: two error types
        // with different consequence weights.
        let types = [
            ErrorType {
                rate: 0.01,
                probability_undetected: 0.5,
                probability_acted_upon: 0.5,
                cost_per_acted_upon_error: 100.0,
            },
            ErrorType {
                rate: 0.001,
                probability_undetected: 0.2,
                probability_acted_upon: 1.0,
                cost_per_acted_upon_error: 10_000.0,
            },
        ];
        let got = expected_harm_cost_per_output(&types);
        // 0.01×0.5×0.5×100 = 0.25 ; 0.001×0.2×1×10000 = 2.0
        assert!((got - 2.25).abs() < 1e-9);
    }

    #[test]
    fn gpt5_thinking_mode_hallucination_drop_3_6_to_1_6_percent() {
        // "Why it matters": 3.6% → 1.6% on HealthBench.
        let before = hallucination_rate(3.6, 100.0).unwrap();
        let after = hallucination_rate(1.6, 100.0).unwrap();
        assert!((before - 0.036).abs() < 1e-9);
        assert!((after - 0.016).abs() < 1e-9);
    }

    #[test]
    fn faithfulness_is_supported_over_total_claims() {
        // SE connection: the "97% faithful" operating-point example.
        let got = faithfulness(97.0, 100.0).unwrap();
        assert!((got - 0.97).abs() < 1e-9);
    }

    #[test]
    fn zero_denominators_return_none() {
        // Edge-case semantics: rates are undefined with nothing evaluated.
        assert!(hallucination_rate(1.0, 0.0).is_none());
        assert!(faithfulness(1.0, 0.0).is_none());
    }
}
