mod helper;
mod parser;
mod typing;

use nom::error::convert_error;
use std::{env, fs};

#[derive(Debug)]
enum LinError {
    Arguments,
    File,
    Typing,
    Parse,
}

fn main() -> Result<(), LinError> {
    // コマンドライン引数の検査
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("以下のようにファイル名を指定して実行してください\ncargo run codes/ex1.lin");
        return Err(LinError::Arguments);
    }

    // ファイル読み込み
    let content = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("エラー: {}", e);
            return Err(LinError::File);
        }
    };

    let ast = parser::parse_expr(&content); // パース
    println!("AST:\n{:#?}\n", ast);
    match ast {
        Ok((_, expr)) => {
            let mut ctx = typing::TypeEnv::new();
            println!("式:\n{}", content);

            // 型付け
            match typing::typing(&expr, &mut ctx, 0) {
                Ok(a) => {
                    println!("の型は\n{}\nです。", a);
                }
                Err(e) => {
                    eprintln!("型付けエラー: {}", e);
                    return Err(LinError::Typing);
                }
            }
        }
        Err(nom::Err::Error(e)) => {
            let msg = convert_error(content.as_str(), e);
            eprintln!("パースエラー:\n{}", msg);
            return Err(LinError::Parse);
        }
        _ => (),
    }

    Ok(())
}
