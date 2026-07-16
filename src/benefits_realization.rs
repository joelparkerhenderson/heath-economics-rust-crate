//! # Benefits Realization
//!
//! Benefits realization management (BRM) is the discipline of identifying,
//! baselining, tracking, and *evidencing* that the benefits promised in a
//! business case actually materialized after delivery. In UK public
//! investment it lives inside HM Treasury's Green Book **Five Case Model**;
//! in medicine, its cousin is post-market surveillance.
//!
//! Each benefit is classed (cash-releasing / non-cash-releasing /
//! qualitative) and tracked separately, with a named owner, a pre-go-live
//! baseline, and a measurement schedule. Forecasts are adjusted for optimism
//! bias at appraisal, and observed realization rates feed back into how much
//! the next forecast is discounted.
//!
//! ## Formula
//!
//! ```text
//! Realization rate  = benefits realized / benefits forecast   (per benefit, per period)
//! Optimism error    = (forecast − realized) / forecast
//! Adjusted forecast = raw forecast × (1 − optimism error)
//!
//! realized  — value evidenced post-go-live, measured against a pre-go-live baseline
//! forecast  — value promised in the business case, in the benefit's own units
//! ```
//!
//! ## Why it matters
//!
//! Business cases are promises; benefits realization is the audit.
//! Evaluations of major NHS digital programmes repeatedly found forecast
//! benefits that never materialized — and when benefits weren't
//! cash-releasing, they did nothing for the bottom line of the trust. The
//! Green Book's response: every spending case must pass **five cases**
//! (strategic, economic, commercial, financial, management), with benefits
//! realization planned in the management case *before approval* — owners
//! named, baselines captured, measurement dates set. Without this, "the
//! software saved 30 minutes per nurse" remains vendor fiction forever.
//! A 64% realization rate is not failure — it is *knowledge*; unmeasured
//! cases claim 100% forever.
//!
//! ## Example
//!
//! An e-rostering business case promised £450k/year agency-spend reduction
//! (cash) and 8,000 ward-manager hours (capacity). Twelve months post-go-live
//! the ledger shows £287,000 realized (64%) and a time-motion sample shows
//! 5,100 hours (64%); fill-rate compliance beat forecast at 120%.
//!
//! ```rust
//! use health_economics::benefits_realization::{
//!     Benefit, BenefitClass, optimism_adjusted_forecast, optimism_error, realization_rate,
//! };
//!
//! // Agency spend: forecast £450,000, realized £287,000 → 64%.
//! let agency = Benefit {
//!     name: "Agency spend reduction".to_string(),
//!     class: BenefitClass::CashReleasing,
//!     forecast: 450_000.0,
//!     realized: 287_000.0,
//! };
//! let rate = agency.realization_rate().unwrap();
//! assert!((rate - 0.64).abs() < 0.005);
//!
//! // Manager hours: forecast 8,000 h, realized 5,100 h → 64%.
//! let rate = realization_rate(5_100.0, 8_000.0).unwrap();
//! assert!((rate - 0.64).abs() < 0.005);
//!
//! // Fill compliance: forecast +10pp, realized +12pp → 120%.
//! let rate = realization_rate(12.0, 10.0).unwrap();
//! assert!((rate - 1.20).abs() < 1e-9);
//!
//! // The review logs the ~36% optimism error and haircuts the next forecast
//! // (the doc's review action logs a 30%-optimism error for the model).
//! let err = optimism_error(450_000.0, 287_000.0).unwrap();
//! assert!((err - 163_000.0 / 450_000.0).abs() < 1e-9);
//! let next = optimism_adjusted_forecast(100_000.0, 0.30);
//! assert!((next - 70_000.0).abs() < 1e-9);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineering orgs approve platform investments on forecast benefits and
//!   almost never audit them — the exact pathology BRM fixes.
//! - The lightweight port: every proposal above a threshold names benefit
//!   owners, baseline metrics, and a T+6-month review date.
//! - Realization rates feed back into how much the org discounts that team's
//!   (or vendor's) next forecast.
//! - The MIT finding that ~95% of GenAI pilots showed no measurable P&L
//!   return is a benefits-realization result — the pilots that *did* return
//!   had trackable, owned benefit lines.
//! - Forecast → measure → recalibrate is the same loop as EVPI-priced
//!   pilots, run at portfolio scale.
//!
//! ## Pitfalls
//!
//! - **No pre-go-live baseline** — the fatal, unfixable omission.
//! - **Benefit orphanhood**: no named owner means no one collects the data
//!   and every review says "broadly on track."
//! - **Double-counted benefits across programmes** claiming the same freed
//!   capacity — keep a benefit register across the portfolio.
//! - **Realization theater**: measuring the easy qualitative wins while the
//!   cash lines go quietly unexamined.
//!
//! ## Sources
//!
//! - HM Treasury, Green Book and Five Case Model guidance.
//!   <https://www.gov.uk/government/collections/the-green-book-and-accompanying-guidance-and-documents>
//! - Global Digital Exemplar programme evaluation (NHS digital benefits lessons).
//!   <https://pmc.ncbi.nlm.nih.gov/articles/PMC8685936/>
//!
//! Topic doc: health-economics-metrics/topics/benefits-realization.md

/// Classification of a claimed benefit, per NHS/Green Book benefit frameworks.
///
/// Classes are tracked and reported separately: only cash-releasing lines
/// reduce actual expenditure; non-cash-releasing lines are capacity; and
/// qualitative lines are described, never scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenefitClass {
    /// Reduces actual expenditure (a budget line gets smaller).
    CashReleasing,
    /// Frees time or capacity that is reused rather than banked.
    NonCashReleasing,
    /// Described, not scored (e.g. compliance, experience).
    Qualitative,
}

/// One benefit line from a business case, with its forecast and the value
/// realized (measured against a pre-go-live baseline).
///
/// `forecast` and `realized` share the benefit's own units — pounds for a
/// cash line, hours for a capacity line, percentage points for a compliance
/// line — so the realization rate is dimensionless.
#[derive(Debug, Clone)]
pub struct Benefit {
    /// Human-readable benefit name (e.g. "Agency spend reduction").
    pub name: String,
    /// Benefit class; classes are tracked and reported separately.
    pub class: BenefitClass,
    /// Value promised in the business case, in the benefit's own units.
    pub forecast: f64,
    /// Value evidenced post-go-live, in the same units.
    pub realized: f64,
}

impl Benefit {
    /// Realization rate for this benefit line: realized / forecast.
    ///
    /// Dimensionless; 1.0 means the forecast was met exactly, values above
    /// 1.0 mean the benefit over-delivered.
    ///
    /// # Returns
    ///
    /// `Some(rate)`, or `None` when `forecast` is zero (rate undefined).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::benefits_realization::{Benefit, BenefitClass};
    ///
    /// // Agency spend line: £287,000 realized of £450,000 forecast ≈ 64%.
    /// let b = Benefit {
    ///     name: "Agency spend".to_string(),
    ///     class: BenefitClass::CashReleasing,
    ///     forecast: 450_000.0,
    ///     realized: 287_000.0,
    /// };
    /// assert!((b.realization_rate().unwrap() - 0.64).abs() < 0.005);
    /// ```
    pub fn realization_rate(&self) -> Option<f64> {
        realization_rate(self.realized, self.forecast)
    }
}

/// Realization rate = benefits realized / benefits forecast.
///
/// Computed per benefit, per period, in the benefit's own units (both
/// arguments must share units). Rates above 1.0 mean over-delivery.
///
/// # Arguments
///
/// * `realized` — value evidenced post-go-live, in the benefit's units.
/// * `forecast` — value promised in the business case, in the same units.
///
/// # Returns
///
/// `Some(realized / forecast)`, or `None` when `forecast` is zero (the rate
/// is undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::benefits_realization::realization_rate;
///
/// // Manager hours: 5,100 realized of 8,000 forecast = 63.75% ≈ 64%.
/// let rate = realization_rate(5_100.0, 8_000.0).unwrap();
/// assert!((rate - 0.6375).abs() < 1e-9);
///
/// // Fill compliance beat its forecast: +12pp realized vs +10pp → 120%.
/// assert!((realization_rate(12.0, 10.0).unwrap() - 1.20).abs() < 1e-9);
///
/// // A zero forecast has no defined rate.
/// assert!(realization_rate(5.0, 0.0).is_none());
/// ```
pub fn realization_rate(realized: f64, forecast: f64) -> Option<f64> {
    if forecast == 0.0 { None } else { Some(realized / forecast) }
}

/// Observed optimism error of a forecast: (forecast − realized) / forecast.
///
/// Positive means the forecast overshot (realized less than promised);
/// negative means it undershot. Dimensionless fraction of the forecast.
///
/// # Arguments
///
/// * `forecast` — value promised in the business case.
/// * `realized` — value evidenced post-go-live, in the same units.
///
/// # Returns
///
/// `Some(error)`, or `None` when `forecast` is zero (error undefined).
///
/// # Examples
///
/// ```rust
/// use health_economics::benefits_realization::optimism_error;
///
/// // Agency line overshot by (450k − 287k)/450k ≈ 36%.
/// let err = optimism_error(450_000.0, 287_000.0).unwrap();
/// assert!((err - 163_000.0 / 450_000.0).abs() < 1e-9);
/// assert!(err > 0.36 && err < 0.37);
/// ```
pub fn optimism_error(forecast: f64, realized: f64) -> Option<f64> {
    if forecast == 0.0 {
        None
    } else {
        Some((forecast - realized) / forecast)
    }
}

/// Adjust a raw forecast for a logged optimism error rate (Green Book
/// pattern): adjusted = raw × (1 − error).
///
/// Feeding realized error back into the next appraisal is the point of the
/// BRM loop: a team whose lines historically overshoot by 30% gets its next
/// raw forecast haircut by 30%.
///
/// # Arguments
///
/// * `raw_forecast` — the unadjusted forecast, in the benefit's units.
/// * `optimism_error_rate` — logged error as a fraction (e.g. 0.30 for the
///   30%-optimism error noted in the worked example's review).
///
/// # Returns
///
/// The haircut forecast, in the same units as `raw_forecast`.
///
/// # Examples
///
/// ```rust
/// use health_economics::benefits_realization::optimism_adjusted_forecast;
///
/// // The review logged a 30% optimism error; the next £100k raw forecast
/// // is presented as £70k.
/// let adjusted = optimism_adjusted_forecast(100_000.0, 0.30);
/// assert!((adjusted - 70_000.0).abs() < 1e-9);
/// ```
pub fn optimism_adjusted_forecast(raw_forecast: f64, optimism_error_rate: f64) -> f64 {
    raw_forecast * (1.0 - optimism_error_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn benefit(name: &str, class: BenefitClass, forecast: f64, realized: f64) -> Benefit {
        Benefit { name: name.to_string(), class, forecast, realized }
    }

    // Worked-example row: "Agency spend  £450,000  £287,000  64%".
    #[test]
    fn agency_spend_realization_rate_is_64_percent() {
        let b = benefit("Agency spend", BenefitClass::CashReleasing, 450_000.0, 287_000.0);
        let rate = b.realization_rate().unwrap();
        // Doc quotes 64% (rounded); exact is 287,000/450,000.
        assert!((rate - 287_000.0 / 450_000.0).abs() < 1e-9);
        assert!((rate - 0.64).abs() < 0.005);
    }

    // Worked-example row: "Manager hours  8,000  5,100  64%".
    #[test]
    fn manager_hours_realization_rate_is_64_percent() {
        let b = benefit("Manager hours", BenefitClass::NonCashReleasing, 8_000.0, 5_100.0);
        let rate = b.realization_rate().unwrap();
        assert!((rate - 0.6375).abs() < 1e-9);
        assert!((rate - 0.64).abs() < 0.005);
    }

    // Worked-example row: "Fill compliance  +10pp  +12pp  120%".
    #[test]
    fn fill_compliance_realization_rate_is_120_percent() {
        // Forecast +10pp, realized +12pp.
        let rate = realization_rate(12.0, 10.0).unwrap();
        assert!((rate - 1.20).abs() < 1e-9);
    }

    // Edge case implied by the formula "realized / forecast": a zero
    // forecast makes the rate (and the optimism error) undefined.
    #[test]
    fn zero_forecast_has_undefined_rate() {
        assert!(realization_rate(5.0, 0.0).is_none());
        assert!(optimism_error(0.0, 5.0).is_none());
    }

    // Worked-example review action: "forecast model's 30%-optimism error
    // logged → applied to the next case".
    #[test]
    fn optimism_error_feeds_next_forecast() {
        // Agency line overshot by (450k − 287k)/450k ≈ 36%.
        let err = optimism_error(450_000.0, 287_000.0).unwrap();
        assert!((err - 163_000.0 / 450_000.0).abs() < 1e-9);
        // Applying the logged error to the next raw forecast haircuts it.
        let adjusted = optimism_adjusted_forecast(450_000.0, err);
        assert!((adjusted - 287_000.0).abs() < 1e-6);
        // A logged 30% optimism error (as in the doc's review action).
        let adjusted_30 = optimism_adjusted_forecast(100_000.0, 0.30);
        assert!((adjusted_30 - 70_000.0).abs() < 1e-9);
    }
}
