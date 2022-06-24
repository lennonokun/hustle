# Hustle

![preview](data/preview.png)

## Overview
Hustle is a terminal-based wordle clone and wordle solver written in
rust, and geared towards speedrunning. The solver is inspired by Alex
Selby's article [The best strategies for
Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
and [code](https://github.com/alex1770/wordle), and the game is
inspired by the many wordle spin-offs like
[octordle](https://octordle.com),
[hellowordl](https://hellowordl.net), and
[speedle](https://tck.mn/speedle/).

## Running
Either build the crate with `cargo build --release`, which should
create an executable `hustle` in `target/release`. You can then run
the binary like in the following examples:

```
# play wordle
target/release/hustle play

# solve a wordle game
target/release/hustle solve salet.bbbbb.courd
target/release/hustle solve salet.bbbgg

# solve and output decision tree to file
target/release/hustle solve salet.bbbgg --dt out

# solve using specific heuristic data
target/release/hustle solve salet.bbbgg --hdp-in myhdata.csv

# solve 6 letter words with hellowordl word bank
target/release/hustle solve salet.bbbgg --gwp data/guess_words2 --awp data/answer_words2

# run expensive heuristic data generation
target/release/hustle gen

# run expensive heuristic data generation and specify output
target/release/hustle gen --hdp-out1 data1.csv --hdp-out2 data2.csv
```

## TODO
* change name
* remove python dependency, do isotonic regression in rust
* make solving constants a cmd option
* make heuristics work for any word bank
* combine word banks into one file
* generally refactor, don't ignore warnings
* combine answer + guess word bank files? (and maybe struct)
* optimize solving
* create benchmarks and unit tests
* dictionary capabilities
* keep statistics and track pb's
* show untested letters?
* show known letters
  - display list below each column?
  - is this cheating?
  - regardless, it should be an option
* single word
  - different layout for single
  - different modes like hard mode
- sync with wordle, duordle, quordle, octordle's, etc daily
* make easily installable
  - first publish crate
  - PKGBUILD, try to publish to AUR?
  - config file (colors, replacement method, etc)
