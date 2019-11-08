//! Tweaked version of [vulkano_shader!](https://docs.rs/crate/vulkano-shaders) macro.
//! 
//! The changes include tweaks to the generated code to use gaclen::vulkano to avoid the necessity of including vulkano in gaclen-dependent projects.

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate proc_macro;

use std::env;
use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::path::Path;

use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, LitStr, LitBool};

mod codegen;
mod descriptor_sets;
mod entry_point;
mod enums;
mod parse;
mod spec_consts;
mod structs;
mod spirv_search;

use crate::codegen::ShaderKind;

enum SourceKind {
    Src(String),
    Path(String),
}

struct MacroInput {
    shader_kind: ShaderKind,
    source_kind: SourceKind,
    include_directories: Vec<String>,
    macro_defines: Vec<(String, String)>,
    dump: bool,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut dump = None;
        let mut shader_kind = None;
        let mut source_kind = None;
        let mut include_directories = Vec::new();
        let mut macro_defines = Vec::new();

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match name.to_string().as_ref() {
                "ty" => {
                    if shader_kind.is_some() {
                        panic!("Only one `ty` can be defined")
                    }

                    let ty: LitStr = input.parse()?;
                    let ty = match ty.value().as_ref() {
                        "vertex" => ShaderKind::Vertex,
                        "fragment" => ShaderKind::Fragment,
                        "geometry" => ShaderKind::Geometry,
                        "tess_ctrl" => ShaderKind::TessControl,
                        "tess_eval" => ShaderKind::TessEvaluation,
                        "compute" => ShaderKind::Compute,
                        _ => panic!("Unexpected shader type, valid values: vertex, fragment, geometry, tess_ctrl, tess_eval, compute")
                    };
                    shader_kind = Some(ty);
                }
                "src" => {
                    if source_kind.is_some() {
                        panic!("Only one `src` or `path` can be defined")
                    }

                    let src: LitStr = input.parse()?;
                    source_kind = Some(SourceKind::Src(src.value()));
                }
                "path" => {
                    if source_kind.is_some() {
                        panic!("Only one `src` or `path` can be defined")
                    }

                    let path: LitStr = input.parse()?;
                    source_kind = Some(SourceKind::Path(path.value()));
                }
                "define" => {
                    let array_input;
                    bracketed!(array_input in input);

                    while !array_input.is_empty() {
                        let tuple_input;
                        parenthesized!(tuple_input in array_input);

                        let name: LitStr = tuple_input.parse()?;
                        tuple_input.parse::<Token![,]>()?;
                        let value: LitStr = tuple_input.parse()?;
                        macro_defines.push((name.value(), value.value()));

                        if !array_input.is_empty() {
                            array_input.parse::<Token![,]>()?;
                        }
                    }
                }
                "include" => {
                    let in_brackets;
                    bracketed!(in_brackets in input);

                    while !in_brackets.is_empty() {
                        let path: LitStr = in_brackets.parse()?;

                        include_directories.push(path.value());

                        if !in_brackets.is_empty() {
                            in_brackets.parse::<Token![,]>()?;
                        }
                    }
                }
                "dump" => {
                    if dump.is_some() {
                        panic!("Only one `dump` can be defined")
                    }
                    let dump_lit: LitBool = input.parse()?;
                    dump = Some(dump_lit.value);
                }
                name => panic!(format!("Unknown field name: {}", name))
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let shader_kind = match shader_kind {
            Some(shader_kind) => shader_kind,
            None => panic!("Please provide a shader type e.g. `ty: \"vertex\"`")
        };

        let source_kind = match source_kind {
            Some(source_kind) => source_kind,
            None => panic!("Please provide a source e.g. `path: \"foo.glsl\"` or `src: \"glsl source code here ...\"`")
        };

        let dump = dump.unwrap_or(false);

        Ok(MacroInput { shader_kind, source_kind, include_directories, dump, macro_defines })
    }
}

pub(self) fn read_file_to_string(full_path: &Path) -> IoResult<String> {
    let mut buf = String::new();
    File::open(full_path)
        .and_then(|mut file| file.read_to_string(&mut buf))?;
    Ok(buf)
}

#[proc_macro]
pub fn shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
    let root_path = Path::new(&root);

    let (path, source_code) = match input.source_kind {
        SourceKind::Src(source) => (None, source),
        SourceKind::Path(path) => (Some(path.clone()), {
            let full_path = root_path.join(&path);

            if full_path.is_file() {
                read_file_to_string(&full_path)
                    .expect(&format!("Error reading source from {:?}", path))
            } else {
                panic!("File {:?} was not found ; note that the path must be relative to your Cargo.toml", path);
            }
        })
    };

    let include_paths = input.include_directories.iter().map(|include_directory| {
        let include_path = Path::new(include_directory);
        let mut full_include_path = root_path.to_owned();
        full_include_path.push(include_path);
        full_include_path
    }).collect::<Vec<_>>();

    let content = codegen::compile(path, &root_path, &source_code, input.shader_kind, &include_paths, &input.macro_defines).unwrap();
    codegen::reflect("Shader", content.as_binary(), input.dump).unwrap().into()
}