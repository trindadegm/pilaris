pub struct Arguments {
    pub source: String,
}

impl Arguments {
    /// # Exits
    /// This function will stop the program execution if the arguments cannot be parsed.
    pub fn from_args() -> Self {
        let source = std::env::args().nth(1).unwrap();
        Arguments { source }
    }
}
