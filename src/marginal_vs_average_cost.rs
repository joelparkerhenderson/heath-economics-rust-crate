//! # Marginal vs Average Cost
//!
//! Average cost is total cost divided by units produced. Marginal cost is the
//! cost of producing one *additional* (or one fewer) unit. Decisions should
//! be made on marginal cost — but published unit costs are almost always
//! averages.
//!
//! Fixed costs make MC < AC for capacity reductions, and MC can approach zero
//! when spare capacity exists.
//!
//! ## Formula
//!
//! ```text
//! Average cost:  AC = TC / Q
//! Marginal cost: MC = dTC/dQ   (cost of one more/one fewer unit)
//!
//! TC = total cost, Q = quantity
//!
//! True saving = ΔQ × MC           (small changes)
//! True saving = step change in TC (large changes crossing a capacity
//!                                  threshold, e.g. closing a ward)
//! ```
//!
//! ## Why it matters
//!
//! The single most common error in digital health business cases is valuing a
//! saved resource at its **average** cost when the real saving is the
//! **marginal** cost. A hospital bed day has an average (fully absorbed) cost
//! of £400+, but freeing one bed day does not save £400 — the building,
//! heating, and most staffing costs continue. The cash actually released may
//! be £50–£150 unless enough beds are freed to close a ward.
//!
//! ## Example
//!
//! Software reduces average length of stay, freeing 1,000 bed days/year at a
//! trust. Same intervention, three defensible numbers, depending on whether
//! the change crosses a capacity step.
//!
//! ```rust
//! use health_economics::marginal_vs_average_cost::{
//!     crosses_capacity_step, marginal_saving, naive_average_cost_saving,
//!     step_change_saving, ward_bed_days_per_year,
//! };
//!
//! // Naive claim: 1,000 × £400 average cost = £400,000 "saved". Wrong.
//! assert!((naive_average_cost_saving(1_000.0, 400.0) - 400_000.0).abs() < 1e-9);
//!
//! // Marginal claim: variable cost ≈ £120/bed day → 1,000 × £120 = £120,000.
//! assert!((marginal_saving(1_000.0, 120.0) - 120_000.0).abs() < 1e-9);
//!
//! // Step-change claim: 7,300 bed days/year (a 20-bed ward) crosses the
//! // capacity step, so closing the ward releases ≈ £1.5M/year of real cash.
//! let step = ward_bed_days_per_year(20.0);
//! assert!((step - 7_300.0).abs() < 1e-9);
//! assert!(!crosses_capacity_step(1_000.0, step));
//! assert!(crosses_capacity_step(7_300.0, step));
//! assert!((step_change_saving(10_000_000.0, 8_500_000.0) - 1_500_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Cloud economics is native marginal-cost territory.
//! - The marginal cost of one more CI run on already-reserved capacity is
//!   ≈ £0, while the average cost per run (total platform spend ÷ runs) may
//!   be pounds.
//! - Chargeback systems that bill average cost drive teams to under-use
//!   shared capacity that is actually free at the margin.
//! - Conversely, "we saved 30% of compute" only releases cash if instances
//!   are actually terminated or reservations reduced — the software version
//!   of the bed-day trap.
//!
//! ## Pitfalls
//!
//! - **Valuing capacity at average cost** and presenting it as cash (the
//!   classic).
//! - **Assuming marginal cost is constant.** It steps at capacity boundaries
//!   (ward closures, license tiers, reserved-instance commitments).
//! - **Using marginal cost for expansion decisions but average for
//!   contraction** in the same case — pick per the actual decision.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: marginal cost.
//!   <https://yhec.co.uk/glossary/marginal-cost/>
//! - NHS England, National Cost Collection.
//!   <https://www.england.nhs.uk/costing-in-the-nhs/national-cost-collection/>
//!
//! Topic doc: health-economics-metrics/topics/marginal-vs-average-cost.md

/// Average (fully absorbed) cost: AC = TC / Q.
///
/// This is what published unit costs report — every fixed cost (building,
/// heating, most staffing) spread over the units. It is the right number for
/// *pricing whole services*, not for valuing small capacity changes.
///
/// # Arguments
///
/// * `total_cost` — total cost TC (currency).
/// * `quantity` — units produced Q.
///
/// # Returns
///
/// `Some(TC / Q)`, or `None` when `quantity` is zero (the average is
/// undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::average_cost;
///
/// // £400,000 over 1,000 bed days → £400 average per bed day.
/// let ac = average_cost(400_000.0, 1_000.0).unwrap();
/// assert!((ac - 400.0).abs() < 1e-9);
///
/// assert!(average_cost(400_000.0, 0.0).is_none());
/// ```
pub fn average_cost(total_cost: f64, quantity: f64) -> Option<f64> {
    if quantity == 0.0 {
        None
    } else {
        Some(total_cost / quantity)
    }
}

/// The naive savings claim: units freed × average cost.
///
/// Kept as an explicit function because it is the number business cases
/// wrongly present as cash. Use it only to show the gap against
/// [`marginal_saving`] — the fixed costs inside the average continue to be
/// paid after the units are freed.
///
/// # Arguments
///
/// * `units_freed` — units of capacity released (e.g. bed days).
/// * `average_cost_per_unit` — the published average cost per unit (currency).
///
/// # Returns
///
/// The naive claim `units_freed × average_cost_per_unit` — an overstatement
/// unless the change crosses a capacity step.
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::naive_average_cost_saving;
///
/// // 1,000 bed days × £400 average = £400,000 "saved". Wrong.
/// let naive = naive_average_cost_saving(1_000.0, 400.0);
/// assert!((naive - 400_000.0).abs() < 1e-9);
/// ```
pub fn naive_average_cost_saving(units_freed: f64, average_cost_per_unit: f64) -> f64 {
    units_freed * average_cost_per_unit
}

/// True saving for a small change that does not cross a capacity threshold: ΔQ × MC.
///
/// MC is the variable cost per unit — food, laundry, consumables, staffing
/// flex — the only spend that actually stops when a unit is freed.
///
/// # Arguments
///
/// * `units_freed` — units of capacity released ΔQ (e.g. bed days).
/// * `marginal_cost_per_unit` — variable cost per unit MC (currency).
///
/// # Returns
///
/// The marginal saving `ΔQ × MC`.
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::marginal_saving;
///
/// // Variable cost ≈ £120/bed day → 1,000 × £120 = £120,000 real saving.
/// let saving = marginal_saving(1_000.0, 120.0);
/// assert!((saving - 120_000.0).abs() < 1e-9);
/// ```
pub fn marginal_saving(units_freed: f64, marginal_cost_per_unit: f64) -> f64 {
    units_freed * marginal_cost_per_unit
}

/// True saving for a large change that crosses a capacity threshold: the step change in TC.
///
/// When enough capacity is freed to close a whole ward (or cancel a
/// reservation tier), fixed costs actually stop — staffing plus running —
/// and the average-cost math becomes closer to true.
///
/// # Arguments
///
/// * `total_cost_before` — total cost before the step (currency, per period).
/// * `total_cost_after` — total cost after the step (same units).
///
/// # Returns
///
/// The cash released per period (`before − after`).
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::step_change_saving;
///
/// // Closing the 20-bed ward: staffing + running ≈ £1.5 million/year.
/// let saving = step_change_saving(10_000_000.0, 8_500_000.0);
/// assert!((saving - 1_500_000.0).abs() < 1e-9);
/// ```
pub fn step_change_saving(total_cost_before: f64, total_cost_after: f64) -> f64 {
    total_cost_before - total_cost_after
}

/// Bed days per year released by a whole ward: beds × 365.
///
/// The volume at which a bed-day saving crosses the ward-closure capacity
/// step, making step-change accounting applicable.
///
/// # Arguments
///
/// * `beds` — number of beds in the ward.
///
/// # Returns
///
/// Bed days per year (`beds × 365`).
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::ward_bed_days_per_year;
///
/// // A 20-bed ward is 7,300 bed days/year.
/// assert!((ward_bed_days_per_year(20.0) - 7_300.0).abs() < 1e-9);
/// ```
pub fn ward_bed_days_per_year(beds: f64) -> f64 {
    beds * 365.0
}

/// Does the freed volume reach a capacity step, so step-change accounting applies?
///
/// Below the step, only [`marginal_saving`] is defensible; at or above it,
/// [`step_change_saving`] measures the real cash.
///
/// # Arguments
///
/// * `units_freed` — units of capacity released.
/// * `units_per_capacity_step` — units in one capacity step (e.g. a whole
///   ward's bed days per year).
///
/// # Returns
///
/// `true` when `units_freed >= units_per_capacity_step`.
///
/// # Examples
///
/// ```rust
/// use health_economics::marginal_vs_average_cost::{
///     crosses_capacity_step, ward_bed_days_per_year,
/// };
///
/// let step = ward_bed_days_per_year(20.0); // 7,300 bed days/year
/// assert!(crosses_capacity_step(7_300.0, step));
/// assert!(!crosses_capacity_step(1_000.0, step));
/// ```
pub fn crosses_capacity_step(units_freed: f64, units_per_capacity_step: f64) -> bool {
    units_freed >= units_per_capacity_step
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Naive claim: 1,000 bed days × £400 average cost = £400,000 "saved". Wrong.
    #[test]
    fn naive_claim_is_400k() {
        // Worked example: "Naive claim: 1,000 × £400 average cost
        // = £400,000 saved. Wrong."
        let got = naive_average_cost_saving(1_000.0, 400.0);
        assert!((got - 400_000.0).abs() < 1e-9);
    }

    /// Marginal claim: variable cost per bed day ≈ £120 → saving
    /// = 1,000 × £120 = £120,000.
    #[test]
    fn marginal_claim_is_120k() {
        // Worked example: "Saving = 1,000 × £120 = £120,000".
        let got = marginal_saving(1_000.0, 120.0);
        assert!((got - 120_000.0).abs() < 1e-9);
    }

    /// A 20-bed ward is 7,300 bed days/year — the capacity step.
    #[test]
    fn twenty_bed_ward_is_7300_bed_days_per_year() {
        // Worked example: "7,300 bed days/year (a 20-bed ward)".
        let got = ward_bed_days_per_year(20.0);
        assert!((got - 7_300.0).abs() < 1e-9);
    }

    /// Freeing 7,300 bed days/year crosses the ward-closure step; freeing
    /// 1,000 does not.
    #[test]
    fn only_a_whole_ward_crosses_the_capacity_step() {
        // Worked example: "if the trust frees 7,300 bed days/year (a 20-bed
        // ward) it can actually close the ward".
        let step = ward_bed_days_per_year(20.0);
        assert!(crosses_capacity_step(7_300.0, step));
        assert!(!crosses_capacity_step(1_000.0, step));
    }

    /// Step-change claim: closing the ward removes staffing + running
    /// ≈ £1.5 million/year of real cash from total cost.
    #[test]
    fn ward_closure_releases_1_5_million() {
        // Worked example: "staffing + running ≈ £1.5 million/year of real cash".
        let got = step_change_saving(10_000_000.0, 8_500_000.0);
        assert!((got - 1_500_000.0).abs() < 1e-9);
    }

    /// AC = TC / Q; with fixed costs, MC < AC (£120 marginal vs £400 average
    /// per bed day), and AC is undefined at zero quantity.
    #[test]
    fn average_cost_exceeds_marginal_when_fixed_costs_dominate() {
        // Doc's math: "Fixed costs make MC < AC for capacity reductions" —
        // £120 marginal vs £400 average per bed day.
        let ac = average_cost(400_000.0, 1_000.0).unwrap();
        assert!((ac - 400.0).abs() < 1e-9);
        assert!(120.0 < ac);
        assert!(average_cost(400_000.0, 0.0).is_none());
    }
}
