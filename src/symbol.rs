#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub symbol_type: TypeSymbol,
}

#[derive(Debug, Clone)]
pub struct BuildInSymbol {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub name: String,
    pub param: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TypeSymbol {
    Number,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Symbol {
    VarSymbol(VarSymbol),
    BuiltInSymbol(BuildInSymbol),
    FunctionSymbol(FunctionSymbol),
}
