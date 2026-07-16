//! # Opportunity Cost
//!
//! Opportunity cost is the value of the best alternative you give up when you
//! commit a resource. In a health system with a fixed budget, spending
//! £1 million on one thing means £1 million of health *not* produced somewhere
//! else.
//!
//! A new technology is never funded from "extra" money — it displaces
//! something. The question a payer actually asks is not "is this good?" but
//! "is this better than what the same money currently buys?" A proposal must
//! therefore be compared against the best forgone alternative — never against
//! "do nothing".
//!
//! ## Formula
//!
//! ```text
//! Opportunity cost of choosing A = value of best forgone alternative B
//! Net gain from A = value(A) − value(B)
//! ```
//!
//! Legend:
//! - `A` — the option chosen (funded).
//! - `B` — the best alternative use of the same resource, forgone by choosing A.
//! - `value(·)` — the value each option would produce (money, QALYs, capacity).
//!
//! ## Why it matters
//!
//! Opportunity cost is the deepest idea in health economics, and the one
//! software engineers most often skip. Health budgets are fixed in any given
//! year, so funding one option displaces the health the same money would have
//! produced elsewhere. This is why cost-effectiveness thresholds exist at all:
//! the threshold is an estimate of the health that money buys at the margin of
//! the current system. The empirical benchmark: Claxton et al. (2015)
//! estimated the NHS produces one QALY for roughly **£13,000** at the margin —
//! so £13,000 spent on a technology that produces less than one QALY makes the
//! nation *less* healthy, even if the technology "works."
//!
//! ## Example
//!
//! An NHS trust's transformation budget can fund exactly one of Option A
//! (e-rostering, saving £400,000/year) or Option B (discharge coordination,
//! freeing 2,000 bed days/year at ~£150 per bed day actually freed).
//!
//! ```rust
//! use health_economics::opportunity_cost::{
//!     bed_day_savings_value, net_gain, opportunity_cost, qalys_displaced,
//!     NHS_MARGINAL_COST_PER_QALY_GBP,
//! };
//!
//! // Option B: 2,000 bed days/year × £150 per bed day freed = £300,000/year.
//! let option_b = bed_day_savings_value(2_000.0, 150.0);
//! assert_eq!(option_b, 300_000.0);
//!
//! // The opportunity cost of funding Option A is Option B's £300,000.
//! let oc = opportunity_cost(&[option_b]).unwrap();
//! assert_eq!(oc, 300_000.0);
//!
//! // The net case for A is only the £100,000 difference, not A's headline £400,000.
//! let net = net_gain(400_000.0, oc);
//! assert_eq!(net, 100_000.0);
//!
//! // At the NHS margin (Claxton et al. 2015), £13,000 of displaced spend is one QALY.
//! let displaced = qalys_displaced(13_000.0, NHS_MARGINAL_COST_PER_QALY_GBP).unwrap();
//! assert_eq!(displaced, 1.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineering capacity is a fixed budget too — roadmap slots, not pounds; a
//!   platform team funding tool A that saves engineer-hours at £500/hour when
//!   tool B delivers the same at £200/hour is destroying capacity.
//! - Always name the comparator ("versus what?").
//! - Value engineer time at what it would otherwise produce, not at salary
//!   alone.
//! - Treat "we have budget left" as the beginning of the analysis, not the
//!   end.
//!
//! ## Pitfalls
//!
//! - **Comparing against nothing.** The correct comparator is the next-best
//!   use of the money, which is rarely "do nothing."
//! - **Assuming saved time has zero opportunity cost.** Time saved is only
//!   valuable if redeployed to something valuable (see cash-releasing vs
//!   non-cash-releasing savings).
//! - **Ignoring displacement.** "The budget will expand to fit" is almost
//!   never true in a national health service in-year.
//!
//! ## Sources
//!
//! - Claxton K, et al. "Methods for the estimation of the NICE cost
//!   effectiveness threshold." Health Technology Assessment 2015;19(14).
//!   <https://www.journalslibrary.nihr.ac.uk/hta/hta19140/>
//! - York Health Economics Consortium glossary.
//!   <https://yhec.co.uk/glossary/opportunity-cost/>
//!
//! Topic doc: health-economics-metrics/topics/opportunity-cost.md

/// Claxton et al. (2015) estimate of what the NHS pays for one QALY at the margin.
///
/// Roughly £13,000 (GBP per QALY). Spending that displaces care cheaper than
/// this makes the nation less healthy: any technology consuming £13,000 of
/// NHS budget must produce at least one QALY just to break even in health
/// terms.
pub const NHS_MARGINAL_COST_PER_QALY_GBP: f64 = 13_000.0;

/// The opportunity cost of a choice: the value of the best forgone alternative.
///
/// Scans the supplied alternative values and returns the largest — the value
/// destroyed by not funding it. Values may be in any consistent unit (£/year,
/// QALYs, appointments); negative values are allowed (an alternative can be
/// net-harmful).
///
/// # Arguments
///
/// * `forgone_alternative_values` — value of each alternative that the chosen
///   option displaces, in a consistent unit (e.g. £/year).
///
/// # Returns
///
/// The maximum of the supplied values, or `None` when the slice is empty (no
/// alternatives means opportunity cost is undefined, not zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::opportunity_cost::opportunity_cost;
///
/// // Option B (£300,000/year) is the only forgone alternative.
/// assert_eq!(opportunity_cost(&[300_000.0]), Some(300_000.0));
///
/// // The best of several alternatives is the opportunity cost.
/// assert_eq!(opportunity_cost(&[120_000.0, 300_000.0, 90_000.0]), Some(300_000.0));
///
/// // No alternatives supplied: undefined.
/// assert_eq!(opportunity_cost(&[]), None);
/// ```
pub fn opportunity_cost(forgone_alternative_values: &[f64]) -> Option<f64> {
    forgone_alternative_values
        .iter()
        .copied()
        .fold(None, |best, v| match best {
            None => Some(v),
            Some(b) => Some(if v > b { v } else { b }),
        })
}

/// Net gain from choosing an option: its value minus the value of the best forgone alternative.
///
/// This — not the option's headline value — is the honest case for the
/// option. Both arguments must be in the same unit (e.g. £/year). A negative
/// result means the alternative was the better use of the resource.
///
/// # Arguments
///
/// * `value_chosen` — value produced by the option being funded (e.g. £/year).
/// * `value_best_alternative` — value of the best forgone alternative, same
///   unit.
///
/// # Returns
///
/// `value_chosen − value_best_alternative`.
///
/// # Examples
///
/// ```rust
/// use health_economics::opportunity_cost::net_gain;
///
/// // Option A's headline £400,000 versus Option B's £300,000:
/// // the net case for A is only the £100,000 difference.
/// assert_eq!(net_gain(400_000.0, 300_000.0), 100_000.0);
/// ```
pub fn net_gain(value_chosen: f64, value_best_alternative: f64) -> f64 {
    value_chosen - value_best_alternative
}

/// Annual cash value of freeing bed days, at a marginal cost per bed day actually freed.
///
/// This is the worked example's Option B. Use the *marginal* cost of a bed
/// day (what is actually saved or redeployable when the bed empties), not the
/// average fully-loaded cost.
///
/// # Arguments
///
/// * `bed_days_freed` — bed days released per year.
/// * `marginal_cost_per_bed_day` — £ value per bed day actually freed
///   (worked example: ~£150).
///
/// # Returns
///
/// The annual value in £: `bed_days_freed × marginal_cost_per_bed_day`.
///
/// # Examples
///
/// ```rust
/// use health_economics::opportunity_cost::bed_day_savings_value;
///
/// // 2,000 bed days/year at ~£150 per bed day freed = £300,000/year.
/// assert_eq!(bed_day_savings_value(2_000.0, 150.0), 300_000.0);
/// ```
pub fn bed_day_savings_value(bed_days_freed: f64, marginal_cost_per_bed_day: f64) -> f64 {
    bed_days_freed * marginal_cost_per_bed_day
}

/// QALYs displaced (not produced elsewhere) by diverting spend from the system margin.
///
/// A system that produces one QALY per `marginal_cost_per_qaly` loses
/// `spend / marginal_cost_per_qaly` QALYs when that spend is diverted to
/// something else. Use [`NHS_MARGINAL_COST_PER_QALY_GBP`] (£13,000) for the
/// NHS benchmark.
///
/// # Arguments
///
/// * `spend` — money diverted, in £.
/// * `marginal_cost_per_qaly` — £ the system needs to produce one QALY at the
///   margin.
///
/// # Returns
///
/// `Some(spend / marginal_cost_per_qaly)`, or `None` when
/// `marginal_cost_per_qaly` is zero (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::opportunity_cost::{
///     qalys_displaced, NHS_MARGINAL_COST_PER_QALY_GBP,
/// };
///
/// // £13,000 diverted from the NHS margin displaces exactly one QALY.
/// assert_eq!(qalys_displaced(13_000.0, NHS_MARGINAL_COST_PER_QALY_GBP), Some(1.0));
///
/// // A zero marginal cost per QALY is undefined.
/// assert_eq!(qalys_displaced(1_000.0, 0.0), None);
/// ```
pub fn qalys_displaced(spend: f64, marginal_cost_per_qaly: f64) -> Option<f64> {
    if marginal_cost_per_qaly == 0.0 {
        None
    } else {
        Some(spend / marginal_cost_per_qaly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Option B: 2,000 bed days/year at ~£150 per bed day freed = £300,000/year.
    #[test]
    fn option_b_bed_day_value_is_300_000() {
        // Worked example: "2,000 bed days/year. At a marginal cost of about
        // £150 per bed day actually freed, that is £300,000/year."
        let b = bed_day_savings_value(2_000.0, 150.0);
        assert!((b - 300_000.0).abs() < TOL);
    }

    /// The opportunity cost of funding Option A is Option B's £300,000.
    #[test]
    fn opportunity_cost_of_a_is_best_forgone_alternative() {
        // Worked example: "The opportunity cost of A is B's £300,000 + patient benefit."
        let oc = opportunity_cost(&[300_000.0]).unwrap();
        assert!((oc - 300_000.0).abs() < TOL);
    }

    /// The net case for A (£400,000 headline) versus B (£300,000) is only the
    /// £100,000 difference, not A's headline £400,000.
    #[test]
    fn net_case_for_a_is_the_difference_not_the_headline() {
        // Worked example: "the *net* case for A is only the difference, not
        // A's headline £400,000."
        let net = net_gain(400_000.0, 300_000.0);
        assert!((net - 100_000.0).abs() < TOL);
    }

    /// £13,000 buys one QALY at the NHS margin (Claxton et al. 2015), so a
    /// technology consuming £13,000 must produce at least one QALY.
    #[test]
    fn thirteen_thousand_pounds_displaces_one_qaly() {
        // Doc benchmark: "the NHS produces one QALY for roughly £13,000 at the margin."
        let q = qalys_displaced(13_000.0, NHS_MARGINAL_COST_PER_QALY_GBP).unwrap();
        assert!((q - 1.0).abs() < TOL);
    }

    // Edge case: no alternatives supplied — opportunity cost is undefined.
    #[test]
    fn no_alternatives_means_no_defined_opportunity_cost() {
        assert!(opportunity_cost(&[]).is_none());
    }

    // Edge case: zero marginal cost per QALY makes the displacement ratio undefined.
    #[test]
    fn zero_marginal_cost_per_qaly_is_undefined() {
        assert!(qalys_displaced(1_000.0, 0.0).is_none());
    }
}
