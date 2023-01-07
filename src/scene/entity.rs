use std::collections::HashMap;

#[derive(Clone)]
pub struct Entity {
    pub (crate) properties: HashMap<String, String>,
}

impl Entity {

    pub fn new(properties_string: &String) -> Self {
        let mut pos: usize = 0;
        let mut instance: Entity = Entity {
            properties: HashMap::new(),
        };
        loop {
            if pos > properties_string.len() {
                break;
            }
            if let Some(next_pos) = properties_string[pos..].find('"') {
                pos += next_pos;
            } else {
                break;
            }
            pos += 1;
            let mut end: usize = properties_string[pos..].find('"').unwrap();
            let name: String = properties_string[pos..(pos + end)].to_string();
            pos += end + 1;
            pos += properties_string[pos..].find('"').unwrap();
            pos += 1;
            end = properties_string[pos..].find('"').unwrap();
            let value: String = properties_string[pos..(pos + end)].to_string();
            pos += end + 1;
            instance.properties.insert(name, value);
        }
        return instance;
    }

    pub fn find_property(&self, name: &String) -> Option<&String> {
        return self.properties.get(name);
    }

}
