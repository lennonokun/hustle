import click
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import string

NALPH = 26
ALPH = string.ascii_uppercase

# TODO: add bottom row for each subplot for total frequency
# TODO: resize each axes to make them have constant cell height?

def get_counts(df, wlen):
  out = np.zeros((wlen, NALPH))
  for _, row in df.iterrows():
    out[row["position"], ord(row["letter"]) - ord("A")] += 1

  return 5 * 100 * out / out.sum()

@click.command()
@click.argument("f_bank", type=click.Path(exists=True))
@click.argument("f_out", type=click.Path())
def cli(f_bank, f_out):
  # read word bank
  df = pd.DataFrame(pd.read_csv(f_bank))
  df = df[df.bank == "A"]
  df = df.drop("bank", axis=1)
  wlens = list(sorted(pd.unique(df.wlen)))
  
  # get plot information
  nwlens = len(wlens)
  (nrows, ncols) = (nwlens//2, 2) if nwlens > 1 else (1, 1)
  figsize = (2.5*nrows, 6*ncols) if nwlens > 1 else (7, 3)

  # create plot
  plt.style.use("Solarize_Light2")
  fig = plt.figure(figsize=figsize, constrained_layout=True)
  axs = fig.subplots(nrows=nrows, ncols=ncols, squeeze=False, 
      sharex=True, gridspec_kw={"height_ratios":wlens[::2]})
  fig.suptitle("Character Frequency Distribution (Percent)")

  # plot each axes
  for wlen, ax in zip(wlens, axs.reshape(-1)):
    df2 = df[df.wlen == wlen].dropna()

    # explode into letters
    df2.word = df2.word.map(list)
    df2 = df2.explode("word")
    df2 = df2.rename(columns={"word": "letter"})
    df2["position"] = df2.groupby(level=0).cumcount()
    df2 = df2.reset_index()
    # todo why index column?
    # print(df2.head(20))

    cts = get_counts(df2, wlen)

    ax.imshow(cts, aspect="auto")
    ax.grid(False)

    ax.set_title(f"{wlen = }")
    ax.set_xticks(np.arange(NALPH), labels=list(ALPH))
    ax.set_yticks(np.arange(wlen))
    ax.set_xlabel("character")
    ax.set_ylabel("position")

    for x in range(NALPH):
      for y in range(wlen):
        s = f"{cts[y,x]:.1f}"
        ax.text(x, y, s, ha="center", va="center",
            fontfamily="monospace", color="w", size="x-small")

  fig.savefig(f_out, dpi=250)

if __name__ == "__main__":
  cli()
