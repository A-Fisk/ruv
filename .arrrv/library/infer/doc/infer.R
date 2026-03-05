## -----------------------------------------------------------------------------
knitr::opts_chunk$set(fig.width = 6, fig.height = 4.5) 
options(digits = 4)

## -----------------------------------------------------------------------------
library(dplyr)
library(infer)

## -----------------------------------------------------------------------------
# load in the dataset
data(gss)

# take a look at its structure
dplyr::glimpse(gss)

## -----------------------------------------------------------------------------
gss |>
  specify(response = age)

## -----------------------------------------------------------------------------
gss |>
  specify(response = age) |>
  class()

## -----------------------------------------------------------------------------
# as a formula
gss |>
  specify(age ~ partyid)

# with the named arguments
gss |>
  specify(response = age, explanatory = partyid)

## -----------------------------------------------------------------------------
# specifying for inference on proportions
gss |>
  specify(response = college, success = "degree")

## -----------------------------------------------------------------------------
gss |>
  specify(college ~ partyid, success = "degree") |>
  hypothesize(null = "independence")

## -----------------------------------------------------------------------------
gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40)

## -----------------------------------------------------------------------------
set.seed(1)

gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  generate(reps = 1000, type = "bootstrap")

## -----------------------------------------------------------------------------
gss |>
  specify(partyid ~ age) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute")

## -----------------------------------------------------------------------------
gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  generate(reps = 1000, type = "bootstrap") |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
gss |>
  specify(age ~ college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate("diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
# find the point estimate
obs_mean <- gss |>
  specify(response = hours) |>
  calculate(stat = "mean")

# generate a null distribution
null_dist <- gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  generate(reps = 1000, type = "bootstrap") |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
null_dist |>
  visualize()

## -----------------------------------------------------------------------------
null_dist |>
  visualize() +
  shade_p_value(obs_stat = obs_mean, direction = "two-sided")

## -----------------------------------------------------------------------------
# get a two-tailed p-value
p_value <- null_dist |>
  get_p_value(obs_stat = obs_mean, direction = "two-sided")

p_value

## -----------------------------------------------------------------------------
# generate a distribution like the null distribution, 
# though exclude the null hypothesis from the pipeline
boot_dist <- gss |>
  specify(response = hours) |>
  generate(reps = 1000, type = "bootstrap") |>
  calculate(stat = "mean")

# start with the bootstrap distribution
ci <- boot_dist |>
  # calculate the confidence interval around the point estimate
  get_confidence_interval(
    point_estimate = obs_mean,
    # at the 95% confidence level
    level = .95,
    # using the standard error
    type = "se"
  )

ci

## -----------------------------------------------------------------------------
boot_dist |>
  visualize() +
  shade_confidence_interval(endpoints = ci)

## -----------------------------------------------------------------------------
# calculate an observed t statistic
obs_t <- gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  calculate(stat = "t")

## -----------------------------------------------------------------------------
# switch out calculate with assume to define a distribution
t_dist <- gss |>
  specify(response = hours) |>
  assume(distribution = "t")

## -----------------------------------------------------------------------------
# visualize the theoretical null distribution
visualize(t_dist) +
  shade_p_value(obs_stat = obs_t, direction = "greater")

# more exactly, calculate the p-value
get_p_value(t_dist, obs_t, "greater")

## -----------------------------------------------------------------------------
# find the theory-based confidence interval
theor_ci <- 
  get_confidence_interval(
    x = t_dist,
    level = .95,
    point_estimate = obs_mean
  )

theor_ci

## -----------------------------------------------------------------------------
# visualize the theoretical sampling distribution
visualize(t_dist) +
  shade_confidence_interval(theor_ci)

## -----------------------------------------------------------------------------
observed_fit <- gss |>
  specify(hours ~ age + college) |>
  fit()

## -----------------------------------------------------------------------------
null_fits <- gss |>
  specify(hours ~ age + college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  fit()

null_fits

## -----------------------------------------------------------------------------
get_confidence_interval(
  null_fits, 
  point_estimate = observed_fit, 
  level = .95
)

## -----------------------------------------------------------------------------
visualize(null_fits) + 
  shade_p_value(observed_fit, direction = "both")

