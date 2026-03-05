## -----------------------------------------------------------------------------
knitr::opts_chunk$set(fig.width = 6, fig.height = 4.5) 
options(digits = 4)

## -----------------------------------------------------------------------------
library(ggplot2)
library(dplyr)
library(infer)

## -----------------------------------------------------------------------------
dplyr::glimpse(gss)

## -----------------------------------------------------------------------------
gss |>
  ggplot2::ggplot() +
  ggplot2::aes(x = finrela, fill = college) +
  ggplot2::geom_bar(position = "fill") +
  ggplot2::scale_fill_brewer(type = "qual") +
  ggplot2::theme(axis.text.x = ggplot2::element_text(
    angle = 45,
    vjust = .5
  )) +
  ggplot2::labs(
    x = "finrela: Self-Identification of Income Class",
    y = "Proportion"
  )

## -----------------------------------------------------------------------------
# calculate the observed statistic
observed_indep_statistic <- gss |>
  specify(college ~ finrela) |>
  hypothesize(null = "independence") |>
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
# generate the null distribution using randomization
null_dist_sim <- gss |>
  specify(college ~ finrela) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
# generate the null distribution by theoretical approximation
null_dist_theory <- gss |>
  specify(college ~ finrela) |>
  assume(distribution = "Chisq")

## -----------------------------------------------------------------------------
# visualize the null distribution and test statistic!
null_dist_sim |>
  visualize() +
  shade_p_value(observed_indep_statistic,
    direction = "greater"
  )

## -----------------------------------------------------------------------------
# visualize the theoretical null distribution and test statistic!
gss |>
  specify(college ~ finrela) |>
  assume(distribution = "Chisq") |>
  visualize() +
  shade_p_value(observed_indep_statistic,
    direction = "greater"
  )

## -----------------------------------------------------------------------------
# visualize both null distributions and the test statistic!
null_dist_sim |>
  visualize(method = "both") +
  shade_p_value(observed_indep_statistic,
    direction = "greater"
  )

## -----------------------------------------------------------------------------
# calculate the p value from the observed statistic and null distribution
p_value_independence <- null_dist_sim |>
  get_p_value(
    obs_stat = observed_indep_statistic,
    direction = "greater"
  )

p_value_independence

## -----------------------------------------------------------------------------
pchisq(observed_indep_statistic$stat, 5, lower.tail = FALSE)

## -----------------------------------------------------------------------------
chisq_test(gss, college ~ finrela)

## -----------------------------------------------------------------------------
gss |>
  ggplot2::ggplot() +
  ggplot2::aes(x = finrela) +
  ggplot2::geom_bar() +
  ggplot2::geom_hline(yintercept = 466.3, col = "red") +
  ggplot2::labs(
    x = "finrela: Self-Identification of Income Class",
    y = "Number of Responses"
  )

## -----------------------------------------------------------------------------
# calculating the null distribution
observed_gof_statistic <- gss |>
  specify(response = finrela) |>
  hypothesize(
    null = "point",
    p = c(
      "far below average" = 1 / 6,
      "below average" = 1 / 6,
      "average" = 1 / 6,
      "above average" = 1 / 6,
      "far above average" = 1 / 6,
      "DK" = 1 / 6
    )
  ) |>
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
# generating a null distribution, assuming each income class is equally likely
null_dist_gof <- gss |>
  specify(response = finrela) |>
  hypothesize(
    null = "point",
    p = c(
      "far below average" = 1 / 6,
      "below average" = 1 / 6,
      "average" = 1 / 6,
      "above average" = 1 / 6,
      "far above average" = 1 / 6,
      "DK" = 1 / 6
    )
  ) |>
  generate(reps = 1000, type = "draw") |>
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
# visualize the null distribution and test statistic!
null_dist_gof |>
  visualize() +
  shade_p_value(observed_gof_statistic,
    direction = "greater"
  )

## -----------------------------------------------------------------------------
# calculate the p-value
p_value_gof <- null_dist_gof |>
  get_p_value(
    observed_gof_statistic,
    direction = "greater"
  )

p_value_gof

## -----------------------------------------------------------------------------
pchisq(observed_gof_statistic$stat, 5, lower.tail = FALSE)

## -----------------------------------------------------------------------------
chisq_test(
  gss,
  response = finrela,
  p = c(
    "far below average" = 1 / 6,
    "below average" = 1 / 6,
    "average" = 1 / 6,
    "above average" = 1 / 6,
    "far above average" = 1 / 6,
    "DK" = 1 / 6
  )
)

