//! # Sensitivity Analysis
//!
//! Deterministic sensitivity analysis (DSA) varies one assumption at a time
//! across a plausible range to see whether the conclusion survives. The
//! standard visualization is a tornado diagram: parameters ranked by how
//! much they swing the result, dominant parameter on top.
//!
//! Variants: two-way DSA (vary two parameters on a grid) and threshold
//! analysis (find the parameter value where the decision flips).
//!
//! ## Formula
//!
//! For each parameter p with plausible range [p_low, p_high]:
//!
//! ```text
//! Result_low  = model(p = p_low,  all others at base case)
//! Result_high = model(p = p_high, all others at base case)
//! Swing(p)    = |Result_high − Result_low|
//!
//! p_low, p_high   the parameter's plausible low and high values
//! model(·)        the economic model's result (e.g. annual net benefit)
//! Swing(p)        the tornado-diagram bar length for parameter p
//! ```
//!
//! ## Why it matters
//!
//! Every economic model is built on estimates — time saved, uptake, unit
//! costs. Health technology assessment refuses to accept a point estimate
//! ("ROI is 340%") without evidence that the conclusion is robust to
//! reasonable disagreement about the inputs. A tornado diagram tells the
//! decision-maker which assumption to interrogate: if the case only works
//! when the most contested parameter is at its optimistic end, everyone can
//! see that immediately. This is the single most transferable habit from
//! health economics to software business cases.
//!
//! ## Example
//!
//! AI coding assistant for 200 developers. Base case: £39/dev/month
//! license; 30 min/dev/day saved; loaded cost £60/hour; 220 working days:
//!
//! ```rust
//! use health_economics::sensitivity_analysis::{
//!     CodingAssistantCase, OneWayResult, rank_by_swing,
//! };
//!
//! let base = CodingAssistantCase {
//!     developers: 200.0,
//!     hours_saved_per_day: 0.5,
//!     loaded_cost_per_hour: 60.0,
//!     working_days_per_year: 220.0,
//!     license_per_dev_per_month: 39.0,
//! };
//!
//! // Base-case annual benefit = 200 × 0.5h × 220 × £60 = £1,320,000
//! assert!((base.annual_benefit() - 1_320_000.0).abs() < 1e-9);
//! // Annual cost = 200 × £39 × 12 = £93,600; net = £1,226,400
//! assert!((base.annual_cost() - 93_600.0).abs() < 1e-9);
//! assert!((base.net_benefit() - 1_226_400.0).abs() < 1e-9);
//!
//! // Time saved 0.1–1.0 h/day: net = £170,400 … £2,546,400 (swing £2.38M) ← dominates
//! let mut low = base;
//! low.hours_saved_per_day = 0.1;
//! let mut high = base;
//! high.hours_saved_per_day = 1.0;
//! let time_saved = OneWayResult {
//!     result_at_low: low.net_benefit(),
//!     result_at_high: high.net_benefit(),
//! };
//! assert!((time_saved.swing() - 2_376_000.0).abs() < 1e-9);
//!
//! // Tornado ranking: time saved dominates, license price is last.
//! let mut swings = [
//!     ("license price", 48_000.0),
//!     ("time saved", time_saved.swing()),
//!     ("working days", 240_000.0),
//!     ("loaded cost", 880_000.0),
//! ];
//! rank_by_swing(&mut swings);
//! assert_eq!(swings[0].0, "time saved");
//! assert_eq!(swings[3].0, "license price");
//!
//! // Threshold analysis: net benefit hits zero at about 2.1 minutes/day saved.
//! let minutes = base.threshold_hours_saved_per_day().unwrap() * 60.0;
//! assert!((minutes - 2.1).abs() < 0.05);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineers already do this instinct as "what if we're wrong about X?" —
//!   DSA just makes it systematic and visible.
//! - Put a tornado diagram in every tooling proposal, capacity plan, and
//!   build-vs-buy analysis.
//! - It converts arguments about whose gut feeling is right into agreements
//!   about which parameter to go measure.
//! - The measurement is often a pilot, whose value can itself be priced
//!   (expected value of perfect information).
//! - Remember the result is capacity, not cash (cash-releasing vs
//!   non-cash-releasing).
//!
//! ## Pitfalls
//!
//! - Ranges chosen to flatter: ±10% around every input regardless of actual
//!   uncertainty. Time-saved estimates deserve ±80%; license prices ±10%.
//! - One-at-a-time misses interactions — correlated parameters (uptake and
//!   time saved) need two-way analysis or full probabilistic sensitivity
//!   analysis.
//! - Doing the analysis and ignoring it: if the tornado says the case hinges
//!   on one soft number, the next step is measurement, not sign-off.
//!
//! ## Sources
//!
//! - York Health Economics Consortium glossary: deterministic sensitivity
//!   analysis. <https://yhec.co.uk/glossary/deterministic-sensitivity-analysis/>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/sensitivity-analysis.md

/// Base-case model from the worked example: an AI coding assistant for a
/// developer organization.
///
/// All parameters are held in one struct so a DSA can copy the base case
/// and vary one field at a time, holding the others fixed.
#[derive(Debug, Clone, Copy)]
pub struct CodingAssistantCase {
    /// Number of developers licensed.
    pub developers: f64,
    /// Hours saved per developer per day (the most contested estimate —
    /// plausible range in the worked example is 0.1–1.0 h/day).
    pub hours_saved_per_day: f64,
    /// Loaded developer cost per hour (salary plus overheads), £/hour.
    pub loaded_cost_per_hour: f64,
    /// Working days per year (base case 220).
    pub working_days_per_year: f64,
    /// License cost per developer per month, £/dev/month.
    pub license_per_dev_per_month: f64,
}

impl CodingAssistantCase {
    /// Annual benefit: developers × hours saved/day × working days × loaded cost/hour.
    ///
    /// Values the reclaimed time as capacity at the loaded rate.
    ///
    /// # Returns
    ///
    /// Annual benefit, £/year.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::sensitivity_analysis::CodingAssistantCase;
    ///
    /// let base = CodingAssistantCase {
    ///     developers: 200.0,
    ///     hours_saved_per_day: 0.5,
    ///     loaded_cost_per_hour: 60.0,
    ///     working_days_per_year: 220.0,
    ///     license_per_dev_per_month: 39.0,
    /// };
    /// // Doc: 200 × 0.5h × 220 × £60 = £1,320,000
    /// assert!((base.annual_benefit() - 1_320_000.0).abs() < 1e-9);
    /// ```
    pub fn annual_benefit(&self) -> f64 {
        self.developers * self.hours_saved_per_day * self.working_days_per_year
            * self.loaded_cost_per_hour
    }

    /// Annual cost: developers × license/month × 12.
    ///
    /// # Returns
    ///
    /// Annual license cost, £/year.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::sensitivity_analysis::CodingAssistantCase;
    ///
    /// let base = CodingAssistantCase {
    ///     developers: 200.0,
    ///     hours_saved_per_day: 0.5,
    ///     loaded_cost_per_hour: 60.0,
    ///     working_days_per_year: 220.0,
    ///     license_per_dev_per_month: 39.0,
    /// };
    /// // Doc: 200 × £39 × 12 = £93,600
    /// assert!((base.annual_cost() - 93_600.0).abs() < 1e-9);
    /// ```
    pub fn annual_cost(&self) -> f64 {
        self.developers * self.license_per_dev_per_month * 12.0
    }

    /// Annual net benefit: benefit − cost. This is the model result the DSA varies.
    ///
    /// # Returns
    ///
    /// Annual net benefit, £/year (negative when the license costs more
    /// than the time it saves).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::sensitivity_analysis::CodingAssistantCase;
    ///
    /// let base = CodingAssistantCase {
    ///     developers: 200.0,
    ///     hours_saved_per_day: 0.5,
    ///     loaded_cost_per_hour: 60.0,
    ///     working_days_per_year: 220.0,
    ///     license_per_dev_per_month: 39.0,
    /// };
    /// // Doc: base-case net = £1,226,400
    /// assert!((base.net_benefit() - 1_226_400.0).abs() < 1e-9);
    /// ```
    pub fn net_benefit(&self) -> f64 {
        self.annual_benefit() - self.annual_cost()
    }

    /// Threshold analysis: hours saved per day at which net benefit is zero
    /// (the decision-flip point).
    ///
    /// Solves `net_benefit = 0` for `hours_saved_per_day` with all other
    /// parameters at their current values.
    ///
    /// # Returns
    ///
    /// The break-even hours saved per developer per day, or `None` if the
    /// benefit rate per hour saved is zero (no developers, no working days,
    /// or free time) so no threshold exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::sensitivity_analysis::CodingAssistantCase;
    ///
    /// let base = CodingAssistantCase {
    ///     developers: 200.0,
    ///     hours_saved_per_day: 0.5,
    ///     loaded_cost_per_hour: 60.0,
    ///     working_days_per_year: 220.0,
    ///     license_per_dev_per_month: 39.0,
    /// };
    /// // Doc: net benefit hits zero at about 2.1 minutes/day saved.
    /// let minutes = base.threshold_hours_saved_per_day().unwrap() * 60.0;
    /// assert!((minutes - 2.1).abs() < 0.05);
    /// ```
    pub fn threshold_hours_saved_per_day(&self) -> Option<f64> {
        // £ of benefit generated per hour saved per day, per year:
        // developers × working days × loaded cost.
        let benefit_per_hour_saved =
            self.developers * self.working_days_per_year * self.loaded_cost_per_hour;
        if benefit_per_hour_saved == 0.0 {
            None
        } else {
            // Break-even: hours where benefit exactly covers the annual cost.
            Some(self.annual_cost() / benefit_per_hour_saved)
        }
    }
}

/// Results of a one-way DSA on a single parameter: the model result with the
/// parameter at its low and high plausible values, all others at base case.
#[derive(Debug, Clone, Copy)]
pub struct OneWayResult {
    /// Model result at the parameter's low value.
    pub result_at_low: f64,
    /// Model result at the parameter's high value.
    pub result_at_high: f64,
}

impl OneWayResult {
    /// Swing: |Result_high − Result_low|, the bar length in a tornado diagram.
    ///
    /// The absolute value matters: for cost-like parameters the high value
    /// lowers the result, so the raw difference is negative.
    ///
    /// # Returns
    ///
    /// The swing in the model result's units (e.g. £/year of net benefit).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::sensitivity_analysis::OneWayResult;
    ///
    /// // Doc: time saved 0.1–1.0 h/day → net £170,400 … £2,546,400, swing £2.38M.
    /// let r = OneWayResult { result_at_low: 170_400.0, result_at_high: 2_546_400.0 };
    /// assert!((r.swing() - 2_376_000.0).abs() < 1e-9);
    /// ```
    pub fn swing(&self) -> f64 {
        (self.result_at_high - self.result_at_low).abs()
    }
}

/// Sort (parameter name, swing) pairs in descending order of swing —
/// the tornado-diagram ordering, dominant parameter first.
///
/// # Arguments
///
/// * `parameter_swings` — mutable slice of `(name, swing)` pairs; sorted in
///   place. Non-comparable swings (NaN) are treated as equal.
///
/// # Examples
///
/// ```rust
/// use health_economics::sensitivity_analysis::rank_by_swing;
///
/// // Doc tornado: time saved £2.38M ≫ loaded cost £0.88M ≫ working days
/// // £0.24M ≫ license £48k.
/// let mut swings = [
///     ("license price", 48_000.0),
///     ("time saved", 2_376_000.0),
///     ("working days", 240_000.0),
///     ("loaded cost", 880_000.0),
/// ];
/// rank_by_swing(&mut swings);
/// assert_eq!(swings[0].0, "time saved");
/// assert_eq!(swings[3].0, "license price");
/// ```
pub fn rank_by_swing(parameter_swings: &mut [(&str, f64)]) {
    // Descending order: b compared against a. NaN swings compare as equal
    // rather than panicking.
    parameter_swings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_case() -> CodingAssistantCase {
        CodingAssistantCase {
            developers: 200.0,
            hours_saved_per_day: 0.5,
            loaded_cost_per_hour: 60.0,
            working_days_per_year: 220.0,
            license_per_dev_per_month: 39.0,
        }
    }

    // Doc worked example: "Base-case annual benefit = 200 × 0.5h × 220 × £60
    // = £1,320,000".
    #[test]
    fn base_case_benefit_is_1_320_000() {
        assert!((base_case().annual_benefit() - 1_320_000.0).abs() < 1e-9);
    }

    // Doc worked example: "Annual cost = 200 × £39 × 12 = £93,600".
    #[test]
    fn base_case_cost_is_93_600() {
        assert!((base_case().annual_cost() - 93_600.0).abs() < 1e-9);
    }

    // Doc worked example: "Base-case net = £1,226,400".
    #[test]
    fn base_case_net_is_1_226_400() {
        assert!((base_case().net_benefit() - 1_226_400.0).abs() < 1e-9);
    }

    // Doc tornado: "Time saved 0.1–1.0 h/day: net = £170,400 … £2,546,400
    // (swing £2.38M) ← dominates".
    #[test]
    fn time_saved_dominates_the_tornado() {
        let mut low = base_case();
        low.hours_saved_per_day = 0.1;
        let mut high = base_case();
        high.hours_saved_per_day = 1.0;
        let r = OneWayResult { result_at_low: low.net_benefit(), result_at_high: high.net_benefit() };
        assert!((r.result_at_low - 170_400.0).abs() < 1e-9);
        assert!((r.result_at_high - 2_546_400.0).abs() < 1e-9);
        // Doc: swing £2.38M (exact 2,376,000)
        assert!((r.swing() - 2_376_000.0).abs() < 1e-9);
    }

    // Doc tornado: "Loaded cost £40–£80/h: net = £786,400 … £1,666,400
    // (swing £0.88M)".
    #[test]
    fn loaded_cost_swing_is_0_88m() {
        let mut low = base_case();
        low.loaded_cost_per_hour = 40.0;
        let mut high = base_case();
        high.loaded_cost_per_hour = 80.0;
        let r = OneWayResult { result_at_low: low.net_benefit(), result_at_high: high.net_benefit() };
        assert!((r.result_at_low - 786_400.0).abs() < 1e-9);
        assert!((r.result_at_high - 1_666_400.0).abs() < 1e-9);
        assert!((r.swing() - 880_000.0).abs() < 1e-9);
    }

    // Doc tornado: "Working days 200–240: net = £1,106,400 … £1,346,400
    // (swing £0.24M)".
    #[test]
    fn working_days_swing_is_0_24m() {
        let mut low = base_case();
        low.working_days_per_year = 200.0;
        let mut high = base_case();
        high.working_days_per_year = 240.0;
        let r = OneWayResult { result_at_low: low.net_benefit(), result_at_high: high.net_benefit() };
        assert!((r.result_at_low - 1_106_400.0).abs() < 1e-9);
        assert!((r.result_at_high - 1_346_400.0).abs() < 1e-9);
        assert!((r.swing() - 240_000.0).abs() < 1e-9);
    }

    // Doc tornado: "License £30–£50/mo: net = £1,248,000 … £1,200,000
    // (swing £48k)".
    #[test]
    fn license_price_swing_is_48k() {
        let mut low = base_case();
        low.license_per_dev_per_month = 30.0;
        let mut high = base_case();
        high.license_per_dev_per_month = 50.0;
        let r = OneWayResult { result_at_low: low.net_benefit(), result_at_high: high.net_benefit() };
        assert!((r.result_at_low - 1_248_000.0).abs() < 1e-9);
        assert!((r.result_at_high - 1_200_000.0).abs() < 1e-9);
        assert!((r.swing() - 48_000.0).abs() < 1e-9);
    }

    // Doc: "Threshold analysis: net benefit hits zero at about
    // 2.1 minutes/day saved".
    #[test]
    fn threshold_is_about_2_1_minutes_per_day() {
        // Doc: net benefit hits zero at about 2.1 minutes/day saved.
        let hours = base_case().threshold_hours_saved_per_day().unwrap();
        let minutes = hours * 60.0;
        assert!((minutes - 2.1).abs() < 0.05);
    }

    // Doc tornado ordering: time saved dominates; then loaded cost, working
    // days, license price — "the decision is insensitive to license price".
    #[test]
    fn tornado_ranking_puts_time_saved_first_and_license_last() {
        let mut swings = [
            ("license price", 48_000.0),
            ("time saved", 2_376_000.0),
            ("working days", 240_000.0),
            ("loaded cost", 880_000.0),
        ];
        rank_by_swing(&mut swings);
        assert_eq!(swings[0].0, "time saved");
        assert_eq!(swings[1].0, "loaded cost");
        assert_eq!(swings[2].0, "working days");
        assert_eq!(swings[3].0, "license price");
    }
}
