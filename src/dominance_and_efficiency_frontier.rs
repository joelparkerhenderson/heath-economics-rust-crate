//! # Dominance and the Efficiency Frontier
//!
//! An option is **dominated** if another option costs less *and* delivers
//! more. The **efficiency frontier** is what remains after eliminating
//! dominated options: the set of choices where getting more requires paying
//! more.
//!
//! Incremental comparisons (ICERs) are then computed only *along the
//! frontier*, each option against the next-cheapest non-dominated one —
//! never against "do nothing" when better intermediate options exist.
//!
//! ## Formula
//!
//! ```text
//! Strict dominance:   A dominates B if Cost_A ≤ Cost_B and Effect_A ≥ Effect_B
//!                     (with at least one strict inequality)
//!
//! Extended dominance: B is ruled out if a mix of A and C achieves more effect
//!                     per pound — detected when ICERs decrease as you move up
//!                     the frontier. Valid frontier ICERs must be increasing.
//!
//! ICER(next vs prev) = ΔCost / ΔEffect
//!
//! Cost   — cost per period (e.g. £/year)
//! Effect — outcome in natural units (e.g. appointments recovered/year)
//! ```
//!
//! Procedure: sort options by effect; remove strictly dominated ones; compute
//! pairwise ICERs between neighbors; remove any option whose ICER exceeds
//! that of the next more-effective option (extended dominance); repeat until
//! ICERs increase monotonically.
//!
//! ## Why it matters
//!
//! Before any debate about thresholds or budgets, health technology
//! assessment first eliminates options nobody should ever pick. Plotting
//! every option on a cost-vs-effect plane and drawing the frontier is a
//! five-minute exercise that routinely kills half a shortlist. Comparing
//! everything to baseline instead of to the next frontier option flatters
//! expensive options by hiding cheaper near-equivalents.
//!
//! ## Example
//!
//! The topic doc's worked example: four options for reducing missed
//! appointments. Phone calls (£120,000 for 2,200 recovered) are strictly
//! dominated by SMS + AI triage (£90,000 for 3,500). The frontier is
//! nothing → SMS (£10 per appointment recovered) → SMS + AI (£46.67 per
//! appointment); at ~£160 saved per recovered hospital appointment, both
//! frontier steps are worth taking.
//!
//! ```rust
//! use health_economics::dominance_and_efficiency_frontier::{
//!     Alternative, strictly_dominates, efficiency_frontier, frontier_icers,
//! };
//!
//! let options = vec![
//!     Alternative::new("Do nothing", 0.0, 0.0),
//!     Alternative::new("SMS reminders", 20_000.0, 2_000.0),
//!     Alternative::new("Phone calls", 120_000.0, 2_200.0),
//!     Alternative::new("SMS + AI triage", 90_000.0, 3_500.0),
//! ];
//!
//! // Phone calls are strictly dominated by SMS + AI triage
//! // (costs more, recovers fewer).
//! assert!(strictly_dominates(&options[3], &options[2]));
//!
//! // Frontier: nothing → SMS → SMS + AI.
//! let frontier = efficiency_frontier(&options);
//! let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
//! assert_eq!(names, vec!["Do nothing", "SMS reminders", "SMS + AI triage"]);
//!
//! // ICER(SMS vs nothing) = 20,000 / 2,000 = £10 per appointment recovered;
//! // ICER(SMS+AI vs SMS)  = 70,000 / 1,500 = £46.67 per appointment.
//! let icers: Vec<f64> = frontier_icers(&frontier).into_iter().flatten().collect();
//! assert!((icers[0] - 10.0).abs() < 1e-9);
//! assert!((icers[1] - 46.67).abs() < 0.01);
//! // Increasing ICERs → valid frontier.
//! assert!(icers[0] < icers[1]);
//! ```
//!
//! ## Software engineering connection
//!
//! - Build the same chart for any tooling decision: cost per year on one
//!   axis, measured outcome (hours saved, incidents avoided, deploys enabled)
//!   on the other.
//! - Points up-and-left of the frontier are eliminated before anyone argues
//!   about budget.
//! - This reframes vendor selection from feature-checklist debates to "you
//!   are dominated; the meeting is over."
//! - It also exposes the common enterprise pattern of buying the most
//!   expensive option for a marginal gain — legitimate only if the
//!   incremental price per incremental unit is one the org would knowingly
//!   pay.
//!
//! ## Pitfalls
//!
//! - **Comparing everything to baseline** instead of to the next option on
//!   the frontier — this flatters expensive options by hiding cheaper
//!   near-equivalents.
//! - **Single-dimension effect scores** that hide what matters; if two
//!   outcomes count, either combine them defensibly or show two frontiers.
//! - **Forgetting uncertainty**: options near the frontier may swap places
//!   under sensitivity analysis.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: dominance.
//!   <https://yhec.co.uk/glossary/dominance/>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/dominance-and-efficiency-frontier.md

/// An option on the cost-vs-effect plane.
#[derive(Debug, Clone, PartialEq)]
pub struct Alternative {
    /// Human-readable label for the option.
    pub name: String,
    /// Cost per period (e.g. £/year).
    pub cost: f64,
    /// Effect in natural units (e.g. appointments recovered/year).
    pub effect: f64,
}

impl Alternative {
    /// Convenience constructor from a name, cost, and effect.
    ///
    /// # Arguments
    ///
    /// * `name` — human-readable label.
    /// * `cost` — cost per period (e.g. £/year).
    /// * `effect` — effect in natural units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::dominance_and_efficiency_frontier::Alternative;
    ///
    /// let sms = Alternative::new("SMS reminders", 20_000.0, 2_000.0);
    /// assert_eq!(sms.cost, 20_000.0);
    /// assert_eq!(sms.effect, 2_000.0);
    /// ```
    pub fn new(name: &str, cost: f64, effect: f64) -> Self {
        Alternative { name: name.to_string(), cost, effect }
    }
}

/// Strict dominance test.
///
/// `a` dominates `b` if Cost_a ≤ Cost_b and Effect_a ≥ Effect_b, with at
/// least one strict inequality (otherwise the options are identical, and
/// neither dominates).
///
/// # Arguments
///
/// * `a` — the candidate dominator.
/// * `b` — the option possibly dominated.
///
/// # Returns
///
/// `true` iff `a` strictly dominates `b`.
///
/// # Examples
///
/// ```rust
/// use health_economics::dominance_and_efficiency_frontier::{
///     Alternative, strictly_dominates,
/// };
///
/// // SMS + AI triage (£90k, 3,500) dominates phone calls (£120k, 2,200):
/// // it costs less and recovers more.
/// let sms_ai = Alternative::new("SMS + AI triage", 90_000.0, 3_500.0);
/// let phone = Alternative::new("Phone calls", 120_000.0, 2_200.0);
/// assert!(strictly_dominates(&sms_ai, &phone));
/// assert!(!strictly_dominates(&phone, &sms_ai));
/// ```
pub fn strictly_dominates(a: &Alternative, b: &Alternative) -> bool {
    // Weak inequalities on both axes, plus at least one strict inequality —
    // ties on both axes are not dominance.
    a.cost <= b.cost && a.effect >= b.effect && (a.cost < b.cost || a.effect > b.effect)
}

/// Incremental cost-effectiveness ratio of `next` versus `prev`:
/// ΔCost / ΔEffect.
///
/// On a valid frontier `next` is the next more-effective option after
/// `prev`, so the ratio reads "£ per extra unit of effect for stepping up".
///
/// # Arguments
///
/// * `next` — the more-effective option.
/// * `prev` — the comparator (previous frontier point).
///
/// # Returns
///
/// `Some(ΔCost / ΔEffect)`, or `None` when the two effects are equal (the
/// ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::dominance_and_efficiency_frontier::{
///     Alternative, icer,
/// };
///
/// // ICER(SMS vs nothing) = 20,000 / 2,000 = £10 per appointment recovered.
/// let nothing = Alternative::new("Do nothing", 0.0, 0.0);
/// let sms = Alternative::new("SMS reminders", 20_000.0, 2_000.0);
/// assert_eq!(icer(&sms, &nothing), Some(10.0));
/// ```
pub fn icer(next: &Alternative, prev: &Alternative) -> Option<f64> {
    let delta_effect = next.effect - prev.effect;
    if delta_effect == 0.0 {
        None
    } else {
        Some((next.cost - prev.cost) / delta_effect)
    }
}

/// Build the efficiency frontier from a set of options.
///
/// Implements the standard HTA procedure: sort by effect, remove strictly
/// dominated options, then repeatedly remove extended-dominated options
/// (those whose ICER over the previous frontier point exceeds the ICER of
/// the next point over them) until frontier ICERs increase monotonically.
///
/// # Arguments
///
/// * `options` — the candidate options (any order; the input is not
///   modified).
///
/// # Returns
///
/// The frontier, ordered by increasing effect (and cost). Options not on
/// the frontier should never be chosen at any willingness-to-pay.
///
/// # Examples
///
/// ```rust
/// use health_economics::dominance_and_efficiency_frontier::{
///     Alternative, efficiency_frontier,
/// };
///
/// let options = vec![
///     Alternative::new("Do nothing", 0.0, 0.0),
///     Alternative::new("SMS reminders", 20_000.0, 2_000.0),
///     Alternative::new("Phone calls", 120_000.0, 2_200.0), // strictly dominated
///     Alternative::new("SMS + AI triage", 90_000.0, 3_500.0),
/// ];
/// let frontier = efficiency_frontier(&options);
/// let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
/// assert_eq!(names, vec!["Do nothing", "SMS reminders", "SMS + AI triage"]);
/// ```
pub fn efficiency_frontier(options: &[Alternative]) -> Vec<Alternative> {
    // Step 1: sort by effect ascending; break effect ties by cost ascending
    // so the cheaper of two equal-effect options comes first (the dearer one
    // will then be removed as strictly dominated).
    let mut sorted: Vec<Alternative> = options.to_vec();
    sorted.sort_by(|a, b| {
        a.effect
            .partial_cmp(&b.effect)
            .unwrap()
            .then(a.cost.partial_cmp(&b.cost).unwrap())
    });

    // Step 2: remove strictly dominated options — keep a candidate only if
    // no other option costs no more AND delivers no less (with one strict
    // inequality). This is an O(n²) pairwise sweep over the sorted list.
    let mut frontier: Vec<Alternative> = Vec::new();
    for (i, candidate) in sorted.iter().enumerate() {
        let dominated = sorted
            .iter()
            .enumerate()
            .any(|(j, other)| j != i && strictly_dominates(other, candidate));
        if !dominated {
            frontier.push(candidate.clone());
        }
    }

    // Step 3: remove extended-dominated options. For each interior point i,
    // compare the ICER of stepping up TO i (low) with the ICER of stepping
    // up FROM i to i+1 (high). If high < low, ICERs are decreasing at i: a
    // mix of the neighbors i−1 and i+1 buys effect cheaper than i does, so i
    // leaves the frontier. Removing a point changes its neighbors' pairwise
    // ICERs, so restart the scan after every removal and loop until ICERs
    // increase monotonically (no removal in a full pass).
    loop {
        let mut removed = false;
        if frontier.len() >= 3 {
            for i in 1..frontier.len() - 1 {
                // ICER of frontier[i] over its cheaper neighbor...
                let low = icer(&frontier[i], &frontier[i - 1]);
                // ...and of the next point over frontier[i].
                let high = icer(&frontier[i + 1], &frontier[i]);
                if let (Some(low), Some(high)) = (low, high)
                    && high < low {
                        // Decreasing ICERs: frontier[i] is extended-dominated.
                        frontier.remove(i);
                        removed = true;
                        break;
                    }
            }
        }
        if !removed {
            break;
        }
    }
    frontier
}

/// Pairwise ICERs between consecutive frontier options.
///
/// Element `i` is the ICER of option `i + 1` versus option `i`. On a valid
/// frontier the sequence is strictly increasing.
///
/// # Arguments
///
/// * `frontier` — options ordered by increasing effect (as returned by
///   [`efficiency_frontier`]).
///
/// # Returns
///
/// A vector of length `frontier.len() − 1` (empty for fewer than two
/// options); `None` entries mark equal effects, which a proper frontier will
/// not contain.
///
/// # Examples
///
/// ```rust
/// use health_economics::dominance_and_efficiency_frontier::{
///     Alternative, efficiency_frontier, frontier_icers,
/// };
///
/// let options = vec![
///     Alternative::new("Do nothing", 0.0, 0.0),
///     Alternative::new("SMS reminders", 20_000.0, 2_000.0),
///     Alternative::new("SMS + AI triage", 90_000.0, 3_500.0),
/// ];
/// let frontier = efficiency_frontier(&options);
/// let icers: Vec<f64> = frontier_icers(&frontier).into_iter().flatten().collect();
/// // £10 then £46.67 per appointment recovered — increasing, hence valid.
/// assert!((icers[0] - 10.0).abs() < 1e-9);
/// assert!((icers[1] - 46.67).abs() < 0.01);
/// ```
pub fn frontier_icers(frontier: &[Alternative]) -> Vec<Option<f64>> {
    frontier.windows(2).map(|w| icer(&w[1], &w[0])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worked_example() -> Vec<Alternative> {
        vec![
            Alternative::new("Do nothing", 0.0, 0.0),
            Alternative::new("SMS reminders", 20_000.0, 2_000.0),
            Alternative::new("Phone calls", 120_000.0, 2_200.0),
            Alternative::new("SMS + AI triage", 90_000.0, 3_500.0),
        ]
    }

    // Worked example: "Phone calls are strictly dominated by SMS + AI triage
    // (costs more, recovers fewer)."
    #[test]
    fn phone_calls_are_strictly_dominated_by_sms_plus_ai() {
        let sms_ai = Alternative::new("SMS + AI triage", 90_000.0, 3_500.0);
        let phone = Alternative::new("Phone calls", 120_000.0, 2_200.0);
        assert!(strictly_dominates(&sms_ai, &phone));
        assert!(!strictly_dominates(&phone, &sms_ai));
    }

    // Worked example: "Frontier: nothing → SMS → SMS + AI."
    #[test]
    fn frontier_is_nothing_then_sms_then_sms_plus_ai() {
        let frontier = efficiency_frontier(&worked_example());
        let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, vec!["Do nothing", "SMS reminders", "SMS + AI triage"]);
    }

    // Worked example: "ICER(SMS vs nothing) = 20,000 / 2,000 = £10 per
    // appointment recovered".
    #[test]
    fn icer_of_sms_vs_nothing_is_10_pounds_per_appointment() {
        let frontier = efficiency_frontier(&worked_example());
        let icer_sms = icer(&frontier[1], &frontier[0]).unwrap();
        assert!((icer_sms - 10.0).abs() < 1e-9);
    }

    // Worked example: "ICER(SMS+AI vs SMS) = (90,000 − 20,000) / (3,500 −
    // 2,000) = £46.67 per appointment".
    #[test]
    fn icer_of_sms_plus_ai_vs_sms_is_46_67_pounds_per_appointment() {
        let frontier = efficiency_frontier(&worked_example());
        let icer_ai = icer(&frontier[2], &frontier[1]).unwrap();
        // (90,000 − 20,000) / (3,500 − 2,000) = £46.67
        assert!((icer_ai - 46.67).abs() < 0.01);
    }

    // Worked example: "Increasing ICERs → valid frontier."
    #[test]
    fn frontier_icers_are_increasing_hence_valid() {
        let frontier = efficiency_frontier(&worked_example());
        let icers: Vec<f64> = frontier_icers(&frontier).into_iter().flatten().collect();
        assert_eq!(icers.len(), 2);
        assert!(icers[0] < icers[1]);
    }

    // Worked example: "At ~£160 saved per recovered hospital appointment ...
    // both frontier steps are worth taking."
    #[test]
    fn both_frontier_steps_are_worth_taking_at_160_per_recovered_appointment() {
        let frontier = efficiency_frontier(&worked_example());
        for pair in frontier_icers(&frontier) {
            assert!(pair.unwrap() < 160.0);
        }
    }

    // The math section: "Extended dominance: B is ruled out if a mix of A and
    // C achieves more effect per pound — detected when ICERs decrease."
    #[test]
    fn extended_dominance_removes_middle_option_with_decreasing_icers() {
        // B's ICER over A (100/1 = 100) exceeds C's ICER over B (20/1 = 20):
        // a mix of A and C beats B, so B leaves the frontier.
        let options = vec![
            Alternative::new("A", 0.0, 0.0),
            Alternative::new("B", 100.0, 1.0),
            Alternative::new("C", 120.0, 2.0),
        ];
        let frontier = efficiency_frontier(&options);
        let names: Vec<&str> = frontier.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, vec!["A", "C"]);
    }
}
