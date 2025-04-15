#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenValue {
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Equals,
    Lollipop,
    Times,
    With,
    Plus,
    One,
    Type,
    Routine,
    Receive,
    Send,
    Offer,
    Else,
    Accept,
    Deny,
    Close,
    Identifier(std::string::String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub value: TokenValue,
    pub location: crate::location::Location,
}
