use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

enum ConfigOption {
    Default { value: Lit },
    DefaultFn { path: LitStr },
    Required,
    EnvPrefix { prefix: LitStr },
    FilePath { file_path: LitStr },
}

impl Parse for ConfigOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "default" => {
                input.parse::<Token![=]>()?;
                let value: Lit = input.parse()?;
                Ok(ConfigOption::Default { value })
            }
            "default_fn" => {
                input.parse::<Token![=]>()?;
                let path: LitStr = input.parse()?;
                Ok(ConfigOption::DefaultFn { path })
            }
            "required" => Ok(ConfigOption::Required),
            "env_prefix" => {
                input.parse::<Token![=]>()?;
                let prefix: LitStr = input.parse()?;
                Ok(ConfigOption::EnvPrefix { prefix })
            }
            "file_path" => {
                input.parse::<Token![=]>()?;
                let file_path: LitStr = input.parse()?;
                Ok(ConfigOption::FilePath { file_path })
            }
            other => Err(input.error(format!("unknown config option `{}`", other))),
        }
    }
}

struct ConfigArgs {
    options: Punctuated<ConfigOption, Token![,]>,
}

impl Parse for ConfigArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ConfigArgs {
            options: input.parse_terminated(ConfigOption::parse, Token![,])?,
        })
    }
}

fn collect_config_options(attrs: &[syn::Attribute]) -> Vec<ConfigOption> {
    let mut options = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("config")
            && let Ok(args) = attr.parse_args::<ConfigArgs>()
        {
            options.extend(args.options);
        }
    }
    options
}

#[proc_macro_derive(ConfigLoader, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = if let syn::Data::Struct(ref data) = input.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            &fields.named
        } else {
            panic!("Config can only be derived for structs with named fields");
        }
    } else {
        panic!("Config can only be derived for structs");
    };

    // ---------- struct‑level options ----------
    let struct_opts = collect_config_options(&input.attrs);

    let env_prefix = struct_opts
        .iter()
        .find_map(|opt| {
            if let ConfigOption::EnvPrefix { prefix } = opt {
                Some(prefix.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "APP_".to_string());

    let file_path = struct_opts
        .iter()
        .find_map(|opt| {
            if let ConfigOption::FilePath { file_path } = opt {
                Some(file_path.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "".to_string());

    let field_initializers = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();
        let opts = collect_config_options(&f.attrs);

        let mut required = false;
        let mut default_lit: Option<String> = None; // string representation
        let mut default_fn_path: Option<String> = None;

        for opt in opts {
            match opt {
                ConfigOption::Required => required = true,
                ConfigOption::Default { value } => {
                    default_lit = Some(lit_to_string(&value));
                }
                ConfigOption::DefaultFn { path } => {
                    default_fn_path = Some(path.value());
                }
                _ => {} // ignore others on field level
            }
        }

        // Convert the default literal to a token stream (Option<&str>)
        let default_lit_tokens = match default_lit {
            Some(val) => quote! { ::std::option::Option::Some(#val) },
            None => quote! { ::std::option::Option::None },
        };

        // Convert the default_fn path to a function pointer token stream
        let default_fn_tokens = match default_fn_path {
            Some(fn_path) => {
                let path: syn::Path = syn::parse_str(&fn_path)
                    .expect("default_fn must be a valid function path");
                quote! {
                    ::std::option::Option::Some(#path as fn() -> ::std::result::Result<_, ::unified_config_loader::ConfigError>)
                }
            }
            None => quote! { ::std::option::Option::None },
        };

        let required_lit = required; // bool directly

        quote! {
            let #field_name: _ = {
                let is_required: bool = #required_lit;
                let default_fn: ::std::option::Option<fn() -> ::std::result::Result<_, ::unified_config_loader::ConfigError>> = #default_fn_tokens;
                ::unified_config_loader::resolve_value_with_fn(
                    #field_str,
                    is_required,
                    #default_lit_tokens,
                    default_fn,
                    &merged_map,
                )?
            };
        }
    });

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());

    // ---------- schema generation ----------
    let schema_fields = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap().to_string();
        let opts = collect_config_options(&f.attrs);
        let required = opts.iter().any(|o| matches!(o, ConfigOption::Required));
        let default_json = opts
            .iter()
            .find_map(|o| {
                if let ConfigOption::Default { value } = o {
                    Some(lit_to_json_string(value))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "null".to_string());
        let field_type = type_to_schema_type(&f.ty);

        quote! {
            ::std::format!(
                "    \"{}\": {{ \"type\": \"{}\", \"default\": {}, \"required\": {} }}",
                #field_name,
                #field_type,
                #default_json,
                #required
            )
        }
    });

    let expanded = quote! {
        impl ::unified_config_loader::Config for #name {
            fn load() -> ::std::result::Result<Self, ::unified_config_loader::ConfigError> {
                use ::std::collections::HashMap;
                use ::std::path::Path;
                use ::std::fs;
                use ::unified_config_loader::{load_config_file, load_env_vars, ConfigError};

                let mut merged_map: HashMap<String, (String, ValueSource)> = HashMap::new();
                let config_file = std::env::var("APP_CONFIG_FILE").ok();

                fn load_if_exists(
                    map: &mut HashMap<String, (String, ValueSource)>,
                    path: &Path,
                ) -> Result<(), ConfigError> {
                    if path.exists() {
                        let path_str = path.to_str().ok_or(ConfigError::InvalidPath)?;
                        let file_map = load_config_file(path_str)?;
                        map.extend(file_map);
                    }
                    Ok(())
                }

                if #file_path.is_empty() && config_file.is_none() {
                    // conventional
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    load_if_exists(&mut merged_map, &Path::new(manifest_dir).join(".env"))?;
                    load_if_exists(&mut merged_map, &Path::new(manifest_dir).join(".env.local"))?;
                    // Any other .env.* files
                    if let Ok(entries) = fs::read_dir(manifest_dir) {
                        let mut env_files = Vec::new();
                        for entry in entries {
                            let entry = match entry {
                                Ok(e) => e,
                                Err(e) => return Err(ConfigError::FileRead{path: manifest_dir.to_string(), source: e}),
                            };
                            let path = entry.path();
                            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                                if name.starts_with(".env.") {
                                    env_files.push(path);
                                }
                            }
                        }
                        env_files.sort();
                        for path in env_files {
                            load_if_exists(&mut merged_map, &path)?;
                        }
                    }
                    // ---- config.* files ----
                    for ext in &["toml", "json", "yaml", "ini"] {
                        let path = Path::new(manifest_dir).join(format!("config.{}", ext));
                        load_if_exists(&mut merged_map, &path)?;
                    }
                } else if let Some(ref path) = config_file {
                    // single file from APP_CONFIG_FILE env
                    load_if_exists(&mut merged_map, &Path::new(path))?;
                } else {
                    // single file from struct level attrib 'file_path'
                    load_if_exists(&mut merged_map, &Path::new(#file_path))?;

                }

                // ---- environment variables ----
                let env_map = load_env_vars(#env_prefix);
                merged_map.extend(env_map);

                #(#field_initializers)*

                ::std::result::Result::Ok(#name {
                    #(#field_names),*
                })
            }
        }

        impl #name {
            pub fn schema() -> String {
                let props = vec![#(#schema_fields),*].join(",\n");
                let schema_json = ::std::format!(
                    "{{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"type\": \"object\",\n  \"properties\": {{\n{}\n  }}\n}}",
                    props
                );
                schema_json
            }
        }
    };

    TokenStream::from(expanded)
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

/// Convert a literal to a string that can be used in `Option<&str>`.
fn lit_to_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(s) => s.value(),
        Lit::Int(i) => i.base10_digits().to_string(),
        Lit::Float(f) => f.base10_digits().to_string(),
        Lit::Bool(b) => b.value.to_string(),
        _ => format!("{}", quote!(#lit)), // fallback for other literals
    }
}

/// Convert a literal to a JSON representation (quoted string or bare number/boolean).
fn lit_to_json_string(lit: &Lit) -> String {
    match lit {
        Lit::Str(s) => format!("\"{}\"", s.value()),
        Lit::Int(i) => i.base10_digits().to_string(),
        Lit::Float(f) => f.base10_digits().to_string(),
        Lit::Bool(b) => b.value.to_string(),
        _ => "null".to_string(),
    }
}
