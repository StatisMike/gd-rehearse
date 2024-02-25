/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::parser::{AttributeIdent, AttributeValueParser};
use crate::utils::bail;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use venial::{Declaration, Error, FnParam, Function};

const DEFAULT_REPETITIONS: usize = 100;

pub fn attribute_bench(input_decl: Declaration) -> Result<TokenStream, venial::Error> {
    let func = match input_decl {
        Declaration::Function(f) => f,
        _ => return bail!(&input_decl, "#[gdbench] can only be applied to functions"),
    };

    // Note: allow attributes for things like #[rustfmt] or #[clippy]
    if func.generic_params.is_some() || func.params.len() > 1 || func.where_clause.is_some() {
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
    let mut scene_path = quote! { None };
    let mut setup_function: Option<Ident> = None;
    let mut cleanup_function: Option<Ident> = None;

    let mut parser =
        AttributeValueParser::from_attribute_group_at_path(&func.attributes, "gdbench")?;

    while let Some(ident) = parser.get_one_of_idents(&[
        AttributeIdent::Focus,
        AttributeIdent::Skip,
        AttributeIdent::Repeat,
        AttributeIdent::Keyword,
        AttributeIdent::ScenePath,
        AttributeIdent::Setup,
        AttributeIdent::Cleanup,
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
            AttributeIdent::ScenePath => {
                parser.pop_equal_sign()?;
                let scene_path_lit = parser.get_literal_scene_path()?;
                scene_path = quote! { Some( #scene_path_lit ) };
                parser.progress_puct();
            }
            AttributeIdent::Setup => {
                parser.pop_equal_sign()?;
                setup_function = Some(parser.get_ident()?);
                parser.progress_puct();
            }
            AttributeIdent::Cleanup => {
                parser.pop_equal_sign()?;
                cleanup_function = Some(parser.get_ident()?);
                parser.progress_puct();
            }
        }
    }

    if skipped && focused {
        return bail!(
            &func.name,
            "#[gdbench]: keys `skip` and `focus` are mutually exclusive",
        );
    }

    if cleanup_function.is_some() && setup_function.is_none() {
        return bail!(
            &cleanup_function,
            "#[gdbench]: Cleanup function is unneeded if setup function is not provided",
        );
    }

    let setup_function = if let Some(setup) = setup_function {
        quote! { Some(#setup) }
    } else {
        quote! { None }
    };

    let cleanup_function = if let Some(cleanup) = cleanup_function {
        quote! { Some(#cleanup) }
    } else {
        quote! { None }
    };

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
                .map(|last| last.to_string() == "BenchContext")
                .unwrap_or(false);
            if is_context {
                param.to_token_stream()
            } else {
                return bad_signature(&func);
            }
        } else {
            // TokenStream::new()
            return bad_signature(&func);
        }
    } else {
        quote! { __unused_context: &::gd_rehearse::bench::BenchContext }
    };

    let body = &func.body;

    Ok(quote! {
        pub fn #bench_name(#param) {
            for _ in 0..#repeats {
                let __ret: #ret = #body;
                ::gd_rehearse::bench::bench_used(__ret);
            }
        }

        ::godot::sys::plugin_add!{GD_REHEARSE_RUST_BENCHMARKS in gd_rehearse::bench; ::gd_rehearse::bench::RustBenchmark {
          name: #bench_name_str,
          focused: #focused,
          skipped: #skipped,
          keyword: #keyword,
          file: std::file!(),
          line: std::line!(),
          function: #bench_name,
          repetitions: #repeats,
          scene_path: #scene_path,
          setup_function: #setup_function,
          cleanup_function: #cleanup_function
        }}
    })
}

fn bad_signature(func: &Function) -> Result<TokenStream, Error> {
    bail!(
        func,
        "#[gdbench] function must have one of these signatures:\
        \n  fn {f}() -> Type {{ ... }}\
        \n  fn {f}(ctx: &BenchContext) -> Type {{ ... }}",
        f = func.name,
    )
}
