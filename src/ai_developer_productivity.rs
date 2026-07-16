//! # AI Developer Productivity
//!
//! Metrics for what AI coding assistance actually does to engineering output:
//! suggestion acceptance rates, controlled-study speedups, PR throughput, and
//! code retention — plus the value model that turns measured time saved into
//! a capacity benefit line.
//!
//! The evidence base is genuinely contradictory — which makes it a perfect
//! case study in the efficacy-vs-effectiveness distinction health economics
//! was built to handle.
//!
//! ## Formula
//!
//! ```text
//! Acceptance rate  = accepted suggestions / shown suggestions
//! Retention rate   = AI code surviving to merge / accepted AI code
//! Speedup          = (t_control − t_AI) / t_control   (controlled comparison ONLY)
//! Throughput delta = Δ merged PRs/dev/week
//!
//! Value model      = devs × time saved × loaded rate × utilization factor
//! ```
//!
//! Legend:
//! - `accepted / shown suggestions` — assistant suggestions accepted vs
//!   displayed (counts).
//! - `t_control`, `t_AI` — task time without and with the assistant (same
//!   time unit).
//! - `Δ merged PRs/dev/week` — change in merged pull requests per developer
//!   per week versus baseline.
//! - `time saved` — *measured* hours saved per developer per day.
//! - `loaded rate` — fully loaded hourly cost of a developer (currency/hour).
//! - `utilization factor` — fraction of freed time that becomes productive
//!   capacity (0.0–1.0).
//!
//! ## Why it matters
//!
//! The two most-cited controlled studies point in opposite directions:
//! Peng et al. 2023 (GitHub Copilot RCT) found developers completed a
//! greenfield HTTP-server task 55.8% faster with Copilot (1h11m vs 2h41m,
//! n=95); the METR 2025 RCT found experienced open-source developers working
//! on *their own mature repositories* were 19% slower with early-2025 AI
//! tools (16 devs, 246 tasks) — while *believing* they were 20% faster. Both
//! are good studies. The contradiction is the finding: greenfield-task
//! efficacy does not transfer to mature-codebase effectiveness, and
//! *perceived* benefit cannot substitute for measured benefit. Benchmarks for
//! context: GitHub telemetry acceptance ~30% average (SQL 45%, Python 35%,
//! JS 28%), retention ~88%, GitHub/Accenture field throughput +8.7%.
//!
//! ## Example
//!
//! An org of 500 developers pilots an assistant with a proper control
//! (matched teams, 3 months, pre-registered metrics). Self-reported time
//! saved is 45 min/day but the measured task-level figure is ≈15 min/day
//! (0.25 h). Valuing the *measured* number gives ≈£990,000/year of
//! non-cash-releasing capacity against ≈£234,000/year of tool cost — a
//! net capacity ratio of ≈4:1, fundable at one-third the self-reported claim.
//!
//! ```rust
//! use health_economics::ai_developer_productivity::{
//!     annual_capacity_value, annual_tool_cost, net_capacity_ratio, perception_gap_ratio,
//! };
//!
//! // Value the MEASURED number: 500 × 0.25h × 220d × £60 × 0.6 ≈ £990,000/year.
//! let value = annual_capacity_value(500.0, 0.25, 220.0, 60.0, 0.6);
//! assert!((value - 990_000.0).abs() < 1e-6);
//!
//! // Cost: 500 × £39/mo × 12 ≈ £234,000/year.
//! let cost = annual_tool_cost(500.0, 39.0);
//! assert!((cost - 234_000.0).abs() < 1e-6);
//!
//! // Net capacity ratio ≈ 4:1.
//! let ratio = net_capacity_ratio(value, cost).unwrap();
//! assert!((ratio - 4.0).abs() < 0.3);
//!
//! // Self-reported 45 min/day vs measured 15 min/day: the 3× perception gap.
//! let gap = perception_gap_ratio(45.0, 15.0).unwrap();
//! assert!((gap - 3.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Run **pragmatic trials**: your codebase, your engineers, real tickets —
//!   not vendor demo tasks.
//! - Treat **acceptance rate as a proxy, not an outcome** — it is the PPV of
//!   suggestions from the developer's view; high acceptance with low
//!   retention is overdiagnosis.
//! - Pair every throughput gain with a **stability check** (DORA 2025: AI
//!   lifts throughput, hurts stability — an intervention with side effects
//!   needs net-benefit analysis).
//! - Classify the benefit honestly as capacity (cash-releasing vs
//!   non-cash-releasing).
//!
//! ## Pitfalls
//!
//! - **Vendor-study transplantation**: greenfield RCT numbers applied to
//!   legacy-codebase work — the exact error the METR study exposed.
//! - **Self-report as measurement**: the 20-percentage-point perception gap
//!   is the largest known bias in this literature.
//! - **Activity inflation**: more PRs and more code are Activity, not
//!   outcomes; pair with rework and change-failure rate.
//! - **Ignoring the learning curve**: week-2 measurements capture novelty
//!   effects in either direction; measure at steady state.
//!
//! ## Sources
//!
//! - Peng S, et al. "The Impact of AI on Developer Productivity: Evidence
//!   from GitHub Copilot." 2023. <https://arxiv.org/abs/2302.06590>
//! - METR, "Measuring the Impact of Early-2025 AI on Experienced Open-Source
//!   Developer Productivity." 2025.
//!   <https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/>
//! - DORA 2025 report. <https://dora.dev/dora-report-2025/>
//!
//! Topic doc: health-economics-metrics/topics/ai-developer-productivity.md

/// Acceptance rate: accepted suggestions divided by shown suggestions.
///
/// A proxy, not an outcome — it is the positive predictive value of
/// suggestions from the developer's view. GitHub telemetry averages ~30%
/// (SQL 45%, Python 35%, JS 28%). Result is a fraction (0.0–1.0), not a
/// percentage.
///
/// # Arguments
///
/// * `accepted_suggestions` — suggestions the developer accepted (count).
/// * `shown_suggestions` — suggestions displayed (count).
///
/// # Returns
///
/// The acceptance rate as a fraction, or `None` if `shown_suggestions` is
/// zero (no suggestions were shown).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::acceptance_rate;
///
/// // GitHub telemetry average: ~30% of shown suggestions accepted.
/// let rate = acceptance_rate(30.0, 100.0).unwrap();
/// assert!((rate - 0.30).abs() < 1e-9);
///
/// assert!(acceptance_rate(30.0, 0.0).is_none());
/// ```
pub fn acceptance_rate(accepted_suggestions: f64, shown_suggestions: f64) -> Option<f64> {
    if shown_suggestions == 0.0 {
        None
    } else {
        Some(accepted_suggestions / shown_suggestions)
    }
}

/// Retention rate: AI-generated code surviving to merge divided by accepted AI code.
///
/// Roughly 88% is reported in the literature. High acceptance with low
/// retention is the overdiagnosis pattern — code taken, then thrown away.
/// Result is a fraction (0.0–1.0). Measure numerator and denominator in the
/// same unit (lines or characters).
///
/// # Arguments
///
/// * `code_surviving_to_merge` — accepted AI code still present at merge
///   (lines/chars).
/// * `accepted_ai_code` — AI code accepted into the working tree
///   (same unit).
///
/// # Returns
///
/// The retention rate as a fraction, or `None` if `accepted_ai_code` is zero
/// (no AI code was accepted).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::retention_rate;
///
/// // Reported retention: ~88% of accepted AI code survives to merge.
/// let rate = retention_rate(88.0, 100.0).unwrap();
/// assert!((rate - 0.88).abs() < 1e-9);
///
/// assert!(retention_rate(88.0, 0.0).is_none());
/// ```
pub fn retention_rate(code_surviving_to_merge: f64, accepted_ai_code: f64) -> Option<f64> {
    if accepted_ai_code == 0.0 {
        None
    } else {
        Some(code_surviving_to_merge / accepted_ai_code)
    }
}

/// Speedup from a controlled comparison: (t_control − t_AI) / t_control.
///
/// Use figures from a controlled comparison ONLY — never self-report.
/// Positive means the AI arm is faster; negative means it is slower (the
/// METR 2025 finding: −19% on mature repositories). Result is a fraction of
/// control time saved. Both times must share a unit.
///
/// # Arguments
///
/// * `t_control` — task time without the assistant (any time unit).
/// * `t_ai` — task time with the assistant (same unit).
///
/// # Returns
///
/// The fractional speedup, or `None` if `t_control` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::speedup;
///
/// // Peng et al. 2023: greenfield task in 71 min vs 161 min → 55.8% faster.
/// let s = speedup(161.0, 71.0).unwrap();
/// assert!((s - 0.558).abs() < 0.005);
///
/// // METR 2025: 19% slower — t_AI = 1.19 × t_control → speedup −0.19.
/// let s = speedup(100.0, 119.0).unwrap();
/// assert!((s - (-0.19)).abs() < 1e-9);
/// ```
pub fn speedup(t_control: f64, t_ai: f64) -> Option<f64> {
    if t_control == 0.0 {
        None
    } else {
        Some((t_control - t_ai) / t_control)
    }
}

/// Relative throughput delta: change in merged PRs per developer per week.
///
/// Expressed as a fraction of the baseline (e.g. +0.087 for the
/// GitHub/Accenture field figure of +8.7%). More PRs are Activity, not
/// outcomes — pair with rework and change-failure rate.
///
/// # Arguments
///
/// * `merged_prs_before` — baseline merged PRs/dev/week.
/// * `merged_prs_after` — merged PRs/dev/week after adoption.
///
/// # Returns
///
/// The fractional change versus baseline, or `None` if `merged_prs_before`
/// is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::throughput_delta;
///
/// // Pilot result from the worked example: merged PRs +6%.
/// let delta = throughput_delta(100.0, 106.0).unwrap();
/// assert!((delta - 0.06).abs() < 1e-9);
///
/// assert!(throughput_delta(0.0, 106.0).is_none());
/// ```
pub fn throughput_delta(merged_prs_before: f64, merged_prs_after: f64) -> Option<f64> {
    if merged_prs_before == 0.0 {
        None
    } else {
        Some((merged_prs_after - merged_prs_before) / merged_prs_before)
    }
}

/// Annual capacity value of measured time saved, in currency units.
///
/// devs × hours saved per day × working days per year × loaded hourly rate ×
/// utilization factor. Use the *measured* time saving, not self-report, and
/// classify the result as non-cash-releasing capacity unless headcount or
/// spend actually changes. Every term needs local measurement — time saved
/// dominates all other parameters combined in sensitivity analysis.
///
/// # Arguments
///
/// * `developers` — developers covered (count).
/// * `hours_saved_per_dev_per_day` — measured hours saved per developer per
///   day (hours; e.g. 0.25 for 15 min).
/// * `working_days_per_year` — working days per year (days; e.g. 220).
/// * `loaded_hourly_rate` — fully loaded cost per developer-hour (currency).
/// * `utilization_factor` — fraction of freed time that becomes productive
///   capacity (0.0–1.0).
///
/// # Returns
///
/// Annual capacity value (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::annual_capacity_value;
///
/// // Worked example: 500 × 0.25h × 220d × £60 × 0.6 ≈ £990,000/year.
/// let value = annual_capacity_value(500.0, 0.25, 220.0, 60.0, 0.6);
/// assert!((value - 990_000.0).abs() < 1e-6);
/// ```
pub fn annual_capacity_value(
    developers: f64,
    hours_saved_per_dev_per_day: f64,
    working_days_per_year: f64,
    loaded_hourly_rate: f64,
    utilization_factor: f64,
) -> f64 {
    // devs × h/day × days/year = hours/year; × rate = gross value;
    // × utilization discounts freed time that never becomes output.
    developers
        * hours_saved_per_dev_per_day
        * working_days_per_year
        * loaded_hourly_rate
        * utilization_factor
}

/// Annual tool cost: developers × monthly licence price × 12.
///
/// The licence line only — integration, evaluation, and workflow-redesign
/// costs belong in a fuller AI cost stack.
///
/// # Arguments
///
/// * `developers` — seats licensed (count).
/// * `monthly_price_per_dev` — licence price per developer per month
///   (currency).
///
/// # Returns
///
/// Annual tool cost (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::annual_tool_cost;
///
/// // Worked example: 500 × £39/mo × 12 ≈ £234,000/year.
/// let cost = annual_tool_cost(500.0, 39.0);
/// assert!((cost - 234_000.0).abs() < 1e-6);
/// ```
pub fn annual_tool_cost(developers: f64, monthly_price_per_dev: f64) -> f64 {
    developers * monthly_price_per_dev * 12.0
}

/// Net capacity ratio: capacity value divided by tool cost.
///
/// A ratio above 1.0 means the (non-cash-releasing) capacity value exceeds
/// the tool spend; the worked example lands at ≈4:1 on measured numbers.
///
/// # Arguments
///
/// * `annual_capacity_value` — value of measured time saved (currency/year;
///   see [`annual_capacity_value`]).
/// * `annual_tool_cost` — annual licence cost (currency/year; see
///   [`annual_tool_cost`]).
///
/// # Returns
///
/// The value:cost ratio, or `None` if `annual_tool_cost` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::net_capacity_ratio;
///
/// // Worked example: £990,000 value vs £234,000 cost ≈ 4:1.
/// let ratio = net_capacity_ratio(990_000.0, 234_000.0).unwrap();
/// assert!((ratio - 4.0).abs() < 0.3);
///
/// assert!(net_capacity_ratio(990_000.0, 0.0).is_none());
/// ```
pub fn net_capacity_ratio(annual_capacity_value: f64, annual_tool_cost: f64) -> Option<f64> {
    if annual_tool_cost == 0.0 {
        None
    } else {
        Some(annual_capacity_value / annual_tool_cost)
    }
}

/// Ratio of self-reported to measured time saved — the perception gap.
///
/// The METR 2025 study exposed this bias: developers believed they were 20%
/// faster while measuring 19% slower. A ratio of 1.0 means self-report
/// matches measurement; the worked example's pilot shows 3×. Both arguments
/// must share a unit (e.g. minutes/day).
///
/// # Arguments
///
/// * `self_reported_saving` — what developers say they save (time units).
/// * `measured_saving` — what the controlled measurement shows (same units).
///
/// # Returns
///
/// The self-report:measured ratio, or `None` if `measured_saving` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_developer_productivity::perception_gap_ratio;
///
/// // Worked example: self-reported 45 min/day vs measured 15 min/day → 3×.
/// let gap = perception_gap_ratio(45.0, 15.0).unwrap();
/// assert!((gap - 3.0).abs() < 1e-9);
///
/// assert!(perception_gap_ratio(45.0, 0.0).is_none());
/// ```
pub fn perception_gap_ratio(self_reported_saving: f64, measured_saving: f64) -> Option<f64> {
    if measured_saving == 0.0 {
        None
    } else {
        Some(self_reported_saving / measured_saving)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: an org of 500 developers pilots an assistant with a
    // proper control; measured task-level saving ≈ 15 min/day (0.25 h).

    #[test]
    fn measured_capacity_value_is_about_990_000_per_year() {
        // 500 × 0.25h × 220d × £60 × 0.6 utilization ≈ £990,000/year.
        let got = annual_capacity_value(500.0, 0.25, 220.0, 60.0, 0.6);
        assert!((got - 990_000.0).abs() < 1e-6);
    }

    #[test]
    fn tool_cost_is_about_234_000_per_year() {
        // 500 × £39/mo × 12 ≈ £234,000/year.
        let got = annual_tool_cost(500.0, 39.0);
        assert!((got - 234_000.0).abs() < 1e-6);
    }

    #[test]
    fn net_capacity_ratio_is_about_4_to_1() {
        // Doc: "Net capacity ratio ≈ 4:1 — fundable".
        let value = annual_capacity_value(500.0, 0.25, 220.0, 60.0, 0.6);
        let cost = annual_tool_cost(500.0, 39.0);
        let got = net_capacity_ratio(value, cost).unwrap();
        assert!((got - 4.0).abs() < 0.3);
    }

    #[test]
    fn perception_gap_is_3x_in_the_pilot() {
        // Self-reported 45 min/day vs measured ≈ 15 min/day: a 3× gap.
        let got = perception_gap_ratio(45.0, 15.0).unwrap();
        assert!((got - 3.0).abs() < 1e-9);
    }

    #[test]
    fn peng_2023_copilot_speedup_is_about_55_8_percent() {
        // Greenfield task completed in 1h11m (71 min) vs 2h41m (161 min).
        let got = speedup(161.0, 71.0).unwrap();
        assert!((got - 0.558).abs() < 0.005);
    }

    #[test]
    fn metr_2025_shows_negative_speedup_of_19_percent() {
        // 19% slower with AI: t_AI = 1.19 × t_control.
        let got = speedup(100.0, 119.0).unwrap();
        assert!((got - (-0.19)).abs() < 1e-9);
    }

    #[test]
    fn pilot_throughput_delta_is_plus_6_percent() {
        // Merged PRs +6% in the pilot.
        let got = throughput_delta(100.0, 106.0).unwrap();
        assert!((got - 0.06).abs() < 1e-9);
    }

    #[test]
    fn github_telemetry_average_acceptance_is_about_30_percent() {
        // Doc: "GitHub telemetry ~30% avg" acceptance.
        let got = acceptance_rate(30.0, 100.0).unwrap();
        assert!((got - 0.30).abs() < 1e-9);
    }

    #[test]
    fn reported_retention_rate_is_about_88_percent() {
        // Doc: "Retention rate ... (~88% reported)".
        let got = retention_rate(88.0, 100.0).unwrap();
        assert!((got - 0.88).abs() < 1e-9);
    }

    #[test]
    fn zero_denominators_return_none() {
        // Edge-case semantics: every ratio is undefined at a zero denominator.
        assert!(acceptance_rate(1.0, 0.0).is_none());
        assert!(retention_rate(1.0, 0.0).is_none());
        assert!(speedup(0.0, 1.0).is_none());
        assert!(throughput_delta(0.0, 1.0).is_none());
        assert!(net_capacity_ratio(1.0, 0.0).is_none());
        assert!(perception_gap_ratio(1.0, 0.0).is_none());
    }
}
