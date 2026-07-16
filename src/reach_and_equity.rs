//! # Reach and Equity (RE-AIM)
//!
//! RE-AIM — Reach, Effectiveness, Adoption, Implementation, Maintenance — is
//! the standard framework for judging the *population* impact of an
//! intervention. Its central arithmetic: **public-health impact ≈ reach ×
//! effectiveness**.
//!
//! Digital tools add an equity dimension: the digital divide means reach is
//! systematically uneven, and digital-first delivery can widen the health
//! gaps it aims to close — so impact must be stratified by equity group,
//! because an unstratified average is where inequity hides.
//!
//! ## Formula
//!
//! ```text
//! Population impact ≈ reach × effectiveness
//!   reach         = participants / eligible population
//!   effectiveness = real-world effect among participants
//!
//! Equity-stratified version:
//!   impact_group_g = reach_g × effectiveness_g
//!   equity gap     = impact_top quintile − impact_bottom quintile
//!
//! Distributional cost-effectiveness: equity-weighted QALYs by recipient group
//! ```
//!
//! Legend:
//! - `reach` — fraction of the eligible population that participates (not of
//!   registered users).
//! - `effectiveness` — real-world (retention-weighted) effect per
//!   participant, e.g. QALYs.
//! - `impact_group_g` — impact per eligible person in stratum `g`
//!   (deprivation quintile, age band, language group).
//! - `equity gap` — absolute impact difference between the top (least
//!   deprived) and bottom (most deprived) strata.
//!
//! ## Why it matters
//!
//! Systematic reviews applying RE-AIM to mHealth find a consistent
//! signature: strong Reach and Adoption, **weak Effectiveness and
//! Maintenance** — apps spread easily and fade fast. For a national health
//! service, an impressive per-user product can be a poor population
//! investment, and vice versa: a modestly effective tool reaching millions
//! can outproduce a brilliant one reaching thousands. Equity is not a side
//! constraint but a value driver: digital exclusion tracks age, deprivation,
//! disability, and language — exactly the populations carrying the most
//! treatable burden — so the marginal excluded user often has
//! *above-average* potential benefit.
//!
//! ## Example
//!
//! A digital diabetes-prevention programme, reported two ways:
//!
//! ```rust
//! use health_economics::reach_and_equity::{
//!     equity_gap, impact_ratio, population_impact, Stratum,
//! };
//!
//! // Aggregate: reach 12%, effect 0.02 QALYs/participant
//! // → 0.0024 QALYs/eligible person.
//! let aggregate = population_impact(0.12, 0.02);
//! assert!((aggregate - 0.0024).abs() < 1e-9);
//!
//! // Stratified by deprivation quintile:
//! // Q1 (least deprived): reach 22%, effect 0.02  → 0.0044
//! // Q5 (most deprived):  reach 4%,  effect 0.025 → 0.0010
//! let q1 = Stratum { reach: 0.22, effectiveness: 0.02 };
//! let q5 = Stratum { reach: 0.04, effectiveness: 0.025 };
//! assert!((q1.impact() - 0.0044).abs() < 1e-9);
//! assert!((q5.impact() - 0.0010).abs() < 1e-9);
//!
//! // The programme delivers 4.4× more health to the least deprived —
//! // while Q5's per-participant effect is HIGHER (more headroom).
//! let ratio = impact_ratio(q1.impact(), q5.impact()).unwrap();
//! assert!((ratio - 4.4).abs() < 1e-9);
//!
//! // Equity gap = 0.0044 − 0.0010 = 0.0034 per eligible person.
//! let gap = equity_gap(q1.impact(), q5.impact());
//! assert!((gap - 0.0034).abs() < 1e-9);
//!
//! // An assisted-digital arm lifting Q5 reach to 12% triples Q5 impact
//! // (0.12 × 0.025 = 0.0030) — the equity investment IS the efficiency
//! // investment here.
//! let q5_after = population_impact(0.12, 0.025);
//! assert!((q5_after - 0.0030).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Reach is substantially an engineering artifact: device/OS floor
//!   requirements, bandwidth assumptions, language support, accessibility
//!   conformance (WCAG), identity-verification hurdles, and app-store-only
//!   distribution each carve populations out of the denominator — usually
//!   invisibly, because excluded users never appear in analytics.
//! - Measure the *denominator*: instrument the eligible population, not just
//!   users.
//! - Budget performance for old devices and poor connectivity.
//! - Ship assisted-digital paths (phone, SMS, kiosk) as first-class flows
//!   rather than shame channels.
//! - Stratify every dashboard metric by the equity dimensions — an
//!   unstratified average is where inequity hides.
//!
//! ## Pitfalls
//!
//! - **Effectiveness reported on completers, impact claimed on populations**
//!   — the reach terms silently dropped.
//! - **Equity as an afterthought audit** rather than a design input;
//!   retrofitting reach is far costlier than designing for it.
//! - **Maintenance amnesia**: RE-AIM's weakest mHealth dimension — impact
//!   claims beyond the evidence's time horizon.
//! - **Digital-only channel savings** that shift costs onto excluded users
//!   and front-line staff.
//!
//! ## Sources
//!
//! - RE-AIM framework. <https://re-aim.org/>
//! - RE-AIM systematic reviews of mHealth.
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC12358350/>
//! - CDC, PRISM/RE-AIM for equity planning.
//!   <https://www.cdc.gov/pcd/issues/2018/17_0271.htm>
//!
//! Topic doc: health-economics-metrics/topics/reach-and-equity.md

/// One population stratum (e.g. a deprivation quintile): its reach and its per-participant effectiveness.
pub struct Stratum {
    /// Fraction of the stratum's eligible population that participates
    /// (0–1).
    pub reach: f64,
    /// Real-world effect per participant (e.g. QALYs per participant).
    pub effectiveness: f64,
}

impl Stratum {
    /// Population impact for this stratum: reach × effectiveness, per eligible person.
    ///
    /// # Returns
    ///
    /// Impact per eligible person in the stratum's effect units (e.g. QALYs
    /// per eligible person).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::reach_and_equity::Stratum;
    ///
    /// // Q1 (least deprived): reach 22%, effect 0.02 → 0.0044.
    /// let q1 = Stratum { reach: 0.22, effectiveness: 0.02 };
    /// assert!((q1.impact() - 0.0044).abs() < 1e-9);
    /// ```
    pub fn impact(&self) -> f64 {
        self.reach * self.effectiveness
    }
}

/// Reach: participants divided by the eligible population.
///
/// The denominator is the *eligible population*, not registered users —
/// excluded people never appear in analytics, which is exactly why this
/// denominator must be instrumented deliberately.
///
/// # Arguments
///
/// * `participants` — people actually participating.
/// * `eligible_population` — everyone who could benefit (the true
///   denominator).
///
/// # Returns
///
/// `Some(participants / eligible_population)` as a fraction, or `None` when
/// the eligible population is zero (the fraction is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::reach_and_equity::reach;
///
/// // 12,000 participants of 100,000 eligible → reach 12%.
/// assert_eq!(reach(12_000.0, 100_000.0), Some(0.12));
///
/// // Zero eligible population: undefined.
/// assert_eq!(reach(10.0, 0.0), None);
/// ```
pub fn reach(participants: f64, eligible_population: f64) -> Option<f64> {
    if eligible_population == 0.0 {
        None
    } else {
        Some(participants / eligible_population)
    }
}

/// Population impact per eligible person: reach × effectiveness.
///
/// The RE-AIM central arithmetic. Both reach and effectiveness must refer to
/// the same population stratum.
///
/// # Arguments
///
/// * `reach` — participation fraction (0–1) of the eligible population.
/// * `effectiveness` — real-world effect per participant (e.g. QALYs).
///
/// # Returns
///
/// Impact per eligible person, in the effectiveness units.
///
/// # Examples
///
/// ```rust
/// use health_economics::reach_and_equity::population_impact;
///
/// // Aggregate: reach 12%, effect 0.02 QALYs/participant
/// // → 0.0024 QALYs/eligible person.
/// assert!((population_impact(0.12, 0.02) - 0.0024).abs() < 1e-9);
/// ```
pub fn population_impact(reach: f64, effectiveness: f64) -> f64 {
    reach * effectiveness
}

/// Equity gap: impact in the top (least deprived) group minus impact in the bottom (most deprived) group.
///
/// Positive means the programme delivers more health to the better-off — the
/// usual digital-divide signature.
///
/// # Arguments
///
/// * `impact_top_group` — impact per eligible person in the least deprived
///   stratum.
/// * `impact_bottom_group` — impact per eligible person in the most deprived
///   stratum.
///
/// # Returns
///
/// The absolute impact difference, in the impact units.
///
/// # Examples
///
/// ```rust
/// use health_economics::reach_and_equity::equity_gap;
///
/// // Q1 0.0044 vs Q5 0.0010 → gap 0.0034 QALYs per eligible person.
/// assert!((equity_gap(0.0044, 0.0010) - 0.0034).abs() < 1e-9);
/// ```
pub fn equity_gap(impact_top_group: f64, impact_bottom_group: f64) -> f64 {
    impact_top_group - impact_bottom_group
}

/// How many times more health the top group receives than the bottom group.
///
/// The relative counterpart to [`equity_gap`].
///
/// # Arguments
///
/// * `impact_top_group` — impact per eligible person in the least deprived
///   stratum.
/// * `impact_bottom_group` — impact per eligible person in the most deprived
///   stratum.
///
/// # Returns
///
/// `Some(top / bottom)`, or `None` when the bottom group's impact is zero
/// (the ratio is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::reach_and_equity::impact_ratio;
///
/// // The programme delivers 4.4× more health to the least deprived.
/// assert_eq!(impact_ratio(0.0044, 0.0010), Some(4.4));
///
/// // Zero bottom-group impact: undefined.
/// assert_eq!(impact_ratio(0.0044, 0.0), None);
/// ```
pub fn impact_ratio(impact_top_group: f64, impact_bottom_group: f64) -> Option<f64> {
    if impact_bottom_group == 0.0 {
        None
    } else {
        Some(impact_top_group / impact_bottom_group)
    }
}

/// Distributional cost-effectiveness: apply an equity weight to QALYs by recipient group.
///
/// A QALY to the worst-off counts more — an increasingly mainstream HTA
/// extension. Weights > 1 upweight disadvantaged recipients; 1.0 is the
/// unweighted baseline.
///
/// # Arguments
///
/// * `qalys` — unweighted QALYs delivered to the group.
/// * `equity_weight` — the group's distributional weight (e.g. `1.5` for the
///   most deprived quintile).
///
/// # Returns
///
/// Equity-weighted QALYs: `qalys × equity_weight`.
///
/// # Examples
///
/// ```rust
/// use health_economics::reach_and_equity::equity_weighted_qalys;
///
/// // 10 QALYs to the worst-off at weight 1.5 count as 15.
/// assert_eq!(equity_weighted_qalys(10.0, 1.5), 15.0);
/// ```
pub fn equity_weighted_qalys(qalys: f64, equity_weight: f64) -> f64 {
    qalys * equity_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Aggregate: reach 12%, effect 0.02 QALYs/participant → 0.0024
    /// QALYs/eligible person.
    #[test]
    fn aggregate_impact_is_0_0024() {
        // Worked example: "Aggregate: reach 12%, effect 0.02
        // QALYs/participant → 0.0024 QALYs/eligible person."
        let i = population_impact(0.12, 0.02);
        assert!((i - 0.0024).abs() < TOL);
    }

    /// Q1 (least deprived): reach 22%, effect 0.02 → 0.0044.
    #[test]
    fn least_deprived_quintile_impact_is_0_0044() {
        // Worked example: "Q1 (least deprived): reach 22%, effect 0.02 → 0.0044."
        let q1 = Stratum { reach: 0.22, effectiveness: 0.02 };
        assert!((q1.impact() - 0.0044).abs() < TOL);
    }

    /// Q5 (most deprived): reach 4%, effect 0.025 → 0.0010.
    #[test]
    fn most_deprived_quintile_impact_is_0_0010() {
        // Worked example: "Q5 (most deprived): reach 4%, effect 0.025 → 0.0010."
        let q5 = Stratum { reach: 0.04, effectiveness: 0.025 };
        assert!((q5.impact() - 0.0010).abs() < TOL);
    }

    /// The programme delivers 4.4× more health to the least deprived.
    #[test]
    fn programme_delivers_4_4_times_more_to_least_deprived() {
        // Worked example: "The programme delivers 4.4× more health to the
        // least deprived."
        let r = impact_ratio(0.0044, 0.0010).unwrap();
        assert!((r - 4.4).abs() < TOL);
    }

    /// Equity gap = 0.0044 − 0.0010 = 0.0034 per eligible person.
    #[test]
    fn equity_gap_is_0_0034() {
        // Doc math: "equity gap = impact_top quintile − impact_bottom quintile."
        let g = equity_gap(0.0044, 0.0010);
        assert!((g - 0.0034).abs() < TOL);
    }

    /// An assisted-digital arm lifting Q5 reach to 12% triples Q5 impact
    /// (0.12 × 0.025 = 0.0030 vs 0.0010) and improves the aggregate.
    #[test]
    fn assisted_digital_arm_triples_q5_impact() {
        // Worked example: "an assisted-digital arm ... that lifts Q5 reach to
        // 12% triples Q5 impact and improves the aggregate."
        let before = population_impact(0.04, 0.025);
        let after = population_impact(0.12, 0.025);
        assert!((after - 0.0030).abs() < TOL);
        let ratio = impact_ratio(after, before).unwrap();
        assert!((ratio - 3.0).abs() < TOL);
        assert!(after > before);
    }

    /// Reach is participants over the eligible population, not over users.
    #[test]
    fn reach_is_participants_over_eligible() {
        // Doc math: "reach = participants / eligible population."
        let r = reach(12_000.0, 100_000.0).unwrap();
        assert!((r - 0.12).abs() < TOL);
    }

    /// A QALY to the worst-off counts more under distributional weighting.
    #[test]
    fn equity_weight_scales_qalys() {
        // Doc math: "apply equity weights to QALYs by recipient group — a
        // QALY to the worst-off counts more."
        assert!((equity_weighted_qalys(10.0, 1.5) - 15.0).abs() < TOL);
    }

    // Edge case: a zero eligible population leaves reach undefined.
    #[test]
    fn zero_eligible_population_is_undefined() {
        assert!(reach(10.0, 0.0).is_none());
    }

    // Edge case: zero bottom-group impact leaves the ratio undefined.
    #[test]
    fn zero_bottom_impact_has_no_defined_ratio() {
        assert!(impact_ratio(0.0044, 0.0).is_none());
    }
}
