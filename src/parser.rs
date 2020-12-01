extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, multispace0, multispace1},
    error::ErrorKind,
    sequence::delimited,
    Err, IResult,
};

// 構文
// $VAR  := アルファベットとアラビア数字で表記される変数
//
// $EXPR := $LET | $IF | $SPLIT | $FREE | $QVAL | $APP | $VAR
//
// $LET  := let $VAR : $TYPE = $EXPR { $EXPR }
// $IF   := if $EXPR { $EXPR } else { $EXPR }
// $SPLIT := split $EXPR as $VAR, $VAR { $EXPR }
// $FREE := free $EXPR
// $APP  := ( $EXPR $EXPR )
//
// $QUAL := lin | un
//
// 値
// $QVAL := $QUAL $VAL
// $VAL  := true | false | < $EXPR , $EXPR > | $FN
// $FN   := fn $VAR : $TYPE { $EXPR }
//
// 型
// $TYPE := $QUAL $PRIM
// $PRIM := bool |
//          ( $TYPE * $TYPE )
//          ( $TYPE -> $TYPE )

// 抽象構文木
#[derive(Debug)]
pub enum Expr {
    QVal(QValExpr),   // 値
    Let(LetExpr),     // let式
    If(IfExpr),       // if式
    Split(SplitExpr), // split式
    Free(FreeExpr),   // free文
    App(AppExpr),     // 関数適用
    Var(String),      // 変数
}

// 関数適用
// $APP  := ( $EXPR $EXPR )
//
// (expr1 expr2)
#[derive(Debug)]
pub struct AppExpr {
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

// if式
// $IF   := if $EXPR { $EXPR } else { $EXPR }
//
// if cond_expr {
//     then_expr
// } else {
//     else_expr
// }
#[derive(Debug)]
pub struct IfExpr {
    pub cond_expr: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>,
}

// split式
// $SPLIT := split $EXPR as $VAR, $VAR { $EXPR }
//
// split expr as left, right {
//     body
// }
#[derive(Debug)]
pub struct SplitExpr {
    pub expr: Box<Expr>,
    pub left: String,
    pub right: String,
    pub body: Box<Expr>,
}

// let式
// $LET  := let $VAR : $TYPE = $EXPR { $EXPR }
//
// let var : ty = expr1 { expr2 }
#[derive(Debug)]
pub struct LetExpr {
    pub var: String,
    pub ty: TypeExpr,
    pub expr1: Box<Expr>,
    pub expr2: Box<Expr>,
}

// 値
// 真偽値、λ抽象、ペア値などになる
// $VAL  := true | false | < $EXPR , $EXPR > | $FN
#[derive(Debug)]
pub enum ValExpr {
    Bool(bool),                 // 真偽値リテラル
    Pair(Box<Expr>, Box<Expr>), // ペア
    Fun(FnExpr),                // λ抽象
}

// 修飾子
// $QUAL := lin | un
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Qual {
    Lin, // 線形型
    Un,  // 制約のない一般的な型
}

// 修飾子付き値
// $QVAL := $QUAL $VAL
#[derive(Debug)]
pub struct QValExpr {
    pub qual: Qual,
    pub val: ValExpr,
}

// λ抽象
// $FN   := fn $VAR : $TYPE { $EXPR }
//
// fn var : ty { expr }
#[derive(Debug)]
pub struct FnExpr {
    pub var: String,
    pub ty: TypeExpr,
    pub expr: Box<Expr>,
}

// free文
// $FREE := free $EXPR
//
// free var; expr
#[derive(Debug)]
pub struct FreeExpr {
    pub expr: Box<Expr>,
    pub var: String,
}

// 修飾子付き型
// $TYPE := $QUAL $PRIM
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TypeExpr {
    pub qual: Qual,
    pub prim: PrimType,
}

// プリミティブ型
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PrimType {
    Bool,                                // 真偽値型
    Pair(Box<TypeExpr>, Box<TypeExpr>),  // ペア型
    Arrow(Box<TypeExpr>, Box<TypeExpr>), // 関数型
}

// ここより下はパースしているのみなので、読む必要はない
pub fn parse_expr(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, val) = alt((tag("let"), tag("lin"), tag("un"), alpha1, tag("(")))(i)?;

    match val {
        "let" => parse_let(i),
        "if" => parse_if(i),
        "split" => parse_split(i),
        "free" => parse_free(i),
        "lin" => parse_qval(Qual::Lin, i),
        "un" => parse_qval(Qual::Un, i),
        "(" => parse_app(i),
        _ => Ok((i, Expr::Var(val.to_string()))),
    }
}

fn parse_app(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, e1) = parse_expr(i)?;

    let (i, _) = multispace1(i)?;

    let (i, e2) = parse_expr(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char(')')(i)?;

    Ok((
        i,
        Expr::App(AppExpr {
            expr1: Box::new(e1),
            expr2: Box::new(e2),
        }),
    ))
}

fn parse_free(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace1(i)?;
    let (i, var) = alpha1(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = char(';')(i)?;

    let (i, e) = parse_expr(i)?;
    Ok((
        i,
        Expr::Free(FreeExpr {
            var: var.to_string(),
            expr: Box::new(e),
        }),
    ))
}

fn parse_split(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, e1) = parse_expr(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = tag("as")(i)?;
    let (i, _) = multispace0(i)?;

    let (i, v1) = parse_var(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = multispace0(i)?;

    let (i, v2) = parse_var(i)?;
    let (i, _) = multispace0(i)?;

    let (i, e2) = delimited(
        char('{'),
        delimited(multispace0, parse_expr, multispace0),
        char('}'),
    )(i)?;

    Ok((
        i,
        Expr::Split(SplitExpr {
            expr: Box::new(e1),
            left: v1.to_string(),
            right: v2.to_string(),
            body: Box::new(e2),
        }),
    ))
}

fn parse_if(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, e1) = parse_expr(i)?;
    let (i, _) = multispace0(i)?;

    let (i, e2) = delimited(
        char('{'),
        delimited(multispace0, parse_expr, multispace0),
        char('}'),
    )(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = tag("else")(i)?;
    let (i, _) = multispace0(i)?;

    let (i, e3) = delimited(
        char('{'),
        delimited(multispace0, parse_expr, multispace0),
        char('}'),
    )(i)?;

    Ok((
        i,
        Expr::If(IfExpr {
            cond_expr: Box::new(e1),
            then_expr: Box::new(e2),
            else_expr: Box::new(e3),
        }),
    ))
}

fn parse_let(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;

    let (i, var) = parse_var(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = multispace0(i)?;

    let (i, ty) = parse_type(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char('=')(i)?;
    let (i, _) = multispace0(i)?;

    let (i, e1) = parse_expr(i)?;
    let (i, _) = multispace0(i)?;

    let (i, e2) = delimited(
        char('{'),
        delimited(multispace0, parse_expr, multispace0),
        char('}'),
    )(i)?;

    Ok((
        i,
        Expr::Let(LetExpr {
            var: var,
            ty: ty,
            expr1: Box::new(e1),
            expr2: Box::new(e2),
        }),
    ))
}

fn parse_pair(i: &str) -> IResult<&str, ValExpr> {
    let (i, _) = multispace0(i)?;

    let (i, v1) = parse_expr(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = multispace0(i)?;

    let (i, v2) = parse_expr(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char('>')(i)?;

    Ok((i, ValExpr::Pair(Box::new(v1), Box::new(v2))))
}

fn parse_qual(i: &str) -> IResult<&str, Qual> {
    let (i, val) = alt((tag("lin"), tag("un")))(i)?;
    if val == "lin" {
        Ok((i, Qual::Lin))
    } else {
        Ok((i, Qual::Un))
    }
}

fn parse_fn(i: &str) -> IResult<&str, ValExpr> {
    let (i, _) = multispace0(i)?;
    let (i, var) = parse_var(i)?;

    let (i, _) = multispace0(i)?;
    let (i, _) = char(':')(i)?;
    let (i, _) = multispace0(i)?;

    let (i, ty) = parse_type(i)?;
    let (i, _) = multispace0(i)?;

    let (i, expr) = delimited(
        char('{'),
        delimited(multispace0, parse_expr, multispace0),
        char('}'),
    )(i)?;

    Ok((
        i,
        ValExpr::Fun(FnExpr {
            var: var,
            ty: ty,
            expr: Box::new(expr),
        }),
    ))
}

fn parse_val(i: &str) -> IResult<&str, ValExpr> {
    let (i, val) = alt((tag("fn"), tag("true"), tag("false"), tag("<")))(i)?;
    match val {
        "fn" => parse_fn(i),
        "true" => Ok((i, ValExpr::Bool(true))),
        "false" => Ok((i, ValExpr::Bool(false))),
        "<" => parse_pair(i),
        _ => Err(Err::Error(("internal fail", ErrorKind::Tag))),
    }
}

fn parse_qval(q: Qual, i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, v) = parse_val(i)?;

    Ok((i, Expr::QVal(QValExpr { qual: q, val: v })))
}

fn parse_var(i: &str) -> IResult<&str, String> {
    let (i, v) = alpha1(i)?;
    Ok((i, v.to_string()))
}

fn parse_type(i: &str) -> IResult<&str, TypeExpr> {
    let (i, q) = parse_qual(i)?;
    let (i, _) = multispace0(i)?;
    let (i, val) = alt((tag("bool"), tag("(")))(i)?;
    if val == "bool" {
        Ok((
            i,
            TypeExpr {
                qual: q,
                prim: PrimType::Bool,
            },
        ))
    } else {
        let (i, _) = multispace0(i)?;
        let (i, t1) = parse_type(i)?;
        let (i, _) = multispace0(i)?;

        let (i, op) = alt((tag("*"), tag("->")))(i)?;

        let (i, _) = multispace0(i)?;
        let (i, t2) = parse_type(i)?;
        let (i, _) = multispace0(i)?;

        let (i, _) = char(')')(i)?;

        Ok((
            i,
            TypeExpr {
                qual: q,
                prim: if op == "*" {
                    PrimType::Pair(Box::new(t1), Box::new(t2))
                } else {
                    PrimType::Arrow(Box::new(t1), Box::new(t2))
                },
            },
        ))
    }
}
