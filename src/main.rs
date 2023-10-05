use fugit::{HertzU64, RateExtU64};

const CRYSTAL_FREQUENCY_KHZ: u64 = 12_000; // 12 MHz

const MIN_SYSTEM_CLOCK_HZ: u64 = 15_428_572;
const MAX_SYSTEM_CLOCK_HZ: u64 = 133_000_000;

// Voltage-Controlled Oscillator constants.
const PICO_PLL_VCO_MIN_FREQ_KHZ: u64 = 750_000; // 750 MHz
const PICO_PLL_VCO_MAX_FREQ_KHZ: u64 = 1_600_000; // 1600 MHz

const FEEDBACK_DIVIDER_MIN: u64 = 16;
const FEEDBACK_DIVIDER_MAX: u64 = 320;

const POST_DIVIDER_MIN: u64 = 1;
const POST_DIVIDER_MAX: u64 = 7;

const MAX_PIO_INT_DIVIDER: u64 = 65535;

fn main() {
    println!("Using a {} crystal", CRYSTAL_FREQUENCY_KHZ.kHz::<1, 1>());

    let desired_pio_frequency_hz = 6_144_000u64;

    let nanosecond = 1_000_000_000;

    let nanoseconds_per_cycle = nanosecond / desired_pio_frequency_hz;
    println!("~{nanoseconds_per_cycle} nanoseconds per cycle");

    println!("Trying an integer divider first");

    // First try to find a whole integer divider that works for the desired PIO frequency.
    // Start from the max divider and work down so we end up with a clock with less jitter.
    for integer_multiplier in (1..=MAX_PIO_INT_DIVIDER).rev() {
        let possible_system_clock_freq_hz = desired_pio_frequency_hz * integer_multiplier;

        let Some((vco_freq, post_div1, post_div_2)) = check_sys_clock(possible_system_clock_freq_hz.Hz()) else {
            // println!("\tNo exact divider found");
            continue;
        };

        println!(
            "Possible system clock: {} Hz",
            possible_system_clock_freq_hz
        );

        println!("\tVCO Freq = {}kHz, Post Divider 1 = {}, Post Divider 2 = {}, PIO divider = {}, PIO divider fraction = 0", vco_freq.to_kHz(), post_div1, post_div_2, integer_multiplier);
    }

    println!();
    println!("Now trying with fractional dividers");

    for integer_multiplier in (1..=MAX_PIO_INT_DIVIDER).rev() {
        for fraction in (0..256).rev() {
            let numerator = 256 * integer_multiplier + fraction;
            let possible_system_clock_freq_hz = (desired_pio_frequency_hz * numerator) / 256;

            let Some((vco_freq, post_div1, post_div_2)) = check_sys_clock(possible_system_clock_freq_hz.Hz()) else {
                // println!("\tNo exact divider found");
                continue;
            };

            println!(
                "Possible system clock: {} Hz",
                possible_system_clock_freq_hz
            );

            println!("\tVCO Freq = {}kHz, Post Divider 1 = {}, Post Divider 2 = {}, PIO divider = {}, PIO divider fraction = {}", vco_freq.to_kHz(), post_div1, post_div_2, integer_multiplier, fraction);
        }
    }
}

pub fn check_sys_clock(desired_frequency: HertzU64) -> Option<(HertzU64, u8, u8)> {
    if desired_frequency.to_Hz() < MIN_SYSTEM_CLOCK_HZ
        || desired_frequency.to_Hz() > MAX_SYSTEM_CLOCK_HZ
    {
        return None;
    }

    let freq_khz = desired_frequency.to_kHz();
    let reference_divider = 1;
    let reference_freq_khz: u64 = CRYSTAL_FREQUENCY_KHZ / reference_divider;

    for feedback_divide in (FEEDBACK_DIVIDER_MIN..=FEEDBACK_DIVIDER_MAX).rev() {
        let vco_khz = feedback_divide * reference_freq_khz;

        if !(PICO_PLL_VCO_MIN_FREQ_KHZ..=PICO_PLL_VCO_MAX_FREQ_KHZ).contains(&vco_khz) {
            continue;
        }

        for post_div1 in (POST_DIVIDER_MIN..=POST_DIVIDER_MAX).rev() {
            for post_div_2 in (1..=post_div1).rev() {
                let out = vco_khz / (post_div1 * post_div_2);

                if out == freq_khz && (vco_khz % (post_div1 * post_div_2)) == 0 {
                    return Some((
                        vco_khz.kHz(),
                        post_div1.try_into().unwrap(),
                        post_div_2.try_into().unwrap(),
                    ));
                }
            }
        }
    }

    None
}
