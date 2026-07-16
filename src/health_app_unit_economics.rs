//! # Health App Unit Economics
//!
//! The commercial arithmetic of consumer health products: customer
//! acquisition cost (CAC), lifetime value (LTV), average revenue per user
//! (ARPU), per-member-per-month (PMPM) pricing, and the employer-market
//! distinction between ROI and VOI (value on investment).
//!
//! Health apps face a structural squeeze: acquisition is expensive
//! (regulated claims, trust barriers, compliance costs) while retention is
//! the worst of any software vertical (~90% abandonment within 30 days).
//! The standard viability test — **LTV:CAC ≥ 3:1** — is therefore brutally
//! hard in consumer health, which is why the industry migrates toward B2B2C
//! models: employers, insurers, and health systems paying PMPM for
//! populations, where the buyer is not the churning individual.
//!
//! ## Formula
//!
//! ```text
//! CAC   = sales + marketing spend / new paying customers
//! ARPU  = revenue / active users (per period)
//! LTV   = ARPU × average lifetime  =  ARPU / churn rate
//! Viability: LTV : CAC ≥ 3, payback period ≤ 12–18 months
//!
//! Effective CAC per retained user = CAC / retention(t)
//! PMPM revenue = rate × enrolled members × months
//!   vendor margin = PMPM − cost-to-serve per member per month
//! Health value per acquired user = retention-weighted QALYs × threshold
//!
//! churn rate   = fraction of customers lost per period (same period as ARPU)
//! retention(t) = fraction of acquired users still active at time t (0–1)
//! PMPM         = £ per covered member per month
//! threshold    = cost-effectiveness threshold (£/QALY)
//! ```
//!
//! ## Why it matters
//!
//! At 4% D30 retention, £5 per install becomes £125 per 30-day-retained
//! user — acquisition spend must be judged against the users who actually
//! stay. Under PMPM the engagement sign flips: under B2C subscriptions
//! engagement drives revenue; under PMPM, engaged members COST more to
//! serve than dormant ones — and outcomes contracts flip it back again.
//!
//! ## Example
//!
//! A B2C sleep app: £6.99/month, monthly churn 18%, blended CAC £38 — then
//! a pivot to employer PMPM at £1.20 across 40,000 covered lives.
//!
//! ```
//! use health_economics::health_app_unit_economics::{
//!     ltv, ltv_cac_ratio, is_viable_ltv_cac, pmpm_revenue, pmpm_margin_fraction,
//! };
//!
//! // LTV = 6.99 / 0.18 ≈ £38.8 → LTV:CAC ≈ 1.0 — non-viable.
//! let ltv_value = ltv(6.99, 0.18).unwrap();
//! assert!((ltv_value - 38.8).abs() < 0.05);
//! let ratio = ltv_cac_ratio(ltv_value, 38.0).unwrap();
//! assert!((ratio - 1.0).abs() < 0.05);
//! assert!(!is_viable_ltv_cac(ltv_value, 38.0));
//!
//! // Pivot: £1.20 PMPM × 40,000 covered lives = £48k/month.
//! assert_eq!(pmpm_revenue(1.20, 40_000.0, 1.0), 48_000.0);
//!
//! // Cost-to-serve £0.30/member (infra £0.15 + support £0.10 + content £0.05)
//! // → margin ~75%.
//! let margin = pmpm_margin_fraction(1.20, 0.30).unwrap();
//! assert!((margin - 0.75).abs() < 1e-9);
//! ```
//!
//! The employer's question shifts the metric: hard-dollar ROI (reduced
//! claims, absenteeism) is rarely demonstrable for wellness products — the
//! industry answer is VOI, which is honest only when labeled as VOI, not
//! dressed as ROI.
//!
//! ## Software engineering connection
//!
//! - Engineering choices set both sides of the ratio: **cost-to-serve** is
//!   architecture — the PMPM margin lives or dies on per-member
//!   infrastructure cost.
//! - **LTV is retention engineering**: each churn-point is arithmetic
//!   revenue.
//! - The unit-economics dashboard should carry a third line beside LTV and
//!   CAC: **health value per acquired user** (retention-weighted QALYs ×
//!   threshold) — payer and DiGA-style markets increasingly price on it.
//! - A product whose commercial and clinical unit economics diverge
//!   (profitable but health-inert, or effective but unfundable) needs to
//!   know which problem it has.
//!
//! ## Pitfalls
//!
//! - **LTV from early-cohort churn**: churn stabilizes downward; but also
//!   survivorship — early adopters retain better than scaled audiences. Use
//!   cohort-matured data.
//! - **CAC blended over channels**: paid-social CAC and clinician-referral
//!   CAC differ 10×, with opposite retention profiles — segment or be
//!   misled.
//! - **PMPM without utilization caps**: outlier-engaged members can invert
//!   margins; model the distribution, not the mean.
//! - **VOI presented as ROI** to a CFO — the credibility failure the
//!   employer-wellness industry spent a decade earning.
//!
//! ## Sources
//!
//! - Healthtech unit economics primers.
//!   <https://smart-it.io/blog/how-to-calculate-unit-economics-for-healthcare-startups/>
//! - PMPM pricing frameworks for digital health.
//!   <https://www.quintupleaim.com/blog/strategic-pricing-for-digital-health-startups-in-value-based-care-per-member-per-month-pmpm-frameworks>
//!
//! Topic doc: health-economics-metrics/topics/health-app-unit-economics.md

/// Customer acquisition cost: sales + marketing spend over new paying
/// customers.
///
/// Segment by channel before trusting it — paid-social CAC and
/// clinician-referral CAC differ 10×, with opposite retention profiles.
///
/// # Arguments
///
/// * `sales_and_marketing_spend` — total acquisition spend for the period
///   (£).
/// * `new_paying_customers` — customers acquired in the period (count).
///
/// # Returns
///
/// £ per new paying customer, or `None` if `new_paying_customers` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::cac;
///
/// // £38,000 spend for 1,000 new customers = the worked example's £38 CAC.
/// assert_eq!(cac(38_000.0, 1_000.0), Some(38.0));
/// assert!(cac(1.0, 0.0).is_none());
/// ```
pub fn cac(sales_and_marketing_spend: f64, new_paying_customers: f64) -> Option<f64> {
    if new_paying_customers == 0.0 {
        None
    } else {
        Some(sales_and_marketing_spend / new_paying_customers)
    }
}

/// Average revenue per user for a period.
///
/// # Arguments
///
/// * `revenue` — revenue in the period (£).
/// * `active_users` — users active in the same period (count).
///
/// # Returns
///
/// £ per active user per period, or `None` if `active_users` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::arpu;
///
/// // £6,990/month across 1,000 subscribers = £6.99 ARPU.
/// assert_eq!(arpu(6_990.0, 1_000.0), Some(6.99));
/// ```
pub fn arpu(revenue: f64, active_users: f64) -> Option<f64> {
    if active_users == 0.0 { None } else { Some(revenue / active_users) }
}

/// Lifetime value: `ARPU / churn rate`.
///
/// Equivalent to ARPU × average lifetime, since average lifetime =
/// 1 / churn under constant churn. The churn period must match ARPU's
/// period (monthly ARPU needs monthly churn). Use cohort-matured churn —
/// early-cohort churn misleads in both directions.
///
/// # Arguments
///
/// * `arpu` — average revenue per user per period (£).
/// * `churn_rate` — fraction of customers lost per period (0–1).
///
/// # Returns
///
/// LTV in £, or `None` if `churn_rate` is zero (infinite lifetime).
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::ltv;
///
/// // Worked example: LTV = 6.99 / 0.18 ≈ £38.8.
/// let v = ltv(6.99, 0.18).unwrap();
/// assert!((v - 38.8).abs() < 0.05);
/// assert!(ltv(6.99, 0.0).is_none());
/// ```
pub fn ltv(arpu: f64, churn_rate: f64) -> Option<f64> {
    // LTV = ARPU × average lifetime, and average lifetime = 1 / churn.
    if churn_rate == 0.0 { None } else { Some(arpu / churn_rate) }
}

/// LTV:CAC ratio — the standard viability test asks for ≥ 3.
///
/// # Arguments
///
/// * `ltv` — lifetime value (£).
/// * `cac` — customer acquisition cost (£).
///
/// # Returns
///
/// The ratio, or `None` if `cac` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::ltv_cac_ratio;
///
/// // Worked example: £38.8 LTV against £38 CAC ≈ 1.0 — far below the 3:1 bar.
/// let r = ltv_cac_ratio(38.83, 38.0).unwrap();
/// assert!((r - 1.0).abs() < 0.05);
/// ```
pub fn ltv_cac_ratio(ltv: f64, cac: f64) -> Option<f64> {
    if cac == 0.0 { None } else { Some(ltv / cac) }
}

/// True if the LTV:CAC ratio clears the standard 3:1 viability bar.
///
/// Returns `false` when CAC is zero (no meaningful ratio). The full test
/// also asks for payback ≤ 12–18 months, not modeled here.
///
/// # Arguments
///
/// * `ltv` — lifetime value (£).
/// * `cac` — customer acquisition cost (£).
///
/// # Returns
///
/// `true` iff `ltv / cac ≥ 3.0`.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::is_viable_ltv_cac;
///
/// // Worked example: ≈1:1 is non-viable; 3:1 clears the bar.
/// assert!(!is_viable_ltv_cac(38.83, 38.0));
/// assert!(is_viable_ltv_cac(114.0, 38.0));
/// ```
pub fn is_viable_ltv_cac(ltv: f64, cac: f64) -> bool {
    matches!(ltv_cac_ratio(ltv, cac), Some(r) if r >= 3.0)
}

/// Effective CAC per retained user: acquisition cost inflated by retention.
///
/// Dividing CAC by retention re-prices acquisition against the users who
/// actually stay to time t.
///
/// # Arguments
///
/// * `cac` — cost per acquired user (£, e.g. per install).
/// * `retention_at_t` — fraction of acquired users still active at time t
///   (0–1, e.g. D30 retention).
///
/// # Returns
///
/// £ per retained user, or `None` if `retention_at_t` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::effective_cac_per_retained_user;
///
/// // Doc figure: at 4% D30 retention, £5 per install = £125 per retained user.
/// assert_eq!(effective_cac_per_retained_user(5.0, 0.04), Some(125.0));
/// ```
pub fn effective_cac_per_retained_user(cac: f64, retention_at_t: f64) -> Option<f64> {
    if retention_at_t == 0.0 { None } else { Some(cac / retention_at_t) }
}

/// PMPM revenue: `rate × enrolled members × months`.
///
/// B2B2C contract revenue — paid per covered life, whether or not the
/// member engages; churn is contract-level (annual), not user-level (daily).
///
/// # Arguments
///
/// * `pmpm_rate` — £ per member per month.
/// * `enrolled_members` — covered lives (count).
/// * `months` — contract months.
///
/// # Returns
///
/// Revenue in £.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::pmpm_revenue;
///
/// // Worked example: £1.20 PMPM × 40,000 covered lives = £48k/month.
/// assert_eq!(pmpm_revenue(1.20, 40_000.0, 1.0), 48_000.0);
/// ```
pub fn pmpm_revenue(pmpm_rate: f64, enrolled_members: f64, months: f64) -> f64 {
    pmpm_rate * enrolled_members * months
}

/// Vendor margin per member per month: PMPM rate minus cost-to-serve.
///
/// Note the engagement sign flip: under PMPM, engaged members cost more to
/// serve than dormant ones; model the utilization distribution, not the
/// mean.
///
/// # Arguments
///
/// * `pmpm_rate` — £ per member per month.
/// * `cost_to_serve_pmpm` — £ cost to serve per member per month
///   (infrastructure + support + content).
///
/// # Returns
///
/// Margin in £ per member per month (negative if serving costs exceed the
/// rate).
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::pmpm_margin;
///
/// // Worked example: £1.20 − £0.30 = £0.90 margin per member-month.
/// assert!((pmpm_margin(1.20, 0.30) - 0.90).abs() < 1e-9);
/// ```
pub fn pmpm_margin(pmpm_rate: f64, cost_to_serve_pmpm: f64) -> f64 {
    pmpm_rate - cost_to_serve_pmpm
}

/// Vendor margin as a fraction of the PMPM rate.
///
/// # Arguments
///
/// * `pmpm_rate` — £ per member per month.
/// * `cost_to_serve_pmpm` — £ cost to serve per member per month.
///
/// # Returns
///
/// Margin fraction (e.g. `0.75` for 75%), or `None` if `pmpm_rate` is zero.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::pmpm_margin_fraction;
///
/// // Worked example: (1.20 − 0.30) / 1.20 = 75% margin.
/// let f = pmpm_margin_fraction(1.20, 0.30).unwrap();
/// assert!((f - 0.75).abs() < 1e-9);
/// ```
pub fn pmpm_margin_fraction(pmpm_rate: f64, cost_to_serve_pmpm: f64) -> Option<f64> {
    if pmpm_rate == 0.0 {
        None
    } else {
        Some(pmpm_margin(pmpm_rate, cost_to_serve_pmpm) / pmpm_rate)
    }
}

/// The third dashboard line: health value per acquired user.
///
/// Retention-weighted QALYs monetized at the cost-effectiveness threshold —
/// the clinical twin of LTV, which payer and DiGA-style markets increasingly
/// price on.
///
/// # Arguments
///
/// * `retention_weighted_qalys` — QALYs per acquired user after weighting by
///   realistic retention/engagement.
/// * `threshold_per_qaly` — cost-effectiveness threshold (£/QALY, e.g.
///   £20,000).
///
/// # Returns
///
/// Health value in £ per acquired user.
///
/// # Examples
///
/// ```
/// use health_economics::health_app_unit_economics::health_value_per_acquired_user;
///
/// // 0.01 retention-weighted QALYs at £20,000/QALY = £200 per acquired user.
/// assert_eq!(health_value_per_acquired_user(0.01, 20_000.0), 200.0);
/// ```
pub fn health_value_per_acquired_user(
    retention_weighted_qalys: f64,
    threshold_per_qaly: f64,
) -> f64 {
    retention_weighted_qalys * threshold_per_qaly
}

#[cfg(test)]
mod tests {
    use super::*;

    // Doc line: "LTV = 6.99 / 0.18 ≈ £38.8".
    #[test]
    fn worked_example_ltv_is_about_38_8() {
        let v = ltv(6.99, 0.18).unwrap();
        assert!((v - 38.8).abs() < 0.05, "got {v}");
    }

    // Doc line: "LTV:CAC ≈ 1.0 — non-viable" against blended CAC £38.
    #[test]
    fn worked_example_ltv_cac_is_about_1_and_non_viable() {
        let ltv_value = ltv(6.99, 0.18).unwrap();
        let ratio = ltv_cac_ratio(ltv_value, 38.0).unwrap();
        assert!((ratio - 1.0).abs() < 0.05, "got {ratio}");
        assert!(!is_viable_ltv_cac(ltv_value, 38.0));
    }

    // Doc line: "at 4% D30 retention, £5 per install = £125 per
    // 30-day-retained user".
    #[test]
    fn doc_math_effective_cac_is_125() {
        let v = effective_cac_per_retained_user(5.0, 0.04).unwrap();
        assert!((v - 125.0).abs() < 1e-9, "got {v}");
    }

    // Doc line: "£1.20 PMPM × 40,000 covered lives = £48k/month".
    #[test]
    fn worked_example_pmpm_revenue_is_48000_per_month() {
        let r = pmpm_revenue(1.20, 40_000.0, 1.0);
        assert!((r - 48_000.0).abs() < 1e-9, "got {r}");
    }

    // Doc line: "infrastructure £0.15 + support £0.10 + content £0.05 per
    // member ≈ £0.30".
    #[test]
    fn worked_example_cost_to_serve_is_0_30() {
        let cost: f64 = 0.15 + 0.10 + 0.05;
        assert!((cost - 0.30).abs() < 1e-9, "got {cost}");
    }

    // Doc line: "£0.30 → margin ~75%" at the £1.20 PMPM rate.
    #[test]
    fn worked_example_pmpm_margin_is_about_75_percent() {
        let m = pmpm_margin(1.20, 0.30);
        assert!((m - 0.90).abs() < 1e-9, "got {m}");
        let f = pmpm_margin_fraction(1.20, 0.30).unwrap();
        assert!((f - 0.75).abs() < 1e-9, "got {f}");
    }

    // Guard behavior: CAC and ARPU (and the other ratios) are guarded
    // against zero denominators; CAC reproduces the doc's £38.
    #[test]
    fn cac_and_arpu_are_guarded_ratios() {
        let c = cac(38_000.0, 1_000.0).unwrap();
        assert!((c - 38.0).abs() < 1e-9, "got {c}");
        let a = arpu(6_990.0, 1_000.0).unwrap();
        assert!((a - 6.99).abs() < 1e-9, "got {a}");
        assert!(cac(1.0, 0.0).is_none());
        assert!(arpu(1.0, 0.0).is_none());
        assert!(ltv(1.0, 0.0).is_none());
        assert!(ltv_cac_ratio(1.0, 0.0).is_none());
        assert!(effective_cac_per_retained_user(1.0, 0.0).is_none());
        assert!(pmpm_margin_fraction(0.0, 0.0).is_none());
    }

    // Doc connection: "health value per acquired user (retention-weighted
    // QALYs × threshold)" — the third dashboard line.
    #[test]
    fn health_value_per_acquired_user_is_qalys_times_threshold() {
        let v = health_value_per_acquired_user(0.01, 20_000.0);
        assert!((v - 200.0).abs() < 1e-9, "got {v}");
    }
}
