//! # Hard Cash-Releasing Savings (Deficit Defense)
//!
//! Hard cash-releasing savings are line items a hospital can actively
//! **delete from next month's budget** because of your software. To a strict
//! financial accountant — and to a trust running a deficit — this is the
//! only benefit class that fully counts.
//!
//! The NHS's most reliable hard-cash target is **premium-rate temporary
//! staffing**: trusts cover gaps with internal "Bank" staff (paid
//! standard-ish rates) and external "Agency" staff (often 2–3× Agenda for
//! Change rates, capped but frequently breached for scarce roles), plus
//! overtime premiums and cancellable external contracts.
//!
//! ## Formula
//!
//! ```text
//! Hard saving = premium shifts avoided × (premium rate − substantive rate)
//!             + overtime hours avoided × overtime premium
//!             + external contracts cancelled × contract value
//!
//! premium rate     = £/shift paid for Bank/Agency cover
//! substantive rate = £/shift for the equivalent employed member of staff
//! overtime premium = £/hour paid above the standard rate
//! contract value   = annual £ value of a cancelled external contract
//!
//! Mechanism requirement: name the specific budget line and the manager who
//! will confirm its reduction. If no one can point to the line, it isn't hard cash.
//! ```
//!
//! ## Why it matters
//!
//! Many NHS trusts operate under deficit-recovery plans with intense
//! scrutiny of every expenditure line. In that environment, capacity
//! benefits and quality improvements — however real — do not close the gap;
//! only cash does. A software product that can prove it deletes budget lines
//! is *self-funding from the CFO's perspective*: the conversation stops
//! being "can we afford this?" and becomes "can we afford not to?".
//! (Published NHS workforce models have claimed ratios as high as £11+
//! saved per £1 spent on this mechanism; treat any such ratio as a
//! hypothesis for *your* trust's rostering data, not a portable fact.)
//!
//! ## Example
//!
//! A Band 6 nurse loses ~1 hour/shift to administrative overhead; software
//! returns that hour to the scheduled shift across 300 nurses, ending
//! documentation overtime and Bank catch-up shifts.
//!
//! ```
//! use health_economics::hard_cash_releasing_savings_deficit_defense::{
//!     annual_workforce_overtime_saving, annual_bank_agency_saving, net_of_licence,
//! };
//!
//! // Overtime avoided: 300 nurses × 2.5 hrs/week × £8 premium × 46 wks ≈ £276,000/year.
//! let overtime = annual_workforce_overtime_saving(300.0, 2.5, 8.0, 46.0);
//! assert_eq!(overtime, 276_000.0);
//!
//! // Bank/agency: 15 catch-up shifts/week × £180 premium × 52 ≈ £140,400/year.
//! let bank = annual_bank_agency_saving(15.0, 180.0, 52.0);
//! assert_eq!(bank, 140_400.0);
//!
//! // Hard cash total ≈ £416,000/year against a licence cost of ~£150,000.
//! let total = overtime + bank;
//! assert_eq!(total, 416_400.0);
//! assert_eq!(net_of_licence(total, 150_000.0), 266_400.0);
//! ```
//!
//! Every pound is auditable against the e-rostering and payroll systems —
//! which is exactly how the benefit should be evidenced, monthly, through
//! benefits realization.
//!
//! ## Software engineering connection
//!
//! - The engineering equivalents of agency premium are the org's own
//!   distress purchases: contractor day-rates covering delivery gaps,
//!   incident-driven overtime, expedited-support contracts, cloud
//!   spot-price panic.
//! - Productivity software claiming hard cash should target those lines
//!   with the same discipline — name the budget line, the owner, and the
//!   month it shrinks.
//! - Everything else it delivers is capacity or quality: real, valuable,
//!   and different.
//!
//! ## Pitfalls
//!
//! - **Calling capacity "savings"** — the instant credibility killer with
//!   finance.
//! - **Vendor-model ratios presented as local fact** (the £11:£1 problem) —
//!   rebuild the model on the trust's own rostering data.
//! - **One-off vs recurrent confusion**: a cancelled contract saves its
//!   value once per year, not once; a deleted post saves salary only while
//!   it stays deleted.
//!
//! ## Sources
//!
//! - NHS England, reducing agency spend in the NHS.
//!   <https://www.england.nhs.uk/long-read/reducing-agency-spend-in-the-nhs/>
//! - NHS Digital business case guidance, economic case.
//!   <https://digital.nhs.uk/services/networks-and-connectivity-transformation-frontline-capabilities/connectivity-hub/advice-and-guidance/making-the-business-case-for-connectivity-infrastructure-investment---guidance/economic-case>
//!
//! Topic doc: health-economics-metrics/topics/hard-cash-releasing-savings-deficit-defense.md

/// Saving from avoided premium-rate (Bank/Agency) shifts.
///
/// Only the *premium* over the substantive rate is a saving — the work still
/// gets done by employed staff at the substantive rate.
///
/// # Arguments
///
/// * `premium_shifts_avoided` — Bank/Agency shifts no longer booked (count).
/// * `premium_rate_per_shift` — £ paid per premium shift.
/// * `substantive_rate_per_shift` — £ per shift for equivalent employed
///   staff.
///
/// # Returns
///
/// Saving in £: `shifts × (premium − substantive)`.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::premium_shift_saving;
///
/// // 780 shifts/year (15/week × 52) at a £180/shift premium over substantive.
/// assert_eq!(premium_shift_saving(780.0, 180.0, 0.0), 140_400.0);
/// ```
pub fn premium_shift_saving(
    premium_shifts_avoided: f64,
    premium_rate_per_shift: f64,
    substantive_rate_per_shift: f64,
) -> f64 {
    // Only the premium over the substantive rate releases cash.
    premium_shifts_avoided * (premium_rate_per_shift - substantive_rate_per_shift)
}

/// Saving from avoided overtime hours.
///
/// # Arguments
///
/// * `overtime_hours_avoided` — paid overtime hours no longer worked.
/// * `overtime_premium_per_hour` — £ paid per hour *above* the standard rate
///   (the premium element only).
///
/// # Returns
///
/// Saving in £: `hours × premium`.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::overtime_saving;
///
/// // 34,500 overtime hours (300 nurses × 2.5 hrs/week × 46 wks) at £8 premium.
/// assert_eq!(overtime_saving(34_500.0, 8.0), 276_000.0);
/// ```
pub fn overtime_saving(overtime_hours_avoided: f64, overtime_premium_per_hour: f64) -> f64 {
    overtime_hours_avoided * overtime_premium_per_hour
}

/// Saving from cancelled external contracts.
///
/// A cancelled contract saves its value once per year while it stays
/// cancelled — recurrent, not one-off, but only for as long as the
/// cancellation holds.
///
/// # Arguments
///
/// * `contracts_cancelled` — number of contracts cancelled.
/// * `contract_value` — annual £ value per contract.
///
/// # Returns
///
/// Annual saving in £.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::cancelled_contract_saving;
///
/// // One £50,000/year external contract cancelled.
/// assert_eq!(cancelled_contract_saving(1.0, 50_000.0), 50_000.0);
/// ```
pub fn cancelled_contract_saving(contracts_cancelled: f64, contract_value: f64) -> f64 {
    contracts_cancelled * contract_value
}

/// Total hard cash-releasing saving: the doc's three-line formula.
///
/// Sums premium-shift, overtime, and cancelled-contract savings. Each term
/// must map to a named budget line with a manager who will confirm its
/// reduction — otherwise it isn't hard cash.
///
/// # Arguments
///
/// * `premium_shifts_avoided` — Bank/Agency shifts avoided (count).
/// * `premium_rate_per_shift` — £ per premium shift.
/// * `substantive_rate_per_shift` — £ per substantive shift.
/// * `overtime_hours_avoided` — overtime hours avoided.
/// * `overtime_premium_per_hour` — £ premium per overtime hour.
/// * `contracts_cancelled` — external contracts cancelled (count).
/// * `contract_value` — annual £ value per contract.
///
/// # Returns
///
/// Total hard saving in £/year.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::hard_saving;
///
/// // Worked example as one call: 780 premium shifts at £180, 34,500 overtime
/// // hours at £8, no cancelled contracts → £416,400/year.
/// let total = hard_saving(780.0, 180.0, 0.0, 34_500.0, 8.0, 0.0, 0.0);
/// assert_eq!(total, 416_400.0);
/// ```
pub fn hard_saving(
    premium_shifts_avoided: f64,
    premium_rate_per_shift: f64,
    substantive_rate_per_shift: f64,
    overtime_hours_avoided: f64,
    overtime_premium_per_hour: f64,
    contracts_cancelled: f64,
    contract_value: f64,
) -> f64 {
    premium_shift_saving(premium_shifts_avoided, premium_rate_per_shift, substantive_rate_per_shift)
        + overtime_saving(overtime_hours_avoided, overtime_premium_per_hour)
        + cancelled_contract_saving(contracts_cancelled, contract_value)
}

/// Annual overtime saving for a workforce.
///
/// Convenience form of [`overtime_saving`] built from weekly per-person
/// figures: staff × hours/week × £/hour premium × working weeks/year.
///
/// # Arguments
///
/// * `staff` — number of staff affected (count).
/// * `overtime_hours_avoided_per_week` — paid overtime hours avoided per
///   person per week.
/// * `overtime_premium_per_hour` — £ premium per overtime hour.
/// * `weeks_per_year` — working weeks per year (the worked example uses 46
///   to allow for leave).
///
/// # Returns
///
/// Annual saving in £.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::annual_workforce_overtime_saving;
///
/// // Worked example: 300 nurses × 2.5 hrs/week × £8 × 46 wks ≈ £276,000/year.
/// let s = annual_workforce_overtime_saving(300.0, 2.5, 8.0, 46.0);
/// assert_eq!(s, 276_000.0);
/// ```
pub fn annual_workforce_overtime_saving(
    staff: f64,
    overtime_hours_avoided_per_week: f64,
    overtime_premium_per_hour: f64,
    weeks_per_year: f64,
) -> f64 {
    staff * overtime_hours_avoided_per_week * overtime_premium_per_hour * weeks_per_year
}

/// Annual Bank/Agency saving from weekly shift counts.
///
/// Convenience form of [`premium_shift_saving`]: catch-up shifts avoided per
/// week × premium per shift × weeks per year.
///
/// # Arguments
///
/// * `shifts_avoided_per_week` — Bank/Agency shifts no longer booked per
///   week.
/// * `premium_per_shift` — £ premium per shift (over the substantive rate).
/// * `weeks_per_year` — weeks per year the pattern holds (the worked example
///   uses 52 — ward cover runs year-round).
///
/// # Returns
///
/// Annual saving in £.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::annual_bank_agency_saving;
///
/// // Worked example: 15 shifts/week × £180 × 52 ≈ £140,400/year.
/// let s = annual_bank_agency_saving(15.0, 180.0, 52.0);
/// assert_eq!(s, 140_400.0);
/// ```
pub fn annual_bank_agency_saving(
    shifts_avoided_per_week: f64,
    premium_per_shift: f64,
    weeks_per_year: f64,
) -> f64 {
    shifts_avoided_per_week * premium_per_shift * weeks_per_year
}

/// Net hard cash position: total hard saving minus the software licence cost.
///
/// Positive means the product is self-funding from the CFO's perspective —
/// the deficit-defense claim.
///
/// # Arguments
///
/// * `hard_saving_total` — total hard cash-releasing saving (£/year).
/// * `licence_cost` — annual software licence cost (£/year).
///
/// # Returns
///
/// Net cash released (£/year); negative means the licence costs more than
/// the cash it releases.
///
/// # Examples
///
/// ```
/// use health_economics::hard_cash_releasing_savings_deficit_defense::net_of_licence;
///
/// // Worked example: £416,400 saving against a ~£150,000 licence.
/// assert_eq!(net_of_licence(416_400.0, 150_000.0), 266_400.0);
/// ```
pub fn net_of_licence(hard_saving_total: f64, licence_cost: f64) -> f64 {
    hard_saving_total - licence_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "Overtime avoided: 300 nurses × 2.5 paid overtime hrs/week ×
    // £8 premium × 46 wks ≈ £276,000/year".
    #[test]
    fn worked_example_overtime_saving_is_276000() {
        let s = annual_workforce_overtime_saving(300.0, 2.5, 8.0, 46.0);
        assert!((s - 276_000.0).abs() < 1e-9, "got {s}");
    }

    // Doc line: "Bank/agency shifts: 15 catch-up shifts/week × £180 premium ×
    // 52 ≈ £140,400/year".
    #[test]
    fn worked_example_bank_agency_saving_is_140400() {
        let s = annual_bank_agency_saving(15.0, 180.0, 52.0);
        assert!((s - 140_400.0).abs() < 1e-9, "got {s}");
    }

    // Doc line: "Hard cash total ≈ £416,000/year" (276,000 + 140,400 = 416,400).
    #[test]
    fn worked_example_hard_cash_total_is_about_416000() {
        let total = annual_workforce_overtime_saving(300.0, 2.5, 8.0, 46.0)
            + annual_bank_agency_saving(15.0, 180.0, 52.0);
        assert!((total - 416_400.0).abs() < 1e-9, "got {total}");
        assert!((total - 416_000.0).abs() < 1_000.0, "got {total}");
    }

    // Doc line: "against a licence cost of ~£150,000" — comfortably self-funding.
    #[test]
    fn worked_example_net_of_licence_is_positive() {
        let net = net_of_licence(416_400.0, 150_000.0);
        assert!((net - 266_400.0).abs() < 1e-9, "got {net}");
        assert!(net > 0.0);
    }

    // Doc formula: "Hard saving = premium shifts ... + overtime hours ... +
    // external contracts ..." — reproduces the worked-example total.
    #[test]
    fn hard_saving_sums_three_mechanisms() {
        // 15 shifts/week × 52 weeks avoided at a £180/shift premium over
        // substantive, plus 300 × 2.5 × 46 overtime hours at £8 premium,
        // no cancelled contracts — reproduces the worked-example total.
        let total = hard_saving(
            15.0 * 52.0,
            180.0,
            0.0,
            300.0 * 2.5 * 46.0,
            8.0,
            0.0,
            0.0,
        );
        assert!((total - 416_400.0).abs() < 1e-9, "got {total}");
    }

    // Doc pitfall: "a cancelled contract saves its value once per year, not once".
    #[test]
    fn cancelled_contract_saving_is_annual_value() {
        let s = cancelled_contract_saving(1.0, 50_000.0);
        assert!((s - 50_000.0).abs() < 1e-9, "got {s}");
    }
}
