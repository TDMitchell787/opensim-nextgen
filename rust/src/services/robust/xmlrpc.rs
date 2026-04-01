use std::collections::HashMap;
use anyhow::{anyhow, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum XmlRpcValue {
    String(String),
    Int(i32),
    Bool(bool),
    Double(f64),
    Struct(HashMap<String, XmlRpcValue>),
    Array(Vec<XmlRpcValue>),
}

impl XmlRpcValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            XmlRpcValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            XmlRpcValue::Int(i) => Some(*i),
            XmlRpcValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            XmlRpcValue::Bool(b) => Some(*b),
            XmlRpcValue::String(s) => match s.as_str() {
                "true" | "True" | "1" => Some(true),
                "false" | "False" | "0" => Some(false),
                _ => None,
            },
            XmlRpcValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            XmlRpcValue::Double(d) => Some(*d),
            XmlRpcValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_struct(&self) -> Option<&HashMap<String, XmlRpcValue>> {
        match self {
            XmlRpcValue::Struct(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<XmlRpcValue>> {
        match self {
            XmlRpcValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&XmlRpcValue> {
        self.as_struct()?.get(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.get(key)?.as_i32()
    }
}

pub fn parse_xmlrpc_call(xml: &str) -> Result<(String, Vec<XmlRpcValue>)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut method_name = String::new();
    let mut params = Vec::new();
    let mut in_method_name = false;
    let mut in_params = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "methodName" => in_method_name = true,
                    "params" => in_params = true,
                    "param" if in_params => {
                        let val = parse_xmlrpc_value(&mut reader)?;
                        params.push(val);
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) if in_method_name => {
                method_name = e.unescape().unwrap_or_default().to_string();
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "methodName" => in_method_name = false,
                    "params" => in_params = false,
                    "methodCall" => break,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XmlRpc parse error: {}", e)),
            _ => {}
        }
    }

    Ok((method_name, params))
}

pub fn parse_xmlrpc_response(xml: &str) -> Result<XmlRpcValue> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut in_fault = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "fault" => in_fault = true,
                    "param" if !in_fault => {
                        return parse_xmlrpc_value(&mut reader);
                    }
                    "value" if in_fault => {
                        return parse_xmlrpc_value_inner(&mut reader);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XmlRpc response parse error: {}", e)),
            _ => {}
        }
    }

    Err(anyhow!("No value found in XmlRpc response"))
}

fn parse_xmlrpc_value(reader: &mut Reader<&[u8]>) -> Result<XmlRpcValue> {
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "value" {
                    return parse_xmlrpc_value_inner(reader);
                }
            }
            Ok(Event::Eof) => return Err(anyhow!("Unexpected EOF in XmlRpc value")),
            Err(e) => return Err(anyhow!("XmlRpc value parse error: {}", e)),
            _ => {}
        }
    }
}

fn parse_xmlrpc_value_inner(reader: &mut Reader<&[u8]>) -> Result<XmlRpcValue> {
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let type_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let val = match type_name.as_str() {
                    "string" => {
                        let text = read_text_until(reader, "string")?;
                        XmlRpcValue::String(text)
                    }
                    "int" | "i4" => {
                        let text = read_text_until(reader, &type_name)?;
                        XmlRpcValue::Int(text.parse().unwrap_or(0))
                    }
                    "boolean" => {
                        let text = read_text_until(reader, "boolean")?;
                        XmlRpcValue::Bool(text == "1" || text.eq_ignore_ascii_case("true"))
                    }
                    "double" => {
                        let text = read_text_until(reader, "double")?;
                        XmlRpcValue::Double(text.parse().unwrap_or(0.0))
                    }
                    "struct" => parse_xmlrpc_struct(reader)?,
                    "array" => parse_xmlrpc_array(reader)?,
                    _ => {
                        let text = read_text_until(reader, &type_name)?;
                        XmlRpcValue::String(text)
                    }
                };
                skip_to_end_tag(reader, "value")?;
                return Ok(val);
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                skip_to_end_tag(reader, "value")?;
                return Ok(XmlRpcValue::String(text));
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "value" {
                    return Ok(XmlRpcValue::String(String::new()));
                }
            }
            Ok(Event::Eof) => return Err(anyhow!("Unexpected EOF in value")),
            Err(e) => return Err(anyhow!("XmlRpc parse error: {}", e)),
            _ => {}
        }
    }
}

fn parse_xmlrpc_struct(reader: &mut Reader<&[u8]>) -> Result<XmlRpcValue> {
    let mut map = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "member" {
                    let (key, val) = parse_xmlrpc_member(reader)?;
                    map.insert(key, val);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "struct" {
                    return Ok(XmlRpcValue::Struct(map));
                }
            }
            Ok(Event::Eof) => return Err(anyhow!("Unexpected EOF in struct")),
            Err(e) => return Err(anyhow!("XmlRpc struct parse error: {}", e)),
            _ => {}
        }
    }
}

fn parse_xmlrpc_member(reader: &mut Reader<&[u8]>) -> Result<(String, XmlRpcValue)> {
    let mut key = String::new();
    let mut value = XmlRpcValue::String(String::new());
    let mut in_name = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "name" => in_name = true,
                    "value" => {
                        value = parse_xmlrpc_value_inner(reader)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) if in_name => {
                key = e.unescape().unwrap_or_default().to_string();
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "name" => in_name = false,
                    "member" => return Ok((key, value)),
                    _ => {}
                }
            }
            Ok(Event::Eof) => return Err(anyhow!("Unexpected EOF in member")),
            Err(e) => return Err(anyhow!("XmlRpc member parse error: {}", e)),
            _ => {}
        }
    }
}

fn parse_xmlrpc_array(reader: &mut Reader<&[u8]>) -> Result<XmlRpcValue> {
    let mut items = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "value" {
                    items.push(parse_xmlrpc_value_inner(reader)?);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "array" {
                    return Ok(XmlRpcValue::Array(items));
                }
            }
            Ok(Event::Eof) => return Err(anyhow!("Unexpected EOF in array")),
            Err(e) => return Err(anyhow!("XmlRpc array parse error: {}", e)),
            _ => {}
        }
    }
}

fn read_text_until(reader: &mut Reader<&[u8]>, end_tag: &str) -> Result<String> {
    let mut text = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == end_tag {
                    return Ok(text);
                }
            }
            Ok(Event::Eof) => return Ok(text),
            Err(e) => return Err(anyhow!("XmlRpc read text error: {}", e)),
            _ => {}
        }
    }
}

fn skip_to_end_tag(reader: &mut Reader<&[u8]>, tag: &str) -> Result<()> {
    let mut depth = 0i32;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => depth += 1,
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == tag && depth == 0 {
                    return Ok(());
                }
                depth -= 1;
            }
            Ok(Event::Eof) => return Ok(()),
            Err(e) => return Err(anyhow!("XmlRpc skip error: {}", e)),
            _ => {}
        }
    }
}

pub fn build_xmlrpc_call(method: &str, params: &[XmlRpcValue]) -> String {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", None, None))).ok();

    writer.write_event(Event::Start(BytesStart::new("methodCall"))).ok();
    writer.write_event(Event::Start(BytesStart::new("methodName"))).ok();
    writer.write_event(Event::Text(BytesText::new(method))).ok();
    writer.write_event(Event::End(BytesEnd::new("methodName"))).ok();

    writer.write_event(Event::Start(BytesStart::new("params"))).ok();
    for param in params {
        writer.write_event(Event::Start(BytesStart::new("param"))).ok();
        write_xmlrpc_value(&mut writer, param);
        writer.write_event(Event::End(BytesEnd::new("param"))).ok();
    }
    writer.write_event(Event::End(BytesEnd::new("params"))).ok();

    writer.write_event(Event::End(BytesEnd::new("methodCall"))).ok();

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap_or_default()
}

pub fn build_xmlrpc_response(value: &XmlRpcValue) -> String {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", None, None))).ok();

    writer.write_event(Event::Start(BytesStart::new("methodResponse"))).ok();
    writer.write_event(Event::Start(BytesStart::new("params"))).ok();
    writer.write_event(Event::Start(BytesStart::new("param"))).ok();
    write_xmlrpc_value(&mut writer, value);
    writer.write_event(Event::End(BytesEnd::new("param"))).ok();
    writer.write_event(Event::End(BytesEnd::new("params"))).ok();
    writer.write_event(Event::End(BytesEnd::new("methodResponse"))).ok();

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap_or_default()
}

pub fn build_xmlrpc_fault(code: i32, msg: &str) -> String {
    let mut fault = HashMap::new();
    fault.insert("faultCode".to_string(), XmlRpcValue::Int(code));
    fault.insert("faultString".to_string(), XmlRpcValue::String(msg.to_string()));

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", None, None))).ok();
    writer.write_event(Event::Start(BytesStart::new("methodResponse"))).ok();
    writer.write_event(Event::Start(BytesStart::new("fault"))).ok();
    write_xmlrpc_value(&mut writer, &XmlRpcValue::Struct(fault));
    writer.write_event(Event::End(BytesEnd::new("fault"))).ok();
    writer.write_event(Event::End(BytesEnd::new("methodResponse"))).ok();

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap_or_default()
}

fn write_xmlrpc_value(writer: &mut Writer<Cursor<Vec<u8>>>, value: &XmlRpcValue) {
    writer.write_event(Event::Start(BytesStart::new("value"))).ok();

    match value {
        XmlRpcValue::String(s) => {
            writer.write_event(Event::Start(BytesStart::new("string"))).ok();
            writer.write_event(Event::Text(BytesText::new(s))).ok();
            writer.write_event(Event::End(BytesEnd::new("string"))).ok();
        }
        XmlRpcValue::Int(i) => {
            writer.write_event(Event::Start(BytesStart::new("int"))).ok();
            writer.write_event(Event::Text(BytesText::new(&i.to_string()))).ok();
            writer.write_event(Event::End(BytesEnd::new("int"))).ok();
        }
        XmlRpcValue::Bool(b) => {
            writer.write_event(Event::Start(BytesStart::new("boolean"))).ok();
            writer.write_event(Event::Text(BytesText::new(if *b { "1" } else { "0" }))).ok();
            writer.write_event(Event::End(BytesEnd::new("boolean"))).ok();
        }
        XmlRpcValue::Double(d) => {
            writer.write_event(Event::Start(BytesStart::new("double"))).ok();
            writer.write_event(Event::Text(BytesText::new(&d.to_string()))).ok();
            writer.write_event(Event::End(BytesEnd::new("double"))).ok();
        }
        XmlRpcValue::Struct(map) => {
            writer.write_event(Event::Start(BytesStart::new("struct"))).ok();
            for (key, val) in map {
                writer.write_event(Event::Start(BytesStart::new("member"))).ok();
                writer.write_event(Event::Start(BytesStart::new("name"))).ok();
                writer.write_event(Event::Text(BytesText::new(key))).ok();
                writer.write_event(Event::End(BytesEnd::new("name"))).ok();
                write_xmlrpc_value(writer, val);
                writer.write_event(Event::End(BytesEnd::new("member"))).ok();
            }
            writer.write_event(Event::End(BytesEnd::new("struct"))).ok();
        }
        XmlRpcValue::Array(items) => {
            writer.write_event(Event::Start(BytesStart::new("array"))).ok();
            writer.write_event(Event::Start(BytesStart::new("data"))).ok();
            for item in items {
                write_xmlrpc_value(writer, item);
            }
            writer.write_event(Event::End(BytesEnd::new("data"))).ok();
            writer.write_event(Event::End(BytesEnd::new("array"))).ok();
        }
    }

    writer.write_event(Event::End(BytesEnd::new("value"))).ok();
}

pub fn struct_to_hashmap(value: &XmlRpcValue) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let XmlRpcValue::Struct(map) = value {
        for (k, v) in map {
            if let Some(s) = v.as_str() {
                result.insert(k.clone(), s.to_string());
            } else if let Some(i) = v.as_i32() {
                result.insert(k.clone(), i.to_string());
            } else if let Some(b) = v.as_bool() {
                result.insert(k.clone(), b.to_string());
            } else if let Some(d) = v.as_f64() {
                result.insert(k.clone(), d.to_string());
            }
        }
    }
    result
}

pub fn hashmap_to_struct(map: &HashMap<String, String>) -> XmlRpcValue {
    let mut s = HashMap::new();
    for (k, v) in map {
        s.insert(k.clone(), XmlRpcValue::String(v.clone()));
    }
    XmlRpcValue::Struct(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_parse_call() {
        let mut params_map = HashMap::new();
        params_map.insert("region_name".to_string(), XmlRpcValue::String("Welcome".to_string()));
        let params = vec![XmlRpcValue::Struct(params_map)];

        let xml = build_xmlrpc_call("link_region", &params);
        assert!(xml.contains("link_region"));
        assert!(xml.contains("Welcome"));

        let (method, parsed_params) = parse_xmlrpc_call(&xml).unwrap();
        assert_eq!(method, "link_region");
        assert_eq!(parsed_params.len(), 1);
        assert_eq!(parsed_params[0].get_str("region_name"), Some("Welcome"));
    }

    #[test]
    fn test_build_and_parse_response() {
        let mut result = HashMap::new();
        result.insert("result".to_string(), XmlRpcValue::String("true".to_string()));
        result.insert("uuid".to_string(), XmlRpcValue::String("00000000-0000-0000-0000-000000000001".to_string()));

        let xml = build_xmlrpc_response(&XmlRpcValue::Struct(result));
        assert!(xml.contains("methodResponse"));

        let parsed = parse_xmlrpc_response(&xml).unwrap();
        assert_eq!(parsed.get_str("result"), Some("true"));
        assert_eq!(parsed.get_str("uuid"), Some("00000000-0000-0000-0000-000000000001"));
    }

    #[test]
    fn test_fault_response() {
        let xml = build_xmlrpc_fault(1, "test error");
        assert!(xml.contains("fault"));
        assert!(xml.contains("test error"));
    }

    #[test]
    fn test_nested_struct() {
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), XmlRpcValue::Int(128));
        inner.insert("y".to_string(), XmlRpcValue::Int(256));

        let mut outer = HashMap::new();
        outer.insert("position".to_string(), XmlRpcValue::Struct(inner));
        outer.insert("name".to_string(), XmlRpcValue::String("test".to_string()));

        let xml = build_xmlrpc_response(&XmlRpcValue::Struct(outer));
        let parsed = parse_xmlrpc_response(&xml).unwrap();

        assert_eq!(parsed.get_str("name"), Some("test"));
        let pos = parsed.get("position").unwrap().as_struct().unwrap();
        assert_eq!(pos.get("x").unwrap().as_i32(), Some(128));
        assert_eq!(pos.get("y").unwrap().as_i32(), Some(256));
    }
}
