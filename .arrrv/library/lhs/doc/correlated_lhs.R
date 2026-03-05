## ----setup, include = FALSE---------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)
source("VignetteCommonCode.R")
require(lhs)

## -----------------------------------------------------------------------------
lhs_A <- correlatedLHS(lhs::randomLHS(30, 4),
                       marginal_transform_function = function(W, ...) {
                         W[,1] <- qunif(W[,1], 2, 4)
                         W[,2] <- qnorm(W[,2], 1, 3)
                         W[,3] <- qexp(W[,3], 3)
                         W[,4] <- qlnorm(W[,4], 1, 1)
                         return(W)
                       },
                       cost_function = function(W, ...) {
                         (cor(W[,1], W[,2]) - 0.3)^2 + (cor(W[,3], W[,4]) - 0.5)^2
                       },
                       debug = FALSE, maxiter = 1000)

## -----------------------------------------------------------------------------
cor(lhs_A$transformed_lhs[,1:2])[1,2]
cor(lhs_A$transformed_lhs[,3:4])[1,2]

## -----------------------------------------------------------------------------
lhs_B <- correlatedLHS(lhs::randomLHS(30, 4),
                       marginal_transform_function = function(W, ...) {
                         W[,1] <- qbeta(W[,1], 4, 6)
                         W[,2] <- qbeta(W[,2], 3, 7)
                         W[,3] <- qbeta(W[,3], 2, 8)
                         W[,4] <- qbeta(W[,4], 1, 9)
                         return(W)
                       },
                       cost_function = function(W, ...) {
                         sum((apply(W, 1, sum) - 1)^2)
                       },
                       debug = FALSE,
                       maxiter = 1000)

## -----------------------------------------------------------------------------
range(apply(lhs_B$transformed_lhs, 1, sum)) # close to 1
apply(lhs_B$transformed_lhs, 2, mean) # close to 4/10, 3/10, 2/10, 1/10

## -----------------------------------------------------------------------------
lhs_B <- lhs::qdirichlet(lhs::randomLHS(30, 4), c(4,3,2,1))

## -----------------------------------------------------------------------------
all(abs(apply(lhs_B, 1, sum) - 1) < 1E-9) # all exactly 1
apply(lhs_B, 2, mean) # close to 4/10, 3/10, 2/10, 1/10

## -----------------------------------------------------------------------------
set.seed(3803)
N <- 100000

reject_samp <- data.frame(
  v1 = runif(N, 1, 4),
  v2 = runif(N, 1E-6, 2),
  v3 = runif(N, 2, 6),
  v4 = runif(N, 1E-6, 0.1)
)

p <- with(reject_samp, v1*v2*v3*v4)
ind <- which(p < 1 & p > 0.3)
reject_samp <- reject_samp[ind,]

## -----------------------------------------------------------------------------
lhs_C <- correlatedLHS(lhs::randomLHS(30, 4),
                       marginal_transform_function = function(W, empirical_sample, ...) {
                         res <- W
                         for (i in 1:ncol(W)) {
                           res[,i] <- quantile(empirical_sample[,i], probs = W[,i])
                         }
                         return(res)
                       },
                       cost_function = function(W, ...) {
                         p <- W[,1]*W[,2]*W[,3]*W[,4]
                         pp <- length(which(p > 0.3 & p < 1)) / nrow(W)
                         return(1-pp)
                       },
                       debug = FALSE,
                       maxiter = 10000,
                       empirical_sample = reject_samp)

