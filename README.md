# DiffDing

It's easy to get lost in what you're doing. Diff ding counts the changes in your repo and
reminds you to commit your changes once you exceed a certain number of inserts and deletes.

## Usage

`diffding [interval] [threshold]`

* **interval**: the number of seconds between checks
* **threshold**: the number of inserts and deletes allowed before a reminder

## Installation

`cargo install diffding`

## Coming soon

* **Snooze** - press space to suppress the dings for a bit
* **Git integration** - commit directly from diffding
* **Config file(s)** - set preferences on a per-repo or global basis
* **More bells and whistles** -- literally. different sounds.