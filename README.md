# Nenuphar

Visualize pressed keys for Linux

> [!IMPORTANT]
> this software is alpha quality

## TODO

- add timer based additive key display
- fix meta keys display

## How it works

It opens all keyboard event devices and listens for key press, then display key
in a gtk window

## Install

### from source

```shell
git clone https://github.com/tomaszkubacki/nenuphar.git
cd nenuphar
cargo build
```

## how to find my keyboard event input device

Display all input devices

```shell
cat /proc/bus/input/devices
```

All devices with kbd flag in handlers are keyboard devices.
Find event id in the "H:" row (eg. my keyboard is event4 not event0)
