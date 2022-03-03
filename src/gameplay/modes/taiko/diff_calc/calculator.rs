use glfw_window::GlfwWindow;
use prelude::taiko::TaikoGame;

use crate::prelude::*;
use super::difficulty_hit_object::{DifficultyHitObject, DECAY_BASE};

// constants
const STAR_SCALING_FACTOR:f64 = 0.04125;
const STRAIN_STEP:f64 = 400.0;
const DECAY_WEIGHT:f64 = 0.9;

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
    // pub fn compute_difficulty(&mut self) -> f64 {
    //     if !self.calc_strain_values() {
    //         return 0.0;
    //     };
        
    //     let star_rating = self.calculate_difficulty() * STAR_SCALING_FACTOR;

    //     // if (CategoryDifficulty != null) {
    //     //     CategoryDifficulty.Add("Strain", StarRating.ToString("0.00", GameBase.nfi));
    //     //     CategoryDifficulty.Add("Hit window 300", (this.HitObjectManager.HitWindow300_noSlider / this.TimeRate_noSlider).ToString("0.00", GameBase.nfi));
    //     // }

    //     star_rating
    // }

    // fn calc_strain_values(&mut self) -> bool {
    //     let mut enumerator = self.difficulty_hitobjects.iter_mut();

    //     let x = enumerator.next();
    //     if let None = x {
    //         println!("bad");
    //         return false;
    //     }
    //     let mut previous = x.unwrap();

    //     while let Some(current) = enumerator.next() {
    //         // println!("calc!");
    //         current.calculate_strains(&previous, self.time_rate);
    //         previous = current;
    //     }
    //     true
        
    //     // // Traverse hitObjects in pairs to calculate the strain value of NextHitObject from the strain value of CurrentHitObject and environment.
    //     // List<DifficultyHitObjectTaiko>.Enumerator HitObjectsEnumerator = this.DifficultyHitObjects.GetEnumerator();
    //     // if (HitObjectsEnumerator.MoveNext() == false) return false;

    //     // DifficultyHitObjectTaiko CurrentHitObject = HitObjectsEnumerator.Current;
    //     // DifficultyHitObjectTaiko NextHitObject;

    //     // // First hitObject starts at strain 1. 1 is the default for strain values, so we don't need to set it here. See DifficultyHitObject.

    //     // while (HitObjectsEnumerator.MoveNext()) {
    //     //     NextHitObject = HitObjectsEnumerator.Current;
    //     //     NextHitObject.CalculateStrains(CurrentHitObject, this.TimeRate_noSlider);
    //     //     CurrentHitObject = NextHitObject;
    //     // }

    //     // return true;
    // }
    // fn calculate_difficulty(&mut self) -> f64 {
    //     let actual_strain_step = STRAIN_STEP * self.time_rate;

    //     // Find the highest strain value within each strain step
    //     let mut highest_strains = Vec::new();
    //     let mut interval_end_time = actual_strain_step;
    //     let mut maximum_strain = 0.0; // We need to keep track of the maximum strain in the current interval

    //     let iter = self.difficulty_hitobjects.iter_mut();
    //     let mut previous:Option<DifficultyHitObject> = None;

    //     for hitobject in iter {
    //         // While we are beyond the current interval push the currently available maximum to our strain list
    //         while hitobject.time > interval_end_time {
    //             highest_strains.push(maximum_strain);

    //             // The maximum strain of the next interval is not zero by default! We need to take the last hitObject we encountered, take its strain and apply the decay
    //             // until the beginning of the next interval.
    //             if let Some(previous) = &previous {
    //                 let decay = DECAY_BASE.powf(interval_end_time - previous.time as f64 / 1000.0);
    //                 maximum_strain = previous.strain * decay;
    //             } else {
    //                 maximum_strain = 0.0;
    //             }

    //             // Go to the next time interval
    //             interval_end_time += actual_strain_step;
    //         }

    //         // Obtain maximum strain
    //         if hitobject.strain > maximum_strain {
    //             maximum_strain = hitobject.strain;
    //         }

    //         previous = Some(hitobject.clone());
    //     }

    //     // Build the weighted sum over the highest strains for each interval
    //     let mut difficulty = 0.0;
    //     let mut weight = 1.0;
    //     highest_strains.sort_by(|a, b| b.partial_cmp(a).unwrap()); // Sort from highest to lowest strain.

    //     for strain in highest_strains {
    //         difficulty += weight * strain;
    //         weight *= DECAY_WEIGHT;
    //     }

    //     difficulty
    // }

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

            println!("strain: {}, density: {}, strain value: {}, density value: {}, combined: {}", strain, density, strain_value, density_value, combined);
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
    let mode = TaikoGame::new(&beatmap)?;

    // test calc
    let mut calc = TaikoDifficultyCalculator::new(&mode)?;
    let diff = calc.calc()?;
    println!("got diff: {}", diff);

    Ok(())
}

#[test]
fn taiko_calc_test() -> TatakuResult<()> {

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