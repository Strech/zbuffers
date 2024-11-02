# zbuffers

[Zellij]: https://zellij.dev/
[Zellij config]: https://zellij.dev/documentation/configuration.html
[vertico/switch-workspace-buffer]: https://github.com/minad/vertico
[session-manager]: https://zellij.dev/documentation/session-resurrection

Convenient switching between tabs with search capabilities.

A [Zellij] plugin heavily inspired by Emacs [vertico/switch-workspace-buffer] and
Zellij [session-manager] resurection tab.

![screen-recording](./misc/screen-recording.mp4)

# Installation

Download plugin binary from the [latest release]

```console
$ mkdir -p ~/.config/zellij/plugins && \
   curl -L "https://github.com/strech/zbuffers/releases/latest/download/zbuffers.wasm" \
        -o ~/.config/zellij/plugins/zbuffers.wasm
```

Add this shared key binding into `keybinds` section of your [Zellij config]

```
shared_except "locked" {
    bind "Ctrl b" {
        LaunchOrFocusPlugin "file:~/.config/zellij/plugins/zbufers.wasm" {
            floating true
        }
    }
}
```

Plugin key bindings are:

| Key                                    | Action                               |
|:---------------------------------------|:-------------------------------------|
| <kbd>Esc</kbd> or <kbd>Ctrl + c</kbd>  | Hide the plugin                      |
| <kbd>Up</kbd> or <kbd>Ctrl + p</kbd>   | Move up                              |
| <kbd>Down</kbd> or <kbd>Ctrl + n</kbd> | Move down                            |
| <kbd>(any character)</kbd>             | Start searching                      |
| <kbd>Backspace</kbd>                   | Delete character from search         |
| <kbd>Enter</kbd>                       | Switch to the selected tab           |

> [!IMPORTANT]
> Key bindings prefixed with <kbd>Ctrl +</kbd> might interfere with default key bindings

# Development

```sh
# If you don't have Zellij installed already
cargo install zellij

# Building the plugin
cargo build

# Running in Zellij
zellij --layout plugin-dev-workspace.kdl
zellij --layout plugin-dev-workspace.kdl options --theme dark
```

# Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/Strech/zbuffers/issues.
