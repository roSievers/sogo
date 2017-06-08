
#[derive(Copy, Clone, Debug)]
pub enum EqualityVerifier {
    NoValue,
    Value(u8),
    Conflict,
}

impl EqualityVerifier {
    pub fn update(self, new : u8) -> Self {
        match self {
            EqualityVerifier::NoValue => EqualityVerifier::Value(new),
            EqualityVerifier::Value(current) => {
                if current == new {
                    self
                } else {
                    EqualityVerifier::Conflict
                }
            },
            EqualityVerifier::Conflict => self
        }
    }
    pub fn unwrap(self) -> u8 {
        match self {
            EqualityVerifier::NoValue => panic!("The EqualityVerifier never got a single value."),
            EqualityVerifier::Value(value) => value,
            EqualityVerifier::Conflict => panic!("The EqualityVerifier never got conflicting values."),
        }
    }
}
