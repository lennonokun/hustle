#![feature(proc_macro_diagnostic)]
use proc_macro;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, Ident, Type, Attribute};

fn is_color(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path.is_ident("color"))
}

#[proc_macro_derive(Loadable, attributes(color))] 
pub fn loadable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let out = loadable_impl(TokenStream::from(input));
  proc_macro::TokenStream::from(out)
}

fn loadable_impl(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse2(input).unwrap();
  let name = &ast.ident;
  // let generics = &ast.generics;

  if let syn::Data::Struct(DataStruct {ref fields, ..}) = ast.data {
    // let infos: Vec<FieldInfo> = fields.iter().cloned().map(FieldInfo::from).collect();
    let (colors, non_colors): (Vec<Field>, Vec<Field>) = fields.iter().cloned().partition(is_color);
    let c_idents: Vec<Ident> = colors.iter().cloned()
      .filter_map(|f| f.ident).collect();
    let c_tys: Vec<Type> = colors.iter().cloned()
      .map(|f| f.ty).collect();
    let n_idents: Vec<Ident> = non_colors.iter().cloned()
      .filter_map(|f| f.ident).collect();
    let n_tys: Vec<Type> = non_colors.iter().cloned()
      .map(|f| f.ty).collect();
    let gen = quote! {
      #[derive(Deserialize)]
      struct Loader {
        #(#c_idents: Option<String>,)*
        #(#n_idents: Option<#n_tys>,)*
      }

      impl Loader {
        fn load(p: &Path) -> io::Result<Self> {
          let mut f = File::open(p)?;
          let mut s = String::new();
          f.read_to_string(&mut s);
          // TODO bad
          let out: Self = toml::from_str(&s).unwrap();
          Ok(out)
        }
      }

      impl Loadable for #name {
        fn load(ps: Vec<&Path>) -> Option<Self> {
          #(let mut #c_idents: Option<Color> = None;)*
          #(let mut #n_idents: Option<#n_tys> = None;)*
          for p in ps {
            let loader = Loader::load(p).ok()?;
            // right now improper colors are ignored
            #(#c_idents = #c_idents.or(loader.#c_idents
                                       .map(|s| Color::parse(&s)).flatten());)*
            #(#n_idents = #n_idents.or(loader.#n_idents);)*
          }

          Some(#name {
            #(#c_idents: #c_idents?,)*
            #(#n_idents: #n_idents?,)*
          })
        }
      }
    };
    gen.into()
  } else {
    name.span().unstable()
      .error("jaboba")
      .emit();
    let gen = quote! {};
    gen.into()
  }
}
