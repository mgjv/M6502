mod computer;

use computer::Computer;


fn main() {
    env_logger::init();

    let program_name = std::env::args().nth(1).expect("Need a program name to run");
    let program = std::fs::read(program_name).expect("Was not able to load program");

    let mut computer = Computer::new();
    print!("{}\n", computer.startup_message());

    computer.load_program(&program);

    dbg!(&computer);

    computer.start();
}
