use crate::expr::Expr;



#[derive(Copy, Clone, Debug)]
pub struct Reg (u8);

pub const X0: Reg = Reg(0);
pub const X1: Reg = Reg(1);
pub const X2: Reg = Reg(2);
pub const X3: Reg = Reg(3);
pub const X4: Reg = Reg(4);
pub const X5: Reg = Reg(5);
pub const X6: Reg = Reg(6);
pub const X7: Reg = Reg(7);
pub const X8: Reg = Reg(8);
pub const X9: Reg = Reg(9);
pub const X10: Reg = Reg(10);
pub const X11: Reg = Reg(11);
pub const X12: Reg = Reg(12);
pub const X13: Reg = Reg(13);
pub const X14: Reg = Reg(14);
pub const X15: Reg = Reg(15);
pub const X16: Reg = Reg(16);
pub const X17: Reg = Reg(17);
pub const X18: Reg = Reg(18);
pub const X19: Reg = Reg(19);
pub const X20: Reg = Reg(20);
pub const X21: Reg = Reg(21);
pub const X22: Reg = Reg(22);
pub const X23: Reg = Reg(23);
pub const X24: Reg = Reg(24);
pub const X25: Reg = Reg(25);
pub const X26: Reg = Reg(26);
pub const X27: Reg = Reg(27);
pub const X28: Reg = Reg(28);
pub const X29: Reg = Reg(29);
pub const X30: Reg = Reg(30);
pub const X31: Reg = Reg(31);

pub const ZERO: Reg = Reg(0);
pub const RA: Reg = Reg(1);
pub const SP: Reg = Reg(2);
pub const GP: Reg = Reg(3);
pub const TP: Reg = Reg(4);
pub const T0: Reg = Reg(5);
pub const T1: Reg = Reg(6);
pub const T2: Reg = Reg(7);
pub const S0: Reg = Reg(8);
pub const FP: Reg = Reg(8);
pub const S1: Reg = Reg(9);
pub const A0: Reg = Reg(10);
pub const A1: Reg = Reg(11);
pub const A2: Reg = Reg(12);
pub const A3: Reg = Reg(13);
pub const A4: Reg = Reg(14);
pub const A5: Reg = Reg(15);
pub const A6: Reg = Reg(16);
pub const A7: Reg = Reg(17);
pub const S2: Reg = Reg(18);
pub const S3: Reg = Reg(19);
pub const S4: Reg = Reg(20);
pub const S5: Reg = Reg(21);
pub const S6: Reg = Reg(22);
pub const S7: Reg = Reg(23);
pub const S8: Reg = Reg(24);
pub const S9: Reg = Reg(25);
pub const S10: Reg = Reg(26);
pub const S11: Reg = Reg(27);
pub const T3: Reg = Reg(28);
pub const T4: Reg = Reg(29);
pub const T5: Reg = Reg(30);
pub const T6: Reg = Reg(31);

impl std::fmt::Display for Reg {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 > 31 { Err(std::fmt::Error) } else {
      write!(f, "x{}", self.0)
    }
  }
}

// yeah this is just a bool but more explicit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RState {
  Used,
  Free,
}

type RegMap = [RState; 32];

pub struct RVBuilder {
  pub instrs: Vec<String>,
  regs: RegMap,
}

impl RVBuilder {
  pub fn push(&mut self, s: String) {
    self.instrs.push(s)
  }

  pub fn dump(&self) {
    for l in &self.instrs {
      println!("    {}", l)
    }
  }

  pub fn new() -> Self {
    Self { instrs: vec![], regs: [RState::Free; 32] }
  }

  fn get_reg(&mut self) -> Option<Reg> {
    const REG_ORDER: &[Reg] = &[T0, T1, T2, T3, T4, T5, T6];
    for reg in REG_ORDER {
      if self.regs[reg.0 as usize] == RState::Free {
        self.regs[reg.0 as usize] = RState::Used;
        return Some(*reg);
      }
    }
    None
  }

  /**
   * I bet this could be managed using the borrow checker somehow.
   * I'm not going to waste time on that for now.
   */
  fn free_reg(&mut self, r: Reg) {
    self.regs[r.0 as usize] = RState::Free;
  }

  pub fn compile_expr(&mut self, e: &Expr) -> Option<Reg> {
    match e {
      Expr::Lit(val) => {
        let reg = self.get_reg().expect("failed to allocate reg for immediate");
        if *val > u32::MAX as _ || *val < i32::MIN as _ {
          eprintln!("WARN: immediate {} is out of 32 bit range", val);
        }
        self.push(format!("li {}, {}", reg, val));
        Some(reg)
      },
      Expr::Bin(left, op, right) =>{
        use crate::expr::BinOp::*;

        let left = self.compile_expr(&left);
        let right = self.compile_expr(&right);
        let left = left?;
        let right = right?;
        match op {
          Add => {
            self.push(format!("add {}, {}, {}", left, left, right));
            self.free_reg(right);
            Some(left)
          },
          Sub => {
            self.push(format!("sub {}, {}, {}", left, left, right));
            self.free_reg(right);
            Some(left)
          },
          Mul => {
            self.push(format!("mul {}, {}, {}", left, left, right));
            self.free_reg(right);
            Some(left)
          },
          Div => {
            self.push(format!("div {}, {}, {}", left, left, right));
            self.free_reg(right);
            Some(left)
          },
        }
      }
      Expr::Ident(_name) => unimplemented!(),
    }
  }
}
