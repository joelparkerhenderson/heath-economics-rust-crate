//! # Cost-Consequence Analysis (CCA)
//!
//! CCA presents costs alongside a **disaggregated table of all outcomes** —
//! clinical, operational, experiential — without collapsing them into a
//! single ratio or score. The decision-maker weighs the trade-offs
//! explicitly.
//!
//! There is deliberately no aggregation formula. Each row keeps its own
//! units; qualitative outcomes are described, not scored. Rules: every
//! consequence pre-specified (no cherry-picking after results); same
//! perspective and horizon throughout; uncertainty per row.
//!
//! ## Formula
//!
//! ```text
//!                           Intervention   Comparator   Difference
//! Costs (annual)            £X             £Y           ΔC  = X − Y
//! Outcome i (natural units) a_i            b_i          Δ_i = a_i − b_i
//! Qualitative outcomes      described, not scored
//!
//! ΔC  — incremental cost of the intervention (positive = costs more)
//! Δ_i — per-row difference, in that row's own natural units
//! ```
//!
//! ## Why it matters
//!
//! CCA is NICE's **preferred economic format for most digital health
//! technologies** under the Evidence Standards Framework. Digital products
//! produce heterogeneous effects (time saved, satisfaction, DNA reduction,
//! small clinical gains) that resist honest aggregation into one QALY
//! number. Rather than force a fragile composite, CCA shows the full
//! ledger. For most software business cases, it is both the most honest
//! format and the most persuasive one, because every stakeholder can find
//! their own decision-relevant row.
//!
//! ## Example
//!
//! Digital pre-operative assessment platform vs phone-based process, per
//! year, one trust: +£85,000 running cost buys 5,600 nurse-hours
//! (≈ £15/hour), 82 avoided on-day cancellations (each wasting a theatre
//! slot worth ~£1,200), +0.6 CSAT, and −3.6pp lost assessments.
//!
//! ```rust
//! use health_economics::cost_consequence_analysis::{
//!     ConsequenceRow, CostConsequenceTable, cost_per_unit_gained, value_of_avoided_events,
//! };
//!
//! let row = |name: &str, digital: f64, phone: f64| ConsequenceRow {
//!     name: name.to_string(),
//!     intervention: digital,
//!     comparator: phone,
//! };
//! let table = CostConsequenceTable {
//!     cost: row("Running cost", 180_000.0, 95_000.0),
//!     consequences: vec![
//!         row("Nurse hours on assessments", 6_200.0, 11_800.0),
//!         row("On-day surgery cancellations", 92.0, 174.0),
//!         row("Patient satisfaction (CSAT)", 4.5, 3.9),
//!         row("Assessments lost/incomplete (%)", 1.2, 4.8),
//!     ],
//! };
//!
//! // Cost row: £180,000 − £95,000 = +£85,000.
//! assert!((table.incremental_cost() - 85_000.0).abs() < 1e-6);
//!
//! // Nurse hours: 6,200 − 11,800 = −5,600 hrs; cancellations: −82.
//! assert!((table.consequences[0].difference() - (-5_600.0)).abs() < 1e-9);
//! assert!((table.consequences[1].difference() - (-82.0)).abs() < 1e-9);
//!
//! // £85,000 buys 5,600 nurse-hours ≈ £15/hour, far below any staffing cost.
//! let per_hour = cost_per_unit_gained(table.incremental_cost(), 5_600.0).unwrap();
//! assert!((per_hour - 15.0).abs() < 0.2);
//!
//! // 82 avoided cancellations × ~£1,200 theatre slot = £98,400.
//! assert!((value_of_avoided_events(82.0, 1_200.0) - 98_400.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - CCA is the formal version of the balanced scorecard a good platform
//!   proposal already uses: cost next to DORA metrics, DevEx scores,
//!   incident counts — unaggregated.
//! - **Pre-specify the rows**: decide what counts before the pilot, so you
//!   can't quietly drop the metric that got worse.
//! - **Show unfavorable rows** — a CCA with only good news is marketing.
//! - Use CCA when no defensible composite exists, which for developer
//!   tooling is almost always.
//!
//! ## Pitfalls
//!
//! - **Cherry-picked consequences** — the format's integrity depends on
//!   pre-specification.
//! - **Smuggled aggregation**: color-coding or "overall scores" reintroduce
//!   the arbitrary weights CCA exists to avoid.
//! - **Decision paralysis**: CCA needs a decision-maker willing to weigh
//!   trade-offs; pair it with a recommendation and the reasoning.
//!
//! ## Sources
//!
//! - NICE Evidence Standards Framework for digital health technologies (ECD7).
//!   <https://www.nice.org.uk/corporate/ecd7>
//! - ESF evidence standards tables.
//!   <https://www.nice.org.uk/corporate/ecd7/chapter/section-c-evidence-standards-tables>
//!
//! Topic doc: health-economics-metrics/topics/cost-consequence-analysis.md

/// One pre-specified row of a cost-consequence table, in its own units.
///
/// A row may be a cost (£), a workload (hours), an event count, a score, or
/// a percentage — each row keeps its own natural units and is never combined
/// with the others.
#[derive(Debug, Clone)]
pub struct ConsequenceRow {
    /// What this row measures (e.g. "Nurse hours on assessments").
    pub name: String,
    /// Value under the intervention.
    pub intervention: f64,
    /// Value under the comparator.
    pub comparator: f64,
}

impl ConsequenceRow {
    /// Difference column: intervention − comparator, in the row's own units.
    ///
    /// Sign follows the row's units: negative is an improvement for
    /// "lower-is-better" rows (hours, cancellations, loss rates), positive
    /// for "higher-is-better" rows (satisfaction).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cost_consequence_analysis::ConsequenceRow;
    ///
    /// // Nurse hours: 6,200 digital vs 11,800 phone → −5,600 hrs.
    /// let row = ConsequenceRow {
    ///     name: "Nurse hours on assessments".to_string(),
    ///     intervention: 6_200.0,
    ///     comparator: 11_800.0,
    /// };
    /// assert!((row.difference() - (-5_600.0)).abs() < 1e-9);
    /// ```
    pub fn difference(&self) -> f64 {
        self.intervention - self.comparator
    }
}

/// A full CCA table: a cost row plus every pre-specified consequence row,
/// kept disaggregated.
///
/// Rows must share the same perspective and horizon; consequences must be
/// pre-specified before results are known, or the format's integrity is
/// lost.
#[derive(Debug, Clone)]
pub struct CostConsequenceTable {
    /// Annual (or per-period) cost row.
    pub cost: ConsequenceRow,
    /// Quantitative outcome rows, each in natural units.
    pub consequences: Vec<ConsequenceRow>,
}

impl CostConsequenceTable {
    /// Incremental cost of the intervention (ΔC of the cost row).
    ///
    /// Positive means the intervention costs more than the comparator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cost_consequence_analysis::{
    ///     ConsequenceRow, CostConsequenceTable,
    /// };
    ///
    /// // Running cost £180,000 digital vs £95,000 phone → ΔC = +£85,000.
    /// let table = CostConsequenceTable {
    ///     cost: ConsequenceRow {
    ///         name: "Running cost".to_string(),
    ///         intervention: 180_000.0,
    ///         comparator: 95_000.0,
    ///     },
    ///     consequences: vec![],
    /// };
    /// assert!((table.incremental_cost() - 85_000.0).abs() < 1e-6);
    /// ```
    pub fn incremental_cost(&self) -> f64 {
        self.cost.difference()
    }
}

/// Informal reading aid for one row of the table: what the incremental cost
/// buys per unit of a single improved outcome (e.g. £ per nurse-hour freed).
///
/// Not an aggregation — a per-row sanity check that helps a committee reason
/// ("£85,000 buys 5,600 nurse-hours ≈ £15/hour, far below any staffing
/// cost").
///
/// # Arguments
///
/// * `incremental_cost` — ΔC of the cost row, in currency.
/// * `units_gained` — improvement in one outcome row, in that row's units
///   (sign-flipped for lower-is-better rows).
///
/// # Returns
///
/// `Some(cost per unit)`, or `None` when `units_gained` is zero (nothing
/// was gained on this row).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_consequence_analysis::cost_per_unit_gained;
///
/// // £85,000 / 5,600 freed nurse-hours ≈ £15/hour.
/// let per_hour = cost_per_unit_gained(85_000.0, 5_600.0).unwrap();
/// assert!((per_hour - 15.0).abs() < 0.2);
/// assert!(cost_per_unit_gained(85_000.0, 0.0).is_none());
/// ```
pub fn cost_per_unit_gained(incremental_cost: f64, units_gained: f64) -> Option<f64> {
    if units_gained == 0.0 { None } else { Some(incremental_cost / units_gained) }
}

/// Value of avoided events at a unit value (e.g. avoided on-day
/// cancellations × theatre-slot value).
///
/// Another per-row reading aid: monetizes a single event-count row without
/// aggregating the table.
///
/// # Arguments
///
/// * `events_avoided` — number of events avoided (a positive count).
/// * `value_per_event` — value of each avoided event, in currency (~£1,200
///   per wasted theatre slot in the worked example).
///
/// # Returns
///
/// The total value of the avoided events, in currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_consequence_analysis::value_of_avoided_events;
///
/// // 82 avoided cancellations × £1,200 = £98,400.
/// assert!((value_of_avoided_events(82.0, 1_200.0) - 98_400.0).abs() < 1e-6);
/// ```
pub fn value_of_avoided_events(events_avoided: f64, value_per_event: f64) -> f64 {
    events_avoided * value_per_event
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: digital pre-operative assessment platform vs
    // phone-based process, per year, one trust.

    fn row(name: &str, digital: f64, phone: f64) -> ConsequenceRow {
        ConsequenceRow { name: name.to_string(), intervention: digital, comparator: phone }
    }

    fn table() -> CostConsequenceTable {
        CostConsequenceTable {
            cost: row("Running cost", 180_000.0, 95_000.0),
            consequences: vec![
                row("Nurse hours on assessments", 6_200.0, 11_800.0),
                row("On-day surgery cancellations", 92.0, 174.0),
                row("Patient satisfaction (CSAT)", 4.5, 3.9),
                row("Assessments lost/incomplete (%)", 1.2, 4.8),
            ],
        }
    }

    // Table row: "Running cost £180,000 £95,000 +£85,000".
    #[test]
    fn running_cost_difference_is_plus_85_000() {
        assert!((table().incremental_cost() - 85_000.0).abs() < 1e-6);
    }

    // Table row: "Nurse hours on assessments 6,200 11,800 −5,600 hrs".
    #[test]
    fn nurse_hours_difference_is_minus_5_600() {
        assert!((table().consequences[0].difference() - (-5_600.0)).abs() < 1e-9);
    }

    // Table row: "On-day surgery cancellations 92 174 −82".
    #[test]
    fn cancellations_difference_is_minus_82() {
        assert!((table().consequences[1].difference() - (-82.0)).abs() < 1e-9);
    }

    // Table row: "Patient satisfaction (CSAT) 4.5/5 3.9/5 +0.6".
    #[test]
    fn satisfaction_difference_is_plus_0_6() {
        assert!((table().consequences[2].difference() - 0.6).abs() < 1e-9);
    }

    // Table row: "Assessments lost/incomplete 1.2% 4.8% −3.6 pp".
    #[test]
    fn lost_assessments_difference_is_minus_3_6_points() {
        assert!((table().consequences[3].difference() - (-3.6)).abs() < 1e-9);
    }

    // Worked-example line: "£85,000 buys 5,600 nurse-hours (≈ £15/hour, far
    // below any staffing cost)".
    #[test]
    fn extra_cost_buys_nurse_hours_at_about_15_per_hour() {
        // £85,000 buys 5,600 nurse-hours ≈ £15/hour.
        let hours_freed = -table().consequences[0].difference();
        let per_hour = cost_per_unit_gained(table().incremental_cost(), hours_freed).unwrap();
        assert!((per_hour - 15.0).abs() < 0.2);
    }

    // Worked-example line: "82 avoided cancellations (each wasting a theatre
    // slot worth ~£1,200)".
    #[test]
    fn avoided_cancellations_valued_at_theatre_slot_worth() {
        // 82 avoided cancellations, each wasting a theatre slot worth ~£1,200.
        let value = value_of_avoided_events(82.0, 1_200.0);
        assert!((value - 98_400.0).abs() < 1e-6);
    }

    // Edge case: the per-row reading aid is undefined when nothing was
    // gained on the row.
    #[test]
    fn cost_per_unit_gained_undefined_for_zero_gain() {
        assert!(cost_per_unit_gained(85_000.0, 0.0).is_none());
    }
}
