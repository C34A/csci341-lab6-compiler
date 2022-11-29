fn main() {
  let args: Vec<String> = std::env::args().collect();
  let mut args = args.iter();
  args.next();
  // let path = args.next().expect("filename expected");
  let path = &String::from("tests/expr.oh");

  let text = std::fs::read_to_string(path).expect(&format!("file {} not found.", path));

  let parse = compiler::parse::parse(&text);

  println!("{:#?}", parse)
}
