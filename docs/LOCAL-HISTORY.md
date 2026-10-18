# Adaptive Triggers — bbmod.dualsense

Phase 1 implementation of the DualSense specification in this folder. Built on Windows on 2026-10-18. The folder name remains `adaptive-cards`; the mod ID is `bbmod.dualsense`.

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

F1 opens **Adaptive Triggers**. Physical R2 defaults to a strong-attack resistance ramp, followed by a give at 900 ms. Physical L2 gives a short resistance pulse followed by a brief buzz. These input effects do not confirm that a shot or attack happened in the game. Mapping, intensity, firearm feel, presets and RGB lightbar are configurable in the panel.

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

`build.ps1 -Test` runs four Rust logic tests, two SDK tests, native fake-backend tests, the actual runtime_pad packet test with stubbed effect sends, and a real DLL integration test. The latter exercises panel preview/reset, hot reload with settings transfer, simulated host fault cleanup, configuration overrides and unload. Packet tests cover exact 47-byte layout, optional trigger masks, LED calls, neutral packets and non-DualSense rejection. The modified full bbport executable also compiles and links.

No gameplay or controller hardware validation is claimed. Check USB first, then Bluetooth, rumble coexistence, reconnect, overlay capture, unload, normal exit and DualSense Edge. If rumble clears trigger effects, add the conditional refresh mitigation from specification section 8.4 after reproducing it. A hard process kill or unrecoverable process crash cannot guarantee controller cleanup through atexit.

Phase 2 is not implemented: stable HP, stamina, bullets, weapon/form and in-game-menu pointer chains need discovery and restart validation with Inspector. In-game menus are not detected in phase 1; disable effects using the panel as needed. The panel shows requested bytes and conflict/error status, not hardware acknowledgements, owner names or measured sends per second. Preset tests cover resistance and buzz; richer community modes and a fully scripted charge-preview sequence remain follow-up work.

The original specification remains the design record. Applied source is authoritative; the older `integration/bbport-modhost.patch` has not been regenerated with these additions.

Broader framework regressions on 2026-10-18: all original 12 CTest cases passed across initial runs and reruns. `host` and `inline_hooks_os` exceeded an initial 30-second timeout, then passed with 180 seconds allowed. After periodic discovery was added, host, pad-effects and discovery regressions also passed.
