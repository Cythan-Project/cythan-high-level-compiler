use crate::{
    compiler::{scope::ScopedState, state::State},
    execute_file,
};

#[derive(serde::Serialize, serde::Deserialize)]
struct Test {
    file: String,
    input: Option<String>,
    output: Option<String>,
    error: Option<String>,
    base: Option<u8>,
}

impl Test {
    fn run_test(&self) {
        let mut state = State::default();
        state.base = self.base.unwrap_or(4);
        let mut scope = ScopedState::new();

        if let Err(e) = execute_file(
            &format!("src/tests/cythan_tests/{}.ct1", self.file),
            &mut state,
            &mut scope,
            vec![]
        ) {
            assert_eq!(
                e.to_string().replace(" ", "").replace("\n", ""),
                self.error
                    .clone()
                    .unwrap_or_default()
                    .replace(" ", "")
                    .replace("\n", "")
            );
            return;
        }
        match crate::compile_and_run(
            &state,
            self.input.clone().unwrap_or_default().chars().collect(),
        ) {
            Ok(e) => {
                assert_eq!(e, self.output.clone().unwrap_or_default());
            }
            Err(e) => {
                assert_eq!(
                    e.to_string().replace(" ", "").replace("\n", ""),
                    self.error
                        .clone()
                        .unwrap_or_default()
                        .replace(" ", "")
                        .replace("\n", "")
                );
            }
        }
    }
}

#[test]
fn run_cythan_tests() {
    for i in std::fs::read_dir("src/tests/cythan_tests").unwrap() {
        let i = i.unwrap();
        if i.file_name().to_str().unwrap().ends_with(".json") {
            let k: Vec<Test> =
                serde_json::from_str(&std::fs::read_to_string(i.path()).unwrap()).unwrap();
            let mut o = 0;
            for n in &k {
                n.run_test();
                println!("Done {}. ({}%)", n.file, o * 100 / k.len());
                o += 1;
            }
        }
    }
}
