mod computer;

use computer::Computer;

fn main() {
    env_logger::init();

    let program_name = std::env::args().nth(1).expect("Need a program name to run");
    let program = std::fs::read(program_name).expect("Was not able to load program");

    let mut computer = Computer::new();
    show_debug(&computer.startup_message());

    computer.load_program(&program);

    // dbg!(&computer);
    show_debug(&computer.show_state());

    computer.run();

    // dbg!(&computer);
    show_debug(&computer.show_state());
}

use log;

fn show_debug(s: &str) {
    if log::max_level() >= log::LevelFilter::Debug {
        print!("{}\n", s);
    }
}
