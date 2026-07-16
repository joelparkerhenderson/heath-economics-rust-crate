//! # Total Cost of Ownership (TCO)
//!
//! TCO is the full cost of a system over its life: acquisition or build,
//! integration, operation, maintenance, support, training, and
//! decommissioning. The uncomfortable baseline: maintenance is 50–80% of
//! software TCO — roughly three-quarters of lifetime cost arrives *after*
//! launch.
//!
//! ## Formula
//!
//! ```text
//! TCO = initial cost (build/licence + integration + data migration + training)
//!     + Σ_t [operations + maintenance + support + infrastructure + upgrades
//!            + compliance/assurance]_t / (1 + r)^t
//!     + decommissioning cost (exit, data extraction, parallel running)
//!
//! t   year index over the horizon
//! r   discount rate: 3.5% public sector (Green Book), 8–12% commercial
//!
//! Horizon: 3–5 years commercial, system-lifetime for clinical infrastructure
//! Benchmark: annual maintenance ≈ 15–20% of build cost; ~78% of lifetime
//! TCO post-launch
//! ```
//!
//! ## Why it matters
//!
//! Health technology assessment learned long ago that a drug's price is not
//! its cost — administration, monitoring, and managing side effects all
//! belong in the model. Software business cases that count only
//! build/license cost repeat the naive-drug-price error and systematically
//! understate the cost side of every ICER and budget impact they feed. For
//! NHS procurement, TCO discipline is what makes a digital product's
//! cost-effectiveness claim honest — and it is where cheap-looking options
//! lose. Benchmarks: annual maintenance ≈ 15–20% of build cost, ~78% of
//! lifetime TCO post-launch; ignore decommissioning and vendor lock-in
//! prices itself.
//!
//! ## Example
//!
//! Two options for an e-observations system over a 5-year horizon — vendor
//! SaaS (£250k licence, £180k integration, £120k/yr, £60k exit) versus
//! in-house build (£900k build, £150k integration, £190k/yr, £30k exit):
//!
//! ```rust
//! use health_economics::total_cost_of_ownership::{
//!     TcoProfile, tco_advantage,
//! };
//!
//! let saas = TcoProfile {
//!     initial_cost: 250_000.0,
//!     integration_and_training: 180_000.0,
//!     annual_run_cost: 120_000.0,
//!     horizon_years: 5,
//!     decommission_cost: 60_000.0,
//! };
//! let build = TcoProfile {
//!     initial_cost: 900_000.0,
//!     integration_and_training: 150_000.0,
//!     annual_run_cost: 190_000.0,
//!     horizon_years: 5,
//!     decommission_cost: 30_000.0,
//! };
//!
//! // Undiscounted TCO: £1,090,000 vs £2,030,000.
//! assert!((saas.undiscounted_tco() - 1_090_000.0).abs() < 1e-9);
//! assert!((build.undiscounted_tco() - 2_030_000.0).abs() < 1e-9);
//!
//! // SaaS wins by ~£940k (cost-minimization logic, equivalent outcomes).
//! assert!((tco_advantage(&saas, &build) - 940_000.0).abs() < 1e-9);
//!
//! // The £900k engineering estimate was only 44% of the build's true TCO.
//! let share = build.initial_cost_share().unwrap();
//! assert!((share - 0.44).abs() < 0.005);
//! ```
//!
//! ## Software engineering connection
//!
//! - Engineers underweight their own field's maintenance data when
//!   advocating builds: the 15–20%-of-build-cost rule means every £1M
//!   system quietly commits £150–200k/year of future capacity.
//! - That liability belongs on the same mental balance sheet as technical
//!   debt.
//! - TCO is the cost half of every metric in this repo: cost per
//!   deployment, cloud unit economics, and the denominator discipline HTA
//!   enforces on drug sponsors.
//! - When your product's price is challenged, a TCO comparison including
//!   the incumbent's true running costs is usually the strongest reframe
//!   available.
//!
//! ## Pitfalls
//!
//! - Launch-cost anchoring: comparing options at year-0 cost when the
//!   ranking reverses by year 3.
//! - Free-internal-labor fallacy: in-house maintenance costed at zero
//!   because "the team's already paid" — see opportunity cost.
//! - Ignoring exit costs: data egress, contract termination, and parallel
//!   running are where "cheap" SaaS gets expensive.
//! - Same-horizon violations: comparing a 3-year SaaS TCO against a
//!   10-year build amortization.
//!
//! ## Sources
//!
//! - IBM, total cost of ownership.
//!   <https://www.ibm.com/think/topics/total-cost-of-ownership>
//! - Software maintenance cost benchmarks.
//!   <https://pegotec.net/software-maintenance-cost-percentage-2026-industry-benchmarks/>
//!
//! Topic doc: health-economics-metrics/topics/total-cost-of-ownership.md

/// Cost profile of one option over a fixed horizon.
///
/// Year-0 costs (initial + integration/training) are undiscounted; running
/// costs accrue in years 1..=horizon; decommissioning falls at the end of
/// the horizon.
#[derive(Debug, Clone, Copy)]
pub struct TcoProfile {
    /// Year-0 build or licence cost.
    pub initial_cost: f64,
    /// Integration, data migration, and training (year 0).
    pub integration_and_training: f64,
    /// Annual running cost (operations, maintenance, support, infrastructure).
    pub annual_run_cost: f64,
    /// Horizon in years over which annual running costs accrue
    /// (3–5 years commercial; system-lifetime for clinical infrastructure).
    pub horizon_years: u32,
    /// Exit / decommissioning cost (data extraction, parallel running),
    /// incurred at the end of the horizon.
    pub decommission_cost: f64,
}

impl TcoProfile {
    /// Undiscounted TCO: initial + integration + horizon × annual run + exit.
    ///
    /// # Returns
    ///
    /// Total cost over the horizon at face value, currency units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::total_cost_of_ownership::TcoProfile;
    ///
    /// // Doc: vendor SaaS = 250k + 180k + 5 × 120k + 60k = £1,090,000.
    /// let saas = TcoProfile {
    ///     initial_cost: 250_000.0,
    ///     integration_and_training: 180_000.0,
    ///     annual_run_cost: 120_000.0,
    ///     horizon_years: 5,
    ///     decommission_cost: 60_000.0,
    /// };
    /// assert!((saas.undiscounted_tco() - 1_090_000.0).abs() < 1e-9);
    /// ```
    pub fn undiscounted_tco(&self) -> f64 {
        self.initial_cost
            + self.integration_and_training
            + self.annual_run_cost * self.horizon_years as f64
            + self.decommission_cost
    }

    /// Discounted TCO at rate `r`.
    ///
    /// Year-0 costs at face value, annual running costs discounted over
    /// years 1..=horizon, decommissioning discounted at the end of the
    /// horizon. With `discount_rate` = 0 this equals
    /// [`TcoProfile::undiscounted_tco`].
    ///
    /// # Arguments
    ///
    /// * `discount_rate` — annual discount rate as a fraction (0.035 for
    ///   the UK Green Book public-sector rate; 0.08–0.12 commercial).
    ///
    /// # Returns
    ///
    /// Present value of the option's total cost, currency units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::total_cost_of_ownership::TcoProfile;
    ///
    /// let saas = TcoProfile {
    ///     initial_cost: 250_000.0,
    ///     integration_and_training: 180_000.0,
    ///     annual_run_cost: 120_000.0,
    ///     horizon_years: 5,
    ///     decommission_cost: 60_000.0,
    /// };
    /// // Discounting at the Green Book 3.5% shrinks future running costs.
    /// assert!(saas.discounted_tco(0.035) < saas.undiscounted_tco());
    /// assert!((saas.discounted_tco(0.0) - saas.undiscounted_tco()).abs() < 1e-9);
    /// ```
    pub fn discounted_tco(&self, discount_rate: f64) -> f64 {
        // Running costs: year-t cost discounted by (1 + r)^t, t = 1..=horizon.
        let running: f64 = (1..=self.horizon_years)
            .map(|t| self.annual_run_cost / (1.0 + discount_rate).powi(t as i32))
            .sum();
        // Exit cost falls at the end of the horizon, so it gets the
        // horizon-year discount factor.
        let exit = self.decommission_cost / (1.0 + discount_rate).powi(self.horizon_years as i32);
        self.initial_cost + self.integration_and_training + running + exit
    }

    /// Share of undiscounted TCO represented by the year-0 build/licence
    /// cost — the launch-cost-anchoring check.
    ///
    /// A low share means most of the cost arrives after launch, so a
    /// comparison anchored on year-0 price is misleading.
    ///
    /// # Returns
    ///
    /// Fraction in 0..1, or `None` if the undiscounted TCO is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::total_cost_of_ownership::TcoProfile;
    ///
    /// // Doc: the £900k engineering estimate was only 44% of its true TCO.
    /// let build = TcoProfile {
    ///     initial_cost: 900_000.0,
    ///     integration_and_training: 150_000.0,
    ///     annual_run_cost: 190_000.0,
    ///     horizon_years: 5,
    ///     decommission_cost: 30_000.0,
    /// };
    /// let share = build.initial_cost_share().unwrap();
    /// assert!((share - 0.44).abs() < 0.005);
    /// ```
    pub fn initial_cost_share(&self) -> Option<f64> {
        let tco = self.undiscounted_tco();
        if tco == 0.0 {
            None
        } else {
            Some(self.initial_cost / tco)
        }
    }
}

/// TCO advantage of option A over option B (positive means A is cheaper),
/// on undiscounted TCO.
///
/// Cost-minimization logic: valid only when the two options deliver
/// materially equivalent outcomes over the same horizon. Comparing options
/// over different horizons is a pitfall in its own right.
///
/// # Arguments
///
/// * `a` — option A's cost profile.
/// * `b` — option B's cost profile.
///
/// # Returns
///
/// `b.undiscounted_tco() − a.undiscounted_tco()`, currency units.
///
/// # Examples
///
/// ```rust
/// use health_economics::total_cost_of_ownership::{TcoProfile, tco_advantage};
///
/// // Doc: SaaS (£1.09M) wins over in-house build (£2.03M) by ~£940k.
/// let saas = TcoProfile {
///     initial_cost: 250_000.0, integration_and_training: 180_000.0,
///     annual_run_cost: 120_000.0, horizon_years: 5, decommission_cost: 60_000.0,
/// };
/// let build = TcoProfile {
///     initial_cost: 900_000.0, integration_and_training: 150_000.0,
///     annual_run_cost: 190_000.0, horizon_years: 5, decommission_cost: 30_000.0,
/// };
/// assert!((tco_advantage(&saas, &build) - 940_000.0).abs() < 1e-9);
/// ```
pub fn tco_advantage(a: &TcoProfile, b: &TcoProfile) -> f64 {
    b.undiscounted_tco() - a.undiscounted_tco()
}

/// Benchmark estimate of annual maintenance: build cost × maintenance fraction.
///
/// The industry benchmark fraction is typically 0.15–0.20: every £1M
/// system quietly commits £150–200k/year of future capacity.
///
/// # Arguments
///
/// * `build_cost` — the system's build cost, currency units.
/// * `maintenance_fraction` — annual maintenance as a fraction of build
///   cost (typically 0.15–0.20).
///
/// # Returns
///
/// Estimated annual maintenance cost, currency units per year.
///
/// # Examples
///
/// ```rust
/// use health_economics::total_cost_of_ownership::annual_maintenance_benchmark;
///
/// // Doc: every £1M system commits £150–200k/year of future capacity.
/// assert!((annual_maintenance_benchmark(1_000_000.0, 0.15) - 150_000.0).abs() < 1e-9);
/// assert!((annual_maintenance_benchmark(1_000_000.0, 0.20) - 200_000.0).abs() < 1e-9);
/// ```
pub fn annual_maintenance_benchmark(build_cost: f64, maintenance_fraction: f64) -> f64 {
    build_cost * maintenance_fraction
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendor_saas() -> TcoProfile {
        TcoProfile {
            initial_cost: 250_000.0,
            integration_and_training: 180_000.0,
            annual_run_cost: 120_000.0,
            horizon_years: 5,
            decommission_cost: 60_000.0,
        }
    }

    fn in_house_build() -> TcoProfile {
        TcoProfile {
            initial_cost: 900_000.0,
            integration_and_training: 150_000.0,
            annual_run_cost: 190_000.0,
            horizon_years: 5,
            decommission_cost: 30_000.0,
        }
    }

    // Doc table: "Vendor SaaS ... Undiscounted TCO £1,090,000".
    #[test]
    fn saas_undiscounted_tco_is_1_090_000() {
        assert!((vendor_saas().undiscounted_tco() - 1_090_000.0).abs() < 1e-9);
    }

    // Doc table: "In-house build ... Undiscounted TCO £2,030,000".
    #[test]
    fn build_undiscounted_tco_is_2_030_000() {
        assert!((in_house_build().undiscounted_tco() - 2_030_000.0).abs() < 1e-9);
    }

    // Doc: "cost-minimization logic applies and SaaS wins by ~£940k".
    #[test]
    fn saas_wins_by_about_940k() {
        // Doc: SaaS wins by ~£940k (exact 940,000 undiscounted).
        let advantage = tco_advantage(&vendor_saas(), &in_house_build());
        assert!((advantage - 940_000.0).abs() < 1e-9);
    }

    // Doc: "The build option's engineering estimate (£900k) was only 44% of
    // its true TCO".
    #[test]
    fn build_estimate_is_only_44_percent_of_true_tco() {
        // Doc: the £900k engineering estimate was only 44% of its true TCO.
        let share = in_house_build().initial_cost_share().unwrap();
        assert!((share - 0.44).abs() < 0.005);
    }

    // Consistency: discounting at r = 0 must reproduce the undiscounted TCO.
    #[test]
    fn discounted_tco_at_zero_rate_equals_undiscounted() {
        let p = in_house_build();
        assert!((p.discounted_tco(0.0) - p.undiscounted_tco()).abs() < 1e-9);
    }

    // Doc (The math): "r: 3.5% public sector (Green Book)" — discounting
    // shrinks future running costs and leaves the SaaS-vs-build ranking intact.
    #[test]
    fn discounting_reduces_tco_and_preserves_the_ranking() {
        // Green Book 3.5%: future running costs shrink, ranking is unchanged.
        let saas = vendor_saas().discounted_tco(0.035);
        let build = in_house_build().discounted_tco(0.035);
        assert!(saas < vendor_saas().undiscounted_tco());
        assert!(build < in_house_build().undiscounted_tco());
        assert!(saas < build);
    }

    // Doc: "annual maintenance ≈ 15–20% of build cost ... every £1M system
    // quietly commits £150–200k/year of future capacity".
    #[test]
    fn maintenance_benchmark_is_150_to_200k_per_million_built() {
        // Doc: every £1M system commits £150–200k/year of future capacity.
        let low = annual_maintenance_benchmark(1_000_000.0, 0.15);
        let high = annual_maintenance_benchmark(1_000_000.0, 0.20);
        assert!((low - 150_000.0).abs() < 1e-9);
        assert!((high - 200_000.0).abs() < 1e-9);
    }
}
