use std::borrow::Cow;

pub struct Template<'a> {
    pub pieces: Vec<TemplatePiece<'a>>,
    pub current_code_section: Cow<'a, str>,
}

impl<'a> Template<'a> {
    pub fn new(string: &'a str) -> Self {
        let mut pieces = Vec::new();
        let mut current_template = Vec::new();
        for i in string.lines() {
            if i.starts_with("# header ") {
                pieces.push(TemplatePiece::Section(current_template));
                current_template = vec![];
                pieces.push(TemplatePiece::NamedSection(
                    Cow::Borrowed(&i["# header ".len()..]),
                    vec![],
                ));
            } else {
                current_template.push(Cow::Borrowed(i));
            }
        }
        pieces.push(TemplatePiece::Section(current_template));
        Self {
            pieces,
            current_code_section: Cow::Borrowed("CODE"),
        }
    }

    pub fn section_contains(&self, section: &str, needle: &str) -> bool {
        if let Some(e) = self.get_section(section) {
            e.iter().any(|x| x.contains(needle))
        } else {
            false
        }
    }

    pub fn get_section(&self, section: &'a str) -> Option<&Vec<Cow<'a, str>>> {
        for i in self.pieces.iter() {
            match i {
                TemplatePiece::Section(_) => (),
                TemplatePiece::NamedSection(a, b) => {
                    if a == section {
                        return Some(b);
                    }
                }
            }
        }
        return None;
    }

    pub fn set_code_section(&mut self, section: Cow<'a, str>) {
        self.current_code_section = section;
    }

    pub fn add_code(&mut self, string: Cow<'a, str>) {
        for i in self.pieces.iter_mut() {
            match i {
                TemplatePiece::Section(_) => (),
                TemplatePiece::NamedSection(a, b) => {
                    if a == &self.current_code_section {
                        b.push(string);
                        break;
                    }
                }
            }
        }
    }

    pub fn add_section(&mut self, section: &'a str, string: Cow<'a, str>) {
        for i in self.pieces.iter_mut() {
            match i {
                TemplatePiece::Section(_) => (),
                TemplatePiece::NamedSection(a, b) => {
                    if a == section {
                        b.push(string);
                        break;
                    }
                }
            }
        }
    }

    pub fn build(&self) -> String {
        self.pieces
            .iter()
            .map(|x| match x {
                TemplatePiece::Section(a) | TemplatePiece::NamedSection(_, a) => a.join("\n"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn apply(&mut self, a: &impl Instruction) {
        a.apply(self)
    }
}

pub trait Instruction {
    fn apply(&self, template: &mut Template);
}

pub enum TemplatePiece<'a> {
    Section(Vec<Cow<'a, str>>),
    NamedSection(Cow<'a, str>, Vec<Cow<'a, str>>),
}
