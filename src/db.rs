use rocket_db_pools::Database;
use rocket_db_pools::mongodb::Client;

#[derive(Database)]
#[database("bruss")]
pub struct BrussData(Client);

