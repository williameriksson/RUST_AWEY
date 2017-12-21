# Description
Three tasks program. It prints CPU statistics over ITM port 0, it takes commands over USART1 to control the blinking of the onboard LED.

Commands:
start
pause
period 1-1000 

# Build
Install required tools according to the quick start guide provided in the course. Build by `xargo build --target thumbv7m-none-eabi`.

# Run
Connect PA9 to usb <-> serial RX, Connect PA10 to usb <-> serial TX.
Connect STM32 device to the computer, connect to it with openocd by `openocd -f interface/stlink-v2-1.cfg -f target/stm32f1x.cfg`.
Flash and debug the device with `arm-none-eabi-gdb <PROJECT_PATH>/target/thumbv7m-none-eabi/debug/assignment_5a`. 
