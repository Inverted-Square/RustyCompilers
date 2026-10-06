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

    if let Some(parsed_result3) = parsers::nat().parse("\n \t 0001234hi") {
        println!("{parsed_result3:?}");
    }

    if let Some(parsed_result4) = parsers::boolean().parse("\n \t false0001234hi") {
        println!("{parsed_result4:?}");
    }

    if let Some(parsed_result5) = parsers::boolean().parse("\n \t true0001234hi") {
        println!("{parsed_result5:?}");
    }

    if let Some(parsed_result6) = parsers::expr().parse("1 ? 2\n\n ? 3 : 4 : 5 || 6 || 7") {
        println!("{parsed_result6:?}");
    }
}
