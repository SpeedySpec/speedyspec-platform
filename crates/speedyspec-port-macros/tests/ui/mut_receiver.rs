#[speedyspec_port_macros::port(key = "T\\P")]
pub trait P {
    fn set(&mut self, v: i64);
}
fn main() {}
