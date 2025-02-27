//! Erlang types.
//!
//! <https://www.erlang.org/doc/reference_manual/typespec.html>
use self::components::{BinaryOp, BitstringItem, UnaryOp};
use crate::format::{Format, Formatter, Indent, Newline};
use crate::items::components::{
    Args, BinaryOpLike, BinaryOpStyle, BitstringLike, Either, Element, ListLike, MapLike, Maybe,
    NonEmptyItems, Params, Parenthesized, RecordLike, TupleLike, UnaryOpLike,
};
use crate::items::keywords::FunKeyword;
use crate::items::symbols::{
    ColonSymbol, DoubleColonSymbol, RightArrowSymbol, SharpSymbol, TripleDotSymbol,
    VerticalBarSymbol,
};
use crate::items::tokens::{AtomToken, CharToken, IntegerToken, VariableToken};
use crate::items::Type;
use crate::parse::{self, Parse, ResumeParse};
use crate::span::Span;

pub mod components;

/// [Type] `|` [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct UnionType(NonEmptyItems<NonUnionType, UnionDelimiter>);

#[derive(Debug, Clone, Span, Parse)]
struct UnionDelimiter(VerticalBarSymbol);

impl Format for UnionDelimiter {
    fn format(&self, fmt: &mut Formatter) {
        fmt.add_space();
        self.0.format(fmt);
        fmt.add_space();
    }
}

#[derive(Debug, Clone, Span, Format)]
enum NonUnionType {
    Base(BaseType),
    BinaryOp(Box<BinaryOpType>),
}

impl Parse for NonUnionType {
    fn parse(ts: &mut parse::TokenStream) -> parse::Result<Self> {
        let expr: BaseType = ts.parse()?;
        if ts.peek::<BinaryOp>().is_some() {
            ts.resume_parse(expr).map(Self::BinaryOp)
        } else {
            Ok(Self::Base(expr))
        }
    }
}

// Non left-recursive type.
#[derive(Debug, Clone, Span, Parse, Format)]
enum BaseType {
    Mfargs(Box<MfargsType>),
    List(Box<ListType>),
    Tuple(Box<TupleType>),
    Map(Box<MapType>),
    Record(Box<RecordType>),
    Bitstring(Box<BitstringType>),
    Function(Box<FunctionType>),
    UnaryOp(Box<UnaryOpType>),
    Parenthesized(Box<Parenthesized<Type>>),
    Annotated(Box<AnnotatedVariableType>),
    Literal(LiteralType),
}

/// [VariableToken] `::` [Type]
#[derive(Debug, Clone, Span, Parse)]
pub struct AnnotatedVariableType {
    variable: VariableToken,
    colon: DoubleColonSymbol,
    ty: Type,
}

impl Format for AnnotatedVariableType {
    fn format(&self, fmt: &mut Formatter) {
        self.variable.format(fmt);
        fmt.add_space();
        self.colon.format(fmt);
        fmt.add_space();
        self.ty.format(fmt);
    }
}

/// [Type] [BinaryOp] [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct BinaryOpType(BinaryOpLike<BaseType, BinaryOp, Type>);

impl ResumeParse<BaseType> for BinaryOpType {
    fn resume_parse(ts: &mut parse::TokenStream, left: BaseType) -> parse::Result<Self> {
        ts.resume_parse(left).map(Self)
    }
}

/// [UnaryOp] [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct UnaryOpType(UnaryOpLike<UnaryOp, BaseType>);

/// `fun` `(` (`$PARAMS` `->` `$RETURN`)? `)`
///
/// - $PARAMS: `(` `...` `)` | `(` ([Type] `,`?)* `)`
/// - $RETURN: [Type]
#[derive(Debug, Clone, Span, Parse)]
pub struct FunctionType {
    fun: FunKeyword,
    params_and_return: Parenthesized<Maybe<FunctionParamsAndReturn>>,
}

impl Format for FunctionType {
    fn format(&self, fmt: &mut Formatter) {
        self.fun.format(fmt);
        self.params_and_return.format(fmt);
    }
}

type FunctionParamsAndReturn = BinaryOpLike<FunctionParams, RightArrowDelimiter, Type>;

#[derive(Debug, Clone, Span, Parse, Format)]
struct RightArrowDelimiter(RightArrowSymbol);

impl<RHS> BinaryOpStyle<RHS> for RightArrowDelimiter {
    fn indent(&self) -> Indent {
        Indent::Offset(8)
    }

    fn newline(&self, _rhs: &RHS, _fmt: &Formatter) -> Newline {
        Newline::IfTooLongOrMultiLine
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
enum FunctionParams {
    Any(Parenthesized<TripleDotSymbol>),
    Params(Params<Type>),
}

/// [AtomToken] | [CharToken] | [IntegerToken] | [VariableToken]
#[derive(Debug, Clone, Span, Parse, Format)]
pub enum LiteralType {
    Atom(AtomToken),
    Char(CharToken),
    Integer(IntegerToken),
    Variable(VariableToken),
}

/// (`$MODULE` `:`)? `$NAME` `(` (`$ARG` `,`)* `)`
///
/// - $MODULE: [AtomToken]
/// - $NAME: [AtomToken]
/// - $ARG: [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct MfargsType {
    module: Maybe<(AtomToken, ColonSymbol)>,
    name: AtomToken,
    args: Args<Type>,
}

/// `[` (`$ITEM` `,`?)* `]`
///
/// - $ITEM: [Type] | `...`
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct ListType(ListLike<ListItem>);

#[derive(Debug, Clone, Span, Parse, Format)]
struct ListItem(Either<Type, TripleDotSymbol>);

impl Element for ListItem {
    fn is_packable(&self) -> bool {
        false
    }
}

/// `{` ([Type] `,`)* `}`
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct TupleType(TupleLike<TupleItem>);

#[derive(Debug, Clone, Span, Parse, Format, Element)]
struct TupleItem(Type);

/// `#` `{` ([Type] (`:=` | `=>`) [Type] `,`?)* `}`
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct MapType(MapLike<SharpSymbol, Type>);

/// `#` `$NAME` `{` (`$FIELD` `,`?)* `}`
///
/// - $NAME: [AtomToken]
/// - $FIELD: [AtomToken] `::` [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct RecordType {
    record: RecordLike<(SharpSymbol, AtomToken), RecordItem>,
}

#[derive(Debug, Clone, Span, Parse, Format, Element)]
struct RecordItem(BinaryOpLike<AtomToken, DoubleColonDelimiter, Type>);

#[derive(Debug, Clone, Span, Parse, Format)]
struct DoubleColonDelimiter(DoubleColonSymbol);

impl<RHS> BinaryOpStyle<RHS> for DoubleColonDelimiter {
    fn indent(&self) -> Indent {
        Indent::Offset(4)
    }

    fn newline(&self, _rhs: &RHS, _fmt: &Formatter) -> Newline {
        Newline::IfTooLong
    }
}

/// `<<` `$BITS_SIZE`? `,`? `$UNIT_SIZE`? `>>`
///
/// - $BITS_SIZE: `_` `:` [Type]
/// - $UNIT_SIZE: `_` `:` `_` `*` [Type]
#[derive(Debug, Clone, Span, Parse, Format)]
pub struct BitstringType(BitstringLike<BitstringItem>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mfargs_works() {
        let texts = [
            "foo()",
            "foo:bar(A, 1)",
            indoc::indoc! {"
            %---10---|%---20---|
            foo:bar(A,
                    BB,
                    baz())"},
            indoc::indoc! {"
            %---10---|%---20---|
            foo:bar(A,
                    B,
                    baz(12, 34),
                    qux())"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn list_works() {
        let texts = [
            "[]",
            "[foo()]",
            "[10, ...]",
            indoc::indoc! {"
            %---10---|%---20---|
            [fooooooooooo(),
             ...]"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn tuple_works() {
        let texts = [
            "{}",
            "{foo()}",
            "{atom, 1}",
            indoc::indoc! {"
            %---10---|%---20---|
            {foo(),
             bar(),
             [baz()]}"},
            indoc::indoc! {"
            %---10---|%---20---|
            {foo(),
             bar(),
             [baz(A, B)]}"},
            indoc::indoc! {"
            %---10---|%---20---|
            {foo, bar,
                  [baz(A, B)]}"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn map_works() {
        let expected = [
            "#{}",
            "#{a => b, 1 := 2}",
            indoc::indoc! {"
            %---10---|%---20---|
            #{
              atom() :=
                  integer()
             }"},
            indoc::indoc! {"
            %---10---|%---20---|
            #{
              atom() := {Aaa,
                         bbb,
                         ccc}
             }"},
            indoc::indoc! {"
            %---10---|%---20---|
            #{
              a => b,
              1 := 2,
              atom() := atom()
             }"},
        ];
        for text in expected {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn record_works() {
        let texts = [
            "#foo{}",
            indoc::indoc! {"
            %---10---|%---20---|
            #foo{
              bar :: integer()
             }"},
            indoc::indoc! {"
            %---10---|%---20---|
            #foo{
              bar :: b,
              baz :: 2
             }"},
            indoc::indoc! {"
            %---10---|%---20---|
            #foo{
              bar :: b,
              baz :: bb()
             }"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn function_works() {
        let texts = [
            "fun()",
            "fun(() -> integer())",
            indoc::indoc! {"
            %---10---|%---20---|
            fun((...) -> atom())"},
            indoc::indoc! {"
            %---10---|%---20---|
            fun((A, b, $c) ->
                        tuple())"},
            indoc::indoc! {"
            %---10---|%---20---|
            fun((A,
                 b,
                 $c,
                 {foo()}) ->
                        tuple())"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn unary_op_works() {
        let texts = ["-10", "+10", "bnot 100", "- -+ +3"];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn binary_op_works() {
        let texts = [
            "-10 + 20 rem 3",
            indoc::indoc! {"
            %---10---|%---20---|
            foo |
            (3 + 10) |
            -1..+20"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn bitstring_works() {
        let texts = [
            "<<>>",
            "<<_:10>>",
            "<<_:_*8>>",
            "<<_:8, _:_*4>>",
            indoc::indoc! {"
            %---10---|%---20---|
            <<_:(1 + 3 + 4),
              _:_*4>>"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }

    #[test]
    fn annotated_variable_works() {
        let texts = [
            "Foo :: atom()",
            indoc::indoc! {"
            %---10---|%---20---|
            Foo :: [bar:baz(qux)]"},
            indoc::indoc! {"
            %---10---|%---20---|
            Foo :: atom() |
                   integer() |
                   bar"},
        ];
        for text in texts {
            crate::assert_format!(text, Type);
        }
    }
}
