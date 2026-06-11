use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Lit, Meta, parse_macro_input};

#[proc_macro_derive(ConfigLoader, attributes(required, default, default_fn))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => panic!("ConfigLoader can only be derived for structs with named fields"),
        },
        _ => panic!("ConfigLoader can only be derived for structs"),
    };

    let field_initializers = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        let field_ty = &f.ty;
        let required = has_attr(&f.attrs, "required");

        let has_default = has_attr(&f.attrs, "default");
        let has_default_fn = has_attr(&f.attrs, "default_fn");
        if has_default && has_default_fn {
            panic!(
                "Field `{}` cannot have both `#[default = ...]` and `#[default_fn = ...]`",
                field_name
            );
        }

        let default_literal = get_default_literal(&f.attrs);

        let default_fn = get_default_fn(&f.attrs, field_ty);

        quote! {
            let #field_name = {
                let is_required = #required;
                let default_literal = #default_literal;
                let default_fn = #default_fn;
                ::unified_config_loader::resolve_value_with_fn(
                    #field_str,
                    is_required,
                    default_literal,
                    default_fn,
                    &file_kvs,
                )?
            };
        }
    });

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());

    let schema_props: Vec<String> = fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap().to_string();
            let required = has_attr(&f.attrs, "required");
            let default = extract_default_string(&f.attrs)
                .map(|s| format!("\"{}\"", s))
                .unwrap_or_else(|| "null".to_string());
            let ty = type_to_schema_type(&f.ty);
            format!(
                r#"  "{}": {{ "type": "{}", "default": {}, "required": {} }}"#,
                name, ty, default, required
            )
        })
        .collect();

    let schema_json = format!(
        "{{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"type\": \"object\",\n  \"properties\": {{\n{}\n  }}\n}}",
        schema_props.join(",\n")
    );

    let expanded = quote! {
        impl ::unified_config_loader::Config for #name {
            fn load() -> ::std::result::Result<Self, ::unified_config_loader::ConfigError> {
                let file_path = ::std::env::var("CONFIG_FILE").ok();
                let file_kvs: Option<
                    ::std::collections::HashMap<String, String>,
                > = if let Some(path) = file_path {
                    Some(::unified_config_loader::load_config_file(&path)?)
                } else {
                    None
                };

                #(#field_initializers)*

                ::std::result::Result::Ok(#name {
                    #(#field_names),*
                })
            }
        }

        impl #name {
            pub fn schema() -> &'static str {
                #schema_json
            }
        }
    };

    TokenStream::from(expanded)
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn get_default_literal(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("default") {
            if let Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(expr_lit) = &meta.value
                && let Lit::Str(lit) = &expr_lit.lit
            {
                return quote! { ::std::option::Option::Some(#lit) };
            }
            return quote! { ::std::option::Option::None };
        }
    }
    quote! { ::std::option::Option::None }
}

fn get_default_fn(attrs: &[syn::Attribute], field_ty: &syn::Type) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("default_fn") {
            if let Meta::NameValue(meta) = &attr.meta
                && let syn::Expr::Lit(expr_lit) = &meta.value
                && let Lit::Str(lit) = &expr_lit.lit
            {
                let path: syn::Path =
                    syn::parse_str(&lit.value()).expect("default_fn must be a valid function path");
                // Cast to the exact function pointer type required by the field
                return quote! {
                    ::std::option::Option::Some(
                        #path as fn() -> ::std::result::Result<#field_ty, ::unified_config_loader::ConfigError>
                    )
                };
            }
            return quote! { ::std::option::Option::None };
        }
    }
    quote! { ::std::option::Option::None }
}
fn type_to_schema_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            let ident = &last_segment.ident;
            match ident.to_string().as_str() {
                "String" | "str" => "string".to_string(),
                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => {
                    "integer".to_string()
                }
                "f32" | "f64" => "number".to_string(),
                "bool" => "boolean".to_string(),
                _ => "string".to_string(),
            }
        }
        _ => "string".to_string(),
    }
}

fn extract_default_string(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("default")
            && let Meta::NameValue(meta) = &attr.meta
            && let syn::Expr::Lit(expr_lit) = &meta.value
            && let Lit::Str(lit) = &expr_lit.lit
        {
            return Some(lit.value());
        }
    }
    None
}
