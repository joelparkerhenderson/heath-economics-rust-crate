# Tutorial 2: Building the financial case

Health value gets an intervention taken seriously; the financial case gets it
funded. This tutorial covers the money side: discounted cost-benefit
analysis, optimism bias, the cash-releasing/economic ROI split, payback and
break-even horizons, and the payer's budget impact.

The running scenario: an £800,000 build, then five years of £150,000/year
running costs against £450,000/year of benefits.

## Step 1 — Discount both streams (HM Treasury: 3.5%)

Money later is worth less than money now. Discount every year's flow back to
present value before comparing:

```rust
use health_economics::cost_benefit_analysis::{
    GREEN_BOOK_DISCOUNT_RATE, present_value, net_present_value, benefit_cost_ratio,
};

let costs    = [800_000.0, 150_000.0, 150_000.0, 150_000.0, 150_000.0, 150_000.0];
let benefits = [0.0, 450_000.0, 450_000.0, 450_000.0, 450_000.0, 450_000.0];

let pv_c = present_value(&costs, GREEN_BOOK_DISCOUNT_RATE);
let pv_b = present_value(&benefits, GREEN_BOOK_DISCOUNT_RATE);
let npv = net_present_value(pv_b, pv_c);
let bcr = benefit_cost_ratio(pv_b, pv_c).unwrap();

assert!(npv > 500_000.0 && npv < 600_000.0);   // ≈ £555k
assert!(bcr > 1.3 && bcr < 1.5);               // ≈ 1.38
```

## Step 2 — Apply optimism bias before anyone else does

Forecasts flatter. The Green Book expects explicit uplifts on costs and
haircuts on benefits — and a case that survives them:

```rust
use health_economics::cost_benefit_analysis::{
    optimism_bias_cost_uplift, optimism_bias_benefit_haircut, net_present_value,
};

let adj_costs = optimism_bias_cost_uplift(1_477_258.0, 0.2);      // +20%
let adj_benefits = optimism_bias_benefit_haircut(2_031_774.0, 0.2); // −20%
let adj_npv = net_present_value(adj_benefits, adj_costs);

// This case does NOT survive a 20/20 adjustment — better to know now.
assert!(adj_npv < 0.0);
```

## Step 3 — Separate cash-releasing from economic ROI

Finance directors bank cash, not "capacity". Label every benefit line and
report both ROIs:

```rust
use health_economics::return_on_investment::{
    BenefitLine, BenefitClass, strict_financial_roi, economic_roi, payback_period_years,
};

let lines = [
    BenefitLine { class: BenefitClass::CashReleasing, amount: 180_000.0 }, // agency shifts stopped
    BenefitLine { class: BenefitClass::Capacity,      amount: 240_000.0 }, // clinician time freed
    BenefitLine { class: BenefitClass::Qualitative,   amount: 0.0 },       // staff satisfaction
];

let strict = strict_financial_roi(&lines, 200_000.0).unwrap();
let economic = economic_roi(&lines, 200_000.0).unwrap();
assert!(strict < 0.0);      // cash alone: −10% — a loss
assert!(economic > 1.0);    // cash + capacity: +110%

// Payback: how long until cumulative net benefit covers the build?
let payback = payback_period_years(800_000.0, 300_000.0).unwrap();
assert!((payback - 2.6666666666666665).abs() < 1e-9);
```

The honest sentence is: "−10% cash-releasing, +110% in economic terms" — and
saying it that way is what keeps the capacity claims credible.

## Step 4 — Pick the horizon deliberately

Benefits arrive over years; the horizon you choose can flip the verdict:

```rust
use health_economics::time_horizon::{
    net_benefit_at_horizon, break_even_horizon_years,
};

let at_1yr = net_benefit_at_horizon(800_000.0, 450_000.0, 150_000.0, 1.0);
let at_5yr = net_benefit_at_horizon(800_000.0, 450_000.0, 150_000.0, 5.0);
assert!(at_1yr < 0.0);
assert!(at_5yr > 0.0);

let breakeven = break_even_horizon_years(800_000.0, 450_000.0, 150_000.0).unwrap();
assert!((breakeven - 2.6666666666666665).abs() < 1e-9);
```

## Step 5 — Budget impact: can the payer afford it at all?

Cost-effective and affordable are different questions. Budget impact is
undiscounted, near-term, and uses real uptake:

```rust
use health_economics::budget_impact_analysis::{
    net_cost_per_patient, scenario_cost, budget_impact, PatientGroup,
};

// £600 intervention, displacing £450 of care, inducing £30 of extra visits.
let net = net_cost_per_patient(600.0, 450.0, 30.0);
assert!((net - 180.0).abs() < 1e-9);

let with_new = scenario_cost(&[PatientGroup {
    eligible_population: 20_000.0,
    uptake: 0.30,
    net_cost_per_patient: net,
}]);
let impact = budget_impact(with_new, 0.0);
assert!((impact - 1_080_000.0).abs() < 1e-6); // £1.08M in year one
```

## Where to go next

- Whose costs count? Declare it first: [`analysis_perspective`](../../analysis_perspective/index.html).
- Full lifetime costs, not sticker price: [`total_cost_of_ownership`](../../total_cost_of_ownership/index.html).
- Which "savings" are real cash: [`cash_releasing_vs_non_cash_releasing`](../../cash_releasing_vs_non_cash_releasing/index.html).
- Did the benefits actually arrive? [`benefits_realization`](../../benefits_realization/index.html).
