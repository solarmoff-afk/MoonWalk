// Часть проекта MoonWalk с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 MoonWalk

#[cfg(feature = "video")]
use crate::rendering::video::MoonVideo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoFormat {
    #[default]
    Mp4, // H.264 (Cамый совместимый)
    Mov, // H.264 (Apple совместимый)
    Mkv, // H.264 (Matroska контейнер)
    
    // TODO
    Gif, // Анимация (RGB, без звука)
    Raw, // Несжатое видео (RawVideo), огромный размер, но без потерь (avi)
}

// TODO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoPreset {
    #[default]
    Balanced,
    HighQuality,
    LowLatency,
}

use crate::{MoonWalk, MoonWalkError};

impl MoonWalk {
    /// Создаёт новый видеорекордер. Он позволяет делать снапшоты из основного экрана,
    /// рендер контейнера или CustomPaint и добавлять их в видео, а потом экспортировать
    /// его на диск. Операция достаточно медленная что очевидно. Принимает ширину
    /// видео, высоту видео, частоту кадров видео, путь на диске к видео и формат
    /// видео. Формат видео нужно взять из перечисления VideoPreset, на выбор:
    /// Этот метод доступен только если есть фича "video". Требует наличия ffmpeg в системе
    /// поэтому на android его использование не тривиально. В метод нужно передавать
    /// чётные размеры видео чтобы избежать ошибки. Raw и Gif не работают на данный момент
    /// как и присеты. Возвращает экземпляр структуры
    /// MoonVideo у которого есть методы:
    ///
    ///  Добавить в видео кадр. Принимает экземпляр MoonWalk и айди текстуры
    ///  pub fn add_frame(&mut self, mw: &MoonWalk, texture_id: u32) -> Result<(), MoonWalkError> 
    ///
    ///  Завершить работу с видео закодировав его и сохранив его на диск
    ///  pub fn finish(mut self) -> Result<(), MoonWalkError> 
    #[cfg(feature = "video")]
    pub fn new_video_recorder(
        &self,
        width: u32,
        height: u32,
        fps: usize,
        path: &str,
        format: VideoFormat,
        preset: VideoPreset,
    ) -> Result<MoonVideo, MoonWalkError> {
        // Проверка чётности размеров (ffmpeg требует для yuv420p)
        if width % 2 != 0 || height % 2 != 0 {
             return Err(MoonWalkError::IOError("Video dimensions must be even numbers".to_string()));
        }
        
        MoonVideo::new(width, height, fps, path, format, preset)
    }
}
