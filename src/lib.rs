mod error;
mod parser;
mod perfdata;
mod thresholds;

pub use perfdata::Perfdata;

type Value = f64;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
