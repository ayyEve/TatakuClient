// use crate::prelude::*;
use super::super::prelude::*;

// lazy_static::lazy_static! {
//     static ref CHAR_MAPPING: Arc<HashMap<&'static str, Vec<CharVariant>>> = {
//         // many of these are probably wrong because i typed them out on my eng kb
//         let list = vec![
//             ("", vec![]),
//             ("ん", vec![CharVariant::new(&['n'])]),

//             // vowel only
//             ("あ", vec![CharVariant::new(&['a'])]),
//             ("い", vec![CharVariant::new(&['i'])]),
//             ("う", vec![CharVariant::new(&['u'])]),
//             ("え", vec![CharVariant::new(&['e'])]),
//             ("お", vec![CharVariant::new(&['o'])]),

//             // starts with 'b'
//             ("ば", vec![CharVariant::new(&['b','a'])]),
//             ("び", vec![CharVariant::new(&['b','i'])]),
//             ("ぶ", vec![CharVariant::new(&['b','u'])]),
//             ("べ", vec![CharVariant::new(&['b','e'])]),
//             ("ぼ", vec![CharVariant::new(&['b','o'])]),

//             // starts with 'n'
//             ("な", vec![CharVariant::new(&['n','a'])]),
//             ("に", vec![CharVariant::new(&['n','i'])]),
//             ("ぬ", vec![CharVariant::new(&['n','u'])]),
//             ("ね", vec![CharVariant::new(&['n','e'])]),
//             ("の", vec![CharVariant::new(&['n','o'])]),

//             // starts with 'w'
//             ("わ", vec![CharVariant::new(&['w','a'])]),
//             ("ゐ", vec![CharVariant::new(&['w','i'])]), // doesnt exist but dont care
//             // ("𛄟", vec!['w','u']),
//             ("ゑ", vec![CharVariant::new(&['w','e'])]), // doesnt exist but dont care
//             ("を", vec![CharVariant::new(&['w','o'])]),
            
//             // starts with 'r'
//             ("ら", vec![CharVariant::new(&['r','a'])]),
//             ("り", vec![CharVariant::new(&['r','i'])]),
//             ("る", vec![CharVariant::new(&['r','u'])]),
//             ("れ", vec![CharVariant::new(&['r','e'])]),
//             ("ろ", vec![CharVariant::new(&['r','o'])]),
            
//             // starts with 'y'
//             ("や", vec![CharVariant::new(&['y','a'])]),
//             // ("い", vec!['y','i']),
//             ("ゆ", vec![CharVariant::new(&['y','u'])]),
//             // ("いぇ", vec!['y','e']),
//             ("よ", vec![CharVariant::new(&['y','o'])]),
            
//             // starts with 'm'
//             ("ま", vec![CharVariant::new(&['m','a'])]),
//             ("み", vec![CharVariant::new(&['m','i'])]),
//             ("む", vec![CharVariant::new(&['m','u'])]),
//             ("め", vec![CharVariant::new(&['m','e'])]),
//             ("も", vec![CharVariant::new(&['m','o'])]),

//             // starts with 'h'
//             ("は", vec![CharVariant::new(&['h','a'])]),
//             ("ひ", vec![CharVariant::new(&['h','i'])]),
//             ("ふ", vec![CharVariant::new(&['f','u'])]), // fu
//             ("へ", vec![CharVariant::new(&['h','e'])]),
//             ("ほ", vec![CharVariant::new(&['h','o'])]),

//             // starts with 't'
//             ("た", vec![CharVariant::new(&['t','a'])]),
//             ("ち", vec![CharVariant::new(&['c','h','i'])]), // chi
//             ("つ", vec![CharVariant::new(&['t','s','u'])]), // tsu
//             ("っ", vec![CharVariant::new(&['t','u'])]),
//             ("て", vec![CharVariant::new(&['t','e'])]),
//             ("と", vec![CharVariant::new(&['t','o'])]),

//             // starts with 's'
//             ("さ", vec![CharVariant::new(&['s','a'])]),
//             ("し", vec![CharVariant::new(&['s', 'h','i'])]), // shi
//             ("じ", vec![CharVariant::new(&['s', 'h','i'])]), // shi
//             ("す", vec![CharVariant::new(&['s','u'])]),
//             ("せ", vec![CharVariant::new(&['s','e'])]),
//             ("そ", vec![CharVariant::new(&['s','o'])]),

//             // starts with 'k'
//             ("か", vec![CharVariant::new(&['k','a'])]),
//             ("き", vec![CharVariant::new(&['k','i'])]),
//             ("く", vec![CharVariant::new(&['k','u'])]),
//             ("け", vec![CharVariant::new(&['k','e'])]),
//             ("こ", vec![CharVariant::new(&['k','o'])]),

//             // starts with 'g'
//             ("が", vec![CharVariant::new(&['g','a'])]),
//             ("ぎ", vec![CharVariant::new(&['g','i'])]),
//             ("ぐ", vec![CharVariant::new(&['g','u'])]),
//             ("げ", vec![CharVariant::new(&['g','e'])]),
//             ("ご", vec![CharVariant::new(&['g','o'])]),

//             // starts with 'd'
//             ("だ", vec![CharVariant::new(&['d','a'])]),
//             ("ぢ", vec![CharVariant::new(&['d','i'])]),
//             ("づ", vec![CharVariant::new(&['d','u'])]),
//             ("で", vec![CharVariant::new(&['d','e'])]),
//             ("ど", vec![CharVariant::new(&['d','o'])]),

//             // starts with 'z'
//             ("ざ", vec![CharVariant::new(&['z','a'])]),
//             // ("じ", vec!['z','i']),
//             ("ず", vec![CharVariant::new(&['z','u'])]),
//             ("ぜ", vec![CharVariant::new(&['z','e'])]),
//             ("ぞ", vec![CharVariant::new(&['z','o'])]),
//         ];

//         Arc::new(list.into_iter().collect::<_>())
//     };
// }

// list of branches for a string of text
#[derive(Clone, Debug)]
pub struct Branch {
    branches: Vec<TextVariant>,
    available_branches: Vec<TextVariant>,

    current_text: String,
    current_chars: Vec<char>
}
impl Branch {
    /// text can be as many chars
    pub fn new(text: &String) -> Self {

        // branch 1: [chi]
        // branch 2: [na, ra]
        // branch 3: [i, a]
        let branches_per_char: Vec<Vec<CharVariant>> = 
        // try to get an exact match from the mapping
        CHAR_MAPPING
        .get(&**text)
        .map(|a|vec![a.clone()])
        
        // if no exact mapping exists, manually parse
        .unwrap_or_else(||
            text
            .chars()
            .into_iter()
            .map(|c|
                CHAR_MAPPING
                .get(&*c.to_string())
                .cloned()
                .unwrap_or_else(||vec![CharVariant::new(&[c])])
            )
            .collect()
        );
        
        // branch 1: [chi, ra, i]
        // branch 2: [chi, ra, a]
        // branch 3: [chi, na, i]
        // branch 4: [chi, na, a]
        let branches = cartesian_product(&branches_per_char);
        let branches: Vec<TextVariant> = branches.into_iter().map(|b|TextVariant::new(b)).collect();

        Self {
            available_branches: branches.clone(),
            branches,
            current_text: String::new(),
            current_chars: Vec::new()
        }
    }

    /// add char, returns true if one possible branch is complete
    pub fn add_char(&mut self, c:char) -> bool {
        self.current_text.push(c);
        self.current_chars.push(c);
        self.available_branches.retain_mut(|b|b.add_char(c));

        self.available_branches.iter().any(|b|b.is_complete())
    }

    // check to see if a char is accepted
    pub fn check_char(&self, c: char) -> bool {
        self.available_branches.iter().any(|b|b.check_char(c))
    }

    pub fn reset(&mut self) {
        self.current_text.clear();
        self.current_chars.clear();
        self.available_branches = self.branches.clone();
    }

    pub fn get_strs(&self) -> Vec<String> {
        self.available_branches.iter().map(|b|b.get_text()).collect()
    }

    pub fn current_text(&self) -> String {
        self.current_text.clone()
    }

    pub fn get_first(&self) -> Vec<char> {
        self.available_branches.first().map(|b|b.get_first()).unwrap_or_default()
    }
}


// list of branches for a single char
#[derive(Clone, Debug)]
pub struct TextVariant {
    /// concatenated list of texts
    char_list: Vec<char>,
    
    /// list of currently entered chars
    current_chars: Vec<char>,
}
impl TextVariant {
    fn new(branches: Vec<CharVariant>) -> Self {
        let char_list = branches.iter().map(|a|a.0.clone()).collect::<Vec<_>>().concat();

        Self {
            char_list,
            current_chars: Vec::new()
        }
    }

    /// add char, returns if this branch is still valid
    pub fn add_char(&mut self, c:char) -> bool {
        self.current_chars.push(c);
        self.current_chars.iter().zip(self.char_list.iter()).all(|(a, b)| a == b)
    }

    pub fn is_complete(&self) -> bool {
        self.current_chars == self.char_list
    }

    pub fn check_char(&self, c: char) -> bool { 
        // assume everything up until this point has been correct
        // logic higher up should enforce this

        let i = self.current_chars.len();
        if i >= self.char_list.len() { return false }
        self.char_list[i] == c
    }

    pub fn get_text(&self) -> String {
        String::from_iter(self.char_list.iter())
    }

    pub fn get_first(&self) -> Vec<char> {
        self.char_list.clone()
    }
}


/// list of romaji chars for a single jap/eng char
#[derive(Clone, Default, Debug)]
pub struct CharVariant(Vec<char>);
impl CharVariant {
    pub fn new(chars: impl AsRef<[char]>) -> Self {
        Self(Vec::from_iter(chars.as_ref().iter().map(|c|*c)))
    }
}


// from https://rosettacode.org/wiki/Category:Rust
fn cartesian_product<T:Clone>(lists: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut res = Vec::new();

    let mut list_iter = lists.iter();
    if let Some(first_list) = list_iter.next() {
        for i in first_list {
            res.push(vec![i.clone()]);
        }
    }

    for l in list_iter {
        let mut tmp = Vec::new();
        for r in res {
            for el in l {
                let mut tmp_el = r.clone();
                tmp_el.push(el.clone());
                tmp.push(tmp_el);
            }
        }
        res = tmp;
    }
    res
}


#[test]
fn test() {
    let n = vec![vec!["chi"], vec!["ra", "na"], vec!["i", "a"]];
    let x = cartesian_product(&n);
    assert_eq!(x,vec![vec!["chi", "ra", "i"], vec!["chi", "ra", "a"], vec!["chi", "na", "i"], vec!["chi", "na", "a"]]);
}
