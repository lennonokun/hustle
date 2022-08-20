import click
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.model_selection import train_test_split
from sklearn.pipeline import make_pipeline
from sklearn.preprocessing import PolynomialFeatures
from sklearn.linear_model import Lasso

MAX_DEGREE = 4

@click.command()
@click.argument("f_gdata")
@click.argument("f_plot")
def cli(f_gdata, f_plot):
  # read data
  df = pd.DataFrame(pd.read_csv(f_gdata, comment="#"))
  df_train, df_test = train_test_split(df)

  regs = [
    make_pipeline(
      PolynomialFeatures(degree=n, include_bias=False),
      Lasso(alpha=0.5),
    ) for n in range(1, MAX_DEGREE+1)
  ]

  # plot data
  plt.style.use("Solarize_Light2")
  fig, axs = plt.subplots(MAX_DEGREE, figsize=(6,10))
  sns.scatterplot(data=df, x="alen", y="tot", s=2, ax=axs[0])
  axs[0].set_title("alen vs tot")
  axs[0].set_xlabel("alen")
  axs[0].set_ylabel("tot")

  # fit models, print info, and plot residuals
  for i, (ax, reg) in enumerate(zip(axs[1:], regs)):
    # fit model
    reg.fit(df_train[["alen"]], df_train["tot"])
    lasso = reg.named_steps.lasso
    coefs = np.concatenate(([lasso.intercept_], lasso.coef_), axis=0)
    
    # print info
    print(f"degree: {i+1}") 
    print(f"score: {reg.score(df_test[['alen']], df_test['tot'])}")
    print(f"coefs: {coefs}")

    # plot regression + residuals
    res = reg.predict(df[["alen"]]) - df["tot"]
    ax.scatter(df["alen"], res, s=2)
    ax.set_title(f"tot residuals for degree {i+1}")
    ax.set_xlabel("alen")
    ax.set_ylabel("tot residual")

  plt.subplots_adjust(hspace=0.5)
  fig.savefig(f_plot)

if __name__ == "__main__":
  cli()
