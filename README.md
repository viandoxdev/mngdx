# Rust mangadex TUI

## Features

### use\_tempfiles

if your terminal doesn't support the tempfile transfer mode of the kitty image protocol (i.e. konsole), but does support direct, you should disable the `use_tempfiles` feature (i.e. by disabling default features).

### set\_padding

On kitty only, if remote control is enabled, this feature lets mngdx change the padding size of the window (see KITTY\_PADDING in `src/main.rs`) to use more of the window.
