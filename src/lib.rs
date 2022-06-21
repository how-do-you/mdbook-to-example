use std::io::{BufReader, Read, Write};

pub struct Builder {
    name: String,
    source: String,
    logo_url: String,
    as_comment: bool,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            name: "book".to_string(),
            source: "book".to_string(),
            logo_url: "".to_string(),
            as_comment: false,
        }
    }

    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn set_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }
    pub fn set_logo_url(mut self, logo_url: &str) -> Self {
        self.logo_url = logo_url.to_string();
        self
    }
    pub fn include_as_comment(mut self, as_comment: bool) -> Self {
        self.as_comment = as_comment;
        self
    }

    pub fn run(self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(format!("examples/{}", self.name))?;
        let mut main_rs = std::fs::File::create(format!("examples/{}/main.rs", self.name))?;
        let readme_md = std::fs::File::open("README.md")?;
        let mut reader = BufReader::new(readme_md);
        let mut contents = "".to_string();
        let _ = reader.read_to_string(&mut contents);

        let mut lines: Vec<String> = vec![];

        if self.logo_url != "" {
            lines.push(format!("#![doc(html_logo_url = \"{}\")]", self.logo_url));
        }

        let mut readme_lines: Vec<String> = contents.split("\n").map(|n| format!("//! {}", n)).collect();
        lines.append(&mut readme_lines);
        lines.push("\nfn main () {}\n".to_string());

        let summary_md = std::fs::File::open(format!("{}/src/SUMMARY.md", self.source))?;
        let mut reader = BufReader::new(summary_md);
        let mut summary_contents = "".to_string();
        let _ = reader.read_to_string(&mut summary_contents);
        let mut summary_lines: Vec<String> = summary_contents.split("\n")
            .filter(|n| n.contains("(") && n.contains("["))
            .map(|n| n.to_string())
            .collect();

        #[derive(Debug)]
        enum StringOrVec {
            String(String),
            Vec(Vec<StringOrVec>),
        }

        fn create_branch(lines: &mut Vec<String>, current_level: i32) -> Vec<StringOrVec> {
            let mut branch: Vec<StringOrVec> = vec![];
            while lines.len() > 0 {
                let value = lines.remove(0);

                let mut x: Vec<String> = value.split("").map(|n| n.to_string()).collect();
                let mut spaces: Vec<String> = vec![];
                if x[0] == "" {
                    x.remove(0);
                }
                while x[0] == " " {
                    spaces.push(x.remove(0));
                }

                let new_level = (spaces.len() / 2) as i32;

                if current_level == new_level {
                    branch.push(StringOrVec::String(value));
                } else if current_level < new_level {
                    lines.insert(0, value);
                    branch.push(StringOrVec::Vec(create_branch(lines, new_level)));
                } else if current_level > new_level {
                    lines.insert(0, value);
                    return branch;
                }
            }
            branch
        }
        let tree = create_branch(&mut summary_lines, 0);


        fn add_lines(lines: &mut Vec<String>, tree: Vec<StringOrVec>, source: String, as_comment: bool) -> Result<(), std::io::Error> {
            let mut level = 0;
            for entry in tree {
                match entry {
                    StringOrVec::String(s) => {
                        level += 1;
                        let m_header: Vec<String> = s.trim().split("[").map(|n| n.to_string()).collect();
                        let n_header: Vec<String> = m_header[1].split("]").map(|n| n.to_string()).collect();
                        let header = n_header[0].to_lowercase().replace(" ", "_");

                        let m_link: Vec<String> = s.trim().split("(").map(|n| n.to_string()).collect();
                        let link: Vec<String> = m_link[1].split(")").map(|n| n.to_string()).collect();


                        if as_comment {
                            println!("{}/src/{}", source, link[0]);
                            let source_file = std::fs::File::open(format!("{}/src/{}", source, link[0]))?;
                            let mut reader = BufReader::new(source_file);
                            let mut contents = "".to_string();
                            let _ = reader.read_to_string(&mut contents);
                            let mut source_lines: Vec<String> = contents.split("\n").map(|n| format!("/// {}", n)).collect();
                            lines.append(&mut source_lines);
                        } else {
                            lines.push(format!("#[doc = include_str!(\"../../{}/src/{}\")]", source.to_string(), link[0]));
                        }
                        lines.push(format!("pub mod s{}_{} {{}}", level, header));
                    }
                    StringOrVec::Vec(v) => {
                        let old = lines.remove(lines.len() - 1);
                        lines.push(format!("{}", old.strip_suffix("}").unwrap()));
                        let _ = add_lines(lines, v, source.to_string(), as_comment);
                        lines.push(format!("}}\n"));
                    }
                }
            }
            Ok(())
        }
        let _ = add_lines(&mut lines, tree, self.source, self.as_comment);


        lines.push("".to_string());
        contents = lines.join("\n");


        main_rs.write_all(contents.as_bytes())?;
        Ok(())
    }
}
