#![allow(dead_code)]

use std::collections::HashMap;

use crate::ir::tac::{
    InstBlock, Label, ProcDecl, TACBinaryOpcode, TACInst, TACJumpOpcode, TACTemp,
};
use crate::types::Name;

pub struct ProcBuilder {
    pub name: Name,
    pub arguments: Option<Vec<Name>>,
    pub instructions: InstBlock,

    variables: HashMap<Name, TACTemp>,
    labels: HashMap<Name, Label>,
    next_temp: i64,
    next_label: i64,
}

impl ProcBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            arguments: None,
            instructions: InstBlock::new(),
            variables: HashMap::new(),
            labels: HashMap::new(),
            next_temp: 0,
            next_label: 0,
        }
    }

    pub fn build(self) -> ProcDecl {
        ProcDecl {
            name: self.name,
            arguments: self.arguments,
            instructions: self.instructions,
        }
    }

    pub fn add_argument<N: Into<Name> + Clone>(&mut self, arg: N) {
        self.define(arg.clone());
        self.arguments.get_or_insert_with(Vec::new).push(arg.into());
    }

    pub fn add_instruction(&mut self, instr: TACInst) -> &mut Self {
        self.instructions.push(instr);
        self
    }

    pub fn constant<N: Into<Name>>(&mut self, name: N, value: i64) -> &mut Self {
        let destination = self.define(name);

        self.add_instruction(TACInst::Const {
            destination,
            constant: value,
        })
    }

    pub fn copy<S: Into<Name>, D: Into<Name>>(&mut self, source: S, destination: D) -> &mut Self {
        let source = self.resolve(source);
        let destination = self.define(destination);

        self.add_instruction(TACInst::Copi {
            destination,
            source,
        })
    }

    pub fn binary<L: Into<Name>, R: Into<Name>, D: Into<Name>>(
        &mut self,
        opcode: TACBinaryOpcode,
        lhs: L,
        rhs: R,
        destination: D,
    ) -> &mut Self {
        let lhs = self.resolve(lhs);
        let rhs = self.resolve(rhs);
        let result = self.define(destination);

        self.add_instruction(TACInst::BinaryOperation {
            opcode,
            lhs,
            rhs,
            result,
        })
    }

    pub fn add<L: Into<Name>, R: Into<Name>, D: Into<Name>>(
        &mut self,
        lhs: L,
        rhs: R,
        destination: D,
    ) -> &mut Self {
        self.binary(TACBinaryOpcode::ADD, lhs, rhs, destination)
    }

    pub fn sub<L: Into<Name>, R: Into<Name>, D: Into<Name>>(
        &mut self,
        lhs: L,
        rhs: R,
        destination: D,
    ) -> &mut Self {
        self.binary(TACBinaryOpcode::SUB, lhs, rhs, destination)
    }

    pub fn mul<L: Into<Name>, R: Into<Name>, D: Into<Name>>(
        &mut self,
        lhs: L,
        rhs: R,
        destination: D,
    ) -> &mut Self {
        self.binary(TACBinaryOpcode::MUL, lhs, rhs, destination)
    }

    pub fn div<L: Into<Name>, R: Into<Name>, D: Into<Name>>(
        &mut self,
        lhs: L,
        rhs: R,
        destination: D,
    ) -> &mut Self {
        self.binary(TACBinaryOpcode::DIV, lhs, rhs, destination)
    }

    pub fn label_decl<L: Into<Name>>(&mut self, label: L) -> &mut Self {
        let label = self.resolve_label(label);
        self.add_instruction(TACInst::LabelDecl(label))
    }

    pub fn jump<L: Into<Name>>(&mut self, destination: L) -> &mut Self {
        let destination = self.resolve_label(destination);
        self.add_instruction(TACInst::UnconditionalJump(destination))
    }

    pub fn branch_named<N: Into<Name>>(
        &mut self,
        opcode: TACJumpOpcode,
        condition: N,
        destination: N,
    ) -> &mut Self {
        let condition = self.resolve(condition);
        let destination = self.resolve_label(destination);

        self.add_instruction(TACInst::ConditionalJump {
            opcode,
            condition,
            destination,
        })
    }

    pub fn return_void(&mut self) -> &mut Self {
        self.add_instruction(TACInst::Return(None))
    }

    pub fn return_value(&mut self, value: TACTemp) -> &mut Self {
        self.add_instruction(TACInst::Return(Some(value)))
    }

    fn define<N: Into<Name>>(&mut self, name: N) -> TACTemp {
        let name = name.into();
        let temp = self.fresh_temp();

        self.variables.insert(name, temp.clone());

        temp
    }

    fn resolve<N: Into<Name>>(&self, name: N) -> TACTemp {
        let name = name.into();

        self.variables
            .get(&name)
            .cloned()
            .unwrap_or_else(|| panic!("use of undefined variable `{}`", name))
    }

    fn define_label<L: Into<Name>>(&mut self, name: L) -> Label {
        let name = name.into();
        let label = self.fresh_label();

        self.labels.insert(name, label.clone());

        label
    }

    fn resolve_label<L: Into<Name>>(&mut self, name: L) -> Label {
        let name = name.into();

        match self.labels.get(&name) {
            Some(label) => label.clone(),
            None => self.define_label(name),
        }
    }

    fn fresh_temp(&mut self) -> TACTemp {
        let temp = TACTemp::Temp(self.next_temp);
        self.next_temp += 1;
        temp
    }

    fn fresh_label(&mut self) -> Label {
        let label = Label::Numeric(self.next_label);
        self.next_label += 1;
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_addition() {
        let mut proc = ProcBuilder::new(Name::from("main"));

        proc.constant("rhs", 10)
            .constant("lhs", 20)
            .add("lhs", "rhs", "result");

        let procedure = proc.build();

        eprintln!("{}", procedure);
    }
}
