//! # Cost of Delay (CoD)
//!
//! Cost of Delay is the economic value lost per unit time that a feature,
//! product, or service is *not* delivered. It is the single strongest bridge
//! between software delivery metrics and health economics: it converts "we
//! shipped late" into currency — or into QALYs.
//!
//! For clinical software, CoD can be denominated in health as well as money:
//! every week a pathway improvement is delayed, patients wait longer in worse
//! health states, and that health loss can be priced at the willingness-to-pay
//! threshold.
//!
//! ## Formula
//!
//! ```text
//! CoD = benefit per unit time forgone while undelivered   (£/week or QALYs/week)
//!
//! Total delay loss = CoD × delay duration
//!
//! CoD_health = patients affected per week × QALY gain per patient
//! CoD_money  = CoD_health × λ + operational savings per week forgone
//!
//! CoD        — cost of delay per unit time
//! λ          — willingness-to-pay threshold (£/QALY, typically £20k–30k)
//! QALY gain per patient = (waiting weeks removed / 52) × utility gain
//! ```
//!
//! ## Why it matters
//!
//! Reinertsen's rule: "If you only quantify one thing, quantify the Cost of
//! Delay." Most organizations know what a project costs but not what a month
//! of delay costs, so they optimize budgets while hemorrhaging time-value. For
//! healthcare software the stakes are literal: every week a pathway
//! improvement is delayed, patients wait longer in worse health states. CoD is
//! the strongest mathematical framework to present to NHS stakeholders because
//! it prices the *absence* of your software (benchmark for scale: Black Swan
//! Farming's Maersk analysis found single features with CoD ≈ $200k/week that
//! had waited 38 weeks).
//!
//! ## Example
//!
//! The topic doc's worked example: software saves £200 per patient on a
//! pathway processing 50 patients/week (CoD = £10,000/week; a 10-week
//! procurement delay wastes £100,000), and a triage improvement that removes
//! 5 weeks of waiting (utility 0.68 → 0.80) for 100 patients/week is worth
//! ~1.15 QALYs/week ≈ £23,000/week at £20,000/QALY.
//!
//! ```rust
//! use health_economics::cost_of_delay::{
//!     operational_cost_of_delay, total_delay_loss, qaly_gain_per_patient,
//!     cost_of_delay_health, cost_of_delay_money,
//! };
//!
//! // Operational: £200/patient × 50 patients/week = £10,000/week.
//! let cod = operational_cost_of_delay(200.0, 50.0);
//! assert_eq!(cod, 10_000.0);
//! // A 10-week procurement delay costs £100,000 in avoidable waste.
//! assert_eq!(total_delay_loss(cod, 10.0), 100_000.0);
//!
//! // Clinical: 5 weeks of waiting removed, utility 0.68 → 0.80.
//! let gain = qaly_gain_per_patient(5.0, 0.80 - 0.68);
//! assert!((gain - 0.0115).abs() < 1e-4);
//! let cod_health = cost_of_delay_health(100.0, gain);
//! assert!((cod_health - 1.15).abs() < 5e-3); // ≈ 1.15 QALYs/week
//! let cod_money = cost_of_delay_money(cod_health, 20_000.0, 0.0);
//! assert!((cod_money - 23_000.0).abs() < 100.0); // ≈ £23,000/week
//!
//! // A 6-month (26-week) deployment delay "costs" ~30 QALYs.
//! assert!((total_delay_loss(cod_health, 26.0) - 30.0).abs() < 0.1);
//! ```
//!
//! ## Software engineering connection
//!
//! - CoD makes DORA lead time and flow efficiency financially legible:
//!   lead time × CoD = money (or health) burned in queues.
//! - **Prioritization**: rank work by CoD/duration (WSJF/CD3) instead of by
//!   loudest stakeholder.
//! - **Process economics**: a 2-week release cadence has an expected delay
//!   cost of ~1 week × CoD per feature versus continuous delivery — price the
//!   batch.
//! - **Procurement**: NHS procurement cycles of 6–18 months have a CoD;
//!   showing it changes urgency conversations.
//!
//! ## Pitfalls
//!
//! - **Assuming linear CoD**: some work has deadline-shaped value (regulatory
//!   dates — infinite CoD after the date, zero before) or decaying value
//!   (first-mover windows). Classify the urgency profile before multiplying.
//! - **CoD on outputs nobody wants**: delay only costs if the thing has
//!   value; garbage delayed is free.
//! - **Double counting delay and discounting**: discounting already prices
//!   time on multi-year horizons; CoD is the within-horizon operational
//!   version. Use CoD for weeks/months, NPV shift for years.
//!
//! ## Sources
//!
//! - Reinertsen DG, *The Principles of Product Development Flow*.
//! - Black Swan Farming, Cost of Delay. <https://blackswanfarming.com/cost-of-delay/>
//! - Cost of delay overview. <https://en.wikipedia.org/wiki/Cost_of_delay>
//!
//! Topic doc: health-economics-metrics/topics/cost-of-delay.md

/// Operational cost of delay per unit time.
///
/// Multiplies the saving the software produces per patient by the number of
/// patients processed per week, yielding the £/week forgone while the
/// software is undelivered. Any consistent time unit works as long as
/// `patients_per_week` and downstream uses agree.
///
/// # Arguments
///
/// * `saving_per_patient` — operational saving per patient (£).
/// * `patients_per_week` — patients processed on the pathway per week.
///
/// # Returns
///
/// Cost of delay in £/week (same currency as `saving_per_patient`).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_of_delay::operational_cost_of_delay;
///
/// // £200 per patient × 50 patients/week = £10,000/week.
/// assert_eq!(operational_cost_of_delay(200.0, 50.0), 10_000.0);
/// ```
pub fn operational_cost_of_delay(saving_per_patient: f64, patients_per_week: f64) -> f64 {
    saving_per_patient * patients_per_week
}

/// Total loss from a delay of a given duration.
///
/// Multiplies the per-week cost of delay by the delay duration; both
/// arguments must use the same time unit (weeks here by convention). Works
/// equally for money (£/week) and health (QALYs/week) denominations.
///
/// # Arguments
///
/// * `cost_of_delay_per_week` — CoD per week (£/week or QALYs/week).
/// * `delay_weeks` — delay duration, in weeks.
///
/// # Returns
///
/// Total delay loss, in the same value unit as the CoD (£ or QALYs).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_of_delay::total_delay_loss;
///
/// // A 10-week procurement delay at £10,000/week costs £100,000.
/// assert_eq!(total_delay_loss(10_000.0, 10.0), 100_000.0);
/// ```
pub fn total_delay_loss(cost_of_delay_per_week: f64, delay_weeks: f64) -> f64 {
    cost_of_delay_per_week * delay_weeks
}

/// QALY gain per patient from removing weeks of waiting in a worse health
/// state.
///
/// Converts the removed waiting time to years (dividing by 52) and multiplies
/// by the utility gain — the difference between the utilities of the better
/// and worse health states on the 0 (dead) to 1 (full health) scale.
///
/// # Arguments
///
/// * `waiting_weeks_removed` — weeks of waiting eliminated per patient.
/// * `utility_gain` — utility difference gained sooner (e.g. 0.80 − 0.68 = 0.12).
///
/// # Returns
///
/// QALYs gained per patient.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_of_delay::qaly_gain_per_patient;
///
/// // (5/52) × 0.12 ≈ 0.0115 QALYs per patient.
/// let gain = qaly_gain_per_patient(5.0, 0.12);
/// assert!((gain - 0.0115).abs() < 1e-4);
/// ```
pub fn qaly_gain_per_patient(waiting_weeks_removed: f64, utility_gain: f64) -> f64 {
    // Convert weeks to years (52 weeks/year) before applying the utility gain.
    (waiting_weeks_removed / 52.0) * utility_gain
}

/// Health-denominated cost of delay (QALYs/week).
///
/// Multiplies the number of patients affected each week the software is
/// undelivered by the QALY gain each of them would have received.
///
/// # Arguments
///
/// * `patients_per_week` — patients affected per week of delay.
/// * `qaly_gain_per_patient` — QALYs each patient would gain (see
///   [`qaly_gain_per_patient`]).
///
/// # Returns
///
/// Cost of delay in QALYs per week.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_of_delay::{
///     cost_of_delay_health, qaly_gain_per_patient,
/// };
///
/// // 100 patients/week × ~0.0115 QALYs each ≈ 1.15 QALYs/week.
/// let gain = qaly_gain_per_patient(5.0, 0.12);
/// let cod_health = cost_of_delay_health(100.0, gain);
/// assert!((cod_health - 1.15).abs() < 5e-3);
/// ```
pub fn cost_of_delay_health(patients_per_week: f64, qaly_gain_per_patient: f64) -> f64 {
    patients_per_week * qaly_gain_per_patient
}

/// Money-denominated cost of delay (£/week).
///
/// Monetizes the health CoD at the willingness-to-pay threshold λ
/// (typically £20,000–30,000 per QALY in NICE terms), then adds any
/// operational savings per week that are also forgone while undelivered.
///
/// # Arguments
///
/// * `cost_of_delay_health_qalys_per_week` — health CoD, QALYs/week (see
///   [`cost_of_delay_health`]).
/// * `willingness_to_pay_per_qaly` — λ, in £/QALY.
/// * `operational_savings_per_week` — operational £/week additionally forgone
///   (0.0 if none).
///
/// # Returns
///
/// Cost of delay in £/week.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_of_delay::cost_of_delay_money;
///
/// // 1.15 QALYs/week × £20,000/QALY ≈ £23,000/week of health value.
/// let cod_money = cost_of_delay_money(1.15, 20_000.0, 0.0);
/// assert!((cod_money - 23_000.0).abs() < 1e-9);
/// ```
pub fn cost_of_delay_money(
    cost_of_delay_health_qalys_per_week: f64,
    willingness_to_pay_per_qaly: f64,
    operational_savings_per_week: f64,
) -> f64 {
    // Monetize health at λ, then add operational £/week forgone.
    cost_of_delay_health_qalys_per_week * willingness_to_pay_per_qaly
        + operational_savings_per_week
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "CoD = 200 × 50 = £10,000/week".
    #[test]
    fn operational_cod_is_10k_per_week() {
        // £200 per patient × 50 patients/week = £10,000/week
        let cod = operational_cost_of_delay(200.0, 50.0);
        assert!((cod - 10_000.0).abs() < 1e-9);
    }

    // Worked example: "A 10-week procurement delay costs 200 × 50 × 10 =
    // £100,000 in avoidable waste."
    #[test]
    fn ten_week_procurement_delay_costs_100k() {
        let cod = operational_cost_of_delay(200.0, 50.0);
        assert!((total_delay_loss(cod, 10.0) - 100_000.0).abs() < 1e-9);
    }

    // Worked example: "QALY gain per patient = (5/52) × 0.12 ≈ 0.0115".
    #[test]
    fn qaly_gain_per_patient_is_about_0_0115() {
        // (5/52) × 0.12 ≈ 0.0115
        let gain = qaly_gain_per_patient(5.0, 0.80 - 0.68);
        assert!((gain - 0.0115).abs() < 1e-4);
    }

    // Worked example: "CoD_health = 100 × 0.0115 = 1.15 QALYs/week".
    #[test]
    fn clinical_cod_is_about_1_15_qalys_per_week() {
        let gain = qaly_gain_per_patient(5.0, 0.12);
        let cod_health = cost_of_delay_health(100.0, gain);
        assert!((cod_health - 1.15).abs() < 5e-3);
    }

    // Worked example: "CoD_money = 1.15 × £20,000 ≈ £23,000/week of health value".
    #[test]
    fn clinical_cod_money_is_about_23k_per_week() {
        let gain = qaly_gain_per_patient(5.0, 0.12);
        let cod_health = cost_of_delay_health(100.0, gain);
        let cod_money = cost_of_delay_money(cod_health, 20_000.0, 0.0);
        assert!((cod_money - 23_000.0).abs() < 100.0);
    }

    // Worked example: "A 6-month deployment delay 'costs' ~30 QALYs".
    #[test]
    fn six_month_deployment_delay_costs_about_30_qalys() {
        // 26 weeks × 1.1538 QALYs/week ≈ 30 QALYs
        let gain = qaly_gain_per_patient(5.0, 0.12);
        let cod_health = cost_of_delay_health(100.0, gain);
        let qalys_lost = total_delay_loss(cod_health, 26.0);
        assert!((qalys_lost - 30.0).abs() < 1e-9);
    }
}
