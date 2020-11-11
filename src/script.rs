use std::
{
    hash::BuildHasherDefault,
    collections::HashMap,
};
use twox_hash::XxHash32;
use serde::{Serialize, Deserialize};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotData
{
    id: String,
    xPos: f32, yPos: f32, attitude: f32
}

type BHD = BuildHasherDefault<XxHash32>;
pub struct Script
{
    current_step: f32,
    data: HashMap<u32, HashMap<String, RobotData, BHD>, BHD>
}

const TEST_STR: &'static str = "\
{
    \"0.836\": [
        {
            \"id\": \"Dolphin0\",
            \"xPos\": 0.0,
            \"yPos\": 0.0,
            \"attitude\": 270.0
        }
    ],
    \"0.940\": [
        {
            \"id\": \"Dolphin0\",
            \"xPos\": 0.0,
            \"yPos\": 0.0,
            \"attitude\": 270.0
        }
    ]
}
";
impl Script
{
    pub fn new() -> Script
    {
        Script
        {
            current_step: 0.0,
            data: Default::default()
        }
    }

    pub fn read(&mut self, script: &str)
    {
        let parsed: HashMap<String, Vec<RobotData>> = serde_json::from_str(TEST_STR).unwrap();
        crate::log_s(format!("{:?}", parsed));

/*        let id = "Dolphin0".to_string();
        let robot_data = RobotData{ id: id.clone(), xPos: 0.0, yPos: 0.0, attitude: 0.0};
        if !self.data.contains_key(&0.0f32.to_bits())
        {
            self.data.insert(0.0f32.to_bits(), Default::default());
        }
        let map = self.data.get_mut(&0.0f32.to_bits()).unwrap();
        *map.get_mut(&id).unwrap() = robot_data;*/
    }
}