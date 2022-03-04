
use prelude::taiko::TaikoGame;

use crate::prelude::*;
use super::difficulty_hit_object::DifficultyHitObject;

// how long each "group" of notes is
const BUCKET_LENGTH:f32 = 500.0;


pub trait DiffCalc<G:GameMode> where Self:Sized {
    fn new(g: &G) -> TatakuResult<Self>;
    fn calc(&mut self) -> TatakuResult<f32>;
}


pub struct TaikoDifficultyCalculator {
    time_rate: f64,
    difficulty_hitobjects: Vec<DifficultyHitObject>
}
impl TaikoDifficultyCalculator {

    fn note_density(&mut self) -> TatakuResult<Vec<usize>> {
        let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;

        let mut note_density:Vec<usize> = Vec::new();
        let mut notes = 0;

        for o in self.difficulty_hitobjects.iter() {
            // If over threshold, move to the next bucket.
            if o.time > start_bucket_time + BUCKET_LENGTH {
                note_density.push(notes);
                notes = 0;
                start_bucket_time = o.time;
            }

            match o.note_type {
                NoteType::Note => {
                    notes += 1;
                },
                
                NoteType::Spinner => {
                    // TODO: assume notes are evenly spread across duration.
                },

                // Do not affect density.
                NoteType::Slider => {}

                // Not relevant to this gamemode.
                NoteType::Hold => panic!("hold in taiko map?!?!"),
            }
        }

        // Push last changes amount.
        note_density.push(notes);
        
        Ok(note_density)
    }

    fn strain(&mut self) -> TatakuResult<Vec<usize>> {
        // 0th hand is the dominant hand.
        let mut hands = [Thing::None; 2];
        let mut count_since_reset = 0;

        let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;

        let mut change_density:Vec<usize> = Vec::new();
        let mut changes = 0;

        for o in self.difficulty_hitobjects.iter() {
            // If over threshold, move to the next bucket.
            if o.time > start_bucket_time + BUCKET_LENGTH {
                change_density.push(changes);
                changes = 0;
                start_bucket_time = o.time;
            }

            match o.note_type {
                NoteType::Note => {
                    let hand_index = count_since_reset % 2;

                    let current_note = if o.is_kat {
                        Thing::Kat
                    } else {
                        Thing::Don
                    };

                    // Check for change.
                    if hands[hand_index] != current_note {
                        changes += 1;
                        hands[hand_index] = current_note;
                    }
                    
                    count_since_reset += 1;
                },

                NoteType::Slider 
                | NoteType::Spinner => {
                    // Reset hands for sliders and spinners.
                    for i in hands.iter_mut() {
                        *i = Thing::None
                    }
                    count_since_reset = 0;
                },

                // Not relevant to this gamemode.
                NoteType::Hold => panic!("hold in taiko map?!?!"),
            }
        }

        // Push last changes amount.
        change_density.push(changes);
        
        Ok(change_density)
    }
}

impl DiffCalc<TaikoGame> for TaikoDifficultyCalculator {
    fn new(g: &TaikoGame) -> TatakuResult<Self> {
        
        let mut difficulty_hitobjects:Vec<DifficultyHitObject> = Vec::new();
        for n in g.notes.iter() {
            let x = DifficultyHitObject::new(n);
            difficulty_hitobjects.push(x);
        }
        

        difficulty_hitobjects.sort_by(|a, b| {
            let a = a.time;
            let b = b.time;
            a.partial_cmp(&b).unwrap()
        });

        Ok(Self {
            time_rate: 1.0,
            difficulty_hitobjects
        })
    }

    fn calc(&mut self) -> TatakuResult<f32> {
        let strain = self.strain()?;

        let note_density = self.note_density()?;

        let mut total = 0.0;
        let mut total_squared = 0.0;
        let count = strain.len() as f32;

        for (strain, density) in strain.into_iter().zip(note_density.into_iter()) {
            let mut strain_value = strain as f32;
            let mut density_value = density as f32;

            strain_value = strain_value.powf(1.2);
            density_value = density_value.powf(1.2);

            let combined = strain_value + density_value;

            total += combined;
            total_squared += combined * combined;
            
            if strain == 0 && density == 0 {
                continue;
            }

            // println!("strain: {}, density: {}, strain value: {}, density value: {}, combined: {}", strain, density, strain_value, density_value, combined);
        }
        
        let mean = total / count;
        let variance = total_squared / count - mean * mean;

        let standard_deviation = variance.sqrt();

        // Calculate difficulty based on normal distribution.
        // z = 1.0 corresponds to 15.9% from the peak difficulty.
        // z = 2.0 corresponds to 2.3% from the peak difficulty.
        // Difficulty values which lie at z>2.0 are considered statistically improbable.
        let z = 1.5; // 6.7% 

        let difficulty = mean + z * standard_deviation;

        println!("mean: {}, std dev: {}, diff: {}", mean, standard_deviation, difficulty);

        Ok(difficulty)
    }

}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Thing {
    None,
    Don,
    Kat
}


fn try_calc(path: impl AsRef<Path>) -> TatakuResult<()> {

    // muzu
    // let map = "D:/Games/osu!/Songs/60452 Reasoner - Coming Home (Ambient Mix)/Reasoner - Coming Home (Ambient Mix) (Blue Dragon) [Taiko Muzukashii].osu";
    
    // cancer
    // let map =  "D:/Games/osu!/Songs/646325 Diabarha - Uranoid/Diabarha - Uranoid (Dargin) [Futsuu].osu";

    // load map
    let beatmap = Beatmap::load(path)?;
    if beatmap.playmode(PlayMode::Standard) != PlayMode::Taiko {return Ok(())}

    
    println!("--- trying map: {}", beatmap.get_beatmap_meta().version_string());
    let mut benchmark = BenchmarkHelper::new("a");
    let mode = TaikoGame::new(&beatmap)?;

    // test calc
    let mut calc = TaikoDifficultyCalculator::new(&mode)?;
    let diff = calc.calc()?;
    println!("got diff: {}", diff);
    benchmark.log("done", true);

    Ok(())
}

#[test]
fn taiko_calc_test() -> TatakuResult<()> {
    use glfw_window::GlfwWindow;

    let window: GlfwWindow = piston::WindowSettings::new("Tataku!", [10, 10])
        .graphics_api(opengl_graphics::OpenGL::V3_2)
        .build()
        .expect("Error creating window");
    

    for folder in std::fs::read_dir("C:/Users/Eve/Desktop/Projects/rust/tataku/tataku-client/songs")? {
        let f = folder?;
        for map in std::fs::read_dir(f.path())? {
            let map = map?;
            if map.file_name().to_str().unwrap().ends_with(".osu") {
                try_calc(map.path())?;
            }
        }
    }

    panic!();
    Ok(())
}