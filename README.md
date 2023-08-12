<div align="center">

  ![sphero image](./docs/sphero.png)

  <h3>sphero-rs</h3>

  Rust Implementation of Orbotix Communication API v1.20
  <br>
  <strong>[Explore API Docs »][https://docs.gosphero.com/api/Sphero_API_1.20.pdf]</strong>
</div>

## Overview
A complete implementation of the command and control protocol for the Sphero v1.20 API, tailored for the SPRK+ robot. This library is crafted in pure Rust and boasts (TBD) compatibility with no_std environments. Additionally, client-side infrastructure is available to efficiently send and receive packets.

## Planned Features
- [ ] Complete Protocol Support: Implements the entire command and control protocol for the Sphero v1.20 API.
- [ ] SPRK+ Support: Tailored specifically for communication with the Sphero SPRK+ robot.
- [ ] Pure Rust: All implementations are in Rust, ensuring memory safety without compromising performance.
- [ ] no_std Compatibility: Suitable for constrained and embedded environments.
- [ ] Client-side Infrastructure: Ready-made tools and utilities to send and receive packets from a connected device.

## Example: LED Animation
The [example application](./examples/sprk.rs) starts by scanning for Bluetooth devices. It then looks for a device with a name containing "SK-", which identifies the Sphero SPRK+. After finding the device, it establishes a connection, wakes up the Sphero using a specific sequence of writes, and then begins animating the LED.

The animation is achieved by continuously cycling through hues in the HSV color space and converting them to RGB values, which are then sent as commands to the Sphero.