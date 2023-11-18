use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use venial::{AttributeValue, Declaration, Error, Function};

use crate::utils::bail;

const DEFAULT_REPETITIONS: usize = 100;

pub fn attribute_bench(input_decl: Declaration) -> Result<TokenStream, venial::Error> {
    let func = match input_decl {
        Declaration::Function(f) => f,
        _ => return bail!(&input_decl, "#[bench] can only be applied to functions"),
    };

    // Note: allow attributes for things like #[rustfmt] or #[clippy]
    if func.generic_params.is_some() || !func.params.is_empty() || func.where_clause.is_some() {
        return bad_signature(&func);
    }

    // Ignore -> (), as no one does that by accident.
    // We need `ret` to make sure the type is correct and to avoid unused imports (by IDEs).
    let Some(ret) = func.return_ty else {
        return bail!(
            func,
            "#[bench] function must return a value from its computation, to prevent optimizing the operation away"
        );
    };

    let mut repeats = DEFAULT_REPETITIONS;

    for attr in &mut func.attributes.clone() {
        if attr.path.len() == 1 && attr.path[0].to_string() == "gdbench" {
            if let AttributeValue::Group(_, tokens) = &mut attr.value {
                if !tokens.is_empty() {
                    tokens.reverse();

                    let Some(ident) = tokens.pop() else {
                        return Err(venial::Error::new_at_tokens(
                            &attr.value,
                            "expected 'repeat' identifier",
                        ));
                    };
                    let _ident = match ident {
                        TokenTree::Ident(ident) if ident == "repeat" => ident,
                        _ => {
                            return Err(venial::Error::new_at_tokens(
                                &attr.value,
                                "expected 'repeat' identifier",
                            ))
                        }
                    };

                    let Some(sign) = tokens.pop() else {
                        return Err(venial::Error::new_at_tokens(
                            &attr.value,
                            "expected 'repeat' identifier",
                        ));
                    };
                    let _sign = match sign {
                        TokenTree::Punct(punct) if punct.to_string() == "=" => punct,
                        _ => {
                            return Err(venial::Error::new_at_tokens(
                                &attr.value,
                                "expected equal sign",
                            ))
                        }
                    };

                    let reps = tokens.pop().ok_or_else(|| {
                        venial::Error::new_at_tokens(&attr.value, "expected int literal")
                    })?;
                    let reps = match reps {
                        TokenTree::Literal(reps) => reps,
                        _ => {
                            return Err(venial::Error::new_at_tokens(
                                &attr.value,
                                "expected int literal",
                            ))
                        }
                    };

                    repeats = reps.to_string().parse().map_err(|_| {
                        venial::Error::new_at_tokens(&attr.value, "expected int literal")
                    })?;
                }
            }
        }
    }

    let bench_name = &func.name;
    let bench_name_str = func.name.to_string();

    let body = &func.body;

    Ok(quote! {
        pub fn #bench_name() {
            for _ in 0..#repeats {
                let __ret: #ret = #body;
                ::godot_test::bench::bench_used(__ret);
            }
        }

        ::godot_test::bench::register_benchmark(::godot_test::bench::RustBenchmark {
          name: #bench_name_str,
          file: std::file!(),
          line: std::line!(),
          function: #bench_name,
          repetitions: #repeats,
        })
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
