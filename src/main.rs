mod vm;
mod svg;

use vm::{Op, Vm};

fn main() {
    let prog1 = vec![
        Op::Push(5),
        Op::Push(3),
        Op::Add,
        Op::Print,
        Op::Halt,
    ];

    let prog2 = vec![
        Op::Push(1),
        Op::Push(1),
        Op::Add,
        Op::Dup,
        Op::Print,
        Op::Push(2),
        Op::Add,
        Op::Print,
        Op::Halt,
    ];

    let prog3 = vec![
        Op::Push(10),
        Op::Push(2),
        Op::Mul,
        Op::Push(3),
        Op::Sub,
        Op::Print,
        Op::Halt,
    ];

    let programs = vec![
        ("prog1", prog1),
        ("prog2", prog2),
        ("prog3", prog3),
    ];

    for (name, ops) in programs {
        println!("--- {} ---", name);
        let mut vm = Vm::new();
        vm.run(&ops);
        svg::ops_to_svg(&ops, &format!("{}.svg", name));
        println!("Wrote {}.svg\n", name);
    }
}
