#![allow(unused, dead_code)]
// opens on right hand side
// should take up 1/3 of the screen
// is full height, or as tall as needed
use crate::prelude::*;
use crate::commits::*;

const ITEM_HEIGHT:f64 = 18.0;
const ITEM_PADDING:Vector2 = Vector2::new(2.0, 2.0);

pub struct ChangelogDialog {
    items: Vec<String>,
    bounds: Rectangle,

    should_close: bool
}
impl ChangelogDialog {
    pub async fn new() -> Self {
        // assume the settings hasnt been updated
        let mut settings = Settings::get_mut();
        let last_commit = &settings.last_git_hash;

        let mut items = Vec::new();
        for &(id, title, _message, date, _url) in COMMITS {
            if id == last_commit {break}

            let date = GitDate::from_str(date);
            items.push(format!(
                "{}:{}:{} - {}", 
                date.day, date.month, date.year, 
                title
            ))
        }
        settings.last_git_hash = COMMIT_HASH.to_owned();
        let height = (items.len()+1) as f64 * (ITEM_HEIGHT + ITEM_PADDING.y) + ITEM_PADDING.y;

        let window = Settings::window_size();
        let bounds = Rectangle::new(
            Color::BLUE.alpha(0.8),
            0.0,
            Vector2::new(
                window.x * (2.0/3.0),
                0.0
            ),
            Vector2::new(
                window.x / 3.0, 
                height
            ),
            Some(Border::new(
                Color::BLACK, 
                1.5
            ))
        );
        
        Self {
            // should immediately close if theres no items to show
            should_close: items.len() == 0,

            items, 
            bounds,
        }
    }
}

#[async_trait]
impl Dialog<Game> for ChangelogDialog {
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    fn should_close(&self) -> bool {
        self.should_close
    }

    async fn on_key_press(&mut self, key:&Key, _mods:&KeyModifiers, _g:&mut Game) -> bool {
        if key == &Key::Escape {
            self.should_close = true;
            return true
        }

        false
    }

    async fn draw(&mut self, _args:&RenderArgs, depth: &f64, list: &mut Vec<Box<dyn Renderable>>) {
        // background and border
        let mut bg_rect = self.bounds.clone();
        bg_rect.depth = *depth;

        let font = get_font();
        list.push(Box::new(Text::new(
            Color::BLACK,
            depth - 0.00001,
            Vector2::new(bg_rect.current_pos.x + ITEM_PADDING.x, ITEM_HEIGHT),
            ITEM_HEIGHT as u32,
            format!("Whats new:"),
            font.clone()
        )));
        
        for (i, text) in self.items.iter().enumerate() {
            let y = (i+1) as f64 * (ITEM_HEIGHT + ITEM_PADDING.y) + ITEM_HEIGHT;
            list.push(Box::new(Text::new(
                Color::BLACK,
                depth - 0.00001,
                Vector2::new(bg_rect.current_pos.x + ITEM_PADDING.x, y),
                ITEM_HEIGHT as u32,
                text.clone(),
                font.clone()
            )));
        }

        list.push(Box::new(bg_rect));
    }

}



// bc i dont want to add another dep :/
// im so sorry for this code
struct GitDate {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    timezone: i8
}
impl GitDate {
    fn from_str(line:&str) -> Self {
        // 2021-12-07T22:44:

        let mut split = line.split("T");
        let date = split.next().unwrap();
        let time = split.next().unwrap();

        // date things
        let mut date_split = date.split("-");
        let year = date_split.next().unwrap().parse().unwrap();
        let month = date_split.next().unwrap().parse().unwrap();
        let day = date_split.next().unwrap().parse().unwrap();
        
        
        // time things
        let mut time_split = time.split(":");
        let hour = time_split.next().unwrap().parse().unwrap();
        let minute = time_split.next().unwrap().parse().unwrap();
        let second_timezone = time_split.next().unwrap();

        let (tz_mul, mut second_timezone_split) = if second_timezone.contains("-") {
            (-1, second_timezone.split("-"))
        } else {
            (1, second_timezone.split("+"))
        };
        let second = second_timezone_split.next().unwrap().parse::<f32>().unwrap() as u8;
        let timezone = second_timezone_split.next().unwrap().parse::<i8>().unwrap() * tz_mul;

        
        Self {
            year,
            month,
            day,

            hour, 
            minute, 
            second,
            timezone
        }

    }
}
