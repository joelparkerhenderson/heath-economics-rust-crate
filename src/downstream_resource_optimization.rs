//! # Downstream Resource Optimization
//!
//! Saving an hour for a senior practitioner — a GP, a senior registrar, a
//! consultant — often prevents bottleneck delays for an entire
//! multi-disciplinary team (MDT) of nurses, administrative clerks, and
//! therapists who are waiting on clinical sign-offs. The value of unblocking
//! the bottleneck is the throughput of everyone downstream of it.
//!
//! This is the theory of constraints applied to clinical pathways: an hour
//! saved *at the constraint* is worth the whole system's marginal
//! throughput; an hour saved elsewhere is worth much less.
//!
//! ## Formula
//!
//! ```text
//! Value of unblocking = Σ over downstream roles (blocked hours released × unit cost)
//!                     + pathway throughput gain × value per pathway completion
//!
//! blocked hours released       — dependent-role time no longer idle-then-crunched
//! unit cost                    — £/hour for that role's time
//! pathway throughput gain      — extra completions (e.g. same-day discharges)
//! value per pathway completion — e.g. £ per avoided bed day
//!
//! Contrast: value of the same hour saved at a non-gating role ≈ that role's
//! capacity value alone.
//! ```
//!
//! Identify the constraint empirically: where does work queue longest? Whose
//! inbox do delays trace back to?
//!
//! ## Why it matters
//!
//! Health care runs on authorization chains: discharges wait for consultant
//! sign-off, treatment plans wait for MDT review, referrals wait for triage.
//! When the gating role is delayed, the cost is not one person's hour — it
//! is idle or blocked time across every dependent role, plus patient time in
//! limbo (extra bed days, longer RTT waits).
//!
//! ## Example
//!
//! The topic doc's worked example: a discharge-summary dashboard cuts a
//! consultant's morning information-assembly from 90 to 20 minutes, moving
//! review completion from 14:00 to 11:30. Four of six previously-late
//! discharges complete the same day (≈ 1,460 bed days/year avoided), and ~3
//! staff-hours/day of downstream blocked time is released (≈ 1,100 hrs/yr).
//! The consultant's own 70 minutes is the *smallest* part of the value.
//!
//! ```rust
//! use health_economics::downstream_resource_optimization::{
//!     DownstreamRelease, gating_task_minutes_saved, annualize,
//!     bed_days_avoided_per_year, value_of_unblocking,
//! };
//!
//! // The dashboard cuts assembly from 90 to 20 minutes.
//! assert_eq!(gating_task_minutes_saved(90.0, 20.0), 70.0);
//!
//! // Bed days avoided = 4 of the 6 late discharges × 365 ≈ 1,460/year.
//! let bed_days = bed_days_avoided_per_year(4.0, 365.0);
//! assert_eq!(bed_days, 1_460.0);
//!
//! // ~3 staff-hours/day of blocked time released ≈ 1,100 hrs/yr.
//! let hours = annualize(3.0, 365.0);
//! assert!((hours - 1_100.0).abs() < 10.0);
//!
//! // Value: staff release at £30/hr plus bed days at £300/day.
//! let releases = [DownstreamRelease {
//!     blocked_hours_released: hours,
//!     unit_cost_per_hour: 30.0,
//! }];
//! let value = value_of_unblocking(&releases, bed_days, 300.0);
//! assert_eq!(value, 1_095.0 * 30.0 + 1_460.0 * 300.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - This is code review, architecture sign-off, and the staff engineer's
//!   inbox: when five engineers wait a day for the one person who can
//!   approve a design, the cost is five engineer-days plus a day of cost of
//!   delay on the work itself — not one reviewer-hour.
//! - Tooling that compresses the gating role's task (better review context,
//!   automated pre-checks, dashboards that assemble what the approver needs)
//!   buys system throughput, not individual convenience.
//! - Measure pickup/wait time at the constraint — it is the software
//!   equivalent of the 14:00 discharge cliff.
//!
//! ## Pitfalls
//!
//! - **Optimizing a non-constraint**: beautiful tooling for a role nothing
//!   queues behind produces near-zero system value.
//! - **Constraint migration**: unblock the consultant and the constraint
//!   moves (to pharmacy, to transport) — model the *next* constraint before
//!   claiming full throughput gains.
//! - **Counting downstream hours as cash**: blocked-time release is
//!   capacity, subject to the usual redeployment test.
//!
//! ## Sources
//!
//! - Goldratt EM, *The Goal* (theory of constraints).
//! - NHS England, NHS productivity.
//!   <https://www.england.nhs.uk/long-read/nhs-productivity/>
//!
//! Topic doc: health-economics-metrics/topics/downstream-resource-optimization.md

/// Blocked time released for one downstream role when the constraint is
/// unblocked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownstreamRelease {
    /// Blocked hours released for this role over the period (e.g. hrs/year).
    pub blocked_hours_released: f64,
    /// Unit cost of an hour of this role's time (£/hour).
    pub unit_cost_per_hour: f64,
}

/// Value of unblocking the constraint.
///
/// Sums (blocked hours released × unit cost) over the downstream roles, then
/// adds pathway throughput gain × value per pathway completion. Note the
/// staff term is capacity value, not cash — apply the redeployment test
/// before claiming savings.
///
/// # Arguments
///
/// * `downstream_releases` — one entry per dependent role whose blocked time
///   is released.
/// * `pathway_throughput_gain` — extra pathway completions over the period
///   (e.g. avoided bed days/year).
/// * `value_per_pathway_completion` — £ per completion (e.g. £/bed day,
///   valued by mechanism).
///
/// # Returns
///
/// Total value of unblocking, £ over the period. Zero when there are no
/// releases and no throughput gain — the non-gating-role contrast case.
///
/// # Examples
///
/// ```rust
/// use health_economics::downstream_resource_optimization::{
///     DownstreamRelease, value_of_unblocking,
/// };
///
/// // 1,095 blocked staff-hours/yr at £30/hr + 1,460 bed days at £300/day.
/// let releases = [DownstreamRelease {
///     blocked_hours_released: 1_095.0,
///     unit_cost_per_hour: 30.0,
/// }];
/// let value = value_of_unblocking(&releases, 1_460.0, 300.0);
/// assert_eq!(value, 32_850.0 + 438_000.0);
///
/// // An hour saved at a non-gating role releases nothing downstream.
/// assert_eq!(value_of_unblocking(&[], 0.0, 300.0), 0.0);
/// ```
pub fn value_of_unblocking(
    downstream_releases: &[DownstreamRelease],
    pathway_throughput_gain: f64,
    value_per_pathway_completion: f64,
) -> f64 {
    // Σ over downstream roles: blocked hours released × unit cost.
    let staff_value: f64 = downstream_releases
        .iter()
        .map(|r| r.blocked_hours_released * r.unit_cost_per_hour)
        .sum();
    // Plus the pathway term: throughput gain × value per completion.
    staff_value + pathway_throughput_gain * value_per_pathway_completion
}

/// Time saved at the gating role itself, in minutes (before − after).
///
/// In this metric's worked example this is the *smallest* part of the value
/// — the point of the metric is that the downstream release and throughput
/// gain dwarf the gating role's own time.
///
/// # Arguments
///
/// * `minutes_before` — minutes the gating task took before the tooling.
/// * `minutes_after` — minutes it takes after.
///
/// # Returns
///
/// Minutes saved per occurrence (per day in the worked example).
///
/// # Examples
///
/// ```rust
/// use health_economics::downstream_resource_optimization::gating_task_minutes_saved;
///
/// // 90 min of information assembly cut to 20 min: 70 minutes/day saved.
/// assert_eq!(gating_task_minutes_saved(90.0, 20.0), 70.0);
/// ```
pub fn gating_task_minutes_saved(minutes_before: f64, minutes_after: f64) -> f64 {
    minutes_before - minutes_after
}

/// Annualize a per-day quantity over the number of operating days per year.
///
/// Works for bed days avoided per day, blocked staff hours released per day,
/// or any other daily rate.
///
/// # Arguments
///
/// * `per_day` — the daily quantity.
/// * `days_per_year` — operating days per year (365 for a ward that
///   discharges every day).
///
/// # Returns
///
/// The annual quantity (same unit as `per_day`, per year).
///
/// # Examples
///
/// ```rust
/// use health_economics::downstream_resource_optimization::annualize;
///
/// // ~3 staff-hours/day of blocked time released ≈ 1,095 hrs/yr.
/// assert_eq!(annualize(3.0, 365.0), 1_095.0);
/// ```
pub fn annualize(per_day: f64, days_per_year: f64) -> f64 {
    per_day * days_per_year
}

/// Bed days avoided per year from recovering late discharges.
///
/// Each discharge that completes the same day instead of slipping to the
/// next costs one fewer avoidable bed day; value the bed days by mechanism.
///
/// # Arguments
///
/// * `late_discharges_recovered_per_day` — discharges per day that now
///   complete on time.
/// * `days_per_year` — operating days per year.
///
/// # Returns
///
/// Bed days avoided per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::downstream_resource_optimization::bed_days_avoided_per_year;
///
/// // 4 of the 6 late discharges recovered × 365 = 1,460 bed days/year.
/// assert_eq!(bed_days_avoided_per_year(4.0, 365.0), 1_460.0);
/// ```
pub fn bed_days_avoided_per_year(
    late_discharges_recovered_per_day: f64,
    days_per_year: f64,
) -> f64 {
    annualize(late_discharges_recovered_per_day, days_per_year)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: assembly cut from "90 min/day" to "20 minutes" — the
    // consultant's own 70 minutes.
    #[test]
    fn dashboard_saves_70_consultant_minutes_per_day() {
        // 90 min of assembly cut to 20 min
        assert!((gating_task_minutes_saved(90.0, 20.0) - 70.0).abs() < 1e-9);
    }

    // Worked example: "Bed days avoided = 4 of the 6 late discharges × 365
    // ≈ 1,460 bed days/year".
    #[test]
    fn bed_days_avoided_is_about_1460_per_year() {
        // 4 of the 6 late discharges recovered × 365 ≈ 1,460 bed days/year
        let bed_days = bed_days_avoided_per_year(4.0, 365.0);
        assert!((bed_days - 1_460.0).abs() < 1e-9);
    }

    // Worked example: "~3 staff-hours/day of blocked time released
    // ≈ 1,100 hrs/yr".
    #[test]
    fn downstream_blocked_time_released_is_about_1100_hours_per_year() {
        // ~3 staff-hours/day of blocked time released ≈ 1,100 hrs/yr
        let hours = annualize(3.0, 365.0);
        assert!((hours - 1_100.0).abs() < 10.0);
    }

    // The math section: "Value of unblocking = Σ (blocked hours released ×
    // unit cost) + pathway throughput gain × value per pathway completion".
    #[test]
    fn value_of_unblocking_sums_staff_release_and_throughput() {
        // 1,095 blocked staff-hours/yr at £30/hr + 1,460 bed days at £300/day
        let releases = [DownstreamRelease {
            blocked_hours_released: annualize(3.0, 365.0),
            unit_cost_per_hour: 30.0,
        }];
        let value = value_of_unblocking(&releases, bed_days_avoided_per_year(4.0, 365.0), 300.0);
        assert!((value - (1_095.0 * 30.0 + 1_460.0 * 300.0)).abs() < 1e-9);
    }

    // Worked example: "The consultant's own 70 minutes is the *smallest*
    // part of the value — the point of this metric."
    #[test]
    fn consultant_time_is_the_smallest_part_of_the_value() {
        // The point of the metric: 70 min/day at even a consultant rate is
        // dwarfed by downstream staff release plus bed days.
        let consultant_hours_per_year = annualize(70.0 / 60.0, 365.0);
        let consultant_value = consultant_hours_per_year * 120.0; // £/hr, generous
        let releases = [DownstreamRelease {
            blocked_hours_released: annualize(3.0, 365.0),
            unit_cost_per_hour: 30.0,
        }];
        let system_value =
            value_of_unblocking(&releases, bed_days_avoided_per_year(4.0, 365.0), 300.0);
        assert!(consultant_value < system_value);
    }

    // The math section's contrast: "value of the same hour saved at a
    // non-gating role ≈ that role's capacity value alone".
    #[test]
    fn hour_saved_at_a_non_gating_role_is_worth_only_that_role_alone() {
        // Contrast case: no downstream release, no throughput gain.
        let value = value_of_unblocking(&[], 0.0, 300.0);
        assert!((value - 0.0).abs() < 1e-9);
    }
}
