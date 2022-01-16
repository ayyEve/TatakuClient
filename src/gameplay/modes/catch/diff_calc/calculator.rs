use std::sync::Arc;

use parking_lot::Mutex;

use crate::gameplay::Beatmap;
use super::difficulty_hit_object::{DifficultyHitObject, DECAY_BASE};

// constants
const STAR_SCALING_FACTOR:f64 = 0.04125;
const STRAIN_STEP:f64 = 400.0;
const DECAY_WEIGHT:f64 = 0.9;

pub struct DifficultyCalculator {
    time_rate: f64,
    difficulty_hitobjects: Vec<DifficultyHitObject>
}
impl DifficultyCalculator {
    pub fn new(beatmap: Arc<Mutex<Beatmap>>) -> DifficultyCalculator {

        let mut difficulty_hitobjects:Vec<DifficultyHitObject> = Vec::new();
        {
            // let lock = beatmap.lock().clone();
            // for i in lock.notes.lock().iter_mut() {
            //     let x = DifficultyHitObject::new(&i);
            //     difficulty_hitobjects.push(x);
            // }
        }

        difficulty_hitobjects.sort_by(|a, b| {
            let a = a.time;
            let b = b.time;
            a.partial_cmp(&b).unwrap()
        });

        DifficultyCalculator {
            time_rate: 1.0,
            difficulty_hitobjects
        }
    }

    pub fn compute_difficulty(&mut self) -> f64 {
        if !self.calc_strain_values() {
            return 0.0;
        };
        
        let star_rating = self.calculate_difficulty() * STAR_SCALING_FACTOR;

        // if (CategoryDifficulty != null) {
        //     CategoryDifficulty.Add("Strain", StarRating.ToString("0.00", GameBase.nfi));
        //     CategoryDifficulty.Add("Hit window 300", (this.HitObjectManager.HitWindow300_noSlider / this.TimeRate_noSlider).ToString("0.00", GameBase.nfi));
        // }

        star_rating
    }

    fn calc_strain_values(&mut self) -> bool {
        let mut enumerator = self.difficulty_hitobjects.iter_mut();

        let x = enumerator.next();
        if let None = x {
            println!("bad");
            return false;
        }
        let mut previous = x.unwrap();

        while let Some(current) = enumerator.next() {
            // println!("calc!");
            current.calculate_strains(&previous, self.time_rate);
            previous = current;
        }
        true
        
        // // Traverse hitObjects in pairs to calculate the strain value of NextHitObject from the strain value of CurrentHitObject and environment.
        // List<DifficultyHitObjectTaiko>.Enumerator HitObjectsEnumerator = this.DifficultyHitObjects.GetEnumerator();
        // if (HitObjectsEnumerator.MoveNext() == false) return false;

        // DifficultyHitObjectTaiko CurrentHitObject = HitObjectsEnumerator.Current;
        // DifficultyHitObjectTaiko NextHitObject;

        // // First hitObject starts at strain 1. 1 is the default for strain values, so we don't need to set it here. See DifficultyHitObject.

        // while (HitObjectsEnumerator.MoveNext()) {
        //     NextHitObject = HitObjectsEnumerator.Current;
        //     NextHitObject.CalculateStrains(CurrentHitObject, this.TimeRate_noSlider);
        //     CurrentHitObject = NextHitObject;
        // }

        // return true;
    }
    fn calculate_difficulty(&mut self) -> f64 {
        let actual_strain_step = STRAIN_STEP * self.time_rate;

        // Find the highest strain value within each strain step
        let mut highest_strains = Vec::new();
        let mut interval_end_time = actual_strain_step;
        let mut maximum_strain = 0.0; // We need to keep track of the maximum strain in the current interval

        let iter = self.difficulty_hitobjects.iter_mut();
        let mut previous:Option<DifficultyHitObject> = None;

        for hitobject in iter {
            // While we are beyond the current interval push the currently available maximum to our strain list
            while hitobject.time > interval_end_time {
                highest_strains.push(maximum_strain);

                // The maximum strain of the next interval is not zero by default! We need to take the last hitObject we encountered, take its strain and apply the decay
                // until the beginning of the next interval.
                if let Some(previous) = &previous {
                    let decay = DECAY_BASE.powf(interval_end_time - previous.time as f64 / 1000.0);
                    maximum_strain = previous.strain * decay;
                } else {
                    maximum_strain = 0.0;
                }

                // Go to the next time interval
                interval_end_time += actual_strain_step;
            }

            // Obtain maximum strain
            if hitobject.strain > maximum_strain {
                maximum_strain = hitobject.strain;
            }

            previous = Some(hitobject.clone());
        }

        // Build the weighted sum over the highest strains for each interval
        let mut difficulty = 0.0;
        let mut weight = 1.0;
        highest_strains.sort_by(|a, b| b.partial_cmp(a).unwrap()); // Sort from highest to lowest strain.

        for strain in highest_strains {
            difficulty += weight * strain;
            weight *= DECAY_WEIGHT;
        }

        difficulty
    }
}