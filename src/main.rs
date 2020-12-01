use std::{env, fs};

mod parser;
mod typing;

fn main() {
    // コマンドライン引数の検査
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("以下のようにファイル名を指定して実行してください\ncargo run examples/ex1.lambda");
        return;
    }

    // ファイル読み込み
    let content = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            println!("エラー: {}", e);
            return;
        }
    };

    let ast = parser::parse_expr(&content);
    // このコメントを外すと抽象構文木が表示される
    // println!("AST: {:#?}", ast);
    match ast {
        Ok((_, expr)) => {
            let mut ctx = typing::Context::new();
            if let Some(a) = typing::typing(&expr, &mut ctx) {
                println!("式\n{}", content);
                println!("の型は\n{:#?}\nです。", a);
            } else {
                println!("型付けエラー");
            }
        }
        Err(e) => {
            println!("パースエラー: {:#?}", e);
        }
    }
}
