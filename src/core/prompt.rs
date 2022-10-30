#[derive(Debug)]
pub enum TerlError {
    ExpectVul(usize),
    ExpectSymbol(usize),
    MissBeacket(usize),
    ExpectName(usize),
    ExpectTypeName(usize),
    ExpectOneOf(Vec<char>),
    ExpectNothing(usize),
    ExpectExpr(usize),
}
