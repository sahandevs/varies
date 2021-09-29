use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use std::collections::HashMap;
use syn::ItemFn;
use syn::{parse_macro_input, Attribute, Expr, Stmt};

fn add_to_vars(var: &str, vars: &mut HashMap<String, Vec<Stmt>>, stmt: Stmt) {
    if !vars.contains_key(var) {
        let default_var = vars.get("default").unwrap().clone();
        vars.insert(var.into(), default_var);
    }
    for (cvar, stmts) in vars {
        if var == "default" {
            stmts.push(stmt.clone());
        } else if var == cvar {
            stmts.push(stmt);
            break;
        }
    }
}

fn get_attrs(expr: &Expr) -> &Vec<Attribute> {
    match expr {
        Expr::Array(x) => &x.attrs,
        Expr::Assign(x) => &x.attrs,
        Expr::AssignOp(x) => get_attrs(&x.left),
        Expr::Async(x) => &x.attrs,
        Expr::Await(x) => &x.attrs,
        Expr::Binary(x) => &x.attrs,
        Expr::Block(x) => &x.attrs,
        Expr::Box(x) => &x.attrs,
        Expr::Break(x) => &x.attrs,
        Expr::Call(x) => &x.attrs,
        Expr::Cast(x) => &x.attrs,
        Expr::Closure(x) => &x.attrs,
        Expr::Continue(x) => &x.attrs,
        Expr::Field(x) => &x.attrs,
        Expr::ForLoop(x) => &x.attrs,
        Expr::Group(x) => &x.attrs,
        Expr::If(x) => &x.attrs,
        Expr::Index(x) => &x.attrs,
        Expr::Let(x) => &x.attrs,
        Expr::Lit(x) => &x.attrs,
        Expr::Loop(x) => &x.attrs,
        Expr::Macro(x) => &x.attrs,
        Expr::Match(x) => &x.attrs,
        Expr::MethodCall(x) => &x.attrs,
        Expr::Paren(x) => &x.attrs,
        Expr::Path(x) => &x.attrs,
        Expr::Range(x) => &x.attrs,
        Expr::Reference(x) => &x.attrs,
        Expr::Repeat(x) => &x.attrs,
        Expr::Return(x) => &x.attrs,
        Expr::Struct(x) => &x.attrs,
        Expr::Try(x) => &x.attrs,
        Expr::TryBlock(x) => &x.attrs,
        Expr::Tuple(x) => &x.attrs,
        Expr::Type(x) => &x.attrs,
        Expr::Unary(x) => &x.attrs,
        Expr::Unsafe(x) => &x.attrs,
        Expr::Verbatim(_) => panic!("failed to parse"),
        Expr::While(x) => &x.attrs,
        Expr::Yield(x) => &x.attrs,
        _ => panic!("not supported"),
    }
}

fn clear_attrs(expr: &Expr) -> Expr {
    let target_attrs: Vec<_> = get_attrs(expr)
        .iter()
        .filter(|x| x.path.get_ident().unwrap().to_string() != "variant")
        .map(|x| x.clone())
        .collect();

    let expr = expr.clone();
    let attrs = get_attrs(&expr);
    let attrs = unsafe { &mut *(attrs as *const _ as *mut _) };
    *attrs = target_attrs;
    for att in attrs {
        eprintln!("{}", att.to_token_stream());
    }
    expr
}

#[proc_macro_attribute]
pub fn varies(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let default_fun_name = input.sig.ident.to_string();
    let fun_name = format_ident!("{}", default_fun_name);
    let mut vars = HashMap::new();
    vars.insert("default".into(), Vec::new());

    for stmt in &input.block.stmts {
        match stmt {
            Stmt::Semi(exp, s) => {
                let attrs = get_attrs(exp);
                if let Some(attr) = attrs
                    .iter()
                    .find(|x| x.path.get_ident().unwrap().to_string() == "variant")
                {
                    let exper = clear_attrs(exp);
                    let var_name = attr.tokens.to_string().replace("(", "").replace(")", "");
                    add_to_vars(&var_name, &mut vars, Stmt::Semi(exper, s.clone()));
                }
            }
            _ => add_to_vars("default", &mut vars, stmt.clone()),
        }
    }

    let fn_sig_str = &input.sig.to_token_stream().to_string();
    let block = &input.block;
    let variants: Vec<_> = vars
        .into_iter()
        .map(|(name, stmts)| {
            let fn_sig = fn_sig_str.replace(&default_fun_name, &name);

            let fn_sig: TokenStream = fn_sig.parse().unwrap();
            let mut block = block.clone();
            block.stmts = stmts;
            let block = block.to_token_stream();
            format!("pub {} {}", fn_sig, block)
        })
        .collect();

    let result = format!("pub mod {} {{\n{}\n}}", fun_name, variants.join("\n"));
    result.parse().unwrap()
}
