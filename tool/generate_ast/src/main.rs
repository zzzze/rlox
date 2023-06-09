extern crate proc_macro;
use syn::parse_file;
use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;
use prettyplease::unparse;
use std::{io::Write, path::PathBuf, vec};
use phf::phf_map;

static RENAME_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "Literal" => "LoxLiteral",
};

static HAS_LIFETIME_OBJECTS: &[&'static str] = &[
    "Expr",
    "Token",
    "LoxLiteral",
];

#[derive(Debug)]
struct FieldInfo<'a> {
    field_name: &'a str,
    field_type: &'a str,
}

#[derive(Debug)]
struct StructInfo<'a> {
    name: &'a str,
    fields: Vec<FieldInfo<'a>>
}

#[derive(Debug)]
struct FileInfo<'a> {
    dependencies: &'a[&'a str],
    base_object_name: &'a str,
    variant_info_list: Vec<&'a str>,
}

fn parse_struct_info(definition: &str) -> Option<StructInfo> {
    if let [struct_name, field_info_str] = definition.split(":").collect::<Vec<&str>>()[..] {
        let mut field_info_list: Vec<FieldInfo> = Vec::new();
        for field in field_info_str.split(",") {
            if let [field_type, field_name] = field.trim().split(" ").collect::<Vec<&str>>()[..] {
                field_info_list.push(FieldInfo {
                    field_name: field_name.trim(),
                    field_type: field_type.trim(),
                });
            }
        }
        return Some(StructInfo {
            name: struct_name.trim(),
            fields: field_info_list,
        });
    }
    None
}

fn define_base_object(base_object_name: &str, variants: &Vec<StructInfo>) -> TokenStream {
    let struct_name = Ident::new(base_object_name, Span::call_site());
    let variant_names: Vec<&str> = variants.iter().map(|variant| {
        variant.name
    }).collect();
    let has_lifetime = enum_has_lifetime(variants);
    let variant_methods: Vec<TokenStream> = variant_names.iter().map(|variant_name| {
        let identifier = Ident::new(variant_name, Span::call_site());
        let identifier_lowercase = Ident::new(&variant_name.to_lowercase(), Span::call_site());
        quote!{
            #struct_name::#identifier(#identifier_lowercase) => #identifier_lowercase.accept(visitor)
        }
    }).collect();
    let variants: Vec<TokenStream> = variants.iter().map(|variant| {
        let identifier = Ident::new(variant.name, Span::call_site());
        if struct_has_lifetime(&variant.fields) {
            quote!{#identifier(#identifier<'a>)}
        } else {
            quote!{#identifier(#identifier)}
        }
    }).collect();
    let variants = quote!(#(#variants,)*);
    if has_lifetime {
        quote!{
            enum #struct_name<'a> {
                #variants
            }
            impl<'a> #struct_name<'a> {
                pub fn accept<R>(&self, visitor: &'a mut dyn Visitor<R>) -> R {
                    match self {
                        #(#variant_methods),*
                    }
                }
            }
        }
    } else {
        quote!{
            enum #struct_name {
                #variants
            }
            impl #struct_name {
                pub fn accept<'a, R>(&self, visitor: &'a mut dyn Visitor<R>) -> R {
                    match self {
                        #(#variant_methods),*
                    }
                }
            }
        }
    }
}

fn field_has_lifetime(field: &FieldInfo) -> bool {
    HAS_LIFETIME_OBJECTS.contains(&field.field_type)
}

fn struct_has_lifetime(fields: &Vec<FieldInfo>) -> bool {
    fields.iter().any(|field| {
        field_has_lifetime(field)
    })
}

fn enum_has_lifetime(variants: &Vec<StructInfo>) -> bool {
    variants.iter().any(|variant| {
        struct_has_lifetime(&variant.fields)
    })
}

fn define_variant(object_name: &str, variant_name: &str, fields: &Vec<FieldInfo>) -> TokenStream {
    let variant_identifier = Ident::new(variant_name, Span::call_site());
    let field_exprs: Vec<TokenStream> = fields.iter().map(|field| {
        let left = Ident::new(field.field_name, Span::call_site());
        let right = {
            let identifier = Ident::new(field.field_type, Span::call_site());
            let identifier_expr = if field_has_lifetime(field) {
                quote!{#identifier<'a>}
            } else {
                quote!{#identifier}
            };
            if object_name == field.field_type {
                quote!{Box<#identifier_expr>}
            } else {
                quote!{#identifier_expr}
            }
        };
        quote!(#left: #right)
    }).collect();
    let func_field_exprs: Vec<TokenStream> = fields.iter().map(|field| {
        let left = Ident::new(field.field_name, Span::call_site());
        let right = {
            let identifier = Ident::new(field.field_type, Span::call_site());
            if field_has_lifetime(field) {
                quote!{#identifier<'a>}
            } else {
                quote!{#identifier}
            }
        };
        quote!(#left: #right)
    }).collect();
    let struct_fields: Vec<TokenStream> = fields.iter().map(|field| {
        let field_name = syn::Ident::new(field.field_name, Span::call_site());
        if object_name == field.field_type {
            quote!{#field_name: Box::new(#field_name)}
        } else {
            quote!{#field_name}
        }
    }).collect();
    let method_identifier = Ident::new(&format!("visit_{}_{}", variant_name.to_lowercase(), object_name.to_lowercase()), Span::call_site());
    if struct_has_lifetime(fields) {
        quote!{
            pub struct #variant_identifier<'a> {
                #(pub #field_exprs),*
            }
            impl<'a> #variant_identifier<'a> {
                pub fn new(#(#func_field_exprs,)*) -> Self {
                    #variant_identifier { #(#struct_fields,)* }
                }

                pub fn accept<R>(&self, visitor: &'a mut dyn Visitor<R>) -> R {
                    visitor.#method_identifier(&self)
                }
            }
        }
    } else {
        quote!{
            pub struct #variant_identifier {
                #(pub #field_exprs),*
            }
            impl #variant_identifier {
                pub fn new(#(#func_field_exprs,)*) -> Self {
                    #variant_identifier { #(#struct_fields,)* }
                }

                pub fn accept<'a, R>(&self, visitor: &'a mut dyn Visitor<R>) -> R {
                    visitor.#method_identifier(&self)
                }
            }
        }
    }
}

fn define_visitor_trait(object_name: &str, variant_names: &Vec<&str>) -> TokenStream {
    let methods = variant_names.iter().map(|variant_name| {
        let variant_identifier = Ident::new(variant_name, Span::call_site());
        let variant_identifier_lowercase = Ident::new(&variant_name.to_lowercase(), Span::call_site());
        let method_name = Ident::new(&format!("visit_{}_{}", &variant_name.to_lowercase(), &object_name.to_lowercase()), Span::call_site());
        quote!(fn #method_name(&mut self, #variant_identifier_lowercase: &#variant_identifier) -> T)
    }).collect::<Vec<TokenStream>>();
    quote!{
        pub trait Visitor<T> {
            #(#methods;)*
        }
    }
}

fn define_use_statements<'a>(dependencies: &'a[&'a str]) -> TokenStream {
    let dependencies: Vec<TokenStream> = dependencies.iter().map(|dependency| {
        let tokens: Vec<TokenStream> = dependency.split("::").enumerate().map(|(i, token_str)| {
            let identifier = Ident::new(token_str, Span::call_site());
            if i == dependencies.len() {
                if let Some(name) = RENAME_MAP.get(&token_str) {
                    let new_identifier = Ident::new(name, Span::call_site());
                    return quote!(#identifier as #new_identifier);
                }
            }
            quote!(#identifier)
        }).collect();
        quote!(use #(#tokens)::*)
    }).collect();
    quote!(#(#dependencies;)*)
}

fn define_type(output_dir: PathBuf, file_info: FileInfo) -> Result<(), std::io::Error> {
    let mut variants: Vec<StructInfo> = Vec::new();
    for variant_info_str in file_info.variant_info_list {
        if let Some(struct_info) = parse_struct_info(&variant_info_str) {
            variants.push(struct_info);
        }
    }
    let expr_names: Vec<&str> = variants.iter().map(|variant| {
        variant.name
    }).collect();
    let expr_enum = define_base_object(file_info.base_object_name, &variants);
    let expr_variants: Vec<TokenStream> = variants.iter().map(|variant| {
        define_variant(file_info.base_object_name, variant.name, &variant.fields)
    }).collect();
    let visitor_trait = define_visitor_trait(file_info.base_object_name, &expr_names);
    let use_statements = define_use_statements(file_info.dependencies);
    let tokens = quote!{
        #use_statements
        pub #expr_enum
        #visitor_trait
        #(#expr_variants)*
    };
    let syntax_tree = parse_file(&tokens.to_string()).unwrap();
    let result = unparse(&syntax_tree);
    let mut file_path = output_dir;
    file_path.push(file_info.base_object_name.to_lowercase() + ".rs");
    std::fs::write(file_path.to_str().unwrap(), result)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut output_path: Option<String> = None;
    for i in 0..args.len() {
        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            output_path = Some(args[i + 1].to_string());
        }
    }
    let output_path = match output_path {
        Some(output_path) => output_path,
        _ => {
            std::io::stdout()
                .lock()
                .write_all(b"Usage: generate_ast --output <output directory>").unwrap();
            std::process::exit(64);
        },
    };
    let mut output_dir = PathBuf::new();
    output_dir.push(output_path);
    let file_info_list = vec![
        FileInfo {
            dependencies: &["crate::token::Token", "crate::token::Literal"],
            base_object_name: "Expr",
            variant_info_list: vec![
                "Binary   : Expr left, Token operator, Expr right",
                "Grouping : Expr expression",
                "Literal  : LoxLiteral value",
                "Unary    : Token operator, Expr right",
            ],
        },
    ];
    for file_info in file_info_list {
        define_type(output_dir.clone(), file_info).unwrap();
    }
}
