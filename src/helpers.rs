
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

// This function finds the lowest index i s.th. vector[i] > value.
pub fn upper_bound_index(vector : &Vec<f32>, value : f32) -> usize {
    let mut lower = 0;
    let mut upper = vector.len()-1;
    while lower != upper {
        let next = (lower + upper) / 2;
        if vector[next] < value {
            lower = next+1;
        } else {
            upper = next;
        }
    }
    return lower;
}

#[test]
fn test_upper_bound_index() {
    let vector = vec![0.3, 0.9, 1.3, 2.7, 3.8, 6.1];
    let index = upper_bound_index(&vector, 2.0);
    assert_eq!(index, 3);
}
