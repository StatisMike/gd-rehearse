/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use proc_macro2::{Literal, TokenTree};
use std::collections::VecDeque;
use venial::{Attribute, AttributeValue};

pub struct AttributeValueParser {
    tokens: VecDeque<TokenTree>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AttributeIdent {
    Repeat,
    Focus,
    Skip,
    Keyword,
}

impl AttributeIdent {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "repeat" => Some(Self::Repeat),
            "focus" => Some(Self::Focus),
            "skip" => Some(Self::Skip),
            "keyword" => Some(Self::Keyword),
            _ => None,
        }
    }

    pub fn to_str(self) -> String {
        match self {
            AttributeIdent::Repeat => "repeat".to_owned(),
            AttributeIdent::Focus => "focus".to_owned(),
            AttributeIdent::Skip => "skip".to_owned(),
            AttributeIdent::Keyword => "keyword".to_owned(),
        }
    }

    pub fn expected_oneof_message(arr: &[Self]) -> String {
        let mut idents_str = String::new();

        for (i, ident) in arr.iter().enumerate() {
            if i == arr.len() - 1 {
                idents_str.push_str(" or ");
            } else if i != 0 {
                idents_str.push_str(", ");
            }
            idents_str.push_str(&ident.to_str());
        }

        format!("expected one of the identifiers: {}", idents_str)
    }
}

impl AttributeValueParser {
    pub fn from_tokens(tokens: Vec<TokenTree>) -> Self {
        Self {
            tokens: tokens.into(),
        }
    }

    pub fn get_one_of_idents(
        &mut self,
        values: &[AttributeIdent],
    ) -> Result<Option<AttributeIdent>, venial::Error> {
        if let Some(token) = self.tokens.pop_front() {
            let str_token = token.to_string();
            if let Some(ident) = AttributeIdent::from_str(str_token.as_str()) {
                if values.contains(&ident) {
                    return Ok(Some(ident));
                }
            }
            Err(venial::Error::new_at_tokens(
                token,
                AttributeIdent::expected_oneof_message(values),
            ))
        } else {
            Ok(None)
        }
    }

    pub fn pop_equal_sign(&mut self) -> Result<(), venial::Error> {
        if let Some(token) = self.tokens.pop_front() {
            if let TokenTree::Punct(punct) = &token {
                if punct.as_char() == '=' {
                    return Ok(());
                }
            }
            return Err(venial::Error::new_at_tokens(token, "expected equal sign"));
        }
        Err(venial::Error::new("expected equal sign"))
    }

    pub fn get_literal(&mut self) -> Result<Literal, venial::Error> {
        if let Some(token) = self.tokens.pop_front() {
            if let TokenTree::Literal(literal) = token {
                return Ok(literal);
            }
            return Err(venial::Error::new_at_tokens(token, "expected literal"));
        }
        Err(venial::Error::new("expected literal"))
    }

    pub fn from_attribute_group_at_path(
        attributes: &[Attribute],
        path: &str,
    ) -> Result<Self, venial::Error> {
        for attribute in attributes.iter() {
            if attribute.path.len() == 1 && attribute.path[0].to_string() == path {
                if let AttributeValue::Group(_, tokens) = attribute.value.clone() {
                    return Ok(Self::from_tokens(tokens));
                }
                return Err(venial::Error::new_at_span(
                    attribute.tk_hash.span(),
                    "expected group attribute",
                ));
            }
        }
        Err(venial::Error::new(format!(
            "couldn't find '{}' attribute",
            path
        )))
    }

    pub fn progress_puct(&mut self) {
        if let Some(TokenTree::Punct(_punct)) = self.tokens.front() {
            _ = self.tokens.pop_front();
        }
    }
}
