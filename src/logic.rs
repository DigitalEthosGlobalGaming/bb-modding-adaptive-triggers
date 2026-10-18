use bbmod::dualsense::{self as ds, Effect};
#[derive(Clone, Debug)]
pub struct Settings {
    pub enabled: bool,
    pub intensity: f32,
    pub firearm: bool,
    pub lightbar: bool,
    pub colour: [u8; 3],
    pub mapping: [usize; 2],
    pub rest: [u8; 2],
    pub charge: [u8; 2],
    pub charge_ms: u64,
    pub press: [u8; 2],
    pub shot: [u8; 3],
    pub shot_ms: u64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            firearm: true,
            lightbar: true,
            colour: [120, 0, 10],
            mapping: [2, 1],
            rest: [20, 30],
            charge: [30, 180],
            charge_ms: 900,
            press: [10, 90],
            shot: [15, 63, 128],
            shot_ms: 100,
        }
    }
}
#[derive(Default)]
pub struct Logic {
    held: [Option<u64>; 2],
}
impl Logic {
    pub fn reset(&mut self) {
        self.held = [None; 2];
    }
    pub fn effects(&mut self, s: &Settings, depths: [u8; 2], now: u64) -> [Effect; 2] {
        let mut out = [ds::off(); 2];
        if !s.enabled {
            self.reset();
            return out;
        }
        let scale = |v: u8| (v as f32 * s.intensity.clamp(0.0, 1.5)).clamp(0.0, 255.0) as u8;
        for i in 0..2 {
            // Hysteresis prevents retriggering around the press threshold.
            if depths[i] < 20 {
                self.held[i] = None;
            } else if depths[i] >= 40 && self.held[i].is_none() {
                self.held[i] = Some(now);
            }
            let elapsed = self.held[i].map(|t| now.saturating_sub(t));
            out[i] = match s.mapping[i] {
                1 => match elapsed {
                    None => ds::resistance(s.rest[0], scale(s.rest[1])),
                    Some(t) if t < s.charge_ms => {
                        let force = s.charge[0] as f32
                            + (s.charge[1] as f32 - s.charge[0] as f32) * t as f32
                                / s.charge_ms.max(1) as f32;
                        ds::resistance(s.rest[0], scale(force as u8))
                    }
                    _ => ds::off(),
                },
                2 if s.firearm => match elapsed {
                    Some(t) if t < 34 => ds::resistance(s.press[0], scale(s.press[1])),
                    Some(t) if t < 34 + s.shot_ms => {
                        ds::resistance_vibration(s.shot[0], scale(s.shot[1]), s.shot[2])
                    }
                    _ => ds::off(),
                },
                _ => ds::off(),
            };
            if s.intensity <= 0.0 {
                out[i] = ds::off();
            }
        }
        out
    }
}
pub fn raw(text: &str) -> Option<Effect> {
    let bytes: Vec<_> = text
        .split_whitespace()
        .map(|s| u8::from_str_radix(s, 16))
        .collect::<Result<_, _>>()
        .ok()?;
    Some(ds::raw(bytes.try_into().ok()?))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn charge_ramp_give_release() {
        let mut l = Logic::default();
        let s = Settings::default();
        assert_eq!(l.effects(&s, [0, 0], 0)[1], ds::resistance(20, 30));
        assert_eq!(l.effects(&s, [0, 255], 100)[1], ds::resistance(20, 30));
        assert_eq!(l.effects(&s, [0, 255], 550)[1], ds::resistance(20, 105));
        assert_eq!(l.effects(&s, [0, 255], 1000)[1], ds::off());
        assert_eq!(l.effects(&s, [0, 0], 1100)[1], ds::resistance(20, 30));
    }
    #[test]
    fn shot_is_one_pulse_per_press() {
        let mut l = Logic::default();
        let s = Settings::default();
        assert_eq!(l.effects(&s, [200, 0], 0)[0], ds::resistance(10, 90));
        assert_eq!(
            l.effects(&s, [200, 0], 34)[0],
            ds::resistance_vibration(15, 63, 128)
        );
        assert_eq!(l.effects(&s, [200, 0], 134)[0], ds::off());
        assert_eq!(l.effects(&s, [200, 0], 1000)[0], ds::off());
    }
    #[test]
    fn mapping_disable_and_zero_intensity() {
        let mut l = Logic::default();
        let mut s = Settings::default();
        s.mapping = [1, 2];
        assert_eq!(l.effects(&s, [0, 0], 0)[0], ds::resistance(20, 30));
        s.enabled = false;
        assert_eq!(l.effects(&s, [255, 255], 1), [ds::off(); 2]);
        s.enabled = true;
        s.intensity = 0.0;
        assert_eq!(l.effects(&s, [255, 255], 2), [ds::off(); 2]);
    }
    #[test]
    fn raw_requires_exact_length() {
        assert!(raw("05").is_none());
        assert!(raw("gg 00 00 00 00 00 00 00 00 00 00").is_none());
        assert_eq!(raw("05 00 00 00 00 00 00 00 00 00 00"), Some(ds::off()));
    }
}

impl Settings {
    pub fn encode(&self) -> Vec<u8> {
        let mut b = vec![
            1,
            self.enabled as u8,
            self.firearm as u8,
            self.lightbar as u8,
        ];
        b.extend(self.intensity.to_le_bytes());
        b.extend(self.colour);
        b.extend(self.mapping.map(|v| v as u8));
        b.extend(self.rest);
        b.extend(self.charge);
        b.extend(self.charge_ms.to_le_bytes());
        b.extend(self.press);
        b.extend(self.shot);
        b.extend(self.shot_ms.to_le_bytes());
        b
    }
    pub fn decode(b: &[u8]) -> Option<Self> {
        if b.len() != 38
            || b[0] != 1
            || b[1..4].iter().any(|v| *v > 1)
            || b[11..13].iter().any(|v| *v > 2)
        {
            return None;
        }
        let intensity = f32::from_le_bytes(b[4..8].try_into().ok()?);
        let charge_ms = u64::from_le_bytes(b[17..25].try_into().ok()?);
        let shot_ms = u64::from_le_bytes(b[30..38].try_into().ok()?);
        if !intensity.is_finite()
            || !(0.0..=1.5).contains(&intensity)
            || !(100..=5000).contains(&charge_ms)
            || !(34..=500).contains(&shot_ms)
        {
            return None;
        }
        Some(Self {
            enabled: b[1] != 0,
            firearm: b[2] != 0,
            lightbar: b[3] != 0,
            intensity,
            colour: b[8..11].try_into().ok()?,
            mapping: [b[11] as usize, b[12] as usize],
            rest: b[13..15].try_into().ok()?,
            charge: b[15..17].try_into().ok()?,
            charge_ms,
            press: b[25..27].try_into().ok()?,
            shot: b[27..30].try_into().ok()?,
            shot_ms,
        })
    }
}
