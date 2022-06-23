# wordlers

![preview](data/preview.png)

## Overview
wordlers is a terminal-based wordle clone and wordle solver written in
rust, inspired by Alex Selby's article [The best strategies for
Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
and [code](https://github.com/alex1770/wordle), along with the many
wordle spin-offs like [octordle](https://octordle.com) and
[hellowordl](https://hellowordl.net).

## Running
Either build the crate with `cargo build --release`, an executable
`wordlers` in `target/release`. You can then run the binary like in
the following examples:

```
# run expensive heuristic data generation
./target/release/wordlers gen

# solve a wordle game
./target/release/wordlers solve salet.bbbbb.courd
./target/release/wordlers solve salet.bbbgg

# solve and output decision tree to file
./target/release/wordlers solve salet.bbbgg --dt out

# play wordle
./target/release/wordlers play
```

## TODO
* don't ignore warnings
* generally refactor
* dictionary capabilities
* show untested letters?
* syncing with wordle's daily
* make installable
  - PKGBUILD
  - config file (colors, replacement method, etc)
