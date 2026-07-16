//! # Probabilistic Sensitivity Analysis (PSA)
//!
//! PSA assigns a probability distribution to every uncertain parameter,
//! samples them all simultaneously thousands of times (Monte Carlo), and
//! reports the *probability* that an option is the best choice — instead of a
//! single point estimate.
//!
//! Randomness in this module is provided by a small seeded, deterministic
//! linear congruential generator ([`Lcg`]) local to this module (no external
//! crates), so every simulation is exactly reproducible from its seed.
//!
//! ## Formula
//!
//! ```text
//! For each of N draws (N ≈ 10,000):
//!   sample every parameter θ from its distribution
//!     (costs ~ Gamma, probabilities ~ Beta, utilities ~ Beta, effects ~ Normal/logNormal)
//!   compute NMB_j(θ) = λ × Effect_j(θ) − Cost_j(θ) for each option j
//!
//! CEAC_j(λ) = fraction of draws in which option j has the highest NMB at threshold λ
//! ```
//!
//! Legend:
//! - `N` — number of Monte Carlo draws (conventionally ≈ 10,000).
//! - `θ` — one joint sample of all uncertain parameters.
//! - `NMB_j(θ)` — net monetary benefit of option `j` under sample `θ`.
//! - `λ` — willingness-to-pay threshold (£ per unit of effect).
//! - `CEAC_j(λ)` — cost-effectiveness acceptability curve: probability that
//!   option `j` is the best choice at threshold `λ`.
//!
//! ## Why it matters
//!
//! NICE's reference case *requires* PSA. Deterministic analysis answers "what
//! if one input is wrong?"; PSA answers "given everything we don't know at
//! once, how likely is it that we're making the right call?" Its signature
//! output, the **cost-effectiveness acceptability curve (CEAC)**, plots the
//! probability an option is cost-effective against the willingness-to-pay
//! threshold — turning "the ICER is £24,000/QALY" into "there is a 78% chance
//! this is the right choice at £30,000/QALY."
//!
//! ## Example
//!
//! Platform migration business case with three uncertain inputs: migration
//! cost ~ Gamma(mean £800k, sd £200k), annual benefit ~ Normal(mean £350k,
//! sd £150k), benefit duration ~ Uniform(3–6 years). For each of 10,000 draws
//! compute net benefit = duration × annual − cost:
//!
//! ```rust
//! use health_economics::probabilistic_sensitivity_analysis::{
//!     mean, probability_positive, simulate_migration_net_benefits, MigrationCase,
//! };
//!
//! let case = MigrationCase {
//!     cost_mean: 800_000.0,
//!     cost_sd: 200_000.0,
//!     benefit_mean: 350_000.0,
//!     benefit_sd: 150_000.0,
//!     duration_low: 3.0,
//!     duration_high: 6.0,
//! };
//! let draws = simulate_migration_net_benefits(&case, 10_000, 42).unwrap();
//!
//! // Mean net benefit: £775k (analytically 4.5 × £350k − £800k).
//! let m = mean(&draws).unwrap();
//! assert!((m - 775_000.0).abs() < 30_000.0);
//!
//! // Probability net benefit > 0: ≈ 0.86.
//! let p = probability_positive(&draws).unwrap();
//! assert!((p - 0.86).abs() < 0.03);
//! ```
//!
//! The point estimate said "obviously yes." The PSA says "86% yes, with a
//! real tail where we lose £180k+" — which is what a portfolio owner actually
//! needs, and it prices the case for running a discovery spike first.
//!
//! ## Software engineering connection
//!
//! - Engineers already trust Monte Carlo for delivery forecasting
//!   (throughput sampling beats point estimates) — extend the same machinery
//!   to money.
//! - Put distributions on adoption, time saved, and salary, then report
//!   "probability this platform investment is net-positive" instead of a
//!   false-precision ROI.
//! - A CEAC-style curve — probability of being the best option as a function
//!   of how the org values an engineer-hour — is a genuinely better artifact
//!   for a funding committee than any single number.
//!
//! ## Pitfalls
//!
//! - **Garbage distributions**: PSA with made-up standard deviations is
//!   deterministic analysis wearing a lab coat. Base spreads on data or
//!   structured expert elicitation.
//! - **Ignoring correlation** between parameters (high adoption usually
//!   correlates with high time-saved); independent sampling understates tail
//!   risk.
//! - **Reporting only the mean** of the simulation — the entire point is the
//!   distribution and decision probability.
//!
//! ## Sources
//!
//! - Fenwick E, Claxton K, Sculpher M. "Representing uncertainty: the role of
//!   cost-effectiveness acceptability curves." Health Economics 2001.
//!   <https://pubmed.ncbi.nlm.nih.gov/11316594/>
//! - NICE health technology evaluations: the manual (PMG36).
//!   <https://www.nice.org.uk/process/pmg36>
//!
//! Topic doc: health-economics-metrics/topics/probabilistic-sensitivity-analysis.md

/// A tiny seeded, deterministic pseudo-random generator with just the sampling methods PSA needs.
///
/// A 64-bit linear congruential generator with output mixing. It is
/// deterministic by design: the same seed always reproduces the same draw
/// sequence, so a PSA run is exactly reproducible — a feature for audits and
/// regression tests, not a bug. Not cryptographically secure; statistical
/// quality is adequate for Monte Carlo at PSA scale.
pub struct Lcg {
    state: u64,
}

impl Lcg {
    /// Create a generator from a seed; the same seed always reproduces the same draw sequence.
    ///
    /// # Arguments
    ///
    /// * `seed` — any 64-bit value; distinct seeds give independent-looking
    ///   streams.
    ///
    /// # Returns
    ///
    /// A generator whose internal state has been scrambled once so that
    /// small seeds (0, 1, 2, …) do not produce correlated first draws.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// // Determinism: the same seed reproduces the same draws.
    /// let mut a = Lcg::new(42);
    /// let mut b = Lcg::new(42);
    /// assert_eq!(a.next_uniform(), b.next_uniform());
    /// ```
    pub fn new(seed: u64) -> Self {
        Lcg {
            // One LCG step applied to the seed itself, so consecutive seeds
            // start from well-separated states.
            state: seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407),
        }
    }

    fn next_u64(&mut self) -> u64 {
        // Knuth's MMIX LCG constants: multiplier 6364136223846793005 and
        // increment 1442695040888963407 give a full period of 2^64.
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Mix the high bits down; raw LCG low bits are weak.
        let x = self.state;
        // xorshift then multiply by the 64-bit golden-ratio constant
        // (0x9E3779B97F4A7C15) — a cheap avalanche step.
        (x ^ (x >> 31)).wrapping_mul(0x9E3779B97F4A7C15)
    }

    /// Uniform draw in [0, 1).
    ///
    /// # Returns
    ///
    /// A value in `[0, 1)` with 53 bits of precision (the full mantissa of an
    /// `f64`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// let mut rng = Lcg::new(1);
    /// let u = rng.next_uniform();
    /// assert!((0.0..1.0).contains(&u));
    /// ```
    pub fn next_uniform(&mut self) -> f64 {
        // Keep the top 53 bits so the result is an exact multiple of 2^-53:
        // uniform on [0, 1) without rounding bias.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform draw in [low, high).
    ///
    /// The conventional PSA distribution for bounded "we only know a range"
    /// parameters — e.g. benefit duration ~ Uniform(3, 6) years in the worked
    /// example.
    ///
    /// # Arguments
    ///
    /// * `low` — inclusive lower bound.
    /// * `high` — exclusive upper bound (must be ≥ `low` for a meaningful
    ///   draw).
    ///
    /// # Returns
    ///
    /// A value in `[low, high)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// // Benefit duration ~ Uniform(3, 6) years.
    /// let mut rng = Lcg::new(2);
    /// let d = rng.uniform(3.0, 6.0);
    /// assert!((3.0..6.0).contains(&d));
    /// ```
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.next_uniform()
    }

    /// Normal draw via the Box–Muller transform.
    ///
    /// The conventional PSA distribution for additive effects — e.g. annual
    /// benefit ~ Normal(mean £350k, sd £150k) in the worked example.
    ///
    /// # Arguments
    ///
    /// * `mean` — distribution mean.
    /// * `sd` — standard deviation (a negative value mirrors the
    ///   distribution; pass sd ≥ 0).
    ///
    /// # Returns
    ///
    /// One N(mean, sd²) sample.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// // Annual benefit ~ Normal(£350k, £150k): sample and sanity-check the
    /// // sample mean over many draws.
    /// let mut rng = Lcg::new(3);
    /// let m: f64 = (0..10_000).map(|_| rng.normal(350_000.0, 150_000.0)).sum::<f64>() / 10_000.0;
    /// assert!((m - 350_000.0).abs() < 10_000.0);
    /// ```
    pub fn normal(&mut self, mean: f64, sd: f64) -> f64 {
        // Box–Muller: two independent uniforms → one standard normal.
        // Use 1 − U so u1 ∈ (0, 1]: ln(0) would be −∞.
        let u1 = 1.0 - self.next_uniform(); // (0, 1]
        let u2 = self.next_uniform();
        // z = √(−2 ln u1) · cos(2π u2) is exactly N(0, 1); the sine branch
        // (the second Box–Muller variate) is discarded for simplicity.
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + sd * z
    }

    /// Gamma draw with the given shape and scale (Marsaglia–Tsang method).
    ///
    /// The conventional PSA distribution for costs: strictly positive and
    /// right-skewed. For the mean/sd parameterization used in worked
    /// examples, see [`Lcg::gamma_mean_sd`].
    ///
    /// # Arguments
    ///
    /// * `shape` — Gamma shape parameter k > 0.
    /// * `scale` — Gamma scale parameter θ > 0 (mean = k·θ, variance = k·θ²).
    ///
    /// # Returns
    ///
    /// One Gamma(shape, scale) sample (> 0 for valid parameters).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// // Gamma(shape 16, scale 50,000) has mean 16 × 50,000 = £800k.
    /// let mut rng = Lcg::new(4);
    /// let m: f64 = (0..10_000).map(|_| rng.gamma(16.0, 50_000.0)).sum::<f64>() / 10_000.0;
    /// assert!((m - 800_000.0).abs() < 20_000.0);
    /// ```
    pub fn gamma(&mut self, shape: f64, scale: f64) -> f64 {
        if shape < 1.0 {
            // Marsaglia–Tsang only covers shape ≥ 1; for shape < 1 use the
            // boost identity Gamma(shape) = Gamma(shape + 1) × U^(1/shape).
            let u = self.next_uniform().max(f64::MIN_POSITIVE);
            return self.gamma(shape + 1.0, scale) * u.powf(1.0 / shape);
        }
        // Marsaglia–Tsang constants: d = k − 1/3 and c = 1/√(9d) define the
        // squeeze transformation v = (1 + c·x)³.
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        // Rejection loop: propose x ~ N(0,1), accept d·v with high probability
        // (the method accepts >95% of proposals for shape ≥ 1).
        loop {
            let x = self.normal(0.0, 1.0);
            let v = (1.0 + c * x).powi(3);
            if v <= 0.0 {
                // v must be positive to be a valid Gamma candidate.
                continue;
            }
            let u = self.next_uniform();
            // Fast squeeze check first (avoids the logs); fall back to the
            // exact log acceptance test.
            if u < 1.0 - 0.0331 * x.powi(4)
                || u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln())
            {
                return d * v * scale;
            }
        }
    }

    /// Gamma draw parameterized by mean and standard deviation.
    ///
    /// The natural parameterization for PSA cost inputs — e.g. migration cost
    /// ~ Gamma(mean £800k, sd £200k) in the worked example. Internally
    /// converts to shape/scale via shape = (mean/sd)², scale = sd²/mean.
    ///
    /// # Arguments
    ///
    /// * `mean` — desired distribution mean (> 0), e.g. £800,000.
    /// * `sd` — desired standard deviation (> 0), e.g. £200,000.
    ///
    /// # Returns
    ///
    /// `Some(sample)`, or `None` when `mean` or `sd` is not strictly positive
    /// (shape/scale would be undefined).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use health_economics::probabilistic_sensitivity_analysis::Lcg;
    ///
    /// // Migration cost ~ Gamma(mean £800k, sd £200k): draws are positive.
    /// let mut rng = Lcg::new(5);
    /// let cost = rng.gamma_mean_sd(800_000.0, 200_000.0).unwrap();
    /// assert!(cost > 0.0);
    ///
    /// // Non-positive parameters are undefined.
    /// assert!(rng.gamma_mean_sd(0.0, 200_000.0).is_none());
    /// ```
    pub fn gamma_mean_sd(&mut self, mean: f64, sd: f64) -> Option<f64> {
        if mean <= 0.0 || sd <= 0.0 {
            None
        } else {
            // Method of moments: mean = shape × scale, var = shape × scale²
            // ⇒ shape = (mean/sd)², scale = sd²/mean.
            let shape = (mean / sd).powi(2);
            let scale = sd * sd / mean;
            Some(self.gamma(shape, scale))
        }
    }
}

/// Net monetary benefit at threshold λ: NMB = λ × effect − cost.
///
/// Converts an effect (e.g. QALYs) into money at the willingness-to-pay
/// threshold and subtracts cost, so options can be ranked on a single £
/// scale.
///
/// # Arguments
///
/// * `threshold` — willingness-to-pay λ, in £ per unit of effect.
/// * `effect` — health/benefit effect (e.g. QALYs).
/// * `cost` — cost in £.
///
/// # Returns
///
/// NMB in £: `threshold × effect − cost`.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::net_monetary_benefit;
///
/// // At λ = £30,000/QALY, 2 QALYs costing £24,000 give NMB = £36,000.
/// assert_eq!(net_monetary_benefit(30_000.0, 2.0, 24_000.0), 36_000.0);
/// ```
pub fn net_monetary_benefit(threshold: f64, effect: f64, cost: f64) -> f64 {
    threshold * effect - cost
}

/// CEAC point at one threshold: for each option, the fraction of draws in which it has the highest NMB.
///
/// `option_nmb_draws[j][i]` is option `j`'s NMB in draw `i`; all options must
/// supply the same number of draws (the draws must come from the same joint
/// parameter samples). The returned fractions sum to 1. Repeating this
/// computation across a grid of thresholds traces the full
/// cost-effectiveness acceptability curve.
///
/// # Arguments
///
/// * `option_nmb_draws` — one NMB-per-draw slice per option, all of equal
///   length.
///
/// # Returns
///
/// `Some(vec)` with one win-fraction per option, or `None` when there are no
/// options, no draws, or the options' draw counts differ. Ties credit the
/// earliest-listed option.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::ceac;
///
/// // Two options over four joint draws: each wins twice → 50% / 50%.
/// let a = [1.0, 5.0, 3.0, 2.0];
/// let b = [2.0, 1.0, 4.0, 1.0];
/// let c = ceac(&[&a, &b]).unwrap();
/// assert_eq!(c, vec![0.5, 0.5]);
///
/// // No options: undefined.
/// assert!(ceac(&[]).is_none());
/// ```
pub fn ceac(option_nmb_draws: &[&[f64]]) -> Option<Vec<f64>> {
    let n = option_nmb_draws.first()?.len();
    if n == 0 || option_nmb_draws.iter().any(|d| d.len() != n) {
        return None;
    }
    let mut wins = vec![0usize; option_nmb_draws.len()];
    // For each joint draw i, find the option with the highest NMB and count
    // it a win; strict '>' means ties go to the earliest-listed option.
    for i in 0..n {
        let mut best = 0;
        for j in 1..option_nmb_draws.len() {
            if option_nmb_draws[j][i] > option_nmb_draws[best][i] {
                best = j;
            }
        }
        wins[best] += 1;
    }
    // CEAC_j = wins_j / N: the probability option j is the best choice.
    Some(wins.iter().map(|&w| w as f64 / n as f64).collect())
}

/// Mean of a set of draws.
///
/// Note the pitfall: reporting *only* the mean defeats the purpose of PSA —
/// pair it with [`probability_positive`] and [`percentile`].
///
/// # Arguments
///
/// * `draws` — simulation outputs (e.g. net benefits in £).
///
/// # Returns
///
/// `Some(arithmetic mean)`, or `None` for an empty set.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::mean;
///
/// assert_eq!(mean(&[700_000.0, 850_000.0]), Some(775_000.0));
/// assert!(mean(&[]).is_none());
/// ```
pub fn mean(draws: &[f64]) -> Option<f64> {
    if draws.is_empty() {
        None
    } else {
        Some(draws.iter().sum::<f64>() / draws.len() as f64)
    }
}

/// Fraction of draws that are strictly positive — e.g. "probability net benefit > 0".
///
/// # Arguments
///
/// * `draws` — simulation outputs (e.g. net benefits in £).
///
/// # Returns
///
/// `Some(fraction in [0, 1])`, or `None` for an empty set. Draws exactly
/// equal to zero do not count as positive.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::probability_positive;
///
/// // 3 of 4 draws are net-positive.
/// assert_eq!(probability_positive(&[100.0, -50.0, 200.0, 1.0]), Some(0.75));
/// assert!(probability_positive(&[]).is_none());
/// ```
pub fn probability_positive(draws: &[f64]) -> Option<f64> {
    if draws.is_empty() {
        None
    } else {
        Some(draws.iter().filter(|&&x| x > 0.0).count() as f64 / draws.len() as f64)
    }
}

/// Empirical percentile (0–100) of a set of draws, by linear interpolation between order statistics.
///
/// Used to report tail risk: the worked example quotes the 5th–95th
/// percentile band of net benefit. `p` outside [0, 100] is clamped.
///
/// # Arguments
///
/// * `draws` — simulation outputs (unsorted is fine; a sorted copy is made).
/// * `p` — percentile in [0, 100], e.g. `5.0` for the 5th percentile.
///
/// # Returns
///
/// `Some(interpolated value)`, or `None` for an empty set.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::percentile;
///
/// let draws = [10.0, 20.0, 30.0, 40.0, 50.0];
/// assert_eq!(percentile(&draws, 50.0), Some(30.0));
/// assert_eq!(percentile(&draws, 25.0), Some(20.0));
/// assert!(percentile(&[], 50.0).is_none());
/// ```
pub fn percentile(draws: &[f64], p: f64) -> Option<f64> {
    if draws.is_empty() {
        return None;
    }
    let mut sorted = draws.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Fractional rank over the n−1 gaps between order statistics; lo/hi are
    // the neighboring order statistics and frac interpolates between them.
    let rank = (p / 100.0).clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    let frac = rank - lo as f64;
    Some(sorted[lo] + (sorted[hi] - sorted[lo]) * frac)
}

/// The worked example's platform-migration business case: three uncertain inputs with their PSA distributions.
///
/// Costs are modeled Gamma (positive, right-skewed), the annual benefit
/// Normal, and the benefit duration Uniform — the conventional distribution
/// choices for each parameter type.
pub struct MigrationCase {
    /// Migration cost ~ Gamma with this mean (£; worked example: £800,000).
    pub cost_mean: f64,
    /// Migration cost ~ Gamma with this standard deviation (£; worked
    /// example: £200,000).
    pub cost_sd: f64,
    /// Annual benefit ~ Normal with this mean (£/year; worked example:
    /// £350,000).
    pub benefit_mean: f64,
    /// Annual benefit ~ Normal with this standard deviation (£/year; worked
    /// example: £150,000).
    pub benefit_sd: f64,
    /// Benefit duration ~ Uniform lower bound (years; worked example: 3).
    pub duration_low: f64,
    /// Benefit duration ~ Uniform upper bound (years; worked example: 6).
    pub duration_high: f64,
}

/// Run the PSA: for each of `n` draws sample every parameter and compute net benefit.
///
/// Net benefit per draw = duration × annual benefit − cost (discounting
/// omitted, as in the worked example). The generator is deterministic for a
/// given seed, so results are exactly reproducible — rerun with the same
/// `case`, `n`, and `seed` to get identical draws.
///
/// # Arguments
///
/// * `case` — the three parameter distributions (see [`MigrationCase`]).
/// * `n` — number of Monte Carlo draws (worked example: 10,000).
/// * `seed` — seed for the deterministic generator.
///
/// # Returns
///
/// `Some(vec of n net-benefit draws in £)`, or `None` when the Gamma cost
/// parameters (`cost_mean`, `cost_sd`) are not strictly positive.
///
/// # Examples
///
/// ```rust
/// use health_economics::probabilistic_sensitivity_analysis::{
///     mean, simulate_migration_net_benefits, MigrationCase,
/// };
///
/// let case = MigrationCase {
///     cost_mean: 800_000.0,
///     cost_sd: 200_000.0,
///     benefit_mean: 350_000.0,
///     benefit_sd: 150_000.0,
///     duration_low: 3.0,
///     duration_high: 6.0,
/// };
///
/// // 10,000 draws: mean net benefit ≈ £775k (4.5 × £350k − £800k).
/// let draws = simulate_migration_net_benefits(&case, 10_000, 42).unwrap();
/// assert!((mean(&draws).unwrap() - 775_000.0).abs() < 30_000.0);
///
/// // Determinism: the same seed reproduces the run exactly.
/// let again = simulate_migration_net_benefits(&case, 10_000, 42).unwrap();
/// assert_eq!(draws, again);
/// ```
pub fn simulate_migration_net_benefits(
    case: &MigrationCase,
    n: usize,
    seed: u64,
) -> Option<Vec<f64>> {
    let mut rng = Lcg::new(seed);
    let mut draws = Vec::with_capacity(n);
    for _ in 0..n {
        // One joint parameter sample θ per draw: all three parameters vary
        // simultaneously (the essence of PSA vs one-way sensitivity).
        let cost = rng.gamma_mean_sd(case.cost_mean, case.cost_sd)?;
        let annual = rng.normal(case.benefit_mean, case.benefit_sd);
        let duration = rng.uniform(case.duration_low, case.duration_high);
        draws.push(duration * annual - cost);
    }
    Some(draws)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worked_example() -> MigrationCase {
        // Doc worked example: "Migration cost ~ Gamma, mean £800k, sd £200k;
        // Annual benefit ~ Normal, mean £350k, sd £150k; Benefit duration ~
        // Uniform, 3–6 years."
        MigrationCase {
            cost_mean: 800_000.0,
            cost_sd: 200_000.0,
            benefit_mean: 350_000.0,
            benefit_sd: 150_000.0,
            duration_low: 3.0,
            duration_high: 6.0,
        }
    }

    fn draws() -> Vec<f64> {
        simulate_migration_net_benefits(&worked_example(), 10_000, 42).unwrap()
    }

    /// Mean net benefit: £775k (analytically 4.5 × £350k − £800k; the
    /// simulation should agree within Monte Carlo error).
    #[test]
    fn mean_net_benefit_is_about_775k() {
        // Worked example: "Mean net benefit: £775k."
        let m = mean(&draws()).unwrap();
        assert!((m - 775_000.0).abs() < 30_000.0);
    }

    /// Probability net benefit > 0: ≈ 0.86.
    #[test]
    fn probability_net_positive_is_about_0_86() {
        // Worked example: "Probability net > 0: 0.86."
        let p = probability_positive(&draws()).unwrap();
        assert!((p - 0.86).abs() < 0.03);
    }

    /// 5th–95th percentile: the doc quotes an illustrative −£180k … +£1.9M —
    /// "a real tail where we lose £180k+". The stated distributions actually
    /// produce a slightly wider band (≈ −£420k … +£2.1M), so the doc's
    /// figures are verified as bounds: at least a 5% chance of losing £180k+,
    /// and an upside tail reaching £1.9M+.
    #[test]
    fn tail_covers_minus_180k_to_plus_1_9m() {
        // Worked example: "5th–95th percentile: −£180k … +£1.9M."
        let d = draws();
        let p5 = percentile(&d, 5.0).unwrap();
        let p95 = percentile(&d, 95.0).unwrap();
        assert!(p5 <= -180_000.0, "5th percentile {p5} should lose £180k+");
        assert!(p95 >= 1_900_000.0, "95th percentile {p95} should reach £1.9M+");
    }

    /// NMB_j(θ) = λ × Effect_j(θ) − Cost_j(θ).
    #[test]
    fn nmb_is_threshold_times_effect_minus_cost() {
        // Doc math: "compute NMB_j(θ) = λ × Effect_j(θ) − Cost_j(θ)."
        let nmb = net_monetary_benefit(30_000.0, 2.0, 24_000.0);
        assert!((nmb - 36_000.0).abs() < 1e-9);
    }

    /// CEAC: the fraction of draws in which each option has the highest NMB.
    #[test]
    fn ceac_reports_fraction_of_draws_each_option_wins() {
        // Doc math: "CEAC_j(λ) = fraction of draws in which option j has the
        // highest NMB at threshold λ."
        let a = [1.0, 5.0, 3.0, 2.0];
        let b = [2.0, 1.0, 4.0, 1.0];
        let c = ceac(&[&a, &b]).unwrap();
        assert!((c[0] - 0.5).abs() < 1e-9);
        assert!((c[1] - 0.5).abs() < 1e-9);
        assert!((c.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    /// The generator is deterministic: the same seed reproduces the run.
    #[test]
    fn same_seed_reproduces_the_simulation() {
        // Module design note: deterministic by design for reproducibility.
        let a = simulate_migration_net_benefits(&worked_example(), 100, 7).unwrap();
        let b = simulate_migration_net_benefits(&worked_example(), 100, 7).unwrap();
        assert_eq!(a, b);
    }

    /// Sampler sanity: gamma-by-mean/sd reproduces its mean and sd, and
    /// uniform stays in range.
    #[test]
    fn samplers_match_their_parameterization() {
        // Verifies the worked example's input distributions are sampled
        // faithfully: Gamma(mean £800k, sd £200k) and Uniform(3, 6).
        let mut rng = Lcg::new(1);
        let n = 20_000;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for _ in 0..n {
            let g = rng.gamma_mean_sd(800_000.0, 200_000.0).unwrap();
            assert!(g > 0.0);
            sum += g;
            sum_sq += g * g;
        }
        let m = sum / n as f64;
        let sd = (sum_sq / n as f64 - m * m).sqrt();
        assert!((m - 800_000.0).abs() < 10_000.0);
        assert!((sd - 200_000.0).abs() < 10_000.0);

        let mut rng = Lcg::new(2);
        for _ in 0..1_000 {
            let u = rng.uniform(3.0, 6.0);
            assert!((3.0..6.0).contains(&u));
        }
    }

    // Edge cases: empty draw sets leave every summary statistic undefined.
    #[test]
    fn empty_draws_yield_none() {
        assert!(mean(&[]).is_none());
        assert!(probability_positive(&[]).is_none());
        assert!(percentile(&[], 50.0).is_none());
        assert!(ceac(&[]).is_none());
    }
}
