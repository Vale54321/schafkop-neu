#include <Arduino.h>
#include "Stepper.h"
#include "ULN2003Stepper.h"


ULN2003Stepper driver1({18, 19, 20, 21}, 4096);
int revSteps = 0;

void setup() {
  Serial.begin(115200);
  revSteps = driver1.get_steps_per_rev();
  Serial.print("Steps: ");
  Serial.println(revSteps);
}

void loop() {
  static String input = "";
  while (Serial.available()) {
    char c = Serial.read();
    if (c == '\n' || c == '\r') {
      if (input.length() > 0) {
        Serial.print("Received: ");
        Serial.println(input);
        // Parse command
        if (input.startsWith("STEP ")) {
          int idx1 = input.indexOf(' ');
          int idx2 = input.indexOf(' ', idx1 + 1);
          int steps = input.substring(idx1 + 1, idx2).toInt();
          int dir = input.substring(idx2 + 1).toInt();
          driver1.step(steps, dir != 0);
          Serial.print("Motor moved ");
          Serial.print(steps);
          Serial.print(" steps in direction ");
          Serial.println(dir);
        } else if (input.startsWith("SPEED ")) {
          int delay_us = input.substring(6).toInt();
          driver1.setSpeed(delay_us);
          Serial.print("Speed set to ");
          Serial.println(delay_us);
        } else if (input.startsWith("LOG ")) {
          Serial.print("LOG: ");
          Serial.println(input.substring(4));
        } else {
          Serial.println("Unknown command");
        }
        input = "";
      }
    } else {
      input += c;
    }
  }
  delay(10); // avoid busy loop
}
