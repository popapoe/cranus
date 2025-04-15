lazy_static::lazy_static! {
    static ref KEYWORDS: std::collections::HashMap<&'static str, crate::token::TokenValue> = {
        let mut map = std::collections::HashMap::new();
        map.insert("receive", crate::token::TokenValue::Receive);
        map.insert("send", crate::token::TokenValue::Send);
        map.insert("offer", crate::token::TokenValue::Offer);
        map.insert("else", crate::token::TokenValue::Else);
        map.insert("accept", crate::token::TokenValue::Accept);
        map.insert("deny", crate::token::TokenValue::Deny);
        map.insert("close", crate::token::TokenValue::Close);
        map
    };
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter {
        character: char,
        location: crate::location::Location,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedCharacter {
                character,
                location,
            } => {
                write!(f, "unexpected character {:?} at {}", character, location)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub fn scan(
    characters: impl std::iter::Iterator<
        Item = std::result::Result<char, std::boxed::Box<dyn std::error::Error>>,
    >,
) -> std::result::Result<
    impl std::iter::Iterator<
        Item = std::result::Result<crate::token::Token, Box<dyn std::error::Error>>,
    >,
    std::boxed::Box<dyn std::error::Error>,
> {
    Ok(Scanner::from_characters(characters)?)
}

pub struct Scanner<I> {
    characters: I,
    lookahead: std::option::Option<char>,
    location: crate::location::Location,
}

impl<
    I: std::iter::Iterator<Item = std::result::Result<char, std::boxed::Box<dyn std::error::Error>>>,
> std::iter::Iterator for Scanner<I>
{
    type Item = std::result::Result<crate::token::Token, Box<dyn std::error::Error>>;
    fn next(&mut self) -> std::option::Option<Self::Item> {
        self.read_token().transpose()
    }
}

impl<
    I: std::iter::Iterator<Item = std::result::Result<char, std::boxed::Box<dyn std::error::Error>>>,
> Scanner<I>
{
    fn from_characters(
        mut characters: I,
    ) -> std::result::Result<Self, std::boxed::Box<dyn std::error::Error>> {
        let lookahead = characters.next().transpose()?;
        Ok(Scanner {
            characters,
            lookahead,
            location: crate::location::Location::from_indexs(1, 1),
        })
    }
    fn peek(&self) -> std::option::Option<char> {
        self.lookahead
    }
    fn advance(&mut self) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        let character = self.lookahead.unwrap();
        if character == '\n' {
            self.location.next_line();
        } else {
            self.location.next_column();
        }
        self.lookahead = self.characters.next().transpose()?;
        Ok(())
    }
    fn skip_whitespace(
        &mut self,
    ) -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
        while let Some(character) = self.peek() {
            if !character.is_ascii_whitespace() {
                break;
            }
            self.advance()?;
        }
        Ok(())
    }
    fn read_word(
        &mut self,
    ) -> std::result::Result<std::string::String, std::boxed::Box<dyn std::error::Error>> {
        let mut word = String::new();
        while let Some(character) = self.peek() {
            if !character.is_ascii_alphabetic() && !character.is_ascii_digit() && character != '_' {
                break;
            }
            word.push(character);
            self.advance()?;
        }
        Ok(word)
    }
    fn read_token(
        &mut self,
    ) -> std::result::Result<
        std::option::Option<crate::token::Token>,
        std::boxed::Box<dyn std::error::Error>,
    > {
        self.skip_whitespace()?;
        let character = if let Some(character) = self.peek() {
            character
        } else {
            return Ok(None);
        };
        let location = self.location;
        let value = match character {
            '(' => {
                self.advance()?;
                crate::token::TokenValue::LeftParenthesis
            }
            ')' => {
                self.advance()?;
                crate::token::TokenValue::RightParenthesis
            }
            '{' => {
                self.advance()?;
                crate::token::TokenValue::LeftBrace
            }
            '}' => {
                self.advance()?;
                crate::token::TokenValue::RightBrace
            }
            ',' => {
                self.advance()?;
                crate::token::TokenValue::Comma
            }
            ':' => {
                self.advance()?;
                crate::token::TokenValue::Colon
            }
            '=' => {
                self.advance()?;
                crate::token::TokenValue::Equals
            }
            character => {
                if character.is_ascii_alphabetic() || character == '_' {
                    let word = self.read_word()?;
                    if let Some(value) = KEYWORDS.get(&*word) {
                        value.clone()
                    } else {
                        crate::token::TokenValue::Identifier(word)
                    }
                } else {
                    return Err(std::boxed::Box::new(Error::UnexpectedCharacter {
                        character,
                        location,
                    }));
                }
            }
        };
        Ok(Some(crate::token::Token { value, location }))
    }
}
