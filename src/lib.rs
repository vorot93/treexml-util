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
    fn find_value0<T>(&self, name: &str) -> Result<Option<T>, treexml::Error>
    where
        T: std::str::FromStr;

    fn find_value1<T>(&self, name: &str) -> Result<T, treexml::Error>
    where
        T: std::str::FromStr;

    fn find_bool(&self, name: &str) -> Result<bool, treexml::Error>;

    fn unmarshal_into<T>(&self, out: &mut T) -> Result<bool, treexml::Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display;
    fn unmarshal_bool_into(&self, out: &mut bool) -> Result<bool, treexml::Error>;
}

impl ElementExt for treexml::Element {
    fn find_value0<T>(&self, name: &str) -> Result<Option<T>, treexml::Error>
    where
        T: std::str::FromStr,
    {
        self.find_value(name).or_else(|e| match e {
            treexml::Error::ElementNotFound { .. } => Ok(None),
            _ => Err(e),
        })
    }

    fn find_value1<T>(&self, name: &str) -> Result<T, treexml::Error>
    where
        T: std::str::FromStr,
    {
        self.find_value0(name).and_then(|v| {
            v.ok_or_else(|| treexml::Error::ParseError(format_err!("Value not found: {}", name)))
        })
    }

    fn find_bool(&self, name: &str) -> Result<bool, treexml::Error> {
        match self.find_value(name) {
            Ok(None) => Err(treexml::Error::ParseError(
                format_err!("Boolean value not found for key {}", name).into(),
            )),
            Ok(Some(v)) => Ok(v),
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
                            return Err(treexml::Error::ValueFromStr { t: e.to_string() }.into());
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
                            return Err(treexml::Error::ValueFromStr { t: e.to_string() }.into());
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
}
