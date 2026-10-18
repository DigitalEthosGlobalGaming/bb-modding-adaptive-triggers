//! Authored actuator waveforms; input cues are not native hit/footstep events.
use crate::{
    game::{Kind, Loadout},
    logic::Settings,
};
#[derive(Clone, Copy, Debug)]
pub enum Feel {
    Light,
    Heavy,
    Shot,
    Blast,
    Spray,
    Stone,
    Wood,
    Grass,
    Water,
}
pub const TEXTURES: [Feel; 4] = [Feel::Stone, Feel::Wood, Feel::Grass, Feel::Water];
pub fn waveform(feel: Feel, gain: f32, left: bool) -> Vec<[f32; 2]> {
    let (ms, hz, grit, decay) = match feel {
        Feel::Light => (90, 150.0, 0.15, 4.0),
        Feel::Heavy => (230, 65.0, 0.2, 3.0),
        Feel::Shot => (100, 170.0, 0.25, 6.0),
        Feel::Blast => (220, 75.0, 0.45, 4.0),
        Feel::Spray => (110, 110.0, 0.6, 0.4),
        Feel::Stone => (100, 190.0, 0.3, 7.0),
        Feel::Wood => (170, 95.0, 0.12, 4.0),
        Feel::Grass => (140, 145.0, 0.85, 5.0),
        Feel::Water => (220, 70.0, 0.65, 2.5),
    };
    let gain = if gain.is_finite() {
        gain.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let frames = ms * 48;
    let mut seed = 0x12345678u32;
    (0..frames)
        .map(|i| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let noise = (seed as f32 / u32::MAX as f32) * 2.0 - 1.0;
            let t = i as f32 / 48000.0;
            let x = i as f32 / (frames - 1) as f32;
            let envelope = (t / 0.004).min(1.0) * (-decay * x).exp() * ((1.0 - x) / 0.1).min(1.0);
            let carrier = (std::f32::consts::TAU * hz * t).sin();
            let sample = 0.45 * gain * envelope * (carrier * (1.0 - grit) + noise * grit);
            if left {
                [sample, sample * 0.4]
            } else {
                [sample * 0.4, sample]
            }
        })
        .collect()
}
#[derive(Default)]
pub struct Input {
    held: [bool; 2],
    next_spray: u64,
}
impl Input {
    pub fn sync(&mut self, depths: [u8; 2]) {
        self.held = depths.map(|d| d >= 20);
        self.next_spray = 0;
    }
    pub fn cue(
        &mut self,
        s: &Settings,
        depths: [u8; 2],
        r1: bool,
        now: u64,
        loadout: Option<&Loadout>,
    ) -> Option<(Feel, bool)> {
        let mut pressed = [false; 2];
        for i in 0..2 {
            if depths[i] < 20 {
                self.held[i] = false;
            }
            if depths[i] >= 40 && !self.held[i] {
                pressed[i] = true;
                self.held[i] = true;
            }
        }
        let l = loadout?;
        for i in 0..2 {
            if s.mapping[i] != 2 || !s.firearm {
                continue;
            }
            let Some(p) = l.left_profile() else {
                continue;
            };
            if l.ammo == Some(0) {
                continue;
            }
            match p.kind {
                Kind::Gun if pressed[i] => {
                    return Some((
                        if p.force >= 120 {
                            Feel::Blast
                        } else {
                            Feel::Shot
                        },
                        true,
                    ))
                }
                Kind::Sprayer if depths[i] >= 40 && now >= self.next_spray => {
                    self.next_spray = now + 120;
                    return Some((Feel::Spray, true));
                }
                _ => {}
            }
        }
        if r1 || (0..2).any(|i| pressed[i] && s.mapping[i] == 1) {
            let p = l.right_profile()?;
            return Some((
                if p.force >= 165 {
                    Feel::Heavy
                } else {
                    Feel::Light
                },
                false,
            ));
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_bounded_stereo_waveforms_and_textures() {
        for feel in [
            Feel::Light,
            Feel::Heavy,
            Feel::Shot,
            Feel::Blast,
            Feel::Spray,
            Feel::Stone,
            Feel::Wood,
            Feel::Grass,
            Feel::Water,
        ] {
            let w = waveform(feel, 1.0, true);
            assert!(w.len() <= 12000 && w.len() > 1000);
            assert_eq!(w[0], [0.0; 2]);
            assert_eq!(*w.last().unwrap(), [0.0; 2]);
            assert!(w.iter().flatten().all(|x| x.is_finite() && x.abs() <= 0.5));
            assert!(w.iter().any(|x| x[0].abs() > x[1].abs()));
        }
        assert_ne!(
            waveform(Feel::Stone, 1.0, true),
            waveform(Feel::Wood, 1.0, true)
        );
        assert!(waveform(Feel::Water, f32::NAN, true)
            .iter()
            .flatten()
            .all(|x| *x == 0.0));
    }
    #[test]
    fn empty_ammo_press_edges_and_capture_sync() {
        let s = Settings::default();
        let mut input = Input::default();
        let mut l = Loadout {
            left: Some("hunter_pistol"),
            right: Some("kirkhammer"),
            ammo: Some(0),
            ..Default::default()
        };
        assert!(input.cue(&s, [255, 0], false, 0, Some(&l)).is_none());
        l.ammo = Some(5);
        assert!(input.cue(&s, [255, 0], false, 10, Some(&l)).is_none());
        input.cue(&s, [0, 0], false, 20, Some(&l));
        assert!(matches!(
            input.cue(&s, [255, 0], false, 30, Some(&l)),
            Some((Feel::Shot, true))
        ));
        input.sync([0, 255]);
        assert!(input.cue(&s, [0, 255], false, 40, Some(&l)).is_none());
        assert!(matches!(
            input.cue(&s, [0, 255], true, 50, Some(&l)),
            Some((Feel::Heavy, false))
        ));
    }
}
