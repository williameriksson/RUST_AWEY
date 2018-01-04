# RUST - Are We Embedded Yet
An in-depth course in embedded system given at Luleå university of technology. The course consists of lectures, 5 assignments and a small project.

## Project - Printing device
William Eriksson | wileri-2@student.ltu.se | 19921105-xxxx

### Description
The project consists of making a "printing device". The device consists of a motor, a stick with LEDs mounted at the end of the stick, a hall sensor, magnets, a battery and a STM32 microcontroller. The device shall be able to print human readable text by turning LEDs on and off with precise timings. The code should be written in Rust.

A link to a video of an equivalent project:
https://www.youtube.com/watch?v=eMfcs0iO8zE

### Grade requirements

#### Grade 3
Build the project with the required components listed in the description and implement functionality to create simple patterns with the LEDs while the stick is rotating.

#### Grade 4
Discuss and define how to implement functionality to generate human readable text that only requires a string as input. The solution shall be as hardware independent as possible so that it can be used on many targets with just small modifications or additions.

#### Grade 5
Implement the solution with the requirements listed in Grade 4.
