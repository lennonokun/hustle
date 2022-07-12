# Hustle
![preview](data/preview.png)

## Overview
Hustle is a terminal-based wordle clone and wordle solver written in
rust, geared towards speedrunning. The solver is inspired by Alex
Selby's article [The best strategies for Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
and [code](https://github.com/alex1770/wordle), and the game is
inspired by the many wordle spin-offs like
[octordle](https://octordle.com),
[hellowordl](https://hellowordl.net), and
[speedle](https://tck.mn/speedle/).

## Installation
On Arch Linux, you can install using the PKGBUILD:
```
$ makepkg --clean PKGBUILD
# tarball may be named something else
$ sudo pacman -U hustle-1.2.4-1-x86_64.pkg.tar.zst
```
Otherwise, you can install by cloning and using the Makefile:
```
$ git clone https://github.com/lennonokun/hustle.git
$ cd hustle
$ make install
```

## Usage
Here are some examples of how to use hustle:
```
# play wordle
$ hustle play

# solve a wordle game
$ hustle solve salet.bbbbb.courd

# solve all of wordle and output to file (takes me 1m34s)
$ hustle solve --dt out

# solve and list potential answers
$ hustle solve crate.bybyb --alist

# solve and list results of top words
$ hustle solve crate.bybyb --elist

# solve with specific number of top words to try
$ hustle solve lodge.bbbbb --ntops 8

# solve using specific heuristic data
$ hustle solve salet.bbbgg --hdp hdata.csv

# solve 6 letter words with hellowordl word bank
$ hustle solve traces.bgbbyy --wbp /usr/share/hustle/bank2.csv --wlen 6

# generate heuristic data with n=100
$ hustle hgen 100 hdata.csv

# generate analysis data with n=100
$ hustle agen 100 adata.csv
```
Run hustle --help for more information.

## TODO
### General
* get rid of WBank?
* generally refactor, don't ignore warnings
* create benchmarks and unit tests
* make man page with clap
* explain scripts + dependencies in README
* look at each files TODOs
### Solver
* rename "Config"
* should hard be in config?
* add cache config to main?
* record cache stats
* standardize types for stuff like NLETS and wlen
* make heuristics work for any word bank
* maybe wbank csvs should re list answers for guess bank?
* check if solve strings are impossible? (allow impossible with --dirty)
* improve dtree pprint format
* multiple heuristic options? (linear reg, precomputed, etc)
* optimize solving
  - better lower bounds
  - re-add multithreading
    (beta pruning is mostly single-threaded though)
  - look at flame graphs, etc
### Game
* warn when impossible to guess (red text)?
* swap Game and Play?
* see how vim clears without leaving weird history
* is wrt! slow?
* show wlen in end, and maybe wbank?
* better resizing
* make prompt trait + macro like clap
* dictionary capabilities
* draw input beneath each col?
* keep statistics and track pb's
* show known letters
  - display list below each column?
  - is this cheating?
  - regardless, it should be an option
* single word
  - different layout for single
  - different modes like hard mode
- sync with wordle, duordle, quordle, octordle's, etc daily
* create github releases?
* make more easily installable
  - try to publish to AUR?
  - create packages for more distros
  - config file (colors, replacement method, etc)
