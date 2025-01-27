# univeme

*the **univ**ersal th**eme** tool*

A tool to theme many programs at once. More programs will be added over time.

## Support

- [LedFx](https://github.com/LedFx/LedFx) scenes (control your computer or even the entire house with OpenRGB, WLED, and more!)
- Firefox themes (through [pprefox](https://github.com/duckfromdiscord/pprefox))
- Windows themes, and light/dark schemes - not fully tested
- Windows cursors!
- Wallpaper engine wallpapers

## Config

Configs for univeme are written in the `.toml` format. Each module you want to use will have configuration here, but since you can technically have more than one of each module, they're defined as arrays. All that means is that you need to use `[[double brackets]]` around the name of your module when defining its use in a preset. That way, you're allowed to configure multiple at once.

It's preferred you give, at the very least, a `name` and `author` as global variables in your preset.

It's also preferred that you add a `comment` to each module with URLs for a user to download and install anything that each module needs to select its theme. For example, links to Firefox themes, or Steam Workshop links for Wallpaper Engine wallpapers.

Full preset examples, with URLs inside, are in the `example_configs` folder in this repository.

### Wallpaper engine

It's important that you define Wallpaper Engine wallpapers in terms of their monitor IDs. Also, if you do not define a wallpaper name, the wallpaper on the desktop ID you selected will be removed.

```toml
# This will apply to desktop ID 0.
[[wpeng]]
comment = "https://steamcommunity.com/sharedfiles/filedetails/?id=some_id"
desktop_id = 0
name = "name of wallpaper"

[[wpeng]]
comment = "This would remove a wallpaper from desktop ID 1."
desktop_id = 1
```

### Firefox
You must define a `theme_name`. There is no default theme in Firefox since there is light and dark mode.

```toml
[[pprefox]]
comment = "https://addons.mozilla.org/en-US/firefox/addon/..."
endpoint = "http://127.0.0.1:8080/"
theme_name = "..."
```

### Windows
Setting a cursor scheme:
```toml
[[windows]]
comment = "..."
cursor_scheme = "..."
```

### Ledfx
Not setting a `scene_name` will deactivate all scenes.
```toml
[[ledfx]]
endpoint = "http://127.0.0.1:8888/"
scene_name = "..."
```

## Linux use

It compiles on Linux, but you cannot use Wallpaper Engine, and of course not the Windows-specific settings. Firefox themes through `pprefox` won't work since you cannot install with `natemess` on Linux yet.

You will need the usual Linux packages, for Ubuntu they are installed with `sudo apt install pkg-config libssl-dev`.