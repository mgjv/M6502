mod computer;

use computer::Computer;

fn main() {
    let computer = Computer::new();
    print!("{}\n", computer.startup_message());
}
