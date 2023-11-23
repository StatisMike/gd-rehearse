use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use venial::{Declaration, Error, FnParam, Function};

use crate::{
    parser::{AttributeIdent, AttributeValueParser},
    utils::bail,
};

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
    let mut keyword = quote! { None };

    let mut parser =
        AttributeValueParser::from_attribute_group_at_path(&func.attributes, "gditest")?;

    while let Some(ident) = parser.get_one_of_idents(&[
        AttributeIdent::Focus,
        AttributeIdent::Skip,
        AttributeIdent::Keyword,
    ])? {
        match ident {
            AttributeIdent::Focus => {
                focused = true;
                parser.progress_puct();
            }
            AttributeIdent::Skip => {
                skipped = true;
                parser.progress_puct();
            }
            AttributeIdent::Keyword => {
                parser.pop_equal_sign()?;
                let keyword_literal = parser.get_literal()?;
                keyword = quote! { Some(#keyword_literal) };
                parser.progress_puct();
            }
            _ => unreachable!(),
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
        quote! { __unused_context: &::godot_test::CaseContext }
    };

    let body = &func.body;

    Ok(quote! {
        pub fn #test_name(#param) {
            #body
        }

        ::godot::sys::plugin_add!(GODOT_TEST_RUST_TEST_CASES; ::godot_test::itest::RustTestCase {
            name: #test_name_str,
            skipped: #skipped,
            focused: #focused,
            keyword: #keyword,
            file: std::file!(),
            line: std::line!(),
            function: #test_name,
        });

        // ::godot_test::rust_test_case_add!(::godot_test::itest::RustTestCase {
        //     name: #test_name_str,
        //     skipped: #skipped,
        //     focused: #focused,
        //     file: std::file!(),
        //     line: std::line!(),
        //     function: #test_name,
        // });
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
