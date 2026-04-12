use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Apu;

impl Apu {
    pub fn init() -> Self {
        Self
    }
}
