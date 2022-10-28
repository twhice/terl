use terl::core::complier::Complier;

fn main() {
    let mut complier = Complier::new();
    complier.input("a=(a,b,c):qicq").lex();
}
