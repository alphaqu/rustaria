use proc_macro::TokenStream;
use std::time::Instant;

use quote::{format_ident, quote, ToTokens};
use syn::{Expr, ExprMethodCall, ReturnType, Stmt};

#[proc_macro_attribute]
pub fn profile(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected fn")
    };


    let ident = fn_item.sig.ident.to_string();
    let varname = format_ident!("_{}",  &ident);

    let start = syn::parse(quote!(let #varname = std::time::Instant::now();).into()).unwrap();
    // add sub methods

    let arr_name = format_ident!("_sm_arr{}",  &ident);

    let array_create = syn::parse(quote!(
        let mut #arr_name = Vec::new();
    ).into()).unwrap();

    fn_item.block.stmts.insert(0, start);

    let mut sub_methods = Vec::new();
    for x in &fn_item.block.stmts {
        if let Stmt::Expr(expr) = x {
            if let Expr::MethodCall(ExprMethodCall { method, .. }) = expr {
                sub_methods.push(method.to_string());
            }
        }
    }


    for x in sub_methods {
        let sub_method_name = format_ident!("{}",  &ident);
        fn_item.block.stmts.insert(0, syn::parse(quote!(
               #arr_name.push(#sub_method_name);
         ).into()).unwrap());
    }


    fn_item.block.stmts.insert(0, array_create);


    let end = syn::parse(quote!(
        profiler_service::method_time(#ident, #varname.elapsed(), #arr_name);
    ).into()).unwrap();


    let leng = fn_item.block.stmts.len();

    //for x in &fn_item.block.stmts {
    //    if let Stmt::Expr(expr) = some {
    //        if let Stmt::Expr(expr) = some {}
    //    }
    //}
    if let ReturnType::Type(arrow, box_type) = &fn_item.sig.output {
        fn_item.block.stmts.insert(leng - 1, end);
    } else {
        fn_item.block.stmts.push(end);
    }

    item.into_token_stream().into()
}