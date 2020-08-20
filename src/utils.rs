#[cfg(test)]
pub fn remove_whitespace(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}
