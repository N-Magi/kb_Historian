
use std::collections::HashMap;

use dbcontext::kb_dbcontext::{self, KbDbContext, KbDiffEntity};
use download_progress::{KbPage};
use iced::advanced::graphics::futures::backend::default;
use iced::advanced::Text;
use iced::alignment::{Horizontal, Vertical};
use iced::futures::stream::Filter;
use iced::futures::{task, Stream, StreamExt};
use iced::stream::try_channel;
use iced::widget::{
    button, center, column, container, horizontal_space, progress_bar, row, scrollable, text, text_input, Column, Container, ProgressBar, Row};
use iced::window::{self, Settings};
use iced::{Alignment, Center, Element, Fill, Length, Padding, Size, Subscription, Task, Theme, Vector};

use iced_aw::sidebar::row;
use iced_aw::{date_picker, iced_fonts};
use iced_aw::date_picker::Date;
use chrono::{self, DateTime, Datelike, Days, Local, Utc};

use omnissa_kblib::{self};
use omnissa_kblib::search::{SearchFilter, SearchResult};
use tokio::task::futures::TaskLocalFuture;
use uuid::Uuid;


pub mod diff_tool_error;
pub mod dbcontext;
pub mod download_progress;

//use download_progress;

fn main() -> iced::Result  {
    iced::application("difftool", Page::update, Page::view)
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .window_size(Size::new(360.0,200.0))
    .run_with(||{
        return (Page::new(),Task::done(Message::ApplicationInitialize));
    })
}

#[derive(Debug, Default)]
struct Page {

    now:DateTime<Utc>,

    //日付選択画面表示用
    start_date:Date,
    spicker_show:bool,
    end_date :Date,
    epicker_show:bool,
    //KBリスト表示
    downloads:Vec<KbPage>,
    //ダウンロードボタン表示
    show_btn_download:bool,
    //ダウンロードプログレス用
    all_contents:i64,
    show_download_progress:bool,
    task_status:HashMap<Uuid,TaskStatus>,
    theme: Theme,
    //DB_connection
    db_context: KbDbContext,
    
}

#[derive(Debug, PartialEq)]
enum TaskStatus {
    InQueue,
    Completed,
    Skipped,
}



#[derive(Debug, Clone)]
enum Message {
    ApplicationInitialize,
    EnterStartDateBtnPress,
    StartDateCancelled,
    StartDateSumit(Date),
    EnterEndDateBtnPress,
    EndDateCancelled,
    EndDateSumit(Date),
    
    Search,
    Download,
    BackGroudJob(SearchResult),
    PageResult(Result<(Uuid,omnissa_kblib::page::Page),Uuid>),
    ChainTest(&'static KbPage),
}

fn date_into_datetime(date:Date) -> DateTime<Utc>{
    let ddate = format!("{:>4}-{:0>2}-{:0>2}T00:00:00Z",date.year,date.month,date.day);
    println!("{}",&ddate);
    return DateTime::parse_from_rfc3339(&ddate).unwrap().to_utc();
}

fn datetime_into_date(dt:DateTime<Utc>) -> Date{
    Date::from_ymd(dt.year(), dt.month(), dt.day())
}


impl Page {

    fn new() -> Self {

        let mut instance = Page::default();
        instance.now = Utc::now();
        let dayofweek = Days::new(instance.now.weekday().number_from_monday() as u64); //sunday = 0, monday = 1
        let sunday = instance.now - dayofweek;
        let prev_monday = sunday - Days::new(6); 
        instance.start_date = datetime_into_date(prev_monday);
        instance.end_date = datetime_into_date(sunday);

        return instance;
    }

    fn update(&mut self, message: Message) -> Task<Message> {


        match message {

            Message::ApplicationInitialize => {
                self.db_context = KbDbContext::new();
                let _ = self.db_context.connect("./kbdb.sqlite".to_string());
                let _ = self.db_context.create_db();
                return  Task::none();
            }

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

            Message::Search => {

                self.show_btn_download = false;
                self.show_download_progress = false;

                let start_date = date_into_datetime(self.start_date);
                let end_date = date_into_datetime(self.end_date);


                let task = Task::perform(Page::search(start_date, end_date), Message::BackGroudJob);
                
                return task;
            }

            Message::BackGroudJob(search_result) => {

                self.all_contents = search_result.total_count;
                self.task_status.clear();

                self.downloads = search_result.kb_items.iter().map(|result| {
                    let mut prgss = KbPage::new();
                    let task_id = prgss.task_id.clone();
                    prgss.download_name = result.kb_num.to_string();
                    prgss.url = result.click_uri.clone();
                    self.task_status.insert(task_id, TaskStatus::InQueue);
                    return prgss;
                }).collect();
                self.show_btn_download = true;
                return  Task::none();
            }

            Message::Download =>  {

                //let mut tasks:Vec<Task<Message>> = Vec::new();
                self.show_download_progress = true;

                let task:Vec<Task<Message>> = self.downloads.iter_mut().map(|dd| {
                    let download = dd.clone();
                    let task = Task::perform(Page::download_page(download.task_id,download.download_name), Message::PageResult);
                    return  task;
                }).collect();


                return Task::batch(task);
            }

            Message::PageResult(page_result) => {

                //エラー処理　エラーが発生したｋｂはスキップする。
                let Ok((id,result)) = page_result else {

                    let id = page_result.err().unwrap();

                    if let Some(task) = self.task_status.get_mut(&id){
                        *task = TaskStatus::Skipped;
                    }

                    return Task::none();
                };

                //タスクを完了にマーク
                if let Some(task) = self.task_status.get_mut(&id){
                    *task = TaskStatus::Completed;
                }
                
                let mut kb_entity = KbDiffEntity::from(result);
                kb_entity.id = Some(id);
                println!("{}",kb_entity.kb_num);
                let record_qty = self.db_context.get_record(kb_entity.kb_num, kb_entity.last_modified_date).unwrap();
                
                if record_qty == 0 {
                    
                    let _ = self.db_context.insert(kb_entity);
                    return Task::none();
                }

                return Task::none();
            }

            default => {
                println!("Default Hello");
                return  Task::none();
            }
        }

        todo!()
    }


    async fn search(start_time:DateTime<Utc>, end_time:DateTime<Utc>) -> SearchResult {
        let client  = omnissa_kblib::search::SearchClient::new().await.unwrap();
        let mut filter = SearchFilter::default();
        filter.start_date = Some(start_time);//Some(chrono::Local::now().to_utc());
        filter.end_date = Some(end_time);//Some(DateTime::parse_from_rfc3339("2025-01-31T00:00:00.00Z").unwrap().to_utc());
        let result = client.search(filter).await.unwrap();
        return result;
    }

    async fn download_page(task_id:Uuid,kb_num:String) -> Result<(Uuid,omnissa_kblib::page::Page),Uuid> {

        let Ok(page) = omnissa_kblib::page::PageClient::get_content(kb_num).await else {
            return Err(task_id);
        };

        return Ok((task_id,page));
    }

    fn view(&self) -> Element<Message> {
    
        let input_start_date = button(text(self.start_date.to_string())).on_press(Message::EnterStartDateBtnPress);
        let start_dpicker = date_picker(self.spicker_show, self.start_date, input_start_date, Message::StartDateCancelled, Message::StartDateSumit);

        let input_end_date = button(text(self.end_date.to_string())).on_press(Message::EnterEndDateBtnPress);
        let end_dpicker = date_picker(self.epicker_show, self.end_date, input_end_date, Message::EndDateCancelled, Message::EndDateSumit);

        let search_button = button(text("Search").align_x(Center)).on_press(Message::Search).width(Length::Fill);

        let download_button = button(text(format!("Download({})",self.downloads.iter().count())).align_x(Center)).on_press(Message::Download).width(Length::Fill);

        // let download_complete = self.task_status.iter().filter(|task| *task.1 == true).count();
        let qty_download_completed = self.task_status.iter().filter(|task| *task.1 == TaskStatus::Completed ).count();
        let qty_download_skipped = self.task_status.iter().filter(|task| {*task.1 == TaskStatus::Skipped}).count();
        let qty_download_inqueue = self.task_status.iter().filter(|task| {*task.1 == TaskStatus::InQueue}).count();
        let qty_download_all = qty_download_skipped + qty_download_completed;

        let progress_value = if self.all_contents <= 0 {0.0} 
            else {(qty_download_all as f32 / self.all_contents as f32)};

        let download_progress = ProgressBar::new(
        std::ops::RangeInclusive::new(0.0, 1.0), 
        progress_value)
        .height(Length::Fixed(10.0));
        
        let mut progress_label= text(format!("{}/{} in Complete:{},Skip:{}",qty_download_all,self.all_contents,qty_download_completed,qty_download_skipped));
        
        if qty_download_all == self.all_contents as usize {
            progress_label = text("Completed!");
        }


        let date_select_group = row![
            text("Period:")
            ,start_dpicker
            ,text("~")
            ,end_dpicker
        ].spacing(10).width(Length::Fill).align_y(Vertical::Center);

        let mut col = column![
            date_select_group
            ,search_button
            //,scrollable_list
            ].align_x(Horizontal::Center).padding([10,10]).spacing(10);

        if self.show_btn_download {
            col = col.push(download_button);
        }

        if self.show_download_progress {
            col = col.push(download_progress);
            col = col.push(progress_label);
        }

        let container = Container::new(col);
        return container.into();

    }

}

