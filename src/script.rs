use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
};
use twox_hash::XxHash32;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RobotData
{
    pub id: String,
    #[serde(rename(deserialize="x"))]
    pub x_pos: f32,
    #[serde(rename(deserialize="y"))]
    pub y_pos: f32,
    #[serde(rename(deserialize="r"))]
    pub rotation: f32,
    #[serde(rename(deserialize="s"))]
    pub current_speed: f32
}

type BHD = BuildHasherDefault<XxHash32>;
#[derive(Debug, Clone)]
pub struct Script
{
    timestamp_increment: f32,
    timestamp_rounding: i32,
    last_timestamp: f32,
    current_timestamp: f32,

    // The hashmap key is 0.0 - X, in `timestamp_increment` increments
    // Key is an f32 as bits (f32::to_bits()) so that the timestamps can be hashable
    timestamps: HashMap<u32, Vec<RobotData>, BHD>,
}

impl Script
{
    pub fn new() -> Script
    {
        Script
        {
            timestamp_increment: 0.0,
            timestamp_rounding: 0,
            last_timestamp: 0.0,
            current_timestamp: 0.0,
            timestamps: Default::default()
        }
    }

    pub fn read(&mut self, script: &str) -> Result<(), std::num::ParseFloatError>
    {
        // Format of the incoming script
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct JsonData
        {
            timeinc: f32,
            timeround: i32,
            timeend: f32,
            timestamps: HashMap<String, Vec<RobotData>, BHD>
        }

        let parsed: JsonData = serde_json::from_str(script).unwrap();

        self.timestamp_increment = parsed.timeinc;
        self.timestamp_rounding = parsed.timeround;
        self.last_timestamp = parsed.timeend;

        for(timestamp, robots) in parsed.timestamps
        {
            let timestamp = timestamp.parse::<f32>()?;
            self.timestamps.insert(timestamp.to_bits(), robots);
        }

        crate::log_s(format!("{:?}", self));
        crate::log_s(format!("{:?}", self.timestamps.get(&2.0f32.to_bits())));

        Ok(())
    }

    fn compute_timestamp(&self, timestamp: f32) -> f32
    {
        let modifier = 10.0f32.powi(self.timestamp_rounding);
        (timestamp * modifier).round() / modifier
    }

    /// Goto a specific timestamp (or as close as possible)
    pub fn goto(&mut self, timestamp: f32)
    {
        self.current_timestamp = self.compute_timestamp(timestamp);
        if self.current_timestamp > self.last_timestamp
        {
            self.current_timestamp = self.last_timestamp;
        }
    }

    /// Step forward by `time`
    pub fn advance_by(&mut self, time: f32)
    {
        self.goto(self.current_timestamp + time);
    }

    /// Step forward by `timestamp_increment`
    pub fn advance(&mut self)
    {
        self.advance_by(self.timestamp_increment);
    }

    pub fn current_timestamp(&self) -> f32
    {
        self.current_timestamp
    }

    pub fn last_timestamp(&self) -> f32
    {
        self.last_timestamp
    }

    pub fn is_done(&self) -> bool
    {
        self.current_timestamp >= self.last_timestamp
    }

    pub fn reset(&mut self)
    {
        self.goto(0.0);
    }

    pub fn data_at(&self, timestamp: f32) -> Option<&Vec<RobotData>>
    {
        self.timestamps.get(&self.compute_timestamp(timestamp).to_bits())
    }

    /// Get the robot data for the current position in the script
    pub fn current_data(&self) -> Option<&Vec<RobotData>>
    {
        self.data_at(self.current_timestamp)
    }
}