pub trait Backend{
    fn draw_frame(&mut self, framebuffer: &[u8; 64 * 32]);
    fn poll_keys(&mut self) -> Vec<Keys>;
}

pub enum Keys{

}
