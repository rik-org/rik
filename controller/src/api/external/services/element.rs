use crate::api::types::element::Element;

pub fn elements_set_right_name(elements: Vec<Element>) -> Vec<Element> {
    let mut result: Vec<Element> = Vec::new();
    for mut element in elements.clone() {
        let mut split: Vec<&str> = element.name.split("/").collect();
        match split.pop() {
            Some(val) => {
                &element.set_name(val.to_string());
            }
            _ => {}
        }
        result.push(element.clone());
    }
    return result;
}
