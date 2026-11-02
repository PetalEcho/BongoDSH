use std::{
    fs::{self, File},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self, JoinHandle},
};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DIMENSION: u32 = 4096;
const TEXTURE_DIMENSION: u32 = 1024;
const SIGNATURE: &[u8] = b"BDSHIMG2";
const LEGACY_SIGNATURE: &[u8] = b"BDSHIMG1";
const HEADER_BYTES: usize = 22;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Part {
    #[default]
    Body = 0,
    LeftHand = 1,
    RightHand = 2,
}

impl Part {
    pub(crate) const ALL: [Self; 3] = [Self::Body, Self::LeftHand, Self::RightHand];

    pub(crate) fn label(self, chinese: bool) -> &'static str {
        match (self, chinese) {
            (Self::Body, true) => "身体",
            (Self::Body, false) => "Body",
            (Self::LeftHand, true) => "左手",
            (Self::LeftHand, false) => "Left hand",
            (Self::RightHand, true) => "右手",
            (Self::RightHand, false) => "Right hand",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Problem {
    Read,
    Write,
    Unsupported,
    TooLarge,
    Decode,
    Empty,
    InvalidSettings,
    Worker,
    NoDataDirectory,
    BodyRequired,
    MultipleFiles,
    Busy,
}

impl Problem {
    pub(crate) fn message(self, chinese: bool) -> &'static str {
        let (zh, en) = match self {
            Self::Read => (
                "无法读取图片，请检查文件及访问权限。",
                "Cannot read the image. Check the file and permissions.",
            ),
            Self::Write => (
                "角色已更新，但未能保存。请检查应用数据目录权限。",
                "Character updated, but not saved. Check app data folder permissions.",
            ),
            Self::Unsupported => (
                "请选择 PNG、JPEG 或 WebP 图片。",
                "Choose a PNG, JPEG or WebP image.",
            ),
            Self::TooLarge => (
                "图片不能超过 16 MB，每边不能超过 4096 像素。",
                "Use images up to 16 MB and 4096 pixels per side.",
            ),
            Self::Decode => (
                "图片损坏或无法解码，请换一张。原图片已保留。",
                "Cannot decode this image. Choose another. Your previous image is unchanged.",
            ),
            Self::Empty => (
                "图片完全透明，请选择有可见内容的图片。",
                "This image is fully transparent. Choose an image with visible content.",
            ),
            Self::InvalidSettings => (
                "本地角色设置无法读取，已恢复初始角色。",
                "Saved appearance could not be loaded. Using the original character.",
            ),
            Self::Worker => (
                "图片处理未完成，请重试。",
                "Image processing did not finish. Please try again.",
            ),
            Self::NoDataDirectory => (
                "未找到应用数据目录，本次选择仅在当前运行中有效。",
                "No app data folder is available. Changes last for this session only.",
            ),
            Self::BodyRequired => (
                "请先导入身体，再添加独立手部。",
                "Import a body before adding hand layers.",
            ),
            Self::MultipleFiles => (
                "每次拖入一张图片，先选择要替换的图层。",
                "Drop one image at a time. Select the layer you want to replace first.",
            ),
            Self::Busy => (
                "正在处理上一张图片，请稍后再试。",
                "An image is still being processed. Please try again shortly.",
            ),
        };
        if chinese { zh } else { en }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SavedAppearance {
    pub(crate) use_custom: bool,
    pub(crate) layers: [Vec<u8>; 3],
}

impl SavedAppearance {
    pub(crate) fn load(path: &Path) -> Result<Self, Problem> {
        let source = match fs::metadata(path) {
            Ok(_) => path.to_owned(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let legacy = path.with_file_name("appearance-v1.bin");
                if path
                    .file_name()
                    .is_some_and(|name| name == "appearance-v2.bin")
                    && legacy.exists()
                {
                    legacy
                } else {
                    return Ok(Self::default());
                }
            }
            Err(_) => return Err(Problem::InvalidSettings),
        };
        let bytes = read_bounded(&source, MAX_FILE_BYTES * 3 + HEADER_BYTES as u64)
            .map_err(|_| Problem::InvalidSettings)?;
        Self::from_bytes(&bytes)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Problem> {
        if bytes.starts_with(LEGACY_SIGNATURE) {
            if bytes.len() < 10
                || bytes[8] > 1
                || bytes[9] > 1
                || bytes.len() as u64 > MAX_FILE_BYTES + 10
            {
                return Err(Problem::InvalidSettings);
            }
            let saved = Self {
                use_custom: bytes[8] == 1,
                layers: [bytes[10..].to_vec(), Vec::new(), Vec::new()],
            };
            // Legacy synthetic hands are deliberately not migrated.
            return saved.validate();
        }
        if bytes.len() < HEADER_BYTES
            || !bytes.starts_with(SIGNATURE)
            || bytes[8] > 1
            || bytes[9] != 0
        {
            return Err(Problem::InvalidSettings);
        }
        let mut saved = Self {
            use_custom: bytes[8] == 1,
            ..Self::default()
        };
        let mut offset = HEADER_BYTES;
        for (index, layer) in saved.layers.iter_mut().enumerate() {
            let length = u32::from_le_bytes(
                bytes[10 + index * 4..14 + index * 4]
                    .try_into()
                    .map_err(|_| Problem::InvalidSettings)?,
            ) as usize;
            if length as u64 > MAX_FILE_BYTES || length > bytes.len().saturating_sub(offset) {
                return Err(Problem::InvalidSettings);
            }
            *layer = bytes[offset..offset + length].to_vec();
            offset += length;
        }
        if offset != bytes.len() {
            return Err(Problem::InvalidSettings);
        }
        saved.validate()
    }

    fn validate(self) -> Result<Self, Problem> {
        if (self.use_custom || !self.layers[1].is_empty() || !self.layers[2].is_empty())
            && self.layers[0].is_empty()
        {
            return Err(Problem::InvalidSettings);
        }
        Ok(self)
    }

    pub(crate) fn save(&self, path: &Path) -> Result<(), Problem> {
        let parent = path.parent().ok_or(Problem::Write)?;
        fs::create_dir_all(parent).map_err(|_| Problem::Write)?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        let result = (|| {
            let mut file = File::create(&temporary)?;
            file.write_all(SIGNATURE)?;
            file.write_all(&[u8::from(self.use_custom), 0])?;
            for layer in &self.layers {
                file.write_all(&(layer.len() as u32).to_le_bytes())?;
            }
            for layer in &self.layers {
                file.write_all(layer)?;
            }
            file.sync_all()?;
            fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result.map_err(|_: std::io::Error| Problem::Write)
    }
}

pub(crate) struct PreparedImage {
    pub(crate) color: ColorImage,
    pub(crate) png: Vec<u8>,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<PreparedImage, Problem> {
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(Problem::TooLarge);
    }
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| Problem::Decode)?;
    if !matches!(
        reader.format(),
        Some(ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP)
    ) {
        return Err(Problem::Unsupported);
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(image_problem)?;
    let (width, height) = decoder.dimensions();
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(Problem::TooLarge);
    }
    let orientation = decoder.orientation().map_err(image_problem)?;
    let mut decoded = DynamicImage::from_decoder(decoder).map_err(image_problem)?;
    decoded.apply_orientation(orientation);
    if decoded.width() > TEXTURE_DIMENSION || decoded.height() > TEXTURE_DIMENSION {
        decoded = decoded.thumbnail(TEXTURE_DIMENSION, TEXTURE_DIMENSION);
    }
    let rgba = trim_transparent(&decoded.into_rgba8())?;
    let color = ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    );
    let mut png = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(rgba)
        .write_to(&mut png, ImageFormat::Png)
        .map_err(|_| Problem::Decode)?;
    Ok(PreparedImage {
        color,
        png: png.into_inner(),
    })
}

fn image_problem(error: image::ImageError) -> Problem {
    if matches!(error, image::ImageError::Limits(_)) {
        Problem::TooLarge
    } else {
        Problem::Decode
    }
}

pub(crate) fn import(path: &Path) -> Result<PreparedImage, Problem> {
    decode(&read_bounded(path, MAX_FILE_BYTES)?)
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, Problem> {
    let metadata = fs::metadata(path).map_err(|_| Problem::Read)?;
    if !metadata.is_file() {
        return Err(Problem::Unsupported);
    }
    if metadata.len() > limit {
        return Err(Problem::TooLarge);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| Problem::Read)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Problem::Read)?;
    if bytes.len() as u64 > limit {
        return Err(Problem::TooLarge);
    }
    Ok(bytes)
}

enum Completed {
    Loaded(SavedAppearance, [Option<ColorImage>; 3]),
    Imported(Part, PreparedImage),
    Saved,
}

struct Pending {
    receiver: Receiver<Result<Completed, Problem>>,
    thread: JoinHandle<()>,
}

pub(crate) struct Appearance {
    saved: SavedAppearance,
    custom: [Option<TextureHandle>; 3],
    path: Option<PathBuf>,
    pending: Option<Pending>,
    dirty: bool,
    pub(crate) problem: Option<Problem>,
}

impl Appearance {
    pub(crate) fn new(ctx: &egui::Context, path: Option<PathBuf>) -> Self {
        let mut appearance = Self {
            saved: SavedAppearance::default(),
            custom: [None, None, None],
            path,
            pending: None,
            dirty: false,
            problem: None,
        };
        if let Some(path) = appearance.path.clone() {
            appearance.start(ctx, move || {
                let saved = SavedAppearance::load(&path)?;
                let mut colors = [None, None, None];
                for (index, bytes) in saved.layers.iter().enumerate() {
                    if !bytes.is_empty() {
                        colors[index] =
                            Some(decode(bytes).map_err(|_| Problem::InvalidSettings)?.color);
                    }
                }
                Ok(Completed::Loaded(saved, colors))
            });
        }
        appearance
    }

    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub(crate) fn uses_custom(&self) -> bool {
        self.saved.use_custom
    }
    pub(crate) fn has_custom(&self) -> bool {
        self.custom[0].is_some()
    }
    pub(crate) fn custom_layer(&self, part: Part) -> Option<&TextureHandle> {
        self.custom[part as usize].as_ref()
    }

    pub(crate) fn layers(&self) -> [Option<&TextureHandle>; 3] {
        if self.saved.use_custom {
            self.custom.each_ref().map(Option::as_ref)
        } else {
            [None, None, None]
        }
    }

    pub(crate) fn select_default(&mut self, ctx: &egui::Context) {
        if self.busy() {
            return;
        }
        self.saved.use_custom = false;
        self.save(ctx);
    }

    pub(crate) fn select_custom(&mut self, ctx: &egui::Context) {
        if self.busy() || !self.has_custom() {
            return;
        }
        self.saved.use_custom = true;
        self.save(ctx);
    }

    pub(crate) fn remove(&mut self, part: Part, ctx: &egui::Context) {
        if self.busy() {
            return;
        }
        self.custom[part as usize] = None;
        self.saved.layers[part as usize].clear();
        if part == Part::Body {
            self.custom = [None, None, None];
            self.saved.layers = Default::default();
            self.saved.use_custom = false;
        }
        self.save(ctx);
    }

    pub(crate) fn import(&mut self, path: PathBuf, part: Part, ctx: &egui::Context) {
        if self.busy() {
            self.problem = Some(Problem::Busy);
            return;
        }
        if part != Part::Body && !self.has_custom() {
            self.problem = Some(Problem::BodyRequired);
            return;
        }
        self.problem = None;
        self.start(ctx, move || {
            import(&path).map(|image| Completed::Imported(part, image))
        });
    }

    fn start(
        &mut self,
        ctx: &egui::Context,
        work: impl FnOnce() -> Result<Completed, Problem> + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        match thread::Builder::new()
            .name("bongodsh-image".to_owned())
            .spawn(move || {
                let _ = sender.send(work());
                ctx.request_repaint_of(egui::ViewportId::ROOT);
            }) {
            Ok(thread) => self.pending = Some(Pending { receiver, thread }),
            Err(_) => self.problem = Some(Problem::Worker),
        }
    }

    fn save(&mut self, ctx: &egui::Context) {
        self.dirty = true;
        self.problem = None;
        let Some(path) = self.path.clone() else {
            self.problem = Some(Problem::NoDataDirectory);
            return;
        };
        let saved = self.saved.clone();
        self.start(ctx, move || saved.save(&path).map(|()| Completed::Saved));
    }

    pub(crate) fn poll(&mut self, ctx: &egui::Context) -> bool {
        let Some(pending) = &self.pending else {
            return false;
        };
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => Err(Problem::Worker),
        };
        self.pending = None;
        match result {
            Ok(Completed::Loaded(saved, colors)) => {
                self.saved = saved;
                self.custom = colors.map(|color| {
                    color.map(|image| {
                        ctx.load_texture("custom-layer", image, TextureOptions::LINEAR)
                    })
                });
            }
            Ok(Completed::Imported(part, prepared)) => {
                self.custom[part as usize] =
                    Some(ctx.load_texture("custom-layer", prepared.color, TextureOptions::LINEAR));
                self.saved.layers[part as usize] = prepared.png;
                self.saved.use_custom = true;
                self.save(ctx);
            }
            Ok(Completed::Saved) => self.dirty = false,
            Err(problem) => self.problem = Some(problem),
        }
        ctx.request_repaint();
        true
    }

    // Join pending work before exit so a background save cannot overwrite the final record.
    pub(crate) fn flush(&mut self) -> Result<(), Problem> {
        if let Some(pending) = self.pending.take() {
            pending.thread.join().map_err(|_| Problem::Worker)?;
        }
        if self.dirty {
            let path = self.path.as_ref().ok_or(Problem::NoDataDirectory)?;
            self.saved.save(path)?;
            self.dirty = false;
        }
        Ok(())
    }
}

pub(crate) fn data_path() -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("BONGODSH_DATA_DIR") {
        let directory = PathBuf::from(directory);
        return directory
            .is_absolute()
            .then(|| directory.join("appearance-v2.bin"));
    }
    #[cfg(target_os = "macos")]
    let directory = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/BongoDSH"));
    #[cfg(target_os = "windows")]
    let directory =
        std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("BongoDSH"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let directory: Option<PathBuf> = None;
    directory.map(|directory| directory.join("appearance-v2.bin"))
}

fn trim_transparent(image: &image::RgbaImage) -> Result<image::RgbaImage, Problem> {
    let (mut left, mut top) = image.dimensions();
    let (mut right, mut bottom) = (0, 0);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[3] > 0 {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    if right <= left || bottom <= top {
        return Err(Problem::Empty);
    }
    Ok(image::imageops::crop_imm(image, left, top, right - left, bottom - top).to_image())
}
