use led_control::{Pixel, Effect, AuxEffect};

//#[derive(Serialize, Deserialize, Debug, Clone)]
//pub struct Message {
//    pub color: Pixel,
//    pub effect: Effect,
//    pub aux_color: Option<Pixel>,
//    pub aux_effect: AuxEffect,
//}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub pixels: Vec<Pixel>,
}
