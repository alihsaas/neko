#[derive(Debug)]
pub struct VarSymbol {
    pub name: String,
    pub symbol_type: TypeSymbol,
}

#[derive(Debug)]
pub enum BuiltInSymbol {}

#[derive(Debug)]
pub enum TypeSymbol {
    Number,
    Unknown,
}

#[derive(Debug)]
pub enum Symbol {
    VarSymbol(Box<VarSymbol>),
    BuiltInSymbol(BuiltInSymbol),
}
