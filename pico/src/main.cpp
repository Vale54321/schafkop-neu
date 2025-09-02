#include <Arduino.h>
#include "Stepper.h"
#include "ULN2003Stepper.h"

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

    int revSteps = driver1.get_steps_per_rev();
    Serial.print("EVENT:START STEPS_PER_REV ");
    Serial.println(revSteps);

    waitForHost(1500);
    logStartup();
  }

  void loop() {
    pollSerial();
  }

private:
  String inputBuffer;

  enum class CmdType { PING, STEP, SPEED, UNKNOWN };

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

  void logStartup(){
    Serial.println(F("HELLO START"));
    for(int i=0;i<3;i++) blink(60,100);
  }

  void pollSerial(){
    while(Serial.available()){
      char c = Serial.read();
      if(c=='\r') continue; // ignore CR
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
    CmdType type = classify(cmdToken);
    handleCommand(type, cmdToken, rest);
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

  CmdType classify(String token){
    token.toUpperCase();
    if(token==F("PING")) return CmdType::PING;
    if(token==F("STEP")) return CmdType::STEP;
    if(token==F("SPEED")) return CmdType::SPEED;
    return CmdType::UNKNOWN;
  }

  void handleCommand(CmdType type, const String &token, const String &args){
    switch(type){
      case CmdType::PING: {
        Serial.println(F("PONG"));
        blink();
        break; }
      case CmdType::STEP: {
        // Expected: STEP <steps> <dir>
        int steps = -1; int dir = -1;
        if(parseStepArgs(args, steps, dir)) {
          Serial.print(F("STEP: moving ")); Serial.print(steps); Serial.print(F(" dir=")); Serial.println(dir);
          driver1.step(steps, dir!=0);
          blink(60);
        } else {
          Serial.println(F("ERR:STEP usage STEP <steps> <0|1>"));
          blink(20);
        }
        break; }
      case CmdType::SPEED: {
        int delayUs = args.toInt();
        if(delayUs > 0) {
          driver1.setSpeed(delayUs);
          Serial.print(F("SPEED: set delay_us=")); Serial.println(delayUs);
          blink();
        } else {
          Serial.println(F("ERR:SPEED usage SPEED <positive_delay_us>"));
          blink(20);
        }
        break; }
      case CmdType::UNKNOWN: {
        Serial.print(F("ERR:UNKNOWN COMMAND '")); Serial.print(token); Serial.println("'");
        blink(20);
        break; }
      default: {
        Serial.println(F("ERR:UNHANDLED"));
        blink(20);
        break; }
    }
  }
  // Helpers for parsing arguments
  bool parseStepArgs(const String &args, int &steps, int &dir){
    int sp = args.indexOf(' ');
    if(sp == -1) return false;
    String a = args.substring(0, sp); a.trim();
    String b = args.substring(sp+1); b.trim();
    if(!a.length() || !b.length()) return false;
    steps = a.toInt(); dir = b.toInt();
    if(steps <= 0) return false;
    if(!(dir==0 || dir==1)) return false;
    return true;
  }
};

SchafkopfSerialApp app;

void setup(){ app.begin(); }
void loop(){ app.loop(); }
