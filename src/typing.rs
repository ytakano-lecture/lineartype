use crate::{helper::safe_add, parser};
use std::{cmp::Ordering, collections::BTreeMap, mem};

type VarToType = BTreeMap<String, Option<parser::TypeExpr>>;

/// 型環境
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeEnv {
    env_lin: TypeEnvStack, // lin用
    env_un: TypeEnvStack,  // un用
}

impl TypeEnv {
    pub fn new() -> TypeEnv {
        TypeEnv {
            env_lin: TypeEnvStack::new(),
            env_un: TypeEnvStack::new(),
        }
    }

    /// 型環境をpush
    fn push(&mut self, depth: usize) {
        self.env_lin.push(depth);
        self.env_un.push(depth);
    }

    /// 型環境をpop
    fn pop(&mut self, depth: usize) -> (Option<VarToType>, Option<VarToType>) {
        let t1 = self.env_lin.pop(depth);
        let t2 = self.env_un.pop(depth);
        (t1, t2)
    }

    /// 型環境へ変数と型をpush
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        if value.qual == parser::Qual::Lin {
            self.env_lin.insert(key, value);
        } else {
            self.env_un.insert(key, value);
        }
    }

    /// linとunの型環境からget_mutし、depthが大きい方を返す
    fn get_mut(&mut self, key: &str) -> Option<&mut Option<parser::TypeExpr>> {
        match (self.env_lin.get_mut(key), self.env_un.get_mut(key)) {
            (Some((d1, t1)), Some((d2, t2))) => match d1.cmp(&d2) {
                Ordering::Less => Some(t2),
                Ordering::Greater => Some(t1),
                Ordering::Equal => panic!("invalid type environment"),
            },
            (Some((_, t1)), None) => Some(t1),
            (None, Some((_, t2))) => Some(t2),
            _ => None,
        }
    }
}

/// 型環境のスタック
#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct TypeEnvStack {
    vars: BTreeMap<usize, VarToType>,
}

impl TypeEnvStack {
    fn new() -> TypeEnvStack {
        TypeEnvStack {
            vars: BTreeMap::new(),
        }
    }

    // 型環境をpush
    fn push(&mut self, depth: usize) {
        self.vars.insert(depth, BTreeMap::new());
    }

    // 型環境をpop
    fn pop(&mut self, depth: usize) -> Option<VarToType> {
        self.vars.remove(&depth)
    }

    // スタックの最も上にある型環境に変数名と型を追加
    fn insert(&mut self, key: String, value: parser::TypeExpr) {
        if let Some(last) = self.vars.iter_mut().next_back() {
            last.1.insert(key, Some(value));
        }
    }

    // スタックを上からたどっていき、はじめに見つかる変数の型を取得
    fn get_mut(&mut self, key: &str) -> Option<(usize, &mut Option<parser::TypeExpr>)> {
        for (depth, elm) in self.vars.iter_mut().rev() {
            if let Some(e) = elm.get_mut(key) {
                return Some((*depth, e));
            }
        }
        None
    }
}

type TResult = Result<parser::TypeExpr, String>;

/// 型付け関数
/// 式を受け取り、型を返す
pub fn typing(expr: &parser::Expr, env: &mut TypeEnv, depth: usize) -> TResult {
    match expr {
        parser::Expr::App(e) => typing_app(e, env, depth),
        parser::Expr::QVal(e) => typing_qval(e, env, depth),
        parser::Expr::Free(e) => typing_free(e, env, depth),
        parser::Expr::If(e) => typing_if(e, env, depth),
        parser::Expr::Split(e) => typing_split(e, env, depth),
        parser::Expr::Var(e) => typing_var(e, env),
        parser::Expr::Let(e) => typing_let(e, env, depth),
    }
}

/// 関数適用の型付け
fn typing_app(expr: &parser::AppExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    // TODO: ここを実装せよ
    // todo!()は削除すること
    todo!()
}

/// 修飾子付き値の型付け
fn typing_qval(expr: &parser::QValExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    // プリミティブ型を計算
    let p = match &expr.val {
        parser::ValExpr::Bool(_) => parser::PrimType::Bool,
        parser::ValExpr::Pair(e1, e2) => {
            // 式e1とe2をtypingにより型付け
            let t1 = typing(e1, env, depth)?;
            let t2 = typing(e2, env, depth)?;

            // expr.qualがUnであり、
            // e1か、e2の型にlinが含まれていた場合、型付けエラー
            if expr.qual == parser::Qual::Un
                && (t1.qual == parser::Qual::Lin || t2.qual == parser::Qual::Lin)
            {
                return Err("un型のペア内でlin型を利用している".to_string());
            }

            // ペア型を返す
            parser::PrimType::Pair(Box::new(t1), Box::new(t2))
        }
        parser::ValExpr::Fun(e) => {
            // 関数の型付け

            // un型の関数内では、lin型の自由変数をキャプチャできないため
            // lin用の型環境を置き換え
            let env_prev = if expr.qual == parser::Qual::Un {
                Some(mem::take(&mut env.env_lin))
            } else {
                None
            };

            // depthをインクリメントしてpush
            let mut depth = depth;
            safe_add(&mut depth, &1, || {
                "変数スコープのネストが深すぎる".to_string()
            })?;
            env.push(depth);
            env.insert(e.var.clone(), e.ty.clone());

            // 関数中の式を型付け
            let t = typing(&e.expr, env, depth)?;

            // スタックをpopし、popした型環境の中にlin型が含まれていた場合、型付けエラー
            let (elin, _) = env.pop(depth);
            for (k, v) in elin.unwrap().iter() {
                if v.is_some() {
                    return Err(format!("関数定義内でlin型の変数\"{}\"を消費していない", k));
                }
            }

            // lin用の型環境を復元
            if let Some(ep) = env_prev {
                env.env_lin = ep;
            }

            // 関数型を返す
            parser::PrimType::Arrow(Box::new(e.ty.clone()), Box::new(t))
        }
    };

    // 修飾子付き型を返す
    Ok(parser::TypeExpr {
        qual: expr.qual,
        prim: p,
    })
}

/// free式の型付け
fn typing_free(expr: &parser::FreeExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    // TODO: ここを実装せよ
    // todo!()は削除すること
    todo!()
}

/// if式の型付け
fn typing_if(expr: &parser::IfExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    let t1 = typing(&expr.cond_expr, env, depth)?;
    // 条件の式の型はbool
    if t1.prim != parser::PrimType::Bool {
        return Err("ifの条件式がboolでない".to_string());
    }

    let mut e = env.clone();
    let t2 = typing(&expr.then_expr, &mut e, depth)?;
    let t3 = typing(&expr.else_expr, env, depth)?;

    // thenとelse部の型は同じで、
    // thenとelse部評価後の型環境は同じかをチェック
    if t2 != t3 || e != *env {
        return Err("ifのthenとelseの式の型が異なる".to_string());
    }

    Ok(t2)
}

/// split式の型付け
fn typing_split(expr: &parser::SplitExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    // TODO: ここを実装せよ
    // todo!()は削除すること
    todo!()
}

/// 変数の型付け
fn typing_var(expr: &str, env: &mut TypeEnv) -> TResult {
    let ret = env.get_mut(expr);
    if let Some(it) = ret {
        // 定義されている
        if let Some(t) = it {
            // 消費されていない
            if t.qual == parser::Qual::Lin {
                // lin型
                let eret = t.clone();
                *it = None; // linを消費
                return Ok(eret);
            } else {
                return Ok(t.clone());
            }
        }
    }

    Err(format!(
        "\"{}\"という変数は定義されていないか、利用済みか、キャプチャできない",
        expr
    ))
}

/// let式の型付け
fn typing_let(expr: &parser::LetExpr, env: &mut TypeEnv, depth: usize) -> TResult {
    // TODO: ここを実装せよ
    // todo!()は削除すること
    todo!()
}
