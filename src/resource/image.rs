#[derive(Default,Clone)]
pub struct Image {
    pub channels: usize,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Image {

    pub fn new() -> Self {
        return Image {
            channels: 4,
            width: 0,
            height: 0,
            data: Vec::with_capacity(0),
        };
    }

}
