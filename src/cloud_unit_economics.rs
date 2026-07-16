//! # Cloud Unit Economics (FinOps)
//!
//! Cloud unit economics translate raw cloud spend into **cost per unit of
//! output** — per customer, per transaction, per case resolved, per token.
//! It is the FinOps capability that turns "our AWS bill is £400k/month" into
//! "serving one patient costs £0.83."
//!
//! Two families of unit: resource-efficiency units (cost/GB stored,
//! cost/vCPU-hour, cost/token, cost/build-minute) and business units
//! (cost/customer, cost/transaction, cost/consultation, cost/case-resolved).
//! Marginal-vs-average discipline applies: committed/reserved spend makes
//! marginal unit cost ≈ 0 until the next commitment step — price expansion
//! decisions at marginal, efficiency trends at average.
//!
//! ## Formula
//!
//! ```text
//! Unit cost = total allocated cost (incl. shared/platform costs) / units delivered
//!
//! total allocated cost — the period's spend including the allocated share of
//!                        platform/security/on-call costs
//! units delivered      — output units in the same period (episodes, transactions,
//!                        tokens, customers)
//! ```
//!
//! ## Why it matters
//!
//! Total spend numbers can't answer the questions that matter: is the
//! product getting more or less efficient? Does growth improve or destroy
//! margin? What should we charge? Unit costs answer all three. For digital
//! health specifically, "cost per case resolved" *is* a health-service unit
//! cost — directly comparable to the National Cost Collection figures a
//! commissioner uses for every other service (telephone triage ≈ £8–12/call,
//! GP consultation ≈ £42), which makes it the natural language for pricing
//! digital pathways against traditional ones.
//!
//! ## Example
//!
//! A digital triage service: cloud spend £62,000/month (compute £30k, data
//! £18k, shared platform allocation £14k), handling 380,000 triage
//! episodes/month → ≈ £0.163 per episode, ~2% of the cheapest human
//! alternative, and down from £0.21/episode at 240k episodes last year.
//!
//! ```rust
//! use health_economics::cloud_unit_economics::{
//!     CloudSpend, unit_cost, unit_cost_change, unit_cost_ratio,
//! };
//!
//! // £62,000/month fully-allocated spend.
//! let spend = CloudSpend { compute: 30_000.0, data: 18_000.0, shared_platform: 14_000.0 };
//! assert!((spend.total() - 62_000.0).abs() < 1e-9);
//!
//! // Average cost per episode = 62,000 / 380,000 ≈ £0.163.
//! let episode = unit_cost(spend.total(), 380_000.0).unwrap();
//! assert!((episode - 0.163).abs() < 5e-4);
//!
//! // ~2% of the cheapest human alternative (telephone triage ≈ £8/call).
//! let ratio = unit_cost_ratio(episode, 8.0).unwrap();
//! assert!((ratio - 0.02).abs() < 0.005);
//!
//! // Trend check: £0.21/episode last year → improving scale economics.
//! let change = unit_cost_change(0.21, episode).unwrap();
//! assert!(change < 0.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Unit economics is where engineering choices become finance-legible: an
//!   architecture that halves cost-per-episode is a pricing advantage; one
//!   that scales super-linearly is a time bomb visible only in this metric.
//! - **Publish the allocation rules**: shared costs distorted per-unit
//!   figures until PLICS standardized patient-level costing — your
//!   platform-cost allocation needs the same rigor.
//! - **Pick units the buyer thinks in**: commissioners buy episodes, not
//!   vCPUs.
//! - Feed unit costs into every ICER and budget-impact model as the
//!   authoritative cost denominator.
//! - For AI features, the unit is the token (inference unit economics).
//!
//! ## Pitfalls
//!
//! - **Ignoring shared costs**: unit costs excluding platform/security/
//!   on-call allocations understate by 30–50% and collapse on audit.
//! - **Vanity denominators**: "cost per API call" flatters; "cost per
//!   completed patient episode" informs.
//! - **Average-cost pricing of marginal decisions**: charging teams average
//!   unit cost for usage that is marginally free drives waste-avoidance
//!   theater.
//!
//! ## Sources
//!
//! - FinOps Foundation, unit economics.
//!   <https://www.finops.org/framework/capabilities/unit-economics/>
//! - FinOps Foundation, introduction to cloud unit economics.
//!   <https://www.finops.org/wg/introduction-cloud-unit-economics/>
//!
//! Topic doc: health-economics-metrics/topics/cloud-unit-economics.md

/// A period's fully-allocated cloud spend, including the shared-platform
/// allocation that unit costs must not omit.
///
/// All three lines are in the same currency for the same period (typically a
/// month). Omitting the shared line understates unit costs by 30–50%.
#[derive(Debug, Clone, Copy)]
pub struct CloudSpend {
    /// Compute spend for the period.
    pub compute: f64,
    /// Data (storage/transfer) spend for the period.
    pub data: f64,
    /// Allocated share of platform/security/on-call costs for the period.
    pub shared_platform: f64,
}

impl CloudSpend {
    /// Total allocated cost: compute + data + shared platform allocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cloud_unit_economics::CloudSpend;
    ///
    /// // Worked example: £30k compute + £18k data + £14k shared = £62k/month.
    /// let spend = CloudSpend { compute: 30_000.0, data: 18_000.0, shared_platform: 14_000.0 };
    /// assert!((spend.total() - 62_000.0).abs() < 1e-9);
    /// ```
    pub fn total(&self) -> f64 {
        self.compute + self.data + self.shared_platform
    }
}

/// Unit cost = total allocated cost / units delivered.
///
/// The numerator must include shared/platform allocations; the denominator
/// should be a unit the buyer thinks in (episodes, not vCPUs). Both refer to
/// the same period.
///
/// # Arguments
///
/// * `total_allocated_cost` — the period's fully-allocated spend, in currency.
/// * `units_delivered` — output units delivered in the same period.
///
/// # Returns
///
/// `Some(cost per unit)`, or `None` when `units_delivered` is zero (unit
/// cost undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::cloud_unit_economics::unit_cost;
///
/// // £62,000 / 380,000 episodes ≈ £0.163 per triage episode.
/// let cost = unit_cost(62_000.0, 380_000.0).unwrap();
/// assert!((cost - 0.163).abs() < 5e-4);
/// assert!(unit_cost(62_000.0, 0.0).is_none());
/// ```
pub fn unit_cost(total_allocated_cost: f64, units_delivered: f64) -> Option<f64> {
    if units_delivered == 0.0 {
        None
    } else {
        Some(total_allocated_cost / units_delivered)
    }
}

/// Ratio of one unit cost to another (e.g. digital episode vs cheapest human
/// alternative).
///
/// Dimensionless; below 1.0 means option A is cheaper per unit than the
/// reference B.
///
/// # Arguments
///
/// * `unit_cost_a` — the unit cost being compared (e.g. digital episode).
/// * `unit_cost_b` — the reference unit cost (e.g. telephone triage call).
///
/// # Returns
///
/// `Some(a / b)`, or `None` when the reference cost is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::cloud_unit_economics::unit_cost_ratio;
///
/// // £0.163 digital episode vs £8 telephone triage → ~2%.
/// let ratio = unit_cost_ratio(62_000.0 / 380_000.0, 8.0).unwrap();
/// assert!((ratio - 0.02).abs() < 0.005);
/// assert!(unit_cost_ratio(1.0, 0.0).is_none());
/// ```
pub fn unit_cost_ratio(unit_cost_a: f64, unit_cost_b: f64) -> Option<f64> {
    if unit_cost_b == 0.0 { None } else { Some(unit_cost_a / unit_cost_b) }
}

/// Fractional change in unit cost between periods:
/// (current − previous) / previous.
///
/// Negative means improving scale economics (fixed platform costs
/// amortizing over more units) — worth a headline in the QBR.
///
/// # Arguments
///
/// * `previous` — unit cost in the earlier period.
/// * `current` — unit cost in the later period, same currency and unit.
///
/// # Returns
///
/// `Some(fractional change)`, or `None` when `previous` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::cloud_unit_economics::unit_cost_change;
///
/// // £0.21/episode last year → ≈ £0.163 now: unit cost falling ~22%.
/// let change = unit_cost_change(0.21, 62_000.0 / 380_000.0).unwrap();
/// assert!(change < 0.0);
/// assert!(unit_cost_change(0.0, 1.0).is_none());
/// ```
pub fn unit_cost_change(previous: f64, current: f64) -> Option<f64> {
    if previous == 0.0 {
        None
    } else {
        Some((current - previous) / previous)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: digital triage service, £62,000/month cloud spend
    // (compute £30k, data £18k, shared platform £14k), 380,000 episodes/month.

    fn spend() -> CloudSpend {
        CloudSpend { compute: 30_000.0, data: 18_000.0, shared_platform: 14_000.0 }
    }

    // Worked-example setup: "cloud spend £62,000/month (compute £30k, data
    // £18k, shared platform allocation £14k)".
    #[test]
    fn monthly_total_allocated_cost_is_62_000() {
        assert!((spend().total() - 62_000.0).abs() < 1e-9);
    }

    // Worked-example line: "Average cost per episode = 62,000 / 380,000 ≈
    // £0.163".
    #[test]
    fn average_cost_per_episode_is_about_0_163() {
        let cost = unit_cost(spend().total(), 380_000.0).unwrap();
        assert!((cost - 62_000.0 / 380_000.0).abs() < 1e-12);
        assert!((cost - 0.163).abs() < 5e-4);
    }

    // Worked-example line: "digital episode runs at ~2% of the cheapest
    // human alternative" (telephone triage ≈ £8–12/call).
    #[test]
    fn digital_episode_is_about_2_percent_of_cheapest_human_alternative() {
        // Telephone triage ≈ £8–12/call; against the cheapest (£8) the
        // digital episode runs at ~2%.
        let episode = unit_cost(62_000.0, 380_000.0).unwrap();
        let ratio = unit_cost_ratio(episode, 8.0).unwrap();
        assert!((ratio - 0.02).abs() < 0.005);
    }

    // Worked-example line: "last year £0.21/episode at 240k episodes →
    // improving scale economics".
    #[test]
    fn trend_shows_improving_scale_economics() {
        // Last year £0.21/episode at 240k episodes; now ≈ £0.163 at 380k.
        let last_year = unit_cost(0.21 * 240_000.0, 240_000.0).unwrap();
        assert!((last_year - 0.21).abs() < 1e-9);
        let now = unit_cost(62_000.0, 380_000.0).unwrap();
        let change = unit_cost_change(last_year, now).unwrap();
        assert!(change < 0.0, "unit cost should be falling as volume grows");
    }

    // Edge cases: every ratio in the module is undefined on a zero
    // denominator.
    #[test]
    fn zero_denominators_are_undefined() {
        assert!(unit_cost(62_000.0, 0.0).is_none());
        assert!(unit_cost_ratio(1.0, 0.0).is_none());
        assert!(unit_cost_change(0.0, 1.0).is_none());
    }
}
