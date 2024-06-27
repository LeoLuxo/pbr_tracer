use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data::Struct, DeriveInput};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[proc_macro_derive(ShaderStruct)]
pub fn shader_struct_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let out = match input.data {
		Struct(s) => {
			let fields = s.fields.into_iter().map(|f| {
				let field_name = f.ident.expect("All struct fields need an identifier");
				let field_type = f.ty;

				quote!(format!("{}: {}", stringify!(#field_name), <#field_type as ShaderType>::type_name()),)
			});

			quote! {
				impl ShaderType for #name {
					fn type_name() -> String {
						stringify!(#name).to_string()
					}

					fn struct_definition() -> Option<String> {
						Some(format!(
							r#"
								struct {struct_name} {{
									{fields}
								}};
							"#, struct_name=stringify!(#name), fields=vec![#(#fields)*].join(",")
						))
					}
				}
			}
		}
		_ => panic!("Must be a struct"),
	};

	// println!("\n{}", out);

	out.into()
}
