//! # Build vs Buy
//!
//! Build-vs-buy is a structured comparison of custom development against
//! commercial acquisition, on risk-adjusted total cost of ownership (TCO),
//! time-to-value, and cost of delay. The empirical priors are one-sided:
//! **actual build costs typically exceed projections by 30–40%**, bought
//! solutions deploy 40–60% faster, and MIT's 2025 GenAI research found
//! purchased AI tools succeeded ~67% of the time while internal builds
//! succeeded about a third as often.
//!
//! The economic frame forces the honest comparison: both options priced over
//! the same horizon, both risk-adjusted, and the *time difference priced as
//! cost of delay* — the term that most often decides the answer and most
//! often gets omitted.
//!
//! ## Formula
//!
//! ```text
//! Risk-adjusted build cost    = estimate × 1.3–1.4        (overrun prior)
//! Risk-adjusted time-to-value = estimate × (1 + 0.4–0.6)  (delay prior)
//! TCO(horizon)   = upfront + annual running cost × years
//! Cost of delay  = value per month × months the slower option arrives later
//! Effective cost = TCO + cost of delay
//!
//! estimate        — the un-adjusted build cost or schedule estimate
//! overrun prior   — Green Book "optimism bias" uplift for build cost
//! delay prior     — fractional uplift on the estimated time-to-value
//! value per month — benefit stream lost for each month of later arrival
//! ```
//!
//! ## Why it matters
//!
//! Health systems face this decision constantly ("make vs commission" in NHS
//! language), and engineering organizations systematically get it wrong in
//! the build direction — because builders estimate the build, not the TCO,
//! and because building is more fun. Decision drivers, in the order they
//! usually decide: (1) differentiation — is this capability your product, or
//! plumbing? (2) time-to-value × cost of delay; (3) risk-adjusted TCO. Build
//! remains right when the capability is differentiating, when no vendor
//! meets a hard constraint (clinical safety, data residency), or when vendor
//! lock-in risk is severe and priced.
//!
//! ## Example
//!
//! A trust needs an e-consent system. Buy: £150k/year SaaS, live in 3
//! months. Build: estimated £600k + £120k/year maintenance, live in 12
//! months. Risk-adjusted, the effective comparison is £750k vs £1,785k —
//! buy wins by ~£1M.
//!
//! ```rust
//! use health_economics::build_vs_buy::{
//!     cost_of_delay, effective_cost, risk_adjusted_build_cost,
//!     risk_adjusted_time_to_value, total_cost_of_ownership,
//! };
//!
//! // Risk-adjusted build: 600k × 1.35 = £810k; time-to-value ≈ 18 months.
//! let build_cost = risk_adjusted_build_cost(600_000.0, 1.35);
//! assert!((build_cost - 810_000.0).abs() < 1e-6);
//! let build_ttv = risk_adjusted_time_to_value(12.0, 0.5);
//! assert!((build_ttv - 18.0).abs() < 1e-9);
//!
//! // 5-yr TCO: buy = 150k × 5 = £750k; build = 810k + 120k × 5 = £1,410k.
//! let buy_tco = total_cost_of_ownership(0.0, 150_000.0, 5.0);
//! let build_tco = total_cost_of_ownership(build_cost, 120_000.0, 5.0);
//! assert!((buy_tco - 750_000.0).abs() < 1e-6);
//! assert!((build_tco - 1_410_000.0).abs() < 1e-6);
//!
//! // Delay term: £25k/month saved, build arrives 15 months later → £375k.
//! let cod = cost_of_delay(25_000.0, 18.0 - 3.0);
//! assert!((cod - 375_000.0).abs() < 1e-6);
//!
//! // Effective comparison: £750k vs £1,785k — buy wins by ~£1M.
//! let buy_effective = effective_cost(buy_tco, 0.0);
//! let build_effective = effective_cost(build_tco, cod);
//! assert!((build_effective - 1_785_000.0).abs() < 1e-6);
//! assert!((build_effective - buy_effective - 1_035_000.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Prior-based risk adjustment**: the 30–40% overrun uplift is the
//!   software Green Book optimism bias — apply it mechanically, argue for
//!   exceptions rather than from them.
//! - **Comparator honesty**: the alternative to building isn't "nothing,"
//!   it's the best available buy (opportunity cost).
//! - **Equivalence testing before cost comparison**: if buy and build
//!   genuinely meet the same spec, this is cost-minimization analysis and
//!   the cheap one wins; if not, the outcome difference must be valued, not
//!   asserted.
//! - Price the time difference as cost of delay — the term that most often
//!   decides the answer and most often gets omitted.
//!
//! ## Pitfalls
//!
//! - **Comparing vendor list price to un-risk-adjusted build estimates** —
//!   double flattery toward build.
//! - **Zero-priced internal labor** ("the team's already here").
//! - **Unpriced lock-in in both directions**: vendor exit costs, but also
//!   the build's bus-factor and maintenance tenure.
//! - **Identity-driven builds**: "this is core to us" claimed for plumbing —
//!   test differentiation against whether customers would notice.
//!
//! ## Sources
//!
//! - Build-vs-buy TCO analyses. <https://neontri.com/blog/build-vs-buy-software/>
//! - MIT GenAI divide findings (buy-vs-build success rates).
//!   <https://blueflame.ai/blog/achieving-ai-roi-key-findings-from-mits-genai-report>
//! - HM Treasury Green Book (optimism bias).
//!   <https://www.gov.uk/government/publications/the-green-book-appraisal-and-evaluation-in-central-government/the-green-book-2020>
//!
//! Topic doc: health-economics-metrics/topics/build-vs-buy.md

/// Apply the build-cost overrun prior: estimate × overrun factor.
///
/// This is the Green Book optimism-bias pattern applied to software builds;
/// the empirical prior for the factor is 1.3–1.4 (builds overrun projections
/// by 30–40%).
///
/// # Arguments
///
/// * `estimated_cost` — the raw build estimate, in currency.
/// * `overrun_factor` — multiplicative uplift (e.g. 1.35 for the midpoint
///   of the 30–40% prior).
///
/// # Returns
///
/// The risk-adjusted build cost, in the same currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::build_vs_buy::risk_adjusted_build_cost;
///
/// // Worked example: £600k estimate × 1.35 = £810k.
/// assert!((risk_adjusted_build_cost(600_000.0, 1.35) - 810_000.0).abs() < 1e-6);
/// ```
pub fn risk_adjusted_build_cost(estimated_cost: f64, overrun_factor: f64) -> f64 {
    estimated_cost * overrun_factor
}

/// Apply the deployment-delay prior to an estimated time-to-value:
/// months × (1 + uplift).
///
/// The empirical prior for the uplift is 0.4–0.6 (bought solutions deploy
/// 40–60% faster than builds actually arrive).
///
/// # Arguments
///
/// * `estimated_months` — the raw time-to-value estimate, in months.
/// * `delay_uplift` — fractional uplift (e.g. 0.5 for the midpoint of the
///   40–60% prior).
///
/// # Returns
///
/// The risk-adjusted time-to-value, in months.
///
/// # Examples
///
/// ```rust
/// use health_economics::build_vs_buy::risk_adjusted_time_to_value;
///
/// // Worked example: 12-month build estimate → ≈ 18 months at the 50% prior.
/// assert!((risk_adjusted_time_to_value(12.0, 0.5) - 18.0).abs() < 1e-9);
/// ```
pub fn risk_adjusted_time_to_value(estimated_months: f64, delay_uplift: f64) -> f64 {
    estimated_months * (1.0 + delay_uplift)
}

/// Total cost of ownership over a horizon: upfront cost plus annual running
/// cost × years.
///
/// Both options must be priced over the same horizon (typically 3–5 years)
/// for the comparison to be honest. Upfront cost is zero for a pure SaaS
/// buy; for a build it should already be risk-adjusted.
///
/// # Arguments
///
/// * `upfront_cost` — one-time cost at the start of the horizon, in currency.
/// * `annual_running_cost` — recurring cost per year (licences, maintenance).
/// * `years` — length of the comparison horizon, in years.
///
/// # Returns
///
/// The TCO over the horizon, in the same currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::build_vs_buy::total_cost_of_ownership;
///
/// // 5-yr buy TCO: £150k/year SaaS = £750k.
/// assert!((total_cost_of_ownership(0.0, 150_000.0, 5.0) - 750_000.0).abs() < 1e-6);
///
/// // 5-yr build TCO: £810k risk-adjusted build + £120k/yr = £1,410k.
/// assert!((total_cost_of_ownership(810_000.0, 120_000.0, 5.0) - 1_410_000.0).abs() < 1e-6);
/// ```
pub fn total_cost_of_ownership(upfront_cost: f64, annual_running_cost: f64, years: f64) -> f64 {
    upfront_cost + annual_running_cost * years
}

/// Cost of delay: value lost per month × months the option arrives later
/// than the alternative.
///
/// The term that most often decides the build-vs-buy answer and most often
/// gets omitted. Months later is measured between risk-adjusted
/// time-to-value figures, not raw estimates.
///
/// # Arguments
///
/// * `value_per_month` — benefit stream lost per month of delay, in currency.
/// * `months_later` — how many months later this option delivers value.
///
/// # Returns
///
/// The delay cost, in the same currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::build_vs_buy::cost_of_delay;
///
/// // Consent digitization saves £25k/month; build arrives 15 months later
/// // (18 risk-adjusted vs the buy's 3) → CoD = £375k.
/// assert!((cost_of_delay(25_000.0, 15.0) - 375_000.0).abs() < 1e-6);
/// ```
pub fn cost_of_delay(value_per_month: f64, months_later: f64) -> f64 {
    value_per_month * months_later
}

/// Effective comparison cost for an option: its TCO plus its delay cost
/// relative to the faster alternative.
///
/// The faster option carries a delay cost of zero; the slower option adds
/// its cost of delay on top of its TCO.
///
/// # Arguments
///
/// * `tco` — the option's total cost of ownership over the shared horizon.
/// * `delay_cost` — the option's cost of delay vs the faster alternative
///   (0.0 for the faster option).
///
/// # Returns
///
/// The effective cost used for the head-to-head comparison.
///
/// # Examples
///
/// ```rust
/// use health_economics::build_vs_buy::effective_cost;
///
/// // Effective comparison: buy £750k vs build £1,410k + £375k = £1,785k.
/// let buy = effective_cost(750_000.0, 0.0);
/// let build = effective_cost(1_410_000.0, 375_000.0);
/// assert!((build - 1_785_000.0).abs() < 1e-6);
/// assert!(build - buy > 1_000_000.0); // buy wins by ~£1M
/// ```
pub fn effective_cost(tco: f64, delay_cost: f64) -> f64 {
    tco + delay_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: e-consent system.
    // Buy: £150k/year SaaS, live in 3 months.
    // Build: £600k + £120k/year maintenance, live in 12 months (raw estimate).

    // Worked-example line: "Risk-adjusted build: 600k × 1.35 = £810k".
    #[test]
    fn risk_adjusted_build_is_810k() {
        let adjusted = risk_adjusted_build_cost(600_000.0, 1.35);
        assert!((adjusted - 810_000.0).abs() < 1e-6);
    }

    // Worked-example line: "time-to-value ≈ 18 months".
    #[test]
    fn risk_adjusted_time_to_value_is_18_months() {
        // 12 months uplifted by the midpoint 50% delay prior ≈ 18 months.
        let months = risk_adjusted_time_to_value(12.0, 0.5);
        assert!((months - 18.0).abs() < 1e-9);
    }

    // Worked-example line: "5-yr TCO: buy = 150k × 5 = £750k".
    #[test]
    fn five_year_buy_tco_is_750k() {
        let buy = total_cost_of_ownership(0.0, 150_000.0, 5.0);
        assert!((buy - 750_000.0).abs() < 1e-6);
    }

    // Worked-example line: "build = 810k + 120k × 5 = £1,410k".
    #[test]
    fn five_year_build_tco_is_1_410k() {
        let build = total_cost_of_ownership(810_000.0, 120_000.0, 5.0);
        assert!((build - 1_410_000.0).abs() < 1e-6);
    }

    // Worked-example line: "CoD = 15 × 25k = £375k".
    #[test]
    fn delay_term_is_375k() {
        // Consent digitization saves £25k/month; build arrives 15 months later.
        let cod = cost_of_delay(25_000.0, 15.0);
        assert!((cod - 375_000.0).abs() < 1e-6);
    }

    // Worked-example line: "Effective comparison: £750k vs £1,785k — buy
    // wins by ~£1M".
    #[test]
    fn effective_comparison_buy_wins_by_about_one_million() {
        let buy_effective = effective_cost(750_000.0, 0.0);
        let build_effective = effective_cost(1_410_000.0, 375_000.0);
        assert!((buy_effective - 750_000.0).abs() < 1e-6);
        assert!((build_effective - 1_785_000.0).abs() < 1e-6);
        let advantage = build_effective - buy_effective;
        assert!((advantage - 1_035_000.0).abs() < 1e-6);
        // Doc: "buy wins by ~£1M".
        assert!((advantage - 1_000_000.0).abs() < 50_000.0);
    }
}
