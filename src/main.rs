use terl::core::complier::Complier;

fn main() {
    let mut complier = Complier::new();
    complier.input("#").lex();
}
