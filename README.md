# Nenuphar

Visualize pressed keys for Linux

!!!THIS PROJECT IS IN DEV mode - not ready to use!!!

## Install

### from source

```shell
git clone https://github.com/tomaszkubacki/nenuphar.git
cd nenuphar
cargo build
```

## how to find my keyboard event input device

list all input devices

```shell
cat /proc/bus/input/devices
```

All devices with kbd flag in handlers are keyboard devices. Find event id in the "H:" row (eg. my keyboard is event4 not event0)
