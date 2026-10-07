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

    if let Some(parsed_result3) = parsers::expr().parse("13 + (13 / 6 + 7) * 2 - -2") {
        println!("{parsed_result3:?}");
    }

    if let Some(parsed_result4) = parsers::expr().parse(
        "12 <= 3 ?
    1 + 3
:
    (17) - 10 / 2 != 11 ?
        (4 - -2) * 7
    :
        !(8 > 8)",
    ) {
        println!("{parsed_result4:?}");
    }
}
