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
            while light_status.load(Ordering::SeqCst) {
                if Self::light_limit(start, &LimitMinutes::Five) {
                    light_status.store(false, Ordering::SeqCst);
                }
                Self::light_limit(start, &LimitMinutes::Five);
                led_strip.show().unwrap_or(());
                sleep(Duration::from_millis(250)).await;
            }
        } else {
            while light_status.load(Ordering::SeqCst) {
                if Self::light_limit(start, &LimitMinutes::Five) {
                    light_status.store(false, Ordering::SeqCst);
                }
                debug!("light on");
                sleep(Duration::from_millis(250)).await;
            }
        }
        sx.send(InternalMessage::Light).unwrap_or_default();
    }

    /// Turn light on in steps of 10% brightness, 5 minutes for each step, except last step which stays on for 45 minutes
    /// Will stop if the `light_status` atomic bool is changed elsewhere during the execution
    pub async fn alarm_illuminate(light_status: Arc<AtomicBool>, sx: Sender<InternalMessage>) {
        light_status.store(true, Ordering::SeqCst);
        sx.send(InternalMessage::Light).unwrap_or_default();
        tokio::spawn(async move {
            let mut brightness = 1.0;
            let mut step = 0;
            let mut start = Instant::now();
            if let Ok(mut led_strip) = Blinkt::new() {
                led_strip.clear();
                led_strip.set_all_pixels(255, 200, 15);
                led_strip.set_all_pixels_brightness(brightness / 10.0);
                while light_status.load(Ordering::SeqCst) {
                    led_strip.show().unwrap_or(());
                    let limit = if step < 9 {
                        LimitMinutes::Five
                    } else {
                        LimitMinutes::FortyFive
                    };
                    if Self::light_limit(start, &limit) {
                        start = Instant::now();
                        step += 1;
                        brightness += 1.0;
                        led_strip.set_all_pixels_brightness(brightness / 10.0);
                        if let LimitMinutes::FortyFive = limit {
                            light_status.store(false, Ordering::SeqCst);
                            led_strip.clear();
                        };
                    };
                    sleep(Duration::from_millis(250)).await;
                }
            } else {
                while light_status.load(Ordering::SeqCst) {
                    let limit = if step < 9 {
                        LimitMinutes::Five
                    } else {
                        LimitMinutes::FortyFive
                    };
                    if Self::light_limit(start, &limit) {
                        debug!("step: {}, brightness: {}", step, brightness / 10.0);
                        step += 1;
                        brightness += 1.0;
                        start = Instant::now();
                        if let LimitMinutes::FortyFive = limit {
                            light_status.store(false, Ordering::SeqCst);
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
        if !x.load(Ordering::SeqCst) {
            for (pixel, color) in RAINBOW_COLORS.into_iter().enumerate() {
                Self::show_rainbow(pixel, color).await;
            }

            for (pixel, color) in RAINBOW_COLORS.into_iter().rev().enumerate() {
                Self::show_rainbow(RAINBOW_COLORS.len() - 1 - pixel, color).await;
            }
        }
    }
}
