use syn::{parse::{Parse, ParseStream}, Token};

pub fn parse_one_or_more<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut result: Vec<T> = Vec::new();
    result.push(input.parse::<T>()?);
    while let Ok(_comma) = input.parse::<Token![,]>() {
        result.push(input.parse::<T>()?);
    }

    Ok(result)
}