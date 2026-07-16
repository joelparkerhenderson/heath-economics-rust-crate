//! Unit economics for digital health products: cloud, inference, and the
//! LTV/CAC funnel — plus the health value most dashboards leave out.
//!
//! Run with: `cargo run --example unit_economics`

use health_economics::cloud_unit_economics as cloud;
use health_economics::health_app_unit_economics as app;
use health_economics::inference_unit_economics as infer;
use health_economics::retention_and_churn as retain;

fn main() {
    println!("=== Cloud cost per care episode ===");
    let spend = cloud::CloudSpend {
        compute: 42_000.0,
        data: 18_000.0,
        shared_platform: 12_000.0,
    };
    let episodes = 450_000.0;
    let per_episode = cloud::unit_cost(spend.total(), episodes).unwrap();
    println!("Monthly spend £{:.0} / {episodes} episodes = £{per_episode:.3}/episode", spend.total());
    println!(
        "vs a £42 GP consultation: {:.0}× cheaper per contact",
        cloud::unit_cost_ratio(42.0, per_episode).unwrap()
    );

    println!("\n=== LLM inference cost per triage ===");
    // One triage = a short classification call plus a longer summary call.
    let calls = [
        infer::LlmCall { input_tokens: 1_200.0, output_tokens: 150.0 },
        infer::LlmCall { input_tokens: 3_500.0, output_tokens: 900.0 },
    ];
    let (in_rate, out_rate) = (3.0, 15.0); // $ per million tokens
    let per_triage = infer::cost_per_unit(&calls, in_rate, out_rate);
    println!("Cost per triage: ${per_triage:.4}");
    println!("At 2M triages/year: ${:.0}", infer::annual_cost(per_triage, 2_000_000.0));
    println!(
        "Cost share of a £25 (≈$31) unit of value: {:.2}%",
        infer::cost_share_of_value(per_triage, 31.0).unwrap() * 100.0
    );
    println!(
        "Same workload if prices fall 30%/yr for 3 yrs: ${:.4}/triage",
        infer::projected_cost(per_triage, 0.7, 3.0)
    );

    println!("\n=== The LTV/CAC funnel ===");
    let cac = app::cac(300_000.0, 5_000.0).unwrap();
    let arpu = app::arpu(45_000.0, 5_000.0).unwrap(); // per month
    let ltv = app::ltv(arpu, 0.08).unwrap(); // 8% monthly churn
    println!("CAC £{cac:.0}, ARPU £{arpu:.0}/mo, churn 8%/mo → LTV £{ltv:.0}");
    println!(
        "LTV:CAC = {:.1} (viable ≥ 3): {}",
        app::ltv_cac_ratio(ltv, cac).unwrap(),
        app::is_viable_ltv_cac(ltv, cac)
    );

    println!("\n=== Retention gates every benefit ===");
    // Day-90 retention of 25% means benefits only accrue to a quarter of buys.
    let day90 = retain::retention_percent(1_250.0, 5_000.0).unwrap();
    println!("Day-90 retention: {day90:.0}%");
    println!(
        "Effective CAC per retained user: £{:.0}",
        app::effective_cac_per_retained_user(cac, day90 / 100.0).unwrap()
    );
    // Quarterly retention curve × benefit accrual per quarter:
    let expected = retain::expected_benefit_per_acquired_user(
        &[1.0, 0.40, 0.25, 0.18],
        &[30.0, 30.0, 30.0, 30.0],
    );
    println!("Expected benefit per acquired user over 4 quarters: £{expected:.0}");

    println!("\n=== And the health value most dashboards omit ===");
    let done = retain::completers(5_000.0, 0.25);
    let qalys = retain::qalys_delivered(done, 0.02);
    let value = retain::monetized_health_value(qalys, 20_000.0);
    println!(
        "{done:.0} completers × 0.02 QALYs × £20k = £{value:.0} of health value ({:.0}£/download)",
        retain::health_value_per_download(value, 5_000.0).unwrap()
    );
}
