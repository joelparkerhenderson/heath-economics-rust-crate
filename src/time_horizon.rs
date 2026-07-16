//! # Time Horizon
//!
//! The time horizon is the period over which an analysis counts costs and
//! effects. It must be long enough to capture all meaningful differences
//! between the options being compared.
//!
//! The horizon is the upper limit of the summation in any evaluation;
//! results should be reported with the horizon stated, ideally shown at
//! multiple horizons, with the break-even point made explicit.
//!
//! ## Formula
//!
//! ```text
//! Net present value = Σ (t = 0 … T) [ (Benefits_t − Costs_t) / (1 + r)^t ]
//!
//! T = time horizon (years)
//! r = discount rate
//! Benefits_t, Costs_t = benefits and costs accruing in year t
//! ```
//!
//! ## Why it matters
//!
//! Choose a short horizon and you miss late benefits (prevention) and late
//! costs (maintenance). Choose an over-long horizon and everything drowns
//! in uncertainty. Health technology assessment often uses a lifetime
//! horizon for treatments with mortality effects; budget impact analysis
//! deliberately uses a short 1–5 year horizon because its question is
//! affordability, not value. The horizon is a declared modeling choice, and
//! mismatched horizons are a classic way to game a comparison.
//!
//! ## Example
//!
//! An electronic prescribing system costs £2 million to implement and
//! £200,000/year to run. It prevents medication errors worth £600,000/year:
//!
//! ```rust
//! use health_economics::time_horizon::{
//!     net_benefit_at_horizon, break_even_horizon_years, net_benefit_by_horizons,
//! };
//!
//! // Horizon 1 year:  −2,000,000 − 200,000 + 600,000 = −£1,600,000
//! // Horizon 3 years: −2,000,000 + 3 × 400,000       = −£800,000
//! // Horizon 5 years: −2,000,000 + 5 × 400,000       =  £0
//! // Horizon 10 years:−2,000,000 + 10 × 400,000      = +£2,000,000
//! let report = net_benefit_by_horizons(2_000_000.0, 600_000.0, 200_000.0, &[1.0, 3.0, 5.0, 10.0]);
//! let expected = [-1_600_000.0, -800_000.0, 0.0, 2_000_000.0];
//! for ((_, got), want) in report.iter().zip(expected.iter()) {
//!     assert!((got - want).abs() < 1e-9);
//! }
//!
//! // The honest report states the break-even point: 5 years.
//! let t = break_even_horizon_years(2_000_000.0, 600_000.0, 200_000.0).unwrap();
//! assert!((t - 5.0).abs() < 1e-9);
//!
//! // Single-horizon check: the system "fails" at 3 years...
//! assert!(net_benefit_at_horizon(2_000_000.0, 600_000.0, 200_000.0, 3.0) < 0.0);
//! // ...and "succeeds" at 10. Neither is the true answer by itself.
//! assert!(net_benefit_at_horizon(2_000_000.0, 600_000.0, 200_000.0, 10.0) > 0.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Tool evaluations measured over one sprint systematically miss the
//!   learning-curve dip (costs front-loaded) and long-run maintenance
//!   (costs back-loaded).
//! - AI coding-assistant pilots measured in week 2 capture peak novelty,
//!   not steady state.
//! - Contract length ≠ benefit horizon: a 1-year SaaS contract can still be
//!   appraised over 5 years if you realistically expect renewal — but say so.
//! - Legacy replacement cases should run to the credible end-of-life of the
//!   old system, not to an arbitrary round number.
//!
//! ## Pitfalls
//!
//! - Horizon shopping: picking whichever horizon makes your option win.
//!   Pre-register the horizon before computing results.
//! - Different horizons for different options in the same comparison.
//! - Lifetime horizons without discounting or uncertainty analysis —
//!   year-30 benefits at face value are fiction. Pair long horizons with
//!   sensitivity analysis.
//!
//! ## Sources
//!
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//! - Sullivan SD, et al. "Budget Impact Analysis — Principles of Good
//!   Practice: Report of the ISPOR 2012 Budget Impact Analysis Good
//!   Practice II Task Force." Value in Health 2014;17(1):5–14.
//!   <https://pubmed.ncbi.nlm.nih.gov/24438712/>
//!
//! Topic doc: health-economics-metrics/topics/time-horizon.md

/// Net present value of a stream of net flows (benefits − costs).
///
/// The horizon is implicit in the slice length: `net_flows[t]` is the net
/// flow in year t, with t = 0 the first, undiscounted year.
///
/// # Arguments
///
/// * `net_flows` — net flow per year; index is the year (year 0 first).
/// * `discount_rate` — annual discount rate r as a fraction (e.g. 0.035).
///
/// # Returns
///
/// NPV in currency units. An empty slice returns 0.0.
///
/// # Examples
///
/// ```rust
/// use health_economics::time_horizon::net_present_value;
///
/// // E-prescribing case: year-0 outlay −£2M, then +£400k in years 1..=10.
/// let mut flows = vec![-2_000_000.0];
/// flows.extend(std::iter::repeat(400_000.0).take(10));
/// // Undiscounted (r = 0) this matches the +£2M horizon-10 figure.
/// assert!((net_present_value(&flows, 0.0) - 2_000_000.0).abs() < 1e-9);
/// // Discounted at 3.5%, later benefits shrink and NPV falls below £2M.
/// let npv = net_present_value(&flows, 0.035);
/// assert!(npv < 2_000_000.0 && npv > 0.0);
/// ```
pub fn net_present_value(net_flows: &[f64], discount_rate: f64) -> f64 {
    net_flows
        .iter()
        .enumerate()
        // Year-t flow discounted by (1 + r)^t; t = 0 is undiscounted.
        .map(|(t, flow)| flow / (1.0 + discount_rate).powi(t as i32))
        .sum()
}

/// Undiscounted net benefit at a given horizon for the common pattern of an
/// up-front implementation cost followed by constant annual costs and
/// benefits.
///
/// # Arguments
///
/// * `implementation_cost` — one-off up-front cost.
/// * `annual_benefit` — constant benefit per year.
/// * `annual_running_cost` — constant running cost per year.
/// * `horizon_years` — the horizon T, years.
///
/// # Returns
///
/// −implementation + horizon × (annual benefit − annual cost), undiscounted.
///
/// # Examples
///
/// ```rust
/// use health_economics::time_horizon::net_benefit_at_horizon;
///
/// // Doc: horizon 5 years: −2,000,000 + 5 × 400,000 = £0 (break-even).
/// let net = net_benefit_at_horizon(2_000_000.0, 600_000.0, 200_000.0, 5.0);
/// assert!(net.abs() < 1e-9);
/// ```
pub fn net_benefit_at_horizon(
    implementation_cost: f64,
    annual_benefit: f64,
    annual_running_cost: f64,
    horizon_years: f64,
) -> f64 {
    -implementation_cost + horizon_years * (annual_benefit - annual_running_cost)
}

/// Break-even horizon in years: implementation cost / annual net benefit.
///
/// The horizon at which undiscounted net benefit crosses zero — the number
/// the honest report states alongside a justified horizon.
///
/// # Arguments
///
/// * `implementation_cost` — one-off up-front cost.
/// * `annual_benefit` — constant benefit per year.
/// * `annual_running_cost` — constant running cost per year.
///
/// # Returns
///
/// Break-even horizon in years, or `None` if the annual net benefit
/// (benefit − running cost) is zero, in which case the case never breaks
/// even. A negative annual net benefit yields a negative (meaningless)
/// horizon — the case simply never pays back.
///
/// # Examples
///
/// ```rust
/// use health_economics::time_horizon::break_even_horizon_years;
///
/// // Doc: £2M implementation, £400k/year net → break-even at 5 years.
/// let t = break_even_horizon_years(2_000_000.0, 600_000.0, 200_000.0).unwrap();
/// assert!((t - 5.0).abs() < 1e-9);
/// ```
pub fn break_even_horizon_years(
    implementation_cost: f64,
    annual_benefit: f64,
    annual_running_cost: f64,
) -> Option<f64> {
    let annual_net = annual_benefit - annual_running_cost;
    if annual_net == 0.0 {
        None
    } else {
        Some(implementation_cost / annual_net)
    }
}

/// Report the same case at several horizons — the honest presentation the
/// topic doc asks for.
///
/// # Arguments
///
/// * `implementation_cost` — one-off up-front cost.
/// * `annual_benefit` — constant benefit per year.
/// * `annual_running_cost` — constant running cost per year.
/// * `horizons_years` — horizons at which to report.
///
/// # Returns
///
/// `(horizon, undiscounted net benefit)` pairs, in the order given.
///
/// # Examples
///
/// ```rust
/// use health_economics::time_horizon::net_benefit_by_horizons;
///
/// // Doc table: −£1.6M at 1yr, −£800k at 3yr, £0 at 5yr, +£2M at 10yr.
/// let report = net_benefit_by_horizons(2_000_000.0, 600_000.0, 200_000.0, &[1.0, 3.0, 5.0, 10.0]);
/// assert!((report[0].1 - (-1_600_000.0)).abs() < 1e-9);
/// assert!((report[3].1 - 2_000_000.0).abs() < 1e-9);
/// ```
pub fn net_benefit_by_horizons(
    implementation_cost: f64,
    annual_benefit: f64,
    annual_running_cost: f64,
    horizons_years: &[f64],
) -> Vec<(f64, f64)> {
    horizons_years
        .iter()
        .map(|&h| {
            (h, net_benefit_at_horizon(implementation_cost, annual_benefit, annual_running_cost, h))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // E-prescribing worked example: £2M implementation, £200k/year to run,
    // prevents errors worth £600k/year.
    const IMPL: f64 = 2_000_000.0;
    const BENEFIT: f64 = 600_000.0;
    const RUN: f64 = 200_000.0;

    // Doc table: "Horizon 1 year: −2,000,000 − 200,000 + 600,000 = −£1,600,000".
    #[test]
    fn horizon_1_year_is_minus_1_6m() {
        let net = net_benefit_at_horizon(IMPL, BENEFIT, RUN, 1.0);
        assert!((net - (-1_600_000.0)).abs() < 1e-9);
    }

    // Doc table: "Horizon 3 years: −2,000,000 + 3 × 400,000 = −£800,000".
    #[test]
    fn horizon_3_years_is_minus_800k() {
        let net = net_benefit_at_horizon(IMPL, BENEFIT, RUN, 3.0);
        assert!((net - (-800_000.0)).abs() < 1e-9);
    }

    // Doc table: "Horizon 5 years: −2,000,000 + 5 × 400,000 = £0".
    #[test]
    fn horizon_5_years_is_zero() {
        let net = net_benefit_at_horizon(IMPL, BENEFIT, RUN, 5.0);
        assert!(net.abs() < 1e-9);
    }

    // Doc table: "Horizon 10 years: −2,000,000 + 10 × 400,000 = +£2,000,000".
    #[test]
    fn horizon_10_years_is_plus_2m() {
        let net = net_benefit_at_horizon(IMPL, BENEFIT, RUN, 10.0);
        assert!((net - 2_000_000.0).abs() < 1e-9);
    }

    // Doc: "the honest report states the break-even point" — 5 years here.
    #[test]
    fn break_even_is_5_years() {
        let t = break_even_horizon_years(IMPL, BENEFIT, RUN).unwrap();
        assert!((t - 5.0).abs() < 1e-9);
        assert!(break_even_horizon_years(IMPL, 400_000.0, 400_000.0).is_none());
    }

    // Doc: results "ideally shown at multiple horizons" — the full table.
    #[test]
    fn multi_horizon_report_matches_worked_example_table() {
        let report = net_benefit_by_horizons(IMPL, BENEFIT, RUN, &[1.0, 3.0, 5.0, 10.0]);
        let expected = [-1_600_000.0, -800_000.0, 0.0, 2_000_000.0];
        for ((_, got), want) in report.iter().zip(expected.iter()) {
            assert!((got - want).abs() < 1e-9);
        }
    }

    // Doc (The math): NPV formula with r = 0 reproduces the undiscounted
    // horizon-10 figure of +£2M.
    #[test]
    fn npv_at_zero_discount_matches_undiscounted_horizon_10() {
        // t = 0 carries implementation plus the first year's net flow is modeled
        // as: year 0 outlay −2M, then +400k in each of years 1..=10.
        let mut flows = vec![-IMPL];
        flows.extend(std::iter::repeat_n(BENEFIT - RUN, 10));
        let npv = net_present_value(&flows, 0.0);
        assert!((npv - 2_000_000.0).abs() < 1e-9);
    }

    // Doc pitfall: "year-30 benefits at face value are fiction" —
    // discounting must strictly shrink long-horizon benefits.
    #[test]
    fn discounting_shrinks_long_horizon_benefits() {
        // Doc pitfall: year-30 benefits at face value are fiction — discounted
        // NPV must be strictly below the undiscounted figure.
        let mut flows = vec![-IMPL];
        flows.extend(std::iter::repeat_n(BENEFIT - RUN, 10));
        let npv = net_present_value(&flows, 0.035);
        assert!(npv < 2_000_000.0);
        assert!(npv > 0.0);
    }
}
