use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data::Struct, DeriveInput};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

#[proc_macro_derive(UniformBuffer)]
pub fn uniform_buffer_derive(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let out = match input.data {
		Struct(s) => {
			let fields = s.fields.into_iter().map(|f| {
				let field_name = f.ident.expect("All struct fields need an identifier");
				let field_type = f.ty;

				quote!(format!("{}: {}", stringify!(#field_name), <#field_type as WgslType>::name()),)
			});

			quote! {
				impl Bufferable for #name {}
				impl UniformBuffer for #name {
					fn get_source_code(&self, group: u32, binding: u32, name: &str) -> String {
						format!(
							r#"
								struct {struct_name} {{
									{fields}
								}};
								@group({group}) @binding({binding}) var<uniform> {name}: {struct_name};
							"#, struct_name=stringify!(#name), fields=vec![#(#fields)*].join(",")
						)
					}

					fn get_size(&self) -> u64 {
						mem::size_of::<Self>() as u64
					}

					fn get_data(&self) -> Vec<u8> {
						bytemuck::bytes_of(self).to_owned()
					}
				}
				impl WgslType for #name {
					fn name() -> String {
						stringify!(#name).to_string()
					}
				}
			}
		}
		_ => panic!("Must be a struct"),
	};

	// println!("\n{}", out);

	out.into()
}
