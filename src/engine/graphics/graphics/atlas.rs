use crate::prelude::*;
use rectangle_pack::*;
use std::collections::BTreeMap;

pub type TextureReference = AtlasData;

pub struct Atlas {
    entries: Vec<AtlasData>,
    available_width: u32,
    available_height: u32,
    // available_layers: u32,

    bins: BTreeMap<usize, TargetBin>,

    pub(super) tex: WgpuTexture,
}
impl Atlas {
    pub fn new(width: u32, height: u32, layers: u32, tex: WgpuTexture) -> Self {
        let mut bins = BTreeMap::new();
        bins.insert(0, TargetBin::new(width, height, layers));

        Self {
            entries: Vec::new(),
            available_width: width,
            available_height: height,
            // available_layers: layers,
            bins,
            tex
        }
    }

    // pub fn get_data(&self, tex: &String) -> Option<&AtlasData> {
    //     self.entries.get(tex)
    // }

    pub fn try_insert(&mut self, width: u32, height:u32) -> Option<AtlasData> {
        let mut rects_to_place = GroupedRectsToPlace::<usize>::new();
        rects_to_place.push_rect(
            0,
            None,
            RectToInsert::new(width, height, 1)
        );

        let info = pack_rects(
            &rects_to_place,
            &mut self.bins,
            &volume_heuristic,
            &contains_smallest_box
        ).ok()?;
        let (_, info) = info.packed_locations().get(&0)?;

        let max_size = [self.available_width as f32, self.available_height as f32];

        let x = info.x() as f32;
        let y = info.y() as f32;
        // let z = info.z() as f32;
        let w = info.width() as f32;
        let h = info.height() as f32;
        
        let atlas_data = AtlasData {
            x: info.x(),
            y: info.y(),
            layer: info.z(),
            width: info.width(),
            height: info.height(),

            uvs: Uvs {
                tl: [x, y],
                tr: [x + w, y],
                bl: [x, y + h],
                br: [x + w, y + h],
            }.div(max_size),
        };
        self.entries.push(atlas_data);

        Some(atlas_data)
    }
    
    pub fn try_insert_many(&mut self, data: &Vec<(u32, u32)>) -> Option<Vec<AtlasData>> {
        let mut rects_to_place = GroupedRectsToPlace::<usize>::new();

        for (id, (width, height)) in data.iter().enumerate() {
            rects_to_place.push_rect(
                id,
                None,
                RectToInsert::new(*width, *height, 1)
            );
        }

        let info = pack_rects(
            &rects_to_place,
            &mut self.bins,
            &volume_heuristic,
            &contains_smallest_box
        ).ok()?;

        let max_size = [self.available_width as f32, self.available_height as f32];

        let mut data = info.packed_locations().into_iter().map(|(&id, (_, info))|{
            let x = info.x() as f32;
            let y = info.y() as f32;
            // let z = info.z() as f32;
            let w = info.width() as f32;
            let h = info.height() as f32;
            
            let atlas_data = AtlasData {
                x: info.x(),
                y: info.y(),
                layer: info.z(),
                width: info.width(),
                height: info.height(),

                uvs: Uvs {
                    tl: [x, y],
                    tr: [x + w, y],
                    bl: [x, y + h],
                    br: [x + w, y + h],
                }.div(max_size),
            };

            self.entries.push(atlas_data);
            (id, atlas_data)
        }).collect::<Vec<_>>();

        data.sort_by(|(a,_), (b,_)|a.cmp(b));

        Some(data.into_iter().map(|(_,a)|a).collect())
    }
    
}

#[derive(Copy, Clone, Debug)]
pub struct AtlasData {
    pub x: u32,
    pub y: u32,
    pub layer: u32,
    
    pub width: u32,
    pub height: u32,
    pub uvs: Uvs,
}

#[derive(Copy, Clone, Debug)]
pub struct Uvs {
    pub tl: [f32; 2],
    pub tr: [f32; 2],
    pub bl: [f32; 2],
    pub br: [f32; 2],
}
impl Uvs {
    fn div(mut self, max_size: [f32; 2]) -> Self {
        self.tl = div(self.tl, max_size);
        self.tr = div(self.tr, max_size);
        self.bl = div(self.bl, max_size);
        self.br = div(self.br, max_size);
        self
    }
}

fn div(a: [f32;2], b: [f32;2]) -> [f32;2] {
    [
        a[0] / b[0],
        a[1] / b[1]
    ]
}
