//! # Cost-Effectiveness Analysis (CEA)
//!
//! CEA compares the costs of alternative interventions against a single
//! outcome measured in **natural units** — life-years, cases detected,
//! admissions avoided, mmHg of blood pressure reduced. Its output is a cost
//! per unit of outcome.
//!
//! The comparison statistic is the incremental cost-effectiveness ratio
//! (ICER). Procedure: define the outcome unit; cost every option from the
//! same perspective over the same time horizon; eliminate dominated options
//! (efficiency frontier); compute incremental ratios along the frontier.
//!
//! ## Formula
//!
//! ```text
//! ICER = (Cost_A − Cost_B) / (Effect_A − Effect_B)
//!      = £ per additional case detected / admission avoided / etc.
//!
//! Cost_A, Cost_B     — total costs of option A and comparator B, same
//!                      perspective and horizon
//! Effect_A, Effect_B — outcomes in the single declared natural unit
//! ```
//!
//! ## Why it matters
//!
//! CEA is the workhorse comparison when all options target the same
//! outcome. It answers "which of these ways of achieving X is the best use
//! of money?" — but *not* "is X worth achieving at all?" (that needs
//! cost-benefit analysis) and *not* "how does X compare with unrelated
//! priorities?" (that needs cost-utility analysis and a generic outcome
//! like the QALY). The discipline it enforces — one declared outcome unit,
//! incremental (not average) ratios, dominated options eliminated first —
//! kills most bad comparisons before the pricing discussion starts.
//!
//! ## Example
//!
//! Three ways to find undiagnosed atrial fibrillation in a population of
//! 100,000: pulse checks £150k/300 cases, pharmacy screening £400k/520,
//! wearables £900k/610. Pharmacy vs pulse costs £1,136 per additional case;
//! wearable vs pharmacy £5,556 — while the wearable option's *average*
//! £1,475 per case flatters.
//!
//! ```rust
//! use health_economics::cost_effectiveness_analysis::{
//!     InterventionOption, average_cost_effectiveness_ratio, icer, incremental_icers,
//! };
//!
//! // ICER pharmacy vs pulse: (400k−150k)/(520−300) = £1,136 per additional case.
//! let pharmacy_vs_pulse = icer(400_000.0, 520.0, 150_000.0, 300.0).unwrap();
//! assert!((pharmacy_vs_pulse - 1_136.0).abs() < 0.5);
//!
//! // ICER wearable vs pharmacy: (900k−400k)/(610−520) = £5,556 per additional case.
//! let wearable_vs_pharmacy = icer(900_000.0, 610.0, 400_000.0, 520.0).unwrap();
//! assert!((wearable_vs_pharmacy - 5_556.0).abs() < 0.5);
//!
//! // The wearable's average cost per case (900k/610 = £1,475) looks fine;
//! // the incremental £5,556 is the honest number for the expansion decision.
//! let average = average_cost_effectiveness_ratio(900_000.0, 610.0).unwrap();
//! assert!((average - 1_475.0).abs() < 0.5);
//! assert!(wearable_vs_pharmacy > average);
//!
//! // Same ratios computed along the sorted frontier.
//! let options = vec![
//!     InterventionOption { name: "Opportunistic pulse checks".into(), cost: 150_000.0, effect: 300.0 },
//!     InterventionOption { name: "Pharmacy screening events".into(), cost: 400_000.0, effect: 520.0 },
//!     InterventionOption { name: "Wearable-based screening".into(), cost: 900_000.0, effect: 610.0 },
//! ];
//! let icers = incremental_icers(&options);
//! assert_eq!(icers.len(), 2);
//! assert!((icers[0].unwrap() - 250_000.0 / 220.0).abs() < 1e-9);
//! assert!((icers[1].unwrap() - 500_000.0 / 90.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - CEA is the right template whenever options share one outcome: cost per
//!   flaky test eliminated across three remediation approaches; cost per
//!   incident avoided across observability vendors; cost per successful
//!   deployment across CI architectures.
//! - Declare one outcome unit before comparing.
//! - Use incremental, not average, ratios for expansion decisions.
//! - Eliminate dominated options first — this kills most bad vendor
//!   comparisons before the pricing discussion starts.
//!
//! ## Pitfalls
//!
//! - **Comparing options with different outcomes** ("cases found" vs
//!   "satisfaction") in one CEA — that needs cost-consequence analysis or a
//!   generic outcome.
//! - **Average cost-effectiveness ratios** presented where incremental ones
//!   are needed (the wearable example above).
//! - **Outcome units chosen for flattery**: "alerts generated" is an
//!   output, not an outcome; insist on units that carry value.
//!
//! ## Sources
//!
//! - CDC POLARIS: cost-effectiveness analysis.
//!   <https://www.cdc.gov/policy/polaris/economics/cost-effectiveness/index.html>
//! - York Health Economics Consortium glossary.
//!   <https://yhec.co.uk/glossary/cost-effectiveness-analysis/>
//!
//! Topic doc: health-economics-metrics/topics/cost-effectiveness-analysis.md

/// One intervention option: its total cost and its effect in the single
/// declared natural outcome unit.
///
/// All options in a CEA must be costed from the same perspective over the
/// same horizon, and must share one declared outcome unit.
#[derive(Debug, Clone)]
pub struct InterventionOption {
    /// Option name (e.g. "Pharmacy screening events").
    pub name: String,
    /// Total cost from the declared perspective over the declared horizon.
    pub cost: f64,
    /// Outcome in natural units (e.g. cases found).
    pub effect: f64,
}

/// ICER = (cost_a − cost_b) / (effect_a − effect_b): £ per additional unit
/// of outcome for option A over comparator B.
///
/// The honest number for an expansion decision; the sign convention assumes
/// A is the more effective option so a positive ICER reads "£ per extra
/// unit".
///
/// # Arguments
///
/// * `cost_a` — total cost of option A.
/// * `effect_a` — effect of option A, in the declared natural unit.
/// * `cost_b` — total cost of comparator B, same perspective/horizon.
/// * `effect_b` — effect of comparator B, same unit.
///
/// # Returns
///
/// `Some(icer)`, or `None` when the effects are equal (the ratio is
/// undefined; the options differ only in cost — pick the cheaper one).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_effectiveness_analysis::icer;
///
/// // Pharmacy (£400k, 520 cases) vs pulse checks (£150k, 300 cases):
/// // (400k−150k)/(520−300) = £1,136 per additional case found.
/// let got = icer(400_000.0, 520.0, 150_000.0, 300.0).unwrap();
/// assert!((got - 1_136.0).abs() < 0.5);
///
/// // Equal effects → undefined ratio.
/// assert!(icer(400_000.0, 300.0, 150_000.0, 300.0).is_none());
/// ```
pub fn icer(cost_a: f64, effect_a: f64, cost_b: f64, effect_b: f64) -> Option<f64> {
    let delta_effect = effect_a - effect_b;
    if delta_effect == 0.0 {
        None
    } else {
        // ΔCost / ΔEffect — both deltas taken A minus B.
        Some((cost_a - cost_b) / delta_effect)
    }
}

/// Average cost-effectiveness ratio = cost / effect.
///
/// Flattering but not the honest number for an expansion decision — the
/// wearable option's average £1,475/case hides its incremental £5,556/case.
/// Use the ICER for expansion decisions.
///
/// # Arguments
///
/// * `cost` — total cost of the option.
/// * `effect` — its effect in the declared natural unit.
///
/// # Returns
///
/// `Some(cost / effect)`, or `None` when the effect is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_effectiveness_analysis::average_cost_effectiveness_ratio;
///
/// // Wearable screening: 900k/610 = £1,475 per case — the flattering view.
/// let average = average_cost_effectiveness_ratio(900_000.0, 610.0).unwrap();
/// assert!((average - 1_475.0).abs() < 0.5);
/// assert!(average_cost_effectiveness_ratio(100.0, 0.0).is_none());
/// ```
pub fn average_cost_effectiveness_ratio(cost: f64, effect: f64) -> Option<f64> {
    if effect == 0.0 { None } else { Some(cost / effect) }
}

/// Incremental ICERs along a list of options already sorted by increasing
/// effect (dominated options eliminated beforehand).
///
/// # Arguments
///
/// * `options_sorted_by_effect` — the efficiency frontier, sorted by
///   increasing effect, dominated options removed.
///
/// # Returns
///
/// One entry per adjacent pair: ICER of option i+1 vs option i (`None` in a
/// slot when the pair's effects are equal). Empty for fewer than two
/// options.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_effectiveness_analysis::{
///     InterventionOption, incremental_icers,
/// };
///
/// let options = vec![
///     InterventionOption { name: "Pulse checks".into(), cost: 150_000.0, effect: 300.0 },
///     InterventionOption { name: "Pharmacy screening".into(), cost: 400_000.0, effect: 520.0 },
///     InterventionOption { name: "Wearable screening".into(), cost: 900_000.0, effect: 610.0 },
/// ];
/// let icers = incremental_icers(&options);
/// // £1,136 per additional case, then £5,556 per additional case.
/// assert!((icers[0].unwrap() - 1_136.0).abs() < 0.5);
/// assert!((icers[1].unwrap() - 5_556.0).abs() < 0.5);
/// ```
pub fn incremental_icers(options_sorted_by_effect: &[InterventionOption]) -> Vec<Option<f64>> {
    options_sorted_by_effect
        .windows(2)
        // Each adjacent pair: ICER of the more effective (pair[1]) vs the
        // previous step on the frontier (pair[0]).
        .map(|pair| icer(pair[1].cost, pair[1].effect, pair[0].cost, pair[0].effect))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: three ways to find undiagnosed atrial fibrillation in
    // a population of 100,000.

    fn options() -> Vec<InterventionOption> {
        vec![
            InterventionOption {
                name: "Opportunistic pulse checks".to_string(),
                cost: 150_000.0,
                effect: 300.0,
            },
            InterventionOption {
                name: "Pharmacy screening events".to_string(),
                cost: 400_000.0,
                effect: 520.0,
            },
            InterventionOption {
                name: "Wearable-based screening".to_string(),
                cost: 900_000.0,
                effect: 610.0,
            },
        ]
    }

    // Worked-example line: "ICER pharmacy vs pulse: (400k−150k)/(520−300) =
    // £1,136 per additional case".
    #[test]
    fn icer_pharmacy_vs_pulse_is_about_1_136_per_case() {
        let got = icer(400_000.0, 520.0, 150_000.0, 300.0).unwrap();
        assert!((got - 250_000.0 / 220.0).abs() < 1e-9);
        assert!((got - 1_136.0).abs() < 0.5);
    }

    // Worked-example line: "ICER wearable vs pharmacy: (900k−400k)/(610−520)
    // = £5,556 per additional case".
    #[test]
    fn icer_wearable_vs_pharmacy_is_about_5_556_per_case() {
        let got = icer(900_000.0, 610.0, 400_000.0, 520.0).unwrap();
        assert!((got - 500_000.0 / 90.0).abs() < 1e-9);
        assert!((got - 5_556.0).abs() < 0.5);
    }

    // Worked-example line: "the wearable option's average cost per case
    // (900k/610 = £1,475) looks fine; the incremental £5,556 is the honest
    // number for the expansion decision".
    #[test]
    fn wearable_average_ratio_is_about_1_475_and_flatters() {
        let average = average_cost_effectiveness_ratio(900_000.0, 610.0).unwrap();
        assert!((average - 1_475.0).abs() < 0.5);
        // The incremental £5,556 is the honest number for expansion.
        let incremental = icer(900_000.0, 610.0, 400_000.0, 520.0).unwrap();
        assert!(incremental > average);
    }

    // Both worked-example ICERs (£1,136 and £5,556) reproduced along the
    // sorted frontier.
    #[test]
    fn incremental_icers_along_the_options() {
        let icers = incremental_icers(&options());
        assert_eq!(icers.len(), 2);
        assert!((icers[0].unwrap() - 250_000.0 / 220.0).abs() < 1e-9);
        assert!((icers[1].unwrap() - 500_000.0 / 90.0).abs() < 1e-9);
    }

    // Edge case for "ICER = ΔC/ΔE": equal effects make the ratio undefined,
    // and a zero effect makes the average ratio undefined.
    #[test]
    fn equal_effects_make_icer_undefined() {
        assert!(icer(400_000.0, 300.0, 150_000.0, 300.0).is_none());
        assert!(average_cost_effectiveness_ratio(100.0, 0.0).is_none());
    }
}
