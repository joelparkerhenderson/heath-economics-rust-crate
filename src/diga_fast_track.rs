//! # Germany's DiGA Fast-Track
//!
//! DiGA (Digitale Gesundheitsanwendungen) is Germany's statutory "apps on
//! prescription" pathway — the world's first national system where doctors
//! prescribe approved health apps and statutory insurance must reimburse
//! them. It is the leading live experiment in paying for digital therapeutics
//! at national scale.
//!
//! The pathway's core innovation: apps can list *provisionally* for 12 months
//! while still generating evidence — earning revenue during their pivotal
//! study — then either prove a "positive healthcare effect" (usually via an
//! RCT) and convert to permanent listing, or be delisted. Roughly half of
//! provisional entries fail to convert.
//!
//! ## Formula
//!
//! ```text
//! Revenue = prescriptions × activation rate × price per prescription period
//! Expected value = P(evidence succeeds) × steady-state revenue − evidence cost
//!
//! prescriptions   — scripts written per period
//! activation rate — fraction of prescriptions patients actually activate (~81% field average)
//! price           — reimbursed price per prescription period (median initial ~€500/quarter)
//! evidence cost   — pivotal RCT, typically €1M–3M, within the 12-month window
//! P(evidence succeeds) — honestly-assessed probability the pivotal study converts
//! ```
//!
//! ## Why it matters
//!
//! DiGA answered the question every digital health company asks — "who will
//! actually pay?" — with legislation (the DVG, 2019). BfArM must decide
//! within 3 months; the manufacturer sets the year-1 price freely, then
//! negotiates with the insurers' federation (performance-based pricing
//! elements arriving from 2026). Market reality check (through end-2024):
//! ~68 apps listed, >1M cumulative prescriptions, ~81% of prescriptions
//! activated, ~€234M cumulative insurer spend — a real market, but modest
//! against the hype, and adherence after activation remains the weak point.
//! With ~50% conversion failure, half the field spends the RCT money and
//! loses the listing.
//!
//! ## Example
//!
//! The topic doc's worked example: a depression-management app lists
//! provisionally at €450/quarter. Year 1: 20,000 prescriptions × 81%
//! activation × €450 ≈ €7.3M — enough to finance the €2M concurrent RCT.
//! Outcome A (evidence positive): permanent listing at a negotiated ~€380,
//! 60,000 scripts/yr ≈ €18.5M/yr. Outcome B: delisted at month 12.
//!
//! ```rust
//! use health_economics::diga_fast_track::{
//!     revenue, activated_prescriptions, expected_value,
//!     provisional_year_finances_evidence,
//! };
//!
//! // Year 1: 20,000 prescriptions × 81% activation × €450 ≈ €7.3M revenue.
//! let year_one = revenue(20_000.0, 0.81, 450.0);
//! assert!((year_one - 7_290_000.0).abs() < 1e-6);
//! assert!((activated_prescriptions(20_000.0, 0.81) - 16_200.0).abs() < 1e-9);
//!
//! // The provisional year finances the €2M RCT — the pathway's core innovation.
//! assert!(provisional_year_finances_evidence(year_one, 2_000_000.0));
//!
//! // Outcome A: negotiated price ~€380, 60,000 scripts/yr ≈ €18.5M/yr.
//! let steady_state = revenue(60_000.0, 0.81, 380.0);
//! assert!((steady_state - 18_468_000.0).abs() < 1e-6);
//!
//! // Expected value at the field-average ~50% conversion rate.
//! let ev = expected_value(0.5, steady_state, 2_000_000.0);
//! assert!((ev - 7_234_000.0).abs() < 1e-6);
//!
//! // Outcome B: evidence fails, revenue stops — EV is minus the RCT cost.
//! assert!((expected_value(0.0, steady_state, 2_000_000.0) - (-2_000_000.0)).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - DiGA's pattern — **provisional adoption with a pre-registered success
//!   metric and an automatic sunset** — is directly copyable for
//!   engineering-tool governance: ship the tool to production users for 12
//!   months, pre-register the metric (measured time saved, incident
//!   reduction), auto-expire unless the evidence lands.
//! - It solves the pilot paradox (tools that need scale to prove value never
//!   get scale) without granting permanent tenure to unproven tech.
//! - The 81%-activation/low-adherence data carries a product lesson:
//!   prescription (or executive mandate) gets installs; only product quality
//!   gets sustained use.
//!
//! ## Pitfalls
//!
//! - **Treating listing as the finish line** — prescriptions require
//!   prescriber trust; many listed DiGAs see negligible volume.
//! - **Underpowering the pivotal study** to save money during the revenue
//!   year — the false economy that explains much of the 50% failure rate.
//! - **Porting the model without the payer**: DiGA works because
//!   reimbursement is statutory; a copy without mandated payment is just a
//!   pilot program.
//!
//! ## Sources
//!
//! - Analysis of the DiGA market, npj Digital Medicine 2024.
//!   <https://www.nature.com/articles/s41746-024-01137-1>
//! - DiGA pricing trends, npj Digital Medicine 2025.
//!   <https://www.nature.com/articles/s41746-025-01879-6>
//! - BfArM, Digital Health Applications.
//!   <https://www.bfarm.de/EN/Medical-devices/Tasks/DiGA-and-DiPA/Digital-Health-Applications/_node.html>
//!
//! Topic doc: health-economics-metrics/topics/diga-fast-track.md

/// DiGA revenue for a period.
///
/// Only activated prescriptions are reimbursed, so revenue is
/// prescriptions × activation rate × price per prescription period.
///
/// # Arguments
///
/// * `prescriptions` — scripts written in the period.
/// * `activation_rate` — fraction of prescriptions activated by patients
///   (0–1; the field average is ~0.81).
/// * `price_per_prescription` — reimbursed price per prescription period (€).
///
/// # Returns
///
/// Revenue for the period (€).
///
/// # Examples
///
/// ```rust
/// use health_economics::diga_fast_track::revenue;
///
/// // Year 1: 20,000 prescriptions × 81% activation × €450 = €7,290,000 ≈ €7.3M.
/// assert!((revenue(20_000.0, 0.81, 450.0) - 7_290_000.0).abs() < 1e-6);
/// ```
pub fn revenue(prescriptions: f64, activation_rate: f64, price_per_prescription: f64) -> f64 {
    prescriptions * activation_rate * price_per_prescription
}

/// Prescriptions actually activated by patients.
///
/// Multiplies prescriptions by the activation rate; the field average is
/// ~81%, and adherence *after* activation remains the weak point.
///
/// # Arguments
///
/// * `prescriptions` — scripts written.
/// * `activation_rate` — fraction activated (0–1).
///
/// # Returns
///
/// Count of activated prescriptions.
///
/// # Examples
///
/// ```rust
/// use health_economics::diga_fast_track::activated_prescriptions;
///
/// // 20,000 prescriptions at 81% activation = 16,200 activated.
/// assert!((activated_prescriptions(20_000.0, 0.81) - 16_200.0).abs() < 1e-9);
/// ```
pub fn activated_prescriptions(prescriptions: f64, activation_rate: f64) -> f64 {
    prescriptions * activation_rate
}

/// Expected value of the DiGA bet.
///
/// P(evidence succeeds) × steady-state revenue − evidence (pivotal RCT)
/// cost. With ~50% conversion failure across the field,
/// `probability_evidence_succeeds` must be honestly assessed — half the
/// field spends the RCT money and loses the listing.
///
/// # Arguments
///
/// * `probability_evidence_succeeds` — probability (0–1) the pivotal study
///   demonstrates a positive healthcare effect.
/// * `steady_state_revenue` — annual revenue if permanently listed (€/yr).
/// * `evidence_cost` — pivotal RCT cost, typically €1M–3M (€).
///
/// # Returns
///
/// Expected value (€); negative when the evidence bet is not worth taking.
///
/// # Examples
///
/// ```rust
/// use health_economics::diga_fast_track::expected_value;
///
/// // P = 0.5 (field average), steady state €18.468M/yr, RCT €2M.
/// assert_eq!(expected_value(0.5, 18_468_000.0, 2_000_000.0), 7_234_000.0);
/// // Evidence fails (P = 0): EV is minus the RCT cost.
/// assert_eq!(expected_value(0.0, 18_468_000.0, 2_000_000.0), -2_000_000.0);
/// ```
pub fn expected_value(
    probability_evidence_succeeds: f64,
    steady_state_revenue: f64,
    evidence_cost: f64,
) -> f64 {
    probability_evidence_succeeds * steady_state_revenue - evidence_cost
}

/// Whether the provisional year finances the evidence generation.
///
/// The pathway's core innovation: revenue earned during the 12-month
/// provisional listing covers the pivotal RCT cost — in contrast to the
/// traditional sequence (evidence first, revenue years later), which starves
/// exactly the products DiGA wants to exist.
///
/// # Arguments
///
/// * `year_one_revenue` — provisional-year revenue (€).
/// * `evidence_cost` — pivotal RCT cost (€).
///
/// # Returns
///
/// `true` iff year-one revenue is at least the evidence cost.
///
/// # Examples
///
/// ```rust
/// use health_economics::diga_fast_track::{
///     provisional_year_finances_evidence, revenue,
/// };
///
/// // €7.29M year-one revenue comfortably covers the €2M RCT.
/// let year_one = revenue(20_000.0, 0.81, 450.0);
/// assert!(provisional_year_finances_evidence(year_one, 2_000_000.0));
/// assert!(!provisional_year_finances_evidence(1_500_000.0, 2_000_000.0));
/// ```
pub fn provisional_year_finances_evidence(year_one_revenue: f64, evidence_cost: f64) -> bool {
    year_one_revenue >= evidence_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "Year 1: 20,000 prescriptions × 81% activation × €450
    // ≈ €7.3M revenue".
    #[test]
    fn year_one_revenue_is_about_7_3_million_euros() {
        // 20,000 prescriptions × 81% activation × €450 = €7,290,000 ≈ €7.3M
        let rev = revenue(20_000.0, 0.81, 450.0);
        assert!((rev - 7_290_000.0).abs() < 1e-6);
        assert!((rev - 7_300_000.0).abs() < 50_000.0);
    }

    // Worked example: "Outcome A ... negotiated price ~€380, steady state
    // 60,000 scripts/yr ≈ €18.5M/yr".
    #[test]
    fn steady_state_revenue_is_about_18_5_million_euros_per_year() {
        // Outcome A: negotiated price ~€380, 60,000 scripts/yr ≈ €18.5M/yr
        let rev = revenue(60_000.0, 0.81, 380.0);
        assert!((rev - 18_468_000.0).abs() < 1e-6);
        assert!((rev - 18_500_000.0).abs() < 50_000.0);
    }

    // Worked example: "RCT cost: €2M, running concurrently" — "the
    // provisional year finances the evidence generation".
    #[test]
    fn provisional_year_covers_the_2m_rct() {
        let year_one = revenue(20_000.0, 0.81, 450.0);
        assert!(provisional_year_finances_evidence(year_one, 2_000_000.0));
    }

    // Market reality check: "~81% of prescriptions activated".
    #[test]
    fn activated_prescriptions_at_81_percent() {
        assert!((activated_prescriptions(20_000.0, 0.81) - 16_200.0).abs() < 1e-9);
    }

    // The math section: "Expected value = P(evidence succeeds) × steady-state
    // revenue − evidence cost", at the field-average ~50% conversion.
    #[test]
    fn expected_value_with_field_average_conversion() {
        // P = 0.5 (half the field fails), steady state €18.468M, RCT €2M
        let ev = expected_value(0.5, revenue(60_000.0, 0.81, 380.0), 2_000_000.0);
        assert!((ev - 7_234_000.0).abs() < 1e-6);
    }

    // Worked example: "Outcome B (evidence fails): delisted at month 12;
    // revenue stops."
    #[test]
    fn outcome_b_evidence_fails_revenue_stops() {
        // Delisted at month 12: P = 0, EV is minus the RCT cost.
        let ev = expected_value(0.0, revenue(60_000.0, 0.81, 380.0), 2_000_000.0);
        assert!((ev - (-2_000_000.0)).abs() < 1e-9);
    }
}
