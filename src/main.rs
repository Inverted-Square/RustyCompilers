use crate::parser::parsers;

mod monadic_type_classes;
mod parser;

fn main() {
    if let Some(parsed_result1) = parsers::string("hello").parse("\n \t hello, how are you") {
        println!("{parsed_result1:?}");
    }

    let c = 'h';
    if let Some(parsed_result2) = parsers::character(c).parse("\n \t hello, how are you") {
        println!("{parsed_result2:?}");
    }
}
