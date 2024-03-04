use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ordinal(pub usize);

impl Display for Ordinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tag(pub usize);

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
// bool has_write_event(PERMISSION);
// bool has_read_events(PERMISSION);
// bool has_reply_on_query(PERMISSION);
// bool has_client_actions(PERMISSION);

// struct TOKEN{
//     NAME name;             // who is the token owner?
//     PERMISSION permission; // what permissions does it have?
// }; 

#[derive(Serialize, Deserialize)]
pub struct Event {
    ordinal: Ordinal,
    tag: Tag,
    ext: bool,
    name: String,
    data: Vec<u8>,
    answer: Option<Answer>,
}

#[derive(Serialize, Deserialize)]
pub struct NewEvent {
    pub tag: Tag,
    pub external: bool,
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct NewHttpEvent {
    pub tag: Tag,
    pub external: bool,
    pub data: Vec<u8>,
}

impl NewHttpEvent {
    pub fn name(self, name: String) -> NewEvent {
        NewEvent {
            tag: self.tag,
            external: self.external,
            name,
            data: self.data,
        }
    }
}

impl Event {
    pub fn fresh(tag: Tag, name: String, data: Vec<u8>, external: bool) -> NewEvent {
        NewEvent { tag, external, name, data }
    }

    pub fn from_values(ordinal: Ordinal, tag: Tag, external: bool, name: String, data: Vec<u8>, answer: Option<Answer>) -> Self {
        Self {
            ordinal,
            tag,
            ext: external,
            name,
            data,
            answer,
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        f.write_fmt(format_args!("#{} - tag {}\n  external - [{}], created by {}\n  data: ",
            self.ordinal, self.tag, if self.ext {"x"} else {" "}, self.name
        ))?;

        if let Ok(string) = String::from_utf8(self.data.clone()) {
            f.write_fmt(format_args!("{}", string))
        } else {
            f.write_fmt(format_args!("{:X?}", &self.data))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Answer{
    pub name: String,
    pub data: Vec<u8>
}
