use egui::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub struct File {
    pub name: String,
}

impl File {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub struct Dir {
    pub name: String,
    pub path: PathBuf,
    pub dirs: Vec<Dir>,
    pub files: Vec<File>,
    pub modified: SystemTime,
}

impl Dir {
    pub fn new(name: impl Into<String>, path: PathBuf, modified: SystemTime) -> Self {
        Self {
            name: name.into(),
            path,
            dirs: Vec::new(),
            files: Vec::new(),
            modified,
        }
    }

    pub fn load(name: impl Into<String>, path: &Path) -> std::io::Result<Self> {
        let meta = std::fs::metadata(path)?;

        let mut dir = Dir::new(name, path.into(), meta.modified()?);

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let name = String::from(entry.path().file_name().unwrap().to_str().unwrap());
            let path = entry.path();

            if path.is_dir() {
                dir.dirs.push(Dir::load(name, &path)?);
            } else {
                dir.files.push(File::new(name));
            }
        }

        Ok(dir)
    }

    pub fn update(&mut self) -> std::io::Result<()> {
        let meta = std::fs::metadata(&self.path)?;

        if meta.modified()? > self.modified {
            for dir in &mut self.dirs {
                dir.update()?;
            }
        }

        Ok(())
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        for dir in &mut self.dirs {
            ui.collapsing(&dir.name.clone(), |ui| {
                dir.ui(ui);
            });
        }

        for file in &self.files {
            ui.label(&file.name);
        }
    }
}

pub struct Project {
    pub path: PathBuf,
    pub files: Dir,
}

impl Project {
    pub fn new(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();

        Ok(Self {
            files: Dir::load(".", &path)?,
            path,
        })
    }

    pub fn update_files(&mut self) -> std::io::Result<()> {
        self.files.update()?;

        Ok(())
    }
}
