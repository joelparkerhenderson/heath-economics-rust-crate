//! # Activation and Uptake
//!
//! Activation rate is the share of sign-ups who reach first meaningful value
//! (the "aha" action — first reading logged, first lesson done). Uptake is the
//! population version: the share of the *eligible* population that adopts at
//! all. Together they are the front gates of the value funnel:
//! acquisition → uptake → activation → retention → outcome.
//!
//! The funnel value model multiplies the stages together, so improving the
//! smallest factor usually dominates (theory-of-constraints for funnels).
//!
//! ## Formula
//!
//! ```text
//! Activation rate = users completing key action within window / sign-ups × 100
//! Uptake rate     = adopters / eligible population × 100
//! DTx fill rate   = activated prescription codes / issued prescriptions × 100
//!
//! Funnel value model:
//!   eligible × uptake × activation × retention-weighted benefit = population value
//! ```
//!
//! Legend:
//! - `users completing key action` — sign-ups who perform the clinically
//!   meaningful first action within the defined window (count).
//! - `sign-ups` — registered users (count).
//! - `adopters` — people who register/adopt at all (count).
//! - `eligible population` — everyone the offer could apply to (count).
//! - `activated prescription codes` / `issued prescriptions` — for prescribed
//!   digital therapeutics (counts).
//! - Funnel stages (`uptake`, `activation`, …) enter the value model as
//!   fractions (0.0–1.0), not percentages.
//!
//! ## Why it matters
//!
//! Non-activated users are pure cost: acquisition spend, provisioning, support
//! surface — zero clinical value. Benchmarks put healthcare software's
//! activation *below* the cross-industry average (≈24% vs ≈37% for new-user
//! activation in one SaaS benchmark set; onboarding-checklist completion ~20%),
//! reflecting heavier onboarding (identity, consent, clinical safety). Uptake
//! carries the population stakes: in the RE-AIM framing, public-health impact
//! ≈ reach × effectiveness — a superb app adopted by 3% of the eligible
//! population moves the population needle 3%'s worth. For prescribed digital
//! therapeutics the uptake gate is visible in national data: ~81% of German
//! DiGA prescriptions get activated — one in five prescribed-and-paid-for
//! treatments never starts.
//!
//! ## Example
//!
//! A commissioner offers a diabetes-prevention app to 80,000 eligible
//! residents: 12,000 register (uptake 15%), 5,400 activate (45%), 1,600
//! complete the 6-month programme (30%). The programme effect among completers
//! is 0.03 QALYs + £180 avoided costs, so population value ≈ £1.25M and
//! per-eligible-person value is £15.60 (versus £780 if everyone completed).
//!
//! ```rust
//! use health_economics::activation_and_uptake::{
//!     activation_rate_percent, per_eligible_person_value, population_value,
//!     uptake_rate_percent, value_per_completer,
//! };
//!
//! let uptake = uptake_rate_percent(12_000.0, 80_000.0).unwrap();
//! assert!((uptake - 15.0).abs() < 1e-9);
//!
//! let activation = activation_rate_percent(5_400.0, 12_000.0).unwrap();
//! assert!((activation - 45.0).abs() < 1e-9);
//!
//! // 0.03 QALYs × £20,000/QALY + £180 avoided costs = £780 per completer.
//! let per_completer = value_per_completer(0.03, 20_000.0, 180.0);
//! assert!((per_completer - 780.0).abs() < 1e-9);
//!
//! // 1,600 completers × £780 = £1,248,000 ≈ £1.25M population value.
//! let pop_value = population_value(1_600.0, per_completer);
//! assert!((pop_value - 1_248_000.0).abs() < 1e-9);
//!
//! // Spread over the 80,000 eligible: £15.60 per eligible person.
//! let per_person = per_eligible_person_value(pop_value, 80_000.0).unwrap();
//! assert!((per_person - 15.6).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Activation is the most engineering-tractable stage of the funnel:
//!   identity-verification friction, consent flows, empty-state design, and
//!   time-to-first-value are code, not policy.
//! - Healthcare median time-to-value ≈ 1 day 7 hours in benchmark data —
//!   every hour of it is churn exposure.
//! - Uptake is a distribution-systems problem: integration into referral
//!   pathways (the prescription moment), GP-endorsed invitations (trust
//!   transfers), and accessibility (language, digital skills).
//! - The funnel-value model is the business case generator: multiply the
//!   factors, find the constraint, price the fix against the population value
//!   it releases.
//!
//! ## Pitfalls
//!
//! - **Activation defined as convenience** (email verified) rather than
//!   clinical meaning (first therapeutic action) — inflates the metric,
//!   breaks the value chain.
//! - **Uptake denominator games**: "of those who visited the site" vs the
//!   truly eligible population — commissioners care about the latter.
//! - **Selection effects**: easy-to-activate users are the least sick and
//!   least deprived; funnel improvements can widen equity gaps while
//!   improving averages.
//!
//! ## Sources
//!
//! - Activation benchmarks (healthcare SaaS).
//!   <https://userpilot.com/blog/healthcare-product-metrics-benchmark-report/>
//! - DiGA activation data, npj Digital Medicine 2024.
//!   <https://www.nature.com/articles/s41746-024-01137-1>
//! - RE-AIM framework. <https://re-aim.org/>
//!
//! Topic doc: health-economics-metrics/topics/activation-and-uptake.md

/// Activation rate as a percentage of sign-ups.
///
/// Users completing the key action within the window divided by sign-ups,
/// times 100. Define the "key action" clinically (first therapeutic action),
/// not as a convenience event like email verification. Both arguments are
/// counts; the result is a percentage (0–100 in normal use).
///
/// # Arguments
///
/// * `users_completing_key_action` — sign-ups who completed the key action
///   within the activation window (count).
/// * `sign_ups` — total registered users (count).
///
/// # Returns
///
/// The activation rate in percent, or `None` if `sign_ups` is zero (the rate
/// is undefined with no sign-ups).
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::activation_rate_percent;
///
/// // Worked example: 5,400 of 12,000 registered users activate → 45%.
/// let rate = activation_rate_percent(5_400.0, 12_000.0).unwrap();
/// assert!((rate - 45.0).abs() < 1e-9);
///
/// assert!(activation_rate_percent(5_400.0, 0.0).is_none());
/// ```
pub fn activation_rate_percent(users_completing_key_action: f64, sign_ups: f64) -> Option<f64> {
    if sign_ups == 0.0 {
        None
    } else {
        Some(users_completing_key_action / sign_ups * 100.0)
    }
}

/// Uptake rate as a percentage of the eligible population.
///
/// Adopters divided by the eligible population, times 100. Use the truly
/// eligible population as the denominator — not "those who visited the site".
/// Both arguments are counts; the result is a percentage.
///
/// # Arguments
///
/// * `adopters` — people who adopt (register) at all (count).
/// * `eligible_population` — everyone the offer could apply to (count).
///
/// # Returns
///
/// The uptake rate in percent, or `None` if `eligible_population` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::uptake_rate_percent;
///
/// // Worked example: 12,000 of 80,000 eligible residents register → 15%.
/// let rate = uptake_rate_percent(12_000.0, 80_000.0).unwrap();
/// assert!((rate - 15.0).abs() < 1e-9);
///
/// assert!(uptake_rate_percent(12_000.0, 0.0).is_none());
/// ```
pub fn uptake_rate_percent(adopters: f64, eligible_population: f64) -> Option<f64> {
    if eligible_population == 0.0 {
        None
    } else {
        Some(adopters / eligible_population * 100.0)
    }
}

/// Digital-therapeutics (DTx) fill rate as a percentage of issued prescriptions.
///
/// Activated prescription codes divided by issued prescriptions, times 100.
/// This is the uptake gate for prescribed digital therapeutics: German DiGA
/// data shows ~81%, i.e. one in five prescribed-and-paid-for treatments never
/// starts. Both arguments are counts; the result is a percentage.
///
/// # Arguments
///
/// * `activated_prescription_codes` — prescription codes actually activated
///   by patients (count).
/// * `issued_prescriptions` — prescriptions issued (count).
///
/// # Returns
///
/// The fill rate in percent, or `None` if `issued_prescriptions` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::dtx_fill_rate_percent;
///
/// // ~81% of German DiGA prescriptions get activated.
/// let rate = dtx_fill_rate_percent(81.0, 100.0).unwrap();
/// assert!((rate - 81.0).abs() < 1e-9);
///
/// assert!(dtx_fill_rate_percent(81.0, 0.0).is_none());
/// ```
pub fn dtx_fill_rate_percent(
    activated_prescription_codes: f64,
    issued_prescriptions: f64,
) -> Option<f64> {
    if issued_prescriptions == 0.0 {
        None
    } else {
        Some(activated_prescription_codes / issued_prescriptions * 100.0)
    }
}

/// Value realized per programme completer, in currency units.
///
/// The QALY gain per completer monetized at the willingness-to-pay threshold,
/// plus avoided downstream costs per completer.
///
/// # Arguments
///
/// * `qalys_per_completer` — QALY gain per completer (QALYs, e.g. 0.03).
/// * `willingness_to_pay_per_qaly` — monetary value per QALY (e.g. £20,000,
///   a conventional NICE threshold).
/// * `avoided_costs_per_completer` — avoided downstream costs per completer
///   (currency).
///
/// # Returns
///
/// The monetized value per completer (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::value_per_completer;
///
/// // Worked example: 0.03 QALYs × £20,000 + £180 avoided costs = £780.
/// let value = value_per_completer(0.03, 20_000.0, 180.0);
/// assert!((value - 780.0).abs() < 1e-9);
/// ```
pub fn value_per_completer(
    qalys_per_completer: f64,
    willingness_to_pay_per_qaly: f64,
    avoided_costs_per_completer: f64,
) -> f64 {
    qalys_per_completer * willingness_to_pay_per_qaly + avoided_costs_per_completer
}

/// Funnel value model: population value from multiplying the funnel stages.
///
/// eligible × uptake × activation × completion × retention-weighted benefit
/// per completer. Because value is a product of stage factors, improving the
/// smallest factor usually dominates. All stage rates are fractions
/// (0.0–1.0), not percentages.
///
/// # Arguments
///
/// * `eligible_population` — everyone the offer could apply to (count).
/// * `uptake_fraction` — share of eligibles who register (0.0–1.0).
/// * `activation_fraction` — share of registered who activate (0.0–1.0).
/// * `completion_fraction` — share of activated who complete (0.0–1.0).
/// * `value_per_completer` — retention-weighted benefit per completer
///   (currency; see [`value_per_completer`]).
///
/// # Returns
///
/// The population value (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::funnel_population_value;
///
/// // Worked example: 80,000 × 15% uptake × 45% activation × ~30% completion
/// // × £780 per completer ≈ £1.25M.
/// let value = funnel_population_value(80_000.0, 0.15, 0.45, 1_600.0 / 5_400.0, 780.0);
/// assert!((value - 1_248_000.0).abs() < 1.0);
///
/// // Doubling uptake (15% → 30%) doubles population value.
/// let doubled = funnel_population_value(80_000.0, 0.30, 0.45, 1_600.0 / 5_400.0, 780.0);
/// assert!((doubled / value - 2.0).abs() < 1e-9);
/// ```
pub fn funnel_population_value(
    eligible_population: f64,
    uptake_fraction: f64,
    activation_fraction: f64,
    completion_fraction: f64,
    value_per_completer: f64,
) -> f64 {
    // Four multiplications: each stage scales the population that survives to
    // the next; the product is the completer count times value per completer.
    eligible_population
        * uptake_fraction
        * activation_fraction
        * completion_fraction
        * value_per_completer
}

/// Population value from a known count of completers.
///
/// Completers times the value each one realizes — the short form of the
/// funnel model when the completer count is observed rather than derived.
///
/// # Arguments
///
/// * `completers` — people who completed the programme (count).
/// * `value_per_completer` — monetized value per completer (currency).
///
/// # Returns
///
/// The population value (currency units).
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::population_value;
///
/// // Worked example: 1,600 completers × £780 = £1,248,000 ≈ £1.25M.
/// let value = population_value(1_600.0, 780.0);
/// assert!((value - 1_248_000.0).abs() < 1e-9);
/// ```
pub fn population_value(completers: f64, value_per_completer: f64) -> f64 {
    completers * value_per_completer
}

/// Population value spread across every eligible person, in currency units.
///
/// Divides total population value by the eligible population — the honest
/// per-head figure a commissioner sees.
///
/// # Arguments
///
/// * `population_value` — total realized population value (currency).
/// * `eligible_population` — everyone the offer could apply to (count).
///
/// # Returns
///
/// Value per eligible person (currency units), or `None` if
/// `eligible_population` is zero.
///
/// # Examples
///
/// ```rust
/// use health_economics::activation_and_uptake::per_eligible_person_value;
///
/// // Worked example: £1,248,000 over 80,000 eligible = £15.60 per person —
/// // versus £780 if every eligible person completed.
/// let per_person = per_eligible_person_value(1_248_000.0, 80_000.0).unwrap();
/// assert!((per_person - 15.6).abs() < 1e-9);
///
/// assert!(per_eligible_person_value(1_248_000.0, 0.0).is_none());
/// ```
pub fn per_eligible_person_value(
    population_value: f64,
    eligible_population: f64,
) -> Option<f64> {
    if eligible_population == 0.0 {
        None
    } else {
        Some(population_value / eligible_population)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: a commissioner offers a diabetes-prevention app to
    // 80,000 eligible residents.

    #[test]
    fn uptake_is_15_percent() {
        // Doc: "Invited → registered: 80,000 → 12,000 (uptake 15%)".
        let got = uptake_rate_percent(12_000.0, 80_000.0).unwrap();
        assert!((got - 15.0).abs() < 1e-9);
    }

    #[test]
    fn activation_is_45_percent() {
        // Doc: "Registered → activated ... 12,000 → 5,400 (45%)".
        let got = activation_rate_percent(5_400.0, 12_000.0).unwrap();
        assert!((got - 45.0).abs() < 1e-9);
    }

    #[test]
    fn completion_is_about_30_percent() {
        // 1,600 of 5,400 activated complete the 6-month programme (doc: 30%).
        let got = activation_rate_percent(1_600.0, 5_400.0).unwrap();
        assert!((got - 30.0).abs() < 0.7);
    }

    #[test]
    fn value_per_completer_is_780() {
        // 0.03 QALYs × £20,000 + £180 avoided costs = £780.
        let got = value_per_completer(0.03, 20_000.0, 180.0);
        assert!((got - 780.0).abs() < 1e-9);
    }

    #[test]
    fn population_value_is_about_1_25_million() {
        // 1,600 × £780 = £1,248,000 ≈ £1.25M.
        let got = population_value(1_600.0, value_per_completer(0.03, 20_000.0, 180.0));
        assert!((got - 1_248_000.0).abs() < 1e-9);
        assert!((got - 1_250_000.0).abs() < 10_000.0);
    }

    #[test]
    fn per_eligible_person_value_is_15_60() {
        // Doc: "Per-eligible-person value = £15.6".
        let pv = population_value(1_600.0, 780.0);
        let got = per_eligible_person_value(pv, 80_000.0).unwrap();
        assert!((got - 15.6).abs() < 1e-9);
    }

    #[test]
    fn per_eligible_value_is_780_if_everyone_completed() {
        // Versus £780 if every eligible person completed.
        let pv = population_value(80_000.0, 780.0);
        let got = per_eligible_person_value(pv, 80_000.0).unwrap();
        assert!((got - 780.0).abs() < 1e-9);
    }

    #[test]
    fn doubling_uptake_doubles_value() {
        // Doubling uptake (15% → 30%) doubles population value.
        let base = funnel_population_value(80_000.0, 0.15, 0.45, 1_600.0 / 5_400.0, 780.0);
        let doubled = funnel_population_value(80_000.0, 0.30, 0.45, 1_600.0 / 5_400.0, 780.0);
        assert!((doubled / base - 2.0).abs() < 1e-9);
    }

    #[test]
    fn raising_activation_45_to_65_adds_about_44_percent() {
        // Doc: "raising activation 45→65% adds ~44%".
        let base = funnel_population_value(80_000.0, 0.15, 0.45, 1_600.0 / 5_400.0, 780.0);
        let lifted = funnel_population_value(80_000.0, 0.15, 0.65, 1_600.0 / 5_400.0, 780.0);
        let gain = lifted / base - 1.0;
        assert!((gain - 0.44).abs() < 0.01);
    }

    #[test]
    fn zero_denominators_return_none() {
        // Edge-case semantics: every ratio is undefined at a zero denominator.
        assert!(activation_rate_percent(1.0, 0.0).is_none());
        assert!(uptake_rate_percent(1.0, 0.0).is_none());
        assert!(dtx_fill_rate_percent(1.0, 0.0).is_none());
        assert!(per_eligible_person_value(1.0, 0.0).is_none());
    }

    #[test]
    fn diga_fill_rate_is_81_percent() {
        // "Why it matters": ~81% of German DiGA prescriptions get activated.
        let got = dtx_fill_rate_percent(81.0, 100.0).unwrap();
        assert!((got - 81.0).abs() < 1e-9);
    }
}
