extern crate byteorder;
extern crate ole;
extern crate encoding;

mod eqn;
mod error;
mod constants;


fn main() {
    let eqn = eqn::MTEquation::from_ole("assets/oleObject1.bin").unwrap();
    println!("{:?}", eqn);
    let latex = eqn.translate();
    println!("{:?}", latex);
}
