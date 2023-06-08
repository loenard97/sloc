use std::collections::HashMap;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};
use std::path::{PathBuf, Path};
use std::vec;

#[derive(Debug)]
struct GitIgnore {
    ignored_files: Vec<PathBuf>,
}

impl GitIgnore {
    fn new() -> Self {
        Self { ignored_files: vec![] }
    }

    fn add_file(&mut self, path: PathBuf) {
        println!("add_file {:?}", path.file_name());
        let file = match File::open(&path) {
            Ok(val) => val,
            Err(_) => return,
        };
        println!("add_file {:?}", path.file_name());
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();

            self.ignored_files.push(PathBuf::from(line));
        }
    }
}

#[derive(Debug)]
enum Language {
    Rust,
    Python,

    Unknown,
}

impl Language {
    fn from_path(path: PathBuf) -> Self {
        let ext = path.file_name().unwrap().to_str().unwrap();
        let split = ext.split(".");
        let ext = split.last().unwrap();

        match ext {
            "rs"    => Language::Rust,
            "py"    => Language::Python,
            _       => Language::Unknown,
        }
    }

    fn name(&self) -> &str {
        match self {
            Language::Rust      => "Rust",
            Language::Python    => "Python",
            Language::Unknown   => "Unknown",
        }
    }

    fn file_extension(&self) -> &str {
        match self {
            Language::Rust      => "rs",
            Language::Python    => "py",
            Language::Unknown   => "",
        }
    }
}

#[derive(Debug)]
struct FileInfo {
    language: Language,
    nlines: usize,
    nsloc: usize,
}

impl FileInfo {
    fn new(path: PathBuf) -> Self {
        let file = File::open(&path).unwrap();
        let reader = BufReader::new(file);

        let language = Language::from_path(path);

        let mut nlines = 0;
        let mut nsloc = 0;

        for line in reader.lines() {
            let line = match line {
                Ok(val) => val,
                Err(_) => continue,
            };

            nlines += 1;

            if line != "" {
                nsloc += 1;
            }

        }

        FileInfo { language, nlines, nsloc }
    }
}

#[derive(Debug)]
struct ProjectInfo {
    files: Vec<FileInfo>,
    git_ignore: GitIgnore,
}

impl ProjectInfo {
    fn new(path: PathBuf) -> Self {
        let mut git_ignore = GitIgnore::new();

        let mut files = Vec::new();
        files.append(&mut ProjectInfo::_parse_dir(path, &mut git_ignore));

        println!("{:?}", git_ignore);

        ProjectInfo { files, git_ignore }
    }

    fn _parse_dir(path: PathBuf, git_ignore: &mut GitIgnore) -> Vec<FileInfo> {

        let dirs = read_dir(&path).unwrap();

        let git_path = path.file_name().unwrap_or_default().to_str().unwrap();

        let mut git_path = String::from(git_path);
        git_path.push_str("/.gitignore");

        let git_path = PathBuf::from(git_path);
        println!("git path {:?}", git_path);
        let exists = Path::new(&git_path).exists();

        println!("exists {}", exists);

        git_ignore.add_file(git_path);

        let mut files = Vec::new();

        for item in dirs {
            let item = item.unwrap();

            let file_name = item.file_name();
            let file_name = file_name.to_str().unwrap();

            if file_name.starts_with(".") {
                // println!("skipping {:?}", item);
                continue;
            }

            if item.path().is_file() {

                // if !file_name.ends_with(".rs") {
                    // println!("skipping {:?}", item);
                //     continue;
                // }

                files.push(FileInfo::new(item.path()));
            } else if item.path().is_dir() {
                files.append(&mut ProjectInfo::_parse_dir(item.path(), git_ignore));
            }
        }

        files
    }

    fn to_string(&self) -> String {
        let mut ret = String::new();

        let mut map: HashMap<&str, (usize, usize, usize)> = HashMap::new();

        for file in &self.files {

            let (n_files, n_lines, n_sloc) = map.get(file.language.name())
                .unwrap_or_else(|| { &(0, 0, 0) });

            map.insert(file.language.name(), (n_files + 1, n_lines + file.nlines, n_sloc + file.nsloc));
        }

        ret.push_str("Language  Nr Files    Nr Lines    Nr SLOC\n");

        for (k, v) in map {
            ret.push_str(&format!("{}:     {}           {}         {}\n", k, v.0, v.1, v.2));
        }

        ret
    }
}

fn main() {
    let file_path = ".".to_string();

    let path = PathBuf::from(file_path);

    let fi = ProjectInfo::new(path);

    println!("{}", fi.to_string());
}
