use std::{
    borrow::Cow,
    env, fs,
    ops::{Div, Mul},
};

use egui_macroquad::egui;
use macroquad::{prelude::*, rand::ChooseRandom};
use serde::{Deserialize, Serialize};

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

// FIXME: This is dumb
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

enum State {
    Race(RaceState),
    Config,
    Idle,
}

enum RaceState {
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

fn generate_race(mut race_gen: RaceGen, mut race_points: Vec<Point>) -> Race {
    race_points.shuffle();

    let points = &race_points[0..race_gen.length];
    let tryhisuojaus = &race_points[race_gen.length..(race_gen.length + race_gen.n_tryhisuojaus)];

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

fn random_point(race: &Race, race_points: &[Point]) -> Point {
    **race_points
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

const CIRCLE_RADIUS: f32 = 20.0;
const CIRCLE_THICKNESS: f32 = 5.0;
const LINE_THICKNESS: f32 = 7.0;
const TEXT_SIZE: f32 = 50.0;

fn draw_race(race: &Race, font: Font, scale_factor: f32) {
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
                font,
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
                font,
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
                font,
                font_size: (TEXT_SIZE * scale_factor) as u16,
                color: Color::from_rgba(0, 0, 255, 255),
                ..Default::default()
            },
        );
    }
}

fn race_image(gtav_map: Texture2D, font: Font, race: &Race) -> Image {
    let render_target = render_target(gtav_map.width() as u32, gtav_map.height() as u32);

    set_camera(&Camera2D {
        target: Vec2::new(gtav_map.width() / 2.0, gtav_map.height() / 2.0),
        zoom: Vec2::new(2.0 / gtav_map.width(), 2.0 / gtav_map.height()),
        render_target: Some(render_target),
        ..Default::default()
    });

    draw_texture(gtav_map, 0.0, 0.0, WHITE);

    draw_race(race, font, 1.0);

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

#[macroquad::main("GTAV Race Gen 2: Electric Boogaloo")]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../res/Hack-Regular.ttf")).unwrap();
    let gtav_map = load_texture("assets/gtav-map2.png").await.unwrap();
    let mut state = State::Idle;
    let mut race: Option<Race> = None;
    let mut clipboard = arboard::Clipboard::new().unwrap();

    let config_path = format!("{}/.config/gtav-map-gen.ron", env::var("HOME").unwrap());
    let mut config: Config = match fs::read(&config_path) {
        Ok(data) => ron::de::from_bytes(&data).unwrap_or_default(),
        Err(_) => Default::default(),
    };

    loop {
        clear_background(BLACK);

        let scale_factor = screen_height() / gtav_map.height();

        draw_texture_ex(
            gtav_map,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: gtav_map.width() * scale_factor,
                    y: gtav_map.height() * scale_factor,
                }),
                ..Default::default()
            },
        );

        match &state {
            State::Idle => {}
            State::Config => {
                let mouse_pos = mouse_position();

                let mut delete = None;

                for (i, point) in config.race_points.iter().enumerate() {
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
                    config.race_points.remove(i);
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    config
                        .race_points
                        .push(Point::new(mouse_pos.0, mouse_pos.1).div(scale_factor))
                }
            }
            State::Race(_) => {
                if let Some(race) = &race {
                    draw_race(race, font, scale_factor);
                }
            }
        }

        egui_macroquad::ui(|ctx| {
            egui::Window::new("GTAV Race Gen 2: Electric Boogaloo")
                .anchor(egui::Align2::RIGHT_TOP, (0.0, 0.0))
                .resizable(false)
                .show(ctx, |ui| match &mut state {
                    State::Race(RaceState::Gen(race_gen)) => {
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

                        if race_gen.length > 1 {
                            let race_gen = race_gen.clone();
                            ui.horizontal(|ui| {
                                if ui.button("Generate race").clicked() {
                                    race = Some(generate_race(race_gen, config.race_points.clone()))
                                }
                                if race.is_some() && ui.button("Tweak the race").clicked() {
                                    state = State::Race(RaceState::Tweak);
                                }
                            });
                        }
                    }
                    State::Race(RaceState::Tweak) => {
                        ui.label("Checkpoints");

                        let mut tweak_action = None;

                        for (i, (_, class)) in
                            race.as_mut().unwrap().checkpoints.iter_mut().enumerate()
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
                            race.as_ref().unwrap().tryhisuojaus.iter().enumerate()
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

                        ui.horizontal(|ui| {
                            if ui.button("Export map").clicked() {
                                let image = race_image(gtav_map, font, race.as_ref().unwrap());

                                let path = rfd::FileDialog::new().save_file().unwrap();

                                image.export_png(&path.as_path().to_string_lossy());
                            }
                            if ui.button("Copy map to clipboard").clicked() {
                                let image = race_image(gtav_map, font, race.as_ref().unwrap());

                                clipboard
                                    .set_image(arboard::ImageData {
                                        width: image.width(),
                                        height: image.height(),
                                        bytes: Cow::from_iter(
                                            image.get_image_data().iter().flatten().cloned(),
                                        ),
                                    })
                                    .unwrap();
                            }
                        });

                        match tweak_action {
                            Some(TweakAction::Delete(i)) => {
                                race.as_mut().unwrap().checkpoints.remove(i);
                            }
                            Some(TweakAction::Reroll(i)) => {
                                race.as_mut().unwrap().checkpoints.get_mut(i).unwrap().0 =
                                    random_point(race.as_ref().unwrap(), &config.race_points);
                            }
                            Some(TweakAction::Add(i)) => {
                                let point =
                                    random_point(race.as_ref().unwrap(), &config.race_points);
                                race.as_mut()
                                    .unwrap()
                                    .checkpoints
                                    .insert(i + 1, (point, String::new()));
                            }
                            Some(TweakAction::DeleteTs(i)) => {
                                race.as_mut().unwrap().tryhisuojaus.remove(i);
                            }
                            Some(TweakAction::RerollTsPoint(i)) => {
                                race.as_mut().unwrap().tryhisuojaus.get_mut(i).unwrap().1 =
                                    random_point(race.as_ref().unwrap(), &config.race_points);
                            }
                            Some(TweakAction::RerollTs(i)) => {
                                let len = race.as_ref().unwrap().checkpoints.len();
                                *race.as_mut().unwrap().tryhisuojaus.get_mut(i).unwrap() = (
                                    rand::gen_range(0, len - 1),
                                    random_point(race.as_ref().unwrap(), &config.race_points),
                                );
                            }
                            Some(TweakAction::AddTs(index)) => {
                                let point =
                                    random_point(race.as_ref().unwrap(), &config.race_points);
                                race.as_mut().unwrap().tryhisuojaus.push((index, point));
                                race.as_mut()
                                    .unwrap()
                                    .tryhisuojaus
                                    .sort_by(|(a, _), (b, _)| a.cmp(b));
                            }
                            None => (),
                        }
                    }
                    State::Config => {
                        ui.label("Create new checkpoints by left clicking on a location on the map and delete existing ones by left clicking on them.");
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                fs::write(&config_path, ron::ser::to_string(&config).unwrap()).unwrap();
                            }
                            if ui.button("Back").clicked() {
                                state = State::Idle;
                            }
                        });
                    }
                    State::Idle => {
                        ui.horizontal(|ui| {
                            if ui.button("Create a new race").clicked() {
                                state = State::Race(RaceState::Gen(RaceGen::default()));
                            }
                            if ui.button("Configure checkpoints").clicked() {
                                state = State::Config;
                            }
                        });
                    }
                });
        });

        egui_macroquad::draw();

        next_frame().await;
    }
}
