# Tutorials

Four guided walks through the library, in reading order:

1. **From a waiting list to a business case** — the canonical chain: QALYs →
   willingness-to-pay threshold → ICER → net monetary benefit → price.
2. **Building the financial case** — discounted cost-benefit analysis,
   optimism bias, cash-releasing vs economic ROI, break-even horizons, and
   budget impact.
3. **Quantifying uncertainty** — tornado diagrams, probabilistic sensitivity
   analysis, acceptability curves, and the expected value of perfect
   information.
4. **The engineering mirror** — cost of delay, CD3/WSJF sequencing, DORA,
   Little's Law, and technical debt as principal-plus-interest.

Every code block is a doctest: it compiles and its assertions pass under
`cargo test --doc`. The `examples/` directory contains the same material as
runnable programs (`cargo run --example qaly_to_decision`).
