use std::{collections::HashMap};

use crate::expr::{Expr, Stmt};



#[derive(Copy, Clone, Debug)]
pub struct Reg (u8);

// is this really necessary? idk, but i wrote it before anything else and it
// might pay off eventually.
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

const STDLIB: &'static str = include_str!("../resources/stdlib.s");

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

struct RegMap {
  pub map: [RState; 32]
}

impl RegMap {
  fn get_reg(&mut self) -> Option<Reg> {
    const REG_ORDER: &[Reg] = &[T0, T1, T2, T3, T4, T5, T6];
    for reg in REG_ORDER {
      if self.map[reg.0 as usize] == RState::Free {
        self.map[reg.0 as usize] = RState::Used;
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
    self.map[r.0 as usize] = RState::Free;
  }
}

struct SymTab {
  pub data: HashMap<String, (String, String)>, // (label, initial value)
  pub strings: Vec<(String, String)>, // (label, contents)
}

impl SymTab {
  fn decl(&mut self, name: String, initial: String) -> Option<()> {
    if self.data.contains_key(&name) {
      eprintln!("ERR: Redeclaration of variable {}", name);
      return None;
    }
    let lbl = format!("var_{}", &name);
    self.data.insert(name, (lbl, initial));
    Some(())
  }

  fn get_var(&self, name: &str) -> Option<&str> {
    self.data.get(name).map(|tup| &tup.0 as _)
  }

  fn add_string(&mut self, s: String) -> String {
    self.strings.push((format!("__str_{}", self.strings.len()), s));
    self.strings.last().unwrap().0.clone() /* PERF: avoid clone */
  }

  fn dump_data_asm(&self) {
    println!(".data");
    for (label, initial) in &self.strings {
      println!(r#"{}: .asciz "{}" "#, label, initial);
    }
    for (label, initial) in self.data.values() {
      println!("{}:", label);
      println!("    .word {}", initial);
    }
    println!("");
  }
}

pub struct Compiler {
  stab: SymTab,
  pub instrs: Vec<String>,
  regs: RegMap,
}

impl Compiler {
  pub fn push(&mut self, s: String) {
    self.instrs.push(s)
  }

  pub fn dump(&self) {
    self.stab.dump_data_asm();
    println!(".text");
    println!("    j __start");
    println!("{}", STDLIB);
    println!("__start:");
    for l in &self.instrs {
      println!("    {}", l)
    }
  }

  pub fn new() -> Self {
    Self {
      stab: SymTab {data: HashMap::new(), strings: vec![]},
      instrs: vec![],
      regs: RegMap{map: [RState::Free; 32]}
    }
  }

  pub fn compile(&mut self, stmts: Vec<Stmt>) {
    stmts.iter().for_each(|stmt| self.compile_stmt(stmt));
  }

  pub fn compile_stmt(&mut self, s: &Stmt) {
    match s {
        Stmt::ExprStmt(e) => {
          if let Some(result_reg) = self.compile_expr(e) {
            // result is discarded in an expression statement
            self.regs.free_reg(result_reg);
          } else {
            // compile_expr should report an error so nothing is needed here.
            // FIXME: this probably leaks registers...
          }
        },
        Stmt::Decl(name, init_val) => {
          let val = match init_val {
            crate::expr::DeclInit::Str(contents) => {
              self.stab.add_string(contents.clone())
            },
            crate::expr::DeclInit::Int(val) => format!("{}", val),
          };
          self.stab.decl(name.clone() /* PERF: avoid clone */, val);
        },
        Stmt::Assignment(name, value) => {
          let result = if let Some(result_reg) = self.compile_expr(value) {
            result_reg
          } else {
            return;
          };
          let var_label = match self.stab.get_var(&name) {
            Some(l) => l.to_string(), // helps with ownership trouble
            None => {
              eprintln!("variable not found: {}", name);
              return;
            },
          };

          let addr_reg = match self.regs.get_reg() {
            Some(r) => r,
            None => {
              eprintln!("unable to allocate register to hold address for {} in assignment", name);
              return;
            }
          };

          self.push(format!("sw {}, {}, {}", result, var_label, addr_reg));
          self.regs.free_reg(result);
          self.regs.free_reg(addr_reg);
        },
    }
  }

  pub fn compile_expr(&mut self, e: &Expr) -> Option<Reg> {
    match e {
      Expr::Lit(val) => {
        let reg = self.regs.get_reg().expect("failed to allocate reg for immediate");
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
            self.regs.free_reg(right);
            Some(left)
          },
          Sub => {
            self.push(format!("sub {}, {}, {}", left, left, right));
            self.regs.free_reg(right);
            Some(left)
          },
          Mul => {
            self.push(format!("mul {}, {}, {}", left, left, right));
            self.regs.free_reg(right);
            Some(left)
          },
          Div => {
            self.push(format!("div {}, {}, {}", left, left, right));
            self.regs.free_reg(right);
            Some(left)
          },
        }
      }
      Expr::Ident(name) => {
        let label = match self.stab.get_var(name) {
          Some(l) => l.to_string(), // helps with ownership trouble
          None => {
            eprintln!("variable not found: {}", name);
            return None;
          }
        };
        let r = match self.regs.get_reg() {
          Some(r) => r,
          None => {
            eprintln!("ERR: unable to allocate register for variable {}", name);
            return None;
          }
        };
        self.push(format!("lw {}, {}", r, label));
        Some(r)
      },
      Expr::Call(name, params) => {
        let mut all_ok = true;
        for (i, param_expr) in params.iter().enumerate() {
          if let Some(r) = self.compile_expr(param_expr) {
            self.push(format!("mv a{}, {}", i, r));
            self.regs.free_reg(r);
          } else {
            all_ok = false;
          }
        }

        if all_ok { // if the args are invalid, dont compile the call i guess.
          self.push(format!("call {}", name));
        } else {
          eprintln!("ERR: failed to compile call to {}", name);
        }

        Some(A0)
      },
      Expr::String(s) => {
        let lbl = self.stab.add_string(s.clone());
        let reg = self.regs.get_reg().expect("failed to get register for string");
        self.instrs.push(format!("la {}, {}", reg, lbl));
        Some(reg)
      },
    }
  }
}
