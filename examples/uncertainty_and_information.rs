//! Quantifying uncertainty: tornado diagrams, probabilistic sensitivity
//! analysis, and the expected value of perfect information.
//!
//! Run with: `cargo run --example uncertainty_and_information`

use health_economics::expected_value_of_perfect_information as evpi;
use health_economics::probabilistic_sensitivity_analysis as psa;
use health_economics::sensitivity_analysis as sa;

fn main() {
    println!("=== One-way sensitivity: coding-assistant business case ===");
    let base = sa::CodingAssistantCase {
        developers: 200.0,
        hours_saved_per_day: 0.5,
        loaded_cost_per_hour: 60.0,
        working_days_per_year: 220.0,
        license_per_dev_per_month: 40.0,
    };
    println!("Base-case net benefit: £{:.0}/year", base.net_benefit());
    println!(
        "Break-even time saving: {:.1} minutes/day",
        base.threshold_hours_saved_per_day().unwrap() * 60.0
    );

    // Swing each parameter low/high, rank by |swing| — the tornado ordering.
    let mut swings = [
        ("hours saved/day (0.1..1.0)", swing_hours(&base, 0.1, 1.0)),
        ("loaded rate (£40..£80)", swing_rate(&base, 40.0, 80.0)),
        ("developers (100..300)", swing_devs(&base, 100.0, 300.0)),
        ("license (£20..£60/mo)", swing_license(&base, 20.0, 60.0)),
    ];
    sa::rank_by_swing(&mut swings);
    println!("Tornado (widest swing first):");
    for (name, swing) in &swings {
        println!("  {name:<28} swing £{swing:.0}");
    }

    println!("\n=== Probabilistic sensitivity analysis: platform migration ===");
    // Cost ~ Gamma(mean £800k, sd £200k); benefit ~ Normal(£350k, £150k)/yr;
    // duration ~ Uniform(3, 6) years. Deterministic seeded generator.
    let case = psa::MigrationCase {
        cost_mean: 800_000.0,
        cost_sd: 200_000.0,
        benefit_mean: 350_000.0,
        benefit_sd: 150_000.0,
        duration_low: 3.0,
        duration_high: 6.0,
    };
    let draws = psa::simulate_migration_net_benefits(&case, 10_000, 42).unwrap();
    println!("Mean net benefit:    £{:.0}", psa::mean(&draws).unwrap());
    println!("P(net benefit > 0):  {:.0}%", psa::probability_positive(&draws).unwrap() * 100.0);
    println!(
        "5th–95th percentile: £{:.0} … £{:.0}",
        psa::percentile(&draws, 5.0).unwrap(),
        psa::percentile(&draws, 95.0).unwrap()
    );

    println!("\n=== EVPI: what is resolving the uncertainty worth? ===");
    // Two options across three possible worlds (NMB per world).
    let scenarios = [
        evpi::Scenario { probability: 0.5, option_nmbs: vec![900_000.0, 400_000.0] },
        evpi::Scenario { probability: 0.3, option_nmbs: vec![-200_000.0, 300_000.0] },
        evpi::Scenario { probability: 0.2, option_nmbs: vec![100_000.0, 250_000.0] },
    ];
    let best_now = evpi::expected_nmb_of_best_option(&scenarios).unwrap();
    let with_pi = evpi::expected_nmb_with_perfect_information(&scenarios).unwrap();
    let value = evpi::evpi(&scenarios).unwrap();
    println!("E[NMB], committing now:          £{best_now:.0}");
    println!("E[NMB] with perfect information: £{with_pi:.0}");
    println!("EVPI (max worth paying for research/a spike): £{value:.0}");
}

fn swing_hours(base: &sa::CodingAssistantCase, low: f64, high: f64) -> f64 {
    let mut lo = *base;
    lo.hours_saved_per_day = low;
    let mut hi = *base;
    hi.hours_saved_per_day = high;
    hi.net_benefit() - lo.net_benefit()
}

fn swing_rate(base: &sa::CodingAssistantCase, low: f64, high: f64) -> f64 {
    let mut lo = *base;
    lo.loaded_cost_per_hour = low;
    let mut hi = *base;
    hi.loaded_cost_per_hour = high;
    hi.net_benefit() - lo.net_benefit()
}

fn swing_devs(base: &sa::CodingAssistantCase, low: f64, high: f64) -> f64 {
    let mut lo = *base;
    lo.developers = low;
    let mut hi = *base;
    hi.developers = high;
    hi.net_benefit() - lo.net_benefit()
}

fn swing_license(base: &sa::CodingAssistantCase, low: f64, high: f64) -> f64 {
    let mut lo = *base;
    lo.license_per_dev_per_month = low;
    let mut hi = *base;
    hi.license_per_dev_per_month = high;
    hi.net_benefit() - lo.net_benefit()
}
