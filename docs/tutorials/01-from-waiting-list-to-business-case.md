# Tutorial 1: From a waiting list to a business case

This tutorial walks the canonical health-economics chain end to end: quantify
health gain in QALYs, monetize it at a willingness-to-pay threshold, compare
it with what the intervention costs (ICER), and turn the comparison into a
decision (net monetary benefit).

The running scenario: your team ships software that accelerates a cardiac
pathway, removing a 6-month wait for 400 patients a year. While waiting,
patients live at utility 0.6; treated, at 0.85.

## Step 1 — Count the QALYs

A QALY is a year of life weighted by its quality (1 = perfect health,
0 = dead). Waiting in a worse health state is a QALY loss you can recover:

```rust
use health_economics::quality_adjusted_life_year::{
    HealthState, qaly_loss_from_delay, qalys, population_qalys,
};

// Treated now: a full year at utility 0.85.
let treated_now = qalys(&[HealthState { duration_years: 1.0, utility: 0.85 }]);

// Treated after a 6-month delay: half a year waiting, half treated.
let treated_late = qalys(&[
    HealthState { duration_years: 0.5, utility: 0.60 },
    HealthState { duration_years: 0.5, utility: 0.85 },
]);
assert!((treated_now - 0.85).abs() < 1e-9);
assert!((treated_late - 0.725).abs() < 1e-9);

// The same number, computed directly: delay × (utility gap).
let per_patient = qaly_loss_from_delay(0.5, 0.60, 0.85);
assert!((per_patient - 0.125).abs() < 1e-9);

// 400 patients/year → 50 QALYs/year.
let total = population_qalys(per_patient, 400.0);
assert!((total - 50.0).abs() < 1e-9);
```

## Step 2 — Monetize at the threshold

NICE values a QALY at £20,000–£30,000. That converts health gain into the
same currency as your licence fee:

```rust
use health_economics::quality_adjusted_life_year::monetized_value;

let low  = monetized_value(50.0, 20_000.0);
let high = monetized_value(50.0, 30_000.0);
assert!((low - 1_000_000.0).abs() < 1e-6);
assert!((high - 1_500_000.0).abs() < 1e-6);
```

Fifty QALYs a year is £1.0–1.5M of health value — before counting any
operational savings.

## Step 3 — The ICER: what does a QALY cost through your product?

The incremental cost-effectiveness ratio divides the extra cost by the extra
health. Suppose the software costs £500,000/year to run:

```rust
use health_economics::incremental_cost_effectiveness_ratio::{
    icer, adopt_at_threshold,
};

let ratio = icer(500_000.0, 50.0).unwrap();
assert!((ratio - 10_000.0).abs() < 1e-9);

// £10,000/QALY is below the £20,000 threshold: fund it.
assert!(adopt_at_threshold(ratio, 20_000.0));
```

## Step 4 — Net monetary benefit: ratio-free decisions

Ratios misbehave near zero and can't be summed across options. NMB rescales
everything to money — `NMB = λ × ΔQALYs − ΔCost` — so bigger is simply
better:

```rust
use health_economics::net_monetary_benefit::{net_monetary_benefit, adopt};

let nmb = net_monetary_benefit(50.0, 500_000.0, 20_000.0);
assert!((nmb - 500_000.0).abs() < 1e-9);
assert!(adopt(nmb));
```

## Step 5 — What's the most you could charge?

Flip the logic: at the payer's threshold, the health gain plus any cost
offsets cap the defensible price:

```rust
use health_economics::willingness_to_pay_thresholds::max_defensible_price;

// 50 QALYs at £20k, no operational offsets: £1M/year price ceiling.
let cap = max_defensible_price(20_000.0, 50.0, 0.0);
assert!((cap - 1_000_000.0).abs() < 1e-6);
```

## Where to go next

- Discount multi-year QALY and cost streams first: [`discounting_and_time_preference`](../../discounting_and_time_preference/index.html).
- Comparing more than two options? Build the frontier: [`dominance_and_efficiency_frontier`](../../dominance_and_efficiency_frontier/index.html).
- Utilities come from validated instruments, not intuition: [`eq_5d`](../../eq_5d/index.html).
- Severity can shift the effective threshold: [`qaly_shortfall_and_severity_modifiers`](../../qaly_shortfall_and_severity_modifiers/index.html).
