use crate::bytecode::Op;

struct VM<'b> {
    pub bytecode: &'b [u32],
    pub ip: usize,
    pub regs: Vec<u64>,
    pub csts: Vec<u64>,
}

impl<'b> VM<'b> {
    pub fn execute_arith_instr(&mut self, instr: u32, op: impl Fn(u64, u64) -> u64) {
        let src1 = (if (instr >> 25) & 1 == 0 {
            &mut self.regs
        } else {
            &mut self.csts
        })[(instr >> 8 & 0xff) as usize];

        let src2 = (if (instr >> 24) & 1 == 0 {
            &mut self.regs
        } else {
            &mut self.csts
        })[(instr & 0xff) as usize];
        
        
    }

    pub fn execute_next_instr(&mut self) {
        let instr = self.bytecode[self.ip];
        let op = Op::from_repr((instr >> 26) as u8).expect("invalid operation!");

        match op {
            Op::MOV => todo!(),
            Op::IADD => todo!(),
            Op::ISUB => todo!(),
            Op::IMUL => todo!(),
            Op::IDIV => todo!(),
            Op::IMOD => todo!(),
            Op::FADD => todo!(),
            Op::FSUB => todo!(),
            Op::FMUL => todo!(),
            Op::FDIV => todo!(),
            Op::FMOD => todo!(),
            Op::BEQ => todo!(),
            Op::BNE => todo!(),
            Op::IBLT => todo!(),
            Op::IBLE => todo!(),
            Op::FBLT => todo!(),
            Op::FBLE => todo!(),
            Op::JMP => todo!(),
            Op::CALL => todo!(),
            Op::RET => todo!(),
        }
    }
}
