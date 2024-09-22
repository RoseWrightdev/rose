
#[derive(Default)]
pub struct E {
  list: Vec<String>,
  has_error: bool
}

impl E {

  pub fn new() -> Self{
    Self{
      ..Default::default()
    }
  }

  pub fn error(&mut self, line: usize, message: &str) -> () {
    let error = format!("[{}] Error: {}" , line, message);
    self.list.push(error);
    self.has_error = true;
    ()
  }

  pub fn check_errors(&self) {
    if self.has_error {
      for e in &self.list {
        println!("{}\n", e);
      }
    }
  }
}

pub fn check_errors(errors: &E) {
  errors.check_errors();
}
