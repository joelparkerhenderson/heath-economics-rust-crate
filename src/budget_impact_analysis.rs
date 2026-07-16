//! # Budget Impact Analysis (BIA)
//!
//! BIA estimates what adopting an intervention does to a specific payer's
//! **budget** over the next 1–5 years. It answers *affordability*;
//! cost-effectiveness answers *value*. A technology can be excellent value
//! and still unaffordable — or affordable and poor value. Serious appraisals
//! require both.
//!
//! Per ISPOR good-practice guidance, BIA takes the payer's own perspective,
//! a 1–5 year horizon, *undiscounted* annual cash flows, realistic uptake
//! curves, and scenario (not probabilistic) uncertainty.
//!
//! ## Formula
//!
//! ```text
//! BI_year_t = Cost_scenario_with_new(t) − Cost_scenario_current(t)
//!
//! Cost_scenario(t) = Σ over patient groups:
//!    eligible population(t) × uptake(t) × net cost per patient(t)
//!
//! net cost per patient = intervention cost − displaced care cost + induced care cost
//!
//! BI_year_t             — budget impact in year t (payer currency, undiscounted)
//! eligible population   — payer members eligible for the intervention in year t
//! uptake                — fraction of the eligible population using it in year t (0–1)
//! displaced care cost   — other care the intervention replaces, per patient per year
//! induced care cost     — extra demand the intervention creates, per patient per year
//! ```
//!
//! ## Why it matters
//!
//! The finance director's question is never "what's the ICER?" — it's "what
//! does this do to next year's budget?" ISPOR good-practice guidance (the
//! field standard) specifies the payer's own perspective, a 1–5 year
//! horizon, undiscounted annual cash flows, realistic uptake curves, and
//! scenario uncertainty. NICE requires budget-impact information alongside
//! cost-effectiveness; a product with national budget impact above
//! ~£20M/year in England triggers commercial negotiation regardless of its
//! ICER.
//!
//! ## Example
//!
//! A payer covering 2M people considers a digital therapeutic at
//! £300/patient/year; 1.5% of members eligible (30,000); uptake
//! 20% → 40% → 60% over 3 years; each user displaces £120/year of other
//! care. Net cost per user is £180; budget impact grows £1.08M → £2.16M →
//! £3.24M.
//!
//! ```rust
//! use health_economics::budget_impact_analysis::{
//!     PatientGroup, budget_impact, net_cost_per_patient, scenario_cost,
//! };
//!
//! // Net cost per user = 300 − 120 = £180.
//! let net = net_cost_per_patient(300.0, 120.0, 0.0);
//! assert!((net - 180.0).abs() < 1e-9);
//!
//! // Uptake ramps 20% → 40% → 60% over 3 years; eligible = 2M × 1.5% = 30,000.
//! let year = |uptake: f64| PatientGroup {
//!     eligible_population: 30_000.0,
//!     uptake,
//!     net_cost_per_patient: net,
//! };
//!
//! // Year 1: 30,000 × 0.20 × 180 = £1.08M.
//! let bi_1 = budget_impact(scenario_cost(&[year(0.20)]), 0.0);
//! assert!((bi_1 - 1_080_000.0).abs() < 1e-6);
//!
//! // Year 2: 30,000 × 0.40 × 180 = £2.16M.
//! assert!((year(0.40).cost() - 2_160_000.0).abs() < 1e-6);
//!
//! // Year 3: 30,000 × 0.60 × 180 = £3.24M of new money the payer must find.
//! assert!((year(0.60).cost() - 3_240_000.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - BIA is the CFO-facing complement to a per-seat ROI claim: "it's
//!   cost-effective per developer, but can we afford org-wide rollout this
//!   fiscal year?"
//! - Model license tiers and an adoption S-curve — adoption is never
//!   instant.
//! - Displaced tooling spend only cash-releases when old contracts actually
//!   terminate.
//! - Model induced usage: cheaper CI → more CI.
//! - Presenting a 3-year budget-impact table alongside the ROI is what makes
//!   an enterprise tooling proposal finance-credible.
//!
//! ## Pitfalls
//!
//! - **Instant-uptake fantasy**: year-1 impact computed at steady-state
//!   adoption.
//! - **Counting displaced cost as cash** when it's diffuse capacity.
//! - **Ignoring induced demand** — access improvements grow the eligible
//!   population's usage.
//! - **Confusing BIA and CEA horizons/discounting**: BIA is short-horizon,
//!   undiscounted, payer-specific by design.
//!
//! ## Sources
//!
//! - Sullivan SD, et al. ISPOR BIA Good Practice II Task Force. Value in
//!   Health 2014;17(1):5–14. <https://pubmed.ncbi.nlm.nih.gov/24438712/>
//! - ISPOR good practices: budget impact analysis.
//!   <https://www.ispor.org/heor-resources/good-practices/article/principles-of-good-practice-for-budget-impact-analysis-ii>
//!
//! Topic doc: health-economics-metrics/topics/budget-impact-analysis.md

/// Net cost per patient = intervention cost − displaced care cost + induced care cost.
///
/// All three arguments are per patient per year, in the payer's currency.
/// The displaced term enters negatively (care the intervention replaces);
/// the induced term positively (extra demand it creates).
///
/// # Arguments
///
/// * `intervention_cost` — price of the intervention per patient per year.
/// * `displaced_care_cost` — other care replaced, per patient per year.
/// * `induced_care_cost` — extra care demand created, per patient per year.
///
/// # Returns
///
/// The net cost per patient per year (may be negative if displacement
/// exceeds the intervention's price).
///
/// # Examples
///
/// ```rust
/// use health_economics::budget_impact_analysis::net_cost_per_patient;
///
/// // £300 digital therapeutic displacing £120 of other care → £180 net.
/// assert!((net_cost_per_patient(300.0, 120.0, 0.0) - 180.0).abs() < 1e-9);
///
/// // Induced demand raises the net cost: 300 − 120 + 40 = £220.
/// assert!((net_cost_per_patient(300.0, 120.0, 40.0) - 220.0).abs() < 1e-9);
/// ```
pub fn net_cost_per_patient(
    intervention_cost: f64,
    displaced_care_cost: f64,
    induced_care_cost: f64,
) -> f64 {
    intervention_cost - displaced_care_cost + induced_care_cost
}

/// One patient group in a budget-impact scenario for a single year.
///
/// A scenario for year t is the sum over such groups; each group carries its
/// own eligible-population size, uptake fraction, and net cost per patient
/// for that year.
#[derive(Debug, Clone, Copy)]
pub struct PatientGroup {
    /// Number of payer members eligible for the intervention this year.
    pub eligible_population: f64,
    /// Fraction of the eligible population actually using it this year (0–1).
    pub uptake: f64,
    /// Net cost per patient this year (intervention − displaced + induced).
    pub net_cost_per_patient: f64,
}

impl PatientGroup {
    /// This group's contribution to the scenario cost:
    /// eligible population × uptake × net cost per patient.
    ///
    /// Returned in the payer's currency, undiscounted (BIA convention).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::budget_impact_analysis::PatientGroup;
    ///
    /// // Year 1 of the worked example: 30,000 × 0.20 × £180 = £1.08M.
    /// let group = PatientGroup {
    ///     eligible_population: 30_000.0,
    ///     uptake: 0.20,
    ///     net_cost_per_patient: 180.0,
    /// };
    /// assert!((group.cost() - 1_080_000.0).abs() < 1e-6);
    /// ```
    pub fn cost(&self) -> f64 {
        self.eligible_population * self.uptake * self.net_cost_per_patient
    }
}

/// Total scenario cost for one year: sum of every patient group's cost.
///
/// # Arguments
///
/// * `groups` — the patient groups making up the scenario for this year.
///
/// # Returns
///
/// The year's scenario cost in the payer's currency (0.0 for an empty
/// slice), undiscounted.
///
/// # Examples
///
/// ```rust
/// use health_economics::budget_impact_analysis::{PatientGroup, scenario_cost};
///
/// // Single-group scenario, year 3: 30,000 × 0.60 × £180 = £3.24M.
/// let groups = [PatientGroup {
///     eligible_population: 30_000.0,
///     uptake: 0.60,
///     net_cost_per_patient: 180.0,
/// }];
/// assert!((scenario_cost(&groups) - 3_240_000.0).abs() < 1e-6);
/// ```
pub fn scenario_cost(groups: &[PatientGroup]) -> f64 {
    groups.iter().map(PatientGroup::cost).sum()
}

/// Budget impact for year t: cost of the scenario with the new intervention
/// minus cost of the current (comparator) scenario.
///
/// Undiscounted by design — BIA reports annual cash flows as the payer will
/// experience them. Positive means new money the payer must find.
///
/// # Arguments
///
/// * `cost_scenario_with_new` — year-t cost of the scenario including the
///   new intervention.
/// * `cost_scenario_current` — year-t cost of the current-care scenario.
///
/// # Returns
///
/// The year's budget impact in the payer's currency.
///
/// # Examples
///
/// ```rust
/// use health_economics::budget_impact_analysis::budget_impact;
///
/// // Year 1: the new scenario costs £1.08M and displaces nothing already
/// // in the current scenario's books → BI = £1.08M.
/// assert!((budget_impact(1_080_000.0, 0.0) - 1_080_000.0).abs() < 1e-6);
/// ```
pub fn budget_impact(cost_scenario_with_new: f64, cost_scenario_current: f64) -> f64 {
    cost_scenario_with_new - cost_scenario_current
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: 2M members, 1.5% eligible (30,000); £300/patient/year
    // digital therapeutic displacing £120/year of other care; uptake
    // 20% → 40% → 60% over 3 years.

    // Worked-example setup: "2M people ... 1.5% of members eligible (30,000)".
    #[test]
    fn eligible_population_is_30_000() {
        let eligible: f64 = 2_000_000.0 * 0.015;
        assert!((eligible - 30_000.0).abs() < 1e-9);
    }

    // Worked-example line: "Net cost per user = 300 − 120 = £180".
    #[test]
    fn net_cost_per_user_is_180() {
        let net = net_cost_per_patient(300.0, 120.0, 0.0);
        assert!((net - 180.0).abs() < 1e-9);
    }

    // Worked-example line: "Year 1: 30,000 × 0.20 × 180 = £1.08M".
    #[test]
    fn year_1_budget_impact_is_1_08_million() {
        let group = PatientGroup {
            eligible_population: 30_000.0,
            uptake: 0.20,
            net_cost_per_patient: 180.0,
        };
        // Current scenario carries no cost for the new intervention.
        let bi = budget_impact(scenario_cost(&[group]), 0.0);
        assert!((bi - 1_080_000.0).abs() < 1e-6);
    }

    // Worked-example line: "Year 2: 30,000 × 0.40 × 180 = £2.16M".
    #[test]
    fn year_2_budget_impact_is_2_16_million() {
        let group = PatientGroup {
            eligible_population: 30_000.0,
            uptake: 0.40,
            net_cost_per_patient: 180.0,
        };
        assert!((group.cost() - 2_160_000.0).abs() < 1e-6);
    }

    // Worked-example line: "Year 3: 30,000 × 0.60 × 180 = £3.24M".
    #[test]
    fn year_3_budget_impact_is_3_24_million() {
        let group = PatientGroup {
            eligible_population: 30_000.0,
            uptake: 0.60,
            net_cost_per_patient: 180.0,
        };
        assert!((group.cost() - 3_240_000.0).abs() < 1e-6);
    }

    // Formula term check: "+ induced care cost" enters with a positive sign
    // (the doc's pitfall "Ignoring induced demand").
    #[test]
    fn induced_care_raises_net_cost() {
        // Induced demand term enters with a positive sign.
        let net = net_cost_per_patient(300.0, 120.0, 40.0);
        assert!((net - 220.0).abs() < 1e-9);
    }
}
