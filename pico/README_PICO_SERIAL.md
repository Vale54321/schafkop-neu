# Pico Serial Motor Control Integration

This project enables a Raspberry Pi Pico to be controlled via serial commands from a TypeScript backend. The Pico firmware parses commands received over serial and controls a stepper motor accordingly, while also sending log/status messages back.

## Serial Commands Supported
- `STEP <steps> <direction>`: Move the motor by the specified number of steps. Direction: 1 for forward, 0 for reverse.
- `SPEED <delay_us>`: Set the motor speed (delay in microseconds between steps).
- `LOG <message>`: Print a log message to serial.

## Firmware Files
- `src/main.cpp`: Main firmware logic, serial command parsing, motor control, logging.
- `src/ULN2003Stepper.h`, `src/Stepper.h`: Stepper motor driver classes.

## How to Extend
- Add new serial commands by updating the parser in `main.cpp`.
- Implement additional motor features or logging as needed.
- Ensure any new global variables are only defined in one source file, and declared as `extern` elsewhere.

## TypeScript Backend
- Use a library like `serialport` to communicate with the Pico.
- Send commands as plain text terminated by `\n`.
- Read and process log/status messages from serial.

## Example Serial Session
```
SPEED 3000
STEP 4096 1
LOG Motor moved
```

## Build & Upload
Use PlatformIO to build and upload the firmware to the Pico.

---
This README is intended for another developer or Copilot to implement further changes or features.
