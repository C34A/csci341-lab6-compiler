use std::{collections::HashMap};

use crate::expr::{Expr, Stmt, UnaryOp, Block};



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
  pub data: HashMap<String, (String, String)>, // name -> (label, initial value)
  pub strings: HashMap<String, String>, // label -> contents
}

impl SymTab {
  fn decl(&mut self, name: String, initial: String) -> Option<()> {
    if self.data.contains_key(&name) {
      eprintln!("ERR: Redeclaration of variable {}", name);
      return None;
    }
    let lbl = format!("__var_{}", &name);
    self.data.insert(name, (lbl, initial));
    Some(())
  }

  fn get_var(&self, name: &str) -> Option<&str> {
    self.data.get(name).map(|tup| &tup.0 as _)
  }

  fn add_string(&mut self, s: String) -> String {
    /* PERF: avoid cloning!! slow!! */
    if !self.strings.contains_key(&s) {
      self.strings.insert(s.clone(), format!("__str_{}", self.strings.len()));
    }
    self.strings.get(&s).unwrap().clone() 
  }

  fn dump_data_asm(&self) {
    println!(".data");
    for (contents, label) in &self.strings {
      println!(r#"{}: .asciz "{}" "#, label, contents);
    }
    for (label, initial) in self.data.values() {
      println!("{}:", label);
      println!("    .word {}", initial);
    }
    println!("");
  }

  fn new() -> Self {
    Self {data: HashMap::new(), strings: HashMap::new()}
  }
}


struct LabelCounter {
  count: u32
}

impl LabelCounter {
  fn next(&mut self) -> String {
    let ret = format!("__L_{}", self.count);
    self.count += 1;
    ret
  }

  fn new() -> Self { Self { count: 0 } }
}
pub struct Compiler {
  stab: SymTab,
  pub instrs: IBlock,
  regs: RegMap,
  label_counter: LabelCounter,
}

type IBlock = Vec<String>;

impl Compiler {

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
      stab: SymTab::new(),
      instrs: vec![],
      regs: RegMap{map: [RState::Free; 32]},
      label_counter: LabelCounter::new(),
    }
  }

  pub fn compile(&mut self, stmts: Vec<Stmt>) {
    let mut b = vec![];
    self.compile_block(&mut b, &stmts);
    self.instrs = b;
  }

  fn compile_block(&mut self, b: &mut IBlock, stmts: &Block) {
    stmts.iter().for_each(|stmt| self.compile_stmt(b, stmt));
  }

  fn compile_stmt(&mut self, b: &mut IBlock, s: &Stmt) {
    match s {
        Stmt::ExprStmt(e) => {
          if let Some(result_reg) = self.compile_expr(b, e) {
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
          let result = if let Some(result_reg) = self.compile_expr(b, value) {
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

          b.push(format!("sw {}, {}, {}", result, var_label, addr_reg));
          self.regs.free_reg(result);
          self.regs.free_reg(addr_reg);
        },
        Stmt::If(cond, true_block, false_block) => {
          let cond_result = self.compile_expr(b, cond);
          
          // use these later
          let false_label = self.label_counter.next();
          // if the condition failed to compile, always assume false so that the rest of the
          // if can still be compiled.
          b.push(format!("beqz {}, {}", cond_result.unwrap_or(ZERO), false_label));
          
          cond_result.map(|r| self.regs.free_reg(r)); // the condition isnt used in the body, so free it now.
          
          let mut true_iblock = vec![];
          self.compile_block(&mut true_iblock, true_block);
          
          // if there is an else block, compile it. We also need to build a jump into the true block in this case.
          let false_iblock = if false_block.is_some() {
            let mut false_iblock = vec![];
            self.compile_block(&mut false_iblock, false_block.as_ref().unwrap());

            Some(false_iblock)
          } else {
            None as _
          };

          // join true after jump
          b.append(&mut true_iblock);

          if let Some(mut false_iblock) = false_iblock {
            // if there is an else block, we need to:
            // - add another jump over the else block
            // - emit the "false" label
            // - append in the else code
            // - add the end label used by the "true" block
            let end_label = self.label_counter.next();
            b.push(format!("j {}", end_label));
            b.push(format!("{}:", false_label));
            b.append(&mut false_iblock);
            b.push(format!("{}:", end_label));
          } else {
            // if there is no else block, all that needs to be done is to complete the jump by emitting a label.
            b.push(format!("{}:", false_label));
          }
        }
    }
  }

  pub fn compile_expr(&mut self, b: &mut IBlock, e: &Expr) -> Option<Reg> {
    match e {
      Expr::Lit(val) => {
        let reg = self.regs.get_reg().expect("failed to allocate reg for immediate");
        if *val > u32::MAX as _ || *val < i32::MIN as _ {
          eprintln!("WARN: immediate {} is out of 32 bit range", val);
        }
        b.push(format!("li {}, {}", reg, val));
        Some(reg)
      },
      Expr::Bin(left, op, right) =>{
        use crate::expr::BinOp::*;

        let right = self.compile_expr(b, &right); // compiling right first helps with register management
        let left = self.compile_expr(b, &left);
        let left = left?;
        let right = right?;

        fn simple(this: &mut Compiler, b: &mut IBlock, mnemonic: &'static str, left: Reg, right: Reg) -> Option<Reg> {
          b.push(format!("{} {}, {}, {}", mnemonic, left, left, right));
          this.regs.free_reg(right);
          Some(left)
        }

        match op {
          Add => {
            simple(self, b, "add", left, right)
          },
          Sub => {
            simple(self, b, "sub", left, right)
          },
          Mul => {
            simple(self, b, "mul", left, right)
          },
          Div => {
            simple(self, b, "div", left, right)
          },
          Rem => {
            simple(self, b, "rem", left, right)
          }
          Srl => {
            // todo: implement immediate versions
            simple(self, b, "srl", left, right)
          },
          Sra => {
            // todo: implement immediate versions
            simple(self, b, "sra", left, right)
          },
          Sll => {
            // todo: implement immediate versions
            simple(self, b, "sll", left, right)
          },
          And => {
            // todo: implement immediate versions
            simple(self, b, "and", left, right)
          },
          Or => {
            // todo: implement immediate versions
            simple(self, b, "or", left, right)
          },
          Xor => {
            // todo: implement immediate versions
            simple(self, b, "xor", left, right)
          },
          Less => {
            // todo: implement immediate versions
            simple(self, b, "slt", left, right)
          },
          LessUnsigned => {
            // todo: implement immediate versions
            simple(self, b, "sltu", left, right)
          },
          Greater => {
            // todo: implement immediate versions
            simple(self, b, "sltu", right, left)
          },
          TestEq => {
            b.push(format!("xor {}, {}, {}", left, left, right));
            b.push(format!("seqz {}, {}", left, left));
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
        b.push(format!("lw {}, {}", r, label));
        Some(r)
      },
      Expr::Call(name, params) => {
        let mut all_ok = true;
        for (i, param_expr) in params.iter().enumerate() {
          if let Some(r) = self.compile_expr(b, param_expr) {
            b.push(format!("mv a{}, {}", i, r));
            self.regs.free_reg(r);
          } else {
            all_ok = false;
          }
        }

        if all_ok { // if the args are invalid, dont compile the call i guess.
          b.push(format!("call {}", name));
        } else {
          eprintln!("ERR: failed to compile call to {}", name);
        }

        Some(A0)
      },
      Expr::String(s) => {
        let lbl = self.stab.add_string(s.clone());
        let reg = self.regs.get_reg().expect("failed to get register for string");
        b.push(format!("la {}, {}", reg, lbl));
        Some(reg)
      },
      Expr::Unary(operator, operand) => {
        match operator {
          UnaryOp::Deref => {
            let operand_result = self.compile_expr(b, operand)?;
            b.push(format!("lw {}, ({})", operand_result, operand_result));
            Some(operand_result)
          },
          UnaryOp::BoolNot => {
            let operand_result = self.compile_expr(b, operand)?;
            b.push(format!("seqz {}, {}", operand_result, operand_result));
            Some(operand_result)
          },
          UnaryOp::Addr => {
            match operand.as_ref() {
              Expr::Lit(_) => {
                eprintln!("ERR: cannot take address of an immediate (put it in a variable)");
                None
              },
              Expr::String(_) => {
                eprintln!("ERR: cannot take address of string as strings are already addresses");
                None
              },
              Expr::Ident(name) => {
                let reg = self.regs.get_reg().expect("failed to get register for addressof temporary");
                let label = match self.stab.get_var(&name) {
                  Some(l) => l,
                  None => {
                    eprintln!("ERR: variable not found: {}", name);
                    return None;
                  },
                };
                b.push(format!("la {}, {}", reg, label));
                Some(reg)
              },
              _ => {
                eprintln!("ERR: cannot take address of a temporary value");
                None
              }
            }
          },
          UnaryOp::Neg => {
            let operand_result = self.compile_expr(b, operand)?;
            b.push(format!("sub {}, x0, {}", operand_result, operand_result));
            Some(operand_result)
          },
          UnaryOp::Not => {
            let operand_result = self.compile_expr(b, operand)?;
            b.push(format!("xori {}, {}, -1", operand_result, operand_result));
            Some(operand_result)
          },
        }
      },
    }
  }
}
