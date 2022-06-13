import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from sklearn.isotonic import IsotonicRegression
from sklearn.svm import SVR

def main():
  df = pd.DataFrame(pd.read_csv("data/hdata.csv"))
  print(df.head(10))
  df.loc[df.index.max(), "y"] = 3.4201
  df.loc[df.index.max(), "ct"] = 1
  df = df[df.y.gt(0)]
  print(df)

  rgr = IsotonicRegression()
  rgr.fit(df[["x"]], df.y)
  xs = np.arange(0, 2316).reshape(-1, 1)
  yh = rgr.transform(xs)

  plt.scatter(df.x, df.y, label="y")
  plt.plot(xs, rgr.predict(xs), label="yh")
  plt.semilogx()
  plt.savefig("data/fig.png")

  # multiply by x bc each group should be weighted by its size
  print(rgr.predict([[0], [1], [2], [3]]))
  yh[1] = 0.0
  yh[2] = 1.5
  heuristic = yh * xs.reshape(-1)
  pd.Series(heuristic).fillna(0) \
    .to_csv("data/happrox.csv", index=False, header=False)
  
if __name__ == "__main__":
  main()
