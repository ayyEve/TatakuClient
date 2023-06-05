use crate::prelude::*;
use rectangle_pack::*;
use std::collections::BTreeMap;

pub struct Atlas {
    entries: HashMap<String, AtlasData>,
    available_width: u32,
    available_height: u32,
    available_layers: u32,

    bins: BTreeMap<usize, TargetBin>,

    pub(super) tex: WgpuTexture,
}
impl Atlas {
    pub fn new(width: u32, height: u32, layers: u32, tex: WgpuTexture) -> Self {
        let mut bins = BTreeMap::new();
        bins.insert(0, TargetBin::new(width, height, layers));

        Self {
            entries: HashMap::new(),
            available_width: width,
            available_height: height,
            available_layers: layers,
            bins,
            tex
        }
    }

    pub fn get_data(&self, tex: &String) -> Option<&AtlasData> {
        self.entries.get(tex)
    }

    pub fn try_insert(&mut self, path: &String, width: u32, height:u32) -> Option<AtlasData> {
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

        let max_size = Vector3::new(self.available_width as f32, self.available_height as f32, self.available_layers as f32);

        let x = info.x() as f32;
        let y = info.y() as f32;
        let z = info.z() as f32;
        let w = info.width() as f32;
        let h = info.height() as f32;

        let z = z - (0.5/self.available_layers as f32);
        
        let atlas_data = AtlasData {
            x: info.x(),
            y: info.y(),
            layer: info.z(),
            width: info.width(),
            height: info.height(),

            uvs: Uvs {
                tl: Vector3::new(x, y, z),
                tr: Vector3::new(x + w, y, z),
                bl: Vector3::new(x, y + h, z),
                br: Vector3::new(x + w, y + h, z),
            }.div(max_size),
        };

        Some(atlas_data)
    }

}

#[derive(Copy, Clone, Debug)]
pub struct AtlasData {
    pub uvs: Uvs,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub layer: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Uvs {
    pub tl: Vector3,
    pub tr: Vector3,
    pub bl: Vector3,
    pub br: Vector3,
}
impl Uvs {
    fn div(mut self, max_size: Vector3) -> Self {
        self.tl = div(self.tl, max_size);
        self.tr = div(self.tr, max_size);
        self.bl = div(self.bl, max_size);
        self.br = div(self.br, max_size);
        self
    }
}

pub type TextureReference = AtlasData;

fn div(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(
        a.x / b.x,
        a.y / b.y,
        a.z / b.z
    )
}