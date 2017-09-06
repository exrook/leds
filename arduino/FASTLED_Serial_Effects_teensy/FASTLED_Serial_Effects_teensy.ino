#include <FastLED.h>

// Which pin on the Arduino is connected to the NeoPixels?
// On a Trinket or Gemma we suggest changing this to 1
#define PIN            2

// How many NeoPixels are attached to the Arduino?
#define NUMPIXELS      427

CRGB leds[NUMPIXELS + 10];
//Adafruit_NeoPixel pixels = Adafruit_NeoPixel(300, PIN, NEO_GRB + NEO_KHZ800);

void setup() {
  pinMode(13, OUTPUT);
  Serial.begin(230400);
  Serial.setTimeout(500); // Can probably lower this
  FastLED.addLeds<NEOPIXEL, PIN>(leds, NUMPIXELS);
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
effect_t neweffect = effect_t::Constant, effect = effect_t::Constant;
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
effect_aux_t neweffect1 = effect_aux_t::None, effect1 = effect_aux_t::None;

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
  char data[DATA_LEN];
  while (true) {
    if (Serial.readBytes(data, DATA_LEN) == DATA_LEN) {
      red = data[0];
      green = data[1];
      blue = data[2];
      neweffect = (effect_t)data[3];
      param = data[4];
      red1 = data[5];
      green1 = data[6];
      blue1 = data[7];
      param1 = data[9];
      neweffect1 = (effect_aux_t)data[8];
      //      Serial.write(red);
      //      Serial.write(green);
      //      Serial.write(blue);
      //      Serial.write(neweffect);
      //      Serial.write(param);
      effect = Change;
      effect1 = AuxChange;
      //leds[count] = CRGB(red,green,blue);
    }
    switch (effect) {
      case Change:
        //        Serial.write("Changing\n to");
        switch (neweffect) {
          default:
            red = 255;
            green = 255;
            blue = 255;
            effect = Constant;
          case Constant:
            //            Serial.write("Constant\n");
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            effect = neweffect;
            break;
          case Flash:
            //            Serial.write("Flash\n");
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            count = 0;
            effect = neweffect;
            break;
          case SetPix:
            count = (count + param)%NUMPIXELS;
//            for (int i = 0; i < NUMPIXELS; i++) {
//              leds[i] = CRGB(0, 0, 0);
//            }
            leds[count] = CRGB(red,green,blue);
            needs_update = true;
            effect = neweffect;
            break;
          case Width:
            start = (NUMPIXELS/2)-param;
            end = (NUMPIXELS/2)+param;
            for (int i = 0; i < NUMPIXELS; i++) {
              int r = leds[i].r-2;
              int g = leds[i].g-2;
              int b = leds[i].b-2;
              leds[i] = CRGB(0,0,0);
            }
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            effect = neweffect;
            break;
          case DoubleWidth:
//            int start2 = ((3*NUMPIXELS)/4)-param;
//            int endd2 = ((3*NUMPIXELS)/4)+param;
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(0,0,0);
            }
            start = (NUMPIXELS/4)-param;
            end = (NUMPIXELS/4)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            start = ((3*NUMPIXELS)/4)-param;
            end = ((3*NUMPIXELS)/4)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            effect = neweffect;
            break;
          case QuadWidth:
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(0,0,0);
            }
            start = (NUMPIXELS/8)-param;
            end = (NUMPIXELS/8)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            start = ((3*NUMPIXELS)/8)-param;
            end = ((3*NUMPIXELS)/8)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            start = ((5*NUMPIXELS)/8)-param;
            end = ((5*NUMPIXELS)/8)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            start = ((7*NUMPIXELS)/8)-param;
            end = ((7*NUMPIXELS)/8)+param;
            for (int i = start; i < end; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            effect = neweffect;
            break;
          case Edges:
            start = param;
            end = (NUMPIXELS)-param;
            for (int i = 0; i < NUMPIXELS; i++) {
              int r = leds[i].r-2;
              int g = leds[i].g-2;
              int b = leds[i].b-2;
              leds[i] = CRGB(0,0,0);
            }
            for (int i = 0; i < start; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            for (int i = end; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
            effect = neweffect;
            break;
//          case QuadLinear:
//            for (int i = 0; i < NUMPIXELS; i++) {
//              leds[i] = CRGB(0,0,0);
//            }
//            start = (NUMPIXELS/8)-param;
//            end = (NUMPIXELS/8)+param;
//            for (int i = start; i < end; i++) {
//              leds[i] = CRGB(red, green, blue);
//            }
//            start = ((3*NUMPIXELS)/8)-param;
//            end = ((3*NUMPIXELS)/8)+param;
//            for (int i = start; i < end; i++) {
//              leds[i] = CRGB(red, green, blue);
//            }
//            start = ((5*NUMPIXELS)/8)-param;
//            end = ((5*NUMPIXELS)/8)+param;
//            for (int i = start; i < end; i++) {
//              leds[i] = CRGB(red, green, blue);
//            }
//            start = ((7*NUMPIXELS)/8)-param;
//            end = ((7*NUMPIXELS)/8)+param;
//            for (int i = start; i < end; i++) {
//              leds[i] = CRGB(red, green, blue);
//            }
//            FastLED.show();
//            effect = neweffect;
//            break;
        }
        
      case Constant:

        break;
      case Flash:
        if ((time_ms - last_ms) > param) {
          if (count % 2 == 1) {
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red, green, blue);
            }
            needs_update = true;
          } else {
            for (int i = 0; i < NUMPIXELS; i++) {
              leds[i] = CRGB(0, 0, 0);
            }
            needs_update = true;
          }
          last_ms = time_ms;
          count++;

        }
        break;
      case SetPix:
        
        break;
      case Width:

        break;
      case DoubleWidth:

        break;
      case QuadWidth:

        break;
      case Edges:

        break;
    }
    switch (effect1) {
      case AuxChange:
        switch (neweffect1) {
          default:
            effect1 = None;
            break;
          case None:
            effect1 = neweffect1;
            break;
          case Offset:
            if (param1 > 0) {
              CRGB tmp[NUMPIXELS];
              for (int i = 0; i < param1; i++) {
                tmp[i] = leds[i];
              }
              for (int i = param1; i < NUMPIXELS; i++) {
                leds[i-param1] = leds[i];
              }
              for (int i = NUMPIXELS-param1; i < NUMPIXELS; i++) {
                leds[i] = tmp[(i+param1)-NUMPIXELS];
              }
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
          case FillLeft:
            for (int i = 0; i < param1; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
          case FillCenter:
            for (int i = (NUMPIXELS/2)-param1; i < (NUMPIXELS/2)+param1; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
          case FillRight:
            for (int i = NUMPIXELS-param1; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
          case FillEdges:
            for (int i = 0; i < param1; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            for (int i = NUMPIXELS-param1; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
          case FillDouble:
            for (int i = 0; i < param1; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            for (int i = NUMPIXELS-param1; i < NUMPIXELS; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            for (int i = (NUMPIXELS/2)-param1; i < (NUMPIXELS/2)+param1; i++) {
              leds[i] = CRGB(red1, green1, blue1);
            }
            needs_update = true;
            effect1 = neweffect1;
            break;
        }
      default:
      case None:
        break;
      case FillLeft:
      case FillCenter:
      case FillRight:
      case FillEdges:
      case FillDouble:
        break;
    }
    if (needs_update) {
      FastLED.show();
      needs_update = false;
    }
  }
}
