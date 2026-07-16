//! # Wearable Validation
//!
//! Validation metrics quantify how well a wearable's measurements agree with
//! a clinical gold standard (ECG for heart rate, polysomnography for sleep):
//! MAPE, Lin's concordance correlation coefficient (CCC), and Bland–Altman
//! limits of agreement — plus the operational metrics that gate real-world
//! data quality: wear-time compliance and data completeness.
//!
//! Validation must be reported per activity condition (rest, motion, sleep)
//! and per population — PPG optical sensing degrades with motion artifact,
//! poor contact, and darker skin tones, a documented equity-relevant failure
//! mode.
//!
//! ## Formula
//!
//! ```text
//! MAPE = (1/n) Σ |measured_i − reference_i| / reference_i × 100
//!
//! CCC  = 2·s_xy / (s_x² + s_y² + (x̄ − ȳ)²)
//!
//! Bland–Altman: mean bias ± 1.96 SD limits of agreement
//!
//! Wear-time compliance = time worn / protocol time × 100
//! Data completeness    = observed data points / expected × 100
//!
//! where:
//!   measured_i, reference_i — paired device and gold-standard readings
//!   n         — number of pairs
//!   s_xy      — covariance of measured (x) and reference (y)
//!   s_x², s_y² — variances of measured and reference
//!   x̄, ȳ      — means of measured and reference (the (x̄ − ȳ)² term
//!               penalizes systematic bias)
//!   mean bias — mean of paired differences (measured − reference)
//!   SD        — standard deviation of the paired differences
//! ```
//!
//! ## Why it matters
//!
//! Validation is the precondition for everything downstream: a device that
//! can't prove agreement with reference measurement cannot anchor digital
//! endpoints, support RPM billing, or carry clinical claims. The field's
//! accepted thresholds for heart rate: **MAPE ≤ 5%** (strict) or **≤ 10%**
//! (lenient) against ECG. Reference points from the literature: Oura Gen 3
//! resting HR MAPE 1.67% (CCC 0.97); Fitbit Charge 6 MAPE ~5.5% — consumer
//! devices now span the clinical-grade boundary, which is exactly why the
//! measurement matters per-device and per-condition.
//!
//! ## Example
//!
//! A virtual-ward program selects a monitoring wearable for
//! deteriorating-patient detection at home — alerts trigger on sustained
//! elevated HR, often during activity. Candidate A: rest MAPE 2.1%, exercise
//! MAPE 11.4%. Candidate B: rest 3.8%, exercise 6.9%.
//!
//! ```rust
//! use health_economics::wearable_validation::{
//!     classify_heart_rate_mape, absolute_error_at, annual_false_alert_cost,
//!     HeartRateMapeGrade,
//! };
//!
//! // Candidate A's headline (rest 2.1%) wins the brochure...
//! assert_eq!(classify_heart_rate_mape(2.1), HeartRateMapeGrade::Strict);
//! // ...but at the alert-relevant condition (motion), A fails and B passes.
//! assert_eq!(classify_heart_rate_mape(11.4), HeartRateMapeGrade::Fail);
//! assert_eq!(classify_heart_rate_mape(6.9), HeartRateMapeGrade::Lenient);
//!
//! // A's 11.4% error at HR 100 = ±11 bpm — spanning the entire alert
//! // threshold band.
//! let err = absolute_error_at(11.4, 100.0);
//! assert!((err - 11.0).abs() < 0.5);
//!
//! // False-alert economics: 500 patients × 2 extra false alerts/week × £40
//! // = £2.08M/year of error cost from choosing the wrong validation number.
//! let cost = annual_false_alert_cost(500.0, 2.0, 40.0);
//! assert_eq!(cost, 2_080_000.0);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineers consume validation data when choosing sensors and *produce*
//!   it when building measurement features — both roles need the same
//!   discipline.
//! - Test at the deployment condition, not the demo condition — the software
//!   analogue is benchmarking on your production workload, not the vendor's.
//! - Wear-time and completeness are product-engineering outcomes: comfort,
//!   battery life, charging ritual design, and sync reliability determine
//!   whether the 16-days-in-30 RPM billing gate is met and whether trial
//!   datasets are analyzable.
//! - Treat missingness as a designed signal: distinguish "not worn," "worn
//!   but no signal," and "sync failed" in the schema from day one —
//!   collapsed into null, they poison every downstream analysis.
//!
//! ## Pitfalls
//!
//! - **Aggregate MAPE hiding condition-specific failure** — the worked
//!   example's trap.
//! - **Validation population ≠ deployment population**: age, skin tone,
//!   tremor, obesity all shift optical-sensor error; check the study
//!   demographics.
//! - **Correlation reported where agreement is needed**: high Pearson r with
//!   systematic bias still misclassifies against absolute thresholds —
//!   insist on CCC/Bland–Altman.
//! - **Completeness inflated by imputation**: filled gaps reported as
//!   observed data.
//!
//! ## Sources
//!
//! - Consumer wearable HR validation (Oura Gen 3/4).
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC12367097/>
//! - Wearable validity thresholds (MAPE standards).
//!   <https://formative.jmir.org/2025/1/e70835>
//! - Multi-device validation studies.
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC6431828/>
//!
//! Topic doc: health-economics-metrics/topics/wearable-validation.md

/// Mean absolute percentage error (%) of `measured` against `reference`.
///
/// MAPE = (1/n) Σ |measured − reference| / reference × 100. For heart rate
/// the accepted validation bars are ≤ 5% (strict) and ≤ 10% (lenient)
/// against ECG; report it per activity condition, not in aggregate.
///
/// # Arguments
///
/// * `measured` — device readings (same units as `reference`, e.g. bpm).
/// * `reference` — paired gold-standard readings (e.g. ECG heart rate).
///
/// # Returns
///
/// `Some(mape)` in percent; `None` if the slices are empty, differ in
/// length, or any reference value is zero (the percentage error would be
/// undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::mape_percent;
///
/// // Errors of 2, 3, and 1 bpm against a 100-bpm reference → mean 2%.
/// let measured = [102.0, 97.0, 101.0];
/// let reference = [100.0, 100.0, 100.0];
/// let mape = mape_percent(&measured, &reference).unwrap();
/// assert!((mape - 2.0).abs() < 1e-9);
///
/// // A zero reference value makes the percentage undefined.
/// assert!(mape_percent(&[1.0], &[0.0]).is_none());
/// ```
pub fn mape_percent(measured: &[f64], reference: &[f64]) -> Option<f64> {
    if measured.is_empty() || measured.len() != reference.len() {
        return None;
    }
    let mut sum = 0.0;
    for (m, r) in measured.iter().zip(reference.iter()) {
        if *r == 0.0 {
            return None;
        }
        // Absolute percentage error term: |measured − reference| / reference.
        sum += (m - r).abs() / r;
    }
    // Mean of the per-pair terms, scaled to percent.
    Some(sum / measured.len() as f64 * 100.0)
}

/// Lin's concordance correlation coefficient (CCC) of `measured` against
/// `reference`.
///
/// CCC measures agreement, not just association: it is the Pearson
/// correlation penalized by location and scale shift, so a device with
/// perfect correlation but a constant systematic bias scores below 1. Uses
/// population (1/n) moments. Insist on CCC (or Bland–Altman) whenever
/// absolute thresholds matter — high Pearson r with systematic bias still
/// misclassifies.
///
/// # Arguments
///
/// * `measured` — device readings (x series).
/// * `reference` — paired gold-standard readings (y series).
///
/// # Returns
///
/// `Some(ccc)` in [−1, 1]; `None` if the slices are empty, differ in
/// length, or the denominator is zero (both series constant with equal
/// means).
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::concordance_correlation;
///
/// let reference = [60.0, 70.0, 80.0, 90.0, 100.0];
///
/// // Perfect agreement: CCC = 1.
/// let perfect = concordance_correlation(&reference, &reference).unwrap();
/// assert!((perfect - 1.0).abs() < 1e-9);
///
/// // A constant +10 bpm bias keeps Pearson r = 1 but pulls CCC to 0.8:
/// // 2·200 / (200 + 200 + 10²) = 0.8.
/// let biased: Vec<f64> = reference.iter().map(|r| r + 10.0).collect();
/// let ccc = concordance_correlation(&biased, &reference).unwrap();
/// assert!((ccc - 0.8).abs() < 1e-9);
/// ```
pub fn concordance_correlation(measured: &[f64], reference: &[f64]) -> Option<f64> {
    let n = measured.len();
    if n == 0 || n != reference.len() {
        return None;
    }
    let nf = n as f64;
    let mean_x = measured.iter().sum::<f64>() / nf;
    let mean_y = reference.iter().sum::<f64>() / nf;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    let mut cov = 0.0;
    for (x, y) in measured.iter().zip(reference.iter()) {
        // Accumulate sums of squares/products about the means.
        var_x += (x - mean_x) * (x - mean_x);
        var_y += (y - mean_y) * (y - mean_y);
        cov += (x - mean_x) * (y - mean_y);
    }
    // Population (1/n) moments, per Lin's original definition.
    var_x /= nf;
    var_y /= nf;
    cov /= nf;
    // Denominator s_x² + s_y² + (x̄ − ȳ)²: the squared-mean-difference term
    // is the location-shift (systematic bias) penalty that distinguishes
    // CCC from Pearson correlation.
    let denom = var_x + var_y + (mean_x - mean_y) * (mean_x - mean_y);
    if denom == 0.0 {
        None
    } else {
        // CCC = 2·s_xy / (s_x² + s_y² + (x̄ − ȳ)²).
        Some(2.0 * cov / denom)
    }
}

/// Bland–Altman agreement summary: mean bias and 95% limits of agreement.
///
/// Together the three fields describe where ~95% of device−reference
/// differences are expected to fall, revealing systematic bias and the
/// spread of disagreement (and, plotted against magnitude, whether error
/// depends on the value's size).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlandAltman {
    /// Mean of the paired differences (measured − reference) — the
    /// systematic bias: positive means the device reads high on average.
    pub mean_bias: f64,
    /// Lower 95% limit of agreement: bias − 1.96 × SD of the differences.
    pub lower_limit: f64,
    /// Upper 95% limit of agreement: bias + 1.96 × SD of the differences.
    pub upper_limit: f64,
}

/// Bland–Altman analysis of `measured` against `reference`.
///
/// Computes the mean of the paired differences (measured − reference) and
/// the 95% limits of agreement, bias ± 1.96 × SD, using the sample standard
/// deviation (n − 1). Roughly 95% of differences are expected to lie within
/// the limits if the differences are approximately normal.
///
/// # Arguments
///
/// * `measured` — device readings.
/// * `reference` — paired gold-standard readings.
///
/// # Returns
///
/// `Some(BlandAltman)`; `None` if the slices differ in length or hold fewer
/// than two pairs (the sample SD needs n ≥ 2).
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::bland_altman;
///
/// // A constant +10 bpm bias: mean bias 10, zero-width limits.
/// let reference = [60.0, 70.0, 80.0];
/// let measured = [70.0, 80.0, 90.0];
/// let ba = bland_altman(&measured, &reference).unwrap();
/// assert!((ba.mean_bias - 10.0).abs() < 1e-9);
/// assert!((ba.lower_limit - 10.0).abs() < 1e-9);
/// assert!((ba.upper_limit - 10.0).abs() < 1e-9);
///
/// // A single pair cannot yield a sample SD.
/// assert!(bland_altman(&[1.0], &[1.0]).is_none());
/// ```
pub fn bland_altman(measured: &[f64], reference: &[f64]) -> Option<BlandAltman> {
    let n = measured.len();
    if n < 2 || n != reference.len() {
        return None;
    }
    let nf = n as f64;
    // Paired differences d_i = measured_i − reference_i.
    let diffs: Vec<f64> = measured
        .iter()
        .zip(reference.iter())
        .map(|(m, r)| m - r)
        .collect();
    let mean_bias = diffs.iter().sum::<f64>() / nf;
    // Sample variance of the differences (n − 1 denominator).
    let var = diffs
        .iter()
        .map(|d| (d - mean_bias) * (d - mean_bias))
        .sum::<f64>()
        / (nf - 1.0);
    let sd = var.sqrt();
    // 95% limits of agreement: bias ± 1.96 SD (normal-quantile convention).
    Some(BlandAltman {
        mean_bias,
        lower_limit: mean_bias - 1.96 * sd,
        upper_limit: mean_bias + 1.96 * sd,
    })
}

/// Wear-time compliance (%): time worn / protocol time × 100.
///
/// An operational gate on real-world data quality: e.g. RPM billing
/// typically requires 16 days of wear in a 30-day protocol window. Both
/// arguments must share a time unit (days, hours).
///
/// # Arguments
///
/// * `time_worn` — time the device was actually worn.
/// * `protocol_time` — time the protocol required it to be worn (same unit).
///
/// # Returns
///
/// `Some(percent)`; `None` when `protocol_time` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::wear_time_compliance_percent;
///
/// // 16 days worn in a 30-day protocol = 53.33% compliance.
/// let compliance = wear_time_compliance_percent(16.0, 30.0).unwrap();
/// assert!((compliance - 53.33).abs() < 0.01);
/// ```
pub fn wear_time_compliance_percent(time_worn: f64, protocol_time: f64) -> Option<f64> {
    if protocol_time == 0.0 {
        None
    } else {
        Some(time_worn / protocol_time * 100.0)
    }
}

/// Data completeness (%): observed data points / expected × 100.
///
/// The second operational gate: how much of the expected data stream
/// actually arrived. Beware completeness inflated by imputation — filled
/// gaps reported as observed data.
///
/// # Arguments
///
/// * `observed_points` — data points actually received (not imputed).
/// * `expected_points` — data points the protocol expected.
///
/// # Returns
///
/// `Some(percent)`; `None` when `expected_points` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::data_completeness_percent;
///
/// // 900 of 1,000 expected points = 90% completeness.
/// let completeness = data_completeness_percent(900.0, 1_000.0).unwrap();
/// assert!((completeness - 90.0).abs() < 1e-9);
/// ```
pub fn data_completeness_percent(observed_points: f64, expected_points: f64) -> Option<f64> {
    if expected_points == 0.0 {
        None
    } else {
        Some(observed_points / expected_points * 100.0)
    }
}

/// Heart-rate MAPE grade against the field's accepted thresholds.
///
/// The literature's reference points span these bands: Oura Gen 3 resting HR
/// MAPE 1.67% (strict), Fitbit Charge 6 ~5.5% (lenient) — consumer devices
/// now straddle the clinical-grade boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartRateMapeGrade {
    /// MAPE ≤ 5%: meets the strict clinical-grade threshold against ECG.
    Strict,
    /// 5% < MAPE ≤ 10%: meets only the lenient threshold.
    Lenient,
    /// MAPE > 10%: fails both accepted thresholds.
    Fail,
}

/// Classify a heart-rate MAPE (%) against the accepted validation
/// thresholds.
///
/// Strict ≤ 5%, lenient ≤ 10%, fail above 10% — the field's accepted bars
/// for heart rate against ECG. Classify per activity condition (rest,
/// motion, sleep); an aggregate grade can hide condition-specific failure.
///
/// # Arguments
///
/// * `mape_percent` — heart-rate MAPE in percent (e.g. from
///   [`mape_percent`]).
///
/// # Returns
///
/// The [`HeartRateMapeGrade`] band the value falls in.
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::{
///     classify_heart_rate_mape, HeartRateMapeGrade,
/// };
///
/// // Oura Gen 3 resting HR (1.67%) is clinical grade; candidate B's
/// // exercise MAPE (6.9%) is lenient; candidate A's (11.4%) fails.
/// assert_eq!(classify_heart_rate_mape(1.67), HeartRateMapeGrade::Strict);
/// assert_eq!(classify_heart_rate_mape(6.9), HeartRateMapeGrade::Lenient);
/// assert_eq!(classify_heart_rate_mape(11.4), HeartRateMapeGrade::Fail);
/// ```
pub fn classify_heart_rate_mape(mape_percent: f64) -> HeartRateMapeGrade {
    if mape_percent <= 5.0 {
        HeartRateMapeGrade::Strict
    } else if mape_percent <= 10.0 {
        HeartRateMapeGrade::Lenient
    } else {
        HeartRateMapeGrade::Fail
    }
}

/// Absolute error implied by a MAPE (%) at a given true value.
///
/// Converts a relative error into the measurement's own units — e.g. 11.4%
/// MAPE at HR 100 bpm ≈ ±11 bpm, enough to span an entire alert threshold
/// band.
///
/// # Arguments
///
/// * `mape_percent` — mean absolute percentage error in percent.
/// * `true_value` — the reference value at which to evaluate the error
///   (e.g. 100 bpm).
///
/// # Returns
///
/// Expected absolute error, in the same units as `true_value`.
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::absolute_error_at;
///
/// // Candidate A's 11.4% exercise MAPE at HR 100 → ±11.4 bpm of error.
/// let err = absolute_error_at(11.4, 100.0);
/// assert!((err - 11.4).abs() < 1e-9);
/// ```
pub fn absolute_error_at(mape_percent: f64, true_value: f64) -> f64 {
    mape_percent / 100.0 * true_value
}

/// Annual cost of extra false alerts from an inaccurate device.
///
/// Each false alert costs a response (e.g. a nurse callout ≈ £40).
/// Computes patients × extra false alerts per patient per week × cost per
/// alert × 52 weeks.
///
/// # Arguments
///
/// * `patient_count` — patients monitored on the program.
/// * `extra_false_alerts_per_patient_per_week` — additional false alerts per
///   patient per week attributable to device error.
/// * `cost_per_alert` — cost of responding to one alert (£, e.g. £40 per
///   nurse callout).
///
/// # Returns
///
/// Annual false-alert cost (£/year).
///
/// # Examples
///
/// ```rust
/// use health_economics::wearable_validation::annual_false_alert_cost;
///
/// // 500 patients × 2 extra false alerts/week × £40 × 52 = £2.08M/year of
/// // error cost from choosing the wrong validation number.
/// let cost = annual_false_alert_cost(500.0, 2.0, 40.0);
/// assert_eq!(cost, 2_080_000.0);
/// ```
pub fn annual_false_alert_cost(
    patient_count: f64,
    extra_false_alerts_per_patient_per_week: f64,
    cost_per_alert: f64,
) -> f64 {
    patient_count * extra_false_alerts_per_patient_per_week * cost_per_alert * 52.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "A's 11.4% error at HR 100 = ±11 bpm — spanning the
    // entire alert threshold band".
    /// Candidate A's 11.4% exercise MAPE at HR 100 means ±11 bpm of error —
    /// spanning the entire alert threshold band.
    #[test]
    fn candidate_a_motion_error_at_hr_100_is_about_11_bpm() {
        let err = absolute_error_at(11.4, 100.0);
        assert!((err - 11.0).abs() < 0.5); // exact value 11.4 bpm
    }

    // Worked example: "500 patients × 2 extra false alerts/week × £40
    // = £2.08M/year of error cost".
    /// 500 patients × 2 extra false alerts/week × £40 = £2.08M/year.
    #[test]
    fn false_alert_economics_is_2_08_million_per_year() {
        let cost = annual_false_alert_cost(500.0, 2.0, 40.0);
        assert!((cost - 2_080_000.0).abs() < 1e-9);
    }

    // Worked example + literature: Oura 1.67% and candidate rest MAPEs 2.1%
    // and 3.8% are strict; Fitbit ~5.5% and candidate B exercise 6.9% are
    // lenient; candidate A exercise 11.4% fails.
    /// Candidate grades from the worked example and the reference literature:
    /// Oura rest 1.67% strict; Fitbit 5.5% and candidate B exercise 6.9%
    /// lenient; candidate A exercise 11.4% fails.
    #[test]
    fn mape_grades_match_accepted_thresholds() {
        assert_eq!(classify_heart_rate_mape(1.67), HeartRateMapeGrade::Strict);
        assert_eq!(classify_heart_rate_mape(2.1), HeartRateMapeGrade::Strict);
        assert_eq!(classify_heart_rate_mape(3.8), HeartRateMapeGrade::Strict);
        assert_eq!(classify_heart_rate_mape(5.5), HeartRateMapeGrade::Lenient);
        assert_eq!(classify_heart_rate_mape(6.9), HeartRateMapeGrade::Lenient);
        assert_eq!(classify_heart_rate_mape(11.4), HeartRateMapeGrade::Fail);
    }

    // Verifies the doc's MAPE formula "(1/n) Σ |m − r| / r × 100" on a
    // hand-computable series.
    /// MAPE formula check on a hand-computable series:
    /// errors 2/100, 3/100, 1/100 → mean 2% .
    #[test]
    fn mape_matches_hand_computation() {
        let measured = [102.0, 97.0, 101.0];
        let reference = [100.0, 100.0, 100.0];
        let mape = mape_percent(&measured, &reference).unwrap();
        assert!((mape - 2.0).abs() < 1e-9);
    }

    // Verifies the doc's CCC definition: "agreement including both
    // correlation and systematic bias (Pearson r penalized by
    // location/scale shift)".
    /// Perfect agreement gives CCC = 1; a constant systematic bias pulls it
    /// below the Pearson correlation (which stays 1).
    #[test]
    fn ccc_is_one_for_identity_and_penalizes_bias() {
        let reference = [60.0, 70.0, 80.0, 90.0, 100.0];
        let ccc_perfect = concordance_correlation(&reference, &reference).unwrap();
        assert!((ccc_perfect - 1.0).abs() < 1e-9);

        let biased: Vec<f64> = reference.iter().map(|r| r + 10.0).collect();
        let ccc_biased = concordance_correlation(&biased, &reference).unwrap();
        // var = 200, cov = 200, bias² = 100 → 2·200 / (200 + 200 + 100) = 0.8
        assert!((ccc_biased - 0.8).abs() < 1e-9);
    }

    // Verifies the doc's Bland–Altman line "mean bias ± 1.96 SD limits of
    // agreement" on a pure constant-bias series (SD = 0).
    /// Bland–Altman on a constant +10 bias: bias 10, zero-width limits.
    #[test]
    fn bland_altman_reports_constant_bias() {
        let reference = [60.0, 70.0, 80.0];
        let measured = [70.0, 80.0, 90.0];
        let ba = bland_altman(&measured, &reference).unwrap();
        assert!((ba.mean_bias - 10.0).abs() < 1e-9);
        assert!((ba.lower_limit - 10.0).abs() < 1e-9);
        assert!((ba.upper_limit - 10.0).abs() < 1e-9);
    }

    // Doc's operational gates: the 16-days-in-30 RPM wear gate (53.33%) and
    // "observed data points / expected × 100" completeness (900/1,000 = 90%).
    /// Operational gates: 16 days worn in 30 = 53.33% compliance; 900 of
    /// 1,000 expected points = 90% completeness.
    #[test]
    fn operational_gates_compute_percentages() {
        let compliance = wear_time_compliance_percent(16.0, 30.0).unwrap();
        assert!((compliance - 53.333_333_333_333_336).abs() < 1e-9);
        let completeness = data_completeness_percent(900.0, 1_000.0).unwrap();
        assert!((completeness - 90.0).abs() < 1e-9);
    }

    // Edge-case contract: empty/mismatched slices, zero references, single
    // pairs, and zero denominators all return None.
    /// Degenerate inputs return None rather than panicking.
    #[test]
    fn degenerate_inputs_return_none() {
        assert!(mape_percent(&[], &[]).is_none());
        assert!(mape_percent(&[1.0], &[0.0]).is_none());
        assert!(mape_percent(&[1.0, 2.0], &[1.0]).is_none());
        assert!(concordance_correlation(&[], &[]).is_none());
        assert!(concordance_correlation(&[5.0, 5.0], &[5.0, 5.0]).is_none());
        assert!(bland_altman(&[1.0], &[1.0]).is_none());
        assert!(wear_time_compliance_percent(10.0, 0.0).is_none());
        assert!(data_completeness_percent(10.0, 0.0).is_none());
    }
}
