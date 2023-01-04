use std::collections::HashMap;

pub struct Entity {
    pub (crate) properties: HashMap<String, String>,
}

impl Entity {

    pub fn new(properties_string: &String) -> Self {
        let mut pos: usize = 0;
        let instance: Entity = Entity {
            properties: HashMap::new(),
        };
        loop {
            let read_quoted_string = || -> String {
                pos += 1;
                let end: usize = properties_string[pos..].find('"').unwrap();
                let quote: String = properties_string[pos..(end - pos)].to_string();
                pos = end + 1;
                return quote;
            };
            if let Some(next_pos) = properties_string[pos..].find('"') {
                pos = next_pos;
            } else {
                break;
            }
            let name: String = read_quoted_string();
            pos = properties_string[pos..].find('"').unwrap();
            let value: String = read_quoted_string();
            instance.properties.insert(name, value);
        }
        return instance;
    }

    pub fn find_property<'a>(&self, name: &String) -> Option<&'a String> {
        return self.properties.get(name);
    }

}
