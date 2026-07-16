//! Demo: chains a few of the library's metrics end-to-end using the QALY
//! topic's worked example — a 6-month pathway delay, monetized and taken
//! through an adoption decision.

use health_economics::incremental_cost_effectiveness_ratio as icer;
use health_economics::net_monetary_benefit as nmb;
use health_economics::quality_adjusted_life_year as qaly;

fn main() {
    // A patient waits at utility 0.6; treatment restores utility 0.85.
    // Software that removes a 6-month delay recovers this QALY loss.
    let qaly_gain_per_patient = qaly::qaly_loss_from_delay(0.5, 0.6, 0.85);
    let patients_per_year = 400.0;
    let population_qalys = qaly::population_qalys(qaly_gain_per_patient, patients_per_year);

    println!("QALY gain per patient from removing a 6-month delay: {qaly_gain_per_patient:.3}");
    println!("Population QALYs across {patients_per_year} patients/year: {population_qalys:.1}");
    for threshold in [20_000.0, 30_000.0] {
        let value = qaly::monetized_value(population_qalys, threshold);
        println!("  Health value at £{threshold}/QALY: £{value:.0}/year");
    }

    // Suppose the pathway software costs £500,000/year to run.
    let delta_cost = 500_000.0;
    let ratio = icer::icer(delta_cost, population_qalys)
        .expect("population QALY gain is non-zero");
    let net_benefit = nmb::net_monetary_benefit(population_qalys, delta_cost, 20_000.0);
    println!("ICER at £{delta_cost}/year: £{ratio:.0} per QALY");
    println!(
        "Net monetary benefit at £20,000/QALY: £{net_benefit:.0} — adopt: {}",
        nmb::adopt(net_benefit)
    );
}
