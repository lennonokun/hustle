# Hustle
![preview](extra/preview/main_preview.png)

## Overview
Hustle is a terminal-based wordle clone and wordle solver written in
rust, geared towards speedrunning. The solver is inspired by Alex
Selby's article [The best strategies for Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
and [code](https://github.com/alex1770/wordle), and the game is
inspired by the many wordle spin-offs like
[octordle](https://octordle.com),
[hellowordl](https://hellowordl.net), and
[speedle](https://tck.mn/speedle/).

## Preview
<details><summary>Menu</summary>

![menu](extra/preview/menu_preview.png)

</details>
<details><summary>Game</summary>

![game](extra/preview/game_preview.png)

</details>
<details><summary>Results</summary>

![results](extra/preview/results_preview.png)

</details>

## Installation
Hustle has three feature flags:
* `play`: makes the command `hustle play` in which you can play wordle.
* `solve`: makes the command `hustle solve`, which solves game states.
* `gen`: requires `solve` and makes the following commands:
  * `hustle hgen`: generate heuristic data
  * `hustle ggen`: generate general analysis data
  * `hustle lgen`: generate lower bounds data

You can specify which features you want by joining them with commas
(e.g. FEATURES="play,solve").

On Linux, you can install hustle with specific features by cloning and building it:
```
$ git clone https://github.com/lennonokun/hustle.git
$ cd hustle
$ make install FEATURES=<FEATURES>
```
On Arch Linux, you can install hustle only with all features using the PKGBUILD in extra:
```
$ makepkg --clean PKGBUILD
# tarball may be named something else
$ sudo pacman -U hustle-1.3.1-1-x86_64.pkg.tar.zst
```

## Usage
Refer to manpages with `man hustle`, and `man hustle <SUBCOMMAND>`.

## Configuration
Hustle can be configured with a TOML file at the following locations (with decreasing priority):

1. `$XDG_CONFIG_HOME/hustle/config.toml`
2. `$HOME/hustle/config.toml`

For the configuration options, see defaults at `/usr/share/hustle/config.toml`.

