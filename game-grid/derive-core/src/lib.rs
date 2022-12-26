use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemEnum};

pub fn derive_grid_cell(input: TokenStream) -> TokenStream {
    let input_enum = match parse2::<ItemEnum>(input) {
        Ok(syntax_tree) => syntax_tree,
        Err(error) => return error.to_compile_error(),
    };

    let enum_identifier = input_enum.ident;

    let implementation = quote!(
        impl GridCell for #enum_identifier {
        }
    );

    implementation
}

#[cfg(test)]
mod unit_tests {
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn test_derive_grid_cell() {
        // Empty enum.
        let stream = quote!(
            enum A {}
        );

        let output_stream = derive_grid_cell(stream);
        assert!(!output_stream.is_empty());

        let parsed = parse2::<ItemImpl>(output_stream);
        assert!(parsed.is_ok());
    }
}
