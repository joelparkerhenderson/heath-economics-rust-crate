//! # Discounting and Time Preference
//!
//! Discounting converts future costs and benefits into present values,
//! because a benefit today is worth more than the same benefit in five
//! years.
//!
//! ## Formula
//!
//! ```text
//! PV = FV / (1 + r)^t
//!
//! PV = present value
//! FV = future value in year t
//! r  = discount rate (NICE/Green Book: 0.035)
//! t  = years from now
//!
//! Annuity (constant annual benefit B over n years, first payment in year 1):
//! PV = B × [1 − (1 + r)^(−n)] / r
//! ```
//!
//! ## Why it matters
//!
//! Every health-economics appraisal and every serious public-sector business
//! case discounts multi-year streams. The UK's HM Treasury Green Book
//! mandates a 3.5% annual social time preference rate; NICE's reference case
//! discounts both costs and health effects at 3.5% per year (with a 1.5%
//! non-reference-case rate for near-cure therapies with benefits over 30+
//! years). If your software business case claims "£5 million savings over 10
//! years", a finance reviewer will immediately ask for the discounted figure.
//!
//! ## Example
//!
//! The topic doc's worked example: software saves an NHS trust £100,000/year
//! for 5 years, starting one year after go-live. Undiscounted total:
//! £500,000; discounted at 3.5% the total PV is ≈ £451,505 — roughly 10%
//! less than the naive sum. A one-year delivery slip drops the PV to about
//! £436,000.
//!
//! ```rust
//! use health_economics::discounting_and_time_preference::{
//!     NICE_REFERENCE_RATE, present_value, annuity_present_value, delayed_present_value,
//! };
//!
//! // Year 1: 100,000 / 1.035^1 = £96,618.
//! let year1 = present_value(100_000.0, NICE_REFERENCE_RATE, 1.0);
//! assert!((year1 - 96_618.0).abs() < 0.5);
//!
//! // Total PV of 5 years ≈ £451,505.
//! let pv = annuity_present_value(100_000.0, NICE_REFERENCE_RATE, 5.0);
//! assert!((pv - 451_505.0).abs() < 1.0);
//!
//! // Undiscounted total: £500,000 (the r = 0 limit).
//! assert_eq!(annuity_present_value(100_000.0, 0.0, 5.0), 500_000.0);
//!
//! // A one-year delivery slip drops the PV to about £436,000 —
//! // the discounting view of cost of delay.
//! let slipped = delayed_present_value(pv, NICE_REFERENCE_RATE, 1.0);
//! assert!((slipped - 436_000.0).abs() < 500.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Tech-debt paydown and platform migrations** promise benefit streams
//!   years out; discount them before comparing against work that pays back
//!   this quarter.
//! - **Front-loaded costs, back-loaded benefits** is the standard shape of a
//!   migration. Discounting penalizes that shape, correctly: it prices the
//!   risk-free time value of committing capacity now for value later.
//! - **"Savings in year 5" claims** deserve skepticism twice over — they are
//!   both heavily discounted and highly uncertain.
//!
//! ## Pitfalls
//!
//! - **Discounting costs but not benefits** (or vice versa) — the reference
//!   case discounts both, at the same rate.
//! - **Using a commercial rate (8–12%) in a public-sector case**, or 3.5% in
//!   a venture-backed one. Match the rate to the decision-maker.
//! - **Confusing discounting with inflation.** Discounting applies to *real*
//!   (inflation-adjusted) values; don't do both implicitly.
//!
//! ## Sources
//!
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//! - HM Treasury Green Book, discounting supplementary guidance.
//!   <https://www.gov.uk/government/publications/green-book-supplementary-guidance-discounting>
//!
//! Topic doc: health-economics-metrics/topics/discounting-and-time-preference.md

/// The NICE reference-case / HM Treasury Green Book discount rate: 3.5% per
/// year, applied to both costs and health effects.
pub const NICE_REFERENCE_RATE: f64 = 0.035;

/// Present value of a future amount: PV = FV / (1 + r)^t.
///
/// Applies to *real* (inflation-adjusted) values; the rate is an annual
/// social time preference rate, not an inflation adjustment.
///
/// # Arguments
///
/// * `future_value` — the amount received in year `years` (£, real terms).
/// * `rate` — annual discount rate as a fraction (e.g. 0.035).
/// * `years` — years from now (t); may be fractional.
///
/// # Returns
///
/// The present value (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::discounting_and_time_preference::{
///     present_value, NICE_REFERENCE_RATE,
/// };
///
/// // Year 1: 100,000 / 1.035^1 = £96,618.
/// assert!((present_value(100_000.0, NICE_REFERENCE_RATE, 1.0) - 96_618.0).abs() < 0.5);
/// // Year 5: 100,000 / 1.035^5 = £84,197.
/// assert!((present_value(100_000.0, NICE_REFERENCE_RATE, 5.0) - 84_197.0).abs() < 0.5);
/// ```
pub fn present_value(future_value: f64, rate: f64, years: f64) -> f64 {
    future_value / (1.0 + rate).powf(years)
}

/// Present value of a constant annual benefit over `years` years (an
/// annuity), first payment one year from now.
///
/// Computes B × [1 − (1 + r)^(−n)] / r, which equals the sum of
/// [`present_value`] over years 1..=n. At r = 0 the closed form is 0/0, so
/// the formula's limit B × n is returned instead.
///
/// # Arguments
///
/// * `annual_benefit` — constant benefit B per year (£).
/// * `rate` — annual discount rate as a fraction; 0.0 returns the
///   undiscounted total B × n.
/// * `years` — number of annual payments n.
///
/// # Returns
///
/// The present value of the whole stream (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::discounting_and_time_preference::{
///     annuity_present_value, NICE_REFERENCE_RATE,
/// };
///
/// // £100,000/year for 5 years at 3.5%: total PV ≈ £451,505
/// // (vs the undiscounted £500,000).
/// let pv = annuity_present_value(100_000.0, NICE_REFERENCE_RATE, 5.0);
/// assert!((pv - 451_505.0).abs() < 1.0);
/// assert_eq!(annuity_present_value(100_000.0, 0.0, 5.0), 500_000.0);
/// ```
pub fn annuity_present_value(annual_benefit: f64, rate: f64, years: f64) -> f64 {
    if rate == 0.0 {
        // r → 0 limit of the annuity formula: just B × n.
        annual_benefit * years
    } else {
        // B × [1 − (1 + r)^(−n)] / r
        annual_benefit * (1.0 - (1.0 + rate).powf(-years)) / rate
    }
}

/// Present value of a benefit stream delayed by `delay_years`.
///
/// When delivery slips, every term of the stream shifts later by the same
/// amount, so the whole PV is divided by (1 + r)^delay — the discounting
/// view of cost of delay.
///
/// # Arguments
///
/// * `undelayed_pv` — present value of the stream without the slip (£).
/// * `rate` — annual discount rate as a fraction.
/// * `delay_years` — length of the slip, in years.
///
/// # Returns
///
/// The reduced present value after the delay (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::discounting_and_time_preference::{
///     annuity_present_value, delayed_present_value, NICE_REFERENCE_RATE,
/// };
///
/// // A one-year slip drops the £451,505 PV to about £436,000.
/// let pv = annuity_present_value(100_000.0, NICE_REFERENCE_RATE, 5.0);
/// let slipped = delayed_present_value(pv, NICE_REFERENCE_RATE, 1.0);
/// assert!((slipped - 436_000.0).abs() < 500.0);
/// ```
pub fn delayed_present_value(undelayed_pv: f64, rate: f64, delay_years: f64) -> f64 {
    // Shifting every term later by `delay` discounts the whole PV once more.
    undelayed_pv / (1.0 + rate).powf(delay_years)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "Year 1: £96,618 ... Year 5: £84,197" — the year-by-year
    // discounted values of £100,000 at 3.5%.
    #[test]
    fn each_year_of_100k_discounts_as_in_the_worked_example() {
        let expected = [96_618.0, 93_351.0, 90_194.0, 87_144.0, 84_197.0];
        for (i, &exp) in expected.iter().enumerate() {
            let pv = present_value(100_000.0, NICE_REFERENCE_RATE, (i + 1) as f64);
            assert!((pv - exp).abs() < 0.5, "year {}: got {}", i + 1, pv);
        }
    }

    // Worked example: "Total PV ≈ £451,505".
    #[test]
    fn total_pv_of_five_years_is_about_451_505() {
        let total: f64 = (1..=5)
            .map(|t| present_value(100_000.0, NICE_REFERENCE_RATE, t as f64))
            .sum();
        assert!((total - 451_505.0).abs() < 1.0);
    }

    // The math section: the annuity closed form equals the year-by-year sum.
    #[test]
    fn annuity_formula_matches_the_year_by_year_sum() {
        let total: f64 = (1..=5)
            .map(|t| present_value(100_000.0, NICE_REFERENCE_RATE, t as f64))
            .sum();
        let annuity = annuity_present_value(100_000.0, NICE_REFERENCE_RATE, 5.0);
        assert!((annuity - total).abs() < 1e-6);
    }

    // Worked example: "Undiscounted total: £500,000."
    #[test]
    fn undiscounted_total_is_500k() {
        assert!((annuity_present_value(100_000.0, 0.0, 5.0) - 500_000.0).abs() < 1e-9);
    }

    // Worked example: "delivery slips by one year ... the PV falls to about
    // £436,000 — the discounting view of cost of delay".
    #[test]
    fn one_year_delivery_slip_drops_pv_to_about_436k() {
        let pv = annuity_present_value(100_000.0, NICE_REFERENCE_RATE, 5.0);
        let slipped = delayed_present_value(pv, NICE_REFERENCE_RATE, 1.0);
        assert!((slipped - 436_000.0).abs() < 500.0);
    }
}
