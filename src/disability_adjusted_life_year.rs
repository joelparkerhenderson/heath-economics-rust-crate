//! # Disability-Adjusted Life Year (DALY)
//!
//! A DALY is one lost year of healthy life — the burden-side mirror of the
//! QALY. Where QALYs count health *gained*, DALYs count health *lost* to
//! disease; interventions are valued by DALYs **averted**.
//!
//! The burden combines mortality (years of life lost, YLL) and morbidity
//! (years lived with disability, YLD), with disability weights published by
//! the Global Burden of Disease study.
//!
//! ## Formula
//!
//! ```text
//! DALY = YLL + YLD
//!
//! YLL (years of life lost)          = deaths × standard life expectancy at age of death
//! YLD (years lived with disability) = prevalence × disability weight
//!
//! disability weight ∈ [0, 1], 0 = full health, 1 = equivalent to death
//! (weights published by the Global Burden of Disease study)
//! ```
//!
//! ## Why it matters
//!
//! The DALY is the global-health standard (WHO, the Global Burden of Disease
//! study, and most low- and middle-income-country health ministries plan in
//! DALYs). If your software targets international health systems, donors, or
//! WHO-aligned programs, the value language is DALYs averted, not QALYs
//! gained. WHO-CHOICE's historical benchmark: an intervention averting a
//! DALY for less than 1× GDP per capita is "highly cost-effective", 1–3× GDP
//! per capita "cost-effective" (WHO now discourages rigid use of these bands,
//! but they remain ubiquitous in practice).
//!
//! ## Example
//!
//! The topic doc's worked example: a screening-reminder platform annually
//! prevents 10 premature deaths (each losing 20 years) and prevents 200
//! people living a year with a condition of disability weight 0.2 — 240
//! DALYs averted/year. At $600,000/year the cost per DALY averted is $2,500,
//! well under the 1× GDP benchmark in a country with GDP per capita of
//! $8,000.
//!
//! ```rust
//! use health_economics::disability_adjusted_life_year::{
//!     years_of_life_lost, years_lived_with_disability, dalys,
//!     cost_per_daly_averted, who_choice_band, WhoChoiceBand,
//! };
//!
//! // YLL averted = 10 × 20 = 200; YLD averted = 200 × 0.2 = 40.
//! let yll = years_of_life_lost(10.0, 20.0);
//! let yld = years_lived_with_disability(200.0, 0.2);
//! assert_eq!(yll, 200.0);
//! assert_eq!(yld, 40.0);
//!
//! // DALYs averted = 240 per year.
//! let averted = dalys(yll, yld);
//! assert_eq!(averted, 240.0);
//!
//! // $600,000 / 240 = $2,500 per DALY averted.
//! let cpd = cost_per_daly_averted(600_000.0, averted).unwrap();
//! assert_eq!(cpd, 2_500.0);
//!
//! // Under 1× GDP per capita ($8,000): "highly cost-effective".
//! assert_eq!(who_choice_band(cpd, 8_000.0), WhoChoiceBand::HighlyCostEffective);
//! ```
//!
//! ## Software engineering connection
//!
//! - Digital health aimed at global health funders (Gavi, Global Fund,
//!   national programs) should express impact as **cost per DALY averted** —
//!   it is the metric grant reviewers already think in.
//! - The DALY is also a useful *burden accounting* template for engineering:
//!   incidents, flaky builds, and legacy friction are "years lived with
//!   disability" for a codebase.
//! - A toil-weighted burden inventory tells you where remediation buys the
//!   most "healthy engineering years", the same way GBD burden tables direct
//!   health spending.
//!
//! ## Pitfalls
//!
//! - **QALYs gained ≠ DALYs averted numerically** — different weights,
//!   different life tables, different conventions (DALYs historically used
//!   age-weighting and discounting inside the measure). Don't convert
//!   casually.
//! - **Using GDP-multiple thresholds as a rubber stamp** — WHO itself warns
//!   they ignore budgets and opportunity cost.
//! - **Claiming population-scale DALYs from per-user efficacy** without
//!   multiplying through uptake and adherence.
//!
//! ## Sources
//!
//! - WHO indicator registry: DALYs.
//!   <https://www.who.int/data/gho/indicator-metadata-registry/imr-details/158>
//! - Bertram MY, et al. "Cost-effectiveness thresholds: pros and cons." (WHO)
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC4339959/>
//! - Marseille E, et al. on GDP-based thresholds, Health Policy and Planning.
//!   <https://academic.oup.com/heapol/article/32/1/141/2555408>
//!
//! Topic doc: health-economics-metrics/topics/disability-adjusted-life-year.md

/// WHO-CHOICE cost-effectiveness band for cost per DALY averted, relative to
/// GDP per capita.
///
/// WHO now discourages rigid use of these GDP-multiple bands (they ignore
/// budgets and opportunity cost), but they remain ubiquitous in practice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoChoiceBand {
    /// Cost per DALY averted < 1× GDP per capita.
    HighlyCostEffective,
    /// Cost per DALY averted between 1× and 3× GDP per capita.
    CostEffective,
    /// Cost per DALY averted > 3× GDP per capita.
    NotCostEffective,
}

/// Years of life lost (YLL): deaths × standard life expectancy at age of
/// death.
///
/// The mortality component of the DALY. When computing DALYs *averted*, pass
/// deaths prevented.
///
/// # Arguments
///
/// * `deaths` — count of deaths (or deaths prevented).
/// * `life_expectancy_at_death` — remaining standard life expectancy at the
///   age of death, in years.
///
/// # Returns
///
/// YLL in years.
///
/// # Examples
///
/// ```rust
/// use health_economics::disability_adjusted_life_year::years_of_life_lost;
///
/// // 10 premature deaths prevented × 20 years each = 200 YLL averted.
/// assert_eq!(years_of_life_lost(10.0, 20.0), 200.0);
/// ```
pub fn years_of_life_lost(deaths: f64, life_expectancy_at_death: f64) -> f64 {
    deaths * life_expectancy_at_death
}

/// Years lived with disability (YLD): prevalence × disability weight.
///
/// The morbidity component of the DALY. Prevalence is measured in
/// person-years lived in the condition; the disability weight runs from 0
/// (full health) to 1 (equivalent to death), per the Global Burden of
/// Disease published weights.
///
/// # Arguments
///
/// * `prevalence` — person-years lived in the condition (or prevented).
/// * `disability_weight` — GBD disability weight in [0, 1].
///
/// # Returns
///
/// YLD in healthy-year equivalents.
///
/// # Examples
///
/// ```rust
/// use health_economics::disability_adjusted_life_year::years_lived_with_disability;
///
/// // 200 person-years at disability weight 0.2 = 40 YLD averted.
/// assert_eq!(years_lived_with_disability(200.0, 0.2), 40.0);
/// ```
pub fn years_lived_with_disability(prevalence: f64, disability_weight: f64) -> f64 {
    prevalence * disability_weight
}

/// DALYs: YLL + YLD.
///
/// Equally, DALYs averted = YLL averted + YLD averted — pass the averted
/// components to value an intervention.
///
/// # Arguments
///
/// * `yll` — years of life lost (see [`years_of_life_lost`]).
/// * `yld` — years lived with disability (see [`years_lived_with_disability`]).
///
/// # Returns
///
/// Total DALYs (lost years of healthy life).
///
/// # Examples
///
/// ```rust
/// use health_economics::disability_adjusted_life_year::dalys;
///
/// // 200 YLL + 40 YLD = 240 DALYs averted per year.
/// assert_eq!(dalys(200.0, 40.0), 240.0);
/// ```
pub fn dalys(yll: f64, yld: f64) -> f64 {
    yll + yld
}

/// Cost per DALY averted: annual cost / DALYs averted per year.
///
/// The headline metric for global-health funders; judged against GDP-multiple
/// bands via [`who_choice_band`].
///
/// # Arguments
///
/// * `annual_cost` — cost of running the intervention per year (currency
///   units, e.g. $).
/// * `dalys_averted` — DALYs averted per year.
///
/// # Returns
///
/// `Some(cost per DALY averted)`, or `None` when `dalys_averted` is zero (no
/// DALYs averted — ratio undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::disability_adjusted_life_year::cost_per_daly_averted;
///
/// // $600,000 / 240 = $2,500 per DALY averted.
/// assert_eq!(cost_per_daly_averted(600_000.0, 240.0), Some(2_500.0));
/// assert_eq!(cost_per_daly_averted(600_000.0, 0.0), None);
/// ```
pub fn cost_per_daly_averted(annual_cost: f64, dalys_averted: f64) -> Option<f64> {
    if dalys_averted == 0.0 {
        None
    } else {
        Some(annual_cost / dalys_averted)
    }
}

/// Classify a cost per DALY averted against the WHO-CHOICE GDP-multiple bands.
///
/// Below 1× GDP per capita: highly cost-effective; 1–3×: cost-effective;
/// above 3×: not cost-effective. Both arguments must be in the same currency.
///
/// # Arguments
///
/// * `cost_per_daly_averted` — cost per DALY averted (see
///   [`cost_per_daly_averted`]).
/// * `gdp_per_capita` — GDP per capita of the country in question.
///
/// # Returns
///
/// The [`WhoChoiceBand`] the intervention falls into.
///
/// # Examples
///
/// ```rust
/// use health_economics::disability_adjusted_life_year::{
///     who_choice_band, WhoChoiceBand,
/// };
///
/// // $2,500 per DALY averted at $8,000 GDP per capita: under 1× GDP.
/// assert_eq!(who_choice_band(2_500.0, 8_000.0), WhoChoiceBand::HighlyCostEffective);
/// assert_eq!(who_choice_band(10_000.0, 8_000.0), WhoChoiceBand::CostEffective);
/// assert_eq!(who_choice_band(30_000.0, 8_000.0), WhoChoiceBand::NotCostEffective);
/// ```
pub fn who_choice_band(cost_per_daly_averted: f64, gdp_per_capita: f64) -> WhoChoiceBand {
    if cost_per_daly_averted < gdp_per_capita {
        // Below 1× GDP per capita.
        WhoChoiceBand::HighlyCostEffective
    } else if cost_per_daly_averted <= 3.0 * gdp_per_capita {
        // Between 1× and 3× GDP per capita.
        WhoChoiceBand::CostEffective
    } else {
        WhoChoiceBand::NotCostEffective
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "YLL averted = 10 × 20 = 200".
    #[test]
    fn yll_averted_is_200() {
        // 10 premature deaths prevented × 20 years each = 200
        assert!((years_of_life_lost(10.0, 20.0) - 200.0).abs() < 1e-9);
    }

    // Worked example: "YLD averted = 200 × 0.2 = 40".
    #[test]
    fn yld_averted_is_40() {
        // 200 person-years at disability weight 0.2 = 40
        assert!((years_lived_with_disability(200.0, 0.2) - 40.0).abs() < 1e-9);
    }

    // Worked example: "DALYs averted = 240 per year".
    #[test]
    fn dalys_averted_is_240_per_year() {
        let yll = years_of_life_lost(10.0, 20.0);
        let yld = years_lived_with_disability(200.0, 0.2);
        assert!((dalys(yll, yld) - 240.0).abs() < 1e-9);
    }

    // Worked example: "the cost per DALY averted is 600,000 / 240 = $2,500".
    #[test]
    fn cost_per_daly_averted_is_2500_dollars() {
        // $600,000 / 240 = $2,500
        let dalys_averted = dalys(200.0, 40.0);
        let cpd = cost_per_daly_averted(600_000.0, dalys_averted).unwrap();
        assert!((cpd - 2_500.0).abs() < 1e-9);
    }

    // Worked example: "In a country with GDP per capita of $8,000, that is
    // well under the 1× GDP benchmark — 'highly cost-effective'".
    #[test]
    fn platform_is_highly_cost_effective_at_8000_gdp_per_capita() {
        // $2,500 per DALY averted, GDP per capita $8,000: under 1× GDP
        assert_eq!(
            who_choice_band(2_500.0, 8_000.0),
            WhoChoiceBand::HighlyCostEffective
        );
    }

    // Why it matters: the WHO-CHOICE bands split at 1× and 3× GDP per capita.
    #[test]
    fn who_choice_bands_split_at_1x_and_3x_gdp() {
        assert_eq!(who_choice_band(10_000.0, 8_000.0), WhoChoiceBand::CostEffective);
        assert_eq!(who_choice_band(30_000.0, 8_000.0), WhoChoiceBand::NotCostEffective);
    }

    // Edge case: no DALYs averted means the ratio is undefined.
    #[test]
    fn cost_per_daly_is_none_when_nothing_averted() {
        assert!(cost_per_daly_averted(600_000.0, 0.0).is_none());
    }
}
