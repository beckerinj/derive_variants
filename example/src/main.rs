use derive_variants::EnumVariants;

#[derive(Debug, EnumVariants)]
#[variant_derive(Debug)]
enum Things {
  Thing1(Thing1),
  Thing2(Thing2),
  Thing3(Thing3),
}

#[derive(Debug)]
struct Thing1 {}

#[derive(Debug)]
struct Thing2 {}

#[derive(Debug)]
struct Thing3 {}

fn main() {
  let thing = Things::Thing1(Thing1 {});

  let (variant, data) = derive_variants::extract::<_, _, Thing1>(thing).unwrap();

  println!("{variant:?}");
  println!("{data:?}");
}
