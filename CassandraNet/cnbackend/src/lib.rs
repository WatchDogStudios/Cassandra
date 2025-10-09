/// Copyright (C) 2024-present WD Studios L.L.C. & CassandraNet Contributors.
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub mod requests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
        println!("cnbackend::add() test passed");
    }
}
