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
    pub x_pos: f32,
    pub y_pos: f32,
    pub attitude: f32,
    pub current_speed: f32
}

#[derive(Debug, Default, Clone)]
pub struct Timestamp
{
    pub timestamp: f32,
    pub robots: Vec<RobotData>,
}
impl Ord for Timestamp
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        self.timestamp.to_bits().cmp(&other.timestamp.to_bits())
    }
}
impl PartialOrd for Timestamp
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}
impl PartialEq for Timestamp
{
    fn eq(&self, other: &Self) -> bool
    {
        approx_eq!(f32, self.timestamp, other.timestamp)
    }
}
impl Eq for Timestamp {}

#[derive(Debug, Default, Clone)]
pub struct StepData
{
    // Incremental timestamp used while traversing the script
    pub step: f32,
    pub timestamps: Vec<Timestamp>,
}

// If this changes, the way that `nearest_step` is calculated in `read` needs to be changed too
const DEFAULT_STEP_SIZE: f32 = 0.1;
type BHD = BuildHasherDefault<XxHash32>;
#[derive(Debug, Clone)]
pub struct Script
{
    num_steps: f32,
    current_step: f32,

    // The hashmap key is 0.0 - X, in `DEFAULT_STEP_SIZE` increments
    //      This is so that if a specific time is requested (i.e. in the video playback scrubber)
    //      the entire script doesn't have to be traversed - it'll just search through for the
    //      nearest time.
    //      i.e. if DEFAULT_STEP is 0.1
    //          if 0.541 is requested, 0.5 is accessed, then the entry is searched for 0.541
    // Key is an f32 as bits (f32::to_bits()) so that the timestamps can be hashable
    steps: HashMap<u32, StepData, BHD>,
}

impl Script
{
    pub fn new() -> Script
    {
        Script
        {
            num_steps: 0.0,
            current_step: 0.0,
            steps: Default::default()
        }
    }

    pub fn read(&mut self, script: &str) -> Result<(), std::num::ParseFloatError>
    {
        // Format of the incoming script
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct JsonData
        {
            first_timestamp: f32,
            last_timestamp: f32,
            timestamps: HashMap<String, Vec<RobotData>>
        }

        let parsed: JsonData = serde_json::from_str(script).unwrap();

        for (timestamp, robots) in parsed.timestamps
        {
            let timestamp = timestamp.parse::<f32>()?;

            // Round to 1 decimal place
            let nearest_step = (timestamp * 10.0).round() / 10.0;
            let nearest_step_bits = nearest_step.to_bits();

                // Add a new empty vec if the key doesn't exist
            if !self.steps.contains_key(&nearest_step_bits)
            {
                self.steps.insert(nearest_step_bits, Default::default());
            }

            // Append the timestamp and data to the relevant vec
            let step_data = self.steps.get_mut(&nearest_step_bits).expect("key should exist");
            step_data.step = nearest_step;
            step_data.timestamps.push(Timestamp { timestamp, robots });
        }

        for (_, step_data) in &mut self.steps
        {
            step_data.timestamps.sort();
        }

        crate::log_s(format!("{:?}", self.steps.get(&1.0f32.to_bits())));

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