//! # AI Regulatory Evaluation
//!
//! The regulatory frameworks that govern AI in health care — FDA's Software
//! as a Medical Device (SaMD) regime with Predetermined Change Control Plans
//! (PCCPs), and real-world evaluation programs like the NHS AI in Health and
//! Care Award — and what they cost and enable economically.
//!
//! Regulation determines both the evidence cost of market entry and the cost
//! of every subsequent model update — and for AI products, the second often
//! matters more. This module prices the traditional re-clearance route
//! against the PCCP route over a product life.
//!
//! ## Formula
//!
//! ```text
//! Cost per model update (traditional) = re-submission cost + review delay × CoD
//! Cost per model update (PCCP-scoped) = protocol-execution cost only
//!
//! Update economics over a product life:
//!   N updates × (submission cost + months of review × cost-of-delay per month)
//!   vs one-time PCCP authoring cost + N × protocol executions
//! ```
//!
//! Legend:
//! - `re-submission cost` — cost of preparing and filing a re-clearance
//!   (currency).
//! - `review delay` — months the update waits in regulatory review (months).
//! - `CoD` — cost of delay: the clinical/commercial benefit forgone per month
//!   the update is not shipped (currency/month).
//! - `N updates` — planned model updates over the product life (count).
//! - `PCCP authoring cost` — one-time cost of writing the change control plan
//!   (currency).
//! - `protocol execution` — cost of running the pre-authorized validation
//!   protocol for one update (currency).
//!
//! ## Why it matters
//!
//! FDA's traditional mode (lock the model; re-clear for changes) made
//! continuous improvement economically brutal. The PCCP guidance (finalized
//! December 2024) changed the economics: a manufacturer can pre-authorize
//! *specified* future model updates — a description of planned modifications,
//! a modification protocol (how each will be validated), and an impact
//! assessment — so sanctioned improvements ship without a new submission.
//! Over 1,000 AI-enabled devices have FDA authorization; the FDA now also
//! probes real-world performance monitoring (pre-specified metrics: baseline
//! FP/FN rates, calibration drift, domain-shift indicators). The PCCP is
//! regulatory recognition that deployment frequency has clinical value.
//!
//! ## Example
//!
//! A radiology-AI vendor plans quarterly model improvements over 3 years
//! (12 updates). Traditional route: 12 × (£80k submission + 4 months ×
//! £50k/month cost of delay) = £3.36M. PCCP route: £250k authoring + 12 ×
//! £30k protocol execution = £610k. Saving ≈ £2.75M — and patients receive
//! each improvement ~4 months sooner (48 update-months of benefit).
//!
//! ```rust
//! use health_economics::ai_regulatory_evaluation::{
//!     benefit_months_gained, pccp_lifetime_cost, pccp_saving, traditional_lifetime_cost,
//! };
//!
//! // Traditional: 12 × (£80k + 4 × £50k) = 12 × £280k = £3.36M.
//! let traditional = traditional_lifetime_cost(12.0, 80_000.0, 4.0, 50_000.0);
//! assert!((traditional - 3_360_000.0).abs() < 1e-9);
//!
//! // PCCP route: £250k authoring + 12 × £30k = £610k.
//! let pccp = pccp_lifetime_cost(250_000.0, 12.0, 30_000.0);
//! assert!((pccp - 610_000.0).abs() < 1e-9);
//!
//! // Saving ≈ £2.75M.
//! assert!((pccp_saving(traditional, pccp) - 2_750_000.0).abs() < 1e-9);
//!
//! // Patients receive improvements sooner: 12 × 4 = 48 update-months.
//! assert!((benefit_months_gained(12.0, 4.0) - 48.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineering the PCCP well is a software problem: pre-specified
//!   evaluation suites, versioned datasets, automated validation pipelines,
//!   drift monitoring.
//! - It is the regulated cousin of continuous deployment, where the "deploy
//!   gate" is a validated protocol instead of a code review.
//! - Teams with mature eval infrastructure get PCCPs cheaply; teams without
//!   discover the regulatory constraint is really an engineering-maturity
//!   constraint.
//! - For products entering the NHS, the parallel stack is DTAC (clinical
//!   safety, data protection, interoperability) plus NICE ESF evidence tiers
//!   — budget all of them as market-entry TCO.
//!
//! ## Pitfalls
//!
//! - **PCCP scope-creep dreams**: only *specified* modification types are
//!   pre-authorized; architecture changes or new intended uses still need
//!   full review.
//! - **Real-world drift unmonitored**: authorization at launch performance +
//!   silent population drift = a product performing outside its cleared
//!   envelope; monitoring is both a regulatory expectation and self-defense.
//! - **Confusing clearance with value**: FDA/UKCA clearance ≠ anyone will
//!   pay — that's the HTA hurdle, run separately.
//!
//! ## Sources
//!
//! - FDA, AI-enabled device software / SaMD.
//!   <https://www.fda.gov/medical-devices/software-medical-device-samd/artificial-intelligence-software-medical-device>
//! - PCCP implementation guidance analysis.
//!   <https://intuitionlabs.ai/articles/fda-pccp-implementation-guide-ai-ml-samd>
//! - NHS England, lessons from AI in Health and Care Award real-world
//!   evaluations.
//!   <https://www.england.nhs.uk/long-read/planning-and-implementing-real-world-ai-evaluations-lessons-from-the-ai-in-health-and-care-award/>
//!
//! Topic doc: health-economics-metrics/topics/ai-regulatory-evaluation.md

/// Cost of one model update under the traditional re-clearance route.
///
/// Re-submission cost plus review delay (months) times cost-of-delay per
/// month — the delayed-benefit term is what makes locked-model regulation
/// economically brutal for frequently improved products.
///
/// # Arguments
///
/// * `submission_cost` — cost of preparing and filing the re-clearance
///   (currency).
/// * `review_months` — months the update waits in review (months).
/// * `cost_of_delay_per_month` — benefit forgone per month of delay
///   (currency/month).
///
/// # Returns
///
/// Cost of one update (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_regulatory_evaluation::traditional_update_cost;
///
/// // Worked example: £80k submission + 4 months × £50k/month CoD = £280k.
/// let cost = traditional_update_cost(80_000.0, 4.0, 50_000.0);
/// assert!((cost - 280_000.0).abs() < 1e-9);
/// ```
pub fn traditional_update_cost(
    submission_cost: f64,
    review_months: f64,
    cost_of_delay_per_month: f64,
) -> f64 {
    // Cash outlay + opportunity cost of the benefit stuck in review.
    submission_cost + review_months * cost_of_delay_per_month
}

/// Total cost of N updates over a product life under the traditional route.
///
/// N × (submission cost + months of review × cost-of-delay per month).
///
/// # Arguments
///
/// * `n_updates` — planned model updates over the product life (count).
/// * `submission_cost` — cost per re-clearance filing (currency).
/// * `review_months` — review delay per update (months).
/// * `cost_of_delay_per_month` — benefit forgone per month of delay
///   (currency/month).
///
/// # Returns
///
/// Lifetime regulatory cost of the traditional route (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_regulatory_evaluation::traditional_lifetime_cost;
///
/// // Worked example: 12 quarterly updates over 3 years → 12 × £280k = £3.36M.
/// let cost = traditional_lifetime_cost(12.0, 80_000.0, 4.0, 50_000.0);
/// assert!((cost - 3_360_000.0).abs() < 1e-9);
/// ```
pub fn traditional_lifetime_cost(
    n_updates: f64,
    submission_cost: f64,
    review_months: f64,
    cost_of_delay_per_month: f64,
) -> f64 {
    n_updates * traditional_update_cost(submission_cost, review_months, cost_of_delay_per_month)
}

/// Total cost of N updates under a PCCP: one-time authoring plus N protocol executions.
///
/// The PCCP (Predetermined Change Control Plan) pre-authorizes specified
/// modification types, so each sanctioned update costs only its
/// protocol-execution run — no re-submission, no review delay.
///
/// # Arguments
///
/// * `pccp_authoring_cost` — one-time cost of writing the PCCP: planned
///   modifications, modification protocol, impact assessment (currency).
/// * `n_updates` — updates executed under the plan (count).
/// * `protocol_execution_cost` — cost of running the validation protocol for
///   one update (currency).
///
/// # Returns
///
/// Lifetime regulatory cost of the PCCP route (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_regulatory_evaluation::pccp_lifetime_cost;
///
/// // Worked example: £250k authoring + 12 × £30k executions = £610k.
/// let cost = pccp_lifetime_cost(250_000.0, 12.0, 30_000.0);
/// assert!((cost - 610_000.0).abs() < 1e-9);
/// ```
pub fn pccp_lifetime_cost(
    pccp_authoring_cost: f64,
    n_updates: f64,
    protocol_execution_cost: f64,
) -> f64 {
    // Fixed authoring cost amortizes across every update the plan covers.
    pccp_authoring_cost + n_updates * protocol_execution_cost
}

/// Saving from the PCCP route versus the traditional route over the product life.
///
/// Traditional lifetime cost minus PCCP lifetime cost; positive when the
/// PCCP is the cheaper route.
///
/// # Arguments
///
/// * `traditional_lifetime_cost` — lifetime cost of the re-clearance route
///   (currency; see [`traditional_lifetime_cost`]).
/// * `pccp_lifetime_cost` — lifetime cost of the PCCP route (currency; see
///   [`pccp_lifetime_cost`]).
///
/// # Returns
///
/// The saving (currency units); negative if the PCCP route costs more.
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_regulatory_evaluation::pccp_saving;
///
/// // Worked example: £3.36M − £610k ≈ £2.75M saving.
/// let saving = pccp_saving(3_360_000.0, 610_000.0);
/// assert!((saving - 2_750_000.0).abs() < 1e-9);
/// ```
pub fn pccp_saving(traditional_lifetime_cost: f64, pccp_lifetime_cost: f64) -> f64 {
    traditional_lifetime_cost - pccp_lifetime_cost
}

/// Months of clinical benefit gained because updates ship without review delay.
///
/// N updates × review months avoided per update. Multiply by the update's
/// monthly clinical benefit for a QALY line in its own right — the PCCP is
/// regulatory recognition that deployment frequency has clinical value.
///
/// # Arguments
///
/// * `n_updates` — updates shipped under the PCCP (count).
/// * `review_months_avoided_per_update` — review delay each update no longer
///   waits out (months).
///
/// # Returns
///
/// Total update-months of benefit gained (months).
///
/// # Examples
///
/// ```rust
/// use health_economics::ai_regulatory_evaluation::benefit_months_gained;
///
/// // Worked example: 12 updates × 4 months sooner each = 48 update-months.
/// let months = benefit_months_gained(12.0, 4.0);
/// assert!((months - 48.0).abs() < 1e-9);
/// ```
pub fn benefit_months_gained(n_updates: f64, review_months_avoided_per_update: f64) -> f64 {
    n_updates * review_months_avoided_per_update
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a radiology-AI vendor plans quarterly model improvements
    // over 3 years (12 updates).

    #[test]
    fn traditional_cost_per_update_is_280k() {
        // £80k submission + 4 months × £50k/month CoD = £280k.
        let got = traditional_update_cost(80_000.0, 4.0, 50_000.0);
        assert!((got - 280_000.0).abs() < 1e-9);
    }

    #[test]
    fn traditional_lifetime_cost_is_3_36_million() {
        // 12 × £280k = £3.36M.
        let got = traditional_lifetime_cost(12.0, 80_000.0, 4.0, 50_000.0);
        assert!((got - 3_360_000.0).abs() < 1e-9);
    }

    #[test]
    fn pccp_lifetime_cost_is_610k() {
        // £250k authoring + 12 × £30k protocol execution = £610k.
        let got = pccp_lifetime_cost(250_000.0, 12.0, 30_000.0);
        assert!((got - 610_000.0).abs() < 1e-9);
    }

    #[test]
    fn pccp_saving_is_about_2_75_million() {
        let traditional = traditional_lifetime_cost(12.0, 80_000.0, 4.0, 50_000.0);
        let pccp = pccp_lifetime_cost(250_000.0, 12.0, 30_000.0);
        let got = pccp_saving(traditional, pccp);
        // Exact: £2,750,000 — "Saving ≈ £2.75M".
        assert!((got - 2_750_000.0).abs() < 1e-9);
    }

    #[test]
    fn patients_receive_48_update_months_of_benefit_sooner() {
        // Doc: "patients receive each improvement ~4 months sooner:
        // 12 × 4 months" = 48 update-months.
        let got = benefit_months_gained(12.0, 4.0);
        assert!((got - 48.0).abs() < 1e-9);
    }
}
