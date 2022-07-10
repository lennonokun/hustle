# Hustle
![preview](data/preview.png)

## Overview
Hustle is a terminal-based wordle clone and wordle solver written in
rust, geared towards speedrunning. The solver is inspired by Alex
Selby's article [The best strategies for
Wordle](http://sonorouschocolate.com/notes/index.php/The_best_strategies_for_Wordle)
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
$ hustle solve reast.bbbgg

# solve all of wordle and output to file (takes me 1m40s)
$ hustle solve --dt out

# solve and list results of top words
$ hustle solve crate.bybyb --list

# solve with specific ntops and cutoff
$ hustle solve lodge.bbbbb --ntops 8 --cutoff 10

# solve using specific heuristic data
$ hustle solve salet.bbbgg --hdp myhdata.csv

# solve 6 letter words with hellowordl word bank
$ hustle solve traces.bgbbyy --wbp /usr/share/hustle/bank2.csv --wlen 6

# generate heuristic data with top 10 words
$ hustle gen 10 myhdata2.csv
```

## TODO
### General
* get rid of WBank?
* generally refactor, don't ignore warnings
* create benchmarks and unit tests
* make man page with clap
* explain scripts + dependencies in README
* look at each files TODOs
### Solver
* STATE SHOULD BE SEPARATED FROM HDATA, CFG, AND CACHE
* RENAME CONFIG TO SOMETHING ELSE
* SPLIT CONFIG INTO NEW FILE
* should hard be in config?
* add cache config to main?
* record cache stats
* standardize types for stuff like NLETS and wlen
* make heuristics work for any word bank
* check if solve strings are impossible? (allow impossible with --dirty)
* better upper bounds finding
* caching
* make command to list possible answers
* improve dtree pprint format
* make stats for time to solve also
* also try stats for different n and configs
* multiple heuristic options? (linear reg, precomputed, etc)
* optimize solving
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
* show untested letters?
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
