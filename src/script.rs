use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
    cmp::Ordering,
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
    timestamp_rounding: f32,
    last_timestamp: f32,
    current_timestamp: f32,

    // The hashmap key is 0.0 - X, in `DEFAULT_STEP_SIZE` increments
    //      This is so that if a specific time is requested (i.e. in the video playback scrubber)
    //      the entire script doesn't have to be traversed - it'll just search through for the
    //      nearest time.
    //      i.e. if DEFAULT_STEP is 0.1
    //          if 0.541 is requested, 0.5 is accessed, then the entry is searched for 0.541
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
            timestamp_rounding: 0.0,
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
            timeround: f32,
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

    /// Step forward by `step`
    pub fn step_by(&mut self, step: f32)
    {

    }

    /// Step forward by 0.1
    pub fn step(&mut self)
    {
        self.step_by(0.1);
    }

    /// Goto a specific timestamp (or as close as possible)
    pub fn goto(&mut self, timestamp: f32)
    {

    }

    /// Get the robot data for the current position in the script
    pub fn get_current_data(&self) -> Vec<RobotData>
    {
        vec![]
    }
}