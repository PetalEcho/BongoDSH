use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const FILE_NAME: &str = "keyboard-input-v1";
const MAX_BYTES: u64 = 32;
static NEXT_SAVE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum InputSource {
    #[default]
    Window,
    Global,
}

impl InputSource {
    pub fn should_listen_globally(self, paused: bool) -> bool {
        self == Self::Global && !paused
    }
}

pub(crate) fn path() -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let directory = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/BongoDSH"));
    #[cfg(target_os = "windows")]
    let directory =
        std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("BongoDSH"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let directory: Option<PathBuf> = None;
    directory
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join(FILE_NAME))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "无法确定本机设置目录"))
}

pub(crate) fn load(path: &Path) -> io::Result<InputSource> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(InputSource::Window),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    match bytes.as_slice() {
        b"window" | b"window\n" | b"window\r\n" => Ok(InputSource::Window),
        b"global" | b"global\n" | b"global\r\n" => Ok(InputSource::Global),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "键盘输入设置格式无效",
        )),
    }
}

pub(crate) fn save(path: &Path, source: InputSource) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "设置文件路径无效"))?;
    fs::create_dir_all(parent)?;
    let (temporary, mut file) = create_temporary(parent)?;
    let result = (|| {
        file.write_all(match source {
            InputSource::Window => b"window\n",
            InputSource::Global => b"global\n",
        })?;
        file.sync_all()?;
        // Windows requires our temporary handle to be closed before replacement.
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn create_temporary(parent: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..8 {
        let sequence = NEXT_SAVE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(
            ".{FILE_NAME}.{}.{sequence}.tmp",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "无法创建唯一的临时设置文件",
    ))
}
