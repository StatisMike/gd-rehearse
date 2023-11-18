use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use venial::{Declaration, Error, FnParam, Function};

use crate::utils::bail;

pub fn attribute_gditest(input_decl: Declaration) -> Result<TokenStream, Error> {
    let func = match input_decl {
        Declaration::Function(f) => f,
        _ => return bail!(&input_decl, "#[gditest] can only be applied to functions"),
    };

    // Note: allow attributes for things like #[rustfmt] or #[clippy]
    if func.generic_params.is_some()
        || func.params.len() > 1
        || func.return_ty.is_some()
        || func.where_clause.is_some()
    {
        return bad_signature(&func);
    }

    let mut skipped = false;
    let mut focused = false;

    for attr in func.attributes.iter() {
        if attr.path.len() == 1 && attr.path[0].to_string() == "gditest" {
            if let venial::AttributeValue::Group(_span, tokens) = &attr.value {
                for token in tokens {
                    if let proc_macro2::TokenTree::Ident(ident) = token {
                        let stringified = ident.to_string();
                        if stringified == "skip" {
                            skipped = true;
                        }
                        if stringified == "focus" {
                            focused = true;
                        }
                    }
                }
            }
        }
    }

    if skipped && focused {
        return bail!(
            func.name,
            "#[gditest]: keys `skip` and `focus` are mutually exclusive",
        );
    }

    let test_name = &func.name;
    let test_name_str = func.name.to_string();

    // Detect parameter name chosen by user, or unused fallback
    let param = if let Some((param, _punct)) = func.params.first() {
        if let FnParam::Typed(param) = param {
            // Correct parameter type (crude macro check) -> reuse parameter name
            let is_context = param
                .ty
                .tokens
                .last()
                .map(|last| last.to_string() == "TestContext")
                .unwrap_or(false);
            if is_context {
                param.to_token_stream()
            } else {
                return bad_signature(&func);
            }
        } else {
            return bad_signature(&func);
        }
    } else {
        quote! { __unused_context: &::godot_test::itest::TestContext }
    };

    let body = &func.body;

    Ok(quote! {
        pub fn #test_name(#param) {
            #body
        }

        ::godot_test::itest::register_rust_case(::godot_test::itest::RustTestCase {
            name: #test_name_str,
            skipped: #skipped,
            focused: #focused,
            file: std::file!(),
            line: std::line!(),
            function: #test_name,
        });
    })
}

fn bad_signature(func: &Function) -> Result<TokenStream, Error> {
    bail!(
        func,
        "#[gditest] function must have one of these signatures:\
        \n  fn {f}() {{ ... }}\
        \n  fn {f}(ctx: &TestContext) {{ ... }}",
        f = func.name,
    )
}
