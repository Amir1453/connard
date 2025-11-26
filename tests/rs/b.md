fn main() {
    use TACInst::*;

    let n = NamedTemp("n".into());
    let t0 = Temp(0);
    let t1 = Temp(1);
    let t2 = Temp(2);
    let t3 = Temp(3);

    let ldone = Label::Named("done".into());
    let l1 = Label::Numeric(1);
    let l2 = Label::Numeric(2);
    let l3 = Label::Numeric(3);

    let mut fib_instruction = Vec::new();

    fib_instruction.push(ConditionalJump {
        opcode: TACJumpOpcode::JZ,
        condition: TACTemp::NamedTemp(n.clone()),
        destination: ldone.clone(),
    });
    fib_instruction.push(Const {
        destination: TACTemp::Temp(t0),
        constant: 1,
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::SUB,
        lhs: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(t0),
        result: TACTemp::Temp(t1),
    });
    fib_instruction.push(ConditionalJump {
        opcode: TACJumpOpcode::JZ,
        condition: TACTemp::Temp(t1),
        destination: ldone.clone(),
    });
    fib_instruction.push(Const {
        destination: TACTemp::Temp(t2),
        constant: 2,
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::SUB,
        lhs: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(t2),
        result: TACTemp::Temp(t3),
    });
    fib_instruction.push(Parameter {
        nth_param: 1,
        source_temp: TACTemp::Temp(t1),
    });
    fib_instruction.push(ProcCall {
        proc_name: "Fibonacci".into(),
        arg_count: 1,
    });
    fib_instruction.push(Parameter {
        nth_param: 1,
        source_temp: TACTemp::Temp(t3),
    });
    fib_instruction.push(ProcCall {
        proc_name: "Fibonacci".into(),
        arg_count: 1,
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::ADD,
        result: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(t0),
        lhs: TACTemp::Temp(t1),
    });
    fib_instruction.push(LabelDecl(ldone));
    fib_instruction.push(Return(Some(TACTemp::NamedTemp(n.clone()))));

    let bb = BasicBlocks::from_inst(&fib_instruction);
    let cfg = CFG::from_basic_blocks(bb);
    let fib = ProcDecl::new("Fibonacci".into(), Some(vec![n]), fib_instruction);

    println!("PRE INFERENCE");
    println!("{}", fib);

    for inst in cfg.serialize_tac() {
        println!("{}", inst);
    }

    // let basic_dot = Dot::new(&cfg.graph);
    // std::fs::write("test.dot", format!("{}", basic_dot)).unwrap();
}


