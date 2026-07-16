//! # Expected Value of Perfect Information (EVPI)
//!
//! EVPI is the maximum amount a decision-maker should pay to eliminate
//! uncertainty before deciding — the formal price of "let's run a study
//! first". It is the gap between deciding with perfect foresight (in each
//! possible world you pick that world's best option) and deciding now
//! (you must commit to the single option that is best on average).
//!
//! The order of `max` and expectation is the whole point: perfect
//! information lets you take `E[max]`, committing now only gets you
//! `max E[]` — and `E[max] ≥ max E[]` always, so EVPI ≥ 0.
//!
//! ## Formula
//!
//! ```text
//! EVPI = E_θ[ max_j NMB(j, θ) ]  −  max_j E_θ[ NMB(j, θ) ]
//!
//! θ        = uncertain parameters (with their joint distribution)
//! j        = decision option index (e.g. roll out / don't)
//! NMB(j,θ) = net monetary benefit of option j given θ
//! E_θ[·]   = expectation over the distribution of θ
//!
//! Population EVPI = per-decision EVPI × decisions affected
//! ```
//!
//! First term: average of the best-choice payoff across each possible world
//! (you always pick right). Second term: payoff of the single option that is
//! best on average (you must commit now). Computed directly from PSA draws.
//!
//! ## Why it matters
//!
//! Health systems constantly face the choice: adopt now on imperfect
//! evidence, or fund more research first. EVPI puts a number on the second
//! option. If EVPI is £50,000 and the proposed trial costs £2 million, adopt
//! now. If EVPI is £20 million, the trial is a bargain. The same question —
//! "should we pilot this before rolling it out?" — arises for every
//! enterprise tool decision, and almost nobody prices it.
//!
//! ## Example
//!
//! Roll out an AI documentation assistant to 5,000 clinicians, or not.
//! World A (p = 0.6): rollout NMB +£8M. World B (p = 0.4): rollout NMB −£3M.
//! "Don't roll out" is £0 in both worlds.
//!
//! ```
//! use health_economics::expected_value_of_perfect_information::{
//!     Scenario, expected_nmb_of_best_option, expected_nmb_with_perfect_information, evpi,
//! };
//!
//! let scenarios = vec![
//!     Scenario { probability: 0.6, option_nmbs: vec![8.0, 0.0] },  // world A, £M
//!     Scenario { probability: 0.4, option_nmbs: vec![-3.0, 0.0] }, // world B, £M
//! ];
//!
//! // Decide now: E[NMB rollout] = 0.6 × 8 − 0.4 × 3 = +£3.6M → roll out.
//! assert!((expected_nmb_of_best_option(&scenarios).unwrap() - 3.6).abs() < 1e-9);
//!
//! // Perfect information: 0.6 × 8 + 0.4 × 0 = £4.8M.
//! assert!((expected_nmb_with_perfect_information(&scenarios).unwrap() - 4.8).abs() < 1e-9);
//!
//! // EVPI = 4.8 − 3.6 = £1.2M: a £150k pilot is a bargain; a £2M pilot is not.
//! assert!((evpi(&scenarios).unwrap() - 1.2).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - EVPI is the economics of the spike, the pilot, the A/B test, and the
//!   proof-of-concept.
//! - **A pilot is only worth funding if the decision could actually
//!   change** — if you'd roll out regardless of the result, EVPI = 0 and the
//!   pilot is theater.
//! - **Cap pilot spend at EVPI** — the value of information is bounded by
//!   the value of the decision it informs.
//! - Partial EVPI (EVPPI) extends this to single parameters: "what is it
//!   worth to nail down the time-saved number specifically?" — which tells
//!   you what the pilot should measure.
//!
//! ## Pitfalls
//!
//! - **Running pilots with no decision rule attached** — information that
//!   can't change the choice is worthless by definition.
//! - **Ignoring the delay cost of gathering information**: a 6-month pilot
//!   delays 6 months of benefit; net value of the pilot = EVPI resolved −
//!   delay cost − pilot cost.
//! - **Treating EVPI as a forecast.** It is an upper bound on information
//!   value, not an estimate of what a specific study will deliver.
//!
//! ## Sources
//!
//! - Claxton K. "Exploring uncertainty in cost-effectiveness analysis."
//!   PharmacoEconomics 2008. <https://pubmed.ncbi.nlm.nih.gov/18279550/>
//! - York Health Economics Consortium glossary: EVPI.
//!   <https://yhec.co.uk/glossary/expected-value-of-perfect-information-evpi/>
//!
//! Topic doc: health-economics-metrics/topics/expected-value-of-perfect-information.md

/// One possible world: its probability and the net monetary benefit of each
/// decision option in that world.
///
/// Options must be indexed consistently across all scenarios (option 0 in
/// one scenario is the same real-world choice as option 0 in every other).
/// NMB units are whatever currency scale you choose (the worked example
/// uses £M) — just keep them consistent.
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Probability of this world; scenario probabilities should sum to 1.
    pub probability: f64,
    /// Net monetary benefit of each option, given this world (one entry per
    /// option, same order in every scenario).
    pub option_nmbs: Vec<f64>,
}

fn max_of(values: &[f64]) -> Option<f64> {
    // Fold-based max that works for f64 (no Ord); None on an empty slice.
    values.iter().copied().fold(None, |acc, v| {
        Some(match acc {
            None => v,
            Some(m) if v > m => v,
            Some(m) => m,
        })
    })
}

/// Expected NMB of committing now to the single best-on-average option:
/// `max_j E_θ[ NMB(j, θ) ]`.
///
/// This is the "decide now" arm: take expectations first (per option, across
/// worlds), then pick the maximum — you must commit to one option before the
/// uncertainty resolves.
///
/// # Arguments
///
/// * `scenarios` — the possible worlds with probabilities and per-option
///   NMBs (probabilities should sum to 1).
///
/// # Returns
///
/// The best expected NMB, or `None` if `scenarios` is empty or the first
/// scenario has no options. (Scenarios with fewer options than the first
/// will panic on index; keep option lists the same length.)
///
/// # Examples
///
/// ```
/// use health_economics::expected_value_of_perfect_information::{
///     Scenario, expected_nmb_of_best_option,
/// };
///
/// // Worked example: E[NMB rollout] = 0.6 × 8 − 0.4 × 3 = +£3.6M.
/// let scenarios = vec![
///     Scenario { probability: 0.6, option_nmbs: vec![8.0, 0.0] },
///     Scenario { probability: 0.4, option_nmbs: vec![-3.0, 0.0] },
/// ];
/// let e = expected_nmb_of_best_option(&scenarios).unwrap();
/// assert!((e - 3.6).abs() < 1e-9);
/// ```
pub fn expected_nmb_of_best_option(scenarios: &[Scenario]) -> Option<f64> {
    let n_options = scenarios.first()?.option_nmbs.len();
    if n_options == 0 {
        return None;
    }
    // E-then-max: expectation per option j across worlds ...
    let expected_per_option: Vec<f64> = (0..n_options)
        .map(|j| scenarios.iter().map(|s| s.probability * s.option_nmbs[j]).sum())
        .collect();
    // ... then max over options — the commit-now decision rule.
    max_of(&expected_per_option)
}

/// Expected NMB with perfect information: `E_θ[ max_j NMB(j, θ) ]`.
///
/// This is the "perfect foresight" arm: in each world you pick that world's
/// best option (max first), then average across worlds. Because max is taken
/// inside the expectation, this is always ≥ the commit-now value.
///
/// # Arguments
///
/// * `scenarios` — the possible worlds with probabilities and per-option
///   NMBs.
///
/// # Returns
///
/// The expected best-choice NMB, or `None` if `scenarios` is empty or any
/// scenario has an empty option list.
///
/// # Examples
///
/// ```
/// use health_economics::expected_value_of_perfect_information::{
///     Scenario, expected_nmb_with_perfect_information,
/// };
///
/// // Worked example: 0.6 × 8 + 0.4 × 0 = £4.8M.
/// let scenarios = vec![
///     Scenario { probability: 0.6, option_nmbs: vec![8.0, 0.0] },
///     Scenario { probability: 0.4, option_nmbs: vec![-3.0, 0.0] },
/// ];
/// let e = expected_nmb_with_perfect_information(&scenarios).unwrap();
/// assert!((e - 4.8).abs() < 1e-9);
/// ```
pub fn expected_nmb_with_perfect_information(scenarios: &[Scenario]) -> Option<f64> {
    if scenarios.is_empty() {
        return None;
    }
    // Max-then-E: best option inside each world, then probability-weighted sum.
    // (Sum over Option<f64> yields None if any world had no options.)
    scenarios
        .iter()
        .map(|s| max_of(&s.option_nmbs).map(|m| s.probability * m))
        .sum()
}

/// EVPI: the value of resolving all uncertainty before deciding.
///
/// `EVPI = E_θ[max_j NMB] − max_j E_θ[NMB]` — perfect-foresight value minus
/// commit-now value. Always ≥ 0; exactly 0 when the same option wins in
/// every world (i.e. information could not change the decision).
///
/// # Arguments
///
/// * `scenarios` — the possible worlds with probabilities and per-option
///   NMBs.
///
/// # Returns
///
/// EVPI in the same currency units as the NMBs, or `None` if `scenarios` is
/// empty or options are missing.
///
/// # Examples
///
/// ```
/// use health_economics::expected_value_of_perfect_information::{
///     Scenario, evpi,
/// };
///
/// // Worked example: EVPI = 4.8M − 3.6M = £1.2M.
/// let scenarios = vec![
///     Scenario { probability: 0.6, option_nmbs: vec![8.0, 0.0] },
///     Scenario { probability: 0.4, option_nmbs: vec![-3.0, 0.0] },
/// ];
/// let v = evpi(&scenarios).unwrap();
/// assert!((v - 1.2).abs() < 1e-9);
/// ```
pub fn evpi(scenarios: &[Scenario]) -> Option<f64> {
    // E[max] − max E[]: the gap is the price of deciding blind.
    Some(expected_nmb_with_perfect_information(scenarios)? - expected_nmb_of_best_option(scenarios)?)
}

/// EVPI from equally weighted PSA draws.
///
/// Each row of `draws` is one draw of the uncertain parameters θ, giving the
/// NMB of every option under that draw. All draws are weighted `1/n` — the
/// standard way EVPI is computed from a probabilistic sensitivity analysis.
///
/// # Arguments
///
/// * `draws` — one `Vec<f64>` of per-option NMBs per PSA draw (options in
///   the same order in every row).
///
/// # Returns
///
/// EVPI, or `None` if there are no draws or a draw has no options.
///
/// # Examples
///
/// ```
/// use health_economics::expected_value_of_perfect_information::evpi_from_psa_draws;
///
/// // 6 draws of world A and 4 of world B reproduce the p = 0.6/0.4 example:
/// // EVPI = £1.2M.
/// let mut draws = vec![vec![8.0, 0.0]; 6];
/// draws.extend(vec![vec![-3.0, 0.0]; 4]);
/// let v = evpi_from_psa_draws(&draws).unwrap();
/// assert!((v - 1.2).abs() < 1e-9);
/// ```
pub fn evpi_from_psa_draws(draws: &[Vec<f64>]) -> Option<f64> {
    if draws.is_empty() {
        return None;
    }
    // Each PSA draw is a possible world with equal weight 1/n.
    let p = 1.0 / draws.len() as f64;
    let scenarios: Vec<Scenario> = draws
        .iter()
        .map(|nmbs| Scenario { probability: p, option_nmbs: nmbs.clone() })
        .collect();
    evpi(&scenarios)
}

/// Population EVPI: per-decision EVPI scaled by the number of decisions the
/// information would affect.
///
/// # Arguments
///
/// * `evpi_per_decision` — EVPI for one decision instance (currency units).
/// * `decisions_affected` — number of decisions the information informs
///   (e.g. patients treated per year × years the evidence stays relevant).
///
/// # Returns
///
/// Population EVPI in the same currency units.
///
/// # Examples
///
/// ```
/// use health_economics::expected_value_of_perfect_information::population_evpi;
///
/// // £1.2M per decision across 10 comparable rollout decisions = £12M.
/// assert!((population_evpi(1.2, 10.0) - 12.0).abs() < 1e-9);
/// ```
pub fn population_evpi(evpi_per_decision: f64, decisions_affected: f64) -> f64 {
    evpi_per_decision * decisions_affected
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Worlds from the worked example: rollout vs don't, in £M.
    fn worked_example_scenarios() -> Vec<Scenario> {
        vec![
            // World A (p = 0.6): rollout NMB +£8M; don't roll out £0.
            Scenario { probability: 0.6, option_nmbs: vec![8.0, 0.0] },
            // World B (p = 0.4): rollout NMB −£3M; don't roll out £0.
            Scenario { probability: 0.4, option_nmbs: vec![-3.0, 0.0] },
        ]
    }

    // Doc line: "Decide now: E[NMB rollout] = 0.6 × 8 − 0.4 × 3 = +£3.6M → roll out".
    #[test]
    fn worked_example_decide_now_is_3_6m() {
        let e = expected_nmb_of_best_option(&worked_example_scenarios()).unwrap();
        assert!((e - 3.6).abs() < 1e-9, "got {e}");
    }

    // Doc line: "Expected value = 0.6 × 8 + 0.4 × 0 = £4.8M" with perfect information.
    #[test]
    fn worked_example_perfect_information_is_4_8m() {
        let e = expected_nmb_with_perfect_information(&worked_example_scenarios()).unwrap();
        assert!((e - 4.8).abs() < 1e-9, "got {e}");
    }

    // Doc line: "EVPI = 4.8M − 3.6M = £1.2M".
    #[test]
    fn worked_example_evpi_is_1_2m() {
        let v = evpi(&worked_example_scenarios()).unwrap();
        assert!((v - 1.2).abs() < 1e-9, "got {v}");
    }

    // Doc line: "a 3-month pilot costing £150,000 ... is emphatically worth it —
    // and any pilot costing more than £1.2M is not".
    #[test]
    fn worked_example_pilot_cost_bounded_by_evpi() {
        let v = evpi(&worked_example_scenarios()).unwrap() * 1_000_000.0;
        assert!(150_000.0 < v);
        assert!(1_300_000.0 > v);
    }

    // Doc rule: "If you'd roll out regardless of the pilot result, EVPI = 0".
    #[test]
    fn evpi_is_zero_when_decision_cannot_change() {
        let scenarios = vec![
            Scenario { probability: 0.5, option_nmbs: vec![5.0, 1.0] },
            Scenario { probability: 0.5, option_nmbs: vec![3.0, 1.0] },
        ];
        let v = evpi(&scenarios).unwrap();
        assert!((v - 0.0).abs() < 1e-9, "got {v}");
    }

    // Doc line: "Computed directly from PSA draws" — equally weighted draws
    // matching the 0.6/0.4 worlds give the same £1.2M.
    #[test]
    fn evpi_from_psa_draws_matches_discrete_worlds() {
        // 6 draws of world A, 4 of world B → same distribution as p = 0.6 / 0.4.
        let mut draws = vec![vec![8.0, 0.0]; 6];
        draws.extend(vec![vec![-3.0, 0.0]; 4]);
        let v = evpi_from_psa_draws(&draws).unwrap();
        assert!((v - 1.2).abs() < 1e-9, "got {v}");
    }

    // Doc line: "Population EVPI multiplies by the number of decisions affected".
    #[test]
    fn population_evpi_scales_linearly() {
        let v = population_evpi(1.2, 10.0);
        assert!((v - 12.0).abs() < 1e-9, "got {v}");
    }

    // Guard behavior: empty inputs yield None.
    #[test]
    fn empty_inputs_return_none() {
        assert!(evpi(&[]).is_none());
        assert!(evpi_from_psa_draws(&[]).is_none());
    }
}
