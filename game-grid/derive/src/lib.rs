use derive_core::derive_grid_cell;
use proc_macro::TokenStream;

#[proc_macro_derive(GridCell)]
pub fn grid_cell(input: TokenStream) -> TokenStream {
    derive_grid_cell(input.into()).into()
}
