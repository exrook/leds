#include <FastLED.h>

// Which pin on the Arduino is connected to the NeoPixels?
// On a Trinket or Gemma we suggest changing this to 1
#define PIN            6

// How many NeoPixels are attached to the Arduino?
#define NUMPIXELS      427

CRGB leds[NUMPIXELS*3 + 10];
//Adafruit_NeoPixel pixels = Adafruit_NeoPixel(300, PIN, NEO_GRB + NEO_KHZ800);

CLEDController *controller;


void setup() {
  pinMode(13, OUTPUT);
  //Serial.begin(230400);
  Serial.begin(1000000);
  Serial.setTimeout(500); // Can probably lower this
  controller = &FastLED.addLeds<NEOPIXEL, PIN>(leds, NUMPIXELS);
  FastLED.setMaxPowerInMilliWatts(90000);
}

int count = 0;

#define DATA_LEN 10
byte red = 0, green = 0, blue = 0, param;
byte red1 = 0, green1 = 0, blue1 = 0, param1;
//enum effect_t {
//  Change = -1,
//  Constant = 0,
//  Flash
//};
enum effect_t {
  Change = -1,
  Constant = 0,
  Flash = 1,
  SetPix = 2,
  Width = 3,
  DoubleWidth = 4,
  QuadWidth = 5,
  Edges = 6
};
effect_t neweffect = 0, effect = 0;
enum effect_aux_t {
  AuxChange = -1,
  None = 0,
  Offset = 1,
  FillLeft = 2,
  FillCenter = 3,
  FillRight = 4,
  FillEdges = 5,
  FillDouble = 6
};
effect_aux_t neweffect1 = 0, effect1 = 0;

unsigned long time_ms, last_ms;
int start, end;

bool needs_update = false;

// Protocol:
// Start communication by sending: 255 255 255 255 108 101 100 122 ("ledz")
// Then stream pixel values in the format: R G B 0
// Where R G and B are between 0 and 255
// End stream by replacing the trailing 0 with 255

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
