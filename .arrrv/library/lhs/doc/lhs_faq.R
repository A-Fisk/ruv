## ----setup, include = FALSE---------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)
require(lhs)
set.seed(2893)

## ----q1-----------------------------------------------------------------------
a <- (1:10) 
b <- (20:30) 
dataGrid <- expand.grid(a, b)

## ----a1-----------------------------------------------------------------------
X <- randomLHS(22, 2) 
X[,1] <- 1 + 9*X[,1] 
X[,2] <- 20 + 10*X[,2] 

# OR 

Y <- randomLHS(22, 2) 
Y[,1] <- qunif(Y[,1], 1, 10) 
Y[,2] <- qunif(Y[,2], 20, 30) 

head(X)
head(Y)

## ----a12----------------------------------------------------------------------
X <- randomLHS(3, 2)
X[,1] <- qinteger(X[,1], 1, 10)
X[,2] <- qinteger(X[,2], 20, 30)

head(X)

## ----a21----------------------------------------------------------------------
x <- seq(0.05, 0.95, length = 10) 
y <- 1 - x 
all.equal(x + y, rep(1, length(x))) 
hist(x, main = "") 
hist(y, main = "") 

## ----a22----------------------------------------------------------------------
x <- seq(0.05, 0.95, length = 10) 
y <- runif(length(x), 0, 1 - x) 
z <- 1 - x - y 
hist(x, main = "") 
hist(y, main = "") 
hist(z, main = "") 

## ----a24, fig.width=5, fig.height=5-------------------------------------------
N <- 1000
x <- randomLHS(N, 5) 
y <- x 
y[,1:3] <- qdirichlet(x[,1:3], c(1, 1, 1))
y[,4] <- x[,4] 
y[,5] <- x[,5] 

par(mfrow = c(2,3)) 
dummy <- apply(x, 2, hist, main = "") 

par(mfrow = c(2,3)) 
dummy <- apply(y, 2, hist, main = "") 

all.equal(rowSums(y[,1:3]), rep(1, nrow(y))) 

## ----a25----------------------------------------------------------------------
par(mfrow = c(1,1)) 
pairs(x) 
pairs(y, col = "red") 

## ----qdirichlet---------------------------------------------------------------
X <- randomLHS(1000, 7) 
Y <- qdirichlet(X, rep(1,7)) 
stopifnot(all(abs(rowSums(Y) - 1) < 1E-12)) 
range(Y) 

ws <- randomLHS(1000, 7) 
wsSums <- rowSums(ws) 
wss <- ws / wsSums 
stopifnot(all(abs(rowSums(wss) - 1) < 1E-12)) 
range(wss)

## ----custom, fig.width=5, fig.height=5----------------------------------------
require(lhs) 

# functions you described 
T1 <- function(t) t*t 
WL1 <- function(T1, t) T1*t 
BE1 <- function(WL1, T1, t) WL1*T1*t 

# t is distributed according to some pdf (e.g. normal) 
# draw a lhs with 512 rows and 3 columns (one for each function) 
y <- randomLHS(512, 3) 
# transform the three columns to a normal distribution (these could be any 
# distribution) 
t <- apply(y, 2, function(columny) qnorm(columny, 2, 1)) 
# transform t using the functions provided 
result <- cbind( 
  T1(t[,1]), 
  WL1(T1(t[,2]), t[,2]), 
  BE1(WL1(T1(t[,3]), t[,3]), T1(t[,3]), t[,3]) 
) 
# check the results 
# these should be approximately uniform 
par(mfrow = c(2,2)) 
dummy <- apply(y, 2, hist, breaks = 50, main = "") 
# these should be approximately normal 
par(mfrow = c(2,2)) 
dummy <- apply(t, 2, hist, breaks = 50, main = "") 
# these should be the results of the functions 
par(mfrow = c(2,2)) 
dummy <- apply(result, 2, hist, breaks = 50, main = "") 

## ----q6, fig.height=5, fig.width=5--------------------------------------------
N <- 1000 

x <- randomLHS(N, 4) 
y <- as.data.frame(x) 
# uniform integers on 1-10 
y[,1] <- qinteger(x[,1], 1, 10)
# three colors 1,2,3 
y[,2] <- qfactor(x[,2], factor(c("R", "G", "B"))) 
# other distributions 
y[,3] <- qunif(x[,3], 5, 10) 
y[,4] <- qnorm(x[,4], 0, 2) 

table(y[,1])
table(y[,2])

hist(y[,3], main="") 
hist(y[,4], main="") 

