//! # Avoidable Outsourcing Costs
//!
//! When a trust cannot meet targets with internal capacity, it buys capacity
//! at premium rates: weekend overtime for its own staff, or outsourcing
//! procedures to private providers. The economic value of capacity-releasing
//! software includes the avoidable cost of that premium-rate work.
//!
//! Unlike ordinary capacity claims, avoided outsourcing is **cash-releasing**:
//! the invoice to the private provider genuinely doesn't get raised. The
//! claim requires released internal capacity to actually absorb the activity
//! — theatre sessions, beds, and staff must all be available (the binding
//! constraint governs).
//!
//! ## Formula
//!
//! ```text
//! Avoidable outsourcing cost = activity moved in-house × (outsourced unit price
//!                              − internal marginal cost per case)
//! ```
//!
//! Legend:
//! - `activity moved in-house` — cases repatriated from the external provider
//!   (count).
//! - `outsourced unit price` — price paid per case externally (currency).
//! - `internal marginal cost per case` — consumables + variable staffing for
//!   the extra activity only (currency); the fixed estate is already paid for.
//!
//! ## Why it matters
//!
//! Under elective-recovery pressure, trusts routinely pay private-sector spot
//! prices (often 1.2–1.5× the NHS scheme price) or premium
//! waiting-list-initiative rates to their own consultants for weekend lists.
//! Avoided outsourcing is one of the strongest benefit lines available to
//! software that increases internal throughput — and one of the easiest to
//! evidence, because the outsourcing spend is already a visible budget line.
//!
//! ## Example
//!
//! A trust outsources 800 cataract procedures/year at £900 each (£720,000/yr
//! external spend, versus scheme price ~£750). Theatre-scheduling software
//! repatriates 500 procedures at an internal marginal cost of ≈£350/case:
//! saving = 500 × (900 − 350) = £275,000/year, cash-releasing. Software cost
//! £90,000/year → net ≈ +£185,000/year in bankable cash.
//!
//! ```rust
//! use health_economics::avoidable_outsourcing_costs::{
//!     avoidable_outsourcing_saving, net_benefit, outsourcing_spend,
//! };
//!
//! // Baseline: 800 × £900 = £720,000/year of external spend.
//! let baseline = outsourcing_spend(800.0, 900.0);
//! assert!((baseline - 720_000.0).abs() < 1e-9);
//!
//! // Saving = 500 × (900 − 350) = £275,000/year — cash-releasing.
//! let saving = avoidable_outsourcing_saving(500.0, 900.0, 350.0);
//! assert!((saving - 275_000.0).abs() < 1e-9);
//!
//! // Software cost £90,000/year → net ≈ +£185,000/year in bankable cash.
//! let net = net_benefit(saving, 90_000.0);
//! assert!((net - 185_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - The direct analogue is **contractor and consultancy premium**: when
//!   internal engineering capacity can't meet commitments, orgs buy external
//!   capacity at 1.5–3× loaded internal rates.
//! - Platform and productivity investments that raise internal throughput
//!   should claim avoided contractor spend exactly as above — external
//!   day-rate minus internal marginal cost, times work repatriated.
//! - It is one of the few genuinely cash-releasing lines in a
//!   developer-productivity business case.
//! - The same caveat applies: the internal capacity must actually exist and
//!   be scheduled onto the repatriated work, or the claim is fiction.
//!
//! ## Pitfalls
//!
//! - **Claiming repatriation without the full capacity chain** — surgeons
//!   freed but no theatre slots (or engineers freed but no product-management
//!   bandwidth) repatriates nothing.
//! - **Comparing outsourced price to internal average cost** instead of
//!   marginal cost — understates the saving, oddly enough; the fixed costs
//!   run either way.
//! - **Quality/complexity asymmetry**: outsourced cases are often the simple
//!   ones; repatriating them changes internal case mix and unit costs.
//!
//! ## Sources
//!
//! - NHS England, elective care recovery plan.
//!   <https://www.england.nhs.uk/coronavirus/publication/delivery-plan-for-tackling-the-covid-19-backlog-of-elective-care/>
//! - NHS England, NHS Payment Scheme.
//!   <https://www.england.nhs.uk/pay-syst/national-tariff/national-tariff-payment-system/>
//!
//! Topic doc: health-economics-metrics/topics/avoidable-outsourcing-costs.md

/// Cash-releasing saving from repatriating outsourced activity.
///
/// cases moved in-house × (outsourced unit price − internal marginal cost per
/// case). Use *marginal* internal cost (consumables + variable staffing) —
/// the fixed estate runs either way, and comparing against average cost
/// understates the saving.
///
/// # Arguments
///
/// * `cases_moved_in_house` — activity repatriated (count).
/// * `outsourced_unit_price` — price per case paid externally (currency).
/// * `internal_marginal_cost_per_case` — consumables + variable staffing per
///   extra internal case (currency).
///
/// # Returns
///
/// The annual saving (currency units); negative if internal marginal cost
/// exceeds the external price.
///
/// # Examples
///
/// ```rust
/// use health_economics::avoidable_outsourcing_costs::avoidable_outsourcing_saving;
///
/// // Worked example: 500 × (£900 − £350) = £275,000/year — cash-releasing.
/// let saving = avoidable_outsourcing_saving(500.0, 900.0, 350.0);
/// assert!((saving - 275_000.0).abs() < 1e-9);
/// ```
pub fn avoidable_outsourcing_saving(
    cases_moved_in_house: f64,
    outsourced_unit_price: f64,
    internal_marginal_cost_per_case: f64,
) -> f64 {
    // Per-case premium avoided = external price − internal marginal cost.
    cases_moved_in_house * (outsourced_unit_price - internal_marginal_cost_per_case)
}

/// Annual outsourcing spend: cases outsourced × unit price paid externally.
///
/// The visible budget line that makes this benefit easy to evidence.
///
/// # Arguments
///
/// * `cases_outsourced` — cases sent to the external provider per year
///   (count).
/// * `outsourced_unit_price` — price per case paid externally (currency).
///
/// # Returns
///
/// Annual external spend (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::avoidable_outsourcing_costs::outsourcing_spend;
///
/// // Worked example baseline: 800 × £900 = £720,000/year.
/// let spend = outsourcing_spend(800.0, 900.0);
/// assert!((spend - 720_000.0).abs() < 1e-9);
///
/// // After repatriating 500: 300 × £900 = £270,000 remaining.
/// let remaining = outsourcing_spend(300.0, 900.0);
/// assert!((remaining - 270_000.0).abs() < 1e-9);
/// ```
pub fn outsourcing_spend(cases_outsourced: f64, outsourced_unit_price: f64) -> f64 {
    cases_outsourced * outsourced_unit_price
}

/// Premium multiple paid externally over the internal scheme (reference) price.
///
/// Private-sector spot prices often run 1.2–1.5× the NHS scheme price;
/// contractor day-rates run 1.5–3× loaded internal engineering rates.
///
/// # Arguments
///
/// * `outsourced_unit_price` — price per case paid externally (currency).
/// * `internal_scheme_price` — the internal scheme/reference price per case
///   (currency).
///
/// # Returns
///
/// The premium ratio (dimensionless; 1.0 = parity), or `None` if
/// `internal_scheme_price` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::avoidable_outsourcing_costs::outsourcing_premium_ratio;
///
/// // Worked example: £900 outsourced vs scheme price ~£750 → 1.2×.
/// let ratio = outsourcing_premium_ratio(900.0, 750.0).unwrap();
/// assert!((ratio - 1.2).abs() < 1e-9);
///
/// assert!(outsourcing_premium_ratio(900.0, 0.0).is_none());
/// ```
pub fn outsourcing_premium_ratio(
    outsourced_unit_price: f64,
    internal_scheme_price: f64,
) -> Option<f64> {
    if internal_scheme_price == 0.0 {
        None
    } else {
        Some(outsourced_unit_price / internal_scheme_price)
    }
}

/// Net annual benefit of the enabling software.
///
/// Repatriation saving minus the software's annual cost — the bankable-cash
/// line of the business case.
///
/// # Arguments
///
/// * `avoidable_outsourcing_saving` — annual repatriation saving (currency;
///   see [`avoidable_outsourcing_saving`]).
/// * `software_annual_cost` — annual cost of the software (currency).
///
/// # Returns
///
/// The net annual benefit (currency units); negative if the software costs
/// more than it saves.
///
/// # Examples
///
/// ```rust
/// use health_economics::avoidable_outsourcing_costs::net_benefit;
///
/// // Worked example: £275,000 saving − £90,000 software = +£185,000/year.
/// let net = net_benefit(275_000.0, 90_000.0);
/// assert!((net - 185_000.0).abs() < 1e-9);
/// ```
pub fn net_benefit(avoidable_outsourcing_saving: f64, software_annual_cost: f64) -> f64 {
    avoidable_outsourcing_saving - software_annual_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a trust outsources 800 cataract procedures/year at £900
    // each; theatre-scheduling software repatriates 500 of them; internal
    // marginal cost ≈ £350/case; software costs £90,000/year.

    #[test]
    fn baseline_external_spend_is_720_000() {
        // 800 × £900 = £720,000/year.
        let got = outsourcing_spend(800.0, 900.0);
        assert!((got - 720_000.0).abs() < 1e-9);
    }

    #[test]
    fn repatriation_saving_is_275_000_cash_releasing() {
        // 500 × (900 − 350) = £275,000/year.
        let got = avoidable_outsourcing_saving(500.0, 900.0, 350.0);
        assert!((got - 275_000.0).abs() < 1e-9);
    }

    #[test]
    fn remaining_outsourcing_is_270_000() {
        // 300 × £900 = £270,000 (was £720,000).
        let got = outsourcing_spend(300.0, 900.0);
        assert!((got - 270_000.0).abs() < 1e-9);
    }

    #[test]
    fn net_benefit_is_about_185_000_in_bankable_cash() {
        // £275,000 saving − £90,000 software = +£185,000/year.
        let saving = avoidable_outsourcing_saving(500.0, 900.0, 350.0);
        let got = net_benefit(saving, 90_000.0);
        assert!((got - 185_000.0).abs() < 1e-9);
    }

    #[test]
    fn spot_price_premium_over_scheme_price() {
        // £900 outsourced vs scheme price ~£750 → 1.2× — inside the doc's
        // quoted 1.2–1.5× spot-premium range.
        let got = outsourcing_premium_ratio(900.0, 750.0).unwrap();
        assert!((got - 1.2).abs() < 1e-9);
        assert!((1.2..=1.5).contains(&got));
    }

    #[test]
    fn zero_internal_price_returns_none() {
        // Edge-case semantics: the premium ratio is undefined without an
        // internal reference price.
        assert!(outsourcing_premium_ratio(900.0, 0.0).is_none());
    }
}
