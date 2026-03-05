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
set.seed(1)

gss_paired <- gss |>
   mutate(
      hours_previous = hours + 5 - rpois(nrow(gss), 4.8),
      diff = hours - hours_previous
   )

gss_paired |>
   select(hours, hours_previous, diff)

## -----------------------------------------------------------------------------
unique_diff <- unique(gss_paired$diff)
gss_paired |>
  ggplot2::ggplot() +
  ggplot2::aes(x = diff) +
  ggplot2::geom_histogram(bins = diff(range(unique_diff))) +
  ggplot2::labs(
    x = "diff: Difference in Number of Hours Worked",
    y = "Number of Responses"
  ) +
  ggplot2::scale_x_continuous(breaks = c(range(unique_diff), 0))

## -----------------------------------------------------------------------------
# calculate the observed statistic
observed_statistic <- 
   gss_paired |> 
   specify(response = diff) |> 
   calculate(stat = "mean")

## -----------------------------------------------------------------------------
# generate the null distribution
null_dist <- 
   gss_paired |> 
   specify(response = diff) |> 
   hypothesize(null = "paired independence") |>
   generate(reps = 1000, type = "permute") |>
   calculate(stat = "mean")
   
null_dist

## -----------------------------------------------------------------------------
# visualize the null distribution and test statistic
null_dist |>
  visualize() + 
  shade_p_value(observed_statistic,
                direction = "two-sided")

## -----------------------------------------------------------------------------
# calculate the p value from the test statistic and null distribution
p_value <- null_dist |>
  get_p_value(obs_stat = observed_statistic,
              direction = "two-sided")

p_value

## -----------------------------------------------------------------------------
# generate a bootstrap distribution
boot_dist <- 
   gss_paired |> 
   specify(response = diff) |> 
   hypothesize(null = "paired independence") |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "mean")
   
visualize(boot_dist)

## -----------------------------------------------------------------------------
# calculate the confidence from the bootstrap distribution
confidence_interval <- boot_dist |>
  get_confidence_interval(level = .95)

confidence_interval

## -----------------------------------------------------------------------------
boot_dist |>
  get_confidence_interval(type = "se",
                          point_estimate = observed_statistic,
                          level = .95)

