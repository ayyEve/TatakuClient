use crate::prelude::*;
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BeatmapLink {
    #[serde(alias="@hash")]
    pub beatmap_hash: String,

    #[serde(alias="@title")]
    pub beatmap_title: String,

    #[serde(alias="@link", default)]
    pub download_link: Option<String>,
}
impl FromStr for BeatmapLink {
    type Err = TatakuError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        quick_xml::de::from_str(s)
            .map_err(TatakuError::from_err)
    }
}
impl std::fmt::Display for BeatmapLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = &self.beatmap_title;
        let hash = &self.beatmap_hash;
        if let Some(link) = &self.download_link {
            write!(f, "<beatmapLink hash=\\{hash}\" title=\"{title}\" link=\"{link}\" />")
        } else {
            write!(f, "<beatmapLink hash=\\{hash}\" title=\"{title}\" />")
        }
    }
}


// pub struct BeatmapLink<'a> {
//     pub beatmap_hash: Cow<'a, str>,
//     pub beatmap_title: Cow<'a, str>,
//     pub download_link: Option<Cow<'a, str>>,
// }
// impl<'a> BeatmapLink<'a> {
//     #[allow(clippy::should_implement_trait, reason = "trait doesnt allow lifetimes")]
//     pub fn from_str(s: &'a str) -> Option<Self> {
//         let mut parser:Parser<'a> = Parser::new(s);
//         parser.read_until('(');
//         parser.read_until('"');
//         parser.advance();
//         if parser.at_end() {
//             return None;
//         }

//         let title = parser.read_until('"').trim_matches('"');
//         // skip )
//         parser.read_until('[');
//         parser.advance();
        
//         if parser.at_end() {
//             return None;
//         }

//         let hash = parser.read_until(']').trim_end_matches(']');
//         if hash.contains('|') {
//             let mut s = hash.split('|');
//             let hash = s.next()?;
//             let link = s.next()?;
//             Some(Self {
//                 beatmap_hash: hash.into(),
//                 beatmap_title: title.into(),
//                 download_link: Some(link.into())
//             })
//         } else {
//             Some(Self {
//                 beatmap_hash: hash.into(),
//                 beatmap_title: title.into(),
//                 download_link: None
//             })
//         }
//     }
// }

// impl std::fmt::Display for BeatmapLink<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let title = &self.beatmap_title;
//         let hash = &self.beatmap_hash;
//         if let Some(link) = &self.download_link {
//             write!(f, "(\"{title}\")[{hash}|{link}]")
//         } else {
//             write!(f, "(\"{title}\")[{hash}]")
//         }
//     }
// }

// struct Parser<'a> {
//     index: usize,
//     s: &'a str,
// }
// impl<'a> Parser<'a> {
//     fn new(s: &'a str) -> Self {
//         Self {
//             s,
//             index: 0,
//         }
//     }

//     fn advance(&mut self) {
//         self.skip(1);
//     }
//     fn skip(&mut self, n: usize) {
//         self.index += n;
//     }

//     fn consume(&mut self, c: char) -> bool {
//         if self.current_char() == Some(c) {
//             self.advance();
//             true
//         } else {
//             false
//         }
//     }

//     fn current_char(&self) -> Option<char> {
//         self.s.chars().nth(self.index)
//     }

//     fn read_until(&mut self, char: char) -> &'a str {
//         let start = self.index;
//         while self.index < self.s.len() && self.current_char() != Some(char) {
//             self.advance();
//         }

//         &self.s[start..self.index]
//     }

//     fn at_end(&self) -> bool {
//         self.index >= self.s.len()
//     }
// }
