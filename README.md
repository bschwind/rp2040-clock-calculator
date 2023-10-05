# RP2040 Clock Calculator

Adjust the constants in `src/main.rs`, run with `cargo run`, and receive PLL (Phase-Locked-Loop) parameters to use when configuring the clocks on the RP2040.

Mainly used when selecting a specific PIO frequency for precise timing. This will try to give you exact integer dividers before exploring the fractional ones.
