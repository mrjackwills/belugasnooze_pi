use crate::{C, blinkt, message_handler::Msg, sleep};
use async_channel::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
enum LimitMinutes {
    Five(Option<()>),
    FortyFive,
}

impl LimitMinutes {
    async fn sleep(self, tx: Sender<LightMsg>) {
        sleep!(self.get_ms());
        tx.send(self.get_message()).await.ok();
    }

    const fn get_message(self) -> LightMsg {
        match self {
            Self::Five(msg) => {
                if msg.is_some() {
                    LightMsg::Alarm
                } else {
                    LightMsg::Off
                }
            }
            Self::FortyFive => LightMsg::Off,
        }
    }

    const fn get_sec(&self) -> u64 {
        match self {
            Self::Five(_) => 5 * 60,
            Self::FortyFive => 45 * 60,
        }
    }

    const fn get_ms(&self) -> u64 {
        self.get_sec() * 1000
    }
}

pub struct LightControl {
    blinkt: Option<blinkt::Blinkt>,
    brightness: f32,
    cancel_token: Option<CancellationToken>,
    colours: (u8, u8, u8),
    light_tx: Sender<LightMsg>,
    msg_tx: Sender<Msg>,
    status: bool,
    step: u8,
}

#[derive(Debug, Clone)]
pub enum LightMsg {
    Alarm,
    Exit,
    Get(Sender<bool>),
    Off,
    Toggle(bool),
}

impl LightControl {
    fn new(msg_tx: &Sender<Msg>, tx: &Sender<LightMsg>) -> Self {
        Self {
            blinkt: blinkt::Blinkt::new().map_or_else(
                |e| {
                    tracing::error!("No Blinkt found: {e}");
                    None
                },
                Some,
            ),
            brightness: 0.0,
            cancel_token: None,
            colours: (0, 0, 0),
            light_tx: C!(tx),
            msg_tx: C!(msg_tx),
            status: false,
            step: 0,
        }
    }

    /// Send settings to the blinkt, to actually turn it on or off
    fn display(&mut self) {
        if let Some(blinkt) = &mut self.blinkt {
            blinkt.clear();
            blinkt.set_all_pixels_brightness(self.brightness);
            blinkt.set_all_pixels(self.colours.0, self.colours.1, self.colours.2);
            blinkt.show().ok();
        }
    }

    /// Turn off the blinkt
    fn turn_off(&mut self) {
        self.brightness = 0.0;
        self.colours = (0, 0, 0);
        self.status = false;
        self.display();
    }

	/// Default colours for the LED strip
    const fn set_default_colour(&mut self) {
        self.colours = (255, 200, 15);
    }

    /// Create and set and cancel token, and copy a sender
    fn get_token_sender(&mut self) -> (CancellationToken, Sender<LightMsg>) {
        let token = CancellationToken::new();
        self.cancel_token = Some(C!(token));
        (token, C!(self.light_tx))
    }

    /// Cancel the sleeping thread
    fn cancel_thead(&self) {
        if let Some(token) = &self.cancel_token {
            token.cancel();
        }
    }

    /// Set the light status
    fn activate(&mut self, limit: LimitMinutes, brightness: f32) {
        self.brightness = brightness;
        self.set_default_colour();
        self.status = true;
        let (token, tx) = self.get_token_sender();
        self.display();
        tokio::spawn(async move {
            token.run_until_cancelled(limit.sleep(tx)).await;
        });
    }

    /// Turn the light on with the default 5-minute timeout.
    fn turn_on(&mut self) {
        self.activate(LimitMinutes::Five(None), 1.0);
    }

    /// Turn the light on for an alarm step.
    async fn alarm_on(&mut self) {
        self.cancel_thead();
        self.msg_tx.send(Msg::SendLEDStatus).await.ok();
        self.step += 1;
        let limit = if self.step < 10 {
            LimitMinutes::Five(Some(()))
        } else {
            LimitMinutes::FortyFive
        };
        let brightness = f32::from(self.step) / 10.0;
        self.activate(limit, brightness);
    }

    /// Toggle the status of the blinkt
    async fn toggle(&mut self, value: bool) {
        self.cancel_thead();
        if value {
            self.turn_on();
        } else {
            self.turn_off();
        }
        self.msg_tx.send(Msg::SendLEDStatus).await.ok();
    }

    /// Message listener
    async fn recv(&mut self, rx: Receiver<LightMsg>) {
        loop {
            if let Ok(x) = rx.recv().await {
                match x {
                    LightMsg::Alarm => self.alarm_on().await,
                    LightMsg::Exit => self.turn_off(),
                    LightMsg::Get(oneshot) => oneshot.send(self.status).await.unwrap_or_default(),
                    LightMsg::Off => self.toggle(false).await,
                    LightMsg::Toggle(status) => self.toggle(status).await,
                }
            }
        }
    }

    /// Start the receiving channel
    pub fn init(msg_tx: &Sender<Msg>) -> Sender<LightMsg> {
        let (tx, rx) = async_channel::bounded(128);
        let mut light_control = Self::new(msg_tx, &tx);
        tokio::spawn(async move {
            light_control.recv(rx).await;
        });
        tx
    }
}
