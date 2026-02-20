pub(crate) fn is_valid_identifier(value: &str) -> bool {
  if value.is_empty() || value.len() > 128 {
    return false;
  }

  value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}

pub(crate) fn is_valid_kyc_level(value: &str) -> bool {
  matches!(value, "tier_0" | "tier_1" | "tier_2" | "tier_3" | "tier_4")
}

pub(crate) fn is_valid_locale_tag(value: &str) -> bool {
  let mut parts = value.split('-');
  let Some(language) = parts.next() else {
    return false;
  };

  if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic()) {
    return false;
  }

  for part in parts {
    if !(2..=8).contains(&part.len()) || !part.chars().all(|c| c.is_ascii_alphanumeric()) {
      return false;
    }
  }

  true
}

pub(crate) fn is_valid_user_agent(value: &str) -> bool {
  let value = value.trim();
  if !(3..=512).contains(&value.len()) {
    return false;
  }

  if !value.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
    return false;
  }

  let Some(first_product) = value.split_whitespace().next() else {
    return false;
  };

  let mut parts = first_product.split('/');
  let Some(name) = parts.next() else {
    return false;
  };
  let Some(version) = parts.next() else {
    return false;
  };

  parts.next().is_none() && is_valid_http_token(name) && is_valid_http_token(version)
}

fn is_valid_http_token(value: &str) -> bool {
  !value.is_empty()
    && value.chars().all(|c| {
      c.is_ascii_alphanumeric()
        || matches!(
          c,
          '!' | '#' | '$' | '%' | '&' | '\'' | '*' | '+' | '-' | '.' | '^' | '_' | '`' | '|' | '~'
        )
    })
}
