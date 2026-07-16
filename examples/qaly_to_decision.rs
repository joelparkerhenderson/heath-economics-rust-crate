//! From a pathway delay to an adoption decision.
//!
//! Walks the canonical health-economics chain: quantify the health lost to
//! waiting (QALYs), monetize it at a willingness-to-pay threshold, compare
//! against the intervention's cost (ICER), and make the call (net monetary
//! benefit), ending with the maximum defensible price.
//!
//! Run with: `cargo run --example qaly_to_decision`

use health_economics::incremental_cost_effectiveness_ratio::{
    self as icer_mod, classify_quadrant,
};
use health_economics::net_monetary_benefit as nmb;
use health_economics::quality_adjusted_life_year as qaly;
use health_economics::willingness_to_pay_thresholds as wtp;

fn main() {
    println!("=== Step 1: Health lost to waiting ===");
    // Patients wait for cardiac treatment at utility 0.6; treatment restores
    // utility 0.85. Software that accelerates the pathway removes a 6-month
    // (0.5-year) delay for 400 patients per year.
    let gain_per_patient = qaly::qaly_loss_from_delay(0.5, 0.6, 0.85);
    let patients = 400.0;
    let total_qalys = qaly::population_qalys(gain_per_patient, patients);
    println!("QALY gain per patient:      {gain_per_patient:.3}");
    println!("QALY gain across {patients} pts: {total_qalys:.1}");

    // The same gain expressed as explicit health-state streams.
    let treated_now = qaly::qalys(&[qaly::HealthState { duration_years: 1.0, utility: 0.85 }]);
    let treated_late = qaly::qalys(&[
        qaly::HealthState { duration_years: 0.5, utility: 0.6 },
        qaly::HealthState { duration_years: 0.5, utility: 0.85 },
    ]);
    println!("Treated now vs late:        {treated_now:.3} vs {treated_late:.3} QALYs");

    println!("\n=== Step 2: Monetize at the threshold ===");
    for threshold in [20_000.0, 30_000.0] {
        let value = qaly::monetized_value(total_qalys, threshold);
        println!("At £{threshold:>6}/QALY: £{value:>9.0}/year of health value");
    }

    println!("\n=== Step 3: ICER against the software's cost ===");
    let delta_cost = 500_000.0; // annual running cost of the pathway software
    let quadrant = classify_quadrant(delta_cost, total_qalys);
    let ratio = icer_mod::icer(delta_cost, total_qalys).expect("non-zero QALY gain");
    println!("Quadrant: {quadrant:?}");
    println!("ICER: £{ratio:.0} per QALY (threshold £20,000–£30,000)");
    println!("Adopt at £20k? {}", icer_mod::adopt_at_threshold(ratio, 20_000.0));

    println!("\n=== Step 4: Net monetary benefit ===");
    let benefit = nmb::net_monetary_benefit(total_qalys, delta_cost, 20_000.0);
    println!("NMB at £20k/QALY: £{benefit:.0} — adopt: {}", nmb::adopt(benefit));

    println!("\n=== Step 5: Maximum defensible price ===");
    // With no operational cost offsets, price is capped by health value alone.
    let cap = wtp::max_defensible_price(20_000.0, total_qalys, 0.0);
    println!("Max price at £20k/QALY, no offsets: £{cap:.0}/year");
}
