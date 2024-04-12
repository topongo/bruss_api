pub mod tracking;
pub mod map;

// pub static TRACKING_ROUTES: Vec<Route> = routes![];
// pub const MAP_ROUTES: Vec<Route> = routes![map::get_areas];

#[options("/<_..>")]
pub fn options() {
    /* Intentionally left empty */
}
