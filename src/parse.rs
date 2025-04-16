#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    UnexpectedEnd,
    UnexpectedToken(crate::token::Token),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedEnd => {
                write!(f, "unexpected end")?;
            }
            Error::UnexpectedToken(token) => {
                write!(
                    f,
                    "unexpected token {:?} at {}",
                    token.value, token.location
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub fn parse(
    tokens: impl std::iter::Iterator<
        Item = std::result::Result<crate::token::Token, std::boxed::Box<dyn std::error::Error>>,
    >,
) -> std::result::Result<crate::tree::Tree, std::boxed::Box<dyn std::error::Error>> {
    let mut parser = Parser::from_tokens(tokens)?;
    let expression = parser.parse_start()?;
    if let Some(token) = parser.peek() {
        Err(std::boxed::Box::new(Error::UnexpectedToken(token)))
    } else {
        Ok(expression)
    }
}

struct Parser<I> {
    tokens: I,
    lookahead: std::option::Option<crate::token::Token>,
}

impl<
    I: std::iter::Iterator<
            Item = std::result::Result<crate::token::Token, std::boxed::Box<dyn std::error::Error>>,
        >,
> Parser<I>
{
    fn from_tokens(
        mut tokens: I,
    ) -> std::result::Result<Self, std::boxed::Box<dyn std::error::Error>> {
        let lookahead = tokens.next().transpose()?;
        Ok(Parser { tokens, lookahead })
    }
    fn peek(&self) -> std::option::Option<crate::token::Token> {
        self.lookahead.clone()
    }
    fn advance(&mut self) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        self.lookahead = self.tokens.next().transpose()?;
        Ok(())
    }
    fn expect(
        &mut self,
        value: crate::token::TokenValue,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Err(std::boxed::Box::new(Error::UnexpectedEnd));
        };
        if value == token.value {
            self.advance()?;
            Ok(())
        } else {
            Err(std::boxed::Box::new(Error::UnexpectedToken(token)))
        }
    }
    fn parse_identifier(
        &mut self,
    ) -> std::result::Result<std::string::String, std::boxed::Box<dyn std::error::Error>> {
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Err(std::boxed::Box::new(Error::UnexpectedEnd));
        };
        if let crate::token::TokenValue::Identifier(name) = token.value {
            self.advance()?;
            Ok(name)
        } else {
            Err(std::boxed::Box::new(Error::UnexpectedToken(token)))
        }
    }
    fn parse_start(
        &mut self,
    ) -> std::result::Result<crate::tree::Tree, std::boxed::Box<dyn std::error::Error>> {
        let mut typees = vec![];
        let mut routinees = vec![];
        while let Some(token) = self.peek() {
            match token.value {
                crate::token::TokenValue::Type => typees.push(self.parse_type()?),
                crate::token::TokenValue::Routine => routinees.push(self.parse_routine()?),
                _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
            }
        }
        Ok(crate::tree::Tree { typees, routinees })
    }
    fn parse_type(
        &mut self,
    ) -> std::result::Result<crate::tree::Type, std::boxed::Box<dyn std::error::Error>> {
        self.expect(crate::token::TokenValue::Type)?;
        let name = self.parse_identifier()?;
        self.expect(crate::token::TokenValue::Equals)?;
        let value = self.parse_multiplicative()?;
        Ok(crate::tree::Type {
            name,
            value: std::boxed::Box::new(value),
        })
    }
    fn parse_multiplicative(
        &mut self,
    ) -> std::result::Result<crate::tree::TypeExpression, std::boxed::Box<dyn std::error::Error>>
    {
        let left = self.parse_additive()?;
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Ok(left);
        };
        match token.value {
            crate::token::TokenValue::Lollipop => {
                self.advance()?;
                let next = self.parse_multiplicative()?;
                Ok(crate::tree::TypeExpression::Lollipop {
                    value: std::boxed::Box::new(left),
                    next: std::boxed::Box::new(next),
                })
            }
            crate::token::TokenValue::Times => {
                self.advance()?;
                let next = self.parse_multiplicative()?;
                Ok(crate::tree::TypeExpression::Times {
                    value: std::boxed::Box::new(left),
                    next: std::boxed::Box::new(next),
                })
            }
            _ => Ok(left),
        }
    }
    fn parse_additive(
        &mut self,
    ) -> std::result::Result<crate::tree::TypeExpression, std::boxed::Box<dyn std::error::Error>>
    {
        let left = self.parse_primary()?;
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Ok(left);
        };
        match token.value {
            crate::token::TokenValue::With => {
                self.advance()?;
                let next = self.parse_additive()?;
                Ok(crate::tree::TypeExpression::With {
                    accept: std::boxed::Box::new(left),
                    deny: std::boxed::Box::new(next),
                })
            }
            crate::token::TokenValue::Plus => {
                self.advance()?;
                let next = self.parse_additive()?;
                Ok(crate::tree::TypeExpression::Plus {
                    accept: std::boxed::Box::new(left),
                    deny: std::boxed::Box::new(next),
                })
            }
            _ => Ok(left),
        }
    }
    fn parse_primary(
        &mut self,
    ) -> std::result::Result<crate::tree::TypeExpression, std::boxed::Box<dyn std::error::Error>>
    {
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Err(std::boxed::Box::new(Error::UnexpectedEnd));
        };
        match token.value {
            crate::token::TokenValue::LeftParenthesis => {
                self.expect(crate::token::TokenValue::LeftParenthesis)?;
                let expression = self.parse_multiplicative()?;
                self.expect(crate::token::TokenValue::RightParenthesis)?;
                Ok(expression)
            }
            crate::token::TokenValue::Identifier(name) => {
                self.advance()?;
                Ok(crate::tree::TypeExpression::Variable {
                    name,
                    is_dual: false,
                })
            }
            crate::token::TokenValue::Times => {
                self.advance()?;
                let name = self.parse_identifier()?;
                Ok(crate::tree::TypeExpression::Variable {
                    name,
                    is_dual: true,
                })
            }
            crate::token::TokenValue::One => {
                self.advance()?;
                Ok(crate::tree::TypeExpression::One)
            }
            _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
        }
    }
    fn parse_routine(
        &mut self,
    ) -> std::result::Result<crate::tree::Routine, std::boxed::Box<dyn std::error::Error>> {
        self.expect(crate::token::TokenValue::Routine)?;
        let name = self.parse_identifier()?;
        self.expect(crate::token::TokenValue::LeftParenthesis)?;
        let mut formals = vec![self.parse_formal()?];
        loop {
            let token = if let Some(token) = self.peek() {
                token
            } else {
                return Err(std::boxed::Box::new(Error::UnexpectedEnd));
            };
            match token.value {
                crate::token::TokenValue::Comma => self.advance()?,
                crate::token::TokenValue::RightParenthesis => break,
                _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
            }
            formals.push(self.parse_formal()?);
        }
        self.expect(crate::token::TokenValue::RightParenthesis)?;
        self.expect(crate::token::TokenValue::LeftBrace)?;
        let mut body = vec![];
        loop {
            let token = if let Some(token) = self.peek() {
                token
            } else {
                return Err(std::boxed::Box::new(Error::UnexpectedEnd));
            };
            if let crate::token::TokenValue::RightBrace = token.value {
                break;
            }
            body.push(self.parse_statement()?);
        }
        self.expect(crate::token::TokenValue::RightBrace)?;
        Ok(crate::tree::Routine {
            name,
            formals,
            body,
        })
    }
    fn parse_formal(
        &mut self,
    ) -> std::result::Result<crate::tree::Formal, std::boxed::Box<dyn std::error::Error>> {
        let name = self.parse_identifier()?;
        self.expect(crate::token::TokenValue::Colon)?;
        let r#type = self.parse_multiplicative()?;
        Ok(crate::tree::Formal { name, r#type })
    }
    fn parse_statement(
        &mut self,
    ) -> std::result::Result<crate::tree::Statement, std::boxed::Box<dyn std::error::Error>> {
        let identifier = self.parse_identifier()?;
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Ok(crate::tree::Statement::Branch { name: identifier });
        };
        match token.value {
            crate::token::TokenValue::Colon => {
                self.advance()?;
                Ok(crate::tree::Statement::Label { name: identifier })
            }
            crate::token::TokenValue::Equals => {
                self.advance()?;
                let value = self.parse_expression()?;
                Ok(crate::tree::Statement::Assign {
                    name: identifier,
                    value: std::boxed::Box::new(value),
                })
            }
            crate::token::TokenValue::LeftParenthesis => {
                self.expect(crate::token::TokenValue::LeftParenthesis)?;
                let mut actuals = vec![self.parse_expression()?];
                loop {
                    let token = if let Some(token) = self.peek() {
                        token
                    } else {
                        return Err(std::boxed::Box::new(Error::UnexpectedEnd));
                    };
                    match token.value {
                        crate::token::TokenValue::Comma => self.advance()?,
                        crate::token::TokenValue::RightParenthesis => break,
                        _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
                    }
                    actuals.push(self.parse_expression()?);
                }
                self.expect(crate::token::TokenValue::RightParenthesis)?;
                Ok(crate::tree::Statement::Call {
                    name: identifier,
                    actuals,
                })
            }
            crate::token::TokenValue::Receive => {
                self.advance()?;
                let variable = self.parse_identifier()?;
                Ok(crate::tree::Statement::Receive {
                    source: identifier,
                    variable,
                })
            }
            crate::token::TokenValue::Send => {
                self.advance()?;
                let variable = self.parse_identifier()?;
                Ok(crate::tree::Statement::Send {
                    destination: identifier,
                    variable,
                })
            }
            crate::token::TokenValue::Offer => {
                self.advance()?;
                self.expect(crate::token::TokenValue::LeftBrace)?;
                let mut accepted = vec![];
                loop {
                    let token = if let Some(token) = self.peek() {
                        token
                    } else {
                        return Err(std::boxed::Box::new(Error::UnexpectedEnd));
                    };
                    if token.value == crate::token::TokenValue::RightBrace {
                        break;
                    }
                    accepted.push(self.parse_statement()?);
                }
                self.expect(crate::token::TokenValue::RightBrace)?;
                self.expect(crate::token::TokenValue::Else)?;
                self.expect(crate::token::TokenValue::LeftBrace)?;
                let mut denied = vec![];
                loop {
                    let token = if let Some(token) = self.peek() {
                        token
                    } else {
                        return Err(std::boxed::Box::new(Error::UnexpectedEnd));
                    };
                    if token.value == crate::token::TokenValue::RightBrace {
                        break;
                    }
                    denied.push(self.parse_statement()?);
                }
                self.expect(crate::token::TokenValue::RightBrace)?;
                Ok(crate::tree::Statement::Offer {
                    client: identifier,
                    accepted,
                    denied,
                })
            }
            crate::token::TokenValue::Accept => {
                self.advance()?;
                Ok(crate::tree::Statement::Accept { server: identifier })
            }
            crate::token::TokenValue::Deny => {
                self.advance()?;
                Ok(crate::tree::Statement::Deny { server: identifier })
            }
            crate::token::TokenValue::Close => {
                self.advance()?;
                Ok(crate::tree::Statement::Close { name: identifier })
            }
            _ => Ok(crate::tree::Statement::Branch { name: identifier }),
        }
    }
    fn parse_expression(
        &mut self,
    ) -> std::result::Result<crate::tree::Expression, std::boxed::Box<dyn std::error::Error>> {
        let identifier = self.parse_identifier()?;
        let token = if let Some(token) = self.peek() {
            token
        } else {
            return Ok(crate::tree::Expression::Variable { name: identifier });
        };
        match token.value {
            crate::token::TokenValue::Comma => {
                return Ok(crate::tree::Expression::Variable { name: identifier });
            }
            crate::token::TokenValue::RightParenthesis => {
                return Ok(crate::tree::Expression::Variable { name: identifier });
            }
            crate::token::TokenValue::Identifier(_) => {
                return Ok(crate::tree::Expression::Variable { name: identifier });
            }
            crate::token::TokenValue::LeftParenthesis => {}
            _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
        }
        self.expect(crate::token::TokenValue::LeftParenthesis)?;
        let mut before = vec![];
        loop {
            let token = if let Some(token) = self.peek() {
                token
            } else {
                return Err(std::boxed::Box::new(Error::UnexpectedEnd));
            };
            match token.value {
                crate::token::TokenValue::RightParenthesis => break,
                crate::token::TokenValue::Comma => break,
                _ => {}
            }
            before.push(self.parse_expression()?);
            self.expect(crate::token::TokenValue::Comma)?;
        }
        let mut after = vec![];
        loop {
            let token = if let Some(token) = self.peek() {
                token
            } else {
                return Err(std::boxed::Box::new(Error::UnexpectedEnd));
            };
            match token.value {
                crate::token::TokenValue::RightParenthesis => break,
                crate::token::TokenValue::Comma => self.advance()?,
                _ => return Err(std::boxed::Box::new(Error::UnexpectedToken(token))),
            }
            after.push(self.parse_expression()?);
        }
        self.expect(crate::token::TokenValue::RightParenthesis)?;
        Ok(crate::tree::Expression::Call {
            name: identifier,
            before,
            after,
        })
    }
}
