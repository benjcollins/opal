use std::collections::{HashMap, hash_map::Entry};

use crate::{
    instr::{ImmSlot, Opcode, Operand, Reg},
    value::Value,
};

pub struct Bytecode {
    buffer: Vec<u16>,
    imm_slots: Vec<Value>,
    imm_slot_value_map: HashMap<Value, ImmSlot>,
}

macro_rules! instr {
    ($name:ident, $imm_opcode:ident, $reg_opcode:ident) => {
        pub fn $name(&mut self, operand: impl Into<Operand>) {
            let (opcode, operand) = match operand.into() {
                Operand::Reg(reg) => (Opcode::$reg_opcode, reg.0),
                Operand::ImmValue(value) => (Opcode::$imm_opcode, self.get_value_imm_slot(value).0),
                Operand::ImmSlot(slot) => (Opcode::$imm_opcode, slot.0),
            };
            self.buffer
                .push(u16::from_be_bytes([opcode as u8, operand]));
        }
    };
}

impl Bytecode {
    pub fn new() -> Bytecode {
        Bytecode {
            buffer: vec![],
            imm_slots: vec![],
            imm_slot_value_map: HashMap::new(),
        }
    }

    pub fn get_value_imm_slot(&mut self, value: Value) -> ImmSlot {
        match self.imm_slot_value_map.entry(value) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let slot = ImmSlot(self.imm_slots.len() as u8);
                self.imm_slots.push(value);
                entry.insert(slot);
                slot
            }
        }
    }

    pub fn alloc_imm_slot(&mut self) -> ImmSlot {
        let slot = ImmSlot(self.imm_slots.len() as u8);
        self.imm_slots.push(Value::UNIT);
        slot
    }

    pub fn store(&mut self, reg: Reg) {
        self.buffer
            .push(u16::from_be_bytes([Opcode::StoreReg as u8, reg.0]));
    }

    pub fn call(&mut self, base: u8) {
        self.buffer
            .push(u16::from_be_bytes([Opcode::Call as u8, base]));
    }

    pub fn ret(&mut self) {
        self.buffer.push(u16::from_be_bytes([Opcode::Ret as u8, 0]));
    }

    instr!(load, LoadImm, LoadReg);

    instr!(iadd, AddIntImm, AddIntReg);
    instr!(fadd, AddFloatImm, AddFloatReg);

    instr!(isub, SubIntImm, SubIntReg);
    instr!(fsub, SubFloatImm, SubFloatReg);

    instr!(imul, MulIntImm, MulIntReg);
    instr!(fmul, MulFloatImm, MulFloatReg);

    instr!(idiv, DivIntImm, DivIntReg);
    instr!(fdiv, DivFloatImm, DivFloatReg);
}
