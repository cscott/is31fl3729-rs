[![Crates.io](https://img.shields.io/crates/v/is31fl3729)](https://crates.io/crates/is31fl3729)
[![docs.rs](https://img.shields.io/docsrs/is31fl3729)](https://docs.rs/is31fl3729/latest/is31fl3729/)

[![lint](https://github.com/cscott/is31fl3729-rs/actions/workflows/lint.yml/badge.svg)](https://github.com/cscott/is31fl3729-rs/actions/workflows/lint.yml)
[![build](https://github.com/cscott/is31fl3729-rs/actions/workflows/build.yml/badge.svg)](https://github.com/cscott/is31fl3729-rs/actions/workflows/build.yml)


# is31fl3729 driver

Driver for [Lumissil Microsystem's IS31FL3729 integrated circuit](https://www.lumissil.com/assets/pdf/core/IS31FL3729_DS.pdf). Some of the major features of this library are:

1. Use of embedded HAL traits (works with any embedded device that supports the required traits). This means that this driver is platform agnostic.
2. Library features (only turn on what devices you need to save compiled binary space).

## Install

To install this driver in your project add the following line to your `Cargo.toml`'s `dependencies` table:

```toml
is31fl3729 = "0.1.1"
```

By default this version will only contain the core driver.
To use a preconfigured device (currently just [the FW16 Seven Segment Display Input Module](https://community.frame.work/t/7-segment-display-input-module/50509)),
you would need to change this line to include that device:

```toml
is31fl3729 = { version = "0.1.1", features = ["sevensegment"] }
```

## Inspiration

This driver was re/written by C. Scott Ananian.

This driver is ~~ripped off~~ modified from Framework's [is31fl3741 crate](https://github.com/FrameworkComputer/is31fl3741-rs) which is itself ~~ripped off~~ modified from [gleich](https://github.com/gleich/)'s [is31fl3731 crate](https://github.com/gleich/is31fl3731).

That driver is a port of [adafruit's driver for the is31fl3731](https://github.com/adafruit/Adafruit_CircuitPython_IS31FL3731) in the Rust programming language.
