use std::fmt;

#[derive(Default)]
pub struct Error {
    list: Vec<String>,
    has_error_flag: bool, // Renamed to avoid confusion with a potential method name
}

impl Error {
    pub fn new() -> Self {
        // Using Default::default() is more explicit here if you want to keep the derive.
        // Self { list: Vec::new(), has_error_flag: false } is also perfectly fine.
        Default::default()
    }

    /// Records an error.
    pub fn error(&mut self, line: usize, message: &str) {
        let error_message = format!("[Line {}] Error: {}", line, message);
        self.list.push(error_message);
        self.has_error_flag = true;
    }

    /// Checks if any error has been recorded.
    /// This is the method `main.rs` expects.
    pub fn has_error_occurred(&self) -> bool {
        self.has_error_flag
    }

    /// Returns a list of all recorded error messages.
    pub fn get_errors(&self) -> &Vec<String> {
        &self.list
    }

    /// Clears all recorded errors and resets the error flag.
    #[allow(dead_code)] // Potentially useful, allow if not used immediately
    pub fn clear_errors(&mut self) {
        self.list.clear();
        self.has_error_flag = false;
    }
}

// Implement Display to allow the Error struct to be easily printed.
// This will be used by `eprintln!("{}", error_reporter.borrow().to_string());` in main.rs.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.has_error_flag {
            for error_message in &self.list {
                writeln!(f, "{}", error_message)?; // Write each error on a new line
            }
            Ok(())
        } else {
            write!(f, "No errors reported.")
        }
    }
}