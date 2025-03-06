use std::ops::RangeInclusive;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, center, column, container, horizontal_space, row, scrollable, text, text_input, Column, Container, ProgressBar, Row};
use iced::window::{self, Settings};
use iced::{Alignment, Center, Element, Fill, Length, Padding, Size, Subscription, Task, Theme, Vector};
use iced::futures::{SinkExt, Stream, StreamExt};
use omnissa_kblib::search::{SearchFilter, SearchResult};
use uuid::Uuid;

#[derive(Debug,Default,Clone)]
pub struct KbPage {
    //progress_id: usize,
    pub progress: f32,
    pub download_name:String,
    pub url:String,
    pub is_complete : bool,
    pub task_id: uuid::Uuid,
}

impl KbPage {
    pub fn new() -> Self {
        return KbPage{
            task_id:Uuid::new_v4(),
            progress : 0.0,
            download_name: String::new(),
            url:String::new(),
            is_complete:false
        };
    }
}
