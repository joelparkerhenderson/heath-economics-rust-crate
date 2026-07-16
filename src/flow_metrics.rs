//! # Flow Metrics
//!
//! Measures how work moves through a delivery system: cycle time, lead time,
//! throughput, work in progress (WIP), and flow efficiency. They are
//! governed by Little's Law — the same queueing math that governs hospital
//! beds and waiting lists.
//!
//! Little's Law is the lever: at fixed throughput, cutting WIP cuts cycle
//! time proportionally. It also runs hospitals:
//! `beds occupied = admissions/day × length of stay`.
//!
//! ## Formula
//!
//! ```text
//! Cycle time      = t(finished) − t(started)
//! Lead time       = t(delivered) − t(requested)     (includes pre-work queue)
//! Throughput      = items completed / period
//! WIP             = items started but unfinished
//! Flow efficiency = active time / (active + wait time) × 100
//!
//! Little's Law:  average WIP = throughput × average cycle time
//!                (equivalently: cycle time = WIP / throughput)
//!
//! t(·)        = timestamp of the named event
//! throughput  = completion rate (items per unit time)
//! active time = time an item is actually worked on; wait time = queued
//! ```
//!
//! ## Why it matters
//!
//! Most delivery time is not work — it is waiting. Flow-efficiency studies
//! of knowledge work routinely find items actively worked on only **5–15%**
//! of their elapsed time; the rest is queues. That means the cheapest
//! acceleration is queue removal, not hiring — precisely the insight
//! hospital patient-flow programs discovered about beds. For anything with a
//! cost of delay, flow metrics locate where the delay cost accrues.
//!
//! ## Example
//!
//! A team has 40 items in progress and completes 10/week. WIP limits cut WIP
//! to 15 at the same throughput; items average £3,000/week of delay cost.
//!
//! ```
//! use health_economics::flow_metrics::{
//!     littles_law_cycle_time, littles_law_wip, delay_cost_eliminated,
//! };
//!
//! // Before: cycle time = 40/10 = 4 weeks. After: 15/10 = 1.5 weeks — 62% faster.
//! let before = littles_law_cycle_time(40.0, 10.0).unwrap();
//! let after = littles_law_cycle_time(15.0, 10.0).unwrap();
//! assert_eq!(before, 4.0);
//! assert_eq!(after, 1.5);
//! assert!(((before - after) / before - 0.62).abs() < 0.01);
//!
//! // 10 items/week × 2.5 weeks less queueing × £3,000/week = £75,000/week eliminated.
//! assert_eq!(delay_cost_eliminated(10.0, 2.5, 3_000.0), 75_000.0);
//!
//! // Hospital mirror: 40 admissions/day × 6.0 days LOS = 240 beds occupied;
//! // cutting LOS to 5.6 days frees 16 beds.
//! let freed = littles_law_wip(40.0, 6.0) - littles_law_wip(40.0, 5.6);
//! assert!((freed - 16.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - **PR sub-stage benchmarks** (LinearB, ~8M PRs): elite pickup time < 7h,
//!   review < 6h, total cycle < ~26h — pickup time is pure queue, the first
//!   thing to attack.
//! - **Waiting lists** are backlogs; **RTT** is lead time; **bed occupancy**
//!   is WIP — improvement transfers in both directions (WIP limits ↔
//!   admission smoothing; queue-time instrumentation ↔ pathway-stage
//!   tracking).
//! - Flow efficiency under 15% is normal in both domains, and both hide it
//!   because *people* are busy while *work* waits — measure the work's
//!   clock, not the workers'.
//!
//! ## Pitfalls
//!
//! - **Utilization worship**: driving worker utilization toward 100%
//!   explodes queue times nonlinearly (M/M/1: wait ∝ ρ/(1−ρ)) — the reason
//!   95%-occupied hospitals gridlock and 95%-allocated teams stall.
//! - **Averages over skewed distributions**: cycle times are heavy-tailed;
//!   forecast with percentiles (p85), not means.
//! - **Cutting WIP by rejecting work upstream** and calling it flow
//!   improvement — the demand didn't vanish, it queued outside the
//!   measurement boundary (the hospital version: ambulances waiting outside
//!   the ED).
//!
//! ## Sources
//!
//! - Little's Law and flow metrics overviews.
//!   <https://agility-at-scale.com/safe/lpm/flow-metrics/> ;
//!   <https://getdx.com/blog/flow-metrics/>
//! - LinearB engineering benchmarks.
//!   <https://linearb.io/resources/engineering-benchmarks>
//! - Reinertsen DG, *The Principles of Product Development Flow*.
//!
//! Topic doc: health-economics-metrics/topics/flow-metrics.md

/// Cycle time: time finished minus time started.
///
/// Timestamps may be in any consistent unit (hours, days, weeks); the result
/// carries the same unit. Measures the in-progress clock only — the pre-work
/// queue belongs to [`lead_time`].
///
/// # Arguments
///
/// * `finished` — timestamp work finished.
/// * `started` — timestamp work started.
///
/// # Returns
///
/// Elapsed cycle time (`finished − started`), same units as the inputs.
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::cycle_time;
///
/// // Started day 6, finished day 10: 4 days of cycle time.
/// assert_eq!(cycle_time(10.0, 6.0), 4.0);
/// ```
pub fn cycle_time(finished: f64, started: f64) -> f64 {
    finished - started
}

/// Lead time: time delivered minus time requested.
///
/// Unlike [`cycle_time`], lead time includes the pre-work queue — the wait
/// between request and start. The health-care analogue is referral-to-
/// treatment (RTT) time.
///
/// # Arguments
///
/// * `delivered` — timestamp the item was delivered.
/// * `requested` — timestamp the item was requested.
///
/// # Returns
///
/// Elapsed lead time (`delivered − requested`), same units as the inputs.
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::lead_time;
///
/// // Requested day 2, delivered day 10: 8 days of lead time.
/// assert_eq!(lead_time(10.0, 2.0), 8.0);
/// ```
pub fn lead_time(delivered: f64, requested: f64) -> f64 {
    delivered - requested
}

/// Throughput: items completed per period.
///
/// # Arguments
///
/// * `items_completed` — count of items finished in the period.
/// * `period` — length of the period (any time unit).
///
/// # Returns
///
/// Completion rate (items per unit time), or `None` if `period` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::throughput;
///
/// // 10 items completed in 1 week.
/// assert_eq!(throughput(10.0, 1.0), Some(10.0));
/// assert!(throughput(5.0, 0.0).is_none());
/// ```
pub fn throughput(items_completed: f64, period: f64) -> Option<f64> {
    if period == 0.0 { None } else { Some(items_completed / period) }
}

/// Flow efficiency as a percentage: active time over total elapsed time.
///
/// Knowledge-work studies routinely find 5–15% — the rest is queues. Measure
/// the work item's clock, not the workers': people can be 100% busy while
/// every item spends most of its life waiting.
///
/// # Arguments
///
/// * `active_time` — time the item was actively worked on.
/// * `wait_time` — time the item spent queued (same units).
///
/// # Returns
///
/// Percentage 0–100, or `None` if `active_time + wait_time` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::flow_efficiency_percent;
///
/// // 1 day of work inside 10 elapsed days = 10% — typical knowledge work.
/// assert_eq!(flow_efficiency_percent(1.0, 9.0), Some(10.0));
/// assert!(flow_efficiency_percent(0.0, 0.0).is_none());
/// ```
pub fn flow_efficiency_percent(active_time: f64, wait_time: f64) -> Option<f64> {
    let total = active_time + wait_time;
    if total == 0.0 { None } else { Some(active_time / total * 100.0) }
}

/// Little's Law solved for cycle time: `WIP / throughput`.
///
/// At fixed throughput, cutting WIP cuts cycle time proportionally — the
/// queue-discipline lever. Valid for averages of a stable (stationary)
/// system.
///
/// # Arguments
///
/// * `wip` — average items started but unfinished (count).
/// * `throughput` — completion rate (items per unit time).
///
/// # Returns
///
/// Average cycle time in the time unit of `throughput`, or `None` if
/// `throughput` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::littles_law_cycle_time;
///
/// // Worked example: 40 items in progress at 10/week = 4 weeks;
/// // WIP-limited to 15 at the same throughput = 1.5 weeks.
/// assert_eq!(littles_law_cycle_time(40.0, 10.0), Some(4.0));
/// assert_eq!(littles_law_cycle_time(15.0, 10.0), Some(1.5));
/// ```
pub fn littles_law_cycle_time(wip: f64, throughput: f64) -> Option<f64> {
    if throughput == 0.0 { None } else { Some(wip / throughput) }
}

/// Little's Law solved for WIP: `throughput × cycle time`.
///
/// The hospital form is `beds occupied = admissions/day × length of stay` —
/// same law, same lever.
///
/// # Arguments
///
/// * `throughput` — arrival/completion rate (items per unit time, e.g.
///   admissions/day).
/// * `average_cycle_time` — average time in the system (same time unit,
///   e.g. length of stay in days).
///
/// # Returns
///
/// Average WIP (items in the system, e.g. beds occupied).
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::littles_law_wip;
///
/// // Hospital mirror: 40 admissions/day × 6.0 days LOS = 240 beds occupied.
/// assert_eq!(littles_law_wip(40.0, 6.0), 240.0);
/// ```
pub fn littles_law_wip(throughput: f64, average_cycle_time: f64) -> f64 {
    throughput * average_cycle_time
}

/// Delay cost eliminated per period by shortening queue time.
///
/// Each completed item now spends less time queued, so its cost of delay
/// accrues for less time; at steady state the saving flows at
/// `throughput × time saved × CoD rate` per period.
///
/// # Arguments
///
/// * `throughput` — items completed per period.
/// * `queue_time_saved_per_item` — queue time removed per item (periods,
///   e.g. weeks).
/// * `cost_of_delay_per_item_period` — delay cost per item per period
///   (£/item-week).
///
/// # Returns
///
/// Delay cost eliminated per period (£/period).
///
/// # Examples
///
/// ```
/// use health_economics::flow_metrics::delay_cost_eliminated;
///
/// // Worked example: 10 items/week × 2.5 weeks saved × £3,000/week
/// // = £75,000/week — from a policy change costing nothing.
/// assert_eq!(delay_cost_eliminated(10.0, 2.5, 3_000.0), 75_000.0);
/// ```
pub fn delay_cost_eliminated(
    throughput: f64,
    queue_time_saved_per_item: f64,
    cost_of_delay_per_item_period: f64,
) -> f64 {
    // Items flowing per period × weeks of queue removed each × £/item-week.
    throughput * queue_time_saved_per_item * cost_of_delay_per_item_period
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "40 items in progress and completes 10/week: cycle time = 40/10 = 4 weeks".
    #[test]
    fn worked_example_cycle_time_before_is_4_weeks() {
        let ct = littles_law_cycle_time(40.0, 10.0).unwrap();
        assert!((ct - 4.0).abs() < 1e-9, "got {ct}");
    }

    // Doc line: "cutting WIP to 15: cycle time = 15/10 = 1.5 weeks".
    #[test]
    fn worked_example_cycle_time_after_is_1_5_weeks() {
        let ct = littles_law_cycle_time(15.0, 10.0).unwrap();
        assert!((ct - 1.5).abs() < 1e-9, "got {ct}");
    }

    // Doc line: "same people, same throughput, 62% faster delivery".
    #[test]
    fn worked_example_delivery_is_62_percent_faster() {
        let before = littles_law_cycle_time(40.0, 10.0).unwrap();
        let after = littles_law_cycle_time(15.0, 10.0).unwrap();
        let reduction = (before - after) / before;
        assert!((reduction - 0.62).abs() < 0.01, "got {reduction}");
    }

    // Doc line: "10 items/week × 2.5 × 3,000 = £75,000/week of delay cost eliminated".
    #[test]
    fn worked_example_delay_cost_eliminated_is_75000_per_week() {
        let saved = delay_cost_eliminated(10.0, 2.5, 3_000.0);
        assert!((saved - 75_000.0).abs() < 1e-9, "got {saved}");
    }

    // Doc line: "40 admissions/day × 6.0 days LOS = 240 beds".
    #[test]
    fn worked_example_hospital_beds_occupied_is_240() {
        let beds = littles_law_wip(40.0, 6.0);
        assert!((beds - 240.0).abs() < 1e-9, "got {beds}");
    }

    // Doc line: "cut ... LOS to 5.6 days and 16 beds free up".
    #[test]
    fn worked_example_16_beds_freed_at_5_6_days_los() {
        let freed = littles_law_wip(40.0, 6.0) - littles_law_wip(40.0, 5.6);
        assert!((freed - 16.0).abs() < 1e-9, "got {freed}");
    }

    // Doc line: "items actively worked on only 5–15% of their elapsed time".
    #[test]
    fn flow_efficiency_percent_computes_active_share() {
        let fe = flow_efficiency_percent(1.0, 9.0).unwrap();
        assert!((fe - 10.0).abs() < 1e-9, "got {fe}");
    }

    // Doc formulas: cycle time and lead time are timestamp differences;
    // lead time includes the pre-work queue.
    #[test]
    fn cycle_and_lead_time_are_differences() {
        assert!((cycle_time(10.0, 6.0) - 4.0).abs() < 1e-9);
        assert!((lead_time(10.0, 2.0) - 8.0).abs() < 1e-9);
    }

    // Guard behavior: zero denominators return None.
    #[test]
    fn zero_denominators_return_none() {
        assert!(throughput(5.0, 0.0).is_none());
        assert!(littles_law_cycle_time(40.0, 0.0).is_none());
        assert!(flow_efficiency_percent(0.0, 0.0).is_none());
    }
}
