# Settings

The normal windowed game offers Settings from the main and pause menus. The
panel has three tabs. Audio holds the Volume group (Master, SFX, Music
sliders). Controls holds the Mouse group (sensitivity slider), the Movement
group, and the Interaction group. Graphics holds the Quality group (quality
preset). Switching tabs cancels a pending binding. Each binding shows its
keyboard prompt image, or the key name when no image is available. Drag a
slider or click its track to set a value; the readout and the saved file follow
the change. Click a binding and press a new key. Escape cancels a pending
binding; Escape or Back closes the panel. Movement uses five distinct keys
(forward, left, back, right, interact). Sprint stays on Left Shift. The
supported binding choices are A-Z, arrow keys, and Space; Escape stays reserved
for Pause. A duplicate binding is ignored.

The menu writes JSON to `$XDG_CONFIG_HOME/horror-game-bevy/settings.json`, or
`$HOME/.config/horror-game-bevy/settings.json` when XDG_CONFIG_HOME is unset.
Writes use a temporary file followed by rename. Invalid or missing files fall
back to defaults; invalid volume, sensitivity, or key values are clamped or
reset. Only the normal windowed game reads and writes this file. Headless,
transport, and review examples use defaults and never change player settings.

Master and SFX multiply all currently played sounds, including UI, footsteps,
and ambience. Music has a saved independent level, but has no output until a
music bus is added. Volume changes also update active ambient loops. Mouse
sensitivity applies to gameplay look only, not the debug free camera.

Graphics presets currently change camera MSAA: Low disables it, Medium uses 2x,
High uses 4x. They do not change window or render resolution. This was an
Automode-selected first step to keep UI and window size stable, not a
user-approved resolution choice. Shadows are already disabled on the facility's
point lights, so lowering shadow resolution alone would not save work in the
current scene. A future low preset could render only the 3D view into a smaller
offscreen target and upscale it while keeping UI native resolution. Other
measured candidates are reducing the number/range of active lights, limiting
shadow casters if shadows are enabled, and mesh/material batching or LOD.
Benchmark any preset on the target GPU before claiming an FPS gain.
