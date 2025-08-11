
pub(crate) struct CssValueParser<'a> {
    s: &'a str,
    pos: usize,
    length: usize,
}
impl<'a> CssValueParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            pos: 0,
            length: s.chars().count()
        }
    }

    pub fn chars(&self) -> std::str::Chars<'a> {
        self.s[self.pos..].chars()
    }
    pub fn advance(&mut self, n: usize) {
        self.pos = self.length.min(self.pos + n);
    }
    pub fn char(&self) -> Option<char> {
        self.chars().next()
    }

    pub fn skip_spaces(&mut self) {
        let chars = self.chars().enumerate();
        for (n, c) in chars {
            if !c.is_whitespace() {
                self.pos += n;
                break;
            }
        }
    }
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.s[start..end]
    }

    pub fn read_until(&self, f: impl Fn(char) -> bool) -> &'a str {
        let start = self.pos;
        while let Some(char) = self.char() {
            if f(char) {
                break;
            }
        }

        self.slice(start, self.pos)
    }
}
