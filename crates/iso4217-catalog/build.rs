use std::{collections::BTreeMap, env, fs, io, path::PathBuf};

use quick_xml::{Reader, events::Event};
use quote::{format_ident, quote};

#[derive(Clone, Debug)]
struct Entry {
  alpha: String,
  numeric: u16,
  minor_units: Option<u8>,
  name: String,
  status: &'static str,
}

fn main() -> io::Result<()> {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
  let source = manifest_dir.join("reference/iso4217/list-one.xml");

  println!("cargo:rerun-if-changed={}", source.display());

  let (published, entries) = parse_list_one(&source).map_err(|e| {
    io::Error::new(io::ErrorKind::InvalidData, format!("list-one parse error: {e}"))
  })?;

  let generated = render_catalog(&published, &entries);
  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
  fs::write(out_dir.join("iso4217_catalog.rs"), generated)
}

fn parse_list_one(path: &PathBuf) -> Result<(String, BTreeMap<String, Entry>), String> {
  let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
  let mut reader = Reader::from_str(&data);
  reader.config_mut().trim_text(true);

  let mut buf = Vec::new();
  let mut published = String::new();
  let mut current_field = String::new();
  let mut in_entry = false;
  let mut ccy = String::new();
  let mut ccy_name = String::new();
  let mut ccy_nbr = String::new();
  let mut ccy_minor = String::new();
  let mut entries = BTreeMap::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(tag)) => {
        let name = String::from_utf8_lossy(tag.name().as_ref()).to_string();
        if name == "ISO_4217" {
          for attr in tag.attributes().flatten() {
            if attr.key.as_ref() == b"Pblshd" {
              published = String::from_utf8_lossy(attr.value.as_ref()).to_string();
            }
          }
        }
        if name == "CcyNtry" {
          in_entry = true;
          current_field.clear();
          ccy.clear();
          ccy_name.clear();
          ccy_nbr.clear();
          ccy_minor.clear();
        } else if in_entry {
          current_field = name;
        }
      }
      Ok(Event::Text(text)) => {
        if !in_entry {
          buf.clear();
          continue;
        }

        let value = String::from_utf8_lossy(text.as_ref()).to_string();
        match current_field.as_str() {
          "Ccy" => ccy = value,
          "CcyNm" => ccy_name = value,
          "CcyNbr" => ccy_nbr = value,
          "CcyMnrUnts" => ccy_minor = value,
          _ => {}
        }
      }
      Ok(Event::End(tag)) => {
        let name = String::from_utf8_lossy(tag.name().as_ref()).to_string();
        if name == "CcyNtry" {
          in_entry = false;
          if ccy.len() == 3 {
            let numeric = ccy_nbr.parse::<u16>().unwrap_or(0);
            if numeric != 0 {
              let status = classify_status(&ccy);
              entries.entry(ccy.clone()).or_insert(Entry {
                alpha: ccy.clone(),
                numeric,
                minor_units: parse_minor_units(&ccy_minor),
                name: ccy_name.clone(),
                status,
              });
            }
          }
        }
        current_field.clear();
      }
      Ok(Event::Eof) => break,
      Err(err) => return Err(err.to_string()),
      _ => {}
    }
    buf.clear();
  }

  if published.is_empty() {
    return Err("missing Pblshd attribute in list-one.xml".to_owned());
  }

  Ok((published, entries))
}

fn classify_status(alpha: &str) -> &'static str {
  match alpha {
    "XTS" => "Testing",
    "XXX" => "NoCurrency",
    "XAU" | "XAG" | "XPD" | "XPT" => "Metal",
    _ => "Active",
  }
}

fn parse_minor_units(raw: &str) -> Option<u8> {
  if raw == "N.A." { None } else { raw.parse::<u8>().ok() }
}

fn variant_name(alpha: &str) -> String {
  alpha.to_ascii_uppercase()
}

fn render_catalog(published: &str, entries: &BTreeMap<String, Entry>) -> String {
  let version = proc_macro2::Literal::string(published);

  let variants: Vec<_> =
    entries.values().map(|entry| format_ident!("{}", variant_name(&entry.alpha))).collect();
  let alphas: Vec<_> =
    entries.values().map(|entry| proc_macro2::Literal::string(&entry.alpha)).collect();
  let numerics: Vec<_> = entries.values().map(|entry| entry.numeric).collect();
  let digits: Vec<_> = entries
    .values()
    .map(|entry| match entry.minor_units {
      Some(d) => quote! { Some(#d) },
      None => quote! { None },
    })
    .collect();
  let names: Vec<_> =
    entries.values().map(|entry| proc_macro2::Literal::string(&entry.name)).collect();
  let statuses: Vec<_> = entries.values().map(|entry| format_ident!("{}", entry.status)).collect();

  let generated = quote! {
    pub const CATALOG_VERSION: &str = #version;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum CurrencyStatus {
      Active,
      Testing,
      Metal,
      NoCurrency,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CurrencyMeta {
      pub alpha: &'static str,
      pub numeric: u16,
      pub minor_units: Option<u8>,
      pub name: &'static str,
      pub status: CurrencyStatus,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub enum CurrencyCode {
      #(#variants,)*
    }

    impl CurrencyCode {
      pub const fn alpha(self) -> &'static str {
        match self {
          #(Self::#variants => #alphas,)*
        }
      }

      pub const fn num(self) -> u16 {
        match self {
          #(Self::#variants => #numerics,)*
        }
      }

      pub const fn digit(self) -> Option<u8> {
        match self {
          #(Self::#variants => #digits,)*
        }
      }

      pub const fn name(self) -> &'static str {
        match self {
          #(Self::#variants => #names,)*
        }
      }

      pub const fn status(self) -> CurrencyStatus {
        match self {
          #(Self::#variants => CurrencyStatus::#statuses,)*
        }
      }

      pub const fn meta(self) -> CurrencyMeta {
        CurrencyMeta {
          alpha: self.alpha(),
          numeric: self.num(),
          minor_units: self.digit(),
          name: self.name(),
          status: self.status(),
        }
      }
    }

    impl core::str::FromStr for CurrencyCode {
      type Err = ();

      fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
          #(#alphas => Ok(Self::#variants),)*
          _ => Err(()),
        }
      }
    }

    impl core::convert::TryFrom<u16> for CurrencyCode {
      type Error = ();

      fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
          #(#numerics => Ok(Self::#variants),)*
          _ => Err(()),
        }
      }
    }

    impl From<CurrencyCode> for &'static str {
      fn from(value: CurrencyCode) -> Self {
        value.alpha()
      }
    }

    impl From<CurrencyCode> for u16 {
      fn from(value: CurrencyCode) -> Self {
        value.num()
      }
    }

    pub const CURRENCY_CODES: &[CurrencyCode] = &[
      #(CurrencyCode::#variants,)*
    ];
  };

  generated.to_string()
}
