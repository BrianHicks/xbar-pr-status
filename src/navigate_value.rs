use anyhow::anyhow;
use anyhow::Result;
use serde_json::Value;

pub trait NavigateValue {
    fn get_str(&self, pointer: &str) -> Result<&str>;

    fn get_bool(&self, pointer: &str) -> Result<bool>;

    fn get_nullable_u64(&self, pointer: &str) -> Result<Option<u64>>;

    fn get_u64(&self, pointer: &str) -> Result<u64>;

    fn get_array(&self, pointer: &str) -> Result<&Vec<Self>>
    where
        Self: Sized;
}

impl NavigateValue for Value {
    fn get_str(&self, pointer: &str) -> Result<&str> {
        self.pointer(pointer)
            .ok_or_else(|| anyhow!("could not get {}", pointer))?
            .as_str()
            .ok_or_else(|| anyhow!("{} was not a string", pointer))
    }

    fn get_bool(&self, pointer: &str) -> Result<bool> {
        self.pointer(pointer)
            .ok_or_else(|| anyhow!("could not get {}", pointer))?
            .as_bool()
            .ok_or_else(|| anyhow!("{} was not a bool", pointer))
    }

    fn get_nullable_u64(&self, pointer: &str) -> Result<Option<u64>> {
        if let Some(value) = self.pointer(pointer) {
            if let Some(num) = value.as_u64() {
                return Ok(Some(num));
            } else {
                anyhow::bail!("{} was not an integer", pointer);
            }
        } else {
            Ok(None)
        }
    }

    fn get_u64(&self, pointer: &str) -> Result<u64> {
        self.pointer(pointer)
            .ok_or_else(|| anyhow!("could not get {}", pointer))?
            .as_u64()
            .ok_or_else(|| anyhow!("{} was not an integer", pointer))
    }

    fn get_array(&self, pointer: &str) -> Result<&Vec<Self>> {
        self.pointer(pointer)
            .ok_or_else(|| anyhow!("could not get {}", pointer))?
            .as_array()
            .ok_or_else(|| anyhow!("{} was not an array", pointer))
    }
}
