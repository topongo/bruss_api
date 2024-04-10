use rocket_db_pools::Database;
use mongodb::Client;

#[derive(Database)]
#[database("bruss")]
pub struct BrussData(Client);

