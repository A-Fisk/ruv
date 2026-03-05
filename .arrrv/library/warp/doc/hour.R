## ----include = FALSE----------------------------------------------------------
knitr::opts_chunk$set(
  collapse = TRUE,
  comment = "#>"
)

## ----setup--------------------------------------------------------------------
library(warp)

## -----------------------------------------------------------------------------
before_dst <- as.POSIXct("1970-04-26 01:59:59", tz = "America/New_York")
before_dst

before_dst + 1

## -----------------------------------------------------------------------------
x <- as.POSIXct("1970-04-26 00:00:00", tz = "America/New_York") + 3600 * 0:7

data.frame(
  x = x,
  hour = warp_distance(x, "hour", every = 2)
)

## -----------------------------------------------------------------------------
y <- as.POSIXct("1970-04-26 22:00:00", tz = "America/New_York") + 3600 * 0:5

data.frame(
  y = y,
  hour = warp_distance(y, "hour", every = 2)
)

## -----------------------------------------------------------------------------
# Or call `lubridate::force_tz(x, "UTC")`
force_utc <- function(x) {
  x_lt <- as.POSIXlt(x)
  x_lt <- unclass(x_lt)
  
  attributes(x) <- NULL
  
  out <- x + x_lt$gmtoff
  
  as.POSIXct(out, tz = "UTC", origin = "1970-01-01")
}

x_utc <- force_utc(x)
y_utc <- force_utc(y)

x_utc

## -----------------------------------------------------------------------------
data.frame(
  x_utc = x_utc,
  hour = warp_distance(x_utc, "hour", every = 2)
)

data.frame(
  y_utc = y_utc,
  hour = warp_distance(y_utc, "hour", every = 2)
)

## -----------------------------------------------------------------------------
before_fallback <- as.POSIXct("1970-10-25 01:00:00", tz = "America/New_York")
before_fallback

# add 1 hour of seconds
before_fallback + 3600

## -----------------------------------------------------------------------------
x <- as.POSIXct("1970-10-25 00:00:00", tz = "America/New_York") + 3600 * 0:7
x

data.frame(
  x = x,
  hour = warp_distance(x, "hour", every = 2)
)

## -----------------------------------------------------------------------------
y <- as.POSIXct("1970-10-25 22:00:00", tz = "America/New_York") + 3600 * 0:5
y

data.frame(
  y = y,
  hour = warp_distance(y, "hour", every = 2)
)

## -----------------------------------------------------------------------------
x_utc <- force_utc(x)
x_utc

## -----------------------------------------------------------------------------
data.frame(
  x_utc = x_utc,
  hour = warp_distance(x_utc, "hour", every = 2)
)

