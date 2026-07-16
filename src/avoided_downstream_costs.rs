//! # Avoided Downstream Costs
//!
//! Avoided downstream costs (cost offsets) are future treatment expenses
//! prevented by earlier or better action, netted against the intervention's
//! own cost. Offsets are the mechanism by which an intervention can become
//! *dominant* — cheaper **and** better — and they are also the most
//! double-counted, over-claimed line in health economics.
//!
//! A valid offset must be: **attributable** (causally linked via comparator
//! evidence), **marginal** (the money actually stops being spent, at marginal
//! not average cost), **probability-weighted** (by P(the downstream event
//! would have occurred)), **discounted** (future avoided costs at present
//! value), and **unique** (counted once, in one benefit line).
//!
//! ## Formula
//!
//! ```text
//! Net cost = intervention cost − Σ offsets
//!
//! Offset value = P(future event) × counterfactual cost × discount factor
//! ```
//!
//! Legend:
//! - `intervention cost` — the intervention's own cost (currency).
//! - `Σ offsets` — sum of valid (attributable, marginal,
//!   probability-weighted, discounted, unique) offsets (currency).
//! - `P(future event)` — probability the downstream event would have
//!   occurred without the intervention (0.0–1.0).
//! - `counterfactual cost` — what the event would have cost, at marginal
//!   cost (currency).
//! - `discount factor` — 1 / (1 + rate)^years for an event `years` ahead.
//!
//! ## Why it matters
//!
//! Almost every digital health value proposition contains an offset claim:
//! "our app prevents admissions," "our alerts prevent deterioration," "our
//! platform avoids duplicate tests." When offsets are real, they transform
//! the economics — a £600k offset can make an ICER case. Payers know this,
//! so offset claims attract the hardest scrutiny in any appraisal; the
//! credibility rules above are what separate a fundable model from marketing
//! (offsets rarely exceed costs — Cohen, Neumann & Weinstein, NEJM 2008).
//!
//! ## Example
//!
//! A wound-monitoring app for 5,000 post-surgical patients claims to avoid
//! infection-related readmissions. Baseline readmission 4.0%; with app (RCT)
//! 3.1% → 45 events avoided/year × £3,200 marginal cost per readmission
//! spell = £144,000/year offset. App cost 5,000 × £20 = £100,000/year → net
//! cost −£44,000: genuinely cost-saving, with attribution from an RCT,
//! marginal costing, and probability from trial data.
//!
//! ```rust
//! use health_economics::avoided_downstream_costs::{
//!     attributable_events_avoided, intervention_cost, net_cost, offset_value,
//! };
//!
//! // Attributable events avoided = 5,000 × (0.040 − 0.031) = 45/year.
//! let events = attributable_events_avoided(5_000.0, 0.040, 0.031);
//! assert!((events - 45.0).abs() < 1e-9);
//!
//! // Offset = 45 × £3,200 = £144,000/year (marginal cost, this trust).
//! let offset = offset_value(events, 3_200.0);
//! assert!((offset - 144_000.0).abs() < 1e-9);
//!
//! // App cost = 5,000 × £20 = £100,000/year.
//! let cost = intervention_cost(5_000.0, 20.0);
//! assert!((cost - 100_000.0).abs() < 1e-9);
//!
//! // Net cost = −£44,000 → genuinely cost-saving.
//! let net = net_cost(cost, &[offset]);
//! assert!((net - (-44_000.0)).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! "This migration avoids the future rewrite" is an offset claim, and the
//! health-economics rules make it honest:
//!
//! - **Counterfactual cost**: what would the rewrite actually cost,
//!   evidenced how?
//! - **Probability**: how likely is that future? (Not 100% — products get
//!   killed, priorities change.)
//! - **Discounting**: a rewrite avoided in year 4 at 3.5–10% discount is
//!   worth much less than face value.
//! - **Uniqueness**: don't also claim the same avoided rewrite in the
//!   tech-debt line and the retention line.
//! - Write `Offset value = P(future event) × counterfactual cost × discount
//!   factor` in the proposal and watch the estimate become debatable — which
//!   is the point.
//!
//! ## Pitfalls
//!
//! - **Double counting** — the same avoided admission claimed as offset, bed
//!   days, and QALYs-with-cost-attached.
//! - **Average-cost offsets** for events whose fixed costs continue
//!   regardless.
//! - **Silent 100% probability** on downstream events that were merely
//!   possible.
//! - **Offsets to other budgets** presented as savings to the payer being
//!   asked to pay.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: cost offset.
//!   <https://yhec.co.uk/glossary/cost-offset/>
//! - Cohen JT, Neumann PJ, Weinstein MC. NEJM 2008 (offsets rarely exceed
//!   costs). <https://www.nejm.org/doi/full/10.1056/NEJMp0708558>
//!
//! Topic doc: health-economics-metrics/topics/avoided-downstream-costs.md

/// Attributable events avoided per year.
///
/// population × (baseline event rate − event rate with the intervention).
/// Attribution must come from comparator evidence such as an RCT — the rate
/// difference *is* the causal claim.
///
/// # Arguments
///
/// * `population` — people covered by the intervention (count).
/// * `baseline_event_rate` — event rate without the intervention (fraction,
///   0.0–1.0).
/// * `intervention_event_rate` — event rate with the intervention (fraction,
///   0.0–1.0).
///
/// # Returns
///
/// Events avoided per period (count); negative if the intervention arm has
/// *more* events.
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::attributable_events_avoided;
///
/// // Worked example: baseline readmission 4.0%, with app (RCT) 3.1% →
/// // 5,000 × 0.009 = 45 events avoided/year.
/// let events = attributable_events_avoided(5_000.0, 0.040, 0.031);
/// assert!((events - 45.0).abs() < 1e-9);
/// ```
pub fn attributable_events_avoided(
    population: f64,
    baseline_event_rate: f64,
    intervention_event_rate: f64,
) -> f64 {
    // Absolute risk reduction × population = attributable events.
    population * (baseline_event_rate - intervention_event_rate)
}

/// Offset value: events avoided × marginal (not average) cost per event.
///
/// Use the marginal cost at *this* provider — the fixed costs of events
/// whose infrastructure continues regardless are not saved.
///
/// # Arguments
///
/// * `events_avoided` — attributable events avoided (count; see
///   [`attributable_events_avoided`]).
/// * `marginal_cost_per_event` — marginal cost per avoided event (currency).
///
/// # Returns
///
/// The offset value per period (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::offset_value;
///
/// // Worked example: 45 × £3,200 = £144,000/year.
/// let offset = offset_value(45.0, 3_200.0);
/// assert!((offset - 144_000.0).abs() < 1e-9);
///
/// // The same claim at the £5,800 average cost inflates to £261,000 —
/// // and fails the marginal-costing test.
/// assert!((offset_value(45.0, 5_800.0) - 261_000.0).abs() < 1e-9);
/// ```
pub fn offset_value(events_avoided: f64, marginal_cost_per_event: f64) -> f64 {
    events_avoided * marginal_cost_per_event
}

/// Intervention cost: population covered × cost per person.
///
/// # Arguments
///
/// * `population` — people covered (count).
/// * `cost_per_person` — intervention cost per person per period (currency).
///
/// # Returns
///
/// The intervention's own cost per period (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::intervention_cost;
///
/// // Worked example: 5,000 patients × £20 = £100,000/year.
/// let cost = intervention_cost(5_000.0, 20.0);
/// assert!((cost - 100_000.0).abs() < 1e-9);
/// ```
pub fn intervention_cost(population: f64, cost_per_person: f64) -> f64 {
    population * cost_per_person
}

/// Net cost: intervention cost minus the sum of valid offsets.
///
/// Negative means genuinely cost-saving — the offset route to dominance
/// (cheaper *and* better). Each offset in the slice must be valid:
/// attributable, marginal, probability-weighted, discounted, and unique.
///
/// # Arguments
///
/// * `intervention_cost` — the intervention's own cost (currency; see
///   [`intervention_cost`]).
/// * `offsets` — valid offset values to net off (currency each).
///
/// # Returns
///
/// The net cost (currency units); negative when offsets exceed the
/// intervention cost.
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::net_cost;
///
/// // Worked example: £100,000 − £144,000 = −£44,000 → cost-saving.
/// let net = net_cost(100_000.0, &[144_000.0]);
/// assert!((net - (-44_000.0)).abs() < 1e-9);
/// assert!(net < 0.0);
/// ```
pub fn net_cost(intervention_cost: f64, offsets: &[f64]) -> f64 {
    intervention_cost - offsets.iter().sum::<f64>()
}

/// Discount factor for a cost avoided `years` in the future.
///
/// 1 / (1 + rate)^years — the present-value weight for a future avoided
/// cost. NICE's reference rate is 3.5%/year; commercial software cases often
/// use up to 10%.
///
/// # Arguments
///
/// * `annual_discount_rate` — discount rate as a fraction (e.g. 0.035 for
///   3.5%).
/// * `years` — years until the avoided cost would have been incurred.
///
/// # Returns
///
/// The discount factor (dimensionless, 1.0 at year 0, shrinking with time
/// for positive rates).
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::discount_factor;
///
/// // A rewrite avoided in year 4 at 3.5%: factor ≈ 0.871.
/// let df = discount_factor(0.035, 4.0);
/// assert!((df - 1.0 / 1.035_f64.powi(4)).abs() < 1e-12);
/// assert!(df < 1.0);
///
/// // At 10% the same year-4 cost is worth even less today.
/// assert!(discount_factor(0.10, 4.0) < df);
/// ```
pub fn discount_factor(annual_discount_rate: f64, years: f64) -> f64 {
    // Compound discounting: each year divides by (1 + rate) once more.
    1.0 / (1.0 + annual_discount_rate).powf(years)
}

/// Probability-weighted, discounted offset value.
///
/// P(future event) × counterfactual cost × discount factor — the honest form
/// of "this migration avoids the future rewrite". Never assume a silent 100%
/// probability: products get killed, priorities change.
///
/// # Arguments
///
/// * `probability_of_future_event` — probability the downstream event would
///   occur without the intervention (0.0–1.0).
/// * `counterfactual_cost` — what the event would cost, at marginal cost
///   (currency).
/// * `discount_factor` — present-value weight (see [`discount_factor`]).
///
/// # Returns
///
/// The offset value in present-value terms (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::avoided_downstream_costs::{
///     discount_factor, probability_weighted_offset,
/// };
///
/// // A £500,000 rewrite, 60% likely, avoided 4 years out at 3.5%.
/// let df = discount_factor(0.035, 4.0);
/// let offset = probability_weighted_offset(0.6, 500_000.0, df);
/// assert!((offset - 0.6 * 500_000.0 * df).abs() < 1e-9);
///
/// // Silent 100% probability would overstate the claim.
/// assert!(offset < probability_weighted_offset(1.0, 500_000.0, df));
/// ```
pub fn probability_weighted_offset(
    probability_of_future_event: f64,
    counterfactual_cost: f64,
    discount_factor: f64,
) -> f64 {
    probability_of_future_event * counterfactual_cost * discount_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a wound-monitoring app for 5,000 post-surgical patients
    // claims to avoid infection-related readmissions.

    #[test]
    fn attributable_events_avoided_are_45_per_year() {
        // Baseline readmission 4.0%; with app (RCT) 3.1%: 5,000 × 0.009 = 45.
        let got = attributable_events_avoided(5_000.0, 0.040, 0.031);
        assert!((got - 45.0).abs() < 1e-9);
    }

    #[test]
    fn offset_is_144_000_per_year() {
        // 45 × £3,200 marginal cost per readmission spell = £144,000/year.
        let got = offset_value(45.0, 3_200.0);
        assert!((got - 144_000.0).abs() < 1e-9);
    }

    #[test]
    fn app_cost_is_100_000_per_year() {
        // 5,000 × £20 = £100,000/year.
        let got = intervention_cost(5_000.0, 20.0);
        assert!((got - 100_000.0).abs() < 1e-9);
    }

    #[test]
    fn net_cost_is_minus_44_000_genuinely_cost_saving() {
        // £100,000 − £144,000 = −£44,000.
        let events = attributable_events_avoided(5_000.0, 0.040, 0.031);
        let offset = offset_value(events, 3_200.0);
        let cost = intervention_cost(5_000.0, 20.0);
        let got = net_cost(cost, &[offset]);
        assert!((got - (-44_000.0)).abs() < 1e-9);
        assert!(got < 0.0);
    }

    #[test]
    fn average_cost_offset_overstates_the_case() {
        // The same claim built on the £5,800 average readmission cost fails
        // the marginal-costing test and inflates the offset.
        let marginal = offset_value(45.0, 3_200.0);
        let average = offset_value(45.0, 5_800.0);
        assert!(average > marginal);
        assert!((average - 261_000.0).abs() < 1e-9);
    }

    #[test]
    fn discount_factor_shrinks_year_4_rewrite_value() {
        // SE connection: a rewrite avoided in year 4 at 3.5–10% discount is
        // worth much less than face value.
        let at_3_5 = discount_factor(0.035, 4.0);
        let at_10 = discount_factor(0.10, 4.0);
        assert!((at_3_5 - 1.0 / 1.035_f64.powi(4)).abs() < 1e-12);
        assert!(at_3_5 < 1.0 && at_10 < at_3_5);
    }

    #[test]
    fn probability_weighted_offset_multiplies_all_three_terms() {
        // SE connection formula: P = 0.6, counterfactual £500,000, 4 years
        // at 3.5% — and a silent 100% probability would overstate it.
        let df = discount_factor(0.035, 4.0);
        let got = probability_weighted_offset(0.6, 500_000.0, df);
        assert!((got - 0.6 * 500_000.0 * df).abs() < 1e-9);
        // Silent 100% probability would overstate it.
        assert!(got < probability_weighted_offset(1.0, 500_000.0, df));
    }
}
