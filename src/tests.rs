// use crate::lexer::Lexer;
// use crate::synchecker::SynChecker;
// use lalrpop_util::lalrpop_mod;
// use std::fs;
//
// lalrpop_mod!(pub bxgrammar);
//
// #[test]
// fn test_parser() {
//     let parser = bxgrammar::BXParser::new();
// }

#![allow(dead_code)]

use std::sync::LazyLock;

use compact_str::CompactString;

use crate::tac::{Label, ProcDecl, TACBinaryOpcode, TACInst, TACJumpOpcode, TACTemp};

pub const FIBONACCI: LazyLock<ProcDecl> = LazyLock::new(|| {
    use TACInst::*;

    let n = CompactString::new("n");

    let ldone = Label::Named("done".into());
    let l1 = Label::Numeric(1);
    let l2 = Label::Numeric(2);
    let l3 = Label::Numeric(3);

    let mut fib_instruction = Vec::new();

    // fib_instruction.push(ConditionalJump {
    //     opcode: TACJumpOpcode::JZ,
    //     condition: TACTemp::NamedTemp(n.clone()),
    //     destination: ldone.clone(),
    // });
    fib_instruction.push(Const {
        destination: TACTemp::Temp(0),
        constant: 1,
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::SUB,
        lhs: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(0),
        result: TACTemp::Temp(1),
    });
    fib_instruction.push(ConditionalJump {
        opcode: TACJumpOpcode::JZ,
        condition: TACTemp::Temp(1),
        destination: ldone.clone(),
    });
    fib_instruction.push(Return(None));
    fib_instruction.push(LabelDecl(l1));
    fib_instruction.push(Const {
        destination: TACTemp::Temp(2),
        constant: 2,
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::SUB,
        lhs: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(2),
        result: TACTemp::Temp(3),
    });
    fib_instruction.push(Parameter {
        nth_param: 1,
        source_temp: TACTemp::Temp(1),
    });
    fib_instruction.push(LabelDecl(l2));
    fib_instruction.push(ProcCall {
        proc_name: "Fibonacci".into(),
        arg_count: 1,
        result: TACTemp::Temp(1),
    });
    fib_instruction.push(Parameter {
        nth_param: 1,
        source_temp: TACTemp::Temp(3),
    });
    fib_instruction.push(LabelDecl(l3));
    fib_instruction.push(ProcCall {
        proc_name: "Fibonacci".into(),
        arg_count: 1,
        result: TACTemp::Temp(1),
    });
    fib_instruction.push(BinaryOperation {
        opcode: TACBinaryOpcode::ADD,
        result: TACTemp::NamedTemp(n.clone()),
        rhs: TACTemp::Temp(0),
        lhs: TACTemp::Temp(1),
    });
    fib_instruction.push(LabelDecl(ldone));
    fib_instruction.push(Return(Some(TACTemp::NamedTemp(n.clone()))));

    ProcDecl::new("Fibonacci".into(), Some(vec![n]), fib_instruction)
});
