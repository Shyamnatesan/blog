use std::fmt::Write;
use std::{ffi::OsStr, fs, path::PathBuf};

const OUTPUT_DIR: &str = "./dist";
const INPUT_DIR: &str = "./archive";

#[derive(Debug)]
pub enum AppError {
    ArchiveEmpty,
    IoError(()),
}

impl From<std::io::Error> for AppError {
    fn from(_value: std::io::Error) -> Self {
        AppError::IoError(())
    }
}

fn get_archives(path: &str) -> Result<Vec<PathBuf>, AppError> {
    let dir_iter = fs::read_dir(path)?;

    let mut paths = Vec::new();

    for dir_entry in dir_iter.flatten() {
        paths.push(dir_entry.path());
    }

    Ok(paths)
}

fn process(path: &PathBuf, fallback_filename: &OsStr) -> Result<PathBuf, AppError> {
    let content = fs::read_to_string(path)?;
    let events = jotdown::Parser::new(&content);
    let body = jotdown::html::render_to_string(events);

    let file_name = if let Some(stem) = path.file_stem() {
        stem
    } else {
        fallback_filename
    };

    let html = format!(
        r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{}</title>
        <link rel="stylesheet" href="style.css">
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
    </head>
    <body>
        <div class="container">
            <nav>
                <a href="index.html">← Home</a>
            </nav>
            {}
        </div>
        <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
        <script>hljs.highlightAll();</script>
    </body>
    </html>"#,
        file_name.to_string_lossy(),
        body
    );

    let mut output_path = PathBuf::from(OUTPUT_DIR);
    output_path.push(file_name);
    output_path.set_extension("html");

    fs::write(&output_path, html)?;

    Ok(output_path)
}

fn create_index(output_paths: &[PathBuf]) -> Result<(), AppError> {
    let mut items = String::new();

    for path in output_paths {
        let title = path.file_stem().map_or_else(
            || "Untitled".to_string(),
            |s| s.to_string_lossy().to_string(),
        );

        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let date = fs::metadata(path)
            .and_then(|m| m.created().or_else(|_| m.modified()))
            .map_or_else(
                |_| "Unknown".to_string(),
                |t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%d %b %Y").to_string()
                },
            );

        let _ = write!(
            items,
            r#"<a href="{file_name}" class="entry">
            <span class="entry-title">{title}</span>
            <span class="entry-date">{date}</span>
        </a>"#
        );
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Archive</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>Archive</h1>
            <p>{} entries</p>
        </header>
        <div class="entries">
            {}
        </div>
    </div>
</body>
</html>"#,
        output_paths.len(),
        items
    );

    let index_path = PathBuf::from(OUTPUT_DIR).join("index.html");
    fs::write(index_path, html)?;

    Ok(())
}

pub fn generate() -> Result<(), AppError> {
    let paths = get_archives(INPUT_DIR)?;
    let fallback_filename = OsStr::new("Untitled");

    if paths.is_empty() {
        return Err(AppError::ArchiveEmpty);
    }

    fs::create_dir_all(OUTPUT_DIR)?;

    let mut output_paths: Vec<PathBuf> = Vec::new();
    for path in paths {
        if let Ok(output_path) = process(&path, fallback_filename) {
            output_paths.push(output_path);
        }
    }

    create_index(&output_paths)?;

    fs::copy(
        "./static/style.css",
        PathBuf::from(OUTPUT_DIR).join("style.css"), // uses OUTPUT_DIR constant
    )?;

    Ok(())
}
