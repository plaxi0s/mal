extern crate mal;

use mal::eval;
use mal::parse;
use mal::tokens::MalTokens;
use mal::Editor;
use mal::Finish;
use mal::MalEnv;
use mal::MalLexer;
use mal::ReadlineError;
fn main() {
    let mut rl = Editor::<()>::new();
    loop {
        let r = rl.readline("user> ");
        match r {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if line.len() > 0 {
                    match MalLexer::lex(line.as_str()) {
                        Ok(t) => {
                            let t = MalTokens(&t.as_slice());
                            match parse(t).finish() {
                                Ok((_, ast)) => {
                                    let env = MalEnv::new();
                                    match eval(ast, &env) {
                                        Ok(res) => println!("{}", res),
                                        Err(e) => {
                                            println!("Error in eval {}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("unbalanced {:?}", e);
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}
