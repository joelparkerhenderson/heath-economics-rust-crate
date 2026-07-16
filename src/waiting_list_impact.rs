//! # Waiting List Impact
//!
//! Waiting list impact converts saved clinical capacity into patients removed
//! from (or moved faster through) the waiting list — extra appointment slots,
//! patients actually seen after DNA (did-not-attend) losses, net list
//! reduction after induced demand, and the queueing pull-forward of waits.
//!
//! Converting saved hours into extra clinic slots directly reduces the size
//! of a trust's waiting list — the most tangible way to show a health system
//! what freed capacity is *for*, and the natural unit of account for
//! non-cash-releasing capacity.
//!
//! ## Formula
//!
//! ```text
//! Extra slots        = hours released / slot duration × utilization
//! Patients seen      = extra slots × (1 − DNA rate)
//! List reduction     = patients seen − induced new demand
//! Waiting-time gain  ≈ ΔN / μ
//!
//! where:
//!   hours released — staff × hours saved per day × working days per year
//!   slot duration  — appointment length in hours (e.g. 0.5 for 30 minutes)
//!   utilization    — usable fraction of released time (0..1)
//!   DNA rate       — did-not-attend fraction (0..1)
//!   ΔN             — backlog reduction (patients)
//!   μ              — service rate (patients per time unit); cutting the
//!                    backlog by ΔN pulls everyone forward ~ΔN/μ
//! ```
//!
//! ## Why it matters
//!
//! The elective waiting list is the NHS's defining post-pandemic challenge
//! (its size is a national political metric), and every trust runs an
//! elective-recovery program against it. A business case that says "saves
//! 2,000 nurse-hours" is abstract; one that says "creates 4,000 additional
//! appointment slots, seeing 3,800 waiting patients, cutting the specialty's
//! list by 9%" is a story a Chief Operating Officer can take to their board.
//! In the worked example, ~5,900 extra appointments against 24,000/year of
//! demand-matched capacity cut average waits by roughly a quarter — moving
//! the trust materially toward the 18-week standard without hiring — and at
//! ~£160 scheme value per attendance the activity is worth ~£949,000/year.
//!
//! ## Example
//!
//! Ambient documentation software saves each of 20 clinic nurses 45 min/day
//! over 250 working days.
//!
//! ```rust
//! use health_economics::waiting_list_impact::{
//!     hours_released, extra_slots, patients_seen, wait_reduction_fraction,
//!     activity_value,
//! };
//!
//! // 20 × 0.75 × 250 = 3,750 hours/year
//! let hours = hours_released(20.0, 0.75, 250.0);
//! assert_eq!(hours, 3_750.0);
//!
//! // Slots (30 min, 85% usable) = 3,750 / 0.5 × 0.85 = 6,375 slots
//! let slots = extra_slots(hours, 0.5, 0.85).unwrap();
//! assert!((slots - 6_375.0).abs() < 1e-9);
//!
//! // Patients seen (7% DNA) = 6,375 × 0.93 ≈ 5,929/year (exact 5,928.75)
//! let seen = patients_seen(slots, 0.07);
//! assert!((seen - 5_929.0).abs() < 0.5);
//!
//! // ~5,900 extra appointments / 24,000 capacity ≈ waits cut by a quarter
//! let fraction = wait_reduction_fraction(seen, 24_000.0).unwrap();
//! assert!((fraction - 0.25).abs() < 0.005);
//!
//! // At ~£160 per attendance the activity is worth ~£949,000/year
//! let value = activity_value(seen, 160.0);
//! assert!((value - 949_000.0).abs() < 500.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - A waiting list is a backlog, and backlog-burndown economics transfer in
//!   both directions.
//! - From health to software: value backlog reduction by how long *users*
//!   wait for value (cost of delay per queued item), not by items closed.
//! - From software to health: Little's Law says the list shrinks only if
//!   service rate exceeds arrival rate — capacity gains absorbed by rising
//!   referrals leave waits unchanged, so model arrivals too.
//! - In both domains, prioritize by severity-weighted value (clinical
//!   urgency categories ↔ severity modifiers), not first-in-first-out.
//!
//! ## Pitfalls
//!
//! - **Slots ≠ patients**: forgetting DNA rates and unusable fragments of
//!   released time.
//! - **Induced demand**: visible extra capacity attracts referrals; net list
//!   impact is smaller than gross.
//! - **Claiming cash**: waiting-list impact is capacity value; the cash
//!   claim (avoided outsourcing of backlog work) is a different line.
//!
//! ## Sources
//!
//! - NHS England, RTT waiting times statistics.
//!   <https://www.england.nhs.uk/statistics/statistical-work-areas/rtt-waiting-times/>
//! - NHS England, elective care recovery plan.
//!   <https://www.england.nhs.uk/coronavirus/publication/delivery-plan-for-tackling-the-covid-19-backlog-of-elective-care/>
//!
//! Topic doc: health-economics-metrics/topics/waiting-list-impact.md

/// Total clinical hours released per year by a time-saving intervention.
///
/// Computes staff × hours saved per day × working days per year.
///
/// # Arguments
///
/// * `staff_count` — number of staff saving time (headcount).
/// * `hours_saved_per_day` — hours saved per staff member per working day
///   (e.g. 0.75 for 45 minutes).
/// * `working_days_per_year` — working days per year (typically ~250).
///
/// # Returns
///
/// Released clinical hours per year (hours/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::hours_released;
///
/// // 20 nurses × 0.75 h/day × 250 days = 3,750 hours/year.
/// let hours = hours_released(20.0, 0.75, 250.0);
/// assert_eq!(hours, 3_750.0);
/// ```
pub fn hours_released(
    staff_count: f64,
    hours_saved_per_day: f64,
    working_days_per_year: f64,
) -> f64 {
    staff_count * hours_saved_per_day * working_days_per_year
}

/// Extra appointment slots created from released hours.
///
/// Divides released hours by the slot duration and applies a utilization
/// factor — the usable fraction of released time, since freed minutes come
/// in fragments that don't all convert into bookable slots.
///
/// # Arguments
///
/// * `hours_released` — total released clinical hours per year.
/// * `slot_duration_hours` — appointment slot length in hours (e.g. 0.5 for
///   a 30-minute slot).
/// * `utilization` — usable fraction of released time (0..1, e.g. 0.85).
///
/// # Returns
///
/// `Some(slots)` — extra appointment slots per year; `None` when
/// `slot_duration_hours` is zero (the division is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::extra_slots;
///
/// // 3,750 hours / 0.5 h-slots × 0.85 usable = 6,375 slots.
/// let slots = extra_slots(3_750.0, 0.5, 0.85).unwrap();
/// assert!((slots - 6_375.0).abs() < 1e-9);
///
/// // Zero slot duration has no defined slot count.
/// assert!(extra_slots(3_750.0, 0.0, 0.85).is_none());
/// ```
pub fn extra_slots(
    hours_released: f64,
    slot_duration_hours: f64,
    utilization: f64,
) -> Option<f64> {
    if slot_duration_hours == 0.0 {
        None
    } else {
        Some(hours_released / slot_duration_hours * utilization)
    }
}

/// Patients actually seen once DNA (did-not-attend) losses are applied.
///
/// Slots are not patients: a fraction of booked appointments are missed.
/// Computes slots × (1 − DNA rate).
///
/// # Arguments
///
/// * `extra_slots` — extra appointment slots per year.
/// * `dna_rate` — did-not-attend fraction (0..1, e.g. 0.07 for 7%).
///
/// # Returns
///
/// Patients seen per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::patients_seen;
///
/// // 6,375 slots × (1 − 0.07) = 5,928.75 ≈ 5,929 patients seen/year.
/// let seen = patients_seen(6_375.0, 0.07);
/// assert!((seen - 5_929.0).abs() < 0.5);
/// ```
pub fn patients_seen(extra_slots: f64, dna_rate: f64) -> f64 {
    extra_slots * (1.0 - dna_rate)
}

/// Net waiting-list reduction after induced demand.
///
/// Visible extra capacity attracts referrals, so the net list impact is
/// smaller than the gross patients-seen figure. Computes patients seen minus
/// induced new demand.
///
/// # Arguments
///
/// * `patients_seen` — patients seen per year via the extra capacity.
/// * `induced_new_demand` — extra referrals per year attracted by the
///   visible additional capacity.
///
/// # Returns
///
/// Net list reduction (patients/year); can be negative if induced demand
/// exceeds the capacity gain.
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::list_reduction;
///
/// // 5,929 patients seen, 1,000 induced referrals → net 4,929 off the list.
/// let net = list_reduction(5_929.0, 1_000.0);
/// assert_eq!(net, 4_929.0);
/// ```
pub fn list_reduction(patients_seen: f64, induced_new_demand: f64) -> f64 {
    patients_seen - induced_new_demand
}

/// Waiting-time gain for a stable queue from a backlog cut.
///
/// For a stable queue served at rate μ, removing ΔN patients from the
/// backlog pulls everyone forward by roughly ΔN/μ (the time the service
/// would have spent processing them).
///
/// # Arguments
///
/// * `backlog_reduction` — ΔN, patients removed from the backlog.
/// * `service_rate` — μ, patients served per time unit (the returned gain is
///   in the same time unit).
///
/// # Returns
///
/// `Some(gain)` — waiting-time improvement ≈ ΔN/μ, in the time unit of
/// `service_rate`; `None` when `service_rate` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::waiting_time_gain;
///
/// // Removing 5,929 patients at a service rate of 24,000/year pulls waits
/// // forward by ≈ 0.247 years (~a quarter of a year).
/// let gain = waiting_time_gain(5_929.0, 24_000.0).unwrap();
/// assert!((gain - 0.247).abs() < 0.001);
///
/// // A zero service rate has no defined gain.
/// assert!(waiting_time_gain(100.0, 0.0).is_none());
/// ```
pub fn waiting_time_gain(backlog_reduction: f64, service_rate: f64) -> Option<f64> {
    if service_rate == 0.0 {
        None
    } else {
        Some(backlog_reduction / service_rate)
    }
}

/// Fraction by which average waits fall when extra appointments are added.
///
/// Relates the extra appointments to the specialty's demand-matched annual
/// appointment capacity: extra / capacity.
///
/// # Arguments
///
/// * `extra_appointments` — additional appointments per year.
/// * `annual_appointment_capacity` — the specialty's demand-matched capacity
///   (appointments/year).
///
/// # Returns
///
/// `Some(fraction)` — proportional reduction in average waits (0.25 = waits
/// cut by a quarter); `None` when `annual_appointment_capacity` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::wait_reduction_fraction;
///
/// // ~5,929 extra appointments against 24,000/year capacity cut average
/// // waits by roughly a quarter (exact ≈ 0.2470).
/// let fraction = wait_reduction_fraction(5_928.75, 24_000.0).unwrap();
/// assert!((fraction - 0.25).abs() < 0.005);
/// ```
pub fn wait_reduction_fraction(
    extra_appointments: f64,
    annual_appointment_capacity: f64,
) -> Option<f64> {
    if annual_appointment_capacity == 0.0 {
        None
    } else {
        Some(extra_appointments / annual_appointment_capacity)
    }
}

/// Tariff (scheme) value of the extra activity.
///
/// Multiplies patients seen by the scheme value per attendance. Present the
/// waiting-list framing first; this line is capacity value, not cash —
/// under blended payment extra activity may not bring extra income.
///
/// # Arguments
///
/// * `patients_seen` — patients seen per year via the extra capacity.
/// * `scheme_value_per_attendance` — tariff / payment-scheme value per
///   attendance (£, e.g. ~£160).
///
/// # Returns
///
/// Annual activity value (£/year), a non-cash-releasing figure.
///
/// # Examples
///
/// ```rust
/// use health_economics::waiting_list_impact::activity_value;
///
/// // 5,928.75 patients × £160 = £948,600 ≈ £949,000/year.
/// let value = activity_value(5_928.75, 160.0);
/// assert!((value - 949_000.0).abs() < 500.0);
/// ```
pub fn activity_value(patients_seen: f64, scheme_value_per_attendance: f64) -> f64 {
    patients_seen * scheme_value_per_attendance
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "20 × 0.75 × 250 = 3,750 hours/year".
    /// 20 nurses × 0.75 h/day × 250 days = 3,750 hours/year.
    #[test]
    fn hours_released_is_3_750_per_year() {
        let hours = hours_released(20.0, 0.75, 250.0);
        assert!((hours - 3_750.0).abs() < 1e-9);
    }

    // Worked example: "Slots (30 min, 85% usable) = 3,750 / 0.5 × 0.85
    // = 6,375 slots".
    /// 3,750 / 0.5 × 0.85 = 6,375 slots (30-minute slots, 85% usable).
    #[test]
    fn extra_slots_is_6_375() {
        let slots = extra_slots(3_750.0, 0.5, 0.85).unwrap();
        assert!((slots - 6_375.0).abs() < 1e-9);
    }

    // Worked example: "Patients seen (7% DNA) = 6,375 × 0.93 ≈ 5,929/year".
    /// 6,375 × 0.93 ≈ 5,929 patients seen/year with a 7% DNA rate.
    #[test]
    fn patients_seen_is_about_5_929() {
        let seen = patients_seen(6_375.0, 0.07);
        assert!((seen - 5_929.0).abs() < 0.5); // exact value 5,928.75
    }

    // Worked example: "~5,900 additional appointments cut average waits by
    // roughly a quarter" against 24,000 appointments/year of capacity.
    /// ~5,900 extra appointments against 24,000/year capacity cut average
    /// waits by roughly a quarter.
    #[test]
    fn wait_reduction_is_roughly_a_quarter() {
        let seen = patients_seen(6_375.0, 0.07);
        let fraction = wait_reduction_fraction(seen, 24_000.0).unwrap();
        assert!((fraction - 0.25).abs() < 0.005); // exact value ≈ 0.2470
    }

    // Worked example: "At ~£160 scheme value per attendance the activity is
    // worth ~£949,000/year".
    /// At ~£160 per attendance the activity is worth ~£949,000/year.
    #[test]
    fn activity_value_is_about_949_000() {
        let seen = patients_seen(6_375.0, 0.07);
        let value = activity_value(seen, 160.0);
        assert!((value - 949_000.0).abs() < 500.0); // exact value £948,600
    }

    // Edge-case contract: zero slot duration, service rate, or capacity
    // yield None rather than a division blow-up.
    /// Zero denominators are reported as None, not a crash or infinity.
    #[test]
    fn zero_denominators_return_none() {
        assert!(extra_slots(100.0, 0.0, 0.85).is_none());
        assert!(waiting_time_gain(100.0, 0.0).is_none());
        assert!(wait_reduction_fraction(100.0, 0.0).is_none());
    }
}
