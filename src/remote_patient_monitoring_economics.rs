//! # Remote Patient Monitoring Economics
//!
//! The reimbursement and cost-offset economics of monitoring patients at
//! home. In the US, remote patient monitoring (RPM) revenue comes from a
//! defined CPT-code stack gated by compliance rules: the 16-day device rule
//! (CPT 99454 requires at least 16 days of readings in a 30-day period) and
//! the 20-minute management rule (CPT 99457 requires at least 20 logged
//! minutes of clinical management per month).
//!
//! In national health services there is no CPT stack; the value case is
//! admission avoidance and virtual-ward (hospital-at-home) bed-day
//! substitution, netted against the cost of running the service.
//!
//! ## Formula
//!
//! ```text
//! RPM revenue (US) = enrolled × billing-compliant fraction × code stack PMPM
//!
//! NHS-style value  = admissions avoided × marginal admission cost
//!                  + bed days substituted × (inpatient − virtual-ward day cost)
//!                  − service cost (devices, platform, monitoring staff)
//!
//! enrolled                    number of patients on the RPM panel
//! billing-compliant fraction  share of patient-months meeting a code's gate
//! PMPM                        per-member-per-month revenue (USD)
//! marginal admission cost     cost actually avoided per admission averted
//! ```
//!
//! ## Why it matters
//!
//! RPM is where device data becomes billable healthcare. The US Medicare
//! structure (2025 national averages) is unusually explicit: CPT 99453 setup
//! ~$19.73 one-time, CPT 99454 device supply ~$43.03 per 30 days (requires
//! >= 16 days of readings), CPT 99457 first 20 minutes of management ~$47.87
//! > (requires >= 20 logged minutes), CPT 99458 each additional 20 minutes
//! > ~$38.49. A compliant patient-month stacks to roughly $90–130 PMPM. On the
//! > cost-offset side, hospital-at-home programs (CMS Acute Hospital Care at
//! > Home waiver: 300+ hospitals) show ~$1,800–$3,000 saved per encounter
//! > versus inpatient care, with lower readmissions and infections.
//!
//! ## Example
//!
//! A US practice enrolls 400 hypertension patients; 70% meet the 16-day
//! threshold in a typical month and management minutes are logged for 60%:
//!
//! ```rust
//! use health_economics::remote_patient_monitoring_economics::{
//!     revenue_per_member_per_month, monthly_revenue, annual_revenue, annual_margin,
//!     compliance_lever_annual_gain, virtual_ward_gross_annual_value,
//!     CPT_99454_DEVICE_SUPPLY, CPT_99457_FIRST_20_MIN,
//! };
//!
//! // Monthly revenue ≈ 400 × [0.70 × 43.03 + 0.60 × 47.87] = 400 × 58.84 ≈ $23,500
//! let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
//! assert!((pmpm - 58.84).abs() < 0.005);
//! let monthly = monthly_revenue(400.0, pmpm);
//! assert!((monthly - 23_537.20).abs() < 0.01);
//!
//! // Annual ≈ $282,000; service cost ≈ $180,000 → margin ≈ $100k/year
//! let annual = annual_revenue(monthly);
//! assert!((annual - 282_446.40).abs() < 0.01);
//! let margin = annual_margin(annual, 180_000.0);
//! assert!((margin - 102_446.40).abs() < 0.01);
//!
//! // Raising the 16-day compliance from 70% → 85% adds ~$31k/year.
//! let gain = compliance_lever_annual_gain(400.0, 0.85 - 0.70, CPT_99454_DEVICE_SUPPLY);
//! assert!((gain - 30_981.60).abs() < 0.01);
//!
//! // NHS mirror: 50-bed virtual ward, 80% occupancy, £150 net saving/day ≈ £2.19M/year gross.
//! let vw = virtual_ward_gross_annual_value(50.0, 0.80, 150.0);
//! assert!((vw - 2_190_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - RPM platforms are the rare product where uptime and sync reliability
//!   convert directly to revenue: a week of failed syncs breaks the 16-day
//!   gate for a cohort.
//! - Audit-grade time-tracking (the 20-minute rule) is a first-class
//!   feature, not an afterthought.
//! - Build per-patient compliance dashboards that surface at-risk billing
//!   months while they are still recoverable.
//! - Keep timestamped, tamper-evident data trails — payer audits are routine.
//! - Tune alert economics: every alert consumes the monitoring team's
//!   minutes, which are both the billable unit and the scarce resource.
//!
//! ## Pitfalls
//!
//! - Enrollment ≠ revenue: the compliant fraction is the number; model it,
//!   don't assume it.
//! - US codes transplanted into NHS cases — national health services buy
//!   admission avoidance, not CPT stacks; run the second model.
//! - Offset claims at average cost for admissions whose fixed costs remain.
//! - Monitoring-team saturation: alert volume scales with enrollment; the
//!   staffing line is the binding constraint most models omit.
//!
//! ## Sources
//!
//! - RPM CPT codes and 2025 rates.
//!   <https://blog.prevounce.com/quick-guide-remote-patient-monitoring-rpm-cpt-codes-to-know>
//! - CMS hospital-at-home outcomes reporting.
//!   <https://www.mcknightshomecare.com/news/hospital-at-home-achieved-cost-savings-among-all-top-diagnosis-groups-cms-reports/>
//! - Telehealth.HHS.gov, billing for RPM.
//!   <https://telehealth.hhs.gov/providers/best-practice-guides/telehealth-and-remote-patient-monitoring/billing-remote-patient>
//!
//! Topic doc: health-economics-metrics/topics/remote-patient-monitoring-economics.md

/// CPT 99453: RPM setup and patient education, one-time (2025 national average, USD ~$19.73).
///
/// Billable once per episode of care, after the patient has transmitted
/// 16 days of data.
pub const CPT_99453_SETUP: f64 = 19.73;

/// CPT 99454: device supply + data transmission per 30 days (2025 national average, USD ~$43.03).
///
/// The 16-day rule: requires at least 16 days of readings within the 30-day
/// period. This gate makes wear-time compliance a revenue variable.
pub const CPT_99454_DEVICE_SUPPLY: f64 = 43.03;

/// CPT 99457: first 20 minutes/month of care management (2025 national average, USD ~$47.87).
///
/// The 20-minute rule: requires at least 20 logged minutes of clinical
/// management time in the month, making audit-grade time logging an
/// engineering requirement.
pub const CPT_99457_FIRST_20_MIN: f64 = 47.87;

/// CPT 99458: each additional 20 minutes of management per month (2025 national average, USD ~$38.49).
///
/// Billable on top of CPT 99457 for each further 20-minute block of logged
/// management time.
pub const CPT_99458_ADDITIONAL_20_MIN: f64 = 38.49;

/// Expected per-member-per-month (PMPM) revenue across a panel.
///
/// The device-supply code is weighted by the fraction of patients meeting
/// the 16-day rule, and the management code by the fraction with >= 20
/// logged minutes. Enrollment alone earns nothing; only the compliant
/// fractions bill.
///
/// # Arguments
///
/// * `device_compliant_fraction` — share of patient-months meeting the
///   16-day readings gate (0..1).
/// * `device_supply_rate` — device-supply reimbursement per 30 days, USD
///   (e.g. [`CPT_99454_DEVICE_SUPPLY`]).
/// * `management_logged_fraction` — share of patient-months with >= 20
///   logged management minutes (0..1).
/// * `management_rate` — management reimbursement per month, USD
///   (e.g. [`CPT_99457_FIRST_20_MIN`]).
///
/// # Returns
///
/// Expected revenue per enrolled member per month, USD.
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::{
///     revenue_per_member_per_month, CPT_99454_DEVICE_SUPPLY, CPT_99457_FIRST_20_MIN,
/// };
///
/// // Doc: 0.70 × 43.03 + 0.60 × 47.87 = 58.84 PMPM (rounded)
/// let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
/// assert!((pmpm - 58.84).abs() < 0.005);
/// ```
pub fn revenue_per_member_per_month(
    device_compliant_fraction: f64,
    device_supply_rate: f64,
    management_logged_fraction: f64,
    management_rate: f64,
) -> f64 {
    device_compliant_fraction * device_supply_rate + management_logged_fraction * management_rate
}

/// Monthly RPM revenue across an enrolled panel.
///
/// # Arguments
///
/// * `enrolled` — number of patients enrolled on the RPM panel.
/// * `revenue_pmpm` — expected per-member-per-month revenue, USD (from
///   [`revenue_per_member_per_month`]).
///
/// # Returns
///
/// Total panel revenue per month, USD.
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::monthly_revenue;
///
/// // Doc: 400 × 58.84 ≈ $23,500/month
/// let monthly = monthly_revenue(400.0, 58.84);
/// assert!((monthly - 23_536.0).abs() < 1.0);
/// ```
pub fn monthly_revenue(enrolled: f64, revenue_pmpm: f64) -> f64 {
    enrolled * revenue_pmpm
}

/// Annual RPM revenue: 12 × monthly revenue.
///
/// # Arguments
///
/// * `monthly_revenue` — panel revenue per month, USD.
///
/// # Returns
///
/// Panel revenue per year, USD.
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::annual_revenue;
///
/// // Doc: monthly $23,537.20 → annual ≈ $282,000 (exact 282,446.40)
/// let annual = annual_revenue(23_537.20);
/// assert!((annual - 282_446.40).abs() < 0.01);
/// ```
pub fn annual_revenue(monthly_revenue: f64) -> f64 {
    monthly_revenue * 12.0
}

/// Annual margin: annual revenue minus annual service cost.
///
/// Service cost covers devices, platform, and monitoring staff — the full
/// running cost of the RPM service, not just the software.
///
/// # Arguments
///
/// * `annual_revenue` — panel revenue per year, USD.
/// * `annual_service_cost` — cost of devices, platform, and monitoring
///   staff per year, USD.
///
/// # Returns
///
/// Net margin per year, USD (negative if the service runs at a loss).
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::annual_margin;
///
/// // Doc: annual ≈ $282,000, service cost ≈ $180,000 → margin ≈ $100k/year
/// let margin = annual_margin(282_446.40, 180_000.0);
/// assert!((margin - 102_446.40).abs() < 0.01);
/// ```
pub fn annual_margin(annual_revenue: f64, annual_service_cost: f64) -> f64 {
    annual_revenue - annual_service_cost
}

/// Annual revenue gained by raising the 16-day device-compliance fraction.
///
/// This is an engineering lever: device comfort, sync reliability, and
/// reminder design all move the compliant fraction, and each point of
/// compliance bills the device-supply code for more patient-months.
///
/// # Arguments
///
/// * `enrolled` — number of patients enrolled on the RPM panel.
/// * `compliance_fraction_increase` — increase in the compliant fraction
///   (e.g. `0.85 - 0.70 = 0.15` for a move from 70% to 85%).
/// * `device_supply_rate` — device-supply reimbursement per 30 days, USD
///   (e.g. [`CPT_99454_DEVICE_SUPPLY`]).
///
/// # Returns
///
/// Additional revenue per year, USD.
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::{
///     compliance_lever_annual_gain, CPT_99454_DEVICE_SUPPLY,
/// };
///
/// // Doc: raising 16-day compliance from 70% → 85% on 400 patients adds ~$31k/year.
/// let gain = compliance_lever_annual_gain(400.0, 0.85 - 0.70, CPT_99454_DEVICE_SUPPLY);
/// assert!((gain - 30_981.60).abs() < 0.01);
/// ```
pub fn compliance_lever_annual_gain(
    enrolled: f64,
    compliance_fraction_increase: f64,
    device_supply_rate: f64,
) -> f64 {
    // Extra compliant patient-months per month × device code rate × 12 months.
    enrolled * compliance_fraction_increase * device_supply_rate * 12.0
}

/// Gross annual value of a virtual ward substituting inpatient bed days.
///
/// Gross: before subtracting the platform, device, and community-nursing
/// costs of running the virtual ward.
///
/// # Arguments
///
/// * `beds` — number of virtual-ward beds.
/// * `occupancy_fraction` — average occupancy (0..1).
/// * `net_saving_per_day` — inpatient day cost minus virtual-ward day cost,
///   currency units per substituted bed day.
///
/// # Returns
///
/// Gross value per year (beds × occupancy × 365 × net saving/day).
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::virtual_ward_gross_annual_value;
///
/// // Doc: 50 beds × 0.8 × 365 × £150 ≈ £2.19M/year gross.
/// let value = virtual_ward_gross_annual_value(50.0, 0.80, 150.0);
/// assert!((value - 2_190_000.0).abs() < 1e-9);
/// ```
pub fn virtual_ward_gross_annual_value(
    beds: f64,
    occupancy_fraction: f64,
    net_saving_per_day: f64,
) -> f64 {
    beds * occupancy_fraction * 365.0 * net_saving_per_day
}

/// NHS-style net value of an RPM / virtual-ward service.
///
/// Admissions avoided are valued at marginal (not average) cost, bed days
/// substituted at the inpatient-minus-virtual-ward day-cost difference, and
/// the full service cost is subtracted.
///
/// # Arguments
///
/// * `admissions_avoided` — number of admissions averted per year.
/// * `marginal_admission_cost` — cost actually avoided per admission
///   (marginal, since fixed hospital costs remain).
/// * `bed_days_substituted` — inpatient bed days replaced by virtual-ward days.
/// * `inpatient_day_cost` — cost per inpatient bed day.
/// * `virtual_ward_day_cost` — cost per virtual-ward day.
/// * `service_cost` — annual cost of devices, platform, and monitoring staff.
///
/// # Returns
///
/// Net value per year (negative if the service costs more than it offsets).
///
/// # Examples
///
/// ```rust
/// use health_economics::remote_patient_monitoring_economics::nhs_style_net_value;
///
/// // Bed-day substitution only: 14,600 bed days at £400 − £250 = £150 net/day
/// // reproduces the £2.19M virtual-ward figure.
/// let value = nhs_style_net_value(0.0, 0.0, 14_600.0, 400.0, 250.0, 0.0);
/// assert!((value - 2_190_000.0).abs() < 1e-9);
/// ```
pub fn nhs_style_net_value(
    admissions_avoided: f64,
    marginal_admission_cost: f64,
    bed_days_substituted: f64,
    inpatient_day_cost: f64,
    virtual_ward_day_cost: f64,
    service_cost: f64,
) -> f64 {
    // Admission-avoidance term at marginal cost, plus per-day substitution
    // margin (inpatient − virtual ward), minus the cost of running the service.
    admissions_avoided * marginal_admission_cost
        + bed_days_substituted * (inpatient_day_cost - virtual_ward_day_cost)
        - service_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc worked example: "400 × [0.70 × 43.03 + 0.60 × 47.87] = 400 × 58.84".
    #[test]
    fn pmpm_stacks_to_58_84() {
        let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
        // Doc: 0.70 × 43.03 + 0.60 × 47.87 = 58.84 (rounded)
        assert!((pmpm - 58.84).abs() < 0.005);
    }

    // Doc worked example: "Monthly revenue ≈ ... ≈ $23,500".
    #[test]
    fn monthly_revenue_is_about_23_500() {
        let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
        let monthly = monthly_revenue(400.0, pmpm);
        // Doc: 400 × 58.84 ≈ $23,500 (exact 23,537.20)
        assert!((monthly - 23_500.0).abs() < 50.0);
    }

    // Doc worked example: "Annual ≈ $282,000".
    #[test]
    fn annual_revenue_is_about_282_000() {
        let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
        let annual = annual_revenue(monthly_revenue(400.0, pmpm));
        // Doc: annual ≈ $282,000 (exact 282,446.40)
        assert!((annual - 282_000.0).abs() < 500.0);
    }

    // Doc worked example: "service cost ... ≈ $180,000; Margin ≈ $100k/year".
    #[test]
    fn annual_margin_is_about_100k() {
        let pmpm = revenue_per_member_per_month(0.70, CPT_99454_DEVICE_SUPPLY, 0.60, CPT_99457_FIRST_20_MIN);
        let margin = annual_margin(annual_revenue(monthly_revenue(400.0, pmpm)), 180_000.0);
        // Doc: margin ≈ $100k/year (exact 102,446.40)
        assert!((margin - 100_000.0).abs() < 3_000.0);
    }

    // Doc worked example: "raising the 16-day compliance from 70% → 85% adds ~$31k/year".
    #[test]
    fn raising_compliance_70_to_85_adds_about_31k_per_year() {
        let gain = compliance_lever_annual_gain(400.0, 0.85 - 0.70, CPT_99454_DEVICE_SUPPLY);
        // Doc: ~$31k/year (exact 30,981.60)
        assert!((gain - 31_000.0).abs() < 100.0);
    }

    // Doc NHS mirror: "50 × 0.8 × 365 × 150 ≈ £2.19M/year gross".
    #[test]
    fn virtual_ward_50_beds_at_80pct_is_2_19m_gross() {
        let value = virtual_ward_gross_annual_value(50.0, 0.80, 150.0);
        // Doc: 50 × 0.8 × 365 × 150 = £2.19M/year (exact)
        assert!((value - 2_190_000.0).abs() < 1e-9);
    }

    // Cross-check: the general NHS-style formula reproduces the £2.19M
    // virtual-ward figure when only the bed-day substitution term is active.
    #[test]
    fn nhs_style_net_value_matches_virtual_ward_form() {
        // Bed-day substitution only: inpatient − virtual-ward day cost = £150 net,
        // 50 beds × 0.8 × 365 = 14,600 bed days, no admissions term, no service cost.
        let value = nhs_style_net_value(0.0, 0.0, 14_600.0, 400.0, 250.0, 0.0);
        assert!((value - 2_190_000.0).abs() < 1e-9);
    }
}
