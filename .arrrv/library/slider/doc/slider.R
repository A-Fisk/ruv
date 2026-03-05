## ----include = FALSE----------------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)

## ----setup--------------------------------------------------------------------
library(slider)
library(dplyr, warn.conflicts = FALSE)
library(lubridate, warn.conflicts = FALSE)

## -----------------------------------------------------------------------------
slide(1:4, ~.x)

## -----------------------------------------------------------------------------
slide(1:4, ~.x, .before = 2)

## -----------------------------------------------------------------------------
slide(1:4, ~.x, .before = 2, .complete = TRUE)

## -----------------------------------------------------------------------------
slide(1:4, ~.x, .before = 1, .after = 1)

## -----------------------------------------------------------------------------
slide(1:4, ~.x, .before = Inf)

## -----------------------------------------------------------------------------
sales_vec <- c(2, 4, 6, 2)

slide_dbl(sales_vec, mean, .before = 2)

## -----------------------------------------------------------------------------
index_vec <- as.Date("2019-08-29") + c(0, 1, 5, 6)
wday_vec <- as.character(wday(index_vec, label = TRUE))

company <- tibble(
  sales = sales_vec,
  index = index_vec,
  wday = wday_vec
)

company

## ----echo=FALSE---------------------------------------------------------------
mutate(
  company, 
  roll_val = slide_dbl(sales, mean, .before = 2),
  roll_day = slide_index_dbl(sales, index, mean, .before = 2)
)

## -----------------------------------------------------------------------------
wday_vec

slide(wday_vec, ~.x, .before = 2)

## -----------------------------------------------------------------------------
slide_index(wday_vec, index_vec, ~.x, .before = days(2))

## -----------------------------------------------------------------------------
mutate(
  company, 
  roll_val = slide_dbl(sales, mean, .before = 2),
  roll_day = slide_index_dbl(sales, index, mean, .before = days(2))
)

## -----------------------------------------------------------------------------
big_index_vec <- c(
  as.Date("2019-08-30") + 0:4,
  as.Date("2019-11-30") + 0:4
)

big_sales_vec <- c(2, 4, 6, 2, 8, 10, 9, 3, 5, 2)

big_company <- tibble(
  sales = big_sales_vec,
  index = big_index_vec
)

big_company

## -----------------------------------------------------------------------------
slide_period(big_company, big_company$index, "month", ~.x)

## -----------------------------------------------------------------------------
monthly_summary <- function(data) {
  summarise(data, index = max(index), sales = sum(sales))
}

slide_period_dfr(
  big_company,
  big_company$index,
  "month",
  monthly_summary
)

## -----------------------------------------------------------------------------
big_company %>%
  mutate(monthly = floor_date(index, "month")) %>%
  group_by(monthly) %>%
  summarise(sales = sum(sales))

## -----------------------------------------------------------------------------
slide_period_dfr(
  big_company,
  big_company$index,
  "month",
  monthly_summary,
  .before = 1
)

## -----------------------------------------------------------------------------
big_company %>%
  mutate(
    monthly = floor_date(index, "month"),
    sales_summary = slide_index_dbl(sales, monthly, sum, .before = months(1))
  )

