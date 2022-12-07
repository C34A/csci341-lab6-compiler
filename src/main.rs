fn main() {
  let args: Vec<String> = std::env::args().collect();
  let mut args = args.iter();
  args.next();
  let path = args.next().map_or_else(|| "tests/if_simple.oh", |s| &s);

  let text = std::fs::read_to_string(path).expect(&format!("file {} not found.", path));

  let ast = compiler::parse::parse(&text).expect("parse failed");


  println!("{:#?}", ast);
  let mut builder = compiler::riscv::Compiler::new();
  builder.compile(ast);
  builder.dump();
}
