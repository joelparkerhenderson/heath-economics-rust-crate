//! # DORA Metrics
//!
//! The DORA (DevOps Research and Assessment) metrics are four measures of
//! software delivery performance — deployment frequency, lead time for
//! changes, change failure rate, and failed-deployment recovery time — plus
//! reliability as a fifth. They are the field's most validated delivery
//! benchmarks, and each has a direct health-economics reading.
//!
//! ## Formula
//!
//! ```text
//! Deployment frequency  = production deployments / time
//! Lead time for changes = t(deploy) − t(commit), median
//! Change failure rate   = failed changes / total changes × 100
//! Recovery time (MTTR)  = t(restored) − t(failure), median
//! Reliability           = SLO attainment (availability, latency, correctness)
//!
//! Health-economics translations:
//! Lead time     → cost of delay: weeks in the pipeline × CoD (£ or QALYs/week)
//! Failure rate  → adverse-event rate of software change: CFR × cost per incident
//! Recovery time → downtime harm: MTTR × (lost clinical activity + safety exposure)/hr
//! Reliability   → benefit discount: a service at 99% availability delivers
//!                 ≈ 0.99 of its modeled benefit — the software analogue of adherence
//! ```
//!
//! ## Why it matters
//!
//! DORA's decade of research links these metrics to organizational
//! performance. The 2024 report's clusters: **elite** teams deploy on demand
//! (multiple times/day), take under a day from commit to production, fail
//! ~5% of changes, and recover in under an hour; **low** performers deploy
//! monthly-or-less, take months, fail ~40% of changes, and recover in weeks.
//! For a health system these are not IT vanity numbers: they determine how
//! fast clinical value reaches patients and how much risk each change
//! carries.
//!
//! ## Example
//!
//! The topic doc's worked example: a trust's patient-flow team improves from
//! monthly deploys / 6-week lead time / 25% CFR / 2-day MTTR to weekly
//! deploys / 4-day lead time / 8% CFR / 2-hour MTTR. With ~30
//! improvements/year at £4,000/week cost of delay each, the ~5.4-week
//! lead-time cut pulls ~£648,000/year of value forward; the CFR improvement
//! avoids ~5 failed changes/year at £15,000 each = £76,500/year.
//!
//! ```rust
//! use health_economics::dora_metrics::{
//!     days_to_weeks, lead_time_reduction_weeks, value_pulled_forward,
//!     failed_changes_avoided, failure_cost_avoided,
//! };
//!
//! // Lead time cut: 6 weeks → 4 days ≈ 5.4 weeks saved per improvement.
//! let reduction = lead_time_reduction_weeks(6.0, days_to_weeks(4.0));
//! assert!((reduction - 5.4).abs() < 0.05);
//!
//! // 30 improvements × 5.4 weeks × £4,000/week ≈ £648,000/year pulled forward.
//! let value = value_pulled_forward(30.0, 5.4, 4_000.0);
//! assert_eq!(value, 648_000.0);
//!
//! // CFR 25% → 8%: 30 × 0.17 ≈ 5 fewer failed changes/year...
//! let avoided = failed_changes_avoided(30.0, 0.25, 0.08);
//! assert!((avoided - 5.1).abs() < 1e-9);
//! // ...× £15,000 average incident cost = £76,500/year.
//! let saved = failure_cost_avoided(30.0, 0.25, 0.08, 15_000.0);
//! assert_eq!(saved, 76_500.0);
//! ```
//!
//! ## Software engineering connection
//!
//! This *is* the software side — the connection worth stating is the reverse
//! mapping: DORA metrics are the hospital's operational metrics wearing
//! different clothes.
//!
//! - Lead time ↔ referral to treatment.
//! - Change failure rate ↔ readmission rate (work that bounced back).
//! - MTTR ↔ emergency response; deployment frequency ↔ clinic throughput.
//! - Improvement methods transfer in both directions because both are
//!   queueing systems under safety constraints.
//! - DORA 2025's AI finding: AI adoption now correlates with higher
//!   throughput but *worse* stability — an intervention with efficacy and
//!   side effects, demanding a net-benefit analysis.
//!
//! ## Pitfalls
//!
//! - **Metric gaming**: deploy counts inflated by no-op releases; CFR
//!   deflated by not counting hotfixes as failures. Define events precisely,
//!   as HTA defines endpoints.
//! - **Cross-team league tables**: DORA clusters compare practices, not
//!   teams with different risk profiles; a clinical-systems team at "high"
//!   may be optimal where "elite" would be reckless.
//! - **Optimizing one metric**: speed without CFR/reliability is the
//!   throughput-instability trade-off — always report the four together
//!   (they are a cost-consequence table, not a score).
//!
//! ## Sources
//!
//! - DORA research and reports. <https://dora.dev/>
//! - 2024 DORA benchmarks summary. <https://octopus.com/devops/metrics/dora-metrics/>
//! - DORA 2025 State of AI-assisted Software Development.
//!   <https://dora.dev/dora-report-2025/>
//!
//! Topic doc: health-economics-metrics/topics/dora-metrics.md

/// Deployment frequency: production deployments / time period.
///
/// Use consistent units for the period (e.g. deployments per year with
/// `period` in years). Elite teams deploy on demand (multiple times/day);
/// low performers deploy monthly or less.
///
/// # Arguments
///
/// * `deployments` — count of production deployments.
/// * `period` — length of the observation period (any time unit).
///
/// # Returns
///
/// `Some(deployments per unit period)`, or `None` when `period` is zero
/// (frequency undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::deployment_frequency;
///
/// // Monthly deploys = 12/year; weekly = 52/year.
/// assert_eq!(deployment_frequency(12.0, 1.0), Some(12.0));
/// assert_eq!(deployment_frequency(52.0, 1.0), Some(52.0));
/// assert_eq!(deployment_frequency(1.0, 0.0), None);
/// ```
pub fn deployment_frequency(deployments: f64, period: f64) -> Option<f64> {
    if period == 0.0 {
        None
    } else {
        Some(deployments / period)
    }
}

/// Change failure rate as a percentage: failed changes / total changes × 100.
///
/// The adverse-event rate of software change. Elite ~5%; low performers
/// ~40%. Define "failed" precisely (hotfixes count) to avoid gaming.
///
/// # Arguments
///
/// * `failed_changes` — changes that caused a failure in production.
/// * `total_changes` — all production changes in the period.
///
/// # Returns
///
/// `Some(percentage)` (e.g. 25.0 for 25%), or `None` when `total_changes` is
/// zero (no changes were made — rate undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::change_failure_rate_percent;
///
/// // Before: 25% of changes fail; after: 8%.
/// assert_eq!(change_failure_rate_percent(25.0, 100.0), Some(25.0));
/// assert_eq!(change_failure_rate_percent(8.0, 100.0), Some(8.0));
/// assert_eq!(change_failure_rate_percent(1.0, 0.0), None);
/// ```
pub fn change_failure_rate_percent(failed_changes: f64, total_changes: f64) -> Option<f64> {
    if total_changes == 0.0 {
        None
    } else {
        Some(failed_changes / total_changes * 100.0)
    }
}

/// Convert a lead time in days to weeks (7-day weeks).
///
/// # Arguments
///
/// * `days` — duration in days.
///
/// # Returns
///
/// The same duration in weeks.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::days_to_weeks;
///
/// // A 4-day lead time ≈ 0.57 weeks.
/// assert!((days_to_weeks(4.0) - 0.5714).abs() < 1e-3);
/// ```
pub fn days_to_weeks(days: f64) -> f64 {
    days / 7.0
}

/// Lead-time reduction in weeks: before − after.
///
/// # Arguments
///
/// * `before_weeks` — lead time before the delivery investment, in weeks.
/// * `after_weeks` — lead time after, in weeks (see [`days_to_weeks`]).
///
/// # Returns
///
/// The reduction in weeks (positive when lead time improved).
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::{
///     days_to_weeks, lead_time_reduction_weeks,
/// };
///
/// // 6 weeks → 4 days is a ~5.4-week reduction.
/// let reduction = lead_time_reduction_weeks(6.0, days_to_weeks(4.0));
/// assert!((reduction - 5.4).abs() < 0.05);
/// ```
pub fn lead_time_reduction_weeks(before_weeks: f64, after_weeks: f64) -> f64 {
    before_weeks - after_weeks
}

/// Annual value of pulling benefit streams forward by shipping sooner.
///
/// Each improvement's benefit starts weeks earlier; the value pulled forward
/// is improvements/year × lead-time reduction × the improvement's cost of
/// delay (£/week).
///
/// # Arguments
///
/// * `improvements_per_year` — shipped improvements per year.
/// * `lead_time_reduction_weeks` — weeks each improvement now arrives sooner.
/// * `value_per_improvement_per_week` — average CoD per improvement, £/week.
///
/// # Returns
///
/// Value delivered sooner, £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::value_pulled_forward;
///
/// // 30 improvements × 5.4 weeks × £4,000/week ≈ £648,000/year.
/// assert_eq!(value_pulled_forward(30.0, 5.4, 4_000.0), 648_000.0);
/// ```
pub fn value_pulled_forward(
    improvements_per_year: f64,
    lead_time_reduction_weeks: f64,
    value_per_improvement_per_week: f64,
) -> f64 {
    improvements_per_year * lead_time_reduction_weeks * value_per_improvement_per_week
}

/// Failed changes avoided per year by improving CFR.
///
/// changes/year × (CFR before − CFR after), with rates as fractions (0.25
/// for 25%), not percentages.
///
/// # Arguments
///
/// * `changes_per_year` — production changes per year.
/// * `cfr_before` — change failure rate before, as a fraction.
/// * `cfr_after` — change failure rate after, as a fraction.
///
/// # Returns
///
/// Expected count of failed changes avoided per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::failed_changes_avoided;
///
/// // 30 × (0.25 − 0.08) = 5.1 ≈ ~5 fewer failed changes/year.
/// let avoided = failed_changes_avoided(30.0, 0.25, 0.08);
/// assert!((avoided - 5.1).abs() < 1e-9);
/// ```
pub fn failed_changes_avoided(changes_per_year: f64, cfr_before: f64, cfr_after: f64) -> f64 {
    changes_per_year * (cfr_before - cfr_after)
}

/// Annual incident cost avoided by improving CFR.
///
/// Failed changes avoided × average cost per incident (clinical-system
/// downtime, remediation).
///
/// # Arguments
///
/// * `changes_per_year` — production changes per year.
/// * `cfr_before` — change failure rate before, as a fraction.
/// * `cfr_after` — change failure rate after, as a fraction.
/// * `cost_per_incident` — average cost of a failed change, £.
///
/// # Returns
///
/// Incident cost avoided, £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::failure_cost_avoided;
///
/// // 30 × (0.25 − 0.08) × £15,000 = £76,500/year.
/// assert_eq!(failure_cost_avoided(30.0, 0.25, 0.08, 15_000.0), 76_500.0);
/// ```
pub fn failure_cost_avoided(
    changes_per_year: f64,
    cfr_before: f64,
    cfr_after: f64,
    cost_per_incident: f64,
) -> f64 {
    failed_changes_avoided(changes_per_year, cfr_before, cfr_after) * cost_per_incident
}

/// Downtime harm of an outage: MTTR (hours) × harm per hour.
///
/// Harm per hour bundles lost clinical activity and safety exposure — the
/// health-economics reading of recovery time.
///
/// # Arguments
///
/// * `mttr_hours` — mean/median time to restore, in hours.
/// * `harm_per_hour` — £ (or other harm unit) per hour of downtime.
///
/// # Returns
///
/// Total harm per outage, in the harm unit.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::downtime_harm;
///
/// // MTTR 2 days (48h) vs 2 hours at a fixed harm rate: 24× less harm.
/// let before = downtime_harm(48.0, 1_000.0);
/// let after = downtime_harm(2.0, 1_000.0);
/// assert_eq!(before / after, 24.0);
/// ```
pub fn downtime_harm(mttr_hours: f64, harm_per_hour: f64) -> f64 {
    mttr_hours * harm_per_hour
}

/// Reliability as a benefit discount.
///
/// A service at SLO attainment `a` delivers ≈ a × its modeled benefit — the
/// software analogue of adherence: benefits modeled at 100% availability
/// must be discounted by the availability actually delivered.
///
/// # Arguments
///
/// * `modeled_benefit` — benefit assuming perfect reliability (any unit).
/// * `slo_attainment` — SLO attainment as a fraction (e.g. 0.99).
///
/// # Returns
///
/// The reliability-adjusted benefit, in the same unit as `modeled_benefit`.
///
/// # Examples
///
/// ```rust
/// use health_economics::dora_metrics::reliability_adjusted_benefit;
///
/// // A service at 99% availability delivers ≈ 99% of its modeled benefit.
/// assert!((reliability_adjusted_benefit(1.0, 0.99) - 0.99).abs() < 1e-9);
/// ```
pub fn reliability_adjusted_benefit(modeled_benefit: f64, slo_attainment: f64) -> f64 {
    modeled_benefit * slo_attainment
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: lead time "6 weeks" before, "4 days" after — a
    // "lead-time cut of ~5.4 weeks".
    #[test]
    fn lead_time_cut_from_6_weeks_to_4_days_is_about_5_4_weeks() {
        let reduction = lead_time_reduction_weeks(6.0, days_to_weeks(4.0));
        assert!((reduction - 5.4).abs() < 0.05);
    }

    // Worked example: "30 × 5.4 × 4,000 ≈ £648,000/year of value delivered
    // sooner".
    #[test]
    fn value_pulled_forward_is_about_648k_per_year() {
        // 30 improvements × 5.4 weeks × £4,000/week ≈ £648,000/year
        let value = value_pulled_forward(30.0, 5.4, 4_000.0);
        assert!((value - 648_000.0).abs() < 1e-6);
    }

    // Worked example: "CFR improvement: 30 × (0.25 − 0.08) = ~5 fewer failed
    // changes/year".
    #[test]
    fn cfr_improvement_avoids_about_5_failed_changes_per_year() {
        // 30 × (0.25 − 0.08) = 5.1 ≈ ~5 fewer failed changes/year
        let avoided = failed_changes_avoided(30.0, 0.25, 0.08);
        assert!((avoided - 5.0).abs() < 0.2);
    }

    // Worked example: "~5 fewer failed changes/year × £15,000 average
    // incident cost ... = £76,500/year".
    #[test]
    fn cfr_improvement_saves_76_500_per_year() {
        // 30 × (0.25 − 0.08) × £15,000 = £76,500/year
        let saved = failure_cost_avoided(30.0, 0.25, 0.08, 15_000.0);
        assert!((saved - 76_500.0).abs() < 1e-6);
    }

    // Worked example table: "CFR 25% (before), 8% (after)".
    #[test]
    fn change_failure_rate_matches_before_and_after() {
        // Before: 25% of changes fail; after: 8%.
        assert!((change_failure_rate_percent(25.0, 100.0).unwrap() - 25.0).abs() < 1e-9);
        assert!((change_failure_rate_percent(8.0, 100.0).unwrap() - 8.0).abs() < 1e-9);
        assert!(change_failure_rate_percent(1.0, 0.0).is_none());
    }

    // Translation table: "a service at 99% availability delivers ≈ 0.99 of
    // its modeled benefit — the software analogue of adherence".
    #[test]
    fn service_at_99_percent_availability_delivers_99_percent_of_benefit() {
        assert!((reliability_adjusted_benefit(1.0, 0.99) - 0.99).abs() < 1e-9);
    }

    // Worked example table: "MTTR 2 days (before), 2 hours (after)".
    #[test]
    fn downtime_harm_scales_with_mttr() {
        // MTTR 2 days (48h) vs 2 hours at a fixed harm rate: 24× less harm.
        let before = downtime_harm(48.0, 1_000.0);
        let after = downtime_harm(2.0, 1_000.0);
        assert!((before / after - 24.0).abs() < 1e-9);
    }

    // Worked example table: "Deploys monthly (before), weekly (after)".
    #[test]
    fn deployment_frequency_moves_from_monthly_to_weekly() {
        // 12/year → 52/year
        assert!((deployment_frequency(12.0, 1.0).unwrap() - 12.0).abs() < 1e-9);
        assert!((deployment_frequency(52.0, 1.0).unwrap() - 52.0).abs() < 1e-9);
        assert!(deployment_frequency(1.0, 0.0).is_none());
    }
}
