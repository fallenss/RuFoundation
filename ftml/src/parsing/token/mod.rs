/*
 * parsing/token/mod.rs
 *
 * ftml - Library to parse Wikidot text
 * Copyright (C) 2019-2022 Wikijump Team
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#[cfg(test)]
mod test;

mod lexer {
    // Since pest makes enums automatically that clippy doesn't like
    #![allow(clippy::upper_case_acronyms)]

    // The actual parser definition, which we will re-export
    #[derive(Parser, Debug)]
    #[grammar = "parsing/lexer.pest"]
    pub struct TokenLexer;
}

use self::lexer::*;
use crate::utf16::Utf16IndexMap;
use pest::iterators::Pair;
use pest::Parser;
use std::ops::Range;
use strum_macros::IntoStaticStr;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExtractedToken<'a> {
    pub token: Token,
    pub slice: &'a str,
    pub span: Range<usize>,
}

impl<'a> ExtractedToken<'a> {
    #[must_use]
    pub fn to_utf16_indices(&self, map: &Utf16IndexMap) -> Self {
        // Copy fields
        let ExtractedToken { token, slice, span } = self.clone();

        // Map indices to UTF-16
        let start = map.get_index(span.start);
        let end = map.get_index(span.end);
        let span = start..end;

        // Output new ExtractedToken
        ExtractedToken { token, slice, span }
    }
}

#[derive(
    Serialize, Deserialize, Enum, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq,
)]
#[serde(rename_all = "kebab-case")]
pub enum Token {
    //
    // Symbols
    //
    LeftBracket,
    RightBracket,
    LeftBlock,
    LeftBlockEnd,
    LeftMath,
    RightBlock,
    RightMath,
    DoubleDash,
    TripleDash,
    LeftDoubleAngle,
    RightDoubleAngle,
    ClearFloatBoth,
    ClearFloatLeft,
    ClearFloatRight,
    Pipe,
    Equals,
    Colon,
    Underscore,
    Quote,
    Heading,

    //
    // Whitespace
    //
    LineBreak,
    ParagraphBreak,
    Whitespace,

    //
    // Formatting
    //
    Bold,
    Italics,
    Underline,
    Superscript,
    Subscript,
    LeftMonospace,
    RightMonospace,
    Color,
    Raw,
    HtmlEntity,
    LeftRaw,
    RightRaw,

    //
    // Lists
    //
    BulletItem,
    NumberedItem,

    //
    // Links
    //
    LeftLink,
    RightLink,

    //
    // Tables
    //
    TableColumn,
    TableColumnLeft,
    TableColumnRight,
    TableColumnCenter,
    TableColumnTitle,

    //
    // Text components
    //
    Identifier,
    Email,
    Url,
    Variable,
    StringQuote,
    StringQuoteEscape,
    StringQuoteDoubleEscape,

    //
    // Miscellaneous
    //
    LeftComment,
    RightComment,
    InputStart,
    InputEnd,

    //
    // Catch-all case
    //
    Other,
}

impl Token {
    pub(crate) fn extract_all(text: &str) -> Vec<ExtractedToken> {
        info!("Running lexer on input");

        match TokenLexer::parse(Rule::document, text) {
            Ok(pairs) => {
                info!("Lexer produced pairs for processing");

                // Map pairs to tokens, and add a Token::InputStart at the beginning
                // Pest already adds a Token::InputEnd at the end
                let start = ExtractedToken {
                    token: Token::InputStart,
                    slice: "",
                    span: 0..0,
                };

                let mut tokens = vec![start];
                tokens.extend(pairs.map(Token::convert_pair));
                tokens
            }
            Err(error) => {
                // Return all of the input as one big raw text
                // and log this as an error, since it shouldn't be happening

                error!("Error while lexing input in pest: {error}");
                vec![ExtractedToken {
                    token: Token::Other,
                    slice: text,
                    span: 0..text.len(),
                }]
            }
        }
    }

    /// Converts a single `Pair` from pest into its corresponding `ExtractedToken`.
    fn convert_pair(pair: Pair<Rule>) -> ExtractedToken {
        // Extract values from the Pair
        let rule = pair.as_rule();
        let slice = pair.as_str();
        let start = pair.as_span().start();
        let end = pair.as_span().end();
        let span = start..end;

        // Get matching Token.
        let token = Token::get_from_rule(rule);
        debug!("Converting pair '{:?}' into token {}", rule, token.name());

        ExtractedToken { token, slice, span }
    }

    /// Mapping of a pest `Rule` to its corresponding `Token` enum.
    fn get_from_rule(rule: Rule) -> Token {
        match rule {
            // Symbols
            Rule::left_comment => Token::LeftComment,
            Rule::right_comment => Token::RightComment,
            Rule::left_bracket => Token::LeftBracket,
            Rule::right_bracket => Token::RightBracket,
            Rule::left_block => Token::LeftBlock,
            Rule::left_block_end => Token::LeftBlockEnd,
            Rule::left_math => Token::LeftMath,
            Rule::right_block => Token::RightBlock,
            Rule::right_math => Token::RightMath,
            Rule::color => Token::Color,
            Rule::double_dash => Token::DoubleDash,
            Rule::triple_dash => Token::TripleDash,
            Rule::left_double_angle => Token::LeftDoubleAngle,
            Rule::right_double_angle => Token::RightDoubleAngle,
            Rule::clear_float => Token::ClearFloatBoth,
            Rule::clear_float_left => Token::ClearFloatLeft,
            Rule::clear_float_right => Token::ClearFloatRight,
            Rule::pipe => Token::Pipe,
            Rule::colon => Token::Colon,
            Rule::underscore => Token::Underscore,
            Rule::equals => Token::Equals,
            Rule::quote => Token::Quote,
            Rule::heading => Token::Heading,

            // Whitespace
            Rule::line_break => Token::LineBreak,
            Rule::paragraph_break => Token::ParagraphBreak,
            Rule::space => Token::Whitespace,

            // Formatting
            Rule::bold => Token::Bold,
            Rule::italics => Token::Italics,
            Rule::underline => Token::Underline,
            Rule::superscript => Token::Superscript,
            Rule::subscript => Token::Subscript,
            Rule::left_monospace => Token::LeftMonospace,
            Rule::right_monospace => Token::RightMonospace,
            Rule::raw => Token::Raw,
            Rule::html_entity => Token::HtmlEntity,
            Rule::left_raw => Token::LeftRaw,
            Rule::right_raw => Token::RightRaw,

            // Lists
            Rule::bullet_item => Token::BulletItem,
            Rule::numbered_item => Token::NumberedItem,

            // Links
            Rule::left_link => Token::LeftLink,
            Rule::right_link => Token::RightLink,

            // Tables
            Rule::table_column => Token::TableColumn,
            Rule::table_column_left => Token::TableColumnLeft,
            Rule::table_column_right => Token::TableColumnRight,
            Rule::table_column_center => Token::TableColumnCenter,
            Rule::table_column_title => Token::TableColumnTitle,

            // Text components
            Rule::identifier => Token::Identifier,
            Rule::email => Token::Email,
            Rule::url => Token::Url,
            Rule::variable => Token::Variable,
            Rule::string_quote => Token::StringQuote,
            Rule::string_quote_escape => Token::StringQuoteEscape,
            Rule::string_quote_doubleescape => Token::StringQuoteDoubleEscape,

            // Other
            Rule::other => Token::Other,
            Rule::EOI => Token::InputEnd,

            // Invalid
            Rule::document | Rule::token => {
                panic!("Received invalid pest rule: {:?}", rule)
            }
        }
    }

    #[inline]
    pub fn name(self) -> &'static str {
        self.into()
    }
}
