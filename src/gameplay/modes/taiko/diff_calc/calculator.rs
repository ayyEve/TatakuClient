
use super::super::TaikoGame;

use crate::prelude::*;
use super::difficulty_hit_object::DifficultyHitObject;
use super::super::FINISHER_LENIENCY;

// how long each "group" of notes is (ms)
const BUCKET_LENGTH:f32 = 500.0;


pub struct TaikoDifficultyCalculator {
    difficulty_hitobjects: Vec<DifficultyHitObject>,
    version_string: String,
}
impl TaikoDifficultyCalculator {

    fn note_density(&mut self, mods: &ModManager) -> TatakuResult<Vec<f32>> {
        let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;
        let mut last_note_time = start_bucket_time;

        let mut note_density = Vec::new();
        let mut density = 0.0;

        let bucket_length = BUCKET_LENGTH * mods.speed;

        for o in self.difficulty_hitobjects.iter().skip(1) {
            let o_time = o.time;// * mods.speed;

            // If over threshold, move to the next bucket.
            if o_time > start_bucket_time + bucket_length {
                // Add final note to current bucket density
                density += bucket_length / (o_time - last_note_time).max(FINISHER_LENIENCY);

                note_density.push(density);
                density = 0.0;
                start_bucket_time = o_time;
            }

            match o.note_type {
                NoteType::Note => {
                    density += bucket_length / (o_time - last_note_time).max(FINISHER_LENIENCY);

                    last_note_time = o_time;
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

    fn strain(&mut self, mods: &ModManager) -> TatakuResult<Vec<usize>> {
        // 0th hand is the dominant hand.
        let mut hands = [Thing::None; 2];
        let mut count_since_reset = 0;

        let mut start_bucket_time = self.difficulty_hitobjects.first().unwrap().time;

        let mut change_density = Vec::new();
        let mut changes = 0;

        
        let bucket_length = BUCKET_LENGTH * mods.speed;

        for o in self.difficulty_hitobjects.iter() {
            let o_time = o.time; // * mods.speed;

            // If over threshold, move to the next bucket.
            if o_time > start_bucket_time + bucket_length {
                change_density.push(changes);
                changes = 0;
                start_bucket_time = o_time;
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

const WRITE_DEBUG_FILES:bool = false;

#[async_trait]
impl DiffCalc<TaikoGame> for TaikoDifficultyCalculator {
    async fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        let g = Beatmap::from_metadata(g)?;
        let g = TaikoGame::new(&g, true).await?;
        if g.notes.is_empty() { return Err(BeatmapError::InvalidFile.into()) }
        
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
            difficulty_hitobjects,
            version_string: String::new()
        })
    }

    async fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        let strain = self.strain(mods)?;
        let note_density = self.note_density(mods)?;

        let mut diff = Vec::new();

        let mut lines = vec!["strainvalue,densityvalue,combined,diff".to_owned()];
        for (strain, density) in strain.into_iter().zip(note_density.into_iter()) {
            let strain_value = (strain as f32).powf(1.75);
            let density_value = density;

            let combined = strain_value + density_value;

            diff.push(combined);
            if WRITE_DEBUG_FILES {
                lines.push(format!("{},{},{}", strain_value, density_value, combined));
            }
        }
        
        let count = diff.len() as f32;

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

        Ok(difficulty)
    }

}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Thing {
    None,
    Don,
    Kat
}






// #[test]
// fn taiko_calc_test() -> TatakuResult<()> {
//     use glfw_window::GlfwWindow;

//     // need to init opengl
//     let _window: GlfwWindow = piston::WindowSettings::new("Tataku!", [10, 10])
//         .graphics_api(opengl_graphics::OpenGL::V3_2)
//         .build()
//         .expect("Error creating window");


//     // let path = "C:/Users/Eve/Desktop/Projects/rust/tataku/tataku-client/songs";
//     let path = "D:/Games/osu!/Songs";
    

//     for folder in std::fs::read_dir(path)? {
//         let f = folder?;
//         for map in std::fs::read_dir(f.path())? {
//             let map = map?;
//             if map.file_name().to_str().unwrap().ends_with(".osu") {
//                 let _ = try_calc(map.path());
//             }
//         }
//     }

//     panic!();
//     Ok(())
// }

// #[allow(unused)]
// async fn try_calc(path: impl AsRef<Path>) -> TatakuResult<()> {

//     // muzu
//     // let map = "D:/Games/osu!/Songs/60452 Reasoner - Coming Home (Ambient Mix)/Reasoner - Coming Home (Ambient Mix) (Blue Dragon) [Taiko Muzukashii].osu";
    
//     // cancer
//     // let map =  "D:/Games/osu!/Songs/646325 Diabarha - Uranoid/Diabarha - Uranoid (Dargin) [Futsuu].osu";

//     // load map
//     let fake_maps = Beatmap::load_multiple(path)?;
//     let beatmap = fake_maps.first().unwrap();
//     // if beatmap.playmode(PlayMode::Standard) != PlayMode::Taiko {return Ok(())}

//     let mods = ModManager::new();

//     let s = beatmap.get_beatmap_meta().version_string();
//     trace!("\n\n\n--- trying map: {}", s);
//     // let mut benchmark = BenchmarkHelper::new("calc");
//     if let Ok(mode) = TaikoGame::new(&beatmap, true) {
//         // test calc
//         let mut calc = TaikoDifficultyCalculator::new(&beatmap.get_beatmap_meta())?;
//         calc.version_string = s;
//         let diff = calc.calc(&mods)?;
//     }


//     Ok(())
// }
