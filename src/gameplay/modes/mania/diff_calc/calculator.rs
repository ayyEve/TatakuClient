use crate::prelude::*;

// how long each "group" of notes is (ms)
const BUCKET_LENGTH:f32 = 500.0;
// const WRITE_DEBUG_FILES:bool = false;

pub struct ManiaDifficultyCalculator {
    difficulty_hitobjects: Vec<DifficultyHitObject>,
    col_count: usize
}

#[async_trait]
impl DiffCalc<super::super::Game> for ManiaDifficultyCalculator {
    async fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        let g = Beatmap::from_metadata(g)?;
        let g = super::super::Game::new(&g, true).await?;
        if g.columns.iter().fold(0, |sum, c| sum + c.len()) == 0 { 
            return Err(BeatmapError::InvalidFile.into()) 
        }
        
        let mut difficulty_hitobjects:Vec<DifficultyHitObject> = Vec::new();
        for c in 0..g.columns.len() {
            for n in g.columns[c].iter() {
                difficulty_hitobjects.push(DifficultyHitObject::new(n.time(), c as u8));

                if n.note_type() == NoteType::Hold {
                    difficulty_hitobjects.push(DifficultyHitObject::new(n.end_time(0.0), c as u8));
                }
            }
        }
        
        difficulty_hitobjects.sort_by(|a, b| {
            let a = a.time;
            let b = b.time;
            a.partial_cmp(&b).unwrap()
        });


        Ok(Self {
            difficulty_hitobjects,
            col_count: g.columns.len()
        })
    }

    async fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        // let strain = self.strain(mods)?;
        let note_density = self.note_density(mods)?;
        let mut diff = Vec::new();

        // let mut lines = vec!["strainvalue,densityvalue,combined,diff".to_owned()];
        // for (strain, density) in strain.into_iter().zip(note_density.into_iter()) {
        for density in note_density.into_iter() {
            // let strain_value = (strain as f32).powf(1.75);
            // let density_value = density;

            // let combined = strain_value + density_value;
            // diff.push(combined);
            
            diff.push(density / self.col_count as f32);
            // if WRITE_DEBUG_FILES {
            //     lines.push(format!("{},{},{}", strain_value, density_value, combined));
            // }
        }
        
        // let count = diff.len() as f32;

        let mut difficulty = 0.0;
        let mut weight = 1.0;

        const PERCENT: f32 = 0.99;

        // Sort by descending
        diff.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        for x in diff {
            difficulty += x * weight;
            weight *= PERCENT;
        }

        difficulty /= (1.0 - weight) / (1.0 - PERCENT);

        /*
        // TEMP: for writing to csv, nicer graphs
        if WRITE_DEBUG_FILES {
            for i in 1..lines.len() {
                lines[i] += &format!(",{}", difficulty);
            }
            let file_name = self
                .version_string
                .replace("/", "")
                .replace("\\", "")
                .replace("?", "")
                .replace("'", "")
                .replace("*", "")
                .replace("&", "")
                .replace("<", "")
                .replace(">", "")
                .replace(";", "")
                .replace("\"", "")
                .replace("?", "")
                .replace("|", "")
                ;
            
            std::fs::write(format!("./csv/{}.csv", file_name), lines.join("\n"))?;

            {
                let mut hashmap = HashMap::new();
                let column_count = lines[0].split(",").count();
                let mut labels = Vec::new();

                for i in 0..column_count {
                    hashmap.insert(i, Vec::new());
                }

                for (i, line) in lines.iter().enumerate() {
                    let split = line.split(",");

                    if i == 0 {
                        for (_n, c) in split.into_iter().enumerate() {
                            labels.push(c);
                        }
                    } else {
                        for (n, c) in split.into_iter().enumerate() {
                            hashmap.get_mut(&n).unwrap().push(c);
                        }
                    }
                }

                let colors = [
                    "66,133,244",
                    "234,67,53",
                    "251,188,4",
                    "52,168,83"
                ];

                let mut data_sets = Vec::new();
                for (i, values) in hashmap.iter() {
                    let label = labels[*i];
                    let color = colors[i % colors.len()];
                    let data = values.join(",");
                    {
                        data_sets.push(format!(r#"{{
                            label: '{label}',
                            data: [{data}],
                            backgroundColor: ['rgba({color}, 0.2)'],
                            borderColor: ['rgba({color}, 1)'],
                            borderWidth: 1
                        }}"#));
                    }
                }


                let x_line = (0..lines.len()-1).into_iter().fold(String::new(), |f, g| format!("{}'{}',", f, g));

                let datasets = data_sets.join(",");
                let all_data = format!(r#"
                <script src='https://cdn.jsdelivr.net/npm/chart.js@3.7.1/dist/chart.min.js'></script>
                <canvas id="myChart" width="1280" height="720"></canvas>
                <script>
                const ctx = document.getElementById('myChart').getContext('2d');
                const myChart = new Chart(ctx, {{
                    type: 'line',
                    data: {{
                        labels: [{x_line}],
                        datasets: [{datasets}]
                    }},
                    options: {{
                        scales: {{
                            y: {{
                                beginAtZero: true
                            }}
                        }}
                    }}
                }});
                </script>
                "#);

                std::fs::write(format!("./html/{}.html", file_name), all_data)?
            }
        }
        */

        Ok(difficulty)
    }
}

impl ManiaDifficultyCalculator {
    
    fn note_density(&mut self, mods: &ModManager) -> TatakuResult<Vec<f32>> {
        let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;
        let mut last_note_time = start_bucket_time;

        let mut note_density = Vec::new();
        let mut density = 0.0;

        let bucket_length = BUCKET_LENGTH * mods.speed;

        let window = 43.0; // perfect window

        for o in self.difficulty_hitobjects.iter().skip(1) {
            let o_time = o.time;// * mods.speed;

            // If over threshold, move to the next bucket.
            if o_time > start_bucket_time + bucket_length {
                // Add final note to current bucket density
                density += bucket_length / (o_time - last_note_time).max(window);

                note_density.push(density);
                density = 0.0;
                start_bucket_time = o_time;
            }

            density += bucket_length / (o_time - last_note_time).max(window);
            last_note_time = o_time;
        }

        // Push last changes amount.
        note_density.push(density);
        
        Ok(note_density)
    }

    // fn strain(&mut self, mods: &ModManager) -> TatakuResult<Vec<usize>> {
    //     // 0th hand is the dominant hand.
    //     let mut hands = [Thing::None; 2];
    //     let mut count_since_reset = 0;

    //     let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;

    //     let mut change_density = Vec::new();
    //     let mut changes = 0;

        
    //     let bucket_length = BUCKET_LENGTH * mods.speed;

    //     for o in self.difficulty_hitobjects.iter() {
    //         let o_time = o.time; // * mods.speed;

    //         // If over threshold, move to the next bucket.
    //         if o_time > start_bucket_time + bucket_length {
    //             change_density.push(changes);
    //             changes = 0;
    //             start_bucket_time = o_time;
    //         }

    //         match o.note_type {
    //             NoteType::Note => {
    //                 let hand_index = count_since_reset % 2;

    //                 let current_note = if o.is_kat {
    //                     Thing::Kat
    //                 } else {
    //                     Thing::Don
    //                 };

    //                 // Check for change.
    //                 if hands[hand_index] != current_note {
    //                     changes += 1;
    //                     hands[hand_index] = current_note;
    //                 }
                    
    //                 count_since_reset += 1;
    //             },

    //             NoteType::Hold => {},

    //             _ => {}
    //         }
    //     }

    //     // Push last changes amount.
    //     change_density.push(changes);
        
    //     Ok(change_density)
    // }
}




pub struct DifficultyHitObject {
    pub col: u8,
    pub time: f32,
}

impl DifficultyHitObject {
    pub fn new(time: f32, col: u8) -> Self {
        Self {
            col,
            time,
        }
    }
}