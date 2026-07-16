//! Screening programmes and diagnostic AI: why prevalence rules everything.
//!
//! Sensitivity and specificity belong to the test; positive predictive value
//! belongs to the population. The same classifier that looks brilliant in a
//! balanced benchmark can drown a low-prevalence clinic in false positives.
//!
//! Run with: `cargo run --example screening_and_diagnostics`

use health_economics::clinical_ai_evaluation as ai;
use health_economics::number_needed_to_treat as nnt;
use health_economics::screening_economics as screen;

fn main() {
    println!("=== The prevalence cliff ===");
    let (sens, spec) = (0.90, 0.95);
    for prevalence in [0.20, 0.05, 0.005] {
        let ppv = ai::ppv_from_rates(sens, spec, prevalence).unwrap();
        println!(
            "sens 90%, spec 95%, prevalence {:>5.1}% → PPV {:>5.1}%",
            prevalence * 100.0,
            ppv * 100.0
        );
    }
    println!(
        "Flag rate at 0.5% prevalence: {:.1}% of everyone tested",
        ai::positive_rate(sens, spec, 0.005) * 100.0
    );

    println!("\n=== Ranking quality: AUROC from raw scores ===");
    // Exact pairwise P(score_positive > score_negative), ties counted half.
    let positives = [0.91, 0.85, 0.78, 0.60];
    let negatives = [0.70, 0.45, 0.30, 0.20, 0.10];
    println!("AUROC: {:.3}", ai::auroc(&positives, &negatives).unwrap());

    println!("\n=== Screening programme economics ===");
    let population = 100_000.0;
    let prevalence = 0.005;
    let tp = screen::true_positives(population, prevalence, sens);
    let fp = screen::false_positives(population, prevalence, spec);
    let cost = screen::total_programme_cost(population, 15.0, 400.0, tp, fp);
    println!("True positives:  {tp:.0}");
    println!("False positives: {fp:.0}");
    println!("Programme cost:  £{cost:.0}");
    println!(
        "Cost per true case found: £{:.0}",
        screen::cost_per_true_case(cost, tp).unwrap()
    );
    println!(
        "Net value per case (earlier treatment £12k − overdiagnosis £2k): £{:.0}",
        screen::net_value_per_case_found(12_000.0, 2_000.0)
    );

    println!("\n=== Number needed to treat ===");
    // Event rate falls from 8% (control) to 5% (with the intervention).
    let arr = nnt::absolute_risk_reduction(0.08, 0.05);
    let needed = nnt::number_needed_to_treat(arr).unwrap();
    println!("ARR: {:.1} percentage points → NNT {needed:.0}", arr * 100.0);
    let cost_per_prevented = nnt::cost_per_event_prevented(needed, 150.0);
    println!("At £150/course: £{cost_per_prevented:.0} per event prevented");
    println!(
        "Payoff ratio vs a £20,000 event: {:.1}:1",
        nnt::prevention_payoff_ratio(20_000.0, cost_per_prevented).unwrap()
    );
}
