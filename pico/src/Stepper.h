#pragma once
// ...existing code...
class Stepper {
protected:
    int steps_per_rev;
public:
    virtual void step(int steps, bool direction) = 0;
    void step_rev(int revs, bool direction){
        step(steps_per_rev*revs, direction);
    }
    int get_steps_per_rev(){
        return steps_per_rev;
    }
    virtual ~Stepper() {}
};
