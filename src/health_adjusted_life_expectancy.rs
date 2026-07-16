//! # Health-Adjusted Life Expectancy (HALE)
//!
//! HALE is a population-level summary: the number of years a person can
//! expect to live *in full health*, discounting years spent in illness or
//! disability. Global HALE at birth was about 61.9 years against a life
//! expectancy of 73.3 (WHO, 2019 data) — humanity lives its last decade, on
//! average, in less-than-full health.
//!
//! The standard computation is the **Sullivan method**: take a life table,
//! weight the person-years lived in each age interval by the proportion of
//! the population in full health at that age, and divide by survivors at the
//! starting age.
//!
//! ## Formula
//!
//! ```text
//! HALE_age_x = Σ (life-table person-years at each age ≥ x × proportion in full health)
//!              / survivors at age x
//!
//! "proportion in full health" = 1 − Σ (prevalence_condition × disability weight)
//!
//! person-years      = L_a, life-table person-years lived in age interval a
//! prevalence        = fraction of the population with the condition (0–1)
//! disability weight = severity of the condition (0 = full health, 1 = death-like),
//!                     from Global Burden of Disease data
//! survivors at x    = l_x, life-table survivors at the starting age
//! HALE gap          = life expectancy − HALE (expected years in ill health)
//! ```
//!
//! ## Why it matters
//!
//! HALE is the north-star metric of national and global health policy — the
//! numerator of "healthy aging" targets — and the gap it exposes (life
//! expectancy minus HALE, globally 73.3 − 61.9 = 11.4 years) is the burden
//! that prevention, early intervention, and chronic-disease management aim
//! to close. Digital health strategies at ministry level are justified in
//! HALE terms; a portfolio of apps, screening services, and monitoring
//! programs ultimately rolls up here.
//!
//! ## Example
//!
//! A national digital hypertension program: 500,000 enrolled; strokes
//! averted over the cohort's lifetime save 15,000 disability-weighted
//! healthy years (YLD at weight 0.32 plus YLL from fatal strokes).
//!
//! ```
//! use health_economics::health_adjusted_life_expectancy::{
//!     hale_contribution_per_person, years_to_days, hale_gap,
//! };
//!
//! // 15,000 healthy years / 500,000 people ≈ 0.03 years of HALE per person.
//! let per_person = hale_contribution_per_person(15_000.0, 500_000.0).unwrap();
//! assert!((per_person - 0.03).abs() < 1e-9);
//!
//! // ≈ 11 days per enrolled person — ministries buy millions of tiny gains.
//! let days = years_to_days(per_person);
//! assert!((days - 11.0).abs() < 0.5);
//!
//! // WHO 2019: global life expectancy 73.3 vs HALE 61.9 — an 11.4-year gap.
//! assert!((hale_gap(73.3, 61.9) - 11.4).abs() < 1e-9);
//! ```
//!
//! Eleven days sounds small — but at population scale it's how national
//! metrics actually move. The arithmetic also shows why **reach dominates**:
//! an intervention twice as effective with a tenth the enrollment moves HALE
//! five times less.
//!
//! ## Software engineering connection
//!
//! - HALE is a fleet-health metric pattern: **expected service life ×
//!   proportion of that life spent healthy**.
//! - A platform team can compute "healthy service life expectancy" across
//!   its estate — years a service is expected to run, discounted by time in
//!   degraded, deprecated, or incident states (weights from SLO shortfall).
//! - It reframes reliability from point availability to lifetime health.
//! - It directs remediation at the systems dragging the estate's HALE down.
//!
//! ## Pitfalls
//!
//! - **HALE moves slowly and multi-causally** — no single intervention
//!   "moves HALE" measurably; claim the modeled contribution, not the
//!   national statistic.
//! - **Prevalence data lags** years behind; recent gains won't show in
//!   official HALE.
//! - **Comparing HALE across countries** with different health-state
//!   measurement is treacherous; use it longitudinally within one system.
//!
//! ## Sources
//!
//! - WHO indicator registry: HALE.
//!   <https://www.who.int/data/gho/indicator-metadata-registry/imr-details/66>
//! - Global Burden of Disease study (IHME).
//!   <https://www.healthdata.org/research-analysis/gbd>
//!
//! Topic doc: health-economics-metrics/topics/health-adjusted-life-expectancy.md

/// A condition's contribution to population health loss at some age.
///
/// Used to compute the proportion of the population in full health:
/// each condition subtracts `prevalence × disability_weight`.
#[derive(Debug, Clone, Copy)]
pub struct ConditionBurden {
    /// Prevalence of the condition: fraction of the population affected
    /// (0–1).
    pub prevalence: f64,
    /// Disability weight for the condition: 0 = full health, 1 = death-like
    /// (from Global Burden of Disease data; e.g. ~0.32 for post-stroke
    /// disability in the worked example).
    pub disability_weight: f64,
}

/// Proportion of the population in full health at an age.
///
/// Computes `1 − Σ (prevalence × disability weight)` over the given
/// conditions. Assumes independent, additive burdens; with high prevalences
/// or many comorbid conditions the sum can exceed 1 and the result go
/// negative — real analyses adjust for comorbidity.
///
/// # Arguments
///
/// * `conditions` — the prevalent conditions with their disability weights.
///
/// # Returns
///
/// Fraction of full health (≤ 1; 1.0 for an empty slice).
///
/// # Examples
///
/// ```
/// use health_economics::health_adjusted_life_expectancy::{
///     ConditionBurden, proportion_in_full_health,
/// };
///
/// // 10% prevalence at stroke weight 0.32 plus 20% at weight 0.10:
/// // 1 − 0.032 − 0.02 = 0.948.
/// let conditions = [
///     ConditionBurden { prevalence: 0.10, disability_weight: 0.32 },
///     ConditionBurden { prevalence: 0.20, disability_weight: 0.10 },
/// ];
/// let p = proportion_in_full_health(&conditions);
/// assert!((p - 0.948).abs() < 1e-9);
/// ```
pub fn proportion_in_full_health(conditions: &[ConditionBurden]) -> f64 {
    // 1 − Σ (prevalence × disability weight): each condition removes its
    // prevalence-weighted severity from the full-health share.
    1.0 - conditions
        .iter()
        .map(|c| c.prevalence * c.disability_weight)
        .sum::<f64>()
}

/// Sullivan-method HALE at age x.
///
/// Health-weighted life-table person-years at each age ≥ x, divided by
/// survivors at age x. `person_years` and `full_health_proportion` are
/// parallel slices over the remaining age intervals: `person_years[i]` is
/// the life-table L value for interval i, and `full_health_proportion[i]`
/// the proportion of that interval's population in full health. With every
/// proportion at 1.0, Sullivan HALE reduces to ordinary life expectancy.
///
/// # Arguments
///
/// * `person_years` — life-table person-years lived in each remaining age
///   interval (L_a).
/// * `full_health_proportion` — proportion in full health per interval
///   (0–1), same length as `person_years`.
/// * `survivors_at_x` — life-table survivors at the starting age (l_x).
///
/// # Returns
///
/// HALE in years, or `None` if `survivors_at_x` is zero or the slices
/// differ in length.
///
/// # Examples
///
/// ```
/// use health_economics::health_adjusted_life_expectancy::sullivan_hale;
///
/// // Toy life table: 950 person-years at 90% full health + 800 at 70%,
/// // 100 survivors at age x → (855 + 560) / 100 = 14.15 healthy years.
/// let hale = sullivan_hale(&[950.0, 800.0], &[0.9, 0.7], 100.0).unwrap();
/// assert!((hale - 14.15).abs() < 1e-9);
///
/// // Everyone in full health: HALE equals plain life expectancy (17.5).
/// let le = sullivan_hale(&[950.0, 800.0], &[1.0, 1.0], 100.0).unwrap();
/// assert!((le - 17.5).abs() < 1e-9);
/// ```
pub fn sullivan_hale(
    person_years: &[f64],
    full_health_proportion: &[f64],
    survivors_at_x: f64,
) -> Option<f64> {
    if survivors_at_x == 0.0 || person_years.len() != full_health_proportion.len() {
        return None;
    }
    // Sullivan method: weight each age interval's person-years (L_a) by the
    // proportion of that interval lived in full health, then sum — turning
    // total person-years into *healthy* person-years.
    let healthy_person_years: f64 = person_years
        .iter()
        .zip(full_health_proportion)
        .map(|(py, prop)| py * prop)
        .sum();
    // Divide by survivors at age x (l_x): per-person expectation of healthy
    // years, exactly parallel to life expectancy e_x = Σ L_a / l_x.
    Some(healthy_person_years / survivors_at_x)
}

/// The HALE gap: life expectancy minus HALE.
///
/// The expected years lived in less-than-full health — the burden that
/// prevention and early intervention aim to close.
///
/// # Arguments
///
/// * `life_expectancy` — life expectancy at the same age (years).
/// * `hale` — HALE at the same age (years).
///
/// # Returns
///
/// The gap in years (positive whenever any ill health exists).
///
/// # Examples
///
/// ```
/// use health_economics::health_adjusted_life_expectancy::hale_gap;
///
/// // WHO 2019 global figures: 73.3 − 61.9 = 11.4 years.
/// assert!((hale_gap(73.3, 61.9) - 11.4).abs() < 1e-9);
/// ```
pub fn hale_gap(life_expectancy: f64, hale: f64) -> f64 {
    life_expectancy - hale
}

/// Per-person HALE contribution of an intervention.
///
/// Disability-weighted healthy years saved across a cohort, divided by
/// cohort size. Claim this modeled contribution — not a movement in the
/// national HALE statistic.
///
/// # Arguments
///
/// * `healthy_years_saved` — disability-weighted healthy years saved by the
///   intervention across the cohort (YLD averted + YLL averted).
/// * `cohort_size` — people enrolled (count).
///
/// # Returns
///
/// Years of HALE per enrolled person, or `None` if `cohort_size` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_adjusted_life_expectancy::hale_contribution_per_person;
///
/// // Worked example: 15,000 healthy years / 500,000 enrolled ≈ 0.03 years.
/// let c = hale_contribution_per_person(15_000.0, 500_000.0).unwrap();
/// assert!((c - 0.03).abs() < 1e-9);
/// ```
pub fn hale_contribution_per_person(healthy_years_saved: f64, cohort_size: f64) -> Option<f64> {
    if cohort_size == 0.0 {
        None
    } else {
        Some(healthy_years_saved / cohort_size)
    }
}

/// Convert a HALE contribution in years to days (365.25-day year).
///
/// # Arguments
///
/// * `years` — duration in years.
///
/// # Returns
///
/// The same duration in days.
///
/// # Examples
///
/// ```
/// use health_economics::health_adjusted_life_expectancy::years_to_days;
///
/// // Worked example: 0.03 years ≈ 11 days of HALE per enrolled person.
/// assert!((years_to_days(0.03) - 11.0).abs() < 0.5);
/// ```
pub fn years_to_days(years: f64) -> f64 {
    years * 365.25
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "HALE contribution ≈ 15,000 healthy years / 500,000 people
    // ≈ 0.03 years".
    #[test]
    fn worked_example_hale_contribution_is_0_03_years() {
        let c = hale_contribution_per_person(15_000.0, 500_000.0).unwrap();
        assert!((c - 0.03).abs() < 1e-9, "got {c}");
    }

    // Doc line: "≈ 11 days of HALE per enrolled person".
    #[test]
    fn worked_example_hale_contribution_is_about_11_days() {
        let days = years_to_days(hale_contribution_per_person(15_000.0, 500_000.0).unwrap());
        assert!((days - 11.0).abs() < 0.5, "got {days}");
    }

    // Doc line: "an intervention twice as effective with a tenth the
    // enrollment moves HALE five times less".
    #[test]
    fn worked_example_reach_dominates() {
        // Population-level healthy years: effect per person × enrolled.
        let base = 15_000.0; // healthy years from 500,000 enrolled
        let smaller = 2.0 * (15_000.0 / 500_000.0) * 50_000.0; // 2× effect, 1/10 reach
        let ratio: f64 = base / smaller;
        assert!((ratio - 5.0).abs() < 1e-9, "got {ratio}");
    }

    // Doc line: "Global HALE at birth was about 61.9 years against a life
    // expectancy of 73.3 (WHO, 2019)" — an 11.4-year gap.
    #[test]
    fn who_global_hale_gap_is_11_4_years() {
        let gap = hale_gap(73.3, 61.9);
        assert!((gap - 11.4).abs() < 1e-9, "got {gap}");
    }

    // Doc formula: "proportion in full health = 1 − Σ (prevalence ×
    // disability weight)", using the worked example's stroke weight 0.32.
    #[test]
    fn proportion_in_full_health_subtracts_weighted_prevalence() {
        let conditions = [
            ConditionBurden { prevalence: 0.10, disability_weight: 0.32 },
            ConditionBurden { prevalence: 0.20, disability_weight: 0.10 },
        ];
        let p = proportion_in_full_health(&conditions);
        assert!((p - (1.0 - 0.032 - 0.02)).abs() < 1e-9, "got {p}");
    }

    // Doc formula: Sullivan HALE = health-weighted person-years / survivors;
    // with all proportions at 1.0 it reduces to life expectancy.
    #[test]
    fn sullivan_hale_weights_person_years_by_health() {
        // Two remaining age intervals from age x with 100 survivors:
        // 950 person-years at 90% full health + 800 at 70%.
        let hale = sullivan_hale(&[950.0, 800.0], &[0.9, 0.7], 100.0).unwrap();
        let expected = (950.0 * 0.9 + 800.0 * 0.7) / 100.0;
        assert!((hale - expected).abs() < 1e-9, "got {hale}");
        // With everyone in full health, Sullivan HALE equals life expectancy.
        let le = sullivan_hale(&[950.0, 800.0], &[1.0, 1.0], 100.0).unwrap();
        assert!((le - 17.5).abs() < 1e-9, "got {le}");
        assert!(hale < le);
    }

    // Guard behavior: zero survivors or mismatched slices yield None.
    #[test]
    fn sullivan_hale_guards_invalid_input() {
        assert!(sullivan_hale(&[1.0], &[1.0], 0.0).is_none());
        assert!(sullivan_hale(&[1.0, 2.0], &[1.0], 10.0).is_none());
        assert!(hale_contribution_per_person(1.0, 0.0).is_none());
    }
}
