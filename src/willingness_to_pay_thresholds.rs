//! # Willingness-to-Pay Thresholds
//!
//! A willingness-to-pay (WTP) threshold is the maximum a decision-maker will
//! pay per unit of health gain — the line λ that turns an ICER (incremental
//! cost-effectiveness ratio) into an adopt/reject decision.
//!
//! Run in reverse, the threshold is a value-based pricing model: the same
//! product carries a different maximum defensible price in every
//! jurisdiction's threshold regime. Two theories of what λ *is*: the
//! demand side (what society is willing to pay for health — a value
//! judgment) and the supply side (the health the budget currently produces
//! at the margin — an empirical quantity, Claxton's ~£13k/QALY for the NHS;
//! if the decision λ exceeds it, approving new tech displaces more health
//! than it adds).
//!
//! ## Formula
//!
//! ```text
//! Adopt if ICER = ΔC/ΔE < λ
//! Equivalently: adopt if NMB = λ×ΔE − ΔC > 0
//! Maximum defensible price: price_max = λ × ΔE + offsets
//!
//! where:
//!   ΔC      — incremental cost vs. the comparator (£)
//!   ΔE      — incremental effect (health gain, e.g. QALYs)
//!   λ       — willingness-to-pay threshold (£ per QALY)
//!   NMB     — net monetary benefit (£)
//!   offsets — cost offsets the product generates elsewhere (£)
//! ```
//!
//! ## Why it matters
//!
//! The threshold is where health economics stops being measurement and
//! becomes policy. Every national system has one, explicit or implicit, and
//! knowing the local number tells you exactly how to price a health-value
//! claim. Benchmarks (as researched, 2024–2025): NICE (England)
//! £20,000–£30,000 per QALY, with an empirical average decision threshold of
//! ≈ £24,400 (2022–24), severity modifiers raising the effective ceiling to
//! ~£36k–£51k, and highly specialised technologies up to £100k+; ICER (US,
//! non-governmental) $100,000–$150,000 per QALY/evLYG benchmarks; Canada
//! (CADTH / CDA-AMC) ≈ CAD$50,000 per QALY; WHO-CHOICE historically 1–3×
//! GDP per capita per DALY averted (now discouraged as too blunt); the
//! empirical UK supply side (Claxton et al.) ≈ £13,000 per QALY actually
//! displaced at the NHS margin.
//!
//! ## Example
//!
//! A digital therapeutic delivers 0.05 QALYs per treated patient at a net
//! cost (price minus offsets) of £800.
//!
//! ```rust
//! use health_economics::willingness_to_pay_thresholds::{
//!     icer, adopt_by_icer, adopt_by_nmb, net_monetary_benefit,
//!     max_defensible_price,
//! };
//!
//! // ICER = 800 / 0.05 = £16,000 per QALY
//! let r = icer(800.0, 0.05).unwrap();
//! assert_eq!(r, 16_000.0);
//!
//! // England: below the £20k threshold → fundable (both rule forms agree).
//! assert_eq!(adopt_by_icer(800.0, 0.05, 20_000.0), Some(true));
//! assert!(adopt_by_nmb(20_000.0, 0.05, 800.0));
//! assert_eq!(net_monetary_benefit(20_000.0, 0.05, 800.0), 200.0);
//!
//! // Maximum defensible price at λ = £20,000: 0.05 × 20,000 = £1,000 + offsets.
//! assert_eq!(max_defensible_price(20_000.0, 0.05, 0.0), 1_000.0);
//! // US commercial framing at $150k/QALY: value-based price is far higher.
//! assert_eq!(max_defensible_price(150_000.0, 0.05, 0.0), 7_500.0);
//! // A GDP-per-capita country threshold of $4,000: under ~$200 net.
//! assert_eq!(max_defensible_price(4_000.0, 0.05, 0.0), 200.0);
//! ```
//!
//! Same product, three markets, three prices — the threshold *is* the
//! pricing model.
//!
//! ## Software engineering connection
//!
//! - Every engineering org has an implicit λ: the hurdle at which it funds
//!   tooling per engineer-hour saved.
//! - Making it explicit — "we fund anything under £40 per credible
//!   engineer-hour saved" — enables league-table comparison of platform
//!   investments, exactly as cost-per-QALY league tables rank health
//!   spending.
//! - The supply-side lesson transfers too: your true internal λ is what your
//!   *current* backlog produces at the margin, not what leadership says time
//!   is worth.
//!
//! ## Pitfalls
//!
//! - **Threshold shopping** across jurisdictions or citing the HST ceiling
//!   for an ordinary product.
//! - **Treating λ as a price floor**: clearing the threshold is necessary,
//!   not sufficient — budget impact can still sink an affordable-per-unit
//!   product.
//! - **Ignoring that thresholds move**: NICE's severity modifiers (2022) and
//!   periodic reviews change effective λ; date your claims.
//!
//! ## Sources
//!
//! - NICE: changes to cost-effectiveness thresholds.
//!   <https://www.nice.org.uk/news/articles/changes-to-nice-s-cost-effectiveness-thresholds-confirmed>
//! - Empirical NICE threshold analysis, Value in Health 2024.
//!   <https://www.sciencedirect.com/science/article/pii/S1098301524000858>
//! - Claxton K, et al. HTA 2015;19(14).
//!   <https://www.journalslibrary.nihr.ac.uk/hta/hta19140/>
//! - ICER 2023 Value Assessment Framework.
//!   <https://icer.org/wp-content/uploads/2023/09/ICER_2023_VAF_For-Publication_092523.pdf>
//!
//! Topic doc: health-economics-metrics/topics/willingness-to-pay-thresholds.md

/// Incremental cost-effectiveness ratio: ΔC / ΔE.
///
/// The price of one unit of health gain (e.g. £ per QALY) bought by
/// switching from the comparator to the intervention.
///
/// # Arguments
///
/// * `incremental_cost` — ΔC, extra cost vs. the comparator (£).
/// * `incremental_effect` — ΔE, extra health gain (e.g. QALYs).
///
/// # Returns
///
/// `Some(icer)` in £ per effect unit; `None` when `incremental_effect` is
/// zero (the ratio is undefined — no ICER exists).
///
/// # Examples
///
/// ```rust
/// use health_economics::willingness_to_pay_thresholds::icer;
///
/// // £800 net cost for 0.05 QALYs → ICER = £16,000 per QALY.
/// let r = icer(800.0, 0.05).unwrap();
/// assert_eq!(r, 16_000.0);
///
/// // Zero incremental effect: no ICER.
/// assert!(icer(800.0, 0.0).is_none());
/// ```
pub fn icer(incremental_cost: f64, incremental_effect: f64) -> Option<f64> {
    if incremental_effect == 0.0 {
        None
    } else {
        Some(incremental_cost / incremental_effect)
    }
}

/// Net monetary benefit at threshold λ: NMB = λ × ΔE − ΔC.
///
/// Converts the health gain into money at the threshold and subtracts the
/// incremental cost; positive NMB means the intervention is worth adopting
/// at that λ.
///
/// # Arguments
///
/// * `threshold` — λ, willingness-to-pay per effect unit (£/QALY).
/// * `incremental_effect` — ΔE, extra health gain (e.g. QALYs).
/// * `incremental_cost` — ΔC, extra cost vs. the comparator (£).
///
/// # Returns
///
/// Net monetary benefit (£); positive means adopt at this threshold.
///
/// # Examples
///
/// ```rust
/// use health_economics::willingness_to_pay_thresholds::net_monetary_benefit;
///
/// // NMB = 20,000 × 0.05 − 800 = £200 > 0 → fundable in England.
/// let nmb = net_monetary_benefit(20_000.0, 0.05, 800.0);
/// assert_eq!(nmb, 200.0);
/// ```
pub fn net_monetary_benefit(
    threshold: f64,
    incremental_effect: f64,
    incremental_cost: f64,
) -> f64 {
    threshold * incremental_effect - incremental_cost
}

/// Decision rule on the ICER form: adopt if ICER = ΔC/ΔE < λ.
///
/// # Arguments
///
/// * `incremental_cost` — ΔC, extra cost vs. the comparator (£).
/// * `incremental_effect` — ΔE, extra health gain (e.g. QALYs).
/// * `threshold` — λ, willingness-to-pay per effect unit (£/QALY).
///
/// # Returns
///
/// `Some(true)` to adopt, `Some(false)` to reject; `None` when
/// `incremental_effect` is zero (no ICER exists — use [`adopt_by_nmb`],
/// which is defined even at ΔE = 0).
///
/// # Examples
///
/// ```rust
/// use health_economics::willingness_to_pay_thresholds::adopt_by_icer;
///
/// // £16,000/QALY is below England's £20,000 threshold → fundable.
/// assert_eq!(adopt_by_icer(800.0, 0.05, 20_000.0), Some(true));
///
/// // Below a $4,000 GDP-per-capita threshold it is not.
/// assert_eq!(adopt_by_icer(800.0, 0.05, 4_000.0), Some(false));
///
/// // Zero incremental effect: the ICER rule is undefined.
/// assert_eq!(adopt_by_icer(800.0, 0.0, 20_000.0), None);
/// ```
pub fn adopt_by_icer(
    incremental_cost: f64,
    incremental_effect: f64,
    threshold: f64,
) -> Option<bool> {
    icer(incremental_cost, incremental_effect).map(|r| r < threshold)
}

/// Decision rule on the NMB form: adopt if λ × ΔE − ΔC > 0.
///
/// Equivalent to the ICER rule (for positive ΔE) but defined even when
/// ΔE = 0, since it never divides.
///
/// # Arguments
///
/// * `threshold` — λ, willingness-to-pay per effect unit (£/QALY).
/// * `incremental_effect` — ΔE, extra health gain (e.g. QALYs).
/// * `incremental_cost` — ΔC, extra cost vs. the comparator (£).
///
/// # Returns
///
/// `true` when the net monetary benefit is strictly positive.
///
/// # Examples
///
/// ```rust
/// use health_economics::willingness_to_pay_thresholds::adopt_by_nmb;
///
/// // NMB = 20,000 × 0.05 − 800 = £200 > 0 → adopt.
/// assert!(adopt_by_nmb(20_000.0, 0.05, 800.0));
///
/// // With no health gain, a costly product is never adopted.
/// assert!(!adopt_by_nmb(20_000.0, 0.0, 800.0));
/// ```
pub fn adopt_by_nmb(threshold: f64, incremental_effect: f64, incremental_cost: f64) -> bool {
    net_monetary_benefit(threshold, incremental_effect, incremental_cost) > 0.0
}

/// Maximum defensible price: value-based pricing run in reverse from λ.
///
/// The most a jurisdiction with threshold λ can be asked to pay is the
/// health gain valued at the threshold, plus any cost offsets the product
/// generates elsewhere: price_max = λ × ΔE + offsets. Same product, three
/// markets, three prices — the threshold *is* the pricing model.
///
/// # Arguments
///
/// * `threshold` — λ, willingness-to-pay per QALY in the target market
///   (£ or $ per QALY).
/// * `qalys_gained` — ΔE, QALYs gained per treated patient.
/// * `cost_offsets` — costs the product avoids elsewhere in the system, per
///   patient (same currency).
///
/// # Returns
///
/// Maximum defensible price per treated patient (same currency as
/// `threshold`).
///
/// # Examples
///
/// ```rust
/// use health_economics::willingness_to_pay_thresholds::max_defensible_price;
///
/// // England, λ = £20,000: price_max = 0.05 × 20,000 = £1,000 + offsets.
/// assert_eq!(max_defensible_price(20_000.0, 0.05, 0.0), 1_000.0);
/// assert_eq!(max_defensible_price(20_000.0, 0.05, 300.0), 1_300.0);
///
/// // US commercial framing at $150k/QALY: $7,500 for the same 0.05 QALYs.
/// assert_eq!(max_defensible_price(150_000.0, 0.05, 0.0), 7_500.0);
/// ```
pub fn max_defensible_price(threshold: f64, qalys_gained: f64, cost_offsets: f64) -> f64 {
    threshold * qalys_gained + cost_offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    // Worked example: "ICER = 800 / 0.05 = £16,000 per QALY".
    /// ICER = 800 / 0.05 = £16,000 per QALY.
    #[test]
    fn icer_is_16_000_per_qaly() {
        let r = icer(800.0, 0.05).unwrap();
        assert!((r - 16_000.0).abs() < 1e-9);
    }

    // Worked example: "England: below £20k → fundable" — checked through
    // both equivalent rule forms.
    /// England: £16,000 is below the £20,000 threshold, so fundable — by both
    /// the ICER rule and the equivalent NMB rule.
    #[test]
    fn fundable_in_england_below_20k_threshold() {
        assert_eq!(adopt_by_icer(800.0, 0.05, 20_000.0), Some(true));
        assert!(adopt_by_nmb(20_000.0, 0.05, 800.0));
        // NMB = 20,000 × 0.05 − 800 = £200 > 0
        let nmb = net_monetary_benefit(20_000.0, 0.05, 800.0);
        assert!((nmb - 200.0).abs() < 1e-9);
    }

    // Worked example: "at λ = £20,000, price_max = 0.05 × 20,000 + offsets
    // = £1,000 + offsets".
    /// At λ = £20,000, price_max = 0.05 × 20,000 + offsets = £1,000 + offsets.
    #[test]
    fn max_price_in_england_is_1_000_plus_offsets() {
        let price = max_defensible_price(20_000.0, 0.05, 0.0);
        assert!((price - 1_000.0).abs() < 1e-9);
        let price_with_offsets = max_defensible_price(20_000.0, 0.05, 300.0);
        assert!((price_with_offsets - 1_300.0).abs() < 1e-9);
    }

    // Worked example: "A GDP-per-capita country threshold of $4,000: the
    // same product must cost under ~$200 net".
    /// A GDP-per-capita country threshold of $4,000: the same product must
    /// cost under ~$200 net.
    #[test]
    fn max_price_at_4_000_threshold_is_200() {
        let price = max_defensible_price(4_000.0, 0.05, 0.0);
        assert!((price - 200.0).abs() < 1e-9);
    }

    // Worked example: "US commercial framing at $150k/QALY: value-based
    // price is far higher".
    /// US commercial framing at $150k/QALY: value-based price is far higher
    /// ($7,500 for the same 0.05 QALYs).
    #[test]
    fn us_framing_at_150k_gives_far_higher_price() {
        let price = max_defensible_price(150_000.0, 0.05, 0.0);
        assert!((price - 7_500.0).abs() < 1e-9);
    }

    // Edge-case contract: ΔE = 0 means the ratio ΔC/ΔE — and hence the
    // ICER decision rule — is undefined.
    /// Zero incremental effect: no ICER, and the ICER decision rule is None.
    #[test]
    fn zero_effect_has_no_icer() {
        assert!(icer(800.0, 0.0).is_none());
        assert!(adopt_by_icer(800.0, 0.0, 20_000.0).is_none());
    }
}
