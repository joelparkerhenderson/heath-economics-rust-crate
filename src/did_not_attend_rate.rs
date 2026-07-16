//! # Did-Not-Attend (DNA) Rate
//!
//! The DNA rate is the percentage of booked appointments where the patient
//! neither attends nor cancels. The clinician, the room, and the slot are
//! paid for; nothing happens. It is the purest waste metric in healthcare —
//! and one of the most software-fixable (reminders, one-tap rebooking,
//! waiting-list backfill).
//!
//! ## Formula
//!
//! ```text
//! DNA rate = DNAs / booked appointments × 100
//!
//! Value of reduction = appointments × ΔDNA rate × value per recovered slot
//!
//! DNAs                    — appointments neither attended nor cancelled
//! ΔDNA rate               — percentage-point reduction / 100
//! value per recovered slot — depends on mechanism: refilled (activity value /
//!                            waiting-list reduction) or not (staff time only
//!                            partially reusable)
//! ```
//!
//! ## Why it matters
//!
//! NHS England's figures (2019): missed GP appointments exceed 15 million/year
//! at ~£30 each — over **£216M/year** — and hospital outpatient DNAs run
//! ~8M/year (~6.4% of appointments) at an average ~**£160** per missed slot.
//! Because the marginal cost of a reminder is pennies and the recovered value
//! is a fully-staffed clinical slot, DNA reduction has some of the best ROI
//! arithmetic in digital health — which is why SMS reminders, easy rebooking,
//! and predictive overbooking were among the first proven digital health wins.
//!
//! ## Example
//!
//! The topic doc's worked example: an outpatient department with 200,000
//! appointments/year at an 8% DNA rate deploys a reminder-plus-rebooking
//! service that cuts DNAs to 5.5%. That recovers 5,000 slots/year, worth
//! £800,000/year refilled at ~£160 each, against an £80,000/year service cost
//! — a ~10:1 return.
//!
//! ```rust
//! use health_economics::did_not_attend_rate::{
//!     dna_rate_percent, recovered_slots, value_of_reduction, service_cost,
//!     return_ratio, relative_reduction,
//! };
//!
//! // Baseline: 16,000 DNAs of 200,000 booked = 8%.
//! assert_eq!(dna_rate_percent(16_000.0, 200_000.0), Some(8.0));
//!
//! // Cutting 8% → 5.5% recovers 200,000 × 0.025 = 5,000 slots/year.
//! let slots = recovered_slots(200_000.0, 8.0 - 5.5);
//! assert_eq!(slots, 5_000.0);
//!
//! // Refilled from the waiting list at ~£160: £800,000/year recovered.
//! let value = value_of_reduction(slots, 160.0);
//! assert_eq!(value, 800_000.0);
//!
//! // Service cost: 200,000 × £0.40 = £80,000/year → return ≈ 10:1.
//! let cost = service_cost(200_000.0, 0.40);
//! assert_eq!(cost, 80_000.0);
//! assert_eq!(return_ratio(value, cost), Some(10.0));
//!
//! // Effect size: 2.5 points off 8% = 31.25% relative reduction,
//! // inside the 25–40% band reminder RCTs consistently show.
//! let rel = relative_reduction(8.0, 5.5).unwrap();
//! assert!((rel - 0.3125).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - **This is a scheduling-systems problem**: reminders, self-service
//!   rebooking, waiting-list auto-backfill from cancellations, and no-show
//!   prediction models that drive targeted double-booking — ordinary software
//!   engineering with an unusually crisp economic case.
//! - **The engineering analogue**: no-shows for reserved capacity —
//!   booked-but-idle CI slots, reserved cloud capacity, meeting rooms,
//!   interview panels. A cheap automated nudge (or auto-release of unused
//!   reservations) recovers expensive committed capacity.
//! - **Prediction ethics**: no-show models trained on attendance data encode
//!   deprivation and access barriers; using them to *deprioritize* likely
//!   non-attenders amplifies inequity, using them to *support* attendance
//!   (transport help, telephone alternatives) reduces it.
//!
//! ## Pitfalls
//!
//! - **Counting cancelled-and-rebooked as recovered value twice.**
//! - **Valuing recovered slots that aren't refilled** — an empty slot with a
//!   reminder sent is still empty.
//! - **Chasing DNA to zero**: the last points of DNA are patients facing real
//!   barriers; punitive approaches (discharge after N DNAs) cut the metric by
//!   abandoning the patients.
//!
//! ## Sources
//!
//! - NHS England, "Missed GP appointments costing NHS millions" (2019).
//!   <https://www.england.nhs.uk/2019/01/missed-gp-appointments-costing-nhs-millions/>
//! - DNA cost summaries. <https://www.deep-medical.ai/cost-of-missed-nhs-appointments/>
//!
//! Topic doc: health-economics-metrics/topics/did-not-attend-rate.md

/// DNA rate as a percentage: DNAs / booked appointments × 100.
///
/// A DNA is an appointment the patient neither attends nor cancels; cancelled
/// (and rebookable) appointments are not DNAs.
///
/// # Arguments
///
/// * `dnas` — count of missed (not attended, not cancelled) appointments.
/// * `booked_appointments` — total booked appointments in the same period.
///
/// # Returns
///
/// `Some(rate)` as a percentage (e.g. 8.0 for 8%), or `None` when
/// `booked_appointments` is zero (rate undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::dna_rate_percent;
///
/// // 16,000 DNAs of 200,000 booked = 8%.
/// assert_eq!(dna_rate_percent(16_000.0, 200_000.0), Some(8.0));
/// assert_eq!(dna_rate_percent(0.0, 0.0), None);
/// ```
pub fn dna_rate_percent(dnas: f64, booked_appointments: f64) -> Option<f64> {
    if booked_appointments == 0.0 {
        None
    } else {
        Some(dnas / booked_appointments * 100.0)
    }
}

/// Slots recovered per year by cutting the DNA rate.
///
/// Multiplies annual appointment volume by the DNA-rate reduction expressed
/// in percentage points (divided by 100 to get a fraction).
///
/// # Arguments
///
/// * `appointments` — booked appointments per year.
/// * `dna_rate_reduction_percentage_points` — percentage-point reduction
///   (e.g. 2.5 for a cut from 8% to 5.5%).
///
/// # Returns
///
/// Recovered slots per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::recovered_slots;
///
/// // 200,000 × 0.025 = 5,000 slots/year.
/// assert_eq!(recovered_slots(200_000.0, 2.5), 5_000.0);
/// ```
pub fn recovered_slots(appointments: f64, dna_rate_reduction_percentage_points: f64) -> f64 {
    // Percentage points / 100 converts to the ΔDNA-rate fraction.
    appointments * dna_rate_reduction_percentage_points / 100.0
}

/// Value of the DNA reduction, assuming recovered slots are refilled.
///
/// Mechanism matters: value at the per-slot activity rate only when the slot
/// is genuinely refilled (e.g. from the waiting list); an empty slot with a
/// reminder sent is still empty.
///
/// # Arguments
///
/// * `recovered_slots` — slots recovered per year (see [`recovered_slots`]).
/// * `value_per_recovered_slot` — £ per refilled slot (~£160 average
///   outpatient value).
///
/// # Returns
///
/// Recovered activity value, £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::value_of_reduction;
///
/// // 5,000 slots × £160 = £800,000/year of recovered activity.
/// assert_eq!(value_of_reduction(5_000.0, 160.0), 800_000.0);
/// ```
pub fn value_of_reduction(recovered_slots: f64, value_per_recovered_slot: f64) -> f64 {
    recovered_slots * value_per_recovered_slot
}

/// Annual cost of the reminder/rebooking service.
///
/// Assumes the service is priced per appointment messaged (SMS with one-tap
/// rebook, transport info, accessible formats).
///
/// # Arguments
///
/// * `appointments` — appointments messaged per year.
/// * `cost_per_appointment` — £ per appointment messaged (pennies for SMS).
///
/// # Returns
///
/// Service cost, £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::service_cost;
///
/// // 200,000 × £0.40 = £80,000/year.
/// assert_eq!(service_cost(200_000.0, 0.40), 80_000.0);
/// ```
pub fn service_cost(appointments: f64, cost_per_appointment: f64) -> f64 {
    appointments * cost_per_appointment
}

/// Return ratio of the service: recovered value / service cost.
///
/// A ratio of 10.0 means £10 recovered per £1 spent ("10:1").
///
/// # Arguments
///
/// * `recovered_value` — £/year of recovered activity (see
///   [`value_of_reduction`]).
/// * `service_cost` — £/year the service costs (see [`service_cost`]).
///
/// # Returns
///
/// `Some(ratio)`, or `None` when `service_cost` is zero (ratio undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::return_ratio;
///
/// // £800,000 recovered / £80,000 cost = 10:1.
/// assert_eq!(return_ratio(800_000.0, 80_000.0), Some(10.0));
/// assert_eq!(return_ratio(800_000.0, 0.0), None);
/// ```
pub fn return_ratio(recovered_value: f64, service_cost: f64) -> Option<f64> {
    if service_cost == 0.0 {
        None
    } else {
        Some(recovered_value / service_cost)
    }
}

/// Relative DNA reduction as a fraction: (before − after) / before.
///
/// Use this to sanity-check an assumed effect size: reminder RCTs
/// consistently show 25–40% relative DNA reduction.
///
/// # Arguments
///
/// * `dna_rate_before` — baseline DNA rate (any consistent unit, e.g. percent).
/// * `dna_rate_after` — DNA rate after the intervention (same unit).
///
/// # Returns
///
/// `Some(fraction)` (e.g. 0.3125 for a 31.25% relative reduction), or `None`
/// when the baseline rate is zero (relative change undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::did_not_attend_rate::relative_reduction;
///
/// // 8% → 5.5% is a 31.25% relative reduction, inside the 25–40% RCT band.
/// let rel = relative_reduction(8.0, 5.5).unwrap();
/// assert!((rel - 0.3125).abs() < 1e-9);
/// assert!((0.25..=0.40).contains(&rel));
/// ```
pub fn relative_reduction(dna_rate_before: f64, dna_rate_after: f64) -> Option<f64> {
    if dna_rate_before == 0.0 {
        None
    } else {
        Some((dna_rate_before - dna_rate_after) / dna_rate_before)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Formula check: "DNA rate = DNAs / booked appointments × 100" at the
    // worked example's 8% baseline (200,000 appointments/year).
    #[test]
    fn dna_rate_is_dnas_over_booked_times_100() {
        // 16,000 DNAs of 200,000 booked = 8%
        assert!((dna_rate_percent(16_000.0, 200_000.0).unwrap() - 8.0).abs() < 1e-9);
    }

    // Edge case: rate undefined with zero bookings.
    #[test]
    fn dna_rate_is_none_with_no_bookings() {
        assert!(dna_rate_percent(0.0, 0.0).is_none());
    }

    // Worked example: "Recovered slots = 200,000 × 0.025 = 5,000/year".
    #[test]
    fn cutting_8_to_5_5_percent_recovers_5000_slots() {
        // 200,000 × 0.025 = 5,000/year
        let slots = recovered_slots(200_000.0, 8.0 - 5.5);
        assert!((slots - 5_000.0).abs() < 1e-9);
    }

    // Worked example: "5,000 × £160 = £800,000/year of recovered activity".
    #[test]
    fn refilled_slots_at_160_pounds_are_worth_800k() {
        let slots = recovered_slots(200_000.0, 2.5);
        assert!((value_of_reduction(slots, 160.0) - 800_000.0).abs() < 1e-9);
    }

    // Worked example: "Service cost: 200,000 × £0.40 = £80,000/year".
    #[test]
    fn service_cost_is_80k_per_year() {
        // 200,000 × £0.40 = £80,000/year
        assert!((service_cost(200_000.0, 0.40) - 80_000.0).abs() < 1e-9);
    }

    // Worked example: "Return ≈ 10:1".
    #[test]
    fn return_is_about_10_to_1() {
        let value = value_of_reduction(recovered_slots(200_000.0, 2.5), 160.0);
        let cost = service_cost(200_000.0, 0.40);
        assert!((return_ratio(value, cost).unwrap() - 10.0).abs() < 1e-9);
    }

    // Worked example: "The effect size (2.5 points) is realistic: reminder
    // RCTs consistently show 25–40% relative DNA reduction."
    #[test]
    fn effect_size_is_within_realistic_relative_reduction_band() {
        // 2.5 points off an 8% base = 31.25% relative, inside the 25–40% RCT band
        let rel = relative_reduction(8.0, 5.5).unwrap();
        assert!((rel - 0.3125).abs() < 1e-9);
        assert!((0.25..=0.40).contains(&rel));
    }
}
