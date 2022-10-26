use rusqlite::{Connection, Statement, Result};
use std::rc::Rc;

struct SmhiConn<'a> {
    conn: Connection,
    temp_stmt: Statement<'a>,
}

pub fn setup_smhi() -> Result<SmhiConn<'static>> {

    let conn = Connection::open("smhi.db3")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS temp (
            time: INTEGER PRIMARY KEY,
            temp REAL)", ())?;

    let mut temp_stmt = conn.prepare("INSERT OR REPLACE INTO temp (time, temp) VALUES(?,?)")?;

    return Ok(SmhiConn{ conn, temp_stmt });
}

pub async fn run_smhi_fetcher(conn: &SmhiConn<'_>) -> Result<()> {
    todo!();
}