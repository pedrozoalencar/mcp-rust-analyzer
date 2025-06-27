// Test file for MCP Rust Analyzer tests

pub struct TestStruct {
    pub field: String,
}

impl TestStruct {
    pub fn new(field: String) -> Self {
        Self { field }
    }
    
    pub fn get_field(&self) -> &str {
        &self.field
    }
}

pub trait TestTrait {
    fn test_method(&self);
}

impl TestTrait for TestStruct {
    fn test_method(&self) {
        println!("Test method called");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_creation() {
        let test = TestStruct::new("test".to_string());
        assert_eq!(test.get_field(), "test");
    }
}