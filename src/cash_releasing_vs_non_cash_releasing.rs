//! # Cash-Releasing vs Non-Cash-Releasing Savings
//!
//! Cash-releasing savings reduce actual expenditure — a budget line gets
//! smaller. Non-cash-releasing savings free time or capacity that gets
//! *reused* rather than banked. Health-system finance directors treat these
//! as different species, and so should you.
//!
//! The same physical event (an hour saved) lands in one category or the
//! other depending on what happens next: an overtime/agency shift cancelled
//! is cash-releasing; a clinician seeing one more waiting patient is
//! non-cash-releasing capacity; an hour absorbed into slack is no benefit at
//! all.
//!
//! ## Formula
//!
//! ```text
//! Cash-releasing saving    = budget line before − budget line after
//!                            (must be extractable: a cancelled contract, closed
//!                             ward, reduced agency spend, avoided purchase)
//!
//! Non-cash-releasing value = time released × unit cost of that time
//!                            (valued at opportunity cost; the money is NOT extractable)
//!
//! hour saved → overtime/agency shift cancelled        → cash-releasing
//! hour saved → clinician sees one more waiting patient → non-cash-releasing (capacity)
//! hour saved → absorbed into slack, nothing changes    → no benefit at all
//! ```
//!
//! ## Why it matters
//!
//! This is the sharpest honesty test applied to any digital business case in
//! a national health service. NHS benefit frameworks explicitly categorize
//! every claimed benefit as cash-releasing, non-cash-releasing, or
//! qualitative. Most digital health "savings" — clinician minutes saved per
//! patient, faster documentation — are non-cash-releasing: valuable, but
//! they don't reduce the deficit. A trust CFO facing a funding gap can only
//! spend cash. A business case that presents £80.5k cash + £172.5k capacity
//! is credible; one that presents £287.5k "savings" gets rejected by the
//! first accountant who reads it.
//!
//! ## Example
//!
//! Software saves each of 100 nurses 30 minutes per shift: 100 × 0.5 ×
//! 5 shifts/week × 46 weeks ≈ 11,500 hours/year. At a Band 5 employer cost
//! of ~£25/hour the tempting headline is £287,500/year — but the honest
//! split is £80,500 cash-releasing (2,300 h at the £35 agency rate),
//! £172,500 capacity (6,900 h at £25), and £0 for the 20% that dissipates.
//!
//! ```rust
//! use health_economics::cash_releasing_vs_non_cash_releasing::{
//!     SavingCategory, TimeAllocation, annual_hours_saved, category_value,
//!     non_cash_releasing_value,
//! };
//!
//! // 100 nurses × 0.5 h/shift × 5 shifts/week × 46 weeks ≈ 11,500 h/year.
//! let hours = annual_hours_saved(100.0, 0.5, 5.0, 46.0);
//! assert!((hours - 11_500.0).abs() < 1e-9);
//!
//! // The tempting headline: 11,500 h × £25 = £287,500 — the canonical sin.
//! let headline = non_cash_releasing_value(hours, 25.0);
//! assert!((headline - 287_500.0).abs() < 1e-6);
//!
//! // The honest split: 20% cancels agency cover, 60% redeploys, 20% dissipates.
//! let split = [
//!     TimeAllocation { category: SavingCategory::CashReleasing, fraction: 0.20, hourly_rate: 35.0 },
//!     TimeAllocation { category: SavingCategory::NonCashReleasing, fraction: 0.60, hourly_rate: 25.0 },
//!     TimeAllocation { category: SavingCategory::NoBenefit, fraction: 0.20, hourly_rate: 25.0 },
//! ];
//! let cash = category_value(hours, &split, SavingCategory::CashReleasing);
//! let capacity = category_value(hours, &split, SavingCategory::NonCashReleasing);
//! let nothing = category_value(hours, &split, SavingCategory::NoBenefit);
//! assert!((cash - 80_500.0).abs() < 1e-6);      // 2,300 h × £35 agency rate
//! assert!((capacity - 172_500.0).abs() < 1e-6); // 6,900 h × £25 employer cost
//! assert!(nothing.abs() < 1e-9);                // claiming it would be fiction
//! ```
//!
//! ## Software engineering connection
//!
//! - Identical logic governs AI coding-assistant ROI: "30 minutes per
//!   developer per day" is non-cash-releasing capacity unless headcount,
//!   contractor spend, or cloud cost actually falls.
//! - Cash-releasing: cancelled contractor engagements, decommissioned
//!   tooling licenses, reduced cloud spend.
//! - Capacity: features shipped sooner (value via cost of delay), backlog
//!   burned down.
//! - Nothing: minutes saved that fragment into context-switching.
//! - Track *where released time actually went* — benefits realization exists
//!   because claimed capacity gains frequently evaporate on audit.
//!
//! ## Pitfalls
//!
//! - **Multiplying minutes by salary and calling it savings** — the
//!   canonical sin.
//! - **Valuing released time at average loaded cost** when the marginal use
//!   of that time is low-value (see marginal vs average cost).
//! - **Counting the same hour twice**: as cash (shift avoided) and as
//!   capacity (extra patients seen).
//!
//! ## Sources
//!
//! - NHS Digital connectivity business case guidance, economic case (benefit categories).
//!   <https://digital.nhs.uk/services/networks-and-connectivity-transformation-frontline-capabilities/connectivity-hub/advice-and-guidance/making-the-business-case-for-connectivity-infrastructure-investment---guidance/economic-case>
//! - NHS England, NHS productivity. <https://www.england.nhs.uk/long-read/nhs-productivity/>
//!
//! Topic doc: health-economics-metrics/topics/cash-releasing-vs-non-cash-releasing.md

/// What actually happens to a saved hour, which determines its category.
///
/// NHS benefit frameworks require every claimed benefit to be categorized;
/// the same physical hour lands in a different category depending on what
/// happens next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavingCategory {
    /// A budget line genuinely shrinks (cancelled shift, contract, purchase).
    CashReleasing,
    /// Time is redeployed to more output; real value, but not extractable money.
    NonCashReleasing,
    /// Time dissipates into slack; claiming any value would be fiction.
    NoBenefit,
}

/// Cash-releasing saving = budget line before − budget line after.
///
/// Only a genuine budget delta counts: a cancelled contract, closed ward,
/// reduced agency spend, or avoided purchase. Both arguments in the same
/// currency and period.
///
/// # Arguments
///
/// * `budget_line_before` — the budget line before the change.
/// * `budget_line_after` — the same budget line after the change.
///
/// # Returns
///
/// The extractable saving (positive when the line shrank).
///
/// # Examples
///
/// ```rust
/// use health_economics::cash_releasing_vs_non_cash_releasing::cash_releasing_saving;
///
/// // An agency-spend line falling from £500,000 to £419,500 releases £80,500.
/// assert!((cash_releasing_saving(500_000.0, 419_500.0) - 80_500.0).abs() < 1e-6);
/// ```
pub fn cash_releasing_saving(budget_line_before: f64, budget_line_after: f64) -> f64 {
    budget_line_before - budget_line_after
}

/// Non-cash-releasing value = time released (hours) × unit cost of that time.
///
/// Valued at opportunity cost; the money is NOT extractable. Report it as
/// capacity, never as "savings". Applied to *all* saved hours at loaded
/// salary this produces the tempting-headline figure the honest split exists
/// to correct.
///
/// # Arguments
///
/// * `hours_released` — hours of time freed over the period.
/// * `unit_cost_per_hour` — £ per hour valuing that time (e.g. £25 Band 5
///   employer cost).
///
/// # Returns
///
/// The capacity value in currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::cash_releasing_vs_non_cash_releasing::non_cash_releasing_value;
///
/// // 11,500 hours × £25/hour = £287,500 — the tempting headline that must
/// // then be split honestly.
/// assert!((non_cash_releasing_value(11_500.0, 25.0) - 287_500.0).abs() < 1e-6);
/// ```
pub fn non_cash_releasing_value(hours_released: f64, unit_cost_per_hour: f64) -> f64 {
    hours_released * unit_cost_per_hour
}

/// Annual hours saved = staff count × hours saved per shift × shifts per week
/// × working weeks per year.
///
/// # Arguments
///
/// * `staff_count` — number of staff affected.
/// * `hours_saved_per_shift` — hours saved per person per shift (0.5 for
///   30 minutes).
/// * `shifts_per_week` — shifts each person works per week.
/// * `weeks_per_year` — working weeks per year (46 in the worked example).
///
/// # Returns
///
/// Total saved hours per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::cash_releasing_vs_non_cash_releasing::annual_hours_saved;
///
/// // 100 nurses × 30 min/shift × 5 shifts/week × 46 weeks ≈ 11,500 h/year.
/// assert!((annual_hours_saved(100.0, 0.5, 5.0, 46.0) - 11_500.0).abs() < 1e-9);
/// ```
pub fn annual_hours_saved(
    staff_count: f64,
    hours_saved_per_shift: f64,
    shifts_per_week: f64,
    weeks_per_year: f64,
) -> f64 {
    staff_count * hours_saved_per_shift * shifts_per_week * weeks_per_year
}

/// One slice of the honest split of saved time: which category it lands in,
/// what fraction of the total hours, and the rate that values an hour there.
///
/// Fractions across a full split should sum to 1.0; the `NoBenefit` slice's
/// rate is ignored because dissipated time is worth exactly £0.
#[derive(Debug, Clone, Copy)]
pub struct TimeAllocation {
    /// Where this slice of the saved time actually goes.
    pub category: SavingCategory,
    /// Fraction of total saved hours in this slice (0–1).
    pub fraction: f64,
    /// £ per hour used to value this slice (e.g. agency rate for cancelled
    /// shifts, employer cost for redeployed capacity). Ignored for NoBenefit.
    pub hourly_rate: f64,
}

impl TimeAllocation {
    /// Hours of the total falling in this slice: total × fraction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cash_releasing_vs_non_cash_releasing::{
    ///     SavingCategory, TimeAllocation,
    /// };
    ///
    /// // The 20% cash-releasing slice of 11,500 h is 2,300 h.
    /// let slice = TimeAllocation {
    ///     category: SavingCategory::CashReleasing,
    ///     fraction: 0.20,
    ///     hourly_rate: 35.0,
    /// };
    /// assert!((slice.hours(11_500.0) - 2_300.0).abs() < 1e-9);
    /// ```
    pub fn hours(&self, total_hours: f64) -> f64 {
        total_hours * self.fraction
    }

    /// Value of this slice: hours × hourly rate, except that dissipated
    /// time is worth exactly £0 regardless of the rate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::cash_releasing_vs_non_cash_releasing::{
    ///     SavingCategory, TimeAllocation,
    /// };
    ///
    /// // 2,300 h at the £35 agency rate → £80,500 cash-releasing.
    /// let cash = TimeAllocation {
    ///     category: SavingCategory::CashReleasing,
    ///     fraction: 0.20,
    ///     hourly_rate: 35.0,
    /// };
    /// assert!((cash.value(11_500.0) - 80_500.0).abs() < 1e-6);
    ///
    /// // The dissipated 20% is worth £0 even though a rate is set.
    /// let slack = TimeAllocation {
    ///     category: SavingCategory::NoBenefit,
    ///     fraction: 0.20,
    ///     hourly_rate: 25.0,
    /// };
    /// assert!(slack.value(11_500.0).abs() < 1e-9);
    /// ```
    pub fn value(&self, total_hours: f64) -> f64 {
        match self.category {
            // Dissipated time carries no value: claiming it would be fiction.
            SavingCategory::NoBenefit => 0.0,
            _ => self.hours(total_hours) * self.hourly_rate,
        }
    }
}

/// Total value in one category across an allocation split.
///
/// Sums `TimeAllocation::value` over the slices whose category matches, so
/// cash and capacity can be reported separately (never summed into a single
/// "savings" figure).
///
/// # Arguments
///
/// * `total_hours` — total saved hours the split divides.
/// * `allocations` — the honest split of where those hours actually went.
/// * `category` — which category to total.
///
/// # Returns
///
/// The category's value in currency (0.0 if no slice matches, and always
/// 0.0 for `NoBenefit`).
///
/// # Examples
///
/// ```rust
/// use health_economics::cash_releasing_vs_non_cash_releasing::{
///     SavingCategory, TimeAllocation, category_value,
/// };
///
/// let split = [
///     TimeAllocation { category: SavingCategory::CashReleasing, fraction: 0.20, hourly_rate: 35.0 },
///     TimeAllocation { category: SavingCategory::NonCashReleasing, fraction: 0.60, hourly_rate: 25.0 },
///     TimeAllocation { category: SavingCategory::NoBenefit, fraction: 0.20, hourly_rate: 25.0 },
/// ];
/// // £80,500 cash-releasing and £172,500 capacity, reported separately.
/// let cash = category_value(11_500.0, &split, SavingCategory::CashReleasing);
/// let capacity = category_value(11_500.0, &split, SavingCategory::NonCashReleasing);
/// assert!((cash - 80_500.0).abs() < 1e-6);
/// assert!((capacity - 172_500.0).abs() < 1e-6);
/// ```
pub fn category_value(
    total_hours: f64,
    allocations: &[TimeAllocation],
    category: SavingCategory,
) -> f64 {
    allocations
        .iter()
        .filter(|a| a.category == category)
        .map(|a| a.value(total_hours))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: software saves each of 100 nurses 30 minutes per shift,
    // 5 shifts/week, 46 weeks/year; Band 5 employer cost ~£25/hour.
    // Honest split: 20% cancels agency cover at £35/hour, 60% redeployed to
    // care at £25/hour, 20% dissipates.

    fn split() -> [TimeAllocation; 3] {
        [
            TimeAllocation {
                category: SavingCategory::CashReleasing,
                fraction: 0.20,
                hourly_rate: 35.0,
            },
            TimeAllocation {
                category: SavingCategory::NonCashReleasing,
                fraction: 0.60,
                hourly_rate: 25.0,
            },
            TimeAllocation {
                category: SavingCategory::NoBenefit,
                fraction: 0.20,
                hourly_rate: 25.0,
            },
        ]
    }

    // Worked-example line: "100 × 0.5 × 5 shifts/week × 46 weeks ≈ 11,500
    // hours/year".
    #[test]
    fn annual_hours_saved_is_11_500() {
        let hours = annual_hours_saved(100.0, 0.5, 5.0, 46.0);
        assert!((hours - 11_500.0).abs() < 1e-9);
    }

    // Worked-example line: "the tempting headline is £287,500/year".
    #[test]
    fn tempting_headline_is_287_500() {
        // Minutes × salary: the canonical sin, shown only to be split honestly.
        let headline = non_cash_releasing_value(11_500.0, 25.0);
        assert!((headline - 287_500.0).abs() < 1e-6);
    }

    // Worked-example line: "2,300 hours × £35 agency rate = £80,500
    // cash-releasing".
    #[test]
    fn cash_releasing_slice_is_80_500() {
        // 20% of 11,500 h = 2,300 h at £35 agency rate.
        let cash = category_value(11_500.0, &split(), SavingCategory::CashReleasing);
        assert!((split()[0].hours(11_500.0) - 2_300.0).abs() < 1e-9);
        assert!((cash - 80_500.0).abs() < 1e-6);
    }

    // Worked-example line: "6,900 hours × £25 = £172,500 non-cash-releasing
    // capacity".
    #[test]
    fn non_cash_releasing_capacity_is_172_500() {
        // 60% of 11,500 h = 6,900 h at £25 employer cost.
        let capacity = category_value(11_500.0, &split(), SavingCategory::NonCashReleasing);
        assert!((split()[1].hours(11_500.0) - 6_900.0).abs() < 1e-9);
        assert!((capacity - 172_500.0).abs() < 1e-6);
    }

    // Worked-example line: "20% dissipates into breaks and interruptions: £0".
    #[test]
    fn dissipated_slice_is_worth_zero() {
        let nothing = category_value(11_500.0, &split(), SavingCategory::NoBenefit);
        assert!(nothing.abs() < 1e-9);
    }

    // Worked-example line: "a business case that presents £80.5k cash +
    // £172.5k capacity is credible" — smaller than the £287.5k headline.
    #[test]
    fn credible_case_totals_cash_plus_capacity() {
        let cash = category_value(11_500.0, &split(), SavingCategory::CashReleasing);
        let capacity = category_value(11_500.0, &split(), SavingCategory::NonCashReleasing);
        // £80.5k cash + £172.5k capacity, reported separately — never summed
        // into a single "savings" figure; the sum below is only the check
        // that the credible case is smaller than the £287.5k headline.
        assert!(cash + capacity < 287_500.0);
    }

    // Formula line: "Cash-releasing saving = budget line before − budget
    // line after" — the £80,500 as a genuine budget delta.
    #[test]
    fn cash_releasing_saving_is_a_budget_delta() {
        let saving = cash_releasing_saving(500_000.0, 419_500.0);
        assert!((saving - 80_500.0).abs() < 1e-6);
    }
}
