use std::ops::RangeInclusive;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, center, column, container, horizontal_space, row, scrollable, text, text_input, Column, Container, ProgressBar, Row};
use iced::window::{self, Settings};
use iced::{Alignment, Center, Element, Fill, Length, Padding, Size, Subscription, Task, Theme, Vector};


#[derive(Debug,Default)]
pub struct DownloadProgress {
    progress_id: usize,
    progress: f32,
    download_name:String,
}

impl DownloadProgress {
    pub fn view(&self) -> Element<super::Message> {

        let download_text = text(&self.download_name);
        let progress_bar = ProgressBar::new(RangeInclusive::new(0.0, 1.0), self.progress);
        let col = column![download_text, progress_bar];
        return  col.into();

    }
}
