# crownlauncher

A Spotlight-style launcher for CrownOS. It opens over the desktop as a home grid of frosted-glass widgets and a search field. Plugins built for [crownos-plugins](https://github.com/crown-os/crownos-plugins) can add tiles to the grid and answers to the search.

- **Widgets:**
  - weather for two cities, from Open-Meteo;
  - an analog world clock;
  - the next three agenda entries;
  - a countdown card;
  - large app shortcuts;
  - a tray of pinned apps, where a calendar app shows today's date.
- **Search:**
  - installed apps, matched by name, generic name and keywords;
  - every crownplugin search provider, as its answers arrive.
- **Glass:** the compositor blurs and shadows each tile, and the launcher lays its own milky tint and rim on top. When the compositor cannot blur, the tiles are drawn nearly solid instead.
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

Clicking anywhere off the grid closes the launcher.

## Configure

`~/.config/crownos/launcher.ron` is created with the defaults on first run and followed live.

```ron
(
    layout: [
        (height: Short, tiles: ["weather", "shortcuts"]),
        (height: Tall,  tiles: ["clock", "events", "countdown"]),
        (height: Short, tiles: ["pinned", "search"]),
    ],
    weather: [(name: "Munich", latitude: 48.137, longitude: 11.575), (name: "Paris", latitude: 48.857, longitude: 2.352)],
    units: Celsius,
    weather_refresh: 30,
    clock: (label: "Munich", time_zone: "Europe/Berlin"),
    events: [
        (title: "Weekly Meeting", date: "2026-05-24", repeat: Weekly, time: Some((start: "17:00:00", end: "18:00:00"))),
        (title: "John's Birthday", date: "2026-06-02", repeat: Yearly),
    ],
    countdown: (title: "Till Birthday", date: "2026-08-18", repeat: Yearly),
    shortcuts: ["crownos-myphone.desktop", "crownos-messages.desktop"],
    pinned: ["org.gnome.Calendar.desktop", "org.gnome.TextEditor.desktop", "org.gnome.Nautilus.desktop"],
)
```

- **The grid** has three columns.
  - Rows are `Short` (a strip) or `Tall` (a square).
  - `weather` and `search` span two columns. Every other built-in spans one.
- **Built-in tiles** are `weather`, `shortcuts`, `clock`, `events`, `countdown`, `pinned` and `search`.
- **Plugin tiles** are written `plugin:<plugin id>/<tile id>`.
  - Tiles you don't list are packed into rows of their own, above a search field in the last row.
  - A tile listed in a row of the wrong height is left out.
- **Search:** a layout without `search` gets one in a row of its own. Results take the rows above the search field, or the rows below it when nothing is above.
- **Events** repeat `Never`, `Weekly`, `Monthly` or `Yearly`.
- **Apps** that aren't installed are skipped.

## Extend with crownplugins

A plugin declares tiles and search providers in `plugin.ron`:

```ron
contributes: (
    launcher: [(id: "system", size: Small)],          // Small, Wide, Square or Large
    search: [(id: "web", name: "Web", keyword: Some("g"))],
),
```

and answers from its script:

```lua
ui.render("system", ui.text("42%", "text-xl text-text.primary"))

return {
    on_launcher = function(shown) end,              -- the launcher opened or closed
    on_search = function(provider, query)           -- keyword already stripped
        launcher.results(provider, query, { { id = query, title = "Search the web", icon = "globe" } })
    end,
    on_activate = function(provider, id) end,        -- the user picked a hit
}
```

`crownos-plugins/examples/launcher-tools` is a complete example: a calculator, a `g <query>` web search and a system tile. `crownplugin skill` has the full authoring guide.

## Build

The CrownOS repositories are expected side by side; `Cargo.toml` patches their GitHub sources to the local checkouts.

```sh
cargo build --release
cargo test
cargo run --example preview -- out.png [wallpaper.png] [--dark] [--query fi]
```

The `preview` example renders the launcher offscreen through Impeller to a PNG, at one fixed moment. The blur is done in the app there, so screenshots and design checks need no compositor.
