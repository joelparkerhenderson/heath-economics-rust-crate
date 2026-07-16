//! # Cost-Utility Analysis (CUA)
//!
//! CUA is cost-effectiveness analysis with a **generic, preference-weighted
//! outcome** — almost always the QALY (or DALY averted). Because the outcome
//! unit is universal, CUA can compare interventions across completely
//! different diseases.
//!
//! Its output — the incremental cost-utility ratio (ICUR), cost per QALY —
//! is judged against a willingness-to-pay threshold, and is the closest thing
//! health policy has to a universal exchange rate.
//!
//! ## Formula
//!
//! ```text
//! ICUR = ΔCost / ΔQALYs      (the ICER with QALYs as the effect unit)
//!
//! ΔQALYs = Σ (duration_i × utility_i)_new − Σ (duration_i × utility_i)_old
//!
//! ΔCost      — incremental cost of the new pathway over the old (£)
//! ΔQALYs     — incremental QALYs gained
//! duration_i — time spent in health state i (years)
//! utility_i  — preference-weighted utility of state i (0 = dead, 1 = full health)
//! ```
//!
//! Utilities come from validated instruments (EQ-5D); costs and QALYs are
//! both discounted at 3.5% in the NICE reference case.
//!
//! ## Why it matters
//!
//! A national health service must choose between a cancer drug, a
//! mental-health app, and a surgical robot from one budget. Natural units
//! can't compare them; QALYs can. CUA is therefore the reference-case method
//! at NICE and most HTA bodies, with results judged against the £20,000–30,000
//! per QALY threshold. If you want your software funded *instead of something
//! else*, CUA is the arena.
//!
//! ## Example
//!
//! The topic doc's worked example: a CBT app for moderate anxiety vs the
//! waiting list. App costs £250; 40% of users no longer need £1,700 therapy
//! (−£680), so ΔC = −£430; six months at utility 0.76 instead of 0.68 gives
//! ΔE = +0.04 QALYs — the app dominates. With only 10% displacement,
//! ΔC = +£80 and ICUR = £2,000/QALY, still far below £20,000.
//!
//! ```rust
//! use health_economics::cost_utility_analysis::{
//!     HealthState, delta_qalys, displaced_care_saving, icur, is_dominant,
//! };
//!
//! // Costs: app licence £250, therapy displaced −£680 (40% × £1,700).
//! let delta_cost = 250.0 - displaced_care_saving(0.40, 1_700.0);
//! assert_eq!(delta_cost, -430.0); // saves money
//!
//! // QALYs: 6 months at utility 0.76 instead of 0.68 while waiting.
//! let new = [HealthState { duration_years: 0.5, utility: 0.76 }];
//! let old = [HealthState { duration_years: 0.5, utility: 0.68 }];
//! let de = delta_qalys(&new, &old);
//! assert!((de - 0.04).abs() < 1e-9);
//!
//! // ΔC < 0 and ΔE > 0: the app dominates — no ratio needed.
//! assert!(is_dominant(delta_cost, de));
//!
//! // Slash the key assumption to 10% displacement: ΔC = +£80,
//! // ICUR = 80 / 0.04 = £2,000/QALY — still far below £20,000.
//! let delta_cost_10 = 250.0 - displaced_care_saving(0.10, 1_700.0);
//! assert_eq!(delta_cost_10, 80.0);
//! let ratio = icur(delta_cost_10, de).unwrap();
//! assert!((ratio - 2_000.0).abs() < 1e-9);
//! assert!(ratio < 20_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - CUA's deep idea — *one composite, preference-weighted unit to compare
//!   unlike things* — is the pattern for comparing unlike engineering
//!   investments (security vs developer experience vs reliability).
//! - The honest options are either a defensible composite unit (rare) or an
//!   explicit cost-consequence table (usual).
//! - What CUA warns against is the fake composite: a weighted "impact score"
//!   whose weights were tuned after the fact to make the preferred option win.
//! - Health economics spent decades standardizing utility elicitation
//!   precisely so weights precede the comparison.
//!
//! ## Pitfalls
//!
//! - **Utility gains below the instrument's sensitivity** (minimal clinically
//!   important difference) — tiny ΔE times large populations is a classic
//!   laundering trick.
//! - **Missing comparator care displacement** — the biggest cost term for
//!   digital products is often what they replace.
//! - **Mapping non-preference scores to utilities** with unvalidated
//!   crosswalks.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: cost-utility analysis.
//!   <https://yhec.co.uk/glossary/cost-utility-analysis/>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/cost-utility-analysis.md

/// A health state occupied for a period.
///
/// QALYs are accrued as duration × utility; a sequence of `HealthState`s
/// describes a pathway (e.g. "6 months waiting at utility 0.68, then treated").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthState {
    /// Time spent in the state, in years (e.g. 0.5 for six months).
    pub duration_years: f64,
    /// Preference-weighted utility of the state, 0 = dead, 1 = full health
    /// (typically from EQ-5D).
    pub utility: f64,
}

/// Total QALYs accrued over a sequence of health states.
///
/// Computes Σ duration_i × utility_i across the given states.
///
/// # Arguments
///
/// * `states` — the health states occupied, each with duration (years) and
///   utility (0–1).
///
/// # Returns
///
/// Total QALYs (years of full-health-equivalent life).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_utility_analysis::{HealthState, total_qalys};
///
/// // 6 months at utility 0.76 = 0.38 QALYs.
/// let states = [HealthState { duration_years: 0.5, utility: 0.76 }];
/// assert!((total_qalys(&states) - 0.38).abs() < 1e-9);
/// ```
pub fn total_qalys(states: &[HealthState]) -> f64 {
    states.iter().map(|s| s.duration_years * s.utility).sum()
}

/// Incremental QALYs of the new pathway over the old.
///
/// ΔQALYs = QALYs(new) − QALYs(old); positive means the new pathway gains
/// health.
///
/// # Arguments
///
/// * `new_states` — health states under the new pathway.
/// * `old_states` — health states under the comparator pathway.
///
/// # Returns
///
/// ΔQALYs (may be negative if the new pathway is worse).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_utility_analysis::{HealthState, delta_qalys};
///
/// // 6 months at 0.76 instead of 0.68: ΔE = 0.5 × (0.76 − 0.68) = +0.04 QALYs.
/// let new = [HealthState { duration_years: 0.5, utility: 0.76 }];
/// let old = [HealthState { duration_years: 0.5, utility: 0.68 }];
/// assert!((delta_qalys(&new, &old) - 0.04).abs() < 1e-9);
/// ```
pub fn delta_qalys(new_states: &[HealthState], old_states: &[HealthState]) -> f64 {
    total_qalys(new_states) - total_qalys(old_states)
}

/// Cost displaced by the intervention.
///
/// The fraction of users who no longer need the comparator care × the cost of
/// that care — often the biggest cost term for digital products.
///
/// # Arguments
///
/// * `displacement_fraction` — fraction of users (0–1) who no longer need the
///   comparator care.
/// * `comparator_cost` — cost of the comparator care per user (£).
///
/// # Returns
///
/// Expected saving per user (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_utility_analysis::displaced_care_saving;
///
/// // 40% of app users no longer need £1,700 face-to-face therapy: £680 saved.
/// assert_eq!(displaced_care_saving(0.40, 1_700.0), 680.0);
/// ```
pub fn displaced_care_saving(displacement_fraction: f64, comparator_cost: f64) -> f64 {
    displacement_fraction * comparator_cost
}

/// Incremental cost-utility ratio: ΔCost / ΔQALYs (£/QALY).
///
/// The ICER with QALYs as the effect unit; judged against the
/// willingness-to-pay threshold (£20,000–30,000/QALY at NICE). Note that a
/// negative ICUR is ambiguous on its own (dominant vs dominated) — check
/// [`is_dominant`] first.
///
/// # Arguments
///
/// * `delta_cost` — incremental cost, £ (negative = saves money).
/// * `delta_qalys` — incremental QALYs (negative = loses health).
///
/// # Returns
///
/// `Some(£/QALY)`, or `None` when `delta_qalys` is exactly zero (no ratio is
/// defined).
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_utility_analysis::icur;
///
/// // ΔC = +£80, ΔE = +0.04 QALYs: ICUR = £2,000/QALY.
/// assert_eq!(icur(80.0, 0.04), Some(2_000.0));
/// // No QALY difference: no ratio.
/// assert_eq!(icur(100.0, 0.0), None);
/// ```
pub fn icur(delta_cost: f64, delta_qalys: f64) -> Option<f64> {
    if delta_qalys == 0.0 {
        None
    } else {
        Some(delta_cost / delta_qalys)
    }
}

/// True when the intervention dominates its comparator.
///
/// Dominance means it saves money (ΔC < 0) *and* gains health (ΔE > 0) —
/// better and cheaper, no ratio needed.
///
/// # Arguments
///
/// * `delta_cost` — incremental cost, £.
/// * `delta_qalys` — incremental QALYs.
///
/// # Returns
///
/// `true` iff ΔC < 0 and ΔE > 0.
///
/// # Examples
///
/// ```rust
/// use health_economics::cost_utility_analysis::is_dominant;
///
/// // ΔC = −£430, ΔE = +0.04: the CBT app dominates the waiting list.
/// assert!(is_dominant(-430.0, 0.04));
/// assert!(!is_dominant(80.0, 0.04)); // costs money — not dominant
/// ```
pub fn is_dominant(delta_cost: f64, delta_qalys: f64) -> bool {
    delta_cost < 0.0 && delta_qalys > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const APP_COST: f64 = 250.0;
    const THERAPY_COST: f64 = 1_700.0; // £680 = 40% displacement

    // Worked example: "therapy displaced −£680 (40% of users no longer need it)".
    #[test]
    fn therapy_displacement_at_40_percent_saves_680() {
        assert!((displaced_care_saving(0.40, THERAPY_COST) - 680.0).abs() < 1e-9);
    }

    // Worked example: "ΔC = 250 − 680 = −£430 (saves money)".
    #[test]
    fn delta_cost_is_minus_430() {
        let dc = APP_COST - displaced_care_saving(0.40, THERAPY_COST);
        assert!((dc - (-430.0)).abs() < 1e-9);
    }

    // Worked example: "ΔE = 0.5 × (0.76 − 0.68) = +0.04 QALYs".
    #[test]
    fn delta_qalys_is_plus_0_04() {
        // 6 months at utility 0.76 instead of 0.68 while waiting
        let new = [HealthState { duration_years: 0.5, utility: 0.76 }];
        let old = [HealthState { duration_years: 0.5, utility: 0.68 }];
        assert!((delta_qalys(&new, &old) - 0.04).abs() < 1e-9);
    }

    // Worked example: "ΔC < 0 and ΔE > 0: the app dominates".
    #[test]
    fn app_dominates_waiting_list() {
        let dc = APP_COST - displaced_care_saving(0.40, THERAPY_COST);
        let new = [HealthState { duration_years: 0.5, utility: 0.76 }];
        let old = [HealthState { duration_years: 0.5, utility: 0.68 }];
        assert!(is_dominant(dc, delta_qalys(&new, &old)));
    }

    // Worked example: "Had the therapy-displacement assumption been only 10%,
    // ΔC = 250 − 170 = +£80".
    #[test]
    fn ten_percent_displacement_gives_delta_cost_plus_80() {
        let dc = APP_COST - displaced_care_saving(0.10, THERAPY_COST);
        assert!((dc - 80.0).abs() < 1e-9);
    }

    // Worked example: "ICUR = 80 / 0.04 = £2,000/QALY — still far below £20,000".
    #[test]
    fn icur_at_10_percent_displacement_is_2000_per_qaly() {
        let dc = APP_COST - displaced_care_saving(0.10, THERAPY_COST);
        let icur = icur(dc, 0.04).unwrap();
        assert!((icur - 2_000.0).abs() < 1e-9);
        // still far below the £20,000/QALY threshold
        assert!(icur < 20_000.0);
    }

    // Edge case: ΔQALYs = 0 means no ratio is defined.
    #[test]
    fn icur_is_none_when_no_qaly_difference() {
        assert!(icur(100.0, 0.0).is_none());
    }
}
