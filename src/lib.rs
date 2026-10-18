mod logic;
use bbmod::dualsense::{self as ds, Effect, Trigger};
use bbmod::{Cx, Mod, Result, Ui};
use logic::{Logic, Settings};
use std::sync::{Arc, Mutex};
use std::time::Instant;
struct State {
    settings: Settings,
    logic: Logic,
    start: Instant,
    status: String,
    last: [Effect; 2],
    error: String,
    raw: String,
    raw_side: usize,
    preview: Option<(usize, Effect, Option<u64>)>,
    device: u64,
}
struct Adaptive {
    state: Arc<Mutex<State>>,
}
fn config(cx: &Cx) -> Settings {
    let mut s = Settings::default();
    s.enabled = cx.config_bool("general.enabled", true);
    s.firearm = cx.config_bool("map.firearm_feel_on_l2", true);
    s.intensity = cx
        .config_parse::<f32>("general.intensity")
        .filter(|v| v.is_finite())
        .unwrap_or(1.0)
        .clamp(0.0, 1.5);
    s.lightbar = cx.config("general.lightbar").as_deref() != Some("off");
    if let Some(v) = cx.config("general.lightbar_colour") {
        if let Ok(v) = v
            .split(',')
            .map(|v| v.trim().parse::<u8>())
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            if let Ok(rgb) = v.try_into() {
                s.colour = rgb;
            }
        }
    }
    for (i, key) in ["map.l2", "map.r2"].iter().enumerate() {
        if let Some(v) = cx.config(key) {
            s.mapping[i] = match v.as_str() {
                "strong_attack" => 1,
                "left_hand" => 2,
                _ => 0,
            };
        }
    }
    // Parse the specification's small preset grammar, falling back on malformed values.
    for (key, target) in [("r2.rest", &mut s.rest), ("l2.press", &mut s.press)] {
        if let Some(v) = cx.config(key) {
            let p: Vec<_> = v.split_whitespace().collect();
            if p.len() == 3 && p[0] == "resistance" {
                if let (Ok(a), Ok(b)) = (p[1].parse(), p[2].parse()) {
                    *target = [a, b];
                }
            }
        }
    }
    if let Some(v) = cx.config("r2.charge") {
        let p: Vec<_> = v.split_whitespace().collect();
        if p.len() == 5 && p[0] == "ramp" && p[3] == "over" {
            if let (Ok(a), Ok(b), Ok(ms)) = (
                p[1].parse(),
                p[2].parse(),
                p[4].trim_end_matches("ms").parse::<u64>(),
            ) {
                s.charge = [a, b];
                s.charge_ms = ms.clamp(100, 5000);
            }
        }
    }
    if let Some(v) = cx.config("l2.shot") {
        let p: Vec<_> = v.split_whitespace().collect();
        if p.len() == 6 && p[0] == "resistance_vibration" && p[4] == "for" {
            if let (Ok(a), Ok(b), Ok(c), Ok(ms)) = (
                p[1].parse(),
                p[2].parse(),
                p[3].parse(),
                p[5].trim_end_matches("ms").parse::<u64>(),
            ) {
                s.shot = [a, b, c];
                s.shot_ms = ms.clamp(34, 500);
            }
        }
    }
    s
}
fn reset(cx: &Cx, s: &mut State) {
    s.preview = None;
    s.logic.reset();
    for side in [Trigger::Left, Trigger::Right] {
        let _ = cx.pad_release(side);
    }
    s.last = [ds::off(); 2];
}
fn frame(cx: &Cx, s: &mut State) {
    if cx.key_down(bbmod::key::ESCAPE) {
        s.settings.enabled = false;
        reset(cx, s);
        let _ = cx.pad_lightbar([0; 3]);
        return;
    }
    let p = match cx.pad_info() {
        Ok(p) => p,
        Err(e) => {
            s.status = format!("Pad API unavailable: {e}");
            reset(cx, s);
            return;
        }
    };
    if p.features & ds::HAS_TRIGGERS == 0 {
        s.status = "No DualSense detected".into();
        reset(cx, s);
        s.device = 0;
        return;
    }
    if s.device != p.device_id {
        s.device = p.device_id;
        s.logic.reset();
        s.preview = None;
    }
    let captured = p.features & ds::CAPTURED != 0;
    s.status = format!(
        "DualSense | {} | {}",
        match p.connection {
            1 => "USB",
            2 => "Wireless",
            _ => "Unknown connection",
        },
        if captured {
            "Input captured: triggers off"
        } else {
            "Input-driven effects"
        }
    );
    if p.features & 8 != 0 {
        s.status.push_str(" | Output failed; host is retrying");
    }
    let now = s.start.elapsed().as_millis() as u64;
    if !s.settings.enabled {
        reset(cx, s);
        let _ = cx.pad_lightbar([0; 3]);
        return;
    }
    let mut effects = if captured {
        s.logic.reset();
        [ds::off(); 2]
    } else {
        s.logic.effects(&s.settings, [p.l2_depth, p.r2_depth], now)
    };
    // Preview is armed in the panel and starts only after capture ends. It is
    // cancelled when the overlay reopens; never bypass host capture safety.
    if let Some((side, effect, started)) = s.preview {
        if captured && started.is_some() {
            s.preview = None;
        } else if !captured {
            let start = started.unwrap_or(now);
            if now - start < 1000 {
                effects[side] = effect;
                s.preview = Some((side, effect, Some(start)));
            } else {
                s.preview = None;
            }
        }
    }
    let mut error = String::new();
    for (i, side) in [Trigger::Left, Trigger::Right].into_iter().enumerate() {
        let result = if s.settings.mapping[i] == 0 && s.preview.is_none() {
            cx.pad_release(side)
        } else {
            cx.pad_trigger(side, effects[i])
        };
        if let Err(e) = result {
            error.push_str(&format!("{side:?}: {e}; "));
        }
    }
    if p.features & ds::HAS_LIGHTBAR != 0 {
        if let Err(e) = cx.pad_lightbar(if s.settings.lightbar {
            s.settings.colour
        } else {
            [0; 3]
        }) {
            error.push_str(&format!("Lightbar: {e}"));
        }
    }
    if error != s.error && !error.is_empty() {
        cx.log_warn(&error);
    }
    s.error = error;
    s.last = effects;
}
fn byte(ui: &Ui, label: &str, v: &mut u8) {
    let mut n = *v as i32;
    ui.slider_i32(label, &mut n, 0, 255);
    *v = n as u8;
}
fn panel(ui: &Ui, s: &mut State) {
    ui.text(&s.status);
    if !s.error.is_empty() {
        ui.text(&s.error);
    }
    ui.checkbox("Enabled", &mut s.settings.enabled);
    ui.slider_f32("Intensity", &mut s.settings.intensity, 0.0, 1.5);
    ui.checkbox("Firearm feel", &mut s.settings.firearm);
    ui.checkbox("Fixed lightbar", &mut s.settings.lightbar);
    for (i, label) in ["Red", "Green", "Blue"].iter().enumerate() {
        byte(ui, label, &mut s.settings.colour[i]);
    }
    ui.combo(
        "Physical L2",
        &mut s.settings.mapping[0],
        &["Off", "Strong attack", "Left hand"],
    );
    ui.combo(
        "Physical R2",
        &mut s.settings.mapping[1],
        &["Off", "Strong attack", "Left hand"],
    );
    if ui.collapsing_header("Presets") {
        byte(ui, "Rest start", &mut s.settings.rest[0]);
        byte(ui, "Rest force", &mut s.settings.rest[1]);
        byte(ui, "Charge initial force", &mut s.settings.charge[0]);
        byte(ui, "Charge final force", &mut s.settings.charge[1]);
        let mut ms = s.settings.charge_ms as i32;
        ui.slider_i32("Charge ms", &mut ms, 100, 5000);
        s.settings.charge_ms = ms as u64;
        byte(ui, "Firearm start", &mut s.settings.press[0]);
        byte(ui, "Firearm force", &mut s.settings.press[1]);
        for (i, label) in ["Buzz A (raw)", "Buzz B (raw)", "Buzz C (raw)"]
            .iter()
            .enumerate()
        {
            byte(ui, label, &mut s.settings.shot[i]);
        }
    }
    ui.text(
        "Tests run for 1 second after closing with F1. Esc resets and disables. Reopen to cancel.",
    );
    if ui.button("Test R2 resistance") {
        s.preview = Some((
            1,
            ds::resistance(s.settings.rest[0], s.settings.charge[1]),
            None,
        ));
    }
    if ui.button("Test L2 buzz") {
        let v = s.settings.shot;
        s.preview = Some((0, ds::resistance_vibration(v[0], v[1], v[2]), None));
    }
    if ui.collapsing_header("Experimental raw effect") {
        ui.input_text("11 hex bytes", &mut s.raw, 64);
        ui.combo("Test trigger", &mut s.raw_side, &["L2", "R2"]);
        if ui.button("Arm raw test") {
            if let Some(e) = logic::raw(&s.raw) {
                s.preview = Some((s.raw_side, e, None));
            } else {
                s.error = "Enter exactly 11 hexadecimal bytes".into();
            }
        }
    }
    if ui.button("Reset triggers and disable") {
        s.settings.enabled = false;
        reset(ui.cx(), s);
    }
    ui.text(&format!(
        "Requested L2: {:02X?}\nRequested R2: {:02X?}",
        s.last[0].0, s.last[1].0
    ));
    ui.text("Phase 2: HP, stamina, bullets, weapon and in-game menus not discovered.");
    ui.text("Panel changes survive hot reload. Edit config/bbmod.dualsense.toml to persist across launches.");
}
impl Mod for Adaptive {
    fn load(cx: &mut Cx) -> Result<Self> {
        let state = Arc::new(Mutex::new(State {
            settings: config(cx),
            logic: Logic::default(),
            start: Instant::now(),
            status: "Waiting for controller".into(),
            last: [ds::off(); 2],
            error: String::new(),
            raw: "05 00 00 00 00 00 00 00 00 00 00".into(),
            raw_side: 1,
            preview: None,
            device: 0,
        }));
        let f = state.clone();
        cx.on_frame(move |cx| {
            if let Ok(mut s) = f.lock() {
                frame(cx, &mut s);
            }
        })?;
        let p = state.clone();
        cx.panel("Adaptive Triggers", move |ui| {
            if let Ok(mut s) = p.lock() {
                panel(ui, &mut s);
            }
        })?;
        cx.log_info(
            "Adaptive Triggers: input-only phase 1; use the panel to disable during in-game menus.",
        );
        Ok(Self { state })
    }
    fn save_state(&self) -> Option<Vec<u8>> {
        self.state.lock().ok().map(|s| s.settings.encode())
    }
    fn restore_state(&mut self, _cx: &mut Cx, bytes: &[u8], _from: bbmod::Version) {
        if let (Some(settings), Ok(mut s)) = (Settings::decode(bytes), self.state.lock()) {
            s.settings = settings;
            s.logic.reset();
            s.preview = None;
        }
    }
    fn unload(&mut self, cx: &mut Cx) {
        if let Ok(mut s) = self.state.lock() {
            reset(cx, &mut s);
        }
    }
}
bbmod::export!(Adaptive);
