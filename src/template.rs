use std::borrow::Cow;

pub fn get_interrupt_pos_from_base(base: u8) -> usize {
    2 * 2_usize.pow(base as u32) + 2
}

pub struct Template<'a> {
    pub pieces: Vec<TemplatePiece<'a>>,
    pub current_code_section: Cow<'a, str>,
}

impl<'a> Template<'a> {
    pub fn new(string: &'a str, base: u8) -> Self {
        let mut pieces = Vec::new();
        let mut current_template = Vec::new();
        for i in string.lines() {
            if let Some(e) = i.strip_prefix("# header ") {
                pieces.push(TemplatePiece::Section(current_template));
                current_template = vec![];
                pieces.push(TemplatePiece::NamedSection(Cow::Borrowed(e), vec![]));
            } else {
                current_template.push(Cow::Borrowed(i));
            }
        }
        pieces.push(TemplatePiece::Section(current_template));

        let number_of_eles = 2_u64.pow(base as u32);

        let mut this = Self {
            pieces,
            current_code_section: Cow::Borrowed("CODE"),
        };

        this.add_section(
            "START",
            Cow::Owned(
                (0..number_of_eles)
                    .map(|_| "0")
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );

        this.add_section(
            "START",
            Cow::Owned(format!(
                "'#0:{nb_ele} {} '#null:0",
                (1..number_of_eles)
                    .map(|x| format!("'#{}:{}", x, x))
                    .collect::<Vec<_>>()
                    .join(" "),
                nb_ele = number_of_eles
            )),
        );

        this.add_section(
            "INTERRUPTS",
            Cow::Owned(
                (0..((7+base)/base +1))
                    .map(|x| format!("'#int_{}:0", x))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );

        this.add_section("V3_VAR_DEF", Cow::Borrowed("no_op = (1 1)"));
        this.add_section("V3_VAR_DEF", Cow::Borrowed("earasable = (999)"));
        this.add_section("V3_VAR_DEF", Cow::Borrowed("stop = (~+2 0 ~-2)"));

        this.add_section("V3_FCT_DEF", Cow::Borrowed("jump {~+2 0 self.0}"));
        this.add_section("V3_FCT_DEF", Cow::Borrowed("exit {self.0 '#int_0 stop}"));

        this.add_section("V3_FCT_DEF", 
        Cow::Owned(format!(
            "inc {{ self.0 'test {} 'test:earasable self.0 }}",
            (0..(number_of_eles))
                .rev()
                .map(|x| format!("'#{} {}", x,x.checked_sub(2).unwrap_or(x+number_of_eles -2)+1))
                .collect::<Vec<_>>()
                .join(" ")
        )));
        this.add_section("V3_FCT_DEF", 
        Cow::Owned(format!(
            "dec {{ self.0 'test {} 'test:earasable self.0 }}",
            (0..(number_of_eles))
                .rev()
                .map(|x| format!("'#{} {}", x,x+1))
                .collect::<Vec<_>>()
                .join(" ")
        )));
        this.add_section("V3_FCT_DEF", 
        Cow::Owned(format!(
            "if_0 {{ self.0 'test 'pt {} 1 'test:earasable 0 jump('end1) 'pt:self.1 'end:~+1 'end1:no_op }}",
            (1..(number_of_eles+1))
                .rev()
                .map(|x| format!("{} 'end", x))
                .collect::<Vec<_>>()
                .join(" ")
        )));
        
        
        // Cow::Borrowed("inc { self.0 'test '#15 14 '#14 13 '#13 12 '#12 11 '#11 10 '#10 9 '#9 8 '#8 7 '#7 6 '#6 5 '#5 4 '#4 3 '#3 2 '#2 1 '#1 16 '#0 15 'test:earasable self.0 }"));
        // this.add_section("V3_FCT_DEF", Cow::Borrowed("dec { self.0 'test '#15 16 '#14 15 '#13 14 '#12 13 '#11 12 '#10 11 '#9 10 '#8 9 '#7 8 '#6 7 '#5 6 '#4 5 '#3 4 '#2 3 '#1 2 '#0 1 'test:earasable self.0 }"));
        // this.add_section("V3_FCT_DEF", Cow::Borrowed("if_0 { self.0 'test 'pt 16 'end 15 'end 14 'end 13 'end 12 'end 11 'end 10 'end 9 'end 8 'end 7 'end 6 'end 5 'end 4 'end 3 'end 2 'end 1 'test:earasable 0 jump('end1) 'pt:self.1 'end:~+1 'end1:no_op }"));

        // println!("--- CODE ---\n{}", this.build());

        this
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
        None
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
        // retourne aussi la case ou ce situe les interrupts
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
