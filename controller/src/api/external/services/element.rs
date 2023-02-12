use crate::api::types::element::Element;

pub fn elements_set_right_name(elements: Vec<Element>) -> Vec<Element> {
    let mut result: Vec<Element> = Vec::new();
    for element in elements {
        result.push(element_set_right_name(element));
    }
    result
}

pub fn element_set_right_name(mut element: Element) -> Element {
    let mut split: Vec<&str> = element.name.split('/').collect();
    if let Some(v) = split.pop() {
        let _ = &element.set_name(v.to_string());
    }
    element
}
