//! # WSJF and CD3
//!
//! CD3 (Cost of Delay Divided by Duration) and WSJF (Weighted Shortest Job
//! First) are prioritization rules that schedule work by **value density**:
//! how much delay cost is removed per unit of scarce capacity consumed.
//! Under a shared, fixed capacity, highest-CD3-first is the mathematically
//! optimal sequence for minimizing total delay cost.
//!
//! CD3 uses real currency (Black Swan Farming); WSJF is SAFe's
//! relative-scale proxy with modified-Fibonacci scores. CD3 with genuine
//! currency is strictly stronger than WSJF's unitless points — WSJF is to
//! CD3 what multi-criteria scoring is to full cost-utility analysis: usable
//! when monetization is impractical, gameable when the scores have no
//! anchor.
//!
//! ## Formula
//!
//! ```text
//! CD3  = Cost of Delay (£/week) / Duration (weeks)
//!
//! WSJF = (user-business value + time criticality
//!         + risk reduction/opportunity enablement) / job size
//!
//! where:
//!   Cost of Delay — value lost per week the item waits (£/week)
//!   Duration      — calendar time the item occupies the constrained
//!                   capacity (weeks), not effort
//!   WSJF numerator terms and job size — relative modified-Fibonacci scores
//!                   (1, 2, 3, 5, 8, 13, 20), unitless
//! ```
//!
//! ## Why it matters
//!
//! Every backlog is a rationing problem: many worthy items, one pipeline.
//! Health economics solved the same problem for health budgets with
//! cost-effectiveness league tables — rank interventions by health gained
//! per pound, fund down the list until the budget exhausts. CD3 is the
//! identical logic for delivery capacity: benefit per unit of the
//! *constrained resource*, funded in rank order. Getting sequencing right is
//! free money — in the worked example the same three features on the same
//! team cost £100k in delay when sequenced by CD3 versus £180k when
//! sequenced biggest-CoD-first: sequencing alone saves £80,000.
//!
//! ## Example
//!
//! Three features, one team: A (CoD £30,000/wk, 10 wks), B (£12,000/wk,
//! 2 wks), C (£5,000/wk, 1 wk).
//!
//! ```rust
//! use health_economics::wsjf_and_cd3::{
//!     Feature, sequence_by_cd3, total_delay_cost, sequencing_savings,
//! };
//!
//! let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
//! let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
//! let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
//!
//! // CD3 scores: A = 3,000; B = 6,000; C = 5,000 → CD3 order is B, C, A.
//! assert_eq!(a.cd3(), Some(3_000.0));
//! assert_eq!(b.cd3(), Some(6_000.0));
//! assert_eq!(c.cd3(), Some(5_000.0));
//! assert_eq!(sequence_by_cd3(&[a, b, c]), vec![b, c, a]);
//!
//! // CD3 order (B,C,A): A waits 3 wks, C waits 2 → 30k×3 + 5k×2 = £100k.
//! assert_eq!(total_delay_cost(&[b, c, a]), 100_000.0);
//! // CoD order (A,B,C): B waits 10, C waits 12 → 12k×10 + 5k×12 = £180k.
//! assert_eq!(total_delay_cost(&[a, b, c]), 180_000.0);
//!
//! // Same features, same team — sequencing alone saves £80,000.
//! assert_eq!(sequencing_savings(&[a, b, c]), 80_000.0);
//! ```
//!
//! The intuition: small, urgent items go first because they release their
//! delay cost cheaply; the big item loses little by waiting briefly.
//!
//! ## Software engineering connection
//!
//! - For healthcare software portfolios, denominate CoD in QALYs/week ×
//!   threshold + operational £/week, and the backlog becomes directly
//!   commensurable with how the health system ranks everything else it buys.
//! - Duration means *calendar time occupying the constraint*, not effort —
//!   a 2-week elapsed item that needs 2 days of the bottleneck team is
//!   cheaper than it looks.
//! - Hospitals run the same rule implicitly when they order theatre lists by
//!   urgency-weighted throughput — clinical prioritization categories are
//!   severity-weighted CD3.
//!
//! ## Pitfalls
//!
//! - **WSJF score theater**: unitless Fibonacci debates converge on whoever
//!   argues loudest; anchor at least the top-of-backlog items in real CoD.
//! - **Duration gaming**: splitting items to inflate CD3 rank — fine when
//!   splits deliver value independently, fraud when they don't.
//! - **Ignoring urgency profiles**: deadline-shaped CoD (regulatory dates)
//!   breaks the steady-rate assumption; schedule those by date feasibility,
//!   then CD3 the rest.
//! - **Re-ranking churn**: CD3 is for sequencing decisions at commitment
//!   time, not for daily reshuffling of in-flight work.
//!
//! ## Sources
//!
//! - Black Swan Farming, CD3 and WSJF.
//!   <https://blackswanfarming.com/wsjf-weighted-shortest-job-first/>
//! - SAFe, WSJF. <https://framework.scaledagile.com/wsjf>
//! - Reinertsen DG, *The Principles of Product Development Flow*.
//!
//! Topic doc: health-economics-metrics/topics/wsjf-and-cd3.md

/// A backlog item with a real-currency cost of delay and a duration on the
/// constrained capacity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Feature {
    /// Cost of delay while the item waits (e.g. £/week): the value lost per
    /// week this item is not delivered.
    pub cost_of_delay_per_week: f64,
    /// Calendar time the item occupies the constraint (weeks) — not effort:
    /// a 2-week elapsed item that needs 2 days of the bottleneck team is
    /// cheaper than it looks.
    pub duration_weeks: f64,
}

impl Feature {
    /// The item's CD3 score: cost of delay divided by duration.
    ///
    /// # Returns
    ///
    /// `Some(cd3)` in £/week², i.e. delay cost removed per week of the
    /// constraint consumed; `None` when the item's duration is zero
    /// (the ratio is undefined).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::wsjf_and_cd3::Feature;
    ///
    /// // Feature B from the worked example: 12,000 / 2 = 6,000.
    /// let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
    /// assert_eq!(b.cd3(), Some(6_000.0));
    /// ```
    pub fn cd3(&self) -> Option<f64> {
        cd3(self.cost_of_delay_per_week, self.duration_weeks)
    }
}

/// CD3 (Cost of Delay Divided by Duration) — value density in real units.
///
/// # Arguments
///
/// * `cost_of_delay_per_week` — value lost per week the item waits (£/week).
/// * `duration_weeks` — calendar time the item occupies the constrained
///   capacity (weeks), not effort.
///
/// # Returns
///
/// `Some(cd3)` — the value-density score used for ranking; `None` when
/// `duration_weeks` is zero (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::wsjf_and_cd3::cd3;
///
/// // Worked example scores: A = 30,000/10 = 3,000; B = 12,000/2 = 6,000;
/// // C = 5,000/1 = 5,000.
/// assert_eq!(cd3(30_000.0, 10.0), Some(3_000.0));
/// assert_eq!(cd3(12_000.0, 2.0), Some(6_000.0));
/// assert_eq!(cd3(5_000.0, 1.0), Some(5_000.0));
///
/// // Zero duration: undefined.
/// assert_eq!(cd3(10_000.0, 0.0), None);
/// ```
pub fn cd3(cost_of_delay_per_week: f64, duration_weeks: f64) -> Option<f64> {
    if duration_weeks == 0.0 {
        None
    } else {
        Some(cost_of_delay_per_week / duration_weeks)
    }
}

/// SAFe's WSJF proxy: relative-scale value density.
///
/// (user-business value + time criticality + risk reduction/opportunity
/// enablement) / job size, all on relative modified-Fibonacci scales
/// (1, 2, 3, 5, 8, 13, 20). Unitless and gameable when the scores have no
/// anchor — prefer CD3 with real currency where monetization is practical.
///
/// # Arguments
///
/// * `user_business_value` — relative value to users/business (Fibonacci
///   score).
/// * `time_criticality` — relative urgency/decay of the value (Fibonacci
///   score).
/// * `risk_reduction_opportunity_enablement` — relative risk reduction or
///   opportunity enablement (Fibonacci score).
/// * `job_size` — relative size of the job (Fibonacci score), the proxy for
///   duration.
///
/// # Returns
///
/// `Some(wsjf)` — the unitless priority score (higher first); `None` when
/// `job_size` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::wsjf_and_cd3::wsjf;
///
/// // (8 + 5 + 3) / 2 = 8.
/// assert_eq!(wsjf(8.0, 5.0, 3.0, 2.0), Some(8.0));
///
/// // Zero job size: undefined.
/// assert_eq!(wsjf(8.0, 5.0, 3.0, 0.0), None);
/// ```
pub fn wsjf(
    user_business_value: f64,
    time_criticality: f64,
    risk_reduction_opportunity_enablement: f64,
    job_size: f64,
) -> Option<f64> {
    if job_size == 0.0 {
        None
    } else {
        Some(
            (user_business_value + time_criticality + risk_reduction_opportunity_enablement)
                / job_size,
        )
    }
}

/// Total delay cost of executing `features` in the given order on a single
/// shared pipeline.
///
/// Each item accrues its cost of delay for the weeks it waits before
/// starting: Σ CoD_i × start_time_i. (Charging the wait-to-start rather than
/// wait-to-finish shifts every schedule by the same constant Σ CoD_i × d_i,
/// so rankings and savings are unaffected.)
///
/// # Arguments
///
/// * `features` — the backlog items, in execution order.
///
/// # Returns
///
/// Total delay cost (£) of this sequence.
///
/// # Examples
///
/// ```rust
/// use health_economics::wsjf_and_cd3::{Feature, total_delay_cost};
///
/// let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
/// let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
/// let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
///
/// // CD3 order (B,C,A): A waits 3 wks, C waits 2 → 30k×3 + 5k×2 = £100k.
/// assert_eq!(total_delay_cost(&[b, c, a]), 100_000.0);
/// // CoD order (A,B,C): B waits 10, C waits 12 → 12k×10 + 5k×12 = £180k.
/// assert_eq!(total_delay_cost(&[a, b, c]), 180_000.0);
/// ```
pub fn total_delay_cost(features: &[Feature]) -> f64 {
    let mut elapsed_weeks = 0.0;
    let mut cost = 0.0;
    for f in features {
        // This item waited `elapsed_weeks` before starting: charge its CoD
        // for that wait, then it occupies the pipeline for its duration.
        cost += f.cost_of_delay_per_week * elapsed_weeks;
        elapsed_weeks += f.duration_weeks;
    }
    cost
}

/// The features re-ordered highest-CD3-first.
///
/// Under a shared, fixed capacity this is the mathematically optimal
/// sequence for minimizing total delay cost. Items with zero duration
/// (undefined CD3) are placed first: they cost nothing to run immediately.
///
/// # Arguments
///
/// * `features` — the backlog items, in any order.
///
/// # Returns
///
/// A new `Vec<Feature>` sorted by descending CD3 (input is not modified).
///
/// # Examples
///
/// ```rust
/// use health_economics::wsjf_and_cd3::{Feature, sequence_by_cd3};
///
/// let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
/// let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
/// let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
///
/// // CD3 scores 3,000 / 6,000 / 5,000 → order is B, C, A.
/// assert_eq!(sequence_by_cd3(&[a, b, c]), vec![b, c, a]);
/// ```
pub fn sequence_by_cd3(features: &[Feature]) -> Vec<Feature> {
    let mut ordered: Vec<Feature> = features.to_vec();
    ordered.sort_by(|a, b| {
        // Zero-duration items (cd3() == None) rank as +∞: run them first,
        // they consume no capacity.
        let ka = a.cd3().unwrap_or(f64::INFINITY);
        let kb = b.cd3().unwrap_or(f64::INFINITY);
        // Descending order: compare b against a.
        kb.partial_cmp(&ka).unwrap_or(std::cmp::Ordering::Equal)
    });
    ordered
}

/// Delay cost saved by CD3 sequencing relative to executing the features in
/// the order given.
///
/// Computes total_delay_cost(as given) − total_delay_cost(CD3 order).
/// Getting sequencing right is free money — same work, same capacity, less
/// total delay cost.
///
/// # Arguments
///
/// * `features` — the backlog items, in the order currently planned.
///
/// # Returns
///
/// Delay cost saved (£); non-negative when CD3 order is optimal, zero if
/// the given order already is CD3 order.
///
/// # Examples
///
/// ```rust
/// use health_economics::wsjf_and_cd3::{Feature, sequencing_savings};
///
/// let a = Feature { cost_of_delay_per_week: 30_000.0, duration_weeks: 10.0 };
/// let b = Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 };
/// let c = Feature { cost_of_delay_per_week: 5_000.0, duration_weeks: 1.0 };
///
/// // £180k (biggest-CoD-first) − £100k (CD3 order) = £80,000 saved.
/// assert_eq!(sequencing_savings(&[a, b, c]), 80_000.0);
/// ```
pub fn sequencing_savings(features: &[Feature]) -> f64 {
    total_delay_cost(features) - total_delay_cost(&sequence_by_cd3(features))
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Feature = Feature {
        cost_of_delay_per_week: 30_000.0,
        duration_weeks: 10.0,
    };
    const B: Feature = Feature {
        cost_of_delay_per_week: 12_000.0,
        duration_weeks: 2.0,
    };
    const C: Feature = Feature {
        cost_of_delay_per_week: 5_000.0,
        duration_weeks: 1.0,
    };

    // Worked example table: "A 30,000 10 wks 3,000; B 12,000 2 wks 6,000;
    // C 5,000 1 wk 5,000".
    /// CD3 scores: A = 3,000; B = 6,000; C = 5,000.
    #[test]
    fn cd3_scores_are_3000_6000_5000() {
        assert!((A.cd3().unwrap() - 3_000.0).abs() < 1e-9);
        assert!((B.cd3().unwrap() - 6_000.0).abs() < 1e-9);
        assert!((C.cd3().unwrap() - 5_000.0).abs() < 1e-9);
    }

    // Worked example: "CD3 order: B, C, A".
    /// CD3 order is B, C, A (highest value density first).
    #[test]
    fn cd3_order_is_b_c_a() {
        let ordered = sequence_by_cd3(&[A, B, C]);
        assert_eq!(ordered, vec![B, C, A]);
    }

    // Worked example: "CD3 order (B,C,A): A waits 3 wks, C waits 2 →
    // 30k×3 + 5k×2 = £100k delay cost".
    /// CD3 order (B,C,A): A waits 3 wks, C waits 2 → 30k×3 + 5k×2 = £100k.
    #[test]
    fn cd3_order_delay_cost_is_100k() {
        let cost = total_delay_cost(&[B, C, A]);
        assert!((cost - 100_000.0).abs() < 1e-9);
    }

    // Worked example: "CoD order (A,B,C): B waits 10, C waits 12 →
    // 12k×10 + 5k×12 = £180k".
    /// CoD order (A,B,C): B waits 10, C waits 12 → 12k×10 + 5k×12 = £180k.
    #[test]
    fn biggest_cod_first_delay_cost_is_180k() {
        let cost = total_delay_cost(&[A, B, C]);
        assert!((cost - 180_000.0).abs() < 1e-9);
    }

    // Worked example: "Same features, same team — sequencing alone saves
    // £80,000" (£180k − £100k).
    /// Same features, same team — sequencing alone saves £80,000.
    #[test]
    fn sequencing_alone_saves_80k() {
        let savings = sequencing_savings(&[A, B, C]);
        assert!((savings - 80_000.0).abs() < 1e-9);
    }

    // Verifies the doc's WSJF formula: "(user-business value + time
    // criticality + risk reduction/opportunity enablement) / job size".
    /// WSJF proxy: (value + criticality + risk) / size, None on zero size.
    #[test]
    fn wsjf_divides_summed_scores_by_job_size() {
        let score = wsjf(8.0, 5.0, 3.0, 2.0).unwrap();
        assert!((score - 8.0).abs() < 1e-9);
        assert!(wsjf(8.0, 5.0, 3.0, 0.0).is_none());
    }

    // Edge-case contract: CD3 = CoD / duration is undefined at zero
    // duration.
    /// Zero-duration items have no CD3 score.
    #[test]
    fn zero_duration_has_no_cd3() {
        assert!(cd3(10_000.0, 0.0).is_none());
    }
}
