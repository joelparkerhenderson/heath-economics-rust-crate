//! # SPACE and DevEx
//!
//! SPACE (Satisfaction & well-being, Performance, Activity, Communication &
//! collaboration, Efficiency & flow) and DevEx (feedback loops, cognitive
//! load, flow state) are frameworks for measuring developer productivity
//! multi-dimensionally — the field's answer to the discovery that no single
//! metric survives contact with reality.
//!
//! Neither framework is a formula; both are measurement designs. The
//! quantifiable parts are the SPACE composition rule and the worked
//! example's capacity-value arithmetic for a DevEx investment.
//!
//! ## Formula
//!
//! ```text
//! SPACE rule: ≥ 3 dimensions, ≥ 1 perceptual (survey) + ≥ 1 system
//!             (telemetry) metric
//!
//! time reclaimed/day = builds/day × minutes saved/build × usable fraction
//! capacity value/yr  = devs × hours reclaimed/day × working days × £/hour
//!
//! vendor index claim ≈ minutes/dev/week per index point (validate locally)
//!
//! usable fraction   fragmentation discount — reclaimed slivers of time
//!                   are not fully usable (0.4 in the worked example)
//! £/hour            loaded developer cost per hour
//! ```
//!
//! ## Why it matters
//!
//! Both frameworks encode the same hard-won lesson health outcomes research
//! learned decades earlier: a single number (lines of code; blood pressure)
//! misrepresents a multi-dimensional reality, and optimizing it produces
//! gaming, not improvement. SPACE prescribes combining metrics from at
//! least three dimensions, mixing telemetry with self-report —
//! structurally identical to how EQ-5D profiles five dimensions before any
//! index is computed. Satisfaction/well-being isn't soft garnish either: it
//! feeds workforce-retention economics, where attrition is priced in months
//! of loaded salary. Derived indices (e.g. DX's DXI) map survey composites
//! to time — the vendor claim is ~13 min/dev/week per index point, a
//! benchmark to validate locally, not a constant of nature.
//!
//! ## Example
//!
//! A platform team justifies a DevEx investment (CI speedup + docs
//! overhaul) for 300 developers. CI p75 falls from 28 to 9 minutes;
//! survey agreement with "I lose focus waiting for builds" falls 62% → 24%:
//!
//! ```rust
//! use health_economics::space_and_devex::{
//!     time_reclaimed_minutes_per_day, capacity_value_per_year,
//!     space_rule_satisfied, SpaceMetric, SpaceDimension, MetricSource,
//! };
//!
//! // Time reclaimed (telemetry): 6 builds/day × 19 min × 0.4 usable ≈ 45 min/day/dev
//! let minutes = time_reclaimed_minutes_per_day(6.0, 28.0 - 9.0, 0.4);
//! assert!((minutes - 45.6).abs() < 1e-9);
//!
//! // Capacity value: 300 × 0.75h × 220d × £60/h ≈ £2.97M/year (non-cash-releasing)
//! let value = capacity_value_per_year(300.0, 0.75, 220.0, 60.0);
//! assert!((value - 2_970_000.0).abs() < 1e-9);
//!
//! // The measurement design satisfies the SPACE rule: three dimensions,
//! // telemetry + survey.
//! let metrics = [
//!     SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::System },
//!     SpaceMetric { dimension: SpaceDimension::Satisfaction, source: MetricSource::Perceptual },
//!     SpaceMetric { dimension: SpaceDimension::Performance, source: MetricSource::System },
//! ];
//! assert!(space_rule_satisfied(&metrics));
//! ```
//!
//! Perceptual corroboration is what makes the telemetry claim credible —
//! either alone is gameable; together they triangulate.
//!
//! ## Software engineering connection
//!
//! - This topic is the software side; the transfer runs toward health
//!   economics.
//! - A "quality-adjusted engineer year" — time weighted by a standardized
//!   experience index — is the QALY's construction applied to engineering
//!   capacity.
//! - It inherits the QALY's rules: weights from a validated instrument
//!   (consistent survey, published scoring), elicited before the
//!   comparison, never tuned to flatter a favored tool.
//! - The SF-6D vs EQ-5D lesson applies: different instruments give
//!   systematically different numbers, so never compare DevEx indices
//!   across vendors' instruments.
//!
//! ## Pitfalls
//!
//! - Single-metric collapse: dashboards that reduce SPACE to one score
//!   recreate the problem the framework exists to prevent.
//! - Activity metrics as outcomes: commits, PRs, and story points are
//!   Activity — the dimension SPACE explicitly warns is most gameable
//!   (health analogue: counting procedures, not recoveries).
//! - Survey fatigue and Hawthorne effects: quarterly light-touch
//!   instruments beat weekly interrogation.
//! - Comparing teams: like hospital league tables without case-mix
//!   adjustment — context differences (domain, legacy load, on-call)
//!   dominate.
//!
//! ## Sources
//!
//! - Forsgren N, et al. "The SPACE of Developer Productivity." ACM Queue
//!   2021. <https://queue.acm.org/detail.cfm?id=3454124>
//! - Noda A, Forsgren N, Storey MA, Greiler M. "DevEx: What Actually Drives
//!   Productivity." ACM Queue 2023.
//!   <https://queue.acm.org/detail.cfm?id=3595878>
//!
//! Topic doc: health-economics-metrics/topics/space-and-devex.md

/// The five SPACE dimensions.
///
/// A compliant measurement design draws metrics from at least three of
/// these (see [`space_rule_satisfied`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceDimension {
    /// Satisfaction and well-being: how developers feel about their work,
    /// tools, and environment. Feeds workforce-retention economics.
    Satisfaction,
    /// Performance: outcomes of work (quality, impact), not volume of activity.
    Performance,
    /// Activity: counts of actions (commits, PRs, story points) — the
    /// dimension SPACE explicitly warns is most gameable.
    Activity,
    /// Communication and collaboration: how people and teams work together
    /// (discoverability, review quality, knowledge flow).
    Communication,
    /// Efficiency and flow: ability to make progress with minimal delays
    /// and interruptions (e.g. CI wait time, interrupt density).
    Efficiency,
}

/// How a metric is captured.
///
/// The SPACE rule requires at least one of each: telemetry alone is
/// gameable, surveys alone are unanchored; together they triangulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSource {
    /// Perceptual: self-report / survey (e.g. "I lose focus waiting for builds").
    Perceptual,
    /// System: telemetry (e.g. CI p75 duration).
    System,
}

/// One metric in a SPACE measurement design.
#[derive(Debug, Clone, Copy)]
pub struct SpaceMetric {
    /// Which SPACE dimension the metric belongs to.
    pub dimension: SpaceDimension,
    /// Whether it is perceptual (survey) or system (telemetry).
    pub source: MetricSource,
}

/// SPACE composition rule: the metric set must cover at least three
/// distinct dimensions and include at least one perceptual and one system
/// metric.
///
/// # Arguments
///
/// * `metrics` — the proposed measurement design.
///
/// # Returns
///
/// `true` if the design satisfies the rule; `false` for telemetry-only
/// dashboards or designs spanning fewer than three dimensions.
///
/// # Examples
///
/// ```rust
/// use health_economics::space_and_devex::{
///     space_rule_satisfied, SpaceMetric, SpaceDimension, MetricSource,
/// };
///
/// // CI duration (telemetry) + "waiting feels slow" (survey) + performance telemetry.
/// let ok = [
///     SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::System },
///     SpaceMetric { dimension: SpaceDimension::Satisfaction, source: MetricSource::Perceptual },
///     SpaceMetric { dimension: SpaceDimension::Performance, source: MetricSource::System },
/// ];
/// assert!(space_rule_satisfied(&ok));
///
/// // Telemetry-only across three dimensions still fails: no perceptual metric.
/// let telemetry_only = [
///     SpaceMetric { dimension: SpaceDimension::Activity, source: MetricSource::System },
///     SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::System },
///     SpaceMetric { dimension: SpaceDimension::Performance, source: MetricSource::System },
/// ];
/// assert!(!space_rule_satisfied(&telemetry_only));
/// ```
pub fn space_rule_satisfied(metrics: &[SpaceMetric]) -> bool {
    let mut dimensions: Vec<SpaceDimension> = Vec::new();
    let mut has_perceptual = false;
    let mut has_system = false;
    for m in metrics {
        // Collect distinct dimensions only.
        if !dimensions.contains(&m.dimension) {
            dimensions.push(m.dimension);
        }
        match m.source {
            MetricSource::Perceptual => has_perceptual = true,
            MetricSource::System => has_system = true,
        }
    }
    dimensions.len() >= 3 && has_perceptual && has_system
}

/// Minutes of developer time reclaimed per day from a feedback-loop speedup.
///
/// The usable fraction is the fragmentation discount: reclaimed slivers of
/// time between builds are not fully usable, so only a fraction converts to
/// productive capacity.
///
/// # Arguments
///
/// * `builds_per_day` — builds a developer waits on per day.
/// * `minutes_saved_per_build` — reduction in wait per build, minutes
///   (e.g. p75 falling 28 → 9 min saves 19).
/// * `usable_fraction` — fraction (0..1) of reclaimed slivers that is
///   actually usable (0.4 in the worked example).
///
/// # Returns
///
/// Usable minutes reclaimed per developer per day.
///
/// # Examples
///
/// ```rust
/// use health_economics::space_and_devex::time_reclaimed_minutes_per_day;
///
/// // Doc: 6 builds/day × 19 min × 0.4 usable = ~45 min/day/dev (exact 45.6).
/// let minutes = time_reclaimed_minutes_per_day(6.0, 28.0 - 9.0, 0.4);
/// assert!((minutes - 45.6).abs() < 1e-9);
/// ```
pub fn time_reclaimed_minutes_per_day(
    builds_per_day: f64,
    minutes_saved_per_build: f64,
    usable_fraction: f64,
) -> f64 {
    builds_per_day * minutes_saved_per_build * usable_fraction
}

/// Annual capacity value of reclaimed time.
///
/// Non-cash-releasing: this values capacity at the loaded rate, it does not
/// bank cash — label it accordingly in any business case.
///
/// # Arguments
///
/// * `developers` — number of developers affected.
/// * `hours_reclaimed_per_day` — usable hours reclaimed per developer per day.
/// * `working_days_per_year` — working days per year (e.g. 220).
/// * `loaded_cost_per_hour` — loaded developer cost per hour, £/hour.
///
/// # Returns
///
/// Capacity value per year, £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::space_and_devex::capacity_value_per_year;
///
/// // Doc: 300 × 0.75h × 220d × £60/h ≈ £2.97M/year.
/// let value = capacity_value_per_year(300.0, 0.75, 220.0, 60.0);
/// assert!((value - 2_970_000.0).abs() < 1e-9);
/// ```
pub fn capacity_value_per_year(
    developers: f64,
    hours_reclaimed_per_day: f64,
    working_days_per_year: f64,
    loaded_cost_per_hour: f64,
) -> f64 {
    developers * hours_reclaimed_per_day * working_days_per_year * loaded_cost_per_hour
}

/// Minutes per developer per week implied by a vendor index claim.
///
/// E.g. DX's DXI claim of ~13 min/dev/week per index point. Treat as a
/// vendor benchmark to validate locally, not a constant of nature — and
/// never compare index points across vendors' instruments.
///
/// # Arguments
///
/// * `index_points_gained` — improvement in the vendor's index.
/// * `minutes_per_point` — the vendor's claimed minutes/dev/week per point
///   (e.g. 13.0).
///
/// # Returns
///
/// Implied minutes reclaimed per developer per week.
///
/// # Examples
///
/// ```rust
/// use health_economics::space_and_devex::vendor_index_minutes_per_week;
///
/// // Doc: vendor claim ≈ 13 min/dev/week per index point.
/// let minutes = vendor_index_minutes_per_week(1.0, 13.0);
/// assert!((minutes - 13.0).abs() < 1e-9);
/// ```
pub fn vendor_index_minutes_per_week(index_points_gained: f64, minutes_per_point: f64) -> f64 {
    index_points_gained * minutes_per_point
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc worked example: "Time reclaimed (telemetry): 6 builds/day × 19 min
    // × 0.4 usable = ~45 min/day/dev".
    #[test]
    fn time_reclaimed_is_about_45_min_per_day() {
        // Doc: 6 builds/day × 19 min × 0.4 usable = ~45 min/day/dev (exact 45.6)
        let minutes = time_reclaimed_minutes_per_day(6.0, 28.0 - 9.0, 0.4);
        assert!((minutes - 45.6).abs() < 1e-9);
        assert!((minutes - 45.0).abs() < 1.0);
    }

    // Doc worked example: "Capacity value: 300 × 0.75h × 220d × £60/h
    // ≈ £2.97M/year".
    #[test]
    fn capacity_value_is_2_97m_per_year() {
        // Doc: 300 × 0.75h × 220d × £60/h ≈ £2.97M/year (exact 2,970,000)
        let value = capacity_value_per_year(300.0, 0.75, 220.0, 60.0);
        assert!((value - 2_970_000.0).abs() < 1e-9);
    }

    // Doc (The math): "SPACE rule: ≥ 3 dimensions, ≥ 1 perceptual + ≥ 1
    // system metric" — a compliant design passes.
    #[test]
    fn space_rule_accepts_three_dimensions_with_mixed_sources() {
        let metrics = [
            // CI duration (telemetry) + "waiting feels slow" (survey)
            SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::System },
            SpaceMetric { dimension: SpaceDimension::Satisfaction, source: MetricSource::Perceptual },
            SpaceMetric { dimension: SpaceDimension::Performance, source: MetricSource::System },
        ];
        assert!(space_rule_satisfied(&metrics));
    }

    // SPACE rule: two dimensions is below the ≥ 3 threshold even with mixed
    // sources.
    #[test]
    fn space_rule_rejects_too_few_dimensions() {
        let metrics = [
            SpaceMetric { dimension: SpaceDimension::Activity, source: MetricSource::System },
            SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::Perceptual },
        ];
        assert!(!space_rule_satisfied(&metrics));
    }

    // Doc: telemetry alone is gameable — a telemetry-only dashboard fails
    // the rule even with three dimensions.
    #[test]
    fn space_rule_rejects_telemetry_only_dashboards() {
        // Three dimensions but no perceptual metric — gameable, per the doc.
        let metrics = [
            SpaceMetric { dimension: SpaceDimension::Activity, source: MetricSource::System },
            SpaceMetric { dimension: SpaceDimension::Efficiency, source: MetricSource::System },
            SpaceMetric { dimension: SpaceDimension::Performance, source: MetricSource::System },
        ];
        assert!(!space_rule_satisfied(&metrics));
    }

    // Doc (The math): "vendor claim ≈ 13 min/dev/week per index point".
    #[test]
    fn vendor_claim_is_13_minutes_per_point_per_week() {
        // Doc: vendor claim ≈ 13 min/dev/week per index point.
        let minutes = vendor_index_minutes_per_week(1.0, 13.0);
        assert!((minutes - 13.0).abs() < 1e-9);
    }
}
