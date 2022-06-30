import click
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.isotonic import IsotonicRegression

@click.command()
@click.argument("f_hdata")
@click.argument("f_happrox")
@click.argument("f_plot")
def cli(f_hdata, f_happrox, f_plot):
  # read data
  df = pd.DataFrame(pd.read_csv(f_hdata))
  df["v"] = df.h / df.n
  reg1 = IsotonicRegression()
  reg2 = IsotonicRegression()
  df["hh"] = reg1.fit_transform(df.n, df.h)
  df["vh"] = reg2.fit_transform(df.n, df.h / df.n)
  df["hr"] = df.hh - df.h
  df["vr"] = df.vh - df.v

  xs = np.arange(1, 2310)
  hs = reg1.transform(xs.reshape(-1, 1))
  vs = reg2.transform(xs.reshape(-1, 1))

  # save file
  df2 = pd.DataFrame({"n": xs, "h": hs})
  df2.to_csv(f_happrox, index=False)

  # plot
  plt.style.use("Solarize_Light2")
  fig, axs = plt.subplots(2, 2)
  fig.suptitle("Potential Answers vs Optimal Solution")
  
  axs[0,0].scatter(df.n, df.h, s=8, alpha=0.3)
  axs[0,0].plot(xs, hs, linewidth=1, c="orange")
  axs[0,0].set_ylabel("heuristic", size="large")
  axs[0,0].set_title("true")

  axs[1,0].scatter(df.n, df.h / df.n, s=8, alpha=0.3)
  axs[1,0].plot(xs, vs, linewidth=1, c="orange")
  axs[1,0].set_xlabel("number of answers")
  axs[1,0].set_ylabel("evaluation", size="large")

  g1 = sns.histplot(data=df, x="n", y="hr", ax=axs[0,1],
                    bins=50, stat="frequency",
                    cmap="crest", cbar=True)
  g1.set_title("residual")
  g1.set_xlabel("")
  g1.set_ylabel("")

  g2 = sns.histplot(data=df, x="n", y="vr", ax=axs[1,1],
                    bins=50, stat="frequency",
                    cmap="crest", cbar=True)
  g2.set_xlabel("number of answers")
  g2.set_ylabel("")

  plt.subplots_adjust(wspace=0.25, hspace=0.2)
  fig.savefig(f_plot, dpi=400)

if __name__ == "__main__":
  cli()
