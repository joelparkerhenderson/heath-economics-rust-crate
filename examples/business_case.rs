//! Building a Green Book-style business case.
//!
//! Discounts benefit and cost streams to present value, computes NPV and the
//! benefit-cost ratio, applies optimism bias, separates cash-releasing from
//! economic ROI, and sizes the payer's budget impact.
//!
//! Run with: `cargo run --example business_case`

use health_economics::budget_impact_analysis as bia;
use health_economics::cost_benefit_analysis as cba;
use health_economics::return_on_investment as roi;
use health_economics::time_horizon;

fn main() {
    println!("=== Discounted cost-benefit analysis (HM Treasury 3.5%) ===");
    // Year 0: build (£800k). Years 1-5: £150k running cost, £450k benefit.
    let costs: Vec<f64> = vec![800_000.0, 150_000.0, 150_000.0, 150_000.0, 150_000.0, 150_000.0];
    let benefits: Vec<f64> = vec![0.0, 450_000.0, 450_000.0, 450_000.0, 450_000.0, 450_000.0];
    let pv_costs = cba::present_value(&costs, cba::GREEN_BOOK_DISCOUNT_RATE);
    let pv_benefits = cba::present_value(&benefits, cba::GREEN_BOOK_DISCOUNT_RATE);
    let npv = cba::net_present_value(pv_benefits, pv_costs);
    let bcr = cba::benefit_cost_ratio(pv_benefits, pv_costs).unwrap();
    println!("PV(costs):    £{pv_costs:>10.0}");
    println!("PV(benefits): £{pv_benefits:>10.0}");
    println!("NPV:          £{npv:>10.0}   BCR: {bcr:.2}");

    // Green Book optimism bias: uplift costs, haircut benefits, re-test.
    let adj_costs = cba::optimism_bias_cost_uplift(pv_costs, 0.2);
    let adj_benefits = cba::optimism_bias_benefit_haircut(pv_benefits, 0.2);
    let adj_npv = cba::net_present_value(adj_benefits, adj_costs);
    println!("Bias-adjusted (costs +20%, benefits −20%): NPV £{adj_npv:.0}");

    println!("\n=== ROI: cash-releasing vs economic ===");
    // Only the first line releases cash a budget holder can bank.
    let lines = [
        roi::BenefitLine { class: roi::BenefitClass::CashReleasing, amount: 180_000.0 },
        roi::BenefitLine { class: roi::BenefitClass::Capacity, amount: 240_000.0 },
        roi::BenefitLine { class: roi::BenefitClass::Qualitative, amount: 0.0 },
    ];
    let annual_cost = 200_000.0;
    let strict = roi::strict_financial_roi(&lines, annual_cost).unwrap();
    let economic = roi::economic_roi(&lines, annual_cost).unwrap();
    println!("Strict financial ROI (cash only): {:.0}%", strict * 100.0);
    println!("Economic ROI (cash + capacity):   {:.0}%", economic * 100.0);
    let payback = roi::payback_period_years(800_000.0, 300_000.0).unwrap();
    println!("Payback on £800k at £300k/yr net: {payback:.1} years");

    println!("\n=== Time horizon: when does it break even? ===");
    let breakeven = time_horizon::break_even_horizon_years(800_000.0, 450_000.0, 150_000.0).unwrap();
    for years in [1.0, 3.0, 5.0] {
        let net = time_horizon::net_benefit_at_horizon(800_000.0, 450_000.0, 150_000.0, years);
        println!("Net benefit at {years:.0}-year horizon: £{net:>9.0}");
    }
    println!("Break-even horizon: {breakeven:.1} years");

    println!("\n=== Budget impact for the payer ===");
    // Net cost per treated patient: intervention − displaced care + induced care.
    let net_per_patient = bia::net_cost_per_patient(600.0, 450.0, 30.0);
    let current = bia::scenario_cost(&[]); // no spend today
    let with_new = bia::scenario_cost(&[bia::PatientGroup {
        eligible_population: 20_000.0,
        uptake: 0.30,
        net_cost_per_patient: net_per_patient,
    }]);
    println!("Net cost per patient: £{net_per_patient:.0}");
    println!("Year-1 budget impact: £{:.0}", bia::budget_impact(with_new, current));
}
