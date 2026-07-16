//! # Clinical AI Evaluation
//!
//! The core statistics for evaluating a clinical AI or diagnostic model:
//! sensitivity, specificity, AUROC, predictive values, and number needed to
//! screen. The central economic lesson: **a great AUROC does not make a
//! cost-effective deployment** — value depends on the operating point, the
//! prevalence, and what happens downstream of every positive.
//!
//! Regulators (FDA, MHRA) authorize clinical AI at a **locked operating
//! point** — a specific sensitivity/specificity pair (e.g., the first
//! FDA-cleared autonomous diabetic-retinopathy system: sensitivity 87.2%,
//! specificity 90.7% in its pivotal trial).
//!
//! ## Formula
//!
//! ```text
//! Sensitivity = TP / (TP + FN)        — of the truly positive, share caught
//! Specificity = TN / (TN + FP)        — of the truly negative, share cleared
//! AUROC       = P(model ranks a random positive above a random negative)
//!               0.5 chance … 1.0 perfect; threshold-independent — and therefore
//!               deployment-decision-insufficient
//!
//! PPV = TP / (TP + FP)   ← prevalence-dependent (Bayes); collapses when rare
//! NPV = TN / (TN + FN)
//!
//! NNS  ≈ 1 / (prevalence × sensitivity)       — screened per true case found
//! Cost per true case = program cost / TP      — the economic bottom line
//!
//! TP/FP/TN/FN — true/false positive/negative counts
//! prevalence  — fraction of the deployment population truly positive
//! ```
//!
//! ## Why it matters
//!
//! Health economists ask the question accuracy metrics can't answer: at your
//! deployment population's prevalence, what does each detection *cost*, and
//! is acting on it worth it? An economic evaluation of retinopathy-screening
//! AI (npj Digital Medicine 2024) showed higher accuracy alone did not
//! guarantee cost-effectiveness once referral costs were counted. Identical
//! models produce radically different economics at different prevalences —
//! which is why site-specific evaluation is a regulatory theme and why "our
//! model has 0.95 AUROC" is the beginning of an economic case, not the end.
//!
//! ## Example
//!
//! Same model, two settings — sensitivity 90%, specificity 93%. In a
//! specialist clinic (prevalence 20%) PPV ≈ 76%: 3 in 4 alerts real. In
//! primary care (prevalence 1%) PPV ≈ 11.5%: 8 in 9 alerts false, and with a
//! £350 workup per positive the cost per true case found is ≈ £3,045.
//!
//! ```rust
//! use health_economics::clinical_ai_evaluation::{
//!     cost_per_true_case, positive_rate, ppv_from_rates,
//! };
//!
//! // Specialist clinic, prevalence 20%: PPV = 0.18/0.236 ≈ 76%.
//! let clinic = ppv_from_rates(0.90, 0.93, 0.20).unwrap();
//! assert!((clinic - 0.76).abs() < 0.005);
//!
//! // Primary care, prevalence 1%: PPV = 0.009/0.0783 ≈ 11.5%.
//! let primary = ppv_from_rates(0.90, 0.93, 0.01).unwrap();
//! assert!((primary - 0.115).abs() < 0.001);
//!
//! // Positive rate 0.0783; workup £350 each → ≈ £3,045 per true case found.
//! assert!((positive_rate(0.90, 0.93, 0.01) - 0.0783).abs() < 1e-9);
//! let cost = cost_per_true_case(0.90, 0.93, 0.01, 350.0).unwrap();
//! assert!((cost - 3_045.0).abs() < 0.5);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Ship the confusion matrix at the deployment prevalence**, not the ROC
//!   curve alone.
//! - **Let the threshold be an economic decision** — the sens/spec trade-off
//!   should minimize expected cost (missed cases × miss cost vs false alarms
//!   × workup cost), not maximize a benchmark statistic.
//! - Alert systems, anomaly detectors, and security scanners are diagnostic
//!   tests over low-prevalence event streams, with alert fatigue as the NNH.
//! - Model updates that shift the operating point re-open the economics (and
//!   the regulatory clearance).
//!
//! ## Pitfalls
//!
//! - **AUROC shopping**: comparing models on AUROC when they'll run at one
//!   threshold — compare at the operating point.
//! - **Trial-prevalence PPV quoted for real-world deployment** — the
//!   classic; always recompute at local prevalence.
//! - **Spectrum bias**: models validated on obvious cases vs healthy
//!   controls overperform on the ambiguous middle that dominates practice.
//! - **No downstream pathway costing**: every positive triggers a workup; a
//!   model is an intervention on the *whole pathway's* economics.
//!
//! ## Sources
//!
//! - Diagnostic accuracy measures reference.
//!   <https://www.medcalc.org/en/manual/roc-curves.php>
//! - Economic evaluation of AI retinopathy screening, npj Digital Medicine 2024.
//!   <https://www.nature.com/articles/s41746-024-01032-9>
//! - Laupacis et al., NEJM 1988 (NNT foundations).
//!   <https://pubmed.ncbi.nlm.nih.gov/3374545/>
//!
//! Topic doc: health-economics-metrics/topics/clinical-ai-evaluation.md

/// Sensitivity = TP / (TP + FN): of the truly positive, the share caught.
///
/// Dimensionless fraction in 0–1; counts may be raw integers or rates, as
/// long as both share units.
///
/// # Arguments
///
/// * `true_positives` — positives the model correctly flagged.
/// * `false_negatives` — positives the model missed.
///
/// # Returns
///
/// `Some(sensitivity)`, or `None` when `TP + FN` is zero (no truly positive
/// cases, so the fraction is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::sensitivity;
///
/// // 90 caught, 10 missed → sensitivity 90% (the worked example's model).
/// assert!((sensitivity(90.0, 10.0).unwrap() - 0.90).abs() < 1e-9);
/// assert!(sensitivity(0.0, 0.0).is_none());
/// ```
pub fn sensitivity(true_positives: f64, false_negatives: f64) -> Option<f64> {
    let denom = true_positives + false_negatives;
    if denom == 0.0 { None } else { Some(true_positives / denom) }
}

/// Specificity = TN / (TN + FP): of the truly negative, the share cleared.
///
/// Dimensionless fraction in 0–1.
///
/// # Arguments
///
/// * `true_negatives` — negatives the model correctly cleared.
/// * `false_positives` — negatives the model wrongly flagged.
///
/// # Returns
///
/// `Some(specificity)`, or `None` when `TN + FP` is zero (no truly negative
/// cases).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::specificity;
///
/// // 93 cleared, 7 wrongly flagged → specificity 93%.
/// assert!((specificity(93.0, 7.0).unwrap() - 0.93).abs() < 1e-9);
/// assert!(specificity(0.0, 0.0).is_none());
/// ```
pub fn specificity(true_negatives: f64, false_positives: f64) -> Option<f64> {
    let denom = true_negatives + false_positives;
    if denom == 0.0 { None } else { Some(true_negatives / denom) }
}

/// Positive predictive value from counts: PPV = TP / (TP + FP).
///
/// Use this form when you have an actual confusion matrix; use
/// [`ppv_from_rates`] to recompute at a different deployment prevalence.
///
/// # Arguments
///
/// * `true_positives` — correct positive calls.
/// * `false_positives` — incorrect positive calls.
///
/// # Returns
///
/// `Some(ppv)`, or `None` when the model made no positive calls
/// (`TP + FP` is zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::ppv_from_counts;
///
/// // Primary care per 10,000 screened at prevalence 1%: TP = 90, FP = 693.
/// let ppv = ppv_from_counts(90.0, 693.0).unwrap();
/// assert!((ppv - 90.0 / 783.0).abs() < 1e-9); // ≈ 11.5%
/// assert!(ppv_from_counts(0.0, 0.0).is_none());
/// ```
pub fn ppv_from_counts(true_positives: f64, false_positives: f64) -> Option<f64> {
    let denom = true_positives + false_positives;
    if denom == 0.0 { None } else { Some(true_positives / denom) }
}

/// Negative predictive value from counts: NPV = TN / (TN + FN).
///
/// # Arguments
///
/// * `true_negatives` — correct negative calls.
/// * `false_negatives` — incorrect negative calls (missed positives).
///
/// # Returns
///
/// `Some(npv)`, or `None` when the model made no negative calls
/// (`TN + FN` is zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::npv_from_counts;
///
/// // Primary care per 10,000 screened: TN = 9,207, FN = 10 → NPV > 99.8%.
/// let npv = npv_from_counts(9_207.0, 10.0).unwrap();
/// assert!(npv > 0.998);
/// assert!(npv_from_counts(0.0, 0.0).is_none());
/// ```
pub fn npv_from_counts(true_negatives: f64, false_negatives: f64) -> Option<f64> {
    let denom = true_negatives + false_negatives;
    if denom == 0.0 { None } else { Some(true_negatives / denom) }
}

/// Share of the screened population flagged positive at a given prevalence:
/// sensitivity × prevalence + (1 − specificity) × (1 − prevalence).
///
/// The first term is the true-positive rate in the population; the second is
/// the false-positive rate. All arguments are fractions in 0–1.
///
/// # Arguments
///
/// * `sensitivity` — the model's sensitivity at its operating point.
/// * `specificity` — the model's specificity at its operating point.
/// * `prevalence` — fraction of the deployment population truly positive.
///
/// # Returns
///
/// The fraction of screened people flagged positive (0–1).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::positive_rate;
///
/// // Primary care: 0.9×0.01 + 0.07×0.99 = 0.0783 of screens flag positive.
/// assert!((positive_rate(0.90, 0.93, 0.01) - 0.0783).abs() < 1e-9);
/// ```
pub fn positive_rate(sensitivity: f64, specificity: f64, prevalence: f64) -> f64 {
    // TP rate in population + FP rate in population.
    sensitivity * prevalence + (1.0 - specificity) * (1.0 - prevalence)
}

/// PPV at a deployment prevalence (Bayes):
/// sens × prev / (sens × prev + (1 − spec) × (1 − prev)).
///
/// This is the number to recompute at every site: PPV collapses when the
/// condition is rare, no matter how accurate the model.
///
/// # Arguments
///
/// * `sensitivity` — the model's sensitivity at its operating point (0–1).
/// * `specificity` — the model's specificity at its operating point (0–1).
/// * `prevalence` — fraction of the deployment population truly positive.
///
/// # Returns
///
/// `Some(ppv)`, or `None` when nothing is flagged positive (the positive
/// rate is zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::ppv_from_rates;
///
/// // Same model (sens 90%, spec 93%), two settings:
/// // specialist clinic at prevalence 20% → PPV ≈ 76% (3 in 4 alerts real);
/// let clinic = ppv_from_rates(0.90, 0.93, 0.20).unwrap();
/// assert!((clinic - 0.18 / 0.236).abs() < 1e-9);
///
/// // primary care at prevalence 1% → PPV ≈ 11.5% (8 in 9 alerts false).
/// let primary = ppv_from_rates(0.90, 0.93, 0.01).unwrap();
/// assert!((primary - 0.115).abs() < 0.001);
/// ```
pub fn ppv_from_rates(sensitivity: f64, specificity: f64, prevalence: f64) -> Option<f64> {
    // Bayes: P(disease | positive) = TP rate / (TP rate + FP rate).
    let denom = positive_rate(sensitivity, specificity, prevalence);
    if denom == 0.0 {
        None
    } else {
        Some(sensitivity * prevalence / denom)
    }
}

/// NPV at a deployment prevalence (Bayes):
/// spec × (1 − prev) / (spec × (1 − prev) + (1 − sens) × prev).
///
/// # Arguments
///
/// * `sensitivity` — the model's sensitivity at its operating point (0–1).
/// * `specificity` — the model's specificity at its operating point (0–1).
/// * `prevalence` — fraction of the deployment population truly positive.
///
/// # Returns
///
/// `Some(npv)`, or `None` when nothing is cleared negative (the negative
/// rate is zero).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::npv_from_rates;
///
/// // At low prevalence a negative call is near-certainly right.
/// let npv = npv_from_rates(0.90, 0.93, 0.01).unwrap();
/// assert!(npv > 0.998);
/// ```
pub fn npv_from_rates(sensitivity: f64, specificity: f64, prevalence: f64) -> Option<f64> {
    // Bayes: P(healthy | negative) = TN rate / (TN rate + FN rate).
    let denom = specificity * (1.0 - prevalence) + (1.0 - sensitivity) * prevalence;
    if denom == 0.0 {
        None
    } else {
        Some(specificity * (1.0 - prevalence) / denom)
    }
}

/// Number needed to screen ≈ 1 / (prevalence × sensitivity): people screened
/// per true case found.
///
/// # Arguments
///
/// * `prevalence` — fraction of the screened population truly positive.
/// * `sensitivity` — the model's sensitivity at its operating point.
///
/// # Returns
///
/// `Some(nns)`, or `None` when `prevalence × sensitivity` is zero (no true
/// case can ever be found).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::number_needed_to_screen;
///
/// // Primary care: prevalence 1%, sensitivity 90% → ≈ 111 screened per case.
/// let nns = number_needed_to_screen(0.01, 0.90).unwrap();
/// assert!((nns - 1.0 / 0.009).abs() < 1e-9);
/// assert!(number_needed_to_screen(0.0, 0.9).is_none());
/// ```
pub fn number_needed_to_screen(prevalence: f64, sensitivity: f64) -> Option<f64> {
    let denom = prevalence * sensitivity;
    if denom == 0.0 { None } else { Some(1.0 / denom) }
}

/// Cost per true case from counts = program cost / true positives.
///
/// The economic bottom line, from the programme ledger view.
///
/// # Arguments
///
/// * `program_cost` — total programme cost, in currency.
/// * `true_positives` — true cases the programme found.
///
/// # Returns
///
/// `Some(cost)`, or `None` when `true_positives` is zero (no case was
/// found, so cost per case is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::cost_per_true_case_from_counts;
///
/// // Per 10,000 screened in primary care: 783 workups at £350 found 90 cases
/// // → ≈ £3,045 per case, matching the rate-based view.
/// let cost = cost_per_true_case_from_counts(783.0 * 350.0, 90.0).unwrap();
/// assert!((cost - 3_045.0).abs() < 0.5);
/// assert!(cost_per_true_case_from_counts(1000.0, 0.0).is_none());
/// ```
pub fn cost_per_true_case_from_counts(program_cost: f64, true_positives: f64) -> Option<f64> {
    if true_positives == 0.0 { None } else { Some(program_cost / true_positives) }
}

/// Cost per true case found when every positive triggers a workup:
/// positive rate × workup cost / (sensitivity × prevalence).
///
/// Rate-based per-person-screened view of the same bottom line as
/// [`cost_per_true_case_from_counts`]: every flagged positive (true or
/// false) incurs the workup cost; only the true positives count as cases.
///
/// # Arguments
///
/// * `sensitivity` — the model's sensitivity at its operating point (0–1).
/// * `specificity` — the model's specificity at its operating point (0–1).
/// * `prevalence` — fraction of the deployment population truly positive.
/// * `workup_cost_per_positive` — downstream cost triggered by each
///   positive, in currency (£350 in the worked example).
///
/// # Returns
///
/// `Some(cost)` per true case found, or `None` when
/// `sensitivity × prevalence` is zero (no true positives arise).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::cost_per_true_case;
///
/// // Primary care: (0.009 + 0.0693) × £350 / 0.009 ≈ £3,045 per case found.
/// let cost = cost_per_true_case(0.90, 0.93, 0.01, 350.0).unwrap();
/// assert!((cost - 3_045.0).abs() < 0.5);
/// ```
pub fn cost_per_true_case(
    sensitivity: f64,
    specificity: f64,
    prevalence: f64,
    workup_cost_per_positive: f64,
) -> Option<f64> {
    // sens × prev = true positives per person screened.
    let true_positive_rate_in_population = sensitivity * prevalence;
    if true_positive_rate_in_population == 0.0 {
        None
    } else {
        // All positives (true + false) pay for a workup; divide by the true
        // positives to get cost per case actually found.
        Some(
            positive_rate(sensitivity, specificity, prevalence) * workup_cost_per_positive
                / true_positive_rate_in_population,
        )
    }
}

/// AUROC = P(a randomly chosen positive scores above a randomly chosen
/// negative), ties counting 1/2 — computed exactly over all pairs of the
/// given scores.
///
/// 0.5 is chance, 1.0 perfect separation. Threshold-independent, and
/// therefore deployment-decision-insufficient: two models with the same
/// AUROC can have very different economics at their operating points.
/// O(|positives| × |negatives|) exact pairwise computation.
///
/// # Arguments
///
/// * `positive_scores` — model scores for truly positive cases.
/// * `negative_scores` — model scores for truly negative cases.
///
/// # Returns
///
/// `Some(auroc)` in 0–1, or `None` if either class is empty (no pair to
/// rank).
///
/// # Examples
///
/// ```rust
/// use health_economics::clinical_ai_evaluation::auroc;
///
/// // Perfect separation → 1.0; identical distributions → 0.5 (chance).
/// assert!((auroc(&[0.9, 0.8], &[0.2, 0.1]).unwrap() - 1.0).abs() < 1e-9);
/// assert!((auroc(&[0.5, 0.5], &[0.5, 0.5]).unwrap() - 0.5).abs() < 1e-9);
/// assert!(auroc(&[], &[0.1]).is_none());
/// ```
pub fn auroc(positive_scores: &[f64], negative_scores: &[f64]) -> Option<f64> {
    if positive_scores.is_empty() || negative_scores.is_empty() {
        return None;
    }
    // Count concordant pairs (positive ranked above negative); ties score ½.
    let mut favorable = 0.0;
    for &p in positive_scores {
        for &n in negative_scores {
            if p > n {
                favorable += 1.0;
            } else if p == n {
                favorable += 0.5;
            }
        }
    }
    // Normalize by the number of (positive, negative) pairs.
    Some(favorable / (positive_scores.len() as f64 * negative_scores.len() as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: same model in two settings — sensitivity 90%,
    // specificity 93%.

    // Worked-example line: "Specialist clinic (prevalence 20%): PPV =
    // 0.18/0.236 ≈ 76% → 3 in 4 alerts real".
    #[test]
    fn specialist_clinic_ppv_is_about_76_percent() {
        // Prevalence 20%: PPV = 0.18/0.236 ≈ 76% → 3 in 4 alerts real.
        let ppv = ppv_from_rates(0.90, 0.93, 0.20).unwrap();
        assert!((ppv - 0.18 / 0.236).abs() < 1e-9);
        assert!((ppv - 0.76).abs() < 0.005);
    }

    // Worked-example line: "Primary care (prevalence 1%): PPV =
    // 0.009/0.0783 ≈ 11.5% → 8 in 9 alerts false".
    #[test]
    fn primary_care_ppv_is_about_11_5_percent() {
        // Prevalence 1%: PPV = 0.009/0.0783 ≈ 11.5% → 8 in 9 alerts false.
        let ppv = ppv_from_rates(0.90, 0.93, 0.01).unwrap();
        assert!((ppv - 0.009 / 0.0783).abs() < 1e-9);
        assert!((ppv - 0.115).abs() < 0.001);
    }

    // Worked-example denominator: "(0.9×0.01 + 0.07×0.99) = 0.0783".
    #[test]
    fn primary_care_positive_rate_is_0_0783() {
        let rate = positive_rate(0.90, 0.93, 0.01);
        assert!((rate - 0.0783).abs() < 1e-9);
    }

    // Worked-example line: "cost per true case = (0.009 + 0.0693) × 350 /
    // 0.009 ≈ £3,045 per case found".
    #[test]
    fn primary_care_cost_per_true_case_is_about_3045() {
        // (0.009 + 0.0693) × £350 / 0.009 ≈ £3,045 per case found.
        let cost = cost_per_true_case(0.90, 0.93, 0.01, 350.0).unwrap();
        assert!((cost - 3_045.0).abs() < 0.5);
    }

    // Formula lines: "Sensitivity = TP / (TP + FN)", "Specificity =
    // TN / (TN + FP)" — counts consistent with the worked example's
    // sens 90% / spec 93% model.
    #[test]
    fn sensitivity_and_specificity_from_counts() {
        // Counts consistent with sens 90% / spec 93%.
        assert!((sensitivity(90.0, 10.0).unwrap() - 0.90).abs() < 1e-9);
        assert!((specificity(93.0, 7.0).unwrap() - 0.93).abs() < 1e-9);
        assert!(sensitivity(0.0, 0.0).is_none());
        assert!(specificity(0.0, 0.0).is_none());
    }

    // Formula lines: "PPV = TP / (TP + FP)", "NPV = TN / (TN + FN)" with
    // the primary-care confusion matrix per 10,000 screened at 1% prevalence.
    #[test]
    fn predictive_values_from_counts() {
        // Primary-care setting per 10,000 screened at prevalence 1%:
        // TP = 90, FP = 693, TN = 9,207, FN = 10.
        let ppv = ppv_from_counts(90.0, 693.0).unwrap();
        assert!((ppv - 90.0 / 783.0).abs() < 1e-9);
        let npv = npv_from_counts(9_207.0, 10.0).unwrap();
        assert!(npv > 0.998);
        assert!(ppv_from_counts(0.0, 0.0).is_none());
    }

    // Bayes NPV at the worked example's primary-care setting (sens 90%,
    // spec 93%, prevalence 1%).
    #[test]
    fn npv_from_rates_is_high_at_low_prevalence() {
        let npv = npv_from_rates(0.90, 0.93, 0.01).unwrap();
        assert!((npv - (0.93 * 0.99) / (0.93 * 0.99 + 0.10 * 0.01)).abs() < 1e-9);
    }

    // Formula line: "NNS ≈ 1 / (prevalence × sensitivity)" at the primary-
    // care operating point.
    #[test]
    fn number_needed_to_screen_matches_definition() {
        // At prevalence 1% and sensitivity 90%: NNS ≈ 111 screened per case.
        let nns = number_needed_to_screen(0.01, 0.90).unwrap();
        assert!((nns - 1.0 / 0.009).abs() < 1e-9);
        assert!(number_needed_to_screen(0.0, 0.9).is_none());
    }

    // Formula line: "Cost per true case = program cost / TP" — the £3,045
    // figure reproduced from the counts view.
    #[test]
    fn cost_per_true_case_from_counts_matches_program_view() {
        // £3,045 per case × 90 cases = program cost for 10,000 screened.
        let cost = cost_per_true_case_from_counts(783.0 * 350.0, 90.0).unwrap();
        assert!((cost - 3_045.0).abs() < 0.5);
        assert!(cost_per_true_case_from_counts(1000.0, 0.0).is_none());
    }

    // Formula line: "AUROC = P(model ranks a random positive above a random
    // negative), 0.5 chance … 1.0 perfect".
    #[test]
    fn auroc_bounds_and_ties() {
        // Perfect separation → 1.0; identical score distributions → 0.5.
        assert!((auroc(&[0.9, 0.8], &[0.2, 0.1]).unwrap() - 1.0).abs() < 1e-9);
        assert!((auroc(&[0.5, 0.5], &[0.5, 0.5]).unwrap() - 0.5).abs() < 1e-9);
        assert!(auroc(&[], &[0.1]).is_none());
    }
}
