//! # Social Return on Investment (SROI)
//!
//! SROI extends ROI to outcomes that markets don't price — wellbeing,
//! social connection, carer relief, environmental impact — by monetizing
//! them with financial proxies, for *all* stakeholders affected.
//!
//! Each monetized claim is then discounted by adjustment factors —
//! deadweight, attribution, displacement, drop-off — that are the method's
//! integrity: without them, SROI is fiction with a currency sign.
//!
//! ## Formula
//!
//! ```text
//! SROI ratio = PV(monetized social outcomes) / PV(investment)
//!
//! For each outcome:
//!   value = quantity × financial proxy × attribution
//!           × (1 − deadweight) × (1 − displacement)
//!
//! quantity        people (or units) experiencing the outcome
//! financial proxy monetary value per unit of outcome (stated source)
//! attribution     share of the outcome caused by this intervention
//!                 (1 − share credited to other services)
//! deadweight      fraction that would have happened anyway
//! displacement    fraction merely moved from elsewhere, not created
//! drop-off        annual decay of the outcome over the years it persists
//! ```
//!
//! ## Why it matters
//!
//! Much of what health and community interventions produce never touches a
//! budget line: reduced loneliness, carer relief, employment gains,
//! dignity. SROI, governed by Social Value International's seven principles
//! (involve stakeholders, value what matters, don't over-claim, be
//! transparent, verify…), produces statements like "£3.20 of social value
//! per £1 invested." UK public procurement's social-value requirements make
//! SROI-style evidence commercially relevant: bids for public contracts
//! (including NHS) score points for demonstrated social value.
//!
//! ## Example
//!
//! A befriending app connecting isolated older adults to volunteers;
//! program cost £200,000/year; 1,500 active pairs. Loneliness relief is
//! proxied at ~£1,800/person/yr (wellbeing valuation), with 25% deadweight
//! and 80% attribution; reduced GP visits are payer-real:
//!
//! ```rust
//! use health_economics::social_return_on_investment::{
//!     SocialOutcome, sroi_ratio, proxy_valued_share,
//! };
//!
//! // Value = 1,500 × 1,800 × 0.80 × 0.75 = £1,620,000
//! let loneliness = SocialOutcome {
//!     quantity: 1_500.0,
//!     financial_proxy: 1_800.0,
//!     attribution: 0.80,
//!     deadweight: 0.25,
//!     displacement: 0.0,
//! };
//! assert!((loneliness.value() - 1_620_000.0).abs() < 1e-9);
//!
//! // Reduced GP visits: 1,500 × 1.2 visits × £42 = £75,600 (payer-real)
//! let gp = SocialOutcome {
//!     quantity: 1_500.0 * 1.2,
//!     financial_proxy: 42.0,
//!     attribution: 1.0,
//!     deadweight: 0.0,
//!     displacement: 0.0,
//! };
//! assert!((gp.value() - 75_600.0).abs() < 1e-9);
//!
//! // SROI = (1,620,000 + 75,600) / 200,000 ≈ 8.5 : 1
//! let ratio = sroi_ratio(loneliness.value() + gp.value(), 200_000.0).unwrap();
//! assert!((ratio - 8.5).abs() < 0.05);
//!
//! // Honesty check: the ratio is 96% proxy-valued wellbeing, 4% hard cash.
//! let share = proxy_valued_share(loneliness.value(), gp.value()).unwrap();
//! assert!((share - 0.96).abs() < 0.005);
//! ```
//!
//! That's legitimate SROI — but it must be presented as social value, never
//! allowed to imply £1.7M is bankable.
//!
//! ## Software engineering connection
//!
//! - SROI is the honest framework for engineering work whose beneficiaries
//!   are outside the paying team: open-source maintenance, accessibility
//!   improvements, platform work consumed by other teams,
//!   developer-community investment.
//! - Identify all stakeholders and monetize with stated proxies.
//! - Apply deadweight/attribution discounts: would that OSS fix have
//!   happened anyway? How much of the gain is your work vs the ecosystem's?
//! - The discipline of discounting your own impact claims is what separates
//!   SROI from a marketing number.
//!
//! ## Pitfalls
//!
//! - Proxy shopping: choosing the most generous wellbeing valuation
//!   available.
//! - Skipping deadweight/attribution — the most common inflation, often
//!   doubling the ratio.
//! - Ratio comparison across studies: SROI ratios are method-sensitive;
//!   compare only within a consistent framework.
//! - Presenting social value as cashable savings to a budget holder.
//!
//! ## Sources
//!
//! - Social Value International, Guide to SROI.
//!   <https://www.socialvalueint.org/guide-to-sroi>
//! - UK Government guide to SROI.
//!   <https://www.gov.uk/government/publications/a-guide-to-social-return-on-investment>
//!
//! Topic doc: health-economics-metrics/topics/social-return-on-investment.md

/// One monetized social outcome with its SROI adjustment factors.
///
/// All factor fields are fractions in 0..1. For a payer-real outcome
/// (e.g. avoided GP visits) with no adjustments, set `attribution` to 1.0
/// and `deadweight`/`displacement` to 0.0.
#[derive(Debug, Clone, Copy)]
pub struct SocialOutcome {
    /// Number of people (or units) experiencing the outcome.
    pub quantity: f64,
    /// Financial proxy: monetary value per unit of outcome (e.g. a
    /// wellbeing valuation of "relief from loneliness" ≈ £1,800/person/yr).
    /// The proxy's source should be stated, not shopped.
    pub financial_proxy: f64,
    /// Share of the outcome caused by this intervention, 0..1
    /// (1 − share credited to other services).
    pub attribution: f64,
    /// Fraction of the outcome that would have happened anyway, 0..1.
    pub deadweight: f64,
    /// Fraction merely moved from elsewhere rather than created, 0..1.
    pub displacement: f64,
}

impl SocialOutcome {
    /// Adjusted outcome value:
    /// quantity × proxy × attribution × (1 − deadweight) × (1 − displacement).
    ///
    /// The adjustment chain is the method's integrity — it removes the
    /// share that would have happened anyway (deadweight), the share owed
    /// to other actors (attribution), and the share merely relocated
    /// (displacement) before any value is claimed.
    ///
    /// # Returns
    ///
    /// The adjusted, monetized value of the outcome in the proxy's
    /// currency units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::social_return_on_investment::SocialOutcome;
    ///
    /// // Doc: 1,500 × 1,800 × 0.80 × 0.75 = £1,620,000
    /// let loneliness = SocialOutcome {
    ///     quantity: 1_500.0,
    ///     financial_proxy: 1_800.0,
    ///     attribution: 0.80,
    ///     deadweight: 0.25,
    ///     displacement: 0.0,
    /// };
    /// assert!((loneliness.value() - 1_620_000.0).abs() < 1e-9);
    /// ```
    pub fn value(&self) -> f64 {
        // Gross claim (quantity × proxy), then the discount chain:
        //   × attribution        — keep only the share this intervention caused
        //   × (1 − deadweight)   — remove what would have happened anyway
        //   × (1 − displacement) — remove what was moved, not created
        self.quantity
            * self.financial_proxy
            * self.attribution
            * (1.0 - self.deadweight)
            * (1.0 - self.displacement)
    }
}

/// Total adjusted value across a set of outcomes.
///
/// # Arguments
///
/// * `outcomes` — the monetized, adjusted outcome lines of the SROI case.
///
/// # Returns
///
/// Sum of [`SocialOutcome::value`] over all outcomes.
///
/// # Examples
///
/// ```rust
/// use health_economics::social_return_on_investment::{
///     SocialOutcome, total_outcome_value,
/// };
///
/// let loneliness = SocialOutcome {
///     quantity: 1_500.0, financial_proxy: 1_800.0,
///     attribution: 0.80, deadweight: 0.25, displacement: 0.0,
/// };
/// let gp = SocialOutcome {
///     quantity: 1_800.0, financial_proxy: 42.0,
///     attribution: 1.0, deadweight: 0.0, displacement: 0.0,
/// };
/// // £1,620,000 + £75,600 = £1,695,600
/// let total = total_outcome_value(&[loneliness, gp]);
/// assert!((total - 1_695_600.0).abs() < 1e-9);
/// ```
pub fn total_outcome_value(outcomes: &[SocialOutcome]) -> f64 {
    outcomes.iter().map(SocialOutcome::value).sum()
}

/// SROI ratio: present value of monetized social outcomes / present value
/// of the investment.
///
/// Quoted as "N : 1" — e.g. 8.5 means £8.50 of social value per £1
/// invested.
///
/// # Arguments
///
/// * `pv_outcomes` — present value of all adjusted, monetized outcomes.
/// * `pv_investment` — present value of the investment.
///
/// # Returns
///
/// The SROI ratio, or `None` if `pv_investment` is zero (the ratio is
/// undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::social_return_on_investment::sroi_ratio;
///
/// // Doc: (1,620,000 + 75,600) / 200,000 ≈ 8.5 : 1
/// let ratio = sroi_ratio(1_620_000.0 + 75_600.0, 200_000.0).unwrap();
/// assert!((ratio - 8.5).abs() < 0.05);
/// assert!(sroi_ratio(1_000.0, 0.0).is_none());
/// ```
pub fn sroi_ratio(pv_outcomes: f64, pv_investment: f64) -> Option<f64> {
    if pv_investment == 0.0 {
        None
    } else {
        Some(pv_outcomes / pv_investment)
    }
}

/// Outcome value after `years` of drop-off: value × (1 − drop_off_rate)^years.
///
/// Models the decay of an outcome over the years it is claimed to persist —
/// the fourth SROI adjustment factor, applied per year rather than once.
///
/// # Arguments
///
/// * `initial_value` — the outcome's adjusted value in year zero.
/// * `drop_off_rate` — annual decay fraction, 0..1 (e.g. 0.10 for 10%/year).
/// * `years` — number of years of decay (0 returns the value unchanged).
///
/// # Returns
///
/// The decayed outcome value.
///
/// # Examples
///
/// ```rust
/// use health_economics::social_return_on_investment::value_after_drop_off;
///
/// // 10% annual drop-off over 2 years: 1,000 × 0.9² = 810.
/// let v = value_after_drop_off(1_000.0, 0.10, 2);
/// assert!((v - 810.0).abs() < 1e-9);
/// ```
pub fn value_after_drop_off(initial_value: f64, drop_off_rate: f64, years: u32) -> f64 {
    // Geometric decay: each year retains (1 − drop-off) of the prior year.
    initial_value * (1.0 - drop_off_rate).powi(years as i32)
}

/// Share of total claimed value that is proxy-valued (soft) rather than
/// payer-real cash — the honesty check on any SROI headline.
///
/// In the worked example the 8.5:1 ratio is 96% proxy-valued wellbeing and
/// only 4% hard cash; presenting it as bankable savings would be the
/// module's final pitfall.
///
/// # Arguments
///
/// * `proxy_valued` — value monetized via financial proxies (wellbeing
///   valuations etc.).
/// * `payer_real` — value that is actual cash to a payer (e.g. avoided GP
///   visit costs).
///
/// # Returns
///
/// Proxy-valued share of the total as a fraction 0..1, or `None` if the
/// total (proxy + payer-real) is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::social_return_on_investment::proxy_valued_share;
///
/// // Doc: the ratio is 96% proxy-valued wellbeing and 4% hard cash.
/// let share = proxy_valued_share(1_620_000.0, 75_600.0).unwrap();
/// assert!((share - 0.96).abs() < 0.005);
/// ```
pub fn proxy_valued_share(proxy_valued: f64, payer_real: f64) -> Option<f64> {
    let total = proxy_valued + payer_real;
    if total == 0.0 {
        None
    } else {
        Some(proxy_valued / total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loneliness_outcome() -> SocialOutcome {
        SocialOutcome {
            quantity: 1_500.0,
            financial_proxy: 1_800.0,
            attribution: 0.80,
            deadweight: 0.25,
            displacement: 0.0,
        }
    }

    // Doc worked example: "Value = 1,500 × 1,800 × 0.80 × 0.75 = £1,620,000".
    #[test]
    fn loneliness_relief_values_at_1_62m() {
        // Doc: 1,500 × 1,800 × 0.80 × 0.75 = £1,620,000
        assert!((loneliness_outcome().value() - 1_620_000.0).abs() < 1e-9);
    }

    // Doc worked example: "reduced GP visits, 1,500 × 1.2 visits × £42 =
    // £75,600 (payer-real)".
    #[test]
    fn gp_visit_reduction_values_at_75_600() {
        // Doc: 1,500 × 1.2 visits × £42 = £75,600 (payer-real; no adjustments applied)
        let gp = SocialOutcome {
            quantity: 1_500.0 * 1.2,
            financial_proxy: 42.0,
            attribution: 1.0,
            deadweight: 0.0,
            displacement: 0.0,
        };
        assert!((gp.value() - 75_600.0).abs() < 1e-9);
    }

    // Doc worked example: "SROI = (1,620,000 + 75,600) / 200,000 ≈ 8.5 : 1".
    #[test]
    fn sroi_ratio_is_about_8_5_to_1() {
        // Doc: (1,620,000 + 75,600) / 200,000 ≈ 8.5 : 1 (exact 8.478)
        let ratio = sroi_ratio(1_620_000.0 + 75_600.0, 200_000.0).unwrap();
        assert!((ratio - 8.5).abs() < 0.05);
    }

    // Sum of the two worked-example outcome lines: £1,620,000 + £75,600.
    #[test]
    fn total_outcome_value_sums_both_outcomes() {
        let gp = SocialOutcome {
            quantity: 1_800.0,
            financial_proxy: 42.0,
            attribution: 1.0,
            deadweight: 0.0,
            displacement: 0.0,
        };
        let total = total_outcome_value(&[loneliness_outcome(), gp]);
        assert!((total - 1_695_600.0).abs() < 1e-9);
    }

    // Doc: "the ratio is 96% proxy-valued wellbeing and 4% hard cash".
    #[test]
    fn ratio_is_96_percent_proxy_valued() {
        // Doc: the ratio is 96% proxy-valued wellbeing and 4% hard cash.
        let share = proxy_valued_share(1_620_000.0, 75_600.0).unwrap();
        assert!((share - 0.96).abs() < 0.005);
    }

    // Edge case: the ratio is undefined with zero investment.
    #[test]
    fn sroi_ratio_with_zero_investment_is_none() {
        assert!(sroi_ratio(1_000.0, 0.0).is_none());
    }

    // Doc (The math): "drop-off = decay of the outcome over years".
    #[test]
    fn drop_off_decays_outcome_value_over_years() {
        // 10% annual drop-off over 2 years: 1,000 × 0.9² = 810.
        let v = value_after_drop_off(1_000.0, 0.10, 2);
        assert!((v - 810.0).abs() < 1e-9);
        // Year zero: no decay.
        assert!((value_after_drop_off(1_000.0, 0.10, 0) - 1_000.0).abs() < 1e-9);
    }
}
