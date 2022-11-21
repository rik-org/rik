use crate::api::types::element::Element;

pub fn elements_set_right_name(elements: Vec<Element>) -> Vec<Element> {
    let mut result: Vec<Element> = Vec::new();
    for mut element in elements {
        let mut split: Vec<&str> = element.name.split('/').collect();
        if let Some(v) = split.pop() {
            let _ = &element.set_name(v.to_string());
        }
        result.push(element.clone());
    }
    result
}
