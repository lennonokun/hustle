import click
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

@click.command()
@click.argument("f_ldata")
@click.argument("step", type=int) # for now dont want to parse from comments
@click.argument("f_out")
@click.argument("f_plot")
def cli(f_ldata, step, f_out, f_plot):
  df = pd.DataFrame(pd.read_csv(f_ldata, comment="#"))
  df = df.sort_values("alen").reset_index(drop=True)

  # process
  a = df.loc[0, "alen"]
  b = df.loc[len(df)-1, "alen"]
  alens = np.arange(a, b+1)
  lbs = df.lb.to_numpy()
  if step > 1:
    lbs = lbs.repeat(step)[:-(step-1)]
  
  df_out = pd.DataFrame({"alen": alens, "lb": lbs})
  df_out.to_csv(f_out, index=False)

  # plot
  fig, ax = plt.subplots()
  fig.suptitle("Number of Answers vs Lower Bound")
  ax.set_xlabel("alen")
  ax.set_ylabel("lb")

  ax.plot(df.alen, df.lb)
  fig.savefig(f_plot, dpi=400)

if __name__ == "__main__":
  cli()
