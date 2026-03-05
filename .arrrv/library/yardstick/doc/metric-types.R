## ----setup, include = FALSE---------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)

## ----warning = FALSE, message = FALSE-----------------------------------------
library(yardstick)
library(dplyr)
data("hpc_cv")

hpc_cv %>%
  group_by(Resample) %>%
  slice(1:3)

## -----------------------------------------------------------------------------
hpc_cv %>%
  filter(Resample == "Fold01") %>%
  accuracy(obs, pred)

## -----------------------------------------------------------------------------
hpc_cv %>%
  group_by(Resample) %>%
  accuracy(obs, pred)

## -----------------------------------------------------------------------------
class_metrics <- metric_set(accuracy, kap)

hpc_cv %>%
  group_by(Resample) %>%
  class_metrics(obs, estimate = pred)

## ----echo=FALSE, warning=FALSE, message=FALSE, results='asis'-----------------
library(knitr)
library(dplyr)

yardns <- asNamespace("yardstick")
fns <- lapply(names(yardns), get, envir = yardns)
names(fns) <- names(yardns)

get_metrics <- function(fns, type) {
  where <- vapply(fns, inherits, what = type, FUN.VALUE = logical(1))
  paste0("`", sort(names(fns[where])), "()`")
}

all_metrics <- bind_rows(
  tibble(type = "class", metric = get_metrics(fns, "class_metric")),
  tibble(type = "class prob", metric = get_metrics(fns, "prob_metric")),
  tibble(type = "numeric", metric = get_metrics(fns, "numeric_metric")),
  tibble(type = "dynamic survival", metric = get_metrics(fns, "dynamic_survival_metric")),
  tibble(type = "static survival", metric = get_metrics(fns, "static_survival_metric"))
)

kable(all_metrics, format = "html")

