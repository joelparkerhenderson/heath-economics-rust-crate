//! # Inference Unit Economics
//!
//! Inference unit economics price AI features by their marginal compute:
//! **cost per token**, rolled up to cost per call, per business unit (triage
//! episode, drafted letter, consultation summary), per year.
//!
//! The defining dynamic: LLM prices have fallen roughly **an order of
//! magnitude every 1–2 years** at constant capability — a deflation rate with
//! no precedent in health-technology costing — so multi-year models need
//! explicit price-decline scenarios.
//!
//! ## Formula
//!
//! ```text
//! Cost per call = input tokens × input rate + output tokens × output rate
//! Cost per unit = Σ calls per unit of business output
//!                 (base call + retries + verification/guardrail passes)
//! cost_t        = cost_0 × d^t   (price-decline scenario)
//!
//! input/output rate = price per token, quoted per million tokens
//!                     (e.g. $3/M in, $15/M out)
//! d                 = year-on-year price ratio; test d ∈ {0.3, 0.5, 0.7}/year
//! t                 = years from now
//! ```
//!
//! ## Why it matters
//!
//! Two consequences follow from the price collapse. Commercially, an AI
//! feature that is marginal today may be trivially profitable in 18 months —
//! and a competitor priced on today's costs will be undercut. For economic
//! evaluation, any cost-effectiveness model for an AI-enabled clinical
//! service that freezes 2024 inference prices materially overstates ongoing
//! cost — the analysis needs price-decline scenarios the way drug models
//! handle patent expiry and generic entry. (Reference points: frontier output
//! tokens ~$15–75/M in mid-2026, mid-tier models an order cheaper,
//! GPT-4-level capability down from ~$20/M in 2022 to ~$0.40/M; Epoch AI
//! measured 9×–900×/year declines depending on the capability milestone.)
//!
//! ## Example
//!
//! An AI discharge-summary service: the average summary uses 12,000 input
//! tokens + 1,200 output, plus a verification pass (6,000 in / 300 out), at
//! $3/M in and $15/M out.
//!
//! ```rust
//! use health_economics::inference_unit_economics::{
//!     annual_cost, cost_per_call, cost_per_unit, cost_share_of_value, LlmCall,
//! };
//!
//! let draft = LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 };
//! let verify = LlmCall { input_tokens: 6_000.0, output_tokens: 300.0 };
//!
//! // Draft: 12,000 × 3/1M + 1,200 × 15/1M = $0.036 + $0.018 = $0.054.
//! assert!((cost_per_call(&draft, 3.0, 15.0) - 0.054).abs() < 1e-9);
//! // Verify: 6,000 × 3/1M + 300 × 15/1M = $0.018 + $0.0045 ≈ $0.023.
//! assert!((cost_per_call(&verify, 3.0, 15.0) - 0.0225).abs() < 1e-9);
//!
//! // Per summary ≈ $0.077 → per 100,000 summaries/year ≈ $7,700.
//! let per_summary = cost_per_unit(&[draft, verify], 3.0, 15.0);
//! assert!((per_summary - 0.0765).abs() < 1e-9);
//! assert!((annual_cost(per_summary, 100_000.0) - 7_650.0).abs() < 1e-9);
//!
//! // Against ~20 clinician-minutes saved (≈ £25 ≈ $31.25), inference is
//! // ~0.25% of the value created.
//! let share = cost_share_of_value(per_summary, 31.25).unwrap();
//! assert!((share - 0.0025).abs() < 0.0005);
//! ```
//!
//! ## Software engineering connection
//!
//! - This is cloud unit economics specialized for AI.
//! - **Meter per business unit**, not per API call, so the number plugs into
//!   ICER/budget-impact models directly.
//! - **Watch the input/output asymmetry**: output is typically ~4× input
//!   price; RAG architectures are input-heavy — architecture choices are
//!   pricing choices.
//! - **Route by task tier**: matching model capability to task difficulty
//!   (cheap models for classification, frontier for synthesis) routinely cuts
//!   blended cost 5–10× at equal quality.
//! - The worked example's finding travels well: inference cost is rarely the
//!   binding constraint at current prices for high-value clinical tasks — the
//!   economics are dominated by integration, evaluation, governance, adoption.
//!
//! ## Pitfalls
//!
//! - **Frozen-price multi-year models** — overstates cost; but also
//!   **assumed-deflation revenue models** — a price war is not a contract;
//!   scenario both.
//! - **Ignoring evaluation overhead**: guardrails, judges, and retries are
//!   real tokens, often the majority in regulated settings.
//! - **Per-token myopia**: latency, rate limits, and context-window
//!   constraints carry costs no token price captures.
//!
//! ## Sources
//!
//! - Epoch AI, LLM inference price trends.
//!   <https://epoch.ai/data-insights/llm-inference-price-trends>
//! - LLM pricing comparisons. <https://www.silicondata.com/blog/llm-cost-per-token>
//!
//! Topic doc: health-economics-metrics/topics/inference-unit-economics.md

/// One LLM call, measured in tokens.
///
/// Token counts are `f64` so averages ("the mean summary uses 12,000 input
/// tokens") can be represented directly.
pub struct LlmCall {
    /// Prompt-side tokens: record context, RAG context, instructions.
    /// RAG-heavy architectures make this the dominant term.
    pub input_tokens: f64,
    /// Completion-side tokens. Output is typically priced ~4× input.
    pub output_tokens: f64,
}

/// Cost of one call: `input tokens × input rate + output tokens × output rate`.
///
/// Rates are quoted per million tokens (e.g. $3/M in, $15/M out), so each
/// term is divided by 1,000,000. The result is in the same currency as the
/// rates.
///
/// # Arguments
///
/// * `call` — the call's input and output token counts.
/// * `input_rate_per_million` — price per million input tokens (e.g. 3.0 for $3/M).
/// * `output_rate_per_million` — price per million output tokens (e.g. 15.0 for $15/M).
///
/// # Returns
///
/// The call's cost in currency units (e.g. dollars).
///
/// # Examples
///
/// ```rust
/// use health_economics::inference_unit_economics::{cost_per_call, LlmCall};
///
/// // Draft: 12,000 × 3/1M + 1,200 × 15/1M = $0.036 + $0.018 = $0.054.
/// let draft = LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 };
/// assert!((cost_per_call(&draft, 3.0, 15.0) - 0.054).abs() < 1e-9);
/// ```
pub fn cost_per_call(call: &LlmCall, input_rate_per_million: f64, output_rate_per_million: f64) -> f64 {
    // Rates are per million tokens, so scale each side down by 1e6:
    // input term + output term, priced independently (output ≈ 4× input).
    call.input_tokens * input_rate_per_million / 1_000_000.0
        + call.output_tokens * output_rate_per_million / 1_000_000.0
}

/// Cost per unit of business output: the sum over every call the unit needs.
///
/// "Unit" is a business unit — a triage episode, drafted letter, or
/// consultation summary — and the slice should include the base call plus
/// retries and verification/guardrail passes (often 20–50% overhead, and
/// sometimes the majority in regulated settings).
///
/// # Arguments
///
/// * `calls` — every LLM call one unit of output requires.
/// * `input_rate_per_million` — price per million input tokens.
/// * `output_rate_per_million` — price per million output tokens.
///
/// # Returns
///
/// The summed cost of all calls, in currency units. An empty slice costs 0.
///
/// # Examples
///
/// ```rust
/// use health_economics::inference_unit_economics::{cost_per_unit, LlmCall};
///
/// // Draft ($0.054) + verify ($0.0225) ≈ $0.077 per discharge summary.
/// let calls = [
///     LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 },
///     LlmCall { input_tokens: 6_000.0, output_tokens: 300.0 },
/// ];
/// let per_summary = cost_per_unit(&calls, 3.0, 15.0);
/// assert!((per_summary - 0.0765).abs() < 1e-9);
/// ```
pub fn cost_per_unit(calls: &[LlmCall], input_rate_per_million: f64, output_rate_per_million: f64) -> f64 {
    calls
        .iter()
        .map(|c| cost_per_call(c, input_rate_per_million, output_rate_per_million))
        .sum()
}

/// Annual cost at a given volume of business units.
///
/// A plain multiplication kept explicit so the per-unit number can plug into
/// budget-impact models at any volume.
///
/// # Arguments
///
/// * `cost_per_unit` — cost of one business unit (currency).
/// * `units_per_year` — annual volume of units.
///
/// # Returns
///
/// The annual cost, `cost_per_unit × units_per_year`.
///
/// # Examples
///
/// ```rust
/// use health_economics::inference_unit_economics::annual_cost;
///
/// // $0.0765 per summary × 100,000 summaries/year = $7,650 ≈ $7,700.
/// let got = annual_cost(0.0765, 100_000.0);
/// assert!((got - 7_650.0).abs() < 1e-9);
/// ```
pub fn annual_cost(cost_per_unit: f64, units_per_year: f64) -> f64 {
    cost_per_unit * units_per_year
}

/// Inference cost as a fraction of the value created per unit.
///
/// Both arguments must be in the same currency. A small share (the worked
/// example lands at ~0.25%) is the signal that the economics are dominated by
/// everything *except* the tokens.
///
/// # Arguments
///
/// * `cost_per_unit` — inference cost per business unit (currency).
/// * `value_per_unit` — value created per unit, same currency (e.g. clinician
///   time saved priced at loaded cost).
///
/// # Returns
///
/// `Some(cost / value)` as a fraction (0.0025 = 0.25%), or `None` when
/// `value_per_unit` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::inference_unit_economics::cost_share_of_value;
///
/// // $0.0765 inference against ~$31.25 of clinician time saved ≈ 0.25%.
/// let share = cost_share_of_value(0.0765, 31.25).unwrap();
/// assert!((share - 0.0025).abs() < 0.0005);
///
/// assert!(cost_share_of_value(0.0765, 0.0).is_none());
/// ```
pub fn cost_share_of_value(cost_per_unit: f64, value_per_unit: f64) -> Option<f64> {
    if value_per_unit == 0.0 {
        None
    } else {
        Some(cost_per_unit / value_per_unit)
    }
}

/// Price-decline scenario for multi-year models: `cost_t = cost_0 × d^t`.
///
/// `d` is the year-on-year price ratio at constant capability — d = 0.5 means
/// prices halve every year. Sensitivity analysis should test
/// d ∈ {0.3, 0.5, 0.7}/year rather than freezing today's price.
///
/// # Arguments
///
/// * `cost_0` — cost today (currency, any unit level: per call, per unit, per year).
/// * `annual_decline_ratio` — year-on-year price ratio `d` (0 < d ≤ 1 for a
///   decline; 1.0 means flat prices).
/// * `years` — years from now `t` (fractional years allowed).
///
/// # Returns
///
/// The projected cost at year `t`, same units as `cost_0`.
///
/// # Examples
///
/// ```rust
/// use health_economics::inference_unit_economics::projected_cost;
///
/// // d = 0.5 halves the cost each year: ×0.5 after one, ×0.25 after two.
/// assert!((projected_cost(1_000.0, 0.5, 1.0) - 500.0).abs() < 1e-9);
/// assert!((projected_cost(1_000.0, 0.5, 2.0) - 250.0).abs() < 1e-9);
/// // Test the sensitivity band d ∈ {0.3, 0.7} too.
/// assert!((projected_cost(1_000.0, 0.3, 1.0) - 300.0).abs() < 1e-9);
/// assert!((projected_cost(1_000.0, 0.7, 1.0) - 700.0).abs() < 1e-9);
/// ```
pub fn projected_cost(cost_0: f64, annual_decline_ratio: f64, years: f64) -> f64 {
    // Geometric decay: the ratio compounds annually, cost_t = cost_0 × d^t.
    cost_0 * annual_decline_ratio.powf(years)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT_RATE: f64 = 3.0; // $/M tokens
    const OUTPUT_RATE: f64 = 15.0; // $/M tokens

    fn draft() -> LlmCall {
        LlmCall { input_tokens: 12_000.0, output_tokens: 1_200.0 }
    }

    fn verify() -> LlmCall {
        LlmCall { input_tokens: 6_000.0, output_tokens: 300.0 }
    }

    /// Draft: 12,000 × 3/1M + 1,200 × 15/1M = $0.036 + $0.018 = $0.054.
    #[test]
    fn draft_call_costs_54_thousandths() {
        // Worked example: "Draft: 12,000 × 3/1M + 1,200 × 15/1M
        // = $0.036 + $0.018 = $0.054".
        let input_part = 12_000.0 * INPUT_RATE / 1_000_000.0;
        let output_part = 1_200.0 * OUTPUT_RATE / 1_000_000.0;
        assert!((input_part - 0.036).abs() < 1e-9);
        assert!((output_part - 0.018).abs() < 1e-9);
        let got = cost_per_call(&draft(), INPUT_RATE, OUTPUT_RATE);
        assert!((got - 0.054).abs() < 1e-9);
    }

    /// Verify: 6,000 × 3/1M + 300 × 15/1M = $0.018 + $0.0045 ≈ $0.023.
    #[test]
    fn verify_pass_costs_about_23_thousandths() {
        // Worked example: "Verify: 6,000 × 3/1M + 300 × 15/1M
        // = $0.018 + $0.0045 ≈ $0.023".
        let input_part = 6_000.0 * INPUT_RATE / 1_000_000.0;
        let output_part = 300.0 * OUTPUT_RATE / 1_000_000.0;
        assert!((input_part - 0.018).abs() < 1e-9);
        assert!((output_part - 0.0045).abs() < 1e-9);
        let got = cost_per_call(&verify(), INPUT_RATE, OUTPUT_RATE);
        assert!((got - 0.023).abs() < 0.001); // exact value 0.0225, doc rounds to ≈ 0.023
    }

    /// Per summary ≈ $0.077 (exact 0.0765).
    #[test]
    fn per_summary_cost_is_about_77_thousandths() {
        // Worked example: "Per summary ≈ $0.077" (draft + verify).
        let got = cost_per_unit(&[draft(), verify()], INPUT_RATE, OUTPUT_RATE);
        assert!((got - 0.0765).abs() < 1e-9);
        assert!((got - 0.077).abs() < 0.001);
    }

    /// Per 100,000 summaries/year ≈ $7,700 (exact $7,650).
    #[test]
    fn annual_cost_for_100k_summaries_is_about_7700() {
        // Worked example: "per 100,000 summaries/year ≈ $7,700".
        let per_summary = cost_per_unit(&[draft(), verify()], INPUT_RATE, OUTPUT_RATE);
        let got = annual_cost(per_summary, 100_000.0);
        assert!((got - 7_650.0).abs() < 1e-9);
        assert!((got - 7_700.0).abs() < 100.0);
    }

    /// Against ~20 clinician-minutes saved per summary (≈ £25 ≈ $31.25 at
    /// $1.25/£), inference is ≈ 0.25% of the value created.
    #[test]
    fn inference_is_about_a_quarter_percent_of_value_created() {
        // Worked example: "Against ~20 clinician-minutes saved per summary
        // (≈ £25), inference is 0.25% of the value created".
        let per_summary = cost_per_unit(&[draft(), verify()], INPUT_RATE, OUTPUT_RATE);
        let value_per_summary_usd = 25.0 * 1.25;
        let share = cost_share_of_value(per_summary, value_per_summary_usd).unwrap();
        assert!((share - 0.0025).abs() < 0.0005);
    }

    /// Price-decline scenario: cost_t = cost_0 × d^t, e.g. d = 0.5 halves the
    /// cost each year (×0.25 after two years).
    #[test]
    fn price_decline_scenario_compounds_annually() {
        // Doc's math: "cost_t = cost_0 × d^t, test d ∈ {0.3, 0.5, 0.7}/year".
        assert!((projected_cost(1_000.0, 0.5, 1.0) - 500.0).abs() < 1e-9);
        assert!((projected_cost(1_000.0, 0.5, 2.0) - 250.0).abs() < 1e-9);
        assert!((projected_cost(1_000.0, 0.3, 1.0) - 300.0).abs() < 1e-9);
        assert!((projected_cost(1_000.0, 0.7, 1.0) - 700.0).abs() < 1e-9);
    }

    /// Zero value per unit makes the cost share undefined.
    #[test]
    fn cost_share_is_undefined_at_zero_value() {
        // Edge case: cost share of value is a division — undefined at zero value.
        assert!(cost_share_of_value(0.0765, 0.0).is_none());
    }
}
