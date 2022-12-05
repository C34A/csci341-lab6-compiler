fn main() {
  let args: Vec<String> = std::env::args().collect();
  let mut args = args.iter();
  args.next();
  // let path = args.next().expect("filename expected");
  let path = &String::from("tests/vars.oh");

  let text = std::fs::read_to_string(path).expect(&format!("file {} not found.", path));

  let ast = compiler::parse::parse(&text).expect("parse failed");

  // let mut builder = compiler::riscv::RVBuilder::new();
  // builder.compile_expr(&ast).expect("compile failed");
  // builder.dump();

  println!("{:?}", ast);
}
