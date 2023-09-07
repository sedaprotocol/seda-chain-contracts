pub mod contract;
mod error;
pub mod msg;
pub mod state;

#[path = ""]
#[cfg(test)]
mod tests {
    mod test;
}
