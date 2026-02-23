mod compile;
mod decompile;
use compile::compile;
use decompile::decompile;
use std::env;
use std::fs::File;
use std::io::Read;
use std::{cell::RefCell, rc::Rc};

fn main() {
    let args: Vec<String> = env::args().collect();
    let source = if args.len() > 1 {
        let mut file = File::open(&args[1]).expect("Could not open source file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read source file");
        contents
    } else {
        panic!("No source file provided");
    };
    let program = compile(&source);
    println!("{}", decompile(&program));

    use coreset_vm::{Controller, Memory};
    let mut controller = Controller::new();
    controller.set_program(program);
    controller.add_memory(Rc::new(RefCell::new(Memory::medium())));

    while !controller.halted {
        controller.step();
    }
}
