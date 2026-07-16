//! # Analysis Perspective
//!
//! Perspective defines *whose* costs and benefits count in an economic
//! analysis: the payer's, the provider's, or society's as a whole. The same
//! intervention can look brilliant from one perspective and terrible from
//! another, so every economic evaluation must declare its perspective up
//! front — the perspective determines which line items exist.
//!
//! NICE's reference case uses the NHS and Personal Social Services (PSS)
//! perspective for costs; the US Second Panel on Cost-Effectiveness
//! recommends reporting both a healthcare-sector and a societal analysis
//! with an "impact inventory" listing what's included.
//!
//! ## Formula
//!
//! There is no formula — a scoping rule applied before any math:
//!
//! ```text
//! Included cost/benefit categories = f(perspective)
//! ```
//!
//! Legend:
//! - `perspective` — payer (only costs the payer reimburses), provider
//!   (internal delivery costs, staffing, estates), or societal (everything —
//!   patient time, travel, informal care, productivity losses).
//! - The check is an impact inventory: one row per cost/benefit, one column
//!   per perspective, marking which cells count.
//!
//! ## Why it matters
//!
//! The declaration of perspective is what makes numbers comparable and
//! honest. The same symptom-checker app that saves an NHS payer £420,000/year
//! nets only £120,000/year from the societal perspective once patients' time
//! savings (£300,000) and false-reassurance harm (£600,000) are counted —
//! same app, three different answers, and the societal result is dominated by
//! the safety assumption. Counting societal benefits against payer-only costs
//! makes anything look cost-effective, which is why reviewers demand the
//! perspective be stated.
//!
//! ## Example
//!
//! A symptom-checker app diverts 10,000 GP visits per year to self-care.
//! Payer (NHS): 10,000 × £42 = £420,000/year saved. Societal: add 10,000 ×
//! 2 hours × £15/hour = £300,000 of patient time value, subtract harm if 2%
//! are falsely reassured and present later, sicker (200 × £3,000 = £600,000):
//! net £120,000/year.
//!
//! ```rust
//! use health_economics::analysis_perspective::{
//!     false_reassurance_harm, net_value_from_perspective, patient_time_value,
//!     payer_savings, ImpactItem, Perspective,
//! };
//!
//! let items = vec![
//!     // Saved GP consultations: 10,000 × £42 = £420,000.
//!     ImpactItem {
//!         amount: payer_savings(10_000.0, 42.0),
//!         counts_for_payer: true,
//!         counts_for_provider: false,
//!         counts_for_societal: true,
//!     },
//!     // Patients' saved travel and waiting time: 10,000 × 2h × £15 = £300,000.
//!     ImpactItem {
//!         amount: patient_time_value(10_000.0, 2.0, 15.0),
//!         counts_for_payer: false,
//!         counts_for_provider: false,
//!         counts_for_societal: true,
//!     },
//!     // Harm: 2% falsely reassured, 200 × £3,000 = £600,000 of extra treatment.
//!     ImpactItem {
//!         amount: -false_reassurance_harm(10_000.0, 0.02, 3_000.0),
//!         counts_for_payer: false,
//!         counts_for_provider: false,
//!         counts_for_societal: true,
//!     },
//! ];
//!
//! // Payer: £420,000/year — strongly positive.
//! let payer = net_value_from_perspective(&items, Perspective::Payer);
//! assert!((payer - 420_000.0).abs() < 1e-9);
//!
//! // Societal: 420,000 + 300,000 − 600,000 = £120,000/year.
//! let societal = net_value_from_perspective(&items, Perspective::Societal);
//! assert!((societal - 120_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! Tool and platform ROI has perspectives too:
//!
//! - **Team budget ("payer")**: does the license fee fit my cost center?
//! - **Platform org ("provider")**: total cost including integration,
//!   support, and maintenance.
//! - **Company ("societal")**: include customer impact, security
//!   externalities, and the time of every team affected.
//! - A CI tool that is cheap for the buying team but pushes migration work
//!   onto 40 other teams is the software version of cost-shifting — visible
//!   only from the wider perspective.
//! - State the perspective in every business case; reviewers can't challenge
//!   assumptions they can't see.
//!
//! ## Pitfalls
//!
//! - **Silent perspective switching**: counting societal benefits but only
//!   payer costs makes anything look cost-effective.
//! - **Double counting** when perspectives are merged (e.g., counting a saved
//!   GP appointment as both payer savings and patient time savings when the
//!   payer figure already includes staff time).
//! - **Ignoring cost-shifting**: "savings" that just move cost to patients,
//!   carers, or another department.
//!
//! ## Sources
//!
//! - Sanders GD, et al. "Recommendations for Conduct, Methodological
//!   Practices, and Reporting of Cost-effectiveness Analyses: Second Panel on
//!   Cost-Effectiveness in Health and Medicine." JAMA 2016.
//!   <https://jamanetwork.com/journals/jama/fullarticle/2552214>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/analysis-perspective.md

/// Whose costs and benefits count in the analysis.
///
/// Declare the perspective up front — it determines which line items exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perspective {
    /// Payer (e.g. NHS commissioner, insurer): only costs the payer
    /// reimburses.
    Payer,
    /// Provider (e.g. a hospital trust): internal delivery costs, staffing,
    /// estates.
    Provider,
    /// Societal: everything — patient time, travel, informal care by family,
    /// and productivity losses to employers.
    Societal,
}

/// One row of an impact inventory.
///
/// A signed amount (positive = benefit, negative = cost) plus flags for the
/// perspectives from which it counts — the row-by-column inventory table the
/// US Second Panel recommends, in struct form. Marking a cell in only one
/// perspective is how double counting is avoided when perspectives merge.
pub struct ImpactItem {
    /// Signed value of the item (currency units; positive benefit, negative
    /// cost).
    pub amount: f64,
    /// Whether the item counts from the payer perspective.
    pub counts_for_payer: bool,
    /// Whether the item counts from the provider perspective.
    pub counts_for_provider: bool,
    /// Whether the item counts from the societal perspective.
    pub counts_for_societal: bool,
}

impl ImpactItem {
    /// Whether this item is included under the given perspective.
    ///
    /// # Arguments
    ///
    /// * `perspective` — the declared analysis perspective.
    ///
    /// # Returns
    ///
    /// `true` if this row's cell is marked for that perspective.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::analysis_perspective::{ImpactItem, Perspective};
    ///
    /// // Patient time savings count only from the societal perspective.
    /// let patient_time = ImpactItem {
    ///     amount: 300_000.0,
    ///     counts_for_payer: false,
    ///     counts_for_provider: false,
    ///     counts_for_societal: true,
    /// };
    /// assert!(!patient_time.included_in(Perspective::Payer));
    /// assert!(patient_time.included_in(Perspective::Societal));
    /// ```
    pub fn included_in(&self, perspective: Perspective) -> bool {
        match perspective {
            Perspective::Payer => self.counts_for_payer,
            Perspective::Provider => self.counts_for_provider,
            Perspective::Societal => self.counts_for_societal,
        }
    }
}

/// Net value of an impact inventory from one declared perspective.
///
/// The sum of every item's signed amount that counts from that perspective —
/// the scoping rule `included categories = f(perspective)` applied, then
/// summed.
///
/// # Arguments
///
/// * `items` — the impact inventory rows (see [`ImpactItem`]).
/// * `perspective` — the declared analysis perspective.
///
/// # Returns
///
/// The net value (currency units); 0.0 if no items count from that
/// perspective.
///
/// # Examples
///
/// ```rust
/// use health_economics::analysis_perspective::{
///     net_value_from_perspective, ImpactItem, Perspective,
/// };
///
/// // Payer sees only the £420,000 of saved consultations; societal also
/// // sees patient time (+£300,000) and harm (−£600,000).
/// let items = vec![
///     ImpactItem { amount: 420_000.0, counts_for_payer: true, counts_for_provider: false, counts_for_societal: true },
///     ImpactItem { amount: 300_000.0, counts_for_payer: false, counts_for_provider: false, counts_for_societal: true },
///     ImpactItem { amount: -600_000.0, counts_for_payer: false, counts_for_provider: false, counts_for_societal: true },
/// ];
/// assert!((net_value_from_perspective(&items, Perspective::Payer) - 420_000.0).abs() < 1e-9);
/// assert!((net_value_from_perspective(&items, Perspective::Societal) - 120_000.0).abs() < 1e-9);
/// ```
pub fn net_value_from_perspective(items: &[ImpactItem], perspective: Perspective) -> f64 {
    items
        .iter()
        .filter(|item| item.included_in(perspective))
        .map(|item| item.amount)
        .sum()
}

/// Payer savings from diverted visits: visits diverted × cost per visit.
///
/// # Arguments
///
/// * `visits_diverted` — visits diverted to self-care per period (count).
/// * `cost_per_visit` — what the payer reimburses per visit (currency), e.g.
///   £42 per GP consultation.
///
/// # Returns
///
/// The payer saving per period (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::analysis_perspective::payer_savings;
///
/// // Worked example: 10,000 × £42 per GP consultation = £420,000/year.
/// let savings = payer_savings(10_000.0, 42.0);
/// assert!((savings - 420_000.0).abs() < 1e-9);
/// ```
pub fn payer_savings(visits_diverted: f64, cost_per_visit: f64) -> f64 {
    visits_diverted * cost_per_visit
}

/// Value of patients' saved travel and waiting time.
///
/// visits × hours saved per visit × value per hour. Counts only from the
/// societal perspective — the payer never reimburses patient time.
///
/// # Arguments
///
/// * `visits` — avoided visits (count).
/// * `hours_per_visit` — travel and waiting hours saved per visit (hours).
/// * `value_per_hour` — monetary value of an hour of patient time
///   (currency/hour).
///
/// # Returns
///
/// The patient time value (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::analysis_perspective::patient_time_value;
///
/// // Worked example: 10,000 × 2 hours × £15/hour = £300,000 of time value.
/// let value = patient_time_value(10_000.0, 2.0, 15.0);
/// assert!((value - 300_000.0).abs() < 1e-9);
/// ```
pub fn patient_time_value(visits: f64, hours_per_visit: f64, value_per_hour: f64) -> f64 {
    visits * hours_per_visit * value_per_hour
}

/// Harm cost from false reassurance.
///
/// visits × rate falsely reassured × extra treatment cost per late, sicker
/// presentation. The worked example's societal result is dominated by this
/// safety assumption.
///
/// # Arguments
///
/// * `visits` — diverted visits exposed to the risk (count).
/// * `false_reassurance_rate` — fraction falsely reassured who present
///   later, sicker (0.0–1.0).
/// * `extra_treatment_cost_per_case` — extra treatment cost per such case
///   (currency).
///
/// # Returns
///
/// The harm cost (currency units, positive; negate it when entering it as an
/// [`ImpactItem`] amount).
///
/// # Examples
///
/// ```rust
/// use health_economics::analysis_perspective::false_reassurance_harm;
///
/// // Worked example: 2% of 10,000 = 200 cases × £3,000 = £600,000.
/// let harm = false_reassurance_harm(10_000.0, 0.02, 3_000.0);
/// assert!((harm - 600_000.0).abs() < 1e-9);
/// ```
pub fn false_reassurance_harm(
    visits: f64,
    false_reassurance_rate: f64,
    extra_treatment_cost_per_case: f64,
) -> f64 {
    visits * false_reassurance_rate * extra_treatment_cost_per_case
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a symptom-checker app diverts 10,000 GP visits per year
    // to self-care.

    fn worked_example_inventory() -> Vec<ImpactItem> {
        vec![
            // Saved GP consultations: counts for the payer and society (the
            // payer figure already includes staff time — no double counting).
            ImpactItem {
                amount: payer_savings(10_000.0, 42.0),
                counts_for_payer: true,
                counts_for_provider: false,
                counts_for_societal: true,
            },
            // Patients' saved travel and waiting time: societal only.
            ImpactItem {
                amount: patient_time_value(10_000.0, 2.0, 15.0),
                counts_for_payer: false,
                counts_for_provider: false,
                counts_for_societal: true,
            },
            // Harm if 2% are falsely reassured and present later, sicker.
            ImpactItem {
                amount: -false_reassurance_harm(10_000.0, 0.02, 3_000.0),
                counts_for_payer: false,
                counts_for_provider: false,
                counts_for_societal: true,
            },
        ]
    }

    #[test]
    fn payer_savings_are_420_000_per_year() {
        // 10,000 × £42 per GP consultation = £420,000/year.
        let got = payer_savings(10_000.0, 42.0);
        assert!((got - 420_000.0).abs() < 1e-9);
    }

    #[test]
    fn patient_time_value_is_300_000() {
        // 10,000 × 2 hours × £15/hour = £300,000 of time value.
        let got = patient_time_value(10_000.0, 2.0, 15.0);
        assert!((got - 300_000.0).abs() < 1e-9);
    }

    #[test]
    fn false_reassurance_harm_is_600_000() {
        // 2% of 10,000 = 200 cases × £3,000 = £600,000 of extra treatment.
        let got = false_reassurance_harm(10_000.0, 0.02, 3_000.0);
        assert!((got - 600_000.0).abs() < 1e-9);
    }

    #[test]
    fn payer_perspective_net_is_420_000() {
        // Doc: "Payer (NHS): ... £420,000/year — strongly positive."
        let net = net_value_from_perspective(&worked_example_inventory(), Perspective::Payer);
        assert!((net - 420_000.0).abs() < 1e-9);
    }

    #[test]
    fn societal_perspective_net_is_120_000() {
        // 420,000 + 300,000 − 600,000 = £120,000/year.
        let net = net_value_from_perspective(&worked_example_inventory(), Perspective::Societal);
        assert!((net - 120_000.0).abs() < 1e-9);
    }

    #[test]
    fn provider_perspective_sees_none_of_these_lines() {
        // Capitation-paid practices: income unchanged, workload falls — none
        // of the inventory's cash lines land on the provider.
        let net = net_value_from_perspective(&worked_example_inventory(), Perspective::Provider);
        assert!((net - 0.0).abs() < 1e-9);
    }

    #[test]
    fn same_app_three_different_answers() {
        // Doc: "Same app, three different answers."
        let items = worked_example_inventory();
        let payer = net_value_from_perspective(&items, Perspective::Payer);
        let provider = net_value_from_perspective(&items, Perspective::Provider);
        let societal = net_value_from_perspective(&items, Perspective::Societal);
        assert!(payer != provider && provider != societal && payer != societal);
    }
}
