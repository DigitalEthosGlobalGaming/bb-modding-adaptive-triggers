# Adaptive Triggers — bbmod.dualsense

Input and equipment-driven implementation of the DualSense specification in this folder. Built on Windows on 2026-10-18. The folder name is `adaptive-triggers`; the mod ID is `bbmod.dualsense`.

## Build and install

```powershell
.\build.ps1           # build and install into the working game
.\build.ps1 -Test     # run checks before installing
.\build.ps1 -NoDeploy # package only
```

A successful build installs the DLL directly into `C:\games\bloodborne-pc\bbport-windows\bbmods\bbmod.dualsense`. The default path is resolved relative to the script, so it also works from another working directory. Use `-ModsDir <path>` for another runtime's bbmods folder.

Replacement is atomic. The previous installed DLL is saved to `out/bbmod_dualsense.previous.dll`. Existing `mod.toml` settings and external config overrides are preserved; the manifest is installed only if missing. Build/test failures stop before deployment. The `dist/bbmod.dualsense` package is still produced with the current source manifest.

For an already discovered mod, use **F1 > Mods > Reload** after building, or launch with `BB_MODS_DEV=1` to reload rebuilt DLLs automatically. Five-second discovery applies to new folders, not updates to existing DLLs.

Requires Rust and MSYS2 CLANG64. Optional `-ClangBin` and `-CargoBin` override their locations. The mod requires **bbmod ABI 1.3**, implemented in the adjacent source tree. It will not run on the original v1.8.2 installation or an unmodified ABI 1.2 host. The updated host and mod were deployed to the working installation on 2026-10-18. This script updates only the mod, not the host executable.

Build the separate host with:

```powershell
$env:PATH = 'C:\msys64\clang64\bin;' + $env:PATH
cmake --build ..\..\bloodborne_pc\out\gpu --target bb-probe --parallel 2
```

Host output: `../../bloodborne_pc/out/gpu/bb-probe.exe`. Deployment of future host changes is a separate step requiring a rollback copy and a game restart.

## Controls and configuration

F1 opens the overlay. ABI 1.4 places status in **bbmod.dashboard** and controls in **bbmod.settings > Adaptive Triggers**, alongside other mods. Older compatible hosts use the standalone **Adaptive Triggers** panel. Physical R2 defaults to a strong-attack resistance ramp, followed by a give at 900 ms. Physical L2 gives a short resistance pulse followed by a brief buzz. These input effects do not confirm that a shot or attack happened in the game. Mapping, intensity, firearm feel, presets and RGB lightbar are configurable in the panel.

Overlay/settings capture forces triggers off and dims the lightbar. Tests are armed in the panel and run for one second after closing with F1. Reopening cancels a running test. Esc, when observed by the mod's frame input, resets and disables effects; the panel's **Reset triggers and disable** is the explicit reset. Raw tests require exactly eleven hexadecimal bytes and are experimental. Restore Enabled to resume.

Panel settings survive a hot reload. For settings across launches, copy relevant sections from `mod.toml` to `config/bbmod.dualsense.toml` beside the runtime's `bbmods` directory. The host layers those keys over packaged defaults when the mod loads. A hot reload preserves panel settings; unload/load to re-read changed file settings without saved state. Invalid preset strings fall back to defaults. HP lightbar mode is not implemented; use `fixed` or `off`.

## Framework support

- Append-only ABI 1.3: pad info, trigger effects, release and lightbar. Existing C ABI fields retain their offsets.
- Host ownership per trigger and for the lightbar, conflict errors, cleanup on unload/reload/recovered mod faults, and resource counts in the Mods view.
- Coalesced updates at most once every 34 ms. Safety resets bypass the cap. Failed sends remain dirty, retry at 1 Hz and log once per outage.
- Device IDs invalidate cached output after reconnect. Physical trigger axes are sampled directly, independently of input.ini action remapping.
- SDL calls use bbport's existing pad mutex and handle, with no separate HID connection. Both triggers receive off on normal process exit.
- SDK builders for off, resistance, resistance-with-vibration, and experimental raw packets.
- No output to other controller types. DualSense Edge is treated as a PS5-family controller, not separately identified.

## Verification and remaining work

`build.ps1 -Test` runs thirteen mod unit tests, sixteen SDK tests, native fake-backend tests, the actual runtime_pad packet test with stubbed effect sends, and a real DLL integration test. The latter exercises panel preview/reset, hot reload with settings transfer, simulated host fault cleanup, configuration overrides and unload. Packet tests cover exact 47-byte layout, optional trigger masks, LED calls, neutral packets and non-DualSense rejection. The modified full bbport executable also compiles and links.

No gameplay or controller hardware validation is claimed. Check USB first, then Bluetooth, rumble coexistence, reconnect, overlay capture, unload, normal exit and DualSense Edge. If rumble clears trigger effects, add the conditional refresh mitigation from specification section 8.4 after reproducing it. A hard process kill or unrecoverable process crash cannot guarantee controller cleanup through atexit.

Equipment-driven effects are now implemented using only the shared read-only equipment and inventory APIs. No new hooks, memory writes, or native calls are installed. Enable/disable **Equipment and empty-ammo effects** in the shared settings window, or set `[gameplay] enabled = "false"` in your external config to return to input-only presets. This setting survives hot reload, including migration from the original saved-state format.

The mod reads a revision-consistent inventory and checks equipment before/after, at most every 250 ms. Hand names are not assigned to even/odd indices: a unique known melee selection supplies the strong-attack profile and a unique known gun/shield/torch selection supplies the left-hand profile. Both remain unresolved if ambiguous. Current presets cover 25 catalog families (including supported upgrades and Uncanny/Lost melee variants). Unknown items do not get guessed firearm effects. Unknown melee selections retain the input-only strong-attack preset. The panel displays the selected profile names and sampled ammunition; changes are logged as `Equipment feedback sample`.

Feel presets are authored, not measured physical weapon weights: heavy weapons such as Kirkhammer and Logarius' Wheel have stronger/slower ramps; Blade of Mercy and Threaded Cane have lighter/faster ramps. Pistol, blunderbuss and cannon pulses differ in strength and duration. Shields give steady resistance, torches gentle resistance, and sprayers sustained feedback while held. These effects follow input rather than claiming a successful native attack. Armor weight, transformed/two-handed states, durability, gems, DLC-only families and native menu detection remain unimplemented because their semantics are unverified.

A complete resolved carried inventory with zero ammunition produces **off** on the firearm trigger: no resistance and no shot buzz. Nonzero sampled blood bullets prevent the empty state. Ammunition uses the SDK constants `items::ids::QUICKSILVER_BULLETS` and `items::ids::BLOOD_BULLETS`. Any unresolved entry or ID absent from the SDK catalog makes ammo unknown rather than claiming zero (including the previously observed ID 1020, which has no current catalog row). This is carried-inventory observation; whether every temporary blood-bullet state appears there still needs a HUD comparison. It does not implement per-weapon bullet costs or detect successful shots.

Unsupported services/layouts fall back to input-only presets. Temporarily unavailable/unstable samples or an unavailable/dead player cause neutral effects until a fresh sample. Cached gameplay data is never saved across reloads. Hardware preview tests intentionally override gameplay profiles, including empty ammo, for their one-second test duration.

The real-DLL harness now also covers cleaver/pistol selection, empty-to-loaded-to-empty ammo, axe/shield swaps and neutral cleanup. These synthetic tests passed; Inspector did not respond during this task, so no live weapon/HUD matching or trigger-feel validation is claimed. In game, compare panel names/counts to the HUD, fire the last bullet, generate blood bullets, switch weapon/gear, and test loading/respawn. Input/menu controls and output ownership remain as documented above.

The original specification remains the design record. Applied source is authoritative; the older `integration/bbport-modhost.patch` has not been regenerated with these additions.

Broader framework regressions on 2026-10-18: all original 12 CTest cases passed across initial runs and reruns. `host` and `inline_hooks_os` exceeded an initial 30-second timeout, then passed with 180 seconds allowed. After periodic discovery was added, host, pad-effects and discovery regressions also passed.

Item identification uses the SDK `items` catalog: resolved inventory IDs are translated once to exact named codes, such as `saw_cleaver`, before profile selection. Profiles explicitly list supported base/Lost/Uncanny codes; upgrade suffixes are considered only after an exact catalog lookup. There is no numeric family or upgrade inference. The panel uses catalog display names, including variants and upgrade levels. This requires rebuilding the mod against the updated SDK, with no host ABI change.

## USB grip haptics

The mod now drives the DualSense's grip actuators using authored stereo PCM,
separately from the trigger motors. The host opens only a uniquely matching
four-channel DualSense/Wireless Controller audio endpoint with one connected
PS5 controller. It never uses the default audio device. Speaker/headphone
channels remain silent; the rear pair carries the left/right actuator waveform.
Endpoint matching uses name and channel layout, not verified USB container IDs;
renamed endpoints and multiple matching devices are deliberately unsupported.
Bluetooth actuator audio is not implemented. Normal Bluetooth trigger effects
remain available.

A new optional, size-gated `pad_haptics` SDK/host extension is required. Older hosts
continue to support triggers and report grip haptics unavailable. The updated
bbport host must be installed once and the game restarted; subsequent mod-only
changes still use the normal build/hot-reload workflow.

In F1 > bbmod.settings > Adaptive Triggers, enable **USB grip haptics**, adjust
**Grip intensity**, and expand **Grip texture tests**. Choose stone, wood, grass,
or water, arm the test, then close F1. Each preview is a finite 100–220 ms clip;
reopening the overlay stops it. Esc and the global reset also stop grip output.
Settings survive hot reload; persistent overrides are `[haptics] enabled =
"true"` and `intensity = "0.45"` in the external mod config.

With a resolved loadout, R1/strong-attack input gives lighter, sharper cues for
lighter weapons and deeper, longer pulses for heavy weapons. Firearm input gives
a sharp shot or a heavier blast; sprayers give repeated textured clips while
held. Empty ammunition suppresses firearm grip effects. These are input-driven
cues, not confirmed hits, recoil events or animation-synchronised impacts.
Shield/torch trigger resistance remains unchanged; no grip pulse is invented.

**Automatic footsteps and floor-material selection are not implemented.** The
framework has no verified surface/contact event yet. Stone/wood/grass/water are
manual authored previews only; they do not claim to identify the floor below the
player. No movement-stick timer substitutes for actual footsteps.

Clips are at most 250 ms, bounded in amplitude, and replace rather than queue up.
The host owns output and stops it on capture, disconnect, unload or recovered mod
fault. Native emulated rumble is suppressed only during a grip clip (plus 30 ms)
because SDL rumble can disable audio haptics; later game rumble requests work
normally. Missed native rumble requests during the clip are not replayed. A crash
that skips cleanup can leave only the finite already-submitted clip, not a loop.

Validation: 13 mod and 16 SDK tests, host ownership/capture/fault/bounds tests,
SDL quad-channel routing/endpoint rejection tests, and real-DLL input, empty-ammo,
preview/capture/reload checks pass. A read-only SDL probe detected the attached
DualSense over USB and its four-channel 48 kHz audio endpoint. No claim of
in-game footstep matching or human-verified tactile quality is made.
