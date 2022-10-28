#[derive(Debug)]
pub enum TerlError {
    ExpectAVul(usize),
    ExpectASymbol(usize),
    MissBeacket(usize),
}
