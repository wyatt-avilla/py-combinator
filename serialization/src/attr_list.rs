use std::fmt::Display;

use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Ident, Token};

use syn::token;

#[derive(Debug, Clone)]
pub struct AttributeArgsList(pub Vec<AttributeArg>);

#[derive(Debug, Clone)]
pub enum AttributeArg {
    KeyValueArg(KeyValueArg),
    Arg(Arg),
    Group(GroupArg),
}

#[derive(Debug, Clone)]
pub struct KeyValueArg {
    pub key: Ident,
    _eq_token: Token![=],
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub value: Ident,
}

#[derive(Debug, Clone)]
pub struct GroupArg {
    pub paren_token: token::Paren,
    pub content: AttributeArgsList,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Ident(Ident),
    Group(GroupArg),
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::Ident(ident) => write!(f, "{ident}"),
            AttributeValue::Group(group) => {
                write!(f, "(")?;
                for (i, arg) in group.content.0.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match arg {
                        AttributeArg::Arg(arg) => write!(f, "{arg}")?,
                        AttributeArg::KeyValueArg(kv) => write!(f, "{}={}", kv.key, kv.value)?,
                        AttributeArg::Group(g) => {
                            write!(f, "{}", AttributeValue::Group(g.clone()))?;
                        }
                    }
                }
                write!(f, ")")
            }
        }
    }
}

impl Parse for AttributeValue {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(token::Paren) {
            Ok(AttributeValue::Group(input.parse()?))
        } else {
            Ok(AttributeValue::Ident(input.parse()?))
        }
    }
}

impl Parse for GroupArg {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let parsed_content: AttributeArgsList = content.parse()?;

        Ok(GroupArg {
            paren_token,
            content: parsed_content,
        })
    }
}

impl Parse for KeyValueArg {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self {
            value: input.parse()?,
        })
    }
}

impl Parse for AttributeArg {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        if input.peek(token::Paren) {
            return Ok(AttributeArg::Group(input.parse()?));
        }

        let fork = input.fork();
        if fork.parse::<Ident>().is_ok() && fork.peek(Token![=]) {
            return Ok(AttributeArg::KeyValueArg(input.parse()?));
        }

        Ok(AttributeArg::Arg(input.parse()?))
    }
}

impl Parse for AttributeArgsList {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let args: Punctuated<AttributeArg, Token![,]> =
            input.parse_terminated(AttributeArg::parse, Token![,])?;
        Ok(AttributeArgsList(args.into_iter().collect()))
    }
}
