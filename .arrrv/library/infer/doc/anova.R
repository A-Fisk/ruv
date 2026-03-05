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
  ggplot2::aes(x = partyid, y = age) +
  ggplot2::geom_boxplot() +
  ggplot2::scale_fill_brewer(type = "qual") +
  ggplot2::theme(axis.text.x = ggplot2::element_text(angle = 45, 
                                                     vjust = .5)) +
    ggplot2::labs(x = "partyid: Political Party Affiliation",
                  y = "age: Age of Respondent")

## -----------------------------------------------------------------------------
# calculate the observed statistic
observed_f_statistic <- gss |>
  specify(age ~ partyid) |>
  hypothesize(null = "independence") |>
  calculate(stat = "F")

## -----------------------------------------------------------------------------
# generate the null distribution using randomization
null_dist <- gss |>
  specify(age ~ partyid) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate(stat = "F")

## -----------------------------------------------------------------------------
# visualize the null distribution and test statistic!
null_dist |>
  visualize() + 
  shade_p_value(observed_f_statistic,
                direction = "greater")

## -----------------------------------------------------------------------------
# visualize the theoretical null distribution and test statistic!
null_dist_theory <- gss |>
  specify(age ~ partyid) |>
  assume(distribution = "F")

visualize(null_dist_theory) +
  shade_p_value(observed_f_statistic,
                direction = "greater")

## -----------------------------------------------------------------------------
# visualize both null distributions and the test statistic!
null_dist |>
  visualize(method = "both") + 
  shade_p_value(observed_f_statistic,
                direction = "greater")

## -----------------------------------------------------------------------------
# calculate the p value from the observed statistic and null distribution
p_value <- null_dist |>
  get_p_value(obs_stat = observed_f_statistic,
              direction = "greater")

p_value

## -----------------------------------------------------------------------------
pf(observed_f_statistic$stat, 3, 496, lower.tail = FALSE)

