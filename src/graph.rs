#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub nodees: std::vec::Vec<Node>,
    pub routines: std::collections::HashMap<std::string::String, Routine>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Routine {
    pub start: usize,
    pub formals: std::vec::Vec<std::string::String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    Branch {
        next: usize,
    },
    Assign {
        name: std::string::String,
        value: std::boxed::Box<Expression>,
        next: usize,
    },
    Call {
        name: std::string::String,
        actuals: std::vec::Vec<Expression>,
        next: usize,
    },
    Receive {
        source: std::string::String,
        variable: std::string::String,
        next: usize,
    },
    Send {
        destination: std::string::String,
        variable: std::string::String,
        next: usize,
    },
    Offer {
        client: std::string::String,
        accepted: usize,
        denied: usize,
    },
    Accept {
        server: std::string::String,
        next: usize,
    },
    Deny {
        server: std::string::String,
        next: usize,
    },
    Close {
        name: std::string::String,
        next: usize,
    },
    End,
}

pub type Expression = crate::tree::Expression;
