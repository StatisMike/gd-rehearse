use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use venial::{Declaration, Error, FnParam, Function};

use crate::{
    parser::{AttributeIdent, AttributeValueParser},
    utils::bail,
};

const DEFAULT_REPETITIONS: usize = 100;

pub fn attribute_bench(input_decl: Declaration) -> Result<TokenStream, venial::Error> {
    let func = match input_decl {
        Declaration::Function(f) => f,
        _ => return bail!(&input_decl, "#[gdbench] can only be applied to functions"),
    };

    // Note: allow attributes for things like #[rustfmt] or #[clippy]
    if func.generic_params.is_some() || !func.params.is_empty() || func.where_clause.is_some() {
        return bad_signature(&func);
    }

    // Ignore -> (), as no one does that by accident.
    // We need `ret` to make sure the type is correct and to avoid unused imports (by IDEs).
    let Some(ret) = &func.return_ty else {
        return bail!(
            func,
            "#[gdbench] function must return a value from its computation, to prevent optimizing the operation away"
        );
    };

    let mut repeats = DEFAULT_REPETITIONS;
    let mut focused = false;
    let mut skipped = false;
    let mut keyword = quote! { None };

    let mut parser =
        AttributeValueParser::from_attribute_group_at_path(&func.attributes, "gdbench")?;

    while let Some(ident) = parser.get_one_of_idents(&[
        AttributeIdent::Focus,
        AttributeIdent::Skip,
        AttributeIdent::Repeat,
        AttributeIdent::Keyword,
    ])? {
        match ident {
            AttributeIdent::Repeat => {
                parser.pop_equal_sign()?;
                let repeats_lit = parser.get_literal()?;
                repeats = repeats_lit
                    .to_string()
                    .parse::<usize>()
                    .map_err(|_| venial::Error::new("expected integer"))?;
                parser.progress_puct();
            }
            AttributeIdent::Focus => {
                focused = true;
                parser.progress_puct()
            }
            AttributeIdent::Skip => {
                skipped = true;
                parser.progress_puct()
            }
            AttributeIdent::Keyword => {
                parser.pop_equal_sign()?;
                let keyword_lit = parser.get_literal()?;
                keyword = quote! { Some( #keyword_lit ) };
                parser.progress_puct();
            }
        }
    }

    if skipped && focused {
        return bail!(
            &func.name,
            "#[gditest]: keys `skip` and `focus` are mutually exclusive",
        );
    }

    let bench_name = &func.name;
    let bench_name_str = func.name.to_string();

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
        pub fn #bench_name(#param) {
            for _ in 0..#repeats {
                let __ret: #ret = #body;
                ::godot_test::bench::bench_used(__ret);
            }
        }

        ::godot::sys::plugin_add!{GODOT_TEST_RUST_BENCHMARKS; ::godot_test::bench::RustBenchmark {
          name: #bench_name_str,
          focused: #focused,
          skipped: #skipped,
          keyword: #keyword,
          file: std::file!(),
          line: std::line!(),
          function: #bench_name,
          repetitions: #repeats,
        }}
    })
}

fn bad_signature(func: &Function) -> Result<TokenStream, Error> {
    bail!(
        func,
        "#[bench] function must have one of these signatures:\
        \n  fn {f}() {{ ... }}",
        f = func.name,
    )
}
