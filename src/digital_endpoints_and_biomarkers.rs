//! # Digital Endpoints and Biomarkers
//!
//! A digital biomarker is an objective physiological or behavioral measure
//! collected via sensors (gait speed from a phone, sleep from a wearable,
//! tremor from accelerometry). A digital endpoint is such a measure elevated
//! to a **trial outcome** — used to demonstrate treatment effect. The
//! promotion from "data the device emits" to "evidence a regulator accepts"
//! runs through a defined validation ladder: verification/analytical
//! validation, clinical validation, and a demonstrated meaningful aspect of
//! health. An endpoint without all three is telemetry, not evidence.
//!
//! The economic core implemented here: dense passive sampling cuts the
//! variance of the change estimate, which shrinks trial sample size (or
//! improves detectable effect size at fixed power).
//!
//! ## Formula
//!
//! ```text
//! Trial power:  N ∝ σ² / Δ²   (σ² falls with dense sampling)
//!
//! detectable effect improvement at fixed power = √(variance reduction fold)
//! sample size ratio at fixed Δ                 = σ²_new / σ²_old
//! trial saving = patients cut × cost per enrolled patient
//!
//! N  — required sample size
//! σ² — variance of the (annual-change) estimate
//! Δ  — detectable effect size at fixed power
//! k  — constant bundling power/significance choices (≈16 for 80% power, α = 0.05)
//! ```
//!
//! ## Why it matters
//!
//! Traditional trial endpoints are episodic (clinic visits every 3 months)
//! and expensive; digital endpoints are continuous, ecological (real life,
//! not clinic performance), and cheap per observation — they can shrink
//! trials, detect effects earlier, and enable decentralized studies. The
//! catch is validation: the accepted FDA-aligned, three-pillar framework
//! requires **verification/analytical validation** (the sensor measures the
//! physical quantity accurately), **clinical validation** (the measure
//! reflects the clinical state it claims to), and a demonstrated
//! **meaningful aspect of health** (patients care about what it captures).
//!
//! ## Example
//!
//! The topic doc's worked example: a Parkinson's trial compares gait speed
//! from a wrist sensor (~200 passive measurements/patient/year) with
//! quarterly clinic-rated scores (4/patient/year). Dense sampling cuts the
//! variance of the annual-change estimate ~5×, improving the detectable
//! effect ~√5 ≈ 2.2×, or shrinking sample size ~40–60% at the same
//! hypothesis. At £25,000 per enrolled patient, cutting 200 patients saves
//! ~£5M per trial — against a validation investment of perhaps £1–2M.
//!
//! ```rust
//! use health_economics::digital_endpoints_and_biomarkers::{
//!     sampling_density_ratio, detectable_effect_improvement, sample_size_ratio,
//!     required_sample_size, trial_cost_saving,
//! };
//!
//! // ~200 passive vs 4 clinic measurements/patient/year: 50× denser.
//! assert_eq!(sampling_density_ratio(200.0, 4.0), Some(50.0));
//!
//! // Variance of the annual-change estimate falls ~5× →
//! // detectable effect improves √5 ≈ 2.2× at fixed power.
//! assert!((detectable_effect_improvement(5.0) - 2.2).abs() < 0.05);
//!
//! // Equivalently, sample size shrinks to 1/5 at the same hypothesis (N ∝ σ²).
//! assert_eq!(sample_size_ratio(1.0, 5.0), Some(0.2));
//! let n_old = required_sample_size(5.0, 1.0, 16.0).unwrap();
//! let n_new = required_sample_size(1.0, 1.0, 16.0).unwrap();
//! assert!((n_new / n_old - 0.2).abs() < 1e-9);
//!
//! // At £25,000 per enrolled patient, cutting 200 patients ≈ £5M saved.
//! assert_eq!(trial_cost_saving(200.0, 25_000.0), 5_000_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! Digital endpoints are a data-engineering discipline wearing clinical
//! clothes:
//!
//! - **Provenance and versioning**: algorithm updates mid-study threaten
//!   comparability — the PCCP problem in trial form; version-lock and
//!   bridge-validate.
//! - **Missing-data design**: wear-time gaps are informative, not random;
//!   imputation choices are scientific claims.
//! - **Edge/cloud split decisions** change what raw signal is even
//!   recoverable later.
//! - Teams that treat the measurement pipeline as regulated software from day
//!   one — tested, versioned, documented — buy their endpoints' credibility
//!   cheaply; retrofitting validation onto a moved-fast pipeline is where
//!   digital-endpoint programs die.
//!
//! ## Pitfalls
//!
//! - **Correlation-with-clinic as full validation**: matching a flawed clinic
//!   measure proves inheritance, not truth; validate against the meaningful
//!   health aspect.
//! - **Novel-endpoint regulatory risk**: an unprecedented endpoint may be
//!   scientifically superior and still sink a submission — engage regulators
//!   early (qualification programs exist).
//! - **Sensor-population mismatch**: validation in young healthy wrists,
//!   deployment in elderly patients with tremor and pigmentation differences
//!   the PPG never saw.
//! - **Feature drift**: retraining the gait algorithm on new data silently
//!   redefines the endpoint mid-study.
//!
//! ## Sources
//!
//! - Coravos A, Khozin S, Mandl KD. "Developing and adopting safe and
//!   effective digital biomarkers to improve patient outcomes." npj Digital
//!   Medicine 2019. <https://www.nature.com/articles/s41746-019-0090-4>
//! - Digital Medicine Society (DiMe), digital endpoints resources.
//!   <https://dimesociety.org/>
//!
//! Topic doc: health-economics-metrics/topics/digital-endpoints-and-biomarkers.md

/// Required sample size from the power relation N = k × σ² / Δ².
///
/// `k` bundles the power/significance constants (≈16 for 80% power at
/// α = 0.05 in the classic two-arm approximation). Units of `variance` and
/// `detectable_effect` must be consistent (σ² in squared units of Δ).
///
/// # Arguments
///
/// * `variance` — variance σ² of the change estimate.
/// * `detectable_effect` — effect size Δ the trial must detect.
/// * `k` — power/significance constant.
///
/// # Returns
///
/// `Some(N)`, or `None` when `detectable_effect` is zero (a null hypothesis
/// of no effect needs infinite N — undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::digital_endpoints_and_biomarkers::required_sample_size;
///
/// // σ² = 5, Δ = 1, k = 16: N = 80. Cutting σ² to 1 cuts N to 16.
/// assert_eq!(required_sample_size(5.0, 1.0, 16.0), Some(80.0));
/// assert_eq!(required_sample_size(1.0, 1.0, 16.0), Some(16.0));
/// assert_eq!(required_sample_size(1.0, 0.0, 16.0), None);
/// ```
pub fn required_sample_size(variance: f64, detectable_effect: f64, k: f64) -> Option<f64> {
    if detectable_effect == 0.0 {
        None
    } else {
        // N = k × σ² / Δ²
        Some(k * variance / (detectable_effect * detectable_effect))
    }
}

/// Sampling density ratio: digital measurements per patient-year over clinic
/// measurements per patient-year.
///
/// A first-order indicator of how much denser the passive stream is than
/// episodic clinic visits (the variance benefit is downstream of this).
///
/// # Arguments
///
/// * `digital_measurements_per_year` — passive sensor measurements per
///   patient-year.
/// * `clinic_measurements_per_year` — clinic-visit measurements per
///   patient-year.
///
/// # Returns
///
/// `Some(ratio)`, or `None` when the clinic count is zero (ratio undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::digital_endpoints_and_biomarkers::sampling_density_ratio;
///
/// // ~200 passive vs 4 clinic measurements/patient/year: 50× denser.
/// assert_eq!(sampling_density_ratio(200.0, 4.0), Some(50.0));
/// ```
pub fn sampling_density_ratio(
    digital_measurements_per_year: f64,
    clinic_measurements_per_year: f64,
) -> Option<f64> {
    if clinic_measurements_per_year == 0.0 {
        None
    } else {
        Some(digital_measurements_per_year / clinic_measurements_per_year)
    }
}

/// Improvement in detectable effect size at fixed power when variance falls.
///
/// From N ∝ σ²/Δ²: at fixed N and power, Δ scales with σ, so a
/// `variance_reduction_fold`-fold fall in σ² improves the detectable effect
/// by √fold.
///
/// # Arguments
///
/// * `variance_reduction_fold` — factor by which σ² falls (e.g. 5.0 for a
///   5× fall).
///
/// # Returns
///
/// The multiplicative improvement in detectable effect size (dimensionless).
///
/// # Examples
///
/// ```rust
/// use health_economics::digital_endpoints_and_biomarkers::detectable_effect_improvement;
///
/// // A 5× variance fall improves the detectable effect √5 ≈ 2.2×.
/// assert!((detectable_effect_improvement(5.0) - 2.236).abs() < 1e-3);
/// ```
pub fn detectable_effect_improvement(variance_reduction_fold: f64) -> f64 {
    // Δ ∝ σ = √(σ²): a fold-change in variance improves Δ by its square root.
    variance_reduction_fold.sqrt()
}

/// Sample size ratio (new/old) at a fixed hypothesis Δ when variance changes.
///
/// Since N ∝ σ² at fixed Δ and power, the ratio is σ²_new / σ²_old; a value
/// of 0.2 means the new design needs one fifth of the patients.
///
/// # Arguments
///
/// * `variance_new` — variance of the change estimate under the new (dense)
///   design.
/// * `variance_old` — variance under the old (episodic) design.
///
/// # Returns
///
/// `Some(ratio)`, or `None` when `variance_old` is zero (ratio undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::digital_endpoints_and_biomarkers::sample_size_ratio;
///
/// // Variance falling 5× cuts N to 1/5 at the same Δ.
/// assert_eq!(sample_size_ratio(1.0, 5.0), Some(0.2));
/// ```
pub fn sample_size_ratio(variance_new: f64, variance_old: f64) -> Option<f64> {
    if variance_old == 0.0 {
        None
    } else {
        Some(variance_new / variance_old)
    }
}

/// Trial cost saved by enrolling fewer patients.
///
/// Multiplies the patients cut from the enrollment target by the fully-loaded
/// cost per enrolled patient.
///
/// # Arguments
///
/// * `patients_cut` — reduction in enrolled patients.
/// * `cost_per_enrolled_patient` — cost per enrolled patient (£; ~£25,000 in
///   the worked example).
///
/// # Returns
///
/// Trial cost saving (£).
///
/// # Examples
///
/// ```rust
/// use health_economics::digital_endpoints_and_biomarkers::trial_cost_saving;
///
/// // Cutting 200 patients at £25,000 each saves £5M per trial —
/// // the commercial case for a £1–2M validation investment.
/// assert_eq!(trial_cost_saving(200.0, 25_000.0), 5_000_000.0);
/// ```
pub fn trial_cost_saving(patients_cut: f64, cost_per_enrolled_patient: f64) -> f64 {
    patients_cut * cost_per_enrolled_patient
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "Clinic endpoint: 4 measurements/patient/year ...
    // Digital endpoint: ~200 passive measurements/patient/year".
    #[test]
    fn digital_sampling_is_50_times_denser_than_clinic() {
        // ~200 passive vs 4 clinic measurements/patient/year
        assert!((sampling_density_ratio(200.0, 4.0).unwrap() - 50.0).abs() < 1e-9);
    }

    // Worked example: "detectable effect size at fixed power improves
    // ~√5 ≈ 2.2×".
    #[test]
    fn five_fold_variance_fall_improves_detectable_effect_about_2_2x() {
        // √5 ≈ 2.2×
        assert!((detectable_effect_improvement(5.0) - 2.2).abs() < 0.05);
    }

    // The math section: "N ∝ σ²/Δ²" — a 5× variance fall cuts N to 1/5 at
    // the same hypothesis.
    #[test]
    fn sample_size_scales_with_variance_at_fixed_hypothesis() {
        // N ∝ σ²: variance falling 5× cuts N to 1/5 at the same Δ
        let ratio = sample_size_ratio(1.0, 5.0).unwrap();
        assert!((ratio - 0.2).abs() < 1e-9);
        // and via the power relation directly
        let n_old = required_sample_size(5.0, 1.0, 16.0).unwrap();
        let n_new = required_sample_size(1.0, 1.0, 16.0).unwrap();
        assert!((n_new / n_old - 0.2).abs() < 1e-9);
    }

    // Worked example: "At £25,000 per enrolled patient, cutting 200 patients
    // ≈ £5M saved per trial".
    #[test]
    fn cutting_200_patients_at_25k_saves_5_million() {
        // 200 × £25,000 = £5M saved per trial
        assert!((trial_cost_saving(200.0, 25_000.0) - 5_000_000.0).abs() < 1e-9);
    }

    // Worked example: "the commercial case for the validation investment
    // (itself perhaps £1–2M)".
    #[test]
    fn saving_exceeds_the_validation_investment() {
        // Commercial case: £5M saving vs a £1–2M validation investment
        let saving = trial_cost_saving(200.0, 25_000.0);
        assert!(saving > 2_000_000.0);
    }

    // Edge case: Δ = 0 has no defined sample size.
    #[test]
    fn required_sample_size_is_none_for_zero_effect() {
        assert!(required_sample_size(1.0, 0.0, 16.0).is_none());
    }
}
