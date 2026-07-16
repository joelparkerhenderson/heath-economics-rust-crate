//! # Earlier Intervention
//!
//! Values the double dividend of moving patients from waiting list to active
//! treatment sooner. Disease progression is the compounding interest of
//! health care: a patient waiting with an untreated condition is not in a
//! steady state — cancers stage-shift, heart failure decompensates, mild
//! depression becomes severe. Intervening earlier therefore delivers both
//! **better outcomes** (more QALYs, treated from a healthier baseline) and
//! often **lower treatment costs** (early-stage treatment is less intensive
//! than late-stage rescue).
//!
//! The essential discipline is probability weighting: not every waiting
//! patient progresses, so the value is weighted by the probability that a
//! patient actually progresses during the delay that was removed.
//!
//! ## Formula
//!
//! ```text
//! Value of earlier intervention (per patient) =
//!     [Cost_late − Cost_early]                       (treatment-cost offset)
//!   + [QALYs_early − QALYs_late] × λ                 (health gain × threshold)
//!   × P(progression during the delay)                 (probability weighting)
//!
//! Progression events avoided = patients × annual progression rate × years of acceleration
//!
//! Cost_late / Cost_early   = treatment cost if treated late vs early (£)
//! QALYs_early / QALYs_late = quality-adjusted life years if treated early vs late
//! λ (lambda)               = cost-effectiveness threshold (£/QALY, e.g. £20,000)
//! P(progression)           = probability a waiting patient progresses during the delay
//! ```
//!
//! ## Why it matters
//!
//! This mechanism is what elevates "faster pathways" from an operational
//! nicety to a clinical and economic imperative — and it is the deep reason
//! cost of delay applies to clinical software. Model the transition
//! probability per unit time from natural-history data (e.g. ~2% of waiting
//! diabetic-retinopathy patients per year progress to sight-threatening
//! stages), not the worst case; then discount, and note most early
//! intervention is cost-*effective* rather than cost-*saving*.
//!
//! ## Example
//!
//! Diabetic retinopathy screening backlog: 4,000 patients, 6 months behind.
//! AI-assisted grading clears the queue ~4 months sooner. Natural history:
//! ~2% of waiting patients/year progress. Per avoided progression: £4,000
//! treatment offset plus 0.8 QALYs × £20,000 = £16,000.
//!
//! ```
//! use health_economics::earlier_intervention::{
//!     progression_events_avoided, value_per_avoided_progression, total_backlog_value,
//! };
//!
//! // 4,000 × 2% × (4/12) ≈ 27 progression events avoided.
//! let events = progression_events_avoided(4_000.0, 0.02, 4.0 / 12.0);
//! assert!((events - 26.666_666).abs() < 0.001);
//!
//! // Treatment offset £4,000 + 0.8 QALYs × £20,000 = £20,000 per event.
//! let per_event = value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0);
//! assert_eq!(per_event, 20_000.0);
//!
//! // 27 × £20,000 ≈ £540,000 from one backlog cleared once.
//! let value = total_backlog_value(27.0, per_event);
//! assert_eq!(value, 540_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Software that accelerates diagnostic and treatment pathways (triage, AI
//!   grading, results routing) monetizes via this exact model.
//! - The model tells you which pathway to accelerate: the one with the
//!   steepest progression curve, not the longest queue.
//! - The engineering mirror: **defects progress too** — a bug caught in
//!   design costs a conversation; in production it costs an incident.
//! - The "shift-left" cost curve (10–100× by stage) is a progression model,
//!   with the same caveat: early detection is usually cost-effective, not
//!   free money, because reviews and tests have real costs and most caught
//!   issues would never have progressed.
//!
//! ## Pitfalls
//!
//! - **Worst-case progression assumed for everyone** — the probability
//!   weighting is the difference between analysis and advocacy.
//! - **Lead-time bias**: finding disease earlier without changing outcomes
//!   looks like benefit but isn't; earlier *effective intervention* is the
//!   claim, not earlier detection alone.
//! - **Double counting** with waiting-list and RTT claims built on the same
//!   acceleration — one pathway improvement, one set of benefits, allocated
//!   once.
//!
//! ## Sources
//!
//! - Cohen JT, Neumann PJ, Weinstein MC. "Does preventive care save money?"
//!   NEJM 2008. <https://www.nejm.org/doi/full/10.1056/NEJMp0708558>
//! - NHS England, diabetic eye screening programme.
//!   <https://www.gov.uk/topic/population-screening-programmes/diabetic-eye>
//!
//! Topic doc: health-economics-metrics/topics/earlier-intervention.md

/// Expected number of progression events avoided by clearing a backlog sooner.
///
/// Models progression as a constant hazard: the expected count is the cohort
/// size times the annual progression probability times the fraction of a year
/// by which review was accelerated. Returns a fractional expectation, not an
/// integer count.
///
/// # Arguments
///
/// * `patients` — number of patients in the backlog (count).
/// * `annual_progression_rate` — probability an un-reviewed patient
///   progresses per year (fraction, e.g. `0.02` for 2%/year).
/// * `acceleration_years` — how much sooner the backlog is reviewed (years,
///   e.g. `4.0 / 12.0` for four months).
///
/// # Returns
///
/// Expected number of progression events avoided (may be fractional).
///
/// # Examples
///
/// ```
/// use health_economics::earlier_intervention::progression_events_avoided;
///
/// // Worked example: 4,000 patients × 2%/year × 4/12 year ≈ 27 events.
/// let events = progression_events_avoided(4_000.0, 0.02, 4.0 / 12.0);
/// assert!((events - 27.0).abs() < 0.5);
/// ```
pub fn progression_events_avoided(
    patients: f64,
    annual_progression_rate: f64,
    acceleration_years: f64,
) -> f64 {
    // Expected events = cohort × per-year hazard × exposure time (years).
    patients * annual_progression_rate * acceleration_years
}

/// Monetary value of one avoided progression event.
///
/// Sums the two dividends of earlier treatment: the treatment-cost offset
/// (late-stage cost minus early-stage cost, in £) and the health gain
/// (QALY difference) monetized at the cost-effectiveness threshold.
///
/// # Arguments
///
/// * `cost_late` — treatment cost if the patient progresses and is treated
///   late (£).
/// * `cost_early` — treatment cost if treated early (£).
/// * `qalys_early` — QALYs accrued when treated early.
/// * `qalys_late` — QALYs accrued when treated late.
/// * `lambda` — cost-effectiveness threshold (£/QALY, e.g. £20,000).
///
/// # Returns
///
/// Value per avoided progression (£): `(cost_late − cost_early) +
/// (qalys_early − qalys_late) × lambda`. Negative if late treatment were
/// somehow cheaper and better.
///
/// # Examples
///
/// ```
/// use health_economics::earlier_intervention::value_per_avoided_progression;
///
/// // Worked example: £4,000 offset + 0.8 QALYs × £20,000 = £20,000.
/// let v = value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0);
/// assert_eq!(v, 20_000.0);
/// ```
pub fn value_per_avoided_progression(
    cost_late: f64,
    cost_early: f64,
    qalys_early: f64,
    qalys_late: f64,
    lambda: f64,
) -> f64 {
    // Treatment-cost offset + monetized QALY gain (health gain × threshold λ).
    (cost_late - cost_early) + (qalys_early - qalys_late) * lambda
}

/// Per-patient value of earlier intervention, probability-weighted.
///
/// The per-progression value applies only to patients who would actually
/// have progressed during the removed delay, so it is weighted by that
/// probability — the difference between analysis and advocacy.
///
/// # Arguments
///
/// * `cost_late` — late-treatment cost (£).
/// * `cost_early` — early-treatment cost (£).
/// * `qalys_early` — QALYs when treated early.
/// * `qalys_late` — QALYs when treated late.
/// * `lambda` — cost-effectiveness threshold (£/QALY).
/// * `probability_of_progression` — probability a waiting patient progresses
///   during the delay removed (fraction 0–1, e.g. `0.02 × 4/12` for a
///   4-month acceleration at 2%/year).
///
/// # Returns
///
/// Expected value per waiting patient (£).
///
/// # Examples
///
/// ```
/// use health_economics::earlier_intervention::value_per_patient;
///
/// // 2%/year over 4 months of acceleration, £20,000 per avoided progression.
/// let p = 0.02 * (4.0 / 12.0);
/// let v = value_per_patient(4_000.0, 0.0, 0.8, 0.0, 20_000.0, p);
/// // ≈ £133 per waiting patient; × 4,000 patients ≈ £533k.
/// assert!((v * 4_000.0 - 533_333.33).abs() < 1.0);
/// ```
pub fn value_per_patient(
    cost_late: f64,
    cost_early: f64,
    qalys_early: f64,
    qalys_late: f64,
    lambda: f64,
    probability_of_progression: f64,
) -> f64 {
    // Full per-progression value × P(progression during the delay removed).
    value_per_avoided_progression(cost_late, cost_early, qalys_early, qalys_late, lambda)
        * probability_of_progression
}

/// Total value of clearing a backlog earlier.
///
/// Multiplies the expected number of avoided progression events by the value
/// of each avoided progression. Equivalent to summing `value_per_patient`
/// over the whole backlog.
///
/// # Arguments
///
/// * `events_avoided` — expected progression events avoided (from
///   [`progression_events_avoided`]).
/// * `value_per_event` — value of one avoided progression (£, from
///   [`value_per_avoided_progression`]).
///
/// # Returns
///
/// Total backlog value (£).
///
/// # Examples
///
/// ```
/// use health_economics::earlier_intervention::total_backlog_value;
///
/// // Worked example: 27 × (£4,000 + £16,000) ≈ £540,000.
/// let v = total_backlog_value(27.0, 20_000.0);
/// assert_eq!(v, 540_000.0);
/// ```
pub fn total_backlog_value(events_avoided: f64, value_per_event: f64) -> f64 {
    events_avoided * value_per_event
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "4,000 × 2% × (4/12) ≈ 27 patients".
    #[test]
    fn worked_example_progression_events_avoided_is_about_27() {
        let events = progression_events_avoided(4_000.0, 0.02, 4.0 / 12.0);
        assert!((events - 27.0).abs() < 0.5, "got {events}");
    }

    // Doc lines: "treatment offset ≈ £4,000" + "0.8 QALYs × £20,000 = £16,000"
    // → £20,000 per avoided progression.
    #[test]
    fn worked_example_value_per_avoided_progression_is_20000() {
        // The doc quotes the offset directly (£4,000) and the QALY gain (0.8);
        // express the offset as cost_late − cost_early = 4,000.
        let value = value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0);
        assert!((value - 20_000.0).abs() < 1e-9, "got {value}");
    }

    // Doc line: "QALY gain (vision preserved) ≈ 0.8 QALYs × £20,000 = £16,000".
    #[test]
    fn worked_example_qaly_component_is_16000() {
        let value = value_per_avoided_progression(0.0, 0.0, 0.8, 0.0, 20_000.0);
        assert!((value - 16_000.0).abs() < 1e-9, "got {value}");
    }

    // Doc line: "Value ≈ 27 × (4,000 + 16,000) ≈ £540,000 — from one backlog
    // cleared once".
    #[test]
    fn worked_example_total_backlog_value_is_about_540000() {
        let value = total_backlog_value(27.0, 20_000.0);
        assert!((value - 540_000.0).abs() < 1e-9, "got {value}");
        // Un-rounded event count gives ≈ £533k, within the doc's "≈ £540,000".
        let events = progression_events_avoided(4_000.0, 0.02, 4.0 / 12.0);
        let value_unrounded = total_backlog_value(
            events,
            value_per_avoided_progression(4_000.0, 0.0, 0.8, 0.0, 20_000.0),
        );
        assert!((value_unrounded - 540_000.0).abs() < 10_000.0, "got {value_unrounded}");
    }

    // Doc formula: value per patient is probability-weighted; summed over the
    // 4,000-patient backlog it reproduces the un-rounded ≈ £533k total.
    #[test]
    fn per_patient_value_applies_probability_weighting() {
        // Probability a waiting patient progresses during 4 months: 2% × 4/12.
        let p = 0.02 * (4.0 / 12.0);
        let per_patient = value_per_patient(4_000.0, 0.0, 0.8, 0.0, 20_000.0, p);
        let total = per_patient * 4_000.0;
        assert!((total - 533_333.333_333_333_3).abs() < 1e-6, "got {total}");
    }
}
