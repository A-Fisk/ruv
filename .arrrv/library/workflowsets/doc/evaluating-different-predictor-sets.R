## -----------------------------------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  eval = rlang::is_installed(c("modeldata", "recipes")),
  comment = "#>"
)
library(parsnip)
library(recipes)
library(dplyr)
library(workflowsets)
library(ggplot2)
theme_set(theme_bw() + theme(legend.position = "top"))

## -----------------------------------------------------------------------------
data(mlc_churn, package = "modeldata")
ncol(mlc_churn)

## -----------------------------------------------------------------------------
library(workflowsets)
library(parsnip)
library(rsample)
library(dplyr)
library(ggplot2)

lr_model <- logistic_reg() |> set_engine("glm")

set.seed(1)
trn_tst_split <- initial_split(mlc_churn, strata = churn)

# Resample the training set
set.seed(1)
folds <- vfold_cv(training(trn_tst_split), strata = churn)

## -----------------------------------------------------------------------------
formulas <- leave_var_out_formulas(churn ~ ., data = mlc_churn)
length(formulas)

formulas[["area_code"]]

## -----------------------------------------------------------------------------
churn_workflows <-
  workflow_set(
    preproc = formulas,
    models = list(logistic = lr_model)
  )
churn_workflows

## -----------------------------------------------------------------------------
churn_workflows <-
  churn_workflows |>
  workflow_map("fit_resamples", resamples = folds)
churn_workflows

## -----------------------------------------------------------------------------
roc_values <-
  churn_workflows |>
  collect_metrics(summarize = FALSE) |>
  filter(.metric == "roc_auc") |>
  mutate(wflow_id = gsub("_logistic", "", wflow_id))

full_model <-
  roc_values |>
  filter(wflow_id == "everything") |>
  select(full_model = .estimate, id)

differences <-
  roc_values |>
  filter(wflow_id != "everything") |>
  full_join(full_model, by = "id") |>
  mutate(performance_drop = full_model - .estimate)

summary_stats <-
  differences |>
  group_by(wflow_id) |>
  summarize(
    std_err = sd(performance_drop) / sum(!is.na(performance_drop)),
    performance_drop = mean(performance_drop),
    lower = performance_drop - qnorm(0.975) * std_err,
    upper = performance_drop + qnorm(0.975) * std_err,
    .groups = "drop"
  ) |>
  mutate(
    wflow_id = factor(wflow_id),
    wflow_id = reorder(wflow_id, performance_drop)
  )

summary_stats |> filter(lower > 0)

ggplot(summary_stats, aes(x = performance_drop, y = wflow_id)) +
  geom_point() +
  geom_errorbar(aes(xmin = lower, xmax = upper), width = .25) +
  ylab("")

