use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChannelData {
    pub enabled: bool,
    pub waveform: String,
    pub frequency: f64,
    pub amplitude: f64,
    pub offset: f64,
    pub duty_cycle: f64,
}

impl From<crate::driver::ChannelState> for ChannelData {
    fn from(s: crate::driver::ChannelState) -> Self {
        Self {
            enabled: s.enabled,
            waveform: s.waveform,
            frequency: s.frequency,
            amplitude: s.amplitude,
            offset: s.offset,
            duty_cycle: s.duty_cycle,
        }
    }
}

impl From<ChannelData> for crate::driver::ChannelState {
    fn from(d: ChannelData) -> Self {
        Self {
            enabled: d.enabled,
            waveform: d.waveform,
            frequency: d.frequency,
            amplitude: d.amplitude,
            offset: d.offset,
            duty_cycle: d.duty_cycle,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Preset {
    pub ch1: ChannelData,
    pub ch2: ChannelData,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PresetBank {
    #[serde(flatten)]
    pub slots: std::collections::HashMap<String, Option<Preset>>,
}

impl PresetBank {
    pub fn load_or_default(path: &str) -> Self {
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(bank) = serde_json::from_str::<Self>(&text) {
                return bank;
            }
        }
        let mut slots = std::collections::HashMap::new();
        for i in 1..=8 {
            slots.insert(i.to_string(), None);
        }
        Self { slots }
    }

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }
}
