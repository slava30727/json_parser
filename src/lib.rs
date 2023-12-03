use std::{collections::HashMap, str::FromStr, hint::unreachable_unchecked};



#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let JsonValue::Integer(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let JsonValue::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let JsonValue::String(value) = self {
            Some(value.as_str())
        } else {
            None
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let JsonValue::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&[Self]> {
        if let JsonValue::Array(value) = self {
            Some(value.as_slice())
        } else {
            None
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        if let JsonValue::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Self>> {
        if let JsonValue::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Self>> {
        if let JsonValue::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn parse_char(src: &str, value: char) -> (&str, Option<char>) {
        if src.starts_with(value) {
            (src.split_once(value).unwrap().1, Some(value))
        } else {
            (src, None)
        }
    }

    pub fn parse_sequence<'c, 'q>(src: &'c str, seq: &'q str)
        -> (&'c str, Option<&'q str>)
    {
        if src.starts_with(seq) {
            (src.split_once(seq).unwrap().1, Some(seq))
        } else {
            (src, None)
        }
    }

    pub fn parse_whitespaces(src: &str) -> (&str, &str) {
        Self::parse_span(src, |&c| c.is_whitespace())
    }

    pub fn parse_span(src: &str, pred: impl Fn(&char) -> bool) -> (&str, &str) {
        let trimmed = src.trim_start_matches(|c| pred(&c));
        (trimmed, unsafe { src.get_unchecked(0..src.len() - trimmed.len()) })
    }

    pub fn parse_null(src: &str) -> (&str, Option<JsonValue>) {
        let (src, sub_string) = Self::parse_sequence(src, "null");
        (src, sub_string.map(|_| JsonValue::Null))
    }

    pub fn parse_bool(src: &str) -> (&str, Option<JsonValue>) {
        let (mut new_src, mut sub_string) = Self::parse_sequence(src, "true");

        if sub_string.is_some() {
            return (new_src, Some(JsonValue::from(true)))
        }

        (new_src, sub_string) = Self::parse_sequence(src, "false");

        if sub_string.is_some() {
            return (new_src, Some(JsonValue::from(false)));
        }

        (src, None)
    }

    pub fn parse_integer(src: &str) -> (&str, Option<JsonValue>) {
        let (src, sub_string) = Self::parse_span(src, char::is_ascii_digit);

        if sub_string.is_empty() {
            return (src, None);
        }

        (
            src,
            sub_string
                .parse::<i64>()
                .ok()
                .map(JsonValue::Integer)
        )
    }

    pub fn parse_float(src: &str) -> (&str, Option<JsonValue>) {
        let (mut new_src, whole_value) = Self::parse_integer(src);

        let (whole, has_whole) = match whole_value {
            None => (0, false),
            Some(JsonValue::Integer(value)) => (value, true),
            _ => unsafe { unreachable_unchecked() },
        };

        let point;
        (new_src, point) = Self::parse_char(new_src, '.');

        if point.is_none() {
            return (src, None);
        }

        let frac_value;
        (new_src, frac_value) = Self::parse_integer(new_src);

        let (frac, has_frac) = match frac_value {
            None => (0, false),
            Some(JsonValue::Integer(value)) => (value, true),
            _ => unsafe { unreachable_unchecked() },
        };

        if !has_whole && !has_frac {
            return (src, None);
        }

        let mut frac_part = frac as f64;
        
        while 1.0 < frac_part {
            frac_part /= 10.0;
        }

        (new_src, Some(JsonValue::from(whole as f64 + frac_part)))
    }

    pub fn parse_string(src: &str) -> (&str, Option<JsonValue>) {
        let (mut new_src, open_quote) = Self::parse_char(src, '"');

        if open_quote.is_none() {
            return (src, None);
        }

        let mut string;
        (new_src, string) = Self::parse_span(new_src, |&c| c != '"');

        while string.ends_with('\\') {
            let quote;
            (new_src, quote) = Self::parse_char(new_src, '"');

            if quote.is_none() {
                return (src, None);
            }

            let tail;
            (new_src, tail) = Self::parse_span(new_src, |&c| c != '"');

            // Safety:
            // 
            // - '"' is an ASCII character so it requres only one bytes
            // - `src` contains only valid UTF-8
            // - we parsed '"' so `src` contains '"'
            // - we parsed `tail` so `src` contains `tail`
            string = unsafe {
                std::str::from_utf8_unchecked(
                    std::slice::from_raw_parts(
                        string.as_ptr(),
                        string.len() + tail.len() + 1
                    )
                )
            };
        }

        let close_quote;
        (new_src, close_quote) = Self::parse_char(new_src, '"');

        if close_quote.is_none() {
            return (src, None);
        }

        (new_src, Some(JsonValue::from(string)))
    }

    pub fn parse_array(src: &str) -> (&str, Option<JsonValue>) {
        let (mut new_src, open_bracket) = Self::parse_char(src, '[');

        if open_bracket.is_none() {
            return (src, None);
        }

        (new_src, _) = Self::parse_span(new_src, |&c| char::is_whitespace(c));

        let mut values = vec![];

        loop {
            let value;
            (new_src, value) = Self::parse_value(new_src);
            
            match value {
                None => break,
                Some(value) => values.push(value),
            }

            (new_src, _) = Self::parse_whitespaces(new_src);

            let comma;
            (new_src, comma) = Self::parse_char(new_src, ',');

            if comma.is_none() {
                break;
            }

            (new_src, _) = Self::parse_whitespaces(new_src);
        }

        (new_src, _) = Self::parse_whitespaces(new_src);

        let close_bracket;
        (new_src, close_bracket) = Self::parse_char(new_src, ']');

        if close_bracket.is_none() {
            return (src, None);
        }

        (new_src, Some(JsonValue::from(values)))
    }

    pub fn parse_object(src: &str) -> (&str, Option<JsonValue>) {
        let (mut new_src, open_brace) = Self::parse_char(src, '{');

        if open_brace.is_none() {
            return (src, None);
        }

        (new_src, _) = Self::parse_span(new_src, |&c| char::is_whitespace(c));

        let mut values = HashMap::new();

        loop {
            let key;
            (new_src, key) = Self::parse_string(new_src);

            let Some(JsonValue::String(key)) = key else { break };

            (new_src, _) = Self::parse_whitespaces(new_src);

            let colon;
            (new_src, colon) = Self::parse_char(new_src, ':');

            if colon.is_none() {
                return (src, None);
            }

            (new_src, _) = Self::parse_whitespaces(new_src);

            let value;
            (new_src, value) = Self::parse_value(new_src);
            
            let Some(value) = value else {
                return (src, None);
            };

            values.insert(key, value);

            (new_src, _) = Self::parse_whitespaces(new_src);

            let comma;
            (new_src, comma) = Self::parse_char(new_src, ',');

            if comma.is_none() {
                break;
            }

            (new_src, _) = Self::parse_whitespaces(new_src);
        }

        (new_src, _) = Self::parse_whitespaces(new_src);

        let close_bracket;
        (new_src, close_bracket) = Self::parse_char(new_src, '}');

        if close_bracket.is_none() {
            return (src, None);
        }

        (new_src, Some(JsonValue::from(values)))
    }

    pub fn parse_value(src: &str) -> (&str, Option<JsonValue>) {
        Self::parse_try(src, [
            Self::parse_null,
            Self::parse_bool,
            Self::parse_float,
            Self::parse_integer,
            Self::parse_string,
            Self::parse_array,
            Self::parse_object,
        ])
    }

    pub fn parse_try(
        mut src: &str,
        parsers: impl IntoIterator<Item = fn(&str) -> (&str, Option<JsonValue>)>
    ) -> (&str, Option<JsonValue>) {
        let mut value = None;

        for parse in parsers.into_iter() {
            (src, value) = parse(src);

            if let Some(value) = value {
                return (src, Some(value));
            }
        }

        (src, value)
    }
}

impl From<bool> for JsonValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for JsonValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for JsonValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for JsonValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&'_ str> for JsonValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<Vec<Self>> for JsonValue {
    fn from(value: Vec<Self>) -> Self {
        Self::Array(value)
    }
}

impl From<HashMap<String, Self>> for JsonValue {
    fn from(value: HashMap<String, Self>) -> Self {
        Self::Object(value)
    }
}

impl FromStr for JsonValue {
    type Err = String;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let (src, value) = Self::parse_value(src.trim());

        let Some(value) = value else {
            return Err(format!("failed to parse \"{src}\""));
        };

        if !src.is_empty() {
            return Err(
                format!("failed to parse entire value, reminder: \"{src}\"")
            );
        }

        Ok(value)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let input = r#"{
            "quiz": {
                "sport": {
                    "q1": {
                        "question": "Which one is correct team name in NBA?",
                        "options": [
                            "New York Bulls",
                            "Los Angeles Kings",
                            "Golden State Warriros",
                            "Huston Rocket"
                        ],
                        "answer": "Huston \"Rocket\""
                    }
                },
                "maths": {
                    "q1": {
                        "question": "5 + 7 = ?",
                        "options": [
                            "10",
                            "11",
                            "12",
                            "13"
                        ],
                        "answer": "12"
                    },
                    "q2": {
                        "question": "12 - 8 = ?",
                        "options": [
                            "1",
                            "2",
                            "3",
                            "4"
                        ],
                        "answer": "4"
                    }
                }
            }
        }"#;

        let value: JsonValue = input.parse().unwrap();

        println!("{value:#?}");
    }

    #[test]
    fn test_parse_float() {
        let input = 1324.34576.to_string();

        let JsonValue::Float(value) = input.parse().unwrap() else {
            panic!()
        };

        assert_eq!(input, value.to_string());
    }

    #[test]
    fn test_parse_object() {
        let input
            = r#"{   "key"  :     true,  "key341": null  ,   "true" : 234  }"#;
        
        let JsonValue::Object(value) = input.parse().unwrap() else {
            panic!()
        };

        println!("{value:?}");
    }

    #[test]
    fn test_print_ages() {
        let input = r#"[
            {
                "name": "Jonson",
                "age": 19,
                "gender": "male"
            },
            {
                "name": "Mary",
                "age": 13,
                "gender": "female"
            },
            {
                "name": "Sofie",
                "age": 7,
                "gender": "female"
            },
            {
                "name": "Max",
                "age": 21,
                "gender": "male"
            },
            {
                "name": "Phill",
                "age": 35,
                "gender": "male"
            },
            {
                "name": "Joseph",
                "age": 15,
                "gender": "male"
            },
            {
                "name": "Kristy",
                "age": 25,
                "gendr": "female"
            }
        ]"#;

        let json: JsonValue = input.parse().unwrap();

        if let JsonValue::Array(array) = json {
            for value in array {
                if let JsonValue::Object(object) = value {
                    if let Some(JsonValue::Integer(age)) = object.get("age") {
                        if *age >= 18 {
                            println!("{object:#?}");
                        }
                    }
                }
            }
        }
    }
}