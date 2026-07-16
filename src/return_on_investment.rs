//! # Return on Investment (ROI)
//!
//! ROI is the ratio of net gain to money invested. It is the metric
//! engineering and finance already share — health economics adds the
//! discipline that makes an ROI claim survive scrutiny: declared
//! perspective, comparator, horizon, and benefit class.
//!
//! In particular, this module separates cash-releasing benefits (money the
//! budget holder can actually bank) from capacity benefits (valued in
//! money but not banked) and qualitative ones (real but not monetized).
//!
//! ## Formula
//!
//! ```text
//! ROI = (Benefits − Costs) / Costs      (often × 100%)
//!
//! Payback period = Costs / annual net benefit
//!
//! Benefits   monetized benefits over the declared horizon
//! Costs      total investment over the same horizon
//! ```
//!
//! An ROI claim is under-specified without four declarations: perspective
//! (whose benefits count), comparator (versus what alternative), horizon
//! (over how long, discounted), and benefit class (cash-releasing, capacity,
//! or qualitative).
//!
//! ## Why it matters
//!
//! ROI is the lingua franca of budget holders, and public health uses it
//! too: the landmark Masters et al. review found a median ROI of 14.3:1 for
//! public health interventions (every £1 returns ~£14 to the wider economy
//! and health system). But that 14:1 is a societal, long-horizon figure; a
//! hospital CFO's ROI is payer-perspective and 1–3 years. Most ROI fights
//! are actually undeclared-perspective fights.
//!
//! ## Example
//!
//! E-rostering system, cost £500,000 over 3 years: £450,000 cash-releasing
//! (agency shift reduction), £600,000 capacity (ward-manager admin time
//! freed, valued but not banked), plus non-monetized qualitative benefits:
//!
//! ```rust
//! use health_economics::return_on_investment::{
//!     BenefitClass, BenefitLine, strict_financial_roi, economic_roi,
//! };
//!
//! let lines = [
//!     BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
//!     BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
//!     BenefitLine { class: BenefitClass::Qualitative, amount: 0.0 },
//! ];
//!
//! // Strict financial ROI = (450,000 − 500,000)/500,000 = −10%
//! let strict = strict_financial_roi(&lines, 500_000.0).unwrap();
//! assert!((strict - (-0.10)).abs() < 1e-9);
//!
//! // Economic ROI = (1,050,000 − 500,000)/500,000 = +110%
//! let economic = economic_roi(&lines, 500_000.0).unwrap();
//! assert!((economic - 1.10).abs() < 1e-9);
//! ```
//!
//! Both numbers are true. A vendor quoting "+110% ROI" to a CFO who can
//! only bank £450k will lose trust; presenting both, labeled, wins it.
//!
//! ## Software engineering connection
//!
//! - Every tooling proposal has an ROI slide; almost none declare the four
//!   parameters (perspective, comparator, horizon, benefit class).
//! - The most common failure is category-blending: capacity gains
//!   (developer minutes) presented as financial return.
//! - Structure AI/platform ROI as a cash line, a capacity line, and a
//!   qualitative line — then add sensitivity analysis on the soft numbers.
//! - The cash/capacity split protects an internal champion when finance
//!   audits the benefits two years later.
//!
//! ## Pitfalls
//!
//! - Perspective laundering: societal benefits over a decade quoted to a
//!   budget holder with a 12-month horizon.
//! - Gross instead of net: "returns £3M" on £2M spend is 50% ROI, not 300%.
//! - Ratio maximization: tiny denominators produce spectacular ROIs on
//!   trivial investments; rank portfolios by NPV or net monetary benefit,
//!   use ROI as a screen.
//! - No benefits audit: forecast ROI without benefits-realization tracking
//!   is a promise, not a result.
//!
//! ## Sources
//!
//! - Masters R, et al. "Return on investment of public health interventions:
//!   a systematic review." J Epidemiol Community Health 2017.
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC5537512/>
//! - HM Treasury Green Book.
//!   <https://www.gov.uk/government/publications/the-green-book-appraisal-and-evaluation-in-central-government/the-green-book-2020>
//!
//! Topic doc: health-economics-metrics/topics/return-on-investment.md

/// Classification of a benefit line, per the cash-releasing vs
/// non-cash-releasing distinction.
///
/// The class determines which ROI a benefit may feed: only
/// [`BenefitClass::CashReleasing`] lines belong in a strict financial ROI;
/// capacity lines may be added for an economic ROI; qualitative lines are
/// never monetized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenefitClass {
    /// Cash the budget holder can actually bank (e.g. agency shifts not paid for).
    CashReleasing,
    /// Capacity freed and valued in money, but not banked (e.g. admin time freed).
    Capacity,
    /// Real but not monetized (e.g. staff satisfaction, safety).
    Qualitative,
}

/// One benefit line in an ROI case: its class and monetized amount.
///
/// Qualitative lines that are deliberately not monetized carry an amount of
/// zero — they stay visible in the case without inflating any ratio.
#[derive(Debug, Clone, Copy)]
pub struct BenefitLine {
    /// Benefit classification (cash-releasing, capacity, or qualitative).
    pub class: BenefitClass,
    /// Monetized value of the benefit in currency units (0.0 if not monetized).
    pub amount: f64,
}

/// ROI as a fraction: (benefits − costs) / costs.
///
/// Multiply by 100 for a percentage. Note the numerator is *net* benefit —
/// quoting gross benefits over costs is the classic inflation (see the
/// module pitfalls: "returns £3M" on £2M spend is 50% ROI, not 300%).
///
/// # Arguments
///
/// * `benefits` — total monetized benefits over the declared horizon.
/// * `costs` — total investment over the same horizon.
///
/// # Returns
///
/// ROI as a fraction (e.g. `0.50` = 50%), or `None` if `costs` is zero
/// (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::return_on_investment::roi;
///
/// // Doc pitfall: "returns £3M" on £2M spend is 50% ROI, not 300%.
/// let r = roi(3_000_000.0, 2_000_000.0).unwrap();
/// assert!((r - 0.50).abs() < 1e-9);
/// assert!(roi(100.0, 0.0).is_none());
/// ```
pub fn roi(benefits: f64, costs: f64) -> Option<f64> {
    if costs == 0.0 {
        None
    } else {
        Some((benefits - costs) / costs)
    }
}

/// Payback period in years: costs / annual net benefit.
///
/// The undiscounted time for cumulative net benefit to repay the investment.
///
/// # Arguments
///
/// * `costs` — total investment, currency units.
/// * `annual_net_benefit` — net benefit per year (benefits minus running
///   costs), currency units per year.
///
/// # Returns
///
/// Payback period in years, or `None` if `annual_net_benefit` is zero
/// (the investment never pays back). A negative annual net benefit yields a
/// negative (meaningless) period — the case simply does not pay back.
///
/// # Examples
///
/// ```rust
/// use health_economics::return_on_investment::payback_period_years;
///
/// // £500,000 cost with £250,000/year net benefit pays back in 2 years.
/// let p = payback_period_years(500_000.0, 250_000.0).unwrap();
/// assert!((p - 2.0).abs() < 1e-9);
/// ```
pub fn payback_period_years(costs: f64, annual_net_benefit: f64) -> Option<f64> {
    if annual_net_benefit == 0.0 {
        None
    } else {
        Some(costs / annual_net_benefit)
    }
}

/// Sum of benefit lines in the given classes.
///
/// The filtering primitive behind [`strict_financial_roi`] and
/// [`economic_roi`]: choose which benefit classes count, sum only those.
///
/// # Arguments
///
/// * `lines` — the benefit lines of the case.
/// * `include` — benefit classes to include in the total.
///
/// # Returns
///
/// Sum of `amount` over lines whose class is in `include`.
///
/// # Examples
///
/// ```rust
/// use health_economics::return_on_investment::{
///     BenefitClass, BenefitLine, total_benefits,
/// };
///
/// let lines = [
///     BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
///     BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
/// ];
/// let cash = total_benefits(&lines, &[BenefitClass::CashReleasing]);
/// assert!((cash - 450_000.0).abs() < 1e-9);
/// ```
pub fn total_benefits(lines: &[BenefitLine], include: &[BenefitClass]) -> f64 {
    lines
        .iter()
        .filter(|line| include.contains(&line.class))
        .map(|line| line.amount)
        .sum()
}

/// Strict financial ROI: counts only cash-releasing benefits.
///
/// This is the number a CFO can bank — the honest headline when the
/// audience is a budget holder.
///
/// # Arguments
///
/// * `lines` — the benefit lines of the case.
/// * `costs` — total investment, currency units.
///
/// # Returns
///
/// ROI as a fraction, or `None` if `costs` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::return_on_investment::{
///     BenefitClass, BenefitLine, strict_financial_roi,
/// };
///
/// // Doc: (450,000 − 500,000)/500,000 = −10%
/// let lines = [
///     BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
///     BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
/// ];
/// let r = strict_financial_roi(&lines, 500_000.0).unwrap();
/// assert!((r - (-0.10)).abs() < 1e-9);
/// ```
pub fn strict_financial_roi(lines: &[BenefitLine], costs: f64) -> Option<f64> {
    roi(total_benefits(lines, &[BenefitClass::CashReleasing]), costs)
}

/// Economic ROI: counts cash-releasing plus valued capacity benefits.
///
/// Legitimate as an economic statement, but must be labeled — quoting it as
/// if it were bankable cash is the category-blending failure.
///
/// # Arguments
///
/// * `lines` — the benefit lines of the case.
/// * `costs` — total investment, currency units.
///
/// # Returns
///
/// ROI as a fraction, or `None` if `costs` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::return_on_investment::{
///     BenefitClass, BenefitLine, economic_roi,
/// };
///
/// // Doc: (1,050,000 − 500,000)/500,000 = +110%
/// let lines = [
///     BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
///     BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
/// ];
/// let r = economic_roi(&lines, 500_000.0).unwrap();
/// assert!((r - 1.10).abs() < 1e-9);
/// ```
pub fn economic_roi(lines: &[BenefitLine], costs: f64) -> Option<f64> {
    roi(
        total_benefits(lines, &[BenefitClass::CashReleasing, BenefitClass::Capacity]),
        costs,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e_rostering_benefits() -> Vec<BenefitLine> {
        vec![
            // Cash-releasing: agency shift reduction £450,000
            BenefitLine { class: BenefitClass::CashReleasing, amount: 450_000.0 },
            // Capacity: ward-manager admin time freed £600,000 (valued, not banked)
            BenefitLine { class: BenefitClass::Capacity, amount: 600_000.0 },
            // Qualitative: staff satisfaction, safety — not monetized
            BenefitLine { class: BenefitClass::Qualitative, amount: 0.0 },
        ]
    }

    // Doc worked example: "Strict financial ROI = (450,000 − 500,000)/500,000 = −10%".
    #[test]
    fn strict_financial_roi_is_minus_10_percent() {
        let r = strict_financial_roi(&e_rostering_benefits(), 500_000.0).unwrap();
        // Doc: (450,000 − 500,000)/500,000 = −10%
        assert!((r - (-0.10)).abs() < 1e-9);
    }

    // Doc worked example: "Economic ROI = (1,050,000 − 500,000)/500,000 = +110%".
    #[test]
    fn economic_roi_is_plus_110_percent() {
        let r = economic_roi(&e_rostering_benefits(), 500_000.0).unwrap();
        // Doc: (1,050,000 − 500,000)/500,000 = +110%
        assert!((r - 1.10).abs() < 1e-9);
    }

    // Doc pitfall: "Gross instead of net: 'returns £3M' on £2M spend is 50%
    // ROI, not 300%".
    #[test]
    fn gross_vs_net_pitfall_3m_on_2m_is_50_percent_not_300() {
        // Doc pitfall: "returns £3M" on £2M spend is 50% ROI, not 300%.
        let r = roi(3_000_000.0, 2_000_000.0).unwrap();
        assert!((r - 0.50).abs() < 1e-9);
    }

    // Edge case: ROI is undefined at zero cost (ratio-maximization guard).
    #[test]
    fn roi_with_zero_costs_is_none() {
        assert!(roi(100.0, 0.0).is_none());
        assert!(strict_financial_roi(&e_rostering_benefits(), 0.0).is_none());
    }

    // Doc (The math): "Payback period = Costs / annual net benefit".
    #[test]
    fn payback_period_divides_costs_by_annual_net_benefit() {
        // £500,000 cost with £250,000/year net benefit pays back in 2 years.
        let p = payback_period_years(500_000.0, 250_000.0).unwrap();
        assert!((p - 2.0).abs() < 1e-9);
        assert!(payback_period_years(500_000.0, 0.0).is_none());
    }
}
