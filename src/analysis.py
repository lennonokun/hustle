import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from sklearn.isotonic import IsotonicRegression
from sklearn.svm import SVR

NWORDS = 2309
OPT_SCORE = 3.4201
NGUESSES = 6

def main():
  # read data and add optimal score
  df = pd.DataFrame(pd.read_csv("data/hdata.csv"))
  df = pd.DataFrame(df.dropna())
  df = pd.concat([df, pd.DataFrame(data={
    "m": [NGUESSES], "n": [NWORDS],
    "h": [OPT_SCORE], "ct": [1]
  })], ignore_index=True)

  # fit per turns left (m)
  ns = np.arange(0, NWORDS+1).reshape(-1, 1)
  for m in range(1,7):
    rgr = IsotonicRegression()
    df2 = df[df.m == m]
    rgr.fit(df2[["n"]], df2.h, sample_weight=df2.ct)

    plt.scatter(df2.n, df2.h, label="h")
    plt.plot(ns.reshape(-1), rgr.predict(ns), label="hh")
    plt.savefig(f"data/fig{m}.png")
    plt.cla()
    plt.clf()

  # fit generally
  df.h *= df.ct
  df = df.groupby(by="n").sum()
  df.h /= df.ct
  df = df.drop("m", axis=1)
  df = df.reset_index()
  rgr = IsotonicRegression()
  rgr.fit(df[["n"]], df.h, sample_weight=df.ct)
  print(f"rgr score: {rgr.score(df[['n']], df.h)}")

  # predict and plot
  df2 = pd.DataFrame(data={"n": np.arange(0, NWORDS+1)})
  hh = rgr.predict(df2[["n"]])
  plt.scatter(df.n, df.h, label="h")
  plt.plot(df2.n, hh, label="hh")
  plt.semilogx()
  plt.savefig(f"data/fig.png")
  plt.cla()
  plt.clf()
  print(hh.size)

  # use this model to approximate heuristics
  # multiply by n bc each set is weighted by its size
  heuristic = hh * np.arange(0, NWORDS+1)
  pd.Series(heuristic).fillna(0) \
     .to_csv("data/happrox.csv", index=False, header=False)

if __name__ == "__main__":
  main()
