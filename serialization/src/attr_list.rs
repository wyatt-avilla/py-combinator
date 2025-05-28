use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Ident, Token};

#[derive(Debug, Clone)]
pub struct AttributeArgsList(pub Vec<AttributeArg>);

#[derive(Debug, Clone)]
pub enum AttributeArg {
    KeyValueArg(KeyValueArg),
    Arg(Arg),
}

#[derive(Debug, Clone)]
pub struct KeyValueArg {
    pub key: Ident,
    _eq_token: Token![=],
    pub value: Ident,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub value: Ident,
}

impl ToString for Arg {
    fn to_string(&self) -> String {
        self.value.to_string()
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
        let fork = input.fork();
        if fork.parse::<KeyValueArg>().is_ok() {
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
