#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tree {
    pub typees: std::vec::Vec<Type>,
    pub routinees: std::vec::Vec<Routine>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type {
    pub name: std::string::String,
    pub value: std::boxed::Box<TypeExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeExpression {
    Variable {
        name: std::string::String,
        is_dual: bool,
    },
    Lollipop {
        value: std::boxed::Box<TypeExpression>,
        next: std::boxed::Box<TypeExpression>,
    },
    Times {
        value: std::boxed::Box<TypeExpression>,
        next: std::boxed::Box<TypeExpression>,
    },
    With {
        accept: std::boxed::Box<TypeExpression>,
        deny: std::boxed::Box<TypeExpression>,
    },
    Plus {
        accept: std::boxed::Box<TypeExpression>,
        deny: std::boxed::Box<TypeExpression>,
    },
    One,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Routine {
    pub name: std::string::String,
    pub formals: std::vec::Vec<Formal>,
    pub body: std::vec::Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Formal {
    pub name: std::string::String,
    pub r#type: TypeExpression,
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
    Connect {
        left: std::string::String,
        right: std::string::String,
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
