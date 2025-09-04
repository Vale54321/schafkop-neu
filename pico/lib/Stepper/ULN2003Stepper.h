#pragma once

#include <Arduino.h>
#include "Stepper.h"
#include <array>

class ULN2003Stepper : public Stepper {
private:
    std::array<uint8_t, 4> pins;

    static constexpr int steps_per_seq = 8;
    const uint8_t sequence[8][4] = {
        {1, 0, 0, 0},
        {1, 1, 0, 0},
        {0, 1, 0, 0},
        {0, 1, 1, 0},
        {0, 0, 1, 0},
        {0, 0, 1, 1},
        {0, 0, 0, 1},
        {1, 0, 0, 1}
    };

    int step_delay_us = 5000;
    int current_seq_idx = 0;

    void step(int steps, bool direction) override {
        if (steps <= 0) return;

        for (int i = 0; i < steps; ++i) {
            // calculate next step index
            current_seq_idx = direction
                ? (current_seq_idx + 1) % steps_per_seq
                : (current_seq_idx - 1 + steps_per_seq) % steps_per_seq;

            // set pins according to the current sequence
            digitalWrite(pins[0], sequence[current_seq_idx][0]);
            digitalWrite(pins[1], sequence[current_seq_idx][1]);
            digitalWrite(pins[2], sequence[current_seq_idx][2]);
            digitalWrite(pins[3], sequence[current_seq_idx][3]);

            // wait for the specified delay
            delayMicroseconds(step_delay_us);
        }
    }

public:
    ULN2003Stepper(std::array<uint8_t, 4> pins, int rev_steps) : pins(pins) {
        steps_per_rev = rev_steps;

        for (auto pin : pins) {
            pinMode(pin, OUTPUT);
        }
    }

    void setStepDelay(int delay_us) { step_delay_us = delay_us; }

    void resetPhase() { current_seq_idx = 0; }
};
