#include <Arduino.h>
#include <Stepper.h>
#include <ULN2003Stepper.h>

#ifndef LED_BUILTIN
#define LED_BUILTIN 25
#endif

ULN2003Stepper driver1({18, 19, 20, 21}, 4096);

class SchafkopfSerialApp {
public:
  void begin(uint32_t baud = 115200) {
    pinMode(LED_BUILTIN, OUTPUT);
    digitalWrite(LED_BUILTIN, LOW);

    Serial.begin(baud);

    waitForHost(1500);

    // Startup indication
    for(int i=0;i<3;i++) blink(60,100);
  }

  void loop() {
    pollSerial();
  }

private:
  String inputBuffer;

  struct CommandHandler {
    const char* name;
    void (SchafkopfSerialApp::*fn)(const String &args);
  };
  static const CommandHandler commandTable[]; // defined after class

  void waitForHost(unsigned long timeoutMs){
    unsigned long start = millis();
    while(!Serial && (millis()-start) < timeoutMs) { /* wait */ }
  }

  void blink(uint16_t onMs = 40, uint16_t offMs = 0){
    digitalWrite(LED_BUILTIN, HIGH);
    delay(onMs);
    digitalWrite(LED_BUILTIN, LOW);
    if(offMs) delay(offMs);
  }

  void pollSerial(){
    while(Serial.available()){
      char c = Serial.read();
      if(c=='\r') continue;
      if(c=='\n') {
        processLine(inputBuffer);
        inputBuffer = "";
      } else if (inputBuffer.length() < 200) {
        inputBuffer += c;
      }
    }
  }

  void processLine(const String &raw){
    String line = raw;
    line.trim();

    if(!line.length()) return;

    String cmdToken = firstToken(line);
    String rest = remainingAfterFirst(line);

    cmdToken.toUpperCase();
    dispatchCommand(cmdToken, rest);
  }

  static String firstToken(const String &line){
    int sp = line.indexOf(' ');
    return (sp==-1)? line : line.substring(0, sp);
  }

  static String remainingAfterFirst(const String &line){
    int sp = line.indexOf(' ');
    if (sp==-1) return String("");
    String r = line.substring(sp+1);
    r.trim();
    return r;
  }

  void dispatchCommand(const String &uppercaseToken, const String &args){
    for(size_t i=0; commandTable[i].name != nullptr; ++i){
      if(uppercaseToken.equals(commandTable[i].name)){
        (this->*commandTable[i].fn)(args);
        return;
      }
    }
    Serial.print(F("ERR:UNKNOWN COMMAND '")); Serial.print(uppercaseToken); Serial.println("'");
    blink(20);
  }

  // ---- COMMANDS ----
  void cmdHealthcheck(const String &){
    Serial.println(F("OK"));
    blink();
  }

  void cmdStep(const String &args){
    double steps = -1; int dir = -1;
    if(parseStepArgs(args, steps, dir)) {
      Serial.print(F("STEP: moving ")); Serial.print(steps); Serial.print(F(" dir=")); Serial.println(dir);
      driver1.step_rev(steps, dir!=0);
      blink(60);
    } else {
      Serial.println(F("ERR:STEP usage STEP <revs> <0|1>"));
      blink(20);
    }
  }

  void cmdSpeed(const String &args){
    int delayUs = args.toInt();
    if(delayUs > 0) {
      driver1.setStepDelay(delayUs);
      Serial.print(F("SPEED: set delay_us=")); Serial.println(delayUs);
      blink();
    } else {
      Serial.println(F("ERR:SPEED usage SPEED <positive_delay_us>"));
      blink(20);
    }
  }

  bool parseStepArgs(const String &args, double &steps, int &dir){
    int sp = args.indexOf(' ');
    if(sp == -1) return false;
    String a = args.substring(0, sp); a.trim();
    String b = args.substring(sp+1); b.trim();
    if(!a.length() || !b.length()) return false;
    steps = a.toDouble(); dir = b.toInt();
    if(steps <= 0) return false;
    if(!(dir==0 || dir==1)) return false;
    return true;
  }
};

// ---- COMMAND TABLE ----
const SchafkopfSerialApp::CommandHandler SchafkopfSerialApp::commandTable[] = {
  {"HEALTHCHECK",  &SchafkopfSerialApp::cmdHealthcheck},
  {"STEP",  &SchafkopfSerialApp::cmdStep},
  {"SPEED", &SchafkopfSerialApp::cmdSpeed},
  {nullptr, nullptr}
};

SchafkopfSerialApp app;

void setup(){ app.begin(); }
void loop(){ app.loop(); }
