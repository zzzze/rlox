extern crate proc_macro;
use syn::parse_file;
use proc_macro2::{TokenStream, Ident, Span};
use quote::quote;
use prettyplease::unparse;
use std::{io::Write, path::PathBuf, vec};

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

fn define_base_object(base_object_name: &str, variants: &Vec<&str>) -> TokenStream {
    let struct_name = syn::Ident::new(base_object_name, Span::call_site());
    let variants: Vec<Ident> = variants.iter().map(|item| {
        syn::Ident::new(item, Span::call_site())
    }).collect();
    let variants = quote!(#(#variants,)*);
    quote!{
        enum #struct_name {
            #variants
        }
    }
}

fn define_variant(name: &str, fields: &Vec<FieldInfo>) -> TokenStream {
    let struct_name = syn::Ident::new(name, Span::call_site());
    let field_exprs: Vec<TokenStream> = fields.iter().map(|field| {
        let left = syn::Ident::new(field.field_name, Span::call_site());
        let right = syn::Ident::new(field.field_type, Span::call_site());
        quote!(#left: #right)
    }).collect();
    let field_names: Vec<syn::Ident> = fields.iter().map(|field| {
        syn::Ident::new(field.field_name, Span::call_site())
    }).collect();
    quote!{
        struct #struct_name {
            #(#field_exprs),*
        }
        impl #struct_name {
            fn new(#(#field_exprs,)*) -> Self {
                #struct_name { #(#field_names,)* }
            }
        }
    }
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
    let expr_enum = define_base_object(file_info.base_object_name, &expr_names);
    let expr_variants: Vec<TokenStream> = variants.iter().map(|variant| {
        define_variant(&variant.name, &variant.fields)
    }).collect();
    let tokens = quote!{
        use crate::token::{Token, Literal as LoxLiteral};
        #expr_enum
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
