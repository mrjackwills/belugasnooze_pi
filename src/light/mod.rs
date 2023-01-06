use crate::ws::InternalMessage;
use blinkt::Blinkt;
use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Instant};
use tracing::debug;

const RAINBOW_COLORS: [(u8, u8, u8); 8] = [
    (255, 0, 0),
    (255, 127, 0),
    (255, 255, 0),
    (0, 255, 0),
    (0, 0, 255),
    (39, 0, 51),
    (139, 0, 255),
    (255, 255, 255),
];

pub struct LightControl;

#[derive(Debug)]
enum LimitMinutes {
    Five,
    FortyFive,
}

impl LimitMinutes {
    const fn get_sec(&self) -> u64 {
        match self {
            Self::Five => 60 * 5,
            Self::FortyFive => 60 * 45,
        }
    }
}

/// Convert from a step (0-10) to the correct wait LimitMinute value
impl From<u8> for LimitMinutes {
    fn from(step: u8) -> Self {
        if step < 9 {
            Self::Five
        } else {
            Self::FortyFive
        }
    }
}

impl fmt::Display for LimitMinutes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = match self {
            Self::FortyFive => "45",
            Self::Five => "5",
        };
        write!(f, "{x}")
    }
}

impl LightControl {
    /// whilst `light_status` is true, set all lights to on
    /// use `light_limit` to make sure led is only on for 5 minutes max
    pub async fn turn_on(light_status: Arc<AtomicBool>, sx: &Sender<InternalMessage>) {
        let start = Instant::now();
        if let Ok(mut led_strip) = Blinkt::new() {
            led_strip.clear();
            led_strip.set_all_pixels(255, 200, 15);
            led_strip.set_all_pixels_brightness(1.0);
            while light_status.load(Ordering::Relaxed) {
                Self::light_limit(start, &LimitMinutes::Five);
                led_strip.show().unwrap_or(());
                sleep(Duration::from_millis(250)).await;
                if Self::light_limit(start, &LimitMinutes::Five) {
                    light_status.store(false, Ordering::Relaxed);
                }
            }
        } else {
            while light_status.load(Ordering::Relaxed) {
                if Self::light_limit(start, &LimitMinutes::Five) {
                    light_status.store(false, Ordering::Relaxed);
                }
                debug!("light on");
                sleep(Duration::from_millis(250)).await;
            }
        }
        sx.send(InternalMessage::Light).unwrap_or_default();
    }

    /// Increment the brightness & associated values
    fn increment_step(step: &mut u8, brightness: &mut f32, start: &mut Instant) {
        *step += 1;
        *brightness += 1.0;
        *start = Instant::now();
    }

    /// Turn light on in steps of 10% brightness, 5 minutes for each step, except last step which stays on for 45 minutes
    /// Will stop if the `light_status` atomic bool is changed elsewhere during the execution
    /// TODO this is messy, need to clean & refactor
    pub async fn alarm_illuminate(light_status: Arc<AtomicBool>, sx: Sender<InternalMessage>) {
        light_status.store(true, Ordering::Relaxed);
        sx.send(InternalMessage::Light).unwrap_or_default();
        tokio::spawn(async move {
            let mut brightness = 1.0;
            let mut step = 0u8;
            let mut start = Instant::now();

            if let Ok(mut led_strip) = Blinkt::new() {
                led_strip.clear();
                led_strip.set_all_pixels(255, 200, 15);
                led_strip.set_all_pixels_brightness(brightness / 10.0);

                while light_status.load(Ordering::Relaxed) {
                    led_strip.show().unwrap_or(());
                    let limit = LimitMinutes::from(step);
                    if Self::light_limit(start, &limit) {
                        Self::increment_step(&mut step, &mut brightness, &mut start);
                        led_strip.set_all_pixels_brightness(brightness / 10.0);
                        if let LimitMinutes::FortyFive = limit {
                            light_status.store(false, Ordering::Relaxed);
                            led_strip.clear();
                        };
                    };
                    sleep(Duration::from_millis(250)).await;
                }
            } else {
                while light_status.load(Ordering::Relaxed) {
                    let limit = LimitMinutes::from(step);
                    if Self::light_limit(start, &limit) {
                        debug!("step: {}, brightness: {}", step, brightness / 10.0);
                        Self::increment_step(&mut step, &mut brightness, &mut start);
                        if let LimitMinutes::FortyFive = limit {
                            light_status.store(false, Ordering::Relaxed);
                        };
                    };
                    sleep(Duration::from_millis(250)).await;
                }
            }
            sx.send(InternalMessage::Light).unwrap_or_default();
        });
    }

    /// Return true if start time longer than given limit
    fn light_limit(start: Instant, limit: &LimitMinutes) -> bool {
        start.elapsed().as_secs() > limit.get_sec()
    }

    /// Show color on single led light for 50 ms
    async fn show_rainbow(pixel: usize, color: (u8, u8, u8)) {
        let brightness = 1.0;
        if let Ok(mut led_strip) = Blinkt::new() {
            led_strip.clear();
            led_strip.set_pixel_brightness(pixel, brightness);
            led_strip.set_pixel(pixel, color.0, color.1, color.2);
            led_strip.show().unwrap_or(());
            sleep(Duration::from_millis(50)).await;
        } else {
            debug!(
                "show_rainbow::{pixel} - ({},{},{})",
                color.0, color.1, color.2
            );
            sleep(Duration::from_millis(50)).await;
        }
    }

    /// Loop over array of rgb colors, send each to the led strip one at a time
    pub async fn rainbow(x: Arc<AtomicBool>) {
        if !x.load(Ordering::Relaxed) {
            for (pixel, color) in RAINBOW_COLORS.into_iter().enumerate() {
                Self::show_rainbow(pixel, color).await;
            }
            for (pixel, color) in RAINBOW_COLORS.into_iter().rev().enumerate() {
                Self::show_rainbow(RAINBOW_COLORS.len() - 1 - pixel, color).await;
            }
        }
    }
}
