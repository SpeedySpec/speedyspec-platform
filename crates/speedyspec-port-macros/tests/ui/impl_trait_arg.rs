#[speedyspec_port_macros::port(key = "T\\P")]
pub trait P {
    fn each(&self, f: impl Fn(i64)) -> i64;
}
fn main() {}
