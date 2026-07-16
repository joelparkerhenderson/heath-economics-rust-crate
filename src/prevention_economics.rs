//! # Prevention Economics
//!
//! The economics of intervening before disease occurs or progresses. The
//! headline finding is counterintuitive: **most prevention does not save
//! money** — it buys health at a good price. Cohen, Neumann and Weinstein's
//! landmark NEJM analysis found fewer than 20% of preventive interventions
//! are net cost-saving; the rest are cost-effective at best.
//!
//! The prevention paradox drives the arithmetic: intervention cost multiplies
//! over the *whole* population, while benefits accrue only to the
//! counterfactual few who would have progressed.
//!
//! ## Formula
//!
//! ```text
//! Net cost of prevention =
//!     intervention cost × everyone treated
//!   − downstream costs avoided × the few who would have progressed
//!   (both discounted)
//!
//! Cost-saving requires:    intervention cost < P(progression) × avoided cost × discount factor
//! Cost-effective requires: net cost / QALYs gained < threshold
//! ```
//!
//! Legend:
//! - `intervention cost` — per-person cost of the preventive program.
//! - `P(progression)` — probability an individual would have progressed to
//!   the costly event without the program.
//! - `avoided cost` — downstream £ cost per case prevented.
//! - `discount factor` — present-value factor for benefits that arrive years
//!   later.
//! - `threshold` — willingness-to-pay per QALY (NICE: £20,000–£30,000).
//!
//! ## Why it matters
//!
//! "Prevention saves money" is the most repeated false claim in health
//! policy, and business cases built on it get demolished by health
//! economists. The honest structure: prevention costs money now (screening
//! whole populations, treating risk factors in people who would never have
//! gotten sick) and returns health later — usually at a *good* cost per QALY,
//! occasionally at a saving, sometimes at a terrible price. Knowing which
//! regime you're in is the analysis. The distinction matters commercially: a
//! prevention product sold as "saves the NHS money" invites an audit it will
//! fail; sold as "buys QALYs at £4,000" it can win on the same facts.
//!
//! ## Example
//!
//! A hypertension-management app offered to 100,000 at-risk adults at
//! £25/person/year. Over 10 years it prevents 400 strokes (each costing
//! £45,000 discounted, and 3 QALYs lost):
//!
//! ```rust
//! use health_economics::prevention_economics::{
//!     cost_per_qaly, downstream_offsets, is_cost_effective, is_cost_saving,
//!     net_cost, program_cost, qalys_gained,
//! };
//!
//! // Cost: 100,000 × £25 × 10 yrs (discounted ≈ ×8.3) ≈ £20.8M.
//! let cost = program_cost(100_000.0, 25.0, 8.3);
//! assert_eq!(cost, 20_750_000.0);
//!
//! // Offsets: 400 × £45,000 = £18.0M.
//! let offsets = downstream_offsets(400.0, 45_000.0);
//! assert_eq!(offsets, 18_000_000.0);
//!
//! // Net cost ≈ £2.8M — NOT cost-saving.
//! let net = net_cost(cost, offsets);
//! assert_eq!(net, 2_750_000.0);
//! assert!(!is_cost_saving(net));
//!
//! // QALYs gained = 400 × 3 = 1,200.
//! let q = qalys_gained(400.0, 3.0);
//! assert_eq!(q, 1_200.0);
//!
//! // Cost per QALY = 2.75M / 1,200 ≈ £2,300/QALY — outstandingly cost-effective.
//! let cpq = cost_per_qaly(net, q).unwrap();
//! assert!((cpq - 2_300.0).abs() < 50.0);
//! assert!(is_cost_effective(cpq, 20_000.0));
//! ```
//!
//! Same program, both truths: it loses £2.8M in cash and buys health at a
//! tenth of the NICE threshold. Fund it on the second number; never promise
//! the first.
//!
//! ## Software engineering connection
//!
//! - Shift-left quality is prevention economics, caveat included: reviews,
//!   tests, and static analysis apply cost to *every* change to catch issues
//!   in the few that would have progressed to production incidents.
//! - The defect-cost curve (10–100× by stage) plays the role of stroke costs.
//! - The honest conclusion mirrors health: shift-left is usually
//!   cost-*effective*, not automatically cost-*saving*, because most flagged
//!   issues would never have become incidents (the counterfactual few
//!   problem).
//! - Compute it: total gate cost per period vs incidents actually avoided ×
//!   incident cost — the same worked-example structure, with NNT as the
//!   per-catch unit.
//!
//! ## Pitfalls
//!
//! - **Claiming cost savings when the evidence supports cost-effectiveness**
//!   — the defining error of prevention advocacy in both domains.
//! - **Undiscounted future offsets**: benefits 15 years out at face value.
//! - **Ignoring overdiagnosis/overtreatment costs**: prevention finds
//!   pseudo-disease too (see screening economics).
//!
//! ## Sources
//!
//! - Cohen JT, Neumann PJ, Weinstein MC. "Does preventive care save money?"
//!   NEJM 2008. <https://www.nejm.org/doi/full/10.1056/NEJMp0708558>
//! - Masters R, et al. "Return on investment of public health interventions."
//!   JECH 2017. <https://pmc.ncbi.nlm.nih.gov/articles/PMC5537512/>
//!
//! Topic doc: health-economics-metrics/topics/prevention-economics.md

/// Total discounted program cost across the treated population.
///
/// Population × annual cost per person × discounted-years factor. The
/// discounted-years factor is the present-value sum of one £/year over the
/// horizon (e.g. 10 years at 3.5% ≈ 8.3).
///
/// # Arguments
///
/// * `population` — everyone treated (the whole at-risk population, not just
///   those who would have progressed).
/// * `annual_cost_per_person` — £ per person per year (worked example: £25).
/// * `discounted_years_factor` — present-value factor for the year stream
///   (worked example: ≈ 8.3 for 10 years at 3.5%).
///
/// # Returns
///
/// Total discounted program cost in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::program_cost;
///
/// // 100,000 people × £25/year × 10 years discounted (≈ ×8.3) ≈ £20.8M.
/// assert_eq!(program_cost(100_000.0, 25.0, 8.3), 20_750_000.0);
/// ```
pub fn program_cost(
    population: f64,
    annual_cost_per_person: f64,
    discounted_years_factor: f64,
) -> f64 {
    population * annual_cost_per_person * discounted_years_factor
}

/// Downstream costs avoided: cases prevented × discounted avoided cost per case.
///
/// The benefit side of the prevention ledger — it accrues only to the
/// counterfactual few who would have progressed. The avoided cost must
/// already be discounted to present value.
///
/// # Arguments
///
/// * `cases_prevented` — costly events prevented over the horizon (worked
///   example: 400 strokes).
/// * `avoided_cost_per_case` — discounted £ cost per case (worked example:
///   £45,000 per stroke).
///
/// # Returns
///
/// Total discounted downstream offsets in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::downstream_offsets;
///
/// // 400 strokes prevented × £45,000 discounted = £18.0M.
/// assert_eq!(downstream_offsets(400.0, 45_000.0), 18_000_000.0);
/// ```
pub fn downstream_offsets(cases_prevented: f64, avoided_cost_per_case: f64) -> f64 {
    cases_prevented * avoided_cost_per_case
}

/// Net cost of prevention: program cost minus downstream offsets.
///
/// Positive means the program is *not* cost-saving (it costs money net);
/// negative means genuine cash savings. Both inputs must be discounted to the
/// same present-value basis.
///
/// # Arguments
///
/// * `program_cost` — total discounted program cost in £ (see
///   [`program_cost`]).
/// * `downstream_offsets` — total discounted avoided costs in £ (see
///   [`downstream_offsets`]).
///
/// # Returns
///
/// `program_cost − downstream_offsets` in £.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::net_cost;
///
/// // £20.75M cost − £18.0M offsets ≈ £2.8M net cost: NOT cost-saving.
/// assert_eq!(net_cost(20_750_000.0, 18_000_000.0), 2_750_000.0);
/// ```
pub fn net_cost(program_cost: f64, downstream_offsets: f64) -> f64 {
    program_cost - downstream_offsets
}

/// Whether a program is net cost-saving (net cost below zero).
///
/// Fewer than 20% of preventive interventions pass this test (Cohen et al.,
/// NEJM 2008) — most prevention is cost-effective, not cost-saving.
///
/// # Arguments
///
/// * `net_cost` — net cost in £ (see [`net_cost`]).
///
/// # Returns
///
/// `true` when `net_cost < 0.0`. Exactly zero counts as *not* cost-saving.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::is_cost_saving;
///
/// // The worked example's £2.75M net cost is not a saving.
/// assert!(!is_cost_saving(2_750_000.0));
/// assert!(is_cost_saving(-500_000.0));
/// ```
pub fn is_cost_saving(net_cost: f64) -> bool {
    net_cost < 0.0
}

/// Per-person cost-saving condition.
///
/// Cost-saving requires intervention cost per person <
/// P(progression) × avoided cost × discount factor. Because P(progression)
/// is usually small, this condition fails for most prevention programs — the
/// prevention paradox in one inequality.
///
/// # Arguments
///
/// * `intervention_cost_per_person` — £ spent per person treated over the
///   horizon.
/// * `probability_of_progression` — probability (0–1) a person would have
///   progressed to the costly event without the program.
/// * `avoided_cost_per_case` — £ cost of the event avoided.
/// * `discount_factor` — present-value factor for the avoided cost (1.0 if
///   already discounted).
///
/// # Returns
///
/// `true` when the strict inequality holds (the program saves cash per
/// person).
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::per_person_cost_saving_condition;
///
/// // Worked example per person: £25 × 8.3 = £207.50 spent versus
/// // 0.4% progression × £45,000 = £180 avoided → not cost-saving.
/// assert!(!per_person_cost_saving_condition(25.0 * 8.3, 0.004, 45_000.0, 1.0));
/// ```
pub fn per_person_cost_saving_condition(
    intervention_cost_per_person: f64,
    probability_of_progression: f64,
    avoided_cost_per_case: f64,
    discount_factor: f64,
) -> bool {
    intervention_cost_per_person
        < probability_of_progression * avoided_cost_per_case * discount_factor
}

/// QALYs gained: cases prevented × QALYs that would have been lost per case.
///
/// # Arguments
///
/// * `cases_prevented` — costly events prevented (worked example: 400
///   strokes).
/// * `qalys_lost_per_case` — QALYs each event would have destroyed (worked
///   example: 3 per stroke).
///
/// # Returns
///
/// Total QALYs gained.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::qalys_gained;
///
/// // 400 strokes × 3 QALYs lost per stroke = 1,200 QALYs gained.
/// assert_eq!(qalys_gained(400.0, 3.0), 1_200.0);
/// ```
pub fn qalys_gained(cases_prevented: f64, qalys_lost_per_case: f64) -> f64 {
    cases_prevented * qalys_lost_per_case
}

/// Cost per QALY: net cost divided by QALYs gained.
///
/// The cost-effectiveness ratio to compare against a willingness-to-pay
/// threshold. A negative result (negative net cost) means the program
/// dominates: it saves money *and* gains health.
///
/// # Arguments
///
/// * `net_cost` — net cost in £ (see [`net_cost`]).
/// * `qalys_gained` — total QALYs gained (see [`qalys_gained`]).
///
/// # Returns
///
/// `Some(net_cost / qalys_gained)`, or `None` when `qalys_gained` is zero
/// (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::cost_per_qaly;
///
/// // £2.75M / 1,200 QALYs ≈ £2,300/QALY.
/// let cpq = cost_per_qaly(2_750_000.0, 1_200.0).unwrap();
/// assert!((cpq - 2_300.0).abs() < 50.0);
///
/// // No QALYs gained: undefined.
/// assert!(cost_per_qaly(1_000.0, 0.0).is_none());
/// ```
pub fn cost_per_qaly(net_cost: f64, qalys_gained: f64) -> Option<f64> {
    if qalys_gained == 0.0 {
        None
    } else {
        Some(net_cost / qalys_gained)
    }
}

/// Cost-effectiveness test: cost per QALY strictly below the willingness-to-pay threshold.
///
/// # Arguments
///
/// * `cost_per_qaly` — £ per QALY (see [`cost_per_qaly`]).
/// * `threshold_per_qaly` — willingness-to-pay per QALY in £ (NICE:
///   £20,000–£30,000).
///
/// # Returns
///
/// `true` when `cost_per_qaly < threshold_per_qaly`.
///
/// # Examples
///
/// ```rust
/// use health_economics::prevention_economics::is_cost_effective;
///
/// // ≈ £2,300/QALY is a tenth of the £20,000 NICE lower threshold.
/// assert!(is_cost_effective(2_300.0, 20_000.0));
/// assert!(!is_cost_effective(35_000.0, 30_000.0));
/// ```
pub fn is_cost_effective(cost_per_qaly: f64, threshold_per_qaly: f64) -> bool {
    cost_per_qaly < threshold_per_qaly
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cost: 100,000 people × £25/year × 10 years discounted (≈ ×8.3)
    /// ≈ £20.8M.
    #[test]
    fn hypertension_app_program_cost_is_about_20_8_million() {
        // Worked example: "Cost: 100,000 × £25 × 10 yrs (discounted ≈ ×8.3) ≈ £20.8M."
        let c = program_cost(100_000.0, 25.0, 8.3);
        assert!((c - 20_750_000.0).abs() < 1e-9);
        assert!((c - 20_800_000.0).abs() < 100_000.0);
    }

    /// Offsets: 400 strokes prevented × £45,000 discounted = £18.0M.
    #[test]
    fn stroke_offsets_are_18_million() {
        // Worked example: "Offsets: 400 × £45,000 = £18.0M."
        let o = downstream_offsets(400.0, 45_000.0);
        assert!((o - 18_000_000.0).abs() < 1e-9);
    }

    /// Net cost ≈ £2.8M — NOT cost-saving.
    #[test]
    fn net_cost_is_about_2_8_million_and_not_cost_saving() {
        // Worked example: "Net cost ≈ £2.8M — NOT cost-saving."
        let n = net_cost(program_cost(100_000.0, 25.0, 8.3), 18_000_000.0);
        assert!((n - 2_750_000.0).abs() < 1e-9);
        assert!((n - 2_800_000.0).abs() < 100_000.0);
        assert!(!is_cost_saving(n));
    }

    /// QALYs gained = 400 strokes × 3 QALYs lost per stroke = 1,200.
    #[test]
    fn qalys_gained_are_1_200() {
        // Worked example: "QALYs gained = 400 × 3 = 1,200."
        let q = qalys_gained(400.0, 3.0);
        assert!((q - 1_200.0).abs() < 1e-9);
    }

    /// Cost per QALY = £2.75M / 1,200 ≈ £2,300/QALY — outstandingly
    /// cost-effective at NICE thresholds.
    #[test]
    fn cost_per_qaly_is_about_2_300_and_cost_effective() {
        // Worked example: "Cost per QALY = 2.8M / 1,200 ≈ £2,300/QALY —
        // outstandingly cost-effective."
        let n = net_cost(program_cost(100_000.0, 25.0, 8.3), 18_000_000.0);
        let cpq = cost_per_qaly(n, 1_200.0).unwrap();
        assert!((cpq - 2_300.0).abs() < 50.0);
        assert!(is_cost_effective(cpq, 20_000.0));
    }

    /// Per-person check for the worked example: £25 × 8.3 discounted years
    /// (£207.50 per person) versus 0.4% progression × £45,000 (£180): the
    /// condition confirms the program is not cost-saving.
    #[test]
    fn per_person_condition_confirms_not_cost_saving() {
        // Doc math: "Cost-saving requires: intervention cost <
        // P(progression) × avoided cost × discount factor."
        // 400 strokes / 100,000 people = 0.004 probability of progression.
        // Intervention cost per person over the horizon: £25 × 8.3 = £207.50.
        // Avoided cost of £45,000 is already discounted (factor 1.0).
        assert!(!per_person_cost_saving_condition(
            25.0 * 8.3,
            0.004,
            45_000.0,
            1.0
        ));
    }

    // Edge case: zero QALYs gained leaves cost per QALY undefined.
    #[test]
    fn zero_qalys_gained_has_no_defined_cost_per_qaly() {
        assert!(cost_per_qaly(1_000.0, 0.0).is_none());
    }
}
