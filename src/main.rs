use fugit::{HertzU32, RateExtU32};

const CRYSTAL_FREQUENCY_KHZ: u32 = 12_000; // 12 MHz

// Voltage-Controlled Oscillator constants.
const PICO_PLL_VCO_MIN_FREQ_KHZ: u32 = 750_000; // 750 MHz
const PICO_PLL_VCO_MAX_FREQ_KHZ: u32 = 1_600_000; // 1600 MHz

const FEEDBACK_DIVIDER_MIN: u32 = 16;
const FEEDBACK_DIVIDER_MAX: u32 = 320;

const POST_DIVIDER_MIN: u32 = 1;
const POST_DIVIDER_MAX: u32 = 7;

fn main() {
    println!("Using a {} crystal", CRYSTAL_FREQUENCY_KHZ.kHz::<1, 1>());

    let desired_pio_frequency = 3_072_000;

    let max_int_divider = 65535u32;

    for multiplier_increment in 1..=(max_int_divider * 256) {
        let integer_divider = multiplier_increment / max_int_divider;
        let frac_divider = multiplier_increment % max_int_divider;

        // dbg!(integer_divider, frac_divider);

        let desired_frequency = desired_pio_frequency * multiplier_increment;

        if desired_frequency % 256 != 0 {
            continue;
        }

        let desired_frequency_khz = desired_frequency.Hz::<1, 1>().to_kHz();

        let Some((vco_freq, post_div1, post_div_2)) = check_sys_clock(desired_frequency_khz.kHz()) else {
            // println!("\tNo exact divider found");
            continue;
        };

        println!("Desired frequency: {}MHz", desired_frequency_khz);

        println!("\tVCO Freq = {}kHz, Post Divider 1 = {}, Post Divider 2 = {}", vco_freq.to_kHz(), post_div1, post_div_2);
    }

    // for desired_frequency_khz in 122_880_000..=122_880_001 {
    //     println!("Desired frequency: {}kHz", desired_frequency_khz);
    //     let Some((vco_freq, post_div1, post_div_2)) = check_sys_clock(desired_frequency_khz.kHz()) else {
    //         println!("\tNo exact divider found");
    //         continue;
    //     };

    //     println!("\tVCO Freq = {}kHz, Post Divider 1 = {}, Post Divider 2 = {}", vco_freq.to_kHz(), post_div1, post_div_2);
    // }
}

// TODO(bschwind) - It seems lower jitter can be achieved by optimizing for higher frequencies
//                  and larger dividers.
pub fn check_sys_clock(desired_frequency: HertzU32) -> Option<(HertzU32, u8, u8)> {
    let freq_khz = desired_frequency.to_kHz();
    let reference_divider = 1;
    let reference_freq_khz: u32 = CRYSTAL_FREQUENCY_KHZ / reference_divider;

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

// Audio sample rate: 48_000
// Bit clock rate: 48_000 * 64 = 3,072,000 Hz
// Desired system clock: 40 - 122,880,000 Hz
//                       39 - 119,808,000