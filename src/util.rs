pub struct Hex<T: std::fmt::Debug>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Hex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X?}", self.0)
    }
}
