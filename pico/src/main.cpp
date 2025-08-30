// Supports two transport modes:
// 1. USB CDC (Serial) when connected over USB (preferred)
// 2. Hardware UART0 (Serial1) on GPIO pins for headless Raspberry Pi connection
//    Wiring for UART mode (3.3V logic only):
//      Pi GPIO14 (TXD0, physical pin 8)  --> Pico GP1 (UART0 RX)
//      Pi GPIO15 (RXD0, physical pin 10) <-- Pico GP0 (UART0 TX)
//      Pi GND (any) <--------------------> Pico GND
//    Optionally power Pico via USB. If powering from Pi 3V3, DO NOT also power via USB simultaneously.

#include <Arduino.h>
#include "Stepper.h"
#include "ULN2003Stepper.h"

ULN2003Stepper driver1({18, 19, 20, 21}, 4096);
int revSteps = 0;

// Generic IO pointers (assigned to USB Serial or Serial1 at runtime)
Stream* serialIn = nullptr;  // for available()/read()
Print*  serialOut = nullptr; // for print()/println()

void setup() {
  // Start USB CDC first
  Serial.begin(115200);
  unsigned long start = millis();
  while (!Serial && (millis() - start) < 2000UL) {
    // wait up to 2 seconds for host to open USB (non-blocking fallback)
  }
  if (Serial) {
    serialIn = &Serial;
    serialOut = &Serial;
    Serial.println("INFO: Using USB CDC Serial");
  } else {
    // Fall back to UART0 on GPIO0 (TX) / GPIO1 (RX)
    Serial1.begin(115200); // default pins for UART0 in Arduino-Pico core
    serialIn = &Serial1;
    serialOut = &Serial1;
    Serial1.println("INFO: Using UART0 (Serial1) on GP0(TX)/GP1(RX)");
  }
  revSteps = driver1.get_steps_per_rev();
  serialOut->print("EVENT:START STEPS_PER_REV ");
  serialOut->println(revSteps);
}

// helper: trim leading/trailing whitespace
static String trim(const String &s) {
  int start = 0;
  int end = s.length() - 1;
  while (start <= end && isspace(s.charAt(start))) start++;
  while (end >= start && isspace(s.charAt(end))) end--;
  if (end < start) return String("");
  return s.substring(start, end + 1);
}

void loop() {
  if (!serialIn || !serialOut) return; // safety
  static String input = "";
  while (serialIn->available()) {
    char c = serialIn->read();
    if (c == '\n' || c == '\r') {
      String cmd = trim(input);
      if (cmd.length() > 0) {
        // Emit event: received
        serialOut->print("EVENT:RECEIVED ");
        serialOut->println(cmd);

        bool success = true;
        String result = "OK";

        // Parse command
        if (cmd.startsWith("STEP ")) {
          int idx1 = cmd.indexOf(' ');
          int idx2 = cmd.indexOf(' ', idx1 + 1);
          if (idx1 != -1 && idx2 != -1) {
            int steps = cmd.substring(idx1 + 1, idx2).toInt();
            int dir = cmd.substring(idx2 + 1).toInt();
            // execute
            driver1.step(steps, dir != 0);
            // report completion
            serialOut->print("EVENT:COMPLETED STEP ");
            serialOut->print(steps);
            serialOut->print(" ");
            serialOut->println(dir);
          } else {
            success = false;
            result = "ERROR: malformed STEP command";
          }
        } else if (cmd.startsWith("SPEED ")) {
          int delay_us = cmd.substring(6).toInt();
          driver1.setSpeed(delay_us);
          serialOut->print("EVENT:COMPLETED SPEED ");
          serialOut->println(delay_us);
        } else {
          success = false;
          result = "ERROR: unknown command";
          serialOut->print("EVENT:COMPLETED ");
          serialOut->println(result);
        }

        // always print a short status summary
        if (success) {
          serialOut->print("STATUS: ");
          serialOut->println("OK");
        } else {
          serialOut->print("STATUS: ");
          serialOut->println(result);
        }
      }
      input = "";
    } else {
      input += c;
    }
  }
  delay(10); // avoid busy loop
}
