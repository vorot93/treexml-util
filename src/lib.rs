#![cfg_attr(feature="cargo-clippy", allow(needless_pass_by_value))]

#[macro_use]
extern crate error_chain;
extern crate treexml;

use std::str::FromStr;

pub fn parse_node(s: &str) -> treexml::Result<Option<treexml::Element>> {
    let doc = treexml::Document::parse(s.as_bytes())?;

    Ok(doc.root)
}

pub fn trimmed_optional(e: &Option<String>) -> Option<String> {
    e.clone().map(|v| v.trim().into())
}

pub fn find_value<T>(name: &str, root: &treexml::Element) -> treexml::Result<Option<T>>
where
    T: std::str::FromStr,
{
    root.find_value(name).or_else(|e| match *e.kind() {
        treexml::ErrorKind::ElementNotFound(_) => Ok(None),
        _ => Err(e),
    })
}

fn deserialize_node<T>(out: &mut T, node: &treexml::Element) -> treexml::Result<bool>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match node.text {
        None => Ok(false),
        Some(ref text) => {
            std::mem::swap(
                out,
                &mut match T::from_str(text) {
                    Ok(v) => v,
                    Err(e) => {
                        bail!(treexml::ErrorKind::ValueFromStr(e.to_string()));
                    }
                },
            );
            Ok(true)
        }
    }
}


fn deserialize_node_bool(out: &mut bool, node: &treexml::Element) -> treexml::Result<bool> {
    match node.text {
        None => {
            std::mem::swap(out, &mut true);
            Ok(true)
        }
        Some(ref text) => {
            std::mem::swap(
                out,
                &mut match bool::from_str(text) {
                    Ok(v) => v,
                    Err(e) => {
                        bail!(treexml::ErrorKind::ValueFromStr(e.to_string()));
                    }
                },
            );
            Ok(true)
        }
    }
}

pub trait Unmarshaller {
    fn unmarshal(&mut self, &treexml::Element) -> treexml::Result<bool>;
}

impl Unmarshaller for bool {
    fn unmarshal(&mut self, node: &treexml::Element) -> treexml::Result<bool> {
        deserialize_node_bool(self, node)
    }
}

impl Unmarshaller for i64 {
    fn unmarshal(&mut self, node: &treexml::Element) -> treexml::Result<bool> {
        deserialize_node(self, node)
    }
}

impl Unmarshaller for f64 {
    fn unmarshal(&mut self, node: &treexml::Element) -> treexml::Result<bool> {
        deserialize_node(self, node)
    }
}

impl Unmarshaller for String {
    fn unmarshal(&mut self, node: &treexml::Element) -> treexml::Result<bool> {
        deserialize_node(self, node)
    }
}

/// Creates an XML element that contains child elements
pub fn make_tree_element(name: &str, v: Vec<treexml::Element>) -> treexml::Element {
    treexml::Element {
        name: name.into(),
        children: v,
        ..Default::default()
    }
}

/// Creates an XML element with text contents
pub fn make_text_element<T>(name: &str, v: T) -> treexml::Element
where
    T: std::fmt::Display,
{
    treexml::Element {
        name: name.into(),
        text: Some(v.to_string()),
        ..Default::default()
    }
}

/// Creates an XML element with cdata contents
pub fn make_cdata_element<T>(name: &str, v: T) -> treexml::Element
where
    T: std::fmt::Display,
{
    treexml::Element {
        name: name.into(),
        cdata: Some(v.to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_deserialize() {
        let fixture = parse_node("<data>5</data>").unwrap().unwrap();
        let expectation = 5;

        let mut result = i64::default();
        result.unmarshal(&fixture).unwrap();

        assert_eq!(expectation, result);
    }

    #[test]
    fn test_deserialize_bool() {
        let fixture = parse_node("<do_want/>").unwrap().unwrap();
        let expectation = true;

        let mut result = bool::default();
        result.unmarshal(&fixture).unwrap();

        assert_eq!(expectation, result);
    }
}