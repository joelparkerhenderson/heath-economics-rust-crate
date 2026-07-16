//! # GDS Service Metrics
//!
//! The UK Government Digital Service (GDS) Service Manual mandates four KPIs
//! for every government digital service: **cost per transaction, user
//! satisfaction, completion rate, and digital take-up**. Together they are
//! the minimal economics of a public digital service — and the template NHS
//! digital services inherit.
//!
//! The four KPIs are one economic model, not four dashboards: savings only
//! materialize when people *complete* the digital journey (completion rate)
//! *instead of* the expensive channel (take-up).
//!
//! ## Formula
//!
//! ```text
//! Cost per transaction = total service cost / completed transactions
//! Completion rate      = completed / started transactions × 100
//! Digital take-up      = digital transactions / all-channel transactions × 100
//! User satisfaction    = % satisfied+very satisfied (5-point, in-service survey)
//!
//! Channel-shift saving = volume × take-up shift × (cost_old_channel − cost_digital)
//! … minus failure demand: (1 − completion rate) × fallback channel cost
//!
//! volume        = all-channel transactions per year
//! take-up shift = fraction of volume moved to the digital channel (0–1)
//! cost_*        = unit cost per transaction in each channel (£)
//! failure demand = failed digital journeys that fall back to the expensive channel
//! ```
//!
//! ## Why it matters
//!
//! The GDS metrics encode the channel-shift business case that funded a
//! decade of government digitization: the Digital Efficiency Report found
//! digital transactions ~20× cheaper than phone and ~50× cheaper than
//! face-to-face (local-gov figures: web £0.15, phone £2.83, face-to-face
//! £8.62). Satisfaction is the leading indicator: dissatisfied users revert
//! to phone, so a satisfaction drop forecasts take-up decay before it
//! appears.
//!
//! ## Example
//!
//! An NHS appointment-management service: 2M transactions/year, 70% phone
//! (£3.20/call) / 30% digital (£0.25). A redesign lifts take-up to 55% and
//! completion from 84% to 93%.
//!
//! ```
//! use health_economics::gds_service_metrics::{
//!     channel_shift_saving, failure_demand_cost,
//! };
//!
//! // Take-up shift saving = 2M × 0.25 × (3.20 − 0.25) = £1,475,000/year.
//! let shift = channel_shift_saving(2_000_000.0, 0.25, 3.20, 0.25);
//! assert!((shift - 1_475_000.0).abs() < 1e-6);
//!
//! // Failure demand before: 2M × 0.30 × 0.16 × £3.20 = £307,200.
//! let before = failure_demand_cost(2_000_000.0, 0.30, 0.84, 3.20);
//! assert!((before - 307_200.0).abs() < 1e-6);
//!
//! // After: 2M × 0.55 × 0.07 × £3.20 = £246,400 — net £60,800/year saved:
//! // completion improvements protect the take-up gains.
//! let after = failure_demand_cost(2_000_000.0, 0.55, 0.93, 3.20);
//! assert!((after - 246_400.0).abs() < 1e-6);
//! assert!((before - after - 60_800.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - The four KPIs are a production-grade cost-consequence table: one cost
//!   metric, three outcome metrics, never collapsed into a score.
//! - **Completion rate is a funnel-instrumentation problem** — every
//!   abandonment point is findable and fixable.
//! - **Cost per transaction is cloud unit economics** plus staff-assisted-
//!   channel costs.
//! - **Take-up is an equity metric in disguise** — the users who can't or
//!   won't shift channels are disproportionately elderly, disabled, and
//!   deprived, so aggressive channel closure converts "savings" into access
//!   harm.
//! - Publishing the KPIs (GOV.UK does, per service) is itself a mechanism:
//!   transparency disciplines forecasts.
//!
//! ## Pitfalls
//!
//! - **Take-up by coercion**: closing the phone line lifts take-up and dumps
//!   failure demand on front-line staff; measure total-system cost.
//! - **Completion measured from page-2**: starting the funnel after the
//!   drop-off point flatters the rate.
//! - **Per-transaction cost ignoring assisted-digital support** and
//!   failure-demand handling.
//! - **Satisfaction surveys only at successful completion** — the
//!   dissatisfied mostly never reach the survey.
//!
//! ## Sources
//!
//! - GOV.UK Service Manual, measuring success / mandatory KPIs.
//!   <https://www.gov.uk/service-manual/measuring-success/data-you-must-publish>
//! - Digital Efficiency Report.
//!   <https://www.gov.uk/government/publications/digital-efficiency-report/digital-efficiency-report>
//!
//! Topic doc: health-economics-metrics/topics/gds-service-metrics.md

/// Cost per transaction: total service cost over completed transactions.
///
/// GDS KPI 1. Include assisted-digital support and failure-demand handling
/// in the total cost, or the figure flatters the service.
///
/// # Arguments
///
/// * `total_service_cost` — full service cost for the period (£).
/// * `completed_transactions` — transactions completed in the period (count).
///
/// # Returns
///
/// £ per completed transaction, or `None` if `completed_transactions` is
/// zero.
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::cost_per_transaction;
///
/// // £500,000 service cost over 2M completed transactions = £0.25 each —
/// // the worked example's digital unit cost.
/// assert_eq!(cost_per_transaction(500_000.0, 2_000_000.0), Some(0.25));
/// assert!(cost_per_transaction(1_000.0, 0.0).is_none());
/// ```
pub fn cost_per_transaction(total_service_cost: f64, completed_transactions: f64) -> Option<f64> {
    if completed_transactions == 0.0 {
        None
    } else {
        Some(total_service_cost / completed_transactions)
    }
}

/// Completion rate as a percentage: completed over started transactions.
///
/// GDS KPI 2. Measure from the true start of the funnel — starting the
/// funnel after the drop-off point flatters the rate.
///
/// # Arguments
///
/// * `completed` — transactions completed (count).
/// * `started` — transactions started (count).
///
/// # Returns
///
/// Percentage 0–100, or `None` if `started` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::completion_rate_percent;
///
/// // Worked example rates: 84% before the redesign, 93% after.
/// assert_eq!(completion_rate_percent(84.0, 100.0), Some(84.0));
/// assert_eq!(completion_rate_percent(93.0, 100.0), Some(93.0));
/// ```
pub fn completion_rate_percent(completed: f64, started: f64) -> Option<f64> {
    if started == 0.0 { None } else { Some(completed / started * 100.0) }
}

/// Digital take-up as a percentage: digital over all-channel transactions.
///
/// GDS KPI 3. Take-up is an equity metric in disguise — lift it by making
/// the digital journey better, not by closing the phone line.
///
/// # Arguments
///
/// * `digital_transactions` — transactions via the digital channel (count).
/// * `all_channel_transactions` — transactions across every channel (count).
///
/// # Returns
///
/// Percentage 0–100, or `None` if `all_channel_transactions` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::digital_take_up_percent;
///
/// // Worked example after the redesign: 1.1M of 2M transactions = 55%.
/// let take_up = digital_take_up_percent(1_100_000.0, 2_000_000.0).unwrap();
/// assert!((take_up - 55.0).abs() < 1e-9);
/// ```
pub fn digital_take_up_percent(digital_transactions: f64, all_channel_transactions: f64) -> Option<f64> {
    if all_channel_transactions == 0.0 {
        None
    } else {
        Some(digital_transactions / all_channel_transactions * 100.0)
    }
}

/// User satisfaction as a percentage of survey respondents.
///
/// GDS KPI 4: respondents answering satisfied or very satisfied on the
/// 5-point in-service survey, over all respondents. Beware surveying only at
/// successful completion — the dissatisfied mostly never reach the survey.
///
/// # Arguments
///
/// * `satisfied_or_very_satisfied` — respondents in the top two categories
///   (count).
/// * `respondents` — all survey respondents (count).
///
/// # Returns
///
/// Percentage 0–100, or `None` if `respondents` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::user_satisfaction_percent;
///
/// // 80 of 100 respondents satisfied or very satisfied = 80%.
/// assert_eq!(user_satisfaction_percent(80.0, 100.0), Some(80.0));
/// assert!(user_satisfaction_percent(1.0, 0.0).is_none());
/// ```
pub fn user_satisfaction_percent(
    satisfied_or_very_satisfied: f64,
    respondents: f64,
) -> Option<f64> {
    if respondents == 0.0 {
        None
    } else {
        Some(satisfied_or_very_satisfied / respondents * 100.0)
    }
}

/// Channel-shift saving: volume moved to digital times the unit-cost gap.
///
/// # Arguments
///
/// * `volume` — all-channel transactions per year (count).
/// * `take_up_shift` — fraction of volume moved to digital (e.g. `0.25` for
///   a 25-percentage-point shift, 30% → 55%).
/// * `cost_old_channel` — unit cost of the channel shifted away from
///   (£, e.g. £3.20/phone call).
/// * `cost_digital` — unit cost of the digital channel (£, e.g. £0.25).
///
/// # Returns
///
/// Gross channel-shift saving (£/year).
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::channel_shift_saving;
///
/// // Worked example: 2M × 0.25 × (3.20 − 0.25) = £1,475,000/year.
/// let s = channel_shift_saving(2_000_000.0, 0.25, 3.20, 0.25);
/// assert!((s - 1_475_000.0).abs() < 1e-6);
/// ```
pub fn channel_shift_saving(
    volume: f64,
    take_up_shift: f64,
    cost_old_channel: f64,
    cost_digital: f64,
) -> f64 {
    // Transactions moved × per-transaction cost gap between channels.
    volume * take_up_shift * (cost_old_channel - cost_digital)
}

/// Failure-demand cost: failed digital journeys falling back to the
/// expensive channel.
///
/// # Arguments
///
/// * `volume` — all-channel transactions per year (count).
/// * `digital_share` — fraction of transactions attempted digitally (0–1).
/// * `completion_rate` — fraction of digital journeys completed (0–1);
///   `1 − completion_rate` is the failure share.
/// * `fallback_channel_cost` — unit cost of the channel failures fall back
///   to (£, e.g. £3.20/phone call).
///
/// # Returns
///
/// Failure-demand cost (£/year).
///
/// # Examples
///
/// ```
/// use health_economics::gds_service_metrics::failure_demand_cost;
///
/// // Worked example, before: 2M × 0.30 × (1 − 0.84) × £3.20 = £307,200.
/// let c = failure_demand_cost(2_000_000.0, 0.30, 0.84, 3.20);
/// assert!((c - 307_200.0).abs() < 1e-6);
/// ```
pub fn failure_demand_cost(
    volume: f64,
    digital_share: f64,
    completion_rate: f64,
    fallback_channel_cost: f64,
) -> f64 {
    // Digital attempts × failure share (1 − completion) × fallback unit cost.
    volume * digital_share * (1.0 - completion_rate) * fallback_channel_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "Take-up shift saving = 2M × 0.25 × (3.20 − 0.25) = £1,475,000/year".
    #[test]
    fn worked_example_take_up_shift_saving_is_1475000() {
        let s = channel_shift_saving(2_000_000.0, 0.25, 3.20, 0.25);
        assert!((s - 1_475_000.0).abs() < 1e-6, "got {s}");
    }

    // Doc line: "before: 2M × 0.30 × 0.16 × £3.20 = £307,200".
    #[test]
    fn worked_example_failure_demand_before_is_307200() {
        let c = failure_demand_cost(2_000_000.0, 0.30, 0.84, 3.20);
        assert!((c - 307_200.0).abs() < 1e-6, "got {c}");
    }

    // Doc line: "after: 2M × 0.55 × 0.07 × £3.20 = £246,400".
    #[test]
    fn worked_example_failure_demand_after_is_246400() {
        let c = failure_demand_cost(2_000_000.0, 0.55, 0.93, 3.20);
        assert!((c - 246_400.0).abs() < 1e-6, "got {c}");
    }

    // Doc line: "net £60,800/year — completion improvements protect the take-up gains".
    #[test]
    fn worked_example_net_failure_demand_saving_is_60800() {
        let before = failure_demand_cost(2_000_000.0, 0.30, 0.84, 3.20);
        let after = failure_demand_cost(2_000_000.0, 0.55, 0.93, 3.20);
        let net = before - after;
        assert!((net - 60_800.0).abs() < 1e-6, "got {net}");
    }

    // Doc lines: "completion from 84% to 93%", "lifts digital take-up to 55%".
    #[test]
    fn kpi_percentages_from_counts() {
        let completion_before = completion_rate_percent(84.0, 100.0).unwrap();
        assert!((completion_before - 84.0).abs() < 1e-9);
        let completion_after = completion_rate_percent(93.0, 100.0).unwrap();
        assert!((completion_after - 93.0).abs() < 1e-9);
        let take_up = digital_take_up_percent(1_100_000.0, 2_000_000.0).unwrap();
        assert!((take_up - 55.0).abs() < 1e-9);
    }

    // Doc line: "web £0.15, phone £2.83, face-to-face £8.62" — digital
    // ~20× cheaper than phone, ~50× cheaper than face-to-face.
    #[test]
    fn digital_efficiency_report_cost_ratios() {
        let phone_ratio: f64 = 2.83 / 0.15;
        let face_ratio: f64 = 8.62 / 0.15;
        assert!((phone_ratio - 20.0).abs() < 2.0, "got {phone_ratio}");
        assert!((face_ratio - 50.0).abs() < 10.0, "got {face_ratio}");
    }

    // Guard behavior: the KPI ratios return None on zero denominators, and
    // reproduce the worked example's £0.25 digital unit cost.
    #[test]
    fn guarded_ratios_return_none_on_zero() {
        assert!(cost_per_transaction(1000.0, 0.0).is_none());
        assert!(completion_rate_percent(1.0, 0.0).is_none());
        assert!(digital_take_up_percent(1.0, 0.0).is_none());
        assert!(user_satisfaction_percent(1.0, 0.0).is_none());
        let cpt = cost_per_transaction(500_000.0, 2_000_000.0).unwrap();
        assert!((cpt - 0.25).abs() < 1e-9);
        let sat = user_satisfaction_percent(80.0, 100.0).unwrap();
        assert!((sat - 80.0).abs() < 1e-9);
    }
}
