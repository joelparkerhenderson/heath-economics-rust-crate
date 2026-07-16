//! # Engagement Metrics
//!
//! Measures how much users actually use a health app: DAU/MAU stickiness,
//! session frequency and duration, feature usage. In digital health,
//! engagement is not vanity — it is **dose**: the exposure through which any
//! clinical effect must flow. A drug that stays in the bottle heals nobody;
//! an app that stays uninstalled or unopened is the same failure mode.
//!
//! The health-economics upgrade is dose-response framing: trial efficacy was
//! measured at some usage level, and real-world value scales with how close
//! deployment usage gets to that level.
//!
//! ## Formula
//!
//! ```text
//! Stickiness (DAU/MAU) = daily active users / monthly active users × 100
//! Session metrics      = sessions/user/period; avg duration = total time / sessions
//! Feature engagement   = users performing key action / active users
//!
//! Dose-response framing:
//!   realized effect ≈ trial effect × f(actual usage / trial usage)
//!   effective-dose share = users at trial-level usage / registered users
//!
//! DAU / MAU            = daily / monthly active users (counts)
//! key action           = the clinically meaningful action (readings logged,
//!                        lessons completed), not app opens
//! trial effect         = efficacy measured in the pivotal study
//! f(·)                 = dose-response function (from adherence data)
//! ```
//!
//! ## Why it matters
//!
//! Every health-economic claim for a consumer health product multiplies
//! through engagement. Standard product benchmarks: DAU/MAU around **20% is
//! considered healthy** for mobile apps generally, >25% exceptional; health
//! apps often run lower. The multiplication through the engagement funnel to
//! the effective dose is the single most common place digital-health
//! economics inflate.
//!
//! ## Example
//!
//! A blood-pressure app's pivotal study showed a 6 mmHg systolic reduction
//! among users logging ≥4 readings/week. In deployment: 50,000 registered,
//! MAU 20,000 (40%), of whom 7,000 log at trial level.
//!
//! ```
//! use health_economics::engagement_metrics::{
//!     effective_dose_share, overstatement_factor, population_effect,
//! };
//!
//! // Effective-dose users = 7,000 / 50,000 = 14% of the registered base.
//! let share = effective_dose_share(7_000.0, 50_000.0).unwrap();
//! assert!((share - 0.14).abs() < 1e-9);
//!
//! // Any model quoting "50,000 users × 6 mmHg" overstates ~7×.
//! let factor = overstatement_factor(50_000.0, 7_000.0).unwrap();
//! assert!((factor - 7.14).abs() < 0.01);
//!
//! // Honest population-level effect: 6 mmHg delivered to 14%, not 100%.
//! let effect = population_effect(6.0, share);
//! assert!((effect - 0.84).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineers own the engagement funnel, which makes them owners of a
//!   *clinical* variable: onboarding friction, notification strategy, load
//!   time, and offline resilience all move the dose delivered.
//! - Instrument the **clinically meaningful action** (readings logged,
//!   lessons completed), not opens — DAU built on notification-bounce
//!   sessions is dose-fraud.
//! - Treat engagement targets as *sufficiency* targets, not maximization —
//!   an app that achieves its outcome in 5 minutes/week and gets out of the
//!   way is clinically ideal and metrically "poor".
//! - Value engagement work via the population-effect model: a 2-point gain
//!   in effective-dose share is a quantifiable QALY line.
//!
//! ## Pitfalls
//!
//! - **Engagement as outcome**: usage is a means; the outcome is the PROM or
//!   clinical endpoint.
//! - **Averages over bimodal usage**: health-app populations split into
//!   devoted users and ghosts; means describe nobody — cohort it.
//! - **Dark-pattern dose inflation**: streaks and guilt notifications lift
//!   metrics and can harm the anxious populations health apps serve;
//!   clinical products carry clinical ethics.
//! - **Vendor-benchmark provenance**: most published engagement benchmarks
//!   come from analytics vendors, not peer review; calibrate against your
//!   own trials.
//!
//! ## Sources
//!
//! - App engagement benchmarks. <https://getstream.io/blog/app-retention-guide/>
//! - Health app KPI guides.
//!   <https://www.darly.solutions/blog/key-metrics-for-health-apps-success-a-guide-to-kpis-and-outcomes>
//! - Yardley L, et al. on effective engagement.
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC8726056/>
//!
//! Topic doc: health-economics-metrics/topics/engagement-metrics.md

/// Stickiness as a percentage: `DAU / MAU × 100`.
///
/// The share of monthly actives who show up on a given day. Benchmark:
/// around 20% is considered healthy for mobile apps generally, >25%
/// exceptional; health apps often run lower.
///
/// # Arguments
///
/// * `daily_active_users` — DAU (count).
/// * `monthly_active_users` — MAU (count).
///
/// # Returns
///
/// Stickiness in percent (0–100), or `None` if `monthly_active_users` is
/// zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::stickiness_percent;
///
/// // 4,000 DAU over 20,000 MAU hits the ~20% "healthy" benchmark.
/// let s = stickiness_percent(4_000.0, 20_000.0).unwrap();
/// assert_eq!(s, 20.0);
/// assert!(stickiness_percent(1.0, 0.0).is_none());
/// ```
pub fn stickiness_percent(daily_active_users: f64, monthly_active_users: f64) -> Option<f64> {
    if monthly_active_users == 0.0 {
        None
    } else {
        Some(daily_active_users / monthly_active_users * 100.0)
    }
}

/// Sessions per user over a period.
///
/// # Arguments
///
/// * `sessions` — total session count in the period.
/// * `users` — users active in the period (count).
///
/// # Returns
///
/// Mean sessions per user, or `None` if `users` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::sessions_per_user;
///
/// // 80,000 sessions across 20,000 monthly actives = 4 sessions/user/month.
/// assert_eq!(sessions_per_user(80_000.0, 20_000.0), Some(4.0));
/// assert!(sessions_per_user(1.0, 0.0).is_none());
/// ```
pub fn sessions_per_user(sessions: f64, users: f64) -> Option<f64> {
    if users == 0.0 { None } else { Some(sessions / users) }
}

/// Average session duration: total time divided by session count.
///
/// Time units are whatever `total_time` is measured in (seconds, minutes).
///
/// # Arguments
///
/// * `total_time` — total in-app time across all sessions.
/// * `sessions` — session count.
///
/// # Returns
///
/// Mean duration per session (same units as `total_time`), or `None` if
/// `sessions` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::average_session_duration;
///
/// // 240,000 minutes over 80,000 sessions = 3 minutes/session.
/// assert_eq!(average_session_duration(240_000.0, 80_000.0), Some(3.0));
/// assert!(average_session_duration(1.0, 0.0).is_none());
/// ```
pub fn average_session_duration(total_time: f64, sessions: f64) -> Option<f64> {
    if sessions == 0.0 { None } else { Some(total_time / sessions) }
}

/// Feature engagement: share of active users performing the key action.
///
/// The key action should be the clinically meaningful one (readings logged,
/// lessons completed), not app opens.
///
/// # Arguments
///
/// * `users_performing_key_action` — users doing the key action (count).
/// * `active_users` — active users in the same period (count).
///
/// # Returns
///
/// Fraction 0–1, or `None` if `active_users` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::feature_engagement;
///
/// // Worked example: of MAU 20,000, 7,000 log ≥4 readings/week → 35%.
/// let fe = feature_engagement(7_000.0, 20_000.0).unwrap();
/// assert!((fe - 0.35).abs() < 1e-9);
/// ```
pub fn feature_engagement(users_performing_key_action: f64, active_users: f64) -> Option<f64> {
    if active_users == 0.0 {
        None
    } else {
        Some(users_performing_key_action / active_users)
    }
}

/// Share of the registered base reaching the effective (trial-level) dose.
///
/// The denominator is deliberately the *registered* base, not MAU — the
/// economic claim is usually made about everyone acquired.
///
/// # Arguments
///
/// * `effective_dose_users` — users at trial-level usage (count).
/// * `registered_users` — total registered base (count).
///
/// # Returns
///
/// Fraction 0–1, or `None` if `registered_users` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::effective_dose_share;
///
/// // Worked example: 7,000 / 50,000 = 14% of the registered base.
/// let share = effective_dose_share(7_000.0, 50_000.0).unwrap();
/// assert!((share - 0.14).abs() < 1e-9);
/// ```
pub fn effective_dose_share(effective_dose_users: f64, registered_users: f64) -> Option<f64> {
    if registered_users == 0.0 {
        None
    } else {
        Some(effective_dose_users / registered_users)
    }
}

/// Population-level realized effect: the trial effect delivered only to the
/// effective-dose share of the population.
///
/// Gives zero credit for sub-threshold users; add partial credit separately
/// if dose-response data exists.
///
/// # Arguments
///
/// * `trial_effect` — efficacy measured in the pivotal study (any effect
///   unit, e.g. mmHg reduction).
/// * `effective_dose_share` — fraction of the population at trial-level
///   usage (0–1).
///
/// # Returns
///
/// Mean effect per registered user (same units as `trial_effect`).
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::population_effect;
///
/// // Worked example: 6 mmHg trial effect delivered to 14%, not 100%.
/// let effect = population_effect(6.0, 0.14);
/// assert!((effect - 0.84).abs() < 1e-9);
/// ```
pub fn population_effect(trial_effect: f64, effective_dose_share: f64) -> f64 {
    // Zero credit below threshold: realized effect = trial effect × dose share.
    trial_effect * effective_dose_share
}

/// Overstatement factor of a model quoting all registered users at full
/// trial effect.
///
/// How many times an economic model overstates when it quotes "all
/// registered users × full trial effect" instead of effective-dose users.
///
/// # Arguments
///
/// * `registered_users` — total registered base (count).
/// * `effective_dose_users` — users at trial-level usage (count).
///
/// # Returns
///
/// The overstatement multiple (≥ 1 when effective-dose users are a subset),
/// or `None` if `effective_dose_users` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::engagement_metrics::overstatement_factor;
///
/// // Worked example: quoting "50,000 users × 6 mmHg" overstates ~7×.
/// let factor = overstatement_factor(50_000.0, 7_000.0).unwrap();
/// assert!((factor - 7.0).abs() < 0.2);
/// ```
pub fn overstatement_factor(registered_users: f64, effective_dose_users: f64) -> Option<f64> {
    if effective_dose_users == 0.0 {
        None
    } else {
        Some(registered_users / effective_dose_users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "MAU 20,000 (40%)" of 50,000 registered.
    #[test]
    fn worked_example_mau_share_is_40_percent() {
        // MAU share of registered base, expressed with the same ratio helper.
        let share = effective_dose_share(20_000.0, 50_000.0).unwrap();
        assert!((share - 0.40).abs() < 1e-9, "got {share}");
    }

    // Doc line: "Effective-dose users = 7,000 / 50,000 = 14% of the registered base".
    #[test]
    fn worked_example_effective_dose_share_is_14_percent() {
        let share = effective_dose_share(7_000.0, 50_000.0).unwrap();
        assert!((share - 0.14).abs() < 1e-9, "got {share}");
    }

    // Doc line: "any economic model quoting '50,000 users × 6 mmHg' overstates ~7×".
    #[test]
    fn worked_example_overstatement_is_about_7x() {
        let factor = overstatement_factor(50_000.0, 7_000.0).unwrap();
        assert!((factor - 7.0).abs() < 0.2, "got {factor}");
    }

    // Doc line: "Population-level effect ≈ trial effect delivered to 14%, not 100%".
    #[test]
    fn worked_example_population_effect_is_trial_effect_times_14_percent() {
        let effect = population_effect(6.0, 0.14);
        assert!((effect - 0.84).abs() < 1e-9, "got {effect}");
    }

    // Doc lines: "MAU 20,000; of those, logging ≥4×/week: 7,000" → 35%.
    #[test]
    fn worked_example_feature_engagement_among_mau() {
        let fe = feature_engagement(7_000.0, 20_000.0).unwrap();
        assert!((fe - 0.35).abs() < 1e-9, "got {fe}");
    }

    // Doc benchmark: "DAU/MAU around 20% is considered healthy".
    #[test]
    fn stickiness_percent_computes_dau_over_mau() {
        let s = stickiness_percent(4_000.0, 20_000.0).unwrap();
        assert!((s - 20.0).abs() < 1e-9, "got {s}");
    }

    // Guard behavior: zero denominators return None rather than dividing.
    #[test]
    fn zero_denominators_return_none() {
        assert!(stickiness_percent(1.0, 0.0).is_none());
        assert!(sessions_per_user(1.0, 0.0).is_none());
        assert!(average_session_duration(1.0, 0.0).is_none());
        assert!(feature_engagement(1.0, 0.0).is_none());
        assert!(effective_dose_share(1.0, 0.0).is_none());
        assert!(overstatement_factor(1.0, 0.0).is_none());
    }
}
