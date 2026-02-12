pub struct Frame;

pub trait Frontend {
    type Error;

    fn render_frame(&mut self, frame: Frame) -> Result<Frame, Self::Error>;
}
