//! # National Tariff and Unit Costs
//!
//! The NHS pays providers for activity under a rules-based national price
//! list — historically the National Tariff / Payment by Results, replaced by
//! the **NHS Payment Scheme (NHSPS)** on 1 April 2023. Behind the prices sits
//! a national unit-costing infrastructure: the **National Cost Collection
//! (NCC)** and the **PSSRU Unit Costs of Health and Social Care** compendium
//! (~80 standard unit costs: GP consultation, nurse-hour by band, ED
//! attendance — the default source in UK economic evaluations).
//!
//! ## Formula
//!
//! ```text
//! Tariff price per unit of activity (HRG-coded spell, outpatient attendance)
//!   = national average unit cost (from NCC) × Market Forces Factor
//!     under NHSPS: blended fixed + variable ("aligned payment and
//!     incentive") elements
//!
//! NCC unit cost  = trust-reported total cost of an activity type / activity volume
//!                  (built on Patient-Level Information and Costing Systems, PLICS)
//! Capacity value = hours freed per year × published unit cost per hour
//!
//! Market Forces Factor = local cost adjustment (e.g. 1.15 in high-cost areas)
//! ```
//!
//! ## Why it matters
//!
//! These are the denominators of every credible NHS business case. When a
//! claim says "an outpatient attendance is worth £160" or "a Band 6
//! nurse-hour costs £31", those numbers come from this infrastructure — and
//! using the official figures rather than invented ones is what makes
//! independent evaluations comparable and finance teams cooperative. For a
//! vendor, the tariff also defines the *revenue* side: activity your software
//! enables (extra clinics, backfilled beds) is valued at scheme prices.
//!
//! ## Example
//!
//! Software frees 1 hour/day of a Band 6 nurse's time in a 250-day working
//! year. The same freed hour supports two very different — both auditable —
//! claims, depending on the redeployment mechanism.
//!
//! ```rust
//! use health_economics::national_tariff_and_unit_costs::{
//!     redeployed_activity_value, staff_capacity_value, valuation_ratio,
//! };
//!
//! // Capacity value: 250 days × £31/hour = £7,750/nurse/year (non-cash-releasing).
//! let capacity = staff_capacity_value(1.0, 250.0, 31.0);
//! assert!((capacity - 7_750.0).abs() < 1e-9);
//!
//! // Redeployed: 2 extra outpatient follow-ups/day × 250 × £160
//! // = £80,000/year of funded activity.
//! let funded = redeployed_activity_value(2.0, 250.0, 160.0);
//! assert!((funded - 80_000.0).abs() < 1e-9);
//!
//! // A tenfold difference in claimed value, all from official unit costs.
//! let ratio = valuation_ratio(funded, capacity).unwrap();
//! assert!((ratio - 10.0).abs() < 0.5);
//! ```
//!
//! ## Software engineering connection
//!
//! - This is the **internal price book** pattern: UK health economics works
//!   because every evaluation uses the same published unit costs.
//! - Engineering orgs mostly lack this, so every business case invents its
//!   own cost of an engineer-hour, an incident, a deploy.
//! - A platform team can publish exactly such a book — loaded cost per
//!   engineer-hour by level, per incident by severity, per build-minute —
//!   and require its use in all proposals.
//! - Chargeback/showback systems replicate the tariff's known failure modes:
//!   average-cost pricing drives volume gaming, fixed payments drive
//!   under-provision.
//! - The NHSPS's evolution from pure activity payment to blended
//!   fixed+variable is twenty years of lessons in incentive design for
//!   internal platform pricing.
//!
//! ## Pitfalls
//!
//! - **Stale figures**: NCC, PSSRU, and NHSPS prices refresh annually — date
//!   every number.
//! - **Tariff price ≠ cost**: prices are national averages with adjustments;
//!   your local marginal cost differs.
//! - **Valuing capacity at tariff without a mechanism** to actually deliver
//!   and be paid for the extra activity.
//!
//! ## Sources
//!
//! - NHS England, NHS Payment Scheme.
//!   <https://www.england.nhs.uk/pay-syst/national-tariff/national-tariff-payment-system/>
//! - NHS England, National Cost Collection.
//!   <https://www.england.nhs.uk/costing-in-the-nhs/national-cost-collection/>
//! - PSSRU, Unit Costs of Health and Social Care.
//!   <https://www.pssru.ac.uk/unitcostsreport/>
//!
//! Topic doc: health-economics-metrics/topics/national-tariff-and-unit-costs.md

/// NCC unit cost: trust-reported total cost of an activity type divided by activity volume.
///
/// Built on patient-level costing (PLICS). This is the national average cost
/// that seeds tariff prices — remember it is an *average*, not your local
/// marginal cost.
///
/// # Arguments
///
/// * `total_cost` — trust-reported total cost of the activity type (currency).
/// * `activity_volume` — number of activity units delivered (spells,
///   attendances).
///
/// # Returns
///
/// `Some(total_cost / activity_volume)`, or `None` when the volume is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::ncc_unit_cost;
///
/// // £16M reported cost over 100,000 outpatient attendances → £160 each.
/// let unit = ncc_unit_cost(16_000_000.0, 100_000.0).unwrap();
/// assert!((unit - 160.0).abs() < 1e-9);
///
/// assert!(ncc_unit_cost(16_000_000.0, 0.0).is_none());
/// ```
pub fn ncc_unit_cost(total_cost: f64, activity_volume: f64) -> Option<f64> {
    if activity_volume == 0.0 {
        None
    } else {
        Some(total_cost / activity_volume)
    }
}

/// Tariff price per unit of activity: national average unit cost × Market Forces Factor.
///
/// The Market Forces Factor is the local adjustment for unavoidable cost
/// differences (land, labour); 1.0 means the national average, values above
/// 1.0 mark high-cost areas.
///
/// # Arguments
///
/// * `national_average_unit_cost` — the NCC-derived national average cost
///   per unit (currency).
/// * `market_forces_factor` — the local MFF multiplier (dimensionless,
///   ≈ 0.9–1.3).
///
/// # Returns
///
/// The locally adjusted tariff price per unit.
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::tariff_price;
///
/// // £160 national average × 1.15 MFF = £184 local price.
/// let price = tariff_price(160.0, 1.15);
/// assert!((price - 184.0).abs() < 1e-9);
/// ```
pub fn tariff_price(national_average_unit_cost: f64, market_forces_factor: f64) -> f64 {
    national_average_unit_cost * market_forces_factor
}

/// NHSPS blended ("aligned payment and incentive") payment: fixed element + variable element.
///
/// The NHSPS moved from pure activity payment to a blend: a fixed element
/// (paid regardless of volume) plus a variable price per unit of activity —
/// an incentive-design lesson for internal platform pricing.
///
/// # Arguments
///
/// * `fixed_element` — the fixed payment for the period (currency).
/// * `variable_price_per_unit` — price paid per unit of activity (currency).
/// * `activity_units` — units of activity delivered in the period.
///
/// # Returns
///
/// The total payment `fixed + variable × activity`.
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::blended_payment;
///
/// // £1M fixed + £160 × 500 attendances = £1,080,000.
/// let payment = blended_payment(1_000_000.0, 160.0, 500.0);
/// assert!((payment - 1_080_000.0).abs() < 1e-9);
/// ```
pub fn blended_payment(fixed_element: f64, variable_price_per_unit: f64, activity_units: f64) -> f64 {
    fixed_element + variable_price_per_unit * activity_units
}

/// Non-cash-releasing capacity value of freed staff time.
///
/// Hours freed per working day × working days per year × published unit cost
/// per hour (e.g. PSSRU Band 6 nurse ≈ £31/hour including overheads). This
/// values the *capacity*; no cash leaves the payroll unless posts change.
///
/// # Arguments
///
/// * `hours_freed_per_day` — staff hours freed per working day.
/// * `working_days_per_year` — working days per year (e.g. 250).
/// * `unit_cost_per_hour` — published unit cost per staff hour, including
///   overheads (currency).
///
/// # Returns
///
/// The annual capacity value (currency/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::staff_capacity_value;
///
/// // 1 hour/day × 250 days × £31/hour = £7,750/nurse/year.
/// let value = staff_capacity_value(1.0, 250.0, 31.0);
/// assert!((value - 7_750.0).abs() < 1e-9);
/// ```
pub fn staff_capacity_value(
    hours_freed_per_day: f64,
    working_days_per_year: f64,
    unit_cost_per_hour: f64,
) -> f64 {
    hours_freed_per_day * working_days_per_year * unit_cost_per_hour
}

/// Funded-activity value when freed time is redeployed into extra tariff-paid activity.
///
/// Extra units per working day × working days per year × scheme price per
/// unit (e.g. outpatient follow-up ≈ £160). Only claimable with a real
/// mechanism to deliver and be paid for the extra activity.
///
/// # Arguments
///
/// * `extra_units_per_day` — extra tariff-paid activity units per working day.
/// * `working_days_per_year` — working days per year (e.g. 250).
/// * `scheme_price_per_unit` — NHSPS price per activity unit (currency).
///
/// # Returns
///
/// The annual funded-activity value (currency/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::redeployed_activity_value;
///
/// // 2 extra follow-ups/day × 250 days × £160 = £80,000/year.
/// let value = redeployed_activity_value(2.0, 250.0, 160.0);
/// assert!((value - 80_000.0).abs() < 1e-9);
/// ```
pub fn redeployed_activity_value(
    extra_units_per_day: f64,
    working_days_per_year: f64,
    scheme_price_per_unit: f64,
) -> f64 {
    extra_units_per_day * working_days_per_year * scheme_price_per_unit
}

/// Ratio of two valuations of the same freed capacity.
///
/// E.g. funded-activity value over capacity value — the multiple that depends
/// entirely on the redeployment mechanism (the worked example lands near 10×).
///
/// # Arguments
///
/// * `higher_claim` — the larger valuation (currency/year).
/// * `lower_claim` — the smaller valuation (currency/year).
///
/// # Returns
///
/// `Some(higher / lower)`, or `None` when `lower_claim` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::national_tariff_and_unit_costs::valuation_ratio;
///
/// // £80,000 funded-activity claim over £7,750 capacity claim ≈ 10.3×.
/// let ratio = valuation_ratio(80_000.0, 7_750.0).unwrap();
/// assert!((ratio - 10.0).abs() < 0.5);
///
/// assert!(valuation_ratio(80_000.0, 0.0).is_none());
/// ```
pub fn valuation_ratio(higher_claim: f64, lower_claim: f64) -> Option<f64> {
    if lower_claim == 0.0 {
        None
    } else {
        Some(higher_claim / lower_claim)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1 hour/day × 250 working days × £31/hour = £7,750/nurse/year
    /// (non-cash-releasing capacity value).
    #[test]
    fn band_6_nurse_hour_freed_is_worth_7750_per_year() {
        // Worked example: "Capacity value = 250 × £31 = £7,750/nurse/year
        // (non-cash-releasing)".
        let got = staff_capacity_value(1.0, 250.0, 31.0);
        assert!((got - 7_750.0).abs() < 1e-9);
    }

    /// 2 extra outpatient follow-ups/day × 250 days = 500 × £160
    /// = £80,000/year of funded activity.
    #[test]
    fn redeployment_into_clinics_is_worth_80000_per_year() {
        // Worked example: "500 × £160 = £80,000/year of funded activity".
        let got = redeployed_activity_value(2.0, 250.0, 160.0);
        assert!((got - 80_000.0).abs() < 1e-9);
    }

    /// A tenfold difference in claimed value depending on redeployment, all
    /// from official unit costs (exact ratio 80,000 / 7,750 ≈ 10.3).
    #[test]
    fn redeployment_claim_is_about_tenfold_the_capacity_claim() {
        // Worked example: "a tenfold difference in claimed value depending on
        // redeployment, all from official unit costs".
        let capacity = staff_capacity_value(1.0, 250.0, 31.0);
        let funded = redeployed_activity_value(2.0, 250.0, 160.0);
        let ratio = valuation_ratio(funded, capacity).unwrap();
        assert!((ratio - 80_000.0 / 7_750.0).abs() < 1e-9);
        assert!((ratio - 10.0).abs() < 0.5);
    }

    /// NCC unit cost is total cost over volume; undefined at zero volume.
    #[test]
    fn ncc_unit_cost_divides_total_cost_by_volume() {
        // Doc's math: "NCC unit cost = trust-reported total cost of an
        // activity type / activity volume" — £160 per outpatient attendance.
        let got = ncc_unit_cost(16_000_000.0, 100_000.0).unwrap();
        assert!((got - 160.0).abs() < 1e-9);
        assert!(ncc_unit_cost(16_000_000.0, 0.0).is_none());
    }

    /// Tariff price applies the Market Forces Factor to the national average.
    #[test]
    fn tariff_price_applies_market_forces_factor() {
        // Doc's math: "national average unit cost (from NCC) × Market Forces
        // Factor (local adjustment)".
        let got = tariff_price(160.0, 1.15);
        assert!((got - 184.0).abs() < 1e-9);
    }

    /// NHSPS blended payment is fixed plus variable × activity.
    #[test]
    fn blended_payment_is_fixed_plus_variable() {
        // Doc's math: "under NHSPS: blended fixed + variable ('aligned
        // payment and incentive') elements".
        let got = blended_payment(1_000_000.0, 160.0, 500.0);
        assert!((got - 1_080_000.0).abs() < 1e-9);
    }
}
