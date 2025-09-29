# Porsmo

A rust program for pomodoro, timer, stopwatch - all in one.

## Configuration

Porsmo uses the [`dirs`](https://crates.io/crates/dirs) crate to determine the correct platform-specific directory for configuration files.

- **macOS:**
  - `~/Library/Application Support/porsmo/porsmo`
- **Linux:**
  - `$XDG_CONFIG_HOME/porsmo/porsmo` (if set)
  - Otherwise: `~/.config/porsmo/porsmo`
- **Windows:**
  - `%APPDATA%\porsmo\porsmo`

If the config file is not found in the default directory, Porsmo will also check `~/.config/porsmo/porsmo` as a fallback on Unix-like systems.

### Example configuration file

Create the file and add your custom settings:

```toml
# Short break duration (e.g., 7 minutes)
short_break_duration = "7m"

# Long break duration (e.g., 20 minutes and 50 seconds)
long_break_duration = "20m50s"

# Work session duration (e.g., 30 minutes)
work_time_duration = "30m"

# Frequency of long breaks (e.g., every 3 work sessions)
long_break_frequency = 3
```
