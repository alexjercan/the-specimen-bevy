# Settings

The menu settings panel writes `GameSettings` to the user config directory. Existing settings files remain valid: new fields use defaults, and old Low/Medium/High presets retain their original MSAA when `msaa` is absent. Out-of-range sensitivity, FOV, UI scale, and volume values are sanitized on load.

| Group | Setting | Effect |
| --- | --- | --- |
| Audio | Master, SFX, Music | Master multiplies all buses. SFX affects world and UI. Music affects the looping menu track independently. |
| Controls | Mouse sensitivity, Invert Y | Mouse look uses the selected sensitivity and pitch direction. |
| Graphics | Low, Medium, High, Custom | Presets choose MSAA off, 2x, and 4x. Custom allows MSAA off, 2x, 4x, or 8x independently; changing MSAA selects Custom. |
| Graphics | Field of view | Perspective player-camera vertical FOV, 50-110 degrees. |
| Graphics | UI scale | Interface scale, 75-150%. |
| Graphics | Reduce flicker | Keeps flickering and pulsing facility and menu lights steady, steadies the loading label, and disables the white flashbang screen overlay. Gameplay exposure remains unchanged. |
| Graphics | Display mode, VSync, FPS cap | Display/window presentation controls. The FPS cap limits rendered frame pacing; it does not change the fixed simulation step or recorded video FPS. VSync may impose a lower limit. On the web, the browser controls frame scheduling and the native FPS cap is unavailable. |
| Graphics | FPS overlay | Toggles the frame counter. |

Camera quality applies to both active and newly spawned 2D and 3D cameras so stacked cameras use the same MSAA. FOV applies to active and newly spawned player cameras. Runtime-only controls do not add render or audio systems to the headless frontend. No brightness or captions setting exists.
