use crate::prelude::*;
use super::super::osu::*;
use super::super::osu_notes::*;


const BUCKET_LENGTH:f32 = 500.0;

pub struct OsuDifficultyCalculator {
    notes: Vec<OsuDifficultyHitObject>,
}
impl OsuDifficultyCalculator {
    fn calc_aim(&mut self, mods: &ModManager) -> TatakuResult<Vec<f64>> {
        // 0th hand is the dominant hand.
        let mut start_bucket_time = self.notes.first().unwrap().time;

        let mut aim_density = Vec::new();
        let mut aims = 0.0;

        
        let bucket_length = BUCKET_LENGTH * mods.speed;

        for i in 0..self.notes.len() - 1 {
            let note1 = &self.notes[i];
            let note2 = &self.notes[i + 1];

            // If over threshold, move to the next bucket.
            if note1.time > start_bucket_time + bucket_length {
                aim_density.push(aims);
                aims = 0.0;
                start_bucket_time = note1.time;
            }

            match note1.note_type {
                NoteType::Note => {
                    aims += note1.end_pos.distance(note2.pos);
                },

                NoteType::Slider 
                | NoteType::Spinner => {
                },

                // Not relevant to this gamemode.
                NoteType::Hold => panic!("hold in std map?!?!"),
            }
        }

        // Push last changes amount.
        aim_density.push(aims);
        
        Ok(aim_density)
    }

    fn calc_density(&mut self, mods: &ModManager) -> TatakuResult<Vec<f32>> {
        let mut start_bucket_time = self.notes.first().unwrap().time;
        let mut last_note_time = start_bucket_time;

        let mut note_density = Vec::new();
        let mut density = 0.0;

        let bucket_length = BUCKET_LENGTH * mods.speed;

        for o in self.notes.iter().skip(1) {
            let o_time = o.time;// * mods.speed;

            // If over threshold, move to the next bucket.
            if o_time > start_bucket_time + bucket_length {
                // Add final note to current bucket density
                density += bucket_length / (o_time - last_note_time).max(1.0);

                note_density.push(density);
                density = 0.0;
                start_bucket_time = o_time;
            }

            match o.note_type {
                NoteType::Note => {
                    density += bucket_length / (o_time - last_note_time).max(1.0);

                    last_note_time = o.end_time;
                },
                
                NoteType::Spinner => {
                    // TODO: assume d,k are evenly spread across duration.

                    // let duration = o.end_time - o.time;
                    // let count = o.hits_to_complete;

                    // let add_per = duration / count as f32;

                    // for i in 0..count {
                    //     let time = o.time + add_per * (i as f32);
                        
                    //     if time > start_bucket_time + BUCKET_LENGTH {
                    //         note_density.push(density);
                    //         density = 0.0;
                    //         start_bucket_time = time;
                    //     }

                    //     density += 0.5 / (o.time - last_note_time);

                    //     last_note_time = o.time;
                    // }
                },

                // Do not affect density.
                NoteType::Slider => {}

                // Not relevant to this gamemode.
                NoteType::Hold => panic!("hold in taiko map?!?!"),
            }
        }

        // Push last changes amount.
        note_density.push(density);
        
        Ok(note_density)
    }
}

impl DiffCalc<super::super::Game> for OsuDifficultyCalculator {
    fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        
        let g = Beatmap::from_metadata(g)?;
        let g = StandardGame::new(&g)?;

        let mut notes = Vec::new();
        for n in g.notes.iter() {
            notes.push(OsuDifficultyHitObject {
                pos: n.pos_at(n.time()),
                end_pos: n.pos_at(n.end_time(0.0)),
                time: n.time(),
                end_time: n.end_time(0.0),
                note_type: n.note_type()
            });
        }

        notes.sort_by(|a, b| {
            let a = a.time;
            let b = b.time;
            a.partial_cmp(&b).unwrap()
        });

        Ok(Self {
            notes
        })
    }

    fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        let aim = self.calc_aim(mods)?;
        let note_density = self.calc_density(mods)?;

        let mut diff = Vec::new();

        for (strain, density) in aim.into_iter().zip(note_density.into_iter()) {
            let strain_value = (strain as f32).powf(1.75);
            let density_value = density;

            let combined = strain_value + density_value;

            diff.push(combined);
        }
        
        let count = diff.len() as f32;

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        const PERCENT: f32 = 0.99;

        // Sort by descending
        diff.sort_by(|a, b| b.partial_cmp(a).unwrap());

        for x in diff {
            //println!("hi: {} * {}%", x, weight);
            difficulty += x * weight;
            weight *= PERCENT;
        }

        difficulty /= (1.0 - weight) / (1.0 - PERCENT);


        Ok(difficulty)
    }
}


struct OsuDifficultyHitObject {
    pos: Vector2,
    end_pos: Vector2,
    time: f32,
    end_time: f32,
    note_type: NoteType
}