//! # Net Monetary Benefit (NMB)
//!
//! NMB converts a cost-effectiveness result into a single money value: health
//! gain priced at the willingness-to-pay threshold, minus cost. Its twin,
//! Net Health Benefit (NHB), expresses the same rule in health units.
//!
//! Unlike ICERs, NMB is linear: it ranks any number of options, averages
//! across Monte Carlo draws, and never explodes near zero effect. NMB > 0 ⇔
//! ICER < λ (when ΔE > 0), so the two rules agree — NMB is just better
//! behaved.
//!
//! ## Formula
//!
//! ```text
//! NMB = (ΔE × λ) − ΔC
//! NHB = ΔE − (ΔC / λ)
//!
//! ΔE = incremental effect (e.g., QALYs)
//! ΔC = incremental cost
//! λ  = willingness-to-pay threshold
//!
//! Decision rule: adopt if NMB > 0 (equivalently NHB > 0).
//! Among alternatives: choose the highest NMB.
//! ```
//!
//! ## Why it matters
//!
//! Ratios (ICERs) are awkward: they explode near zero effect, can't be
//! averaged across uncertainty draws, and can't rank three or more options
//! cleanly. NMB fixes all of that — it is linear, so you can rank options,
//! average Monte Carlo draws, and decompose contributions. It is also the
//! form of health-economic math every engineer already knows: *value minus
//! cost*. Report it at named thresholds (e.g. £20,000 and £30,000/QALY).
//!
//! ## Example
//!
//! Three options for a diabetes service, per 1,000 patients, at
//! λ = £20,000/QALY: extra clinics gain the most QALYs but destroy value at
//! this threshold; app + coaching wins the ranking.
//!
//! ```rust
//! use health_economics::net_monetary_benefit::{
//!     adopt, best_option_index, net_health_benefit, net_monetary_benefit,
//!     EvaluatedOption,
//! };
//!
//! // App + coaching: 20,000 × 30 − 400,000 = £200,000.
//! assert!((net_monetary_benefit(30.0, 400_000.0, 20_000.0) - 200_000.0).abs() < 1e-9);
//! // App only: 20,000 × 12 − 150,000 = £90,000.
//! assert!((net_monetary_benefit(12.0, 150_000.0, 20_000.0) - 90_000.0).abs() < 1e-9);
//! // Extra clinics: 20,000 × 32 − 700,000 = −£60,000 → rejected.
//! let clinics = net_monetary_benefit(32.0, 700_000.0, 20_000.0);
//! assert!((clinics - -60_000.0).abs() < 1e-9);
//! assert!(!adopt(clinics));
//!
//! // NMB ranks all three at once: App + coaching wins.
//! let options = [
//!     EvaluatedOption { name: "App + coaching", delta_cost: 400_000.0, delta_effect: 30.0 },
//!     EvaluatedOption { name: "App only", delta_cost: 150_000.0, delta_effect: 12.0 },
//!     EvaluatedOption { name: "Extra clinics", delta_cost: 700_000.0, delta_effect: 32.0 },
//! ];
//! assert_eq!(best_option_index(&options, 20_000.0), Some(0));
//!
//! // NHB view of the winner: 30 − 400,000/20,000 = 10 QALYs net.
//! let nhb = net_health_benefit(30.0, 400_000.0, 20_000.0).unwrap();
//! assert!((nhb - 10.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - `(hours saved × loaded hourly rate) − tool cost` — the everyday tooling
//!   business case — is literally an NMB calculation with λ = loaded
//!   engineer cost.
//! - **Make λ a variable, not a constant**: plot NMB against λ ("value of an
//!   engineer-hour") and show where the decision flips; stakeholders can
//!   apply their own valuation without redoing your math.
//! - **NHB thinking**: "this platform saves 5,000 engineer-hours but consumes
//!   budget that would have bought 3,000 engineer-hours of contractor
//!   capacity — net 2,000 hours" forces the opportunity-cost comparison in
//!   capacity units.
//!
//! ## Pitfalls
//!
//! - **Hiding the threshold**: an NMB is meaningless without stating λ;
//!   report NMB at £20k and £30k, or plot the curve.
//! - **Using NMB to launder tiny effects**: a huge population times a
//!   negligible per-person effect can produce a big NMB — report per-person
//!   effects alongside.
//! - **Forgetting NMB inherits every uncertainty** in ΔC and ΔE — pair with
//!   probabilistic sensitivity analysis.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: net monetary benefit.
//!   <https://yhec.co.uk/glossary/net-monetary-benefit/>
//! - Stinnett AA, Mullahy J. "Net health benefits: a new framework for the
//!   analysis of uncertainty in cost-effectiveness analysis."
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC2528971/>
//!
//! Topic doc: health-economics-metrics/topics/net-monetary-benefit.md

/// One option under evaluation: its incremental cost and effect versus the common baseline.
///
/// All options in a ranking must share the same baseline and the same units
/// (currency for ΔC, e.g. QALYs for ΔE) or their NMBs are not comparable.
pub struct EvaluatedOption {
    /// Name of the option (e.g. "App + coaching").
    pub name: &'static str,
    /// Incremental cost ΔC versus the common baseline (currency).
    pub delta_cost: f64,
    /// Incremental effect ΔE versus the common baseline (e.g. QALYs).
    pub delta_effect: f64,
}

/// Net monetary benefit: NMB = (ΔE × λ) − ΔC.
///
/// The health gain priced at the threshold, minus the cost — in currency
/// units. Linear in both ΔE and ΔC, so it can be averaged across Monte Carlo
/// draws and summed across components.
///
/// # Arguments
///
/// * `delta_effect` — incremental effect ΔE (e.g. QALYs).
/// * `delta_cost` — incremental cost ΔC (currency).
/// * `lambda` — willingness-to-pay threshold λ (currency per effect unit).
///
/// # Returns
///
/// The NMB in currency units; positive means adopt at this λ.
///
/// # Examples
///
/// ```rust
/// use health_economics::net_monetary_benefit::net_monetary_benefit;
///
/// // App + coaching: 20,000 × 30 − 400,000 = £200,000.
/// let nmb = net_monetary_benefit(30.0, 400_000.0, 20_000.0);
/// assert!((nmb - 200_000.0).abs() < 1e-9);
///
/// // Extra clinics: 20,000 × 32 − 700,000 = −£60,000 (value destroyed).
/// let clinics = net_monetary_benefit(32.0, 700_000.0, 20_000.0);
/// assert!((clinics - -60_000.0).abs() < 1e-9);
/// ```
pub fn net_monetary_benefit(delta_effect: f64, delta_cost: f64, lambda: f64) -> f64 {
    delta_effect * lambda - delta_cost
}

/// Net health benefit: NHB = ΔE − (ΔC / λ).
///
/// The same decision rule as NMB, expressed in health units: the health
/// gained beyond what the same money would have produced elsewhere (ΔC / λ
/// is the health the budget displaces at the margin).
///
/// # Arguments
///
/// * `delta_effect` — incremental effect ΔE (e.g. QALYs).
/// * `delta_cost` — incremental cost ΔC (currency).
/// * `lambda` — willingness-to-pay threshold λ (currency per effect unit).
///
/// # Returns
///
/// `Some(NHB in effect units)`, or `None` when λ = 0 (the health-equivalent
/// of cost is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::net_monetary_benefit::net_health_benefit;
///
/// // Winner's NHB: 30 − 400,000/20,000 = 30 − 20 = 10 QALYs net.
/// let nhb = net_health_benefit(30.0, 400_000.0, 20_000.0).unwrap();
/// assert!((nhb - 10.0).abs() < 1e-9);
///
/// assert!(net_health_benefit(30.0, 400_000.0, 0.0).is_none());
/// ```
pub fn net_health_benefit(delta_effect: f64, delta_cost: f64, lambda: f64) -> Option<f64> {
    if lambda == 0.0 {
        None
    } else {
        // ΔC / λ converts the cost into the health it displaces at the
        // margin; NHB is the health gained net of that displacement.
        Some(delta_effect - delta_cost / lambda)
    }
}

/// Decision rule: adopt if NMB > 0.
///
/// Strictly positive; an NMB of exactly zero is indifference, not adoption.
/// Equivalent to ICER < λ when ΔE > 0.
///
/// # Arguments
///
/// * `nmb` — a computed net monetary benefit (currency).
///
/// # Returns
///
/// `true` when `nmb > 0`.
///
/// # Examples
///
/// ```rust
/// use health_economics::net_monetary_benefit::adopt;
///
/// assert!(adopt(200_000.0));   // App + coaching: £200,000 → adopt
/// assert!(!adopt(-60_000.0));  // Extra clinics: −£60,000 → reject
/// ```
pub fn adopt(nmb: f64) -> bool {
    nmb > 0.0
}

/// Rank options at threshold `lambda` and return the index of the highest NMB.
///
/// This is the linearity that pairwise ICERs lack: any number of options are
/// ranked in one pass, with no frontier procedure. Ties keep the earliest
/// option (a later option must strictly exceed the incumbent to win).
///
/// # Arguments
///
/// * `options` — the options to rank, each with ΔC and ΔE versus the same
///   baseline.
/// * `lambda` — willingness-to-pay threshold λ (currency per effect unit).
///
/// # Returns
///
/// `Some(index of the option with the highest NMB)`, or `None` for an empty
/// slice.
///
/// # Examples
///
/// ```rust
/// use health_economics::net_monetary_benefit::{
///     best_option_index, EvaluatedOption,
/// };
///
/// let options = [
///     EvaluatedOption { name: "App + coaching", delta_cost: 400_000.0, delta_effect: 30.0 },
///     EvaluatedOption { name: "App only", delta_cost: 150_000.0, delta_effect: 12.0 },
///     EvaluatedOption { name: "Extra clinics", delta_cost: 700_000.0, delta_effect: 32.0 },
/// ];
/// // At £20,000/QALY, App + coaching (NMB £200,000) wins.
/// let winner = best_option_index(&options, 20_000.0).unwrap();
/// assert_eq!(options[winner].name, "App + coaching");
///
/// assert!(best_option_index(&[], 20_000.0).is_none());
/// ```
pub fn best_option_index(options: &[EvaluatedOption], lambda: f64) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, opt) in options.iter().enumerate() {
        let nmb = net_monetary_benefit(opt.delta_effect, opt.delta_cost, lambda);
        // Keep the incumbent on ties (strict > required to replace), so the
        // earliest of equally good options wins.
        match best {
            Some((_, best_nmb)) if nmb <= best_nmb => {}
            _ => best = Some((i, nmb)),
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAMBDA: f64 = 20_000.0;

    fn options() -> [EvaluatedOption; 3] {
        [
            EvaluatedOption { name: "App + coaching", delta_cost: 400_000.0, delta_effect: 30.0 },
            EvaluatedOption { name: "App only", delta_cost: 150_000.0, delta_effect: 12.0 },
            EvaluatedOption { name: "Extra clinics", delta_cost: 700_000.0, delta_effect: 32.0 },
        ]
    }

    /// App + coaching: 20,000 × 30 − 400,000 = £200,000.
    #[test]
    fn app_plus_coaching_nmb_is_200k() {
        // Worked example: "App + coaching … 600,000 − 400,000 = £200,000".
        let got = net_monetary_benefit(30.0, 400_000.0, LAMBDA);
        assert!((got - 200_000.0).abs() < 1e-9);
    }

    /// App only: 20,000 × 12 − 150,000 = £90,000.
    #[test]
    fn app_only_nmb_is_90k() {
        // Worked example: "App only … 240,000 − 150,000 = £90,000".
        let got = net_monetary_benefit(12.0, 150_000.0, LAMBDA);
        assert!((got - 90_000.0).abs() < 1e-9);
    }

    /// Extra clinics: 20,000 × 32 − 700,000 = −£60,000 — most QALYs gained,
    /// but value destroyed at this threshold.
    #[test]
    fn extra_clinics_nmb_is_minus_60k_and_rejected() {
        // Worked example: "Extra clinics … 640,000 − 700,000 = −£60,000".
        let got = net_monetary_benefit(32.0, 700_000.0, LAMBDA);
        assert!((got - -60_000.0).abs() < 1e-9);
        assert!(!adopt(got));
    }

    /// NMB ranks all three at once: App + coaching wins.
    #[test]
    fn app_plus_coaching_wins_the_ranking() {
        // Worked example: "App + coaching wins. Note NMB lets you rank all
        // three at once".
        let opts = options();
        let winner = best_option_index(&opts, LAMBDA).unwrap();
        assert_eq!(winner, 0);
        assert_eq!(opts[winner].name, "App + coaching");
    }

    /// NHB view of the winner: 30 − 400,000/20,000 = 30 − 20 = 10 QALYs net.
    #[test]
    fn winner_nhb_is_10_qalys_net() {
        // Worked example: "NHB view of the winner: 30 − 400,000/20,000
        // = 30 − 20 = 10 QALYs net".
        let got = net_health_benefit(30.0, 400_000.0, LAMBDA).unwrap();
        assert!((got - 10.0).abs() < 1e-9);
    }

    /// NMB > 0 ⇔ ICER < λ when ΔE > 0: the two rules agree.
    #[test]
    fn nmb_rule_agrees_with_icer_rule() {
        // Doc's math: "NMB > 0 ⇔ ICER < λ (when ΔE > 0), so the two rules
        // agree — NMB is just better behaved".
        for opt in options() {
            let nmb = net_monetary_benefit(opt.delta_effect, opt.delta_cost, LAMBDA);
            let icer = opt.delta_cost / opt.delta_effect;
            assert_eq!(nmb > 0.0, icer < LAMBDA);
        }
    }

    /// NHB is undefined at λ = 0; ranking an empty option set yields nothing.
    #[test]
    fn degenerate_inputs_return_none() {
        // Edge cases: NHB divides by λ, and an empty ranking has no winner.
        assert!(net_health_benefit(30.0, 400_000.0, 0.0).is_none());
        assert!(best_option_index(&[], LAMBDA).is_none());
    }
}
