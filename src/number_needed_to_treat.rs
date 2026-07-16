//! # Number Needed to Treat (NNT)
//!
//! NNT is the number of patients who must receive an intervention for **one**
//! additional patient to benefit, over a stated time frame. It converts
//! percentage risk reductions — which mislead — into effort-per-benefit units
//! that anyone can reason about.
//!
//! Its mirror, **NNH** (number needed to harm), counts how many treated per
//! person harmed. Always state the time frame and baseline population — NNT
//! is meaningless without both.
//!
//! ## Formula
//!
//! ```text
//! ARR = control event rate − treatment event rate   (absolute risk reduction)
//! NNT = 1 / ARR
//!
//! NNH = 1 / (harm rate_treatment − harm rate_control)
//!
//! Economic bridge:
//! cost per event prevented = NNT × cost per treatment course
//!
//! event rate = proportion experiencing the event (e.g. 0.032 for 3.2%)
//! ```
//!
//! ## Why it matters
//!
//! "Reduces heart attacks by 25%!" sounds decisive. If the baseline risk is
//! 4% over 5 years, the absolute reduction is 1 percentage point, so **100
//! people must take the drug for 5 years for 1 to benefit** — and all 100 pay
//! the costs and side effects. NNT is the antidote to relative-risk
//! marketing, which is why evidence-based medicine leads with it. Statins for
//! primary prevention: NNT ≈ 50–100 over 5 years per heart attack avoided.
//!
//! ## Example
//!
//! A falls-prediction system flags high-risk patients for an intervention
//! bundle (bed sensors, review, supervision). Trial: falls with injury drop
//! from 3.2% to 2.4% of admissions.
//!
//! ```rust
//! use health_economics::number_needed_to_treat::{
//!     absolute_risk_reduction, cost_per_event_prevented, number_needed_to_treat,
//!     prevention_payoff_ratio, relative_risk_reduction,
//! };
//!
//! // ARR = 0.032 − 0.024 = 0.8 percentage points → NNT = 1/0.008 = 125.
//! let arr = absolute_risk_reduction(0.032, 0.024);
//! assert!((arr - 0.008).abs() < 1e-9);
//! let nnt = number_needed_to_treat(arr).unwrap();
//! assert!((nnt - 125.0).abs() < 1e-9);
//!
//! // Intervention ≈ £40/patient → cost per fall prevented = 125 × 40 = £5,000.
//! let cost = cost_per_event_prevented(nnt, 40.0);
//! assert!((cost - 5_000.0).abs() < 1e-9);
//!
//! // An injurious inpatient fall costs ≈ £12,000 → prevention pays ~2.4:1.
//! let payoff = prevention_payoff_ratio(12_000.0, cost).unwrap();
//! assert!((payoff - 2.4).abs() < 1e-9);
//!
//! // "Reduces falls 25%" is the same result, differently persuasive.
//! let rrr = relative_risk_reduction(0.032, 0.024).unwrap();
//! assert!((rrr - 0.25).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - NNT is the right unit for any gate or check that acts on many items to
//!   catch few: "number of PRs that must pass through the AI review gate to
//!   catch one production-bound defect."
//! - If the gate reviews 400 PRs per real catch (NNT = 400) at 4 minutes of
//!   developer attention each, one catch costs ~27 developer-hours — compare
//!   that to the incident cost it prevents.
//! - NNH maps to false positives: how many PRs per *false* flag, and what
//!   does each cost in attention and trust?
//! - Screening-style tooling (linters, security scanners, anomaly detection)
//!   should ship with NNT/NNH arithmetic — low prevalence makes these
//!   numbers brutal.
//!
//! ## Pitfalls
//!
//! - **No time frame**: "NNT = 50" means nothing; "NNT = 50 over 5 years" is
//!   a claim.
//! - **Baseline-risk transplantation**: NNT computed in a high-risk trial
//!   population collapses in a low-risk deployment population.
//! - **Ignoring NNH** — a gate with NNT 400 and NNH 3 is a nuisance
//!   generator, not a safety system.
//!
//! ## Sources
//!
//! - Laupacis A, Sackett DL, Roberts RS. "An assessment of clinically useful
//!   measures of the consequences of treatment." NEJM 1988.
//!   <https://pubmed.ncbi.nlm.nih.gov/3374545/>
//! - TheNNT explained. <https://www.thennt.com/thennt-explained/>
//!
//! Topic doc: health-economics-metrics/topics/number-needed-to-treat.md

/// Absolute risk reduction: control event rate minus treatment event rate.
///
/// Both rates are proportions (e.g. 0.032 for 3.2%), over the same time frame
/// and population. ARR is the honest counterpart to the relative risk
/// reduction: it carries the baseline risk with it.
///
/// # Arguments
///
/// * `control_event_rate` — event rate without the intervention (proportion).
/// * `treatment_event_rate` — event rate with the intervention (proportion).
///
/// # Returns
///
/// The ARR as a proportion; negative when the treatment *increases* the
/// event rate.
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::absolute_risk_reduction;
///
/// // Falls with injury drop from 3.2% to 2.4%: ARR = 0.8 percentage points.
/// let arr = absolute_risk_reduction(0.032, 0.024);
/// assert!((arr - 0.008).abs() < 1e-9);
/// ```
pub fn absolute_risk_reduction(control_event_rate: f64, treatment_event_rate: f64) -> f64 {
    control_event_rate - treatment_event_rate
}

/// Relative risk reduction: ARR / control event rate.
///
/// The flattering marketing number ("reduces falls 25%") — the same trial
/// result as the NNT, differently persuasive, because it hides the baseline
/// risk.
///
/// # Arguments
///
/// * `control_event_rate` — event rate without the intervention (proportion).
/// * `treatment_event_rate` — event rate with the intervention (proportion).
///
/// # Returns
///
/// `Some(RRR as a fraction, 0.25 = 25%)`, or `None` when the control event
/// rate is zero (relative reduction is undefined with no baseline events).
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::relative_risk_reduction;
///
/// // 3.2% → 2.4% is a 25% relative reduction — same trial as NNT = 125.
/// let rrr = relative_risk_reduction(0.032, 0.024).unwrap();
/// assert!((rrr - 0.25).abs() < 1e-9);
///
/// assert!(relative_risk_reduction(0.0, 0.0).is_none());
/// ```
pub fn relative_risk_reduction(control_event_rate: f64, treatment_event_rate: f64) -> Option<f64> {
    if control_event_rate == 0.0 {
        None
    } else {
        Some(absolute_risk_reduction(control_event_rate, treatment_event_rate) / control_event_rate)
    }
}

/// NNT = 1 / ARR: patients treated per additional patient benefiting.
///
/// Always state the time frame and baseline population alongside — "NNT = 50"
/// means nothing; "NNT = 50 over 5 years" is a claim. A negative ARR yields a
/// negative value, which reads as a number needed to harm.
///
/// # Arguments
///
/// * `absolute_risk_reduction` — the ARR as a proportion (e.g. 0.008 for 0.8
///   percentage points).
///
/// # Returns
///
/// `Some(1 / ARR)`, or `None` when the ARR is zero (no effect — no finite
/// number of patients yields one extra benefit).
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::number_needed_to_treat;
///
/// // ARR = 0.008 → NNT = 125 patients per injurious fall prevented.
/// let nnt = number_needed_to_treat(0.008).unwrap();
/// assert!((nnt - 125.0).abs() < 1e-9);
///
/// assert!(number_needed_to_treat(0.0).is_none());
/// ```
pub fn number_needed_to_treat(absolute_risk_reduction: f64) -> Option<f64> {
    if absolute_risk_reduction == 0.0 {
        None
    } else {
        Some(1.0 / absolute_risk_reduction)
    }
}

/// NNH = 1 / (harm rate on treatment − harm rate on control).
///
/// The mirror of NNT on the harm side: how many treated per person harmed.
/// A gate with NNT 400 and NNH 3 is a nuisance generator, not a safety
/// system.
///
/// # Arguments
///
/// * `harm_rate_treatment` — harm rate with the intervention (proportion).
/// * `harm_rate_control` — harm rate without it (proportion).
///
/// # Returns
///
/// `Some(1 / excess harm rate)`, or `None` when the two rates are equal
/// (no excess harm — no finite NNH).
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::number_needed_to_harm;
///
/// // Harm 5% on treatment vs 3% on control → NNH = 1/0.02 = 50.
/// let nnh = number_needed_to_harm(0.05, 0.03).unwrap();
/// assert!((nnh - 50.0).abs() < 1e-9);
///
/// assert!(number_needed_to_harm(0.02, 0.02).is_none());
/// ```
pub fn number_needed_to_harm(harm_rate_treatment: f64, harm_rate_control: f64) -> Option<f64> {
    // Excess harm attributable to treatment; NNH inverts it just as NNT
    // inverts the ARR.
    let excess = harm_rate_treatment - harm_rate_control;
    if excess == 0.0 {
        None
    } else {
        Some(1.0 / excess)
    }
}

/// Economic bridge: cost per event prevented = NNT × cost per treatment course.
///
/// Everyone treated pays the treatment cost, but only one in NNT benefits —
/// so preventing one event costs the whole cohort's treatment.
///
/// # Arguments
///
/// * `nnt` — the number needed to treat.
/// * `cost_per_treatment_course` — cost of treating one patient (currency).
///
/// # Returns
///
/// The cost of preventing one event (`nnt × cost_per_treatment_course`).
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::cost_per_event_prevented;
///
/// // NNT 125 at ≈ £40/patient → cost per fall prevented = £5,000.
/// let cost = cost_per_event_prevented(125.0, 40.0);
/// assert!((cost - 5_000.0).abs() < 1e-9);
/// ```
pub fn cost_per_event_prevented(nnt: f64, cost_per_treatment_course: f64) -> f64 {
    nnt * cost_per_treatment_course
}

/// Benefit-to-cost ratio of prevention: cost of the event avoided over the cost of preventing one.
///
/// A ratio above 1 means prevention pays before counting any health (QALY)
/// gain, which is on top.
///
/// # Arguments
///
/// * `cost_of_event` — cost of one event when it happens (currency, e.g. an
///   injurious inpatient fall ≈ £12,000: extra stay, imaging, litigation).
/// * `cost_per_event_prevented` — cost of preventing one event (currency,
///   from [`cost_per_event_prevented`]).
///
/// # Returns
///
/// `Some(ratio)`, or `None` when the prevention cost is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::number_needed_to_treat::prevention_payoff_ratio;
///
/// // £12,000 fall avoided for £5,000 spent → prevention pays ~2.4:1.
/// let payoff = prevention_payoff_ratio(12_000.0, 5_000.0).unwrap();
/// assert!((payoff - 2.4).abs() < 1e-9);
///
/// assert!(prevention_payoff_ratio(12_000.0, 0.0).is_none());
/// ```
pub fn prevention_payoff_ratio(cost_of_event: f64, cost_per_event_prevented: f64) -> Option<f64> {
    if cost_per_event_prevented == 0.0 {
        None
    } else {
        Some(cost_of_event / cost_per_event_prevented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Falls with injury drop from 3.2% to 2.4%: ARR = 0.8 percentage points.
    #[test]
    fn falls_trial_arr_is_0_8_percentage_points() {
        // Worked example: "falls with injury drop from 3.2% to 2.4% …
        // ARR = 0.8 percentage points".
        let got = absolute_risk_reduction(0.032, 0.024);
        assert!((got - 0.008).abs() < 1e-9);
    }

    /// NNT = 1 / 0.008 = 125 patients per injurious fall prevented.
    #[test]
    fn falls_trial_nnt_is_125() {
        // Worked example: "NNT = 1/0.008 = 125 (125 patients must get the
        // intervention bundle to prevent 1 injurious fall)".
        let arr = absolute_risk_reduction(0.032, 0.024);
        let got = number_needed_to_treat(arr).unwrap();
        assert!((got - 125.0).abs() < 1e-9);
    }

    /// Intervention ≈ £40/patient → cost per fall prevented = 125 × 40 = £5,000.
    #[test]
    fn cost_per_fall_prevented_is_5000() {
        // Worked example: "cost per fall prevented = 125 × 40 = £5,000".
        let got = cost_per_event_prevented(125.0, 40.0);
        assert!((got - 5_000.0).abs() < 1e-9);
    }

    /// Cost of an injurious inpatient fall ≈ £12,000 → prevention pays ~2.4:1.
    #[test]
    fn prevention_pays_about_2_4_to_1() {
        // Worked example: "Net: prevention pays ~2.4:1 — and the QALY gain
        // is on top".
        let got = prevention_payoff_ratio(12_000.0, 5_000.0).unwrap();
        assert!((got - 2.4).abs() < 1e-9);
    }

    /// "Reduces falls 25%" and "prevent one fall per 125 patients treated"
    /// are the same result, differently persuasive.
    #[test]
    fn rrr_25_percent_equals_nnt_125() {
        // Worked example: "'reduces falls 25%' and 'prevent one fall per 125
        // patients treated' are the same result, differently persuasive".
        let rrr = relative_risk_reduction(0.032, 0.024).unwrap();
        assert!((rrr - 0.25).abs() < 1e-9);
    }

    /// The "why it matters" example: 25% relative reduction on a 4% baseline
    /// is 1 percentage point absolute — 100 people treated per benefit.
    #[test]
    fn heart_attack_marketing_example_nnt_is_100() {
        // "Why it matters": "baseline risk is 4% … absolute reduction is 1
        // percentage point, so 100 people must take the drug … for 1 to
        // benefit".
        let arr = absolute_risk_reduction(0.04, 0.03);
        let got = number_needed_to_treat(arr).unwrap();
        assert!((got - 100.0).abs() < 1e-9);
    }

    /// NNH mirrors NNT on the harm side; both are undefined at zero effect.
    #[test]
    fn nnh_and_zero_effect_edge_cases() {
        // Doc's math: "NNH = 1 / (harm rate_treatment − harm rate_control)";
        // all three ratios are undefined at zero effect/baseline.
        let nnh = number_needed_to_harm(0.05, 0.03).unwrap();
        assert!((nnh - 50.0).abs() < 1e-9);
        assert!(number_needed_to_treat(0.0).is_none());
        assert!(number_needed_to_harm(0.02, 0.02).is_none());
        assert!(relative_risk_reduction(0.0, 0.0).is_none());
    }
}
