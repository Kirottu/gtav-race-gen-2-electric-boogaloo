use std::{
    borrow::Cow,
    env, fs,
    ops::{Div, Mul},
};

use egui_macroquad::egui;
use macroquad::{prelude::*, rand::ChooseRandom};
use serde::{Deserialize, Serialize};

const CIRCLE_RADIUS: f32 = 20.0;
const CIRCLE_THICKNESS: f32 = 5.0;
const LINE_THICKNESS: f32 = 7.0;
const TEXT_SIZE: f32 = 50.0;

const DEFAULT_POINTS: &[Point] = &[
    Point { x: 580., y: 352. },
    Point { x: 273., y: 1243. },
    Point { x: 340., y: 1484. },
    Point { x: 886., y: 308. },
    Point { x: 736., y: 747. },
    Point { x: 220., y: 681. },
    Point { x: 976., y: 600. },
    Point { x: 839., y: 1140. },
    Point { x: 640., y: 1495. },
    Point { x: 217., y: 1056. },
    Point { x: 388., y: 279. },
    Point { x: 693., y: 675. },
    Point { x: 875., y: 873. },
    Point { x: 600., y: 1007. },
    Point { x: 504., y: 968. },
    Point { x: 645., y: 780. },
    Point { x: 784., y: 460. },
    Point { x: 725., y: 1080. },
    Point { x: 703., y: 936. },
    Point { x: 69., y: 960. },
    Point { x: 882., y: 1274. },
    Point { x: 521., y: 150. },
    Point { x: 437., y: 823. },
    Point { x: 316., y: 804. },
    Point { x: 173., y: 834. },
    Point { x: 316., y: 974. },
    Point { x: 638., y: 1074. },
    Point { x: 368., y: 439. },
    Point { x: 519., y: 599. },
    Point { x: 887., y: 716. },
    Point { x: 509., y: 1190. },
    Point { x: 359., y: 1078. },
    Point { x: 537., y: 1488. },
    Point { x: 582., y: 1494. },
    Point { x: 357., y: 582. },
    Point { x: 946., y: 409. },
    Point { x: 446., y: 1449. },
];

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Point {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Point {
    #[inline(always)]
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: (f32, f32)) -> f32 {
        let x = other.0 - self.x;
        let y = other.1 - self.y;
        (x * x + y * y).sqrt()
    }
}

impl From<Point> for Vec2 {
    fn from(value: Point) -> Self {
        Vec2::new(value.x, value.y)
    }
}

enum State {
    Race(RaceState),
    Config(Vec<Point>),
    Idle,
}

impl State {
    fn toggle_edit_controls(&mut self) {
        if let State::Race(RaceState {
            edit_controls_collapsed,
            ..
        }) = self
        {
            *edit_controls_collapsed = !*edit_controls_collapsed;
        }
    }
}

struct RaceState {
    inner: RaceStateInner,
    edit_controls_collapsed: bool,
}

enum RaceStateInner {
    Gen(RaceGen),
    Tweak,
}

enum TweakAction {
    Delete(usize),
    Reroll(usize),
    Add(usize),
    DeleteTs(usize),
    RerollTs(usize),
    RerollTsPoint(usize),
    AddTs(usize),
}

#[derive(Clone)]
struct RaceGen {
    length: usize,
    n_tryhisuojaus: usize,
    classes: Vec<String>,
}

impl Default for RaceGen {
    fn default() -> Self {
        Self {
            length: 0,
            n_tryhisuojaus: 0,
            classes: vec!["".to_string(); 10],
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Config {
    race_points: Vec<Point>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            race_points: DEFAULT_POINTS.to_vec(),
        }
    }
}

struct Race {
    // Indices of existing checkpoints
    checkpoints: Vec<(Point, String)>,
    tryhisuojaus: Vec<(usize, Point)>,
}

struct RuntimeData {
    state: State,
    config: Config,
    font: Font,
    gtav_map: Texture2D,
    race: Option<Race>,
    config_path: String,
    clipboard: arboard::Clipboard,
}

impl RuntimeData {
    fn run(&mut self) {
        clear_background(BLACK);

        let scale_factor = screen_height() / self.gtav_map.height();

        draw_texture_ex(
            self.gtav_map,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: self.gtav_map.width() * scale_factor,
                    y: self.gtav_map.height() * scale_factor,
                }),
                ..Default::default()
            },
        );

        let mut window_size =
            egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(0.0, 0.0));

        egui_macroquad::ui(|ctx| {
            egui::Window::new("GTAV Race Gen 2: Electric Boogaloo")
                .title_bar(false)
                .anchor(egui::Align2::RIGHT_TOP, (0.0, 0.0))
                .resizable(false)
                .show(ctx, |ui| {
                    window_size = self.main_ui(ui);
                });
        });

        match &mut self.state {
            State::Idle => {}
            State::Config(race_points) => {
                let mouse_pos = mouse_position();

                let mut delete = None;

                for (i, point) in race_points.iter().enumerate() {
                    let scaled = point.mul(scale_factor);
                    let distance = scaled.distance_to(mouse_pos);

                    if is_mouse_button_pressed(MouseButton::Right) && distance < 10.0 {
                        delete = Some(i);
                    }

                    draw_circle_lines(
                        scaled.x,
                        scaled.y,
                        10.0,
                        3.0,
                        if distance < 10.0 { WHITE } else { RED },
                    );
                }

                if let Some(i) = delete {
                    race_points.remove(i);
                }

                if is_mouse_button_pressed(MouseButton::Left)
                    && !(mouse_pos.0 > screen_width() - window_size.width()
                        && mouse_pos.1 < window_size.height())
                    && mouse_pos.0 < self.gtav_map.width() * scale_factor
                {
                    race_points.push(Point::new(mouse_pos.0, mouse_pos.1).div(scale_factor))
                }
            }
            State::Race(_) => {
                if let Some(race) = &self.race {
                    self.draw_race(race, scale_factor);
                }
            }
        }

        egui_macroquad::draw();
    }

    fn draw_race(&self, race: &Race, scale_factor: f32) {
        let mut last_point: Option<Point> = None;

        for (i, (point, class)) in race.checkpoints.iter().enumerate() {
            let scaled = point.mul(scale_factor);

            if let Some(last_point) = last_point {
                draw_line(
                    scaled.x,
                    scaled.y,
                    last_point.x,
                    last_point.y,
                    LINE_THICKNESS * scale_factor,
                    Color::from_rgba(255, 0, 0, 100),
                );
            }

            draw_circle_lines(
                scaled.x,
                scaled.y,
                CIRCLE_RADIUS * scale_factor,
                CIRCLE_THICKNESS * scale_factor,
                RED,
            );

            draw_text_ex(
                &format!("{}", i + 1),
                scaled.x + 20.0,
                scaled.y,
                TextParams {
                    font: self.font,
                    font_size: (TEXT_SIZE * scale_factor) as u16,
                    color: RED,
                    ..Default::default()
                },
            );
            draw_text_ex(
                class,
                scaled.x + 20.0,
                scaled.y + 30.0,
                TextParams {
                    font: self.font,
                    font_size: (TEXT_SIZE * scale_factor) as u16,
                    color: RED,
                    ..Default::default()
                },
            );

            last_point = Some(scaled);
        }

        for (i, (index, point)) in race.tryhisuojaus.iter().enumerate() {
            let scaled = point.mul(scale_factor);

            let last_index = if i == 0 {
                None
            } else {
                race.tryhisuojaus.get(i - 1)
            };
            let next_index = race.tryhisuojaus.get(i + 1);

            let start = if let Some((last_index, _)) = last_index {
                if last_index == index {
                    None
                } else {
                    Some(race.checkpoints[*index].0.mul(scale_factor))
                }
            } else {
                Some(race.checkpoints[*index].0.mul(scale_factor))
            };

            let end = if let Some((next_index, point)) = next_index {
                if next_index == index {
                    point.mul(scale_factor)
                } else {
                    race.checkpoints[*index + 1].0.mul(scale_factor)
                }
            } else {
                race.checkpoints[*index + 1].0.mul(scale_factor)
            };

            draw_circle_lines(
                scaled.x,
                scaled.y,
                CIRCLE_RADIUS * scale_factor,
                CIRCLE_THICKNESS * scale_factor,
                Color::from_rgba(0, 0, 255, 255),
            );

            if let Some(start) = start {
                draw_line(
                    scaled.x,
                    scaled.y,
                    start.x,
                    start.y,
                    LINE_THICKNESS * scale_factor,
                    Color::from_rgba(0, 0, 255, 100),
                );
            }
            draw_line(
                scaled.x,
                scaled.y,
                end.x,
                end.y,
                LINE_THICKNESS * scale_factor,
                Color::from_rgba(0, 0, 255, 100),
            );

            draw_text_ex(
                &format!("{}", i + 1),
                scaled.x + 20.0,
                scaled.y + TEXT_SIZE * scale_factor / 2.0,
                TextParams {
                    font: self.font,
                    font_size: (TEXT_SIZE * scale_factor) as u16,
                    color: Color::from_rgba(0, 0, 255, 255),
                    ..Default::default()
                },
            );
        }
    }

    fn main_ui(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        ui.heading("GTAV Race Gen 2: Electric Boogaloo");
        ui.separator();

        match &mut self.state {
            State::Race(RaceState {
                inner: RaceStateInner::Gen(race_gen),
                edit_controls_collapsed,
            }) => {
                if !*edit_controls_collapsed {
                    ui.add(
                        egui::Slider::new(&mut race_gen.length, 0..=10)
                            .show_value(true)
                            .text("Race length"),
                    );
                    ui.add(
                        egui::Slider::new(&mut race_gen.n_tryhisuojaus, 0..=4)
                            .show_value(true)
                            .text("Tryhisuojaus checkpoints"),
                    );

                    ui.separator();

                    ui.label("Classes");

                    for i in 0..race_gen.length {
                        ui.text_edit_singleline(&mut race_gen.classes[i]);
                    }

                    ui.separator();
                }

                if race_gen.length > 1 {
                    let race_gen = race_gen.clone();
                    ui.horizontal(|ui| {
                        if ui.button("Generate race").clicked() {
                            self.race = Some(self.generate_race(race_gen))
                        }
                        if self.race.is_some() && ui.button("Tweak the race").clicked() {
                            self.state = State::Race(RaceState {
                                inner: RaceStateInner::Tweak,
                                edit_controls_collapsed: false,
                            });
                        }
                        if ui.button("Toggle edit controls").clicked() {
                            self.state.toggle_edit_controls();
                        }
                    });
                }
            }
            State::Race(RaceState {
                inner: RaceStateInner::Tweak,
                edit_controls_collapsed,
            }) => {
                let mut tweak_action = None;
                if !*edit_controls_collapsed {
                    ui.label("Checkpoints");

                    for (i, (_, class)) in self
                        .race
                        .as_mut()
                        .unwrap()
                        .checkpoints
                        .iter_mut()
                        .enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}: ", i + 1));
                            ui.text_edit_singleline(class);
                            if ui.button("Reroll").clicked() {
                                tweak_action = Some(TweakAction::Reroll(i));
                            }
                            if ui.button("Delete").clicked() {
                                tweak_action = Some(TweakAction::Delete(i));
                            }
                            if ui.button("Add new point").clicked() {
                                tweak_action = Some(TweakAction::Add(i));
                            }
                            if ui.button("Add tryhisuojaus").clicked() {
                                tweak_action = Some(TweakAction::AddTs(i));
                            }
                        });
                    }

                    ui.separator();
                    ui.label("Tryhisuojaus checkpoints");

                    for (i, (index, _)) in
                        self.race.as_ref().unwrap().tryhisuojaus.iter().enumerate()
                    {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{}: Checkpoints {}-{}",
                                i + 1,
                                index + 1,
                                index + 2,
                            ));
                            if ui.button("Reroll point").clicked() {
                                tweak_action = Some(TweakAction::RerollTsPoint(i));
                            }
                            if ui.button("Reroll everything").clicked() {
                                tweak_action = Some(TweakAction::RerollTs(i));
                            }
                            if ui.button("Delete").clicked() {
                                tweak_action = Some(TweakAction::DeleteTs(i));
                            }
                        });
                    }
                    ui.separator();
                }

                ui.horizontal(|ui| {
                    if ui.button("Export map").clicked() {
                        let image = self.race_image(self.race.as_ref().unwrap());

                        let path = rfd::FileDialog::new().save_file().unwrap();

                        image.export_png(&path.as_path().to_string_lossy());
                    }
                    if ui.button("Copy map to clipboard").clicked() {
                        let image = self.race_image(self.race.as_ref().unwrap());

                        self.clipboard
                            .set_image(arboard::ImageData {
                                width: image.width(),
                                height: image.height(),
                                bytes: Cow::from_iter(
                                    image.get_image_data().iter().flatten().cloned(),
                                ),
                            })
                            .unwrap();
                    }
                    if ui.button("Back").clicked() {
                        self.state = State::Race(RaceState {
                            inner: RaceStateInner::Gen(RaceGen::default()),
                            edit_controls_collapsed: false,
                        });
                    }
                    if ui.button("Toggle edit controls").clicked() {
                        self.state.toggle_edit_controls();
                    }
                });

                match tweak_action {
                    Some(TweakAction::Delete(i)) => {
                        self.race.as_mut().unwrap().checkpoints.remove(i);
                    }
                    Some(TweakAction::Reroll(i)) => {
                        self.race
                            .as_mut()
                            .unwrap()
                            .checkpoints
                            .get_mut(i)
                            .unwrap()
                            .0 = self.random_point(self.race.as_ref().unwrap());
                    }
                    Some(TweakAction::Add(i)) => {
                        let point = self.random_point(self.race.as_ref().unwrap());
                        self.race
                            .as_mut()
                            .unwrap()
                            .checkpoints
                            .insert(i + 1, (point, String::new()));
                    }
                    Some(TweakAction::DeleteTs(i)) => {
                        self.race.as_mut().unwrap().tryhisuojaus.remove(i);
                    }
                    Some(TweakAction::RerollTsPoint(i)) => {
                        self.race
                            .as_mut()
                            .unwrap()
                            .tryhisuojaus
                            .get_mut(i)
                            .unwrap()
                            .1 = self.random_point(self.race.as_ref().unwrap());
                    }
                    Some(TweakAction::RerollTs(i)) => {
                        let len = self.race.as_ref().unwrap().checkpoints.len();
                        *self.race.as_mut().unwrap().tryhisuojaus.get_mut(i).unwrap() = (
                            rand::gen_range(0, len - 1),
                            self.random_point(self.race.as_ref().unwrap()),
                        );
                    }
                    Some(TweakAction::AddTs(index)) => {
                        let point = self.random_point(self.race.as_ref().unwrap());
                        self.race
                            .as_mut()
                            .unwrap()
                            .tryhisuojaus
                            .push((index, point));
                        self.race
                            .as_mut()
                            .unwrap()
                            .tryhisuojaus
                            .sort_by(|(a, _), (b, _)| a.cmp(b));
                    }
                    None => (),
                }
            }
            State::Config(race_points) => {
                let race_points = race_points.clone();
                ui.label("Create new checkpoints by left clicking on a location on the map and delete existing ones by left clicking on them.");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.config.race_points = race_points;
                        fs::write(
                            &self.config_path,
                            ron::ser::to_string(&self.config).unwrap(),
                        )
                        .unwrap();
                    }
                    if ui.button("Back").clicked() {
                        self.state = State::Idle;
                    }
                });
            }
            State::Idle => {
                ui.horizontal(|ui| {
                    if ui.button("Create a new race").clicked() {
                        self.state = State::Race(RaceState {
                            inner: RaceStateInner::Gen(RaceGen::default()),
                            edit_controls_collapsed: false,
                        });
                    }
                    if ui.button("Configure checkpoints").clicked() {
                        self.state = State::Config(self.config.race_points.clone());
                    }
                });
            }
        }

        ui.min_rect()
    }

    fn race_image(&self, race: &Race) -> Image {
        let render_target =
            render_target(self.gtav_map.width() as u32, self.gtav_map.height() as u32);

        set_camera(&Camera2D {
            target: Vec2::new(self.gtav_map.width() / 2.0, self.gtav_map.height() / 2.0),
            zoom: Vec2::new(2.0 / self.gtav_map.width(), 2.0 / self.gtav_map.height()),
            render_target: Some(render_target),
            ..Default::default()
        });

        draw_texture(self.gtav_map, 0.0, 0.0, WHITE);

        self.draw_race(race, 1.0);

        set_default_camera();

        let mut image = render_target.texture.get_texture_data();

        let flipped = image
            .get_image_data()
            .chunks(image.width())
            .rev()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        image.get_image_data_mut().copy_from_slice(&flipped);
        image
    }

    fn generate_race(&self, mut race_gen: RaceGen) -> Race {
        let mut race_points = self.config.race_points.clone();
        race_points.shuffle();

        let points = &race_points[0..race_gen.length];
        let tryhisuojaus =
            &race_points[race_gen.length..(race_gen.length + race_gen.n_tryhisuojaus)];

        race_gen.classes.truncate(race_gen.length);
        race_gen.classes.shuffle();

        let mut tryhisuojaus = tryhisuojaus
            .iter()
            .map(|point| (rand::gen_range(0, race_gen.length - 2), *point))
            .collect::<Vec<_>>();
        tryhisuojaus.sort_by(|(a, _), (b, _)| a.cmp(b));

        Race {
            checkpoints: points.iter().copied().zip(race_gen.classes).collect(),
            tryhisuojaus,
        }
    }

    fn random_point(&self, race: &Race) -> Point {
        **self
            .config
            .race_points
            .iter()
            .filter(|point| {
                !(race
                    .checkpoints
                    .iter()
                    .any(|(checkpoint, _)| *checkpoint == **point)
                    || race
                        .tryhisuojaus
                        .iter()
                        .any(|(_, checkpoint)| *checkpoint == **point))
            })
            .collect::<Vec<_>>()
            .choose()
            .unwrap()
    }
}

#[macroquad::main("GTAV Race Gen 2: Electric Boogaloo")]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../res/Hack-Regular.ttf")).unwrap();
    let gtav_map = load_texture("assets/gtav-map2.png").await.unwrap();
    let state = State::Idle;
    let race: Option<Race> = None;
    let clipboard = arboard::Clipboard::new().unwrap();

    let config_path = format!("{}/.config/gtav-map-gen.ron", env::var("HOME").unwrap());
    let config: Config = match fs::read(&config_path) {
        Ok(data) => ron::de::from_bytes(&data).unwrap_or_default(),
        Err(_) => Default::default(),
    };

    let mut runtime_data = RuntimeData {
        state,
        config,
        font,
        gtav_map,
        race,
        config_path,
        clipboard,
    };

    loop {
        runtime_data.run();

        next_frame().await;
    }
}
