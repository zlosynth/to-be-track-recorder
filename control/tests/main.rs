use cucumber::World;

#[derive(Debug, Default, World)]
pub struct ControlWorld {}

fn main() {
    futures::executor::block_on(ControlWorld::run("tests"));
}
