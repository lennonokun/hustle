# wordlers

![preview](data/preview.png)

## Overview
wordlers is a terminal-based wordle clone and wordle solver written in
rust, geared towards speedrunning. The solver is inspired by Alex
Selby's article [The best strategies for
Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
and [code](https://github.com/alex1770/wordle), and the game is
inspired by the many wordle spin-offs like
[octordle](https://octordle.com),
[hellowordl](https://hellowordl.net), and
[speedle](https://tck.mn/speedle/).

## Running
Either build the crate with `cargo build --release`, an executable
`wordlers` in `target/release`. You can then run the binary like in
the following examples:

```
# play wordle
./target/release/wordlers play

# solve a wordle game
./target/release/wordlers solve salet.bbbbb.courd
./target/release/wordlers solve salet.bbbgg

# solve and output decision tree to file
./target/release/wordlers solve salet.bbbgg --dt out

# solve using specific heuristic data
./target/release/wordlers solve salet.bbbgg --hdp-in myhdata.csv

# solve 6 letter words with hellowordl word bank
./target/release/wordlers solve salet.bbbgg --gwp data/guess_words2 --awp data/answer_words2

# run expensive heuristic data generation
./target/release/wordlers gen

# run expensive heuristic data generation and specify output
./target/release/wordlers gen --hdp-out1 data1.csv --hdp-out2 data2.csv
```

## TODO
* change name
* don't ignore warnings
* generally refactor
* dictionary capabilities
* show untested letters?
* show known letters
  - display list below each column?
  - is this cheating?
  - regardless, it should be an option
* single word
  - different layout for single
  - different modes like hard mode
- sync with wordle, duordle, quordle, octordle's, etc daily
* optimize solving
* create benchmarks and unit tests
* make installable
  - first publish crate
  - PKGBUILD, try to publish to AUR?
  - config file (colors, replacement method, etc)
