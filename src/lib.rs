#[macro_use]
extern crate failure;
extern crate treexml;

use std::str::FromStr;

pub fn parse_node(s: &str) -> Result<Option<treexml::Element>, treexml::Error> {
    let doc = treexml::Document::parse(s.as_bytes())?;

    Ok(doc.root)
}

pub fn trimmed_optional(e: &Option<String>) -> Option<String> {
    e.clone().map(|v| v.trim().into())
}

pub trait ElementExt {
    fn find_value0<T, PATH>(&self, path: PATH) -> Result<Option<T>, treexml::Error>
    where
        PATH: Into<String>,
        T: std::str::FromStr;

    fn find_value1<T, PATH>(&self, path: PATH) -> Result<T, treexml::Error>
    where
        PATH: Into<String>,
        T: std::str::FromStr;

    fn find_bool<PATH>(&self, path: PATH) -> Result<bool, treexml::Error>
    where
        PATH: Into<String>;

    fn unmarshal_into<T>(&self, out: &mut T) -> Result<bool, treexml::Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;
    fn unmarshal_bool_into(&self, out: &mut bool) -> Result<bool, treexml::Error>;
}

impl ElementExt for treexml::Element {
    fn find_value0<T, PATH>(&self, path: PATH) -> Result<Option<T>, treexml::Error>
    where
        PATH: Into<String>,
        T: std::str::FromStr,
    {
        let path = path.into();
        self.find_value(&path).or_else(|e| match e {
            treexml::Error::ElementNotFound { .. } => Ok(None),
            _ => Err(e),
        })
    }

    fn find_value1<T, PATH>(&self, path: PATH) -> Result<T, treexml::Error>
    where
        PATH: Into<String>,
        T: std::str::FromStr,
    {
        let path = path.into();
        self.find_value0(path.clone()).and_then(|v| {
            v.ok_or_else(|| {
                treexml::Error::ParseError(format_err!("Value not found at path: {}", &path))
            })
        })
    }

    fn find_bool<PATH>(&self, path: PATH) -> Result<bool, treexml::Error>
    where
        PATH: Into<String>,
    {
        let path = path.into();
        match self.find(&path) {
            Ok(ref e) => match e.text {
                None => Ok(true),
                Some(ref text) => match text.as_str() {
                    "true" => Ok(true),
                    "false" => Ok(false),
                    "1" => Ok(true),
                    "0" => Ok(false),
                    other => Err(treexml::Error::ParseError(format_err!(
                        "Invalid boolean value: {}",
                        &other
                    ))),
                },
            },
            Err(e) => match e {
                treexml::Error::ElementNotFound { .. } => Ok(false),
                _ => Err(e),
            },
        }
    }

    fn unmarshal_into<T>(&self, out: &mut T) -> Result<bool, treexml::Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        match self.text {
            None => Ok(false),
            Some(ref text) => {
                std::mem::swap(
                    out,
                    &mut match T::from_str(text) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(treexml::Error::ValueFromStr { t: e.to_string() });
                        }
                    },
                );
                Ok(true)
            }
        }
    }

    fn unmarshal_bool_into(&self, out: &mut bool) -> Result<bool, treexml::Error> {
        match self.text {
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
                            return Err(treexml::Error::ValueFromStr { t: e.to_string() });
                        }
                    },
                );
                Ok(true)
            }
        }
    }
}

pub trait Unmarshaller {
    fn unmarshal_from(&mut self, &treexml::Element) -> Result<bool, treexml::Error>;
}

impl Unmarshaller for bool {
    fn unmarshal_from(&mut self, node: &treexml::Element) -> Result<bool, treexml::Error> {
        node.unmarshal_bool_into(self)
    }
}

impl Unmarshaller for i64 {
    fn unmarshal_from(&mut self, node: &treexml::Element) -> Result<bool, treexml::Error> {
        node.unmarshal_into(self)
    }
}

impl Unmarshaller for f64 {
    fn unmarshal_from(&mut self, node: &treexml::Element) -> Result<bool, treexml::Error> {
        node.unmarshal_into(self)
    }
}

impl Unmarshaller for String {
    fn unmarshal_from(&mut self, node: &treexml::Element) -> Result<bool, treexml::Error> {
        node.unmarshal_into(self)
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
        result.unmarshal_from(&fixture).unwrap();

        assert_eq!(expectation, result);
    }

    #[test]
    fn test_deserialize_bool() {
        let fixture = parse_node("<do_want/>").unwrap().unwrap();
        let expectation = true;

        let mut result = bool::default();
        result.unmarshal_from(&fixture).unwrap();

        assert_eq!(expectation, result);
    }

    #[test]
    fn test_find_value() {
        let root = treexml::Element {
            name: "root".into(),
            children: vec![treexml::Element {
                name: "key".into(),
                text: Some("value".into()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let expectation = "value".to_string();

        let result = root.find_value1::<String, _>("key").unwrap();

        assert_eq!(expectation, result);
    }
}
