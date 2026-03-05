## -----------------------------------------------------------------------------
knitr::opts_chunk$set(fig.width = 6, fig.height = 4.5, 
                      message = FALSE, warning = FALSE) 
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
x_bar <- gss |>
  specify(response = hours) |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
x_bar <- gss |>
  observe(response = hours, stat = "mean")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  generate(reps = 1000) |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = x_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = x_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
t_bar <- gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  calculate(stat = "t")

## -----------------------------------------------------------------------------
t_bar <- gss |>
  observe(response = hours, null = "point", mu = 40, stat = "t")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  generate(reps = 1000) |>
  calculate(stat = "t")

## -----------------------------------------------------------------------------
null_dist_theory <- gss |>
  specify(response = hours)  |>
  assume("t")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = t_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = t_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist, method = "both") +
  shade_p_value(obs_stat = t_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = t_bar, direction = "two-sided")

## -----------------------------------------------------------------------------
gss |>
  t_test(response = hours, mu = 40)

## -----------------------------------------------------------------------------
x_tilde <- gss |>
  specify(response = age) |>
  calculate(stat = "median")

## -----------------------------------------------------------------------------
x_tilde <- gss |>
  observe(response = age, stat = "median")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(response = age) |>
  hypothesize(null = "point", med = 40) |> 
  generate(reps = 1000) |> 
  calculate(stat = "median")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = x_tilde, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = x_tilde, direction = "two-sided")

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
x_tilde <- gss_paired |>
  specify(response = diff) |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
x_tilde <- gss_paired |>
  observe(response = diff, stat = "mean")

## -----------------------------------------------------------------------------
null_dist <- gss_paired |>
  specify(response = diff) |>
  hypothesize(null = "paired independence") |> 
  generate(reps = 1000, type = "permute") |> 
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = x_tilde, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = x_tilde, direction = "two-sided")

## -----------------------------------------------------------------------------
p_hat <- gss |>
  specify(response = sex, success = "female") |>
  calculate(stat = "prop")

## -----------------------------------------------------------------------------
p_hat <- gss |>
  observe(response = sex, success = "female", stat = "prop")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(response = sex, success = "female") |>
  hypothesize(null = "point", p = .5) |>
  generate(reps = 1000) |>
  calculate(stat = "prop")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = p_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = p_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  dplyr::mutate(is_female = (sex == "female")) |>
  specify(response = is_female, success = "TRUE") |>
  hypothesize(null = "point", p = .5) |>
  generate(reps = 1000) |>
  calculate(stat = "prop")

## -----------------------------------------------------------------------------
p_hat <- gss |>
  specify(response = sex, success = "female") |>
  hypothesize(null = "point", p = .5) |>
  calculate(stat = "z")

## -----------------------------------------------------------------------------
p_hat <- gss |>
  observe(response = sex, success = "female", null = "point", p = .5, stat = "z")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(response = sex, success = "female") |>
  hypothesize(null = "point", p = .5) |>
  generate(reps = 1000, type = "draw") |>
  calculate(stat = "z")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = p_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = p_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
prop_test(gss,
          college ~ NULL,
          p = .2)

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  specify(college ~ sex, success = "no degree") |>
  calculate(stat = "diff in props", order = c("female", "male"))

## -----------------------------------------------------------------------------
d_hat <- gss |>
  observe(
    college ~ sex,
    success = "no degree",
    stat = "diff in props", order = c("female", "male")
  )

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(college ~ sex, success = "no degree") |>
  hypothesize(null = "independence") |>
  generate(reps = 1000) |>
  calculate(stat = "diff in props", order = c("female", "male"))

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
r_hat <- gss |> 
  specify(college ~ sex, success = "no degree") |>
  calculate(stat = "ratio of props", order = c("female", "male"))

## -----------------------------------------------------------------------------
r_hat <- gss |> 
  observe(college ~ sex, success = "no degree",
          stat = "ratio of props", order = c("female", "male"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(college ~ sex, success = "no degree") |>
  hypothesize(null = "independence") |> 
  generate(reps = 1000) |> 
  calculate(stat = "ratio of props", order = c("female", "male"))

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = r_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = r_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
or_hat <- gss |> 
  specify(college ~ sex, success = "no degree") |>
  calculate(stat = "odds ratio", order = c("female", "male"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(college ~ sex, success = "no degree") |>
  hypothesize(null = "independence") |> 
  generate(reps = 1000) |> 
  calculate(stat = "odds ratio", order = c("female", "male"))

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = or_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = or_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
z_hat <- gss |> 
  specify(college ~ sex, success = "no degree") |>
  hypothesize(null = "independence") |>
  calculate(stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
z_hat <- gss |> 
  observe(college ~ sex, success = "no degree",
          stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(college ~ sex, success = "no degree") |>
  hypothesize(null = "independence") |> 
  generate(reps = 1000) |> 
  calculate(stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
null_dist_theory <- gss |>
  specify(college ~ sex, success = "no degree") |>
  assume("z")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = z_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = z_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist, method = "both") +
  shade_p_value(obs_stat = z_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = z_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
prop_test(gss, 
          college ~ sex,  
          order = c("female", "male"))

## -----------------------------------------------------------------------------
Chisq_hat <- gss |>
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
Chisq_hat <- gss |>
  observe(
    response = finrela,
    null = "point",
    p = c(
      "far below average" = 1 / 6,
      "below average" = 1 / 6,
      "average" = 1 / 6,
      "above average" = 1 / 6,
      "far above average" = 1 / 6,
      "DK" = 1 / 6
    ),
    stat = "Chisq"
  )

## -----------------------------------------------------------------------------
null_dist <- gss |>
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
null_dist_theory <- gss |>
  specify(response = finrela) |>
  assume("Chisq")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist_theory, method = "both") +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = Chisq_hat, direction = "greater")

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

## -----------------------------------------------------------------------------
Chisq_hat <- gss |>
  specify(formula = finrela ~ sex) |> 
  hypothesize(null = "independence") |>
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
Chisq_hat <- gss |>
  observe(formula = finrela ~ sex, stat = "Chisq")

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(finrela ~ sex) |>
  hypothesize(null = "independence") |> 
  generate(reps = 1000, type = "permute") |> 
  calculate(stat = "Chisq")

## -----------------------------------------------------------------------------
null_dist_theory <- gss |>
  specify(finrela ~ sex) |>
  assume(distribution = "Chisq")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist, method = "both") +
  shade_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = Chisq_hat, direction = "greater")

## -----------------------------------------------------------------------------
gss |>
  chisq_test(formula = finrela ~ sex)

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  specify(age ~ college) |> 
  calculate(stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  observe(age ~ college,
          stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(age ~ college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate(stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
t_hat <- gss |> 
  specify(age ~ college) |> 
  hypothesize(null = "independence") |>
  calculate(stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
t_hat <- gss |> 
  observe(age ~ college,
          stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(age ~ college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate(stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
null_dist_theory <- gss |>
  specify(age ~ college) |>
  assume("t")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = t_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = t_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
visualize(null_dist, method = "both") +
  shade_p_value(obs_stat = t_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = t_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  specify(age ~ college) |> 
  calculate(stat = "diff in medians", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  observe(age ~ college,
          stat = "diff in medians", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(age ~ college) |> # alt: response = age, explanatory = season
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  calculate(stat = "diff in medians", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = d_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
F_hat <- gss |> 
  specify(age ~ partyid) |>
  calculate(stat = "F")

## -----------------------------------------------------------------------------
F_hat <- gss |> 
  observe(age ~ partyid, stat = "F")

## -----------------------------------------------------------------------------
null_dist <- gss |>
   specify(age ~ partyid) |>
   hypothesize(null = "independence") |>
   generate(reps = 1000, type = "permute") |>
   calculate(stat = "F")

## -----------------------------------------------------------------------------
null_dist_theory <- gss |>
   specify(age ~ partyid) |>
   hypothesize(null = "independence") |>
   assume(distribution = "F")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = F_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist_theory) +
  shade_p_value(obs_stat = F_hat, direction = "greater")

## -----------------------------------------------------------------------------
visualize(null_dist, method = "both") +
  shade_p_value(obs_stat = F_hat, direction = "greater")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = F_hat, direction = "greater")

## -----------------------------------------------------------------------------
slope_hat <- gss |> 
  specify(hours ~ age) |> 
  calculate(stat = "slope")

## -----------------------------------------------------------------------------
slope_hat <- gss |> 
  observe(hours ~ age, stat = "slope")

## -----------------------------------------------------------------------------
null_dist <- gss |>
   specify(hours ~ age) |> 
   hypothesize(null = "independence") |>
   generate(reps = 1000, type = "permute") |>
   calculate(stat = "slope")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = slope_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = slope_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
correlation_hat <- gss |> 
  specify(hours ~ age) |> 
  calculate(stat = "correlation")

## -----------------------------------------------------------------------------
correlation_hat <- gss |> 
  observe(hours ~ age, stat = "correlation")

## -----------------------------------------------------------------------------
null_dist <- gss |>
   specify(hours ~ age) |> 
   hypothesize(null = "independence") |>
   generate(reps = 1000, type = "permute") |>
   calculate(stat = "correlation")

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = correlation_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = correlation_hat, direction = "two-sided")

## -----------------------------------------------------------------------------
obs_fit <- gss |>
  specify(hours ~ age + college) |>
  fit()

## -----------------------------------------------------------------------------
null_dist <- gss |>
  specify(hours ~ age + college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute") |>
  fit()

## -----------------------------------------------------------------------------
null_dist2 <- gss |>
  specify(hours ~ age + college) |>
  hypothesize(null = "independence") |>
  generate(reps = 1000, type = "permute", variables = c(age, college)) |>
  fit()

## -----------------------------------------------------------------------------
visualize(null_dist) +
  shade_p_value(obs_stat = obs_fit, direction = "two-sided")

## -----------------------------------------------------------------------------
null_dist |>
  get_p_value(obs_stat = obs_fit, direction = "two-sided")

## -----------------------------------------------------------------------------
x_bar <- gss |> 
  specify(response = hours) |>
  calculate(stat = "mean")

## -----------------------------------------------------------------------------
x_bar <- gss |> 
  observe(response = hours, stat = "mean")

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(response = hours) |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "mean")

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- get_ci(boot_dist, type = "se", point_estimate = x_bar)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
sampling_dist <- gss |>
   specify(response = hours) |>
   assume(distribution = "t")

## -----------------------------------------------------------------------------
theor_ci <- get_ci(sampling_dist, point_estimate = x_bar)

theor_ci

visualize(sampling_dist) +
  shade_confidence_interval(endpoints = theor_ci)

## -----------------------------------------------------------------------------
t_hat <- gss |> 
  specify(response = hours) |>
  hypothesize(null = "point", mu = 40) |>
  calculate(stat = "t")

## -----------------------------------------------------------------------------
t_hat <- gss |> 
  observe(response = hours,
          null = "point", mu = 40,
          stat = "t")

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(response = hours) |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "t")

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = t_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
p_hat <- gss |> 
   specify(response = sex, success = "female") |>
   calculate(stat = "prop")

## -----------------------------------------------------------------------------
p_hat <- gss |> 
   observe(response = sex, success = "female", stat = "prop")

## -----------------------------------------------------------------------------
boot_dist <- gss |>
 specify(response = sex, success = "female") |>
 generate(reps = 1000, type = "bootstrap") |>
 calculate(stat = "prop")

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = p_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
sampling_dist <- gss |>
   specify(response = sex, success = "female") |>
   assume(distribution = "z")

## -----------------------------------------------------------------------------
theor_ci <- get_ci(sampling_dist, point_estimate = p_hat)

theor_ci

visualize(sampling_dist) +
  shade_confidence_interval(endpoints = theor_ci)

## -----------------------------------------------------------------------------
d_hat <- gss |>
  specify(hours ~ college) |>
  calculate(stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
d_hat <- gss |>
  observe(hours ~ college,
          stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(hours ~ college) |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "diff in means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = d_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
sampling_dist <- gss |>
   specify(hours ~ college) |>
   assume(distribution = "t")

## -----------------------------------------------------------------------------
theor_ci <- get_ci(sampling_dist, point_estimate = d_hat)

theor_ci

visualize(sampling_dist) +
  shade_confidence_interval(endpoints = theor_ci)

## -----------------------------------------------------------------------------
d_hat <- gss |>
  specify(hours ~ college) |>
  calculate(stat = "ratio of means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
d_hat <- gss |>
  observe(hours ~ college,
          stat = "ratio of means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(hours ~ college) |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "ratio of means", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = d_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
t_hat <- gss |>
  specify(hours ~ college) |>
  calculate(stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
t_hat <- gss |>
  observe(hours ~ college,
          stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(hours ~ college) |>
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "t", order = c("degree", "no degree"))

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = t_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  specify(college ~ sex, success = "degree") |>
  calculate(stat = "diff in props", order = c("female", "male"))

## -----------------------------------------------------------------------------
d_hat <- gss |> 
  observe(college ~ sex, success = "degree",
          stat = "diff in props", order = c("female", "male"))

## -----------------------------------------------------------------------------
boot_dist <- gss |>
  specify(college ~ sex, success = "degree") |>
  generate(reps = 1000, type = "bootstrap") |> 
  calculate(stat = "diff in props", order = c("female", "male"))

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = d_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
sampling_dist <- gss |> 
  specify(college ~ sex, success = "degree") |>
   assume(distribution = "z")

## -----------------------------------------------------------------------------
theor_ci <- get_ci(sampling_dist, point_estimate = d_hat)

theor_ci

visualize(sampling_dist) +
  shade_confidence_interval(endpoints = theor_ci)

## -----------------------------------------------------------------------------
z_hat <- gss |> 
  specify(college ~ sex, success = "degree") |>
  calculate(stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
z_hat <- gss |> 
  observe(college ~ sex, success = "degree",
          stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
boot_dist <- gss |>
  specify(college ~ sex, success = "degree") |>
  generate(reps = 1000, type = "bootstrap") |> 
  calculate(stat = "z", order = c("female", "male"))

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = z_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
slope_hat <- gss |> 
  specify(hours ~ age) |>
  calculate(stat = "slope")

## -----------------------------------------------------------------------------
slope_hat <- gss |> 
  observe(hours ~ age, stat = "slope")

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(hours ~ age) |> 
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "slope")

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = slope_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
correlation_hat <- gss |> 
  specify(hours ~ age) |>
  calculate(stat = "correlation")

## -----------------------------------------------------------------------------
correlation_hat <- gss |> 
  observe(hours ~ age, stat = "correlation")

## -----------------------------------------------------------------------------
boot_dist <- gss |>
   specify(hours ~ age) |> 
   generate(reps = 1000, type = "bootstrap") |>
   calculate(stat = "correlation")

## -----------------------------------------------------------------------------
percentile_ci <- get_ci(boot_dist)

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = percentile_ci)

## -----------------------------------------------------------------------------
standard_error_ci <- boot_dist |>
  get_ci(type = "se", point_estimate = correlation_hat)

visualize(boot_dist) +
  shade_confidence_interval(endpoints = standard_error_ci)

## -----------------------------------------------------------------------------
obs_fit <- gss |>
  specify(hours ~ age + college) |>
  fit()

## -----------------------------------------------------------------------------
boot_dist <- gss |>
  specify(hours ~ age + college) |>
  generate(reps = 1000, type = "bootstrap") |>
  fit()

## -----------------------------------------------------------------------------
conf_ints <- 
  get_confidence_interval(
    boot_dist, 
    level = .95, 
    point_estimate = obs_fit
  )

## -----------------------------------------------------------------------------
visualize(boot_dist) +
  shade_confidence_interval(endpoints = conf_ints)

