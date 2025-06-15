use std::io;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    process::exit,
    error::Error,
};
use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

type MyResult<T> = std::result::Result<T, Box<dyn Error>>;

const CONFIG_PATH: &str = ".rs-clone.conf";
const VIDEO_EXTS: [&str; 7] = ["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm"];
const SUBTITLE_EXTS: [&str; 5] = ["srt", "ass", "vtt", "sub", "ssa"];


fn main() -> RlResult<()> {
    let mut config = Config::read_file(CONFIG_PATH).unwrap_or_else(|e| {
        eprintln!("Error reading config ({}):\n{}", CONFIG_PATH, e);
        exit(1);
    });

    let mut source_files = read_filenames(&config.settings.source_dir)
        .unwrap_or_else(|_| {
            eprintln!("Failed to read Source Directory");
            exit(1);
        });

    if source_files.is_empty() {
        println!("No Source Files, Quitting...");
        exit(0);
    }

    source_files.sort();

    let file_menu = source_files
        .iter()
        .map(|f| {
            if config.mapping.contains_key(f) {
                let map = config.mapping[f].to_string();
                if &map != f {
                    format!("*{} ({})", f, map)
                }else{
                    format!("*{}", f)
                }
            } else {
                f.clone()
            }
        })
        .collect::<Vec<String>>();

    let selection: u32 = stdin_input(&file_menu)?;
    if selection == 0 {
        println!("Bye.");
        exit(0);
    }

    let selection_dir = source_files
        .get(selection as usize - 1)
        .expect("Invalid Selection");

    println!("{}\n", selection_dir);

    let copy_src = Path::new(&config.settings.source_dir).join(selection_dir);
    let mut copy_dest = Path::new(&config.settings.destination_dir).to_path_buf();

    if !config.mapping.contains_key(selection_dir) {
        let suggestion = clean_filename(selection_dir);

        let mut rl = DefaultEditor::new()?;
        let user_input = rl
            .readline_with_initial("Destination: ", (&suggestion, ""))
            .unwrap_or_default();

        println!("{}\n", user_input);

        config.mapping.insert(selection_dir.to_string(), user_input.clone());
        config.write_file(CONFIG_PATH).expect("Failed to update config. (No files have been copied)");

        copy_dest = copy_dest.join(&user_input);
    }else{
       let suggestion = config.mapping[selection_dir].to_string();

        let mut rl = DefaultEditor::new()?;
        let user_input = rl
            .readline_with_initial("Destination: ", (&suggestion, ""))
            .unwrap_or_default();

        copy_dest = copy_dest.join(&user_input);

        if suggestion != user_input {
            let mv_src = Path::new(&config.settings.destination_dir).join(suggestion);

            println!("{} -> {}", mv_src.to_string_lossy(), copy_dest.to_string_lossy());
            fs::rename(
                mv_src,
                &copy_dest,
            )?;

            println!("(Move Successful)\n");

            config.mapping.remove(selection_dir);
            config.mapping.insert(selection_dir.to_string(), user_input.clone());
            config.write_file(CONFIG_PATH).expect("Failed to update config. (No files have been copied)");
        }
    }

    println!("Contents:");
    list_folder_contents(&copy_src).expect("Failed to read source.").iter().for_each(|f| println!(" - {}", f));
    println!("\nSelect Extensions to copy:");

    let options = [
        format!("Video     - [{}]", VIDEO_EXTS.join(", ")), 
        format!("Subtitles - [{}]", SUBTITLE_EXTS.join(", ")), 
        format!("Both      - [{}, {}]", VIDEO_EXTS.join(", "), SUBTITLE_EXTS.join(", ")), 
        "Any       - [*] (DEFAULT)".to_string()];
    let selection: u32 = stdin_input(&options)?;
    
    println!("{} -> {}", copy_src.to_string_lossy(), copy_dest.to_string_lossy());

    filtered_clone_dir(
        &copy_src,
        &copy_dest,
        match selection {
            1 => |ext: &str| VIDEO_EXTS.contains(&ext),
            2 => |ext: &str| SUBTITLE_EXTS.contains(&ext),
            3 => |ext: &str| VIDEO_EXTS.contains(&ext) || SUBTITLE_EXTS.contains(&ext),
            _ => |_ext: &str| true, 
        }
    )?;
  
    
    println!("Copy Success!");

    Ok(())
}

fn list_folder_contents(dir: &Path) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();

    for entry_result in fs::read_dir(dir)? {
        let entry = entry_result?;
        if let Some(name) = entry.file_name().to_str() {
            entries.push(name.to_string());
        }
    }

    Ok(entries)
}

fn stdin_input(options: &[String]) -> RlResult<u32> {
    for (i, opt) in options.iter().enumerate() {
        println!("{}: {}", i + 1, opt);
    }

    let mut rl = DefaultEditor::new()?;

    loop {
        match rl.readline("Enter choice number: ") {
            Ok(line) => {
                let choice: u32 = line.trim().parse().unwrap_or(0);
                if choice == 0 || choice as usize > options.len() {
                    return Ok(0);
                }
                return Ok(choice);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                return Ok(0);
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                return Ok(0);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                return Ok(0);
            }
        }
    }
}

fn read_filenames(dir: &str) -> std::io::Result<Vec<String>> {
    let mut filenames = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                filenames.push(name.to_string());
            }
        }
    }
    Ok(filenames)
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    settings: Settings,
    mapping: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    source_dir: String,
    destination_dir: String,
}

impl Config {
    fn read_file(path: &str) -> MyResult<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.is_valid()?;
        Ok(config)
    }

    fn write_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_str = toml::to_string_pretty(self)?;
        fs::write(path, toml_str)?;
        Ok(())
    }

    fn is_valid(&self) -> MyResult<()> {
        if self.settings.source_dir.is_empty() {
            return Err("settings.source_dir is empty".into());
        }
        if !Path::new(&self.settings.source_dir).exists() {
            return Err("settings.source_dir doesn't exist".into());
        }
        if self.settings.destination_dir.is_empty() {
            return Err("settings.destination_dir is empty".into());
        }
        if !Path::new(&self.settings.destination_dir).exists() {
            return Err("settings.destination_dir doesn't exist".into());
        }
        Ok(())
    }
}

fn clean_filename(name: &str) -> String {
    let re = Regex::new(r"(?i)^([a-z0-9 ._-]+?)[ ._-]*(?:19|20)\d{2}").unwrap();
    let name = name.replace(['.', '_'], " ");
    if let Some(caps) = re.captures(&name) {
        caps[1]
            .trim()
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        name.trim()
            .split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn filtered_clone_dir<F>(src: &Path, dst: &Path, is_allowed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
    for entry in WalkDir::new(src) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let is_video = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| is_allowed(&ext.to_ascii_lowercase()))
            .unwrap_or(false);

        if !is_video {
            continue;
        }

        let rel_path = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(rel_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &target)?;
    }
    Ok(())
}
