library(globals)

message("*** findGlobals() ...")

message(" ** findGlobals(..., method = 'conservative'):")

expr <- exprs$A
globals_c <- findGlobals(expr, method = "conservative")
print(globals_c)
assert_identical_sets(globals_c, c("{", "<-", "c", "d", "+"))

message(" ** findGlobals(..., method = 'liberal'):")

expr <- exprs$A
globals_l <- findGlobals(expr, method = "liberal")
print(globals_l)
assert_identical_sets(globals_l, c("{", "<-", "b", "c", "d", "+", "a", "e"))

message(" ** findGlobals(..., method = 'ordered'):")

expr <- exprs$A
globals_i <- findGlobals(expr, method = "ordered")
print(globals_i)
assert_identical_sets(globals_i, c("{", "<-", "b", "c", "d", "+", "a", "e"))


message(" ** findGlobals(..., method = 'dfs'):")
expr <- exprs$A
print(expr)
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, c("x", "y", "z"))
assert_identical_sets(globals_t, c("{", "<-", "b", "c", "d", "+", "a", "e"))


fcn <- function() {
  a <- a + 1
  a
}
print(fcn)
globals_i <- globals::findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("{", "<-", "a", "+"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "<-", "a", "+"))



fcn <- function() {
  a
  a <- a + 1
}
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("{", "a", "<-", "+"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "a", "<-", "+"))


fcn <- function(x) x <- x
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("<-"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("<-"))


fcn <- function(x) x[1] <- 0
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "[", "[<-"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("[<-"))


fcn <- function(x) a <- x$a
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "$"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "a")
assert_identical_sets(globals_t, c("<-", "$"))


fcn <- function(...) args <- list(...)
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "list"))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "args")
assert_identical_sets(globals_t, c("<-", "list"))

fcn <- function() args <- list(...)
print(fcn)
globals_i <- findGlobals(fcn)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "list", "..."))
globals_t <- findGlobals(fcn, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "args")
assert_identical_sets(globals_t, c("<-", "list", "..."))


expr <- quote({ function(x) x; x })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("{", "x"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "x"))


expr <- quote({ "x" <- 1; x })
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("{", "<-"))
globals_t <- findGlobals(expr, method = "dfs")
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "x")
print(globals_t)
assert_identical_sets(globals_t, c("{", "<-"))

x <- list()
globals <- findGlobals(x)
print(globals)
assert_identical_sets(globals, character(0L))
globals_t <- findGlobals(x, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, character(0L))


expr <- quote(list())
attr(expr, "abc") <- quote({ a })
attr(expr, "def") <- quote({ d })
globals <- findGlobals(expr)
print(globals)
assert_identical_sets(globals, c("list", "{", "a", "d"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("list", "{", "a", "d"))


globals <- findGlobals(expr, attributes = "abc")
print(globals)
assert_identical_sets(globals, c("list", "{", "a"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("list", "{", "a", "d"))


message(" ** findGlobals(..., tweak):")
tweak_another_expression <- function(expr) {
  quote({
    x <- B
    B <- 1
    y <- C
    z <- D
  })
}

expr <- exprs$A
print(expr)
globals_i <- findGlobals(expr, tweak = tweak_another_expression)
assert_identical_sets(globals_i, c("{", "<-", "B", "C", "D"))
globals_t <- findGlobals(expr, tweak = tweak_another_expression, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, c("x", "y", "z"))
assert_identical_sets(globals_t, c("{", "<-", "B", "C", "D"))


message(" ** findGlobals(..., trace = TRUE):")

expr <- exprs$A
print(expr)
globals_i <- findGlobals(expr, trace = TRUE)
print(globals_i)
assert_identical_sets(globals_i, c("{", "<-", "b", "c", "d", "+", "a", "e"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, c("x", "y", "z"))
assert_identical_sets(globals_t, c("{", "<-", "b", "c", "d", "+", "a", "e"))

message(" ** findGlobals(a <- pkg::a):")
expr <- exprs$B
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "::"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "a")
assert_identical_sets(globals_t, c("<-", "::"))

message(" ** findGlobals(a[1] <- 0) etc.:")

expr <- quote(a[1] <- 0)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "["
assert_identical_sets(setdiff(globals_i, false_globals), c("<-", "a", "[<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("a", "[<-"))


expr <- quote({ a[1] = 0 })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "["
assert_identical_sets(setdiff(globals_i, false_globals), c("{", "=", "a", "[<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "a", "[<-"))


expr <- quote(a[b <- 1] <- 0)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "["
assert_identical_sets(setdiff(globals_i, false_globals), c("<-", "a", "[<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "b")
assert_identical_sets(globals_t, c("<-", "a", "[<-"))

expr <- quote(a[b = 1] <- 0)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "["
assert_identical_sets(setdiff(globals_i, false_globals), c("<-", "a", "[<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "b")
assert_identical_sets(globals_t, c("a", "[<-"))

expr <- quote({ a[b <- 1] = 0 })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "["
assert_identical_sets(setdiff(globals_i, false_globals), c("{", "=", "a", "<-", "[<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
if (getRversion() < "4.0.0") globals_t <- setdiff(globals_t, "b")
assert_identical_sets(globals_t, c("{", "a", "<-", "[<-"))

expr <- quote(a$b <- 0)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "$"
assert_identical_sets(setdiff(globals_i, false_globals), c("<-", "a", "$<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("a", "$<-"))

expr <- quote({ a$b = 0 })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- "$"
assert_identical_sets(setdiff(globals_i, false_globals), c("{", "=", "a", "$<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(setdiff(globals_t, false_globals), c("{", "a", "$<-"))

expr <- quote(names(a) <- "A")
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("<-", "a", "names", "names<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("a", "names<-"))

expr <- quote({ names(a) = "A" })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("{", "=", "a", "names", "names<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "a", "names<-"))

## In order to handle the following case, we have to accept a few
## false positives (`[`, `[[`, `$`, `[<-`, `[[<-`)
expr <- quote(names(a)[1] <- "A")
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- c("[", "[<-")
assert_identical_sets(setdiff(globals_i, false_globals), c("<-", "a", "names", "names<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("names<-", "a", "[<-", "names"))

expr <- quote({ names(a)[1] = "A" })
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
false_globals <- c("[", "[<-")
assert_identical_sets(setdiff(globals_i, false_globals), c("{", "=", "a", "names", "names<-"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "names<-", "a", "[<-", "names"))


expr <- expression(x)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("x"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("x"))

expr <- expression(x + y)
print(expr)
globals_i <- findGlobals(expr)
print(globals_i)
assert_identical_sets(globals_i, c("+", "x", "y"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("+", "x", "y"))


# BUG: https://github.com/HenrikBengtsson/globals/issues/60
expr <- as.call(list(function(...) GLOBAL, quote(ARG)))
print(expr)
for (method in c("conservative", "liberal", "ordered", "dfs")) {
  message(sprintf("method=%s", sQuote(method)))
  globals_i <- findGlobals(expr, method = method)
  print(globals_i)
  assert_identical_sets(globals_i, c("GLOBAL", "ARG"))
}

expr <- quote({ a * b })
globals <- findGlobals(expr, trace = TRUE)
print(globals)
assert_identical_sets(globals, c("{", "*", "a", "b"))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals_t, c("{", "*", "a", "b"))

# BUG: https://github.com/HenrikBengtsson/globals/issues/93
expr <- asS3(methods::getClass("S4")@prototype, complete = FALSE)
print(expr)
globals <- findGlobals(expr, trace = TRUE)
print(globals)
assert_identical_sets(globals, character(0L))
globals_t <- findGlobals(expr, method = "dfs")
print(globals_t)
assert_identical_sets(globals, character(0L))

message("*** findGlobals() - multiple 'method's ...")

expr <- quote({ a + 1; a <- 1 })
globals <- findGlobals(expr, method = c("ordered", "dfs"))
print(globals)
assert_identical_sets(globals, c("{", "+", "a", "<-"))

expr <- quote({ for (x in NULL) NULL })
globals <- findGlobals(expr, method = c("ordered", "dfs"))
print(globals)
assert_identical_sets(globals, c("{", "for"))

expr <- quote({ for (x in NULL) x })
globals <- findGlobals(expr, method = c("ordered", "dfs"))
print(globals)
assert_identical_sets(globals, c("{", "for"))

message("*** findGlobals() - multiple 'method's ... DONE")


message("*** findGlobals() ... DONE")
