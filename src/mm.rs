use crate::ast::*;
use crate::tac::*;
use crate::types::LoopStack;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct MM {
    vars: HashMap<String, Temp>,
    tacs: Vec<TAC>,
    loop_stack: LoopStack,
    temp_counter: i64,
    label_counter: i64,
}

impl MM {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            tacs: Vec::new(),
            loop_stack: LoopStack::new(),
            temp_counter: -1,
            label_counter: -1,
        }
    }

    pub fn munch(program: &Program) -> Vec<TAC> {
        let mut mm = MM::new();
        mm.munch_program(program);
        mm.tacs
    }

    fn munch_program(&mut self, program: &Program) {
        self.munch_block(program);
    }

    fn munch_block(&mut self, block: &Block) {
        for stmt in &block.0 {
            self.munch_statement(stmt);
        }
    }

    fn munch_statement(&mut self, stmt: &Statement) {
        use TACArgs::*;
        use TACOpcode::*;

        match stmt {
            Statement::Variable { name, value, ty: _ } => {
                let fresh_temporary = self.fresh_temp();
                self.vars.insert(name.into(), fresh_temporary);
                let munched_value = self.munch_expression(value);
                self.emit(COPY, Temporary(munched_value), Some(fresh_temporary));
            }

            Statement::Block(block) => {
                self.munch_block(block);
            }

            Statement::Assignment { name, value } => {
                let munched_value = self.munch_expression(value);
                self.emit(COPY, Temporary(munched_value), Some(self.vars[name]));
            }

            Statement::Print { value } => {
                let munched_value = self.munch_expression(value);
                self.emit(PRINT, Temporary(munched_value), None);
            }

            Statement::If {
                condition,
                then_block,
                else_branch,
            } => {
                let label_then = self.fresh_label();
                let label_else = self.fresh_label();
                let label_end = self.fresh_label();

                self.munch_bool_expression(condition, label_then, label_else);

                self.emit_label(label_then);
                self.munch_block(then_block);
                self.emit(JMP, Label(label_end), None);

                self.emit_label(label_else);
                if let Some(ebr) = else_branch {
                    self.munch_statement(ebr);
                }

                self.emit_label(label_end);
            }

            Statement::While { condition, block } => {
                let label_head = self.fresh_label();
                let label_body = self.fresh_label();
                let label_end = self.fresh_label();

                self.emit_label(label_head);
                self.munch_bool_expression(condition, label_body, label_end);
                self.emit_label(label_body);

                self.loop_stack.push_loop(label_head, label_end);
                self.munch_block(block);
                self.loop_stack.pop_loop();

                self.emit(JMP, Label(label_head), None);
                self.emit_label(label_end);
            }

            Statement::Jump(state) => match state {
                JumpState::Continue => {
                    if let Some((cont, _)) = self.loop_stack.current_loop() {
                        self.emit(JMP, Label(cont), None);
                    }
                }

                JumpState::Break => {
                    if let Some((_, brk)) = self.loop_stack.current_loop() {
                        self.emit(JMP, Label(brk), None);
                    }
                }
            },
        }
    }

    fn munch_expression(&mut self, expr: &Expression) -> Temp {
        use TACArgs::*;
        use TACOpcode::*;

        match expr {
            Expression::Variable(name) => self.vars[name],

            Expression::Number(value) => {
                let target = self.fresh_temp();
                self.emit(CONST, Number(value.clone()), Some(target));
                target
            }

            Expression::Bool(b) => {
                let target = self.fresh_temp();
                let val = if *b { 1 } else { 0 };
                self.emit(CONST, Number(val), Some(target));
                target
            }

            Expression::BinaryOperation {
                lhs, operator, rhs, ..
            } => {
                let target = self.fresh_temp();
                let munched_lhs = self.munch_expression(lhs);
                let munched_rhs = self.munch_expression(rhs);
                println!(
                    "FROM: {:?} INTO {:?}",
                    operator,
                    Into::<TACOpcode>::into(*operator)
                );
                self.emit(
                    operator.clone().into(),
                    Temporaries((munched_lhs, munched_rhs)),
                    Some(target),
                );
                target
            }

            Expression::UnaryOperation {
                operator, value, ..
            } => {
                let target = self.fresh_temp();
                let munched_value = self.munch_expression(value);
                println!("FROM: {:?} INTO", operator);
                self.emit(
                    operator.clone().into(),
                    Temporary(munched_value),
                    Some(target),
                );
                target
            }
        }
    }

    fn munch_bool_expression(&mut self, bool_expr: &Expression, label_true: i64, label_false: i64) {
        use TACArgs::*;
        use TACOpcode::*;

        match bool_expr {
            Expression::Bool(boolo) => {
                if *boolo {
                    self.emit(JMP, Label(label_true), None);
                } else {
                    self.emit(JMP, Label(label_false), None);
                }
            }

            Expression::BinaryOperation {
                lhs, operator, rhs, ..
            } => {
                use Operator::*;

                match operator {
                    Equal | NEqual | L | LTE | G | GTE => {
                        let cmp_temp = self.fresh_temp();
                        let munched_lhs = self.munch_expression(lhs);
                        let munched_rhs = self.munch_expression(rhs);

                        self.emit(SUB, Temporaries((munched_lhs, munched_rhs)), Some(cmp_temp));

                        match operator {
                            Equal => {
                                println!("I am at Equal");
                                self.emit(JZ, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            NEqual => {
                                println!("I am at NEqual");
                                self.emit(JNZ, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            L => {
                                println!("I am at L");
                                self.emit(JL, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            LTE => {
                                println!("I am at LTE");
                                self.emit(JLE, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            G => {
                                println!("I am at G");
                                self.emit(JNLE, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            GTE => {
                                println!("I am at GTE");
                                self.emit(JNL, TempAndLabel((cmp_temp, label_true)), None);
                                self.emit(JMP, Label(label_false), None);
                            }
                            _ => {}
                        }
                    }

                    LAnd => {
                        println!("I am at LAnd");
                        let mid = self.fresh_label();
                        self.munch_bool_expression(lhs, mid, label_false);
                        self.emit(LABEL, Label(mid), None);
                        self.munch_bool_expression(rhs, label_true, label_false);
                    }

                    LOr => {
                        println!("I am at LOr");
                        let mid = self.fresh_label();
                        self.munch_bool_expression(lhs, label_true, mid);
                        self.emit(LABEL, Label(mid), None);
                        self.munch_bool_expression(rhs, label_true, label_false);
                    }
                    _ => {}
                }
            }

            // Only Unary Bool Operation is the LNot
            Expression::UnaryOperation { value, .. } => {
                println!("I am at LNot");
                self.munch_bool_expression(value, label_false, label_true);
            }

            _ => {}
        }
    }

    fn fresh_temp(&mut self) -> Temp {
        self.temp_counter += 1;
        Temp(self.temp_counter)
    }

    fn fresh_label(&mut self) -> i64 {
        self.label_counter += 1;
        self.label_counter
    }

    fn emit(&mut self, opcode: TACOpcode, args: TACArgs, result: Option<Temp>) {
        self.tacs.push(TAC::new(opcode, args, result));
    }

    fn emit_label(&mut self, label: i64) {
        self.tacs
            .push(TAC::new(TACOpcode::LABEL, TACArgs::Label(label), None));
    }
}
