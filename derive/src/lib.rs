use std::collections::HashMap;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Variant};

#[proc_macro_derive(EnumVariants, attributes(variant_derive, variant_attr))]
pub fn derive_partial(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let DeriveInput {
    attrs,
    vis,
    ident,
    data,
    ..
  } = syn::parse(input).unwrap();

  let variant_ident = Ident::new(&format!("{}Variant", ident), Span::call_site());

  let variant_derives = attrs
    .iter()
    .find(|attr| attr.path().is_ident("variant_derive"));

  let variant_derives = if let Some(variant_derives) = variant_derives {
    variant_derives
      .parse_args()
      .expect("failed to parse partial_derive")
  } else {
    proc_macro2::TokenStream::new()
  };

  let variant_attrs = attrs
    .iter()
    .filter(|attr| attr.path().is_ident("variant_attr"))
    .map(|attr| {
      attr
        .parse_args::<proc_macro2::TokenStream>()
        .expect("failed to parse variant_attr args")
    });

  let variants = match data {
    Data::Enum(e) => e.variants,
    _ => panic!(""),
  };

  let variant_variants = variants.iter().map(|v| v.ident.clone());

  let variant_from_full = variants.iter().map(
    |Variant {
       ident: v_ident,
       fields,
       ..
     }| {
      match &fields {
        Fields::Named(_) => quote!(#ident::#v_ident { .. } => #variant_ident::#v_ident),
        Fields::Unnamed(_) => {
          let underscores = fields.iter().map(|_| quote!(_));
          quote!(#ident::#v_ident(#(#underscores),*) => #variant_ident::#v_ident)
        }
        Fields::Unit => quote!(#ident::#v_ident => #variant_ident::#v_ident),
      }
    },
  );

  let derive_extract_data = variants.iter().filter_map(
    |Variant {
       ident: v_ident,
       fields,
       ..
     }| {
      match &fields {
        Fields::Unnamed(_) => {
          let data_tys = fields.iter().map(|syn::Field { ty, .. }| ty);
          let data = quote!((#(#data_tys),*));
          let idents = fields
            .iter()
            .enumerate()
            .map(|(i, _)| Ident::new(&format!("f{i}"), Span::call_site()));
          let idents = quote!((#(#idents),*));
          Some((
            // use as hashmap key
            data.to_string(),
            quote! {
              // impl derive_variants::ExtractData<#variant_ident, #data> for #ident {
              //   fn extract_data(&self, variant: &#variant_ident) -> #data {
              //     match variant {
              #variant_ident::#v_ident => match self {
                #ident::#v_ident(#idents) => Ok(#idents),
                _ => Err(derive_variants::Error::VariantMismatch)
              }
              //     }
              //   }
              // }
            },
            data,
          ))
        }
        Fields::Unit | Fields::Named(_) => None,
      }
    },
  );

  // group the handlers by data ty
  let mut data_handlers =
    HashMap::<String, (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>)>::default();
  for (key, handler, data) in derive_extract_data {
    let entry = data_handlers.entry(key).or_default();
    entry.0 = data;
    entry.1.push(handler);
  }

  let data_impls = data_handlers.values().map(|(data, handlers)| {
    quote! {
      impl derive_variants::ExtractData<#variant_ident, #data> for #ident {
        fn extract_data(self, variant: &#variant_ident) -> Result<#data, derive_variants::Error> {
          match variant {
            #(#handlers),*
            _ => Err(derive_variants::Error::WrongVariantForData)
          }
        }
      }
    }
  });

  quote! {
    #[derive(#variant_derives)]
    #(#variant_attrs)*
    #vis enum #variant_ident {
      #(#variant_variants),*
    }

    impl derive_variants::ExtractVariant<#variant_ident> for #ident {
      fn extract_variant(&self) -> #variant_ident {
        match self {
          #(#variant_from_full),*
        }
      }
    }

    #(#data_impls)*
  }
  .into()
}
