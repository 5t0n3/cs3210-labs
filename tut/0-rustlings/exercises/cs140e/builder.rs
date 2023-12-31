// FIXME: Make me pass! Diff budget: 30 lines.

#[derive(Default)]
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn string<S: ToString>(self, s: S) -> Self {
        Self {
            string: Some(s.to_string()),
            ..self
        }
    }

    fn number(self, value: usize) -> Self {
        Self {
            number: Some(value),
            ..self
        }
    }
}

impl ToString for Builder {
    // Implement the trait
    fn to_string(&self) -> String {
        let mut res = String::new();
        let mut include_space = false;

        if let Some(s) = &self.string {
            res += s;
            include_space = true;
        }

        if let Some(n) = self.number {
            if include_space {
                res += " ";
            }
            res += &n.to_string();
        }

        res
    }
}

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default().string("heap!".to_owned()).to_string();

    assert_eq!(c, "heap!");
}
