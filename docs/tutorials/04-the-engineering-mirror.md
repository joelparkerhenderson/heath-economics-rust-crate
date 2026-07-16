# Tutorial 4: The engineering mirror

Health economics and software delivery keep reinventing each other's math.
Queue time is disutility; deployment lead time is a referral-to-treatment
pathway; technical debt is principal plus interest. This tutorial shows the
software-native modules and their health-economics counterparts.

## Step 1 — Cost of delay: the exchange rate between time and money

Everything starts here. If a feature is worth £15,000/week once live, every
week of delay burns £15,000 — visibly:

```rust
use health_economics::cost_of_delay::{
    operational_cost_of_delay, total_delay_loss,
};

// Discharge automation saves £250/patient across 60 patients/week.
let cod = operational_cost_of_delay(250.0, 60.0);
assert!((cod - 15_000.0).abs() < 1e-9);
assert!((total_delay_loss(cod, 4.0) - 60_000.0).abs() < 1e-9);
```

## Step 2 — CD3: sequence by cost of delay ÷ duration

With a fixed team, the order of work changes total delay cost even when the
work itself doesn't change. Highest CD3 first is optimal:

```rust
use health_economics::wsjf_and_cd3::{
    Feature, total_delay_cost, sequence_by_cd3, sequencing_savings,
};

let features = [
    Feature { cost_of_delay_per_week: 9_000.0,  duration_weeks: 3.0 }, // CD3 3,000
    Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 }, // CD3 6,000
    Feature { cost_of_delay_per_week: 15_000.0, duration_weeks: 3.0 }, // CD3 5,000
];

let as_given = total_delay_cost(&features);
let optimal = total_delay_cost(&sequence_by_cd3(&features));
assert!(optimal <= as_given);
assert!((sequencing_savings(&features) - (as_given - optimal)).abs() < 1e-9);
```

No extra engineers, no scope cuts — money recovered purely by ordering.
This is the same math health systems use to triage waiting lists.

## Step 3 — DORA: monetize faster and safer delivery

Lead-time cuts pull value forward; change-failure-rate cuts avoid incident
costs. Both convert directly:

```rust
use health_economics::dora_metrics::{
    lead_time_reduction_weeks, value_pulled_forward,
    failed_changes_avoided, failure_cost_avoided,
};

// Lead time falls from 6 weeks to 4 days (4/7 of a week).
let weeks_earlier = lead_time_reduction_weeks(6.0, 4.0 / 7.0);

// 12 monetizable improvements/year, each worth £10k/week once live.
let pulled = value_pulled_forward(12.0, weeks_earlier, 10_000.0);
assert!((pulled - 651_428.5714285715).abs() < 1e-6); // ≈ £650k/year

// CFR 15% → 5% across 250 changes/year at £15k per failure.
assert!((failed_changes_avoided(250.0, 0.15, 0.05) - 25.0).abs() < 1e-9);
assert!((failure_cost_avoided(250.0, 0.15, 0.05, 15_000.0) - 375_000.0).abs() < 1e-9);
```

## Step 4 — Flow: Little's Law works on wards and boards alike

`WIP = throughput × cycle time` governs hospital beds and kanban columns
identically:

```rust
use health_economics::flow_metrics::{
    littles_law_cycle_time, littles_law_wip, flow_efficiency_percent,
};

// 24 items in progress at 8 items/week → 3-week cycle time.
assert!((littles_law_cycle_time(24.0, 8.0).unwrap() - 3.0).abs() < 1e-9);

// A ward: 30 admissions/day × 8-day stay = 240 beds occupied.
assert!((littles_law_wip(30.0, 8.0) - 240.0).abs() < 1e-9);

// 3 active days inside a 15-day lead time: 20% flow efficiency —
// the queue, not the work, is the problem.
assert!((flow_efficiency_percent(3.0, 12.0).unwrap() - 20.0).abs() < 1e-9);
```

## Step 5 — Technical debt: principal, interest, and payback

Debt language becomes decision-grade when you split it into principal (cost
to fix) and interest (ongoing drag), then treat paydown like any investment:

```rust
use health_economics::technical_debt::{
    sqale_principal, technical_debt_ratio_percent, annual_interest,
    interest_avoided_per_year, payback_period_years,
};

// 3,800 remediation hours at £75/h.
let principal = sqale_principal(3_800.0, 75.0);
assert!((principal - 285_000.0).abs() < 1e-9);

// Ratio vs £6M redevelopment cost: ~4.75%.
let tdr = technical_debt_ratio_percent(principal, 6_000_000.0).unwrap();
assert!(tdr > 4.0 && tdr < 5.5);

// Interest: 20,000 dev-hours/yr × 25% drag × £75/h + 12 failures × £8k.
let interest = annual_interest(20_000.0, 0.25, 75.0, 12.0, 8_000.0);
assert!((interest - 471_000.0).abs() < 1e-9);

// Fix the worst 30% (£85.5k) to remove 60% of the interest:
let avoided = interest_avoided_per_year(interest, 0.6);
let payback = payback_period_years(85_500.0, avoided).unwrap();
assert!(payback < 0.5); // pays back in under six months
```

## The rosetta stone

| Software concept | Health-economics counterpart | Module |
|---|---|---|
| Cost of delay | Waiting-time disutility | `cost_of_delay`, `referral_to_treatment` |
| WSJF / CD3 sequencing | Waiting-list triage | `wsjf_and_cd3`, `waiting_list_impact` |
| Lead time | Referral-to-treatment time | `dora_metrics`, `referral_to_treatment` |
| WIP via Little's Law | Bed occupancy | `flow_metrics`, `length_of_stay` |
| Change failure rate | Readmission rate | `dora_metrics`, `readmission_rate` |
| Tech-debt interest | Preventable deterioration | `technical_debt`, `prevention_economics` |
| DevEx survey weights | EQ-5D utility weights | `space_and_devex`, `eq_5d` |

## Where to go next

- Developer time has three valuation levels, like clinician time: [`practitioner_time`](../../practitioner_time/index.html).
- Composite wellbeing metrics done properly: [`space_and_devex`](../../space_and_devex/index.html).
- Retaining engineers is workforce economics: [`workforce_retention`](../../workforce_retention/index.html).
