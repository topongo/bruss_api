use std::ops::Sub;

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug)]
pub struct Position {
    lat: f32,
    lng: f32
}

impl Position {
    const DEG_METER: f32 = 113000.44;

    pub fn new(lat: f32, lng: f32) -> Self {
        Self { lat, lng }
    }
}

impl Sub for Position {
    type Output = f32;
    
    /// Returns the difference in meters from two positions
    fn sub(self, rhs: Self) -> Self::Output {
        ((self.lat - rhs.lat).abs().powi(2) + (self.lng - rhs.lng).abs().powi(2)).sqrt()
    }
}



