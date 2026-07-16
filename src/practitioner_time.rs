//! # Practitioner Time
//!
//! Practitioner time is the scarcest resource in most health systems.
//! Measuring the value of saving a clinician minutes per day requires
//! shifting from simple wage math to **opportunity cost and system
//! capacity**: within a national health service, a practitioner's time is a
//! rigid operational bottleneck, not a cost line that flexes.
//!
//! This module values saved clinician minutes at three levels of increasing
//! honesty — wage basis, output basis, bottleneck basis — and applies a
//! fragmentation discount for time saved in unusable scraps.
//!
//! ## Formula
//!
//! ```text
//! 1. Wage basis:       hours × loaded salary rate
//! 2. Output basis:     hours → appointments/procedures enabled × scheme value
//! 3. Bottleneck basis: hours × value of pathway throughput released
//!
//! Fragmentation discount: value × stated utilization factor
//! ```
//!
//! Legend:
//! - `hours` — practitioner hours released per year.
//! - `loaded salary rate` — fully loaded £/hour (PSSRU unit costs), what the
//!   time *costs*, not what it produces.
//! - `scheme value` — £ value per appointment/procedure enabled (national
//!   tariff / unit cost).
//! - `pathway throughput` — value of what the bottleneck role gates (theory
//!   of constraints).
//! - `utilization factor` — stated fraction in [0, 1] of nominal saved time
//!   that consolidates into usable, redeployable blocks.
//!
//! ## Why it matters
//!
//! You cannot quickly make more GPs, consultants, or specialist nurses —
//! training pipelines run 5–15 years, and vacancies are chronic. So an hour
//! of practitioner time saved is not "wages avoided" (the practitioner is
//! still paid); it is *bottleneck capacity released*, and bottleneck capacity
//! is worth what the bottleneck produces. This is why "saves 10 minutes per
//! consultation" claims are simultaneously the most common and the most
//! mispriced line in digital health.
//!
//! ## Example
//!
//! Ambient scribing saves a GP 2 minutes per consultation, 30
//! consultations/day — 60 minutes/day, or 220 hours/year over 220 working
//! days:
//!
//! ```rust
//! use health_economics::practitioner_time::{
//!     annual_extra_appointments, annual_hours_saved, daily_minutes_saved,
//!     extra_appointments_per_day, output_basis_value, wage_basis_value,
//! };
//!
//! // 2 minutes × 30 consultations = 60 minutes/day.
//! let daily = daily_minutes_saved(2.0, 30.0);
//! assert_eq!(daily, 60.0);
//!
//! // 60 minutes/day over 220 working days = 220 hours/year per GP.
//! let hours = annual_hours_saved(daily, 220.0);
//! assert_eq!(hours, 220.0);
//!
//! // Wage basis: 220 × £80 (loaded GP hour, PSSRU-region) ≈ £17,600/GP/year.
//! assert_eq!(wage_basis_value(hours, 80.0), 17_600.0);
//!
//! // Output basis: 60 min/day = 5 extra 12-min consultations/day
//! // = 1,100 extra appointments/GP/year × £42 ≈ £46,200/GP/year.
//! let per_day = extra_appointments_per_day(daily, 12.0).unwrap();
//! assert_eq!(per_day, 5.0);
//! let per_year = annual_extra_appointments(per_day, 220.0);
//! assert_eq!(per_year, 1_100.0);
//! assert_eq!(output_basis_value(per_year, 42.0), 46_200.0);
//!
//! // Across a 50-GP federation: ~£2.3M/year of output-basis capacity.
//! let federation = 50.0 * output_basis_value(per_year, 42.0);
//! assert_eq!(federation, 2_310_000.0);
//! ```
//!
//! The federation figure holds only if the minutes are real (measured, not
//! vendor-claimed), consolidated (whole consultations, not fragments), and
//! redeployed.
//!
//! ## Software engineering connection
//!
//! - Senior engineer time behaves identically: it is the bottleneck through
//!   which designs, reviews, and incidents flow — value it by what the
//!   bottleneck gates, not by salary.
//! - The same three-level valuation applies to any "AI saves each developer X
//!   minutes" claim: wage math flatters small numbers.
//! - The honest questions are whether minutes consolidate into usable blocks
//!   and what the released capacity actually produces.
//! - There is a multiplier when the saved hour belongs to the person everyone
//!   else waits on (downstream resource optimization).
//!
//! ## Pitfalls
//!
//! - **Minutes × salary = savings** — the canonical inflation; it's capacity,
//!   and only at the stated utilization.
//! - **Ignoring the quantum problem**: 12 × 5-minute savings ≠ one free hour.
//! - **Valuing all roles alike**: an hour of the pathway bottleneck is worth
//!   many times an hour of a non-gating role.
//!
//! ## Sources
//!
//! - PSSRU, Unit Costs of Health and Social Care.
//!   <https://www.pssru.ac.uk/unitcostsreport/>
//! - NHS England, NHS productivity.
//!   <https://www.england.nhs.uk/long-read/nhs-productivity/>
//!
//! Topic doc: health-economics-metrics/topics/practitioner-time.md

/// Minutes saved per working day.
///
/// Minutes saved per consultation × consultations per day.
///
/// # Arguments
///
/// * `minutes_saved_per_consultation` — measured (not vendor-claimed) minutes
///   saved per consultation.
/// * `consultations_per_day` — consultations the practitioner performs per
///   working day.
///
/// # Returns
///
/// Minutes saved per working day.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::daily_minutes_saved;
///
/// // Ambient scribing saves 2 minutes × 30 consultations = 60 minutes/day.
/// assert_eq!(daily_minutes_saved(2.0, 30.0), 60.0);
/// ```
pub fn daily_minutes_saved(
    minutes_saved_per_consultation: f64,
    consultations_per_day: f64,
) -> f64 {
    minutes_saved_per_consultation * consultations_per_day
}

/// Hours saved per practitioner per year.
///
/// Daily minutes × working days ÷ 60 (the division converts minutes to
/// hours).
///
/// # Arguments
///
/// * `daily_minutes_saved` — minutes saved per working day (see
///   [`daily_minutes_saved`]).
/// * `working_days_per_year` — working days per year (worked example: 220).
///
/// # Returns
///
/// Hours saved per practitioner per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::annual_hours_saved;
///
/// // 60 minutes/day over 220 working days = 220 hours/year per GP.
/// assert_eq!(annual_hours_saved(60.0, 220.0), 220.0);
/// ```
pub fn annual_hours_saved(daily_minutes_saved: f64, working_days_per_year: f64) -> f64 {
    daily_minutes_saved * working_days_per_year / 60.0
}

/// Level 1 — wage basis: hours × loaded hourly salary rate.
///
/// Uses PSSRU-style loaded unit costs. This values what the time *costs*,
/// not what it produces — the least honest of the three levels, because the
/// practitioner is still paid whether or not the time is redeployed.
///
/// # Arguments
///
/// * `hours_saved` — practitioner hours released per year.
/// * `loaded_hourly_rate` — fully loaded £/hour (worked example: £80 for a
///   GP, PSSRU-region).
///
/// # Returns
///
/// Wage-basis value in £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::wage_basis_value;
///
/// // 220 hours × £80 loaded GP hour ≈ £17,600/GP/year.
/// assert_eq!(wage_basis_value(220.0, 80.0), 17_600.0);
/// ```
pub fn wage_basis_value(hours_saved: f64, loaded_hourly_rate: f64) -> f64 {
    hours_saved * loaded_hourly_rate
}

/// Extra appointments enabled per day by the released minutes.
///
/// Daily minutes saved ÷ scheduled appointment length. Only meaningful when
/// the saved minutes actually consolidate into whole appointment slots (see
/// [`fragmentation_adjusted_value`]).
///
/// # Arguments
///
/// * `daily_minutes_saved` — minutes released per working day.
/// * `minutes_per_appointment` — scheduled appointment length in minutes
///   (worked example: 12-minute consultations).
///
/// # Returns
///
/// `Some(daily_minutes_saved / minutes_per_appointment)`, or `None` when
/// `minutes_per_appointment` is zero (the division is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::extra_appointments_per_day;
///
/// // 60 min/day = 5 extra 12-minute consultations per day.
/// assert_eq!(extra_appointments_per_day(60.0, 12.0), Some(5.0));
///
/// // A zero-length appointment is undefined.
/// assert_eq!(extra_appointments_per_day(60.0, 0.0), None);
/// ```
pub fn extra_appointments_per_day(
    daily_minutes_saved: f64,
    minutes_per_appointment: f64,
) -> Option<f64> {
    if minutes_per_appointment == 0.0 {
        None
    } else {
        Some(daily_minutes_saved / minutes_per_appointment)
    }
}

/// Extra appointments enabled per year.
///
/// Extra appointments per day × working days per year.
///
/// # Arguments
///
/// * `extra_appointments_per_day` — appointments enabled per working day (see
///   [`extra_appointments_per_day`]).
/// * `working_days_per_year` — working days per year (worked example: 220).
///
/// # Returns
///
/// Extra appointments per practitioner per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::annual_extra_appointments;
///
/// // 5/day × 220 days = 1,100 extra appointments/GP/year.
/// assert_eq!(annual_extra_appointments(5.0, 220.0), 1_100.0);
/// ```
pub fn annual_extra_appointments(
    extra_appointments_per_day: f64,
    working_days_per_year: f64,
) -> f64 {
    extra_appointments_per_day * working_days_per_year
}

/// Level 2 — output basis: appointments enabled × scheme value per appointment.
///
/// Values the released time by what it *produces* (national tariff / unit
/// cost per appointment), not what it costs. More honest than the wage basis
/// for a capacity-constrained system.
///
/// # Arguments
///
/// * `annual_extra_appointments` — appointments enabled per year (see
///   [`annual_extra_appointments`]).
/// * `value_per_appointment` — scheme £ value per appointment (worked
///   example: £42).
///
/// # Returns
///
/// Output-basis value in £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::output_basis_value;
///
/// // 1,100 appointments × £42 ≈ £46,200/GP/year.
/// assert_eq!(output_basis_value(1_100.0, 42.0), 46_200.0);
/// ```
pub fn output_basis_value(annual_extra_appointments: f64, value_per_appointment: f64) -> f64 {
    annual_extra_appointments * value_per_appointment
}

/// Level 3 — bottleneck basis: hours × value of pathway throughput released per bottleneck hour.
///
/// When the role gates a whole pathway (theory of constraints), an hour of
/// its time is worth the pathway throughput it releases — typically far more
/// than either the wage or the single-appointment value.
///
/// # Arguments
///
/// * `hours_saved` — bottleneck-role hours released per year.
/// * `pathway_value_per_hour` — £ value of pathway throughput released per
///   bottleneck hour.
///
/// # Returns
///
/// Bottleneck-basis value in £/year.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::bottleneck_basis_value;
///
/// // If each surgeon-hour gates £500 of theatre-pathway throughput,
/// // 220 released hours are worth £110,000/year.
/// assert_eq!(bottleneck_basis_value(220.0, 500.0), 110_000.0);
/// ```
pub fn bottleneck_basis_value(hours_saved: f64, pathway_value_per_hour: f64) -> f64 {
    hours_saved * pathway_value_per_hour
}

/// Fragmentation discount: scale a raw value by a stated utilization factor.
///
/// Time saved in scraps below a usable quantum (e.g., 3 minutes scattered
/// across a clinic) redeploys poorly: 12 × 5-minute savings ≠ one free hour.
/// State the utilization factor explicitly rather than assuming 100%.
///
/// # Arguments
///
/// * `raw_value` — undiscounted value in £ (any of the three bases).
/// * `utilization_factor` — fraction in [0, 1] of nominal saved time that
///   consolidates into usable, redeployable blocks.
///
/// # Returns
///
/// The discounted value: `raw_value × utilization_factor`.
///
/// # Examples
///
/// ```rust
/// use health_economics::practitioner_time::fragmentation_adjusted_value;
///
/// // If only half the nominal minutes consolidate into usable blocks,
/// // the £46,200 output-basis value falls to £23,100.
/// assert_eq!(fragmentation_adjusted_value(46_200.0, 0.5), 23_100.0);
/// ```
pub fn fragmentation_adjusted_value(raw_value: f64, utilization_factor: f64) -> f64 {
    raw_value * utilization_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-9;

    /// Ambient scribing saves 2 minutes × 30 consultations = 60 minutes/day.
    #[test]
    fn scribing_saves_60_minutes_per_day() {
        // Worked example: "2 minutes per consultation, 30 consultations/day: 60 minutes/day."
        let m = daily_minutes_saved(2.0, 30.0);
        assert!((m - 60.0).abs() < TOL);
    }

    /// 60 minutes/day over 220 working days = 220 hours/year per GP.
    #[test]
    fn annual_saving_is_220_hours_per_gp() {
        // Worked example: "220 hours/year per GP over 220 working days."
        let h = annual_hours_saved(60.0, 220.0);
        assert!((h - 220.0).abs() < TOL);
    }

    /// Wage basis: 220 hours × £80 loaded GP hour ≈ £17,600/GP/year.
    #[test]
    fn wage_basis_is_17_600_per_gp_year() {
        // Worked example: "Wage basis: 220 × £80 ≈ £17,600/GP/year."
        let v = wage_basis_value(220.0, 80.0);
        assert!((v - 17_600.0).abs() < TOL);
    }

    /// 60 min/day = 5 extra 12-minute consultations per day.
    #[test]
    fn sixty_minutes_enables_5_extra_consultations_per_day() {
        // Worked example: "Output basis: 60 min/day = 5 extra 12-min consultations/day."
        let a = extra_appointments_per_day(60.0, 12.0).unwrap();
        assert!((a - 5.0).abs() < TOL);
    }

    /// 5/day × 220 days = 1,100 extra appointments/GP/year.
    #[test]
    fn annual_extra_appointments_are_1_100() {
        // Worked example: "= 1,100 extra appointments/GP/year."
        let a = annual_extra_appointments(5.0, 220.0);
        assert!((a - 1_100.0).abs() < TOL);
    }

    /// Output basis: 1,100 appointments × £42 ≈ £46,200/GP/year.
    #[test]
    fn output_basis_is_46_200_per_gp_year() {
        // Worked example: "1,100 extra appointments/GP/year × £42 ≈ £46,200/GP/year."
        let v = output_basis_value(1_100.0, 42.0);
        assert!((v - 46_200.0).abs() < TOL);
    }

    /// Across a 50-GP federation the output-basis capacity is worth
    /// ~£2.3M/year (exactly £2,310,000).
    #[test]
    fn federation_output_basis_is_about_2_point_3_million() {
        // Worked example: "Across a 50-GP federation the output-basis
        // capacity is worth ~£2.3M/year."
        let v = 50.0 * output_basis_value(1_100.0, 42.0);
        assert!((v - 2_310_000.0).abs() < TOL);
        assert!((v - 2_300_000.0).abs() < 50_000.0);
    }

    /// A stated utilization factor discounts fragmented scraps of time.
    #[test]
    fn fragmentation_discount_applies_utilization_factor() {
        // Doc math: "time saved in scraps below a usable quantum redeploys
        // poorly; apply a stated utilization factor."
        let v = fragmentation_adjusted_value(46_200.0, 0.5);
        assert!((v - 23_100.0).abs() < TOL);
    }

    // Edge case: a zero-length appointment makes the division undefined.
    #[test]
    fn zero_length_appointment_is_undefined() {
        assert!(extra_appointments_per_day(60.0, 0.0).is_none());
    }
}
