# Tutorial 3: Quantifying uncertainty

Every business case is built on guesses. This tutorial shows the three
standard moves for handling them honestly: one-way sensitivity analysis (the
tornado diagram), probabilistic sensitivity analysis (Monte Carlo over all
parameters at once), and the expected value of perfect information (what
resolving the uncertainty is worth in money).

## Step 1 — One-way sensitivity: find the load-bearing assumptions

Vary one parameter at a time across its plausible range and watch the
result swing. The running case: a coding assistant for 200 developers.

```rust
use health_economics::sensitivity_analysis::{
    CodingAssistantCase, rank_by_swing,
};

let base = CodingAssistantCase {
    developers: 200.0,
    hours_saved_per_day: 0.5,
    loaded_cost_per_hour: 60.0,
    working_days_per_year: 220.0,
    license_per_dev_per_month: 40.0,
};
assert!((base.net_benefit() - 1_224_000.0).abs() < 1e-6);

// The break-even question is often the most persuasive single number:
// how little time must the tool save to pay for itself?
let threshold_minutes = base.threshold_hours_saved_per_day().unwrap() * 60.0;
assert!(threshold_minutes < 3.0); // ≈ 2.2 minutes/day

// Tornado: rank parameters by how far they swing the answer.
let mut swings = [("hours_saved", 2_376_000.0), ("developers", 1_224_000.0), ("license", 96_000.0)];
rank_by_swing(&mut swings);
assert_eq!(swings[0].0, "hours_saved"); // widest bar on top
```

If the decision only flips when a parameter leaves any defensible range, the
case is robust. If it flips inside the range, that parameter is what the
meeting should be about.

## Step 2 — Probabilistic sensitivity analysis: all guesses at once

One-way analysis understates risk because errors compound. PSA assigns each
parameter a distribution and simulates thousands of scenarios. The module's
generator is deterministic (seeded), so results are reproducible:

```rust
use health_economics::probabilistic_sensitivity_analysis::{
    MigrationCase, simulate_migration_net_benefits, mean, probability_positive, percentile,
};

// A platform migration: cost ~ Gamma(mean £800k, sd £200k),
// benefit ~ Normal(£350k, £150k)/year, duration ~ Uniform(3, 6) years.
let case = MigrationCase {
    cost_mean: 800_000.0, cost_sd: 200_000.0,
    benefit_mean: 350_000.0, benefit_sd: 150_000.0,
    duration_low: 3.0, duration_high: 6.0,
};
let draws = simulate_migration_net_benefits(&case, 10_000, 42).unwrap();

let m = mean(&draws).unwrap();
let p = probability_positive(&draws).unwrap();
assert!((m - 775_000.0).abs() < 30_000.0);  // expected net benefit ≈ £775k
assert!(p > 0.80 && p < 0.90);              // ≈ 86% chance of net benefit

// The tail is the point: a real chance of a six-figure loss.
assert!(percentile(&draws, 5.0).unwrap() < -180_000.0);
```

"£775k expected, 86% chance of positive, 5% chance of losing £400k+" is a
far more honest sentence than "saves £775k".

## Step 3 — CEAC: how often does each option win?

With NMB draws for competing options, the cost-effectiveness acceptability
curve reports the fraction of simulations each option wins:

```rust
use health_economics::probabilistic_sensitivity_analysis::ceac;

let option_a = [120_000.0, -40_000.0, 310_000.0, 75_000.0];
let option_b = [90_000.0, 10_000.0, 150_000.0, 60_000.0];
let curve = ceac(&[&option_a, &option_b]).unwrap();
assert!((curve[0] - 0.75).abs() < 1e-9); // A wins 3 of 4 worlds
assert!((curve[1] - 0.25).abs() < 1e-9);
```

## Step 4 — EVPI: price the uncertainty itself

If you could know the true state of the world before committing, how much
better would your decisions be on average? That gap is the expected value of
perfect information — the rational ceiling on spending for research, pilots,
and spikes:

```rust
use health_economics::expected_value_of_perfect_information::{
    Scenario, expected_nmb_of_best_option, expected_nmb_with_perfect_information, evpi,
};

let scenarios = [
    Scenario { probability: 0.5, option_nmbs: vec![900_000.0, 400_000.0] },
    Scenario { probability: 0.3, option_nmbs: vec![-200_000.0, 300_000.0] },
    Scenario { probability: 0.2, option_nmbs: vec![100_000.0, 250_000.0] },
];

// Committing now: pick the option with the best *expected* NMB.
let commit_now = expected_nmb_of_best_option(&scenarios).unwrap();
// With perfect information you'd pick the winner in *each* world.
let with_pi = expected_nmb_with_perfect_information(&scenarios).unwrap();
let value_of_information = evpi(&scenarios).unwrap();

assert!((commit_now - 410_000.0).abs() < 1e-6);
assert!((with_pi - 590_000.0).abs() < 1e-6);
assert!((value_of_information - 180_000.0).abs() < 1e-6);
```

A two-week £30k spike that resolves £180k of decision risk is a bargain —
and now you can prove it.

## Where to go next

- The full PSA-to-decision pipeline: [`probabilistic_sensitivity_analysis`](../../probabilistic_sensitivity_analysis/index.html).
- Deterministic sensitivity patterns: [`sensitivity_analysis`](../../sensitivity_analysis/index.html).
- Time horizons interact with all of this: [`time_horizon`](../../time_horizon/index.html).
