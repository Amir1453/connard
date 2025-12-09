use bx_compiler::driver::Driver;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Driver::cli()
}
