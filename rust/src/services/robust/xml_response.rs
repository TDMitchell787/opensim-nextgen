use anyhow::{anyhow, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum XmlValue {
    Str(String),
    Dict(HashMap<String, XmlValue>),
    List(Vec<XmlValue>),
}

impl XmlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            XmlValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&HashMap<String, XmlValue>> {
        match self {
            XmlValue::Dict(d) => Some(d),
            _ => None,
        }
    }
}

pub fn build_xml_response(data: &HashMap<String, XmlValue>) -> String {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer
        .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0", None, None,
        )))
        .ok();

    let root = BytesStart::new("ServerResponse");
    writer.write_event(Event::Start(root)).ok();

    write_xml_data(&mut writer, data);

    writer
        .write_event(Event::End(BytesEnd::new("ServerResponse")))
        .ok();

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap_or_default()
}

fn write_xml_data(writer: &mut Writer<Cursor<Vec<u8>>>, data: &HashMap<String, XmlValue>) {
    for (key, value) in data {
        let safe_key = encode_xml_name(key);
        match value {
            XmlValue::Str(s) => {
                let elem = BytesStart::new(safe_key.as_str());
                writer.write_event(Event::Start(elem)).ok();
                writer.write_event(Event::Text(BytesText::new(s))).ok();
                writer
                    .write_event(Event::End(BytesEnd::new(safe_key.as_str())))
                    .ok();
            }
            XmlValue::Dict(dict) => {
                let mut elem = BytesStart::new(safe_key.as_str());
                elem.push_attribute(("type", "List"));
                writer.write_event(Event::Start(elem)).ok();
                write_xml_data(writer, dict);
                writer
                    .write_event(Event::End(BytesEnd::new(safe_key.as_str())))
                    .ok();
            }
            XmlValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    let item_key = format!("{}{}", safe_key, i);
                    if let XmlValue::Dict(dict) = item {
                        let mut elem = BytesStart::new(item_key.as_str());
                        elem.push_attribute(("type", "List"));
                        writer.write_event(Event::Start(elem)).ok();
                        write_xml_data(writer, dict);
                        writer
                            .write_event(Event::End(BytesEnd::new(item_key.as_str())))
                            .ok();
                    } else if let XmlValue::Str(s) = item {
                        let elem = BytesStart::new(item_key.as_str());
                        writer.write_event(Event::Start(elem)).ok();
                        writer.write_event(Event::Text(BytesText::new(s))).ok();
                        writer
                            .write_event(Event::End(BytesEnd::new(item_key.as_str())))
                            .ok();
                    }
                }
            }
        }
    }
}

pub fn parse_xml_response(xml: &str) -> Result<HashMap<String, XmlValue>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut found_root = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "ServerResponse" {
                    found_root = true;
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {}", e)),
            _ => {}
        }
    }

    if !found_root {
        return Err(anyhow!("Missing <ServerResponse> root element"));
    }

    scan_xml_response(&mut reader, "ServerResponse")
}

fn scan_xml_response(
    reader: &mut Reader<&[u8]>,
    end_tag: &str,
) -> Result<HashMap<String, XmlValue>> {
    let mut result = HashMap::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let decoded_name = decode_xml_name(&name);

                let is_list = e.attributes().filter_map(|a| a.ok()).any(|a| {
                    a.key.as_ref() == b"type" && String::from_utf8_lossy(&a.value) == "List"
                });

                if is_list {
                    let dict = scan_xml_response(reader, &name)?;
                    result.insert(decoded_name, XmlValue::Dict(dict));
                } else {
                    let text = read_element_text(reader, &name)?;
                    result.insert(decoded_name, XmlValue::Str(text));
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(result)
}

fn read_element_text(reader: &mut Reader<&[u8]>, tag_name: &str) -> Result<String> {
    let mut text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == tag_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error reading element text: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(text)
}

pub fn success_result() -> String {
    let mut data = HashMap::new();
    data.insert("Result".to_string(), XmlValue::Str("Success".to_string()));
    build_xml_response(&data)
}

pub fn failure_result(msg: &str) -> String {
    let mut data = HashMap::new();
    data.insert("Result".to_string(), XmlValue::Str("Failure".to_string()));
    data.insert("Message".to_string(), XmlValue::Str(msg.to_string()));
    build_xml_response(&data)
}

pub fn null_result() -> String {
    let mut data = HashMap::new();
    data.insert("result".to_string(), XmlValue::Str("null".to_string()));
    build_xml_response(&data)
}

pub fn bool_result(val: bool) -> String {
    let mut data = HashMap::new();
    data.insert(
        "result".to_string(),
        XmlValue::Str(if val { "true" } else { "false" }.to_string()),
    );
    build_xml_response(&data)
}

pub fn single_result(fields: HashMap<String, String>) -> String {
    let mut inner = HashMap::new();
    for (k, v) in fields {
        inner.insert(k, XmlValue::Str(v));
    }
    let mut data = HashMap::new();
    data.insert("result".to_string(), XmlValue::Dict(inner));
    build_xml_response(&data)
}

pub fn list_result(prefix: &str, items: Vec<HashMap<String, String>>) -> String {
    if items.is_empty() {
        return null_result();
    }
    let mut data = HashMap::new();
    for (i, item) in items.iter().enumerate() {
        let mut inner = HashMap::new();
        for (k, v) in item {
            inner.insert(k.clone(), XmlValue::Str(v.clone()));
        }
        data.insert(format!("{}{}", prefix, i), XmlValue::Dict(inner));
    }
    build_xml_response(&data)
}

fn encode_xml_name(name: &str) -> String {
    name.replace(' ', "_x0020_")
        .replace(':', "_x003A_")
        .replace('.', "_x002E_")
}

fn decode_xml_name(name: &str) -> String {
    name.replace("_x0020_", " ")
        .replace("_x003A_", ":")
        .replace("_x002E_", ".")
}

pub fn try_parse_xml_to_flat(body: &str) -> Option<HashMap<String, String>> {
    if !body.trim_start().starts_with('<') {
        return None;
    }
    let xml_data = parse_xml_response(body).ok()?;
    Some(flatten_xml_values(&xml_data, ""))
}

fn flatten_xml_values(data: &HashMap<String, XmlValue>, prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (key, value) in data {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}_{}", prefix, key)
        };
        match value {
            XmlValue::Str(s) => {
                result.insert(full_key, s.clone());
            }
            XmlValue::Dict(d) => {
                for (dk, dv) in d {
                    match dv {
                        XmlValue::Str(s) => {
                            result.insert(dk.clone(), s.clone());
                        }
                        XmlValue::Dict(dd) => {
                            let nested = flatten_xml_values(dd, dk);
                            result.extend(nested);
                        }
                        XmlValue::List(_) => {}
                    }
                }
            }
            XmlValue::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    if let XmlValue::Dict(d) = item {
                        for (dk, dv) in d {
                            if let XmlValue::Str(s) = dv {
                                result.insert(format!("{}{}_{}", dk, prefix, i), s.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn parse_form_body(body: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let key = urlencoding::decode(&key.replace('+', " "))
                .unwrap_or_default()
                .to_string();
            let value = urlencoding::decode(&value.replace('+', " "))
                .unwrap_or_default()
                .to_string();
            result.insert(key, value);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        let xml = success_result();
        assert!(xml.contains("<ServerResponse>"));
        assert!(xml.contains("<Result>Success</Result>"));
    }

    #[test]
    fn test_failure_result() {
        let xml = failure_result("not found");
        assert!(xml.contains("<Result>Failure</Result>"));
        assert!(xml.contains("<Message>not found</Message>"));
    }

    #[test]
    fn test_null_result() {
        let xml = null_result();
        assert!(xml.contains("<result>null</result>"));
    }

    #[test]
    fn test_single_result() {
        let mut fields = HashMap::new();
        fields.insert(
            "uuid".to_string(),
            "00000000-0000-0000-0000-000000000001".to_string(),
        );
        fields.insert("regionName".to_string(), "Test".to_string());
        let xml = single_result(fields);
        assert!(xml.contains("type=\"List\""));
        assert!(xml.contains("<uuid>"));
    }

    #[test]
    fn test_list_result() {
        let mut item1 = HashMap::new();
        item1.insert("uuid".to_string(), "id1".to_string());
        let mut item2 = HashMap::new();
        item2.insert("uuid".to_string(), "id2".to_string());
        let xml = list_result("region", vec![item1, item2]);
        assert!(xml.contains("region0"));
        assert!(xml.contains("region1"));
    }

    #[test]
    fn test_parse_xml_response_simple() {
        let xml =
            r#"<?xml version="1.0"?><ServerResponse><Result>Success</Result></ServerResponse>"#;
        let result = parse_xml_response(xml).unwrap();
        assert_eq!(result.get("Result").unwrap().as_str(), Some("Success"));
    }

    #[test]
    fn test_parse_xml_response_nested() {
        let xml = r#"<?xml version="1.0"?><ServerResponse><result type="List"><uuid>abc</uuid><name>Test</name></result></ServerResponse>"#;
        let result = parse_xml_response(xml).unwrap();
        let inner = result.get("result").unwrap().as_dict().unwrap();
        assert_eq!(inner.get("uuid").unwrap().as_str(), Some("abc"));
        assert_eq!(inner.get("name").unwrap().as_str(), Some("Test"));
    }

    #[test]
    fn test_roundtrip() {
        let mut fields = HashMap::new();
        fields.insert("uuid".to_string(), "test-id".to_string());
        fields.insert("name".to_string(), "My Region".to_string());
        let xml = single_result(fields);
        let parsed = parse_xml_response(&xml).unwrap();
        let inner = parsed.get("result").unwrap().as_dict().unwrap();
        assert_eq!(inner.get("uuid").unwrap().as_str(), Some("test-id"));
        assert_eq!(inner.get("name").unwrap().as_str(), Some("My Region"));
    }

    #[test]
    fn test_parse_form_body() {
        let body = "METHOD=register&REGIONID=abc-123&REGIONNAME=Test+Region";
        let result = parse_form_body(body);
        assert_eq!(result.get("METHOD").unwrap(), "register");
        assert_eq!(result.get("REGIONID").unwrap(), "abc-123");
        assert_eq!(result.get("REGIONNAME").unwrap(), "Test Region");
    }
}
