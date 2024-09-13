use localic_utils::TestContextBuilder;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let test_ctx = TestContextBuilder::default();

    Ok(())
}
