use nom::combinator::verify;
use nom::error::{context, ErrorKind, ParseError};
use nom::multi::{many0, many1};
use nom::IResult;

use crate::ast::ModuleStmt::*;
use crate::ast::*;
use crate::errors::make_error;

use crate::tokenizer::tokenize::tokenize;
use crate::tokenizer::types::{TokenInfo, TokenType};

pub type TokenRef<'a> = &'a TokenInfo<'a>;
pub type TokenSlice<'a> = &'a [TokenInfo<'a>];
pub type TokenResult<'a, O, E> = IResult<TokenSlice<'a>, O, E>;

/// Tokenize the given source code in `source` and filter out tokens not relevant to parsing.
pub fn get_parse_tokens<'a>(source: &'a str) -> Result<Vec<TokenInfo<'a>>, String> {
    let tokens = tokenize(source)?;

    Ok(tokens
        .into_iter()
        .filter(|t| t.typ != TokenType::NL && t.typ != TokenType::COMMENT)
        .collect())
}

/// Parse a single token from a token slice.
pub fn one_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    match input.iter().next() {
        None => make_error(input, ErrorKind::Eof),
        Some(token) => Ok((&input[1..], token)),
    }
}

/// Parse a token of a specific type from a token slice.
pub fn token<'a, E>(typ: TokenType) -> impl Fn(TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    verify(one_token, move |t: &TokenInfo| t.typ == typ)
}

/// Parse a name token from a token slice.
pub fn name_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::NAME)(input)
}

/// Parse a name token containing a specific string from a token slice.
pub fn name_string<'a, E>(
    string: &'a str,
) -> impl Fn(TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    verify(name_token, move |t: &TokenInfo| t.string == string)
}

/// Parse an op token from a token slice.
pub fn op_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::OP)(input)
}

/// Parse an op token containing a specific string from a token slice.
pub fn op_string<'a, E>(
    string: &'a str,
) -> impl Fn(TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    verify(op_token, move |t: &TokenInfo| t.string == string)
}

/// Parse a number token from a token slice.
pub fn number_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::NUMBER)(input)
}

/// Parse a string token from a token slice.
pub fn string_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::STRING)(input)
}

/// Parse an indent token from a token slice.
pub fn indent_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::INDENT)(input)
}

/// Parse a dedent token from a token slice.
pub fn dedent_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::DEDENT)(input)
}

/// Parse a grammatically significant newline token from a token slice.
pub fn newline_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::NEWLINE)(input)
}

/// Parse an endmarker token from a token slice.
pub fn endmarker_token<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, TokenRef<'a>, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    token(TokenType::ENDMARKER)(input)
}

/// Parse a vyper source file into a `Module` AST object.
pub fn parse_file<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, Module, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    // Consume any leading newlines
    let (i, _) = many0(newline_token)(input)?;

    // module_stmt*
    let (i, body) = many0(parse_module_stmt)(i)?;

    // <endmarker>
    let (i, _) = endmarker_token(i)?;

    Ok((i, Module { body }))
}

/// Parse a module statement, such as an event or contract definition, into a `ModuleStmt` object.
pub fn parse_module_stmt<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, ModuleStmt, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    let (i, module_stmt) = context("expected event definition", parse_event_def)(input)?;

    Ok((i, module_stmt))
}

/// Parse an event definition statement into a `ModuleStmt::EventDef` object.
pub fn parse_event_def<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, ModuleStmt, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    // "event" name ":" <newline>
    let (i, _) = name_string("event")(input)?;
    let (i, name) = name_token(i)?;
    let (i, _) = op_string(":")(i)?;
    let (i, _) = newline_token(i)?;

    // <indent> event_field* <dedent>
    let (i, _) = indent_token(i)?;
    let (i, fields) = many1(parse_event_field)(i)?;
    let (i, _) = dedent_token(i)?;

    Ok((
        i,
        EventDef {
            name: name.string.to_string(),
            fields: fields,
        },
    ))
}

/// Parse an event field definition into an `EventField` object.
pub fn parse_event_field<'a, E>(input: TokenSlice<'a>) -> TokenResult<'a, EventField, E>
where
    E: ParseError<TokenSlice<'a>>,
{
    let (i, name) = name_token(input)?;
    let (i, _) = op_string(":")(i)?;
    let (i, typ) = name_token(i)?;
    let (i, _) = newline_token(i)?;

    Ok((
        i,
        EventField {
            name: name.string.to_string(),
            typ: typ.string.into(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use nom::error::ErrorKind;

    type SimpleError<I> = (I, ErrorKind);

    #[test]
    fn test_parse_file() {
        // Empty file
        let examples = vec!["", "  \t ", " \n\n   \t \n \t "];
        let expected: IResult<_, _, SimpleError<_>> = Ok((&[][..], Module { body: vec![] }));

        for inp in examples {
            let tokens = get_parse_tokens(inp).unwrap();
            let actual = parse_file::<SimpleError<_>>(&tokens[..]);
            assert_eq!(actual, expected);
        }

        // Test one stmt
        let examples = vec![
            // No leading or trailing whitespace
            r"event Greet:
    name: bytes32
    age: uint8",
            // Leading whitespace
            r"
event Greet:
    name: bytes32
    age: uint8",
            // Leading and trailing whitespace
            r"
event Greet:
    name: bytes32
    age: uint8
",
        ];
        let expected: IResult<_, _, SimpleError<_>> = Ok((
            &[][..],
            Module {
                body: vec![EventDef {
                    name: "Greet".to_string(),
                    fields: vec![
                        EventField {
                            name: "name".to_string(),
                            typ: "bytes32".into(),
                        },
                        EventField {
                            name: "age".to_string(),
                            typ: "uint8".into(),
                        },
                    ],
                }],
            },
        ));
        for inp in examples {
            let tokens = get_parse_tokens(inp).unwrap();
            let actual = parse_file::<SimpleError<_>>(&tokens[..]);
            assert_eq!(actual, expected);
        }

        // More than one stmt
        let examples = vec![
            // No leading, mid, or trailing whitespace
            r"event Greet:
    name: bytes32
    age: uint8
event Other:
    info1: uint256
    info2: bool",
            // Leading whitespace
            r"
event Greet:
    name: bytes32
    age: uint8
event Other:
    info1: uint256
    info2: bool",
            // Leading and trailing whitespace
            r"
event Greet:
    name: bytes32
    age: uint8
event Other:
    info1: uint256
    info2: bool
",
            // Leading, mid, and trailing whitespace
            r"
event Greet:
    name: bytes32
    age: uint8

event Other:
    info1: uint256
    info2: bool
",
        ];
        let expected: IResult<_, _, SimpleError<_>> = Ok((
            &[][..],
            Module {
                body: vec![
                    EventDef {
                        name: "Greet".to_string(),
                        fields: vec![
                            EventField {
                                name: "name".to_string(),
                                typ: "bytes32".into(),
                            },
                            EventField {
                                name: "age".to_string(),
                                typ: "uint8".into(),
                            },
                        ],
                    },
                    EventDef {
                        name: "Other".to_string(),
                        fields: vec![
                            EventField {
                                name: "info1".to_string(),
                                typ: "uint256".into(),
                            },
                            EventField {
                                name: "info2".to_string(),
                                typ: "bool".into(),
                            },
                        ],
                    },
                ],
            },
        ));
        for inp in examples {
            let tokens = get_parse_tokens(inp).unwrap();
            let actual = parse_file::<SimpleError<_>>(&tokens[..]);
            assert_eq!(actual, expected);
        }
    }
}
