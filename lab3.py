#!/usr/bin/env python3

"""
Lab3 - single instruction assembler
2022-10-11
Theo Marshall
CSCI 341 section C
"""

import csv
from typing import Dict, List
import re

def bin_length(n: int, digits: int) -> str:
  """take an int n, and return a string of it in binary with a given number of digits.
  This will perform sign-extension or zero-extension based on the sign of N.
  """
  if n < 0:
    mask = 1 # manually do sign extension for negative numbers
    for i in range(digits):
      mask <<= 1
      mask |= 1
    return bin(n & mask)[-digits:]
  else: # use built in bin() for positive, and extend with zeroes
    b = bin(n)[2:]
    if len(b) < digits:
      b = ("0" * (digits - len(b))) + b
    return b

# read instructions from csv, and form table mapping from name to template
inst_table = {}
with open("RISC-V_Instructions.csv") as instr_file:
  reader = csv.reader(instr_file)
  next(reader) # skip header row
  for row in reader:
    inst_table[row[0]] = {
      "opcode": row[1],
      "funct3": row[2],
      "funct7": row[3],
      "type": row[4],
      "imm": row[5],
    }



# read register data from csv, and form table mapping from names to binary representation
reg_table = {}
with open("Registers.csv") as reg_file:
  reader = csv.reader(reg_file)
  next(reader) # skip header row
  for row in reader:
    n = int(row[0].strip()[1:])
    if n < 0 or n > 31:
      raise Exception("invalid register number: " + str(n))
    nb = bin_length(n, 5)
    reg_table[row[0]] = nb
    reg_table[row[1]] = nb

def parse(input: str) -> List[str]:
  """split an instruction into parts, based on commas and whitespace.
  >>>parse('addi x0, x0, 1')
  ['addi', 'x0', 'x0', '1']
  """
  l1 = [s.strip() for s in input.split(",")]
  l2 = [s.strip() for s in l1[0].split()]
  return l2 + l1[1:]

def assemble_r_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble an R-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  """

  # assuming the format [mnemonic, rd, rs1, rs2]

  rd = reg_table[parsed[1]]
  rs1 = reg_table[parsed[2]]
  rs2 = reg_table[parsed[3]]

  return f"{template['funct7']}{rs2}{rs1}{template['funct3']}{rd}{template['opcode']}"

# regex used by various functions to parse immediates, registers, and dereferences
re_decimal_imm = re.compile("^(-)?\\d+$") # decimal immediates (123, -4096, etc)
re_hex_imm = re.compile("^0x[\dabcdefABCDEF]+") # hes immediates (0xbeefcafe)

re_reg = re.compile("^[a-zA-Z]+[0-9]*$") # registers (x0-x31, probably excludes stuff like ra (oops))

# register offsets (123(x10), etc)
re_offset_reg = re.compile("^(?P<off>-?\\d+)\\((?P<reg>[a-zA-Z]+[0-9]*)\\)$")
# zero offsets ("(sp)", etc)
re_offset_zero_reg = re.compile("^\\((?P<reg>[a-zA-Z]+[0-9]*)\\)$")

def assemble_i_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble an I-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  """
  rd = None
  rs1 = None

  imm_bin = None
  if template["imm"] != " - ": # ecall and ebreak need special handling
    imm_bin = bin_length(int(template["imm"], base=2), 12) # get immediate value from template and extend
    rd = reg_table["x0"] # these are both always zero
    rs1 = reg_table["x0"]
  else: # anything other than ecall and ebreak
    rd = reg_table[parsed[1]] # there will always be a destination register
    imm_input = None
    # the immediate and rs1 depend on the instruction
    if re_reg.match(parsed[2]): # instructions such as addi x0, x0, etc
      rs1 = reg_table[parsed[2]]
      imm_input = parsed[3]
    elif re_offset_reg.match(parsed[2]): # instructions such as lw x0, 123(x10)
      first, second = re_offset_reg.match(parsed[2]).groups()
      imm_input = first
      rs1 = second # "x6", "sp", etc
      rs1 = reg_table.get(rs1) # "00110", etc
    elif re_offset_zero_reg.match(parsed[2]): # instructions such as lw x0, (sp)
      imm_input = "0"
      rs1 = str(re_offset_zero_reg.match(parsed[2]).group())[1:-1] # "x9", etc
      rs1 = reg_table.get(rs1) # "01001", etc
    else:
      raise Exception("invalid input for I type instruction")

    imm = None # parse immediates...
    if re_decimal_imm.match(imm_input): # decimal
      imm = int(imm_input)
    elif re_hex_imm.match(imm_input): # hex
      imm = int(imm_input, base=16)
    else:
      return f"invalid immediate: {imm_input}, expected decimal or hex literal."

    imm_bin = bin_length(imm, 12)
  
  return f"{imm_bin}{rs1}{template['funct3']}{rd}{template['opcode']}"

def assemble_s_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble an S-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  eg: parsed = ['sw', 'x0', '8(sp)']
  """
  rs2 = reg_table.get(parsed[1]) # rs2 will always be the first parameter

  rs1 = None # format of rs1 may vary
  offset = 0
  offset_reg_match = re_offset_reg.match(parsed[2])
  offset_zero_match = re_offset_zero_reg.match(parsed[2])
  if offset_reg_match: # 123(reg) format
    first, second = offset_reg_match.groups()
    offset = int(first)
    rs1 = second
    rs1 = reg_table.get(rs1)
  elif offset_zero_match: # (reg) format
    rs1 = str(offset_zero_match.group())[1:-1]
    rs1 = reg_table.get(rs1)
    offset = 0
  else: # invalid
    raise Exception("invalid register and/or offset")
  
  # binary form
  offset_bin = bin_length(offset, 12)
  # 1010_1010_1010
  # ba98 7654 3210 <- order in green card
  # 0123 4567 89ab <- index
  offset_11_5 = offset_bin[0:7]
  offset_4_0 = offset_bin[7:12]

  return f"{offset_11_5}{rs2}{rs1}{template['funct3']}{offset_4_0}{template['opcode']}"

def assemble_b_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble a SB-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  eg: parsed = ['beq', 'x0', 'x10', '1234']
  """
  
  rs1 = None
  rs2 = None
  imm = None
  # Make sure registers are correctly formated
  if re_reg.match(parsed[1]):
    rs1 = reg_table[parsed[1]]
  if re_reg.match(parsed[2]):
    rs2 = reg_table[parsed[2]]
    imm_input = parsed[3]
  else:
    raise Exception("invalid input for B type instruction")

  # Parse address to branch to
  imm = None
  if re_decimal_imm.match(imm_input):
    imm = int(imm_input)
  elif re_hex_imm.match(imm_input):
    imm = int(imm_input, base=16)
  else:
    return f"invalid immediate: {imm_input}, expected decimal or hex literal."

  # Assemble

  imm_bin = bin_length(imm, 13)
  # 0_1010_1010_1010
  # c ba98 7654 3210 <- order in green card
  # 0 1234 5678 9abc <- index
  imm_12 = imm_bin[0]
  imm_10_5 = imm_bin[2:8]
  imm_4_1 = imm_bin[8:12]
  imm_11 = imm_bin[1]
  
  return f"{imm_12}{imm_10_5}{rs2}{rs1}{template['funct3']}{imm_4_1}{imm_11}{template['opcode']}"

def assemble_u_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble a UJ-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  eg: parsed = ['lui', 'x10', '1234']
  """

  rd = reg_table[parsed[1]] # First parameter will always be the register
  imm_input = parsed[2]
  imm = None 
  # Parse immediate
  if re_decimal_imm.match(imm_input):
    imm = int(imm_input)
  elif re_hex_imm.match(imm_input):
    imm = int(imm_input, base=16)
  else:
    return f"invalid immediate: {imm_input}, expected decimal or hex literal."
  
  # Simple bounds check
  if imm > 524287 or imm < -524288:
    return f"immediate out of range: {imm}"
  
  # Assemble

  imm_bin = bin_length(imm, 20)

  return f"{imm_bin}{rd}{template['opcode']}"

def assemble_j_type(parsed: List[str], template: Dict[str, str]) -> str:
  """Assemble a J-type instruction.
  `parsed` is expected to be passed from `parse()`, and `template` is expected to be a valid instruction template from
  inst_table.
  eg: parsed = ['jal', 'x10', '1234']
  """

  rd = reg_table[parsed[1]] # First parameter will always be rd
  imm_input = parsed[2]
  imm = None
  # Parse immediate address
  if re_decimal_imm.match(imm_input):
    imm = int(imm_input)
  elif re_hex_imm.match(imm_input):
    imm = int(imm_input, base=16)
  else:
    return f"invalid immediate: {imm_input}, expected decimal or hex literal."
  
  # Assemble
  imm_bin = bin_length(imm, 32)
  imm_20 = imm_bin[-21]
  imm_10_1 = imm_bin[-11:-1]
  imm_11 = imm_bin[-12]
  imm_19_12 = imm_bin[-20:-12]

  return f"{imm_20}{imm_10_1}{imm_11}{imm_19_12}{rd}{template['opcode']}"

def assemble(input: str) -> str:
  """Assembles an instruction, taking user input and returning the binary form as a string.
  input should be any valid instruction, output will be an error message for some
  failures, and exceptions will be raised for some other failures.
  """
  p = parse(input) # break the input into its parts

  # make sure this is a valid instruction, and get its template
  inst_template = inst_table.get(p[0])
  if inst_template is None:
    return f"invalid instruction: {p[0]}"
  
  # assemble based on the type of instruction
  type_func_table = {
    "R": assemble_r_type,
    "I": assemble_i_type,
    "S": assemble_s_type,
    "B": assemble_b_type,
    "J": assemble_j_type,
    "U": assemble_u_type,
  }
  return type_func_table[inst_template["type"]](p, inst_template)

# This code will run if this script is run manually, but not if it is imported
 # (e.g. by test.py)
if __name__ == "__main__":
  # Solicit an instruction and print the assembled binary representation
  user_input = input("Enter an Instruction: ")
  while user_input != "quit":
    output = assemble(user_input)
    print("Result:", output)
    user_input = input("Enter an Instruction: ")

