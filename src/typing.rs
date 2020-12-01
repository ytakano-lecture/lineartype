use super::parser;
use std::collections::{HashMap, LinkedList};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Context {
    vars: LinkedList<HashMap<String, parser::TypeExpr>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            vars: LinkedList::new(),
        }
    }

    // 空のコンテキストをpush
    fn push(&mut self) {
        self.vars.push_back(HashMap::new());
    }

    // 一番上のコンテキストをpop
    fn pop(&mut self) -> Option<HashMap<String, parser::TypeExpr>> {
        self.vars.pop_back()
    }

    // 一番上のコンテキストに追加
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        match self.vars.back_mut() {
            Some(d) => {
                d.insert(key, value);
            }
            None => {
                panic!("failed to update context");
            }
        }
    }

    // コンテキストスタックを上から辿って、最もはじめに見つかる変数を削除
    fn remove(&mut self, key: &String) {
        // TODO: ここを実装せよ
    }

    // コンテキストスタックの最も上にある変数の型を取得
    fn get(&mut self, key: &String) -> Option<&parser::TypeExpr> {
        for d in self.vars.iter().rev() {
            match d.get(key) {
                None => {
                    continue;
                }
                v => {
                    return v;
                }
            }
        }
        None
    }
}

// 型付け関数
// 式を受け取り、型を返す
pub fn typing(expr: &parser::Expr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    match expr {
        parser::Expr::App(e) => typing_app(e, ctx),
        parser::Expr::QVal(e) => typing_qval(e, ctx),
        parser::Expr::Free(e) => typing_free(e, ctx),
        parser::Expr::If(e) => typing_if(e, ctx),
        parser::Expr::Split(e) => typing_split(e, ctx),
        parser::Expr::Var(e) => typing_var(e, ctx),
        parser::Expr::Let(e) => typing_let(e, ctx),
    }
}

// 関数適用の型付け
fn typing_app(expr: &parser::AppExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    // TODO: ここを実装せよ

    // TODO: このparser::PrimType::Boolを消して、正しい型を返すこと
    None
}

// 修飾子付き値の型付け
fn typing_qval(expr: &parser::QValExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    // プリミティブ型を計算
    let p = match &expr.val {
        parser::ValExpr::Bool(_) => parser::PrimType::Bool,
        parser::ValExpr::Pair(e1, e2) => {
            // ペア型の型付け

            // TODO: ここを実装せよ

            // TODO: このparser::PrimType::Boolを消して、正しい型を返すこと
            parser::PrimType::Bool
        }
        parser::ValExpr::Fun(e) => {
            // λ抽象の型付け

            // TODO: ここを実装せよ

            // TODO: このparser::PrimType::Boolを消して、正しい型を返すこと
            parser::PrimType::Bool
        }
    };

    // 修飾子付き型を返す
    Some(parser::TypeExpr {
        qual: expr.qual.clone(),
        prim: p,
    })
}

// free式の型付け
fn typing_free(expr: &parser::FreeExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    let t = ctx.get(&expr.var)?;
    if t.qual == parser::Qual::Lin {
        ctx.remove(&expr.var);
        typing(&expr.expr, ctx)
    } else {
        println!("error: un型をfreeしている");
        None
    }
}

// if式の型付け
fn typing_if(expr: &parser::IfExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    // TODO: ここを実装せよ

    // TODO: このNoneを消して、正しい型を返すこと
    None
}

// split式の型付け
fn typing_split(expr: &parser::SplitExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    let t1 = typing(&expr.expr, ctx)?;

    match t1.prim {
        parser::PrimType::Pair(p1, p2) => {
            ctx.push();
            ctx.insert(expr.left.clone(), *p1);
            ctx.insert(expr.right.clone(), *p2);
        }
        _ => {
            println!("error: ペア型でない");
            return None;
        }
    }

    let ret = typing(&expr.body, ctx);
    let c = ctx.pop().unwrap();
    for (_, t) in c.iter() {
        if t.qual == parser::Qual::Lin {
            println!("error: 未使用のlin型がある");
            return None;
        }
    }

    ret
}

// 変数の型付け
fn typing_var(expr: &String, ctx: &mut Context) -> Option<parser::TypeExpr> {
    // TODO: ここを実装せよ

    // TODO: このNoneを消して、正しい型を返すこと
    None
}

// let式の型付け
fn typing_let(expr: &parser::LetExpr, ctx: &mut Context) -> Option<parser::TypeExpr> {
    // TODO: ここを実装せよ

    // TODO: このNoneを消して、正しい型を返すこと
    None
}
