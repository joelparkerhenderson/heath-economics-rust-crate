//! # EQ-5D
//!
//! EQ-5D is the EuroQol group's standardized questionnaire for measuring
//! health-related quality of life. It is the instrument that produces the
//! utility weights inside most QALY calculations — NICE's reference case
//! names it the preferred measure for adults.
//!
//! The EQ-5D-5L asks one question in each of **5 dimensions** — mobility,
//! self-care, usual activities, pain/discomfort, anxiety/depression — each
//! answered at **5 levels** (1 = no problems … 5 = extreme problems), plus a
//! 0–100 visual analogue scale (EQ VAS). A country-specific value set maps
//! the 5-digit profile to a utility index anchored at 1 = full health and
//! 0 = dead; states worse than death are negative (UK 3L set floor: −0.594).
//!
//! ## Formula
//!
//! ```text
//! Health state  = 5-digit profile, e.g. "21221"
//! Utility index = value_set(profile)
//! QALYs         = duration × utility
//! QALY gain     = (utility_after − utility_before) × duration
//! Attributable  = intervention gain − control-group gain
//! Monetized     = attributable QALYs × threshold (£/QALY)
//!
//! profile   = one level (1–5) per dimension: mobility, self-care,
//!             usual activities, pain/discomfort, anxiety/depression
//! value_set = country-specific mapping from profile to utility, derived
//!             from time-trade-off / discrete-choice surveys of the public
//! threshold = cost-effectiveness threshold (£20,000–£30,000/QALY at NICE)
//! ```
//!
//! ## Why it matters
//!
//! Any digital health product that wants to claim QALYs needs utilities from
//! a validated instrument, and EQ-5D is the default in the UK and much of
//! Europe. It is short enough to embed in an app (5 questions + a visual
//! scale), which means software products can collect HTA-grade outcome data
//! as a side effect of normal use — a structural advantage over drugs, which
//! need dedicated studies. Minimal clinically important differences for the
//! EQ-5D index are commonly in the 0.03–0.08 range.
//!
//! ## Example
//!
//! A musculoskeletal rehab app measures EQ-5D-5L at onboarding and 6 months
//! for 1,000 completing users: mean utility 0.62 → 0.71, sustained 1 year;
//! control-group change 0.03.
//!
//! ```
//! use health_economics::eq_5d::{
//!     qaly_gain, attributable_qaly_gain, monetized_value,
//! };
//!
//! // (0.71 − 0.62) × 1.0 year = 0.09 QALYs per completing user.
//! let gain = qaly_gain(0.62, 0.71, 1.0);
//! assert!((gain - 0.09).abs() < 1e-9);
//!
//! // Net of the control group's 0.03: attributable gain 0.06 QALYs/user.
//! let attributable = attributable_qaly_gain(gain, 0.03);
//! assert!((attributable - 0.06).abs() < 1e-9);
//!
//! // Monetized at £20,000–£30,000/QALY: £1,200–£1,800 per completing user.
//! assert!((monetized_value(attributable, 20_000.0) - 1_200.0).abs() < 1e-6);
//! assert!((monetized_value(attributable, 30_000.0) - 1_800.0).abs() < 1e-6);
//! ```
//!
//! ## Software engineering connection
//!
//! - **Instrument it.** EQ-5D at signup and at follow-up intervals is a few
//!   screens of UI; the payoff is HTA-grade evidence. Get licensing from
//!   EuroQol (required, free for some uses).
//! - **Use the right value set** for the deployment country — the same
//!   answers score differently in the UK vs Germany vs Japan.
//! - **Design lesson**: a tiny standardized survey plus a published scoring
//!   function yields a comparable single index — the pattern for any
//!   credible developer-experience index too (standardized instrument,
//!   published weights, not ad-hoc vibes).
//!
//! ## Pitfalls
//!
//! - **Before/after without a comparator** — regression to the mean and
//!   natural recovery inflate naive gains.
//! - **Survivorship bias**: measuring only users who stayed engaged.
//! - **Mixing 3L and 5L versions or value sets** across studies —
//!   systematically different numbers.
//! - **Ceiling effects** in mildly affected populations: many users score
//!   near 1.0 at baseline, leaving no headroom to demonstrate gain.
//!
//! ## Sources
//!
//! - EuroQol: EQ-5D-5L.
//!   <https://euroqol.org/information-and-support/euroqol-instruments/eq-5d-5l/>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/eq-5d.md

/// An EQ-5D-5L health-state profile: one level in each of the five dimensions.
///
/// Each dimension is answered at one of 5 levels: 1 = no problems,
/// 2 = slight problems, 3 = moderate problems, 4 = severe problems,
/// 5 = extreme problems / unable to. The profile is conventionally written
/// as a 5-digit code in dimension order, e.g. `"21221"`. Converting a
/// profile to a utility index requires a country-specific value set (not
/// included here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eq5dProfile {
    /// Mobility level, 1–5 (1 = no problems walking about … 5 = unable to).
    pub mobility: u8,
    /// Self-care level, 1–5 (washing and dressing).
    pub self_care: u8,
    /// Usual activities level, 1–5 (work, study, housework, family, leisure).
    pub usual_activities: u8,
    /// Pain/discomfort level, 1–5 (1 = none … 5 = extreme).
    pub pain_discomfort: u8,
    /// Anxiety/depression level, 1–5 (1 = not anxious/depressed … 5 = extremely).
    pub anxiety_depression: u8,
}

impl Eq5dProfile {
    /// Construct a validated profile from the five dimension levels.
    ///
    /// # Arguments
    ///
    /// * `mobility`, `self_care`, `usual_activities`, `pain_discomfort`,
    ///   `anxiety_depression` — the level (1–5) answered in each dimension,
    ///   in the standard EQ-5D dimension order.
    ///
    /// # Returns
    ///
    /// `Some(profile)` when every level is within 1–5; `None` if any level
    /// is 0 or greater than 5. This constructor never panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use health_economics::eq_5d::Eq5dProfile;
    ///
    /// // The doc's example health state "21221".
    /// let p = Eq5dProfile::new(2, 1, 2, 2, 1).unwrap();
    /// assert_eq!(p.code(), "21221");
    ///
    /// // Levels outside 1–5 are rejected.
    /// assert!(Eq5dProfile::new(0, 1, 1, 1, 1).is_none());
    /// assert!(Eq5dProfile::new(1, 1, 6, 1, 1).is_none());
    /// ```
    pub fn new(
        mobility: u8,
        self_care: u8,
        usual_activities: u8,
        pain_discomfort: u8,
        anxiety_depression: u8,
    ) -> Option<Self> {
        let levels = [mobility, self_care, usual_activities, pain_discomfort, anxiety_depression];
        if levels.iter().all(|&l| (1..=5).contains(&l)) {
            Some(Self { mobility, self_care, usual_activities, pain_discomfort, anxiety_depression })
        } else {
            None
        }
    }

    /// The 5-digit profile string in standard dimension order, e.g. `"21221"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use health_economics::eq_5d::Eq5dProfile;
    ///
    /// let p = Eq5dProfile::new(2, 1, 2, 2, 1).unwrap();
    /// assert_eq!(p.code(), "21221");
    /// ```
    pub fn code(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.mobility,
            self.self_care,
            self.usual_activities,
            self.pain_discomfort,
            self.anxiety_depression
        )
    }

    /// True if this is the full-health profile `"11111"` (no problems in any
    /// dimension) — the state a value set anchors at utility 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use health_economics::eq_5d::Eq5dProfile;
    ///
    /// assert!(Eq5dProfile::new(1, 1, 1, 1, 1).unwrap().is_full_health());
    /// assert!(!Eq5dProfile::new(2, 1, 2, 2, 1).unwrap().is_full_health());
    /// ```
    pub fn is_full_health(&self) -> bool {
        self.mobility == 1
            && self.self_care == 1
            && self.usual_activities == 1
            && self.pain_discomfort == 1
            && self.anxiety_depression == 1
    }
}

/// QALYs accrued over a duration at a given utility index.
///
/// Anchors: utility 1 = full health (1 year = 1 QALY), 0 = dead; states
/// worse than death carry negative utility (UK 3L floor −0.594) and produce
/// negative QALYs.
///
/// # Arguments
///
/// * `duration_years` — time spent in the state (years).
/// * `utility` — utility index from a value set (≤ 1; may be negative).
///
/// # Returns
///
/// QALYs: `duration_years × utility`.
///
/// # Examples
///
/// ```
/// use health_economics::eq_5d::qalys;
///
/// // One year in full health is exactly 1 QALY.
/// assert_eq!(qalys(1.0, 1.0), 1.0);
/// // One year at the worked example's baseline utility 0.62.
/// assert!((qalys(1.0, 0.62) - 0.62).abs() < 1e-9);
/// ```
pub fn qalys(duration_years: f64, utility: f64) -> f64 {
    duration_years * utility
}

/// QALY gain from a utility improvement sustained over a duration.
///
/// # Arguments
///
/// * `utility_before` — utility index at baseline.
/// * `utility_after` — utility index at follow-up.
/// * `duration_years` — how long the gain is sustained (years).
///
/// # Returns
///
/// QALY gain: `(utility_after − utility_before) × duration_years`. Negative
/// if utility declined.
///
/// # Examples
///
/// ```
/// use health_economics::eq_5d::qaly_gain;
///
/// // Worked example: (0.71 − 0.62) × 1.0 = 0.09 QALYs per user.
/// let gain = qaly_gain(0.62, 0.71, 1.0);
/// assert!((gain - 0.09).abs() < 1e-9);
/// ```
pub fn qaly_gain(utility_before: f64, utility_after: f64, duration_years: f64) -> f64 {
    (utility_after - utility_before) * duration_years
}

/// Attributable QALY gain: intervention gain net of the control-group gain.
///
/// Subtracting the control group's change removes natural recovery and
/// regression to the mean — before/after alone inflates the claim.
///
/// # Arguments
///
/// * `intervention_gain` — QALY gain observed in the intervention group.
/// * `control_gain` — QALY gain observed in the control group over the same
///   period.
///
/// # Returns
///
/// The gain attributable to the intervention (QALYs); negative if controls
/// did better.
///
/// # Examples
///
/// ```
/// use health_economics::eq_5d::attributable_qaly_gain;
///
/// // Worked example: 0.09 gross − 0.03 control = 0.06 QALYs/user.
/// let a = attributable_qaly_gain(0.09, 0.03);
/// assert!((a - 0.06).abs() < 1e-9);
/// ```
pub fn attributable_qaly_gain(intervention_gain: f64, control_gain: f64) -> f64 {
    intervention_gain - control_gain
}

/// Monetized health value: QALYs valued at a cost-effectiveness threshold.
///
/// # Arguments
///
/// * `qalys` — QALYs gained (attributable, ideally).
/// * `threshold_per_qaly` — cost-effectiveness threshold (£/QALY; NICE's
///   conventional range is £20,000–£30,000).
///
/// # Returns
///
/// Health value in £: `qalys × threshold_per_qaly`.
///
/// # Examples
///
/// ```
/// use health_economics::eq_5d::monetized_value;
///
/// // Worked example: 0.06 QALYs → £1,200 at £20,000/QALY, £1,800 at £30,000.
/// assert!((monetized_value(0.06, 20_000.0) - 1_200.0).abs() < 1e-6);
/// assert!((monetized_value(0.06, 30_000.0) - 1_800.0).abs() < 1e-6);
/// ```
pub fn monetized_value(qalys: f64, threshold_per_qaly: f64) -> f64 {
    qalys * threshold_per_qaly
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "(0.71 − 0.62) × 1.0 = 0.09 QALYs per user".
    #[test]
    fn worked_example_naive_gain_is_0_09_qalys() {
        let gain = qaly_gain(0.62, 0.71, 1.0);
        assert!((gain - 0.09).abs() < 1e-9, "got {gain}");
    }

    // Doc line: "Against a control-group change of 0.03 ... the attributable
    // gain is 0.06 QALYs/user".
    #[test]
    fn worked_example_attributable_gain_is_0_06_qalys() {
        let attributable = attributable_qaly_gain(qaly_gain(0.62, 0.71, 1.0), 0.03);
        assert!((attributable - 0.06).abs() < 1e-9, "got {attributable}");
    }

    // Doc line: "£1,200–£1,800 of health value per completing user" — lower bound.
    #[test]
    fn worked_example_monetized_at_20000_is_1200() {
        let v = monetized_value(0.06, 20_000.0);
        assert!((v - 1_200.0).abs() < 1e-9, "got {v}");
    }

    // Doc line: "£1,200–£1,800 of health value per completing user" — upper bound.
    #[test]
    fn worked_example_monetized_at_30000_is_1800() {
        let v = monetized_value(0.06, 30_000.0);
        assert!((v - 1_800.0).abs() < 1e-9, "got {v}");
    }

    // Doc line: 'Health state = 5-digit profile, e.g. "21221"'.
    #[test]
    fn profile_code_matches_doc_example() {
        let p = Eq5dProfile::new(2, 1, 2, 2, 1).unwrap();
        assert_eq!(p.code(), "21221");
        assert!(!p.is_full_health());
        assert!(Eq5dProfile::new(1, 1, 1, 1, 1).unwrap().is_full_health());
    }

    // Constructor contract: levels outside 1–5 are rejected with None.
    #[test]
    fn profile_rejects_out_of_range_levels() {
        assert!(Eq5dProfile::new(0, 1, 1, 1, 1).is_none());
        assert!(Eq5dProfile::new(1, 1, 6, 1, 1).is_none());
    }

    // Doc line: "Anchors: 1 = full health, 0 = dead; states worse than death
    // are negative (UK 3L set floor: −0.594)".
    #[test]
    fn qaly_arithmetic_respects_anchors() {
        assert!((qalys(1.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(qalys(1.0, -0.594) < 0.0);
    }
}
