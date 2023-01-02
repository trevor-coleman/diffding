# DiffDing

It's easy to get lost in what you're doing. Diff ding counts the changes in your repo and
reminds you to commit your changes once you exceed a certain number of inserts and deletes.

## Installation

`cargo install diffding`

## Usage

`diffding [interval] [threshold]`

* **interval**: the number of seconds between checks (defaults to 10)
* **threshold**: the number of inserts and deletes allowed before a reminder (defaults to 100)

## Configuration

You can configure diffding by creating a `~/.config/diffding/config.toml` file in your home directory. 

Example:

```toml
# ~/.config/diffding/config.toml

sound = "14409__acclivity__chimebar-f.wav"  # name of a sound file in `~/.config/diffding`
interval = 10                               # seconds between checks
threshold = 100                             # number of inserts and deletes allowed before a reminder
snooze_length = 5                           # number of minutes to snooze for
```

### Custom Sounds

You can use any sound you like in place of the default bell

Place the file in `~/.config/diffding` and set the `sound` option in the config file to the name of the file.

#### Supported formats:

- wav
- mp3
- ogg

## Coming soon

- **Git integration** - commit directly from diffding
- **More bells and whistles** -- literally. different built-in sounds.
- **Better bell control** - adjust volume, bell frequency, etc.

## Complete

- **Config file(s)** - set preferences in `~/.config/diffding/settings.toml`
- **Snooze** - press space to suppress the dings for a bit
