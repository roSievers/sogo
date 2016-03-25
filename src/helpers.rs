
pub struct EqualityVerifier {
    value : Option<i8>
}

impl EqualityVerifier {
    pub fn new() -> EqualityVerifier {
        EqualityVerifier { value : None }
    }
    pub fn update(&mut self, new : i8) {
        match self.value {
            None => self.value = Some(new),
            Some(current) => if current != new {
                panic!("The EqualityVerifier was holding {} and is updated with {}.", current, new)
            },
        }
    }
    pub fn unwrap(&self) -> i8 {
        match self.value {
            None => panic!("The EqualityVerifier never got a single value."),
            Some(current) => current,
        }
    }
}
