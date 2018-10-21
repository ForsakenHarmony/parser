const input: &'static str = include_str!("../test.js");

fn main() {
    println!("{:#?}", to_tokens(input.to_string()));
}

#[derive(Debug)]
struct Ident(String);

#[derive(Debug)]
enum Lit {
    Num(f32),
    Str(String),
    Bool(bool),
}

#[derive(Debug)]
enum Bracket {
    /// {}
    Brace,
    /// []
    Bracket,
    /// ()
    Paren,
}

#[derive(Debug)]
struct Punct(char);

#[derive(Debug)]
enum TokenTree {
    Ident(Ident),
    Literal(Lit),
    Punct(Punct),
    Group(Bracket, Vec<TokenTree>),
}

#[derive(Debug)]
enum ParseState {
    Done(TokenTree),
    More,
    NoMatch,
}

fn to_tokens(code: String) -> Result<Vec<TokenTree>, String> {
    // let mut tokens = Vec::new();
    let mut chars = Stream::new(code);

    let mut group_stack = vec![(' ', Vec::new())];

    macro_rules! push_token {
        ($tok:expr) => {
            let el = group_stack.last_mut().unwrap();
            el.1.push($tok);
        };
    }

    // let mut push_token = |tok| {
    //    let el = group_stack.last_mut().unwrap();
    //    el.1.push(tok);
    // };

    'outer: while let Some(char) = chars.next() {
        match char {
            '"' => {
                let mut val = String::new();
                while let Some(char) = chars.next() {
                    if char == '"' {
                        push_token!(TokenTree::Literal(Lit::Str(val)));
                        continue 'outer;
                    }
                    val.push(char);
                }
                return Err(format!("unexpected end of file"));
            }
            '0'...'9' => {
                let mut val = String::new();
                val.push(char);
                let mut decimal = false;
                while let Some(char) = chars.peek() {
                    match char {
                        '0'...'9' => val.push(chars.next().unwrap()),
                        '.' if !decimal => val.push(chars.next().unwrap()),
                        '.' if decimal => return Err(format!("unexpected 2nd `.` in number")),
                        _ => {
                            let num: f32 = match val.parse() {
                                Ok(num) => num,
                                Err(_) => return Err(format!("could not parse number {:?}", val)),
                            };
                            push_token!(TokenTree::Literal(Lit::Num(num)));
                            continue 'outer;
                        }
                    }
                }
                return Err(format!("unexpected end of file"));
            }
            'a'...'z' | 'A'...'Z' | '_' => {
                let mut val = String::new();
                val.push(char);
                while let Some(char) = chars.peek() {
                    match char {
                        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => val.push(chars.next().unwrap()),
                        _ => {
                            if val == "true" || val == "false" {
                                push_token!(TokenTree::Literal(Lit::Bool(if val == "true" { true } else {false})));
                            } else {
                                push_token!(TokenTree::Ident(Ident(val)));
                            }
                            continue 'outer;
                        }
                    }
                }
            }
            '(' | '[' | '{' => group_stack.push((char, Vec::new())),
            ')' | ']' | '}' => {
                let (open, inner_tokens) = { group_stack.pop().unwrap() };
                if char == match open {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => {
                        return Err(format!(
                        "Unexpected close token `{:?}`, should be `{:?}` (empty = start of file)",
                        char, open
                    ))
                    }
                } {
                    push_token!(TokenTree::Group(
                        match open {
                            '(' => Bracket::Paren,
                            '[' => Bracket::Bracket,
                            '{' => Bracket::Brace,
                            _ => unreachable!(),
                        },
                        inner_tokens
                    ));
                } else {
                    return Err(format!(
                        "Unexpected close token `{:?}`, should be `{:?}` (empty = start of file)",
                        char, open
                    ));
                }
            }
            ' ' | '\n' => {},
            _ => {
                push_token!(TokenTree::Punct(Punct(char)));
            }
        }
    };

    let tokens = match group_stack.pop().unwrap() {
        (' ', tokens) => tokens,
        (br, _) => return Err(format!("unexpected end of file, group {:?} still open", br)),
    };

    Ok(tokens)
}

struct Stream {
    chars: Vec<char>,
    curr_pos: usize,
}

impl Stream {
    pub fn new(code: String) -> Self {
        Stream {
            chars: code.chars().collect(),
            curr_pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let curr = self.chars.get(self.curr_pos)?;
        self.curr_pos += 1;
        Some(*curr)
    }

    pub fn peek(&self) -> Option<char> {
        Some(*self.chars.get(self.curr_pos)?)
    }
}
