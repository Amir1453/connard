
fn main() {
    use TACInst::*;

    let n = NamedTemp("n".into());
    let t0 = Temp(0);
    let t1 = Temp(1);
    let t2 = Temp(2);
    let t3 = Temp(3);

    let lentry = Label::Named("Entaary".into());
    let l1 = Label::Numeric(1);
    let l2 = Label::Numeric(2);
    let l3 = Label::Numeric(3);

    let mut fib_instruction = Vec::new();
    // fib_instruction.push(LabelDecl(lentry));
    fib_instruction.push(Const {
        destination: TACTemp::Temp(t0),
        constant: 0,
    });
    fib_instruction.push(Const {
        destination: TACTemp::Temp(t1),
        constant: 1,
    });
    fib_instruction.push(Const {
        destination: TACTemp::Temp(t2),
        constant: 1,
    });
    fib_instruction.push(LabelDecl(l1.clone()));
    fib_instruction.push(ConditionalJump {
        opcode: TACJumpOpcode::JZ,
        condition: TACTemp::NamedTemp(n.clone()),
        destination: l3.clone(),
    });
    // fib_instruction.push(UnconditionalJump(l2.clone()));
    // fib_instruction.push(LabelDecl(l2));
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::SUB,
        lhs: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(t2),
        result: TACTemp::NamedTemp(n.clone()),
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::ADD,
        lhs: TACTemp::Temp(t0),
        rhs: TACTemp::Temp(t1),
        result: TACTemp::Temp(t3),
    });
    fib_instruction.push(Copi {
        destination: TACTemp::Temp(t0),
        source: TACTemp::Temp(t1),
    });
    fib_instruction.push(Copi {
        destination: TACTemp::Temp(t1),
        source: TACTemp::Temp(t3),
    });
    // fib_instruction.push(UnconditionalJump(l1.clone()));
    fib_instruction.push(LabelDecl(l3));
    fib_instruction.push(Return(Some(TACTemp::Temp(t0))));

    let bb = BasicBlocks::from_inst(&fib_instruction);
    let cfg = CFG::from_basic_blocks(bb);
    let fib = ProcDecl::new("Fibonacci".into(), Some(vec![n]), fib_instruction);

    println!("PRE INFERENCE");
    println!("{}", fib);

    let basic_dot = Dot::new(&cfg.graph);
    std::fs::write("test.dot", format!("{}", basic_dot)).unwrap();
}
