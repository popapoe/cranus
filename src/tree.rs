#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tree {
    pub routines: std::vec::Vec<Routine>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Routine {
    pub name: std::string::String,
    pub formals: std::vec::Vec<std::string::String>,
    pub body: std::vec::Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Statement {
    Branch {
        name: std::string::String,
    },
    Label {
        name: std::string::String,
    },
    Assign {
        name: std::string::String,
        value: std::boxed::Box<Expression>,
    },
    Call {
        name: std::string::String,
        actuals: std::vec::Vec<Expression>,
    },
    Receive {
        source: std::string::String,
        variable: std::string::String,
    },
    Send {
        destination: std::string::String,
        variable: std::string::String,
    },
    Offer {
        client: std::string::String,
        accepted: std::vec::Vec<Statement>,
        denied: std::vec::Vec<Statement>,
    },
    Accept {
        server: std::string::String,
    },
    Deny {
        server: std::string::String,
    },
    Close {
        name: std::string::String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expression {
    Variable {
        name: std::string::String,
    },
    Call {
        name: std::string::String,
        before: std::vec::Vec<Expression>,
        after: std::vec::Vec<Expression>,
    },
}
