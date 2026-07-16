//! # Retention and Churn
//!
//! Retention measures what fraction of a user cohort is still active N days
//! after starting (D1/D7/D30 curves); churn is its complement. The brutal
//! digital-health baseline: roughly 90% of health-app users abandon within
//! 30 days — digital health D30 retention runs ~3–4% against an all-app
//! average of ~6%.
//!
//! The health-economics move is weighting benefits by the retention curve:
//! benefits accrue only to users still present at each point in time, not
//! to everyone acquired.
//!
//! ## Formula
//!
//! ```text
//! Retention_Dn = users active on day n / cohort size × 100
//! Churn rate   = users lost in period / users at period start × 100
//!
//! expected benefit per acquired user = Σ_t retention(t) × benefit rate(t)
//!
//! Cost per retained-at-D30 user = CAC / D30 retention
//!
//! retention(t)     fraction (0..1) of the cohort still active at time t
//! benefit rate(t)  per-user benefit delivered during time t
//! CAC              customer acquisition cost per acquired user
//! ```
//!
//! ## Why it matters
//!
//! Eysenbach named this in 2005: the "law of attrition" — losing users at
//! high rates is an intrinsic, structural property of eHealth interventions,
//! not an implementation bug, with attrition in eHealth trials routinely
//! exceeding 50%. Retention defines the treatment window within which any
//! benefit can be delivered: CAC paid per user who stays 12 days delivers
//! neither LTV nor QALYs, and at 4% D30 a £5 CAC is really £125 per retained
//! user. Any economic model for a consumer health product that doesn't
//! weight benefits by the retention curve is describing a product that
//! doesn't exist.
//!
//! ## Example
//!
//! A mental-health app: trial showed 0.02 QALYs gained per user completing
//! 8 weeks. Deployment cohort of 100,000 downloads; week-8 retention 4%:
//!
//! ```rust
//! use health_economics::retention_and_churn::{
//!     completers, qalys_delivered, monetized_health_value,
//!     health_value_per_download, retention_improvement_value,
//! };
//!
//! // Completers = 100,000 × 0.04 = 4,000
//! let done = completers(100_000.0, 0.04);
//! assert!((done - 4_000.0).abs() < 1e-9);
//!
//! // QALYs delivered = 4,000 × 0.02 = 80 (not 100,000 × 0.02 = 2,000)
//! let qalys = qalys_delivered(done, 0.02);
//! assert!((qalys - 80.0).abs() < 1e-9);
//!
//! // At £20,000/QALY = £1.6M of health value (not £40M)
//! let value = monetized_health_value(qalys, 20_000.0);
//! assert!((value - 1_600_000.0).abs() < 1e-9);
//!
//! // Per-download health value = £16 — 4% of the naive claim.
//! let per_download = health_value_per_download(value, 100_000.0).unwrap();
//! assert!((per_download - 16.0).abs() < 1e-9);
//!
//! // Moving week-8 completion 4% → 6% adds 40 QALYs/year ≈ £800k.
//! let gain = retention_improvement_value(100_000.0, 0.04, 0.06, 0.02, 20_000.0);
//! assert!((gain - 800_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Retention is the metric where product engineering most directly
//!   manufactures health value: retention engineering IS health production.
//! - The practices that move it are ordinary — onboarding time-to-first-value,
//!   re-engagement design, performance.
//! - Programs with a defined end (8 weeks, then graduation) should measure
//!   completion, not perpetual DAU — aligning the metric with the clinical
//!   model instead of the ad-funded attention model.
//! - Survival analysis is the right toolkit (the same Kaplan-Meier math as
//!   life-years gained).
//! - Segment curves by acquisition channel: channel mix changes retention
//!   more than most features do.
//!
//! ## Pitfalls
//!
//! - Intention-to-treat laundering in reverse: trials report completers;
//!   deployment economics must count everyone acquired (Eysenbach's core
//!   warning).
//! - Retention theater: notification-driven "active" users who never perform
//!   the therapeutic action.
//! - Comparing curves across definitions: "active" defined as open vs
//!   meaningful action shifts D30 by multiples.
//! - Ignoring who churns: if the sickest churn fastest, per-user benefits
//!   fall as retention improves among the healthy — pair curves with
//!   case-mix.
//!
//! ## Sources
//!
//! - Eysenbach G. "The law of attrition." JMIR 2005;7(1):e11.
//!   <https://www.jmir.org/2005/1/e11/>
//! - Mobile app retention benchmarks.
//!   <https://uxcam.com/blog/mobile-app-retention-benchmarks/>
//! - Healthcare product benchmarks.
//!   <https://userpilot.com/blog/healthcare-product-metrics-benchmark-report/>
//!
//! Topic doc: health-economics-metrics/topics/retention-and-churn.md

/// Retention at day n, as a percentage of the starting cohort.
///
/// This is the Dn point on a retention curve (e.g. D7, D30). Benchmarks:
/// digital-health D30 runs ~3–4% versus an all-app average of ~6%.
///
/// # Arguments
///
/// * `users_active_on_day_n` — users from the cohort still active on day n.
/// * `cohort_size` — number of users who started (everyone acquired).
///
/// # Returns
///
/// Retention as a percentage (0–100), or `None` if `cohort_size` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::retention_percent;
///
/// // Doc worked example: D30 retention 8% on a 100,000 cohort.
/// let d30 = retention_percent(8_000.0, 100_000.0).unwrap();
/// assert!((d30 - 8.0).abs() < 1e-9);
/// assert!(retention_percent(1.0, 0.0).is_none());
/// ```
pub fn retention_percent(users_active_on_day_n: f64, cohort_size: f64) -> Option<f64> {
    if cohort_size == 0.0 {
        None
    } else {
        Some(users_active_on_day_n / cohort_size * 100.0)
    }
}

/// Churn rate over a period, as a percentage of users at period start.
///
/// Churn is the complement of retention over the same period and the same
/// activity definition.
///
/// # Arguments
///
/// * `users_lost_in_period` — users who stopped being active during the period.
/// * `users_at_period_start` — users active at the start of the period.
///
/// # Returns
///
/// Churn as a percentage (0–100), or `None` if the period started with zero
/// users.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::churn_rate_percent;
///
/// // 92,000 of 100,000 lost by D30 → 92% churn (complement of 8% retention).
/// let churn = churn_rate_percent(92_000.0, 100_000.0).unwrap();
/// assert!((churn - 92.0).abs() < 1e-9);
/// ```
pub fn churn_rate_percent(users_lost_in_period: f64, users_at_period_start: f64) -> Option<f64> {
    if users_at_period_start == 0.0 {
        None
    } else {
        Some(users_lost_in_period / users_at_period_start * 100.0)
    }
}

/// Expected benefit per acquired user: Σ_t retention(t) × benefit rate(t).
///
/// This is the benefit-weighting move: approximately the area under the
/// retention curve times the per-time benefit — NOT trial benefit × 100% of
/// acquired users. Slices are zipped; extra elements in the longer slice are
/// ignored.
///
/// # Arguments
///
/// * `retention_fractions` — `retention_fractions[t]` is the fraction (0..1)
///   of the cohort still active at time t.
/// * `benefit_rates` — `benefit_rates[t]` is the per-user benefit delivered
///   during time t (any currency or outcome unit).
///
/// # Returns
///
/// Expected benefit per acquired user, in the units of `benefit_rates`.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::expected_benefit_per_acquired_user;
///
/// // Retention curve 100%, D7 25%, D30 8%, week-8 4%; benefit 4 per period.
/// let expected = expected_benefit_per_acquired_user(&[1.0, 0.25, 0.08, 0.04], &[4.0; 4]);
/// assert!((expected - 5.48).abs() < 1e-9); // 1.37 × 4
/// ```
pub fn expected_benefit_per_acquired_user(
    retention_fractions: &[f64],
    benefit_rates: &[f64],
) -> f64 {
    retention_fractions
        .iter()
        .zip(benefit_rates.iter())
        .map(|(r, b)| r * b)
        .sum()
}

/// Cost per retained user: CAC divided by the retention fraction at the
/// milestone of interest (e.g. D30).
///
/// This restates acquisition spend in terms of users who actually stay long
/// enough to matter.
///
/// # Arguments
///
/// * `cac` — customer acquisition cost per acquired user, currency units.
/// * `retention_fraction` — fraction (0..1) retained at the milestone
///   (e.g. 0.04 for 4% D30).
///
/// # Returns
///
/// Cost per retained user, or `None` if `retention_fraction` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::cost_per_retained_user;
///
/// // Doc: at 4% D30, a £5 CAC is really £125 per retained user.
/// let cost = cost_per_retained_user(5.0, 0.04).unwrap();
/// assert!((cost - 125.0).abs() < 1e-9);
/// ```
pub fn cost_per_retained_user(cac: f64, retention_fraction: f64) -> Option<f64> {
    if retention_fraction == 0.0 {
        None
    } else {
        Some(cac / retention_fraction)
    }
}

/// Number of users completing the full program dose.
///
/// For a program with a defined end (e.g. an 8-week course), the completion
/// fraction is the retention at that milestone (e.g. week-8 retention).
///
/// # Arguments
///
/// * `cohort_size` — number of users acquired.
/// * `completion_fraction` — fraction (0..1) completing the full dose.
///
/// # Returns
///
/// Expected number of completers.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::completers;
///
/// // Doc: 100,000 × 0.04 = 4,000 completers.
/// assert!((completers(100_000.0, 0.04) - 4_000.0).abs() < 1e-9);
/// ```
pub fn completers(cohort_size: f64, completion_fraction: f64) -> f64 {
    cohort_size * completion_fraction
}

/// QALYs actually delivered: completers × QALYs gained per completer.
///
/// The trial effect applies only to those who finish the dose — applying it
/// to everyone acquired is the naive claim this module exists to prevent.
///
/// # Arguments
///
/// * `completers` — number of users completing the full program dose.
/// * `qalys_per_completer` — QALYs gained per completer (from trial evidence).
///
/// # Returns
///
/// Total QALYs delivered by the deployment.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::qalys_delivered;
///
/// // Doc: 4,000 × 0.02 = 80 QALYs (not 100,000 × 0.02 = 2,000).
/// assert!((qalys_delivered(4_000.0, 0.02) - 80.0).abs() < 1e-9);
/// ```
pub fn qalys_delivered(completers: f64, qalys_per_completer: f64) -> f64 {
    completers * qalys_per_completer
}

/// Monetized health value: QALYs delivered × willingness-to-pay per QALY.
///
/// # Arguments
///
/// * `qalys` — QALYs delivered.
/// * `value_per_qaly` — willingness-to-pay threshold per QALY (e.g. £20,000).
///
/// # Returns
///
/// Monetized health value in currency units.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::monetized_health_value;
///
/// // Doc: 80 QALYs at £20,000/QALY = £1.6M of health value (not £40M).
/// assert!((monetized_health_value(80.0, 20_000.0) - 1_600_000.0).abs() < 1e-9);
/// ```
pub fn monetized_health_value(qalys: f64, value_per_qaly: f64) -> f64 {
    qalys * value_per_qaly
}

/// Health value per download: total monetized value spread over everyone
/// acquired.
///
/// This is the number that should set what a payer pays per download — and
/// in the worked example it is 4% of the naive claim.
///
/// # Arguments
///
/// * `total_value` — total monetized health value, currency units.
/// * `cohort_size` — everyone acquired (downloads), not just completers.
///
/// # Returns
///
/// Value per download, or `None` if `cohort_size` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::health_value_per_download;
///
/// // Doc: £1.6M over 100,000 downloads = £16 per download.
/// let per = health_value_per_download(1_600_000.0, 100_000.0).unwrap();
/// assert!((per - 16.0).abs() < 1e-9);
/// ```
pub fn health_value_per_download(total_value: f64, cohort_size: f64) -> Option<f64> {
    if cohort_size == 0.0 {
        None
    } else {
        Some(total_value / cohort_size)
    }
}

/// Monetized value of a retention improvement.
///
/// Values the extra completers created by moving completion from
/// `from_fraction` to `to_fraction`, times QALYs per completer, times value
/// per QALY. This is the "retention engineering IS health production" number.
///
/// # Arguments
///
/// * `cohort_size` — number of users acquired.
/// * `from_fraction` — baseline completion fraction (0..1).
/// * `to_fraction` — improved completion fraction (0..1).
/// * `qalys_per_completer` — QALYs gained per completer.
/// * `value_per_qaly` — willingness-to-pay per QALY.
///
/// # Returns
///
/// Monetized value of the improvement (negative if retention falls).
///
/// # Examples
///
/// ```rust
/// use health_economics::retention_and_churn::retention_improvement_value;
///
/// // Doc: moving week-8 completion 4% → 6% adds 40 QALYs/year ≈ £800k.
/// let gain = retention_improvement_value(100_000.0, 0.04, 0.06, 0.02, 20_000.0);
/// assert!((gain - 800_000.0).abs() < 1e-9);
/// ```
pub fn retention_improvement_value(
    cohort_size: f64,
    from_fraction: f64,
    to_fraction: f64,
    qalys_per_completer: f64,
    value_per_qaly: f64,
) -> f64 {
    // Extra completers (cohort × Δcompletion) → extra QALYs → monetized value.
    cohort_size * (to_fraction - from_fraction) * qalys_per_completer * value_per_qaly
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc worked example: "Completers = 100,000 × 0.04 = 4,000".
    #[test]
    fn completers_are_4000_of_100k_at_4_percent() {
        assert!((completers(100_000.0, 0.04) - 4_000.0).abs() < 1e-9);
    }

    // Doc worked example: "QALYs delivered = 4,000 × 0.02 = 80
    // (not 100,000 × 0.02 = 2,000)".
    #[test]
    fn qalys_delivered_are_80_not_2000() {
        let q = qalys_delivered(completers(100_000.0, 0.04), 0.02);
        assert!((q - 80.0).abs() < 1e-9);
        // The naive claim (100% of downloads benefit) would be 2,000 QALYs.
        let naive = qalys_delivered(100_000.0, 0.02);
        assert!((naive - 2_000.0).abs() < 1e-9);
    }

    // Doc worked example: "At £20,000/QALY = £1.6M of health value (not £40M)".
    #[test]
    fn value_at_20k_per_qaly_is_1_6m_not_40m() {
        let value = monetized_health_value(80.0, 20_000.0);
        assert!((value - 1_600_000.0).abs() < 1e-9);
        let naive = monetized_health_value(2_000.0, 20_000.0);
        assert!((naive - 40_000_000.0).abs() < 1e-9);
    }

    // Doc worked example: "Per-download health value = £16".
    #[test]
    fn per_download_health_value_is_16_pounds() {
        let per = health_value_per_download(1_600_000.0, 100_000.0).unwrap();
        assert!((per - 16.0).abs() < 1e-9);
    }

    // Doc worked example: "moving week-8 completion 4% → 6% adds
    // 40 QALYs/year ≈ £800k".
    #[test]
    fn moving_week8_completion_4_to_6_percent_adds_800k() {
        let gain = retention_improvement_value(100_000.0, 0.04, 0.06, 0.02, 20_000.0);
        // Doc: adds 40 QALYs/year ≈ £800k
        assert!((gain - 800_000.0).abs() < 1e-9);
        // 40 QALYs check via the component functions.
        let extra_qalys = qalys_delivered(completers(100_000.0, 0.06), 0.02)
            - qalys_delivered(completers(100_000.0, 0.04), 0.02);
        assert!((extra_qalys - 40.0).abs() < 1e-9);
    }

    // Doc (The math): "at 4% D30, a £5 CAC is really £125 per retained user".
    #[test]
    fn cac_5_at_4_percent_d30_is_125_per_retained_user() {
        // Doc (The math): at 4% D30, a £5 CAC is really £125 per retained user.
        let cost = cost_per_retained_user(5.0, 0.04).unwrap();
        assert!((cost - 125.0).abs() < 1e-9);
    }

    // Doc worked example: "retention D7 25%, D30 8%, week-8 4%" on 100,000.
    #[test]
    fn retention_curve_percentages_from_worked_example() {
        // D7 25%, D30 8%, week-8 4% on a 100,000 cohort.
        assert!((retention_percent(25_000.0, 100_000.0).unwrap() - 25.0).abs() < 1e-9);
        assert!((retention_percent(8_000.0, 100_000.0).unwrap() - 8.0).abs() < 1e-9);
        assert!((retention_percent(4_000.0, 100_000.0).unwrap() - 4.0).abs() < 1e-9);
        assert!(retention_percent(1.0, 0.0).is_none());
    }

    // Doc (The math): "Churn rate = users lost in period / users at period
    // start × 100" — the complement of the worked example's 8% D30 retention.
    #[test]
    fn churn_is_complement_of_retention() {
        // 92,000 of 100,000 lost by D30 → 92% churn (complement of 8% retention).
        let churn = churn_rate_percent(92_000.0, 100_000.0).unwrap();
        assert!((churn - 92.0).abs() < 1e-9);
        assert!(churn_rate_percent(1.0, 0.0).is_none());
    }

    // Doc (The math): "expected benefit per acquired user =
    // Σ_t retention(t) × benefit rate(t)" using the worked example's curve.
    #[test]
    fn expected_benefit_weights_by_retention_curve() {
        // Benefit rate 4 per period; retention curve 1.0, 0.25, 0.08, 0.04.
        let expected =
            expected_benefit_per_acquired_user(&[1.0, 0.25, 0.08, 0.04], &[4.0, 4.0, 4.0, 4.0]);
        assert!((expected - (1.37 * 4.0)).abs() < 1e-9);
    }
}
