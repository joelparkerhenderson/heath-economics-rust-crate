//! # Bed Days Saved
//!
//! A bed day is one patient occupying one hospital bed for one day. "Bed days
//! saved" — through earlier discharge, admission avoidance, or virtual wards
//! — is the workhorse benefit of NHS digital business cases, and the most
//! commonly overvalued: the *kind* of value depends entirely on what happens
//! to the freed bed.
//!
//! Beds are the binding constraint of acute care: when beds fill, elective
//! surgery is cancelled, ambulances queue, and the emergency department backs
//! up. Finance directors have learned to discount naive bed-day claims
//! heavily; getting this arithmetic right is a credibility test.
//!
//! ## Formula
//!
//! ```text
//! Bed days saved = patients affected × Δ length of stay
//!                  (or admissions avoided × avg LOS)
//!
//! Value depends on the use of the freed capacity:
//!   refilled with elective activity → value = activity income or waiting-list benefit
//!   ward closed / flexed down      → value = staffing + running cost released (cash)
//!   absorbed as slack              → value ≈ marginal (hotel) cost only, £50–£150/day
//! ```
//!
//! Legend:
//! - `patients affected` — patients discharged earlier (count).
//! - `Δ length of stay` — reduction in length of stay per patient (days).
//! - `admissions avoided` × `avg LOS` — the admission-avoidance route to the
//!   same bed-day count.
//! - `activity income` — income per backfilling elective spell under
//!   activity-based payment (currency).
//! - `marginal (hotel) cost` — food, laundry, consumables per bed day
//!   (currency/day).
//!
//! ## Why it matters
//!
//! The average fully-absorbed cost of an acute bed day is often quoted at
//! £400+ (National Cost Collection historically ~£350 for excess bed days) —
//! but the average is almost never the saving. Freed capacity that is
//! *reused* is often worth more than the naive cash claim — but it is a
//! different kind of value (non-cash-releasing activity) and must be labeled
//! as such; capacity absorbed as slack is worth only its marginal hotel cost
//! of £50–£150/day.
//!
//! ## Example
//!
//! A remote-monitoring "virtual ward" lets 600 patients/year go home 2 days
//! early: 1,200 bed days saved. Naive claim: 1,200 × £400 = £480,000 — wrong
//! unless a ward closes. Honest claim: backfilling with elective orthopaedic
//! patients gives 1,200 ÷ 3-day average stay = 400 additional spells at
//! ~£6,000 income each = £2.4M of additional funded activity, against which
//! the virtual ward's £350,000 running cost nets.
//!
//! ```rust
//! use health_economics::bed_days_saved::{
//!     additional_elective_spells, bed_days_saved_from_earlier_discharge,
//!     freed_capacity_value, naive_bed_day_value, FreedCapacityUse,
//! };
//!
//! // 600 patients × 2 days = 1,200 bed days saved.
//! let bed_days = bed_days_saved_from_earlier_discharge(600.0, 2.0);
//! assert!((bed_days - 1_200.0).abs() < 1e-9);
//!
//! // Naive claim: 1,200 × £400 = £480,000. Wrong unless a ward closes.
//! let naive = naive_bed_day_value(bed_days, 400.0);
//! assert!((naive - 480_000.0).abs() < 1e-9);
//!
//! // Honest claim: 1,200 ÷ 3-day stay = 400 elective spells...
//! let spells = additional_elective_spells(bed_days, 3.0).unwrap();
//! assert!((spells - 400.0).abs() < 1e-9);
//!
//! // ...at ~£6,000 each = £2.4M of additional funded activity.
//! let refill = FreedCapacityUse::RefilledWithElective {
//!     average_elective_stay_days: 3.0,
//!     income_per_spell: 6_000.0,
//! };
//! let value = freed_capacity_value(bed_days, &refill).unwrap();
//! assert!((value - 2_400_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - "Server days saved" behaves identically: decommissioning always-on
//!   environments only releases cash when instances are terminated or
//!   reservations lapse.
//! - Capacity absorbed back into the pool is worth its marginal cost (~0 on
//!   committed spend).
//! - The parallel discipline: for every claimed saving, name the *mechanism*
//!   — terminated, refilled with valuable work, or evaporated.
//! - Software that reduces hospital length of stay (discharge coordination,
//!   remote monitoring, diagnostics turnaround) should model all three
//!   scenarios and let the trust pick per ward.
//!
//! ## Pitfalls
//!
//! - **Average-cost valuation** of marginal capacity — the canonical error.
//! - **Double counting**: bed days saved *and* admissions avoided *and*
//!   waiting list reduction from the same freed bed.
//! - **Assuming saved days are the expensive days**: the days saved at end
//!   of stay are the cheapest (low-acuity) days.
//!
//! ## Sources
//!
//! - NHS England, National Cost Collection.
//!   <https://www.england.nhs.uk/costing-in-the-nhs/national-cost-collection/>
//! - Economics by Design, NHS cost calculator.
//!   <https://economicsbydesign.com/tools/nhs-cost-calculator/>
//!
//! Topic doc: health-economics-metrics/topics/bed-days-saved.md

/// Bed days saved via shorter stays: patients affected × reduction in length of stay.
///
/// # Arguments
///
/// * `patients_affected` — patients discharged earlier per period (count).
/// * `length_of_stay_reduction_days` — days saved per patient (days).
///
/// # Returns
///
/// Bed days saved per period (bed-days).
///
/// # Examples
///
/// ```rust
/// use health_economics::bed_days_saved::bed_days_saved_from_earlier_discharge;
///
/// // Worked example: 600 patients/year go home 2 days early = 1,200 bed days.
/// let bed_days = bed_days_saved_from_earlier_discharge(600.0, 2.0);
/// assert!((bed_days - 1_200.0).abs() < 1e-9);
/// ```
pub fn bed_days_saved_from_earlier_discharge(
    patients_affected: f64,
    length_of_stay_reduction_days: f64,
) -> f64 {
    patients_affected * length_of_stay_reduction_days
}

/// Bed days saved via admission avoidance: admissions avoided × average length of stay.
///
/// The other route to the same bed-day count — never claim both routes for
/// the same freed bed (double counting).
///
/// # Arguments
///
/// * `admissions_avoided` — admissions avoided per period (count).
/// * `average_length_of_stay_days` — average stay each avoided admission
///   would have used (days).
///
/// # Returns
///
/// Bed days saved per period (bed-days).
///
/// # Examples
///
/// ```rust
/// use health_economics::bed_days_saved::bed_days_saved_from_avoided_admissions;
///
/// // 400 admissions avoided × 3-day average stay = 1,200 bed days.
/// let bed_days = bed_days_saved_from_avoided_admissions(400.0, 3.0);
/// assert!((bed_days - 1_200.0).abs() < 1e-9);
/// ```
pub fn bed_days_saved_from_avoided_admissions(
    admissions_avoided: f64,
    average_length_of_stay_days: f64,
) -> f64 {
    admissions_avoided * average_length_of_stay_days
}

/// What happens to the freed bed capacity — the question that determines what
/// kind of value the bed days carry.
///
/// Name the mechanism for every claimed saving: refilled, closed, or
/// absorbed. Each variant carries the parameters its valuation needs (see
/// [`freed_capacity_value`]).
pub enum FreedCapacityUse {
    /// Beds refilled with elective activity: value is activity income (or
    /// waiting-list benefit) per additional spell — non-cash-releasing
    /// funded activity, not cash.
    RefilledWithElective {
        /// Average length of stay of the backfilling elective spells (days).
        average_elective_stay_days: f64,
        /// Income per additional elective spell under activity-based payment
        /// (currency).
        income_per_spell: f64,
    },
    /// A ward closes or flexes down: staffing and running cost genuinely
    /// released (cash) per bed day.
    WardClosedOrFlexedDown {
        /// Cash released per bed day (staffing + running cost; currency/day).
        cost_released_per_bed_day: f64,
    },
    /// Capacity absorbed as slack: worth only the marginal (hotel) cost,
    /// typically £50–£150/day.
    AbsorbedAsSlack {
        /// Marginal hotel cost per bed day (food, laundry, consumables;
        /// currency/day).
        marginal_hotel_cost_per_bed_day: f64,
    },
}

/// Value of freed bed days under a declared mechanism.
///
/// Refilled: bed days ÷ average elective stay × income per spell (the value
/// is the funded activity the beds can now host). Ward closed: bed days ×
/// cash released per day. Absorbed as slack: bed days × marginal hotel cost.
///
/// # Arguments
///
/// * `bed_days_saved` — freed bed days (bed-days).
/// * `use_of_capacity` — the declared mechanism (see [`FreedCapacityUse`]).
///
/// # Returns
///
/// The value (currency units), or `None` only for
/// [`FreedCapacityUse::RefilledWithElective`] with a zero average elective
/// stay (the spell count would divide by zero); the other mechanisms always
/// yield a value.
///
/// # Examples
///
/// ```rust
/// use health_economics::bed_days_saved::{
///     freed_capacity_value, FreedCapacityUse,
/// };
///
/// // Honest refill claim: 1,200 bed days ÷ 3-day stay × £6,000 = £2.4M.
/// let refill = FreedCapacityUse::RefilledWithElective {
///     average_elective_stay_days: 3.0,
///     income_per_spell: 6_000.0,
/// };
/// assert!((freed_capacity_value(1_200.0, &refill).unwrap() - 2_400_000.0).abs() < 1e-9);
///
/// // Absorbed as slack at £100/day: only £120,000 — a fraction of the
/// // naive £480,000 claim.
/// let slack = FreedCapacityUse::AbsorbedAsSlack {
///     marginal_hotel_cost_per_bed_day: 100.0,
/// };
/// assert!((freed_capacity_value(1_200.0, &slack).unwrap() - 120_000.0).abs() < 1e-9);
/// ```
pub fn freed_capacity_value(bed_days_saved: f64, use_of_capacity: &FreedCapacityUse) -> Option<f64> {
    match use_of_capacity {
        FreedCapacityUse::RefilledWithElective {
            average_elective_stay_days,
            income_per_spell,
        } => {
            if *average_elective_stay_days == 0.0 {
                None
            } else {
                // Bed days ÷ stay = spells hosted; × income = funded activity.
                Some(bed_days_saved / average_elective_stay_days * income_per_spell)
            }
        }
        FreedCapacityUse::WardClosedOrFlexedDown {
            cost_released_per_bed_day,
        } => Some(bed_days_saved * cost_released_per_bed_day),
        FreedCapacityUse::AbsorbedAsSlack {
            marginal_hotel_cost_per_bed_day,
        } => Some(bed_days_saved * marginal_hotel_cost_per_bed_day),
    }
}

/// Additional elective spells the freed bed days can host.
///
/// bed days ÷ average elective stay — also the count of patients coming off
/// the waiting list under the refill mechanism.
///
/// # Arguments
///
/// * `bed_days_saved` — freed bed days (bed-days).
/// * `average_elective_stay_days` — average stay per backfilling spell
///   (days).
///
/// # Returns
///
/// The spell count, or `None` if `average_elective_stay_days` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::bed_days_saved::additional_elective_spells;
///
/// // Worked example: 1,200 bed days ÷ 3-day average stay = 400 spells.
/// let spells = additional_elective_spells(1_200.0, 3.0).unwrap();
/// assert!((spells - 400.0).abs() < 1e-9);
///
/// assert!(additional_elective_spells(1_200.0, 0.0).is_none());
/// ```
pub fn additional_elective_spells(
    bed_days_saved: f64,
    average_elective_stay_days: f64,
) -> Option<f64> {
    if average_elective_stay_days == 0.0 {
        None
    } else {
        Some(bed_days_saved / average_elective_stay_days)
    }
}

/// The naive claim: bed days × the fully-absorbed average cost of a bed day.
///
/// Wrong unless a ward actually closes — average-cost valuation of marginal
/// capacity is the canonical error, and finance directors discount it
/// heavily. Provided so the naive figure can be computed and contrasted.
///
/// # Arguments
///
/// * `bed_days_saved` — freed bed days (bed-days).
/// * `average_cost_per_bed_day` — fully-absorbed average cost per bed day
///   (currency/day; often quoted at £400+).
///
/// # Returns
///
/// The naive valuation (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::bed_days_saved::naive_bed_day_value;
///
/// // Worked example: 1,200 × £400 = £480,000. Wrong unless a ward closes.
/// let naive = naive_bed_day_value(1_200.0, 400.0);
/// assert!((naive - 480_000.0).abs() < 1e-9);
/// ```
pub fn naive_bed_day_value(bed_days_saved: f64, average_cost_per_bed_day: f64) -> f64 {
    bed_days_saved * average_cost_per_bed_day
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a remote-monitoring virtual ward lets 600 patients/year
    // go home 2 days early.

    #[test]
    fn virtual_ward_saves_1_200_bed_days() {
        // 600 patients × 2 days = 1,200 bed days.
        let got = bed_days_saved_from_earlier_discharge(600.0, 2.0);
        assert!((got - 1_200.0).abs() < 1e-9);
    }

    #[test]
    fn naive_claim_is_480_000() {
        // 1,200 × £400 = £480,000. Wrong unless a ward closes.
        let got = naive_bed_day_value(1_200.0, 400.0);
        assert!((got - 480_000.0).abs() < 1e-9);
    }

    #[test]
    fn backfill_yields_400_additional_elective_spells() {
        // 1,200 bed days ÷ 3-day average stay = 400 spells.
        let got = additional_elective_spells(1_200.0, 3.0).unwrap();
        assert!((got - 400.0).abs() < 1e-9);
    }

    #[test]
    fn honest_refill_claim_is_2_4_million_of_funded_activity() {
        // 400 spells × ~£6,000 income each = £2.4M of additional funded
        // activity under activity-based payment.
        let use_of_capacity = FreedCapacityUse::RefilledWithElective {
            average_elective_stay_days: 3.0,
            income_per_spell: 6_000.0,
        };
        let got = freed_capacity_value(1_200.0, &use_of_capacity).unwrap();
        assert!((got - 2_400_000.0).abs() < 1e-9);
    }

    #[test]
    fn refill_value_nets_against_virtual_ward_running_cost() {
        // The virtual ward's £350,000 running cost nets against the £2.4M.
        let value = freed_capacity_value(
            1_200.0,
            &FreedCapacityUse::RefilledWithElective {
                average_elective_stay_days: 3.0,
                income_per_spell: 6_000.0,
            },
        )
        .unwrap();
        let net = value - 350_000.0;
        assert!((net - 2_050_000.0).abs() < 1e-9);
    }

    #[test]
    fn slack_absorption_is_worth_marginal_hotel_cost_only() {
        // Absorbed as slack at £50–£150/day: 1,200 × £100 = £120,000 —
        // a fraction of the naive £480,000 claim.
        let got = freed_capacity_value(
            1_200.0,
            &FreedCapacityUse::AbsorbedAsSlack {
                marginal_hotel_cost_per_bed_day: 100.0,
            },
        )
        .unwrap();
        assert!((got - 120_000.0).abs() < 1e-9);
        assert!(got < naive_bed_day_value(1_200.0, 400.0));
    }

    #[test]
    fn ward_closure_releases_cash_per_bed_day() {
        // "Ward closed / flexed down": staffing + running cost released as
        // cash — the only mechanism where the £400/day figure is honest.
        let got = freed_capacity_value(
            1_200.0,
            &FreedCapacityUse::WardClosedOrFlexedDown {
                cost_released_per_bed_day: 400.0,
            },
        )
        .unwrap();
        assert!((got - 480_000.0).abs() < 1e-9);
    }

    #[test]
    fn avoided_admissions_route_gives_same_bed_days() {
        // Admissions avoided × avg LOS: 400 × 3 = 1,200.
        let got = bed_days_saved_from_avoided_admissions(400.0, 3.0);
        assert!((got - 1_200.0).abs() < 1e-9);
    }

    #[test]
    fn zero_average_stay_returns_none() {
        // Edge-case semantics: spell count and refill value are undefined
        // with a zero average elective stay.
        assert!(additional_elective_spells(1_200.0, 0.0).is_none());
        let use_of_capacity = FreedCapacityUse::RefilledWithElective {
            average_elective_stay_days: 0.0,
            income_per_spell: 6_000.0,
        };
        assert!(freed_capacity_value(1_200.0, &use_of_capacity).is_none());
    }
}
