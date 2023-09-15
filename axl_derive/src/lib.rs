// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse::Parser, Ident, ItemStruct};
//
// #[proc_macro_derive(TryIntoVariantOptions)]
// pub fn try_into_variant_options_macro_derive(input: TokenStream) -> TokenStream {
// // Construct a representation of Rust code as a syntax tree
// // that we can manipulate
// let ast: syn:: = syn::parse(input).unwrap();
// ast.
// let name = &ast.ident;
// let gen = match ast.body {
//     Body::Enum(ref variants) => impl_enum_iter(name, variants),
//     Body::Struct(_) => quote! {
//         impl EnumIteratorOnlyWorksForEnumsNotStructsSorryNotSorry for #name { }
//     },
// };
//
// let gen = quote! {
//     impl TryInto<RustProjectOptions> for #name {
//         fn try_into(other: #name) -> Result<RustProjectOptions> {
//             match other {
//                 #name::$Variant(v) => Ok(v),
//                 _ => Err("Expected $Name.", ),
//             }
//             match other {
//                 $Name::$Variant { options } => match options {
//                     Some(o) => Ok(o),
//                     None => Err(anyhow!("Expected a options to be Some")),
//                 },
//                 _ => Err(anyhow!("Expected a Rust ProjectType")),
//             }
//         }
//     }
// };
// gen.into()
// }
