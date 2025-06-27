// Demo file for testing MCP rust-analyzer

fn main() {
    println!("Hello, MCP!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    
    println!("Sum: {}", sum);
    
    let person = Person {
        name: String::from("Alice"),
        age: 30,
    };
    
    person.greet();
}

struct Person {
    name: String,
    age: u32,
}

impl Person {
    fn greet(&self) {
        println!("Hello, my name is {} and I'm {} years old", self.name, self.age);
    }
    
    fn birthday(&mut self) {
        self.age += 1;
        println!("Happy birthday! Now I'm {} years old", self.age);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_person_creation() {
        let person = Person {
            name: String::from("Bob"),
            age: 25,
        };
        
        assert_eq!(person.name, "Bob");
        assert_eq!(person.age, 25);
    }
}