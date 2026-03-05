## -----------------------------------------------------------------------------
knitr::opts_chunk$set(
  message = FALSE,
  digits = 3,
  collapse = TRUE,
  comment = "#>"
  )
options(digits = 3)
library(dials)
library(rpart)

## -----------------------------------------------------------------------------
library(dials)
cost_complexity()

## -----------------------------------------------------------------------------
cost_complexity() |> range_get()
cost_complexity() |> range_set(c(-5, 1))

# Or using the `range` argument
# during creation
cost_complexity(range = c(-5, 1))

## -----------------------------------------------------------------------------
# Natural units:
cost_complexity() |> value_seq(n = 4)

# Stay in the transformed space:
cost_complexity() |> value_seq(n = 4, original = FALSE)

## -----------------------------------------------------------------------------
set.seed(5473)
cost_complexity() |> value_sample(n = 4)

## -----------------------------------------------------------------------------
try({
library(rpart)
cart_mod <- rpart(mpg ~ ., data = mtcars, control = rpart.control(cp = 0.000001))
cart_mod$cptable
cp_vals <- cart_mod$cptable[, "CP"]

# We should only keep values associated with at least one split:
cp_vals <- cp_vals[ cart_mod$cptable[, "nsplit"] > 0 ]

# Here the specific Cp values, on their natural scale, are added:
mtcars_cp <- cost_complexity() |> value_set(cp_vals)
})

## -----------------------------------------------------------------------------
mtcars_cp <- cost_complexity() |> value_set(log10(cp_vals))
mtcars_cp

## -----------------------------------------------------------------------------
mtcars_cp |> value_seq(2)
# Sampling specific values is done with replacement
mtcars_cp |> 
  value_sample(20) |> 
  table()

## -----------------------------------------------------------------------------
trans_raise <- scales::trans_new(
  "raise", 
  transform = function(x) 2^x , 
  inverse = function(x) -log2(x)
)
custom_cost <- cost(range = c(1, 10), trans = trans_raise)
custom_cost

## -----------------------------------------------------------------------------
-log2(c(10, 1))
value_sample(custom_cost, 100) |> range()

## -----------------------------------------------------------------------------
weight_func()

## -----------------------------------------------------------------------------
# redefine values
weight_func() |> value_set(c("rectangular", "triangular"))
weight_func() |> value_sample(3)

# the sequence is returned in the order of the levels
weight_func() |> value_seq(3)

## -----------------------------------------------------------------------------
mtry()
sample_size()
num_terms()
num_comp()
# and so on

## -----------------------------------------------------------------------------
finalize(mtry(), x = mtcars[, -1])

## -----------------------------------------------------------------------------
glmnet_set <- parameters(list(lambda = penalty(), alpha = mixture()))
glmnet_set

# can be updated too
update(glmnet_set, alpha = mixture(c(.3, .6)))

## -----------------------------------------------------------------------------
grid_regular(
  mixture(),
  penalty(),
  levels = 3 # or c(3, 4), etc
)

## -----------------------------------------------------------------------------
set.seed(1041)
grid_random(
  mixture(),
  penalty(),
  size = 6 
)

