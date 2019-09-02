pub struct Commit {
  pub ty: String,
  pub scope: Option<String>,
  pub message: String,
}

impl Commit {
  pub fn build_message(&self) -> String {
    // This with_capacity is likely excessive
    const PARENS: usize = 2;
    const COLON: usize = 1;
    const SPACE: usize = 1;
    let len = self.ty.len()
      + self.message.len()
      + self.scope.as_ref().map(|s| s.len() + PARENS).unwrap_or(0)
      + COLON
      + SPACE;

    let mut s = String::with_capacity(len);

    s.push_str(&self.ty);

    if let Some(ref scope) = self.scope {
      s.push('(');
      s.push_str(scope);
      s.push(')');
    }

    s.push(':');
    s.push(' ');
    s.push_str(&self.message);

    s
  }
}
