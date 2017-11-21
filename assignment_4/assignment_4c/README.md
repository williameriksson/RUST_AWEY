# Description
Assignment 3b to be run on STM32 microcontroller.

# Build
Install required tools according to the quick start guide provided in the course. Build by `xargo build --target thumbv7m-none-eabi`.

# Run
Connect STM32 device to the computer, connect to it with openocd by `openocd -f interface/stlink-v2.cfg -f target/stm32f1x.cfg`.
Flash and debug the device with `arm-none-eabi-gdb <PROJECT_PATH>/target/thumbv7m-none-eabi/debug/assignment_4a`.

# Comparison debug / release
Debug required 293678 cycles to complete the decoding of ABC while release only required 2451 cycles! 
