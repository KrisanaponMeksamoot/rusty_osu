use std::{
    collections::HashMap, fs::File, io::{BufRead, BufReader}, path::Path
};

#[derive(Debug, Default)]
pub struct OsuFile {
    pub general: General,
    pub metadata: Metadata,
    pub difficulty: Difficulty,
    pub timing_points: Vec<TimingPoint>,
    pub hit_objects: Vec<HitObject>,
    pub colours: Colours,
}

#[derive(Debug, Default)]
pub struct General {
    pub audio_filename: String,
    pub audio_lead_in: i32,
    pub preview_time: i32,
    pub countdown: i32,
    pub sample_set: String,
    pub stack_leniency: f32,
    pub mode: i32,
    pub letterbox_in_breaks: bool,
    pub widescreen_storyboard: bool,
}

#[derive(Debug, Default)]
pub struct Metadata {
    pub title: String,
    pub title_unicode: String,
    pub artist: String,
    pub artist_unicode: String,
    pub creator: String,
    pub version: String,
    pub source: String,
    pub tags: String,
    pub beatmap_id: i32,
    pub beatmap_set_id: i32,
}

#[derive(Debug, Default)]
pub struct Difficulty {
    pub hp_drain_rate: f32,
    pub circle_size: f32,
    pub overall_difficulty: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}

#[derive(Debug)]
pub struct TimingPoint {
    pub offset: f64,
    pub ms_per_beat: f64,
    pub meter: i32,
    pub sample_type: i32,
    pub sample_set: i32,
    pub volume: i32,
    pub uninherited: bool,
    pub effects: i32,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct HitObjectType: u32 {
        const CIRCLE = 1;
        const SLIDER = 2;
        const SPINNER = 8;
        const NEW_COMBO = 4;
    }
}

#[derive(Debug)]
pub enum HitObjectShape {
    Circle,
    Slider,
    Spinner,
    Unknown,
}

#[derive(Debug)]
pub struct HitObject {
    pub x: i32,
    pub y: i32,
    pub time: i32,
    pub obj_type: HitObjectType,
    pub shape: HitObjectShape,
    pub hit_sound: i32,
    pub extras: String,
}

#[derive(Debug)]
pub struct Colours {
    pub combos: Vec<(u8, u8, u8)>,
    pub others: HashMap<String, (u8, u8, u8)>,
}

impl Default for Colours {
    fn default() -> Self {
        Colours {
            combos: Vec::new(),
            others: HashMap::new(),
        }
    }
}

trait ParseKeyValue {
    fn set_field(&mut self, key: &str, value: &str);
}

impl ParseKeyValue for General {
    fn set_field(&mut self, key: &str, value: &str) {
        match key {
            "AudioFilename" => self.audio_filename = value.to_string(),
            "Mode" => self.mode = value.parse().unwrap_or(0),
            _ => {}
        }
    }
}

impl ParseKeyValue for Metadata {
    fn set_field(&mut self, key: &str, value: &str) {
        match key {
            "Title" => self.title = value.to_string(),
            "Artist" => self.artist = value.to_string(),
            "Creator" => self.creator = value.to_string(),
            "Version" => self.version = value.to_string(),
            _ => {}
        }
    }
}

impl ParseKeyValue for Difficulty {
    fn set_field(&mut self, key: &str, value: &str) {
        match key {
            "HPDrainRate" => self.hp_drain_rate = value.parse().unwrap_or(5.0),
            "CircleSize" => self.circle_size = value.parse().unwrap_or(5.0),
            "OverallDifficulty" => self.overall_difficulty = value.parse().unwrap_or(5.0),
            "ApproachRate" => self.approach_rate = value.parse().unwrap_or(5.0),
            "SliderMultiplier" => self.slider_multiplier = value.parse().unwrap_or(1.0),
            "SliderTickRate" => self.slider_tick_rate = value.parse().unwrap_or(1.0),
            _ => {}
        }
    }
}

pub fn parse_osu(path: &Path) -> OsuFile {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut osu = OsuFile::default();
    let mut section = String::new();

    for line in reader.lines().flatten() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }

        match section.as_str() {
            "General" => parse_key_value(line, &mut osu.general),
            "Metadata" => parse_key_value(line, &mut osu.metadata),
            "Difficulty" => parse_key_value(line, &mut osu.difficulty),
            "TimingPoints" => osu.timing_points.push(parse_timing_point(line)),
            "HitObjects" => osu.hit_objects.push(parse_hit_object(line)),
            "Colours" => parse_colour(line, &mut osu.colours),
            _ => {}
        }
    }

    osu
}

fn parse_key_value<T: ParseKeyValue>(line: &str, target: &mut T) {
    if let Some((key, value)) = line.split_once(':') {
        target.set_field(key.trim(), value.trim());
    }
}

fn parse_hit_object(line: &str) -> HitObject {
    let parts: Vec<&str> = line.split(',').collect();
    let obj_type_num: u32 = parts[3].parse().unwrap_or(0);
    let obj_type = HitObjectType::from_bits_truncate(obj_type_num);

    let shape = if obj_type.contains(HitObjectType::CIRCLE) {
        HitObjectShape::Circle
    } else if obj_type.contains(HitObjectType::SLIDER) {
        HitObjectShape::Slider
    } else if obj_type.contains(HitObjectType::SPINNER) {
        HitObjectShape::Spinner
    } else {
        HitObjectShape::Unknown
    };

    HitObject {
        x: parts[0].parse().unwrap_or(0),
        y: parts[1].parse().unwrap_or(0),
        time: parts[2].parse().unwrap_or(0),
        obj_type,
        shape,
        hit_sound: parts[4].parse().unwrap_or(0),
        extras: parts[5..].join(","),
    }
}


fn parse_timing_point(line: &str) -> TimingPoint {
    let parts: Vec<&str> = line.split(',').collect();

    TimingPoint {
        offset: parts[0].parse().unwrap_or(0.0),
        ms_per_beat: parts[1].parse().unwrap_or(0.0),
        meter: parts[2].parse().unwrap_or(4),
        sample_type: parts[3].parse().unwrap_or(0),
        sample_set: parts[4].parse().unwrap_or(0),
        volume: parts[5].parse().unwrap_or(100),
        uninherited: parts[6].parse::<i32>().unwrap_or(1) == 1,
        effects: parts[7].parse().unwrap_or(0),
    }
}

fn parse_colour(line: &str, colours: &mut Colours) {
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim();
        let rgb_str = value.trim();
        let rgb: Vec<u8> = rgb_str
            .split(',')
            .filter_map(|c| c.trim().parse::<u8>().ok())
            .collect();

        if rgb.len() == 3 {
            if key.to_lowercase().starts_with("combo") {
                colours.combos.push((rgb[0], rgb[1], rgb[2]));
            } else {
                colours.others.insert(key.to_string(), (rgb[0], rgb[1], rgb[2]));
            }
        }
    }
}
