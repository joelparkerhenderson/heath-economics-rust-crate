//! # Screening Economics
//!
//! Screening economics govern the value of testing asymptomatic
//! populations. The core mathematical fact: at low disease prevalence, even
//! excellent tests generate mostly false positives — and the downstream
//! cost of chasing them can swamp the benefit of the true finds.
//!
//! Positive predictive value (PPV) — the probability that a positive result
//! is real — collapses at low prevalence, so PPV and the cost per true case
//! found govern whether a screening programme is worthwhile.
//!
//! ## Formula
//!
//! ```text
//! PPV = (sens × prev) / [sens × prev + (1 − spec) × (1 − prev)]
//!
//! Cost per true case found
//!     = (screening cost + workup cost × all positives) / true positives
//!
//! sens   sensitivity: P(test positive | disease), fraction 0..1
//! spec   specificity: P(test negative | no disease), fraction 0..1
//! prev   prevalence: P(disease) in the screened population, fraction 0..1
//! ```
//!
//! ## Why it matters
//!
//! Since 1968, WHO's Wilson–Jungner criteria have set the bar for
//! population screening: the condition must be important, the test
//! acceptable and accurate, effective treatment must exist, and the
//! economics must balance. The UK National Screening Committee applies
//! formal cost-effectiveness analysis before approving any national
//! programme — and rejects most proposals. At sensitivity 90%, specificity
//! 95%, and prevalence 0.5%, PPV is only ~8.3%: eleven out of twelve
//! positives are false. Every "AI will screen everyone for everything"
//! pitch runs into this machinery, and usually loses to the arithmetic.
//!
//! ## Example
//!
//! AI retinal screening for a rare condition: 100,000 people, prevalence
//! 0.5%, sensitivity 90%, specificity 95%, scan £15, confirmatory workup
//! £400:
//!
//! ```rust
//! use health_economics::screening_economics::{
//!     positive_predictive_value, true_positives, false_positives,
//!     total_programme_cost, cost_per_true_case,
//! };
//!
//! // PPV = 0.0045 / (0.0045 + 0.04975) ≈ 8.3%
//! let ppv = positive_predictive_value(0.90, 0.95, 0.005).unwrap();
//! assert!((ppv - 0.083).abs() < 0.001);
//!
//! // True positives:  100,000 × 0.005 × 0.90 = 450
//! let tp = true_positives(100_000.0, 0.005, 0.90);
//! assert!((tp - 450.0).abs() < 1e-9);
//!
//! // False positives: 100,000 × 0.995 × 0.05 = 4,975
//! let fp = false_positives(100_000.0, 0.005, 0.95);
//! assert!((fp - 4_975.0).abs() < 1e-9);
//!
//! // Cost = 100,000 × 15 + (450 + 4,975) × 400 = £3.67M
//! // (tolerance allows for floating-point rounding in tp and fp)
//! let cost = total_programme_cost(100_000.0, 15.0, 400.0, tp, fp);
//! assert!((cost - 3_670_000.0).abs() < 1e-4);
//!
//! // Cost per true case ≈ £8,156
//! let per_case = cost_per_true_case(cost, tp).unwrap();
//! assert!((per_case - 8_156.0).abs() < 1.0);
//! ```
//!
//! Raising specificity to 99% cuts false positives to 995 and cost per case
//! to ~£4,622 — specificity, not sensitivity, is where screening economics
//! are won at low prevalence.
//!
//! ## Software engineering connection
//!
//! - Static analysis, security scanning, and anomaly detection are
//!   screening programmes over codebases and telemetry, with true-defect
//!   prevalence often well under 1% per alert-opportunity.
//! - The identical math explains alert fatigue: a 95%-specific scanner on
//!   low-prevalence code drowns teams in false positives (the clinical term
//!   is screening harm; the engineering term is pager numbness).
//! - Raise specificity before sensitivity.
//! - Screen higher-prevalence subpopulations (risk-based targeting ↔
//!   changed-code-only scanning).
//! - Count triage cost in the tool's economics.
//!
//! ## Pitfalls
//!
//! - Quoting sensitivity/specificity without prevalence — accuracy without
//!   PPV is marketing.
//! - Ignoring overdiagnosis: finding indolent "disease" that would never
//!   have harmed triggers real treatment costs and harms.
//! - Lead-time bias: earlier detection without changed outcomes inflates
//!   apparent survival.
//!
//! ## Sources
//!
//! - Wilson JMG, Jungner G. "Principles and practice of screening for
//!   disease." WHO 1968. <https://apps.who.int/iris/handle/10665/37650>
//! - UK National Screening Committee.
//!   <https://www.gov.uk/government/groups/uk-national-screening-committee-uk-nsc>
//!
//! Topic doc: health-economics-metrics/topics/screening-economics.md

/// Positive predictive value: probability that a positive result is real.
///
/// PPV collapses at low prevalence even for excellent tests — the central
/// fact of screening economics.
///
/// # Arguments
///
/// * `sensitivity` — P(test positive | disease), fraction in 0..1.
/// * `specificity` — P(test negative | no disease), fraction in 0..1.
/// * `prevalence` — P(disease) in the screened population, fraction in 0..1.
///
/// # Returns
///
/// PPV as a fraction in 0..1, or `None` if the overall positive rate (the
/// denominator: true-positive rate + false-positive rate) is zero — i.e.
/// the test can never return a positive.
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::positive_predictive_value;
///
/// // Doc: sens 90%, spec 95%, prev 0.5% → PPV ≈ 8.3%
/// // (eleven out of twelve positives are false).
/// let ppv = positive_predictive_value(0.90, 0.95, 0.005).unwrap();
/// assert!((ppv - 0.083).abs() < 0.001);
/// ```
pub fn positive_predictive_value(
    sensitivity: f64,
    specificity: f64,
    prevalence: f64,
) -> Option<f64> {
    // Bayes' rule terms: P(positive ∧ disease) and P(positive ∧ no disease).
    let true_positive_rate = sensitivity * prevalence;
    let false_positive_rate = (1.0 - specificity) * (1.0 - prevalence);
    // Denominator is the overall positive rate P(positive).
    let denominator = true_positive_rate + false_positive_rate;
    if denominator == 0.0 {
        None
    } else {
        Some(true_positive_rate / denominator)
    }
}

/// Expected number of true positives: population × prevalence × sensitivity.
///
/// # Arguments
///
/// * `population` — number of people screened.
/// * `prevalence` — fraction (0..1) of the population with the condition.
/// * `sensitivity` — fraction (0..1) of true cases the test detects.
///
/// # Returns
///
/// Expected count of true-positive results.
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::true_positives;
///
/// // Doc: 100,000 × 0.005 × 0.90 = 450
/// let tp = true_positives(100_000.0, 0.005, 0.90);
/// assert!((tp - 450.0).abs() < 1e-9);
/// ```
pub fn true_positives(population: f64, prevalence: f64, sensitivity: f64) -> f64 {
    population * prevalence * sensitivity
}

/// Expected number of false positives: population × (1 − prevalence) × (1 − specificity).
///
/// # Arguments
///
/// * `population` — number of people screened.
/// * `prevalence` — fraction (0..1) of the population with the condition.
/// * `specificity` — fraction (0..1) of healthy people correctly cleared.
///
/// # Returns
///
/// Expected count of false-positive results.
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::false_positives;
///
/// // Doc: 100,000 × 0.995 × 0.05 = 4,975
/// let fp = false_positives(100_000.0, 0.005, 0.95);
/// assert!((fp - 4_975.0).abs() < 1e-9);
/// ```
pub fn false_positives(population: f64, prevalence: f64, specificity: f64) -> f64 {
    population * (1.0 - prevalence) * (1.0 - specificity)
}

/// Total programme cost: screening everyone plus confirmatory workup for
/// every positive, true and false.
///
/// The false positives are why specificity dominates the economics at low
/// prevalence: each one incurs the full workup cost.
///
/// # Arguments
///
/// * `population` — number of people screened.
/// * `screening_cost_per_person` — cost of the screening test per person.
/// * `workup_cost_per_positive` — cost of confirmatory workup per positive
///   result.
/// * `true_positives` — expected true-positive count (see [`true_positives`]).
/// * `false_positives` — expected false-positive count (see [`false_positives`]).
///
/// # Returns
///
/// Total programme cost in currency units.
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::total_programme_cost;
///
/// // Doc: 100,000 × 15 + (450 + 4,975) × 400 = 1.5M + 2.17M = £3.67M
/// let cost = total_programme_cost(100_000.0, 15.0, 400.0, 450.0, 4_975.0);
/// assert!((cost - 3_670_000.0).abs() < 1e-9);
/// ```
pub fn total_programme_cost(
    population: f64,
    screening_cost_per_person: f64,
    workup_cost_per_positive: f64,
    true_positives: f64,
    false_positives: f64,
) -> f64 {
    // Screen everyone; work up every positive, real or not.
    population * screening_cost_per_person
        + (true_positives + false_positives) * workup_cost_per_positive
}

/// Cost per true case found: total programme cost / true positives.
///
/// The number the programme's per-case value (see
/// [`net_value_per_case_found`]) must clear.
///
/// # Arguments
///
/// * `total_programme_cost` — total cost of screening plus workup.
/// * `true_positives` — number of true cases found.
///
/// # Returns
///
/// Cost per true case, or `None` if no true positives are found (the ratio
/// is undefined — the programme finds nothing).
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::cost_per_true_case;
///
/// // Doc: £3.67M / 450 ≈ £8,156 per true case.
/// let per_case = cost_per_true_case(3_670_000.0, 450.0).unwrap();
/// assert!((per_case - 8_156.0).abs() < 1.0);
/// ```
pub fn cost_per_true_case(total_programme_cost: f64, true_positives: f64) -> Option<f64> {
    if true_positives == 0.0 {
        None
    } else {
        Some(total_programme_cost / true_positives)
    }
}

/// Net value per case found: earlier-intervention value per case minus
/// overdiagnosis harm per case.
///
/// This is the benchmark [`cost_per_true_case`] must clear for the
/// programme to be worthwhile. Overdiagnosis harm accounts for cases found
/// that would never have mattered but still trigger treatment costs and
/// harms.
///
/// # Arguments
///
/// * `earlier_intervention_value_per_case` — value (savings plus monetized
///   health gain) of treating a case earlier.
/// * `overdiagnosis_harm_per_case` — expected cost/harm per case from
///   overdiagnosis.
///
/// # Returns
///
/// Net value per true case found (can be negative if overdiagnosis harm
/// dominates).
///
/// # Examples
///
/// ```rust
/// use health_economics::screening_economics::net_value_per_case_found;
///
/// // Doc: early treatment saves £20,000 + 1 QALY per case → clears £8,156 easily.
/// let value = net_value_per_case_found(20_000.0, 0.0);
/// assert!(value > 8_156.0);
/// ```
pub fn net_value_per_case_found(
    earlier_intervention_value_per_case: f64,
    overdiagnosis_harm_per_case: f64,
) -> f64 {
    earlier_intervention_value_per_case - overdiagnosis_harm_per_case
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc (The math): "PPV = (0.9 × 0.005) / (0.9 × 0.005 + 0.05 × 0.995)
    // = 0.0045 / (0.0045 + 0.04975) ≈ 8.3%".
    #[test]
    fn ppv_collapses_to_8_3_percent_at_low_prevalence() {
        // Doc: sens 90%, spec 95%, prev 0.5% → PPV ≈ 8.3%
        let ppv = positive_predictive_value(0.90, 0.95, 0.005).unwrap();
        assert!((ppv - 0.083).abs() < 0.001);
        // Intermediate figures: 0.0045 / (0.0045 + 0.04975)
        assert!((0.90_f64 * 0.005 - 0.0045).abs() < 1e-12);
        assert!((0.05_f64 * 0.995 - 0.04975).abs() < 1e-12);
    }

    // Edge case: a test that can never return a positive has undefined PPV.
    #[test]
    fn ppv_returns_none_when_no_positives_possible() {
        assert!(positive_predictive_value(0.0, 1.0, 0.0).is_none());
    }

    // Doc worked example: "True positives: 100,000 × 0.005 × 0.90 = 450".
    #[test]
    fn worked_example_true_positives_450() {
        let tp = true_positives(100_000.0, 0.005, 0.90);
        assert!((tp - 450.0).abs() < 1e-9);
    }

    // Doc worked example: "False positives: 100,000 × 0.995 × 0.05 = 4,975".
    #[test]
    fn worked_example_false_positives_4975() {
        let fp = false_positives(100_000.0, 0.005, 0.95);
        assert!((fp - 4_975.0).abs() < 1e-9);
    }

    // Doc worked example: "Cost = 100,000 × 15 + (450 + 4,975) × 400 = £3.67M".
    #[test]
    fn worked_example_total_cost_3_67m() {
        // 100,000 × 15 + (450 + 4,975) × 400 = 1.5M + 2.17M = £3.67M
        let cost = total_programme_cost(100_000.0, 15.0, 400.0, 450.0, 4_975.0);
        assert!((cost - 3_670_000.0).abs() < 1e-9);
    }

    // Doc worked example: "Cost per true case ≈ £8,156".
    #[test]
    fn worked_example_cost_per_true_case_8156() {
        let cost = total_programme_cost(100_000.0, 15.0, 400.0, 450.0, 4_975.0);
        let per_case = cost_per_true_case(cost, 450.0).unwrap();
        // Doc: ≈ £8,156 (exact 8,155.56)
        assert!((per_case - 8_156.0).abs() < 1.0);
    }

    // Doc: "Raise specificity to 99% ... workup cost falls to
    // (450 + 995) × 400 = £0.58M, total £2.08M, cost per case ≈ £4,622".
    #[test]
    fn raising_specificity_to_99_percent_wins_the_economics() {
        let fp = false_positives(100_000.0, 0.005, 0.99);
        assert!((fp - 995.0).abs() < 1e-9);
        // Workup falls to (450 + 995) × 400 = £0.58M, total £2.08M
        let workup = (450.0 + fp) * 400.0;
        assert!((workup - 580_000.0).abs() < 2_000.0);
        let total = total_programme_cost(100_000.0, 15.0, 400.0, 450.0, fp);
        assert!((total - 2_080_000.0).abs() < 5_000.0);
        // Doc: cost per case ≈ £4,622 (exact 4,617.78; doc rounds via £2.08M/450)
        let per_case = cost_per_true_case(total, 450.0).unwrap();
        assert!((per_case - 4_622.0).abs() < 10.0);
    }

    // Doc: "If early treatment saves £20,000 + 1 QALY per case, the
    // programme clears easily" — value per case exceeds £8,156 cost per case.
    #[test]
    fn programme_clears_when_case_value_exceeds_cost_per_case() {
        // Doc: early treatment saves £20,000 + 1 QALY per case → clears £8,156 easily.
        let value = net_value_per_case_found(20_000.0, 0.0);
        assert!(value > 8_156.0);
    }
}
