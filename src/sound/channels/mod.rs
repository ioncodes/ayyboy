pub mod noise;
pub mod square;
pub mod wave;

pub trait Channel {
    fn tick(&mut self);
    fn get_amplitude(&self) -> f32;
    fn step_length(&mut self);
}
