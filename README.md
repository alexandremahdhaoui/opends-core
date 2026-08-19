# opends-core

The domain logic for OpenDS. Pure Rust. No I/O.

Part of the OpenDS project. OpenDS is a Rust tool for Windows. It reads a Sony pad. It maps the pad to keyboard and mouse. It presents the pad to games as an Xbox pad. It is independent from DS4Windows.

## What lives here

- Report decode. Turns a raw DualSense or DualShock 4 HID report into pad state. Covers USB, Bluetooth Basic, and Bluetooth Full.
- Mapping. Turns pad state into key presses, mouse moves, macros, turbo, and shift layers.
- Virtual pad packing. Turns pad state into the Xbox report the virtual driver sends to XInput.

No file access. No network. No Windows API calls. Every byte offset used to decode a report is checked against a real captured report, not just a guess.

## Build and test

This repo builds and tests on Linux with no Windows machine and no hardware needed.

```sh
forge build
forge test-all
```

`forge test-all` is the gate. A purity stage fails the build if any I/O call shows up in this crate.

## License

Apache License 2.0.
