//! # Value-Generating Capacity (Operational Turnaround)
//!
//! Value-generating capacity is the "opportunity benefit" of freed clinical
//! time: what the hospital can now *achieve* with the hours software
//! releases, expressed as the extra value-generating activity (clinics,
//! assessments, monitoring reviews) the released hours enable, valued at
//! national tariff / NHS Payment Scheme prices.
//!
//! This is the output-basis valuation of practitioner time, scaled to a
//! service line and expressed in the activity units operations teams already
//! plan in — the metric that matters most to Chief Operating Officers and
//! Medical Directors, because it speaks in the currency they are managed on:
//! activity, targets, and turnaround.
//!
//! ## Formula
//!
//! ```text
//! Hidden capacity created = time released → activity units enabled × scheme value
//!
//! Extra activity units = staff × units enabled per staff per day × working days
//! Capacity value       = extra activity units × scheme value per unit
//!
//! where:
//!   staff                     — headcount whose time is released
//!   units per staff per day   — extra activity units (e.g. pre-op
//!                               assessments) each person can now fit per day
//!   working days              — working days per year (typically ~250)
//!   scheme value per unit     — national tariff / NHS Payment Scheme price
//!                               per activity unit (£)
//! ```
//!
//! ## Why it matters
//!
//! The NHS faces massive referral-to-treatment backlogs, and trusts that miss
//! national waiting-time standards face regulatory scrutiny and intervention.
//! Hiring is slow and constrained; estates are fixed. The only fast lever is
//! getting more value-generating activity out of existing staff and space.
//! Software that reclaims specialist time doesn't just "save money" — it
//! *mints capacity*: clinics that couldn't exist, assessments that couldn't
//! be scheduled, without hiring or building. In the worked example, 25 Band 6
//! specialist nurses each reclaiming 1 hour/day (2 assessments) yields 12,500
//! extra pre-op assessments/year, worth £1.5M/year at a ~£120 scheme value.
//!
//! ## Example
//!
//! Documentation automation reclaims 1 hour/day for each of 25 Band 6
//! specialist nurses running pre-operative assessment clinics; each hour fits
//! 2 assessments.
//!
//! ```rust
//! use health_economics::value_generating_capacity_operational_turnaround::{
//!     extra_activity_units, capacity_value, annual_capacity_value,
//! };
//!
//! // Extra assessments = 25 nurses × 2/day × 250 days = 12,500/year
//! let units = extra_activity_units(25.0, 2.0, 250.0);
//! assert_eq!(units, 12_500.0);
//!
//! // At ~£120 scheme value per pre-op assessment: 12,500 × £120 = £1.5M/year
//! let value = capacity_value(units, 120.0);
//! assert_eq!(value, 1_500_000.0);
//!
//! // Composed end-to-end helper gives the same £1.5M/year figure.
//! assert_eq!(annual_capacity_value(25.0, 2.0, 250.0, 120.0), 1_500_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - The same reframing rescues developer-productivity claims from wage math:
//!   released engineering time, expressed as *shipped capability the org
//!   couldn't otherwise afford* — features, migrations, reliability work.
//! - Value that capability at what the org pays for it at the margin:
//!   contractor rates, or deferred-hire equivalents.
//! - Express the benefit in the units the audience is managed on: ops leaders
//!   think in activity and targets, engineering leaders in roadmap items and
//!   headcount — neither thinks in abstract minutes saved.
//!
//! ## Pitfalls
//!
//! - **Capacity claims without demand**: 12,500 extra assessment slots only
//!   matter if the surgical pipeline fills them — check the downstream
//!   constraint.
//! - **Tariff value without a payment mechanism**: under blended payment,
//!   extra activity may not bring extra income; the value may be
//!   waiting-list reduction instead.
//! - **Presenting capacity as cash** — this is the flagship
//!   non-cash-releasing benefit; label it as such.
//!
//! ## Sources
//!
//! - NHS England, NHS Payment Scheme.
//!   <https://www.england.nhs.uk/pay-syst/national-tariff/national-tariff-payment-system/>
//! - NHS England, elective care recovery plan.
//!   <https://www.england.nhs.uk/coronavirus/publication/delivery-plan-for-tackling-the-covid-19-backlog-of-elective-care/>
//!
//! Topic doc: health-economics-metrics/topics/value-generating-capacity-operational-turnaround.md

/// Extra activity units enabled per year by released staff time.
///
/// Computes the additional activity units (e.g. pre-op assessments,
/// outpatient attendances, monitoring reviews) enabled when each member of
/// staff gains capacity for `units_per_staff_per_day` additional units on
/// each of `working_days_per_year` working days.
///
/// # Arguments
///
/// * `staff_count` — number of staff whose time is released (headcount).
/// * `units_per_staff_per_day` — extra activity units each staff member can
///   now fit per working day.
/// * `working_days_per_year` — working days per year (typically ~250).
///
/// # Returns
///
/// Extra activity units per year (units/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::value_generating_capacity_operational_turnaround::extra_activity_units;
///
/// // 25 nurses × 2 assessments/day × 250 days = 12,500 extra assessments/year.
/// let units = extra_activity_units(25.0, 2.0, 250.0);
/// assert_eq!(units, 12_500.0);
/// ```
pub fn extra_activity_units(
    staff_count: f64,
    units_per_staff_per_day: f64,
    working_days_per_year: f64,
) -> f64 {
    staff_count * units_per_staff_per_day * working_days_per_year
}

/// Value of created capacity at scheme (tariff) prices.
///
/// Multiplies extra activity units by the national tariff / NHS Payment
/// Scheme value per unit. This is a **non-cash-releasing capacity value**,
/// not cash income — under blended payment the extra activity may not bring
/// extra revenue.
///
/// # Arguments
///
/// * `extra_activity_units` — extra activity units per year (from
///   [`extra_activity_units`]).
/// * `scheme_value_per_unit` — tariff / payment-scheme value per activity
///   unit (£).
///
/// # Returns
///
/// Annual capacity value (£/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::value_generating_capacity_operational_turnaround::capacity_value;
///
/// // 12,500 assessments × £120 = £1.5M/year of care capacity created.
/// let value = capacity_value(12_500.0, 120.0);
/// assert_eq!(value, 1_500_000.0);
/// ```
pub fn capacity_value(extra_activity_units: f64, scheme_value_per_unit: f64) -> f64 {
    extra_activity_units * scheme_value_per_unit
}

/// Annual capacity value created from released time, composed end-to-end.
///
/// Convenience composition equivalent to
/// `capacity_value(extra_activity_units(staff, units, days), scheme_value)`.
///
/// # Arguments
///
/// * `staff_count` — number of staff whose time is released (headcount).
/// * `units_per_staff_per_day` — extra activity units each staff member can
///   now fit per working day.
/// * `working_days_per_year` — working days per year (typically ~250).
/// * `scheme_value_per_unit` — tariff / payment-scheme value per activity
///   unit (£).
///
/// # Returns
///
/// Annual capacity value (£/year), a non-cash-releasing benefit.
///
/// # Examples
///
/// ```rust
/// use health_economics::value_generating_capacity_operational_turnaround::annual_capacity_value;
///
/// // 25 nurses × 2/day × 250 days × £120 = £1.5M/year — without hiring a
/// // single nurse or building a single room.
/// let value = annual_capacity_value(25.0, 2.0, 250.0, 120.0);
/// assert_eq!(value, 1_500_000.0);
/// ```
pub fn annual_capacity_value(
    staff_count: f64,
    units_per_staff_per_day: f64,
    working_days_per_year: f64,
    scheme_value_per_unit: f64,
) -> f64 {
    capacity_value(
        extra_activity_units(staff_count, units_per_staff_per_day, working_days_per_year),
        scheme_value_per_unit,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "Extra assessments = 25 nurses × 2/day × 250 days
    // = 12,500/year".
    /// 25 nurses × 2 assessments/day × 250 days = 12,500 extra assessments/year.
    #[test]
    fn extra_assessments_per_year_is_12_500() {
        let units = extra_activity_units(25.0, 2.0, 250.0);
        assert!((units - 12_500.0).abs() < 1e-9);
    }

    // Worked example: "12,500 × £120 = £1.5M/year of care capacity created".
    /// 12,500 × £120 = £1.5M/year of care capacity created.
    #[test]
    fn capacity_value_is_1_5_million_per_year() {
        let value = capacity_value(12_500.0, 120.0);
        assert!((value - 1_500_000.0).abs() < 1e-9);
    }

    // Composes the two worked-example lines end-to-end to the same £1.5M/year.
    /// The composed helper reproduces the same £1.5M figure end-to-end.
    #[test]
    fn annual_capacity_value_composes_to_1_5_million() {
        let value = annual_capacity_value(25.0, 2.0, 250.0, 120.0);
        assert!((value - 1_500_000.0).abs() < 1e-9);
    }
}
