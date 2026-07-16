//! # Technical Debt
//!
//! Technical debt is the implied future cost of expedient past decisions in
//! a codebase: the remediation work owed (**principal**) and the ongoing
//! drag it exerts on delivery (**interest**). Quantification methods like
//! SQALE turn it from metaphor into a costed liability.
//!
//! Principal states the liability; interest — velocity drag plus excess
//! failures — makes the investment case for paydown.
//!
//! ## Formula
//!
//! ```text
//! SQALE principal = Σ over violations (remediation time) × developer cost rate
//! Technical debt ratio (TDR) = remediation cost / redevelopment cost × 100
//!     (SonarQube grades: A ≤ 5%, B ≤ 10%, C ≤ 20%, D ≤ 50%, E > 50%)
//!
//! interest/year = Δ delivery velocity × value per unit velocity
//!               + Δ defect rate × cost per defect
//! Paydown case  = PV(interest avoided over horizon) − remediation cost
//!
//! remediation cost     cost to fix the violations (the principal)
//! redevelopment cost   cost to rebuild the system from scratch
//! PV                   present value at the declared discount rate
//! ```
//!
//! ## Why it matters
//!
//! Unquantified, tech debt is a whine; quantified, it is a business case.
//! Industry baselines (CAST Appmarq, 1,400 apps / 550M LOC): historically
//! ≈ $3.61 of technical-debt principal per line of code, with typical
//! codebases carrying a debt ratio of 15–20% of rebuild cost, versus a
//! commonly used health bar of ≤ 5% (SonarQube's "A" grade). The
//! health-economics frame fits precisely: debt is a chronic condition —
//! untreated, it progresses, its "interest" compounds as slower delivery
//! and higher defect rates, and remediation competes for capacity against
//! feature work exactly as prevention competes with treatment. Paying £500k
//! principal to avoid £40k/year interest is a bad trade; to avoid
//! £400k/year, excellent.
//!
//! ## Example
//!
//! A 400k-LOC clinical-records integration layer: SQALE principal 3,800
//! hours × £75 = £285k; TDR ≈ 12% (grade C). The layer absorbs 6,000
//! dev-hours/year with 40% longer cycle times and 12 extra failures/year at
//! £8,000 each:
//!
//! ```rust
//! use health_economics::technical_debt::{
//!     sqale_principal, technical_debt_ratio_percent, sqale_grade, SqaleGrade,
//!     annual_interest, interest_avoided_per_year, payback_period_years,
//! };
//!
//! // SQALE principal: 3,800 hours × £75 = £285k
//! let principal = sqale_principal(3_800.0, 75.0);
//! assert!((principal - 285_000.0).abs() < 1e-9);
//!
//! // TDR ≈ 12% → grade C
//! let tdr = technical_debt_ratio_percent(principal, 2_375_000.0).unwrap();
//! assert!((tdr - 12.0).abs() < 1e-9);
//! assert_eq!(sqale_grade(tdr), SqaleGrade::C);
//!
//! // Interest ≈ 6,000 × 0.40 × £75 + 12 × £8,000 = £276,000/year
//! let interest = annual_interest(6_000.0, 0.40, 75.0, 12.0, 8_000.0);
//! assert!((interest - 276_000.0).abs() < 1e-9);
//!
//! // Remediate the worst 30% of principal (£85.5k), modeled interest
//! // reduction 60% → saves ~£166k/year, payback ≈ 6 months.
//! let saved = interest_avoided_per_year(interest, 0.60);
//! assert!((saved - 165_600.0).abs() < 1e-9);
//! let months = payback_period_years(0.30 * principal, saved).unwrap() * 12.0;
//! assert!((months - 6.0).abs() < 0.5);
//! ```
//!
//! The hotspot targeting matters: debt interest concentrates where change
//! frequency × debt density peaks — remediating rarely-touched debt buys
//! nothing, like treating a condition that would never progress.
//!
//! ## Software engineering connection
//!
//! - Express the estate as a burden inventory (DALY-style — where are the
//!   lost healthy engineering-years?).
//! - Justify paydown with progression math, honestly: usually
//!   cost-effective, not cost-saving.
//! - Weight the worst systems' remediation by severity shortfall.
//! - Submit big remediation proposals with an offset analysis that survives
//!   the avoided-downstream-costs rules — probability-weighted, discounted,
//!   counted once.
//!
//! ## Pitfalls
//!
//! - Principal-only reporting: a big scary number with no interest estimate
//!   justifies nothing.
//! - Tool-generated debt figures taken literally: SQALE counts rule
//!   violations; it misses architectural debt (the expensive kind) and
//!   counts trivia.
//! - Debt-zero utopianism: the optimal debt level is not zero — debt is
//!   leverage; the question is the interest rate.
//! - "The rewrite avoids all of it": rewrite proposals must clear the same
//!   offset rules — counterfactual cost, probability, discounting.
//!
//! ## Sources
//!
//! - CAST, technical debt estimation.
//!   <https://www.castsoftware.com/glossary/technical-debt-estimation>
//! - Letouzey J-L, "The SQALE method for evaluating Technical Debt."
//!   <https://www.researchgate.net/publication/239763591_The_SQALE_method_for_evaluating_Technical_Debt>
//!
//! Topic doc: health-economics-metrics/topics/technical-debt.md

/// SonarQube-style maintainability grade derived from the technical debt ratio.
///
/// Bands are inclusive at the upper bound: a TDR of exactly 5% still
/// grades A.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqaleGrade {
    /// TDR ≤ 5% — the commonly used health bar.
    A,
    /// 5% < TDR ≤ 10%.
    B,
    /// 10% < TDR ≤ 20% — the 15–20% band where typical codebases sit.
    C,
    /// 20% < TDR ≤ 50%.
    D,
    /// TDR > 50%.
    E,
}

/// SQALE principal: total remediation hours × developer cost rate.
///
/// The costed liability — what it would take to fix all counted violations.
/// Remember the pitfall: SQALE counts rule violations, missing
/// architectural debt and counting trivia.
///
/// # Arguments
///
/// * `remediation_hours` — total estimated remediation time, hours
///   (Σ over violations).
/// * `cost_per_hour` — developer cost rate, currency/hour.
///
/// # Returns
///
/// The principal in currency units.
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::sqale_principal;
///
/// // Doc: 3,800 hours × £75 = £285k.
/// assert!((sqale_principal(3_800.0, 75.0) - 285_000.0).abs() < 1e-9);
/// ```
pub fn sqale_principal(remediation_hours: f64, cost_per_hour: f64) -> f64 {
    remediation_hours * cost_per_hour
}

/// Technical debt ratio as a percentage: remediation cost / redevelopment
/// cost × 100.
///
/// # Arguments
///
/// * `remediation_cost` — cost to fix the violations (the principal).
/// * `redevelopment_cost` — cost to rebuild the system from scratch.
///
/// # Returns
///
/// TDR in percent (e.g. `12.0` for 12%), or `None` if `redevelopment_cost`
/// is zero (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::technical_debt_ratio_percent;
///
/// // Doc: £285k remediation vs ~£2.375M rebuild → TDR ≈ 12%.
/// let tdr = technical_debt_ratio_percent(285_000.0, 2_375_000.0).unwrap();
/// assert!((tdr - 12.0).abs() < 1e-9);
/// assert!(technical_debt_ratio_percent(285_000.0, 0.0).is_none());
/// ```
pub fn technical_debt_ratio_percent(
    remediation_cost: f64,
    redevelopment_cost: f64,
) -> Option<f64> {
    if redevelopment_cost == 0.0 {
        None
    } else {
        Some(remediation_cost / redevelopment_cost * 100.0)
    }
}

/// Grade a technical debt ratio (in percent) on the SonarQube scale.
///
/// Bands: A ≤ 5%, B ≤ 10%, C ≤ 20%, D ≤ 50%, E > 50%.
///
/// # Arguments
///
/// * `tdr_percent` — technical debt ratio in percent (see
///   [`technical_debt_ratio_percent`]).
///
/// # Returns
///
/// The [`SqaleGrade`] band containing the ratio.
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::{sqale_grade, SqaleGrade};
///
/// // Doc worked example: TDR ≈ 12% is grade C.
/// assert_eq!(sqale_grade(12.0), SqaleGrade::C);
/// assert_eq!(sqale_grade(5.0), SqaleGrade::A);
/// assert_eq!(sqale_grade(50.1), SqaleGrade::E);
/// ```
pub fn sqale_grade(tdr_percent: f64) -> SqaleGrade {
    if tdr_percent <= 5.0 {
        SqaleGrade::A
    } else if tdr_percent <= 10.0 {
        SqaleGrade::B
    } else if tdr_percent <= 20.0 {
        SqaleGrade::C
    } else if tdr_percent <= 50.0 {
        SqaleGrade::D
    } else {
        SqaleGrade::E
    }
}

/// Annual interest on technical debt: velocity drag plus excess-failure cost.
///
/// This is the number that justifies paydown — principal alone justifies
/// nothing.
///
/// # Arguments
///
/// * `dev_hours_absorbed_per_year` — dev-hours/year spent working in the
///   debt-laden area.
/// * `velocity_drag_fraction` — excess cycle time as a fraction (e.g. 0.40
///   for cycle times 40% longer than the estate baseline).
/// * `cost_per_hour` — developer cost rate, currency/hour.
/// * `extra_failures_per_year` — change failures per year above the estate
///   baseline.
/// * `cost_per_failure` — cost per failure (rework, incident response).
///
/// # Returns
///
/// Interest in currency units per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::annual_interest;
///
/// // Doc: 6,000 × 0.40 × £75 = £180,000 velocity drag
/// //    + 12 extra failures × £8,000 = £96,000 → ≈ £276,000/year.
/// let interest = annual_interest(6_000.0, 0.40, 75.0, 12.0, 8_000.0);
/// assert!((interest - 276_000.0).abs() < 1e-9);
/// ```
pub fn annual_interest(
    dev_hours_absorbed_per_year: f64,
    velocity_drag_fraction: f64,
    cost_per_hour: f64,
    extra_failures_per_year: f64,
    cost_per_failure: f64,
) -> f64 {
    // Velocity-drag term (hours × excess-cycle-time fraction × rate)
    // plus the excess-failure term (failures × cost each).
    dev_hours_absorbed_per_year * velocity_drag_fraction * cost_per_hour
        + extra_failures_per_year * cost_per_failure
}

/// Annual interest avoided by a partial, hotspot-targeted remediation.
///
/// Hotspot targeting is why a 30% principal spend can remove 60% of the
/// interest: debt interest concentrates where change frequency × debt
/// density peaks.
///
/// # Arguments
///
/// * `annual_interest` — current interest, currency/year (see
///   [`annual_interest`]).
/// * `interest_reduction_fraction` — modeled fraction (0..1) of interest
///   removed by the remediation.
///
/// # Returns
///
/// Interest avoided, currency units per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::interest_avoided_per_year;
///
/// // Doc: 60% interest reduction on £276k/year → saves ~£166k/year.
/// let saved = interest_avoided_per_year(276_000.0, 0.60);
/// assert!((saved - 165_600.0).abs() < 1e-9);
/// ```
pub fn interest_avoided_per_year(annual_interest: f64, interest_reduction_fraction: f64) -> f64 {
    annual_interest * interest_reduction_fraction
}

/// Payback period in years for a remediation: cost / annual interest avoided.
///
/// # Arguments
///
/// * `remediation_cost` — cost of the (possibly partial) remediation.
/// * `interest_avoided_per_year` — interest removed per year (see
///   [`interest_avoided_per_year`]).
///
/// # Returns
///
/// Payback period in years, or `None` if no interest is avoided (the
/// remediation never pays back).
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::payback_period_years;
///
/// // Doc: £85.5k remediation against ~£165.6k/year avoided → payback ≈ 6 months.
/// let months = payback_period_years(85_500.0, 165_600.0).unwrap() * 12.0;
/// assert!((months - 6.0).abs() < 0.5);
/// ```
pub fn payback_period_years(
    remediation_cost: f64,
    interest_avoided_per_year: f64,
) -> Option<f64> {
    if interest_avoided_per_year == 0.0 {
        None
    } else {
        Some(remediation_cost / interest_avoided_per_year)
    }
}

/// Present value of a constant annual interest stream avoided over a horizon.
///
/// Discounts interest avoided in years 1..=`horizon_years` at rate `r`;
/// nothing accrues in year 0. With `discount_rate` = 0 this is simply
/// horizon × annual saving.
///
/// # Arguments
///
/// * `interest_avoided_per_year` — constant annual saving, currency/year.
/// * `discount_rate` — annual discount rate as a fraction (e.g. 0.035 for
///   the UK Green Book's 3.5%).
/// * `horizon_years` — number of years the saving is claimed.
///
/// # Returns
///
/// Present value of the stream in currency units.
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::pv_of_interest_avoided;
///
/// // Undiscounted sanity: r = 0 over 3 years gives exactly 3 × £165,600.
/// let pv0 = pv_of_interest_avoided(165_600.0, 0.0, 3);
/// assert!((pv0 - 496_800.0).abs() < 1e-9);
/// // Discounting at 3.5% shrinks it.
/// let pv = pv_of_interest_avoided(165_600.0, 0.035, 3);
/// assert!(pv < pv0);
/// ```
pub fn pv_of_interest_avoided(
    interest_avoided_per_year: f64,
    discount_rate: f64,
    horizon_years: u32,
) -> f64 {
    // Standard annuity PV: each year-t saving discounted by (1 + r)^t.
    (1..=horizon_years)
        .map(|t| interest_avoided_per_year / (1.0 + discount_rate).powi(t as i32))
        .sum()
}

/// The paydown case: PV(interest avoided over the horizon) − remediation cost.
///
/// Positive means the remediation is worth doing on interest alone (any
/// principal reduction is a bonus).
///
/// # Arguments
///
/// * `pv_interest_avoided` — present value of the interest stream avoided
///   (see [`pv_of_interest_avoided`]).
/// * `remediation_cost` — cost of the remediation.
///
/// # Returns
///
/// Net value of the paydown (negative means a bad trade).
///
/// # Examples
///
/// ```rust
/// use health_economics::technical_debt::{
///     paydown_net_value, pv_of_interest_avoided,
/// };
///
/// // Doc (The math): £500k principal to avoid £40k/year is a bad trade;
/// // to avoid £400k/year, excellent (5-year undiscounted horizon).
/// let bad = paydown_net_value(pv_of_interest_avoided(40_000.0, 0.0, 5), 500_000.0);
/// assert!(bad < 0.0);
/// let good = paydown_net_value(pv_of_interest_avoided(400_000.0, 0.0, 5), 500_000.0);
/// assert!(good > 0.0);
/// ```
pub fn paydown_net_value(pv_interest_avoided: f64, remediation_cost: f64) -> f64 {
    pv_interest_avoided - remediation_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc worked example: "SQALE principal 3,800 hours × £75 = £285k".
    #[test]
    fn sqale_principal_is_285k() {
        // Doc: 3,800 hours × £75 = £285k
        assert!((sqale_principal(3_800.0, 75.0) - 285_000.0).abs() < 1e-9);
    }

    // Doc worked example: "TDR ≈ 12% (grade C)".
    #[test]
    fn tdr_12_percent_grades_c() {
        // Doc: TDR ≈ 12% (grade C). 285k remediation vs ~2.375M rebuild → 12%.
        let tdr = technical_debt_ratio_percent(285_000.0, 2_375_000.0).unwrap();
        assert!((tdr - 12.0).abs() < 1e-9);
        assert_eq!(sqale_grade(tdr), SqaleGrade::C);
    }

    // Doc (The math): "SonarQube grades: A ≤ 5%, B ≤ 10%, C ≤ 20%, D ≤ 50%".
    #[test]
    fn grade_boundaries_match_sonarqube_scale() {
        assert_eq!(sqale_grade(5.0), SqaleGrade::A);
        assert_eq!(sqale_grade(10.0), SqaleGrade::B);
        assert_eq!(sqale_grade(20.0), SqaleGrade::C);
        assert_eq!(sqale_grade(50.0), SqaleGrade::D);
        assert_eq!(sqale_grade(50.1), SqaleGrade::E);
    }

    // Edge case: TDR is undefined when redevelopment cost is zero.
    #[test]
    fn tdr_with_zero_redevelopment_cost_is_none() {
        assert!(technical_debt_ratio_percent(285_000.0, 0.0).is_none());
    }

    // Doc worked example: "Interest ≈ 6,000 × 0.40 × £75 = £180,000/year
    // (velocity drag) + 12 extra failures × £8,000 = £96,000/year ≈ £276,000/year".
    #[test]
    fn annual_interest_is_276k() {
        // Doc: 6,000 × 0.40 × £75 = £180,000 velocity drag
        //    + 12 extra failures × £8,000 = £96,000 → ≈ £276,000/year
        let interest = annual_interest(6_000.0, 0.40, 75.0, 12.0, 8_000.0);
        assert!((interest - 276_000.0).abs() < 1e-9);
        let velocity_drag = annual_interest(6_000.0, 0.40, 75.0, 0.0, 0.0);
        assert!((velocity_drag - 180_000.0).abs() < 1e-9);
        let failures = annual_interest(0.0, 0.0, 75.0, 12.0, 8_000.0);
        assert!((failures - 96_000.0).abs() < 1e-9);
    }

    // Doc worked example: "modeled interest reduction 60%: saves ~£166k/year".
    #[test]
    fn hotspot_remediation_saves_about_166k_per_year() {
        // Doc: interest reduction 60% → saves ~£166k/year (exact 165,600)
        let saved = interest_avoided_per_year(276_000.0, 0.60);
        assert!((saved - 166_000.0).abs() < 500.0);
    }

    // Doc worked example: "Remediate the worst 30% of principal (£85k) ...
    // Payback ≈ 6 months".
    #[test]
    fn payback_is_about_6_months() {
        // Doc: remediate worst 30% of principal (£85k, i.e. 0.30 × £285k = £85.5k)
        // against ~£166k/year avoided → payback ≈ 6 months.
        let cost = 0.30 * sqale_principal(3_800.0, 75.0);
        let saved = interest_avoided_per_year(276_000.0, 0.60);
        let months = payback_period_years(cost, saved).unwrap() * 12.0;
        assert!((months - 6.0).abs() < 0.5);
    }

    // Doc (The math): "Paydown case = PV(interest avoided over horizon) −
    // remediation cost" — strongly positive for the worked example over 3 years.
    #[test]
    fn paydown_case_is_strongly_positive_over_three_years() {
        // PV of ~£165.6k/year over 3 years at 3.5% ≫ the £85.5k remediation cost.
        let pv = pv_of_interest_avoided(165_600.0, 0.035, 3);
        let net = paydown_net_value(pv, 85_500.0);
        assert!(net > 300_000.0);
        // Undiscounted sanity: r = 0 gives exactly 3 × the annual saving.
        let pv0 = pv_of_interest_avoided(165_600.0, 0.0, 3);
        assert!((pv0 - 496_800.0).abs() < 1e-9);
    }

    // Doc (The math): "Paying £500k principal to avoid £40k/year interest is
    // a bad trade; to avoid £400k/year, excellent."
    #[test]
    fn bad_trade_vs_good_trade_from_the_math_section() {
        // Doc: paying £500k principal to avoid £40k/year is a bad trade;
        // to avoid £400k/year, excellent. (5-year undiscounted horizon.)
        let bad = paydown_net_value(pv_of_interest_avoided(40_000.0, 0.0, 5), 500_000.0);
        assert!(bad < 0.0);
        let good = paydown_net_value(pv_of_interest_avoided(400_000.0, 0.0, 5), 500_000.0);
        assert!(good > 0.0);
    }
}
