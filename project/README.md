# RUST - Are We Embedded Yet
An in-depth course in embedded system given at Luleå university of technology.

## Project - Printing device

### Description
The project consists of making a "printing device". The device consists of a motor, a stick with LEDs mounted at the end of the stick, a hall sensor, magnets, a battery and a STM32 microcontroller. The device prints human readable text by turning LEDs on and off with precise timings. The code should is written in Rust.  
  
The desired width of the characters is pre-defined and can be changed, the device does thus take into account the current angular velocity to keep the text at constant width. 

![alt text](./device.jpg)
