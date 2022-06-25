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
Hustle can be installed using the PKGBUILD with makepkg and pacman on
Arch Linux:
```
$ makepkg --clean PKGBUILD
# tarball may be named something else
$ sudo pacman -U hustle-1.2.1-1-x86_64.pkg.tar.zst
```

## Usage
Here are some examples of how to use hustle:
```
# play wordle
$ hustle play

# solve a wordle game
$ hustle solve salet.bbbbb.courd
$ hustle solve reast.bbbgg

# solve and output decision tree to file
$ hustle solve trace.gybbb --dt out

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
* make heuristics work for any word bank
* generally refactor, don't ignore warnings
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
* make more easily installable
  - try to publish to AUR?
  - config file (colors, replacement method, etc)
