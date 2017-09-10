use led_control::Pixel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub pixels: Vec<Pixel>,
}
