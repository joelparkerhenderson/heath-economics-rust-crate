//! # Cost-Benefit Analysis (CBA)
//!
//! CBA values both costs *and* outcomes in money. It is the only analysis
//! type that can answer "is this worth doing at all?" — not merely "which
//! option is best?" — because monetized benefits can be compared directly
//! against costs.
//!
//! Health effects can enter monetized as QALYs × λ (a willingness-to-pay
//! threshold). The Green Book also mandates **optimism-bias adjustments** —
//! uplifting cost estimates and haircutting benefits by evidence-based
//! percentages, because appraisals are systematically rosy.
//!
//! ## Formula
//!
//! ```text
//! NPV (net present social value) = Σ_t [ (Benefits_t − Costs_t) / (1 + r)^t ]
//! BCR (benefit-cost ratio)       = PV(benefits) / PV(costs)
//!
//! Adopt if NPV > 0 (equivalently BCR > 1); rank by NPV, not BCR.
//!
//! Benefits_t, Costs_t — monetized benefits and costs in year t
//! r                   — discount rate; 3.5% (Green Book social time preference rate)
//! PV(x)               — present value of the stream x, discounted at r
//! ```
//!
//! ## Why it matters
//!
//! CBA is the standard of the UK's HM Treasury **Green Book** for all public
//! spending appraisal, health included when outcomes can be monetized.
//! Where CEA/CUA stop at "cost per unit of health," CBA prices the health
//! itself (QALY × threshold value) and everything else — time, travel,
//! carbon — and reports one net figure at the 3.5% social time preference
//! rate. Every full NHS digital business case contains a CBA-shaped
//! economic case, and cases are expected to survive their own optimism-bias
//! adjustment.
//!
//! ## Example
//!
//! An e-referral system over a 5-year horizon at 3.5% discount: build £1.2M
//! (year 0), run £300k/yr; benefits £1,130k/yr. PV costs ≈ £2,555k, PV
//! benefits ≈ £5,102k, NPV ≈ +£2,547k, BCR ≈ 2.0 — and after optimism bias
//! (+40% build cost, −20% benefits) NPV is still ≈ +£1,047k.
//!
//! ```rust
//! use health_economics::cost_benefit_analysis::{
//!     GREEN_BOOK_DISCOUNT_RATE, annuity_factor, benefit_cost_ratio, net_present_value,
//!     optimism_bias_benefit_haircut, optimism_bias_cost_uplift,
//! };
//!
//! // Benefits £1,130k/yr = 250k admin + 280k diagnostics + 40,000 h × £15.
//! let annual_benefits: f64 = 250_000.0 + 280_000.0 + 40_000.0 * 15.0;
//! assert!((annual_benefits - 1_130_000.0).abs() < 1e-6);
//!
//! // 5-year annuity factor at 3.5% ≈ 4.515.
//! let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
//! assert!((af - 4.515).abs() < 5e-4);
//!
//! // PV costs = 1,200k + 300k × 4.515 ≈ £2,555k; PV benefits ≈ £5,102k.
//! let pv_costs = 1_200_000.0 + 300_000.0 * af;
//! let pv_benefits = annual_benefits * af;
//! assert!((pv_costs - 2_555_000.0).abs() < 1_000.0);
//! assert!((pv_benefits - 5_102_000.0).abs() < 1_000.0);
//!
//! // NPV = 5,102 − 2,555 ≈ +£2,547k; BCR ≈ 2.0.
//! let npv = net_present_value(pv_benefits, pv_costs);
//! assert!((npv - 2_547_000.0).abs() < 1_000.0);
//! assert!((benefit_cost_ratio(pv_benefits, pv_costs).unwrap() - 2.0).abs() < 0.01);
//!
//! // Optimism bias: +40% build, −20% benefits → NPV ≈ +£1,047k, still positive.
//! let adj_costs = optimism_bias_cost_uplift(1_200_000.0, 0.40) + 300_000.0 * af;
//! let adj_benefits = optimism_bias_benefit_haircut(pv_benefits, 0.20);
//! let adj_npv = net_present_value(adj_benefits, adj_costs);
//! assert!((adj_npv - 1_047_000.0).abs() < 1_000.0);
//! assert!(adj_npv > 0.0);
//! ```
//!
//! ## Software engineering connection
//!
//! Engineering business cases are informal CBAs. The Green Book upgrades
//! worth stealing:
//!
//! - **Optimism bias as a standard uplift** — engineers underestimate
//!   migration cost as reliably as ministries underestimate infrastructure
//!   cost; apply a stated uplift instead of pretending this time is
//!   different.
//! - **Monetize the dominant benefit honestly or not at all** — patient/user
//!   time is monetized at defensible rates; "brand value" is not.
//! - **NPV ranks, BCR doesn't**: a tiny project with BCR 5 can matter less
//!   than a big one with BCR 1.6.
//!
//! ## Pitfalls
//!
//! - **Monetizing the unmonetizable** to inflate benefits (morale,
//!   "strategic alignment") — keep those qualitative, per cost-consequence
//!   analysis.
//! - **Counting transfers as benefits**: money moving between public bodies
//!   nets to zero at the societal perspective.
//! - **No counterfactual**: benefits are measured against the do-minimum
//!   option, not against zero.
//!
//! ## Sources
//!
//! - HM Treasury, The Green Book.
//!   <https://www.gov.uk/government/publications/the-green-book-appraisal-and-evaluation-in-central-government/the-green-book-2020>
//! - Green Book discounting guidance.
//!   <https://www.gov.uk/government/publications/green-book-supplementary-guidance-discounting>
//!
//! Topic doc: health-economics-metrics/topics/cost-benefit-analysis.md

/// Green Book social time preference discount rate (3.5%), as a fraction.
pub const GREEN_BOOK_DISCOUNT_RATE: f64 = 0.035;

/// Discount factor for year t at rate r: 1 / (1 + r)^t.
///
/// Year 0 has factor 1.0 (undiscounted); later years shrink geometrically.
///
/// # Arguments
///
/// * `rate` — discount rate as a fraction (0.035 for the Green Book rate).
/// * `year` — the year the flow occurs, with 0 meaning "now".
///
/// # Returns
///
/// The multiplicative factor converting a year-t flow to present value.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::{
///     GREEN_BOOK_DISCOUNT_RATE, discount_factor,
/// };
///
/// assert!((discount_factor(GREEN_BOOK_DISCOUNT_RATE, 0.0) - 1.0).abs() < 1e-12);
/// // £1 in year 1 is worth 1/1.035 ≈ £0.966 today.
/// assert!((discount_factor(GREEN_BOOK_DISCOUNT_RATE, 1.0) - 1.0 / 1.035).abs() < 1e-12);
/// ```
pub fn discount_factor(rate: f64, year: f64) -> f64 {
    1.0 / (1.0 + rate).powf(year)
}

/// Annuity factor: present value of £1 received at the end of each of
/// years 1..=years, at the given discount rate.
///
/// Multiplying an equal annual flow by this factor gives its present value
/// in one step (the worked example's "300k × 4.515").
///
/// # Arguments
///
/// * `rate` — discount rate as a fraction.
/// * `years` — number of years the £1 recurs (years 1 through `years`).
///
/// # Returns
///
/// The sum of discount factors for years 1..=years (0.0 when `years` is 0).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::{
///     GREEN_BOOK_DISCOUNT_RATE, annuity_factor,
/// };
///
/// // 5 years at 3.5% → the worked example's annuity factor ≈ 4.515.
/// let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
/// assert!((af - 4.515).abs() < 5e-4);
/// ```
pub fn annuity_factor(rate: f64, years: u32) -> f64 {
    // Σ_{t=1..years} 1/(1+r)^t — flows land at the END of each year.
    (1..=years).map(|t| discount_factor(rate, t as f64)).sum()
}

/// Present value of a stream of cash flows, where index 0 is year 0
/// (undiscounted), index 1 is year 1, and so on.
///
/// # Arguments
///
/// * `flows_by_year` — cash flows indexed by year, in currency.
/// * `rate` — discount rate as a fraction.
///
/// # Returns
///
/// The discounted sum of the stream (0.0 for an empty slice).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::{
///     GREEN_BOOK_DISCOUNT_RATE, present_value,
/// };
///
/// // Worked-example cost stream: build £1.2M in year 0, £300k in years 1–5
/// // → PV ≈ £2,555k.
/// let flows = [1_200_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0];
/// let pv = present_value(&flows, GREEN_BOOK_DISCOUNT_RATE);
/// assert!((pv - 2_555_000.0).abs() < 1_000.0);
/// ```
pub fn present_value(flows_by_year: &[f64], rate: f64) -> f64 {
    flows_by_year
        .iter()
        .enumerate()
        // Index doubles as the year: flow_t / (1+r)^t.
        .map(|(t, flow)| flow * discount_factor(rate, t as f64))
        .sum()
}

/// Net present (social) value = PV(benefits) − PV(costs).
///
/// The adoption rule: adopt if NPV > 0; rank competing projects by NPV.
///
/// # Arguments
///
/// * `pv_benefits` — present value of the monetized benefit stream.
/// * `pv_costs` — present value of the cost stream, same currency and rate.
///
/// # Returns
///
/// The NPV in currency (positive means worth doing).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::net_present_value;
///
/// // Worked example: NPV = 5,102k − 2,555k = +£2,547k.
/// let npv = net_present_value(5_102_000.0, 2_555_000.0);
/// assert!((npv - 2_547_000.0).abs() < 1e-6);
/// ```
pub fn net_present_value(pv_benefits: f64, pv_costs: f64) -> f64 {
    pv_benefits - pv_costs
}

/// Benefit-cost ratio = PV(benefits) / PV(costs).
///
/// Adopt if > 1, but rank projects by NPV, not BCR: a tiny project with
/// BCR 5 can matter less than a big one with BCR 1.6.
///
/// # Arguments
///
/// * `pv_benefits` — present value of the benefit stream.
/// * `pv_costs` — present value of the cost stream.
///
/// # Returns
///
/// `Some(bcr)`, or `None` when `pv_costs` is zero (ratio undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::benefit_cost_ratio;
///
/// // Worked example: 5,102k / 2,555k ≈ 2.0.
/// let bcr = benefit_cost_ratio(5_102_000.0, 2_555_000.0).unwrap();
/// assert!((bcr - 2.0).abs() < 0.01);
/// assert!(benefit_cost_ratio(1.0, 0.0).is_none());
/// ```
pub fn benefit_cost_ratio(pv_benefits: f64, pv_costs: f64) -> Option<f64> {
    if pv_costs == 0.0 { None } else { Some(pv_benefits / pv_costs) }
}

/// Green Book optimism-bias uplift on a cost estimate: cost × (1 + uplift).
///
/// Appraisals are systematically rosy; the Green Book mandates uplifting
/// cost estimates by an evidence-based percentage before approval.
///
/// # Arguments
///
/// * `cost` — the raw cost estimate, in currency.
/// * `uplift` — fractional uplift (e.g. 0.40 for the worked example's +40%
///   on build cost).
///
/// # Returns
///
/// The uplifted cost estimate.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::optimism_bias_cost_uplift;
///
/// // +40% on the £1.2M build cost → £1.68M.
/// assert!((optimism_bias_cost_uplift(1_200_000.0, 0.40) - 1_680_000.0).abs() < 1e-6);
/// ```
pub fn optimism_bias_cost_uplift(cost: f64, uplift: f64) -> f64 {
    cost * (1.0 + uplift)
}

/// Green Book optimism-bias haircut on a benefit estimate: benefit × (1 − haircut).
///
/// The mirror of the cost uplift: benefits are trimmed by an evidence-based
/// percentage; a good case survives its own optimism.
///
/// # Arguments
///
/// * `benefit` — the raw benefit estimate (or its PV), in currency.
/// * `haircut` — fractional reduction (e.g. 0.20 for the worked example's
///   −20% on benefits).
///
/// # Returns
///
/// The haircut benefit estimate.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_benefit_analysis::optimism_bias_benefit_haircut;
///
/// // −20% on £5,102k of PV benefits ≈ £4,082k.
/// let adjusted = optimism_bias_benefit_haircut(5_102_000.0, 0.20);
/// assert!((adjusted - 4_081_600.0).abs() < 1e-6);
/// ```
pub fn optimism_bias_benefit_haircut(benefit: f64, haircut: f64) -> f64 {
    benefit * (1.0 - haircut)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: e-referral system, 5-year horizon, 3.5% discount.
    // Costs: build £1.2M (year 0), run £300k/yr (years 1–5).
    // Benefits: £1,130k/yr (years 1–5) = 250k admin + 280k diagnostics
    //           + 40,000 patient hours × £15 = 600k.

    // Worked-example line: "admin savings £250k/yr, avoided duplicate
    // diagnostics £280k/yr, patient time saved 40,000 hrs/yr × £15 = £600k/yr
    // → £1,130k/yr".
    #[test]
    fn annual_benefits_total_1_130k() {
        let benefits: f64 = 250_000.0 + 280_000.0 + 40_000.0 * 15.0;
        assert!((benefits - 1_130_000.0).abs() < 1e-6);
    }

    // Worked-example term: "annuity factor" 4.515 at 3.5% over 5 years.
    #[test]
    fn five_year_annuity_factor_is_about_4_515() {
        let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        assert!((af - 4.515).abs() < 5e-4);
    }

    // Worked-example line: "PV costs = 1,200k + 300k × 4.515 = £2,555k".
    #[test]
    fn pv_costs_is_about_2_555k() {
        let pv = 1_200_000.0 + 300_000.0 * annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        assert!((pv - 2_555_000.0).abs() < 1_000.0);
        // Same figure via the general PV of a year-indexed stream.
        let flows = [1_200_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0, 300_000.0];
        assert!((present_value(&flows, GREEN_BOOK_DISCOUNT_RATE) - pv).abs() < 1e-6);
    }

    // Worked-example line: "PV benefits = 1,130k × 4.515 = £5,102k".
    #[test]
    fn pv_benefits_is_about_5_102k() {
        let pv = 1_130_000.0 * annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        assert!((pv - 5_102_000.0).abs() < 1_000.0);
    }

    // Worked-example line: "NPV = 5,102 − 2,555 = +£2,547k".
    #[test]
    fn npv_is_about_plus_2_547k() {
        let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        let npv = net_present_value(1_130_000.0 * af, 1_200_000.0 + 300_000.0 * af);
        assert!((npv - 2_547_000.0).abs() < 1_000.0);
    }

    // Worked-example line: "BCR = 2.0".
    #[test]
    fn bcr_is_about_2() {
        let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        let bcr = benefit_cost_ratio(1_130_000.0 * af, 1_200_000.0 + 300_000.0 * af).unwrap();
        assert!((bcr - 2.0).abs() < 0.01);
    }

    // Worked-example line: "+40% on build cost, −20% on benefits: PV costs ≈
    // £3,035k, PV benefits ≈ £4,082k, NPV ≈ +£1,047k — still positive".
    #[test]
    fn optimism_bias_adjusted_case_still_positive() {
        // +40% on build cost, −20% on benefits.
        let af = annuity_factor(GREEN_BOOK_DISCOUNT_RATE, 5);
        let pv_costs = optimism_bias_cost_uplift(1_200_000.0, 0.40) + 300_000.0 * af;
        assert!((pv_costs - 3_035_000.0).abs() < 1_000.0);
        let pv_benefits = optimism_bias_benefit_haircut(1_130_000.0 * af, 0.20);
        assert!((pv_benefits - 4_082_000.0).abs() < 1_000.0);
        let npv = net_present_value(pv_benefits, pv_costs);
        assert!((npv - 1_047_000.0).abs() < 1_000.0);
        assert!(npv > 0.0, "the case should survive its own optimism");
    }

    // Edge case for the formula "BCR = PV(benefits) / PV(costs)": zero PV
    // costs make the ratio undefined.
    #[test]
    fn bcr_with_zero_costs_is_undefined() {
        assert!(benefit_cost_ratio(1.0, 0.0).is_none());
    }
}
