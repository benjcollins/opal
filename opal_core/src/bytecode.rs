use std::{
    cell::Cell,
    collections::{HashMap, hash_map::Entry},
};

use crate::{
    instr::{ImmSlot, Opcode, Operand, Reg},
    value::Value,
};

pub struct Bytecode<'v> {
    buffer: Vec<u16>,
    imm_slots: Vec<Value<'v>>,
    imm_slot_value_map: HashMap<Value<'v>, ImmSlot>,
}

pub struct InstrBuilder<'v, 'b> {
    bytecode: &'b mut Bytecode<'v>,
}

pub struct Fun<'a> {
    pub bytecode: Vec<u16>,
    pub immediates: Vec<Cell<Value<'a>>>,
}

macro_rules! instr {
    ($name:ident, $imm_opcode:ident, $reg_opcode:ident, $value_lifetime:lifetime) => {
        pub fn $name(self, operand: impl Into<Operand<$value_lifetime>>) {
            let (opcode, operand) = match operand.into() {
                Operand::Reg(reg) => (Opcode::$reg_opcode, reg.0),
                Operand::ImmValue(value) => (
                    Opcode::$imm_opcode,
                    self.bytecode.get_value_imm_slot(value).0,
                ),
                Operand::ImmSlot(slot) => (Opcode::$imm_opcode, slot.0),
            };
            self.bytecode
                .buffer
                .push(u16::from_be_bytes([opcode as u8, operand]));
        }
    };
}

impl<'v> Bytecode<'v> {
    pub fn new() -> Bytecode<'v> {
        Bytecode {
            buffer: Vec::new(),
            imm_slots: Vec::new(),
            imm_slot_value_map: HashMap::new(),
        }
    }

    pub fn finish(self) -> Fun<'v> {
        Fun {
            bytecode: self.buffer,
            immediates: self.imm_slots,
        }
    }

    pub fn get_value_imm_slot(&mut self, value: Value<'v>) -> ImmSlot {
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

    pub fn instr(&mut self) -> InstrBuilder<'v, '_> {
        InstrBuilder { bytecode: self }
    }
}

impl<'v> Default for Bytecode<'v> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'v, 'b> InstrBuilder<'v, 'b> {
    instr!(load, LoadImm, LoadReg, 'v);

    instr!(iadd, AddIntImm, AddIntReg, 'v);
    instr!(fadd, AddFloatImm, AddFloatReg, 'v);

    instr!(isub, SubIntImm, SubIntReg, 'v);
    instr!(fsub, SubFloatImm, SubFloatReg, 'v);

    instr!(imul, MulIntImm, MulIntReg, 'v);
    instr!(fmul, MulFloatImm, MulFloatReg, 'v);

    instr!(idiv, DivIntImm, DivIntReg, 'v);
    instr!(fdiv, DivFloatImm, DivFloatReg, 'v);

    pub fn store(self, reg: Reg) {
        self.bytecode
            .buffer
            .push(u16::from_be_bytes([Opcode::StoreReg as u8, reg.0]));
    }

    pub fn call(self, base: u8) {
        self.bytecode
            .buffer
            .push(u16::from_be_bytes([Opcode::Call as u8, base]));
    }

    pub fn ret(self) {
        self.bytecode
            .buffer
            .push(u16::from_be_bytes([Opcode::Ret as u8, 0]));
    }
}
