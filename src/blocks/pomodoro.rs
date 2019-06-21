use std::time::Duration;
use chan::Sender;

use crate::block::{Block, ConfigBlock};
use crate::config::Config;
use crate::de::deserialize_duration;
use crate::errors::*;
//use crate::widgets::text::TextWidget;
use crate::widgets::button::ButtonWidget;
use crate::widget::I3BarWidget;
use crate::input::{I3BarEvent, MouseButton};
use crate::scheduler::Task;

use uuid::Uuid;
use libpom::Pomodoro;
use crate::util::FormatTemplate;

pub struct PomBlock {
    pom_block: ButtonWidget,
    id: String,
    update_interval: Duration,
    pomodoro: Pomodoro,
    format: FormatTemplate,

    //useful, but optional
    //#[allow(dead_code)]
    //config: Config,
    //#[allow(dead_code)]
    //tx_update_request: Sender<Task>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct PomBlockConfig {
    /// Update interval in seconds
    #[serde(default = "PomBlockConfig::default_interval", deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    #[serde(default = "PomBlockConfig::default_pom_len")]
    pub pomodoro_length: u64,
    #[serde(default = "PomBlockConfig::default_break_len")]
    pub break_length: u64,
    #[serde(default = "PomBlockConfig::default_long_break_len")]
    pub long_break_length: u64,
    #[serde(default = "PomBlockConfig::default_round")]
    pub rounds: u64,
    #[serde(default = "PomBlockConfig::default_notification")]
    pub notification: bool,
    #[serde(default = "PomBlockConfig::default_format")]
    pub format: String,
}

impl PomBlockConfig {
    fn default_interval() -> Duration {
        Duration::from_secs(1)
    }

    fn default_pom_len() -> u64 {
        (25 * 60)
    }

    fn default_break_len() -> u64 {
        (5 * 60)
    }

    fn default_long_break_len() -> u64 {
        (15 * 60)
    }

    fn default_round() -> u64 {
        4
    }

    fn default_notification() -> bool {
        true
    }

    fn default_format() -> String {
        "{phase} {remainingMinutes}:{remainingSeconds} {round}".into()
    }
}

impl ConfigBlock for PomBlock {
    type Config = PomBlockConfig;

    fn new(block_config: Self::Config, config: Config, _tx_update_request: Sender<Task>) -> Result<Self> {
        let pom = Pomodoro::new(
            block_config.pomodoro_length,
            block_config.break_length,
            block_config.long_break_length,
            block_config.rounds,
            block_config.notification,
        );
        
        let id = Uuid::new_v4().simple().to_string();

        Ok(PomBlock {
            id: id.clone(),
            pom_block: ButtonWidget::new(config, &id),
            pomodoro: pom,
            update_interval: block_config.interval,
            format: FormatTemplate::from_string(&block_config.format)?,
        })
    }
}

impl Block for PomBlock {
    fn update(&mut self) -> Result<Option<Duration>> {
        let status = self.pomodoro.status();
        let remaining = status.remaining.as_secs();
        let minutes = remaining / 60;
        let seconds = remaining % 60;
        let values = map!("{phase}" => format!("{}", status.phase),
                          "{remainingMinutes}" => format!("{:02}", minutes),
                          "{remainingSeconds}" => format!("{:02}", seconds),
                          "{round}" => format!("{}", status.round));
        self.pom_block.set_text(self.format.render_static_str(&values)?);
        Ok(Some(self.update_interval))
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.pom_block]
    }

    fn click(&mut self, event: &I3BarEvent) -> Result<()> {
        if event.matches_name(self.id()) {
            if let MouseButton::Left = event.button {
                self.pomodoro.play();
            }
            if let MouseButton::Right = event.button {
                self.pomodoro.pause();
            }
        }
        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}
