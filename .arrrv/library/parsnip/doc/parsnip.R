## -----------------------------------------------------------------------------
knitr::opts_chunk$set(
  message = FALSE,
  digits = 3,
  collapse = TRUE,
  comment = "#>"
  )
options(digits = 3)
library(parsnip)
set.seed(368783)

## -----------------------------------------------------------------------------
library(parsnip)
rf_mod <- rand_forest(trees = 2000)

## -----------------------------------------------------------------------------
tune_mtry <- rand_forest(trees = 2000, mtry = tune())
tune_mtry

## -----------------------------------------------------------------------------
args(rand_forest)

## -----------------------------------------------------------------------------
rf_with_seed <- 
  rand_forest(trees = 2000, mtry = tune(), mode = "regression") |>
  set_engine("ranger", seed = 63233)
rf_with_seed

## -----------------------------------------------------------------------------
# rf_with_seed |>
#   set_args(mtry = 4) |>
#   set_engine("ranger") |>
#   fit(mpg ~ ., data = mtcars)

## -----------------------------------------------------------------------------
# set.seed(56982)
# rf_with_seed |>
#   set_args(mtry = 4) |>
#   set_engine("randomForest") |>
#   fit(mpg ~ ., data = mtcars)

