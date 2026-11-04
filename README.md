# crownlauncher

A Raycast-style launcher for CrownOS: one rounded glass window over the desktop, with a search field across the top, the list of what it found below and a footer naming what Enter does. With nothing typed it lists every app by name. Plugins built for [crownos-plugins](https://github.com/crown-os/crownos-plugins) add answers to the search. The desktop's widgets belong to crownos-home, not here.

- **Search:**
  - installed apps, matched by name, generic name and keywords;
  - every crownplugin search provider, as its answers arrive.
- **Glass:** the compositor blurs behind the window and casts its shadow, and the launcher lays its own tint and rim on top. When the compositor cannot blur, the window is drawn nearly solid instead.
- **Motion:** the window fades in quickly while its contents drop into place on a spring, and reverses smoothly if toggled mid-way. The glass itself never moves, and the window has a surface only as big as itself, so the compositor redraws just that patch and keeps up with high-refresh displays. An invisible surface under it catches clicks off the window.
- **Instant:** one resident process keeps the overlay ready. Running `crownlauncher toggle` only asks that process to show or hide.

## Use

```sh
crownlauncher          # in the session's autostart: start hidden
crownlauncher toggle   # bind to a key: toggle (starts the launcher if needed)
crownlauncher show | hide
```

Bind it in `keybinds.ron`, for example `custom_keybinds: [(keys: "Super+Space", action: "spawn crownlauncher toggle")]`.

| Key | Action |
|---|---|
| typing | search |
| `↑` `↓` `Tab` | move through the results |
| `Enter` | open the highlighted result |
| `Ctrl+Backspace` / `Ctrl+U` | delete a word / clear the query |
| `Esc` | clear the query, then close |

Clicking anywhere off the window closes the launcher.

## Extend with crownplugins

A plugin declares search providers in `plugin.ron`:

```ron
contributes: (
    search: [(id: "web", name: "Web", keyword: Some("g"))],
),
```

and answers from its script:

```lua
return {
    on_launcher = function(shown) end,              -- the launcher opened or closed
    on_search = function(provider, query)           -- keyword already stripped
        launcher.results(provider, query, { { id = query, title = "Search the web", icon = "globe" } })
    end,
    on_activate = function(provider, id) end,        -- the user picked a hit
}
```

`crownos-plugins/examples/launcher-tools` has a calculator and a `g <query>` web search. `crownplugin skill` has the full authoring guide.

## Build

The CrownOS repositories are expected side by side; `Cargo.toml` patches their GitHub sources to the local checkouts.

```sh
cargo build --release
cargo test
cargo run --example preview -- out.png [wallpaper.png] [--dark] [--query fi] [--opening 0.9 0.2]
```

The `preview` example renders the launcher offscreen through Impeller to a PNG, at one fixed moment. The blur is done in the app there, so screenshots and design checks need no compositor.
