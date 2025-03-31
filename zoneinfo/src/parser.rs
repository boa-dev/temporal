use core::{
    iter::Peekable,
    num::ParseIntError,
    str::{Lines, SplitWhitespace},
};

use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};

use crate::{
    rule::{Rule, RuleTable},
    zone::ZoneTable,
    ZoneInfo,
};

#[derive(Debug)]
pub enum ZoneInfoParseError {
    InvalidZoneHeader(u32),
    MissingIdentifier(u32),
    UnexpectedEndOfLine(u32, &'static str),
    UnknownValue(u32, String),
    ParseIntError(u32, ParseIntError, &'static str),
}

pub trait TryFromStr<C>: Sized {
    type Error;
    fn try_from_str(s: &str, context: &mut C) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct LineParseContext {
    pub line_number: u32,
    pub spans: Vec<&'static str>,
}

impl LineParseContext {
    pub fn enter(&mut self, name: &'static str) {
        self.spans.push(name);
    }

    pub fn span(&self) -> &'static str {
        self.spans.last().expect("span not defined")
    }

    pub fn exit(&mut self) {
        self.spans.pop();
    }
}

impl Default for LineParseContext {
    fn default() -> Self {
        Self {
            line_number: 1,
            spans: vec!["undefined"],
        }
    }
}

pub trait ContextParse {
    fn context_parse<T: TryFromStr<LineParseContext>>(
        &self,
        ctx: &mut LineParseContext,
    ) -> Result<T, <T as TryFromStr<LineParseContext>>::Error>;
}

impl ContextParse for &str {
    fn context_parse<T: TryFromStr<LineParseContext>>(
        &self,
        ctx: &mut LineParseContext,
    ) -> Result<T, <T as TryFromStr<LineParseContext>>::Error> {
        T::try_from_str(self, ctx)
    }
}

impl ContextParse for String {
    fn context_parse<T: TryFromStr<LineParseContext>>(
        &self,
        ctx: &mut LineParseContext,
    ) -> Result<T, <T as TryFromStr<LineParseContext>>::Error> {
        T::try_from_str(self, ctx)
    }
}

impl TryFromStr<LineParseContext> for String {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, _: &mut LineParseContext) -> Result<Self, Self::Error> {
        Ok(s.parse::<String>().expect("parse String is infallible"))
    }
}

impl TryFromStr<LineParseContext> for i8 {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        s.parse::<i8>()
            .map_err(|e| ZoneInfoParseError::ParseIntError(ctx.line_number, e, ctx.span()))
    }
}

impl TryFromStr<LineParseContext> for u8 {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        s.parse::<u8>()
            .map_err(|e| ZoneInfoParseError::ParseIntError(ctx.line_number, e, ctx.span()))
    }
}

impl TryFromStr<LineParseContext> for u16 {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        s.parse::<u16>()
            .map_err(|e| ZoneInfoParseError::ParseIntError(ctx.line_number, e, ctx.span()))
    }
}

impl TryFromStr<LineParseContext> for i32 {
    type Error = ZoneInfoParseError;
    fn try_from_str(s: &str, ctx: &mut LineParseContext) -> Result<Self, Self::Error> {
        s.parse::<i32>()
            .map_err(|e| ZoneInfoParseError::ParseIntError(ctx.line_number, e, ctx.span()))
    }
}

pub(crate) fn next_split<'a>(
    splits: &mut SplitWhitespace<'a>,
    context: &LineParseContext,
) -> Result<&'a str, ZoneInfoParseError> {
    splits.next().ok_or(ZoneInfoParseError::UnexpectedEndOfLine(
        context.line_number,
        context.span(),
    ))
}

pub(crate) fn remove_comments(line: &str) -> &str {
    if let Some((cleaned, _comment)) = line.split_once("#") {
        cleaned
    } else {
        line
    }
}

#[non_exhaustive]
pub struct ZoneInfoParser<'data> {
    lines: Peekable<Lines<'data>>,
}

impl<'data> ZoneInfoParser<'data> {
    /// Creates a parser from a `&str`
    pub fn from_zoneinfo_str(source: &'data str) -> Self {
        Self {
            lines: source.lines().peekable(),
        }
    }

    #[allow(clippy::while_let_on_iterator)]
    pub fn parse(&mut self) -> Result<ZoneInfo, ZoneInfoParseError> {
        let mut zoneinfo = ZoneInfo::default();
        let mut context = LineParseContext::default();
        while let Some(line) = self.lines.peek() {
            if line.is_empty() || line.starts_with("# ") {
                // Check if line is empty or a comment
                //
                // It is important here that a comment matches on "# "
                // Because a ratpacked line is #PACKRATLIST...
            } else if line.starts_with("Rule") {
                // TODO: Return a Rule Table and handle extending the table when needed.
                let (identifier, data) = Rule::parse(line, &mut context).unwrap();
                if let Some(rules) = zoneinfo.rules.get_mut(&identifier) {
                    rules.extend(data);
                } else {
                    zoneinfo
                        .rules
                        .insert(identifier, RuleTable::initialize(data));
                }
            } else if line.starts_with("Zone") {
                let (identifer, table) =
                    ZoneTable::parse_full_table(&mut self.lines, &mut context).unwrap();
                zoneinfo.zones.insert(identifer, table);
            } else if line.starts_with("Link") {
                let mut splits = line.split_whitespace();
                next_split(&mut splits, &context)?; // Consume the Link
                let zone = next_split(&mut splits, &context)?;
                let link = next_split(&mut splits, &context)?;
                zoneinfo.links.insert(link.to_owned(), zone.to_owned());
            // NOTE: This may be able to be consildated with link based off a flag.
            } else if line.starts_with("#PACKRATLIST") {
                let mut splits = line.split_whitespace();
                next_split(&mut splits, &context)?; // Consume the #PACKRATLIST
                next_split(&mut splits, &context)?; // Consume the zone.tab
                next_split(&mut splits, &context)?; // Consume the Link
                let zone = next_split(&mut splits, &context)?;
                let link = next_split(&mut splits, &context)?;
                zoneinfo.pack_rat.insert(link.to_owned(), zone.to_owned());
            }
            self.lines.next();
            context.line_number += 1;
        }
        Ok(zoneinfo)
    }
}
