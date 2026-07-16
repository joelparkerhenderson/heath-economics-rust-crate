//! # Emergency Attendance Avoidance
//!
//! Counts ED (A&E) visits and emergency admissions prevented by upstream
//! intervention — triage apps, remote monitoring, virtual wards, urgent-care
//! redirection — and converts "we caught it earlier" into a costed claim.
//!
//! The model nets gross savings against the cost of the intervention *and*
//! the cost of new pathway usage, because redirected demand isn't free: a 111
//! call, a GP slot, and a virtual-ward day all have unit costs.
//!
//! ## Formula
//!
//! ```text
//! Avoided attendances = population × (baseline rate − intervention rate)
//! Gross saving        = avoided attendances × unit cost per attendance
//!                       (+ avoided admissions × admission cost, counted separately)
//!
//! Net saving          = gross saving − intervention cost − cost of new pathway usage
//!
//! population          = people covered by the intervention (count)
//! baseline rate       = emergency events per person-year without the intervention
//! intervention rate   = emergency events per person-year with the intervention
//! unit cost           = £ per attendance (or per admission)
//! ```
//!
//! ## Why it matters
//!
//! Emergency care is the most expensive routine setting in the system — ED
//! attendance unit costs run £250–£400 per National Cost Collection / PSSRU
//! figures, and an emergency admission is thousands — and ED crowding
//! cascades into ambulance delays and cancelled electives. Anything that
//! safely resolves demand upstream buys the system capacity at its most
//! stressed point; this is the standard benefit line for symptom checkers,
//! 111-style triage, and remote patient monitoring. The causal claim needs a
//! comparator: attendance rates trend and vary seasonally, so before/after
//! alone proves nothing.
//!
//! ## Example
//!
//! A COPD remote-monitoring service for 3,000 high-risk patients: matched
//! controls show ED attendance falling 0.9 → 0.7 per patient-year and
//! emergency admissions 0.5 → 0.42.
//!
//! ```
//! use health_economics::emergency_attendance_avoidance::{
//!     avoided_events, gross_saving_attendances_and_admissions, net_saving,
//! };
//!
//! // Avoided attendances = 3,000 × 0.2 = 600; avoided admissions = 3,000 × 0.08 = 240.
//! let attendances = avoided_events(3_000.0, 0.9, 0.7);
//! let admissions = avoided_events(3_000.0, 0.5, 0.42);
//! assert!((attendances - 600.0).abs() < 1e-9);
//! assert!((admissions - 240.0).abs() < 1e-9);
//!
//! // Gross = 600 × £300 + 240 × £3,800 = £180,000 + £912,000 = £1,092,000/year.
//! let gross = gross_saving_attendances_and_admissions(attendances, 300.0, admissions, 3_800.0);
//! assert!((gross - 1_092_000.0).abs() < 1e-6);
//!
//! // Net = £1,092,000 − £600,000 monitoring − £150,000 nurse responses = +£342,000/year.
//! let net = net_saving(gross, 600_000.0, 150_000.0);
//! assert!((net - 342_000.0).abs() < 1e-6);
//! ```
//!
//! Note the admissions line dominates: attendance avoidance alone rarely pays
//! for a monitoring service; *admission* avoidance is where the money is.
//!
//! ## Software engineering connection
//!
//! - This is **incident-avoidance economics**: the value of observability,
//!   canary deploys, and early-warning systems is avoided "emergency
//!   attendances" — pages, war rooms, sev-1s — each with a loaded cost.
//! - Net out the cost of the new upstream pathway: alert triage isn't free.
//! - Beware substitution: alerts that create work without preventing
//!   incidents are health anxiety, not health.
//! - Prove the counterfactual with a control — teams' incident rates trend
//!   and regress to the mean, exactly like ED attendance.
//!
//! ## Pitfalls
//!
//! - **Regression to the mean**: high-risk cohorts selected on a bad year
//!   improve untreated; matched controls or stepped-wedge designs are
//!   essential.
//! - **Supply-induced demand**: easy digital triage can *increase* total
//!   contacts (lower threshold to seek help) while decreasing ED share —
//!   count total system cost.
//! - **Valuing attendances at average cost** when ED fixed costs don't fall
//!   (see marginal vs average cost).
//!
//! ## Sources
//!
//! - NHS England, National Cost Collection.
//!   <https://www.england.nhs.uk/costing-in-the-nhs/national-cost-collection/>
//! - PSSRU, Unit Costs of Health and Social Care.
//!   <https://www.pssru.ac.uk/unitcostsreport/>
//!
//! Topic doc: health-economics-metrics/topics/emergency-attendance-avoidance.md

/// Number of emergency events (attendances or admissions) avoided.
///
/// Applies the rate difference to the covered population. Rates are per
/// person-year, so the result is events avoided per year. Negative if the
/// intervention rate is higher than baseline (the intervention increased
/// emergency use).
///
/// # Arguments
///
/// * `population` — people covered by the intervention (count).
/// * `baseline_rate` — events per person-year without the intervention.
/// * `intervention_rate` — events per person-year with the intervention.
///
/// # Returns
///
/// Events avoided per year: `population × (baseline_rate − intervention_rate)`.
///
/// # Examples
///
/// ```
/// use health_economics::emergency_attendance_avoidance::avoided_events;
///
/// // Worked example: 3,000 patients, ED attendance 0.9 → 0.7 per patient-year.
/// assert!((avoided_events(3_000.0, 0.9, 0.7) - 600.0).abs() < 1e-9);
/// // Admissions 0.5 → 0.42 per patient-year.
/// assert!((avoided_events(3_000.0, 0.5, 0.42) - 240.0).abs() < 1e-9);
/// ```
pub fn avoided_events(population: f64, baseline_rate: f64, intervention_rate: f64) -> f64 {
    population * (baseline_rate - intervention_rate)
}

/// Gross saving from a set of avoided events valued at a unit cost.
///
/// Values every avoided event at the same unit cost — use marginal (not
/// average) cost when ED fixed costs won't actually fall.
///
/// # Arguments
///
/// * `avoided_events` — number of events avoided (count/year).
/// * `unit_cost` — cost per event (£, e.g. £300/attendance, £3,800/admission).
///
/// # Returns
///
/// Gross saving (£/year).
///
/// # Examples
///
/// ```
/// use health_economics::emergency_attendance_avoidance::gross_saving;
///
/// // Worked example: 600 avoided attendances × £300 = £180,000.
/// assert_eq!(gross_saving(600.0, 300.0), 180_000.0);
/// ```
pub fn gross_saving(avoided_events: f64, unit_cost: f64) -> f64 {
    avoided_events * unit_cost
}

/// Gross saving counting attendances and admissions as separate lines.
///
/// The doc's model counts admissions separately from attendances because the
/// unit costs differ by an order of magnitude — and the admissions line
/// usually dominates.
///
/// # Arguments
///
/// * `avoided_attendances` — ED attendances avoided (count/year).
/// * `attendance_unit_cost` — £ per attendance (£250–£400 range typical).
/// * `avoided_admissions` — emergency admissions avoided (count/year).
/// * `admission_unit_cost` — £ per admission (thousands).
///
/// # Returns
///
/// Combined gross saving (£/year).
///
/// # Examples
///
/// ```
/// use health_economics::emergency_attendance_avoidance::gross_saving_attendances_and_admissions;
///
/// // Worked example: 600 × £300 + 240 × £3,800 = £1,092,000/year.
/// let gross = gross_saving_attendances_and_admissions(600.0, 300.0, 240.0, 3_800.0);
/// assert_eq!(gross, 1_092_000.0);
/// ```
pub fn gross_saving_attendances_and_admissions(
    avoided_attendances: f64,
    attendance_unit_cost: f64,
    avoided_admissions: f64,
    admission_unit_cost: f64,
) -> f64 {
    avoided_attendances * attendance_unit_cost + avoided_admissions * admission_unit_cost
}

/// Net saving after intervention and new-pathway costs.
///
/// Redirected demand isn't free: a 111 call, a GP slot, or a virtual-ward
/// day all have unit costs, and the intervention itself must be paid for.
///
/// # Arguments
///
/// * `gross_saving` — gross saving from avoided events (£/year).
/// * `intervention_cost` — cost of running the intervention (£/year).
/// * `new_pathway_cost` — cost of the new pathway usage the intervention
///   creates (£/year, e.g. extra community-nurse responses).
///
/// # Returns
///
/// Net saving (£/year); negative means the intervention costs more than it
/// saves in cash terms (QALY gains counted separately).
///
/// # Examples
///
/// ```
/// use health_economics::emergency_attendance_avoidance::net_saving;
///
/// // Worked example: £1,092,000 − £600,000 − £150,000 = +£342,000/year.
/// assert_eq!(net_saving(1_092_000.0, 600_000.0, 150_000.0), 342_000.0);
/// ```
pub fn net_saving(gross_saving: f64, intervention_cost: f64, new_pathway_cost: f64) -> f64 {
    gross_saving - intervention_cost - new_pathway_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "Avoided attendances = 3,000 × 0.2 = 600".
    #[test]
    fn worked_example_avoided_attendances_is_600() {
        let a = avoided_events(3_000.0, 0.9, 0.7);
        assert!((a - 600.0).abs() < 1e-9, "got {a}");
    }

    // Doc line: "600 × £300 = £180,000".
    #[test]
    fn worked_example_attendance_saving_is_180000() {
        let s = gross_saving(avoided_events(3_000.0, 0.9, 0.7), 300.0);
        assert!((s - 180_000.0).abs() < 1e-9, "got {s}");
    }

    // Doc line: "Avoided admissions = 3,000 × 0.08 = 240".
    #[test]
    fn worked_example_avoided_admissions_is_240() {
        let a = avoided_events(3_000.0, 0.5, 0.42);
        assert!((a - 240.0).abs() < 1e-9, "got {a}");
    }

    // Doc line: "240 × £3,800 = £912,000".
    #[test]
    fn worked_example_admission_saving_is_912000() {
        let s = gross_saving(avoided_events(3_000.0, 0.5, 0.42), 3_800.0);
        assert!((s - 912_000.0).abs() < 1e-9, "got {s}");
    }

    // Doc line: "Gross £1,092,000/year" (£180,000 + £912,000).
    #[test]
    fn worked_example_gross_saving_is_1092000() {
        let g = gross_saving_attendances_and_admissions(
            avoided_events(3_000.0, 0.9, 0.7),
            300.0,
            avoided_events(3_000.0, 0.5, 0.42),
            3_800.0,
        );
        assert!((g - 1_092_000.0).abs() < 1e-9, "got {g}");
    }

    // Doc line: "Net ≈ +£342,000/year" (£1,092,000 − £600,000 − £150,000).
    #[test]
    fn worked_example_net_saving_is_342000() {
        let n = net_saving(1_092_000.0, 600_000.0, 150_000.0);
        assert!((n - 342_000.0).abs() < 1e-9, "got {n}");
    }
}
