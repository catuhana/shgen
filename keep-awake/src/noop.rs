pub struct KeepAwake;

impl KeepAwake {
    pub fn new(_reason: &str) -> Self {
        Self
    }

    pub fn prevent_sleep(&mut self) {}
    pub fn allow_sleep(&mut self) {}
}
