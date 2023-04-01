<div align="center">
  <h1><code>clidle</code></h1>

  <p>
    <strong>An idle TUI game written in Rust!</strong>
  </p>

  <p>
    <img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
  </p>
</div>

## What's that?

This is an idle game (like cookie clicker) with a terminal UI, written in Rust.

## How-to?

To play:

- `cargo run`

Then:

- press 'c' to produce a code line,
- you can buy devs (press 'd') that will produce code lines for you,
- etc.

## TODO

- choose a license
- error management !!!! 
- save to file
- manage auto generated code line !! in another thread ?
- increase in price
- show price in help
- sell
- more means of production
- bonuses 
    - temporary random events (need to click on dashboard ?)
    - ameliorations (ex: coffee to increase dev's cps)
- achievements
- persistent display of owned items
- hints/autocompletion
- navigate in commands history
- progression bars ? https://docs.rs/indicatif/latest/indicatif/
- shinny interface ? (with macroquad ou bevy ?)
    - dash board (matrix style ?)
    - clicks instead of cli
