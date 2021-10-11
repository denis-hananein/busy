use serde;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
  id_: u128,
  name_: String,
}

impl Category {
  pub fn new(id: u128, name: &str) -> Self {
    Self {
      id_: id,
      name_: name.to_owned(),
    }
  }

  pub fn id(&self) -> u128 {
    return self.id_;
  }

  pub fn name(&self) -> &str {
    self.name_.as_str()
  }
}
