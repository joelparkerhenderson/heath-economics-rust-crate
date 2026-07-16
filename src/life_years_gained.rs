//! # Life-Years Gained (LYG)
//!
//! Life-years gained is the additional survival attributable to an
//! intervention, with no quality adjustment: the area between the survival
//! curves with and without it.
//!
//! The equal-value life year gained (evLYG) is a modern variant that credits
//! all life extension at a fixed utility (the US ICER institute uses ~0.851,
//! the average US population utility), removing the QALY's discount on
//! extending the lives of people with lower utility.
//!
//! ## Formula
//!
//! ```text
//! LYG   = mean survival_new − mean survival_comparator
//!       = area between survival curves (restricted to the time horizon)
//!
//! QALY view of life extension:  extension × patient utility
//! evLYG view of life extension: extension × fixed utility (≈ 0.851)
//!
//! LYG            = life-years gained
//! patient utility = the treated patients' own health-state utility (0–1)
//! fixed utility   = a population-average utility applied to everyone
//! ```
//!
//! Both are discounted in economic models.
//!
//! ## Why it matters
//!
//! LYG is the rawest health outcome: how much longer do people live? It
//! matters when quality data is missing, when comparing against QALY-skeptical
//! audiences, and in oncology where survival curves are the primary trial
//! output. The evLYG exists for an ethical reason: QALYs value a year of
//! extended life by the patient's utility, so extending the life of someone
//! with a disability "counts less" — evLYG values every extended year at a
//! fixed utility (~0.851), removing that discrimination.
//!
//! ## Example
//!
//! A sepsis early-warning algorithm: earlier antibiotics prevent 12
//! deaths/year; those patients average 8 remaining life-years each at
//! utility 0.7.
//!
//! ```rust
//! use health_economics::life_years_gained::{
//!     evlyg_from_life_extension, life_years_gained_from_deaths_prevented,
//!     monetary_value, qalys_from_life_extension, EVLYG_FIXED_UTILITY,
//! };
//!
//! // LYG = 12 × 8 = 96 life-years/year.
//! let lyg = life_years_gained_from_deaths_prevented(12.0, 8.0);
//! assert!((lyg - 96.0).abs() < 1e-9);
//!
//! // QALYs = 96 × 0.7 = 67.2; evLYG = 96 × 0.851 = 81.7 (exact 81.696).
//! let qalys = qalys_from_life_extension(lyg, 0.7);
//! assert!((qalys - 67.2).abs() < 1e-9);
//! let evlyg = evlyg_from_life_extension(lyg, EVLYG_FIXED_UTILITY);
//! assert!((evlyg - 81.696).abs() < 1e-9);
//!
//! // At £20,000/QALY: QALY framing £1.34M/year, evLYG framing £1.63M/year.
//! assert!((monetary_value(qalys, 20_000.0) - 1_344_000.0).abs() < 1e-9);
//! assert!((monetary_value(evlyg, 20_000.0) - 1_633_920.0).abs() < 1e-6);
//! ```
//!
//! The gap is exactly the ethical judgment about whether a life-year at
//! utility 0.7 is worth 70% of a "full" one. Serious dossiers report both.
//!
//! ## Software engineering connection
//!
//! - Survival analysis is the shared toolkit: Kaplan-Meier curves for
//!   patients and for *services* (time-to-failure, time-to-churn) are the
//!   same math.
//! - "Service-years gained" from a reliability investment = area between the
//!   with/without survival curves of the system — a more honest framing than
//!   point MTTF claims.
//! - The evLYG carries a metric-design warning: any productivity metric that
//!   weights output by a "quality of team" factor will systematically
//!   undervalue improvements for constrained or struggling teams.
//! - Sometimes you want the equal-value variant on purpose.
//!
//! ## Pitfalls
//!
//! - **Median vs mean survival**: economic models need mean (area under
//!   curve); trials often headline median. They differ a lot in skewed
//!   distributions.
//! - **Extrapolation beyond trial follow-up** dominates modeled LYG in
//!   chronic disease — state the extrapolation model and test it in
//!   sensitivity analysis.
//! - **Claiming deaths prevented from observational before/after data**
//!   without adjusting for case mix and secular trends.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: life-years gained.
//!   <https://yhec.co.uk/glossary/life-years-gained/>
//! - ICER, "Cost-Effectiveness, the QALY, and the evLYG."
//!   <https://icer.org/our-approach/methods-process/cost-effectiveness-the-qaly-and-the-evlyg/>
//!
//! Topic doc: health-economics-metrics/topics/life-years-gained.md

/// Fixed utility the US ICER institute applies to every extended life-year in the evLYG.
///
/// 0.851 is the average US population utility; using it values a year of
/// extended life equally regardless of the patient's own health state.
pub const EVLYG_FIXED_UTILITY: f64 = 0.851;

/// Life-years gained when an intervention prevents deaths.
///
/// A modeling shortcut for mortality-reducing interventions: deaths
/// prevented × mean remaining life expectancy of those patients. Undiscounted;
/// economic models apply discounting downstream.
///
/// # Arguments
///
/// * `deaths_prevented` — deaths averted (e.g. per year).
/// * `remaining_life_years_each` — mean remaining life expectancy of those
///   patients, in years.
///
/// # Returns
///
/// Life-years gained (per the same period as `deaths_prevented`).
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::life_years_gained_from_deaths_prevented;
///
/// // Sepsis early warning: 12 deaths prevented × 8 remaining life-years
/// // = 96 life-years/year.
/// let lyg = life_years_gained_from_deaths_prevented(12.0, 8.0);
/// assert!((lyg - 96.0).abs() < 1e-9);
/// ```
pub fn life_years_gained_from_deaths_prevented(
    deaths_prevented: f64,
    remaining_life_years_each: f64,
) -> f64 {
    deaths_prevented * remaining_life_years_each
}

/// LYG as the difference in mean survival between arms.
///
/// Mean — not median — survival is what economic models need: mean survival
/// is the area under the survival curve, so this difference equals the area
/// between the curves over the same horizon.
///
/// # Arguments
///
/// * `mean_survival_new` — mean survival in the intervention arm, years.
/// * `mean_survival_comparator` — mean survival in the comparator arm, years.
///
/// # Returns
///
/// Life-years gained per patient (negative if the new arm is worse).
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::life_years_gained_from_mean_survival;
///
/// // Mean survival 5.0 vs 3.0 years → 2.0 life-years gained per patient.
/// let lyg = life_years_gained_from_mean_survival(5.0, 3.0);
/// assert!((lyg - 2.0).abs() < 1e-9);
/// ```
pub fn life_years_gained_from_mean_survival(
    mean_survival_new: f64,
    mean_survival_comparator: f64,
) -> f64 {
    mean_survival_new - mean_survival_comparator
}

/// LYG as the area between two survival curves, by the trapezoidal rule.
///
/// The curves are sampled at common time points (years) as survival
/// probabilities in [0, 1]; the result is restricted to the sampled horizon
/// (no extrapolation beyond the last time point). `times` must be increasing.
///
/// # Arguments
///
/// * `times` — increasing time points, in years.
/// * `survival_new` — survival probability of the intervention arm at each time.
/// * `survival_comparator` — survival probability of the comparator arm at each time.
///
/// # Returns
///
/// `Some(area in life-years per patient)`, or `None` when the three slices do
/// not share the same length of at least 2 (nothing to integrate, or
/// mismatched sampling).
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::area_between_survival_curves;
///
/// // New arm falls 1.0 → 0.0 over 10 years (mean survival 5.0);
/// // comparator falls 1.0 → 0.0 over 6 years (mean 3.0): area = 2.0 LYG.
/// let times = [0.0, 6.0, 10.0];
/// let s_new = [1.0, 0.4, 0.0];
/// let s_comp = [1.0, 0.0, 0.0];
/// let area = area_between_survival_curves(&times, &s_new, &s_comp).unwrap();
/// assert!((area - 2.0).abs() < 1e-9);
///
/// // Mismatched or too-short samples are rejected.
/// assert!(area_between_survival_curves(&[0.0], &[1.0], &[1.0]).is_none());
/// ```
pub fn area_between_survival_curves(
    times: &[f64],
    survival_new: &[f64],
    survival_comparator: &[f64],
) -> Option<f64> {
    let n = times.len();
    if n < 2 || survival_new.len() != n || survival_comparator.len() != n {
        return None;
    }
    let mut area = 0.0;
    // Trapezoidal rule on the *gap* curve g(t) = S_new(t) − S_comp(t):
    // each interval contributes dt × (g_left + g_right) / 2. Integrating the
    // gap directly is equivalent to (area under S_new) − (area under S_comp),
    // i.e. the difference in mean survival restricted to the horizon.
    // Negative gaps (comparator above new arm) subtract, as they should.
    for i in 1..n {
        let dt = times[i] - times[i - 1];
        let gap_left = survival_new[i - 1] - survival_comparator[i - 1];
        let gap_right = survival_new[i] - survival_comparator[i];
        area += dt * (gap_left + gap_right) / 2.0;
    }
    Some(area)
}

/// QALY view of life extension: extension × the patients' own utility.
///
/// This is where the QALY "discounts" extension for patients in worse health
/// states — the exact feature the evLYG removes.
///
/// # Arguments
///
/// * `life_years` — life-years gained.
/// * `patient_utility` — the treated patients' health-state utility (0–1).
///
/// # Returns
///
/// QALYs gained (`life_years × patient_utility`).
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::qalys_from_life_extension;
///
/// // 96 life-years at utility 0.7 → 67.2 QALYs.
/// let qalys = qalys_from_life_extension(96.0, 0.7);
/// assert!((qalys - 67.2).abs() < 1e-9);
/// ```
pub fn qalys_from_life_extension(life_years: f64, patient_utility: f64) -> f64 {
    life_years * patient_utility
}

/// evLYG view of life extension: extension × a fixed utility.
///
/// Values every extended year equally regardless of the patient's health
/// state; pass [`EVLYG_FIXED_UTILITY`] for the ICER-institute convention.
///
/// # Arguments
///
/// * `life_years` — life-years gained.
/// * `fixed_utility` — the fixed utility applied to every extended year
///   (e.g. [`EVLYG_FIXED_UTILITY`] = 0.851).
///
/// # Returns
///
/// Equal-value life-years gained (`life_years × fixed_utility`).
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::{
///     evlyg_from_life_extension, EVLYG_FIXED_UTILITY,
/// };
///
/// // 96 life-years × 0.851 = 81.696 ≈ 81.7 evLYG.
/// let evlyg = evlyg_from_life_extension(96.0, EVLYG_FIXED_UTILITY);
/// assert!((evlyg - 81.696).abs() < 1e-9);
/// ```
pub fn evlyg_from_life_extension(life_years: f64, fixed_utility: f64) -> f64 {
    life_years * fixed_utility
}

/// Monetary value of a health gain at a willingness-to-pay threshold.
///
/// Works for any health unit (QALYs, evLYG, raw life-years) as long as the
/// threshold is quoted per that unit.
///
/// # Arguments
///
/// * `health_units` — the health gain (e.g. QALYs or evLYG).
/// * `threshold_per_unit` — willingness-to-pay per unit (e.g. £20,000 per QALY).
///
/// # Returns
///
/// The monetary value (`health_units × threshold_per_unit`), in the
/// threshold's currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::life_years_gained::monetary_value;
///
/// // 67.2 QALYs at £20,000/QALY → £1,344,000 ≈ £1.34M/year.
/// let value = monetary_value(67.2, 20_000.0);
/// assert!((value - 1_344_000.0).abs() < 1e-9);
/// ```
pub fn monetary_value(health_units: f64, threshold_per_unit: f64) -> f64 {
    health_units * threshold_per_unit
}

#[cfg(test)]
mod tests {
    use super::*;

    /// LYG = 12 deaths prevented × 8 remaining life-years = 96 life-years/year.
    #[test]
    fn sepsis_algorithm_gains_96_life_years_per_year() {
        // Worked example: "LYG = 12 × 8 = 96 life-years/year".
        let got = life_years_gained_from_deaths_prevented(12.0, 8.0);
        assert!((got - 96.0).abs() < 1e-9);
    }

    /// QALYs = 96 × 0.7 = 67.2.
    #[test]
    fn qaly_view_is_67_point_2() {
        // Worked example: "QALYs = 96 × 0.7 = 67.2".
        let lyg = life_years_gained_from_deaths_prevented(12.0, 8.0);
        let got = qalys_from_life_extension(lyg, 0.7);
        assert!((got - 67.2).abs() < 1e-9);
    }

    /// evLYG = 96 × 0.851 = 81.7 (exact 81.696).
    #[test]
    fn evlyg_view_is_81_point_7() {
        // Worked example: "evLYG = 96 × 0.851 = 81.7".
        let lyg = life_years_gained_from_deaths_prevented(12.0, 8.0);
        let got = evlyg_from_life_extension(lyg, EVLYG_FIXED_UTILITY);
        assert!((got - 81.696).abs() < 1e-9);
        assert!((got - 81.7).abs() < 0.05);
    }

    /// At £20,000/QALY the QALY framing values the survival at £1.34M/year.
    #[test]
    fn qaly_framing_values_survival_at_1_34_million() {
        // Worked example: "At £20,000 per QALY, the QALY framing values the
        // survival at £1.34M/year".
        let qalys = qalys_from_life_extension(96.0, 0.7);
        let got = monetary_value(qalys, 20_000.0);
        assert!((got - 1_344_000.0).abs() < 1e-9);
        assert!((got - 1_340_000.0).abs() < 10_000.0);
    }

    /// The evLYG framing values it at £1.63M/year.
    #[test]
    fn evlyg_framing_values_survival_at_1_63_million() {
        // Worked example: "the evLYG framing at £1.63M".
        let evlyg = evlyg_from_life_extension(96.0, EVLYG_FIXED_UTILITY);
        let got = monetary_value(evlyg, 20_000.0);
        assert!((got - 1_633_920.0).abs() < 1e-6);
        assert!((got - 1_630_000.0).abs() < 10_000.0);
    }

    /// LYG as difference in mean survival matches the area between curves for
    /// a simple piecewise-linear case.
    #[test]
    fn area_between_curves_matches_mean_survival_difference() {
        // Doc's math: "LYG = mean survival_new − mean survival_comparator
        // = area between survival curves".
        // New arm: survival falls 1.0 → 0.0 over 10 years (mean survival 5.0).
        // Comparator: falls 1.0 → 0.0 over 6 years then stays 0 (mean 3.0).
        let times = [0.0, 6.0, 10.0];
        let s_new = [1.0, 0.4, 0.0];
        let s_comp = [1.0, 0.0, 0.0];
        let area = area_between_survival_curves(&times, &s_new, &s_comp).unwrap();
        let diff = life_years_gained_from_mean_survival(5.0, 3.0);
        assert!((area - 2.0).abs() < 1e-9);
        assert!((area - diff).abs() < 1e-9);
    }

    /// Mismatched or too-short curve samples are rejected.
    #[test]
    fn malformed_survival_curves_return_none() {
        // Edge case: the trapezoid needs ≥ 2 shared sample points per curve.
        assert!(area_between_survival_curves(&[0.0], &[1.0], &[1.0]).is_none());
        assert!(area_between_survival_curves(&[0.0, 1.0], &[1.0], &[1.0, 0.9]).is_none());
    }
}
