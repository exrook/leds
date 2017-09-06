#include <FastLED.h>

// Which pin on the Arduino is connected to the NeoPixels?
// On a Trinket or Gemma we suggest changing this to 1
#define PIN            2

// How many NeoPixels are attached to the Arduino?
#define NUMPIXELS      427

CRGB leds[NUMPIXELS*3 + 10];

CLEDController *controller;


void setup() {
  pinMode(13, OUTPUT);
  //Serial.begin(230400);
  Serial.begin(1000000);
  Serial.setTimeout(500); // Can probably lower this
  controller = &FastLED.addLeds<NEOPIXEL, PIN>(leds, NUMPIXELS);
  //FastLED.setMaxPowerInMilliWatts(90000);
}

int count = 0;

#define DATA_LEN 10

unsigned long time_ms, last_ms;
int start, end;

bool needs_update = false;

void loop() {
  time_ms = millis();
  byte data[DATA_LEN];
  int count = 0;
  CRGB* cur_buf = &leds[0];
  while (true) {
    count += Serial.readBytes((char*)cur_buf, NUMPIXELS*3 - count);
    if (count >= NUMPIXELS*3) {
      count -= NUMPIXELS*3;
      needs_update = true;
    }
    
    if (needs_update) {
      FastLED.show();
      needs_update = false;
    }
  }
}
