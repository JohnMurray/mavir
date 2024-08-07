use std::collections::HashSet;
use std::sync::OnceLock;
use std::thread::JoinHandle;

fn java_primitive_types() -> &'static HashSet<&'static str> {
    static HASHSET: OnceLock<HashSet<&str>> = OnceLock::new();
    HASHSET.get_or_init(|| {
        let mut set = HashSet::new();
        set.insert("boolean");
        set.insert("byte");
        set.insert("short");
        set.insert("int");
        set.insert("long");
        set.insert("float");
        set.insert("double");
        set.insert("char");
        set
    })
}

pub fn is_primitive_type(identifier: &str) -> bool {
    return java_primitive_types().contains(identifier);
}



/// Implementation for StripMargin (trait and impl for trait) taken from:
/// https://github.com/rami3l/stripmargin
/// License for StripMargin:
///
/// MIT License
//
// Copyright (c) 2021 rami3l
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
pub trait StripMargin {
    fn strip_margin_with(&self, margin_char: char) -> String;

    fn strip_margin(&self) -> String {
        self.strip_margin_with('|')
    }
}

impl<S: AsRef<str>> StripMargin for S {
    fn strip_margin_with(&self, margin_char: char) -> String {
        self.as_ref()
            .split('\n')
            .map(|line| {
                let mut chars = line.chars().skip_while(|ch| ch.is_whitespace());
                match chars.next() {
                    Some(c) if c == margin_char => chars.collect(),
                    _ => line.to_owned(),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}