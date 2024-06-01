pub use derive_variants_derive::EnumVariants;

/// Extract data in an enum into (variant, inner_data)
pub fn extract<T, Variant, Data>(target: T) -> Result<(Variant, Data), Error>
where
  T: ExtractVariant<Variant> + ExtractData<Variant, Data>,
{
  let variant = target.extract_variant();
  let data = target.extract_data(&variant)?;
  Ok((variant, data))
}

pub trait ExtractData<Variant, Data> {
  fn extract_data(self, variant: &Variant) -> Result<Data, Error>;
}

pub trait ExtractVariant<Variant> {
  fn extract_variant(&self) -> Variant;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("The enum variant passed is not associated with the expected inner data")]
  WrongVariantForData,
  #[error("The enum variant passed is different than the variant of the actual enum")]
  VariantMismatch,
}
