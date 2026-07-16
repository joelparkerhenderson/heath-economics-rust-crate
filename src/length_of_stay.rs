//! # Length of Stay (LOS)
//!
//! Length of stay is the number of days from hospital admission to discharge
//! — the core flow-efficiency metric of inpatient care. UK acute means run
//! around 4–5 days; every excess day consumes a scarce bed and exposes the
//! patient to hospital-acquired risks.
//!
//! LOS is heavily right-skewed by long-stay outliers, so report mean AND
//! median; Little's Law connects the flow variables exactly as it does for
//! software queues.
//!
//! ## Formula
//!
//! ```text
//! LOS (per spell)  = discharge date − admission date
//! Average LOS      = occupied bed days / discharges
//! beds occupied    = admission rate × average LOS   (Little's Law)
//!
//! LOS            = length of stay, in days
//! spell          = one inpatient stay, admission through discharge
//! admission rate = patients admitted per day
//! ```
//!
//! Comparisons require case-mix adjustment (age, diagnosis, acuity), or you
//! are measuring who the hospital admits, not how it performs.
//!
//! ## Why it matters
//!
//! LOS drives almost everything in acute-hospital economics: bed capacity,
//! elective throughput, emergency flow, staffing. Reducing average LOS by
//! even fractions of a day at scale releases enormous capacity. LOS is also
//! a quality signal in both directions — too long suggests process failure
//! (delayed diagnostics, discharge paperwork, social-care waits); too short
//! can mean premature discharge, which shows up later as readmissions.
//!
//! ## Example
//!
//! A trust admits 40 emergency medical patients/day at mean LOS 6.0 days:
//! 240 beds permanently occupied. Discharge-coordination software cuts the
//! non-clinical tail of stays by 0.4 days on average.
//!
//! ```rust
//! use health_economics::length_of_stay::{
//!     annual_bed_days_freed, beds_freed, beds_occupied,
//! };
//!
//! // Baseline: 40 admissions/day × 6.0 days = 240 beds permanently occupied.
//! assert!((beds_occupied(40.0, 6.0) - 240.0).abs() < 1e-9);
//!
//! // After a 0.4-day cut: beds needed = 40 × 5.6 = 224 → 16 beds freed.
//! assert!((beds_occupied(40.0, 5.6) - 224.0).abs() < 1e-9);
//! let freed = beds_freed(40.0, 6.0, 5.6);
//! assert!((freed - 16.0).abs() < 1e-9);
//!
//! // 16 × 365 = 5,840 bed days/year released.
//! assert!((annual_bed_days_freed(freed) - 5_840.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - LOS is the hospital's cycle time; the improvement playbook is identical
//!   to delivery-flow work.
//! - Instrument the stages (admission → treatment → medically-fit →
//!   actually-discharged) and find where time pools — it's the handoffs.
//! - Remove wait states rather than adding capacity.
//! - The "medically fit for discharge but still occupying a bed" cohort is
//!   the hospital's version of a PR approved but not merged.
//! - Direct software opportunities: discharge task orchestration, diagnostic
//!   turnaround, e-prescribing of discharge meds, social-care referral
//!   integration.
//!
//! ## Pitfalls
//!
//! - **Mean-only reporting** — outliers dominate; a falling mean can hide a
//!   growing long-stay tail.
//! - **No case-mix adjustment** in before/after claims: admission thresholds
//!   change seasonally and secularly.
//! - **LOS reduction that reappears as readmission** — always pair LOS claims
//!   with 30-day readmission data.
//!
//! ## Sources
//!
//! - OECD, length of hospital stay indicator.
//!   <https://www.oecd.org/en/data/indicators/length-of-hospital-stay.html>
//! - NHS England, National Cost Collection.
//!   <https://www.england.nhs.uk/costing-in-the-nhs/national-cost-collection/>
//!
//! Topic doc: health-economics-metrics/topics/length-of-stay.md

/// Length of stay for one spell, in days: discharge day minus admission day.
///
/// Both arguments are day numbers on the same axis (e.g. days since an
/// epoch); the result is the elapsed inpatient days for the spell.
///
/// # Arguments
///
/// * `admission_day` — day of admission (day number).
/// * `discharge_day` — day of discharge (day number, same axis).
///
/// # Returns
///
/// The spell's length of stay in days (`discharge_day − admission_day`).
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::length_of_stay_days;
///
/// // Admitted day 10, discharged day 16 → a 6-day spell (the trust's mean).
/// assert!((length_of_stay_days(10.0, 16.0) - 6.0).abs() < 1e-9);
/// ```
pub fn length_of_stay_days(admission_day: f64, discharge_day: f64) -> f64 {
    discharge_day - admission_day
}

/// Average LOS = occupied bed days / discharges.
///
/// This is the standard reporting definition: total occupied bed days over a
/// period divided by the discharges in that period.
///
/// # Arguments
///
/// * `occupied_bed_days` — total bed days occupied in the period.
/// * `discharges` — number of discharges in the period.
///
/// # Returns
///
/// `Some(average LOS in days)`, or `None` when there were no discharges
/// (the average is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::average_length_of_stay;
///
/// // 240 occupied bed days across 40 discharges → mean LOS 6.0 days.
/// let avg = average_length_of_stay(240.0, 40.0).unwrap();
/// assert!((avg - 6.0).abs() < 1e-9);
///
/// assert!(average_length_of_stay(240.0, 0.0).is_none());
/// ```
pub fn average_length_of_stay(occupied_bed_days: f64, discharges: f64) -> Option<f64> {
    if discharges == 0.0 {
        None
    } else {
        Some(occupied_bed_days / discharges)
    }
}

/// Mean of a set of per-spell LOS values.
///
/// Report alongside [`median_length_of_stay`]: LOS is right-skewed by
/// long-stay outliers, and a falling mean can hide a growing tail.
///
/// # Arguments
///
/// * `spells` — per-spell LOS values in days.
///
/// # Returns
///
/// `Some(arithmetic mean in days)`, or `None` for an empty set.
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::mean_length_of_stay;
///
/// // One 61-day outlier drags the mean to 12.0 days.
/// let spells = [2.0, 3.0, 3.0, 4.0, 5.0, 6.0, 61.0];
/// let mean = mean_length_of_stay(&spells).unwrap();
/// assert!((mean - 12.0).abs() < 1e-9);
///
/// assert!(mean_length_of_stay(&[]).is_none());
/// ```
pub fn mean_length_of_stay(spells: &[f64]) -> Option<f64> {
    if spells.is_empty() {
        None
    } else {
        Some(spells.iter().sum::<f64>() / spells.len() as f64)
    }
}

/// Median of a set of per-spell LOS values.
///
/// For an even count, the midpoint of the two central values. Robust to the
/// long-stay tail that dominates the mean — report both.
///
/// # Arguments
///
/// * `spells` — per-spell LOS values in days (order does not matter; the
///   function sorts a copy).
///
/// # Returns
///
/// `Some(median in days)`, or `None` for an empty set.
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::median_length_of_stay;
///
/// // Same skewed spells whose mean is 12.0: the median is only 4.0 days.
/// let spells = [2.0, 3.0, 3.0, 4.0, 5.0, 6.0, 61.0];
/// let median = median_length_of_stay(&spells).unwrap();
/// assert!((median - 4.0).abs() < 1e-9);
///
/// assert!(median_length_of_stay(&[]).is_none());
/// ```
pub fn median_length_of_stay(spells: &[f64]) -> Option<f64> {
    if spells.is_empty() {
        return None;
    }
    let mut sorted = spells.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("LOS values must be comparable"));
    let n = sorted.len();
    if n % 2 == 1 {
        Some(sorted[n / 2])
    } else {
        // Even count: midpoint of the two central order statistics.
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    }
}

/// Little's Law: beds permanently occupied = admission rate × average LOS.
///
/// The same law that governs software queues (work in progress = arrival
/// rate × cycle time), applied to inpatient flow.
///
/// # Arguments
///
/// * `admissions_per_day` — arrival rate, patients admitted per day.
/// * `average_los_days` — average length of stay, days.
///
/// # Returns
///
/// The steady-state number of beds occupied.
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::beds_occupied;
///
/// // 40 emergency admissions/day at mean LOS 6.0 days → 240 beds occupied.
/// assert!((beds_occupied(40.0, 6.0) - 240.0).abs() < 1e-9);
/// ```
pub fn beds_occupied(admissions_per_day: f64, average_los_days: f64) -> f64 {
    admissions_per_day * average_los_days
}

/// Beds freed continuously when average LOS falls at a constant admission rate.
///
/// Applies Little's Law before and after: `rate × LOS_before − rate ×
/// LOS_after`. Negative when LOS rises.
///
/// # Arguments
///
/// * `admissions_per_day` — constant admission rate, patients per day.
/// * `los_before_days` — average LOS before the change, days.
/// * `los_after_days` — average LOS after the change, days.
///
/// # Returns
///
/// The number of beds freed continuously (a standing capacity release, not a
/// one-off).
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::beds_freed;
///
/// // A 0.4-day LOS cut at 40 admissions/day frees 16 beds continuously.
/// assert!((beds_freed(40.0, 6.0, 5.6) - 16.0).abs() < 1e-9);
/// ```
pub fn beds_freed(admissions_per_day: f64, los_before_days: f64, los_after_days: f64) -> f64 {
    beds_occupied(admissions_per_day, los_before_days)
        - beds_occupied(admissions_per_day, los_after_days)
}

/// Bed days released per year by beds freed continuously: beds × 365.
///
/// Value the released bed days by mechanism (refill/close/slack) — the freed
/// capacity is not automatically cash.
///
/// # Arguments
///
/// * `beds_freed` — number of beds freed continuously.
///
/// # Returns
///
/// Bed days per year (`beds_freed × 365`).
///
/// # Examples
///
/// ```rust
/// use health_economics::length_of_stay::annual_bed_days_freed;
///
/// // 16 beds freed × 365 = 5,840 bed days/year.
/// assert!((annual_bed_days_freed(16.0) - 5_840.0).abs() < 1e-9);
/// ```
pub fn annual_bed_days_freed(beds_freed: f64) -> f64 {
    beds_freed * 365.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 40 emergency admissions/day at mean LOS 6.0 days: 240 beds permanently
    /// occupied (40 × 6).
    #[test]
    fn baseline_occupies_240_beds() {
        // Worked example: "40 emergency medical patients/day at mean LOS 6.0
        // days: 240 beds permanently occupied (40 × 6)".
        let got = beds_occupied(40.0, 6.0);
        assert!((got - 240.0).abs() < 1e-9);
    }

    /// Discharge-coordination software cuts LOS by 0.4 days: beds needed
    /// = 40 × 5.6 = 224.
    #[test]
    fn reduced_los_needs_224_beds() {
        // Worked example: "Beds needed = 40 × 5.6 = 224".
        let got = beds_occupied(40.0, 6.0 - 0.4);
        assert!((got - 224.0).abs() < 1e-9);
    }

    /// 16 beds freed continuously.
    #[test]
    fn sixteen_beds_freed_continuously() {
        // Worked example: "→ 16 beds freed continuously".
        let got = beds_freed(40.0, 6.0, 5.6);
        assert!((got - 16.0).abs() < 1e-9);
    }

    /// 16 × 365 = 5,840 bed days/year.
    #[test]
    fn frees_5840_bed_days_per_year() {
        // Worked example: "= 16 × 365 = 5,840 bed days/year".
        let freed = beds_freed(40.0, 6.0, 5.6);
        let got = annual_bed_days_freed(freed);
        assert!((got - 5_840.0).abs() < 1e-9);
    }

    /// LOS per spell is discharge minus admission.
    #[test]
    fn spell_los_is_discharge_minus_admission() {
        // Doc's math: "LOS (per spell) = discharge date − admission date".
        assert!((length_of_stay_days(10.0, 16.0) - 6.0).abs() < 1e-9);
    }

    /// Average LOS = occupied bed days / discharges; undefined at zero
    /// discharges.
    #[test]
    fn average_los_divides_bed_days_by_discharges() {
        // Doc's math: "Average LOS = occupied bed days / discharges".
        let got = average_length_of_stay(240.0, 40.0).unwrap();
        assert!((got - 6.0).abs() < 1e-9);
        assert!(average_length_of_stay(240.0, 0.0).is_none());
    }

    /// Mean and median diverge under a right-skewed long-stay tail — report both.
    #[test]
    fn mean_and_median_diverge_with_long_stay_outliers() {
        // Doc's math: "report mean AND median; LOS is heavily right-skewed by
        // long-stay outliers".
        let spells = [2.0, 3.0, 3.0, 4.0, 5.0, 6.0, 61.0];
        let mean = mean_length_of_stay(&spells).unwrap();
        let median = median_length_of_stay(&spells).unwrap();
        assert!((mean - 12.0).abs() < 1e-9);
        assert!((median - 4.0).abs() < 1e-9);
        assert!(mean > median);
    }
}
