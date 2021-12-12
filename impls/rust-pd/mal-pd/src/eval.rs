use crate::{
    ast::{MalFunc, MalHashMap, MalList, MalType, MalVec},
    env::MalEnv,
    Result,
};
use std::rc::Rc;
pub fn eval_ast(mt: MalType, env: &Rc<MalEnv>) -> Result<MalType> {
    if let MalType::Symbol(s) = mt {
        if let Some(env) = MalEnv::find(env, &s) {
            return Ok(env.as_ref().borrow().get(&s).unwrap().clone());
        }
        return Err(format!("symbol {} not found", s).into());
    } else if let MalType::List(l) = mt {
        let mut v: Vec<MalType> = Vec::new();
        for item in l.0 {
            let res = eval(item, env.clone())?;
            v.push(res);
        }
        return Ok(MalType::List(MalList::new(v)));
    } else if let MalType::Vector(l) = mt {
        let mut v: Vec<MalType> = Vec::new();
        for item in l.0 {
            let res = eval(item, env.clone())?;
            v.push(res);
        }
        return Ok(MalType::Vector(MalVec::new(v)));
    } else if let MalType::HashMap(h) = mt {
        let mut hm: Vec<(MalType, MalType)> = Vec::new();
        for (k, v) in h.0 {
            let v = eval(v, env.clone())?;
            hm.push((k, v))
        }
        return Ok(MalType::HashMap(MalHashMap::new(hm)));
    }
    Ok(mt)
}

pub fn eval(mut mt: MalType, mut env: Rc<MalEnv>) -> Result<MalType> {
    loop {
        if let MalType::List(MalList(l)) = &mt {
            if l.len() == 0 {
                return Ok(mt);
            }
            match &l[0] {
                MalType::Symbol(ms) if ms.strcmp("def!") => {
                    let k = &l[1];
                    let v = eval(l[2].clone(), env.clone())?;
                    env.set(k, v.clone())?;
                    return Ok(v);
                }
                MalType::Symbol(ms) if ms.strcmp("let*") => {
                    let new_env = MalEnv::detach(&env);
                    let (a1, a2) = (l[1].clone(), l[2].clone());
                    match a1 {
                        MalType::List(MalList(l)) | MalType::Vector(MalVec(l)) => {
                            if l.len() & 1 != 0 {
                                return Err("even amount of bindings expected".to_string().into());
                            }
                            for kv in l.chunks(2) {
                                let (k, v) = (&kv[0], &kv[1]);
                                let v = eval(v.clone(), new_env.clone())?;
                                new_env.set(k, v)?;
                            }
                        }
                        _ => {
                            return Err("Expected list or vector".to_string().into());
                        }
                    }
                    env = new_env;
                    mt = a2;
                    continue;
                }
                MalType::Symbol(ms) if ms.strcmp("do") => {
                    for mt in l[1..l.len() - 1].iter() {
                        eval(mt.clone(), env.clone())?;
                    }
                    mt = l[l.len() - 1].clone();
                    continue;
                }
                MalType::Symbol(ms) if ms.strcmp("if") => {
                    let cond = eval(l[1].clone(), env.clone())?;
                    if l.len() < 3 {
                        return Err(format!("if expects three arguments").into());
                    }
                    let a = match cond {
                        MalType::Nil | MalType::Bool(false) => {
                            if l.len() >= 4 {
                                l[3].clone()
                            } else {
                                MalType::Nil
                            }
                        }
                        _ => l[2].clone(),
                    };
                    mt = a;
                    continue;
                }
                MalType::Symbol(ms) if ms.strcmp("fn*") => {
                    let a1 = l[1].clone();
                    let mut args = Vec::new();
                    match a1 {
                        MalType::List(MalList(l)) | MalType::Vector(MalVec(l)) => {
                            for mt in l.into_iter() {
                                if let MalType::Symbol(ms) = mt {
                                    args.push(ms);
                                } else {
                                    return Err("Expected symbol in args".into());
                                }
                            }
                        }
                        _ => {
                            return Err("Expected list or vector".to_string().into());
                        }
                    }
                    let body = l[2].clone();
                    return Ok(MalType::Func(Box::new(MalFunc::from_binds(
                        args, body, &env,
                    ))));
                }
                MalType::Symbol(ms) if ms.strcmp("eval") => {
                    mt = eval(l[1].clone(), env.clone())?;
                    if let Some(e) = &env.outer {
                        env = e.clone();
                    }
                    continue;
                }
                _ => {
                    if let MalType::List(MalList(l)) = eval_ast(mt, &env)? {
                        let f = l[0].clone();
                        if let MalType::Func(f) = f {
                            return f.call(l[1..].to_vec());
                        } else {
                            return Err("expected function as first argument".to_string().into());
                        }
                    }
                    return Err("expected function and args in a list".to_string().into());
                }
            };
        } else {
            return eval_ast(mt, &env);
        }
    }
}
