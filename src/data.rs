#[derive(Debug, Eq, PartialEq)]
pub struct V4(pub u32, pub u8);

#[derive(Debug, Eq, PartialEq)]
pub struct V6(pub u128, pub u8);

impl ToString for V4 {
    fn to_string(&self) -> String {
        // Mask end zeros
        if self.1 == 0 {
            return "0.0.0.0/0".to_owned();
        }

        let masked = self.0 & !((1u32 << (32 - self.1)) - 1);
        format!("{}/{}", masked.to_be_bytes().map(|e| e.to_string()).join("."), self.1)
    }
}

impl ToString for V6 {
    fn to_string(&self) -> String {
        todo!()
    }
}