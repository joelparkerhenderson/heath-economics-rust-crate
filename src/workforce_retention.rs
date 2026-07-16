//! # Workforce Retention
//!
//! Workforce retention economics quantify what staff turnover costs a health
//! system — recruitment, onboarding/productivity ramp, and vacancy cover at
//! agency premium — and therefore what software that reduces administrative
//! burnout is worth. Burnout from repetitive administrative data tasks is a
//! primary driver of staff turnover and sickness absence in the NHS.
//!
//! Because turnover costs are real cash, retention improvements are among
//! the few workforce benefits a finance director can bank — and
//! administrative friction is consistently among the top cited drivers of
//! clinical burnout, which makes it a software-addressable cost.
//!
//! ## Formula
//!
//! ```text
//! Cost per leaver = recruitment cost + onboarding/productivity-ramp cost
//!                 + vacancy cover premium × vacancy duration
//!
//! Annual turnover cost = headcount × turnover rate × cost per leaver
//!
//! Value of software  = headcount × Δturnover rate × cost per leaver
//!                    + sickness-absence reduction × cover cost/day
//!
//! where:
//!   recruitment cost       — advertising, agency fees, interviews (£)
//!   onboarding/ramp cost   — months of reduced productivity, supervision (£)
//!   vacancy cover premium  — agency/locum premium (typically 2–3×
//!                            substantive Agenda for Change rates)
//!   turnover rate          — leavers per year / headcount (fraction)
//!   Δturnover rate         — turnover-rate reduction claimed for the
//!                            software (fraction, keep it modest)
//! ```
//!
//! ## Why it matters
//!
//! When a clinician leaves, the trust pays three times: to recruit a
//! replacement (advertising, agency fees, interviews), to onboard them
//! (months of reduced productivity, supervision), and to cover the vacancy
//! meanwhile — typically with agency or locum staff at 2–3× substantive
//! Agenda for Change rates. In the worked example, a leaver costs ≈ £18,500
//! (recruitment £4,500 + onboarding £6,000 + vacancy cover £8,000), so a
//! 1,200-nurse trust at 11% turnover burns ≈ £2.44M/year — and a modest
//! 1-percentage-point turnover improvement from documentation-burden
//! software is worth £222,000/year of cash-relevant value. A 1-point claim
//! backed by staff-survey friction scores is credible; a 4-point claim is
//! not.
//!
//! ## Example
//!
//! A trust employs 1,200 nurses at 11%/year turnover; documentation-burden
//! software (auto-populated assessments, single sign-on, dictation)
//! plausibly moves turnover 1 percentage point.
//!
//! ```rust
//! use health_economics::workforce_retention::{
//!     CostPerLeaver, vacancy_cover_cost, annual_turnover_cost, retention_value,
//! };
//!
//! // Vacancy cover: 4 months × 0.6 WTE at a ~£3,333/month agency premium ≈ £8,000.
//! let cover = vacancy_cover_cost(10_000.0 / 3.0, 4.0, 0.6);
//! assert!((cover - 8_000.0).abs() < 1.0);
//!
//! // Recruitment £4,500 + onboarding £6,000 + vacancy cover £8,000 ≈ £18,500.
//! let leaver = CostPerLeaver {
//!     recruitment: 4_500.0,
//!     onboarding_ramp: 6_000.0,
//!     vacancy_cover: 8_000.0,
//! };
//! assert_eq!(leaver.total(), 18_500.0);
//!
//! // Baseline turnover cost = 1,200 × 0.11 × 18,500 ≈ £2.44M/year.
//! let baseline = annual_turnover_cost(1_200.0, 0.11, leaver.total());
//! assert_eq!(baseline, 2_442_000.0);
//!
//! // Value = 1,200 × 0.01 × 18,500 = £222,000/year cash-relevant.
//! let value = retention_value(1_200.0, 0.01, leaver.total());
//! assert_eq!(value, 222_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineering retention math is identical and worse-documented: replacing
//!   a senior engineer costs 6–12 months of loaded salary (recruiting, ramp,
//!   lost context).
//! - A 200-person org at 15% attrition burns millions annually on churn.
//! - Developer-experience investment (SPACE and DevEx) is the direct
//!   analogue of documentation-burden relief for nurses — justify it the
//!   same way: measured friction scores, a modest claimed effect on
//!   attrition, cost per leaver from your own finance data.
//! - The health-economics discipline to copy is *costing the leaver
//!   honestly* rather than arguing about whether people "really" leave over
//!   tooling.
//!
//! ## Pitfalls
//!
//! - **Attributing all turnover movement to your intervention** — labor
//!   markets move turnover far more than software does; use control groups
//!   or at least sector trend adjustment.
//! - **Double counting**: retention savings and agency-spend savings overlap
//!   (vacancy cover *is* agency spend); reconcile the lines.
//! - **Ignoring the lag**: burnout-driven attrition responds to friction
//!   changes over 1–2 years, not the next quarter.
//!
//! ## Sources
//!
//! - NHS England, reducing agency spend.
//!   <https://www.england.nhs.uk/long-read/reducing-agency-spend-in-the-nhs/>
//! - NHS Staff Survey (burnout and intention-to-leave data).
//!   <https://www.nhsstaffsurveys.com/>
//!
//! Topic doc: health-economics-metrics/topics/workforce-retention.md

/// The cash components of replacing one leaver.
///
/// A trust pays three times when a clinician leaves: recruitment,
/// onboarding, and vacancy cover. All fields are in the same currency (£).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostPerLeaver {
    /// Recruitment cost: advertising, agency fees, interviews (£).
    pub recruitment: f64,
    /// Onboarding/productivity-ramp cost: months of reduced productivity,
    /// supervision (£).
    pub onboarding_ramp: f64,
    /// Vacancy cover cost: agency/locum premium over the vacancy duration
    /// (£) — see [`vacancy_cover_cost`].
    pub vacancy_cover: f64,
}

impl CostPerLeaver {
    /// Total cost per leaver: recruitment + onboarding/ramp + vacancy cover.
    ///
    /// # Returns
    ///
    /// Total replacement cost per leaver (£).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::workforce_retention::CostPerLeaver;
    ///
    /// // Recruitment £4,500 + onboarding £6,000 + vacancy cover £8,000
    /// // ≈ £18,500 per leaver.
    /// let cost = CostPerLeaver {
    ///     recruitment: 4_500.0,
    ///     onboarding_ramp: 6_000.0,
    ///     vacancy_cover: 8_000.0,
    /// };
    /// assert_eq!(cost.total(), 18_500.0);
    /// ```
    pub fn total(&self) -> f64 {
        self.recruitment + self.onboarding_ramp + self.vacancy_cover
    }
}

/// Vacancy cover cost while a post stands empty.
///
/// Agency premium per month × vacancy duration in months × the fraction of
/// the vacant WTE actually covered by agency staff (posts are rarely
/// backfilled at full WTE). Agency/locum rates typically run 2–3×
/// substantive Agenda for Change rates.
///
/// # Arguments
///
/// * `agency_premium_per_month` — agency/locum premium over substantive cost
///   per month of full-WTE cover (£/month).
/// * `vacancy_months` — how long the post stays vacant (months).
/// * `wte_fraction_covered` — fraction of the vacant WTE actually covered by
///   agency staff (0..1).
///
/// # Returns
///
/// Vacancy cover cost per leaver (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::workforce_retention::vacancy_cover_cost;
///
/// // 4 months × 0.6 WTE at a £10,000/3 ≈ £3,333/month premium ≈ £8,000.
/// let cover = vacancy_cover_cost(10_000.0 / 3.0, 4.0, 0.6);
/// assert!((cover - 8_000.0).abs() < 1.0);
/// ```
pub fn vacancy_cover_cost(
    agency_premium_per_month: f64,
    vacancy_months: f64,
    wte_fraction_covered: f64,
) -> f64 {
    agency_premium_per_month * vacancy_months * wte_fraction_covered
}

/// Annual turnover cost: headcount × turnover rate × cost per leaver.
///
/// The baseline cash burn of churn, before any intervention.
///
/// # Arguments
///
/// * `headcount` — staff in the group (e.g. 1,200 nurses).
/// * `turnover_rate` — leavers per year as a fraction of headcount (e.g.
///   0.11 for 11%/year).
/// * `cost_per_leaver` — total replacement cost per leaver (£), e.g. from
///   [`CostPerLeaver::total`].
///
/// # Returns
///
/// Annual turnover cost (£/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::workforce_retention::annual_turnover_cost;
///
/// // 1,200 × 0.11 × £18,500 = £2,442,000 ≈ £2.44M/year.
/// let cost = annual_turnover_cost(1_200.0, 0.11, 18_500.0);
/// assert_eq!(cost, 2_442_000.0);
/// ```
pub fn annual_turnover_cost(headcount: f64, turnover_rate: f64, cost_per_leaver: f64) -> f64 {
    headcount * turnover_rate * cost_per_leaver
}

/// Retention value of software: headcount × Δturnover rate × cost per
/// leaver.
///
/// Keep the claimed Δ modest — the causal chain (software → friction →
/// turnover) has two estimated links, so evidence both (staff surveys
/// pre/post; published burnout-attrition associations). A 1-point claim
/// backed by staff-survey friction scores is credible; a 4-point claim is
/// not.
///
/// # Arguments
///
/// * `headcount` — staff in the group.
/// * `turnover_rate_reduction` — Δturnover rate, as a fraction (0.01 for a
///   1-percentage-point improvement).
/// * `cost_per_leaver` — total replacement cost per leaver (£).
///
/// # Returns
///
/// Annual cash-relevant retention value (£/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::workforce_retention::retention_value;
///
/// // 1,200 × 0.01 × £18,500 = £222,000/year for a 1-point improvement.
/// let value = retention_value(1_200.0, 0.01, 18_500.0);
/// assert_eq!(value, 222_000.0);
/// ```
pub fn retention_value(
    headcount: f64,
    turnover_rate_reduction: f64,
    cost_per_leaver: f64,
) -> f64 {
    headcount * turnover_rate_reduction * cost_per_leaver
}

/// Full software value line: retention value plus the sickness-absence
/// term.
///
/// Adds days of sickness absence avoided × cover cost per day to the
/// retention value. Beware double counting: vacancy cover *is* agency
/// spend, so reconcile this line against any separate agency-savings claim.
///
/// # Arguments
///
/// * `headcount` — staff in the group.
/// * `turnover_rate_reduction` — Δturnover rate, as a fraction.
/// * `cost_per_leaver` — total replacement cost per leaver (£).
/// * `sickness_days_avoided` — days of sickness absence avoided per year.
/// * `cover_cost_per_day` — cost of covering one day of absence (£/day).
///
/// # Returns
///
/// Total annual software value (£/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::workforce_retention::software_value;
///
/// // With no sickness effect claimed, equals the retention value (£222,000).
/// assert_eq!(software_value(1_200.0, 0.01, 18_500.0, 0.0, 0.0), 222_000.0);
///
/// // Adding 100 avoided absence days at £250/day cover: £247,000/year.
/// assert_eq!(software_value(1_200.0, 0.01, 18_500.0, 100.0, 250.0), 247_000.0);
/// ```
pub fn software_value(
    headcount: f64,
    turnover_rate_reduction: f64,
    cost_per_leaver: f64,
    sickness_days_avoided: f64,
    cover_cost_per_day: f64,
) -> f64 {
    retention_value(headcount, turnover_rate_reduction, cost_per_leaver)
        + sickness_days_avoided * cover_cost_per_day
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "Vacancy cover: 4 months × 0.6 WTE covered by agency
    // premium ≈ £8,000".
    /// Vacancy cover ≈ £8,000: 4 months × 0.6 WTE at a £10,000/3 monthly
    /// agency premium reproduces the doc's rounded figure.
    #[test]
    fn vacancy_cover_is_about_8_000() {
        let cover = vacancy_cover_cost(10_000.0 / 3.0, 4.0, 0.6);
        assert!((cover - 8_000.0).abs() < 1.0);
    }

    // Worked example: "Recruitment ≈ £4,500; onboarding/ramp ≈ £6,000; ...
    // Total ≈ £18,500 per leaver".
    /// Recruitment £4,500 + onboarding £6,000 + vacancy cover £8,000
    /// ≈ £18,500 per leaver.
    #[test]
    fn cost_per_leaver_is_18_500() {
        let cost = CostPerLeaver {
            recruitment: 4_500.0,
            onboarding_ramp: 6_000.0,
            vacancy_cover: 8_000.0,
        };
        assert!((cost.total() - 18_500.0).abs() < 1e-9);
    }

    // Worked example: "Baseline turnover cost = 1,200 × 0.11 × 18,500
    // ≈ £2.44M/year".
    /// Baseline turnover cost = 1,200 × 0.11 × 18,500 ≈ £2.44M/year
    /// (exact: £2,442,000).
    #[test]
    fn baseline_turnover_cost_is_about_2_44_million() {
        let cost = annual_turnover_cost(1_200.0, 0.11, 18_500.0);
        assert!((cost - 2_442_000.0).abs() < 1e-9);
        assert!((cost - 2_440_000.0).abs() < 5_000.0); // doc's ≈ £2.44M
    }

    // Worked example: "Value = 1,200 × 0.01 × 18,500 = £222,000/year
    // cash-relevant".
    /// Value = 1,200 × 0.01 × 18,500 = £222,000/year for a 1-point
    /// turnover-rate improvement.
    #[test]
    fn one_point_turnover_reduction_is_worth_222_000() {
        let value = retention_value(1_200.0, 0.01, 18_500.0);
        assert!((value - 222_000.0).abs() < 1e-9);
    }

    // Verifies the doc's "Value of software" line: retention term plus
    // "sickness-absence reduction × cover cost/day".
    /// The full software-value line adds the sickness-absence term; with no
    /// sickness effect claimed it equals the retention value.
    #[test]
    fn software_value_adds_sickness_absence_term() {
        let base = software_value(1_200.0, 0.01, 18_500.0, 0.0, 0.0);
        assert!((base - 222_000.0).abs() < 1e-9);
        let with_sickness = software_value(1_200.0, 0.01, 18_500.0, 100.0, 250.0);
        assert!((with_sickness - 247_000.0).abs() < 1e-9);
    }
}
