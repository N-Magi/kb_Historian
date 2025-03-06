
use std::str::FromStr;

use chrono::{DateTime, Utc};
use omnissa_kblib::{error, page::Page};
use rusqlite::{self, config::DbConfig, params, Connection};
use uuid::{uuid, Uuid};

use crate::{diff_tool_error::{self, DiffToolError}, download_progress::KbPage};

pub struct KbDiffEntity{
    pub id:Option<Uuid>,
    pub kb_num:i64,
    pub create_date:chrono::DateTime<Utc>,
    pub last_modified_date:chrono::DateTime<Utc>,
    pub insert_date:Option<DateTime<Utc>>,
    pub title:String,
    pub content:String,
}

impl From<omnissa_kblib::page::Page> for KbDiffEntity {
    fn from(value: Page) -> Self {
        return KbDiffEntity {
            id:None,
            kb_num:value.kb_num,
            create_date:value.create_date,
            last_modified_date:value.last_modified_date,
            insert_date:None,
            title:value.title,
            content:value.content
        };
    }
}

#[derive(Debug,Default)]
pub struct KbDbContext{
    connection:Option<Connection>,
}

impl KbDbContext {

    pub fn new() -> Self {
        return  KbDbContext {connection : None};
    } 

    pub fn connect(&mut self,path:String) -> Result<(),diff_tool_error::DiffToolError> {
    
        let connection = Connection::open(path)
            .map_err(|error| DiffToolError::KbDBConnectingFailed(error.to_string()))?;


        self.connection = Some(connection);        
        
        return  Ok(());
    }

    pub fn close(&mut self) -> Result<(),diff_tool_error::DiffToolError> {
        let a =self.connection.as_mut().unwrap();
        //a.close();
        return  Ok(());
    }

    pub fn create_db(&mut self) -> Result<(),DiffToolError> {
        if self.connection.is_none() {
            
            return Err(DiffToolError::KbDbConnectionIsNothing("create Db".to_string()));
        }                
        
        let conn = self.connection.as_mut().unwrap();
        
        let tx = conn.transaction().map_err(|err| {DiffToolError::KbDbTransactionError(err.to_string())})?;
        
        let db_scheme = "CREATE TABLE IF NOT EXISTS kb_history (
        id TEXT PRIMARY KEY,
        kb_num INTEGER NOT NULL,
        create_date TEXT,
        last_modified_date TEXT,
        insert_date TEXT,
        titile TEXT,
        content TEXT)
        ";
        let a =tx.execute(db_scheme,()).map_err(|f| DiffToolError::KbDbExecutionError(f.to_string()))?;
        tx.commit().map_err(|err|{DiffToolError::KbDbTransactionError(err.to_string())})?;

        return Ok(());
    }

    pub fn insert(&mut self,entity:KbDiffEntity) -> Result<(),DiffToolError> {

        if self.connection.is_none() {
            return Err(DiffToolError::KbDbConnectionIsNothing("insert error".to_string()));
        }
    
        let conn = self.connection.as_mut().unwrap();
        let tx = conn.transaction().map_err(|err| {DiffToolError::KbDbTransactionError(err.to_string())})?;


        let insert_scheme = "INSERT INTO kb_history VALUES (?,?,?,?,?,?,?)";

        let _execute_result = tx.execute(&insert_scheme, 
            params![entity.id.unwrap().to_string(), 
            entity.kb_num, 
            entity.create_date, 
            entity.last_modified_date, 
            Utc::now(),
            entity.title, 
            entity.content])
            .map_err(|f| DiffToolError::KbDbExecutionError(f.to_string()))?;
        
            tx.commit().map_err(|err|{DiffToolError::KbDbTransactionError(err.to_string())})?;

        return  Ok(());
    }

    pub fn get_record(&mut self, kb_num:i64, last_modified_date:DateTime<Utc>) -> Result<usize,DiffToolError>{
        if self.connection.is_none(){
            return Err(DiffToolError::KbDbConnectionIsNothing("Get Record".to_string()));
        }

        let conn = self.connection.as_mut().unwrap();
        let sql_scheme = "SELECT * FROM kb_history WHERE kb_num = ? AND last_modified_date = ?";

        let tx = conn.transaction().unwrap();
        
        let mut stmts = tx.prepare(&sql_scheme)
            .map_err(|err| DiffToolError::KbDbPreparationError(err.to_string()))?;

        let result = stmts.query(params![kb_num,last_modified_date]).iter().count();

        return Ok(result);
        
        todo!()
    }

    pub fn get_history(&mut self,kb_num:i64) -> Result<Vec<KbDiffEntity>,DiffToolError> {
        
        if self.connection.is_none() {
            return Err(DiffToolError::KbDbConnectionIsNothing("insert error".to_string()));
        }

        let conn = self.connection.as_ref().unwrap();
        let select_scheme = "SELECT * FROM kb_history WHERE kb_num = ?";

        let mut statement = conn.prepare(select_scheme)
            .map_err(|err| DiffToolError::KbDbPreparationError(err.to_string()))?;

        let db_rows =statement.query_map(params![kb_num],|row| {
            Ok(KbDiffEntity{
                id: Some(Uuid::from_str(&row.get::<usize,String>(0)?)
                            .map_err(|_err| {rusqlite::Error::ExecuteReturnedResults})?),
                kb_num:row.get(1)?,
                create_date:row.get(2)?,
                last_modified_date:row.get(3)?,
                insert_date:row.get(4)?,
                title:row.get(5)?,
                content:row.get(6)?,
            })
        }).map_err(|err| DiffToolError::KbDbQueryError(err.to_string()))?;

        let mut kb_diffes:Vec<KbDiffEntity> = Vec::new();

        for db_row in db_rows  {
            kb_diffes.push(db_row.map_err(|err| {DiffToolError::KbDbRowDataPaseError(err.to_string())})?);
        }      

        return Ok(kb_diffes);
        
        todo!()
    }

}


