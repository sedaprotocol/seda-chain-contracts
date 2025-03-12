pub mod execute;
pub mod query;

#[path = ""]
#[cfg(test)]
mod test {
    use super::*;
    mod execute_tests;
    mod query_tests;
}
