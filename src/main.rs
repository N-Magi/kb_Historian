
use download_progress::DownloadProgress;
use iced::advanced::graphics::futures::backend::default;
use iced::advanced::Text;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    button, center, column, container, horizontal_space, row, scrollable, text, text_input, Column, Container, Row,};
use iced::window::{self, Settings};
use iced::{Alignment, Center, Element, Fill, Length, Padding, Size, Subscription, Task, Theme, Vector};

use iced_aw::sidebar::row;
use iced_aw::{date_picker, iced_fonts};
use iced_aw::date_picker::Date;
use tokio::task::futures::TaskLocalFuture;
use chrono::{self, Local};

pub mod download_progress;
//use download_progress;

fn main() -> iced::Result  {
    iced::application("test", Page::update, Page::view,).font(iced_fonts::REQUIRED_FONT_BYTES).window_size(Size::new(360.0,360.0)).run()
}

#[derive(Debug, Default)]
struct Page {

    start_date:Date,
    spicker_show:bool,
    end_date :Date,
    epicker_show:bool,
    downloads:Vec<DownloadProgress>,
    theme: Theme,
    
}

#[derive(Debug, Clone)]
enum Message {
    EnterStartDateBtnPress,
    StartDateCancelled,
    StartDateSumit(Date),
    EnterEndDateBtnPress,
    EndDateCancelled,
    EndDateSumit(Date),
    Search,
    Execute,


}

impl Page {

    fn new() -> Self {

        todo!()
    }

    fn update(&mut self, message: Message) -> Task<Message> {

        match message {

            Message::EnterStartDateBtnPress => {
                self.spicker_show = !self.spicker_show;
                return Task::none();
            }   

            Message::EnterEndDateBtnPress => {
                self.epicker_show = !self.epicker_show;
                return Task::none();
            }

            Message::EndDateCancelled => {
                self.epicker_show = false;
                return  Task::none();
            }

            Message::StartDateCancelled => {
                self.spicker_show = false;
                return  Task::none();
            }


            Message::EndDateSumit(date) => {
                self.epicker_show = false;
                self.end_date = date;
                return  Task::none();
            }
            Message::StartDateSumit(date) => {
                self.spicker_show = false;
                self.start_date = date;

                return  Task::none();
            }

            default => {
                return  Task::none();
            }
        }
        todo!()
    }

    fn view(&self) -> Element<Message> {
    
        let input_start_date = button(text(self.start_date.to_string())).on_press(Message::EnterStartDateBtnPress);
        let start_dpicker = date_picker(self.spicker_show, self.start_date, input_start_date, Message::StartDateCancelled, Message::StartDateSumit);

        let input_end_date = button(text(self.end_date.to_string())).on_press(Message::EnterEndDateBtnPress);
        let end_dpicker = date_picker(self.epicker_show, self.end_date, input_end_date, Message::EndDateCancelled, Message::EndDateSumit);

        let search_button = button(text("Search")).on_press(Message::Search).width(Length::Fill);

        let list = Column::with_children(self.downloads.iter().map(|prgss| prgss.view()));

        let list = row![scrollable(list)];


        //let execute_button = button(text("Execute")).on_press(Message::Execute);


        let date_select_group = row![
            text("Period:")
            ,start_dpicker
            ,text("~")
            ,end_dpicker
        ].spacing(10).width(Length::Fill).align_y(Vertical::Center);

        let mut col = column![
            date_select_group
            ,search_button].align_x(Horizontal::Center).padding([10,10]).spacing(10);



        let container = Container::new(col);

        return container.into();

    }

}

