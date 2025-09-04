#pragma once
class Stepper {
protected:
    int steps_per_rev;
    virtual void step(int steps, bool direction) = 0;
    
public:
    void step_rev(double revs, bool direction){
        if(revs == 0.0) return;
        if(revs < 0.0){
            direction = !direction;
            revs = -revs;
        }

        long total_steps = (long)(revs * steps_per_rev + 0.5);
        if(total_steps <= 0) return;
        
        step((int)total_steps, direction);
    }

    virtual ~Stepper() {}
};
