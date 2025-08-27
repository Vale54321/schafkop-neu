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
public:
    ULN2003Stepper(std::array<uint8_t, 4> pins, int rev_steps) : pins(pins) {
        steps_per_rev = rev_steps;
        for (auto pin : pins) {
            pinMode(pin, OUTPUT);
        }
    }
    void step(int steps, bool direction) override {
        for (int i = 0; i < steps; ++i) {
            int seq_idx = direction ? (i % steps_per_seq) : (steps_per_seq - 1 - (i % steps_per_seq));
            for (int j = 0; j < 4; ++j) {
                digitalWrite(pins[j], sequence[seq_idx][j]);
            }
            delayMicroseconds(step_delay_us);
        }
    }
    void setSpeed(int delay_us) {
        step_delay_us = delay_us;
    }
};
