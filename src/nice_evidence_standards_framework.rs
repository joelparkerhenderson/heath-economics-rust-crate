//! # NICE Evidence Standards Framework (ESF)
//!
//! The ESF is NICE's framework specifying **how much evidence a digital
//! health technology needs, proportionate to its risk**. It is the closest
//! thing to an official answer to "what do we have to prove before the NHS
//! buys our app?"
//!
//! First released 2019, updated 2022 to cover AI and adaptive algorithms, the
//! ESF classifies digital health technologies into tiers by clinical
//! function, with **cumulative** evidence standards — 21 standards across 5
//! groups (design factors, value, performance/effectiveness, economic impact,
//! deployment). For economic evidence, cost-consequence analysis is
//! acceptable for most tiers; cost-utility analysis is expected at the
//! highest risk. The operative calculation is commercial: the ESF defines
//! your evidence cost of market entry — budget for it like any other build
//! cost.
//!
//! ## Formula
//!
//! ```text
//! Evidence investment required = f(tier)
//!   Tier A: documentation + assurance                    ≈ £10k–£50k
//!   Tier B: observational/comparative user-benefit study ≈ £50k–£250k
//!   Tier C: RCT-grade comparative study + economic model ≈ £250k–£2M+
//!
//! Years to break even = evidence cost / incremental annual revenue
//!
//! tier                        = ESF tier from the product's clinical function
//! evidence cost               = the study/assurance investment for the tier
//! incremental annual revenue  = extra revenue the tier-moving claim earns
//! ```
//!
//! Position your product's claims deliberately: claiming "supports clinical
//! decisions" instead of "informs patients" moves you a tier and can 10× the
//! bill.
//!
//! ## Why it matters
//!
//! The ESF prices market entry by risk. Tier A (system services, no direct
//! patient outcome) needs basic standards; Tier B (inform, simple monitoring)
//! adds evidence of user benefit; Tier C (treat, diagnose, guide clinical
//! management) adds high-quality comparative effectiveness evidence — ideally
//! an RCT — plus economic analysis. Because the tiers differ by an order of
//! magnitude in evidence cost (£10k–£50k vs £50k–£250k vs £250k–£2M+),
//! tier classification is a product decision with a price tag.
//!
//! ## Example
//!
//! A medication-reminder app maker considers adding a dose-adjustment
//! recommendation feature: as a reminder app it is Tier B; with dose
//! recommendations it becomes Tier C.
//!
//! ```rust
//! use health_economics::nice_evidence_standards_framework::{
//!     classify_tier, evidence_investment_range, years_to_recoup_evidence_cost,
//!     ClinicalFunction, EsfTier,
//! };
//!
//! // As a reminder app: Tier B — a cohort study showing adherence suffices.
//! assert_eq!(classify_tier(ClinicalFunction::InformOrMonitor), EsfTier::B);
//! // With dose recommendations: Tier C — RCT-grade evidence + economics.
//! assert_eq!(classify_tier(ClinicalFunction::TreatDiagnoseOrGuide), EsfTier::C);
//!
//! // A £600k RCT sits inside Tier C's £250k–£2M indicative range.
//! let (low, high) = evidence_investment_range(EsfTier::C);
//! assert!(low <= 600_000.0 && 600_000.0 <= high);
//!
//! // At £200k/year incremental revenue, the feature must hold value 3+ years.
//! let years = years_to_recoup_evidence_cost(600_000.0, 200_000.0).unwrap();
//! assert!((years - 3.0).abs() < 1e-9);
//! ```
//!
//! Many teams ship the Tier B product and stage the Tier C claim behind
//! funding.
//!
//! ## Software engineering connection
//!
//! - The ESF is the single most transferable governance pattern here:
//!   **risk-tiered evidence requirements for tool adoption**.
//! - Internal version: a code formatter needs a demo (Tier A); a productivity
//!   tool claiming hours saved needs a measured pilot (Tier B); an AI gate
//!   that auto-blocks deploys or auto-writes clinical code needs
//!   controlled-trial-grade evidence before org-wide rollout (Tier C).
//! - Proportionate evidence stops both failure modes — bureaucracy strangling
//!   trivial tools, and vibes shipping consequential ones.
//! - See DiGA fast-track for the "provisional adoption with evidence
//!   deadline" complement.
//!
//! ## Pitfalls
//!
//! - **Tier misclassification by wishful thinking** — regulators and buyers
//!   classify by what the product *does*, not what the marketing says.
//! - **Evidence built after the product**: retrofitting an RCT onto a shipped
//!   product without instrumentation or equipoise is slow and often
//!   impossible.
//! - **Meeting the ESF and forgetting the rest**: ESF sits alongside DTAC
//!   (clinical safety, data protection, interoperability) and, for AI,
//!   regulatory clearance.
//!
//! ## Sources
//!
//! - NICE Evidence Standards Framework (ECD7).
//!   <https://www.nice.org.uk/corporate/ecd7>
//! - ESF evidence standards tables.
//!   <https://www.nice.org.uk/corporate/ecd7/chapter/section-c-evidence-standards-tables>
//!
//! Topic doc: health-economics-metrics/topics/nice-evidence-standards-framework.md

/// ESF evidence tier, classified by what the product *does* (not what the marketing says).
///
/// Standards are cumulative: each tier adds to the ones below. The derived
/// `Ord` reflects the escalation (`A < B < C`).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EsfTier {
    /// System services with no direct patient outcome (e.g. e-rostering).
    /// Evidence expectation: basic standards — credibility with UK health
    /// professionals, data protection, technical assurance. ≈ £10k–£50k.
    A,
    /// Inform, simple monitoring, communication (e.g. symptom diary).
    /// Evidence expectation: adds evidence of user benefit and appropriate
    /// reliability — an observational/comparative study. ≈ £50k–£250k.
    B,
    /// Treat, diagnose, or actively guide clinical management.
    /// Evidence expectation: adds high-quality comparative effectiveness
    /// evidence (ideally an RCT) and economic analysis. ≈ £250k–£2M+.
    C,
}

/// The clinical function a digital health technology performs — the axis on
/// which the ESF classifies it.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClinicalFunction {
    /// System service with no direct patient outcome (e.g. e-rostering).
    SystemService,
    /// Informs patients, simple monitoring, or communication
    /// (e.g. symptom diary, medication reminder).
    InformOrMonitor,
    /// Treats, diagnoses, or actively guides clinical management
    /// (e.g. dose-adjustment recommendations).
    TreatDiagnoseOrGuide,
}

/// Classify a technology into its ESF tier from its clinical function.
///
/// The mapping is the ESF's own: system services → Tier A, inform/monitor →
/// Tier B, treat/diagnose/guide → Tier C. Classify by actual behavior — a
/// "supports clinical decisions" claim moves the product up a tier whatever
/// the marketing intended.
///
/// # Arguments
///
/// * `function` — the clinical function the product actually performs.
///
/// # Returns
///
/// The corresponding [`EsfTier`].
///
/// # Examples
///
/// ```rust
/// use health_economics::nice_evidence_standards_framework::{
///     classify_tier, ClinicalFunction, EsfTier,
/// };
///
/// // A medication-reminder app is Tier B; adding dose-adjustment
/// // recommendations moves it to Tier C.
/// assert_eq!(classify_tier(ClinicalFunction::InformOrMonitor), EsfTier::B);
/// assert_eq!(classify_tier(ClinicalFunction::TreatDiagnoseOrGuide), EsfTier::C);
/// assert_eq!(classify_tier(ClinicalFunction::SystemService), EsfTier::A);
/// ```
pub fn classify_tier(function: ClinicalFunction) -> EsfTier {
    match function {
        ClinicalFunction::SystemService => EsfTier::A,
        ClinicalFunction::InformOrMonitor => EsfTier::B,
        ClinicalFunction::TreatDiagnoseOrGuide => EsfTier::C,
    }
}

/// Indicative evidence-investment range for a tier, in pounds: (low, high).
///
/// Tier A ≈ £10k–£50k (documentation + assurance), Tier B ≈ £50k–£250k
/// (observational user-benefit study), Tier C ≈ £250k–£2M+ (RCT-grade study
/// plus economic model). Tier C's high end is open ("£2M+"); the returned
/// high is its £2M anchor.
///
/// # Arguments
///
/// * `tier` — the ESF tier.
///
/// # Returns
///
/// A `(low, high)` tuple in pounds sterling.
///
/// # Examples
///
/// ```rust
/// use health_economics::nice_evidence_standards_framework::{
///     evidence_investment_range, EsfTier,
/// };
///
/// // A £600k RCT sits inside Tier C's indicative £250k–£2M range.
/// let (low, high) = evidence_investment_range(EsfTier::C);
/// assert!(low <= 600_000.0 && 600_000.0 <= high);
///
/// // Claiming a tier-moving capability "can 10× the bill":
/// let (b_low, _) = evidence_investment_range(EsfTier::B);
/// assert!(low / b_low >= 5.0);
/// ```
pub fn evidence_investment_range(tier: EsfTier) -> (f64, f64) {
    match tier {
        EsfTier::A => (10_000.0, 50_000.0),
        EsfTier::B => (50_000.0, 250_000.0),
        EsfTier::C => (250_000.0, 2_000_000.0),
    }
}

/// Years the feature must hold value before its evidence cost breaks even.
///
/// `evidence cost / incremental annual revenue` — the commercial calculation
/// that prices a tier-moving product claim. A product decision looks entirely
/// different once the ESF tier is priced in.
///
/// # Arguments
///
/// * `evidence_cost` — the tier's evidence investment (e.g. a £600k RCT), in
///   currency.
/// * `incremental_annual_revenue` — extra revenue per year the claim earns
///   (same currency).
///
/// # Returns
///
/// `Some(years to break even)`, or `None` when the incremental revenue is
/// zero (the cost is never recouped).
///
/// # Examples
///
/// ```rust
/// use health_economics::nice_evidence_standards_framework::years_to_recoup_evidence_cost;
///
/// // £600k RCT against £200k/year incremental revenue → 3+ years to break even.
/// let years = years_to_recoup_evidence_cost(600_000.0, 200_000.0).unwrap();
/// assert!((years - 3.0).abs() < 1e-9);
///
/// assert!(years_to_recoup_evidence_cost(600_000.0, 0.0).is_none());
/// ```
pub fn years_to_recoup_evidence_cost(
    evidence_cost: f64,
    incremental_annual_revenue: f64,
) -> Option<f64> {
    if incremental_annual_revenue == 0.0 {
        None
    } else {
        Some(evidence_cost / incremental_annual_revenue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// As a reminder app the product is Tier B — a cohort study showing
    /// adherence improvement suffices.
    #[test]
    fn reminder_app_is_tier_b() {
        // Worked example: "As a reminder app: Tier B — a cohort study
        // showing adherence improvement suffices".
        assert_eq!(classify_tier(ClinicalFunction::InformOrMonitor), EsfTier::B);
    }

    /// With dose-adjustment recommendations it becomes Tier C — comparative
    /// effectiveness evidence plus economic analysis.
    #[test]
    fn dose_recommendation_feature_moves_it_to_tier_c() {
        // Worked example: "With dose recommendations: Tier C — comparative
        // effectiveness evidence (likely an RCT) plus economic analysis".
        assert_eq!(classify_tier(ClinicalFunction::TreatDiagnoseOrGuide), EsfTier::C);
        assert!(EsfTier::C > EsfTier::B);
    }

    /// System services with no direct patient outcome are Tier A.
    #[test]
    fn e_rostering_is_tier_a() {
        // Doc's tier table: "Tier A — system services, no direct patient
        // outcome (e.g., e-rostering)".
        assert_eq!(classify_tier(ClinicalFunction::SystemService), EsfTier::A);
    }

    /// If the RCT costs £600k and the feature's incremental revenue is
    /// £200k/year, the feature must hold value 3+ years before evidence costs
    /// break even.
    #[test]
    fn rct_at_600k_against_200k_per_year_breaks_even_in_3_years() {
        // Worked example: "If the RCT costs £600k and the dose feature's
        // incremental revenue is £200k/year, the feature must hold value for
        // 3+ years before evidence costs break even".
        let got = years_to_recoup_evidence_cost(600_000.0, 200_000.0).unwrap();
        assert!((got - 3.0).abs() < 1e-9);
    }

    /// A £600k RCT sits inside Tier C's indicative £250k–£2M range, an order
    /// beyond Tier B's — claiming a tier-moving capability "can 10× the bill".
    #[test]
    fn tier_c_range_contains_the_600k_rct_and_dwarfs_tier_b() {
        // Doc's math: "Tier A ≈ £10k–50k, Tier B ≈ £50k–250k,
        // Tier C ≈ £250k–£2M+ … can 10× the bill".
        let (c_low, c_high) = evidence_investment_range(EsfTier::C);
        assert!(c_low <= 600_000.0 && 600_000.0 <= c_high);
        let (b_low, _) = evidence_investment_range(EsfTier::B);
        assert!((c_low / b_low - 5.0).abs() < 1e-9);
        let (a_low, a_high) = evidence_investment_range(EsfTier::A);
        assert!((a_low - 10_000.0).abs() < 1e-9 && (a_high - 50_000.0).abs() < 1e-9);
    }

    /// Break-even is undefined with zero incremental revenue.
    #[test]
    fn zero_incremental_revenue_never_breaks_even() {
        // Edge case: break-even is a division by incremental revenue.
        assert!(years_to_recoup_evidence_cost(600_000.0, 0.0).is_none());
    }
}
