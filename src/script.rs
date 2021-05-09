#![allow(dead_code)]
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
    first_timestamp: f32,
    last_timestamp: f32,
    current_timestamp: f32,

    // The hashmap key is 0.0 - X, in `timestamp_increment` increments
    // Key is an f32 as bits (f32::to_bits()) so that the timestamps can be hashable
    timestamps: HashMap<u32, Vec<RobotData>, BHD>,
}

impl Script
{
    pub fn new(script_str: &str) -> Result<Script, std::num::ParseFloatError>
    {
        // Format of a single timestamp in the incoming script
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Timestamp
        {
            #[serde(rename(deserialize="t"))]
            time: f32,
            #[serde(rename(deserialize="u"))]
            updated: Vec<RobotData>,
            #[serde(rename(deserialize="nu"))]
            not_updated: Vec<String>,
        }

        // Format of the incoming script
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct JsonData
        {
            timeinc: f32,
            timeround: i32,
            timestart: f32,
            timeend: f32,
            timestamps: Vec<Timestamp>
        }

        // Deserialize the script
        let parsed: JsonData = serde_json::from_str(script_str).unwrap();

        let mut script = Script
        {
            timestamp_increment: parsed.timeinc,
            timestamp_rounding: parsed.timeround,
            first_timestamp: parsed.timestart,
            last_timestamp: parsed.timeend,
            current_timestamp: parsed.timestart,
            timestamps: Default::default()
        };

        // Hashmap for storing last known data about each robot for filling in the
        // "notUpdated" entries
        let mut last_data: HashMap<String, RobotData, BHD> = Default::default();

        for timestamp in parsed.timestamps
        {
            // Start off with a populated list of robots known to exist
            let mut robots = timestamp.updated;
            // Update the last known data about this robot in the cache
            for robot in &robots
            {
                last_data.insert(robot.id.to_owned(), robot.clone());
            }
            // For any robots that haven't been updated, use the last known data about the robot
            // instead
            for id in timestamp.not_updated
            {
                let robot = last_data.get(&id).expect("cached robot data");
                robots.push(robot.clone());
            }
            // Add the robots to the timestamp
            script.timestamps.insert(timestamp.time.to_bits(), robots);
        }

        crate::log_s(format!("==START SCRIPT==\n{:?}\n==END SCRIPT==", script));

        Ok(script)
    }

    /// Given a time, make sure that timestamp corresponds to the script's timestamp rounding
    /// and increment parameters
    fn compute_timestamp(&self, timestamp: f32) -> f32
    {
        let modifier = 10.0f32.powi(self.timestamp_rounding);
        (timestamp * modifier).round() / modifier
    }

    /// Goto a specific timestamp (or as close as possible)
    pub fn goto(&mut self, timestamp: f32)
    {
        self.current_timestamp = self.compute_timestamp(timestamp);
        // Bounds checking for the timestamp
        if self.current_timestamp > self.last_timestamp
        {
            self.current_timestamp = self.last_timestamp;
        }
        else if self.current_timestamp < self.first_timestamp
        {
            self.current_timestamp = self.first_timestamp;
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
        self.goto(self.first_timestamp);
    }

    /// Get the robot data present at the given timestamp, or `None` if the timestamp is not present
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