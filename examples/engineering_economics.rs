//! The software engineering mirror: delivery metrics as economics.
//!
//! Cost of delay drives sequencing (CD3/WSJF), DORA metrics monetize faster
//! and safer delivery, Little's Law connects flow to capacity, and technical
//! debt behaves like principal plus interest.
//!
//! Run with: `cargo run --example engineering_economics`

use health_economics::cost_of_delay as cod;
use health_economics::dora_metrics as dora;
use health_economics::flow_metrics as flow;
use health_economics::technical_debt as debt;
use health_economics::wsjf_and_cd3 as wsjf;

fn main() {
    println!("=== Cost of delay ===");
    // A discharge-automation feature saves £250/patient for 60 patients/week.
    let cod_per_week = cod::operational_cost_of_delay(250.0, 60.0);
    println!("CoD: £{cod_per_week:.0}/week");
    println!("A 4-week slip costs: £{:.0}", cod::total_delay_loss(cod_per_week, 4.0));

    println!("\n=== Sequencing by CD3 (cost of delay / duration) ===");
    let features = [
        wsjf::Feature { cost_of_delay_per_week: 9_000.0, duration_weeks: 3.0 },
        wsjf::Feature { cost_of_delay_per_week: 12_000.0, duration_weeks: 2.0 },
        wsjf::Feature { cost_of_delay_per_week: 15_000.0, duration_weeks: 3.0 },
    ];
    for f in &features {
        println!(
            "  CoD £{:>6}/wk over {} wks → CD3 {:.0}",
            f.cost_of_delay_per_week,
            f.duration_weeks,
            f.cd3().unwrap()
        );
    }
    println!("Delay cost in given order: £{:.0}", wsjf::total_delay_cost(&features));
    let ordered = wsjf::sequence_by_cd3(&features);
    println!("Delay cost in CD3 order:   £{:.0}", wsjf::total_delay_cost(&ordered));
    println!("Saved by re-sequencing alone: £{:.0}", wsjf::sequencing_savings(&features));

    println!("\n=== DORA: monetizing faster, safer delivery ===");
    let freq = dora::deployment_frequency(120.0, 30.0).unwrap();
    println!("Deployment frequency: {freq:.1}/day");
    let weeks_faster = dora::lead_time_reduction_weeks(6.0, 4.0 / 7.0);
    println!("Lead time cut 6 wks → 4 days: {weeks_faster:.1} weeks earlier per change");
    // 12 monetizable improvements/year, each worth £10k/week once live.
    let pulled_forward = dora::value_pulled_forward(12.0, weeks_faster, 10_000.0);
    println!("Value pulled forward: £{pulled_forward:.0}/year");
    let avoided = dora::failed_changes_avoided(250.0, 0.15, 0.05);
    println!(
        "Failures avoided (CFR 15%→5% on 250 changes): {avoided:.0}, worth £{:.0}",
        dora::failure_cost_avoided(250.0, 0.15, 0.05, 15_000.0)
    );

    println!("\n=== Flow: Little's Law both ways ===");
    // 8 items/week throughput with 24 items in progress:
    let ct = flow::littles_law_cycle_time(24.0, 8.0).unwrap();
    println!("Cycle time = WIP/throughput = {ct:.1} weeks");
    println!(
        "Flow efficiency (3 active days of a 15-day lead time): {:.0}%",
        flow::flow_efficiency_percent(3.0, 12.0).unwrap()
    );

    println!("\n=== Technical debt: principal and interest ===");
    let principal = debt::sqale_principal(3_800.0, 75.0);
    let tdr = debt::technical_debt_ratio_percent(principal, 6_000_000.0).unwrap();
    println!("Principal (3,800 h × £75): £{principal:.0}");
    println!("Debt ratio vs £6M redevelopment: {tdr:.1}% → grade {:?}", debt::sqale_grade(tdr));
    // 20,000 dev hours/year absorbing a 25% velocity drag at £75/h, plus 12
    // extra failures/year at £8k each.
    let interest = debt::annual_interest(20_000.0, 0.25, 75.0, 12.0, 8_000.0);
    println!("Annual interest (velocity drag + failures): £{interest:.0}");
    let avoided_interest = debt::interest_avoided_per_year(interest, 0.6);
    let payback = debt::payback_period_years(85_500.0, avoided_interest).unwrap();
    println!("Paying down the worst 30% (£85.5k) pays back in {payback:.1} years");
}
